//! DICOM reader converting images to NIfTI volumes. Supports Implicit VR LE,
//! Explicit VR LE/BE transfer syntaxes, uncompressed 8/16-bit pixels,
//! multi-frame images (frames stack along the slice normal), RLE Lossless
//! encapsulation, JPEG Baseline/Extended (8-bit, MONOCHROME1/2 and RGB),
//! and whole-series directory conversion. JPEG-LS, JPEG 2000, and other
//! compressed transfer syntaxes are rejected.

use crate::VolumeInfo;
use neuro_core::CoreError;
use serde_json::{json, Value};
use std::path::Path;

const MAX_DICOM_BYTES: u64 = 2 * 1024 * 1024 * 1024;

#[derive(Clone, Copy, PartialEq, Debug)]
enum TransferSyntax {
    ImplicitLE,
    ExplicitLE,
    ExplicitBE,
}

struct Element {
    group: u16,
    element: u16,
    #[allow(dead_code)]
    vr: Option<[u8; 2]>,
    value_offset: usize,
    value_len: usize,
}

fn u16le(b: &[u8]) -> u16 {
    u16::from_le_bytes([b[0], b[1]])
}
fn u16be(b: &[u8]) -> u16 {
    u16::from_be_bytes([b[0], b[1]])
}
fn u32le(b: &[u8]) -> u32 {
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}
fn u32be(b: &[u8]) -> u32 {
    u32::from_be_bytes([b[0], b[1], b[2], b[3]])
}

fn parse_elements(
    bytes: &[u8],
    mut pos: usize,
    end: usize,
    syntax: TransferSyntax,
) -> Result<Vec<Element>, CoreError> {
    let mut out = Vec::new();
    let explicit = syntax != TransferSyntax::ImplicitLE;
    let big_endian = syntax == TransferSyntax::ExplicitBE;
    let u16r = |b: &[u8]| if big_endian { u16be(b) } else { u16le(b) };
    let u32r = |b: &[u8]| if big_endian { u32be(b) } else { u32le(b) };
    while pos + 8 <= end {
        let group = u16r(&bytes[pos..pos + 2]);
        let element = u16r(&bytes[pos + 2..pos + 4]);
        // Item/delimiter markers end the sequence walk.
        if group == 0xFFFE {
            break;
        }
        let mut cursor = pos + 4;
        let mut vr = None;
        let len;
        if explicit {
            let v = [bytes[cursor], bytes[cursor + 1]];
            vr = Some(v);
            cursor += 2;
            let long_vr = matches!(
                &v,
                b"OB"
                    | b"OD"
                    | b"OF"
                    | b"OL"
                    | b"OV"
                    | b"OW"
                    | b"SQ"
                    | b"UC"
                    | b"UR"
                    | b"UT"
                    | b"UN"
            );
            if long_vr {
                cursor += 2; // reserved
                if cursor + 4 > end {
                    break;
                }
                len = u32r(&bytes[cursor..cursor + 4]) as usize;
                cursor += 4;
            } else {
                if cursor + 2 > end {
                    break;
                }
                len = u16r(&bytes[cursor..cursor + 2]) as usize;
                cursor += 2;
            }
        } else {
            if cursor + 4 > end {
                break;
            }
            len = u32r(&bytes[cursor..cursor + 4]) as usize;
            cursor += 4;
        }
        // Undefined length (0xFFFFFFFF) marks encapsulated/sequence data we
        // cannot walk — record it and stop.
        if len == 0xFFFF_FFFFusize {
            out.push(Element {
                group,
                element,
                vr,
                value_offset: cursor,
                value_len: usize::MAX,
            });
            break;
        }
        if cursor + len > end {
            return Err(CoreError::InvalidInput(
                "DICOM element exceeds file bounds".into(),
            ));
        }
        out.push(Element {
            group,
            element,
            vr,
            value_offset: cursor,
            value_len: len,
        });
        // DICOM pads odd-length values to even; the pad byte is not counted
        // in the element length.
        pos = cursor + len + (len & 1);
    }
    Ok(out)
}

fn ascii_trim(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .trim_end_matches('\0')
        .trim()
        .to_string()
}

