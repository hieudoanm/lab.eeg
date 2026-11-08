//! Bounded reader for EDF+ / EDF-C recordings.
//!
//! Parses the fixed 256-byte header plus 256-byte per-signal headers and
//! decodes 16-bit little-endian samples record-by-record. Physical values
//! are recovered via each signal's `(phys_min, phys_max)` / `(dig_min,
//! dig_max)` calibration. Annotations (EDF+ TALs) are skipped.

use crate::EegEvent;
use neuro_core::CoreError;
use serde::Serialize;
use std::path::Path;

const MAX_CHANNELS: usize = 4096;
const MAX_TOTAL_SAMPLES: usize = 200_000_000;
const SIGNAL_HEADER: usize = 256;

#[derive(Clone, Debug, Serialize)]
pub struct EdfInfo {
    pub channel_names: Vec<String>,
    pub units: Vec<String>,
    pub sampling_frequency_hz: f64,
    pub n_samples_per_channel: usize,
    pub n_records: usize,
    pub record_duration_s: f64,
    pub patient: String,
    pub recording: String,
    #[serde(default)]
    pub events: Vec<EegEvent>,
}

fn field(text: &str, start: usize, len: usize) -> String {
    text.get(start..start + len)
        .unwrap_or("")
        .trim()
        .to_string()
}

fn parse_f64(text: &str) -> Option<f64> {
    text.trim().parse::<f64>().ok()
}

pub fn inspect_edf(path: &Path) -> Result<EdfInfo, CoreError> {
    let bytes = std::fs::read(path)?;
    if bytes.len() < 256 {
        return Err(CoreError::InvalidInput("EDF file too small".into()));
    }
    let header = String::from_utf8_lossy(&bytes[..256]).to_string();
    let version = field(&header, 0, 8);
    if !version.starts_with("0") && !version.contains("EDF+C") {
        return Err(CoreError::InvalidInput(format!(
            "unsupported EDF version: {version}"
        )));
    }
    let header_bytes = field(&header, 184, 8)
        .parse::<usize>()
        .map_err(|_| CoreError::InvalidInput("invalid EDF header size".into()))?;
    let n_records = field(&header, 236, 8).trim().parse::<i64>().unwrap_or(-1);
    let record_duration = parse_f64(&field(&header, 244, 8))
        .ok_or_else(|| CoreError::InvalidInput("invalid EDF record duration".into()))?;
    if record_duration <= 0.0 {
        return Err(CoreError::InvalidInput(
            "EDF record duration must be positive".into(),
        ));
    }
    let n_signals = field(&header, 252, 4)
        .trim()
        .parse::<usize>()
        .map_err(|_| CoreError::InvalidInput("invalid EDF signal count".into()))?;
    if n_signals == 0 || n_signals > MAX_CHANNELS {
        return Err(CoreError::InvalidInput("invalid EDF signal count".into()));
    }
    if header_bytes < 256 + n_signals * SIGNAL_HEADER {
        return Err(CoreError::InvalidInput("EDF header size mismatch".into()));
    }
    let signal_headers =
        String::from_utf8_lossy(&bytes[256..header_bytes.min(bytes.len())]).to_string();
    let mut names = Vec::with_capacity(n_signals);
    let mut units = Vec::with_capacity(n_signals);
    let mut samples_per_record = Vec::with_capacity(n_signals);
    for index in 0..n_signals {
        let base = index * SIGNAL_HEADER;
        if base + SIGNAL_HEADER > signal_headers.len() {
            return Err(CoreError::InvalidInput(
                "truncated EDF signal header".into(),
            ));
        }
        let h = &signal_headers[base..base + SIGNAL_HEADER];
        names.push(field(h, 0, 16));
        units.push(field(h, 96, 8));
        let nr = field(h, 216, 8)
            .trim()
            .parse::<usize>()
            .map_err(|_| CoreError::InvalidInput("invalid EDF samples-per-record".into()))?;
        samples_per_record.push(nr);
    }
    if samples_per_record.iter().any(|&n| n == 0) {
        return Err(CoreError::InvalidInput("EDF signal has no samples".into()));
    }
    let data_bytes = bytes.len().saturating_sub(header_bytes);
    let bytes_per_record: usize = samples_per_record.iter().sum::<usize>() * 2;
    if bytes_per_record == 0 || data_bytes % bytes_per_record != 0 {
        return Err(CoreError::InvalidInput(
            "EDF data size does not match header".into(),
        ));
    }
    let records = data_bytes / bytes_per_record;
    if n_records >= 0 && records != n_records as usize {
        return Err(CoreError::InvalidInput(
            "EDF record count does not match header".into(),
        ));
    }
    let total_samples: usize = samples_per_record.iter().map(|n| n * records).sum();
    if total_samples > MAX_TOTAL_SAMPLES {
        return Err(CoreError::InvalidInput("EDF recording too large".into()));
    }
    // Sampling frequency from the first signal's samples per record.
    let sfreq = samples_per_record[0] as f64 / record_duration;
    if !(sfreq > 0.0 && sfreq.is_finite()) {
        return Err(CoreError::InvalidInput(
            "invalid EDF sampling frequency".into(),
        ));
    }
    // EDF+ annotations live in a dedicated signal ("EDF Annotations") whose
    // 16-bit samples carry ASCII TAL bytes.
    let annotation_index = names
        .iter()
        .position(|name| name.to_lowercase().contains("annotation"));
    let events = match annotation_index {
        Some(index) => parse_annotation_channel(
            &bytes,
            header_bytes,
            &samples_per_record,
            index,
            records,
            record_duration,
            sfreq,
        )?,
        None => Vec::new(),
    };
    Ok(EdfInfo {
        channel_names: names,
        units,
        sampling_frequency_hz: sfreq,
        n_samples_per_channel: samples_per_record[0] * records,
        n_records: records,
        record_duration_s: record_duration,
        patient: field(&header, 8, 80),
        recording: field(&header, 88, 80),
        events,
    })
}

