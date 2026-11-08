//! Export MEG windows to NumPy `.npy` files (float32, channels × samples).

use crate::{read_window, MegWindow};
use neuro_core::CoreError;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Hard cap on exported values to keep memory bounded.
pub const MAX_EXPORT_VALUES: usize = 50_000_000;
/// CSV is text — keep the cap tighter so files stay reasonable.
pub const MAX_CSV_VALUES: usize = 5_000_000;

fn write_npy_f32(
    path: &Path,
    channels: usize,
    samples: usize,
    data: &[Vec<f64>],
) -> Result<(), CoreError> {
    let mut file = std::fs::File::create(path)?;
    file.write_all(b"\x93NUMPY")?;
    file.write_all(&[1, 0])?;
    let dict =
        format!("{{'descr': '<f4', 'fortran_order': False, 'shape': ({channels}, {samples}), }}");
    // Header must satisfy: 10 (magic+version+len) + header_len ≡ 0 mod 64,
    // padded with spaces and terminated by \n.
    let prefix = 10;
    let mut pad = 64 - ((prefix + dict.len() + 1) % 64);
    if pad == 64 {
        pad = 0;
    }
    let header_len = dict.len() + pad + 1;
    file.write_all(&(header_len as u16).to_le_bytes())?;
    file.write_all(dict.as_bytes())?;
    for _ in 0..pad {
        file.write_all(b" ")?;
    }
    file.write_all(b"\n")?;
    let mut buf = Vec::with_capacity(channels * samples * 4);
    // C order for shape (channels, samples): channel-major rows.
    for channel in data {
        for &value in channel.iter().take(samples) {
            buf.extend_from_slice(&(value as f32).to_le_bytes());
        }
    }
    file.write_all(&buf)?;
    Ok(())
}

/// Read a window and write it as `.npy`. `data` rows are channels; the file
/// stores shape `(channels, samples)` in C order.
pub fn export_window(
    directory: &Path,
    start_s: f64,
    end_s: f64,
    channel_names: &[String],
    output_path: &Path,
    recording: Option<&str>,
    compensation: Option<i32>,
    apply_projectors: bool,
) -> Result<MegWindow, CoreError> {
    // First pass with a bounded read to learn the native sample count.
    let probe = read_window(
        directory,
        start_s,
        end_s,
        channel_names,
        usize::MAX / 4,
        recording,
        compensation,
        apply_projectors,
    )?;
    let samples = probe.times_s.len();
    let channels = probe.data.len();
    if channels == 0 || samples == 0 {
        return Err(CoreError::InvalidInput("nothing to export".into()));
    }
    if channels.saturating_mul(samples) > MAX_EXPORT_VALUES {
        return Err(CoreError::InvalidInput(
            "export exceeds maximum value count".into(),
        ));
    }
    write_npy_f32(output_path, channels, samples, &probe.data)?;
    Ok(probe)
}

