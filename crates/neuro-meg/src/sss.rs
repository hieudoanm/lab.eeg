//! Simplified signal-space separation (SSS) for MEG.
//!
//! Approximates the Taulu–Kajola internal/external multipole expansion with a
//! truncated real solid-harmonic basis evaluated at each sensor's position,
//! coupled through the sensor orientation's radial component. Internal
//! sources use regular solid harmonics `r^l Y_lm`; external/interference
//! sources use irregular harmonics `r^{-(l+1)} Y_lm`. The internal and
//! external subspaces are separated by joint least squares per sample.
//!
//! Limitations vs. full SSS: no gradient (field-pattern) coupling, no
//! origin optimisation, no fine calibration/coils — a low-order spatial
//! filter suitable for gross interference rejection, not quantitative
//! source imaging.

use crate::{MegChannel, MegMetadata};
use neuro_core::CoreError;

/// Associated Legendre function P_l^m(x) for m >= 0 (no Condon-Shortley
/// phase — sign conventions cancel inside a basis spanning the same space).
fn assoc_legendre(l: usize, m: usize, x: f64) -> f64 {
    if m > l {
        return 0.0;
    }
    let mut pmm = 1.0f64;
    if m > 0 {
        let somx2 = (1.0 - x * x).max(0.0).sqrt();
        let mut fact = 1.0;
        for _ in 0..m {
            pmm *= -fact * somx2;
            fact += 2.0;
        }
    }
    if l == m {
        return pmm;
    }
    let mut pmmp1 = x * (2 * m + 1) as f64 * pmm;
    if l == m + 1 {
        return pmmp1;
    }
    let mut pll = 0.0;
    for ll in m + 2..=l {
        pll = ((2 * ll - 1) as f64 * x * pmmp1 - (ll + m - 1) as f64 * pmm) / (ll - m) as f64;
        pmm = pmmp1;
        pmmp1 = pll;
    }
    pll
}

/// Real spherical-harmonic angular factor for degree `l`, order `m` at
/// polar angle `theta` (from +z) and azimuth `phi`. `m == 0` returns the
/// zonal term; positive/negative `m` give cos/sin terms. Values are
/// unnormalised — subspace spans are unaffected by per-term scaling.
fn real_harmonic(l: usize, m: isize, theta: f64, phi: f64) -> f64 {
    let am = m.unsigned_abs() as usize;
    if am > l {
        return 0.0;
    }
    let p = assoc_legendre(l, am, theta.cos());
    match m.cmp(&0) {
        std::cmp::Ordering::Equal => p,
        std::cmp::Ordering::Greater => p * (am as f64 * phi).cos(),
        std::cmp::Ordering::Less => p * (am as f64 * phi).sin(),
    }
}

/// Solid harmonic value at offset `d` (metres) for degree `l`, order `m`.
/// `internal` selects the regular (`r^l`) or irregular (`r^{-(l+1)}`) form.
fn solid_harmonic(d: [f64; 3], l: usize, m: isize, internal: bool) -> f64 {
    let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if r < 1e-9 {
        return 0.0;
    }
    let theta = d[2].clamp(-1.0, 1.0).acos();
    let phi = d[1].atan2(d[0]);
    let angular = real_harmonic(l, m, theta, phi);
    let radial = if internal {
        r.powi(l as i32)
    } else {
        r.powf(-(l as f64) - 1.0)
    };
    angular * radial
}

/// Basis row for one sensor: internal harmonics l=1..=l_in then external
/// l=1..=l_out, each weighted by the orientation·radial coupling.
fn sensor_basis(offset: [f64; 3], orientation: [f64; 3], l_in: usize, l_out: usize) -> Vec<f64> {
    let r = (offset[0] * offset[0] + offset[1] * offset[1] + offset[2] * offset[2]).sqrt();
    if r < 1e-9 {
        return Vec::new();
    }
    let radial = [offset[0] / r, offset[1] / r, offset[2] / r];
    let coupling =
        orientation[0] * radial[0] + orientation[1] * radial[1] + orientation[2] * radial[2];
    let mut row = Vec::with_capacity(l_in * (l_in + 2) + l_out * (l_out + 2));
    for l in 1..=l_in {
        for m in -(l as isize)..=(l as isize) {
            row.push(coupling * solid_harmonic(offset, l, m, true));
        }
    }
    for l in 1..=l_out {
        for m in -(l as isize)..=(l as isize) {
            row.push(coupling * solid_harmonic(offset, l, m, false));
        }
    }
    row
}

