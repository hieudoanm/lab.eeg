use crate::{inspect_nifti, read_volume, VolumeInfo};
use ndarray::{Array3, Axis, Ix3};
use neuro_core::CoreError;
use nifti::NiftiObject;
use serde_json::{json, Value};
use std::path::Path;

fn write_volume(
    volume: &Array3<f32>,
    source: &Path,
    output: &Path,
    spacing_override: Option<[f64; 3]>,
) -> Result<VolumeInfo, CoreError> {
    let object = nifti::ReaderOptions::new()
        .read_file(source)
        .map_err(|error| CoreError::InvalidInput(format!("invalid NIfTI volume: {error}")))?;
    let mut header = object.header().clone();
    if let Some(spacing) = spacing_override {
        header.pixdim[1] = spacing[0] as f32;
        header.pixdim[2] = spacing[1] as f32;
        header.pixdim[3] = spacing[2] as f32;
        header.dim[1] = volume.shape()[0] as u16;
        header.dim[2] = volume.shape()[1] as u16;
        header.dim[3] = volume.shape()[2] as u16;
    }
    nifti::writer::WriterOptions::new(output)
        .reference_header(&header)
        .write_nifti(volume)
        .map_err(|error| {
            CoreError::InvalidInput(format!("could not write NIfTI volume: {error}"))
        })?;
    inspect_nifti(output)
}

/// Threshold: voxels outside [low, high] become NaN (label-preserving).
pub fn threshold(
    input: &Path,
    output: &Path,
    low: Option<f64>,
    high: Option<f64>,
) -> Result<Value, CoreError> {
    let (volume, info) = read_volume(input)?;
    let mut changed = 0usize;
    let result = volume.mapv(|value| {
        let v = value as f64;
        let keep = v.is_finite()
            && low.map(|l| v >= l).unwrap_or(true)
            && high.map(|h| v <= h).unwrap_or(true);
        if keep {
            value
        } else {
            changed += 1;
            f32::NAN
        }
    });
    write_volume(&result, input, output, None)?;
    Ok(json!({
        "operation": "threshold",
        "voxels_masked": changed,
        "voxel_count": info.voxel_count,
    }))
}

fn gaussian_kernel_1d(sigma_voxels: f64) -> Vec<f64> {
    if sigma_voxels <= 0.0 {
        return vec![1.0];
    }
    let radius = (3.0 * sigma_voxels).ceil() as usize;
    let mut kernel = Vec::with_capacity(2 * radius + 1);
    let mut sum = 0.0;
    for i in -(radius as isize)..=(radius as isize) {
        let w = (-0.5 * (i as f64 / sigma_voxels).powi(2)).exp();
        kernel.push(w);
        sum += w;
    }
    for w in &mut kernel {
        *w /= sum;
    }
    kernel
}

fn convolve_axis(volume: &Array3<f32>, axis: Axis, kernel: &[f64]) -> Array3<f32> {
    let radius = (kernel.len() / 2) as isize;
    let mut out = Array3::<f32>::zeros(volume.raw_dim());
    let len = volume.len_of(axis);
    for pattern in ndarray::indices(volume.raw_dim()) {
        let index = Ix3(pattern.0, pattern.1, pattern.2);
        let center = index[axis.index()];
        let mut acc = 0.0f64;
        let mut weight_sum = 0.0f64;
        for (k, w) in kernel.iter().enumerate() {
            let offset = k as isize - radius;
            let sample = center as isize + offset;
            if sample < 0 || sample >= len as isize {
                continue;
            }
            let mut idx = index;
            idx[axis.index()] = sample as usize;
            let v = volume[idx] as f64;
            if v.is_finite() {
                acc += v * w;
                weight_sum += w;
            }
        }
        out[index] = if weight_sum > 0.0 {
            (acc / weight_sum) as f32
        } else {
            f32::NAN
        };
    }
    out
}

/// Gaussian smoothing with FWHM specified in millimetres (per-axis spacing).
pub fn gaussian_smooth(input: &Path, output: &Path, fwhm_mm: f64) -> Result<Value, CoreError> {
    if fwhm_mm <= 0.0 || !fwhm_mm.is_finite() {
        return Err(CoreError::InvalidInput("fwhm_mm must be positive".into()));
    }
    let (volume, info) = read_volume(input)?;
    if info.shape.iter().any(|&d| d < 3) {
        return Err(CoreError::InvalidInput(
            "volume is too small for smoothing".into(),
        ));
    }
    let sigma_mm = fwhm_mm / 2.354820045;
    let mut result = volume;
    for (axis_index, spacing) in info.spacing_mm.iter().enumerate() {
        let sigma_voxels = sigma_mm / spacing.max(1e-6);
        let kernel = gaussian_kernel_1d(sigma_voxels);
        result = convolve_axis(&result, Axis(axis_index), &kernel);
    }
    write_volume(&result, input, output, None)?;
    Ok(json!({
        "operation": "gaussian_smooth",
        "fwhm_mm": fwhm_mm,
        "voxel_count": info.voxel_count,
    }))
}

