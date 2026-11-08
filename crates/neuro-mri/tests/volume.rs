use neuro_mri::validate_limits;
use neuro_mri::VolumeInfo;

#[test]
fn volume_limits_reject_excessive_spacing() {
    let volume = VolumeInfo {
        shape: [2, 2, 2],
        voxel_count: 8,
        spacing_mm: [1.0, 1.0, 101.0],
        datatype: "Float32".into(),
        affine: [[0.0; 4]; 4],
    };
    assert!(validate_limits(&volume, 100, 100.0).is_err());
}

#[test]
fn synthetic_volume_supports_stats_and_projection() {
    use ndarray::Array3;
    use std::path::PathBuf;

    let dir = std::env::temp_dir().join(format!(
        "neuro-mri-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path: PathBuf = dir.join("vol.nii.gz");

    let mut volume = Array3::<f32>::zeros((4, 4, 4));
    volume[[1, 1, 1]] = 10.0;
    volume[[2, 2, 2]] = 20.0;
    volume[[3, 3, 3]] = -5.0;
    nifti::writer::WriterOptions::new(&path)
        .write_nifti(&volume)
        .unwrap();

    let info = neuro_mri::inspect_nifti(&path).unwrap();
    assert_eq!(info.shape, [4, 4, 4]);
    assert!(info.affine.iter().flatten().any(|v| v.abs() > 0.0));

    let stats = neuro_mri::volume_statistics(&path).unwrap();
    assert_eq!(stats.voxel_count, 64);
    assert_eq!(stats.valid_sample_count, 64);
    assert_eq!(stats.max_value, 20.0);

    let mip = neuro_mri::max_intensity_projection(&path, "z").unwrap();
    assert_eq!(mip.width, 4);
    assert_eq!(mip.height, 4);
    assert_eq!(mip.max_value, 20.0);
    assert!(mip.pixels.contains(&20.0));

    let hist = neuro_mri::intensity_histogram(&path, 8).unwrap();
    assert_eq!(hist.counts.iter().sum::<usize>(), 64);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn zscore_normalize_writes_valid_volume() {
    use ndarray::Array3;
    use std::path::PathBuf;

    let dir = std::env::temp_dir().join(format!(
        "neuro-mri-z-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let input: PathBuf = dir.join("in.nii");
    let output: PathBuf = dir.join("out.nii.gz");

    let mut volume = Array3::<f32>::zeros((3, 3, 3));
    for i in 0..27 {
        volume[(i % 3, (i / 3) % 3, i / 9)] = i as f32;
    }
    nifti::writer::WriterOptions::new(&input)
        .write_nifti(&volume)
        .unwrap();

    let info = neuro_mri::zscore_normalize(&input, &output).unwrap();
    assert_eq!(info.shape, [3, 3, 3]);
    let stats = neuro_mri::volume_statistics(&output).unwrap();
    assert!((stats.mean).abs() < 1e-3);
    assert!((stats.std_dev - 1.0).abs() < 1e-2);

    std::fs::remove_dir_all(&dir).ok();
}

fn write_test_volume(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
    use ndarray::Array3;
    let path = dir.join(name);
    let mut volume = Array3::<f32>::zeros((6, 6, 6));
    for z in 0..6 {
        for y in 0..6 {
            for x in 0..6 {
                volume[[x, y, z]] = (x * 6 + y) as f32 + if z == 3 { 100.0 } else { 0.0 };
            }
        }
    }
    nifti::writer::WriterOptions::new(&path)
        .write_nifti(&volume)
        .unwrap();
    path
}

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "neuro-mri-op-{}-{}-{}",
        tag,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn threshold_and_segment_produce_expected_counts() {
    let dir = temp_dir("thr");
    let input = write_test_volume(&dir, "in.nii");
    let out = dir.join("thr.nii.gz");
    let stats = neuro_mri::operations::threshold(&input, &out, Some(50.0), None).unwrap();
    assert!(stats["voxels_masked"].as_u64().unwrap() > 0);

    let mask = dir.join("seg.nii.gz");
    let seg = neuro_mri::operations::segment_threshold(&input, &mask, Some(95.0), None).unwrap();
    assert!(seg["mask_voxels"].as_u64().unwrap() > 0);
    assert!(seg["region_volume_mm3"].as_f64().unwrap() > 0.0);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn smooth_and_resample_write_valid_volumes() {
    let dir = temp_dir("ops");
    let input = write_test_volume(&dir, "in.nii");
    let smooth = dir.join("sm.nii.gz");
    neuro_mri::operations::gaussian_smooth(&input, &smooth, 2.0).unwrap();
    let info = neuro_mri::inspect_nifti(&smooth).unwrap();
    assert_eq!(info.shape, [6, 6, 6]);

    let resampled = dir.join("rs.nii.gz");
    let result =
        neuro_mri::operations::resample(&input, &resampled, [2.0, 2.0, 2.0], "trilinear").unwrap();
    let out_shape: Vec<usize> = result["output_shape"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as usize)
        .collect();
    assert_eq!(out_shape.len(), 3);

    // Cubic spline resampling produces a valid volume too.
    let cubic = dir.join("rs_cubic.nii.gz");
    let cubic_result =
        neuro_mri::operations::resample(&input, &cubic, [2.0, 2.0, 2.0], "cubic").unwrap();
    assert_eq!(cubic_result["method"], "cubic");
    let cubic_info = neuro_mri::inspect_nifti(&cubic).unwrap();
    assert_eq!(cubic_info.shape.len(), 3);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn bias_field_correct_reduces_smooth_intensity_gradient() {
    use ndarray::Array3;
    let dir = temp_dir("bias");
    let input = dir.join("biased.nii");
    // Strong smooth multiplicative bias ramp along x plus a constant base.
    let mut volume = Array3::<f32>::zeros((8, 8, 8));
    for z in 0..8 {
        for y in 0..8 {
            for x in 0..8 {
                let bias = 0.9 * (x as f64 - 3.5) / 3.5;
                volume[[x, y, z]] = (100.0 * bias.exp()) as f32;
            }
        }
    }
    nifti::writer::WriterOptions::new(&input)
        .write_nifti(&volume)
        .unwrap();
    let out = dir.join("debias.nii.gz");
    let stats = neuro_mri::operations::bias_field_correct(&input, &out, 4.0, 2).unwrap();
    assert_eq!(stats["operation"], "bias_field_correct");
    let log_std = |path: &std::path::Path| -> f64 {
        let (vol, _) = neuro_mri::read_volume(path).unwrap();
        let logs: Vec<f64> = vol
            .iter()
            .filter(|v| v.is_finite() && **v > 0.0)
            .map(|v| (*v as f64).ln())
            .collect();
        let mean = logs.iter().sum::<f64>() / logs.len() as f64;
        (logs.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / logs.len() as f64).sqrt()
    };
    let before = log_std(&input);
    let after = log_std(&out);
    assert!(
        after < before * 0.7,
        "bias not reduced: {before} -> {after}"
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn register_rigid_recovers_known_translation() {
    use ndarray::Array3;
    let dir = temp_dir("reg");
    // Deterministic asymmetric phantom.
    let mut fixed = Array3::<f32>::zeros((8, 8, 8));
    for z in 0..8 {
        for y in 0..8 {
            for x in 0..8 {
                fixed[[x, y, z]] =
                    ((x * 7 + y * 3 + z * 5) % 13) as f32 * 10.0 + ((x * x + y) % 7) as f32;
            }
        }
    }
    // Moving = fixed shifted by +1 voxel along x (content moved right).
    let mut moving = Array3::<f32>::zeros((8, 8, 8));
    for z in 0..8 {
        for y in 0..8 {
            for x in 1..8 {
                moving[[x, y, z]] = fixed[[x - 1, y, z]];
            }
        }
    }
    let fixed_path = dir.join("fixed.nii");
    let moving_path = dir.join("moving.nii");
    nifti::writer::WriterOptions::new(&fixed_path)
        .write_nifti(&fixed)
        .unwrap();
    nifti::writer::WriterOptions::new(&moving_path)
        .write_nifti(&moving)
        .unwrap();
    let out = dir.join("registered.nii.gz");
    let stats =
        neuro_mri::operations::register_rigid(&fixed_path, &moving_path, &out, 400).unwrap();
    let t = stats["translation_mm"].as_array().unwrap();
    // Expect ~ -1 mm on x (sample moving one voxel back), small elsewhere.
    assert!(
        (t[0].as_f64().unwrap() + 1.0).abs() < 0.75,
        "translation: {stats}"
    );
    assert!(t[1].as_f64().unwrap().abs() < 0.75, "translation: {stats}");
    assert!(t[2].as_f64().unwrap().abs() < 0.75, "translation: {stats}");
    let before = stats["ncc_before"].as_f64().unwrap();
    let after = stats["ncc_after"].as_f64().unwrap();
    assert!(after > before + 0.05, "ncc {before} -> {after}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn register_affine_recovers_scale_and_translation() {
    use ndarray::Array3;
    let dir = temp_dir("rega");
    // Phantom with a localized blob so affine has signal to lock onto.
    let mut fixed = Array3::<f32>::zeros((10, 10, 10));
    for z in 0..10 {
        for y in 0..10 {
            for x in 0..10 {
                let dx = x as f32 - 5.0;
                let dy = y as f32 - 5.0;
                let dz = z as f32 - 5.0;
                fixed[[x, y, z]] = (-(dx * dx + dy * dy + dz * dz) / 8.0).exp() * 100.0
                    + ((x * 11 + y * 5 + z * 3) % 7) as f32;
            }
        }
    }
    // Moving = fixed scaled by 1.1 about the centre (voxel units ~mm here),
    // then shifted +1 voxel on x. In world coords: p_m = 1.1*(p_f - c) + c + t.
    // We approximate with voxel coords (isotropic 1mm spacing assumed by nifti writer default).
    let mut moving = Array3::<f32>::zeros((10, 10, 10));
    for z in 0..10usize {
        for y in 0..10usize {
            for x in 0..10usize {
                // Inverse-map: for each fixed voxel find moving source.
                let fx = x as f64 - 4.5;
                let fy = y as f64 - 4.5;
                let fz = z as f64 - 4.5;
                // forward: m = 1.1*f + shift -> sample moving at 1.1*f + shift
                let mx = 1.1 * fx + 1.0 + 4.5;
                let my = 1.1 * fy + 4.5;
                let mz = 1.1 * fz + 4.5;
                let xi = mx.round() as isize;
                let yi = my.round() as isize;
                let zi = mz.round() as isize;
                if (0..10).contains(&xi) && (0..10).contains(&yi) && (0..10).contains(&zi) {
                    moving[[x, y, z]] = fixed[[xi as usize, yi as usize, zi as usize]];
                }
            }
        }
    }
    let fixed_path = dir.join("fixed.nii");
    let moving_path = dir.join("moving.nii");
    nifti::writer::WriterOptions::new(&fixed_path)
        .write_nifti(&fixed)
        .unwrap();
    nifti::writer::WriterOptions::new(&moving_path)
        .write_nifti(&moving)
        .unwrap();
    let out = dir.join("affine.nii.gz");
    let stats =
        neuro_mri::operations::register_affine(&fixed_path, &moving_path, &out, 3000).unwrap();
    assert_eq!(stats["operation"], "register_affine");
    let before = stats["ncc_before"].as_f64().unwrap();
    let after = stats["ncc_after"].as_f64().unwrap();
    assert!(after > before + 0.02, "ncc {before} -> {after}");
    // Translation should recover roughly the +1 voxel x-shift (~ -1mm sign
    // convention as in the rigid test: moving content shifted +x means the
    // optimizer translates the sampling by about -1mm on x).
    let t = stats["translation_mm"].as_array().unwrap();
    assert!(
        t[0].as_f64().unwrap().abs() > 0.3,
        "expected nonzero x translation, got {stats}"
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn skull_strip_keeps_inner_blob_drops_shell() {
    use ndarray::Array3;
    let dir = temp_dir("strip");
    // 16^3 volume: dark bg, mid shell (skull) radius 6-7, bright core radius<5.
    let mut vol = Array3::<f32>::zeros((16, 16, 16));
    for z in 0..16usize {
        for y in 0..16usize {
            for x in 0..16usize {
                let d = ((x as f64 - 7.5).powi(2)
                    + (y as f64 - 7.5).powi(2)
                    + (z as f64 - 7.5).powi(2))
                .sqrt();
                vol[[x, y, z]] = if d < 5.0 {
                    300.0
                } else if d < 7.0 {
                    150.0
                } else {
                    0.0
                } as f32;
            }
        }
    }
    let input = dir.join("head.nii");
    nifti::writer::WriterOptions::new(&input)
        .write_nifti(&vol)
        .unwrap();
    let mask = dir.join("brain.nii.gz");
    let stats = neuro_mri::operations::skull_strip(&input, &mask, None, 1.5).unwrap();
    let brain = stats["brain_voxels"].as_u64().unwrap() as usize;
    let head = stats["head_voxels"].as_u64().unwrap() as usize;
    assert!(brain > 0 && brain < head, "brain {brain} head {head}");
    // Mask must exclude the outer shell: read back and check a shell voxel is 0.
    let (mask_vol, _) = neuro_mri::read_volume(&mask).unwrap();
    assert_eq!(mask_vol[[0, 0, 0]], 0.0);
    assert_eq!(mask_vol[[7, 7, 7]], 1.0);
    assert_eq!(mask_vol[[1, 8, 8]], 0.0, "shell voxel should be excluded");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn register_rigid_mutual_information_recovers_translation() {
    use ndarray::Array3;
    let dir = temp_dir("regmi");
    let mut fixed = Array3::<f32>::zeros((8, 8, 8));
    for z in 0..8 {
        for y in 0..8 {
            for x in 0..8 {
                fixed[[x, y, z]] =
                    ((x * 7 + y * 3 + z * 5) % 13) as f32 * 10.0 + ((x * x + y) % 7) as f32;
            }
        }
    }
    let mut moving = Array3::<f32>::zeros((8, 8, 8));
    for z in 0..8 {
        for y in 0..8 {
            for x in 1..8 {
                moving[[x, y, z]] = fixed[[x - 1, y, z]];
            }
        }
    }
    let fixed_path = dir.join("fixed.nii");
    let moving_path = dir.join("moving.nii");
    nifti::writer::WriterOptions::new(&fixed_path)
        .write_nifti(&fixed)
        .unwrap();
    nifti::writer::WriterOptions::new(&moving_path)
        .write_nifti(&moving)
        .unwrap();
    let out = dir.join("registered.nii.gz");
    let stats = neuro_mri::operations::register_rigid_with_metric(
        &fixed_path,
        &moving_path,
        &out,
        "mi",
        400,
    )
    .unwrap();
    let t = stats["translation_mm"].as_array().unwrap();
    assert!(
        (t[0].as_f64().unwrap() + 1.0).abs() < 0.9,
        "MI translation: {stats}"
    );
    std::fs::remove_dir_all(&dir).ok();
}
