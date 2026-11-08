use neuro_meg::{band_power_summary, MegWindow};
use std::collections::BTreeMap;

#[test]
fn band_power_is_deterministic() {
    let window = MegWindow {
        times_s: vec![0.0, 0.1, 0.2],
        data: vec![vec![1.0, -1.0, 2.0]],
        effective_sampling_frequency_hz: 10.0,
    };
    let mut bands = BTreeMap::new();
    bands.insert("all".to_string(), (0.0, 100.0));
    let result = band_power_summary(&window, &bands);
    assert!(result["all"] > 0.0);
}

#[test]
fn bandpass_and_evoked_are_deterministic() {
    use neuro_meg::pipeline::{bandpass, epoch_window, evoked_average};
    use neuro_meg::MegEvent;

    let sfreq = 100.0;
    let n = 2000;
    let times: Vec<f64> = (0..n).map(|i| i as f64 / sfreq).collect();
    let data: Vec<Vec<f64>> = (0..2)
        .map(|ch| {
            (0..n)
                .map(|i| {
                    let t = i as f64 / sfreq;
                    (2.0 * std::f64::consts::PI * 10.0 * t).sin() * (1.0 + ch as f64)
                        + if (500..700).contains(&i) { 5.0 } else { 0.0 }
                })
                .collect()
        })
        .collect();
    let window = MegWindow {
        times_s: times,
        data,
        effective_sampling_frequency_hz: sfreq,
    };

    let filtered = bandpass(&window, 1.0, 40.0).unwrap();
    assert_eq!(filtered.data.len(), 2);
    assert!(filtered.data[0].iter().all(|v| v.is_finite()));

    let events = vec![
        MegEvent {
            sample: 600,
            time_s: 6.0,
            code: Some("stim".into()),
            value: Some(1),
            description: None,
        },
        MegEvent {
            sample: 1000,
            time_s: 10.0,
            code: Some("stim".into()),
            value: Some(1),
            description: None,
        },
    ];
    let (epochs, skipped) = epoch_window(&filtered, &events, sfreq, -0.1, 0.2).unwrap();
    assert_eq!(epochs.len(), 2);
    assert_eq!(skipped, 0);
    let evoked = evoked_average(&epochs, &["a".to_string(), "b".to_string()]).unwrap();
    assert_eq!(evoked.n_epochs, 2);
    assert_eq!(evoked.data.len(), 2);
}

#[test]
fn epoch_window_rejects_empty_events() {
    use neuro_meg::pipeline::epoch_window;
    let window = MegWindow {
        times_s: vec![0.0, 0.01, 0.02],
        data: vec![vec![1.0, 2.0, 3.0]],
        effective_sampling_frequency_hz: 100.0,
    };
    let err = epoch_window(&window, &[], 100.0, -0.1, 0.1).unwrap_err();
    assert!(err.to_string().contains("no events"));
}

