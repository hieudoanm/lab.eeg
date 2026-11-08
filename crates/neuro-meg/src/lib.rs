pub mod bids;
pub mod ctf;
pub mod export;
pub mod fif;
pub mod formats;
pub mod kit;
pub mod pipeline;
pub mod sample;
pub mod spectral;
pub mod sss;

use neuro_core::CoreError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MegMetadata {
    pub format: String,
    pub name: String,
    pub acquisition_system: String,
    pub sampling_frequency_hz: f64,
    pub channel_count: usize,
    pub duration_seconds: f64,
    pub coordinate_frame: String,
    pub channels: Vec<MegChannel>,
    #[serde(default)]
    pub events: Vec<MegEvent>,
    #[serde(default)]
    pub bad_channel_names: Vec<String>,
    #[serde(default)]
    pub sensor_type: String,
    #[serde(default)]
    pub time_varying: bool,
    #[serde(default)]
    pub device_serial: Option<String>,
    #[serde(default)]
    pub manufacturer: Option<String>,
    #[serde(default)]
    pub digitization_points: usize,
    /// Names of all `*_meg` recordings found in a multi-file BIDS dataset.
    #[serde(default)]
    pub recordings: Vec<String>,
    /// Channel names per BIDS recording (stem -> channels), for
    /// reconciling differing montages across runs.
    #[serde(default)]
    pub recording_channels: std::collections::BTreeMap<String, Vec<String>>,
    /// CTF balance/compensation matrices parsed from res4; applied to
    /// window data when a target `compensation_grade` is requested.
    #[serde(default)]
    pub compensation: Vec<ctf::CtfCompMatrix>,
    /// Digitized head surface / landmark points from BIDS coordsystem.json
    /// (positions in metres, BIDS convention).
    #[serde(default)]
    pub surface_points: Vec<MegSurfacePoint>,
}

/// A digitized point from a BIDS `coordsystem.json` sidecar.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MegSurfacePoint {
    #[serde(default)]
    pub name: Option<String>,
    /// Position in metres (BIDS coordsystem convention).
    pub position_m: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MegEvent {
    pub sample: usize,
    pub time_s: f64,
    pub code: Option<String>,
    pub value: Option<i64>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MegChannel {
    pub name: String,
    pub kind: String,
    pub unit: String,
    #[serde(default = "one")]
    pub unit_mul: f64,
    #[serde(default)]
    pub group_name: Option<String>,
    #[serde(default = "good_status")]
    pub status: String,
    #[serde(default)]
    pub coil_type: Option<String>,
    #[serde(default)]
    pub position_mm: Option<Vec<f64>>,
    #[serde(default)]
    pub orientation: Option<Vec<f64>>,
}

fn one() -> f64 {
    1.0
}

fn good_status() -> String {
    "good".into()
}

fn unit_map(unit: &str) -> &'static str {
    match unit {
        "T" => "tesla",
        "T/m" => "tesla_per_metre",
        "V" => "volt",
        "A" => "amperes",
        "1" => "dimensionless",
        _ => "other",
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MegWindow {
    pub times_s: Vec<f64>,
    pub data: Vec<Vec<f64>>,
    pub effective_sampling_frequency_hz: f64,
}

pub fn inspect_recording(directory: &Path) -> Result<MegMetadata, CoreError> {
    if directory
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("fif"))
    {
        return inspect_fif(directory);
    }
    if directory
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("sqd") || ext.eq_ignore_ascii_case("con"))
    {
        return inspect_kit_file(directory);
    }
    if directory.join("header.json").exists() || directory.join("channels.json").exists() {
        return inspect_opm(directory);
    }
    if directory.is_dir() {
        let has_res4 = std::fs::read_dir(directory)
            .map(|entries| {
                entries.flatten().any(|entry| {
                    entry
                        .path()
                        .extension()
                        .and_then(|value| value.to_str())
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("res4"))
                })
            })
            .unwrap_or(false);
        if has_res4 {
            return inspect_ctf(directory);
        }
        let has_kit = std::fs::read_dir(directory)
            .map(|entries| {
                entries.flatten().any(|entry| {
                    entry
                        .path()
                        .extension()
                        .and_then(|value| value.to_str())
                        .is_some_and(|ext| {
                            ext.eq_ignore_ascii_case("sqd") || ext.eq_ignore_ascii_case("con")
                        })
                })
            })
            .unwrap_or(false);
        if has_kit {
            return inspect_kit_dir(directory);
        }
        if bids::find_bids_meg(directory).is_some() {
            return bids::inspect_recording(directory);
        }
    }
    Err(CoreError::InvalidInput(
        "supported local MEG recording requires a FIF file, CTF .ds, KIT .sqd/.con, BIDS MEG dataset, or UCL OPM JSON sidecar with .bin payload".into(),
    ))
}

