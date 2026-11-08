//! Deterministic EEG processing nodes: bandpass filtering and Welch PSD,
//! operating on `(channels, samples)` matrices.

use crate::EegEvent;
use neuro_core::CoreError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Upper bound on samples per epoch to keep ERP outputs bounded.
const MAX_EPOCH_SAMPLES: usize = 200_000;
/// Upper bound on epochs extracted per run.
const MAX_EPOCHS: usize = 100_000;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EegPipelineResult {
    pub steps: Vec<String>,
    pub band_powers: Option<BTreeMap<String, f64>>,
    pub bad_channels: Vec<String>,
    pub n_channels: usize,
    pub n_samples: usize,
    #[serde(default)]
    pub n_epochs: usize,
    #[serde(default)]
    pub n_epochs_rejected: usize,
    /// Event-related potential: mean across accepted epochs,
    /// `(channels, epoch_samples)`.
    #[serde(default)]
    pub erp: Option<Vec<Vec<f64>>>,
    /// Per-component variance summary from the most recent `ica` node.
    #[serde(default)]
    pub ica_components: Option<Vec<crate::IcaComponentInfo>>,
    /// Per-component topographies from the most recent `ica` node, when the
    /// node requested them (`topographies: true`) and/or supplied electrode
    /// `positions` for scalp-grid interpolation.
    #[serde(default)]
    pub ica_topographies: Option<Vec<crate::ica::IcaTopography>>,
}

pub fn detect_bad_channels(matrix: &[Vec<f64>], channel_names: &[String]) -> Vec<String> {
    let mut stds = Vec::new();
    let mut flats = Vec::new();
    for row in matrix {
        let finite: Vec<f64> = row.iter().copied().filter(|v| v.is_finite()).collect();
        if finite.len() < 4 {
            stds.push(0.0);
            flats.push(true);
            continue;
        }
        let mean = finite.iter().sum::<f64>() / finite.len() as f64;
        let variance =
            finite.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / finite.len() as f64;
        let flat = finite.iter().all(|v| (*v - finite[0]).abs() < 1e-12);
        stds.push(variance.sqrt());
        flats.push(flat);
    }
    let mut positive: Vec<f64> = stds.iter().copied().filter(|s| *s > 0.0).collect();
    positive.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = positive
        .get(positive.len().saturating_sub(1) / 2)
        .copied()
        .unwrap_or(0.0);
    stds.iter()
        .enumerate()
        .filter(|(index, std)| {
            flats[*index]
                || (**std == 0.0 && median > 0.0)
                || (median > 0.0 && **std > median * 8.0)
        })
        .map(|(index, _)| {
            channel_names
                .get(index)
                .cloned()
                .unwrap_or_else(|| format!("channel_{index}"))
        })
        .collect()
}

fn moving_average(data: &[f64], window: usize) -> Vec<f64> {
    let window = window.max(1);
    let half = window / 2;
    let mut out = Vec::with_capacity(data.len());
    let mut sum = 0.0;
    for (index, value) in data.iter().enumerate() {
        sum += value;
        if index >= window {
            sum -= data[index - window];
        }
        let lo = index.saturating_sub(half);
        let hi = (index + half + 1).min(data.len());
        let count = (hi - lo) as f64;
        out.push(sum / count.max(1.0));
    }
    out
}

pub fn bandpass_channel(data: &[f64], sfreq: f64, low: f64, high: f64) -> Vec<f64> {
    let low_pass = moving_average(data, (sfreq / low.max(0.5)).round() as usize);
    let high_source = moving_average(data, (sfreq / high.max(1.0)).round() as usize);
    data.iter()
        .enumerate()
        .map(|(i, v)| {
            let lp = low_pass.get(i).copied().unwrap_or(*v);
            let hp = high_source.get(i).copied().unwrap_or(*v);
            lp - hp
        })
        .collect()
}

