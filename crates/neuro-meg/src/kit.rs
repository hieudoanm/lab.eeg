//! Bounded reader for KIT/Yokogawa/Ricoh `.sqd`/`.con` continuous recordings.
//!
//! Parses the section directory (offset/size/count records), the system
//! header, channel records, calibration sensitivities, amplifier gain, and
//! acquisition conditions, then decodes the sample-major integer payload.
//! ExG gain refinement and `.mrk`/`.elp`/`.hsp` sidecars are not applied.

use crate::{MegChannel, MegEvent, MegSurfacePoint, MegWindow};
use neuro_core::CoreError;
use std::path::Path;

const DIR_INDEX_SYSTEM: usize = 1;
const DIR_INDEX_CHANNELS: usize = 4;
const DIR_INDEX_CALIBRATION: usize = 5;
const DIR_INDEX_AMP_FILTER: usize = 7;
const DIR_INDEX_ACQ_COND: usize = 8;
const DIR_INDEX_RAW_DATA: usize = 9;
const DIR_INDEX_DIG_POINTS: usize = 26;

const CHANNEL_MAGNETOMETER: i32 = 1;
const CHANNEL_MAGNETOMETER_REFERENCE: i32 = 0x101;
const CHANNEL_AXIAL_GRADIOMETER: i32 = 2;
const CHANNEL_AXIAL_GRADIOMETER_REFERENCE: i32 = 0x102;
const CHANNEL_PLANAR_GRADIOMETER: i32 = 3;
const CHANNEL_PLANAR_GRADIOMETER_REFERENCE: i32 = 0x103;
const CHANNEL_2ND_ORDER_AXIAL_GRADIOMETER: i32 = 4;
const CHANNEL_2ND_ORDER_AXIAL_GRADIOMETER_REFERENCE: i32 = 0x104;
const CHANNEL_TRIGGER: i32 = -1;
const CHANNEL_EEG: i32 = -2;
const CHANNEL_ECG: i32 = -3;
const CHANNEL_ETC: i32 = -4;
const CHANNEL_NULL: i32 = 0;

const GAINS: [f64; 8] = [1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0];
const MAX_CHANNELS: usize = 4096;
const MAX_SAMPLES: usize = 200_000_000;

#[derive(Clone, Debug)]
struct DirEntry {
    offset: usize,
    size: usize,
    count: usize,
}

pub struct KitInfo {
    pub n_channels: usize,
    pub sampling_frequency_hz: f64,
    pub n_samples: usize,
    pub channel_names: Vec<String>,
    pub channel_kinds: Vec<String>,
    pub channel_types: Vec<i32>,
    pub system_name: String,
    pub model_name: String,
    pub conv_factors: Vec<f64>,
    pub positions_mm: Vec<Option<Vec<f64>>>,
    pub orientations: Vec<Option<Vec<f64>>>,
    pub digitization_points: usize,
}

fn u32_at(bytes: &[u8], pos: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(pos..pos + 4)?.try_into().ok()?,
    ))
}

fn i32_at(bytes: &[u8], pos: usize) -> Option<i32> {
    Some(i32::from_le_bytes(
        bytes.get(pos..pos + 4)?.try_into().ok()?,
    ))
}

fn f64_at(bytes: &[u8], pos: usize) -> Option<f64> {
    Some(f64::from_le_bytes(
        bytes.get(pos..pos + 8)?.try_into().ok()?,
    ))
}

fn read_name(bytes: &[u8], pos: usize, n: usize) -> String {
    let end = pos + n.min(bytes.len().saturating_sub(pos));
    let raw = &bytes[pos..end];
    let stop = raw.iter().position(|b| *b == 0).unwrap_or(raw.len());
    String::from_utf8_lossy(&raw[..stop]).trim().to_string()
}

fn read_dirs(bytes: &[u8]) -> Result<Vec<DirEntry>, CoreError> {
    if bytes.len() < 16 {
        return Err(CoreError::InvalidInput("KIT file too small".into()));
    }
    let count = u32_at(bytes, 12).unwrap_or(0) as usize;
    if count == 0 || count > 1024 {
        return Err(CoreError::InvalidInput("invalid KIT directory".into()));
    }
    let mut dirs = Vec::with_capacity(count);
    for index in 0..count {
        let pos = index * 16;
        dirs.push(DirEntry {
            offset: u32_at(bytes, pos).unwrap_or(0) as usize,
            size: i32_at(bytes, pos + 4).unwrap_or(0).max(0) as usize,
            count: i32_at(bytes, pos + 12).unwrap_or(0).max(0) as usize,
        });
    }
    Ok(dirs)
}