fn inspect_ctf(dir: &Path) -> Result<MegMetadata, CoreError> {
    let info = ctf::inspect_ctf(dir)?;
    Ok(MegMetadata {
        format: "ctf_ds".into(),
        name: dir
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("ctf_recording")
            .to_string(),
        acquisition_system: "ctf".into(),
        sampling_frequency_hz: info.sampling_frequency_hz,
        channel_count: info.n_channels,
        duration_seconds: (info.n_samples_per_trial * info.n_trials) as f64
            / info.sampling_frequency_hz,
        coordinate_frame: "head".into(),
        channels: info
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
            .collect(),
        events: info.events.clone(),
        compensation: info.compensation.clone(),
        surface_points: Vec::new(),
        bad_channel_names: Vec::new(),
        sensor_type: "unknown".into(),
        time_varying: false,
        device_serial: None,
        manufacturer: Some("CTF".into()),
        digitization_points: 0,
        recordings: Vec::new(),
        recording_channels: Default::default(),
    })
}

fn inspect_kit_dir(dir: &Path) -> Result<MegMetadata, CoreError> {
    // KIT data is a single .sqd/.con file inside the directory.
    let data = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .is_some_and(|ext| {
                    ext.eq_ignore_ascii_case("sqd") || ext.eq_ignore_ascii_case("con")
                })
        })
        .ok_or_else(|| CoreError::NotFound("KIT data file not found".into()))?;
    inspect_kit_file(&data)
}

fn inspect_kit_file(data: &Path) -> Result<MegMetadata, CoreError> {
    let (channels, info) = kit::metadata(data)?;
    // External Polhemus exports (.elp fiducials/HPI, .hsp headshape)
    // that may sit next to the .sqd/.con file.
    let (elp_points, hsp_points) = data.parent().map(kit::sidecar_points).unwrap_or_default();
    let elp_count = elp_points.len();
    let mut surface_points = elp_points;
    surface_points.extend(hsp_points);
    Ok(MegMetadata {
        format: "kit".into(),
        name: data
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("kit_recording")
            .to_string(),
        acquisition_system: "kit".into(),
        sampling_frequency_hz: info.sampling_frequency_hz,
        channel_count: info.n_channels,
        duration_seconds: info.n_samples as f64 / info.sampling_frequency_hz,
        coordinate_frame: "head".into(),
        channels,
        events: kit::read_events(data)?,
        bad_channel_names: Vec::new(),
        sensor_type: info.system_name,
        time_varying: false,
        device_serial: None,
        manufacturer: Some(info.model_name),
        digitization_points: info.digitization_points + elp_count,
        recordings: Vec::new(),
        recording_channels: Default::default(),
        compensation: Vec::new(),
        surface_points,
    })
}

fn fif_channel_kind(kind: i32) -> &'static str {
    match kind {
        1 | 301 => "meg",
        2 | 802 | 803 | 902 => "eeg",
        3 => "stim",
        202 => "eog",
        402 => "ecg",
        _ => "misc",
    }
}

fn fif_channel_unit(unit: i32) -> &'static str {
    match unit {
        107 => "volt",
        112 => "tesla",
        201 => "tesla_per_metre",
        210 => "ampere",
        _ => "other",
    }
}

