//! Bounded reader for CTF `.ds` datasets (`.res4` resource file + `.meg4`
//! data files).
//!
//! Parses the big-endian res4 header through the channel-name table and
//! per-channel sensor records (gain, kind, coil geometry), and decodes meg4
//! int32 samples stored channel-major within each trial, scaled by
//! `1 / (proper_gain * qgain)` into tesla. Compensation/balance matrices,
//! synthetic gradiometers, and head-coil files are not applied.

use crate::{MegChannel, MegEvent, MegWindow};
use neuro_core::CoreError;
use serde::Serialize;
use std::path::{Path, PathBuf};

const MAX_CHANNELS: usize = 4096;
const MAX_TOTAL_SAMPLES: usize = 200_000_000;
/// Packed big-endian per-channel record following the 32-byte name table:
/// 48-byte header + 8 coils x 80 bytes + 8 head coils x 80 bytes.
const CH_INFO_SIZE: usize = 1328;

#[derive(Clone, Debug)]
pub struct CtfChannelMeta {
    pub kind: String,
    pub unit: String,
    pub conv_factor: f64,
    pub proper_gain: f64,
    pub qgain: f64,
    pub grad_order_no: i32,
    pub coil_type: Option<i32>,
    pub position_mm: Option<Vec<f64>>,
    pub orientation: Option<Vec<f64>>,
}

/// A CTF balance/compensation matrix record (`ncomp` block in res4).
/// `sensors`/`coeffs` are parallel lists; applying them to data is not yet
/// implemented — they are exposed for provenance and future compensation.
#[derive(Clone, Debug, PartialEq, Serialize, serde::Deserialize)]
pub struct CtfCompMatrix {
    pub sensor_name: String,
    pub coeff_type: i32,
    pub sensors: Vec<String>,
    pub coeffs: Vec<f64>,
}

pub struct CtfInfo {
    pub n_channels: usize,
    pub sampling_frequency_hz: f64,
    pub n_samples_per_trial: usize,
    pub n_trials: usize,
    pub channel_names: Vec<String>,
    pub channel_meta: Vec<CtfChannelMeta>,
    pub meg4_files: Vec<PathBuf>,
    pub trials_per_file: Vec<usize>,
    pub pre_trig_pts: i32,
    pub events: Vec<MegEvent>,
    pub compensation: Vec<CtfCompMatrix>,
}

fn be_i16(bytes: &[u8], pos: usize) -> Option<i16> {
    Some(i16::from_be_bytes(
        bytes.get(pos..pos + 2)?.try_into().ok()?,
    ))
}

fn be_i32(bytes: &[u8], pos: usize) -> Option<i32> {
    Some(i32::from_be_bytes(
        bytes.get(pos..pos + 4)?.try_into().ok()?,
    ))
}

fn be_f64(bytes: &[u8], pos: usize) -> Option<f64> {
    Some(f64::from_be_bytes(
        bytes.get(pos..pos + 8)?.try_into().ok()?,
    ))
}

fn cstr(bytes: &[u8], pos: usize, len: usize) -> String {
    let end = (pos + len).min(bytes.len());
    let raw = &bytes[pos..end];
    let stop = raw.iter().position(|b| *b == 0).unwrap_or(raw.len());
    String::from_utf8_lossy(&raw[..stop]).trim().to_string()
}

fn find_res4(dir: &Path) -> Result<PathBuf, CoreError> {
    let mut candidates: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("res4"))
        })
        .collect();
    candidates.sort();
    candidates
        .into_iter()
        .next()
        .ok_or_else(|| CoreError::NotFound("CTF .res4 file not found".into()))
}