#[test]
fn epoch_node_filters_events_by_code() {
    use neuro_meg::pipeline::run_pipeline;
    use neuro_meg::{MegEvent, MegMetadata};

    let sfreq = 100.0;
    let n = 2000usize;
    let times: Vec<f64> = (0..n).map(|i| i as f64 / sfreq).collect();
    let window = MegWindow {
        times_s: times,
        data: vec![vec![0.0; n]],
        effective_sampling_frequency_hz: sfreq,
    };
    let metadata = MegMetadata {
        format: "test".into(),
        name: "test".into(),
        acquisition_system: "test".into(),
        sampling_frequency_hz: sfreq,
        channel_count: 1,
        duration_seconds: n as f64 / sfreq,
        coordinate_frame: "head".into(),
        channels: vec![],
        events: vec![
            MegEvent {
                sample: 500,
                time_s: 5.0,
                code: Some("Response".into()),
                value: None,
                description: None,
            },
            MegEvent {
                sample: 1000,
                time_s: 10.0,
                code: Some("Response".into()),
                value: None,
                description: None,
            },
            MegEvent {
                sample: 1500,
                time_s: 15.0,
                code: Some("Fixation".into()),
                value: None,
                description: None,
            },
        ],
        bad_channel_names: vec![],
        sensor_type: String::new(),
        time_varying: false,
        device_serial: None,
        manufacturer: None,
        digitization_points: 0,
        recordings: vec![],
        recording_channels: Default::default(),
        compensation: vec![],
        surface_points: vec![],
    };
    let names = vec!["a".to_string()];
    // Unfiltered: all three events epoch.
    let all = run_pipeline(
        &serde_json::json!({"nodes": [{"type": "epoch", "params": {"tmin_s": -0.1, "tmax_s": 0.1}}]}),
        &window,
        &metadata,
        &names,
        None,
        None,
    )
    .unwrap();
    assert_eq!(all.epoch_summary.as_ref().unwrap().n_epochs, 3);
    // Substring filter on code: only Response events.
    let filtered = run_pipeline(
        &serde_json::json!({"nodes": [{"type": "epoch", "params": {"tmin_s": -0.1, "tmax_s": 0.1, "event": "response"}}]}),
        &window,
        &metadata,
        &names,
        None,
        None,
    )
    .unwrap();
    let summary = filtered.epoch_summary.as_ref().unwrap();
    assert_eq!(summary.n_epochs, 2);
    assert_eq!(summary.event_codes, vec!["Response".to_string()]);
    // Case-insensitive substring also matches partial codes.
    let partial = run_pipeline(
        &serde_json::json!({"nodes": [{"type": "epoch", "params": {"tmin_s": -0.1, "tmax_s": 0.1, "event": "espo"}}]}),
        &window,
        &metadata,
        &names,
        None,
        None,
    )
    .unwrap();
    assert_eq!(partial.epoch_summary.as_ref().unwrap().n_epochs, 2);
}

#[test]
fn synthetic_gradiometer_node_uses_channel_positions() {
    use neuro_meg::pipeline::run_pipeline;
    use neuro_meg::{MegChannel, MegMetadata};

    let channel = |name: &str, kind: &str, position: Option<Vec<f64>>| MegChannel {
        name: name.to_string(),
        kind: kind.to_string(),
        unit: "tesla".into(),
        unit_mul: 1.0,
        group_name: None,
        status: "good".into(),
        coil_type: None,
        position_mm: position,
        orientation: None,
    };
    let metadata = MegMetadata {
        format: "test".into(),
        name: "test".into(),
        acquisition_system: "test".into(),
        sampling_frequency_hz: 100.0,
        channel_count: 3,
        duration_seconds: 0.03,
        coordinate_frame: "head".into(),
        channels: vec![
            channel("MEG1", "meg", Some(vec![0.0, 0.0, 0.0])),
            channel("MEG2", "meg", Some(vec![10.0, 0.0, 0.0])),
            channel("EEG1", "eeg", None),
        ],
        events: vec![],
        bad_channel_names: vec![],
        sensor_type: String::new(),
        time_varying: false,
        device_serial: None,
        manufacturer: None,
        digitization_points: 0,
        recordings: vec![],
        recording_channels: Default::default(),
        compensation: vec![],
        surface_points: vec![],
    };
    let window = MegWindow {
        times_s: vec![0.0, 0.01, 0.02],
        data: vec![
            vec![1.0, 2.0, 3.0],
            vec![3.0, 5.0, 7.0],
            vec![10.0, 10.0, 10.0],
        ],
        effective_sampling_frequency_hz: 100.0,
    };
    let names = vec!["MEG1".to_string(), "MEG2".to_string(), "EEG1".to_string()];
    let result = run_pipeline(
        &serde_json::json!({"nodes": [{"type": "synthetic_gradiometer", "params": {"min_distance_mm": 2.0, "max_distance_mm": 30.0}}]}),
        &window,
        &metadata,
        &names,
        None,
        None,
    )
    .unwrap();
    let out = result.window.unwrap();
    // MEG1/MEG2 are 10 mm apart -> (x1 - x2) / 0.01 m.
    assert_eq!(out.data[0], vec![-200.0, -300.0, -400.0]);
    assert_eq!(out.data[1], vec![200.0, 300.0, 400.0]);
    // EEG channel has no position -> unchanged.
    assert_eq!(out.data[2], vec![10.0, 10.0, 10.0]);
    assert_eq!(result.steps, vec!["synthetic_gradiometer[2]"]);
}

