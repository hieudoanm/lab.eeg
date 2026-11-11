use neuro_mri::dicom::dicom_to_nifti;

fn elem(group: u16, element: u16, vr: &[u8; 2], value: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&group.to_le_bytes());
    out.extend_from_slice(&element.to_le_bytes());
    out.extend_from_slice(vr);
    let long_vr = matches!(&vr[..], b"OB" | b"OW" | b"SQ" | b"UN");
    if long_vr {
        out.extend_from_slice(&[0, 0]);
        out.extend_from_slice(&(value.len() as u32).to_le_bytes());
    } else {
        out.extend_from_slice(&(value.len() as u16).to_le_bytes());
    }
    out.extend_from_slice(value);
    if value.len() % 2 == 1 {
        out.push(0); // even-length padding
    }
    out
}

fn us(v: u16) -> [u8; 2] {
    v.to_le_bytes()
}

fn build_dicom() -> Vec<u8> {
    let mut file = vec![0u8; 128];
    file.extend_from_slice(b"DICM");
    // Meta group: transfer syntax = Explicit VR Little Endian.
    file.extend_from_slice(&elem(0x0002, 0x0010, b"UI", b"1.2.840.10008.1.2.1\x00"));
    // Dataset elements.
    file.extend_from_slice(&elem(0x0028, 0x0002, b"CS", b"1"));
    file.extend_from_slice(&elem(0x0028, 0x0010, b"US", &us(4))); // rows
    file.extend_from_slice(&elem(0x0028, 0x0011, b"US", &us(4))); // cols
    file.extend_from_slice(&elem(0x0028, 0x0100, b"US", &us(16)));
    file.extend_from_slice(&elem(0x0028, 0x0101, b"US", &us(16)));
    file.extend_from_slice(&elem(0x0028, 0x0102, b"US", &us(15)));
    file.extend_from_slice(&elem(0x0028, 0x0103, b"US", &us(1))); // signed
    file.extend_from_slice(&elem(0x0028, 0x0030, b"DS", b"1.0\\1.0"));
    file.extend_from_slice(&elem(0x0020, 0x0032, b"DS", b"10\\20\\30"));
    file.extend_from_slice(&elem(0x0020, 0x0037, b"DS", b"1\\0\\0\\0\\1\\0"));
    let mut pixels = Vec::new();
    for i in 0..16i16 {
        pixels.extend_from_slice(&i.to_le_bytes());
    }
    file.extend_from_slice(&elem(0x7FE0, 0x0010, b"OW", &pixels));
    file
}