fn parse_ui_value(bytes: &[u8]) -> String {
    ascii_trim(bytes)
        .split('\\')
        .next()
        .unwrap_or("")
        .trim()
        .to_string()
}

fn parse_decimal_list(bytes: &[u8]) -> Vec<f64> {
    ascii_trim(bytes)
        .split('\\')
        .filter_map(|part| part.trim().parse::<f64>().ok())
        .collect()
}

fn parse_int(bytes: &[u8], big_endian: bool) -> Option<i64> {
    let text = ascii_trim(bytes);
    if let Ok(v) = text.parse::<i64>() {
        return Some(v);
    }
    match bytes.len() {
        2 => Some(if big_endian {
            i16::from_be_bytes([bytes[0], bytes[1]]) as i64
        } else {
            i16::from_le_bytes([bytes[0], bytes[1]]) as i64
        }),
        4 => Some(if big_endian {
            i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as i64
        } else {
            i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as i64
        }),
        _ => None,
    }
}

/// One decoded DICOM image (single or multi-frame).
struct DicomImage {
    rows: usize,
    cols: usize,
    /// Frames stacked along the slice normal; each frame is rows*cols values.
    frames: Vec<Vec<f32>>,
    spacing: [f64; 3],
    /// Image orientation patient: row cosines then column cosines.
    iop: [f64; 6],
    /// Image position patient (first frame / slice).
    ipp: [f64; 3],
    slice_spacing: f64,
    instance_number: i64,
    syntax_name: String,
}

/// Slice normal from IOP: row_direction x column_direction (LPS).
fn slice_normal(iop: &[f64; 6]) -> [f64; 3] {
    let row = [iop[0], iop[1], iop[2]];
    let col = [iop[3], iop[4], iop[5]];
    [
        row[1] * col[2] - row[2] * col[1],
        row[2] * col[0] - row[0] * col[2],
        row[0] * col[1] - row[1] * col[0],
    ]
}

/// Decode one RLE byte-plane: 16-bit LE segment counts; positive = literal
/// run of `count` bytes, negative = next byte repeated `-count` times.
fn rle_decode_plane(data: &[u8], expected: usize) -> Result<Vec<u8>, CoreError> {
    let mut out = Vec::with_capacity(expected);
    let mut pos = 0usize;
    while pos + 2 <= data.len() && out.len() < expected {
        let count = i16::from_le_bytes([data[pos], data[pos + 1]]) as i32;
        pos += 2;
        if count >= 0 {
            let n = count as usize;
            if pos + n > data.len() {
                return Err(CoreError::InvalidInput(
                    "RLE literal run out of bounds".into(),
                ));
            }
            out.extend_from_slice(&data[pos..pos + n]);
            pos += n;
        } else {
            let n = (-count) as usize;
            if pos >= data.len() {
                return Err(CoreError::InvalidInput(
                    "RLE replicate run out of bounds".into(),
                ));
            }
            out.extend(std::iter::repeat(data[pos]).take(n));
            pos += 1;
        }
    }
    if out.len() != expected {
        return Err(CoreError::InvalidInput(format!(
            "RLE plane decoded {} bytes, expected {expected}",
            out.len()
        )));
    }
    Ok(out)
}

/// Decode one DICOM RLE frame. `segment` is the item's payload: a 64-byte
/// header of 16 little-endian u32 plane offsets followed by PackBits-style
/// planes. Returns interleaved sample bytes.
fn rle_decode_frame(
    segment: &[u8],
    n_samples: usize,
    bytes_per_pixel: usize,
) -> Result<Vec<u8>, CoreError> {
    if segment.len() < 64 {
        return Err(CoreError::InvalidInput(
            "RLE segment header truncated".into(),
        ));
    }
    let mut planes = Vec::with_capacity(bytes_per_pixel);
    for p in 0..bytes_per_pixel {
        let off = u32le(&segment[p * 4..p * 4 + 4]) as usize;
        if off >= segment.len() {
            return Err(CoreError::InvalidInput(
                "RLE plane offset out of bounds".into(),
            ));
        }
        // Plane data runs to the next non-zero offset or end of segment.
        let mut end = segment.len();
        for q in p + 1..16 {
            let o = u32le(&segment[q * 4..q * 4 + 4]) as usize;
            if o > off && o <= segment.len() {
                end = o;
                break;
            }
        }
        planes.push(rle_decode_plane(&segment[off..end], n_samples)?);
    }
    // Interleave: sample s byte b = plane[b][s].
    let mut out = vec![0u8; n_samples * bytes_per_pixel];
    for s in 0..n_samples {
        for b in 0..bytes_per_pixel {
            out[s * bytes_per_pixel + b] = planes[b][s];
        }
    }
    Ok(out)
}

