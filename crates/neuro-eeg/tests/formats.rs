use neuro_eeg::edf;
use neuro_eeg::pipeline;

fn edf_field(text: &str, width: usize) -> String {
    format!("{:<width$}", text, width = width)
}

fn write_edf(path: &std::path::Path, n_channels: usize, records: usize, samples_per_record: usize) {
    let mut header = String::new();
    header.push_str(&edf_field("0", 8)); // version
    header.push_str(&edf_field("patient", 80));
    header.push_str(&edf_field("recording", 80));
    header.push_str(&edf_field("01.01.24", 8));
    header.push_str(&edf_field("00.00.00", 8));
    // header bytes filled later
    header.push_str(&edf_field("", 44)); // reserved... actually 44 reserved + fields below
                                         // rebuild properly: reserved is 44 bytes at offset 192
    let mut h = String::new();
    h.push_str(&edf_field("0", 8));
    h.push_str(&edf_field("patient", 80));
    h.push_str(&edf_field("recording", 80));
    h.push_str(&edf_field("01.01.24", 8));
    h.push_str(&edf_field("00.00.00", 8));
    let signal_headers_len = n_channels * 256;
    let header_bytes = 256 + signal_headers_len;
    h.push_str(&edf_field(&header_bytes.to_string(), 8));
    h.push_str(&edf_field("", 44));
    h.push_str(&edf_field(&records.to_string(), 8));
    h.push_str(&edf_field("1", 8)); // 1 s records
    h.push_str(&edf_field(&n_channels.to_string(), 4));
    assert_eq!(h.len(), 256);
    let _ = header;
    for ch in 0..n_channels {
        let mut sh = String::new();
        sh.push_str(&edf_field(&format!("EEG {ch:03}"), 16)); // label
        sh.push_str(&edf_field("", 80)); // transducer
        sh.push_str(&edf_field("uV", 8)); // phys dim
        sh.push_str(&edf_field("-100", 8));
        sh.push_str(&edf_field("100", 8));
        sh.push_str(&edf_field("-32768", 8));
        sh.push_str(&edf_field("32767", 8));
        sh.push_str(&edf_field("", 80)); // prefiltering
        sh.push_str(&edf_field(&samples_per_record.to_string(), 8));
        sh.push_str(&edf_field("", 32));
        assert_eq!(sh.len(), 256);
        h.push_str(&sh);
    }
    let mut bytes = h.into_bytes();
    // data: record-major, signal-major within record; value = sample index
    for record in 0..records {
        for ch in 0..n_channels {
            for sample in 0..samples_per_record {
                let value = (record * samples_per_record + sample) as i16;
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            let _ = ch;
        }
    }
    std::fs::write(path, bytes).unwrap();
}

#[test]
fn edf_inspect_and_read_roundtrip() {
    let dir = std::env::temp_dir().join(format!("neuro-edf-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test.edf");
    write_edf(&path, 2, 10, 100); // 2 ch, 10 s @ 100 Hz

    let info = edf::inspect_edf(&path).unwrap();
    assert_eq!(info.channel_names.len(), 2);
    assert_eq!(info.channel_names[0], "EEG 000");
    assert!((info.sampling_frequency_hz - 100.0).abs() < 1e-9);
    assert_eq!(info.n_samples_per_channel, 1000);
    assert_eq!(info.units[0], "uV");

    let (matrix, info2) = edf::read_edf(&path).unwrap();
    assert_eq!(matrix.len(), 2);
    assert_eq!(matrix[0].len(), 1000);
    // dig value v maps to phys: -100 + (v + 32768) * (200 / 65535)
    let expected = -100.0 + (0.0 + 32768.0) * (200.0 / 65535.0);
    assert!((matrix[0][0] - expected).abs() < 1e-6);
    assert_eq!(info2.n_records, 10);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn eeg_validate_recording_reports_format_and_findings() {
    let dir = std::env::temp_dir().join(format!("neuro-eeg-validate-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    write_edf(&dir.join("rec.edf"), 2, 5, 100);
    // File path and directory path both resolve.
    for target in [dir.join("rec.edf"), dir.clone()] {
        let (format, findings) = neuro_eeg::validate_recording(&target);
        assert_eq!(format, "edf");
        assert!(findings.iter().any(|f| f.contains("no EDF+ annotation")));
        assert!(findings.iter().any(|f| f.contains("2 channels")));
    }
    // Unsupported extension is an explicit finding, not a panic.
    let bad = dir.join("notes.txt");
    std::fs::write(&bad, "x").unwrap();
    let (format, findings) = neuro_eeg::validate_recording(&bad);
    assert_eq!(format, "unknown");
    assert!(findings.iter().any(|f| f.contains("unsupported")));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn edf_annotations_become_events() {
    let dir = std::env::temp_dir().join(format!("neuro-edf-ann-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("ann.edf");
    // 2 channels x 2 records x 100 samples @100Hz; channel 1 is EDF Annotations.
    let n_channels = 2usize;
    let records = 2usize;
    let spr = 100usize;
    let mut h = String::new();
    h.push_str(&edf_field("0", 8));
    h.push_str(&edf_field("patient", 80));
    h.push_str(&edf_field("recording", 80));
    h.push_str(&edf_field("01.01.24", 8));
    h.push_str(&edf_field("00.00.00", 8));
    let header_bytes = 256 + n_channels * 256;
    h.push_str(&edf_field(&header_bytes.to_string(), 8));
    h.push_str(&edf_field("", 44));
    h.push_str(&edf_field(&records.to_string(), 8));
    h.push_str(&edf_field("1", 8));
    h.push_str(&edf_field(&n_channels.to_string(), 4));
    for (ch, label) in ["EEG 000", "EDF Annotations"].iter().enumerate() {
        let mut sh = String::new();
        sh.push_str(&edf_field(label, 16));
        sh.push_str(&edf_field("", 80));
        sh.push_str(&edf_field("uV", 8));
        sh.push_str(&edf_field("-100", 8));
        sh.push_str(&edf_field("100", 8));
        sh.push_str(&edf_field("-32768", 8));
        sh.push_str(&edf_field("32767", 8));
        sh.push_str(&edf_field("", 80));
        sh.push_str(&edf_field(&spr.to_string(), 8));
        sh.push_str(&edf_field("", 32));
        let _ = ch;
        h.push_str(&sh);
    }
    let mut bytes = h.into_bytes();
    let tal: Vec<u8> = b"+1.000\x15.000\x14Test annot\x14".to_vec();
    for record in 0..records {
        for _ in 0..spr {
            bytes.extend_from_slice(&0i16.to_le_bytes());
        }
        for s in 0..spr {
            let v = tal.get(s).copied().unwrap_or(0) as i16;
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        let _ = record;
    }
    std::fs::write(&path, bytes).unwrap();

    let info = edf::inspect_edf(&path).unwrap();
    assert_eq!(info.events.len(), 2);
    // onset 1.0 s relative to each 1 s record -> samples 100 and 200
    assert_eq!(info.events[0].sample, 100);
    assert_eq!(info.events[0].label, "Test annot");
    assert_eq!(info.events[1].sample, 200);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn brainvision_vhdr_eeg_vmrk_roundtrip() {
    let dir = std::env::temp_dir().join(format!("neuro-bv-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let hdr = dir.join("test.vhdr");
    std::fs::write(
        &hdr,
        "[Common Infos]\n\
         DataFile=test.eeg\n\
         MarkerFile=test.vmrk\n\
         DataFormat=BINARY\n\
         DataOrientation=MULTIPLEXED\n\
         NumberOfChannels=2\n\
         SamplingRate=100\n\
         DataPoints=50\n\
         [Binary Infos]\n\
         BinaryFormat=IEEE_FLOAT_32\n\
         [Channel Infos]\n\
         ch1=Fp1,,0.5,uV\n\
         ch2=Fp2,,1.0,uV\n",
    )
    .unwrap();
    // MULTIPLEXED float32: per sample, ch0 then ch1.
    let mut eeg = Vec::new();
    for s in 0..50usize {
        eeg.extend_from_slice(&(s as f32 * 2.0).to_le_bytes()); // ch0 raw
        eeg.extend_from_slice(&(s as f32).to_le_bytes()); // ch1 raw
    }
    std::fs::write(dir.join("test.eeg"), eeg).unwrap();
    std::fs::write(
        dir.join("test.vmrk"),
        "[Marker Infos]\nMk1=Response,Button 1,10,0\n",
    )
    .unwrap();

    let info = neuro_eeg::brainvision::inspect_brainvision(&hdr).unwrap();
    assert_eq!(info.channel_names, vec!["Fp1", "Fp2"]);
    assert_eq!(info.sampling_frequency_hz, 100.0);
    assert_eq!(info.n_samples, 50);
    assert_eq!(info.events.len(), 1);
    assert_eq!(info.events[0].sample, 9); // 1-indexed -> 0-indexed
    assert_eq!(info.events[0].label, "Button 1");

    let (matrix, _) = neuro_eeg::brainvision::read_brainvision(&hdr).unwrap();
    // ch0 raw=2*s * resolution 0.5 = s ; ch1 raw=s * 1.0 = s
    assert_eq!(matrix[0][3], 3.0);
    assert_eq!(matrix[1][3], 3.0);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn eeg_reref_average_and_mastoid() {
    // 3 channels x 4 samples; Fp1 varies, M1/M2 are the references.
    let matrix = vec![
        vec![1.0, 2.0, 3.0, 4.0],
        vec![10.0, 10.0, 10.0, 10.0],
        vec![20.0, 20.0, 20.0, 20.0],
    ];
    let names = vec!["Fp1".to_string(), "M1".to_string(), "M2".to_string()];

    // mastoid: subtract mean(M1, M2) = 15 from every channel/sample.
    let mut mastoid = matrix.clone();
    pipeline::rereference(&mut mastoid, &names, "mastoid").unwrap();
    assert_eq!(mastoid[0], vec![-14.0, -13.0, -12.0, -11.0]);
    assert_eq!(mastoid[1], vec![-5.0; 4]);
    assert_eq!(mastoid[2], vec![5.0; 4]);

    // average: subtract per-sample mean across all three channels.
    let mut avg = matrix.clone();
    pipeline::rereference(&mut avg, &names, "average").unwrap();
    // sample 0 mean = 31/3 -> Fp1 = 1 - 31/3 = -28/3
    assert!((avg[0][0] - (1.0 - 31.0 / 3.0)).abs() < 1e-12);
    // M1 sample 0 = 10 - 31/3 = -1/3 ; M2 = 20 - 31/3 = 29/3
    assert!((avg[1][0] - (10.0 - 31.0 / 3.0)).abs() < 1e-12);
    assert!((avg[2][0] - (20.0 - 31.0 / 3.0)).abs() < 1e-12);

    // pipeline node records the step.
    let result = pipeline::run_pipeline(
        &serde_json::json!({"nodes": [{"type": "reref", "params": {"method": "average"}}]}),
        &matrix,
        100.0,
        &names,
        &[],
        None,
        None,
    )
    .unwrap();
    assert_eq!(result.steps, vec!["reref[average]"]);

    // car is a common-average alias; unknown methods and missing
    // references are errors.
    let mut car = matrix.clone();
    pipeline::rereference(&mut car, &names, "car").unwrap();
    assert_eq!(car, avg);
    assert!(pipeline::rereference(&mut matrix.clone(), &names, "laplacian").is_err());
    let no_ref = vec!["Fp1".to_string(), "Fp2".to_string()];
    let mut two = matrix[..2].to_vec();
    assert!(pipeline::rereference(&mut two, &no_ref, "mastoid").is_err());
}

#[test]
fn eeg_reref_robust_median_and_laplacian() {
    // 4 channels x 3 samples. ch3 is an outlier; median reference ignores it.
    let matrix = vec![
        vec![1.0, 2.0, 3.0],
        vec![2.0, 4.0, 6.0],
        vec![3.0, 6.0, 9.0],
        vec![100.0, 100.0, 100.0],
    ];
    let names: Vec<String> = (0..4).map(|i| format!("ch{i}")).collect();

    // plain median reference per sample: sample0 sorted [1,2,3,100]
    // -> (2+3)/2 = 2.5 (the outlier still participates).
    let mut median = matrix.clone();
    pipeline::rereference(&mut median, &names, "median").unwrap();
    assert!((median[0][0] - (1.0 - 2.5)).abs() < 1e-12);
    assert!((median[3][0] - (100.0 - 2.5)).abs() < 1e-12);
    // sample1 sorted [2,4,6,100] -> 5.0
    assert!((median[0][1] - (2.0 - 5.0)).abs() < 1e-12);

    // iterated robust_average rejects ch3 (RMS ~97.5 vs ~1 for the rest),
    // so the reference is the median of ch0..ch2 only: sample0 -> 2.0.
    let mut robust = matrix.clone();
    pipeline::rereference(&mut robust, &names, "robust_average").unwrap();
    assert!((robust[0][0] - (1.0 - 2.0)).abs() < 1e-12);
    assert!((robust[0][1] - (2.0 - 4.0)).abs() < 1e-12);

    // common average would give a very different reference on sample 0
    // (mean = 26.5), showing the robust variant actually differs.
    let mut avg = matrix.clone();
    pipeline::rereference(&mut avg, &names, "average").unwrap();
    assert!((avg[0][0] - (1.0 - 26.5)).abs() < 1e-12);

    // Surface Laplacian: 4 channels on a line, k = min(6, 3) = 3 nearest.
    // Positions along x: 0, 1, 2, 10 -> ch3's neighbours are ch0..ch2.
    let positions = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [10.0, 0.0, 0.0],
    ];
    let mut lap = matrix.clone();
    pipeline::surface_laplacian(&mut lap, &positions).unwrap();
    // ch0 neighbours by distance: ch1(1), ch2(2), ch3(10) -> mean of [2,3,100]@s0
    let expected_ch0 = 1.0 - (2.0 + 3.0 + 100.0) / 3.0;
    assert!((lap[0][0] - expected_ch0).abs() < 1e-12);
    // mismatched position count is an error.
    assert!(pipeline::surface_laplacian(&mut matrix.clone(), &positions[..2]).is_err());
}

#[test]
fn eeg_baseline_node_corrects_continuous_signal() {
    // 2 channels x 400 samples @ 100 Hz. Baseline window [0, 1s) = first
    // 100 samples: ch0 mean 2.0, ch1 mean 7.0 -> whole row shifted.
    let n = 400usize;
    let ch0: Vec<f64> = (0..n).map(|i| 2.0 + (i as f64 * 0.001)).collect();
    let ch1: Vec<f64> = (0..n).map(|i| 7.0 - (i as f64 * 0.001)).collect();
    let matrix = vec![ch0, ch1];
    let names = vec!["Fp1".to_string(), "Pz".to_string()];

    // Direct helper: baseline over [0,1) -> ch0 first-100 mean, ch1 same.
    let mut corrected = matrix.clone();
    pipeline::baseline_correct(&mut corrected, 100.0, 0.0, 1.0).unwrap();
    let mean0: f64 = (0..100).map(|i| 2.0 + i as f64 * 0.001).sum::<f64>() / 100.0;
    let mean1: f64 = (0..100).map(|i| 7.0 - i as f64 * 0.001).sum::<f64>() / 100.0;
    assert!((corrected[0][0] - (2.0 - mean0)).abs() < 1e-12);
    assert!((corrected[0][399] - (2.399 - mean0)).abs() < 1e-12);
    assert!((corrected[1][0] - (7.0 - mean1)).abs() < 1e-12);

    // Pipeline node records the step and accepts baseline_s pair form.
    let result = pipeline::run_pipeline(
        &serde_json::json!({"nodes": [{"type": "baseline", "params": {"baseline_s": [0.0, 1.0]}}]}),
        &matrix,
        100.0,
        &names,
        &[],
        None,
        None,
    )
    .unwrap();
    assert_eq!(result.steps, vec!["baseline[0-1s]"]);

    // Invalid windows are explicit errors.
    assert!(pipeline::baseline_correct(&mut matrix.clone(), 100.0, 1.0, 0.0).is_err());
    assert!(pipeline::baseline_correct(&mut matrix.clone(), 100.0, 5.0, 6.0).is_err());
}

#[test]
fn eeg_ica_separates_sources_and_reconstructs() {
    // 3 "sources": distinct sinusoids; mixed into 4 channels via a fixed
    // mixing matrix. ICA should recover uncorrelated components whose
    // span matches the sources, and reconstruction should be near-exact.
    let n = 2000usize;
    let sfreq = 200.0;
    let s1: Vec<f64> = (0..n)
        .map(|i| (2.0 * std::f64::consts::PI * 5.0 * i as f64 / sfreq).sin())
        .collect();
    let s2: Vec<f64> = (0..n)
        .map(|i| (2.0 * std::f64::consts::PI * 13.0 * i as f64 / sfreq).sin())
        .collect();
    let s3: Vec<f64> = (0..n)
        .map(|i| (2.0 * std::f64::consts::PI * 31.0 * i as f64 / sfreq).sin() * 0.5)
        .collect();
    // 4 channels = mixing (4x3) @ sources + small DC offsets.
    let mixing = [
        [1.0, 0.5, -0.3],
        [0.2, 1.0, 0.7],
        [-0.4, 0.3, 1.0],
        [0.6, -0.2, 0.5],
    ];
    let mut matrix = vec![vec![0.0f64; n]; 4];
    for ch in 0..4 {
        for t in 0..n {
            matrix[ch][t] = mixing[ch][0] * s1[t]
                + mixing[ch][1] * s2[t]
                + mixing[ch][2] * s3[t]
                + (ch as f64) * 3.0;
        }
    }
    let model = neuro_eeg::ica::fastica(&matrix, Some(3), 1500, 1e-9).unwrap();
    assert_eq!(model.sources.len(), 3);
    assert_eq!(model.sources[0].len(), n);
    // Sources should be ~uncorrelated: off-diagonal covariance small
    // relative to diagonal power.
    for i in 0..3 {
        for j in i + 1..3 {
            let dot: f64 = model.sources[i]
                .iter()
                .zip(model.sources[j].iter())
                .map(|(a, b)| a * b)
                .sum::<f64>()
                / n as f64;
            let pi: f64 = model.sources[i].iter().map(|v| v * v).sum::<f64>() / n as f64;
            let pj: f64 = model.sources[j].iter().map(|v| v * v).sum::<f64>() / n as f64;
            assert!(
                dot.abs() < 0.05 * (pi * pj).sqrt(),
                "components {i},{j} correlated: {dot}"
            );
        }
    }
    // Full reconstruction (no exclusions) should recover the input closely
    // because the data lies in a 3-D subspace.
    let recon = neuro_eeg::ica::reconstruct(&model, &[]);
    let mut max_err = 0.0f64;
    for ch in 0..4 {
        for t in (0..n).step_by(50) {
            max_err = max_err.max((recon[ch][t] - matrix[ch][t]).abs());
        }
    }
    assert!(max_err < 1e-6, "reconstruction error {max_err}");
    // Dropping one component loses that subspace dimension: error grows.
    let reduced = neuro_eeg::ica::reconstruct(&model, &[0]);
    let mut err_reduced = 0.0f64;
    for ch in 0..4 {
        for t in (0..n).step_by(50) {
            err_reduced = err_reduced.max((reduced[ch][t] - matrix[ch][t]).abs());
        }
    }
    assert!(
        err_reduced > 0.1,
        "expected reconstruction loss, got {err_reduced}"
    );

    // Pipeline nodes: ica then remove component 0.
    let names: Vec<String> = (0..4).map(|i| format!("ch{i}")).collect();
    let result = pipeline::run_pipeline(
        &serde_json::json!({"nodes": [
            {"type": "ica", "params": {"n_components": 3, "max_iter": 1500, "tol": 1e-9}},
            {"type": "ica_remove", "params": {"components": [0]}},
            {"type": "psd", "params": {"fmin_hz": 0.5, "fmax_hz": 80.0, "n_fft": 256}},
        ]}),
        &matrix,
        200.0,
        &names,
        &[],
        None,
        None,
    )
    .unwrap();
    assert!(result.steps.iter().any(|s| s.starts_with("ica[n=3]")));
    assert!(result.steps.iter().any(|s| s.starts_with("ica_remove[1]")));
    // ica_remove without a preceding ica node is an explicit error.
    assert!(pipeline::run_pipeline(
        &serde_json::json!({"nodes": [{"type": "ica_remove", "params": {"components": [0]}}]}),
        &matrix,
        200.0,
        &names,
        &[],
        None,
        None,
    )
    .is_err());
}

#[test]
fn eeg_epoch_node_produces_baselined_erp() {
    use neuro_eeg::EegEvent;
    // 2 channels x 600 samples @ 100 Hz; a 10 uV step locked to events at
    // samples 100 and 300, plus one artifact epoch with a huge spike.
    let n = 600usize;
    let mut ch0 = vec![0.0f64; n];
    let ch1 = vec![5.0f64; n];
    for s in 100..400 {
        ch0[s] += 10.0;
    }
    ch0[305] += 500.0; // artifact inside second epoch
    let matrix = vec![ch0, ch1];
    let names = vec!["Fp1".to_string(), "M1".to_string()];
    let events = vec![
        EegEvent {
            sample: 100,
            time_s: 1.0,
            label: "Response".into(),
        },
        EegEvent {
            sample: 300,
            time_s: 3.0,
            label: "Response".into(),
        },
        EegEvent {
            sample: 200,
            time_s: 2.0,
            label: "Fixation".into(),
        },
    ];
    let def = serde_json::json!({"nodes": [
        {"type": "epoch", "params": {"tmin_s": -0.1, "tmax_s": 0.3, "event": "response", "reject_ptp": 50.0}}
    ]});
    let result = pipeline::run_pipeline(&def, &matrix, 100.0, &names, &events, None, None).unwrap();
    // epoch window = 40 samples; event filter "response" matches 2 events,
    // the sample-300 epoch is rejected by the 50 uV ptp threshold.
    assert_eq!(result.n_epochs, 1);
    assert_eq!(result.n_epochs_rejected, 1);
    let erp = result.erp.as_ref().unwrap();
    assert_eq!(erp.len(), 2);
    assert_eq!(erp[0].len(), 40);
    // baseline = [-0.1, 0) = first 10 samples (all zeros for ch0, 5 for ch1)
    // post-onset ch0 mean should be ~10 (step locked at event).
    assert!((erp[0][15] - 10.0).abs() < 1e-9);
    assert!((erp[1][15] - 0.0).abs() < 1e-9);
    // Fixation event excluded by the label filter: n_epochs would be 3
    // without it; verify via unfiltered run.
    let all = pipeline::run_pipeline(
        &serde_json::json!({"nodes": [{"type": "epoch", "params": {"tmin_s": -0.1, "tmax_s": 0.3}}]}),
        &matrix,
        100.0,
        &names,
        &events,
        None,
        None,
    )
    .unwrap();
    assert_eq!(all.n_epochs, 3);
    assert_eq!(all.n_epochs_rejected, 0);
    // reject_abs rejects on absolute amplitude: the 500 uV spike epoch
    // fails a 20 uV ceiling even though its peak-to-peak is larger anyway.
    let abs_reject = pipeline::run_pipeline(
        &serde_json::json!({"nodes": [{"type": "epoch", "params": {"tmin_s": -0.1, "tmax_s": 0.3, "event": "response", "reject_abs": 20.0}}]}),
        &matrix,
        100.0,
        &names,
        &events,
        None,
        None,
    )
    .unwrap();
    assert_eq!(abs_reject.n_epochs, 1);
    assert_eq!(abs_reject.n_epochs_rejected, 1);
}

#[test]
fn eeg_pipeline_bandpass_psd_and_qc() {
    let n = 2048;
    let mut row = vec![0.0f64; n];
    for i in 0..n {
        row[i] = (2.0 * std::f64::consts::PI * 10.0 * i as f64 / 250.0).sin();
    }
    let matrix = vec![row.clone(), vec![0.0; n], row];
    let names = vec!["a".into(), "b".into(), "c".into()];
    let result = pipeline::run_pipeline(
        &serde_json::json!({"nodes": [
            {"type": "bandpass", "params": {"l_freq_hz": 1.0, "h_freq_hz": 40.0}},
            {"type": "psd", "params": {"fmin_hz": 0.5, "fmax_hz": 80.0, "n_fft": 256}},
            {"type": "bad_channels"},
        ]}),
        &matrix,
        250.0,
        &names,
        &[],
        None,
        None,
    )
    .unwrap();
    assert_eq!(result.steps.len(), 3);
    let powers = result.band_powers.unwrap();
    assert!(powers.contains_key("alpha"));
    assert!(result.bad_channels.contains(&"b".to_string()));
}

#[test]
fn write_edf_roundtrips_channel_data() {
    let dir = std::env::temp_dir().join(format!("neuro-edf-roundtrip-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("export.edf");
    let data = vec![
        (0..500)
            .map(|i| (i as f64 * 0.01).sin())
            .collect::<Vec<_>>(),
        (0..500)
            .map(|i| (i as f64 * 0.02).cos() * 5.0)
            .collect::<Vec<_>>(),
    ];
    let names = vec!["alpha".to_string(), "beta".to_string()];
    let units = vec!["uV".to_string(), "uV".to_string()];
    neuro_eeg::edf::write_edf(&path, &data, &names, &units, 250.0).unwrap();
    let (read, info) = neuro_eeg::edf::read_edf(&path).unwrap();
    assert_eq!(info.channel_names.len(), 2);
    assert_eq!(read.len(), 2);
    assert_eq!(read[0].len(), 500);
    for (written, got) in data.iter().zip(read.iter()) {
        for (w, g) in written.iter().zip(got.iter()) {
            assert!((w - g).abs() < 0.01, "roundtrip drift {w} vs {g}");
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn eeg_ssp_node_projects_out_noise_subspace() {
    let n = 400usize;
    let mut ch1 = vec![0.0f64; n];
    let mut ch2 = vec![0.0f64; n];
    for i in 0..n {
        ch1[i] = (i as f64 * 0.05).sin();
        ch2[i] = (i as f64 * 0.11).cos() * 0.5;
        if (100..200).contains(&i) {
            let noise = ((i * 37 % 101) as f64 - 50.0) * 10.0;
            ch1[i] += 0.6 * noise;
            ch2[i] += 0.8 * noise;
        }
    }
    let matrix = vec![ch1, ch2];
    let names = vec!["Fz".to_string(), "Cz".to_string()];
    // Direct projection check.
    let noise: Vec<usize> = (100..200).collect();
    let cleaned = pipeline::ssp_project(&matrix, &noise, 1).unwrap();
    for t in 0..n {
        let proj = 0.6 * cleaned[0][t] + 0.8 * cleaned[1][t];
        assert!(
            proj.abs() < 1e-3,
            "sample {t} retains noise component {proj}"
        );
    }
    // Node-level smoke test.
    let result = pipeline::run_pipeline(
        &serde_json::json!({"nodes": [{"type": "ssp", "params": {"start_sample": 100, "end_sample": 200, "n_components": 1}}]}),
        &matrix,
        100.0,
        &names,
        &[],
        None,
        None,
    )
    .unwrap();
    assert!(result.steps[0].starts_with("ssp["));
    assert!(pipeline::run_pipeline(
        &serde_json::json!({"nodes": [{"type": "ssp", "params": {}}]}),
        &matrix,
        100.0,
        &names,
        &[],
        None,
        None,
    )
    .is_err());
}

#[test]
fn ica_topographies_weights_and_grid() {
    use neuro_eeg::pipeline;
    // 4 channels on a scalp-like arc; rank-2 mixture so ICA separates.
    let names: Vec<String> = (0..4).map(|i| format!("eeg{i}")).collect();
    let positions: Vec<[f64; 3]> = (0..4)
        .map(|i| {
            let a = -0.6 + i as f64 * 0.4;
            [a.cos() * 0.1, a.sin() * 0.1, 0.05]
        })
        .collect();
    let n = 400usize;
    let s1: Vec<f64> = (0..n).map(|t| (t as f64 * 0.11).sin()).collect();
    let s2: Vec<f64> = (0..n).map(|t| (t as f64 * 0.043 + 1.0).sin()).collect();
    let mut matrix = vec![vec![0.0f64; n]; 4];
    for t in 0..n {
        matrix[0][t] = s1[t] + 0.5 * s2[t];
        matrix[1][t] = 0.8 * s1[t] - 0.3 * s2[t];
        matrix[2][t] = -0.6 * s1[t] + s2[t];
        matrix[3][t] = 0.2 * s1[t] + 0.9 * s2[t];
    }
    let def = serde_json::json!({"nodes": [
        {"type": "ica", "params": {
            "n_components": 2, "max_iter": 1500, "tol": 1e-9,
            "topographies": true, "positions": positions,
        }},
    ]});
    let result = pipeline::run_pipeline(&def, &matrix, 200.0, &names, &[], None, None).unwrap();
    let topos = result.ica_topographies.expect("topographies requested");
    assert_eq!(topos.len(), 2);
    for topo in &topos {
        assert_eq!(topo.channel_weights.len(), 4);
        let norm: f64 = topo
            .channel_weights
            .iter()
            .map(|v| v * v)
            .sum::<f64>()
            .sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
        assert_eq!(topo.grid_cols, 72);
        assert_eq!(topo.grid_rows, 36);
        let grid = topo.grid.as_ref().unwrap();
        assert_eq!(grid.len(), 72 * 36);
    }
    // Positions count mismatch is an explicit error.
    let bad = serde_json::json!({"nodes": [
        {"type": "ica", "params": {"n_components": 2, "positions": [[0.0,0.0,1.0]]}},
    ]});
    assert!(pipeline::run_pipeline(&bad, &matrix, 200.0, &names, &[], None, None).is_err());
}