/// Write a window as CSV: header `time_s,<channel names...>`, one row per
/// sample, values in scientific notation. Channel names are sanitized to
/// keep the file parseable.
pub fn write_csv(
    path: &Path,
    window: &MegWindow,
    channel_names: &[String],
) -> Result<(), CoreError> {
    let samples = window.times_s.len();
    let channels = window.data.len();
    if channels == 0 || samples == 0 {
        return Err(CoreError::InvalidInput("nothing to export".into()));
    }
    if channels.saturating_mul(samples) > MAX_CSV_VALUES {
        return Err(CoreError::InvalidInput(
            "CSV export exceeds maximum value count".into(),
        ));
    }
    let clean = |name: &str| -> String {
        name.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.') {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    };
    let mut file = std::io::BufWriter::new(std::fs::File::create(path)?);
    file.write_all(b"time_s")?;
    for index in 0..channels {
        let name = channel_names.get(index).map(String::as_str).unwrap_or("");
        file.write_all(format!(",{}", clean(name)).as_bytes())?;
    }
    file.write_all(b"\n")?;
    for sample in 0..samples {
        file.write_all(format!("{:.9e}", window.times_s[sample]).as_bytes())?;
        for channel in &window.data {
            file.write_all(format!(",{:.6e}", channel[sample]).as_bytes())?;
        }
        file.write_all(b"\n")?;
    }
    file.flush()?;
    Ok(())
}

/// Write a window as a minimal BIDS MEG dataset inside `output_dir`:
/// `<name>_meg.npy` (channels × samples, float32 C-order), `channels.tsv`,
/// `events.tsv` (when the recording has events), and `dataset_description.json`.
pub fn export_window_bids(
    directory: &Path,
    start_s: f64,
    end_s: f64,
    channel_names: &[String],
    output_dir: &Path,
    dataset_name: &str,
    recording: Option<&str>,
    compensation: Option<i32>,
    apply_projectors: bool,
) -> Result<MegWindow, CoreError> {
    let window = read_window(
        directory,
        start_s,
        end_s,
        channel_names,
        usize::MAX / 4,
        recording,
        compensation,
        apply_projectors,
    )?;
    let samples = window.times_s.len();
    let channels = window.data.len();
    if channels == 0 || samples == 0 {
        return Err(CoreError::InvalidInput("nothing to export".into()));
    }
    if channels.saturating_mul(samples) > MAX_EXPORT_VALUES {
        return Err(CoreError::InvalidInput(
            "export exceeds maximum value count".into(),
        ));
    }
    std::fs::create_dir_all(output_dir)?;
    let clean = |text: &str| -> String {
        text.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || matches!(c, '-' | '_') {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    };
    let stem = if dataset_name.trim().is_empty() {
        "meg".to_string()
    } else {
        clean(dataset_name.trim())
    };
    let npy_path = output_dir.join(format!("{stem}_meg.npy"));
    write_npy_f32(&npy_path, channels, samples, &window.data)?;

    // channels.tsv — BIDS MEG sidecar (name, type, units, status).
    let metadata = crate::inspect_recording(directory)?;
    let effective_names: Vec<String> = if channel_names.is_empty() {
        metadata
            .channels
            .iter()
            .map(|channel| channel.name.clone())
            .collect()
    } else {
        channel_names.to_vec()
    };
    let by_name: std::collections::BTreeMap<&str, &crate::MegChannel> = metadata
        .channels
        .iter()
        .map(|channel| (channel.name.as_str(), channel))
        .collect();
    let mut tsv = String::from("name\ttype\tunits\tstatus\n");
    for name in effective_names.iter().take(channels) {
        let info = by_name.get(name.as_str());
        let kind = info.map(|c| c.kind.as_str()).unwrap_or("meg");
        let unit = info.map(|c| c.unit.as_str()).unwrap_or("");
        let status = info.map(|c| c.status.as_str()).unwrap_or("good");
        tsv.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            name.replace('\t', " "),
            kind,
            unit,
            status
        ));
    }
    std::fs::write(output_dir.join("channels.tsv"), tsv)?;

    if !metadata.events.is_empty() {
        let mut events = String::from("onset\tduration\ttrial_type\tvalue\n");
        for event in &metadata.events {
            let trial = event
                .code
                .clone()
                .or_else(|| event.description.clone())
                .unwrap_or_else(|| "event".into());
            events.push_str(&format!(
                "{:.6}\t0\t{}\t{}\n",
                event.time_s,
                trial.replace('\t', " "),
                event.value.map(|v| v.to_string()).unwrap_or_default()
            ));
        }
        std::fs::write(output_dir.join("events.tsv"), events)?;
    }

    let description = serde_json::json!({
        "Name": stem,
        "BIDSVersion": "1.10.0",
        "DatasetType": "meg",
        "Description": "MEG window exported by Neuro Lab",
    });
    std::fs::write(
        output_dir.join("dataset_description.json"),
        serde_json::to_string_pretty(&description)?,
    )?;
    Ok(window)
}

/// Read a window and write it as CSV next to `output_path`.
pub fn export_window_csv(
    directory: &Path,
    start_s: f64,
    end_s: f64,
    channel_names: &[String],
    output_path: &Path,
    recording: Option<&str>,
    compensation: Option<i32>,
    apply_projectors: bool,
) -> Result<MegWindow, CoreError> {
    let window = read_window(
        directory,
        start_s,
        end_s,
        channel_names,
        usize::MAX / 4,
        recording,
        compensation,
        apply_projectors,
    )?;
    write_csv(output_path, &window, channel_names)?;
    Ok(window)
}

/// Default output path: `<stem>.meg.npy` beside a file input or
/// `<dirname>.meg.npy` beside a directory input.
pub fn default_output_path(input: &Path) -> PathBuf {
    let stem = if input.is_dir() {
        input
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("meg")
            .to_string()
    } else {
        input
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("meg")
            .to_string()
    };
    let parent = if input.is_dir() {
        input.parent().unwrap_or_else(|| Path::new("."))
    } else {
        input.parent().unwrap_or_else(|| Path::new("."))
    };
    parent.join(format!("{stem}.meg.npy"))
}

