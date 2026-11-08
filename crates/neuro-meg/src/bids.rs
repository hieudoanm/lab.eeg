//! BIDS MEG dataset support.
//!
//! Locates `*_meg` data files under `meg/` directories, reads the
//! `*_channels.tsv` and `*_events.tsv` sidecars, and delegates raw decoding
//! to the FIF/KIT/CTF/OPM readers. BIDS is an organisational convention —
//! this module adds metadata, not a new binary format.

use crate::{MegEvent, MegMetadata, MegWindow};
use neuro_core::CoreError;
use std::path::{Path, PathBuf};

pub struct BidsMegFile {
    pub data_path: PathBuf,
    pub channels_path: Option<PathBuf>,
    pub events_path: Option<PathBuf>,
}

pub struct BidsMeg {
    pub files: Vec<BidsMegFile>,
}

impl BidsMeg {
    pub fn data_path(&self) -> Option<&Path> {
        self.files.first().map(|file| file.data_path.as_path())
    }
}

fn is_data_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
        return false;
    };
    let lower = name.to_lowercase();
    lower.contains("_meg.")
        && [".fif", ".sqd", ".con", ".bin", ".ds"]
            .iter()
            .any(|ext| lower.ends_with(ext))
}

fn find_in(dir: &Path, predicate: &dyn Fn(&Path) -> bool) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                out.extend(find_in(&path, predicate));
            } else if predicate(&path) {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

pub fn find_bids_meg(root: &Path) -> Option<BidsMeg> {
    // Only treat directories that look like BIDS datasets.
    let has_description = root.join("dataset_description.json").exists();
    let candidates = find_in(root, &is_data_file);
    if candidates.is_empty() {
        return None;
    }
    if !has_description {
        // Require at least one meg/ directory ancestor to avoid false positives.
        let in_meg_dir = candidates.iter().any(|path| {
            path.ancestors()
                .any(|ancestor| ancestor.file_name().is_some_and(|n| n == "meg"))
        });
        if !in_meg_dir {
            return None;
        }
    }
    let files = candidates
        .into_iter()
        .map(|data_path| {
            let parent = data_path.parent();
            // BIDS sidecars drop the modality suffix: <entities>_meg.<ext>
            // pairs with <entities>_channels.tsv / <entities>_events.tsv.
            let stem = data_path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("")
                .trim_end_matches("_meg")
                .trim_end_matches("_MEG");
            let sidecar = |suffixes: &[&str]| -> Option<PathBuf> {
                let parent = parent?;
                suffixes
                    .iter()
                    .map(|suffix| parent.join(format!("{stem}{suffix}")))
                    .find(|path| path.exists())
            };
            let channels_path = sidecar(&["_channels.tsv"]);
            let events_path = sidecar(&["_events.tsv", "_events.tsv.gz"]);
            BidsMegFile {
                data_path,
                channels_path,
                events_path,
            }
        })
        .collect();
    Some(BidsMeg { files })
}

#[derive(Clone, Debug)]
pub struct ChannelSidecar {
    pub name: String,
    pub kind: String,
    pub unit: String,
    pub status: String,
    pub description: Option<String>,
}

fn parse_tsv(text: &str) -> Vec<Vec<String>> {
    text.lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| {
            line.split('\t')
                .map(|cell| cell.trim().to_string())
                .collect()
        })
        .collect()
}

fn read_text_maybe_gz(path: &Path) -> Result<String, CoreError> {
    let bytes = std::fs::read(path)?;
    if path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
    {
        use std::io::Read;
        let mut decoder = flate2::read::GzDecoder::new(&bytes[..]);
        let mut text = String::new();
        decoder
            .read_to_string(&mut text)
            .map_err(|error| CoreError::InvalidInput(format!("invalid gzip TSV: {error}")))?;
        Ok(text)
    } else {
        String::from_utf8(bytes)
            .map_err(|_| CoreError::InvalidInput("TSV is not valid UTF-8".into()))
    }
}