/// Parse EDF+ TALs (`+onset\x15duration\x14text\x14`) from one record's
/// annotation-channel bytes. `onset` is relative to the record start.
fn parse_tals(record_bytes: &[u8], record_start_s: f64, sfreq: f64, out: &mut Vec<EegEvent>) {
    let mut i = 0usize;
    while i < record_bytes.len() {
        if record_bytes[i] != b'+' {
            i += 1;
            continue;
        }
        i += 1;
        let take_until = |bytes: &[u8], from: usize, delim: u8| -> Option<(String, usize)> {
            let end = bytes[from..].iter().position(|b| *b == delim)? + from;
            Some((
                String::from_utf8_lossy(&bytes[from..end])
                    .trim()
                    .to_string(),
                end + 1,
            ))
        };
        let Some((onset_text, next)) = take_until(record_bytes, i, 0x15) else {
            break;
        };
        i = next;
        let Some((duration_text, next)) = take_until(record_bytes, i, 0x14) else {
            break;
        };
        i = next;
        let Some((text, next)) = take_until(record_bytes, i, 0x14) else {
            break;
        };
        i = next;
        let Ok(onset) = onset_text.parse::<f64>() else {
            continue;
        };
        if !onset.is_finite() || onset < 0.0 {
            continue;
        }
        let _ = duration_text.parse::<f64>();
        let absolute = record_start_s + onset;
        let sample = (absolute * sfreq).round() as usize;
        out.push(EegEvent {
            sample,
            time_s: absolute,
            label: if text.is_empty() {
                "annotation".into()
            } else {
                text
            },
        });
        if out.len() >= 1_000_000 {
            return;
        }
    }
}

fn parse_annotation_channel(
    bytes: &[u8],
    header_bytes: usize,
    samples_per_record: &[usize],
    index: usize,
    records: usize,
    record_duration: f64,
    sfreq: f64,
) -> Result<Vec<EegEvent>, CoreError> {
    let prefix: usize = samples_per_record[..index].iter().sum::<usize>() * 2;
    let count = samples_per_record[index];
    let bytes_per_record: usize = samples_per_record.iter().sum::<usize>() * 2;
    let data = &bytes[header_bytes.min(bytes.len())..];
    let mut events = Vec::new();
    for record in 0..records {
        let base = record * bytes_per_record + prefix;
        if base + count * 2 > data.len() {
            break;
        }
        let mut tal_bytes = Vec::with_capacity(count);
        for s in 0..count {
            let offset = base + s * 2;
            let raw = i16::from_le_bytes([data[offset], data[offset + 1]]) as i64;
            // Annotation samples carry ASCII codes; anything outside byte
            // range becomes a NUL separator.
            tal_bytes.push(if (0..=255).contains(&raw) {
                raw as u8
            } else {
                0
            });
        }
        parse_tals(
            &tal_bytes,
            record as f64 * record_duration,
            sfreq,
            &mut events,
        );
    }
    Ok(events)
}