/// Re-reference the `(channels, samples)` matrix. `average` subtracts the
/// mean across all channels at each sample; `mastoid` subtracts the mean of
/// channels whose names look like mastoid/ear references (m1, m2, a1, a2,
/// mastoid*). Returns an error when a named reference set is empty.
pub fn rereference(
    matrix: &mut [Vec<f64>],
    channel_names: &[String],
    method: &str,
) -> Result<(), CoreError> {
    if matrix.is_empty() || matrix[0].is_empty() {
        return Err(CoreError::InvalidInput("recording is empty".into()));
    }
    let n_samples = matrix[0].len();
    let use_median = matches!(method, "median" | "robust" | "robust_average");
    let mut indices: Vec<usize> = match method {
        "average" | "avg" | "common_average" | "car" | "median" | "robust" | "robust_average" => {
            (0..matrix.len()).collect()
        }
        "mastoid" | "mastoids" | "linked_mastoids" | "ear" => channel_names
            .iter()
            .enumerate()
            .filter(|(_, name)| {
                let lower = name.to_lowercase();
                matches!(lower.as_str(), "m1" | "m2" | "a1" | "a2" | "le" | "re")
                    || lower.contains("mastoid")
                    || lower.contains("mastoide")
            })
            .map(|(index, _)| index)
            .collect(),
        other => {
            return Err(CoreError::InvalidInput(format!(
                "unsupported re-reference method: {other}"
            )))
        }
    };
    if indices.is_empty() {
        return Err(CoreError::InvalidInput(
            "no mastoid/ear reference channels found".into(),
        ));
    }
    // Iterated robust average: after a first median-referenced pass,
    // channels whose RMS exceeds 3x the median channel RMS are treated as
    // artifacts and excluded from the reference set for a second pass.
    // Deterministic: at most one rejection round, ties keep file order.
    // Plain "median" stays a pure median reference; only "robust*" iterates.
    let iterated = matches!(method, "robust" | "robust_average");
    if iterated && indices.len() > 2 {
        let mut probe = matrix.to_vec();
        subtract_reference(&mut probe, &indices, true, n_samples);
        let mut rms: Vec<(f64, usize)> = indices
            .iter()
            .map(|&index| {
                let power = probe[index].iter().map(|value| value * value).sum::<f64>()
                    / n_samples.max(1) as f64;
                (power.sqrt(), index)
            })
            .collect();
        rms.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        let median_rms = rms[rms.len() / 2].0;
        if median_rms > 0.0 {
            let kept: Vec<usize> = rms
                .iter()
                .filter(|(rms_value, _)| *rms_value <= 3.0 * median_rms)
                .map(|&(_, index)| index)
                .collect();
            if !kept.is_empty() && kept.len() < indices.len() {
                indices = kept;
            }
        }
    }
    subtract_reference(matrix, &indices, use_median, n_samples);
    Ok(())
}

fn subtract_reference(
    matrix: &mut [Vec<f64>],
    indices: &[usize],
    use_median: bool,
    n_samples: usize,
) {
    let mut scratch: Vec<f64> = Vec::with_capacity(indices.len());
    for sample in 0..n_samples {
        scratch.clear();
        scratch.extend(
            indices
                .iter()
                .map(|&index| matrix[index].get(sample).copied().unwrap_or(0.0)),
        );
        let reference = if use_median {
            scratch.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mid = scratch.len() / 2;
            if scratch.len() % 2 == 1 {
                scratch[mid]
            } else {
                (scratch[mid - 1] + scratch[mid]) / 2.0
            }
        } else {
            scratch.iter().sum::<f64>() / scratch.len() as f64
        };
        for row in matrix.iter_mut() {
            if let Some(value) = row.get_mut(sample) {
                *value -= reference;
            }
        }
    }
}

/// Surface Laplacian re-referencing: replace each channel by its value
/// minus the mean of its `k` nearest neighbours on the scalp. `positions`
/// must hold one finite `[x, y, z]` triplet per channel (any consistent
/// unit/frame). Channels without usable positions are an error — silently
/// skipping them would corrupt the spatial operator.
pub fn surface_laplacian(matrix: &mut [Vec<f64>], positions: &[[f64; 3]]) -> Result<(), CoreError> {
    let n_channels = matrix.len();
    if n_channels == 0 || positions.len() != n_channels {
        return Err(CoreError::InvalidInput(
            "laplacian requires one position per channel".into(),
        ));
    }
    if n_channels < 2 {
        return Ok(());
    }
    let k = 6usize.min(n_channels - 1);
    let n_samples = matrix[0].len();
    // Nearest-neighbour lists by Euclidean distance.
    let mut neighbours: Vec<Vec<usize>> = Vec::with_capacity(n_channels);
    for i in 0..n_channels {
        let mut dists: Vec<(f64, usize)> = (0..n_channels)
            .filter(|&j| j != i)
            .map(|j| {
                let d = (0..3)
                    .map(|axis| (positions[i][axis] - positions[j][axis]).powi(2))
                    .sum::<f64>();
                (d, j)
            })
            .collect();
        dists.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        neighbours.push(dists[..k].iter().map(|&(_, j)| j).collect());
    }
    let original = matrix.to_vec();
    for i in 0..n_channels {
        for sample in 0..n_samples {
            let neighbour_mean = neighbours[i]
                .iter()
                .map(|&j| original[j].get(sample).copied().unwrap_or(0.0))
                .sum::<f64>()
                / neighbours[i].len() as f64;
            if let Some(value) = matrix[i].get_mut(sample) {
                *value -= neighbour_mean;
            }
        }
    }
    Ok(())
}

