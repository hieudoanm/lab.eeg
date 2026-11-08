//! Real-file validation harness.
//!
//! Generate reference values with MNE:
//!   reference/meg/packages/backend/.venv/bin/python scripts/validate_meg.py <recording> /tmp/ref.json
//!
//! Then run:
//!   NEURO_MEG_PATH=<recording> NEURO_MEG_VALIDATE_JSON=/tmp/ref.json \
//!     cargo test -p neuro-meg --test real_files -- --ignored
//!
//! FIF compares window values against MNE (tolerance 1e-3 relative).
//! CTF/KIT compare metadata only because our readers return raw
//! A/D counts while MNE applies vendor calibration — data-level parity
//! for those formats requires the calibration ports tracked as gaps.

use neuro_meg::{inspect_recording, read_window};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
struct Reference {
    sfreq: f64,
    n_channels: usize,
    channel_names: Vec<String>,
    duration_s: f64,
    window_channels: usize,
    window_samples: usize,
    window: Vec<Vec<f64>>,
}

fn fixture() -> Option<(PathBuf, Reference)> {
    let path = std::env::var("NEURO_MEG_PATH").ok()?;
    let json = std::env::var("NEURO_MEG_VALIDATE_JSON").ok()?;
    let text = std::fs::read_to_string(json).ok()?;
    Some((PathBuf::from(path), serde_json::from_str(&text).ok()?))
}

#[test]
#[ignore = "requires real recording + MNE reference JSON"]
fn real_recording_metadata_matches_mne() {
    let Some((path, reference)) = fixture() else {
        eprintln!("NEURO_MEG_PATH / NEURO_MEG_VALIDATE_JSON not set; skipping");
        return;
    };
    let metadata = inspect_recording(&path).expect("inspect failed");
    assert_eq!(metadata.channel_count, reference.n_channels);
    assert!(
        (metadata.sampling_frequency_hz - reference.sfreq).abs() < 0.01,
        "sfreq {} vs {}",
        metadata.sampling_frequency_hz,
        reference.sfreq
    );
    assert!(
        (metadata.duration_seconds - reference.duration_s).abs() < 0.5,
        "duration {} vs {}",
        metadata.duration_seconds,
        reference.duration_s
    );
    for (ours, theirs) in metadata
        .channels
        .iter()
        .take(reference.channel_names.len())
        .zip(reference.channel_names.iter())
    {
        assert_eq!(&ours.name, theirs, "channel name mismatch");
    }
}

#[test]
#[ignore = "requires real FIF recording + MNE reference JSON"]
fn fif_window_matches_mne() {
    let Some((path, reference)) = fixture() else {
        return;
    };
    if !path.to_string_lossy().ends_with(".fif") {
        eprintln!("not a .fif path; skipping data comparison");
        return;
    }
    let names: Vec<String> = reference.channel_names[..reference.window_channels].to_vec();
    let window = read_window(&path, 0.0, 0.5, &names, 100_000, None, None, false)
        .expect("read_window failed");
    assert_eq!(window.data.len(), reference.window_channels);
    let n = reference.window_samples.min(window.data[0].len());
    for ch in 0..reference.window_channels {
        for sample in 0..n {
            let ours = window.data[ch][sample];
            let theirs = reference.window[ch][sample];
            let tol = 1e-3f64.max(theirs.abs() * 1e-3);
            assert!(
                (ours - theirs).abs() <= tol,
                "ch{ch} sample{sample}: ours {ours} vs mne {theirs}"
            );
        }
    }
}

/// Write a synthetic FIF via our writer for external validation:
///   NEURO_FIF_OUT=/tmp/out.fif cargo test -p neuro-meg --test real_files fif_writer -- --ignored
/// Then read it with MNE (reference venv) and compare channels/sfreq/data.
#[test]
#[ignore = "writes a fixture for external MNE validation"]
fn fif_writer_emits_mne_readable_file() {
    let out = std::env::var("NEURO_FIF_OUT").expect("NEURO_FIF_OUT not set");
    let channels = vec![
        neuro_meg::fif::FifWriteChannel {
            name: "MEG001".into(),
            kind: 1,
            unit: 112,
            unit_mul: 0,
            cal: 1.0,
            range: 1.0,
            coil_type: 0,
            loc: [0.0; 12],
        },
        neuro_meg::fif::FifWriteChannel {
            name: "EEG001".into(),
            kind: 2,
            unit: 107,
            unit_mul: 0,
            cal: 1.0,
            range: 1.0,
            coil_type: 0,
            loc: [0.0; 12],
        },
        neuro_meg::fif::FifWriteChannel {
            name: "STI014".into(),
            kind: 3,
            unit: 112,
            unit_mul: 0,
            cal: 1.0,
            range: 1.0,
            coil_type: 0,
            loc: [0.0; 12],
        },
    ];
    let data: Vec<Vec<f64>> = (0..channels.len())
        .map(|c| {
            (0..200)
                .map(|s| (s as f64) * 0.01 * (c as f64 + 1.0))
                .collect()
        })
        .collect();
    neuro_meg::fif::write_fif(std::path::Path::new(&out), 250.0, &channels, &data)
        .expect("write_fif failed");
}
