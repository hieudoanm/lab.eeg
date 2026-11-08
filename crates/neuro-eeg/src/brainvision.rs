//! Bounded reader for BrainVision `.vhdr`/`.eeg`/`.vmrk` recordings.
//!
//! Parses the INI-style header (`[Common Infos]`, `[Binary Infos]`,
//! `[Channel Infos]`), decodes binary IEEE_FLOAT_32 / INT_16 / INT_32 data
//! in MULTIPLEXED (sample-major) or VECTORIZED (channel-major) order, applies
//! per-channel resolution factors, and parses `.vmrk` marker files into
//! sample-based events. ASCII data is rejected.

use neuro_core::CoreError;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const MAX_CHANNELS: usize = 4096;
const MAX_TOTAL_SAMPLES: usize = 200_000_000;
const MAX_EVENTS: usize = 1_000_000;

#[derive(Clone, Debug, Serialize)]
pub struct BvEvent {
    pub sample: usize,
    pub time_s: f64,
    pub label: String,
    pub marker_type: String,
    pub duration_samples: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct BvInfo {
    pub channel_names: Vec<String>,
    pub units: Vec<String>,
    pub resolutions: Vec<f64>,
    pub sampling_frequency_hz: f64,
    pub n_samples: usize,
    pub data_orientation: String,
    pub binary_format: String,
    pub data_path: PathBuf,
    pub marker_path: Option<PathBuf>,
    #[serde(default)]
    pub events: Vec<BvEvent>,
}

fn parse_vhdr(text: &str) -> Result<BTreeMap<String, BTreeMap<String, String>>, CoreError> {
    let mut sections: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut current: Option<String> = None;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            current = Some(line[1..line.len() - 1].trim().to_string());
            continue;
        }
        let Some(section) = &current else { continue };
        if let Some((key, value)) = line.split_once('=') {
            sections
                .entry(section.clone())
                .or_default()
                .insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    Ok(sections)
}

fn get<'a>(
    sections: &'a BTreeMap<String, BTreeMap<String, String>>,
    section: &str,
    key: &str,
) -> Result<&'a str, CoreError> {
    sections
        .get(section)
        .and_then(|table| table.get(key))
        .map(String::as_str)
        .ok_or_else(|| {
            CoreError::InvalidInput(format!("BrainVision header missing {section}.{key}"))
        })
}

pub fn vhdr_path(path: &Path) -> PathBuf {
    if path
        .extension()
        .and_then(|v| v.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("vhdr"))
    {
        path.to_path_buf()
    } else {
        // Allow pointing at the .eeg file directly.
        path.with_extension("vhdr")
    }
}

