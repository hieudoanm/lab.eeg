use neuro_core::CoreError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChannelManifest {
    #[serde(default)]
    pub sampling_frequency_hz: Option<f64>,
    #[serde(default)]
    pub channel_names: Vec<String>,
    #[serde(default)]
    pub channel_units: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NpyMetadata {
    pub shape: (usize, usize),
    pub dtype: String,
    pub sampling_frequency_hz: f64,
    pub duration_seconds: f64,
    pub channel_names: Vec<String>,
    pub channel_units: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChannelStats {
    pub channel_index: usize,
    pub channel_name: String,
    pub unit: Option<String>,
    pub mean: f64,
    pub standard_deviation: f64,
    pub minimum: f64,
    pub maximum: f64,
    pub peak_to_peak: f64,
    pub valid_sample_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SignalWindow {
    pub start_seconds: f64,
    pub end_seconds: f64,
    pub downsample_factor: usize,
    pub time_seconds: Vec<f64>,
    pub channels: Vec<ChannelSignal>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChannelSignal {
    pub channel_index: usize,
    pub channel_name: String,
    #[serde(default)]
    pub unit: Option<String>,
    pub samples: Vec<Option<f64>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EegAnalysisResult {
    pub dataset_id: String,
    pub algorithm_version: String,
    pub parameters: serde_json::Value,
    pub channels: Vec<ChannelStats>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EegEvent {
    pub sample: usize,
    pub time_s: f64,
    pub label: String,
}

pub mod brainvision;
pub mod edf;
pub use neuro_core::ica;
pub use neuro_core::ica::IcaComponentInfo;
pub mod pipeline;

pub fn channel_manifest(path: &Path) -> ChannelManifest {
    let Some(manifest_path) = manifest_path(path) else {
        return ChannelManifest {
            sampling_frequency_hz: None,
            channel_names: Vec::new(),
            channel_units: None,
        };
    };
    let Ok(text) = std::fs::read_to_string(manifest_path) else {
        return ChannelManifest {
            sampling_frequency_hz: None,
            channel_names: Vec::new(),
            channel_units: None,
        };
    };
    serde_json::from_str(&text).unwrap_or(ChannelManifest {
        sampling_frequency_hz: None,
        channel_names: Vec::new(),
        channel_units: None,
    })
}

fn manifest_path(path: &Path) -> Option<PathBuf> {
    let stem = path.file_stem()?.to_string_lossy();
    let candidate = path.with_file_name(format!("{stem}.json"));
    candidate.exists().then_some(candidate)
}

fn default_channel_names(count: usize) -> Vec<String> {
    (0..count)
        .map(|index| format!("EEG {:03}", index + 1))
        .collect()
}

pub fn validate_npy_file(
    path: &Path,
    max_channels: usize,
    max_samples: usize,
) -> Result<NpyMetadata, CoreError> {
    let bytes = std::fs::read(path)?;
    let parsed = parse_npy_v1(&bytes)?;
    if parsed.shape.0 == 0 || parsed.shape.1 == 0 {
        return Err(CoreError::InvalidInput(
            "recording must contain channels and samples".into(),
        ));
    }
    if parsed.shape.0 > max_channels || parsed.shape.1 > max_samples {
        return Err(CoreError::InvalidInput(
            "recording exceeds configured limits".into(),
        ));
    }
    if parsed.sampling_frequency_hz <= 0.0 || !parsed.sampling_frequency_hz.is_finite() {
        return Err(CoreError::InvalidInput(
            "sampling frequency must be positive".into(),
        ));
    }
    Ok(parsed)
}

pub fn describe_npy(
    path: &Path,
    max_channels: usize,
    max_samples: usize,
    sampling_frequency_hz: f64,
) -> Result<NpyMetadata, CoreError> {
    let shape = read_npy_shape(path)?;
    if shape.0 == 0 || shape.1 == 0 {
        return Err(CoreError::InvalidInput(
            "recording must contain channels and samples".into(),
        ));
    }
    if shape.0 > max_channels || shape.1 > max_samples {
        return Err(CoreError::InvalidInput(
            "recording exceeds configured limits".into(),
        ));
    }
    if sampling_frequency_hz <= 0.0 || !sampling_frequency_hz.is_finite() {
        return Err(CoreError::InvalidInput(
            "sampling frequency must be positive".into(),
        ));
    }
    let manifest = channel_manifest(path);
    let mut channel_names = manifest.channel_names;
    if channel_names.is_empty() {
        channel_names = default_channel_names(shape.0);
    }
    if channel_names.len() != shape.0 {
        return Err(CoreError::InvalidInput(
            "channel metadata does not match recording".into(),
        ));
    }
    let channel_units = manifest
        .channel_units
        .unwrap_or_else(|| vec!["uV".into(); shape.0]);
    if channel_units.len() != shape.0 {
        return Err(CoreError::InvalidInput(
            "channel unit metadata does not match recording".into(),
        ));
    }
    Ok(NpyMetadata {
        shape,
        dtype: "numeric".into(),
        sampling_frequency_hz,
        duration_seconds: shape.1 as f64 / sampling_frequency_hz,
        channel_names,
        channel_units,
    })
}

fn parse_npy_v1(bytes: &[u8]) -> Result<NpyMetadata, CoreError> {
    let (shape, header) = parse_npy_header(bytes)?;
    let sampling_frequency_hz = extract_literal(&header, "sampling_frequency_hz")
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.0);
    Ok(NpyMetadata {
        shape,
        dtype: extract_dtype(&header).unwrap_or_else(|| "numeric".into()),
        sampling_frequency_hz,
        duration_seconds: if sampling_frequency_hz > 0.0 {
            shape.1 as f64 / sampling_frequency_hz
        } else {
            0.0
        },
        channel_names: Vec::new(),
        channel_units: Vec::new(),
    })
}

fn parse_npy_header(bytes: &[u8]) -> Result<((usize, usize), String), CoreError> {
    if bytes.len() < 10 || &bytes[..6] != b"\x93NUMPY" {
        return Err(CoreError::InvalidInput("unsupported NPY header".into()));
    }
    let major = bytes[6];
    let (header_len, offset) = match major {
        1 => {
            let len = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
            (len, 10)
        }
        2 | 3 => {
            if bytes.len() < 12 {
                return Err(CoreError::InvalidInput("truncated NPY header".into()));
            }
            let len = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
            (len, 12)
        }
        _ => return Err(CoreError::InvalidInput("unsupported NPY version".into())),
    };
    let end = offset + header_len;
    if bytes.len() < end {
        return Err(CoreError::InvalidInput("truncated NPY header".into()));
    }
    let header = std::str::from_utf8(&bytes[offset..end])
        .map_err(|_| CoreError::InvalidInput("invalid NPY header".into()))?
        .to_string();
    let shape = extract_shape(&header)?;
    Ok((shape, header))
}

fn read_npy_shape(path: &Path) -> Result<(usize, usize), CoreError> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    use std::io::Read;
    let mut magic = [0_u8; 10];
    reader.read_exact(&mut magic)?;
    if &magic[..6] != b"\x93NUMPY" {
        return Err(CoreError::InvalidInput("unsupported NPY header".into()));
    }
    let (header_len, _offset) = match magic[6] {
        1 => (u16::from_le_bytes([magic[8], magic[9]]) as usize, 10usize),
        2 | 3 => {
            let mut len = [0_u8; 4];
            reader.read_exact(&mut len)?;
            (u32::from_le_bytes(len) as usize, 12usize)
        }
        _ => return Err(CoreError::InvalidInput("unsupported NPY version".into())),
    };
    let mut header = vec![0_u8; header_len];
    reader.read_exact(&mut header)?;
    let header = String::from_utf8_lossy(&header);
    extract_shape(&header)
}

fn extract_shape(header: &str) -> Result<(usize, usize), CoreError> {
    let marker = "'shape': (";
    let start = header
        .find(marker)
        .ok_or_else(|| CoreError::InvalidInput("NPY shape missing".into()))?
        + marker.len();
    let rest = &header[start..];
    let end = rest
        .find(')')
        .ok_or_else(|| CoreError::InvalidInput("NPY shape malformed".into()))?;
    let dims: Vec<usize> = rest[..end]
        .split(',')
        .filter_map(|part| part.trim().parse::<usize>().ok())
        .collect();
    if dims.len() != 2 {
        return Err(CoreError::InvalidInput(
            "EEG NPY recording must be two-dimensional".into(),
        ));
    }
    Ok((dims[0], dims[1]))
}

fn extract_literal(header: &str, key: &str) -> Option<String> {
    let marker = format!("'{key}': ");
    let start = header.find(&marker)? + marker.len();
    let rest = &header[start..];
    let end = rest
        .find(|c| c == ',' || c == '\n' || c == '}')
        .unwrap_or(rest.len());
    Some(rest[..end].trim().trim_matches('\'').to_string())
}

fn extract_dtype(header: &str) -> Option<String> {
    let marker = "'descr': ";
    let start = header.find(marker)? + marker.len();
    let rest = &header[start..];
    let end = rest
        .find(|c| c == ',' || c == '\n' || c == '}')
        .unwrap_or(rest.len());
    Some(rest[..end].trim().trim_matches('\'').to_string())
}

pub fn read_npy_matrix(path: &Path) -> Result<ndarray::Array2<f64>, CoreError> {
    let bytes = std::fs::read(path)?;
    let (_shape, header) = parse_npy_header(&bytes)?;
    let dtype = extract_dtype(&header).unwrap_or_default();
    let array: ndarray::Array2<f64> = match dtype.as_str() {
        "<f4" | "|f4" => {
            let array: ndarray::Array2<f32> = ndarray_npy::read_npy(path).map_err(|error| {
                CoreError::InvalidInput(format!("invalid NPY recording: {error}"))
            })?;
            array.mapv(|value| value as f64)
        }
        "<f8" | "|f8" => ndarray_npy::read_npy(path)
            .map_err(|error| CoreError::InvalidInput(format!("invalid NPY recording: {error}")))?,
        _ => {
            return Err(CoreError::InvalidInput(format!(
                "unsupported NPY dtype: {dtype}"
            )))
        }
    };
    if array.nrows() == 0 || array.ncols() == 0 {
        return Err(CoreError::InvalidInput(
            "EEG NPY recording must be two-dimensional".into(),
        ));
    }
    Ok(array)
}

pub fn read_npy_matrix_f64(path: &Path) -> Result<Vec<Vec<f64>>, CoreError> {
    let array = read_npy_matrix(path)?;
    Ok(array.rows().into_iter().map(|row| row.to_vec()).collect())
}

pub fn channel_statistics(
    matrix: &[Vec<f64>],
    channel_names: &[String],
    channel_units: Option<&[String]>,
) -> Result<Vec<ChannelStats>, CoreError> {
    if matrix.is_empty() {
        return Err(CoreError::InvalidInput(
            "recording must contain at least one channel".into(),
        ));
    }
    if channel_names.len() != matrix.len() {
        return Err(CoreError::InvalidInput(
            "channel names do not match recording".into(),
        ));
    }
    if let Some(units) = channel_units {
        if units.len() != matrix.len() {
            return Err(CoreError::InvalidInput(
                "channel units do not match recording".into(),
            ));
        }
    }
    matrix
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let finite: Vec<f64> = row
                .iter()
                .copied()
                .filter(|value| value.is_finite())
                .collect();
            if finite.is_empty() {
                return Err(CoreError::InvalidInput(format!(
                    "channel '{}' has no valid samples",
                    channel_names[index]
                )));
            }
            let count = finite.len() as f64;
            let mean = finite.iter().sum::<f64>() / count;
            let variance = finite
                .iter()
                .map(|value| (value - mean).powi(2))
                .sum::<f64>()
                / count;
            let minimum = finite.iter().copied().fold(f64::INFINITY, f64::min);
            let maximum = finite.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            Ok(ChannelStats {
                channel_index: index,
                channel_name: channel_names[index].clone(),
                unit: channel_units.map(|units| units[index].clone()),
                mean,
                standard_deviation: variance.sqrt(),
                minimum,
                maximum,
                peak_to_peak: maximum - minimum,
                valid_sample_count: finite.len(),
            })
        })
        .collect()
}

pub fn analyze_recording(
    path: &Path,
    dataset_id: &str,
    sampling_frequency_hz: f64,
) -> Result<EegAnalysisResult, CoreError> {
    let (matrix, channel_names, channel_units, sfreq, _events) =
        load_recording_with_events(path, sampling_frequency_hz)?;
    let stats = channel_statistics(&matrix, &channel_names, Some(&channel_units))?;
    Ok(EegAnalysisResult {
        dataset_id: dataset_id.to_string(),
        algorithm_version: "neuro-eeg-descriptive-v1".into(),
        parameters: serde_json::json!({ "sampling_frequency_hz": sfreq }),
        channels: stats,
    })
}

pub fn extract_signal_window(
    path: &Path,
    start_seconds: f64,
    end_seconds: f64,
    channel_indices: &[usize],
    channel_names: &[String],
    channel_units: Option<&[String]>,
    sampling_frequency_hz: f64,
    max_points_per_channel: usize,
    max_window_seconds: f64,
) -> Result<SignalWindow, CoreError> {
    if !start_seconds.is_finite()
        || !end_seconds.is_finite()
        || start_seconds < 0.0
        || end_seconds <= start_seconds
    {
        return Err(CoreError::InvalidInput("invalid signal window".into()));
    }
    if end_seconds - start_seconds > max_window_seconds {
        return Err(CoreError::InvalidInput(
            "signal window exceeds the maximum duration".into(),
        ));
    }
    let is_edf = path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("edf"));
    let is_brainvision = brainvision::vhdr_path(path).exists();
    let (matrix, sfreq) = if is_edf {
        let (matrix, info) = edf::read_edf(path)?;
        (matrix, info.sampling_frequency_hz)
    } else if is_brainvision {
        let (matrix, info) = brainvision::read_brainvision(path)?;
        (matrix, info.sampling_frequency_hz)
    } else {
        (read_npy_matrix_f64(path)?, sampling_frequency_hz)
    };
    extract_signal_window_from_matrix(
        &matrix,
        start_seconds,
        end_seconds,
        channel_indices,
        channel_names,
        channel_units,
        sfreq,
        max_points_per_channel,
        max_window_seconds,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn extract_signal_window_from_matrix(
    matrix: &[Vec<f64>],
    start_seconds: f64,
    end_seconds: f64,
    channel_indices: &[usize],
    channel_names: &[String],
    channel_units: Option<&[String]>,
    sampling_frequency_hz: f64,
    max_points_per_channel: usize,
    max_window_seconds: f64,
) -> Result<SignalWindow, CoreError> {
    if end_seconds - start_seconds > max_window_seconds {
        return Err(CoreError::InvalidInput(
            "signal window exceeds the maximum duration".into(),
        ));
    }
    if matrix.is_empty() || matrix[0].is_empty() {
        return Err(CoreError::InvalidInput("recording is empty".into()));
    }
    let channels = matrix.len();
    let samples = matrix[0].len();
    let duration_seconds = samples as f64 / sampling_frequency_hz;
    if end_seconds > duration_seconds + 1e-6 {
        return Err(CoreError::InvalidInput(
            "signal window extends beyond the recording".into(),
        ));
    }
    let mut selected = Vec::new();
    for &index in channel_indices {
        if index >= channels {
            return Err(CoreError::InvalidInput(
                "selected channel is outside the recording".into(),
            ));
        }
        if !selected.contains(&index) {
            selected.push(index);
        }
    }
    if selected.is_empty() {
        return Err(CoreError::InvalidInput(
            "at least one channel must be selected".into(),
        ));
    }
    let start = ((start_seconds * sampling_frequency_hz).floor() as usize).min(samples);
    let end = ((end_seconds * sampling_frequency_hz).ceil() as usize)
        .min(samples)
        .max(start);
    if end <= start {
        return Err(CoreError::InvalidInput(
            "signal window contains no samples".into(),
        ));
    }
    let span = end - start;
    let factor =
        ((span + max_points_per_channel.saturating_sub(1)) / max_points_per_channel.max(1)).max(1);
    let mut time_seconds = Vec::new();
    let mut sample_index = start;
    while sample_index < end {
        time_seconds.push(sample_index as f64 / sampling_frequency_hz);
        sample_index += factor;
    }
    let mut channel_signals = Vec::new();
    for channel_index in selected {
        let mut values = Vec::new();
        let mut sample_index = start;
        while sample_index < end {
            let value = matrix[channel_index][sample_index];
            values.push(if value.is_finite() { Some(value) } else { None });
            sample_index += factor;
        }
        channel_signals.push(ChannelSignal {
            channel_index,
            channel_name: channel_names
                .get(channel_index)
                .cloned()
                .unwrap_or_else(|| format!("channel_{channel_index}")),
            unit: channel_units
                .and_then(|units| units.get(channel_index))
                .cloned(),
            samples: values,
        });
    }
    Ok(SignalWindow {
        start_seconds,
        end_seconds,
        downsample_factor: factor,
        time_seconds,
        channels: channel_signals,
    })
}

pub fn synthetic_recording(seed: u64, channels: usize, samples: usize) -> Vec<Vec<f64>> {
    let mut state = seed;
    (0..channels)
        .map(|channel| {
            (0..samples)
                .map(|sample| {
                    state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let noise = ((state >> 33) as f64 / u32::MAX as f64) - 0.5;
                    (sample as f64 * 0.01 + channel as f64 * 0.1 + noise).sin()
                })
                .collect()
        })
        .collect()
}

/// Writes a deterministic synthetic EEG recording as a float32 `.npy` file
/// plus a JSON sidecar manifest (channel names, units, sampling frequency).
pub fn write_synthetic_recording(
    directory: &Path,
    file_name: &str,
    channels: usize,
    samples: usize,
    sampling_frequency_hz: f64,
    seed: u64,
) -> Result<serde_json::Value, CoreError> {
    if channels == 0 || samples == 0 {
        return Err(CoreError::InvalidInput(
            "channels and samples must be positive".into(),
        ));
    }
    if sampling_frequency_hz <= 0.0 || !sampling_frequency_hz.is_finite() {
        return Err(CoreError::InvalidInput(
            "sampling_frequency_hz must be positive".into(),
        ));
    }
    let data = synthetic_recording(seed, channels, samples);
    let array =
        ndarray::Array2::from_shape_vec((channels, samples), data.into_iter().flatten().collect())
            .map_err(|error| {
                CoreError::InvalidInput(format!("invalid synthetic array: {error}"))
            })?;
    let array = array.mapv(|value| value as f32);
    std::fs::create_dir_all(directory)?;
    let npy_path = directory.join(format!("{file_name}.npy"));
    ndarray_npy::write_npy(&npy_path, &array)
        .map_err(|error| CoreError::InvalidInput(format!("could not write NPY: {error}")))?;
    let names: Vec<String> = (0..channels).map(|i| format!("CH{:03}", i + 1)).collect();
    let manifest = serde_json::json!({
        "sampling_frequency_hz": sampling_frequency_hz,
        "channel_names": names,
        "channel_units": vec!["uV"; channels],
        "synthetic": true,
        "seed": seed,
    });
    std::fs::write(
        directory.join(format!("{file_name}.json")),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(serde_json::json!({
        "path": npy_path.to_string_lossy(),
        "channels": channels,
        "samples": samples,
        "sampling_frequency_hz": sampling_frequency_hz,
    }))
}

pub fn load_recording(
    path: &Path,
    fallback_sfreq: f64,
) -> Result<(Vec<Vec<f64>>, Vec<String>, Vec<String>, f64), CoreError> {
    load_recording_with_events(path, fallback_sfreq)
        .map(|(matrix, names, units, sfreq, _)| (matrix, names, units, sfreq))
}

/// Like [`load_recording`] but also returns annotation/marker events where
/// the format provides them (EDF+ TAL channels, BrainVision `.vmrk`).
pub fn load_recording_with_events(
    path: &Path,
    fallback_sfreq: f64,
) -> Result<(Vec<Vec<f64>>, Vec<String>, Vec<String>, f64, Vec<EegEvent>), CoreError> {
    if path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("edf"))
    {
        let (matrix, info) = edf::read_edf(path)?;
        let n = info.channel_names.len();
        let units = if info.units.iter().all(|unit| unit.is_empty()) {
            vec!["uV".to_string(); n]
        } else {
            info.units
        };
        let events = info.events.clone();
        return Ok((
            matrix,
            info.channel_names,
            units,
            info.sampling_frequency_hz,
            events,
        ));
    }
    if brainvision::vhdr_path(path).exists() {
        let (matrix, info) = brainvision::read_brainvision(path)?;
        let events = info
            .events
            .iter()
            .map(|event| EegEvent {
                sample: event.sample,
                time_s: event.time_s,
                label: event.label.clone(),
            })
            .collect();
        return Ok((
            matrix,
            info.channel_names,
            info.units,
            info.sampling_frequency_hz,
            events,
        ));
    }
    let metadata = describe_npy(path, 512, 10_000_000, fallback_sfreq)?;
    let matrix = read_npy_matrix_f64(path)?;
    Ok((
        matrix,
        metadata.channel_names,
        metadata.channel_units,
        metadata.sampling_frequency_hz,
        Vec::new(),
    ))
}

/// Best-effort EEG file-set validation: resolves the format from the path
/// (or the first supported file inside a directory), checks required
/// companions, and returns human-readable findings. An empty list means
/// the file set is usable; findings describe problems or notable gaps.
pub fn validate_recording(path: &Path) -> (String, Vec<String>) {
    let mut findings = Vec::new();
    let target = if path.is_dir() {
        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let candidate = entry.path();
                if let Some(ext) = candidate
                    .extension()
                    .and_then(|value| value.to_str())
                    .map(str::to_lowercase)
                {
                    if matches!(ext.as_str(), "vhdr" | "edf" | "npy") {
                        candidates.push(candidate);
                    }
                }
            }
        }
        candidates.sort();
        match candidates.into_iter().next() {
            Some(candidate) => candidate,
            None => {
                findings.push("no supported EEG file (.vhdr/.edf/.npy) found".into());
                return ("unknown".into(), findings);
            }
        }
    } else {
        path.to_path_buf()
    };
    let ext = target
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "vhdr" => match brainvision::inspect_brainvision(&target) {
            Ok(info) => {
                if !info.data_path.exists() {
                    findings.push(format!(
                        "BrainVision data file missing: {}",
                        info.data_path.display()
                    ));
                }
                if info.marker_path.is_none() {
                    findings.push("no .vmrk marker file; trigger events unavailable".into());
                }
                findings.push(format!(
                    "{} channels, {} samples @ {:.1} Hz ({})",
                    info.channel_names.len(),
                    info.n_samples,
                    info.sampling_frequency_hz,
                    info.binary_format
                ));
                ("brainvision".into(), findings)
            }
            Err(error) => {
                findings.push(format!("invalid BrainVision header: {error}"));
                ("brainvision".into(), findings)
            }
        },
        "edf" => match edf::inspect_edf(&target) {
            Ok(info) => {
                if info.events.is_empty() {
                    findings.push("no EDF+ annotation channel; trigger events unavailable".into());
                }
                findings.push(format!(
                    "{} channels, {} samples @ {:.1} Hz",
                    info.channel_names.len(),
                    info.n_samples_per_channel,
                    info.sampling_frequency_hz
                ));
                ("edf".into(), findings)
            }
            Err(error) => {
                findings.push(format!("invalid EDF file: {error}"));
                ("edf".into(), findings)
            }
        },
        "npy" => match validate_npy_file(&target, 4096, 200_000_000) {
            Ok(parsed) => {
                if manifest_path(&target).is_none() {
                    findings.push("no channel manifest; default channel names/units used".into());
                }
                findings.push(format!(
                    "{} channels x {} samples @ {:.1} Hz ({})",
                    parsed.shape.0, parsed.shape.1, parsed.sampling_frequency_hz, parsed.dtype
                ));
                ("npy".into(), findings)
            }
            Err(error) => {
                findings.push(format!("invalid NPY recording: {error}"));
                ("npy".into(), findings)
            }
        },
        _ => {
            findings.push(format!("unsupported EEG format: .{ext}"));
            ("unknown".into(), findings)
        }
    }
}