/// Walk encapsulated pixel data items (explicit-VR LE item tags) and return
/// the payload of each frame item after the basic offset table.
fn encapsulated_items<'a>(bytes: &'a [u8], pixel_el: &Element) -> Result<Vec<&'a [u8]>, CoreError> {
    let mut items = Vec::new();
    let mut pos = pixel_el.value_offset;
    let end = bytes
        .len()
        .min(pixel_el.value_offset.saturating_add(1 << 30));
    let mut first = true;
    while pos + 8 <= end {
        let group = u16le(&bytes[pos..pos + 2]);
        let elem = u16le(&bytes[pos + 2..pos + 4]);
        if group == 0xFFFE && elem == 0xE0DD {
            break; // sequence delimiter
        }
        if group != 0xFFFE || elem != 0xE000 {
            return Err(CoreError::InvalidInput(
                "malformed encapsulated DICOM item".into(),
            ));
        }
        let len = u32le(&bytes[pos + 4..pos + 8]) as usize;
        pos += 8;
        if pos + len > bytes.len() {
            return Err(CoreError::InvalidInput(
                "encapsulated item out of bounds".into(),
            ));
        }
        if !(first && len == 0) {
            // First item is the basic offset table (may be empty).
            if !(first && items.is_empty()) {
                items.push(&bytes[pos..pos + len]);
            }
        }
        first = false;
        pos += len + (len & 1);
    }
    Ok(items)
}