/// Bias-field correction (N4-style, bounded): estimate the smooth
/// multiplicative bias as the low-frequency component of log-intensity over
/// the valid (finite, positive) mask, then divide it out. Iterating
/// re-estimates on the corrected volume.
pub fn bias_field_correct(
    input: &Path,
    output: &Path,
    fwhm_mm: f64,
    iterations: usize,
) -> Result<Value, CoreError> {
    if fwhm_mm <= 0.0 || !fwhm_mm.is_finite() {
        return Err(CoreError::InvalidInput("fwhm_mm must be positive".into()));
    }
    let iterations = iterations.clamp(1, 5);
    let (mut volume, info) = read_volume(input)?;
    if info.shape.iter().any(|&d| d < 5) {
        return Err(CoreError::InvalidInput(
            "volume is too small for bias-field correction".into(),
        ));
    }
    let sigma_mm = fwhm_mm / 2.354820045;
    let kernels: Vec<Vec<f64>> = info
        .spacing_mm
        .iter()
        .map(|spacing| gaussian_kernel_1d(sigma_mm / spacing.max(1e-6)))
        .collect();
    for _ in 0..iterations {
        // log-intensity over valid voxels; invalid -> NaN (convolve skips).
        let mut logv = Array3::<f32>::from_elem(volume.raw_dim(), f32::NAN);
        let mut sum = 0.0f64;
        let mut count = 0usize;
        for index in ndarray::indices(volume.raw_dim()) {
            let v = volume[index] as f64;
            if v.is_finite() && v > 0.0 {
                let lv = v.ln();
                logv[index] = lv as f32;
                sum += lv;
                count += 1;
            }
        }
        if count < 8 {
            return Err(CoreError::InvalidInput(
                "too few positive voxels for bias-field correction".into(),
            ));
        }
        let mean_log = sum / count as f64;
        let mut smooth = logv;
        for (axis_index, kernel) in kernels.iter().enumerate() {
            smooth = convolve_axis(&smooth, Axis(axis_index), kernel);
        }
        // bias = smooth(logv) - mean(logv); corrected = v / exp(bias)
        let mut next = Array3::<f32>::from_elem(volume.raw_dim(), f32::NAN);
        for index in ndarray::indices(volume.raw_dim()) {
            let v = volume[index] as f64;
            let s = smooth[index] as f64;
            if v.is_finite() && v > 0.0 && s.is_finite() {
                next[index] = (v * (-(s - mean_log)).exp()) as f32;
            }
        }
        volume = next;
    }
    write_volume(&volume, input, output, None)?;
    Ok(json!({
        "operation": "bias_field_correct",
        "fwhm_mm": fwhm_mm,
        "iterations": iterations,
        "voxel_count": info.voxel_count,
    }))
}

/// Apply a binary mask: voxels where the mask is ~0 become NaN.
pub fn apply_mask(input: &Path, mask_path: &Path, output: &Path) -> Result<Value, CoreError> {
    let (volume, info) = read_volume(input)?;
    let (mask, mask_info) = read_volume(mask_path)?;
    if mask.raw_dim() != volume.raw_dim() {
        return Err(CoreError::InvalidInput(
            "mask shape does not match the source volume".into(),
        ));
    }
    let mut kept = 0usize;
    let mut out = Array3::<f32>::from_elem(volume.raw_dim(), f32::NAN);
    for index in ndarray::indices(volume.raw_dim()) {
        let m = mask[index] as f64;
        let v = volume[index] as f64;
        if m.is_finite() && m.abs() > 0.5 && v.is_finite() {
            out[index] = volume[index];
            kept += 1;
        }
    }
    write_volume(&out, input, output, None)?;
    Ok(json!({
        "operation": "apply_mask",
        "voxels_kept": kept,
        "voxel_count": info.voxel_count,
        "mask_voxels": mask_info.voxel_count,
    }))
}

/// Segment by intensity threshold into a binary mask volume plus region stats.
pub fn segment_threshold(
    input: &Path,
    mask_output: &Path,
    low: Option<f64>,
    high: Option<f64>,
) -> Result<Value, CoreError> {
    let (volume, info) = read_volume(input)?;
    let mut mask = Array3::<f32>::zeros(volume.raw_dim());
    let mut count = 0usize;
    for index in ndarray::indices(volume.raw_dim()) {
        let v = volume[index] as f64;
        let inside = v.is_finite()
            && low.map(|l| v >= l).unwrap_or(true)
            && high.map(|h| v <= h).unwrap_or(true);
        if inside {
            mask[index] = 1.0;
            count += 1;
        }
    }
    write_volume(&mask, input, mask_output, None)?;
    let voxel_mm3 = info.spacing_mm.iter().product::<f64>();
    Ok(json!({
        "operation": "segment_threshold",
        "mask_voxels": count,
        "region_volume_mm3": count as f64 * voxel_mm3,
        "voxel_volume_mm3": voxel_mm3,
        "voxel_count": info.voxel_count,
    }))
}

/// Keys cubic convolution kernel with a = -0.5 (Catmull-Rom).
fn cubic_weight(t: f64) -> f64 {
    let t = t.abs();
    const A: f64 = -0.5;
    if t < 1.0 {
        (A + 2.0) * t * t * t - (A + 3.0) * t * t + 1.0
    } else if t < 2.0 {
        A * t * t * t - 5.0 * A * t * t + 8.0 * A * t - 4.0 * A
    } else {
        0.0
    }
}