/// Continuous-signal baseline correction: subtract the per-channel mean
/// over `[start_s, end_s]` from every sample.
pub fn baseline_correct(
    matrix: &mut [Vec<f64>],
    sampling_frequency_hz: f64,
    start_s: f64,
    end_s: f64,
) -> Result<(), CoreError> {
    if !(start_s < end_s) || !start_s.is_finite() || !end_s.is_finite() {
        return Err(CoreError::InvalidInput("invalid baseline window".into()));
    }
    if matrix.is_empty() {
        return Ok(());
    }
    let n_samples = matrix[0].len();
    let lo = ((start_s * sampling_frequency_hz).round() as usize).min(n_samples);
    let hi = ((end_s * sampling_frequency_hz).round() as usize)
        .min(n_samples)
        .max(lo);
    if hi == lo {
        return Err(CoreError::InvalidInput("baseline window is empty".into()));
    }
    for row in matrix.iter_mut() {
        let mean = row[lo..hi].iter().sum::<f64>() / (hi - lo) as f64;
        for value in row.iter_mut() {
            *value -= mean;
        }
    }
    Ok(())
}

/// Signal-space projection: estimate the dominant noise subspace from the
/// given sample indices and project it out of every channel.
pub fn ssp_project(
    matrix: &[Vec<f64>],
    noise_samples: &[usize],
    n_components: usize,
) -> Result<Vec<Vec<f64>>, CoreError> {
    let n_channels = matrix.len();
    if n_channels < 2 {
        return Err(CoreError::InvalidInput(
            "SSP requires at least 2 channels".into(),
        ));
    }
    let n_samples = matrix.first().map_or(0, Vec::len);
    if noise_samples.is_empty() {
        return Err(CoreError::InvalidInput("ssp noise window is empty".into()));
    }
    if noise_samples.iter().any(|&s| s >= n_samples) {
        return Err(CoreError::InvalidInput(
            "ssp noise sample out of range".into(),
        ));
    }
    let mut cov = vec![vec![0.0f64; n_channels]; n_channels];
    for &sample in noise_samples {
        for i in 0..n_channels {
            let xi = matrix[i][sample];
            for j in 0..=i {
                cov[i][j] += xi * matrix[j][sample];
            }
        }
    }
    for i in 0..n_channels {
        for j in 0..=i {
            cov[i][j] /= noise_samples.len() as f64;
            cov[j][i] = cov[i][j];
        }
    }
    let (_evals, eigenvectors) = crate::ica::jacobi_eigen(&cov);
    let k = n_components.clamp(1, 16).min(n_channels);
    let mut current = matrix.to_vec();
    for sample in 0..n_samples {
        for j in 0..k {
            let mut acc = 0.0;
            for i in 0..n_channels {
                acc += eigenvectors[i][j] * current[i][sample];
            }
            for i in 0..n_channels {
                current[i][sample] -= eigenvectors[i][j] * acc;
            }
        }
    }
    Ok(current)
}