/// Parse one DICOM file (native or RLE-encapsulated) into frames of
/// slope/intercept-corrected f32 values.
fn read_dicom_file(path: &Path) -> Result<DicomImage, CoreError> {
    let meta = std::fs::metadata(path)?;
    if meta.len() == 0 || meta.len() > MAX_DICOM_BYTES {
        return Err(CoreError::InvalidInput(
            "DICOM file size out of bounds".into(),
        ));
    }
    let bytes = std::fs::read(path)?;
    if bytes.len() < 132 {
        return Err(CoreError::InvalidInput("file too small for DICOM".into()));
    }
    let has_preamble = &bytes[128..132] == b"DICM";
    let mut syntax = TransferSyntax::ImplicitLE;
    let mut syntax_name = "implicit_vr_le".to_string();
    let mut pos = if has_preamble { 132 } else { 0 };
    let mut rle = false;
    let mut jpeg = false;

    if has_preamble {
        let meta_elems = parse_elements(&bytes, pos, bytes.len(), TransferSyntax::ExplicitLE)?;
        for el in &meta_elems {
            if el.group != 0x0002 {
                break;
            }
            if el.element == 0x0010 {
                let ui = parse_ui_value(&bytes[el.value_offset..el.value_offset + el.value_len]);
                syntax = match ui.as_str() {
                    "1.2.840.10008.1.2" => TransferSyntax::ImplicitLE,
                    "1.2.840.10008.1.2.1" => TransferSyntax::ExplicitLE,
                    "1.2.840.10008.1.2.2" => TransferSyntax::ExplicitBE,
                    "1.2.840.10008.1.2.5" => {
                        rle = true;
                        TransferSyntax::ExplicitLE
                    }
                    "1.2.840.10008.1.2.4.50" | "1.2.840.10008.1.2.4.51" => {
                        jpeg = true;
                        TransferSyntax::ExplicitLE
                    }
                    other => {
                        return Err(CoreError::InvalidInput(format!(
                            "unsupported DICOM transfer syntax: {other}"
                        )))
                    }
                };
                syntax_name = ui;
            }
            pos = el.value_offset + el.value_len + (el.value_len & 1);
        }
    }
    let big_endian = syntax == TransferSyntax::ExplicitBE;
    let elements = parse_elements(&bytes, pos, bytes.len(), syntax)?;
    let find = |group: u16, element: u16| -> Option<&Element> {
        elements
            .iter()
            .find(|e| e.group == group && e.element == element)
    };
    let value = |el: &Element| &bytes[el.value_offset..el.value_offset + el.value_len];

    let rows = find(0x0028, 0x0010)
        .and_then(|el| parse_int(value(el), big_endian))
        .ok_or_else(|| CoreError::InvalidInput("DICOM missing Rows".into()))?;
    let cols = find(0x0028, 0x0011)
        .and_then(|el| parse_int(value(el), big_endian))
        .ok_or_else(|| CoreError::InvalidInput("DICOM missing Columns".into()))?;
    let bits_allocated = find(0x0028, 0x0100)
        .and_then(|el| parse_int(value(el), big_endian))
        .unwrap_or(16);
    let pixel_rep = find(0x0028, 0x0103)
        .and_then(|el| parse_int(value(el), big_endian))
        .unwrap_or(0);
    let samples_per_pixel = find(0x0028, 0x0002)
        .and_then(|el| parse_int(value(el), big_endian))
        .unwrap_or(1);
    let frames = find(0x0028, 0x0008)
        .and_then(|el| parse_int(value(el), big_endian))
        .unwrap_or(1);
    if !(rows > 0 && cols > 0 && rows * cols <= 100_000_000 && frames > 0 && frames <= 4096) {
        return Err(CoreError::InvalidInput("invalid DICOM dimensions".into()));
    }
    let photometric = find(0x0028, 0x0004)
        .map(|el| ascii_trim(value(el)))
        .unwrap_or_default();
    let is_rgb = photometric.contains("RGB");
    let mono1 = photometric.contains("MONOCHROME1");
    let spp_ok = samples_per_pixel == 1 || (jpeg && is_rgb && samples_per_pixel == 3);
    if !spp_ok {
        return Err(CoreError::InvalidInput(
            "unsupported DICOM samples per pixel".into(),
        ));
    }
    if !(matches!(bits_allocated, 8 | 16) || (jpeg && bits_allocated <= 16)) {
        return Err(CoreError::InvalidInput(format!(
            "unsupported DICOM pixel depth: {bits_allocated}"
        )));
    }

    let pixel_el = find(0x7FE0, 0x0010)
        .ok_or_else(|| CoreError::InvalidInput("DICOM missing Pixel Data".into()))?;
    let n = (rows * cols) as usize;
    let bytes_per = (bits_allocated as usize + 7) / 8;
    let slope = find(0x0028, 0x1053)
        .and_then(|el| parse_decimal_list(value(el)).first().copied())
        .unwrap_or(1.0);
    let intercept = find(0x0028, 0x1052)
        .and_then(|el| parse_decimal_list(value(el)).first().copied())
        .unwrap_or(0.0);

    // Decode each frame into raw sample values first.
    let mut frame_raw: Vec<Vec<f64>> = Vec::with_capacity(frames as usize);
    if jpeg {
        let items = encapsulated_items(&bytes, pixel_el)?;
        if items.len() != frames as usize {
            return Err(CoreError::InvalidInput(format!(
                "JPEG frame count {} does not match NumberOfFrames {}",
                items.len(),
                frames
            )));
        }
        for item in items {
            let pixels = zune_jpeg::JpegDecoder::new(item)
                .decode()
                .map_err(|e| CoreError::InvalidInput(format!("DICOM JPEG decode failed: {e}")))?;
            let n_out = n * samples_per_pixel as usize;
            let mut frame = Vec::with_capacity(n);
            if is_rgb {
                // Expect RGB triplets; fall back to luma of first channel.
                if pixels.len() == n * 3 {
                    for px in pixels.chunks_exact(3) {
                        let y = 0.299 * px[0] as f64 + 0.587 * px[1] as f64 + 0.114 * px[2] as f64;
                        frame.push(if mono1 { 255.0 - y } else { y });
                    }
                } else {
                    return Err(CoreError::InvalidInput(
                        "unexpected JPEG pixel count for RGB image".into(),
                    ));
                }
            } else if pixels.len() == n {
                for &b in &pixels {
                    frame.push(if mono1 { 255.0 - b as f64 } else { b as f64 });
                }
            } else if pixels.len() == n * 3 {
                // Decoder expanded grayscale to RGB; take the red channel.
                for px in pixels.chunks_exact(3) {
                    frame.push(if mono1 {
                        255.0 - px[0] as f64
                    } else {
                        px[0] as f64
                    });
                }
            } else {
                return Err(CoreError::InvalidInput(format!(
                    "unexpected JPEG pixel count: {} (expected {n_out})",
                    pixels.len()
                )));
            }
            let _ = n_out;
            frame_raw.push(frame);
        }
    } else if rle {
        let items = encapsulated_items(&bytes, pixel_el)?;
        if items.len() != frames as usize {
            return Err(CoreError::InvalidInput(format!(
                "RLE frame count {} does not match NumberOfFrames {}",
                items.len(),
                frames
            )));
        }
        for item in items {
            let decoded = rle_decode_frame(item, n, bytes_per)?;
            let mut frame = Vec::with_capacity(n);
            for idx in 0..n {
                let off = idx * bytes_per;
                let raw: f64 = match (bits_allocated, pixel_rep) {
                    (16, 0) => {
                        let lo = decoded[off] as i32;
                        let hi = decoded[off + 1] as i32;
                        (hi * 256 + lo) as f64
                    }
                    (16, 1) => {
                        let v = (((decoded[off + 1] as i32) << 8) | decoded[off] as i32) as i16;
                        v as i32 as f64
                    }
                    (8, 0) => decoded[off] as f64,
                    (8, 1) => decoded[off] as i8 as f64,
                    _ => unreachable!(),
                };
                frame.push(raw);
            }
            frame_raw.push(frame);
        }
    } else {
        if pixel_el.value_len == usize::MAX || pixel_el.value_len == 0xFFFF_FFFFusize {
            return Err(CoreError::InvalidInput(
                "compressed/encapsulated DICOM pixel data is not supported".into(),
            ));
        }
        let pixel_bytes = &bytes[pixel_el.value_offset..pixel_el.value_offset + pixel_el.value_len];
        let expected = n * bytes_per * samples_per_pixel as usize * frames as usize;
        if pixel_bytes.len() < expected {
            return Err(CoreError::InvalidInput(
                "DICOM pixel data is truncated".into(),
            ));
        }
        for f in 0..frames as usize {
            let mut frame = Vec::with_capacity(n);
            for idx in 0..n {
                let off = (f * n + idx) * bytes_per;
                let raw: f64 = match (bits_allocated, pixel_rep) {
                    (16, 0) => {
                        let v = if big_endian {
                            u16::from_be_bytes([pixel_bytes[off], pixel_bytes[off + 1]]) as i32
                        } else {
                            u16::from_le_bytes([pixel_bytes[off], pixel_bytes[off + 1]]) as i32
                        };
                        v as f64
                    }
                    (16, 1) => {
                        let v = if big_endian {
                            i16::from_be_bytes([pixel_bytes[off], pixel_bytes[off + 1]]) as i32
                        } else {
                            i16::from_le_bytes([pixel_bytes[off], pixel_bytes[off + 1]]) as i32
                        };
                        v as f64
                    }
                    (8, 0) => pixel_bytes[off] as f64,
                    (8, 1) => pixel_bytes[off] as i8 as f64,
                    _ => unreachable!(),
                };
                frame.push(raw);
            }
            frame_raw.push(frame);
        }
    }
    let frames_out: Vec<Vec<f32>> = frame_raw
        .into_iter()
        .map(|frame| {
            frame
                .iter()
                .map(|v| (v * slope + intercept) as f32)
                .collect()
        })
        .collect();

    let mut spacing = [1.0f64; 3];
    if let Some(el) = find(0x0028, 0x0030) {
        let sp = parse_decimal_list(value(el));
        if sp.len() >= 2 {
            spacing = [sp[0], sp[1], 1.0];
        }
    }
    let slice_spacing = find(0x0018, 0x0088)
        .or_else(|| find(0x0018, 0x0050))
        .and_then(|el| parse_decimal_list(value(el)).first().copied())
        .filter(|v| *v > 0.0)
        .unwrap_or(1.0);
    let mut iop = [1.0f64, 0.0, 0.0, 0.0, 1.0, 0.0];
    if let Some(el) = find(0x0020, 0x0037) {
        let v = parse_decimal_list(value(el));
        if v.len() >= 6 {
            iop[..6].copy_from_slice(&v[..6]);
        }
    }
    let ipp = find(0x0020, 0x0032)
        .map(|el| parse_decimal_list(value(el)))
        .filter(|v| v.len() >= 3)
        .map(|v| [v[0], v[1], v[2]])
        .unwrap_or([0.0, 0.0, 0.0]);
    let instance_number = find(0x0020, 0x0013)
        .and_then(|el| parse_int(value(el), big_endian))
        .unwrap_or(0);
    Ok(DicomImage {
        rows: rows as usize,
        cols: cols as usize,
        frames: frames_out,
        spacing,
        iop,
        ipp,
        slice_spacing,
        instance_number,
        syntax_name,
    })
}