fn find_meg4_files(dir: &Path, res4_name: &str) -> Result<Vec<PathBuf>, CoreError> {
    // Primary file is <base>.meg4; overflow files are <base>.<n>_meg4.
    let base = res4_name
        .trim_end_matches(".res4")
        .trim_end_matches(".RES4");
    let mut primary = None;
    let mut extras: Vec<(usize, PathBuf)> = Vec::new();
    for entry in std::fs::read_dir(dir)?.filter_map(Result::ok) {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let lower = name.to_lowercase();
        if lower == format!("{}.meg4", base.to_lowercase()) {
            primary = Some(path);
        } else if let Some(rest) = lower
            .strip_prefix(&format!("{}.", base.to_lowercase()))
            .and_then(|rest| rest.strip_suffix("_meg4"))
        {
            if let Ok(index) = rest.parse::<usize>() {
                extras.push((index, path));
            }
        }
    }
    extras.sort_by_key(|(index, _)| *index);
    let mut files = Vec::new();
    if let Some(primary) = primary {
        files.push(primary);
    }
    files.extend(extras.into_iter().map(|(_, path)| path));
    if files.is_empty() {
        return Err(CoreError::NotFound("CTF .meg4 data file not found".into()));
    }
    Ok(files)
}

pub fn inspect_ctf(dir: &Path) -> Result<CtfInfo, CoreError> {
    let res4_path = find_res4(dir)?;
    let res4_name = res4_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_string();
    let bytes = std::fs::read(&res4_path)?;
    if bytes.len() < 1848 {
        return Err(CoreError::InvalidInput("CTF .res4 file too small".into()));
    }
    let header = cstr(&bytes, 0, 8);
    if !header.starts_with("MEG4") {
        return Err(CoreError::InvalidInput(format!(
            "unsupported CTF res4 header: {header}"
        )));
    }
    let no_samples = be_i32(&bytes, 1288).unwrap_or(0);
    let no_channels = be_i16(&bytes, 1292).unwrap_or(0);
    let sample_rate = be_f64(&bytes, 1296).unwrap_or(0.0);
    let no_trials = be_i16(&bytes, 1312).unwrap_or(0).max(1) as usize;
    if no_samples <= 0 || no_channels <= 0 || !(sample_rate > 0.0 && sample_rate.is_finite()) {
        return Err(CoreError::InvalidInput(
            "invalid CTF res4 header fields".into(),
        ));
    }
    let n_channels = no_channels as usize;
    if n_channels > MAX_CHANNELS {
        return Err(CoreError::InvalidInput(
            "CTF channel count too large".into(),
        ));
    }
    // pre_trig_pts is a big-endian i32 at offset 1316 (after the 4-byte
    // alignment following no_trials).
    let pre_trig_pts = be_i32(&bytes, 1316).unwrap_or(0);
    // run_description length at offset 1836 (rdlen), text at 1844.
    let rdlen = be_i32(&bytes, 1836).unwrap_or(0).max(0) as usize;
    let mut pos = 1844 + rdlen;
    if pos + 2 > bytes.len() {
        return Err(CoreError::InvalidInput("truncated CTF res4".into()));
    }
    let num_filters = be_i16(&bytes, pos).unwrap_or(0).max(0) as usize;
    pos += 2;
    for _ in 0..num_filters {
        // freq f64, fClass i32, fType i32, numParam i16, numParam x f64
        let num_param = be_i16(&bytes, pos + 16).unwrap_or(0).max(0) as usize;
        pos += 8 + 4 + 4 + 2 + num_param * 8;
    }
    if pos + n_channels * 32 > bytes.len() {
        return Err(CoreError::InvalidInput(
            "truncated CTF channel table".into(),
        ));
    }
    let channel_names = (0..n_channels)
        .map(|index| {
            let name = cstr(&bytes, pos + index * 32, 32);
            if name.is_empty() {
                format!("CTF {:03}", index + 1)
            } else {
                name
            }
        })
        .collect::<Vec<_>>();

    // Per-channel sensor records follow the name table. Tolerate their
    // absence (synthetic/older files) by falling back to raw-count defaults.
    let mut channel_meta = Vec::with_capacity(n_channels);
    let info_base = pos + n_channels * 32;
    for index in 0..n_channels {
        let base = info_base + index * CH_INFO_SIZE;
        let meta = if base + CH_INFO_SIZE <= bytes.len() {
            let sensor_type = be_i16(&bytes, base).unwrap_or(-1);
            let (kind, unit) = match sensor_type {
                0 | 1 | 5 => ("meg", "tesla"),
                9 => ("eeg", "volt"),
                11 => ("stim", "dimensionless"),
                _ => ("other", "other"),
            };
            let proper_gain = be_f64(&bytes, base + 8).unwrap_or(0.0);
            let qgain = be_f64(&bytes, base + 16).unwrap_or(0.0);
            let conv_factor = {
                let product = proper_gain * qgain;
                if product.is_finite() && product != 0.0 {
                    1.0 / product
                } else {
                    1.0
                }
            };
            // First coil record: pos (3 x f64, scaled /100 -> m) and
            // norm (3 x f64 unit vector). positive proper_gain flips the
            // normal, matching MNE's convention.
            let coil = base + 48;
            let pos_raw: Vec<f64> = (0..3)
                .filter_map(|k| be_f64(&bytes, coil + k * 8))
                .collect();
            let mut norm: Vec<f64> = (0..3)
                .filter_map(|k| be_f64(&bytes, coil + 32 + k * 8))
                .collect();
            if proper_gain > 0.0 {
                for v in &mut norm {
                    *v = -*v;
                }
            }
            let norm_len = (norm.iter().map(|v| v * v).sum::<f64>()).sqrt();
            CtfChannelMeta {
                kind: kind.into(),
                unit: unit.into(),
                conv_factor,
                proper_gain,
                qgain,
                grad_order_no: be_i16(&bytes, base + 42).unwrap_or(-1) as i32,
                coil_type: be_i32(&bytes, base + 4),
                position_mm: if pos_raw.len() == 3
                    && pos_raw.iter().all(|v| v.is_finite())
                    && pos_raw.iter().map(|v| v * v).sum::<f64>() > 1e-6
                {
                    // res4 stores coil positions in units that MNE divides
                    // by 100 to obtain metres; convert to millimetres.
                    Some(pos_raw.iter().map(|v| v * 10.0).collect())
                } else {
                    None
                },
                orientation: if norm.len() == 3
                    && norm.iter().all(|v| v.is_finite())
                    && norm_len > 1e-6
                {
                    Some(norm)
                } else {
                    None
                },
            }
        } else {
            CtfChannelMeta {
                kind: "meg".into(),
                unit: "other".into(),
                conv_factor: 1.0,
                proper_gain: 0.0,
                qgain: 0.0,
                grad_order_no: -1,
                coil_type: None,
                position_mm: None,
                orientation: None,
            }
        };
        channel_meta.push(meta);
    }

    // Compensation/balance coefficients follow the channel records:
    // ncomp i16, then ncomp x 1992-byte records (32-byte sensor name,
    // coeff_type i32, d0 i32, ncoeff i16, 50 x 31-byte sensor labels,
    // 50 x f64 coefficients).
    let mut compensation = Vec::new();
    let comp_base = info_base + n_channels * CH_INFO_SIZE;
    if comp_base + 2 <= bytes.len() {
        let ncomp = be_i16(&bytes, comp_base).unwrap_or(0).clamp(0, 1024) as usize;
        const COMP_RECORD: usize = 1992;
        for index in 0..ncomp {
            let base = comp_base + 2 + index * COMP_RECORD;
            if base + COMP_RECORD > bytes.len() {
                break;
            }
            let ncoeff = be_i16(&bytes, base + 40).unwrap_or(0).clamp(0, 50) as usize;
            let sensors = (0..ncoeff)
                .map(|k| cstr(&bytes, base + 42 + k * 31, 31))
                .filter(|name| !name.is_empty())
                .collect::<Vec<_>>();
            let coeffs = (0..ncoeff)
                .filter_map(|k| be_f64(&bytes, base + 1592 + k * 8))
                .collect::<Vec<_>>();
            compensation.push(CtfCompMatrix {
                sensor_name: cstr(&bytes, base, 32),
                coeff_type: be_i32(&bytes, base + 32).unwrap_or(0),
                sensors,
                coeffs,
            });
        }
    }
    let total = no_samples as usize * no_trials;
    if total > MAX_TOTAL_SAMPLES {
        return Err(CoreError::InvalidInput("CTF recording too large".into()));
    }
    let meg4_files = find_meg4_files(dir, &res4_name)?;
    let mut trials_per_file = Vec::with_capacity(meg4_files.len());
    for file in &meg4_files {
        let size = std::fs::metadata(file)?.len() as usize;
        let per_file = size.saturating_sub(8) / (4 * n_channels * no_samples as usize);
        trials_per_file.push(per_file);
    }
    let events = read_markers(
        dir,
        sample_rate,
        no_samples as usize,
        no_trials,
        pre_trig_pts,
    );
    Ok(CtfInfo {
        n_channels,
        sampling_frequency_hz: sample_rate,
        n_samples_per_trial: no_samples as usize,
        n_trials: no_trials,
        channel_names,
        channel_meta,
        meg4_files,
        trials_per_file,
        pre_trig_pts,
        events,
        compensation,
    })
}