/// Extract epochs around events, optionally baseline-correct and reject
/// artifacts by peak-to-peak amplitude, then average into an ERP.
/// `tmin_s`/`tmax_s` are relative to each event onset. `baseline` (default
/// true) subtracts the mean over `[tmin_s, 0)` per channel; an explicit
/// `baseline_s: [start, end]` pair overrides the window. `reject_ptp`
/// (in data units) rejects an epoch when any channel's peak-to-peak
/// amplitude exceeds it. `reject_abs` (in data units) rejects an epoch
/// when any sample's absolute value exceeds it. `event` filters events by
/// label substring.
#[allow(clippy::too_many_arguments)]
pub fn epoch_epochs(
    matrix: &[Vec<f64>],
    sampling_frequency_hz: f64,
    events: &[EegEvent],
    tmin_s: f64,
    tmax_s: f64,
    event_filter: Option<&str>,
    baseline: bool,
    baseline_window: Option<(f64, f64)>,
    reject_ptp: Option<f64>,
    reject_abs: Option<f64>,
    max_epochs: usize,
) -> Result<(Vec<Vec<f64>>, usize, usize), CoreError> {
    if matrix.is_empty() || matrix[0].is_empty() {
        return Err(CoreError::InvalidInput("recording is empty".into()));
    }
    if !(tmin_s < tmax_s) || !tmin_s.is_finite() || !tmax_s.is_finite() {
        return Err(CoreError::InvalidInput("invalid epoch window".into()));
    }
    let n_samples = matrix[0].len();
    let pre = (tmin_s * sampling_frequency_hz).round() as isize;
    let post = (tmax_s * sampling_frequency_hz).round() as isize;
    let epoch_len = (post - pre).max(1) as usize;
    if epoch_len > MAX_EPOCH_SAMPLES {
        return Err(CoreError::InvalidInput("epoch window too long".into()));
    }
    let baseline_range = if baseline {
        baseline_window.unwrap_or((tmin_s, 0.0))
    } else {
        (f64::INFINITY, f64::NEG_INFINITY)
    };
    let b_lo = ((baseline_range.0 * sampling_frequency_hz).round() as isize - pre).max(0);
    let b_hi = ((baseline_range.1 * sampling_frequency_hz).round() as isize - pre)
        .min(epoch_len as isize)
        .max(0) as usize;
    let baseline_len = b_hi.saturating_sub(b_lo as usize);
    let use_baseline = baseline && baseline_len > 0 && b_lo < epoch_len as isize;

    let filter = event_filter.map(|value| value.to_lowercase());
    let mut selected: Vec<&EegEvent> = events
        .iter()
        .filter(|event| {
            filter
                .as_ref()
                .is_none_or(|needle| event.label.to_lowercase().contains(needle))
        })
        .collect();
    selected.sort_by_key(|event| event.sample);
    selected.truncate(max_epochs.min(MAX_EPOCHS));
    if selected.is_empty() {
        return Err(CoreError::InvalidInput(
            "no events match the epoch filter".into(),
        ));
    }

    let mut erp = vec![vec![0.0f64; epoch_len]; matrix.len()];
    let mut accepted = 0usize;
    let mut rejected = 0usize;
    'events: for event in selected {
        let start = event.sample as isize + pre;
        if start < 0 || start + epoch_len as isize > n_samples as isize {
            continue;
        }
        let start = start as usize;
        // Artifact rejection: any channel peak-to-peak above threshold.
        if let Some(threshold) = reject_ptp {
            for row in matrix {
                let segment = &row[start..start + epoch_len];
                let mut lo = f64::INFINITY;
                let mut hi = f64::NEG_INFINITY;
                for &v in segment {
                    if v.is_finite() {
                        lo = lo.min(v);
                        hi = hi.max(v);
                    }
                }
                if hi - lo > threshold {
                    rejected += 1;
                    continue 'events;
                }
            }
        }
        // Artifact rejection: any sample's absolute value above threshold.
        if let Some(threshold) = reject_abs {
            for row in matrix {
                if row[start..start + epoch_len]
                    .iter()
                    .any(|&v| v.is_finite() && v.abs() > threshold)
                {
                    rejected += 1;
                    continue 'events;
                }
            }
        }
        for (channel, row) in matrix.iter().enumerate() {
            let segment = &row[start..start + epoch_len];
            let offset = if use_baseline {
                let b0 = b_lo as usize;
                let b1 = b_hi.min(epoch_len);
                if b1 > b0 {
                    segment[b0..b1].iter().sum::<f64>() / (b1 - b0) as f64
                } else {
                    0.0
                }
            } else {
                0.0
            };
            for (t, &value) in segment.iter().enumerate() {
                erp[channel][t] += value - offset;
            }
        }
        accepted += 1;
    }
    if accepted == 0 {
        return Err(CoreError::InvalidInput(
            "all epochs were rejected or out of bounds".into(),
        ));
    }
    for row in &mut erp {
        for value in row.iter_mut() {
            *value /= accepted as f64;
        }
    }
    Ok((erp, accepted, rejected))
}