/// Build a NIfTI volume (cols × rows × slices) + header from decoded images.
/// `images` must share geometry; z axis follows the slice normal.
fn images_to_nifti(images: &[DicomImage], output: &Path) -> Result<VolumeInfo, CoreError> {
    if images.is_empty() {
        return Err(CoreError::InvalidInput("no DICOM images".into()));
    }
    let first = &images[0];
    let (cols, rows, nslices) = (first.cols, first.rows, images.len());
    let mut volume = ndarray::Array3::<f32>::zeros((cols, rows, nslices));
    for (z, image) in images.iter().enumerate() {
        if image.cols != cols || image.rows != rows || image.frames.len() != 1 {
            return Err(CoreError::InvalidInput(
                "inconsistent DICOM slice geometry".into(),
            ));
        }
        for idx in 0..cols * rows {
            let y = idx / cols;
            let x = idx % cols;
            volume[[x, y, z]] = image.frames[0][idx];
        }
    }
    // Multi-frame files contribute all frames as slices.
    // (Handled by caller flattening frames into `images`.)

    let iop = first.iop;
    let normal = slice_normal(&iop);
    let nlen = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
    let normal = if nlen > 1e-9 {
        [normal[0] / nlen, normal[1] / nlen, normal[2] / nlen]
    } else {
        [0.0, 0.0, 1.0]
    };
    let sz = first.slice_spacing;
    // DICOM LPS -> NIfTI RAS: negate x and y direction components.
    let col_x = [
        -iop[0] * first.spacing[0],
        -iop[1] * first.spacing[0],
        -iop[2] * first.spacing[0],
    ];
    let col_y = [
        -iop[3] * first.spacing[1],
        -iop[4] * first.spacing[1],
        -iop[5] * first.spacing[1],
    ];
    let col_z = [-normal[0] * sz, -normal[1] * sz, normal[2] * sz];
    let origin = [-first.ipp[0], -first.ipp[1], first.ipp[2]];
    let mut affine = [[0.0f64; 4]; 4];
    for i in 0..3 {
        affine[i][0] = col_x[i];
        affine[i][1] = col_y[i];
        affine[i][2] = col_z[i];
        affine[i][3] = origin[i];
    }
    affine[3][3] = 1.0;

    let mut header = nifti::NiftiHeader::default();
    header.dim[0] = 3;
    header.dim[1] = cols as u16;
    header.dim[2] = rows as u16;
    header.dim[3] = nslices as u16;
    header.pixdim[0] = 1.0;
    header.pixdim[1] = first.spacing[0] as f32;
    header.pixdim[2] = first.spacing[1] as f32;
    header.pixdim[3] = sz as f32;
    header.srow_x = [
        affine[0][0] as f32,
        affine[0][1] as f32,
        affine[0][2] as f32,
        affine[0][3] as f32,
    ];
    header.srow_y = [
        affine[1][0] as f32,
        affine[1][1] as f32,
        affine[1][2] as f32,
        affine[1][3] as f32,
    ];
    header.srow_z = [
        affine[2][0] as f32,
        affine[2][1] as f32,
        affine[2][2] as f32,
        affine[2][3] as f32,
    ];
    header.sform_code = 1;
    nifti::writer::WriterOptions::new(output)
        .reference_header(&header)
        .write_nifti(&volume)
        .map_err(|error| {
            CoreError::InvalidInput(format!("could not write NIfTI volume: {error}"))
        })?;
    crate::inspect_nifti(output)
}