pub fn parse_channels_tsv(path: &Path) -> Result<Vec<ChannelSidecar>, CoreError> {
    let text = read_text_maybe_gz(path)?;
    let rows = parse_tsv(&text);
    if rows.len() < 2 {
        return Ok(Vec::new());
    }
    let header = &rows[0];
    let col = |name: &str| header.iter().position(|h| h == name);
    let (name_i, type_i, unit_i, status_i, desc_i) = (
        col("name").unwrap_or(0),
        col("type").unwrap_or(1),
        col("units").or_else(|| col("unit")).unwrap_or(2),
        col("status").unwrap_or(usize::MAX),
        col("description").unwrap_or(usize::MAX),
    );
    Ok(rows[1..]
        .iter()
        .map(|row| ChannelSidecar {
            name: row.get(name_i).cloned().unwrap_or_default(),
            kind: row.get(type_i).cloned().unwrap_or_default(),
            unit: row.get(unit_i).cloned().unwrap_or_default(),
            status: if status_i == usize::MAX {
                "good".into()
            } else {
                row.get(status_i).cloned().unwrap_or_else(|| "good".into())
            },
            description: if desc_i == usize::MAX {
                None
            } else {
                row.get(desc_i)
                    .cloned()
                    .filter(|d| !d.is_empty() && d != "n/a")
            },
        })
        .filter(|row| !row.name.is_empty())
        .collect())
}

pub fn parse_events_tsv(path: &Path, sfreq: f64) -> Result<Vec<MegEvent>, CoreError> {
    let text = read_text_maybe_gz(path)?;
    let rows = parse_tsv(&text);
    if rows.len() < 2 {
        return Ok(Vec::new());
    }
    let header = &rows[0];
    let onset_i = header.iter().position(|h| h == "onset").unwrap_or(0);
    let trial_i = header
        .iter()
        .position(|h| h == "trial_type" || h == "trial")
        .unwrap_or(2);
    let mut events = Vec::new();
    for row in &rows[1..] {
        let onset: f64 = row
            .get(onset_i)
            .and_then(|value| value.parse().ok())
            .unwrap_or(0.0);
        if !onset.is_finite() || onset < 0.0 {
            continue;
        }
        let sample = (onset * sfreq).round() as usize;
        let code = row
            .get(trial_i)
            .cloned()
            .filter(|value| !value.is_empty() && value != "n/a");
        events.push(MegEvent {
            sample,
            time_s: onset,
            code: code.clone(),
            value: None,
            description: code,
        });
    }
    Ok(events)
}

/// Locate `coordsystem.json` in a BIDS dataset and extract the MEG
/// coordinate frame, digitization counts, and digitized surface points
/// (positions in metres per the BIDS coordsystem convention).
pub fn parse_coordsystem(root: &Path) -> Option<(String, usize, Vec<crate::MegSurfacePoint>)> {
    let path = find_in(root, &|path| {
        path.file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| name == "coordsystem.json")
    })
    .into_iter()
    .next()?;
    let text = std::fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&text).ok()?;
    let frame = json
        .get("MEGCoordinateFrame")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown")
        .to_lowercase();
    let mut digitization = 0usize;
    let mut points = Vec::new();
    for key in ["HeadDigitization", "Digitization"] {
        if let Some(array) = json.get(key).and_then(|value| value.as_array()) {
            digitization += array.len();
            for entry in array {
                let position_m: Vec<f64> = ["x", "y", "z"]
                    .iter()
                    .filter_map(|axis| entry.get(*axis).and_then(|v| v.as_f64()))
                    .collect();
                if position_m.len() == 3 {
                    points.push(crate::MegSurfacePoint {
                        name: entry
                            .get("name")
                            .and_then(|v| v.as_str())
                            .map(str::to_string),
                        position_m,
                    });
                }
            }
        }
    }
    Some((frame, digitization, points))
}

fn kind_from_bids(kind: &str) -> String {
    match kind.to_uppercase().as_str() {
        "MEGMAG" | "MEG" | "MEGREF" => "meg".into(),
        "MEGPLANAR1" | "MEGPLANAR2" | "MEGGRAD" | "MEGGRADAXIAL" | "MEGGRADPLANAR" => {
            "gradiometer".into()
        }
        "EEG" => "eeg".into(),
        "EOG" => "eog".into(),
        "ECG" | "EEGECG" => "ecg".into(),
        "STIM" | "TRIGGER" | "UADC" => "stim".into(),
        "MISC" => "misc".into(),
        _ => "other".into(),
    }
}