/// Trilinear resample to a target isotropic-or-anisotropic voxel spacing.
pub fn resample(
    input: &Path,
    output: &Path,
    target_spacing_mm: [f64; 3],
    method: &str,
) -> Result<Value, CoreError> {
    let method = match method {
        "trilinear" | "linear" => "trilinear",
        "nearest" | "nn" => "nearest",
        "cubic" | "spline" => "cubic",
        other => {
            return Err(CoreError::InvalidInput(format!(
                "unsupported resample method: {other}"
            )))
        }
    };
    if target_spacing_mm
        .iter()
        .any(|s| *s <= 0.0 || !s.is_finite())
    {
        return Err(CoreError::InvalidInput(
            "target spacing must be positive and finite".into(),
        ));
    }
    let (volume, info) = read_volume(input)?;
    let fov: Vec<f64> = (0..3)
        .map(|i| info.shape[i] as f64 * info.spacing_mm[i])
        .collect();
    let new_shape = [
        (fov[0] / target_spacing_mm[0]).round().max(1.0) as usize,
        (fov[1] / target_spacing_mm[1]).round().max(1.0) as usize,
        (fov[2] / target_spacing_mm[2]).round().max(1.0) as usize,
    ];
    if new_shape.iter().product::<usize>() > info.voxel_count.saturating_mul(64) {
        return Err(CoreError::InvalidInput(
            "resampling would exceed the voxel limit".into(),
        ));
    }
    let src = |d: usize| info.shape[d] as f64 - 1.0;
    let mut out = Array3::<f32>::from_elem((new_shape[0], new_shape[1], new_shape[2]), f32::NAN);
    for z in 0..new_shape[2] {
        for y in 0..new_shape[1] {
            for x in 0..new_shape[0] {
                // Map output voxel centre to source coordinates.
                let sx = (x as f64 + 0.5) * target_spacing_mm[0] / info.spacing_mm[0] - 0.5;
                let sy = (y as f64 + 0.5) * target_spacing_mm[1] / info.spacing_mm[1] - 0.5;
                let sz = (z as f64 + 0.5) * target_spacing_mm[2] / info.spacing_mm[2] - 0.5;
                if sx < -0.5
                    || sy < -0.5
                    || sz < -0.5
                    || sx > src(0) + 0.5
                    || sy > src(1) + 0.5
                    || sz > src(2) + 0.5
                {
                    continue;
                }
                if method == "nearest" {
                    let nx = sx.round().clamp(0.0, (info.shape[0] - 1) as f64) as usize;
                    let ny = sy.round().clamp(0.0, (info.shape[1] - 1) as f64) as usize;
                    let nz = sz.round().clamp(0.0, (info.shape[2] - 1) as f64) as usize;
                    let value = volume[[nx, ny, nz]] as f64;
                    if value.is_finite() {
                        out[[x, y, z]] = value as f32;
                    }
                    continue;
                }
                if method == "cubic" {
                    // Cubic convolution (Keys kernel, a = -0.5) over a
                    // 4x4x4 neighbourhood; out-of-range taps clamp to the
                    // border voxel.
                    let x_base = sx.floor() as isize;
                    let y_base = sy.floor() as isize;
                    let z_base = sz.floor() as isize;
                    let mut value = 0.0f64;
                    let mut finite = true;
                    for dz in -1..=2 {
                        for dy in -1..=2 {
                            for dx in -1..=2 {
                                let nx =
                                    (x_base + dx).clamp(0, info.shape[0] as isize - 1) as usize;
                                let ny =
                                    (y_base + dy).clamp(0, info.shape[1] as isize - 1) as usize;
                                let nz =
                                    (z_base + dz).clamp(0, info.shape[2] as isize - 1) as usize;
                                let tap = volume[[nx, ny, nz]] as f64;
                                if !tap.is_finite() {
                                    finite = false;
                                    break;
                                }
                                value += tap
                                    * cubic_weight(sx - (x_base + dx) as f64)
                                    * cubic_weight(sy - (y_base + dy) as f64)
                                    * cubic_weight(sz - (z_base + dz) as f64);
                            }
                            if !finite {
                                break;
                            }
                        }
                        if !finite {
                            break;
                        }
                    }
                    if finite {
                        out[[x, y, z]] = value as f32;
                    }
                    continue;
                }
                let x0 = sx.floor().max(0.0) as usize;
                let y0 = sy.floor().max(0.0) as usize;
                let z0 = sz.floor().max(0.0) as usize;
                let x1 = (x0 + 1).min(info.shape[0] - 1);
                let y1 = (y0 + 1).min(info.shape[1] - 1);
                let z1 = (z0 + 1).min(info.shape[2] - 1);
                let fx = (sx - x0 as f64).clamp(0.0, 1.0);
                let fy = (sy - y0 as f64).clamp(0.0, 1.0);
                let fz = (sz - z0 as f64).clamp(0.0, 1.0);
                let c000 = volume[[x0, y0, z0]] as f64;
                let c001 = volume[[x0, y0, z1]] as f64;
                let c010 = volume[[x0, y1, z0]] as f64;
                let c011 = volume[[x0, y1, z1]] as f64;
                let c100 = volume[[x1, y0, z0]] as f64;
                let c101 = volume[[x1, y0, z1]] as f64;
                let c110 = volume[[x1, y1, z0]] as f64;
                let c111 = volume[[x1, y1, z1]] as f64;
                let lerp = |a: f64, b: f64, t: f64| a + (b - a) * t;
                let c00 = lerp(c000, c100, fx);
                let c01 = lerp(c001, c101, fx);
                let c10 = lerp(c010, c110, fx);
                let c11 = lerp(c011, c111, fx);
                let c0 = lerp(c00, c10, fy);
                let c1 = lerp(c01, c11, fy);
                let value = lerp(c0, c1, fz);
                if value.is_finite() {
                    out[[x, y, z]] = value as f32;
                }
            }
        }
    }
    let written = write_volume(&out, input, output, Some(target_spacing_mm))?;
    Ok(json!({
        "operation": "resample",
        "method": method,
        "input_shape": info.shape,
        "output_shape": new_shape,
        "target_spacing_mm": target_spacing_mm,
        "output_spacing_mm": written.spacing_mm,
    }))
}