fn channel_name_nchar(channel_type: i32) -> Option<usize> {
    match channel_type {
        CHANNEL_MAGNETOMETER | CHANNEL_AXIAL_GRADIOMETER => Some(6),
        CHANNEL_TRIGGER => Some(32),
        CHANNEL_EEG => Some(8),
        CHANNEL_ECG | CHANNEL_ETC => Some(32),
        _ => None,
    }
}

fn is_meg_type(channel_type: i32) -> bool {
    matches!(
        channel_type,
        CHANNEL_MAGNETOMETER
            | CHANNEL_MAGNETOMETER_REFERENCE
            | CHANNEL_AXIAL_GRADIOMETER
            | CHANNEL_AXIAL_GRADIOMETER_REFERENCE
            | CHANNEL_PLANAR_GRADIOMETER
            | CHANNEL_PLANAR_GRADIOMETER_REFERENCE
            | CHANNEL_2ND_ORDER_AXIAL_GRADIOMETER
            | CHANNEL_2ND_ORDER_AXIAL_GRADIOMETER_REFERENCE
    )
}

fn kind_name(channel_type: i32) -> &'static str {
    match channel_type {
        CHANNEL_MAGNETOMETER | CHANNEL_MAGNETOMETER_REFERENCE => "magnetometer",
        CHANNEL_AXIAL_GRADIOMETER | CHANNEL_AXIAL_GRADIOMETER_REFERENCE => "axial_gradiometer",
        CHANNEL_PLANAR_GRADIOMETER | CHANNEL_PLANAR_GRADIOMETER_REFERENCE => "planar_gradiometer",
        CHANNEL_2ND_ORDER_AXIAL_GRADIOMETER | CHANNEL_2ND_ORDER_AXIAL_GRADIOMETER_REFERENCE => {
            "second_order_gradiometer"
        }
        CHANNEL_TRIGGER => "trigger",
        CHANNEL_EEG => "eeg",
        CHANNEL_ECG => "ecg",
        CHANNEL_ETC => "misc",
        _ => "unknown",
    }
}

/// KIT stores MEG coil position `(x, y, z)` in metres and orientation as
/// spherical angles `(theta, phi)` in the ALS frame. MNE converts to a
/// Neuromag-like RAS frame with `als_ras = [[0,-1,0],[1,0,0],[0,0,1]]` and
/// derives the coil normal from the angles. We keep the position in mm and
/// the coil normal as the orientation.
fn coil_geometry(loc: &[f64]) -> Option<(Vec<f64>, Vec<f64>)> {
    if loc.len() < 5 {
        return None;
    }
    let (x, y, z) = (loc[0], loc[1], loc[2]);
    if !(x.is_finite() && y.is_finite() && z.is_finite()) || x * x + y * y + z * z < 1e-12 {
        return None;
    }
    let theta = loc[3].to_radians();
    let phi = loc[4].to_radians();
    let mut normal = vec![
        theta.sin() * phi.cos(),
        theta.sin() * phi.sin(),
        theta.cos(),
    ];
    let norm = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    if norm <= 0.0 {
        return None;
    }
    for v in &mut normal {
        *v /= norm;
    }
    // als_ras rotation: (x, y, z) -> (-y, x, z)
    let rotate = |v: &[f64]| vec![-v[1], v[0], v[2]];
    Some((
        rotate(&[x, y, z]).iter().map(|v| v * 1000.0).collect(),
        rotate(&normal),
    ))
}