fn unit_from_bids(unit: &str) -> String {
    match unit.to_uppercase().as_str() {
        "T" => "tesla".into(),
        "T/M" => "tesla_per_metre".into(),
        "V" => "volt".into(),
        "MV" => "millivolt".into(),
        "UV" => "microvolt".into(),
        _ => "other".into(),
    }
}

/// Per-run inspection results aligned with `find_bids_meg().files`:
/// data path, duration (s), sampling rate (Hz), and channel names.
struct RunInfo {
    duration_s: f64,
    sfreq: f64,
    channel_names: Vec<String>,
}

fn run_infos(root: &Path) -> Result<Vec<RunInfo>, CoreError> {
    let bids =
        find_bids_meg(root).ok_or_else(|| CoreError::NotFound("no BIDS MEG data found".into()))?;
    bids.files
        .iter()
        .map(|file| {
            let meta = crate::inspect_recording(&file.data_path)?;
            Ok(RunInfo {
                duration_s: meta.duration_seconds,
                sfreq: meta.sampling_frequency_hz,
                channel_names: meta.channels.iter().map(|c| c.name.clone()).collect(),
            })
        })
        .collect()
}

/// `(offset_s, duration_s)` of `recording` within the concatenated
/// multi-run timeline; `None` when the selector matches nothing.
fn run_span(root: &Path, recording: Option<&str>) -> Option<(f64, f64)> {
    let infos = run_infos(root).ok()?;
    let bids = find_bids_meg(root)?;
    let index = match recording {
        None | Some("") => 0,
        Some(selector) => {
            if let Ok(i) = selector.parse::<usize>() {
                i
            } else {
                let needle = selector.to_lowercase();
                bids.files.iter().position(|file| {
                    file.data_path
                        .file_name()
                        .and_then(|value| value.to_str())
                        .is_some_and(|name| name.to_lowercase().contains(&needle))
                })?
            }
        }
    };
    let offset: f64 = infos
        .iter()
        .take(index.min(infos.len()))
        .map(|i| i.duration_s)
        .sum();
    let duration = infos
        .get(index.min(infos.len().saturating_sub(1)))?
        .duration_s;
    Some((offset, duration))
}

/// Cumulative run offset (seconds) for `recording` within the
/// concatenated multi-run timeline; 0 for the first/only run.
pub fn run_offset_for(root: &Path, recording: Option<&str>) -> f64 {
    run_span(root, recording)
        .map(|(offset, _)| offset)
        .unwrap_or(0.0)
}

/// `(offset_s, duration_s)` of `recording` on the concatenated timeline.
pub fn run_span_for(root: &Path, recording: Option<&str>) -> Option<(f64, f64)> {
    run_span(root, recording)
}