// ---------- Rigid registration ----------

/// Invert a 3x3 matrix (Gauss-Jordan with partial pivoting).
fn invert3(m: &[[f64; 3]; 3]) -> Option<[[f64; 3]; 3]> {
    let mut a = [[0.0f64; 6]; 3];
    for i in 0..3 {
        for j in 0..3 {
            a[i][j] = m[i][j];
        }
        a[i][3 + i] = 1.0;
    }
    for col in 0..3 {
        let mut pivot = col;
        for row in col + 1..3 {
            if a[row][col].abs() > a[pivot][col].abs() {
                pivot = row;
            }
        }
        if a[pivot][col].abs() < 1e-12 {
            return None;
        }
        a.swap(col, pivot);
        let d = a[col][col];
        for j in 0..6 {
            a[col][j] /= d;
        }
        for row in 0..3 {
            if row == col {
                continue;
            }
            let f = a[row][col];
            for j in 0..6 {
                a[row][j] -= f * a[col][j];
            }
        }
    }
    let mut inv = [[0.0f64; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            inv[i][j] = a[i][3 + j];
        }
    }
    Some(inv)
}

/// Trilinear sample of `volume` at fractional voxel coords; clamps to the
/// border. Returns None outside the volume entirely.
fn sample_trilinear(volume: &Array3<f32>, shape: [usize; 3], p: [f64; 3]) -> Option<f64> {
    if p.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let max = [
        shape[0] as f64 - 1.0,
        shape[1] as f64 - 1.0,
        shape[2] as f64 - 1.0,
    ];
    if p[0] < -0.5
        || p[1] < -0.5
        || p[2] < -0.5
        || p[0] > max[0] + 0.5
        || p[1] > max[1] + 0.5
        || p[2] > max[2] + 0.5
    {
        return None;
    }
    let c = [
        p[0].clamp(0.0, max[0]),
        p[1].clamp(0.0, max[1]),
        p[2].clamp(0.0, max[2]),
    ];
    let b = [
        c[0].floor() as usize,
        c[1].floor() as usize,
        c[2].floor() as usize,
    ];
    let f = [c[0] - b[0] as f64, c[1] - b[1] as f64, c[2] - b[2] as f64];
    let mut value = 0.0f64;
    let mut weight_sum = 0.0f64;
    for dz in 0..=1usize {
        for dy in 0..=1usize {
            for dx in 0..=1usize {
                let idx = [
                    (b[0] + dx).min(shape[0] - 1),
                    (b[1] + dy).min(shape[1] - 1),
                    (b[2] + dz).min(shape[2] - 1),
                ];
                let v = volume[[idx[0], idx[1], idx[2]]] as f64;
                if !v.is_finite() {
                    continue;
                }
                let w = (if dx == 1 { f[0] } else { 1.0 - f[0] })
                    * (if dy == 1 { f[1] } else { 1.0 - f[1] })
                    * (if dz == 1 { f[2] } else { 1.0 - f[2] });
                value += v * w;
                weight_sum += w;
            }
        }
    }
    if weight_sum > 0.0 {
        Some(value / weight_sum)
    } else {
        None
    }
}

/// Mutual information between paired samples via a 64x64 joint histogram.
/// Intensity ranges are precomputed from the full volumes.
fn mutual_information(pairs: &[(f64, f64)], fmin: f64, fmax: f64, mmin: f64, mmax: f64) -> f64 {
    const BINS: usize = 64;
    let mut joint = vec![0u64; BINS * BINS];
    let frange = (fmax - fmin).max(1e-12);
    let mrange = (mmax - mmin).max(1e-12);
    let mut mf = vec![0u64; BINS];
    let mut mm = vec![0u64; BINS];
    for &(f, m) in pairs {
        let fb = (((f - fmin) / frange) * (BINS - 1) as f64)
            .round()
            .clamp(0.0, (BINS - 1) as f64) as usize;
        let mb = (((m - mmin) / mrange) * (BINS - 1) as f64)
            .round()
            .clamp(0.0, (BINS - 1) as f64) as usize;
        joint[fb * BINS + mb] += 1;
        mf[fb] += 1;
        mm[mb] += 1;
    }
    let n = pairs.len() as f64;
    let mut mi = 0.0;
    for fb in 0..BINS {
        for mb in 0..BINS {
            let c = joint[fb * BINS + mb] as f64;
            if c == 0.0 {
                continue;
            }
            let pij = c / n;
            let pi = mf[fb] as f64 / n;
            let pj = mm[mb] as f64 / n;
            mi += pij * (pij / (pi * pj)).ln();
        }
    }
    mi
}