pub fn inspect_kit(path: &Path) -> Result<KitInfo, CoreError> {
    let bytes = std::fs::read(path)?;
    let dirs = read_dirs(&bytes)?;
    let get = |index: usize| -> Result<&DirEntry, CoreError> {
        dirs.get(index)
            .ok_or_else(|| CoreError::InvalidInput("KIT section missing".into()))
    };
    let system = get(DIR_INDEX_SYSTEM)?;
    let mut pos = system.offset;
    let version = i32_at(&bytes, pos).unwrap_or(0);
    let revision = i32_at(&bytes, pos + 4).unwrap_or(0);
    if version < 2 || (version == 2 && revision < 3) {
        return Err(CoreError::InvalidInput(format!(
            "unsupported KIT format V{version}R{revision:03}"
        )));
    }
    let sysid = i32_at(&bytes, pos + 8).unwrap_or(0);
    let system_name = read_name(&bytes, pos + 12, 128);
    let model_name = read_name(&bytes, pos + 140, 128);
    pos += 268;
    let n_channels = i32_at(&bytes, pos).unwrap_or(0);
    if n_channels <= 0 || n_channels as usize > MAX_CHANNELS {
        return Err(CoreError::InvalidInput("invalid KIT channel count".into()));
    }
    let n_channels = n_channels as usize;
    pos += 4 + 256 + 8 + 12 + 4 + 12 + 4 + 12 + 4 + 12 + 4 + 116;
    // adc_range: i32 for V2R<=3 else f64; then 3xi32 adc params
    let adc_range = if version < 2 || (version == 2 && revision <= 3) {
        i32_at(&bytes, pos).unwrap_or(0) as f64
    } else {
        f64_at(&bytes, pos).unwrap_or(0.0)
    };
    pos += if version < 2 || (version == 2 && revision <= 3) {
        4
    } else {
        8
    };
    let adc_allocated = i32_at(&bytes, pos + 4).unwrap_or(0);
    let adc_stored = i32_at(&bytes, pos + 8).unwrap_or(0);
    if adc_range <= 0.0 || adc_allocated % 8 != 0 || adc_stored <= 0 {
        return Err(CoreError::InvalidInput("invalid KIT ADC parameters".into()));
    }

    // Channel records
    let channels_dir = get(DIR_INDEX_CHANNELS)?;
    let chan_size = channels_dir.size.max(4);
    let mut channel_names = Vec::with_capacity(n_channels);
    let mut channel_kinds = Vec::with_capacity(n_channels);
    let mut channel_types = Vec::with_capacity(n_channels);
    let mut is_meg = Vec::with_capacity(n_channels);
    let mut positions_mm = vec![None; n_channels];
    let mut orientations = vec![None; n_channels];
    for index in 0..n_channels {
        let base = channels_dir.offset + chan_size * index;
        let channel_type = i32_at(&bytes, base).unwrap_or(CHANNEL_NULL);
        // System 52 mislabeled reference channels as NULL.
        let channel_type = if sysid == 52 && index < 160 && channel_type == CHANNEL_NULL {
            CHANNEL_MAGNETOMETER_REFERENCE
        } else {
            channel_type
        };
        let mut name = String::new();
        if is_meg_type(channel_type) {
            // loc: 5 x f64 (x, y, z, theta, phi), then 16 misc bytes, then name
            let loc: Vec<f64> = (0..5)
                .filter_map(|k| f64_at(&bytes, base + 4 + k * 8))
                .collect();
            if let Some((position, orientation)) = coil_geometry(&loc) {
                positions_mm[index] = Some(position);
                orientations[index] = Some(orientation);
            }
            if let Some(nchar) = channel_name_nchar(channel_type) {
                let name_pos = base + 4 + 5 * 8 + 16;
                name = read_name(&bytes, name_pos, nchar);
            }
        } else if matches!(
            channel_type,
            CHANNEL_TRIGGER | CHANNEL_EEG | CHANNEL_ECG | CHANNEL_ETC
        ) {
            let name_pos = base + 4 + 4 + 4;
            if let Some(nchar) = channel_name_nchar(channel_type) {
                name = read_name(&bytes, name_pos, nchar);
            }
        }
        if name.is_empty() {
            name = if is_meg_type(channel_type) {
                format!("MEG {:03}", index + 1)
            } else {
                format!("CH{:03}", index + 1)
            };
        }
        channel_names.push(name);
        channel_kinds.push(kind_name(channel_type).to_string());
        channel_types.push(channel_type);
        is_meg.push(is_meg_type(channel_type));
    }

    // Calibration sensitivities: (offset, gain) f64 pairs per channel
    let calib = get(DIR_INDEX_CALIBRATION)?;
    let mut channel_gain = vec![1.0f64; n_channels];
    for index in 0..n_channels {
        let pos = calib.offset + index * 16 + 8;
        if let Some(gain) = f64_at(&bytes, pos) {
            if gain != 0.0 {
                channel_gain[index] = gain;
            }
        }
    }

    // Amplifier gain
    let amp_dir = get(DIR_INDEX_AMP_FILTER)?;
    let amp_data = i32_at(&bytes, amp_dir.offset).unwrap_or(0);
    let fll_type = i32_at(&bytes, system.offset + 564).unwrap_or(0);
    let amp_gain = if fll_type >= 100 {
        let g1 = ((amp_data & 0x0000_7000) >> 12) as usize;
        let g2 = ((amp_data & 0x7000_0000) >> 28) as usize;
        let g3 = ((amp_data & 0x0700_0000) >> 24) as usize;
        GAINS.get(g1).copied().unwrap_or(1.0)
            * GAINS.get(g2).copied().unwrap_or(1.0)
            * GAINS.get(g3).copied().unwrap_or(1.0)
    } else {
        let input = ((amp_data & 0x1800) >> 11) as usize;
        let output = (amp_data & 0x0007) as usize;
        GAINS.get(input).copied().unwrap_or(1.0) * GAINS.get(output).copied().unwrap_or(1.0)
    };
    if amp_gain <= 0.0 {
        return Err(CoreError::InvalidInput("invalid KIT amplifier gain".into()));
    }

    // Acquisition conditions
    let acq = get(DIR_INDEX_ACQ_COND)?;
    let acq_type = i32_at(&bytes, acq.offset).unwrap_or(0);
    if acq_type != 1 {
        return Err(CoreError::InvalidInput(
            "only continuous KIT recordings are supported".into(),
        ));
    }
    let sfreq = f64_at(&bytes, acq.offset + 4).unwrap_or(0.0);
    let n_samples = i32_at(&bytes, acq.offset + 16).unwrap_or(0);
    if !(sfreq > 0.0 && sfreq.is_finite()) || n_samples <= 0 {
        return Err(CoreError::InvalidInput(
            "invalid KIT acquisition info".into(),
        ));
    }
    let n_samples = n_samples as usize;
    if n_samples > MAX_SAMPLES {
        return Err(CoreError::InvalidInput("KIT recording too large".into()));
    }

    // Conversion factors (ExG gain refinement not applied)
    let ad_to_volt = adc_range / 2.0f64.powi(adc_stored);
    let conv_factors: Vec<f64> = (0..n_channels)
        .map(|index| {
            if is_meg[index] {
                ad_to_volt / amp_gain * channel_gain[index]
            } else {
                ad_to_volt
            }
        })
        .collect();

    // Digitization points (Polhemus headshape / fiducial / HPI coils):
    // records of an 8-byte name plus 3 f64 coordinates.
    let mut digitization_points = 0usize;
    if let Some(dig) = dirs.get(DIR_INDEX_DIG_POINTS) {
        for index in 0..dig.count.min(4096) {
            let pos = dig.offset + index * 32;
            if pos + 32 > bytes.len() {
                break;
            }
            if !read_name(&bytes, pos, 8).is_empty() {
                digitization_points += 1;
            }
        }
    }

    Ok(KitInfo {
        n_channels,
        sampling_frequency_hz: sfreq,
        n_samples,
        channel_names,
        channel_kinds,
        channel_types,
        system_name,
        model_name,
        conv_factors,
        positions_mm,
        orientations,
        digitization_points,
    })
}