/// Parse `MarkerFile.mrk` — a text file of named marker lists. Each marker
/// is a `(trial, sync_seconds)` pair; onset in seconds is
/// `sync + trial * trial_duration + pre_trig_pts / sfreq`.
fn read_markers(
    dir: &Path,
    sfreq: f64,
    samples_per_trial: usize,
    n_trials: usize,
    pre_trig_pts: i32,
) -> Vec<MegEvent> {
    let marker_path = std::fs::read_dir(dir).ok().and_then(|entries| {
        entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|path| {
                path.file_name()
                    .and_then(|value| value.to_str())
                    .is_some_and(|name| name.eq_ignore_ascii_case("markerfile.mrk"))
            })
    });
    let Some(path) = marker_path else {
        return Vec::new();
    };
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let lines: Vec<&str> = text.lines().collect();
    let mut events = Vec::new();
    let mut i = 0usize;
    let total_samples = samples_per_trial.saturating_mul(n_trials.max(1));
    let trial_duration = samples_per_trial as f64 / sfreq;
    let pre_trig_s = pre_trig_pts as f64 / sfreq;
    // Read a count that may sit on the same line after ':' or on the next.
    fn count_after(lines: &[&str], i: &mut usize, tag: &str) -> Option<usize> {
        let line = lines.get(*i)?;
        if let Some(rest) = line.trim().strip_prefix(tag) {
            if let Ok(value) = rest.trim().parse::<usize>() {
                *i += 1;
                return Some(value);
            }
        }
        *i += 1;
        let value = lines.get(*i)?.trim().parse::<usize>().ok()?;
        *i += 1;
        Some(value)
    }

    while i < lines.len() {
        if !lines[i].trim_start().starts_with("NUMBER OF MARKERS:") {
            i += 1;
            continue;
        }
        let Some(num_markers) = count_after(&lines, &mut i, "NUMBER OF MARKERS:") else {
            break;
        };
        for _ in 0..num_markers {
            // Find "NAME:" then the label on the following line.
            while i < lines.len() && !lines[i].trim_start().starts_with("NAME:") {
                i += 1;
            }
            if i >= lines.len() {
                break;
            }
            i += 1;
            let label = lines
                .get(i)
                .map(|l| l.trim().to_string())
                .unwrap_or_default();
            i += 1;
            while i < lines.len() && !lines[i].trim_start().starts_with("NUMBER OF SAMPLES:") {
                i += 1;
            }
            if i >= lines.len() {
                break;
            }
            let Some(n_samples) = count_after(&lines, &mut i, "NUMBER OF SAMPLES:") else {
                break;
            };
            while i < lines.len() && !lines[i].trim_start().starts_with("LIST OF SAMPLES:") {
                i += 1;
            }
            if i >= lines.len() {
                break;
            }
            i += 2; // skip the "LIST OF SAMPLES:" line and its column header
            for _ in 0..n_samples {
                let Some(line) = lines.get(i) else { break };
                i += 1;
                let mut parts = line.split_whitespace();
                let (Some(trial), Some(sync)) = (
                    parts.next().and_then(|v| v.parse::<f64>().ok()),
                    parts.next().and_then(|v| v.parse::<f64>().ok()),
                ) else {
                    continue;
                };
                let onset = sync + trial * trial_duration + pre_trig_s;
                if !onset.is_finite() || onset < 0.0 {
                    continue;
                }
                let sample = ((onset * sfreq).round() as usize).min(total_samples);
                events.push(MegEvent {
                    sample,
                    time_s: onset,
                    code: Some(label.clone()),
                    value: None,
                    description: Some(label.clone()),
                });
            }
        }
    }
    events
}