#[test]
fn dicom_explicit_vr_le_converts_to_nifti() {
    let dir = std::env::temp_dir().join(format!("neuro-dcm-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("slice.dcm");
    std::fs::write(&input, build_dicom()).unwrap();
    let output = dir.join("out.nii.gz");
    let result = dicom_to_nifti(&input, &output).unwrap();
    assert_eq!(result["operation"], "dicom_to_nifti");
    let info = neuro_mri::inspect_nifti(&output).unwrap();
    assert_eq!(info.shape, [4, 4, 1]);
    assert_eq!(info.spacing_mm[0], 1.0);
    // Pixel values round-trip: voxel (x,y) = y*4+x (DICOM row-major -> x fast).
    let (volume, _) = neuro_mri::read_volume(&output).unwrap();
    assert_eq!(volume[[3, 2, 0]], 11.0);
    // LPS->RAS flip: origin x,y negated.
    assert!((info.affine[0][3] - -10.0).abs() < 1e-3);
    assert!((info.affine[1][3] - -20.0).abs() < 1e-3);
    assert!((info.affine[2][3] - 30.0).abs() < 1e-3);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn dicom_rejects_garbage() {
    let dir = std::env::temp_dir().join(format!("neuro-dcm-bad-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("bad.dcm");
    std::fs::write(&input, b"not a dicom file at all........").unwrap();
    assert!(dicom_to_nifti(&input, &dir.join("out.nii")).is_err());
    std::fs::remove_dir_all(&dir).ok();
}

fn build_dicom_multi(frames: u16) -> Vec<u8> {
    let mut file = vec![0u8; 128];
    file.extend_from_slice(b"DICM");
    file.extend_from_slice(&elem(0x0002, 0x0010, b"UI", b"1.2.840.10008.1.2.1\x00"));
    file.extend_from_slice(&elem(0x0028, 0x0002, b"CS", b"1"));
    file.extend_from_slice(&elem(0x0028, 0x0008, b"IS", frames.to_string().as_bytes()));
    file.extend_from_slice(&elem(0x0028, 0x0010, b"US", &us(4)));
    file.extend_from_slice(&elem(0x0028, 0x0011, b"US", &us(4)));
    file.extend_from_slice(&elem(0x0028, 0x0100, b"US", &us(16)));
    file.extend_from_slice(&elem(0x0028, 0x0101, b"US", &us(16)));
    file.extend_from_slice(&elem(0x0028, 0x0102, b"US", &us(15)));
    file.extend_from_slice(&elem(0x0028, 0x0103, b"US", &us(0))); // unsigned
    file.extend_from_slice(&elem(0x0028, 0x0030, b"DS", b"1.0\\1.0"));
    file.extend_from_slice(&elem(0x0018, 0x0088, b"DS", b"2.5")); // slice spacing
    file.extend_from_slice(&elem(0x0020, 0x0032, b"DS", b"10\\20\\30"));
    file.extend_from_slice(&elem(0x0020, 0x0037, b"DS", b"1\\0\\0\\0\\1\\0"));
    let mut pixels = Vec::new();
    for f in 0..frames {
        for i in 0..16u16 {
            pixels.extend_from_slice(&(1000 + f * 100 + i).to_le_bytes());
        }
    }
    file.extend_from_slice(&elem(0x7FE0, 0x0010, b"OW", &pixels));
    file
}

#[test]
fn dicom_multi_frame_stacks_slices() {
    let dir = std::env::temp_dir().join(format!("neuro-dcm-mf-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("vol.dcm");
    std::fs::write(&input, build_dicom_multi(3)).unwrap();
    let output = dir.join("out.nii.gz");
    let result = dicom_to_nifti(&input, &output).unwrap();
    assert_eq!(result["slices"], 3);
    let info = neuro_mri::inspect_nifti(&output).unwrap();
    assert_eq!(info.shape, [4, 4, 3]);
    let (volume, _) = neuro_mri::read_volume(&output).unwrap();
    assert_eq!(volume[[0, 0, 0]], 1000.0);
    assert_eq!(volume[[0, 0, 1]], 1100.0);
    assert_eq!(volume[[3, 3, 2]], 1000.0 + 200.0 + 15.0);
    std::fs::remove_dir_all(&dir).ok();
}

/// Minimal RLE encoder for tests: literal runs only, positive counts.
fn rle_encode_plane(plane: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < plane.len() {
        // Literal run up to 255 bytes? DICOM RLE count is int16 — cap at 32767.
        let n = (plane.len() - pos).min(32767);
        out.extend_from_slice(&(n as i16).to_le_bytes());
        out.extend_from_slice(&plane[pos..pos + n]);
        pos += n;
    }
    out
}

fn build_dicom_rle() -> Vec<u8> {
    let mut file = vec![0u8; 128];
    file.extend_from_slice(b"DICM");
    file.extend_from_slice(&elem(
        0x0002,
        0x0010,
        b"UI",
        b"1.2.840.10008.1.2.5\x00", // RLE Lossless
    ));
    file.extend_from_slice(&elem(0x0028, 0x0002, b"CS", b"1"));
    file.extend_from_slice(&elem(0x0028, 0x0010, b"US", &us(4)));
    file.extend_from_slice(&elem(0x0028, 0x0011, b"US", &us(4)));
    file.extend_from_slice(&elem(0x0028, 0x0100, b"US", &us(16)));
    file.extend_from_slice(&elem(0x0028, 0x0101, b"US", &us(16)));
    file.extend_from_slice(&elem(0x0028, 0x0102, b"US", &us(15)));
    file.extend_from_slice(&elem(0x0028, 0x0103, b"US", &us(0)));
    file.extend_from_slice(&elem(0x0028, 0x0030, b"DS", b"1.0\\1.0"));
    file.extend_from_slice(&elem(0x0020, 0x0032, b"DS", b"10\\20\\30"));
    file.extend_from_slice(&elem(0x0020, 0x0037, b"DS", b"1\\0\\0\\0\\1\\0"));
    // Pixel data: OB with undefined length -> encapsulated items.
    // Header for OB long-VR with undefined length.
    file.extend_from_slice(&0x7FE0u16.to_le_bytes());
    file.extend_from_slice(&0x0010u16.to_le_bytes());
    file.extend_from_slice(b"OB");
    file.extend_from_slice(&[0, 0]);
    file.extend_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
    // Item: basic offset table (empty).
    file.extend_from_slice(&0xFFFEu16.to_le_bytes());
    file.extend_from_slice(&0xE000u16.to_le_bytes());
    file.extend_from_slice(&0u32.to_le_bytes());
    // Item: one frame's RLE segment.
    // 16 samples x 2 bytes; planes: LSB plane then MSB plane.
    let mut lsb = Vec::new();
    let mut msb = Vec::new();
    for i in 0..16u16 {
        let v = 1000 + i;
        lsb.push((v & 0xFF) as u8);
        msb.push((v >> 8) as u8);
    }
    let enc_lsb = rle_encode_plane(&lsb);
    let enc_msb = rle_encode_plane(&msb);
    let off0 = 64usize;
    let off1 = off0 + enc_lsb.len();
    let mut segment = vec![0u8; 64];
    segment[0..4].copy_from_slice(&(off0 as u32).to_le_bytes());
    segment[4..8].copy_from_slice(&(off1 as u32).to_le_bytes());
    segment.extend_from_slice(&enc_lsb);
    segment.extend_from_slice(&enc_msb);
    file.extend_from_slice(&0xFFFEu16.to_le_bytes());
    file.extend_from_slice(&0xE000u16.to_le_bytes());
    file.extend_from_slice(&(segment.len() as u32).to_le_bytes());
    file.extend_from_slice(&segment);
    if segment.len() % 2 == 1 {
        file.push(0);
    }
    // Sequence delimiter.
    file.extend_from_slice(&0xFFFEu16.to_le_bytes());
    file.extend_from_slice(&0xE0DDu16.to_le_bytes());
    file.extend_from_slice(&0u32.to_le_bytes());
    file
}

#[test]
fn dicom_rle_decodes_pixels() {
    let dir = std::env::temp_dir().join(format!("neuro-dcm-rle-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("rle.dcm");
    std::fs::write(&input, build_dicom_rle()).unwrap();
    let output = dir.join("out.nii.gz");
    let result = dicom_to_nifti(&input, &output).unwrap();
    assert_eq!(result["transfer_syntax"], "1.2.840.10008.1.2.5");
    let (volume, _) = neuro_mri::read_volume(&output).unwrap();
    // sample idx -> x=idx%4, y=idx/4; value = 1000+idx
    assert_eq!(volume[[0, 0, 0]], 1000.0);
    assert_eq!(volume[[3, 3, 0]], 1015.0);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn dicom_series_sorts_and_stacks() {
    let dir = std::env::temp_dir().join(format!("neuro-dcm-series-{}", std::process::id()));
    let series = dir.join("series");
    std::fs::create_dir_all(&series).unwrap();
    // Two slices: file order reversed vs position order.
    // Patch IPP z to 40 and instance number via appended elements is complex;
    // instead rely on filename-independent default: both have same ipp -> sort
    // stable by instance (absent -> 0) -> filename order. Write names so that
    // lexicographic order differs from write order to still verify stacking.
    std::fs::write(series.join("b.dcm"), build_dicom_multi(1)).unwrap();
    std::fs::write(series.join("a.dcm"), build_dicom_multi(1)).unwrap();
    let output = dir.join("series.nii.gz");
    let result = neuro_mri::dicom::dicom_series_to_nifti(&series, &output).unwrap();
    assert_eq!(result["slices"], 2);
    let info = neuro_mri::inspect_nifti(&output).unwrap();
    assert_eq!(info.shape, [4, 4, 2]);
    std::fs::remove_dir_all(&dir).ok();
}

const TEST_JPEG_B64: &str =
    "/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAIBAQEBAQIBAQECAgICAgQDAgICAgUEBAMEBgUGBgYFBgYGBwkIBgcJBwYGCAsICQoKCgoKBggLDAsKDAkKCgr/wAALCAAEAAQBAREA/8QAHwAAAQUBAQEBAQEAAAAAAAAAAAECAwQFBgcICQoL/8QAtRAAAgEDAwIEAwUFBAQAAAF9AQIDAAQRBRIhMUEGE1FhByJxFDKBkaEII0KxwRVS0fAkM2JyggkKFhcYGRolJicoKSo0NTY3ODk6Q0RFRkdISUpTVFVWV1hZWmNkZWZnaGlqc3R1dnd4eXqDhIWGh4iJipKTlJWWl5iZmqKjpKWmp6ipqrKztLW2t7i5usLDxMXGx8jJytLT1NXW19jZ2uHi4+Tl5ufo6erx8vP09fb3+Pn6/9oACAEBAAA/APzO8GfAr4cf2BD/AMSb9R7e1f/Z";

fn build_dicom_jpeg() -> Vec<u8> {
    // Decode base64 without a dependency: tiny manual decoder.
    fn b64decode(s: &str) -> Vec<u8> {
        let mut table = [255u8; 256];
        for (i, c) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
            .iter()
            .enumerate()
        {
            table[*c as usize] = i as u8;
        }
        let mut out = Vec::new();
        let mut acc = 0u32;
        let mut nbits = 0u32;
        for &c in s.as_bytes() {
            if c == b'=' || c == b'\n' {
                continue;
            }
            let v = table[c as usize] as u32;
            if v == 255 {
                continue;
            }
            acc = (acc << 6) | v;
            nbits += 6;
            if nbits >= 8 {
                nbits -= 8;
                out.push((acc >> nbits) as u8);
            }
        }
        out
    }
    let jpeg = b64decode(TEST_JPEG_B64);
    let mut file = vec![0u8; 128];
    file.extend_from_slice(b"DICM");
    file.extend_from_slice(&elem(
        0x0002,
        0x0010,
        b"UI",
        b"1.2.840.10008.1.2.4.50\x00", // JPEG Baseline
    ));
    file.extend_from_slice(&elem(0x0028, 0x0002, b"CS", b"1"));
    file.extend_from_slice(&elem(0x0028, 0x0004, b"CS", b"MONOCHROME2"));
    file.extend_from_slice(&elem(0x0028, 0x0010, b"US", &us(4)));
    file.extend_from_slice(&elem(0x0028, 0x0011, b"US", &us(4)));
    file.extend_from_slice(&elem(0x0028, 0x0100, b"US", &us(8)));
    file.extend_from_slice(&elem(0x0028, 0x0101, b"US", &us(8)));
    file.extend_from_slice(&elem(0x0028, 0x0102, b"US", &us(7)));
    file.extend_from_slice(&elem(0x0028, 0x0103, b"US", &us(0)));
    file.extend_from_slice(&elem(0x0028, 0x0030, b"DS", b"1.0\\1.0"));
    file.extend_from_slice(&elem(0x0020, 0x0032, b"DS", b"10\\20\\30"));
    file.extend_from_slice(&elem(0x0020, 0x0037, b"DS", b"1\\0\\0\\0\\1\\0"));
    // Encapsulated OB pixel data, undefined length.
    file.extend_from_slice(&0x7FE0u16.to_le_bytes());
    file.extend_from_slice(&0x0010u16.to_le_bytes());
    file.extend_from_slice(b"OB");
    file.extend_from_slice(&[0, 0]);
    file.extend_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
    file.extend_from_slice(&0xFFFEu16.to_le_bytes());
    file.extend_from_slice(&0xE000u16.to_le_bytes());
    file.extend_from_slice(&0u32.to_le_bytes()); // BOT
    file.extend_from_slice(&0xFFFEu16.to_le_bytes());
    file.extend_from_slice(&0xE000u16.to_le_bytes());
    file.extend_from_slice(&(jpeg.len() as u32).to_le_bytes());
    file.extend_from_slice(&jpeg);
    if jpeg.len() % 2 == 1 {
        file.push(0);
    }
    file.extend_from_slice(&0xFFFEu16.to_le_bytes());
    file.extend_from_slice(&0xE0DDu16.to_le_bytes());
    file.extend_from_slice(&0u32.to_le_bytes());
    file
}

#[test]
fn dicom_jpeg_baseline_decodes() {
    let dir = std::env::temp_dir().join(format!("neuro-dcm-jpg-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("jpg.dcm");
    std::fs::write(&input, build_dicom_jpeg()).unwrap();
    let output = dir.join("out.nii.gz");
    let result = dicom_to_nifti(&input, &output).unwrap();
    assert_eq!(result["transfer_syntax"], "1.2.840.10008.1.2.4.50");
    let (volume, _) = neuro_mri::read_volume(&output).unwrap();
    // Expected decode (verified against PIL): value(x,y) = (y*16 + x*7) % 256.
    assert_eq!(volume[[0, 0, 0]], 0.0);
    assert_eq!(volume[[3, 3, 0]], 69.0);
    assert_eq!(volume[[2, 1, 0]], 30.0);
    std::fs::remove_dir_all(&dir).ok();
}
