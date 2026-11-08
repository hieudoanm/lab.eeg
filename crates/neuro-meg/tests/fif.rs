use neuro_meg::fif;

// Real FIFF: big-endian tag header [kind i32, ftype u32, nbytes i32, next i32].
// next == 0 means the next tag follows immediately after the data.
fn tag(kind: i32, ftype: u32, data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    for value in [kind, ftype as i32, data.len() as i32, 0] {
        out.extend_from_slice(&value.to_be_bytes());
    }
    out.extend_from_slice(data);
    out
}

fn i32_tag(kind: i32, values: &[i32]) -> Vec<u8> {
    let data: Vec<u8> = values.iter().flat_map(|v| v.to_be_bytes()).collect();
    tag(kind, 3, &data)
}

fn f32_tag(kind: i32, value: f32) -> Vec<u8> {
    tag(kind, 4, &value.to_be_bytes())
}

// FIFF ch_info_rec: 96 bytes, name at offset 80.
fn ch_info_struct(name: &str, kind: i32) -> [u8; 96] {
    let mut buf = [0u8; 96];
    buf[8..12].copy_from_slice(&kind.to_be_bytes()); // kind
    buf[16..20].copy_from_slice(&1.0f32.to_be_bytes()); // cal
    buf[72..76].copy_from_slice(&112i32.to_be_bytes()); // unit = tesla
    let bytes = name.as_bytes();
    buf[80..80 + bytes.len().min(16)].copy_from_slice(&bytes[..bytes.len().min(16)]);
    buf
}

fn write_fixture(path: &std::path::Path, nchan: usize, nsamp: usize, sfreq: f32) {
    let mut file = Vec::new();
    file.extend(i32_tag(104, &[101])); // FIFF_BLOCK_START -> FIFFB_MEAS_INFO
    file.extend(i32_tag(200, &[nchan as i32])); // FIFF_NCHAN
    file.extend(f32_tag(201, sfreq)); // FIFF_SFREQ
    for index in 0..nchan {
        file.extend(tag(203, 30, &ch_info_struct(&format!("MEG{index:03}"), 1)));
    }
    file.extend(i32_tag(105, &[101])); // FIFF_BLOCK_END
    file.extend(i32_tag(104, &[102])); // FIFF_BLOCK_START -> FIFFB_RAW_DATA
    let mut data = Vec::new();
    for sample in 0..nsamp {
        for channel in 0..nchan {
            data.extend_from_slice(&((sample * 10 + channel) as f32).to_be_bytes());
        }
    }
    // Split the buffer into two tags to exercise concatenation.
    let split = data.len() / 2;
    file.extend(tag(300, 4, &data[..split]));
    file.extend(tag(300, 4, &data[split..]));
    file.extend(i32_tag(208, &[0])); // FIFF_FIRST_SAMPLE
    file.extend(i32_tag(105, &[102]));
    file.extend(i32_tag(104, &[361])); // FIFFB_MNE_EVENTS
    file.extend(i32_tag(3561, &[10, -1, -1, 20, 10, 20]));
    file.extend(tag(3566, 10, b"auditory\0visual\0"));
    file.extend(i32_tag(105, &[361]));
    file.extend(i32_tag(104, &[359])); // FIFFB_MNE_BAD_CHANNELS
    file.extend(i32_tag(220, &[1])); // FIFF_BAD_CHS ordinals
    file.extend(i32_tag(105, &[359]));
    std::fs::write(path, file).unwrap();
}