#[test]
fn ica_node_separates_and_reconstructs_meg_channels() {
    use neuro_meg::pipeline::run_pipeline;
    use neuro_meg::{MegChannel, MegMetadata};

    let channel = |name: &str, kind: &str| MegChannel {
        name: name.to_string(),
        kind: kind.to_string(),
        unit: "tesla".into(),
        unit_mul: 1.0,
        group_name: None,
        status: "good".into(),
        coil_type: None,
        position_mm: None,
        orientation: None,
    };
    let n = 400usize;
    let times: Vec<f64> = (0..n).map(|i| i as f64 / 100.0).collect();
    // Two MEG channels = mixtures of two deterministic sources; EEG row
    // must pass through ICA untouched.
    let s1: Vec<f64> = (0..n).map(|i| (i as f64 * 0.05).sin()).collect();
    let s2: Vec<f64> = (0..n).map(|i| (i as f64 * 0.11).cos() * 0.5).collect();
    let meg1: Vec<f64> = s1.iter().zip(&s2).map(|(a, b)| a + 2.0 * b).collect();
    let meg2: Vec<f64> = s1.iter().zip(&s2).map(|(a, b)| 0.5 * a - b).collect();
    let eeg = vec![7.0f64; n];
    let window = MegWindow {
        times_s: times,
        data: vec![meg1.clone(), meg2.clone(), eeg],
        effective_sampling_frequency_hz: 100.0,
    };
    let metadata = MegMetadata {
        format: "test".into(),
        name: "test".into(),
        acquisition_system: "test".into(),
        sampling_frequency_hz: 100.0,
        channel_count: 3,
        duration_seconds: 4.0,
        coordinate_frame: "head".into(),
        channels: vec![
            channel("MEG1", "meg"),
            channel("MEG2", "meg"),
            channel("EEG1", "eeg"),
        ],
        events: vec![],
        bad_channel_names: vec![],
        sensor_type: String::new(),
        time_varying: false,
        device_serial: None,
        manufacturer: None,
        digitization_points: 0,
        recordings: vec![],
        recording_channels: Default::default(),
        compensation: vec![],
        surface_points: vec![],
    };
    let names = vec!["MEG1".to_string(), "MEG2".to_string(), "EEG1".to_string()];
    // ICA then remove ALL components -> MEG rows reconstruct to ~zero,
    // EEG row untouched.
    let result = run_pipeline(
        &serde_json::json!({"nodes": [
            {"type": "ica"},
            {"type": "ica_remove", "params": {"components": [0, 1]}}
        ]}),
        &window,
        &metadata,
        &names,
        None,
        None,
    )
    .unwrap();
    assert!(result.steps.iter().any(|s| s == "ica[2]"));
    assert!(result.steps.iter().any(|s| s == "ica_remove[2]"));
    let out = result.window.unwrap();
    // Excluding every component reconstructs each MEG channel to its
    // pre-ICA mean (the DC offset ICA preserves).
    for (row, original) in out.data[..2].iter().zip([&meg1, &meg2]) {
        let mean = original.iter().sum::<f64>() / original.len() as f64;
        assert!(row.iter().all(|v| (*v - mean).abs() < 1e-9), "mean {mean}");
    }
    assert!(out.data[2].iter().all(|v| (*v - 7.0).abs() < 1e-12));
    // ica_remove without a preceding ica node errors.
    assert!(run_pipeline(
        &serde_json::json!({"nodes": [{"type": "ica_remove", "params": {"components": [0]}}]}),
        &window,
        &metadata,
        &names,
        None,
        None,
    )
    .is_err());
}