fn raw_dir_entry(bytes: &[u8]) -> Result<DirEntry, CoreError> {
    let dirs = read_dirs(bytes)?;
    dirs.get(DIR_INDEX_RAW_DATA)
        .cloned()
        .ok_or_else(|| CoreError::InvalidInput("KIT raw data section missing".into()))
}

fn decode_sample(bytes: &[u8], offset: usize, bytes_per_sample: usize) -> Option<f64> {
    match bytes_per_sample {
        2 => Some(i16::from_le_bytes(bytes.get(offset..offset + 2)?.try_into().ok()?) as f64),
        4 => Some(i32::from_le_bytes(bytes.get(offset..offset + 4)?.try_into().ok()?) as f64),
        _ => None,
    }
}

pub fn read_window(
    path: &Path,
    start_s: f64,
    end_s: f64,
    channel_names: &[String],
    max_samples: usize,
) -> Result<MegWindow, CoreError> {
    let info = inspect_kit(path)?;
    let duration = info.n_samples as f64 / info.sampling_frequency_hz;
    if start_s < 0.0 || end_s <= start_s || end_s > duration + 1e-9 {
        return Err(CoreError::InvalidInput("invalid KIT window".into()));
    }
    let mut indices = Vec::new();
    for name in channel_names {
        let index = info
            .channel_names
            .iter()
            .position(|candidate| candidate == name)
            .ok_or_else(|| CoreError::InvalidInput(format!("unknown KIT channel: {name}")))?;
        if !indices.contains(&index) {
            indices.push(index);
        }
    }
    if indices.is_empty() {
        indices = (0..info.n_channels).collect();
    }
    let bytes = std::fs::read(path)?;
    let raw = raw_dir_entry(&bytes)?;
    // Recompute bytes-per-sample from the ADC allocation stored in system header.
    let system = read_dirs(&bytes)?[DIR_INDEX_SYSTEM].offset;
    let version = i32_at(&bytes, system).unwrap_or(0);
    let revision = i32_at(&bytes, system + 4).unwrap_or(0);
    let range_size = if version < 2 || (version == 2 && revision <= 3) {
        4
    } else {
        8
    };
    let adc_allocated = i32_at(&bytes, system + 716 + range_size + 4).unwrap_or(16);
    let bytes_per_sample = (adc_allocated.max(16) / 8) as usize;
    if bytes_per_sample != 2 && bytes_per_sample != 4 {
        return Err(CoreError::InvalidInput(
            "unsupported KIT sample width".into(),
        ));
    }
    let start = ((start_s * info.sampling_frequency_hz).round() as usize).min(info.n_samples);
    let end = ((end_s * info.sampling_frequency_hz).round() as usize)
        .min(info.n_samples)
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
        times.push(sample as f64 / info.sampling_frequency_hz);
        for (output, &channel) in indices.iter().enumerate() {
            let offset = raw.offset + (sample * info.n_channels + channel) * bytes_per_sample;
            let raw_value = decode_sample(&bytes, offset, bytes_per_sample).unwrap_or(0.0);
            data[output].push(raw_value * info.conv_factors[channel]);
        }
        sample += factor;
    }
    Ok(MegWindow {
        times_s: times,
        data,
        effective_sampling_frequency_hz: info.sampling_frequency_hz / factor as f64,
    })
}