/// Convert a single DICOM file (native or RLE) to NIfTI. Multi-frame files
/// stack frames along the slice normal.
pub fn dicom_to_nifti(input: &Path, output: &Path) -> Result<Value, CoreError> {
    let image = read_dicom_file(input)?;
    // Split multi-frame images into per-slice images.
    let mut slices: Vec<DicomImage> = Vec::new();
    for frame in image.frames {
        slices.push(DicomImage {
            rows: image.rows,
            cols: image.cols,
            frames: vec![frame],
            spacing: image.spacing,
            iop: image.iop,
            ipp: image.ipp,
            slice_spacing: image.slice_spacing,
            instance_number: image.instance_number,
            syntax_name: image.syntax_name.clone(),
        });
    }
    let info = images_to_nifti(&slices, output)?;
    Ok(json!({
        "operation": "dicom_to_nifti",
        "shape": info.shape,
        "spacing_mm": info.spacing_mm,
        "transfer_syntax": image.syntax_name,
        "slices": slices.len(),
        "output": output.to_string_lossy(),
    }))
}

/// Convert a directory of DICOM files (a series) to a single NIfTI volume.
/// Slices are sorted by position along the slice normal, then instance
/// number, then filename. Capped at 1024 slices.
pub fn dicom_series_to_nifti(dir: &Path, output: &Path) -> Result<Value, CoreError> {
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        // Cheap DICOM sniff: "DICM" at offset 128, or no extension with a
        // plausible preamble — attempt parse and keep successes.
        let is_dcm = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("dcm"))
            || std::fs::metadata(&path)
                .ok()
                .and_then(|_m| {
                    std::fs::File::open(&path).ok().and_then(|mut f| {
                        use std::io::{Read, Seek, SeekFrom};
                        f.seek(SeekFrom::Start(128)).ok()?;
                        let mut buf = [0u8; 4];
                        f.read_exact(&mut buf).ok()?;
                        Some(&buf == b"DICM")
                    })
                })
                .unwrap_or(false);
        if is_dcm {
            files.push(path);
        }
    }
    files.sort();
    if files.is_empty() {
        return Err(CoreError::InvalidInput(
            "no DICOM files found in directory".into(),
        ));
    }
    if files.len() > 1024 {
        return Err(CoreError::InvalidInput(
            "DICOM series exceeds 1024 slices".into(),
        ));
    }
    let mut images = Vec::new();
    let mut failures = Vec::new();
    for file in &files {
        match read_dicom_file(file) {
            Ok(image) => {
                if image.frames.len() != 1 {
                    return Err(CoreError::InvalidInput(format!(
                        "multi-frame file in series not supported: {}",
                        file.display()
                    )));
                }
                images.push(image);
            }
            Err(e) => failures.push(format!("{}: {e}", file.display())),
        }
    }
    if images.is_empty() {
        return Err(CoreError::InvalidInput(format!(
            "no readable DICOM files: {}",
            failures.join("; ")
        )));
    }
    if !failures.is_empty() && images.len() < files.len() / 2 {
        return Err(CoreError::InvalidInput(format!(
            "too many unreadable DICOM files: {}",
            failures.join("; ")
        )));
    }
    // Sort by projection on the slice normal of the first image, then
    // instance number for ties.
    let normal = slice_normal(&images[0].iop);
    images.sort_by(|a, b| {
        let pa = a.ipp[0] * normal[0] + a.ipp[1] * normal[1] + a.ipp[2] * normal[2];
        let pb = b.ipp[0] * normal[0] + b.ipp[1] * normal[1] + b.ipp[2] * normal[2];
        pa.partial_cmp(&pb)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.instance_number.cmp(&b.instance_number))
    });
    let info = images_to_nifti(&images, output)?;
    Ok(json!({
        "operation": "dicom_series_to_nifti",
        "shape": info.shape,
        "spacing_mm": info.spacing_mm,
        "slices": images.len(),
        "skipped_files": failures.len(),
        "output": output.to_string_lossy(),
    }))
}
