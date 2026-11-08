use crate::{MegMetadata, MegWindow};
use neuro_core::CoreError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Epoch {
    pub event_sample: usize,
    pub event_code: Option<String>,
    pub times_s: Vec<f64>,
    pub data: Vec<Vec<f64>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvokedResponse {
    pub n_epochs: usize,
    pub channels: Vec<String>,
    pub times_s: Vec<f64>,
    pub data: Vec<Vec<f64>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EpochSummary {
    pub n_epochs: usize,
    pub n_channels: usize,
    pub tmin_s: f64,
    pub tmax_s: f64,
    pub skipped_events: usize,
    pub event_codes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PipelineResult {
    pub steps: Vec<String>,
    pub epochs: Option<Vec<Epoch>>,
    pub epoch_summary: Option<EpochSummary>,
    pub evoked: Option<EvokedResponse>,
    pub band_powers: Option<BTreeMap<String, f64>>,
    pub bad_channels: Vec<String>,
    pub window: Option<MegWindow>,
    /// Per-component variance summary from the most recent `ica` node.
    #[serde(default)]
    pub ica_components: Option<Vec<neuro_core::ica::IcaComponentInfo>>,
}

pub fn detect_bad_channels(window: &MegWindow, channel_names: &[String]) -> Vec<String> {
    let mut stats: Vec<(usize, f64, bool)> = Vec::new();
    for (index, channel) in window.data.iter().enumerate() {
        let finite: Vec<f64> = channel.iter().copied().filter(|v| v.is_finite()).collect();
        if finite.len() < 4 {
            stats.push((index, 0.0, true));
            continue;
        }
        let mean = finite.iter().sum::<f64>() / finite.len() as f64;
        let variance =
            finite.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / finite.len() as f64;
        let std = variance.sqrt();
        let flat = finite.iter().all(|v| (*v - finite[0]).abs() < 1e-12);
        stats.push((index, std, flat));
    }
    let mut stds: Vec<f64> = stats
        .iter()
        .map(|(_, std, _)| *std)
        .filter(|s| *s > 0.0)
        .collect();
    stds.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = stds
        .get(stds.len().saturating_sub(1) / 2)
        .copied()
        .unwrap_or(0.0);
    stats
        .into_iter()
        .filter(|(_, std, flat)| {
            *flat || (*std == 0.0 && median > 0.0) || (median > 0.0 && *std > median * 8.0)
        })
        .map(|(index, _, _)| {
            channel_names
                .get(index)
                .cloned()
                .unwrap_or_else(|| format!("channel_{index}"))
        })
        .collect()
}

pub fn bandpass(window: &MegWindow, low_hz: f64, high_hz: f64) -> Result<MegWindow, CoreError> {
    if !(0.0 <= low_hz && low_hz < high_hz) {
        return Err(CoreError::InvalidInput("invalid bandpass range".into()));
    }
    let nyquist = window.effective_sampling_frequency_hz / 2.0;
    if high_hz > nyquist {
        return Err(CoreError::InvalidInput(
            "high frequency exceeds Nyquist".into(),
        ));
    }
    let mut output = window.clone();
    for channel in &mut output.data {
        *channel = simple_bandpass(
            channel,
            window.effective_sampling_frequency_hz,
            low_hz,
            high_hz,
        );
    }
    Ok(output)
}

fn simple_bandpass(data: &[f64], sfreq: f64, low: f64, high: f64) -> Vec<f64> {
    // Two-pole style smoothing via moving-average low-pass subtraction is
    // intentionally simple and deterministic; adequate for bounded previews.
    let low_pass = moving_average(data, (sfreq / low.max(0.5)).round() as usize);
    let high_pass_source = moving_average(data, (sfreq / high.max(1.0)).round() as usize);
    data.iter()
        .enumerate()
        .map(|(i, v)| {
            let lp = low_pass.get(i).copied().unwrap_or(*v);
            let hp = high_pass_source.get(i).copied().unwrap_or(*v);
            lp - hp
        })
        .collect()
}

fn moving_average(data: &[f64], window: usize) -> Vec<f64> {
    if window <= 1 || data.is_empty() {
        return data.to_vec();
    }
    let half = window / 2;
    let mut out = Vec::with_capacity(data.len());
    let mut sum = 0.0;
    let mut count = 0usize;
    for i in 0..data.len() {
        let add = i + half;
        if add < data.len() {
            sum += data[add];
            count += 1;
        }
        let remove = i as isize - half as isize - 1;
        if remove >= 0 {
            sum -= data[remove as usize];
            count -= 1;
        }
        out.push(if count > 0 {
            sum / count as f64
        } else {
            data[i]
        });
    }
    out
}

/// Row indices of MEG-kind channels in the window, matched by name
/// against `metadata.channels`. Falls back to all rows when the metadata
/// carries no MEG channels (e.g. synthetic/test windows).
fn meg_channel_indices(channel_names: &[String], metadata: &MegMetadata) -> Vec<usize> {
    let indices: Vec<usize> = channel_names
        .iter()
        .enumerate()
        .filter(|(_, name)| {
            metadata
                .channels
                .iter()
                .any(|channel| channel.name == **name && channel.kind == "meg")
        })
        .map(|(index, _)| index)
        .collect();
    if indices.is_empty() {
        (0..channel_names.len()).collect()
    } else {
        indices
    }
}

/// First-order synthetic gradiometer estimate: for each MEG channel with
/// coil positions, replace its signal by the finite difference against its
/// nearest MEG neighbour divided by their separation —
/// `(x_i - x_j) / |pos_i - pos_j|` in T/m. This approximates the local
/// field gradient along the inter-sensor axis; it is a noise-rejection
/// estimate, not a full SSS-style synthetic gradiometer. Channels without
/// positions or a neighbour within `[min_mm, max_mm]` pass through
/// unchanged.
pub fn synthetic_gradiometers(
    window: &mut MegWindow,
    channel_names: &[String],
    metadata: &crate::MegMetadata,
    min_mm: f64,
    max_mm: f64,
) -> Result<usize, CoreError> {
    if !(0.0 <= min_mm && min_mm < max_mm) {
        return Err(CoreError::InvalidInput(
            "synthetic gradiometer requires 0 <= min_distance_mm < max_distance_mm".into(),
        ));
    }
    let positions: Vec<Option<Vec<f64>>> = channel_names
        .iter()
        .map(|name| {
            metadata
                .channels
                .iter()
                .find(|channel| channel.name == *name && channel.kind == "meg")
                .and_then(|channel| channel.position_mm.clone())
        })
        .collect();
    let n_samples = window
        .times_s
        .len()
        .min(window.data.first().map(Vec::len).unwrap_or(0));
    if n_samples == 0 {
        return Ok(0);
    }
    let original = window.data.clone();
    let mut replaced = 0usize;
    for i in 0..window.data.len().min(channel_names.len()) {
        let Some(pos_i) = &positions[i] else { continue };
        if pos_i.len() != 3 {
            continue;
        }
        let mut best: Option<(f64, usize)> = None;
        for j in 0..positions.len().min(original.len()) {
            if j == i {
                continue;
            }
            let Some(pos_j) = &positions[j] else { continue };
            if pos_j.len() != 3 {
                continue;
            }
            let dist_mm = (0..3)
                .map(|axis| (pos_i[axis] - pos_j[axis]).powi(2))
                .sum::<f64>()
                .sqrt();
            if dist_mm >= min_mm
                && dist_mm <= max_mm
                && best.is_none_or(|(best_d, _)| dist_mm < best_d)
            {
                best = Some((dist_mm, j));
            }
        }
        let Some((dist_mm, j)) = best else { continue };
        let scale = 1000.0 / dist_mm; // per-metre gradient
        for sample in 0..n_samples {
            window.data[i][sample] = (original[i][sample] - original[j][sample]) * scale;
        }
        replaced += 1;
    }
    Ok(replaced)
}

pub fn epoch_window(
    window: &MegWindow,
    events: &[crate::MegEvent],
    sfreq: f64,
    tmin_s: f64,
    tmax_s: f64,
) -> Result<(Vec<Epoch>, usize), CoreError> {
    if tmin_s >= tmax_s {
        return Err(CoreError::InvalidInput(
            "tmin must be less than tmax".into(),
        ));
    }
    if window.times_s.is_empty() || window.data.is_empty() {
        return Err(CoreError::InvalidInput("window is empty".into()));
    }
    let eff_sfreq = window.effective_sampling_frequency_hz;
    let t0 = window.times_s[0];
    let n_samples = window.times_s.len();
    let pre = ((-tmin_s) * eff_sfreq).round() as isize;
    let post = (tmax_s * eff_sfreq).round() as isize;
    let epoch_len = (pre + post + 1) as usize;
    let mut epochs = Vec::new();
    let mut skipped = 0usize;
    for event in events {
        let event_time = if event.time_s > 0.0 {
            event.time_s
        } else {
            event.sample as f64 / sfreq
        };
        let center = ((event_time - t0) * eff_sfreq).round() as isize;
        if center - pre < 0 || center + post >= n_samples as isize {
            skipped += 1;
            continue;
        }
        let center = center as usize;
        let mut times = Vec::with_capacity(epoch_len);
        let mut data = vec![Vec::with_capacity(epoch_len); window.data.len()];
        for offset in 0..epoch_len {
            let local = center as isize + offset as isize - pre;
            let sample = local as usize;
            times.push(local as f64 / eff_sfreq - (center as f64 / eff_sfreq));
            for (ch, channel) in window.data.iter().enumerate() {
                data[ch].push(channel[sample]);
            }
        }
        epochs.push(Epoch {
            event_sample: event.sample,
            event_code: event.code.clone(),
            times_s: times,
            data,
        });
    }
    if epochs.is_empty() {
        return Err(CoreError::InvalidInput(
            "no events fall inside the window".into(),
        ));
    }
    Ok((epochs, skipped))
}

pub fn evoked_average(
    epochs: &[Epoch],
    channel_names: &[String],
) -> Result<EvokedResponse, CoreError> {
    if epochs.is_empty() {
        return Err(CoreError::InvalidInput("no epochs to average".into()));
    }
    let channels = epochs[0].data.len();
    let len = epochs[0].times_s.len();
    let mut data = vec![vec![0.0; len]; channels];
    for epoch in epochs {
        for ch in 0..channels {
            for t in 0..len {
                data[ch][t] += epoch.data[ch][t];
            }
        }
    }
    for ch in &mut data {
        for v in ch.iter_mut() {
            *v /= epochs.len() as f64;
        }
    }
    Ok(EvokedResponse {
        n_epochs: epochs.len(),
        channels: channel_names.to_vec(),
        times_s: epochs[0].times_s.clone(),
        data,
    })
}

pub fn run_pipeline(
    definition: &Value,
    window: &MegWindow,
    metadata: &MegMetadata,
    channel_names: &[String],
    cancel: Option<&dyn Fn() -> bool>,
    progress: Option<&dyn Fn(f64, &str)>,
) -> Result<PipelineResult, CoreError> {
    let nodes = definition
        .get("nodes")
        .or_else(|| definition.get("steps"))
        .and_then(Value::as_array)
        .ok_or_else(|| CoreError::InvalidInput("pipeline has no nodes".into()))?;
    let mut current = window.clone();
    let mut epochs: Option<Vec<Epoch>> = None;
    let mut epoch_summary: Option<EpochSummary> = None;
    let mut evoked: Option<EvokedResponse> = None;
    let mut band_powers: Option<BTreeMap<String, f64>> = None;
    let mut bad_channels = Vec::new();
    let mut meg_ica: Option<(neuro_core::ica::IcaModel, Vec<usize>)> = None;
    let mut ica_components: Option<Vec<neuro_core::ica::IcaComponentInfo>> = None;
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
                current = bandpass(&current, low, high)?;
                steps.push(format!("filter[{low}-{high}Hz]"));
            }
            "epoch" | "epochs" => {
                let tmin = params.get("tmin_s").and_then(Value::as_f64).unwrap_or(-0.2);
                let tmax = params.get("tmax_s").and_then(Value::as_f64).unwrap_or(0.5);
                // Optional event-code filter (substring, case-insensitive)
                // so KIT trigger codes and CTF marker names can be selected.
                let event_filter = params
                    .get("event")
                    .or_else(|| params.get("event_code"))
                    .or_else(|| params.get("code"))
                    .and_then(Value::as_str)
                    .map(str::to_lowercase);
                let selected: Vec<crate::MegEvent> = metadata
                    .events
                    .iter()
                    .filter(|event| match &event_filter {
                        Some(filter) => event
                            .code
                            .as_deref()
                            .is_some_and(|code| code.to_lowercase().contains(filter.as_str())),
                        None => true,
                    })
                    .cloned()
                    .collect();
                let (eps, skipped) = epoch_window(
                    &current,
                    &selected,
                    metadata.sampling_frequency_hz,
                    tmin,
                    tmax,
                )?;
                epoch_summary = Some(EpochSummary {
                    n_epochs: eps.len(),
                    n_channels: current.data.len(),
                    tmin_s: tmin,
                    tmax_s: tmax,
                    skipped_events: skipped,
                    event_codes: selected
                        .iter()
                        .filter_map(|event| event.code.clone())
                        .collect::<std::collections::BTreeSet<_>>()
                        .into_iter()
                        .collect(),
                });
                epochs = Some(eps);
                steps.push(format!("epoch[{tmin},{tmax}]"));
            }
            "evoked" | "average" => {
                let eps = epochs.as_ref().ok_or_else(|| {
                    CoreError::InvalidInput("evoked requires a preceding epoch node".into())
                })?;
                evoked = Some(evoked_average(eps, channel_names)?);
                steps.push("evoked".to_string());
            }
            "psd" | "spectral" | "psd_welch" => {
                let fmin: f64 = params.get("fmin_hz").and_then(Value::as_f64).unwrap_or(0.5);
                let fmax: f64 = params
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
                let band_ranges: [(&str, (f64, f64)); 5] = [
                    ("delta", (0.5, 4.0)),
                    ("theta", (4.0, 8.0)),
                    ("alpha", (8.0, 13.0)),
                    ("beta", (13.0, 30.0)),
                    ("gamma", (30.0, 80.0)),
                ];
                for (name, (lo, hi)) in band_ranges {
                    let band_lo = lo.max(fmin);
                    let band_hi = hi.min(fmax);
                    if band_lo < band_hi {
                        bands.insert(name.to_string(), (band_lo, band_hi));
                    }
                }
                let powers = crate::spectral::welch_band_powers(&current, &bands, n_fft);
                band_powers = Some(powers.into_iter().map(|p| (p.band, p.power)).collect());
                steps.push(format!("psd[{fmin}-{fmax}Hz,n_fft={n_fft}]"));
            }
            "synthetic_gradiometer" | "synthetic_grad" | "sgrads" => {
                let max_mm = params
                    .get("max_distance_mm")
                    .and_then(Value::as_f64)
                    .unwrap_or(30.0);
                let min_mm = params
                    .get("min_distance_mm")
                    .and_then(Value::as_f64)
                    .unwrap_or(2.0);
                let n_replaced =
                    synthetic_gradiometers(&mut current, channel_names, metadata, min_mm, max_mm)?;
                steps.push(format!("synthetic_gradiometer[{n_replaced}]"));
            }
            "ica" | "fastica" => {
                let indices = meg_channel_indices(channel_names, metadata);
                if indices.len() < 2 {
                    return Err(CoreError::InvalidInput(
                        "ICA requires at least 2 MEG channels".into(),
                    ));
                }
                let submatrix: Vec<Vec<f64>> = indices
                    .iter()
                    .map(|&index| current.data[index].clone())
                    .collect();
                let max_iter = params
                    .get("max_iter")
                    .and_then(Value::as_u64)
                    .unwrap_or(200) as usize;
                let tol = params.get("tol").and_then(Value::as_f64).unwrap_or(1e-4);
                // Full-rank decomposition keeps a 1:1 row mapping so channel
                // names stay aligned for downstream nodes.
                let model =
                    neuro_core::ica::fastica(&submatrix, Some(indices.len()), max_iter, tol)?;
                ica_components = Some(component_variances(&model.sources));
                for (position, &index) in indices.iter().enumerate() {
                    current.data[index] = model.sources[position].clone();
                }
                meg_ica = Some((model, indices.clone()));
                steps.push(format!("ica[{}]", indices.len()));
            }
            "ica_remove" | "remove_components" | "ica_reconstruct" => {
                let (model, indices) = meg_ica.as_ref().ok_or_else(|| {
                    CoreError::InvalidInput(
                        "component removal requires a preceding ica node".into(),
                    )
                })?;
                let excluded: Vec<usize> = params
                    .get("components")
                    .or_else(|| params.get("exclude"))
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|item| {
                                item.as_u64()
                                    .map(|v| v as usize)
                                    .or_else(|| item.as_str()?.parse().ok())
                            })
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
                let reconstructed = neuro_core::ica::reconstruct(model, &excluded);
                for (position, &index) in indices.iter().enumerate() {
                    current.data[index] = reconstructed[position].clone();
                }
                steps.push(format!("ica_remove[{}]", excluded.len()));
            }
            "ssp" | "signal_space_projection" => {
                let indices = meg_channel_indices(channel_names, metadata);
                if indices.len() < 2 {
                    return Err(CoreError::InvalidInput(
                        "SSP requires at least 2 MEG channels".into(),
                    ));
                }
                let n_samples = current.data.first().map_or(0, Vec::len);
                let sfreq = current
                    .effective_sampling_frequency_hz
                    .max(metadata.sampling_frequency_hz);
                // Collect noise sample indices: either explicit sample/time
                // bounds or event-locked windows.
                let mut noise_samples: Vec<usize> = Vec::new();
                let start_sample = params.get("start_sample").and_then(Value::as_u64);
                let end_sample = params.get("end_sample").and_then(Value::as_u64);
                let start_s = params.get("start_s").and_then(Value::as_f64);
                let end_s = params.get("end_s").and_then(Value::as_f64);
                if let (Some(start), Some(end)) = (start_sample, end_sample) {
                    noise_samples.extend(start as usize..(end as usize).min(n_samples));
                } else if let (Some(start), Some(end)) = (start_s, end_s) {
                    let t0 = current.times_s.first().copied().unwrap_or(0.0);
                    let start_idx = ((start - t0) * sfreq).round().max(0.0) as usize;
                    let end_idx = (((end - t0) * sfreq).round() as usize).min(n_samples);
                    if start_idx < end_idx {
                        noise_samples.extend(start_idx..end_idx);
                    }
                } else {
                    let event_filter = params
                        .get("event")
                        .or_else(|| params.get("event_code"))
                        .or_else(|| params.get("code"))
                        .and_then(Value::as_str)
                        .map(str::to_lowercase);
                    let tmin = params.get("tmin_s").and_then(Value::as_f64).unwrap_or(0.0);
                    let tmax = params.get("tmax_s").and_then(Value::as_f64).unwrap_or(0.5);
                    let selected: Vec<&crate::MegEvent> = metadata
                        .events
                        .iter()
                        .filter(|event| match &event_filter {
                            Some(filter) => event
                                .code
                                .as_deref()
                                .is_some_and(|code| code.to_lowercase().contains(filter.as_str())),
                            None => true,
                        })
                        .collect();
                    if selected.is_empty() {
                        return Err(CoreError::InvalidInput(
                            "ssp found no matching events for the noise window".into(),
                        ));
                    }
                    for event in selected {
                        let center = event.sample as isize;
                        let a = (center + (tmin * sfreq).round() as isize).max(0) as usize;
                        let b =
                            ((center + (tmax * sfreq).round() as isize) as usize).min(n_samples);
                        if a < b {
                            noise_samples.extend(a..b);
                        }
                    }
                }
                noise_samples.sort_unstable();
                noise_samples.dedup();
                noise_samples.truncate(2_000_000);
                if noise_samples.len() < indices.len() {
                    return Err(CoreError::InvalidInput(
                        "ssp noise window has fewer samples than MEG channels".into(),
                    ));
                }
                let n_components = params
                    .get("n_components")
                    .or_else(|| params.get("n_vectors"))
                    .and_then(Value::as_u64)
                    .unwrap_or(1)
                    .clamp(1, 16) as usize;
                // Noise covariance over MEG channels, then top-k eigenvectors.
                let m = indices.len();
                let mut cov = vec![vec![0.0f64; m]; m];
                for &sample in &noise_samples {
                    for i in 0..m {
                        let xi = current.data[indices[i]][sample];
                        for j in 0..=i {
                            cov[i][j] += xi * current.data[indices[j]][sample];
                        }
                    }
                }
                for i in 0..m {
                    for j in 0..=i {
                        cov[i][j] /= noise_samples.len() as f64;
                        cov[j][i] = cov[i][j];
                    }
                }
                let (_eigenvalues, eigenvectors) = neuro_core::ica::jacobi_eigen(&cov);
                // jacobi_eigen returns eigenvalues descending; take the top k.
                let k = n_components.min(m);
                let mut basis = vec![vec![0.0f64; k]; m];
                for j in 0..k {
                    for i in 0..m {
                        basis[i][j] = eigenvectors[i][j];
                    }
                }
                // Project out the noise subspace: x' = (I - UU^T) x per sample.
                for sample in 0..n_samples {
                    for j in 0..k {
                        let mut acc = 0.0;
                        for i in 0..m {
                            acc += basis[i][j] * current.data[indices[i]][sample];
                        }
                        for i in 0..m {
                            current.data[indices[i]][sample] -= basis[i][j] * acc;
                        }
                    }
                }
                steps.push(format!("ssp[k={k},n={}]", noise_samples.len()));
            }
            "sss" | "maxwell" | "signal_space_separation" => {
                let l_in = params
                    .get("l_in")
                    .or_else(|| params.get("internal_order"))
                    .and_then(Value::as_u64)
                    .unwrap_or(4) as usize;
                let l_out = params
                    .get("l_out")
                    .or_else(|| params.get("external_order"))
                    .and_then(Value::as_u64)
                    .unwrap_or(2) as usize;
                let (filtered, n_sensors) = crate::sss::sss_filter(
                    &current.data,
                    channel_names,
                    &metadata.channels,
                    metadata,
                    l_in,
                    l_out,
                )?;
                current.data = filtered;
                steps.push(format!(
                    "sss[l_in={l_in},l_out={l_out},sensors={n_sensors}]"
                ));
            }
            "bad_channels" | "bad-channel-detection" => {
                bad_channels = detect_bad_channels(&current, channel_names);
                steps.push(format!("bad_channels[{}]", bad_channels.len()));
            }
            other => {
                return Err(CoreError::InvalidInput(format!(
                    "unsupported pipeline node: {other}"
                )))
            }
        }
    }
    Ok(PipelineResult {
        steps,
        epochs,
        epoch_summary,
        evoked,
        band_powers,
        bad_channels,
        window: Some(current),
        ica_components,
    })
}

fn component_variances(sources: &[Vec<f64>]) -> Vec<neuro_core::ica::IcaComponentInfo> {
    let variances: Vec<f64> = sources
        .iter()
        .map(|source| {
            let mean = source.iter().sum::<f64>() / source.len().max(1) as f64;
            source.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / source.len().max(1) as f64
        })
        .collect();
    let total: f64 = variances.iter().sum();
    variances
        .iter()
        .enumerate()
        .map(|(index, &variance)| neuro_core::ica::IcaComponentInfo {
            index,
            variance,
            variance_fraction: if total > 0.0 { variance / total } else { 0.0 },
        })
        .collect()
}