/// Decode KIT trigger events from the first 8 misc-type channels
/// (TRIGGER/ETC/NULL), matching MNE's default `stim=">"`, `slope="-"`,
/// `stimthresh=1`, `stim_code="binary"`: a channel counts as active when its
/// converted value is below 1.0, and the event code is the sum of `2^i` over
/// active channels. An event is emitted on each rising edge (0 -> nonzero).
pub fn read_events(path: &Path) -> Result<Vec<MegEvent>, CoreError> {
    let info = inspect_kit(path)?;
    // Misc-type channels in file order, capped at 8 like MNE's default picks.
    let stim: Vec<usize> = info
        .channel_types
        .iter()
        .enumerate()
        .filter(|(_, &t)| matches!(t, CHANNEL_TRIGGER | CHANNEL_ETC | CHANNEL_NULL))
        .map(|(index, _)| index)
        .take(8)
        .collect();
    if stim.is_empty() {
        return Ok(Vec::new());
    }
    let bytes = std::fs::read(path)?;
    let raw = raw_dir_entry(&bytes)?;
    let system = read_dirs(&bytes)?[DIR_INDEX_SYSTEM].offset;
    let version = i32_at(&bytes, system).unwrap_or(0);
    let revision = i32_at(&bytes, system + 4).unwrap_or(0);
    let range_size = if version < 2 || (version == 2 && revision <= 3) {
        4
    } else {
        8
    };
    let adc_allocated = i32_at(&bytes, system + 716 + range_size + 4).unwrap_or(16);
    let bytes_per_sample = (adc_allocated.max(16) / 8) as usize;
    if bytes_per_sample != 2 && bytes_per_sample != 4 {
        return Err(CoreError::InvalidInput(
            "unsupported KIT sample width".into(),
        ));
    }
    let frame = info.n_channels * bytes_per_sample;
    if frame == 0 {
        return Ok(Vec::new());
    }
    let mut events = Vec::new();
    let mut previous = 0u32;
    let block_samples = 8192usize;
    let mut sample = 0usize;
    while sample < info.n_samples {
        let take = block_samples.min(info.n_samples - sample);
        let block_bytes = take * frame;
        let start_byte = raw.offset + sample * frame;
        let end_byte = (start_byte + block_bytes).min(bytes.len());
        if start_byte >= bytes.len() {
            break;
        }
        let block = &bytes[start_byte..end_byte];
        for s in 0..take {
            let base = s * frame;
            let mut code = 0u32;
            for (bit, &channel) in stim.iter().enumerate() {
                let offset = base + channel * bytes_per_sample;
                if let Some(raw_value) = decode_sample(block, offset, bytes_per_sample) {
                    if raw_value * info.conv_factors[channel] < 1.0 {
                        code |= 1 << bit;
                    }
                }
            }
            let global = sample + s;
            if code != 0 && code != previous {
                events.push(MegEvent {
                    sample: global,
                    time_s: global as f64 / info.sampling_frequency_hz,
                    code: Some(code.to_string()),
                    value: Some(code as i64),
                    description: Some(format!("trigger {code}")),
                });
            }
            previous = code;
        }
        sample += take;
    }
    Ok(events)
}

