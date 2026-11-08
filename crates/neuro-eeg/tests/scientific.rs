use neuro_eeg::{
    analyze_recording, channel_statistics, describe_npy, read_npy_matrix_f64, synthetic_recording,
};
use std::process::Command;

#[test]
fn statistics_match_population_convention() {
    let matrix = vec![vec![1.0, 2.0, 3.0], vec![10.0, f64::NAN, 20.0]];
    let names = vec!["a".to_string(), "b".to_string()];
    let stats = channel_statistics(&matrix, &names, None).unwrap();
    assert_eq!(stats[0].mean, 2.0);
    assert!((stats[0].standard_deviation - (2.0_f64 / 3.0).sqrt()).abs() < 1e-12);
    assert_eq!(stats[1].valid_sample_count, 2);
}

#[test]
fn synthetic_recording_is_deterministic() {
    assert_eq!(synthetic_recording(7, 2, 8), synthetic_recording(7, 2, 8));
}

#[test]
fn npy_manifest_and_analysis_work_for_float32() {
    let dir = std::env::temp_dir().join(format!("neuro-eeg-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let npy = dir.join("recording.npy");
    let manifest = dir.join("recording.json");
    let script = r#"
import json
import numpy as np
np.save('recording.npy', np.array([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]], dtype=np.float32))
with open('recording.json', 'w') as f:
    json.dump({'sampling_frequency_hz': 100.0, 'channel_names': ['F3', 'Cz'], 'channel_units': ['uV', 'uV']}, f)
"#;
    let status = Command::new("python3")
        .arg("-c")
        .arg(script)
        .current_dir(&dir)
        .status()
        .unwrap();
    assert!(status.success());

    let metadata = describe_npy(&npy, 8, 1000, 100.0).unwrap();
    assert_eq!(
        metadata.channel_names,
        vec!["F3".to_string(), "Cz".to_string()]
    );
    assert_eq!(
        metadata.channel_units,
        vec!["uV".to_string(), "uV".to_string()]
    );
    assert_eq!(metadata.duration_seconds, 0.03);

    let matrix = read_npy_matrix_f64(&npy).unwrap();
    assert_eq!(matrix[0], vec![1.0, 2.0, 3.0]);

    let result = analyze_recording(&npy, "dataset-1", 100.0).unwrap();
    assert_eq!(result.channels[0].channel_name, "F3");
    assert_eq!(result.channels[0].unit.as_deref(), Some("uV"));

    std::fs::remove_dir_all(&dir).ok();
    let _ = manifest;
}

#[test]
fn synthetic_recording_roundtrips_through_analysis() {
    let dir = std::env::temp_dir().join(format!(
        "neuro-eeg-synth-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let result = neuro_eeg::write_synthetic_recording(&dir, "synth", 4, 500, 250.0, 3).unwrap();
    let path = std::path::PathBuf::from(result["path"].as_str().unwrap());
    let metadata = neuro_eeg::describe_npy(&path, 512, 100_000, 250.0).unwrap();
    assert_eq!(metadata.channel_names.len(), 4);
    let analysis = neuro_eeg::analyze_recording(&path, "synth", 250.0).unwrap();
    assert_eq!(analysis.channels.len(), 4);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn signal_window_enforces_bounds_and_units() {
    let dir = std::env::temp_dir().join(format!(
        "neuro-eeg-win-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let result = neuro_eeg::write_synthetic_recording(&dir, "w", 2, 1000, 100.0, 5).unwrap();
    let path = std::path::PathBuf::from(result["path"].as_str().unwrap());
    let names = vec!["A".to_string(), "B".to_string()];
    let units = vec!["uV".to_string(), "uV".to_string()];

    let window = neuro_eeg::extract_signal_window(
        &path,
        0.0,
        5.0,
        &[0, 1],
        &names,
        Some(&units),
        100.0,
        2000,
        300.0,
    )
    .unwrap();
    assert_eq!(window.channels.len(), 2);
    assert_eq!(window.channels[0].unit.as_deref(), Some("uV"));
    assert_eq!(window.time_seconds.len(), window.channels[0].samples.len());

    assert!(neuro_eeg::extract_signal_window(
        &path,
        9.0,
        20.0,
        &[0],
        &names,
        None,
        100.0,
        2000,
        300.0,
    )
    .is_err());
    assert!(neuro_eeg::extract_signal_window(
        &path,
        0.0,
        500.0,
        &[0],
        &names,
        None,
        100.0,
        2000,
        300.0,
    )
    .is_err());

    std::fs::remove_dir_all(&dir).ok();
}