/// Shared registration core. `affine == false` optimises 6 dof
/// (3 rotations in degrees + 3 translations in mm); `affine == true`
/// optimises a full 3x3 linear map + translation (12 dof). Both use
/// normalized cross-correlation and a deterministic pattern-search
/// optimizer, warping the moving volume onto the fixed grid.
fn register_transform(
    input: &Path,
    moving_path: &Path,
    output: &Path,
    affine: bool,
    metric: &str,
    max_evaluations: usize,
) -> Result<Value, CoreError> {
    let (fixed, fixed_info) = read_volume(input)?;
    let (moving, moving_info) = read_volume(moving_path)?;
    let fixed_shape = fixed_info.shape;
    let moving_shape = moving_info.shape;
    let affine_of = |info: &VolumeInfo| -> ([[f64; 3]; 3], [f64; 3]) {
        let mut rot = [[0.0f64; 3]; 3];
        let mut trans = [0.0f64; 3];
        for i in 0..3 {
            for j in 0..3 {
                rot[i][j] = info.affine[i][j];
            }
            trans[i] = info.affine[i][3];
        }
        (rot, trans)
    };
    let (fa, ft) = affine_of(&fixed_info);
    let (ma, mt) = affine_of(&moving_info);
    let ma_inv = invert3(&ma)
        .ok_or_else(|| CoreError::InvalidInput("moving volume has a singular affine".into()))?;
    // Intensity ranges for the mutual-information histogram.
    let mut fmin = f64::INFINITY;
    let mut fmax = f64::NEG_INFINITY;
    for v in fixed.iter() {
        let v = *v as f64;
        if v.is_finite() {
            fmin = fmin.min(v);
            fmax = fmax.max(v);
        }
    }
    let mut mmin = f64::INFINITY;
    let mut mmax = f64::NEG_INFINITY;
    for v in moving.iter() {
        let v = *v as f64;
        if v.is_finite() {
            mmin = mmin.min(v);
            mmax = mmax.max(v);
        }
    }

    let voxels = fixed_shape.iter().product::<usize>();
    let stride = ((voxels as f64 / 120_000.0).cbrt().ceil() as usize).max(1);
    let mut samples: Vec<(usize, usize, usize)> = Vec::new();
    for z in (0..fixed_shape[2]).step_by(stride) {
        for y in (0..fixed_shape[1]).step_by(stride) {
            for x in (0..fixed_shape[0]).step_by(stride) {
                samples.push((x, y, z));
            }
        }
    }
    let fixed_world: Vec<[f64; 3]> = samples
        .iter()
        .map(|&(x, y, z)| {
            [
                fa[0][0] * x as f64 + fa[0][1] * y as f64 + fa[0][2] * z as f64 + ft[0],
                fa[1][0] * x as f64 + fa[1][1] * y as f64 + fa[1][2] * z as f64 + ft[1],
                fa[2][0] * x as f64 + fa[2][1] * y as f64 + fa[2][2] * z as f64 + ft[2],
            ]
        })
        .collect();
    let fixed_vals: Vec<f64> = samples
        .iter()
        .map(|&(x, y, z)| fixed[[x, y, z]] as f64)
        .collect();

    // params -> world-space transform p_moving = T(p_fixed).
    // rigid:  [rx, ry, rz (deg), tx, ty, tz (mm)]
    // affine: [a00..a22 row-major, tx, ty, tz (mm)]
    if !matches!(metric, "ncc" | "mi") {
        return Err(CoreError::InvalidInput(format!(
            "unsupported registration metric: {metric}"
        )));
    }
    let n_params = if affine { 12 } else { 6 };
    let apply = |params: &[f64], w: [f64; 3]| -> [f64; 3] {
        if affine {
            [
                params[0] * w[0] + params[1] * w[1] + params[2] * w[2] + params[9],
                params[3] * w[0] + params[4] * w[1] + params[5] * w[2] + params[10],
                params[6] * w[0] + params[7] * w[1] + params[8] * w[2] + params[11],
            ]
        } else {
            let (rx, ry, rz) = (
                params[0].to_radians(),
                params[1].to_radians(),
                params[2].to_radians(),
            );
            let (cx, sx) = (rx.cos(), rx.sin());
            let (cy, sy) = (ry.cos(), ry.sin());
            let (cz, sz) = (rz.cos(), rz.sin());
            let r = [
                [cy * cz, sx * sy * cz - cx * sz, cx * sy * cz + sx * sz],
                [cy * sz, sx * sy * sz + cx * cz, cx * sy * sz - sx * cz],
                [-sy, sx * cy, cx * cy],
            ];
            [
                r[0][0] * w[0] + r[0][1] * w[1] + r[0][2] * w[2] + params[3],
                r[1][0] * w[0] + r[1][1] * w[1] + r[1][2] * w[2] + params[4],
                r[2][0] * w[0] + r[2][1] * w[1] + r[2][2] * w[2] + params[5],
            ]
        }
    };

    let eval = |params: &[f64]| -> f64 {
        let mut pairs: Vec<(f64, f64)> = Vec::with_capacity(samples.len());
        for (w, &fv) in fixed_world.iter().zip(fixed_vals.iter()) {
            if !fv.is_finite() {
                continue;
            }
            let pw = apply(params, *w);
            let d = [pw[0] - mt[0], pw[1] - mt[1], pw[2] - mt[2]];
            let v = [
                ma_inv[0][0] * d[0] + ma_inv[0][1] * d[1] + ma_inv[0][2] * d[2],
                ma_inv[1][0] * d[0] + ma_inv[1][1] * d[1] + ma_inv[1][2] * d[2],
                ma_inv[2][0] * d[0] + ma_inv[2][1] * d[1] + ma_inv[2][2] * d[2],
            ];
            if let Some(mv) = sample_trilinear(&moving, moving_shape, v) {
                pairs.push((fv, mv));
            }
        }
        if pairs.len() < 16 {
            return f64::NEG_INFINITY;
        }
        let n = pairs.len() as f64;
        let mf = pairs.iter().map(|p| p.0).sum::<f64>() / n;
        let mm = pairs.iter().map(|p| p.1).sum::<f64>() / n;
        let mut cov = 0.0;
        let mut vf = 0.0;
        let mut vm = 0.0;
        for &(f, m) in &pairs {
            cov += (f - mf) * (m - mm);
            vf += (f - mf).powi(2);
            vm += (m - mm).powi(2);
        }
        if vf <= 0.0 || vm <= 0.0 {
            return f64::NEG_INFINITY;
        }
        if metric == "mi" {
            return mutual_information(&pairs, fmin, fmax, mmin, mmax);
        }
        cov / (vf * vm).sqrt()
    };

    let max_evals = max_evaluations.clamp(10, 4000);
    let mut params = if affine {
        let mut p = vec![0.0f64; n_params];
        for i in 0..3 {
            p[i * 4] = 1.0; // identity diagonal
        }
        p
    } else {
        vec![0.0; n_params]
    };
    let mut steps = vec![0.0f64; n_params];
    let fov = [
        fixed_shape[0] as f64 * fixed_info.spacing_mm[0],
        fixed_shape[1] as f64 * fixed_info.spacing_mm[1],
        fixed_shape[2] as f64 * fixed_info.spacing_mm[2],
    ];
    for dim in 0..n_params {
        steps[dim] = if affine {
            if dim < 9 {
                0.08
            } else {
                (fov[dim - 9] * 0.05).max(1.0)
            }
        } else if dim < 3 {
            4.0
        } else {
            (fov[dim - 3] * 0.05).max(1.0)
        };
    }
    let mut best = eval(&params);
    let mut evals = 1usize;
    while evals < max_evals {
        let mut improved = false;
        for dim in 0..n_params {
            for sign in [-1.0f64, 1.0] {
                let mut trial = params.clone();
                trial[dim] += sign * steps[dim];
                let score = eval(&trial);
                evals += 1;
                if score > best {
                    best = score;
                    params = trial;
                    improved = true;
                }
                if evals >= max_evals {
                    break;
                }
            }
            if evals >= max_evals {
                break;
            }
        }
        if !improved {
            for s in &mut steps {
                *s *= 0.5;
            }
            let conv = if affine {
                steps[0] < 0.005 && steps[9] < 0.02
            } else {
                steps[0] < 0.05 && steps[3] < 0.05
            };
            if conv {
                break;
            }
        }
    }

    let identity_params = if affine {
        let mut p = vec![0.0f64; n_params];
        for i in 0..3 {
            p[i * 4] = 1.0;
        }
        p
    } else {
        vec![0.0; n_params]
    };
    let identity_score = eval(&identity_params);
    let mut out = Array3::<f32>::from_elem(fixed_shape, f32::NAN);
    for z in 0..fixed_shape[2] {
        for y in 0..fixed_shape[1] {
            for x in 0..fixed_shape[0] {
                let w = [
                    fa[0][0] * x as f64 + fa[0][1] * y as f64 + fa[0][2] * z as f64 + ft[0],
                    fa[1][0] * x as f64 + fa[1][1] * y as f64 + fa[1][2] * z as f64 + ft[1],
                    fa[2][0] * x as f64 + fa[2][1] * y as f64 + fa[2][2] * z as f64 + ft[2],
                ];
                let pw = apply(&params, w);
                let d = [pw[0] - mt[0], pw[1] - mt[1], pw[2] - mt[2]];
                let v = [
                    ma_inv[0][0] * d[0] + ma_inv[0][1] * d[1] + ma_inv[0][2] * d[2],
                    ma_inv[1][0] * d[0] + ma_inv[1][1] * d[1] + ma_inv[1][2] * d[2],
                    ma_inv[2][0] * d[0] + ma_inv[2][1] * d[1] + ma_inv[2][2] * d[2],
                ];
                if let Some(mv) = sample_trilinear(&moving, moving_shape, v) {
                    out[[x, y, z]] = mv as f32;
                }
            }
        }
    }
    write_volume(&out, input, output, None)?;
    let mut result = json!({
        "ncc_before": identity_score,
        "ncc_after": best,
        "evaluations": evals,
        "translation_mm": if affine {
            json!([params[9], params[10], params[11]])
        } else {
            json!([params[3], params[4], params[5]])
        },
    });
    if affine {
        result["operation"] = json!("register_affine");
        result["matrix"] = json!([
            [params[0], params[1], params[2]],
            [params[3], params[4], params[5]],
            [params[6], params[7], params[8]],
        ]);
    } else {
        result["operation"] = json!("register_rigid");
        result["rotation_deg"] = json!([params[0], params[1], params[2]]);
    }
    Ok(result)
}