#[test]
fn fif_inspect_and_window_roundtrip() {
    let dir = std::env::temp_dir().join(format!("neuro-fif-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test_raw.fif");
    write_fixture(&path, 2, 100, 100.0);

    let info = fif::inspect_fif(&path).unwrap();
    assert_eq!(info.n_channels, 2);
    assert_eq!(info.sampling_frequency_hz, 100.0);
    assert_eq!(info.n_samples, 100);
    assert_eq!(info.channel_names, vec!["MEG000", "MEG001"]);
    assert_eq!(info.event_samples, vec![10, 20]);
    assert_eq!(info.bad_channels, vec!["MEG001"]);
    assert_eq!(info.channel_info[0].unit, 112);

    let window = fif::read_window(&path, 0.0, 0.05, &[], 1000, None, false).unwrap();
    assert_eq!(window.data.len(), 2);
    assert_eq!(window.data[0].len(), 5);
    // sample-major layout: channel 0 at even positions
    assert_eq!(window.data[0][0], 0.0);
    assert_eq!(window.data[1][0], 1.0);
    assert_eq!(window.data[0][4], 40.0);
    assert_eq!(window.data[1][4], 41.0);

    let subset =
        fif::read_window(&path, 0.0, 0.02, &["MEG001".to_string()], 1000, None, false).unwrap();
    assert_eq!(subset.data.len(), 1);
    assert_eq!(subset.data[0][0], 1.0);

    std::fs::remove_dir_all(&dir).ok();
}

const FIFFT_MATRIX: u32 = 0x4000_0000;

/// Matrix tag: f32 data row-major + dims trailer [ncol, nrow, 2].
fn matrix_tag(kind: i32, nrow: usize, ncol: usize, values: &[f32]) -> Vec<u8> {
    assert_eq!(values.len(), nrow * ncol);
    let mut data = Vec::new();
    for value in values {
        data.extend_from_slice(&value.to_be_bytes());
    }
    for value in [ncol as i32, nrow as i32, 2] {
        data.extend_from_slice(&value.to_be_bytes());
    }
    tag(kind, 4 | FIFFT_MATRIX, &data)
}

fn name_tag(kind: i32, names: &[&str]) -> Vec<u8> {
    tag(kind, 10, names.join(":").as_bytes())
}

#[test]
fn fif_compensation_and_projectors_apply() {
    let dir = std::env::temp_dir().join(format!("neuro-fif-comp-{}", std::process::id()));
    std::fs::create_dir_all(&dir).ok();
    let path = dir.join("comp.fif");
    let nchan = 3usize;
    let nsamp = 20usize;
    let mut file = Vec::new();
    file.extend(i32_tag(104, &[101]));
    file.extend(i32_tag(200, &[nchan as i32]));
    file.extend(f32_tag(201, 100.0));
    for index in 0..nchan {
        file.extend(tag(203, 30, &ch_info_struct(&format!("MEG{index:03}"), 1)));
    }
    file.extend(i32_tag(105, &[101]));
    file.extend(i32_tag(104, &[102]));
    let mut data = Vec::new();
    for sample in 0..nsamp {
        for channel in 0..nchan {
            data.extend_from_slice(&((sample * 10 + channel) as f32).to_be_bytes());
        }
    }
    file.extend(tag(300, 4, &data));
    file.extend(i32_tag(105, &[102]));

    // FIFFB_MNE_CTF_COMP(370) > FIFFB_MNE_CTF_COMP_DATA(371):
    // kind=1 (raw ctfkind), calibrated=1, named matrix 1x2.
    file.extend(i32_tag(104, &[370]));
    file.extend(i32_tag(104, &[371]));
    file.extend(i32_tag(3580, &[1])); // FIFF_MNE_CTF_COMP_KIND
    file.extend(i32_tag(3582, &[1])); // FIFF_MNE_CTF_COMP_CALIBRATED
    file.extend(i32_tag(104, &[357])); // FIFFB_MNE_NAMED_MATRIX
    file.extend(matrix_tag(3581, 1, 2, &[0.5, -0.25]));
    file.extend(name_tag(3502, &["MEG000"]));
    file.extend(name_tag(3503, &["MEG000", "MEG001"]));
    file.extend(i32_tag(3504, &[1]));
    file.extend(i32_tag(3505, &[2]));
    file.extend(i32_tag(105, &[357]));
    file.extend(i32_tag(105, &[371]));
    file.extend(i32_tag(105, &[370]));

    // FIFFB_PROJ(313) > FIFFB_PROJ_ITEM(314): one active SSP vector
    // over MEG000/MEG001 = [0.6, 0.8].
    file.extend(i32_tag(104, &[313]));
    file.extend(i32_tag(104, &[314]));
    file.extend(tag(206, 10, b"ssp-test\0")); // FIFF_COMMENT desc
    file.extend(i32_tag(3411, &[1])); // kind = field
    file.extend(i32_tag(3414, &[1])); // nvec
    file.extend(matrix_tag(3415, 1, 2, &[0.6, 0.8]));
    file.extend(name_tag(3417, &["MEG000", "MEG001"]));
    file.extend(i32_tag(3560, &[1])); // active
    file.extend(i32_tag(105, &[314]));
    file.extend(i32_tag(105, &[313]));
    std::fs::write(&path, file).unwrap();

    let info = fif::inspect_fif(&path).unwrap();
    assert_eq!(info.compensation.len(), 1);
    assert_eq!(info.compensation[0].kind, 1);
    assert_eq!(info.compensation[0].row_names, vec!["MEG000"]);
    assert!(info.compensation[0].calibrated);
    assert_eq!(info.projectors.len(), 1);
    assert!(info.projectors[0].active);
    assert_eq!(info.projectors[0].ch_names, vec!["MEG000", "MEG001"]);

    // Compensation grade 1: comp = I - C_1 ->
    // out[MEG000] = 0.5*d0 + 0.25*d1, out[MEG001] = d1.
    // sample 0: d0=0, d1=1 -> out0 = 0.25, out1 = 1.
    let wc = fif::read_window(&path, 0.0, 0.02, &[], 1000, Some(1), false).unwrap();
    assert!((wc.data[0][0] - 0.25).abs() < 1e-6, "{}", wc.data[0][0]);
    assert_eq!(wc.data[1][0], 1.0);
    assert_eq!(wc.data[2][0], 2.0);
    // sample 1: d0=10, d1=11 -> out0 = 5 + 2.75 = 7.75
    assert!((wc.data[0][1] - 7.75).abs() < 1e-5, "{}", wc.data[0][1]);

    // Projector only: P = I - vv^T over {MEG000, MEG001}, v=[0.6,0.8].
    // sample 0: d=[0,1] -> proj = [0,1] - 0.8*[0.6,0.8] = [-0.48, 0.36]
    let wp = fif::read_window(&path, 0.0, 0.01, &[], 1000, None, true).unwrap();
    assert!((wp.data[0][0] - (-0.48)).abs() < 1e-6, "{}", wp.data[0][0]);
    assert!((wp.data[1][0] - 0.36).abs() < 1e-6, "{}", wp.data[1][0]);
    assert_eq!(wp.data[2][0], 2.0);

    // Channel subset still works after transforms.
    let sub =
        fif::read_window(&path, 0.0, 0.01, &["MEG001".to_string()], 1000, None, true).unwrap();
    assert_eq!(sub.data.len(), 1);
    assert!((sub.data[0][0] - 0.36).abs() < 1e-6);

    // Requesting compensation on a file without matrices errors.
    let mut bare = Vec::new();
    bare.extend(i32_tag(104, &[101]));
    bare.extend(i32_tag(200, &[1]));
    bare.extend(f32_tag(201, 100.0));
    bare.extend(tag(203, 30, &ch_info_struct("MEG000", 1)));
    bare.extend(i32_tag(105, &[101]));
    bare.extend(i32_tag(104, &[102]));
    bare.extend(tag(300, 4, &[0f32.to_be_bytes()].concat()));
    bare.extend(i32_tag(105, &[102]));
    std::fs::write(dir.join("bare.fif"), bare).unwrap();
    assert!(fif::read_window(&dir.join("bare.fif"), 0.0, 0.01, &[], 1000, Some(1), false).is_err());

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn fif_writer_roundtrips_through_reader() {
    let dir = std::env::temp_dir().join(format!("neuro-fif-write-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("written.fif");
    let channels = vec![
        fif::FifWriteChannel {
            name: "MEG001".into(),
            kind: 1,
            unit: 112,
            unit_mul: 0,
            cal: 1.0,
            range: 1.0,
            coil_type: 0,
            loc: [0.0; 12],
        },
        fif::FifWriteChannel {
            name: "EEG001".into(),
            kind: 2,
            unit: 107,
            unit_mul: 0,
            cal: 1.0,
            range: 1.0,
            coil_type: 0,
            loc: [0.0; 12],
        },
    ];
    let data = vec![
        (0..20).map(|s| s as f64 * 0.5).collect::<Vec<_>>(),
        (0..20).map(|s| -(s as f64)).collect::<Vec<_>>(),
    ];
    fif::write_fif(&path, 200.0, &channels, &data).unwrap();

    let info = fif::inspect_fif(&path).unwrap();
    assert_eq!(info.n_channels, 2);
    assert_eq!(info.sampling_frequency_hz, 200.0);
    assert_eq!(info.n_samples, 20);
    assert_eq!(info.channel_names, vec!["MEG001", "EEG001"]);
    assert_eq!(info.channel_info[0].kind, 1);
    assert_eq!(info.channel_info[1].unit, 107);

    let window = fif::read_window(&path, 0.0, 0.1, &[], 1000, None, false).unwrap();
    assert_eq!(window.data.len(), 2);
    assert_eq!(window.data[0].len(), 20);
    assert!((window.data[0][10] - 5.0).abs() < 1e-6);
    assert!((window.data[1][10] + 10.0).abs() < 1e-6);
}