fn welch_band_power(data: &[f64], sfreq: f64, n_fft: usize) -> BTreeMap<String, f64> {
    use rustfft::num_complex::Complex;
    let mut bands = BTreeMap::new();
    let ranges: [(&str, f64, f64); 5] = [
        ("delta", 0.5, 4.0),
        ("theta", 4.0, 8.0),
        ("alpha", 8.0, 13.0),
        ("beta", 13.0, 30.0),
        ("gamma", 30.0, 80.0),
    ];
    let mut band_power = vec![0.0f64; ranges.len()];
    let mut band_count = vec![0usize; ranges.len()];
    let segment = n_fft.min(data.len()).max(16);
    let hop = segment / 2;
    let window: Vec<f64> = (0..segment)
        .map(|i| 0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / (segment - 1) as f64).cos())
        .collect();
    let plan = rustfft::FftPlanner::<f64>::new().plan_fft_forward(segment);
    let mut buffer = vec![Complex { re: 0.0, im: 0.0 }; segment];
    let mut offset = 0usize;
    while offset + segment <= data.len() {
        let mut mean = 0.0;
        for i in 0..segment {
            mean += data[offset + i];
        }
        mean /= segment as f64;
        for i in 0..segment {
            buffer[i] = Complex {
                re: (data[offset + i] - mean) * window[i],
                im: 0.0,
            };
        }
        plan.process(&mut buffer);
        for bin in 1..segment / 2 {
            let freq = bin as f64 * sfreq / segment as f64;
            let power = (buffer[bin].norm_sqr()) / (segment as f64 * segment as f64);
            for (index, (_, low, high)) in ranges.iter().enumerate() {
                if freq >= *low && freq < *high {
                    band_power[index] += power;
                    band_count[index] += 1;
                    break;
                }
            }
        }
        offset += hop;
    }
    for (index, (name, _, _)) in ranges.iter().enumerate() {
        if band_count[index] > 0 {
            bands.insert(
                name.to_string(),
                band_power[index] / band_count[index] as f64,
            );
        }
    }
    bands
}