/// Rigid-register `moving_path` onto `input` (6 dof: 3 rotations in degrees,
/// 3 translations in mm, applied in world coordinates) using normalized
/// cross-correlation and a deterministic pattern-search optimizer.
/// Writes the warped moving volume onto the fixed grid.
pub fn register_rigid(
    input: &Path,
    moving_path: &Path,
    output: &Path,
    max_evaluations: usize,
) -> Result<Value, CoreError> {
    register_rigid_with_metric(input, moving_path, output, "ncc", max_evaluations)
}

/// Rigid registration with a selectable similarity metric (`"ncc"` or `"mi"`).
pub fn register_rigid_with_metric(
    input: &Path,
    moving_path: &Path,
    output: &Path,
    metric: &str,
    max_evaluations: usize,
) -> Result<Value, CoreError> {
    register_transform(input, moving_path, output, false, metric, max_evaluations)
}

/// Affine-register `moving_path` onto `input` (12 dof: full 3x3 linear map
/// plus translation, in world coordinates) using normalized cross-correlation
/// and the same deterministic pattern-search optimizer. Use after
/// `register_rigid` for residual scale/shear correction.
pub fn register_affine(
    input: &Path,
    moving_path: &Path,
    output: &Path,
    max_evaluations: usize,
) -> Result<Value, CoreError> {
    register_affine_with_metric(input, moving_path, output, "ncc", max_evaluations)
}