pub fn read_edf(path: &Path) -> Result<(Vec<Vec<f64>>, EdfInfo), CoreError> {
    let info = inspect_edf(path)?;
    let bytes = std::fs::read(path)?;
    let header_bytes = field(
        &String::from_utf8_lossy(&bytes[..256.min(bytes.len())]),
        184,
        8,
    )
    .parse::<usize>()
    .unwrap_or(256 + info.channel_names.len() * SIGNAL_HEADER);
    let n_signals = info.channel_names.len();
    let signal_headers =
        String::from_utf8_lossy(&bytes[256..header_bytes.min(bytes.len())]).to_string();
    let mut phys_min = Vec::with_capacity(n_signals);
    let mut phys_max = Vec::with_capacity(n_signals);
    let mut dig_min = Vec::with_capacity(n_signals);
    let mut dig_max = Vec::with_capacity(n_signals);
    let mut samples_per_record = Vec::with_capacity(n_signals);
    for index in 0..n_signals {
        let base = index * SIGNAL_HEADER;
        let h = &signal_headers[base..base + SIGNAL_HEADER];
        phys_min.push(parse_f64(&field(h, 104, 8)).unwrap_or(0.0));
        phys_max.push(parse_f64(&field(h, 112, 8)).unwrap_or(1.0));
        dig_min.push(parse_f64(&field(h, 120, 8)).unwrap_or(-32768.0));
        dig_max.push(parse_f64(&field(h, 128, 8)).unwrap_or(32767.0));
        samples_per_record.push(
            field(h, 216, 8)
                .trim()
                .parse::<usize>()
                .map_err(|_| CoreError::InvalidInput("invalid EDF samples-per-record".into()))?,
        );
    }
    let data = &bytes[header_bytes.min(bytes.len())..];
    let mut channels = vec![Vec::with_capacity(info.n_records * samples_per_record[0]); n_signals];
    let mut offset = 0usize;
    for _ in 0..info.n_records {
        for signal in 0..n_signals {
            let count = samples_per_record[signal];
            let scale = (phys_max[signal] - phys_min[signal])
                / (dig_max[signal] - dig_min[signal]).max(f64::MIN_POSITIVE);
            let out = &mut channels[signal];
            for _ in 0..count {
                if offset + 2 > data.len() {
                    return Err(CoreError::InvalidInput("truncated EDF data".into()));
                }
                let raw = i16::from_le_bytes([data[offset], data[offset + 1]]) as f64;
                out.push(phys_min[signal] + (raw - dig_min[signal]) * scale);
                offset += 2;
            }
        }
    }
    Ok((channels, info))
}

/// Write a channel-major float matrix as an EDF+ file.
///
/// Uses a single record spanning the whole recording; per-signal physical
/// ranges are set from the data min/max with 16-bit digital scaling.
pub fn write_edf(
    path: &Path,
    data: &[Vec<f64>],
    channel_names: &[String],
    units: &[String],
    sampling_frequency_hz: f64,
) -> Result<(), CoreError> {
    if data.is_empty() || channel_names.len() != data.len() {
        return Err(CoreError::InvalidInput(
            "EDF export requires matching channels".into(),
        ));
    }
    if !(sampling_frequency_hz > 0.0 && sampling_frequency_hz.is_finite()) {
        return Err(CoreError::InvalidInput("invalid sampling frequency".into()));
    }
    let n_samples = data[0].len();
    if n_samples == 0 || data.iter().any(|row| row.len() != n_samples) {
        return Err(CoreError::InvalidInput(
            "EDF export requires equal-length channels".into(),
        ));
    }
    let n_channels = data.len();
    if n_channels > MAX_CHANNELS {
        return Err(CoreError::InvalidInput("too many EDF channels".into()));
    }
    let record_duration = n_samples as f64 / sampling_frequency_hz;
    let header_bytes = 256 + SIGNAL_HEADER * n_channels;

    let mut ranges = Vec::with_capacity(n_channels);
    for row in data {
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for &v in row {
            if v.is_finite() {
                lo = lo.min(v);
                hi = hi.max(v);
            }
        }
        if !lo.is_finite() {
            lo = 0.0;
            hi = 1.0;
        }
        if hi <= lo {
            hi = lo + 1.0;
        }
        ranges.push((lo, hi));
    }

    let mut header = String::new();
    let mut push = |text: &str, width: usize| {
        let mut field = text.chars().take(width).collect::<String>();
        while field.len() < width {
            field.push(' ');
        }
        header.push_str(&field);
    };
    push("0", 8);
    push("NEUROLAB", 80);
    push("NEUROLAB export", 80);
    push("01.01.01", 8);
    push("00.00.00", 8);
    push(&format!("{header_bytes:08}"), 8);
    push("EDF+C", 44);
    push("1", 8);
    push(&format!("{record_duration:.3}"), 8);
    push(&format!("{n_channels}"), 4);
    debug_assert_eq!(header.len(), 256);

    let widths = [16usize, 80, 8, 8, 8, 8, 8, 80, 8, 32];
    for (index, _row) in data.iter().enumerate() {
        let (lo, hi) = ranges[index];
        let vals = [
            channel_names
                .get(index)
                .cloned()
                .unwrap_or_else(|| format!("CH{index:03}")),
            "NEUROLAB".to_string(),
            units.get(index).cloned().unwrap_or_else(|| "uV".into()),
            format!("{lo:.6}"),
            format!("{hi:.6}"),
            "-32768".to_string(),
            "32767".to_string(),
            String::new(),
            format!("{n_samples}"),
            String::new(),
        ];
        for (value, width) in vals.iter().zip(widths.iter()) {
            let mut field = value.chars().take(*width).collect::<String>();
            while field.len() < *width {
                field.push(' ');
            }
            header.push_str(&field);
        }
    }
    debug_assert_eq!(header.len(), header_bytes);

    let mut out = header.into_bytes();
    for (index, row) in data.iter().enumerate() {
        let (lo, hi) = ranges[index];
        let scale = 65535.0 / (hi - lo);
        for &v in row {
            let finite = if v.is_finite() { v } else { lo };
            let digital = ((finite - lo) * scale).round().clamp(0.0, 65535.0) as i32 - 32768;
            out.extend_from_slice(&(digital as i16).to_le_bytes());
        }
    }
    std::fs::write(path, out)?;
    Ok(())
}