pub fn metadata(path: &Path) -> Result<(Vec<MegChannel>, KitInfo), CoreError> {
    let info = inspect_kit(path)?;
    let channels = info
        .channel_names
        .iter()
        .enumerate()
        .map(|(index, name)| MegChannel {
            name: name.clone(),
            kind: info.channel_kinds[index].clone(),
            unit: if info.channel_kinds[index] == "eeg" {
                "volt".into()
            } else if is_meg_type_name(&info.channel_kinds[index]) {
                "tesla".into()
            } else {
                "other".into()
            },
            unit_mul: 1.0,
            group_name: None,
            status: "good".into(),
            coil_type: None,
            position_mm: info.positions_mm.get(index).cloned().flatten(),
            orientation: info.orientations.get(index).cloned().flatten(),
        })
        .collect();
    Ok((channels, info))
}

/// Parse external Polhemus FastScan text exports (`.elp` fiducial/HPI
/// points and `.hsp` headshape surface points, including `.txt` variants)
/// sitting next to a `.sqd`/`.con` file. MNE does not read these files —
/// KIT digitization normally lives inside the `.con` — but vendors do
/// ship them, so we surface them as metadata. Each line contributes its
/// first three parseable floats as an `(x, y, z)` position; the export
/// unit is stored as-is in `position_m` (Polhemus exports are typically
/// already in metres for KIT systems). Returns `(elp_points, hsp_points)`.
pub fn sidecar_points(dir: &Path) -> (Vec<MegSurfacePoint>, Vec<MegSurfacePoint>) {
    let mut elp = Vec::new();
    let mut hsp = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return (elp, hsp);
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let lower = name.to_lowercase();
        let is_elp = lower.contains(".elp");
        let is_hsp = lower.contains(".hsp");
        if !is_elp && !is_hsp {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        if text.len() > 64 * 1024 * 1024 {
            continue;
        }
        let target = if is_elp { &mut elp } else { &mut hsp };
        for (line_index, line) in text.lines().enumerate() {
            if target.len() >= 50_000 {
                break;
            }
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let values: Vec<f64> = line
                .split_whitespace()
                .filter_map(|token| token.parse::<f64>().ok())
                .collect();
            if values.len() < 3 || values.iter().take(3).any(|v| !v.is_finite()) {
                continue;
            }
            target.push(MegSurfacePoint {
                name: Some(format!("{name}:{line_index}")),
                position_m: values[..3].to_vec(),
            });
        }
    }
    (elp, hsp)
}

fn is_meg_type_name(kind: &str) -> bool {
    matches!(
        kind,
        "magnetometer" | "axial_gradiometer" | "planar_gradiometer" | "second_order_gradiometer"
    )
}
