use neuro_meg::{bids, ctf, kit};

fn be_i32(v: i32) -> [u8; 4] {
    v.to_be_bytes()
}
fn be_i16(v: i16) -> [u8; 2] {
    v.to_be_bytes()
}
fn be_f64(v: f64) -> [u8; 8] {
    v.to_be_bytes()
}
fn pad_str(s: &str, len: usize) -> Vec<u8> {
    let mut v = s.as_bytes().to_vec();
    v.resize(len, 0);
    v.truncate(len);
    v
}

fn write_ctf_fixture(dir: &std::path::Path) {
    // res4: header through channel names
    let n_channels = 2usize;
    let nsamp = 10usize;
    let ntrials = 2usize;
    let mut r = Vec::new();
    r.extend_from_slice(b"MEG41RS\0");
    r.extend(pad_str("app", 256));
    r.extend(pad_str("origin", 256));
    r.extend(pad_str("desc", 256));
    r.extend(be_i16(0)); // no_trials_avgd
    r.extend(pad_str("", 255)); // data_time
    r.extend(pad_str("", 255)); // data_date
    r.extend(be_i32(nsamp as i32)); // no_samples
    r.extend(be_i16(n_channels as i16)); // no_channels (first of pair)
    r.extend(be_i16(0)); // pad
    r.extend(be_f64(250.0)); // sample_rate
    r.extend(be_f64(1.0)); // epoch_time
    r.extend(be_i16(ntrials as i16)); // no_trials
    r.extend(be_i16(0));
    r.extend(be_i32(0)); // preTrigPts
    r.extend(be_i16(0)); // no_trials_done
    r.extend(be_i16(0)); // no_trials_display
    r.extend(be_i32(1)); // save_trials
    r.extend(be_i32(0)); // primaryTrigger
    r.extend(be_i32(0)); // triggerPolarityMask
    r.extend(be_i16(0)); // pad
    r.extend(be_i16(0));
    r.extend(be_i16(0));
    r.extend(be_i16(1)); // trigger_mode=temp(2)
    r.extend(be_i32(0)); // accept_reject
    r.extend(be_i16(0));
    r.extend(be_i16(0)); // run_time_display=temp(1)
    r.extend(be_i32(0)); // zero_Head
    r.extend(be_i32(0)); // artifact_mode
    r.extend(pad_str("run", 32));
    r.extend(pad_str("title", 256));
    r.extend(pad_str("inst", 32));
    r.extend(pad_str("desc2", 32));
    r.extend(pad_str("subj", 32));
    r.extend(pad_str("op", 32));
    r.extend(pad_str("sensor", 56));
    r.extend(be_i32(0));
    r.extend(be_i32(4));
    r.extend(be_i32(0)); // rdlen=temp(2)=4
    r.extend(pad_str("desc", 4)); // run_description
    r.extend(be_i16(0)); // num_filters
    r.extend(pad_str("MEG001", 32));
    r.extend(pad_str("MEG002", 32));
    // Per-channel sensor records (1328 bytes each, big-endian).
    for ch in 0..n_channels {
        let mut rec = vec![0u8; 1328];
        // sensor_type_index i16, original_run_no i16, coil_type i32
        rec[0..2].copy_from_slice(&5i16.to_be_bytes()); // CTFV_MEG_CH
        rec[4..8].copy_from_slice(&1i32.to_be_bytes()); // coil_type
                                                        // proper_gain f64 @8, qgain f64 @16
        let (pg, qg): (f64, f64) = if ch == 0 { (2.0, 4.0) } else { (0.0, 0.0) };
        rec[8..16].copy_from_slice(&pg.to_be_bytes());
        rec[16..24].copy_from_slice(&qg.to_be_bytes());
        // coil @48: pos f64x3 @0, norm f64x3 @32
        if ch == 0 {
            rec[48..56].copy_from_slice(&15.0f64.to_be_bytes());
            rec[96..104].copy_from_slice(&1.0f64.to_be_bytes()); // norm z
        }
        r.extend_from_slice(&rec);
    }
    // Compensation block: ncomp i16 + one 1992-byte record.
    r.extend(be_i16(1));
    let mut comp = vec![0u8; 1992];
    comp[0..6].copy_from_slice(b"MEG001");
    comp[32..36].copy_from_slice(&1i32.to_be_bytes()); // coeff_type
    comp[40..42].copy_from_slice(&2i16.to_be_bytes()); // ncoeff
    comp[42..48].copy_from_slice(b"MEG001"); // sensor 0 label (31-byte field)
    comp[73..79].copy_from_slice(b"MEG002"); // sensor 1 label
    comp[1592..1600].copy_from_slice(&1.0f64.to_be_bytes());
    comp[1600..1608].copy_from_slice(&(-0.5f64).to_be_bytes());
    r.extend_from_slice(&comp);
    std::fs::write(dir.join("test.res4"), &r).unwrap();

    // MarkerFile.mrk: one marker list "Response" with two (trial, sync) pairs.
    std::fs::write(
        dir.join("MarkerFile.mrk"),
        "NUMBER OF MARKERS: 1\nNAME:\nResponse\nNUMBER OF SAMPLES: 2\nLIST OF SAMPLES:\ntrial\tsync\n0\t0.010\n1\t0.020\n",
    )
    .unwrap();

    // meg4: header + per-trial channel-major i32 BE
    let mut m = b"MEG41CP\0".to_vec();
    for trial in 0..ntrials {
        for ch in 0..n_channels {
            for sample in 0..nsamp {
                let v = (trial * 1000 + ch * 100 + sample) as i32;
                m.extend(be_i32(v));
            }
        }
    }
    std::fs::write(dir.join("test.meg4"), &m).unwrap();
}