/// Affine registration with a selectable similarity metric (`"ncc"` or `"mi"`).
pub fn register_affine_with_metric(
    input: &Path,
    moving_path: &Path,
    output: &Path,
    metric: &str,
    max_evaluations: usize,
) -> Result<Value, CoreError> {
    register_transform(input, moving_path, output, true, metric, max_evaluations)
}

/// Heuristic brain extraction ("skull stripping"): Otsu head mask, erosion of
/// the outer `erosion_mm` shell to remove scalp/skull, largest 26-connected
/// component, then hole filling. This is a fast approximation for research
/// use — it is NOT a validated BET-style extraction and needs visual review.
/// Writes a binary mask volume; chain with `apply_mask` to strip the image.
pub fn skull_strip(
    input: &Path,
    mask_output: &Path,
    threshold: Option<f64>,
    erosion_mm: f64,
) -> Result<Value, CoreError> {
    let (volume, info) = read_volume(input)?;
    let shape = volume.raw_dim();
    let mut values: Vec<f64> = Vec::new();
    for index in ndarray::indices(shape) {
        let v = volume[index] as f64;
        if v.is_finite() && v > 0.0 {
            values.push(v);
        }
    }
    if values.is_empty() {
        return Err(CoreError::InvalidInput(
            "volume has no positive intensities for skull stripping".into(),
        ));
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let low = threshold.unwrap_or_else(|| otsu_threshold(&values));

    let mut mask = vec![false; shape[0] * shape[1] * shape[2]];
    let idx = |x: usize, y: usize, z: usize| (z * shape[1] + y) * shape[0] + x;
    let mut head_count = 0usize;
    for z in 0..shape[2] {
        for y in 0..shape[1] {
            for x in 0..shape[0] {
                let v = volume[[x, y, z]] as f64;
                if v.is_finite() && v >= low {
                    mask[idx(x, y, z)] = true;
                    head_count += 1;
                }
            }
        }
    }
    // Erode the outer shell: `steps` unit 6-neighbour erosions approximates a
    // spherical shell of the given thickness.
    let mean_spacing = info.spacing_mm.iter().sum::<f64>() / 3.0;
    let steps = (erosion_mm.max(0.0) / mean_spacing.max(1e-6)).round() as usize;
    for _ in 0..steps {
        let mut next = mask.clone();
        for z in 1..shape[2].saturating_sub(1) {
            for y in 1..shape[1].saturating_sub(1) {
                for x in 1..shape[0].saturating_sub(1) {
                    let i = idx(x, y, z);
                    if !mask[i] {
                        continue;
                    }
                    let kept = [
                        mask[idx(x - 1, y, z)],
                        mask[idx(x + 1, y, z)],
                        mask[idx(x, y - 1, z)],
                        mask[idx(x, y + 1, z)],
                        mask[idx(x, y, z - 1)],
                        mask[idx(x, y, z + 1)],
                    ]
                    .iter()
                    .all(|v| *v);
                    if !kept {
                        next[i] = false;
                    }
                }
            }
        }
        // Border voxels erode away too.
        mask = next;
        for z in 0..shape[2] {
            for y in 0..shape[1] {
                for x in 0..shape[0] {
                    if x == 0
                        || y == 0
                        || z == 0
                        || x + 1 == shape[0]
                        || y + 1 == shape[1]
                        || z + 1 == shape[2]
                    {
                        mask[idx(x, y, z)] = false;
                    }
                }
            }
        }
    }
    // Largest 26-connected component.
    let mut best_label = 0usize;
    let mut best_size = 0usize;
    let mut labels = vec![0usize; mask.len()];
    let mut label = 0usize;
    let mut stack = Vec::new();
    for start in 0..mask.len() {
        if !mask[start] || labels[start] != 0 {
            continue;
        }
        label += 1;
        let mut size = 0usize;
        stack.push(start);
        labels[start] = label;
        while let Some(i) = stack.pop() {
            size += 1;
            let z = i / (shape[1] * shape[0]);
            let rem = i % (shape[1] * shape[0]);
            let y = rem / shape[0];
            let x = rem % shape[0];
            for dz in -1i32..=1 {
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 && dz == 0 {
                            continue;
                        }
                        let (nx, ny, nz) = (x as i32 + dx, y as i32 + dy, z as i32 + dz);
                        if nx < 0
                            || ny < 0
                            || nz < 0
                            || nx >= shape[0] as i32
                            || ny >= shape[1] as i32
                            || nz >= shape[2] as i32
                        {
                            continue;
                        }
                        let ni = idx(nx as usize, ny as usize, nz as usize);
                        if mask[ni] && labels[ni] == 0 {
                            labels[ni] = label;
                            stack.push(ni);
                        }
                    }
                }
            }
        }
        if size > best_size {
            best_size = size;
            best_label = label;
        }
    }
    if best_label == 0 {
        return Err(CoreError::InvalidInput(
            "skull stripping produced an empty mask".into(),
        ));
    }
    for i in 0..mask.len() {
        mask[i] = labels[i] == best_label;
    }
    // Fill holes: flood "outside" from the volume border through non-mask
    // voxels; unreached non-mask voxels are enclosed holes.
    let mut outside = vec![false; mask.len()];
    let mut queue = std::collections::VecDeque::new();
    for x in 0..shape[0] {
        for y in 0..shape[1] {
            for z in [0usize, shape[2] - 1] {
                let i = idx(x, y, z);
                if !mask[i] && !outside[i] {
                    outside[i] = true;
                    queue.push_back(i);
                }
            }
        }
    }
    for z in 0..shape[2] {
        for y in 0..shape[1] {
            for x in [0usize, shape[0] - 1] {
                let i = idx(x, y, z);
                if !mask[i] && !outside[i] {
                    outside[i] = true;
                    queue.push_back(i);
                }
            }
        }
    }
    for y in 0..shape[1] {
        for x in 0..shape[0] {
            for z in [0usize, shape[2] - 1] {
                let i = idx(x, y, z);
                if !mask[i] && !outside[i] {
                    outside[i] = true;
                    queue.push_back(i);
                }
            }
        }
    }
    while let Some(i) = queue.pop_front() {
        let z = i / (shape[1] * shape[0]);
        let rem = i % (shape[1] * shape[0]);
        let y = rem / shape[0];
        let x = rem % shape[0];
        for (dx, dy, dz) in [
            (-1i32, 0, 0),
            (1, 0, 0),
            (0, -1, 0),
            (0, 1, 0),
            (0, 0, -1),
            (0, 0, 1),
        ] {
            let (nx, ny, nz) = (x as i32 + dx, y as i32 + dy, z as i32 + dz);
            if nx < 0
                || ny < 0
                || nz < 0
                || nx >= shape[0] as i32
                || ny >= shape[1] as i32
                || nz >= shape[2] as i32
            {
                continue;
            }
            let ni = idx(nx as usize, ny as usize, nz as usize);
            if !mask[ni] && !outside[ni] {
                outside[ni] = true;
                queue.push_back(ni);
            }
        }
    }
    let mut brain_count = 0usize;
    let mut out = Array3::<f32>::zeros(shape);
    for z in 0..shape[2] {
        for y in 0..shape[1] {
            for x in 0..shape[0] {
                let i = idx(x, y, z);
                // Mask voxel = inside component, or a hole fully enclosed.
                if mask[i] || !outside[i] {
                    out[[x, y, z]] = 1.0;
                    brain_count += 1;
                }
            }
        }
    }
    write_volume(&out, input, mask_output, None)?;
    let voxel_mm3 = info.spacing_mm.iter().product::<f64>();
    Ok(json!({
        "operation": "skull_strip",
        "threshold": low,
        "erosion_mm": erosion_mm,
        "erosion_steps": steps,
        "head_voxels": head_count,
        "brain_voxels": brain_count,
        "brain_volume_mm3": brain_count as f64 * voxel_mm3,
        "voxel_count": info.voxel_count,
        "note": "heuristic extraction; verify mask visually before use",
    }))
}