/// Read a window across ALL `*_meg` runs concatenated into one continuous
/// timeline. Runs must share a sampling rate (within 1e-6 Hz); channel
/// sets are resolved by name against the first run's channels when
/// `channel_names` is empty. Event times in `MegMetadata` for BIDS
/// datasets are already expressed on this concatenated timeline.
pub fn read_window_concat(
    root: &Path,
    start_s: f64,
    end_s: f64,
    channel_names: &[String],
    max_samples: usize,
    compensation: Option<i32>,
    apply_projectors: bool,
) -> Result<MegWindow, CoreError> {
    let infos = run_infos(root)?;
    if infos.is_empty() {
        return Err(CoreError::NotFound("no BIDS MEG data found".into()));
    }
    let sfreq = infos[0].sfreq;
    if infos
        .iter()
        .any(|info| (info.sfreq - sfreq).abs() > 1e-6 || info.sfreq <= 0.0)
    {
        return Err(CoreError::InvalidInput(
            "BIDS runs have inconsistent sampling rates".into(),
        ));
    }
    let total_duration: f64 = infos.iter().map(|info| info.duration_s).sum();
    if start_s < 0.0 || end_s <= start_s || end_s > total_duration + 1e-9 {
        return Err(CoreError::InvalidInput("invalid BIDS window".into()));
    }
    let names: Vec<String> = if channel_names.is_empty() {
        infos[0].channel_names.clone()
    } else {
        channel_names.to_vec()
    };
    if names.is_empty() {
        return Err(CoreError::InvalidInput("no channels to read".into()));
    }
    let bids =
        find_bids_meg(root).ok_or_else(|| CoreError::NotFound("no BIDS MEG data found".into()))?;
    // Reconcile channel sets across runs: every requested channel must
    // exist in every run, otherwise fail with the run and missing names
    // rather than silently zero-filling or dropping columns.
    for (index, info) in infos.iter().enumerate() {
        let missing: Vec<&str> = names
            .iter()
            .filter(|name| !info.channel_names.iter().any(|c| c == *name))
            .map(String::as_str)
            .collect();
        if !missing.is_empty() {
            let label = bids
                .files
                .get(index)
                .and_then(|file| file.data_path.file_name())
                .and_then(|value| value.to_str())
                .unwrap_or("run");
            return Err(CoreError::InvalidInput(format!(
                "BIDS run {label} is missing channels: {}",
                missing.join(", ")
            )));
        }
    }
    let total_native: usize = infos
        .iter()
        .map(|info| (info.duration_s * sfreq).round() as usize)
        .sum();
    let mut factor = 1usize;
    while total_native / factor > max_samples.max(1) {
        factor *= 2;
    }
    let mut times = Vec::new();
    let mut data = vec![Vec::new(); names.len()];
    let mut offset_s = 0.0;
    for (index, file) in bids.files.iter().enumerate() {
        let info = &infos[index];
        let local_start = (start_s - offset_s).max(0.0);
        let local_end = (end_s - offset_s).min(info.duration_s);
        if local_end > local_start {
            let window = crate::read_window(
                &file.data_path,
                local_start,
                local_end,
                &names,
                usize::MAX,
                None,
                compensation,
                apply_projectors,
            )?;
            for (t_index, &local_t) in window.times_s.iter().enumerate() {
                // Decimate on the absolute concatenated sample index so the
                // phase is consistent across run boundaries.
                let global_sample = ((offset_s + local_t) * sfreq).round() as usize;
                if global_sample % factor != 0 {
                    continue;
                }
                times.push(offset_s + local_t);
                for (channel, row) in data.iter_mut().enumerate() {
                    row.push(window.data[channel][t_index]);
                }
            }
        }
        offset_s += info.duration_s;
    }
    Ok(MegWindow {
        times_s: times,
        data,
        effective_sampling_frequency_hz: sfreq / factor as f64,
    })
}