fn inspect_fif(path: &Path) -> Result<MegMetadata, CoreError> {
    let info = fif::inspect_fif(path)?;
    Ok(MegMetadata {
        format: "fif".into(),
        name: path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("fif_recording")
            .to_string(),
        acquisition_system: "squid".into(),
        sampling_frequency_hz: info.sampling_frequency_hz,
        channel_count: info.n_channels,
        duration_seconds: info.n_samples as f64 / info.sampling_frequency_hz,
        coordinate_frame: "head".into(),
        channels: info
            .channel_info
            .iter()
            .map(|channel| MegChannel {
                name: channel.name.clone(),
                kind: fif_channel_kind(channel.kind).into(),
                unit: fif_channel_unit(channel.unit).into(),
                unit_mul: channel.cal,
                group_name: None,
                status: if info.bad_channels.contains(&channel.name) {
                    "bad".into()
                } else {
                    "good".into()
                },
                coil_type: if channel.coil_type != 0 {
                    Some(format!("fiff_coil_{}", channel.coil_type))
                } else {
                    None
                },
                position_mm: Some(channel.loc.to_vec()),
                orientation: None,
            })
            .collect(),
        events: fif::info_events(&info),
        bad_channel_names: info.bad_channels.clone(),
        sensor_type: "unknown".into(),
        time_varying: false,
        device_serial: None,
        manufacturer: None,
        digitization_points: 0,
        recordings: Vec::new(),
        recording_channels: Default::default(),
        compensation: Vec::new(),
        surface_points: Vec::new(),
    })
}