/// Otsu threshold over sorted positive intensities (256-bin histogram).
fn otsu_threshold(sorted: &[f64]) -> f64 {
    let lo = *sorted.first().unwrap_or(&0.0);
    let hi = *sorted.last().unwrap_or(&1.0);
    if hi <= lo {
        return lo;
    }
    let bins = 256usize;
    let mut hist = vec![0u64; bins];
    for &v in sorted {
        let b = (((v - lo) / (hi - lo)) * (bins - 1) as f64).round() as usize;
        hist[b.min(bins - 1)] += 1;
    }
    let total = sorted.len() as f64;
    let mut sum_all = 0.0;
    for (i, &h) in hist.iter().enumerate() {
        sum_all += i as f64 * h as f64;
    }
    let mut best_t = 0usize;
    let mut best_var = -1.0f64;
    let mut w0 = 0.0f64;
    let mut sum0 = 0.0f64;
    for t in 0..bins {
        w0 += hist[t] as f64;
        sum0 += t as f64 * hist[t] as f64;
        let w1 = total - w0;
        if w0 == 0.0 || w1 == 0.0 {
            continue;
        }
        let m0 = sum0 / w0;
        let m1 = (sum_all - sum0) / w1;
        let var = w0 * w1 * (m0 - m1).powi(2);
        if var > best_var {
            best_var = var;
            best_t = t;
        }
    }
    lo + (best_t as f64 / (bins - 1) as f64) * (hi - lo)
}