/// Read a window and write it as a FIFF `.fif` file using the tagged-block
/// container in [`crate::fif::write_fif`]. Channel metadata (kind, unit,
/// coil type, position/orientation) is carried over from the source
/// recording when available.
pub fn export_window_fif(
    directory: &Path,
    start_s: f64,
    end_s: f64,
    channel_names: &[String],
    output_path: &Path,
    recording: Option<&str>,
    compensation: Option<i32>,
    apply_projectors: bool,
) -> Result<MegWindow, CoreError> {
    let probe = read_window(
        directory,
        start_s,
        end_s,
        channel_names,
        usize::MAX / 4,
        recording,
        compensation,
        apply_projectors,
    )?;
    let samples = probe.times_s.len();
    let channels = probe.data.len();
    if channels == 0 || samples == 0 {
        return Err(CoreError::InvalidInput("nothing to export".into()));
    }
    if channels.saturating_mul(samples) > MAX_EXPORT_VALUES {
        return Err(CoreError::InvalidInput(
            "export exceeds maximum value count".into(),
        ));
    }
    let metadata = crate::inspect_recording(directory)?;
    let by_name: std::collections::HashMap<&str, &crate::MegChannel> = metadata
        .channels
        .iter()
        .map(|channel| (channel.name.as_str(), channel))
        .collect();
    // Window channel order matches `channel_names` (or recording order when
    // the selection is empty); resolve names in data order.
    let order: Vec<String> = if channel_names.is_empty() {
        metadata
            .channels
            .iter()
            .map(|channel| channel.name.clone())
            .collect()
    } else {
        channel_names.to_vec()
    };
    if order.len() != channels {
        return Err(CoreError::InvalidInput(
            "FIF export channel count mismatch".into(),
        ));
    }
    let write_channels: Vec<crate::fif::FifWriteChannel> = order
        .iter()
        .map(|name| {
            let source = by_name.get(name.as_str());
            let mut loc = [0.0f64; 12];
            if let Some(position) = source.and_then(|c| c.position_mm.as_ref()) {
                if position.len() >= 12 {
                    for (i, slot) in loc.iter_mut().enumerate() {
                        *slot = position[i];
                    }
                } else if position.len() >= 3 {
                    // FIFF loc is metres; r0 (coil centre) sits at [9..12].
                    for d in 0..3 {
                        loc[9 + d] = position[d] / 1000.0;
                        if let Some(ori) = source.and_then(|c| c.orientation.as_ref()) {
                            if ori.len() >= 3 {
                                loc[6 + d] = ori[d];
                            }
                        }
                    }
                }
            }
            let coil_type = source
                .and_then(|c| c.coil_type.as_deref())
                .and_then(|text| text.rsplit('_').next())
                .and_then(|digits| digits.parse::<i32>().ok())
                .unwrap_or(0);
            crate::fif::FifWriteChannel {
                name: name.clone(),
                kind: fif_kind_code(source.map(|c| c.kind.as_str()).unwrap_or("")),
                unit: fif_unit_code(source.map(|c| c.unit.as_str()).unwrap_or("")),
                unit_mul: source.map(|c| c.unit_mul).unwrap_or(1.0).round() as i32,
                cal: 1.0,
                range: 1.0,
                coil_type,
                loc,
            }
        })
        .collect();
    let sfreq = if probe.effective_sampling_frequency_hz.is_finite()
        && probe.effective_sampling_frequency_hz > 0.0
    {
        probe.effective_sampling_frequency_hz
    } else {
        metadata.sampling_frequency_hz
    };
    crate::fif::write_fif(output_path, sfreq, &write_channels, &probe.data)?;
    Ok(probe)
}

fn fif_kind_code(kind: &str) -> i32 {
    match kind.to_ascii_lowercase().as_str() {
        "meg" | "mag" | "gradiometer" | "planar" => 1,
        "eeg" => 2,
        "stim" | "trigger" | "sti" => 3,
        "eog" => 202,
        "ecg" => 402,
        _ => 201,
    }
}

fn fif_unit_code(unit: &str) -> i32 {
    match unit.to_ascii_lowercase().as_str() {
        "volt" | "v" => 107,
        "tesla/m" | "t/m" | "tesla_per_metre" => 201,
        _ => 112,
    }
}