/// Inspect a BIDS MEG dataset directory: resolve the raw file, delegate to
/// the format reader, then overlay channels.tsv / events.tsv metadata.
pub fn inspect_recording(root: &Path) -> Result<MegMetadata, CoreError> {
    let bids =
        find_bids_meg(root).ok_or_else(|| CoreError::NotFound("no BIDS MEG data found".into()))?;
    let first = bids
        .data_path()
        .ok_or_else(|| CoreError::NotFound("no BIDS MEG data found".into()))?;
    // Delegate to the crate-level dispatcher for the first resolved data
    // path; additional *_meg files contribute sidecar events only.
    let mut metadata = crate::inspect_recording(first)?;
    metadata.format = "bids_meg".into();
    // Duration and event/sample timing are expressed on the concatenated
    // multi-run timeline so `recording = "all"` window reads line up.
    let infos = run_infos(root).unwrap_or_default();
    if !infos.is_empty() {
        metadata.duration_seconds = infos.iter().map(|info| info.duration_s).sum();
    }
    metadata.recordings = bids
        .files
        .iter()
        .filter_map(|file| {
            file.data_path
                .file_name()
                .and_then(|value| value.to_str())
                .map(str::to_string)
        })
        .collect();
    if let Ok(infos) = run_infos(root) {
        metadata.recording_channels = bids
            .files
            .iter()
            .zip(infos.iter())
            .filter_map(|(file, info)| {
                let stem = file
                    .data_path
                    .file_stem()
                    .and_then(|value| value.to_str())?
                    .trim_end_matches("_meg")
                    .trim_end_matches("_MEG")
                    .to_string();
                Some((stem, info.channel_names.clone()))
            })
            .collect();
    }
    if let Some((frame, digitization, points)) = parse_coordsystem(root) {
        metadata.coordinate_frame = frame;
        metadata.digitization_points = digitization;
        metadata.surface_points = points;
    }
    // Merge channels.tsv overlays across every run: kind/unit/description
    // come from the first sidecar that names a channel, while a channel is
    // bad if ANY run's sidecar marks it bad.
    {
        let mut by_name: std::collections::HashMap<String, ChannelSidecar> =
            std::collections::HashMap::new();
        let mut bad_names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for file in &bids.files {
            let Some(channels_path) = &file.channels_path else {
                continue;
            };
            for row in parse_channels_tsv(channels_path)? {
                if row.status.eq_ignore_ascii_case("bad") {
                    bad_names.insert(row.name.clone());
                }
                by_name.entry(row.name.clone()).or_insert(row);
            }
        }
        if !by_name.is_empty() {
            for channel in &mut metadata.channels {
                if let Some(sidecar) = by_name.remove(&channel.name) {
                    channel.kind = kind_from_bids(&sidecar.kind);
                    channel.unit = unit_from_bids(&sidecar.unit);
                }
                if bad_names.contains(&channel.name) {
                    channel.status = "bad".into();
                }
            }
            metadata.bad_channel_names = metadata
                .channels
                .iter()
                .filter(|channel| channel.status == "bad")
                .map(|channel| channel.name.clone())
                .collect();
        }
    }
    // Merge events across every *_meg run's events sidecar. When more than
    // one recording exists, tag each event's description with its source
    // file stem so onsets from different runs stay distinguishable.
    let multi = bids.files.len() > 1;
    let mut events = Vec::new();
    let mut run_offset_s = 0.0f64;
    for (file_index, file) in bids.files.iter().enumerate() {
        let Some(events_path) = &file.events_path else {
            if let Some(info) = infos.get(file_index) {
                run_offset_s += info.duration_s;
            }
            continue;
        };
        let file_sfreq = infos
            .get(file_index)
            .map(|info| info.sfreq)
            .filter(|sfreq| *sfreq > 0.0)
            .unwrap_or(metadata.sampling_frequency_hz);
        let stem = file
            .data_path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .trim_end_matches("_meg")
            .trim_end_matches("_MEG")
            .to_string();
        for mut event in parse_events_tsv(events_path, file_sfreq)? {
            event.time_s += run_offset_s;
            event.sample += (run_offset_s * file_sfreq).round() as usize;
            if multi {
                let tag = format!("{stem}:{}", event.description.as_deref().unwrap_or("event"));
                event.description = Some(tag.clone());
                if event.code.is_none() {
                    event.code = Some(stem.clone());
                }
            }
            events.push(event);
        }
        if let Some(info) = infos.get(file_index) {
            run_offset_s += info.duration_s;
        }
    }
    if !events.is_empty() {
        metadata.events = events;
    }
    Ok(metadata)
}

/// Resolve the raw data path for window reads. `recording` selects among
/// multiple `*_meg` files: a numeric index or a case-insensitive substring
/// of the data file name; `None`/empty picks the first file.
pub fn data_path_for(root: &Path, recording: Option<&str>) -> Option<PathBuf> {
    let bids = find_bids_meg(root)?;
    let selected = match recording {
        None | Some("") => bids.files.first(),
        Some(selector) => {
            if let Ok(index) = selector.parse::<usize>() {
                bids.files.get(index)
            } else {
                let needle = selector.to_lowercase();
                bids.files.iter().find(|file| {
                    file.data_path
                        .file_name()
                        .and_then(|value| value.to_str())
                        .is_some_and(|name| name.to_lowercase().contains(&needle))
                })
            }
        }
    }?;
    Some(selected.data_path.clone())
}

/// Resolve the raw data path for window reads (first `*_meg` file).
pub fn data_path(root: &Path) -> Option<PathBuf> {
    data_path_for(root, None)
}