/// Solve A x = b for square A via Gauss-Jordan with partial pivoting.
fn solve(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Result<Vec<f64>, CoreError> {
    let n = a.len();
    for col in 0..n {
        let mut pivot = col;
        for row in col + 1..n {
            if a[row][col].abs() > a[pivot][col].abs() {
                pivot = row;
            }
        }
        if a[pivot][col].abs() < 1e-12 {
            return Err(CoreError::InvalidInput("SSS basis is singular".into()));
        }
        a.swap(col, pivot);
        b.swap(col, pivot);
        let d = a[col][col];
        for j in col..n {
            a[col][j] /= d;
        }
        b[col] /= d;
        for row in 0..n {
            if row == col {
                continue;
            }
            let f = a[row][col];
            if f == 0.0 {
                continue;
            }
            for j in col..n {
                a[row][j] -= f * a[col][j];
            }
            b[row] -= f * b[col];
        }
    }
    Ok(b)
}

/// SSS-filtered copy of `data` (channels × samples). Channels without coil
/// position/orientation metadata pass through unchanged. `l_in`/`l_out` are
/// the internal/external expansion orders.
pub fn sss_filter(
    data: &[Vec<f64>],
    channel_names: &[String],
    channels: &[MegChannel],
    metadata: &MegMetadata,
    l_in: usize,
    l_out: usize,
) -> Result<(Vec<Vec<f64>>, usize), CoreError> {
    let l_in = l_in.clamp(1, 6);
    let l_out = l_out.clamp(1, 4);
    // Sensors: MEG-kind channels with both position and orientation.
    let by_name: std::collections::BTreeMap<&str, &MegChannel> =
        channels.iter().map(|c| (c.name.as_str(), c)).collect();
    let mut sensor_rows: Vec<usize> = Vec::new();
    let mut basis: Vec<Vec<f64>> = Vec::new();
    // Sphere origin = centroid of positioned MEG sensors.
    let mut origin = [0.0f64; 3];
    let mut origin_count = 0usize;
    for (row, name) in channel_names.iter().enumerate() {
        let info = by_name.get(name.as_str());
        let is_meg = info.is_some_and(|c| {
            c.kind.eq_ignore_ascii_case("meg") || c.kind.eq_ignore_ascii_case("mag")
        });
        let pos = info.and_then(|c| c.position_mm.as_ref());
        let ori = info.and_then(|c| c.orientation.as_ref());
        let has_geom = pos.is_some_and(|p| p.len() >= 3) && ori.is_some_and(|o| o.len() >= 3);
        if is_meg && has_geom {
            for d in 0..3 {
                origin[d] += pos.unwrap()[d] / 1000.0;
            }
            origin_count += 1;
            sensor_rows.push(row);
        }
    }
    if sensor_rows.len() < 8 {
        return Err(CoreError::InvalidInput(
            "SSS requires at least 8 positioned MEG sensors".into(),
        ));
    }
    for d in 0..3 {
        origin[d] /= origin_count as f64;
    }
    let n_internal = l_in * (l_in + 2);
    let mut geom: Vec<(usize, [f64; 3], [f64; 3])> = Vec::new();
    for &row in &sensor_rows {
        let name = &channel_names[row];
        let info = by_name[name.as_str()];
        let pos = info.position_mm.as_ref().unwrap();
        let ori = info.orientation.as_ref().unwrap();
        let offset = [
            pos[0] / 1000.0 - origin[0],
            pos[1] / 1000.0 - origin[1],
            pos[2] / 1000.0 - origin[2],
        ];
        let orientation = [ori[0], ori[1], ori[2]];
        geom.push((row, offset, orientation));
    }
    for (_, offset, orientation) in &geom {
        basis.push(sensor_basis(*offset, *orientation, l_in, l_out));
    }
    // Column-normalise the basis: internal and external harmonics differ by
    // orders of magnitude in magnitude, which would swamp the ridge term and
    // distort the internal subspace. Scaling columns preserves both subspaces.
    let n_basis_cols = basis[0].len();
    for j in 0..n_basis_cols {
        let mut ss = 0.0;
        for row in &basis {
            ss += row[j] * row[j];
        }
        let scale = (ss / basis.len() as f64).sqrt();
        if scale > 1e-12 {
            for row in &mut basis {
                row[j] /= scale;
            }
        }
    }
    let n_sensors = basis.len();
    let n_basis = basis[0].len();
    if n_basis >= n_sensors {
        return Err(CoreError::InvalidInput(
            "SSS basis order exceeds the sensor count".into(),
        ));
    }
    // G = S^T S (n_basis x n_basis); M_in = G^{-1}_{in} S_in^T applied as:
    // q = G^{-1} S^T x, then x_clean = S_in q_in.
    let mut gram = vec![vec![0.0f64; n_basis]; n_basis];
    for i in 0..n_basis {
        for j in 0..=i {
            let mut acc = 0.0;
            for row in &basis {
                acc += row[i] * row[j];
            }
            gram[i][j] = acc;
            gram[j][i] = acc;
        }
    }
    // Ridge-regularise for numerical stability.
    let trace: f64 = (0..n_basis).map(|i| gram[i][i]).sum();
    let ridge = trace / n_basis as f64 * 1e-6 + 1e-12;
    for i in 0..n_basis {
        gram[i][i] += ridge;
    }
    // Precompute T = S_in * G^{-1}[:, :n_internal] * S^T is n_sensors^2 — too
    // big; instead per sample compute q = G^{-1} S^T x then S_in q_in.
    // Precompute Minv = G^{-1} once (n_basis^3, bounded: <=30^3).
    let mut minv = vec![vec![0.0f64; n_basis]; n_basis];
    for col in 0..n_basis {
        let mut e = vec![0.0f64; n_basis];
        e[col] = 1.0;
        let x = solve(gram.clone(), e)?;
        for row in 0..n_basis {
            minv[row][col] = x[row];
        }
    }
    // Precompute A = Minv * S^T -> n_basis x n_sensors.
    let mut a_mat = vec![vec![0.0f64; n_sensors]; n_basis];
    for i in 0..n_basis {
        for s in 0..n_sensors {
            let mut acc = 0.0;
            for j in 0..n_basis {
                acc += minv[i][j] * basis[s][j];
            }
            a_mat[i][s] = acc;
        }
    }
    let n_samples = data.first().map_or(0, Vec::len);
    let mut out = data.to_vec();
    let mut scratch_q = vec![0.0f64; n_basis];
    let mut scratch_x = vec![0.0f64; n_sensors];
    for sample in 0..n_samples {
        for s in 0..n_sensors {
            scratch_x[s] = data[sensor_rows[s]][sample];
        }
        for i in 0..n_basis {
            let mut acc = 0.0;
            for s in 0..n_sensors {
                acc += a_mat[i][s] * scratch_x[s];
            }
            scratch_q[i] = acc;
        }
        for (s, &(row, _, _)) in geom.iter().enumerate() {
            let mut acc = 0.0;
            for j in 0..n_internal {
                acc += basis[s][j] * scratch_q[j];
            }
            // Reconstruct the internal contribution only.
            out[row][sample] = acc;
        }
    }
    let _ = metadata;
    Ok((out, n_sensors))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_channels(n: usize) -> (Vec<Vec<f64>>, Vec<String>, Vec<MegChannel>, MegMetadata) {
        // Sensors on a sphere, radially oriented.
        let mut names = Vec::new();
        let mut channels = Vec::new();
        let mut data = Vec::new();
        let n_samples = 50usize;
        for i in 0..n {
            let theta = (i as f64) * 2.399963; // golden-angle spread
            let phi = ((i * 37 % 11) as f64) * 0.57;
            let r = 0.1;
            let pos = [
                r * phi.sin() * theta.cos(),
                r * phi.sin() * theta.sin(),
                r * phi.cos(),
            ];
            let ori = pos;
            let name = format!("MEG{i:02}");
            names.push(name.clone());
            channels.push(MegChannel {
                name,
                kind: "meg".into(),
                unit: "tesla".into(),
                unit_mul: 1.0,
                group_name: None,
                status: "good".into(),
                coil_type: None,
                position_mm: Some(pos.iter().map(|v| v * 1000.0).collect()),
                orientation: Some(ori.to_vec()),
            });
            // Deterministic per-sensor signal.
            data.push(
                (0..n_samples)
                    .map(|t| ((i * 13 + t * 7) % 17) as f64 * 0.01 - 0.08)
                    .collect(),
            );
        }
        let metadata = MegMetadata {
            format: "test".into(),
            name: "t".into(),
            acquisition_system: "t".into(),
            sampling_frequency_hz: 100.0,
            channel_count: n,
            duration_seconds: 0.5,
            coordinate_frame: "head".into(),
            channels: channels.clone(),
            events: vec![],
            bad_channel_names: vec![],
            sensor_type: String::new(),
            time_varying: false,
            device_serial: None,
            manufacturer: None,
            digitization_points: 0,
            recordings: vec![],
            recording_channels: Default::default(),
            compensation: vec![],
            surface_points: vec![],
        };
        (data, names, channels, metadata)
    }

    #[test]
    fn sss_preserves_internal_and_rejects_external_subspace() {
        let (data, names, channels, metadata) = fake_channels(16);
        let l_in = 2usize;
        let l_out = 1usize;
        let n_internal = l_in * (l_in + 2);
        // Build basis rows the same way sss_filter does, then synthesise
        // signals lying purely in the internal or external subspace.
        let mut origin = [0.0f64; 3];
        for c in &channels {
            let p = c.position_mm.as_ref().unwrap();
            for d in 0..3 {
                origin[d] += p[d] / 1000.0;
            }
        }
        for d in 0..3 {
            origin[d] /= channels.len() as f64;
        }
        let mut basis = Vec::new();
        for c in &channels {
            let p = c.position_mm.as_ref().unwrap();
            let o = c.orientation.as_ref().unwrap();
            basis.push(sensor_basis(
                [
                    p[0] / 1000.0 - origin[0],
                    p[1] / 1000.0 - origin[1],
                    p[2] / 1000.0 - origin[2],
                ],
                [o[0], o[1], o[2]],
                l_in,
                l_out,
            ));
        }
        let n_basis = basis[0].len();
        let n_samples = data[0].len();
        // q_in[t], q_out[t] deterministic.
        let synth = |use_internal: bool| -> Vec<Vec<f64>> {
            let mut out = vec![vec![0.0f64; n_samples]; channels.len()];
            for s in 0..channels.len() {
                for t in 0..n_samples {
                    let mut acc = 0.0;
                    let range = if use_internal {
                        0..n_internal
                    } else {
                        n_internal..n_basis
                    };
                    for j in range {
                        let q = ((j * 31 + t * 17) % 13) as f64 * 0.001 - 0.006;
                        acc += basis[s][j] * q;
                    }
                    out[s][t] = acc;
                }
            }
            out
        };
        let internal_data = synth(true);
        let (filtered_in, _) =
            sss_filter(&internal_data, &names, &channels, &metadata, l_in, l_out).unwrap();
        // Internal signal should be largely preserved.
        let mut num = 0.0;
        let mut den = 0.0;
        for s in 0..channels.len() {
            for t in 0..n_samples {
                num += (filtered_in[s][t] - internal_data[s][t]).abs();
                den += internal_data[s][t].abs();
            }
        }
        assert!(
            num / den.max(1e-12) < 0.2,
            "internal signal distorted: {}",
            num / den.max(1e-12)
        );
        let external_data = synth(false);
        let (filtered_out, _) =
            sss_filter(&external_data, &names, &channels, &metadata, l_in, l_out).unwrap();
        let mut ext_num = 0.0;
        let mut ext_den = 0.0;
        for s in 0..channels.len() {
            for t in 0..n_samples {
                ext_num += filtered_out[s][t].abs();
                ext_den += external_data[s][t].abs();
            }
        }
        assert!(
            ext_num / ext_den.max(1e-12) < 0.3,
            "external signal not rejected: {}",
            ext_num / ext_den.max(1e-12)
        );
    }
}