pub fn inspect_brainvision(path: &Path) -> Result<BvInfo, CoreError> {
    let hdr = vhdr_path(path);
    let text = std::fs::read_to_string(&hdr).map_err(|_| {
        CoreError::NotFound(format!("BrainVision header not found: {}", hdr.display()))
    })?;
    let sections = parse_vhdr(&text)?;
    let common = sections.get("Common Infos").ok_or_else(|| {
        CoreError::InvalidInput("BrainVision header missing [Common Infos]".into())
    })?;

    let n_channels: usize = get(&sections, "Common Infos", "NumberOfChannels")?
        .parse()
        .map_err(|_| CoreError::InvalidInput("invalid NumberOfChannels".into()))?;
    if n_channels == 0 || n_channels > MAX_CHANNELS {
        return Err(CoreError::InvalidInput(
            "invalid BrainVision channel count".into(),
        ));
    }
    let sfreq: f64 = get(&sections, "Common Infos", "SamplingRate")?
        .parse()
        .map_err(|_| CoreError::InvalidInput("invalid SamplingRate".into()))?;
    if !(sfreq > 0.0 && sfreq.is_finite()) {
        return Err(CoreError::InvalidInput(
            "invalid BrainVision sampling rate".into(),
        ));
    }
    let orientation = get(&sections, "Common Infos", "DataOrientation")?.to_string();
    if !matches!(orientation.as_str(), "MULTIPLEXED" | "VECTORIZED") {
        return Err(CoreError::InvalidInput(format!(
            "unsupported BrainVision DataOrientation: {orientation}"
        )));
    }
    let data_format = get(&sections, "Common Infos", "DataFormat")?;
    if !data_format.eq_ignore_ascii_case("BINARY") {
        return Err(CoreError::InvalidInput(
            "only binary BrainVision data is supported".into(),
        ));
    }
    let binary_format = get(&sections, "Binary Infos", "BinaryFormat")?.to_string();
    let bytes_per_value = match binary_format.as_str() {
        "IEEE_FLOAT_32" => 4,
        "INT_16" => 2,
        "INT_32" => 4,
        other => {
            return Err(CoreError::InvalidInput(format!(
                "unsupported BrainVision BinaryFormat: {other}"
            )))
        }
    };
    let _ = bytes_per_value;

    // Channel infos: chN=name,ref,resolution,unit
    let mut names = vec![String::new(); n_channels];
    let mut units = vec![String::new(); n_channels];
    let mut resolutions = vec![f64::NAN; n_channels];
    if let Some(channel_table) = sections.get("Channel Infos") {
        for (key, value) in channel_table {
            let Some(index) = key
                .strip_prefix("ch")
                .and_then(|rest| rest.parse::<usize>().ok())
            else {
                continue;
            };
            if index == 0 || index > n_channels {
                continue;
            }
            let props: Vec<&str> = value.split(',').collect();
            if props.is_empty() {
                continue;
            }
            let slot = index - 1;
            names[slot] = props[0].replace(r"\1", ",");
            if props.len() >= 4 {
                let resolution = props[2].trim();
                resolutions[slot] = if resolution.is_empty() {
                    1.0
                } else {
                    resolution.parse::<f64>().unwrap_or(f64::NAN)
                };
                units[slot] = props[3].trim().replace("\u{c2}", "");
            }
        }
    }
    for (index, name) in names.iter_mut().enumerate() {
        if name.is_empty() {
            *name = format!("EEG {index:03}");
        }
        if resolutions[index].is_nan() {
            resolutions[index] = 1.0;
        }
        if units[index].is_empty() {
            units[index] = "µV".to_string();
        }
    }

    let data_name = get(&sections, "Common Infos", "DataFile")?;
    let data_path = hdr.parent().unwrap_or(Path::new(".")).join(data_name);

    // Sample count: explicit DataPoints (VECTORIZED) or derived from file size.
    let n_samples = if let Some(points) = common.get("DataPoints") {
        points
            .trim()
            .parse::<usize>()
            .map_err(|_| CoreError::InvalidInput("invalid DataPoints".into()))?
    } else {
        let bytes = std::fs::metadata(&data_path)
            .map_err(|_| {
                CoreError::NotFound(format!(
                    "BrainVision data file not found: {}",
                    data_path.display()
                ))
            })?
            .len() as usize;
        let value_bytes = match binary_format.as_str() {
            "IEEE_FLOAT_32" => 4,
            "INT_16" => 2,
            "INT_32" => 4,
            _ => unreachable!(),
        };
        if bytes % (n_channels * value_bytes) != 0 {
            return Err(CoreError::InvalidInput(
                "BrainVision data size does not match header".into(),
            ));
        }
        bytes / (n_channels * value_bytes)
    };
    if n_samples == 0 || n_samples.saturating_mul(n_channels) > MAX_TOTAL_SAMPLES {
        return Err(CoreError::InvalidInput(
            "BrainVision recording too large".into(),
        ));
    }

    let marker_path = common
        .get("MarkerFile")
        .map(|name| name.trim())
        .filter(|name| !name.is_empty())
        .map(|name| hdr.parent().unwrap_or(Path::new(".")).join(name))
        .filter(|path| path.exists())
        .or_else(|| {
            let sibling = hdr.with_extension("vmrk");
            sibling.exists().then_some(sibling)
        });

    let mut info = BvInfo {
        channel_names: names,
        units,
        resolutions,
        sampling_frequency_hz: sfreq,
        n_samples,
        data_orientation: orientation,
        binary_format,
        data_path,
        marker_path,
        events: Vec::new(),
    };
    if let Some(marker) = &info.marker_path {
        info.events = read_vmrk(marker, sfreq)?;
    }
    Ok(info)
}