#[test]
fn bad_channel_detection_flags_flat_and_noisy_channels() {
    use neuro_meg::pipeline::detect_bad_channels;
    use neuro_meg::MegWindow;

    let window = MegWindow {
        times_s: vec![0.0, 0.01, 0.02, 0.03],
        data: vec![
            vec![1.0, 2.0, 3.0, 4.0],
            vec![5.0, 5.0, 5.0, 5.0],
            vec![0.0, 100.0, -50.0, 200.0],
        ],
        effective_sampling_frequency_hz: 100.0,
    };
    let names = vec!["good".to_string(), "flat".to_string(), "noisy".to_string()];
    let bad = detect_bad_channels(&window, &names);
    assert!(bad.contains(&"flat".to_string()));
    assert!(bad.contains(&"noisy".to_string()));
    assert!(!bad.contains(&"good".to_string()));
}

#[test]
fn ssp_node_projects_out_noise_subspace() {
    use neuro_meg::pipeline::run_pipeline;
    use neuro_meg::{MegChannel, MegMetadata, MegWindow};

    let channel = |name: &str, kind: &str| MegChannel {
        name: name.to_string(),
        kind: kind.to_string(),
        unit: "tesla".into(),
        unit_mul: 1.0,
        group_name: None,
        status: "good".into(),
        coil_type: None,
        position_mm: None,
        orientation: None,
    };
    let n = 400usize;
    let times: Vec<f64> = (0..n).map(|i| i as f64 / 100.0).collect();
    // Artifact direction u = (3,4)/5 present only in samples 100..200.
    let mut meg1 = vec![0.0f64; n];
    let mut meg2 = vec![0.0f64; n];
    for i in 0..n {
        meg1[i] = (i as f64 * 0.05).sin();
        meg2[i] = (i as f64 * 0.11).cos() * 0.5;
        if (100..200).contains(&i) {
            let noise = ((i * 37 % 101) as f64 - 50.0) * 10.0;
            meg1[i] += 0.6 * noise;
            meg2[i] += 0.8 * noise;
        }
    }
    let eeg = vec![3.0f64; n];
    let window = MegWindow {
        times_s: times,
        data: vec![meg1, meg2, eeg],
        effective_sampling_frequency_hz: 100.0,
    };
    let metadata = MegMetadata {
        format: "test".into(),
        name: "test".into(),
        acquisition_system: "test".into(),
        sampling_frequency_hz: 100.0,
        channel_count: 3,
        duration_seconds: 4.0,
        coordinate_frame: "head".into(),
        channels: vec![
            channel("MEG1", "meg"),
            channel("MEG2", "meg"),
            channel("EEG1", "eeg"),
        ],
        events: vec![],
        bad_channel_names: vec![],
        sensor_type: String::new(),
        time_varying: false,
        device_serial: None,
        manufacturer: None,
        digitization_points: 0,
        recordings: vec![],
        recording_channels: Default::default(),
        compensation: vec![],
        surface_points: vec![],
    };
    let names = vec!["MEG1".to_string(), "MEG2".to_string(), "EEG1".to_string()];
    let result = run_pipeline(
        &serde_json::json!({"nodes": [{"type": "ssp", "params": {"start_sample": 100, "end_sample": 200, "n_components": 1}}]}),
        &window,
        &metadata,
        &names,
        None,
        None,
    )
    .unwrap();
    assert!(result.steps[0].starts_with("ssp["));
    let out = result.window.unwrap();
    // The u-component must be (near-)zero on every sample of both MEG rows.
    for t in 0..n {
        let proj = 0.6 * out.data[0][t] + 0.8 * out.data[1][t];
        assert!(
            proj.abs() < 1e-3,
            "sample {t} retains noise component {proj}"
        );
    }
    // EEG passthrough.
    assert!(out.data[2].iter().all(|v| *v == 3.0));
    // SSP without a noise window errors.
    assert!(run_pipeline(
        &serde_json::json!({"nodes": [{"type": "ssp", "params": {}}]}),
        &window,
        &metadata,
        &names,
        None,
        None,
    )
    .is_err());
}