#[test]
fn ctf_ds_inspect_and_window() {
    let dir = std::env::temp_dir().join(format!("neuro-ctf-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    write_ctf_fixture(&dir);
    let info = ctf::inspect_ctf(&dir).unwrap();
    assert_eq!(info.n_channels, 2);
    assert_eq!(info.sampling_frequency_hz, 250.0);
    assert_eq!(info.n_samples_per_trial, 10);
    assert_eq!(info.n_trials, 2);
    assert_eq!(info.channel_names, vec!["MEG001", "MEG002"]);
    // onset = sync + trial * (nsamp/sfreq) + pre_trig/sfreq
    // trial0: 0.010 -> sample 2.5 -> 3 (round); trial1: 0.020 + 0.04 = 0.06 -> 15
    assert_eq!(info.events.len(), 2);
    assert_eq!(info.events[0].code.as_deref(), Some("Response"));
    assert_eq!(info.events[0].sample, 3);
    assert_eq!(info.events[1].sample, 15);
    // Channel 0 has proper_gain*qgain = 8 -> conv factor 0.125.
    assert!((info.channel_meta[0].conv_factor - 0.125).abs() < 1e-12);
    assert_eq!(info.channel_meta[0].kind, "meg");
    assert_eq!(info.channel_meta[0].unit, "tesla");
    assert_eq!(
        info.channel_meta[0].position_mm.as_deref(),
        Some(&[150.0, 0.0, 0.0][..])
    );
    // positive proper_gain flips the coil normal
    assert_eq!(
        info.channel_meta[0].orientation.as_deref(),
        Some(&[0.0, 0.0, -1.0][..])
    );
    // Channel 1 has zero gains -> conv factor 1.
    assert_eq!(info.channel_meta[1].conv_factor, 1.0);
    // Compensation record parsed (provenance only, not applied).
    assert_eq!(info.compensation.len(), 1);
    assert_eq!(info.compensation[0].sensor_name, "MEG001");
    assert_eq!(info.compensation[0].coeff_type, 1);
    assert_eq!(
        info.compensation[0].sensors,
        vec!["MEG001".to_string(), "MEG002".to_string()]
    );
    assert_eq!(info.compensation[0].coeffs, vec![1.0, -0.5]);

    // 0..0.02s = samples 0..5 of trial 0 (ch0 scaled by 0.125)
    let window = ctf::read_window(&dir, 0.0, 0.02, &[], 1000, None).unwrap();
    assert_eq!(window.data.len(), 2);
    assert_eq!(window.data[0], vec![0.0, 0.125, 0.25, 0.375, 0.5]);
    assert_eq!(window.data[1][0], 100.0);
    // trial 1 starts at sample 10 -> t=0.04s, ch0 raw 1000 x 0.125 = 125
    let w2 = ctf::read_window(&dir, 0.04, 0.044, &[], 1000, None).unwrap();
    assert_eq!(w2.data[0][0], 125.0);

    // Compensation to grade 1: comp = I - C_1 over {MEG001, MEG002}.
    // C_1[MEG001][MEG001] = 1.0*gain(MEG001)/gain(MEG001) = 1.0
    // C_1[MEG001][MEG002] = -0.5*gain(MEG002)/gain(MEG001) = -0.5*1/8 = -0.0625
    // => out[MEG001] = 0.0625 * MEG002, out[MEG002] unchanged.
    let wc = ctf::read_window(&dir, 0.0, 0.02, &[], 1000, Some(1)).unwrap();
    let expected: Vec<f64> = (0..5).map(|i| 0.0625 * (100.0 + i as f64)).collect();
    for (got, want) in wc.data[0].iter().zip(expected.iter()) {
        assert!((got - want).abs() < 1e-9, "{got} != {want}");
    }
    assert_eq!(wc.data[1], vec![100.0, 101.0, 102.0, 103.0, 104.0]);
    // Requesting the stored grade (0) is a no-op.
    let wn = ctf::read_window(&dir, 0.0, 0.02, &[], 1000, Some(0)).unwrap();
    assert_eq!(wn.data[0], window.data[0]);
    std::fs::remove_dir_all(&dir).ok();
}

fn le_i32(v: i32) -> [u8; 4] {
    v.to_le_bytes()
}
fn le_f64(v: f64) -> [u8; 8] {
    v.to_le_bytes()
}

fn write_kit_fixture(path: &std::path::Path, n_channels: usize, nsamp: usize, sfreq: f64) {
    // 10 dir entries x 16 bytes: offset u32, size i32, max i32, count i32
    let mut sections: Vec<(usize, usize, Vec<u8>)> = Vec::new();
    // section data blobs; offsets patched later
    let mut system = Vec::new();
    system.extend(le_i32(3)); // version
    system.extend(le_i32(4)); // revision
    system.extend(le_i32(52)); // sysid
    system.extend(pad_str("KIT-160", 128));
    system.extend(pad_str("Model", 128));
    system.extend(le_i32(n_channels as i32));
    system.extend(pad_str("comment", 256));
    system.extend(le_i32(0));
    system.extend(le_i32(0)); // times
    system.extend(le_i32(0));
    system.extend(le_i32(0));
    system.extend(le_i32(0)); // reserved
    system.extend(le_i32(0)); // dewar
    system.extend(le_i32(0));
    system.extend(le_i32(0));
    system.extend(le_i32(0)); // spare
    system.extend(le_i32(0)); // fll_type hanger
    system.extend(le_i32(0));
    system.extend(le_i32(0));
    system.extend(le_i32(0)); // spare
    system.extend(le_i32(0)); // trigger_type
    system.extend(le_i32(0));
    system.extend(le_i32(0));
    system.extend(le_i32(0)); // spare
    system.extend(le_i32(0)); // adboard
    for _ in 0..29 {
        system.extend(le_i32(0));
    }
    system.extend(le_f64(5.0)); // adc_range f64 (V3R4)
    system.extend(le_i32(0)); // polarity
    system.extend(le_i32(32)); // allocated bits
    system.extend(le_i32(32)); // stored bits
    sections.push((1, system.len(), system));

    let mut channels = Vec::new();
    for i in 0..n_channels {
        channels.extend(le_i32(1)); // magnetometer
        if i == 0 {
            // loc: x, y, z (m), theta, phi — exercised by coil_geometry
            for v in [0.1f64, 0.2, 0.15, 0.0, 0.0] {
                channels.extend(le_f64(v));
            }
        } else {
            for _ in 0..5 {
                channels.extend(le_f64(0.0));
            } // loc
        }
        channels.extend(vec![0u8; 16]); // misc
        channels.extend(pad_str(&format!("MEG{:02}", i + 1), 6));
    }
    sections.push((4, channels.len() / n_channels.max(1), channels));

    let mut calib = Vec::new();
    for _ in 0..n_channels {
        calib.extend(le_f64(0.0));
        calib.extend(le_f64(1.0)); // gain T/V
    }
    sections.push((5, 0, calib));

    let mut amp = Vec::new();
    amp.extend(le_i32(0)); // amp_data -> gains all 1
    sections.push((7, 0, amp));

    let mut acq = Vec::new();
    acq.extend(le_i32(1)); // continuous
    acq.extend(le_f64(sfreq));
    acq.extend(le_i32(0)); // samples_count pad
    acq.extend(le_i32(nsamp as i32));
    sections.push((8, 0, acq));

    let mut raw = Vec::new();
    for sample in 0..nsamp {
        for ch in 0..n_channels {
            raw.extend(le_i32((sample * 10 + ch) as i32));
        }
    }
    sections.push((9, 0, raw));

    // Digitization points section (dir index 26): 8-byte name + 3 x f64.
    let mut dig = Vec::new();
    for (name, x, y, z) in [("fidnz", 0.0, 0.0, 0.1), ("hpi_1", 0.05, 0.0, 0.08)] {
        dig.extend(pad_str(name, 8));
        dig.extend(le_f64(x));
        dig.extend(le_f64(y));
        dig.extend(le_f64(z));
    }
    sections.push((26, 0, dig));

    // layout: dir entries first (27 entries), then section data
    let n_dirs = 27usize;
    let mut offset = n_dirs * 16;
    let mut dir_entries = vec![(0usize, 0usize, 0usize, 0usize); n_dirs];
    let mut blobs: Vec<(usize, Vec<u8>)> = Vec::new();
    for (index, size_hint, blob) in sections {
        let count = if index == 26 { 2 } else { 1 };
        dir_entries[index] = (offset, if index == 4 { size_hint } else { 0 }, count, 0);
        blobs.push((offset, blob));
        offset += blobs.last().unwrap().1.len();
    }
    // dir entry fields: offset u32, size i32 (record size for channels),
    // max_count i32, count i32
    let mut out = Vec::new();
    for (off, sz, count, _) in &dir_entries {
        out.extend(((*off) as u32).to_le_bytes());
        out.extend((*sz as i32).to_le_bytes());
        out.extend(0i32.to_le_bytes());
        out.extend((*count as i32).to_le_bytes());
    }
    // entry0 count must equal n_dirs: patch count field of entry0 (bytes 12..16)
    out[12..16].copy_from_slice(&(n_dirs as i32).to_le_bytes());
    for (off, blob) in blobs {
        out.resize(off, 0);
        out.extend(blob);
    }
    std::fs::write(path, out).unwrap();
}

fn write_kit_trigger_fixture(path: &std::path::Path, nsamp: usize, sfreq: f64) {
    // Two TRIGGER (-1) channels; record = type i32 + channel_no i32 +
    // reserved i32 + 32-byte name = 44 bytes.
    let mut sections: Vec<(usize, usize, Vec<u8>)> = Vec::new();
    let mut system = Vec::new();
    system.extend(le_i32(3));
    system.extend(le_i32(4));
    system.extend(le_i32(52));
    system.extend(pad_str("KIT-160", 128));
    system.extend(pad_str("Model", 128));
    system.extend(le_i32(2)); // n_channels
    system.extend(pad_str("comment", 256));
    system.extend(le_i32(0));
    system.extend(le_i32(0)); // times
    system.extend(le_i32(0));
    system.extend(le_i32(0));
    system.extend(le_i32(0)); // reserved
    system.extend(le_i32(0)); // dewar
    system.extend(le_i32(0));
    system.extend(le_i32(0));
    system.extend(le_i32(0)); // spare
    system.extend(le_i32(0)); // fll_type hanger
    system.extend(le_i32(0));
    system.extend(le_i32(0));
    system.extend(le_i32(0)); // spare
    system.extend(le_i32(0)); // trigger_type
    system.extend(le_i32(0));
    system.extend(le_i32(0));
    system.extend(le_i32(0)); // spare
    system.extend(le_i32(0)); // adboard
    for _ in 0..29 {
        system.extend(le_i32(0));
    }
    system.extend(le_f64(5.0)); // adc_range f64 (V3R4)
    system.extend(le_i32(0));
    system.extend(le_i32(32));
    system.extend(le_i32(32));
    sections.push((1, system.len(), system));

    let mut channels = Vec::new();
    for i in 0..2 {
        channels.extend(le_i32(-1)); // CHANNEL_TRIGGER
        channels.extend(le_i32(i as i32)); // channel_no
        channels.extend(le_i32(0)); // reserved
        channels.extend(pad_str(&format!("TRIG{:02}", i), 32));
    }
    sections.push((4, 44, channels));

    let mut calib = Vec::new();
    for _ in 0..2 {
        calib.extend(le_f64(0.0));
        calib.extend(le_f64(1.0));
    }
    sections.push((5, 0, calib));

    let mut amp = Vec::new();
    amp.extend(le_i32(0));
    sections.push((7, 0, amp));

    let mut acq = Vec::new();
    acq.extend(le_i32(1));
    acq.extend(le_f64(sfreq));
    acq.extend(le_i32(0));
    acq.extend(le_i32(nsamp as i32));
    sections.push((8, 0, acq));

    // ch0 active (raw 0) on samples 0-4; ch1 active on 2-5 and 8.
    let big: i32 = 1_000_000_000;
    let mut raw = Vec::new();
    for s in 0..nsamp {
        raw.extend(le_i32(if s <= 4 { 0 } else { big }));
        raw.extend(le_i32(if (2..=5).contains(&s) || s == 8 {
            0
        } else {
            big
        }));
    }
    sections.push((9, 0, raw));

    let n_dirs = 10usize;
    let mut offset = n_dirs * 16;
    let mut dir_entries = vec![(0usize, 0usize); n_dirs];
    let mut blobs: Vec<(usize, Vec<u8>)> = Vec::new();
    for (index, size_hint, blob) in sections {
        dir_entries[index] = (offset, if index == 4 { size_hint } else { 0 });
        blobs.push((offset, blob));
        offset += blobs.last().unwrap().1.len();
    }
    let mut out = Vec::new();
    for (off, sz) in &dir_entries {
        out.extend((*off as u32).to_le_bytes());
        out.extend((*sz as i32).to_le_bytes());
        out.extend(0i32.to_le_bytes());
        out.extend(1i32.to_le_bytes());
    }
    out[12..16].copy_from_slice(&(n_dirs as i32).to_le_bytes());
    for (off, blob) in blobs {
        out.resize(off, 0);
        out.extend(blob);
    }
    std::fs::write(path, out).unwrap();
}

#[test]
fn kit_trigger_events() {
    let dir = std::env::temp_dir().join(format!("neuro-kit-ev-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("trig.sqd");
    write_kit_trigger_fixture(&path, 10, 100.0);
    let events = neuro_meg::kit::read_events(&path).unwrap();
    // code = bit0(ch0) + 2*bit1(ch1): s0=1, s2=3, s5=2, s8=2
    let codes: Vec<(usize, i64)> = events
        .iter()
        .map(|e| (e.sample, e.value.unwrap()))
        .collect();
    assert_eq!(codes, vec![(0, 1), (2, 3), (5, 2), (8, 2)]);
    assert!((events[1].time_s - 0.02).abs() < 1e-9);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn kit_inspect_and_window() {
    let dir = std::env::temp_dir().join(format!("neuro-kit-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test.sqd");
    write_kit_fixture(&path, 2, 100, 100.0);

    let info = kit::inspect_kit(&path).unwrap();
    assert_eq!(info.n_channels, 2);
    assert_eq!(info.channel_names, vec!["MEG01", "MEG02"]);
    assert!((info.sampling_frequency_hz - 100.0).abs() < 1e-9);
    assert_eq!(info.n_samples, 100);
    // Coil geometry: ALS (0.1, 0.2, 0.15) m -> RAS mm [-200, 100, 150];
    // theta=phi=0 normal [0,0,1] -> rotated [-0, 0, 1].
    assert_eq!(
        info.positions_mm[0].as_deref(),
        Some(&[-200.0, 100.0, 150.0][..])
    );
    assert_eq!(info.orientations[0].as_deref(), Some(&[0.0, 0.0, 1.0][..]));
    assert!(info.positions_mm[1].is_none());
    assert_eq!(info.digitization_points, 2);

    let window = kit::read_window(&path, 0.0, 0.05, &[], 1000).unwrap();
    assert_eq!(window.data[0].len(), 5);
    // conv factor = adc_range/2^16 / amp_gain(1) * gain(1) = 5/65536
    let expected = 0.0 * (5.0 / 4294967296.0);
    assert!((window.data[0][0] - expected).abs() < 1e-12);
    assert!((window.data[1][0] - 1.0 * (5.0 / 4294967296.0)).abs() < 1e-12);
    assert!((window.data[0][1] - 10.0 * (5.0 / 4294967296.0)).abs() < 1e-12);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn bids_overlay_and_delegation() {
    let dir = std::env::temp_dir().join(format!("neuro-bids-test-{}", std::process::id()));
    let meg_dir = dir.join("sub-01").join("meg");
    std::fs::create_dir_all(&meg_dir).unwrap();
    std::fs::write(dir.join("dataset_description.json"), "{}").unwrap();
    // channels.tsv + events.tsv
    std::fs::write(
        meg_dir.join("sub-01_task-test_run-01_channels.tsv"),
        "name\ttype\tunits\tstatus\tdescription\nMEG01\tMEGMAG\tT\tgood\tsensor\nMEG02\tMEG\tT\tbad\tsensor\n",
    )
    .unwrap();
    // gzipped events.tsv variant (only form present, exercises decompression)
    {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder
            .write_all(b"onset\tduration\ttrial_type\n0.25\t0.1\tgz_event\n")
            .unwrap();
        let bytes = encoder.finish().unwrap();
        std::fs::write(meg_dir.join("sub-01_task-test_run-01_events.tsv.gz"), bytes).unwrap();
    }
    std::fs::write(
        dir.join("coordsystem.json"),
        r#"{"MEGCoordinateFrame": "CTF", "HeadDigitization": [{"name":"nasion","x":0,"y":0,"z":0},{"name":"lpa","x":1,"y":1,"z":1}], "AnatomicalLandmark": []}"#,
    )
    .unwrap();
    // minimal fif? use kit fixture named *_meg.sqd instead for simplicity
    write_kit_fixture(
        &meg_dir.join("sub-01_task-test_run-01_meg.sqd"),
        2,
        100,
        100.0,
    );

    // Second run: another *_meg file with its own events sidecar.
    write_kit_fixture(
        &meg_dir.join("sub-01_task-test_run-02_meg.sqd"),
        2,
        50,
        100.0,
    );
    std::fs::write(
        meg_dir.join("sub-01_task-test_run-02_events.tsv"),
        "onset\tduration\ttrial_type\n0.5\t0.1\trun2_event\n",
    )
    .unwrap();
    // run-02's channels sidecar marks MEG01 bad: bad status merges across
    // runs (union), so both MEG01 and MEG02 end up flagged.
    std::fs::write(
        meg_dir.join("sub-01_task-test_run-02_channels.tsv"),
        "name\ttype\tunits\tstatus\nMEG01\tMEGMAG\tT\tbad\nMEG02\tMEG\tT\tgood\n",
    )
    .unwrap();

    let found = bids::find_bids_meg(&dir).unwrap();
    assert_eq!(found.files.len(), 2);
    assert!(found
        .data_path()
        .unwrap()
        .ends_with("sub-01_task-test_run-01_meg.sqd"));
    assert!(found.files[0].channels_path.is_some());
    assert!(found.files[0].events_path.is_some());
    // Recording selector: index and name substring both resolve run-02.
    assert!(bids::data_path_for(&dir, Some("1"))
        .unwrap()
        .ends_with("sub-01_task-test_run-02_meg.sqd"));
    assert!(bids::data_path_for(&dir, Some("run-02"))
        .unwrap()
        .ends_with("sub-01_task-test_run-02_meg.sqd"));
    assert!(bids::data_path_for(&dir, Some("nonexistent")).is_none());

    let meta = bids::inspect_recording(&dir).unwrap();
    assert_eq!(meta.format, "bids_meg");
    assert_eq!(meta.channel_count, 2);
    assert_eq!(meta.bad_channel_names, vec!["MEG01", "MEG02"]);
    assert_eq!(meta.recordings.len(), 2);
    // Per-run channel reconciliation metadata.
    assert_eq!(meta.recording_channels.len(), 2);
    assert!(meta
        .recording_channels
        .values()
        .all(|channels| channels == &vec!["MEG01".to_string(), "MEG02".to_string()]));
    // Events merged across both runs; multi-file events are stem-tagged.
    assert_eq!(meta.events.len(), 2);
    let run1 = meta
        .events
        .iter()
        .find(|e| e.code.as_deref() == Some("gz_event"))
        .unwrap();
    assert_eq!(run1.sample, 25);
    let run2 = meta
        .events
        .iter()
        .find(|e| e.description.as_deref() == Some("sub-01_task-test_run-02:run2_event"))
        .unwrap();
    // run-02 starts at t=1.0s (run-01 is 100 samples @ 100 Hz), so its
    // 0.5 s onset lands at sample 150 / t=1.5 on the concatenated timeline.
    assert_eq!(run2.sample, 150);
    assert!((run2.time_s - 1.5).abs() < 1e-9);
    // Duration covers both runs: 1.0 s + 0.5 s.
    assert!((meta.duration_seconds - 1.5).abs() < 1e-9);
    assert_eq!(meta.coordinate_frame, "ctf");
    assert_eq!(meta.digitization_points, 2);
    assert_eq!(meta.surface_points.len(), 2);
    assert_eq!(meta.surface_points[0].name.as_deref(), Some("nasion"));
    assert_eq!(meta.surface_points[0].position_m, vec![0.0, 0.0, 0.0]);
    assert_eq!(meta.surface_points[1].position_m, vec![1.0, 1.0, 1.0]);

    // Concatenated read across both runs: 1.5 s @ 100 Hz = 150 samples,
    // times continuous from 0 to ~1.49.
    let all =
        neuro_meg::read_window(&dir, 0.0, 1.5, &[], 10_000, Some("all"), None, false).unwrap();
    assert_eq!(all.data.len(), 2);
    assert_eq!(all.times_s.len(), 150);
    assert!((all.times_s[0] - 0.0).abs() < 1e-9);
    assert!((all.times_s[100] - 1.0).abs() < 1e-6);
    assert!((all.effective_sampling_frequency_hz - 100.0).abs() < 1e-9);

    // Single-run read uses absolute timeline times: run-02 occupies
    // [1.0, 1.5]; a window covering it returns offset times.
    let run2_window =
        neuro_meg::read_window(&dir, 1.0, 1.5, &[], 10_000, Some("run-02"), None, false).unwrap();
    assert_eq!(run2_window.times_s.len(), 50);
    assert!((run2_window.times_s[0] - 1.0).abs() < 1e-6);
    // A window outside the selected run is an explicit error.
    assert!(
        neuro_meg::read_window(&dir, 0.0, 0.5, &[], 10_000, Some("run-02"), None, false).is_err()
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn kit_elp_hsp_sidecars_surface_as_points() {
    let dir = std::env::temp_dir().join(format!("neuro-kit-sidecar-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let sqd = dir.join("rec.sqd");
    write_kit_fixture(&sqd, 2, 20, 100.0);
    // Polhemus FastScan text exports: .hsp headshape, .elp fiducials/HPI.
    std::fs::write(
        dir.join("rec.hsp.txt"),
        "1.0 2.0 3.0\nNasion 10.0 20.0 30.0\n",
    )
    .unwrap();
    std::fs::write(dir.join("rec.elp.txt"), "FidNz 4.0 5.0 6.0\n").unwrap();
    let meta = neuro_meg::inspect_recording(&sqd).unwrap();
    assert_eq!(meta.surface_points.len(), 3);
    assert!(meta
        .surface_points
        .iter()
        .any(|p| p.position_m == vec![10.0, 20.0, 30.0]));
    assert!(meta
        .surface_points
        .iter()
        .any(|p| p.position_m == vec![4.0, 5.0, 6.0]));
    // The .elp fiducial point also counts toward digitization points.
    assert!(meta.digitization_points >= 1);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn bids_concat_rejects_missing_channels() {
    let dir = std::env::temp_dir().join(format!("neuro-bids-missing-{}", std::process::id()));
    let meg_dir = dir.join("sub-01").join("meg");
    std::fs::create_dir_all(&meg_dir).unwrap();
    std::fs::write(dir.join("dataset_description.json"), "{}").unwrap();
    // run-01: 2 channels; run-02: only MEG01 -> montage mismatch.
    write_kit_fixture(
        &meg_dir.join("sub-01_task-test_run-01_meg.sqd"),
        2,
        50,
        100.0,
    );
    write_kit_fixture(
        &meg_dir.join("sub-01_task-test_run-02_meg.sqd"),
        1,
        50,
        100.0,
    );
    let err = neuro_meg::read_window(&dir, 0.0, 1.0, &[], 10_000, Some("all"), None, false)
        .unwrap_err()
        .to_string();
    assert!(err.contains("missing channels"), "error: {err}");
    assert!(err.contains("MEG02"), "error: {err}");
    assert!(err.contains("run-02"), "error: {err}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn meg_export_writes_npy() {
    let dir = std::env::temp_dir().join(format!("neuro-export-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let kit_path = dir.join("rec.sqd");
    write_kit_fixture(&kit_path, 2, 20, 100.0);
    let out = dir.join("out.npy");
    neuro_meg::export::export_window(&kit_path, 0.0, 0.2, &[], &out, None, None, false).unwrap();
    let bytes = std::fs::read(&out).unwrap();
    assert_eq!(&bytes[0..6], b"\x93NUMPY");
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("'shape': (2, 20)"), "header: {text}");
    // payload: 2 channels x 20 samples x 4 bytes f32, after the npy header
    let header_len = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
    assert_eq!(bytes.len(), 10 + header_len + 2 * 20 * 4);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn meg_export_writes_csv() {
    let dir = std::env::temp_dir().join(format!("neuro-export-csv-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let kit_path = dir.join("rec.sqd");
    write_kit_fixture(&kit_path, 2, 20, 100.0);
    let names = vec!["MEG01".to_string(), "MEG02".to_string()];
    let out = dir.join("out.csv");
    neuro_meg::export::export_window_csv(&kit_path, 0.0, 0.2, &names, &out, None, None, false)
        .unwrap();
    let text = std::fs::read_to_string(&out).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], "time_s,MEG01,MEG02");
    assert_eq!(lines.len(), 21); // header + 20 samples
                                 // First data row: t=0, values in scientific notation.
    let cols: Vec<&str> = lines[1].split(',').collect();
    assert_eq!(cols.len(), 3);
    assert!(cols[1].contains('e'));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn meg_export_writes_bids_dataset() {
    let dir = std::env::temp_dir().join(format!("neuro-export-bids-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let kit_path = dir.join("rec.sqd");
    write_kit_fixture(&kit_path, 2, 20, 100.0);
    let out_dir = dir.join("bids_out");
    neuro_meg::export::export_window_bids(
        &kit_path,
        0.0,
        0.2,
        &[],
        &out_dir,
        "testrec",
        None,
        None,
        false,
    )
    .unwrap();
    assert!(out_dir.join("testrec_meg.npy").exists());
    let channels = std::fs::read_to_string(out_dir.join("channels.tsv")).unwrap();
    let lines: Vec<&str> = channels.lines().collect();
    assert_eq!(lines[0], "name\ttype\tunits\tstatus");
    assert_eq!(lines.len(), 3); // header + 2 channels
    let description: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(out_dir.join("dataset_description.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(description["DatasetType"], "meg");
    std::fs::remove_dir_all(&dir).ok();
}