fn inspect_opm(directory: &Path) -> Result<MegMetadata, CoreError> {
    let header_path = ["header.json", "channels.json"]
        .iter()
        .map(|name| directory.join(name))
        .find(|path| path.exists())
        .ok_or_else(|| CoreError::NotFound("MEG JSON sidecar not found".into()))?;
    let text = std::fs::read_to_string(header_path)?;
    let header: Value = serde_json::from_str(&text)?;
    let sfreq = header
        .get("sampling_frequency_hz")
        .and_then(Value::as_f64)
        .ok_or_else(|| CoreError::InvalidInput("sampling_frequency_hz is required".into()))?;
    if sfreq <= 0.0 || !sfreq.is_finite() {
        return Err(CoreError::InvalidInput(
            "sampling_frequency_hz must be positive".into(),
        ));
    }
    let channels = header
        .get("channels")
        .and_then(Value::as_array)
        .ok_or_else(|| CoreError::InvalidInput("channels must be an array".into()))?
        .iter()
        .map(|channel| MegChannel {
            name: channel
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("channel")
                .to_string(),
            kind: channel
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("opm")
                .to_string(),
            unit: unit_map(channel.get("unit").and_then(Value::as_str).unwrap_or("1")).to_string(),
            unit_mul: channel
                .get("unit_mul")
                .and_then(Value::as_f64)
                .unwrap_or(1.0),
            group_name: channel
                .get("group_name")
                .and_then(Value::as_str)
                .map(str::to_string),
            status: channel
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("good")
                .to_string(),
            coil_type: channel
                .get("coil_type")
                .and_then(Value::as_str)
                .map(str::to_string),
            position_mm: channel
                .get("position_mm")
                .and_then(Value::as_array)
                .map(|values| values.iter().filter_map(Value::as_f64).collect::<Vec<_>>()),
            orientation: channel
                .get("orientation")
                .and_then(Value::as_array)
                .map(|values| values.iter().filter_map(Value::as_f64).collect::<Vec<_>>()),
        })
        .collect::<Vec<_>>();
    if channels.is_empty() {
        return Err(CoreError::InvalidInput(
            "recording must contain channels".into(),
        ));
    }
    let channel_count = header
        .get("n_channels")
        .and_then(Value::as_u64)
        .map(|value| value as usize)
        .unwrap_or(channels.len());
    let bytes_per_sample = match header
        .get("dtype")
        .and_then(Value::as_str)
        .unwrap_or("float32")
    {
        "float32" | "<f4" | "f4" => 4,
        "float64" | "<f8" | "f8" => 8,
        other => {
            return Err(CoreError::InvalidInput(format!(
                "unsupported MEG dtype: {other}"
            )))
        }
    };
    let bin = std::fs::read_dir(directory)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.extension().and_then(|value| value.to_str()) == Some("bin"))
        .ok_or_else(|| CoreError::NotFound("MEG .bin payload not found".into()))?;
    let bytes = std::fs::metadata(bin)?.len() as usize;
    let expected = channel_count * bytes_per_sample;
    if expected == 0 || bytes % expected != 0 {
        return Err(CoreError::InvalidInput(
            "MEG payload does not match channel metadata".into(),
        ));
    }
    let samples = bytes / expected;
    let events = header
        .get("event_sets")
        .and_then(Value::as_array)
        .map(|sets| {
            sets.iter()
                .flat_map(|set| {
                    set.get("events")
                        .and_then(Value::as_array)
                        .cloned()
                        .unwrap_or_default()
                })
                .filter_map(|event| {
                    Some(MegEvent {
                        sample: event.get("sample")?.as_u64()? as usize,
                        time_s: event.get("time_s").and_then(Value::as_f64).unwrap_or(0.0),
                        code: event
                            .get("code")
                            .and_then(Value::as_str)
                            .map(str::to_string),
                        value: event.get("value").and_then(Value::as_i64),
                        description: event
                            .get("description")
                            .and_then(Value::as_str)
                            .map(str::to_string),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let bad_channel_names = channels
        .iter()
        .filter(|channel| channel.status == "bad")
        .map(|channel| channel.name.clone())
        .collect();
    Ok(MegMetadata {
        format: "ucl_opm_bin".into(),
        name: header
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("opm_recording")
            .to_string(),
        acquisition_system: "opm".into(),
        sampling_frequency_hz: sfreq,
        channel_count,
        duration_seconds: samples as f64 / sfreq,
        coordinate_frame: header
            .get("coordinate_frame")
            .and_then(Value::as_str)
            .unwrap_or("head")
            .to_string(),
        channels,
        events,
        bad_channel_names,
        sensor_type: header
            .get("sensor_type")
            .and_then(Value::as_str)
            .unwrap_or("opm_vector")
            .to_string(),
        time_varying: header
            .get("movement_tracking")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        device_serial: header
            .get("device_serial")
            .and_then(Value::as_str)
            .map(str::to_string),
        manufacturer: header
            .get("manufacturer")
            .and_then(Value::as_str)
            .map(str::to_string),
        digitization_points: 0,
        recordings: Vec::new(),
        recording_channels: Default::default(),
        compensation: Vec::new(),
        surface_points: Vec::new(),
    })
}

pub fn read_window(
    directory: &Path,
    start_s: f64,
    end_s: f64,
    channel_names: &[String],
    max_samples: usize,
    recording: Option<&str>,
    compensation: Option<i32>,
    apply_projectors: bool,
) -> Result<MegWindow, CoreError> {
    if directory
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("fif"))
    {
        return fif::read_window(
            directory,
            start_s,
            end_s,
            channel_names,
            max_samples,
            compensation,
            apply_projectors,
        );
    }
    if directory.is_file()
        && directory
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("sqd") || ext.eq_ignore_ascii_case("con"))
    {
        return kit::read_window(directory, start_s, end_s, channel_names, max_samples);
    }
    if directory.is_dir() {
        let has_res4 = std::fs::read_dir(directory)
            .map(|entries| {
                entries.flatten().any(|entry| {
                    entry
                        .path()
                        .extension()
                        .and_then(|value| value.to_str())
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("res4"))
                })
            })
            .unwrap_or(false);
        if has_res4 {
            return ctf::read_window(
                directory,
                start_s,
                end_s,
                channel_names,
                max_samples,
                compensation,
            );
        }
        if recording.is_some_and(|selector| {
            matches!(
                selector.to_ascii_lowercase().as_str(),
                "all" | "concat" | "*"
            )
        }) {
            return bids::read_window_concat(
                directory,
                start_s,
                end_s,
                channel_names,
                max_samples,
                compensation,
                apply_projectors,
            );
        }
        if let Some(data) = bids::data_path_for(directory, recording) {
            // BIDS windows and event times use the concatenated multi-run
            // timeline: translate the requested absolute window into the
            // selected run's local time base, then shift returned times back.
            let (offset, run_duration) =
                bids::run_span_for(directory, recording).unwrap_or((0.0, f64::INFINITY));
            let local_start = (start_s - offset).max(0.0);
            let local_end = (end_s - offset).min(run_duration);
            if local_end <= local_start {
                return Err(CoreError::InvalidInput(
                    "window does not intersect the selected BIDS run".into(),
                ));
            }
            let mut window = read_window(
                &data,
                local_start,
                local_end,
                channel_names,
                max_samples,
                None,
                compensation,
                apply_projectors,
            )?;
            if offset > 0.0 {
                for time in &mut window.times_s {
                    *time += offset;
                }
            }
            return Ok(window);
        }
        if recording.is_some() && bids::find_bids_meg(directory).is_some() {
            return Err(CoreError::InvalidInput(format!(
                "no BIDS MEG recording matches {recording:?}"
            )));
        }
        // KIT directory: delegate to the .sqd/.con file inside.
        if let Ok(entries) = std::fs::read_dir(directory) {
            if let Some(kit_file) = entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .find(|path| {
                    path.extension()
                        .and_then(|value| value.to_str())
                        .is_some_and(|ext| {
                            ext.eq_ignore_ascii_case("sqd") || ext.eq_ignore_ascii_case("con")
                        })
                })
            {
                return kit::read_window(&kit_file, start_s, end_s, channel_names, max_samples);
            }
        }
    }
    let metadata = inspect_recording(directory)?;
    if start_s < 0.0 || end_s <= start_s || end_s > metadata.duration_seconds + 1e-9 {
        return Err(CoreError::InvalidInput("invalid MEG window".into()));
    }
    let header_path = ["header.json", "channels.json"]
        .iter()
        .map(|name| directory.join(name))
        .find(|path| path.exists())
        .unwrap();
    let header: Value = serde_json::from_str(&std::fs::read_to_string(header_path)?)?;
    let bytes_per_sample = match header
        .get("dtype")
        .and_then(Value::as_str)
        .unwrap_or("float32")
    {
        "float32" | "<f4" | "f4" => 4usize,
        "float64" | "<f8" | "f8" => 8usize,
        _ => 4,
    };
    let names: Vec<String> = metadata
        .channels
        .iter()
        .map(|channel| channel.name.clone())
        .collect();
    let mut indices = Vec::new();
    for name in channel_names {
        let index = names
            .iter()
            .position(|candidate| candidate == name)
            .ok_or_else(|| CoreError::InvalidInput(format!("unknown MEG channel: {name}")))?;
        if !indices.contains(&index) {
            indices.push(index);
        }
    }
    if indices.is_empty() {
        indices = (0..metadata.channel_count).collect();
    }
    let bin = std::fs::read_dir(directory)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.extension().and_then(|value| value.to_str()) == Some("bin"))
        .unwrap();
    let bytes = std::fs::read(bin)?;
    let frame = metadata.channel_count * bytes_per_sample;
    let samples_per_channel = if frame == 0 { 0 } else { bytes.len() / frame };
    let start =
        ((start_s * metadata.sampling_frequency_hz).round() as usize).min(samples_per_channel);
    let end = ((end_s * metadata.sampling_frequency_hz).round() as usize)
        .min(samples_per_channel)
        .max(start);
    let native = end.saturating_sub(start);
    let mut factor = 1usize;
    while native / factor > max_samples {
        factor *= 2;
    }
    let mut times = Vec::new();
    let mut data = vec![Vec::new(); indices.len()];
    let mut sample = start;
    while sample < end {
        times.push(sample as f64 / metadata.sampling_frequency_hz);
        for (output, &channel) in indices.iter().enumerate() {
            let offset = (sample * metadata.channel_count + channel) * bytes_per_sample;
            let value = if bytes_per_sample == 8 {
                f64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
            } else {
                f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as f64
            };
            data[output].push(value);
        }
        sample += factor;
    }
    Ok(MegWindow {
        times_s: times,
        data,
        effective_sampling_frequency_hz: metadata.sampling_frequency_hz / factor as f64,
    })
}

pub fn band_power_summary(
    window: &MegWindow,
    bands: &BTreeMap<String, (f64, f64)>,
) -> BTreeMap<String, f64> {
    let mut result = BTreeMap::new();
    for (name, (low, high)) in bands {
        let mut total = 0.0;
        let mut count = 0usize;
        for channel in &window.data {
            for (index, value) in channel.iter().enumerate() {
                let frequency = index as f64 * window.effective_sampling_frequency_hz
                    / window.times_s.len().max(1) as f64;
                if frequency >= *low && frequency <= *high {
                    total += value * value;
                    count += 1;
                }
            }
        }
        if count > 0 {
            result.insert(name.clone(), total / count as f64);
        }
    }
    result
}