fn read_vmrk(path: &Path, sfreq: f64) -> Result<Vec<BvEvent>, CoreError> {
    let text = std::fs::read_to_string(path)
        .map_err(|_| CoreError::NotFound(format!("marker file unreadable: {}", path.display())))?;
    let in_markers = text
        .lines()
        .any(|line| line.trim().eq_ignore_ascii_case("[Marker Infos]"));
    if !in_markers {
        return Ok(Vec::new());
    }
    let mut events = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with("Mk") {
            continue;
        }
        let Some((_, payload)) = line.split_once('=') else {
            continue;
        };
        let props: Vec<&str> = payload.split(',').collect();
        if props.len() < 4 {
            continue;
        }
        let marker_type = props[0].replace(r"\1", ",");
        let description = props[1].replace(r"\1", ",");
        // BrainVision marker positions are 1-indexed sample numbers.
        let onset: i64 = props[2].trim().parse().unwrap_or(0);
        let duration: i64 = props[3].trim().parse().unwrap_or(0);
        if onset <= 0 {
            continue;
        }
        let sample = (onset - 1) as usize;
        events.push(BvEvent {
            sample,
            time_s: sample as f64 / sfreq,
            label: if description.is_empty() {
                marker_type.clone()
            } else {
                description
            },
            marker_type,
            duration_samples: duration,
        });
        if events.len() >= MAX_EVENTS {
            break;
        }
    }
    Ok(events)
}

fn decode_value(bytes: &[u8], offset: usize, format: &str) -> Option<f64> {
    match format {
        "IEEE_FLOAT_32" => {
            Some(f32::from_le_bytes(bytes.get(offset..offset + 4)?.try_into().ok()?) as f64)
        }
        "INT_16" => {
            Some(i16::from_le_bytes(bytes.get(offset..offset + 2)?.try_into().ok()?) as f64)
        }
        "INT_32" => {
            Some(i32::from_le_bytes(bytes.get(offset..offset + 4)?.try_into().ok()?) as f64)
        }
        _ => None,
    }
}

fn value_bytes(format: &str) -> usize {
    match format {
        "INT_16" => 2,
        _ => 4,
    }
}

pub fn read_brainvision(path: &Path) -> Result<(Vec<Vec<f64>>, BvInfo), CoreError> {
    let info = inspect_brainvision(path)?;
    let bytes = std::fs::read(&info.data_path)?;
    let n_channels = info.channel_names.len();
    let vb = value_bytes(&info.binary_format);
    let mut channels = vec![Vec::with_capacity(info.n_samples); n_channels];
    if info.data_orientation == "MULTIPLEXED" {
        let frame = n_channels * vb;
        for sample in 0..info.n_samples {
            let base = sample * frame;
            for channel in 0..n_channels {
                let offset = base + channel * vb;
                let raw = decode_value(&bytes, offset, &info.binary_format).unwrap_or(0.0);
                channels[channel].push(raw * info.resolutions[channel]);
            }
        }
    } else {
        // VECTORIZED: each channel occupies a contiguous block.
        let block = info.n_samples * vb;
        for channel in 0..n_channels {
            let start = channel * block;
            for sample in 0..info.n_samples {
                let raw =
                    decode_value(&bytes, start + sample * vb, &info.binary_format).unwrap_or(0.0);
                channels[channel].push(raw * info.resolutions[channel]);
            }
        }
    }
    Ok((channels, info))
}
