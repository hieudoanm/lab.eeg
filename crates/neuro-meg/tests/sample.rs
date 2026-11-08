use neuro_meg::{
    inspect_recording, pipeline, read_window,
    sample::{generate_sample_recording, SampleConfig},
};

#[test]
fn generated_sample_is_readable_and_pipelineable() {
    let dir = std::env::temp_dir().join(format!(
        "neuro-sample-{}",
        format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )
    ));
    let config = SampleConfig {
        magnetometers: 8,
        gradiometers: 16,
        auxiliary: 9,
        sampling_frequency_hz: 100.0,
        duration_seconds: 10.0,
        stimulus_interval_s: 1.5,
        seed: 7,
    };
    generate_sample_recording(&dir, &config).unwrap();

    let metadata = inspect_recording(&dir).unwrap();
    assert_eq!(metadata.channel_count, 33);
    assert!(metadata.events.len() >= 6);

    let names: Vec<String> = metadata.channels.iter().map(|c| c.name.clone()).collect();
    let window = read_window(&dir, 0.0, 5.0, &names[..4], 1000, None, None, false).unwrap();
    assert!(!window.data.is_empty());

    let result = pipeline::run_pipeline(
        &serde_json::json!({"nodes": [
            {"type": "filter", "params": {"l_freq_hz": 1.0, "h_freq_hz": 40.0}},
            {"type": "epoch", "params": {"tmin_s": -0.1, "tmax_s": 0.3}},
            {"type": "evoked"},
        ]}),
        &window,
        &metadata,
        &names[..4],
        None,
        None,
    )
    .unwrap();
    assert!(result.evoked.is_some());
    assert!(result.evoked.unwrap().n_epochs >= 1);

    std::fs::remove_dir_all(&dir).ok();
}