pub fn run_pipeline(
    definition: &Value,
    matrix: &[Vec<f64>],
    sampling_frequency_hz: f64,
    channel_names: &[String],
    events: &[EegEvent],
    cancel: Option<&dyn Fn() -> bool>,
    progress: Option<&dyn Fn(f64, &str)>,
) -> Result<EegPipelineResult, CoreError> {
    let nodes = definition
        .get("nodes")
        .or_else(|| definition.get("steps"))
        .and_then(Value::as_array)
        .ok_or_else(|| CoreError::InvalidInput("pipeline has no nodes".into()))?;
    if matrix.is_empty() || matrix[0].is_empty() {
        return Err(CoreError::InvalidInput("recording is empty".into()));
    }
    let mut current: Vec<Vec<f64>> = matrix.to_vec();
    let mut ica_state: Option<crate::ica::IcaModel> = None;
    let mut ica_components: Option<Vec<crate::IcaComponentInfo>> = None;
    let mut ica_topographies: Option<Vec<crate::ica::IcaTopography>> = None;
    let mut band_powers = None;
    let mut bad_channels = Vec::new();
    let mut n_epochs = 0usize;
    let mut n_epochs_rejected = 0usize;
    let mut erp = None;
    let mut steps = Vec::new();
    let total = nodes.len().max(1) as f64;
    for (index, node) in nodes.iter().enumerate() {
        if cancel.is_some_and(|check| check()) {
            return Err(CoreError::Cancelled);
        }
        if let Some(report) = progress {
            report(index as f64 / total, "running pipeline node");
        }
        let node_type = node
            .get("type")
            .or_else(|| node.get("op"))
            .or_else(|| node.get("name"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let params = node.get("params").cloned().unwrap_or(Value::Null);
        match node_type {
            "filter" | "bandpass" => {
                let low = params
                    .get("l_freq_hz")
                    .and_then(Value::as_f64)
                    .unwrap_or(1.0);
                let high = params
                    .get("h_freq_hz")
                    .and_then(Value::as_f64)
                    .unwrap_or(40.0);
                if !(0.0 <= low && low < high) {
                    return Err(CoreError::InvalidInput("invalid bandpass range".into()));
                }
                if high > sampling_frequency_hz / 2.0 {
                    return Err(CoreError::InvalidInput(
                        "high frequency exceeds Nyquist".into(),
                    ));
                }
                for row in &mut current {
                    *row = bandpass_channel(row, sampling_frequency_hz, low, high);
                }
                steps.push(format!("filter[{low}-{high}Hz]"));
            }
            "psd" | "spectral" | "psd_welch" => {
                let fmin = params.get("fmin_hz").and_then(Value::as_f64).unwrap_or(0.5);
                let fmax = params
                    .get("fmax_hz")
                    .and_then(Value::as_f64)
                    .unwrap_or(80.0);
                if !(0.0 <= fmin && fmin < fmax) {
                    return Err(CoreError::InvalidInput(
                        "psd requires 0 <= fmin_hz < fmax_hz".into(),
                    ));
                }
                let n_fft = params
                    .get("n_fft")
                    .and_then(Value::as_u64)
                    .map(|v| v as usize)
                    .unwrap_or(256)
                    .clamp(16, 8192);
                let mut bands = BTreeMap::new();
                for row in &current {
                    for (name, power) in welch_band_power(row, sampling_frequency_hz, n_fft) {
                        *bands.entry(name).or_insert(0.0) += power;
                    }
                }
                let count = current.len() as f64;
                band_powers = Some(
                    bands
                        .into_iter()
                        .map(|(name, total)| (name, total / count))
                        .collect(),
                );
                steps.push(format!("psd[n_fft={n_fft}]"));
            }
            "bad_channels" | "bad-channel-detection" => {
                bad_channels = detect_bad_channels(&current, channel_names);
                steps.push("bad_channels".to_string());
            }
            "reref" | "re_reference" | "rereference" => {
                let method = params
                    .get("method")
                    .or_else(|| params.get("reference"))
                    .and_then(Value::as_str)
                    .unwrap_or("average");
                if method == "laplacian" {
                    let positions: Vec<[f64; 3]> = params
                        .get("positions")
                        .and_then(Value::as_array)
                        .ok_or_else(|| {
                            CoreError::InvalidInput("laplacian requires a positions array".into())
                        })?
                        .iter()
                        .map(|entry| {
                            let values: Vec<f64> = entry
                                .as_array()
                                .map(|items| items.iter().filter_map(Value::as_f64).collect())
                                .unwrap_or_default();
                            if values.len() != 3 || values.iter().any(|v| !v.is_finite()) {
                                return Err(CoreError::InvalidInput(
                                    "positions entries must be finite [x, y, z]".into(),
                                ));
                            }
                            Ok([values[0], values[1], values[2]])
                        })
                        .collect::<Result<_, CoreError>>()?;
                    surface_laplacian(&mut current, &positions)?;
                    steps.push("reref[laplacian]".to_string());
                } else {
                    rereference(&mut current, channel_names, method)?;
                    steps.push(format!("reref[{method}]"));
                }
            }
            "baseline" => {
                let start = params
                    .get("start_s")
                    .or_else(|| params.get("baseline_s").and_then(|v| v.get(0)))
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0);
                let end = params
                    .get("end_s")
                    .or_else(|| params.get("baseline_s").and_then(|v| v.get(1)))
                    .and_then(Value::as_f64)
                    .unwrap_or(start + 1.0);
                baseline_correct(&mut current, sampling_frequency_hz, start, end)?;
                steps.push(format!("baseline[{start}-{end}s]"));
            }
            "epoch" | "epoching" | "erp" => {
                let tmin = params
                    .get("tmin_s")
                    .or_else(|| params.get("tmin"))
                    .and_then(Value::as_f64)
                    .unwrap_or(-0.2);
                let tmax = params
                    .get("tmax_s")
                    .or_else(|| params.get("tmax"))
                    .and_then(Value::as_f64)
                    .unwrap_or(0.5);
                let event_filter = params.get("event").and_then(Value::as_str);
                let baseline = params
                    .get("baseline")
                    .and_then(Value::as_bool)
                    .unwrap_or(true);
                let baseline_window = params
                    .get("baseline_s")
                    .and_then(Value::as_array)
                    .and_then(|pair| Some((pair.first()?.as_f64()?, pair.get(1)?.as_f64()?)));
                let reject_ptp = params.get("reject_ptp").and_then(Value::as_f64);
                let reject_abs = params
                    .get("reject_abs")
                    .or_else(|| params.get("reject_amplitude"))
                    .and_then(Value::as_f64);
                let max_epochs = params
                    .get("max_epochs")
                    .and_then(Value::as_u64)
                    .unwrap_or(10_000) as usize;
                let (erp_matrix, used, rejected) = epoch_epochs(
                    &current,
                    sampling_frequency_hz,
                    events,
                    tmin,
                    tmax,
                    event_filter,
                    baseline,
                    baseline_window,
                    reject_ptp,
                    reject_abs,
                    max_epochs,
                )?;
                n_epochs = used;
                n_epochs_rejected = rejected;
                erp = Some(erp_matrix);
                steps.push(format!("epoch[n={used} rejected={rejected}]"));
            }
            "ica" | "fastica" => {
                let n_components = params
                    .get("n_components")
                    .and_then(Value::as_u64)
                    .map(|v| v as usize);
                let max_iter = params
                    .get("max_iter")
                    .and_then(Value::as_u64)
                    .unwrap_or(200) as usize;
                let tol = params.get("tol").and_then(Value::as_f64).unwrap_or(1e-4);
                let model = crate::ica::fastica(&current, n_components, max_iter, tol)?;
                let n_comp = model.n_components;
                let variances: Vec<f64> = model
                    .sources
                    .iter()
                    .map(|source| {
                        let mean = source.iter().sum::<f64>() / source.len().max(1) as f64;
                        source.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
                            / source.len().max(1) as f64
                    })
                    .collect();
                let total: f64 = variances.iter().sum();
                ica_components = Some(
                    variances
                        .iter()
                        .enumerate()
                        .map(|(index, &variance)| crate::IcaComponentInfo {
                            index,
                            variance,
                            variance_fraction: if total > 0.0 { variance / total } else { 0.0 },
                        })
                        .collect(),
                );
                // Topographies: mixing matrix A = unwhitening * unmixing,
                // column j is component j's scalp distribution.
                let want_topo = params
                    .get("topographies")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                    || params.get("positions").is_some();
                if want_topo {
                    let n_in = current.len();
                    let positions: Option<Vec<[f64; 3]>> = params
                        .get("positions")
                        .and_then(Value::as_array)
                        .map(|items| {
                            items
                                .iter()
                                .map(|item| {
                                    let triple: Vec<f64> = item
                                        .as_array()
                                        .map(|values| {
                                            values.iter().filter_map(Value::as_f64).collect()
                                        })
                                        .unwrap_or_default();
                                    if triple.len() >= 3 {
                                        Ok([triple[0], triple[1], triple[2]])
                                    } else {
                                        Err(CoreError::InvalidInput(
                                            "ICA positions must be [x,y,z] triples".into(),
                                        ))
                                    }
                                })
                                .collect::<Result<Vec<_>, _>>()
                        })
                        .transpose()?;
                    if let Some(pos) = positions.as_ref() {
                        if pos.len() != n_in {
                            return Err(CoreError::InvalidInput(format!(
                                "ICA positions count {} does not match channel count {n_in}",
                                pos.len()
                            )));
                        }
                    }
                    let mut topographies = Vec::with_capacity(n_comp);
                    for component in 0..n_comp {
                        let mut weights: Vec<f64> = (0..n_in)
                            .map(|ch| {
                                (0..n_comp)
                                    .map(|k| {
                                        model.unwhitening[ch][k] * model.unmixing[k][component]
                                    })
                                    .sum()
                            })
                            .collect();
                        let norm = weights.iter().map(|v| v * v).sum::<f64>().sqrt();
                        if norm > 0.0 {
                            for v in &mut weights {
                                *v /= norm;
                            }
                        }
                        let (grid, grid_cols, grid_rows) = match positions.as_ref() {
                            Some(pos) => {
                                let (grid, cols, rows) = scalp_grid(&weights, pos)?;
                                (Some(grid), cols, rows)
                            }
                            None => (None, 0, 0),
                        };
                        topographies.push(crate::ica::IcaTopography {
                            component,
                            channel_weights: weights,
                            grid,
                            grid_cols,
                            grid_rows,
                        });
                    }
                    ica_topographies = Some(topographies);
                }
                current = model.sources.clone();
                ica_state = Some(model);
                steps.push(format!("ica[n={n_comp}]"));
            }
            "ica_remove" | "remove_components" | "ica_reconstruct" => {
                let model = ica_state.as_ref().ok_or_else(|| {
                    CoreError::InvalidInput(
                        "component removal requires a preceding ica node".into(),
                    )
                })?;
                let excluded: Vec<usize> = params
                    .get("components")
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(Value::as_u64)
                            .map(|v| v as usize)
                            .collect()
                    })
                    .unwrap_or_default();
                for &index in &excluded {
                    if index >= model.n_components {
                        return Err(CoreError::InvalidInput(format!(
                            "ICA component {index} out of range ({} components)",
                            model.n_components
                        )));
                    }
                }
                current = crate::ica::reconstruct(model, &excluded);
                steps.push(format!("ica_remove[{}]", excluded.len()));
            }
            "ssp" | "signal_space_projection" => {
                let n_channels = current.len();
                let n_samples = current.first().map_or(0, Vec::len);
                if n_channels < 2 {
                    return Err(CoreError::InvalidInput(
                        "SSP requires at least 2 channels".into(),
                    ));
                }
                let mut noise_samples: Vec<usize> = Vec::new();
                let start_sample = params.get("start_sample").and_then(Value::as_u64);
                let end_sample = params.get("end_sample").and_then(Value::as_u64);
                let start_s = params.get("start_s").and_then(Value::as_f64);
                let end_s = params.get("end_s").and_then(Value::as_f64);
                if let (Some(start), Some(end)) = (start_sample, end_sample) {
                    noise_samples.extend(start as usize..(end as usize).min(n_samples));
                } else if let (Some(start), Some(end)) = (start_s, end_s) {
                    let start_idx = ((start * sampling_frequency_hz).round().max(0.0)) as usize;
                    let end_idx = ((end * sampling_frequency_hz).round() as usize).min(n_samples);
                    if start_idx < end_idx {
                        noise_samples.extend(start_idx..end_idx);
                    }
                } else {
                    let event_filter = params
                        .get("event")
                        .or_else(|| params.get("event_code"))
                        .and_then(Value::as_str)
                        .map(str::to_lowercase);
                    let tmin = params.get("tmin_s").and_then(Value::as_f64).unwrap_or(0.0);
                    let tmax = params.get("tmax_s").and_then(Value::as_f64).unwrap_or(0.5);
                    let selected: Vec<&EegEvent> = events
                        .iter()
                        .filter(|event| match &event_filter {
                            Some(filter) => event.label.to_lowercase().contains(filter.as_str()),
                            None => true,
                        })
                        .collect();
                    if selected.is_empty() {
                        return Err(CoreError::InvalidInput(
                            "ssp found no matching events for the noise window".into(),
                        ));
                    }
                    for event in selected {
                        let a = (event.sample as isize
                            + (tmin * sampling_frequency_hz).round() as isize)
                            .max(0) as usize;
                        let b = (event.sample as isize
                            + (tmax * sampling_frequency_hz).round() as isize)
                            .max(0) as usize;
                        let b = b.min(n_samples);
                        if a < b {
                            noise_samples.extend(a..b);
                        }
                    }
                }
                noise_samples.sort_unstable();
                noise_samples.dedup();
                noise_samples.truncate(2_000_000);
                if noise_samples.len() < n_channels {
                    return Err(CoreError::InvalidInput(
                        "ssp noise window has fewer samples than channels".into(),
                    ));
                }
                let n_components = params
                    .get("n_components")
                    .or_else(|| params.get("n_vectors"))
                    .and_then(Value::as_u64)
                    .unwrap_or(1)
                    .clamp(1, 16) as usize;
                current = ssp_project(&current, &noise_samples, n_components)?;
                steps.push(format!(
                    "ssp[k={},n={}]",
                    n_components.min(n_channels),
                    noise_samples.len()
                ));
            }
            other => {
                return Err(CoreError::InvalidInput(format!(
                    "unsupported EEG pipeline node: {other}"
                )))
            }
        }
    }
    Ok(EegPipelineResult {
        steps,
        band_powers,
        bad_channels,
        n_channels: current.len(),
        n_samples: current.first().map(Vec::len).unwrap_or(0),
        n_epochs,
        n_epochs_rejected,
        erp,
        ica_components,
        ica_topographies,
    })
}