fn locate_sample(info: &CtfInfo, global_sample: usize) -> Option<(usize, usize, usize)> {
    // Returns (file_index, local_trial, sample_in_trial)
    let nsmp = info.n_samples_per_trial;
    if nsmp == 0 {
        return None;
    }
    let trial = global_sample / nsmp;
    let sample = global_sample % nsmp;
    let mut remaining = trial;
    for (file_index, &per_file) in info.trials_per_file.iter().enumerate() {
        if remaining < per_file || file_index == info.trials_per_file.len() - 1 {
            return Some((file_index, remaining, sample));
        }
        remaining -= per_file;
    }
    None
}

/// Stored compensation grade: the modal `grad_order_no` across MEG-type
/// channels (MNE reads the same field as `compensation_grade`).
fn stored_comp_grade(info: &CtfInfo) -> i32 {
    let mut counts: std::collections::BTreeMap<i32, usize> = std::collections::BTreeMap::new();
    for meta in &info.channel_meta {
        if meta.kind == "meg" && meta.grad_order_no >= 0 {
            *counts.entry(meta.grad_order_no).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(grade, _)| grade)
        .unwrap_or(0)
}

/// Dense matrix inverse via Gauss-Jordan with partial pivoting.
fn invert_matrix(matrix: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, CoreError> {
    let n = matrix.len();
    let mut aug = vec![vec![0.0f64; 2 * n]; n];
    for i in 0..n {
        aug[i][..n].copy_from_slice(&matrix[i]);
        aug[i][n + i] = 1.0;
    }
    for col in 0..n {
        let mut pivot = col;
        for row in col + 1..n {
            if aug[row][col].abs() > aug[pivot][col].abs() {
                pivot = row;
            }
        }
        if aug[pivot][col].abs() < 1e-12 {
            return Err(CoreError::InvalidInput(
                "singular compensation matrix".into(),
            ));
        }
        aug.swap(col, pivot);
        let div = aug[col][col];
        for v in aug[col].iter_mut() {
            *v /= div;
        }
        for row in 0..n {
            if row == col {
                continue;
            }
            let factor = aug[row][col];
            if factor == 0.0 {
                continue;
            }
            for k in 0..2 * n {
                aug[row][k] -= factor * aug[col][k];
            }
        }
    }
    Ok((0..n).map(|i| aug[i][n..].to_vec()).collect())
}

/// Apply CTF gradient compensation to channel-major data (rows = all
/// channels, already in physical units). Replicates MNE's
/// `make_compensator(from_, to)`: `comp = (I - C_to) * inv(I - C_from)`
/// restricted to channels appearing in the coefficient records, where each
/// record contributes `coeffs[k] * (pg_col*qg_col) / (pg_row*qg_row)`.
fn apply_compensation(
    info: &CtfInfo,
    data: &mut [Vec<f64>],
    target_grade: i32,
) -> Result<(), CoreError> {
    if info.compensation.is_empty() {
        return Ok(());
    }
    let from_grade = stored_comp_grade(info);
    if from_grade == target_grade {
        return Ok(());
    }
    let name_to_index: std::collections::HashMap<&str, usize> = info
        .channel_names
        .iter()
        .enumerate()
        .map(|(index, name)| (name.as_str(), index))
        .collect();
    // Involved channel set: rows and columns referenced by the two grades.
    let mut involved: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
    for record in &info.compensation {
        if let Some(&index) = name_to_index.get(record.sensor_name.as_str()) {
            involved.insert(index);
        }
        for sensor in &record.sensors {
            if let Some(&index) = name_to_index.get(sensor.as_str()) {
                involved.insert(index);
            }
        }
    }
    let universe: Vec<usize> = involved.into_iter().collect();
    if universe.is_empty() {
        return Ok(());
    }
    if universe.len() > 1024 {
        return Err(CoreError::InvalidInput(
            "compensation channel set too large".into(),
        ));
    }
    let pos: std::collections::HashMap<usize, usize> = universe
        .iter()
        .enumerate()
        .map(|(position, &channel)| (channel, position))
        .collect();
    let size = universe.len();
    let gain = |index: usize| -> f64 {
        let meta = &info.channel_meta[index];
        let product = meta.proper_gain * meta.qgain;
        if product.is_finite() && product != 0.0 {
            product
        } else {
            1.0
        }
    };
    // Build C_g for a grade: dense over the universe.
    let build = |grade: i32| -> Vec<Vec<f64>> {
        let mut matrix = vec![vec![0.0f64; size]; size];
        for record in info
            .compensation
            .iter()
            .filter(|record| record.coeff_type == grade)
        {
            let Some(&row_channel) = name_to_index.get(record.sensor_name.as_str()) else {
                continue;
            };
            let Some(&row_pos) = pos.get(&row_channel) else {
                continue;
            };
            for (k, sensor) in record.sensors.iter().enumerate() {
                let Some(&col_channel) = name_to_index.get(sensor.as_str()) else {
                    continue;
                };
                let Some(&col_pos) = pos.get(&col_channel) else {
                    continue;
                };
                let Some(&coeff) = record.coeffs.get(k) else {
                    continue;
                };
                matrix[row_pos][col_pos] += coeff * gain(col_channel) / gain(row_channel);
            }
        }
        matrix
    };
    let c_from = if from_grade != 0 {
        build(from_grade)
    } else {
        vec![vec![0.0; size]; size]
    };
    let c_to = if target_grade != 0 {
        build(target_grade)
    } else {
        vec![vec![0.0; size]; size]
    };
    // comp = (I - C_to) * inv(I - C_from); identity when grade == 0.
    let mut left = vec![vec![0.0f64; size]; size];
    for i in 0..size {
        for j in 0..size {
            left[i][j] = (if i == j { 1.0 } else { 0.0 }) - c_to[i][j];
        }
    }
    let right = if from_grade != 0 {
        let mut ident_minus = vec![vec![0.0f64; size]; size];
        for i in 0..size {
            for j in 0..size {
                ident_minus[i][j] = (if i == j { 1.0 } else { 0.0 }) - c_from[i][j];
            }
        }
        invert_matrix(&ident_minus)?
    } else {
        let mut ident = vec![vec![0.0f64; size]; size];
        for i in 0..size {
            ident[i][i] = 1.0;
        }
        ident
    };
    // comp = left * right
    let mut comp = vec![vec![0.0f64; size]; size];
    for i in 0..size {
        for k in 0..size {
            if left[i][k] == 0.0 {
                continue;
            }
            for j in 0..size {
                comp[i][j] += left[i][k] * right[k][j];
            }
        }
    }
    // out[u_i] = sum_j comp[i][j] * data[u_j]; compute into temporaries.
    let n_times = data.first().map(Vec::len).unwrap_or(0);
    let mut compensated = vec![vec![0.0f64; n_times]; size];
    for i in 0..size {
        for j in 0..size {
            let weight = comp[i][j];
            if weight == 0.0 {
                continue;
            }
            let source = &data[universe[j]];
            let target = &mut compensated[i];
            for t in 0..n_times {
                target[t] += weight * source[t];
            }
        }
    }
    for (i, &channel) in universe.iter().enumerate() {
        data[channel] = std::mem::take(&mut compensated[i]);
    }
    Ok(())
}

pub fn read_window(
    dir: &Path,
    start_s: f64,
    end_s: f64,
    channel_names: &[String],
    max_samples: usize,
    compensation: Option<i32>,
) -> Result<MegWindow, CoreError> {
    let info = inspect_ctf(dir)?;
    let total_samples = info.n_samples_per_trial * info.n_trials;
    let duration = total_samples as f64 / info.sampling_frequency_hz;
    if start_s < 0.0 || end_s <= start_s || end_s > duration + 1e-9 {
        return Err(CoreError::InvalidInput("invalid CTF window".into()));
    }
    let mut indices = Vec::new();
    for name in channel_names {
        let index = info
            .channel_names
            .iter()
            .position(|candidate| candidate == name)
            .ok_or_else(|| CoreError::InvalidInput(format!("unknown CTF channel: {name}")))?;
        if !indices.contains(&index) {
            indices.push(index);
        }
    }
    if indices.is_empty() {
        indices = (0..info.n_channels).collect();
    }
    let start = ((start_s * info.sampling_frequency_hz).round() as usize).min(total_samples);
    let end = ((end_s * info.sampling_frequency_hz).round() as usize)
        .min(total_samples)
        .max(start);
    let native = end.saturating_sub(start);
    let mut factor = 1usize;
    while native / factor > max_samples {
        factor *= 2;
    }
    // Compensation mixes channels, so it requires the full channel set.
    let comp_target = compensation.filter(|_| !info.compensation.is_empty());
    let read_indices: Vec<usize> = if comp_target.is_some() {
        (0..info.n_channels).collect()
    } else {
        indices.clone()
    };
    let mut times = Vec::new();
    let mut data = vec![Vec::new(); read_indices.len()];
    // Cache file contents lazily per file index.
    let mut file_cache: Vec<Option<Vec<u8>>> = vec![None; info.meg4_files.len()];
    let mut sample = start;
    while sample < end {
        times.push(sample as f64 / info.sampling_frequency_hz);
        let Some((file_index, local_trial, in_trial)) = locate_sample(&info, sample) else {
            break;
        };
        if file_cache[file_index].is_none() {
            file_cache[file_index] = Some(std::fs::read(&info.meg4_files[file_index])?);
        }
        let bytes = file_cache[file_index].as_ref().unwrap();
        let nsmp = info.n_samples_per_trial;
        for (output, &channel) in read_indices.iter().enumerate() {
            let offset =
                8 + local_trial * info.n_channels * nsmp * 4 + channel * nsmp * 4 + in_trial * 4;
            let value = be_i32(bytes, offset).unwrap_or(0) as f64;
            data[output].push(value * info.channel_meta[channel].conv_factor);
        }
        sample += factor;
    }
    if let Some(target) = comp_target {
        // data rows are in channel order (read_indices == 0..n_channels).
        if read_indices.len() != info.n_channels {
            return Err(CoreError::InvalidInput(
                "compensation requires contiguous channel data".into(),
            ));
        }
        apply_compensation(&info, &mut data, target)?;
        // Subset back to the requested channels.
        let position: std::collections::HashMap<usize, usize> = read_indices
            .iter()
            .enumerate()
            .map(|(row, &channel)| (channel, row))
            .collect();
        data = indices
            .iter()
            .map(|channel| data[position[channel]].clone())
            .collect();
    }
    Ok(MegWindow {
        times_s: times,
        data,
        effective_sampling_frequency_hz: info.sampling_frequency_hz / factor as f64,
    })
}

pub fn channels(dir: &Path) -> Result<Vec<MegChannel>, CoreError> {
    let info = inspect_ctf(dir)?;
    Ok(info
        .channel_names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let meta = &info.channel_meta[index];
            MegChannel {
                name: name.clone(),
                kind: meta.kind.clone(),
                unit: meta.unit.clone(),
                unit_mul: 1.0,
                group_name: None,
                status: "good".into(),
                coil_type: meta.coil_type.map(|value| format!("ctf_coil_{value}")),
                position_mm: meta.position_mm.clone(),
                orientation: meta.orientation.clone(),
            }
        })
        .collect())
}