/// Interpolate per-electrode weights onto a scalp grid. Positions are
/// projected onto the unit sphere; each grid cell (azimuth × elevation)
/// receives an inverse-distance-weighted average of channel weights.
/// Returns `(grid row-major, cols, rows)`.
pub fn scalp_grid(
    weights: &[f64],
    positions: &[[f64; 3]],
) -> Result<(Vec<f64>, usize, usize), CoreError> {
    if weights.len() != positions.len() {
        return Err(CoreError::InvalidInput(
            "scalp grid requires one position per channel weight".into(),
        ));
    }
    const AZ_BINS: usize = 72;
    const EL_BINS: usize = 36;
    // Unit-sphere directions for each electrode.
    let dirs: Vec<[f64; 3]> = positions
        .iter()
        .map(|p| {
            let norm = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            if norm > 1e-12 {
                [p[0] / norm, p[1] / norm, p[2] / norm]
            } else {
                [0.0, 0.0, 1.0]
            }
        })
        .collect();
    let mut grid = vec![0.0f64; AZ_BINS * EL_BINS];
    for row in 0..EL_BINS {
        // Elevation from -90° (bottom) to +90° (top).
        let el = -std::f64::consts::FRAC_PI_2
            + (row as f64 + 0.5) / EL_BINS as f64 * std::f64::consts::PI;
        let cos_el = el.cos();
        for col in 0..AZ_BINS {
            let az = -std::f64::consts::PI
                + (col as f64 + 0.5) / AZ_BINS as f64 * 2.0 * std::f64::consts::PI;
            let target = [az.cos() * cos_el, az.sin() * cos_el, el.sin()];
            let mut num = 0.0;
            let mut den = 0.0;
            for (dir, &w) in dirs.iter().zip(weights.iter()) {
                let dot =
                    (dir[0] * target[0] + dir[1] * target[1] + dir[2] * target[2]).clamp(-1.0, 1.0);
                let dist = (2.0 * (1.0 - dot)).sqrt(); // chord distance
                let inv = 1.0 / (dist * dist + 1e-4);
                num += w * inv;
                den += inv;
            }
            grid[row * AZ_BINS + col] = if den > 0.0 { num / den } else { 0.0 };
        }
    }
    Ok((grid, AZ_BINS, EL_BINS))
}
