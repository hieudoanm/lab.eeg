//! Bounded reader for Neuromag/MNE FIF (`.fif`) files.
//!
//! Implements the tag-level FIFF container: big-endian tag headers
//! (`kind, ftype, nbytes, next`), `next`-pointer traversal, block
//! start/end markers, meas-info fields (nchan, sfreq, `ch_info`
//! structs), bad-channel ordinals, MNE event lists, and concatenated
//! float32/float64 `FIFF_DATA_BUFFER` payloads stored sample-major
//! (`data[sample * nchan + channel]`).
//!
//! Coordinate frames, projectors, compensation, and head-position
//! correction are parsed where cheap but not applied to the data.

use crate::{MegChannel, MegWindow};
use neuro_core::CoreError;
use std::path::Path;

const FIFF_BLOCK_START: i32 = 104;
const FIFF_BLOCK_END: i32 = 105;
const FIFFT_INT: u32 = 3;
const FIFFT_FLOAT: u32 = 4;
const FIFFT_DOUBLE: u32 = 5;
const FIFFT_STRING: u32 = 10;
const FIFFT_CH_INFO_STRUCT: u32 = 30;
const FIFFB_MEAS_INFO: i32 = 101;
const FIFFB_RAW_DATA: i32 = 102;
const FIFFB_CONTINUOUS_DATA: i32 = 112;
const FIFFB_MNE_EVENTS: i32 = 361;
const FIFF_NCHAN: i32 = 200;
const FIFF_SFREQ: i32 = 201;
const FIFF_CH_INFO: i32 = 203;
const FIFF_FIRST_SAMPLE: i32 = 208;
const FIFF_BAD_CHS: i32 = 220;
const FIFF_DATA_BUFFER: i32 = 300;
const FIFF_MNE_EVENT_LIST: i32 = 3561;
const FIFF_MNE_EVENT_COMMENTS: i32 = 3566;
const FIFF_TAG_HEADER: usize = 16;
const FIFFV_NEXT_SEQ: i32 = 0;
const FIFFV_NEXT_NONE: i32 = -1;
const MAX_TAGS: usize = 1_000_000;
const MAX_CHANNELS: usize = 4096;
const MAX_SAMPLES: usize = 200_000_000;
const CH_INFO_SIZE: usize = 96;
const FIFFT_MATRIX: u32 = 0x4000_0000;
const FIFFB_PROJ_ITEM: i32 = 314;
const FIFFB_MNE_CTF_COMP_DATA: i32 = 371;
const FIFF_COMMENT: i32 = 206;
const FIFF_NAME: i32 = 233;
const FIFF_MNE_ROW_NAMES: i32 = 3502;
const FIFF_MNE_COL_NAMES: i32 = 3503;
const FIFF_MNE_NROW: i32 = 3504;
const FIFF_MNE_NCOL: i32 = 3505;
const FIFF_MNE_CTF_COMP_KIND: i32 = 3580;
const FIFF_MNE_CTF_COMP_DATA_TAG: i32 = 3581;
const FIFF_MNE_CTF_COMP_CALIBRATED: i32 = 3582;
const FIFF_PROJ_ITEM_KIND: i32 = 3411;
const FIFF_PROJ_ITEM_NVEC: i32 = 3414;
const FIFF_PROJ_ITEM_VECTORS: i32 = 3415;
const FIFF_PROJ_ITEM_CH_NAME_LIST: i32 = 3417;
const FIFF_MNE_PROJ_ITEM_ACTIVE: i32 = 3560;
const FIFF_MNE_ICA_PCA_EXPLAINED_VAR: i32 = 3605;
const MAX_MATRIX_ELEMENTS: usize = 4_000_000;

#[derive(Clone, Debug)]
struct Tag {
    kind: i32,
    ftype: u32,
    nbytes: usize,
    data_start: usize,
}

fn be_i32(bytes: &[u8], pos: usize) -> Option<i32> {
    Some(i32::from_be_bytes(
        bytes.get(pos..pos + 4)?.try_into().ok()?,
    ))
}

fn be_u32(bytes: &[u8], pos: usize) -> Option<u32> {
    Some(u32::from_be_bytes(
        bytes.get(pos..pos + 4)?.try_into().ok()?,
    ))
}

fn be_f32(bytes: &[u8], pos: usize) -> Option<f32> {
    Some(f32::from_be_bytes(
        bytes.get(pos..pos + 4)?.try_into().ok()?,
    ))
}

fn be_f64(bytes: &[u8], pos: usize) -> Option<f64> {
    Some(f64::from_be_bytes(
        bytes.get(pos..pos + 8)?.try_into().ok()?,
    ))
}

fn read_tag(bytes: &[u8], pos: usize) -> Option<(Tag, usize)> {
    if pos + FIFF_TAG_HEADER > bytes.len() {
        return None;
    }
    let (kind, ftype, nbytes, next) = (
        be_i32(bytes, pos)?,
        be_u32(bytes, pos + 4)?,
        be_i32(bytes, pos + 8)?,
        be_i32(bytes, pos + 12)?,
    );
    if !(-1..=100_000).contains(&kind) || nbytes < 0 || nbytes > (1 << 28) {
        return None;
    }
    let data_start = pos.checked_add(FIFF_TAG_HEADER)?;
    if data_start.checked_add(nbytes as usize)? > bytes.len() {
        return None;
    }
    let next_pos = if next == FIFFV_NEXT_SEQ {
        pos.checked_add(FIFF_TAG_HEADER)?
            .checked_add(nbytes as usize)?
    } else if next == FIFFV_NEXT_NONE {
        return None;
    } else if next > 0 {
        next as usize
    } else {
        return None;
    };
    Some((
        Tag {
            kind,
            ftype,
            nbytes: nbytes as usize,
            data_start,
        },
        next_pos,
    ))
}

fn scan_tags(bytes: &[u8]) -> Result<Vec<Tag>, CoreError> {
    let mut tags = Vec::new();
    let mut pos = 0usize;
    for _ in 0..MAX_TAGS {
        let Some((tag, next)) = read_tag(bytes, pos) else {
            break;
        };
        tags.push(tag);
        if next <= pos || next >= bytes.len() {
            break;
        }
        pos = next;
    }
    if tags.is_empty() {
        return Err(CoreError::InvalidInput(
            "not a FIF file: no readable tags".into(),
        ));
    }
    Ok(tags)
}

fn tag_f64(tag: &Tag, bytes: &[u8]) -> Option<f64> {
    match tag.ftype {
        FIFFT_FLOAT if tag.nbytes >= 4 => be_f32(bytes, tag.data_start).map(f64::from),
        FIFFT_DOUBLE if tag.nbytes >= 8 => be_f64(bytes, tag.data_start),
        _ => None,
    }
}

fn tag_text(tag: &Tag, bytes: &[u8]) -> Option<String> {
    if tag.ftype != FIFFT_STRING || tag.nbytes == 0 {
        return None;
    }
    let raw = &bytes[tag.data_start..tag.data_start + tag.nbytes];
    String::from_utf8(raw.to_vec()).ok()
}

fn tag_string(tag: &Tag, bytes: &[u8]) -> Option<String> {
    if tag.ftype != FIFFT_STRING || tag.nbytes == 0 {
        return None;
    }
    let raw = &bytes[tag.data_start..tag.data_start + tag.nbytes];
    let end = raw.iter().position(|byte| *byte == 0).unwrap_or(raw.len());
    String::from_utf8(raw[..end].to_vec()).ok()
}

#[derive(Clone, Debug)]
pub struct FifChannelInfo {
    pub name: String,
    pub kind: i32,
    pub unit: i32,
    pub unit_mul: i32,
    pub cal: f64,
    pub range: f64,
    pub coil_type: i32,
    pub loc: [f64; 12],
}

fn channel_structs(tag: &Tag, bytes: &[u8]) -> Vec<FifChannelInfo> {
    if tag.ftype != FIFFT_CH_INFO_STRUCT {
        return Vec::new();
    }
    let count = tag.nbytes / CH_INFO_SIZE;
    (0..count)
        .map(|index| {
            let base = tag.data_start + index * CH_INFO_SIZE;
            let name_raw = &bytes[base + 80..base + 96];
            let end = name_raw
                .iter()
                .position(|b| *b == 0)
                .unwrap_or(name_raw.len());
            let mut loc = [0.0f64; 12];
            for (i, slot) in loc.iter_mut().enumerate() {
                *slot = be_f32(bytes, base + 24 + i * 4).unwrap_or(0.0) as f64;
            }
            FifChannelInfo {
                name: String::from_utf8_lossy(&name_raw[..end]).trim().to_string(),
                kind: be_i32(bytes, base + 8).unwrap_or(0),
                unit: be_i32(bytes, base + 72).unwrap_or(0),
                unit_mul: be_i32(bytes, base + 76).unwrap_or(0),
                cal: be_f32(bytes, base + 16).unwrap_or(1.0) as f64,
                range: be_f32(bytes, base + 12).unwrap_or(1.0) as f64,
                coil_type: be_i32(bytes, base + 20).unwrap_or(0),
                loc,
            }
        })
        .collect()
}

/// A CTF compensation matrix stored inside a FIF file
/// (`FIFFB_MNE_CTF_COMP_DATA` named matrix).
#[derive(Clone, Debug)]
pub struct FifCompMatrix {
    /// Compensation grade (MNE `kind` after ctfkind mapping: 1, 2, 3).
    pub kind: i32,
    pub calibrated: bool,
    pub row_names: Vec<String>,
    pub col_names: Vec<String>,
    /// Row-major `nrow x ncol` coefficients.
    pub data: Vec<Vec<f64>>,
}

/// An SSP/projection item (`FIFFB_PROJ_ITEM`).
#[derive(Clone, Debug)]
pub struct FifProjector {
    pub desc: String,
    pub kind: i32,
    pub active: bool,
    pub nvec: usize,
    pub ch_names: Vec<String>,
    /// `nvec` vectors, each `nchan` long, row-major as stored.
    pub vectors: Vec<Vec<f64>>,
    pub explained_var: Option<f64>,
}

pub struct FifInfo {
    pub n_channels: usize,
    pub sampling_frequency_hz: f64,
    pub n_samples: usize,
    pub first_sample: usize,
    pub channel_names: Vec<String>,
    pub channel_info: Vec<FifChannelInfo>,
    pub description: Option<String>,
    pub event_samples: Vec<usize>,
    pub event_comments: Vec<String>,
    pub bad_channels: Vec<String>,
    pub compensation: Vec<FifCompMatrix>,
    pub projectors: Vec<FifProjector>,
}

fn block_id(tag: &Tag, bytes: &[u8]) -> Option<i32> {
    if tag.kind == FIFF_BLOCK_START && tag.ftype == FIFFT_INT && tag.nbytes >= 4 {
        be_i32(bytes, tag.data_start)
    } else {
        None
    }
}

pub fn inspect_fif(path: &Path) -> Result<FifInfo, CoreError> {
    let bytes = std::fs::read(path)?;
    let tags = scan_tags(&bytes)?;
    let mut n_channels = None;
    let mut sfreq = None;
    let mut first_sample = 0usize;
    let mut channel_info: Vec<FifChannelInfo> = Vec::new();
    let mut description = None;
    let mut data_samples = 0usize;
    // Each open block keeps the tags seen inside it (propagated to all
    // ancestors) so projector/comp blocks can be assembled on close.
    let mut block_stack: Vec<(i32, Vec<Tag>)> = Vec::new();
    let mut bad_ordinals: Vec<i32> = Vec::new();
    let mut event_samples: Vec<i64> = Vec::new();
    let mut event_comments: Vec<String> = Vec::new();
    let mut compensation: Vec<FifCompMatrix> = Vec::new();
    let mut projectors: Vec<FifProjector> = Vec::new();
    for tag in &tags {
        if let Some(id) = block_id(tag, &bytes) {
            block_stack.push((id, Vec::new()));
            continue;
        }
        if tag.kind == FIFF_BLOCK_END {
            if let Some((id, contents)) = block_stack.pop() {
                match id {
                    FIFFB_MNE_CTF_COMP_DATA => {
                        if let Some(matrix) = build_comp_matrix(&contents, &bytes) {
                            compensation.push(matrix);
                        }
                    }
                    FIFFB_PROJ_ITEM => {
                        if let Some(projector) = build_projector(&contents, &bytes) {
                            projectors.push(projector);
                        }
                    }
                    _ => {}
                }
            }
            continue;
        }
        for (_, contents) in block_stack.iter_mut() {
            contents.push(tag.clone());
        }
        let in_block = |wanted: &[i32]| block_stack.iter().any(|(id, _)| wanted.contains(id));
        if in_block(&[FIFFB_MNE_EVENTS])
            && tag.kind == FIFF_MNE_EVENT_LIST
            && tag.ftype == FIFFT_INT
        {
            event_samples = bytes[tag.data_start..tag.data_start + tag.nbytes]
                .chunks_exact(4)
                .filter_map(|chunk| {
                    i64::from(i32::from_be_bytes(chunk.try_into().ok()?))
                        .try_into()
                        .ok()
                })
                .collect();
        }
        if in_block(&[FIFFB_MNE_EVENTS]) && tag.kind == FIFF_MNE_EVENT_COMMENTS {
            if let Some(text) = tag_text(tag, &bytes) {
                event_comments = text
                    .split('\0')
                    .filter(|part| !part.is_empty())
                    .map(str::to_string)
                    .collect();
            }
        }
        if tag.kind == FIFF_BAD_CHS && tag.ftype == FIFFT_INT && bad_ordinals.is_empty() {
            bad_ordinals = bytes[tag.data_start..tag.data_start + tag.nbytes]
                .chunks_exact(4)
                .filter_map(|chunk| be_i32(chunk, 0))
                .collect();
        }
        if in_block(&[FIFFB_MEAS_INFO]) {
            match tag.kind {
                FIFF_NCHAN => n_channels = be_i32(&bytes, tag.data_start).map(|v| v as usize),
                FIFF_SFREQ => sfreq = tag_f64(tag, &bytes),
                FIFF_CH_INFO => channel_info.extend(channel_structs(tag, &bytes)),
                212 => description = tag_string(tag, &bytes),
                _ => {}
            }
        }
        if in_block(&[FIFFB_RAW_DATA, FIFFB_CONTINUOUS_DATA]) {
            if tag.kind == FIFF_FIRST_SAMPLE {
                first_sample = be_i32(&bytes, tag.data_start).unwrap_or(0).max(0) as usize;
            }
            if tag.kind == FIFF_DATA_BUFFER
                && matches!(tag.ftype, FIFFT_FLOAT | FIFFT_DOUBLE)
                && tag.nbytes > 0
            {
                let elem = if tag.ftype == FIFFT_FLOAT { 4 } else { 8 };
                if tag.nbytes % elem == 0 {
                    data_samples += tag.nbytes / elem;
                }
            }
        }
    }
    let n_channels = n_channels
        .filter(|value| *value > 0 && *value <= MAX_CHANNELS)
        .ok_or_else(|| CoreError::InvalidInput("FIF meas info missing nchan".into()))?;
    let sfreq = sfreq
        .filter(|value| *value > 0.0 && value.is_finite())
        .ok_or_else(|| CoreError::InvalidInput("FIF meas info missing sfreq".into()))?;
    if data_samples == 0 || data_samples % n_channels != 0 {
        return Err(CoreError::InvalidInput(
            "FIF raw data buffer does not match channel count".into(),
        ));
    }
    let n_samples = data_samples / n_channels;
    if n_samples > MAX_SAMPLES {
        return Err(CoreError::InvalidInput("FIF recording too large".into()));
    }
    if channel_info.len() < n_channels {
        channel_info = (0..n_channels)
            .map(|index| FifChannelInfo {
                name: format!("MEG {index:04}"),
                kind: 1,
                unit: 112,
                unit_mul: 0,
                cal: 1.0,
                range: 1.0,
                coil_type: 0,
                loc: [0.0; 12],
            })
            .collect();
    }
    channel_info.truncate(n_channels);
    let channel_names: Vec<String> = channel_info
        .iter()
        .enumerate()
        .map(|(index, info)| {
            if info.name.is_empty() {
                format!("CH{index:04}")
            } else {
                info.name.clone()
            }
        })
        .collect();
    let bad_channels = bad_ordinals
        .iter()
        .filter_map(|&ord| {
            let index = if ord >= 0 {
                ord as usize
            } else {
                (-ord - 1) as usize
            };
            channel_names.get(index).cloned()
        })
        .collect();
    // Event list is stored as [sample, prev, next] triples.
    let mut samples = Vec::new();
    for chunk in event_samples.chunks(3) {
        if let Some(&sample) = chunk.first() {
            if sample >= 0 {
                samples.push(sample as usize);
            }
        }
    }
    samples.sort_unstable();
    samples.dedup();
    Ok(FifInfo {
        n_channels,
        sampling_frequency_hz: sfreq,
        n_samples,
        first_sample,
        channel_names,
        channel_info,
        description,
        event_samples: samples,
        event_comments,
        bad_channels,
        compensation,
        projectors,
    })
}

pub fn info_events(info: &FifInfo) -> Vec<crate::MegEvent> {
    info.event_samples
        .iter()
        .enumerate()
        .map(|(index, &sample)| crate::MegEvent {
            sample,
            time_s: sample as f64 / info.sampling_frequency_hz,
            code: info
                .event_comments
                .get(index)
                .map(|comment| comment.trim().to_string())
                .filter(|comment| !comment.is_empty()),
            value: None,
            description: info
                .event_comments
                .get(index)
                .map(|comment| comment.trim().to_string())
                .filter(|comment| !comment.is_empty()),
        })
        .collect()
}

/// Parse a FIFF matrix tag: row-major `nrow x ncol` elements followed by a
/// dims trailer (`ncol, nrow, ndim` as big-endian i32s; MNE writes the
/// reversed shape then ndim). Returns `(nrow, ncol, flat_data)`.
fn matrix_payload(tag: &Tag, bytes: &[u8]) -> Option<(usize, usize, Vec<f64>)> {
    let base_type = tag.ftype & !FIFFT_MATRIX;
    let elem = match base_type {
        FIFFT_FLOAT => 4usize,
        FIFFT_DOUBLE => 8usize,
        _ => return None,
    };
    if tag.nbytes < 12 || tag.data_start + tag.nbytes > bytes.len() {
        return None;
    }
    let payload_end = tag.data_start + tag.nbytes;
    // Last i32 is ndim; the ndim i32s before it hold the reversed shape.
    let ndim = be_i32(bytes, payload_end - 4)?;
    if ndim != 2 {
        return None;
    }
    let ncol = be_i32(bytes, payload_end - 12)? as usize;
    let nrow = be_i32(bytes, payload_end - 8)? as usize;
    let count = nrow.checked_mul(ncol)?;
    if count == 0 || count > MAX_MATRIX_ELEMENTS {
        return None;
    }
    let data_bytes = count.checked_mul(elem)?;
    if data_bytes + 12 != tag.nbytes {
        return None;
    }
    let mut data = Vec::with_capacity(count);
    for i in 0..count {
        let offset = tag.data_start + i * elem;
        data.push(if elem == 8 {
            be_f64(bytes, offset)?
        } else {
            be_f32(bytes, offset)? as f64
        });
    }
    Some((nrow, ncol, data))
}

/// Read a colon-separated FIFF name list (`{COLON}` escapes colons).
fn name_list(text: &str) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    text.split(':')
        .map(|part| part.replace("{COLON}", ":"))
        .collect()
}

fn find_tag_in<'a>(contents: &'a [Tag], kind: i32) -> Option<&'a Tag> {
    contents.iter().find(|tag| tag.kind == kind)
}

/// Assemble a `FifCompMatrix` from the tags collected inside a
/// `FIFFB_MNE_CTF_COMP_DATA` block (named-matrix tags are included because
/// contents propagate to all ancestor blocks).
fn build_comp_matrix(contents: &[Tag], bytes: &[u8]) -> Option<FifCompMatrix> {
    let data_tag = find_tag_in(contents, FIFF_MNE_CTF_COMP_DATA_TAG)?;
    let (mat_nrow, mat_ncol, flat) = matrix_payload(data_tag, bytes)?;
    let nrow = find_tag_in(contents, FIFF_MNE_NROW)
        .and_then(|tag| be_i32(bytes, tag.data_start))
        .map(|v| v.max(0) as usize)
        .unwrap_or(mat_nrow);
    let cols = find_tag_in(contents, FIFF_MNE_NCOL)
        .and_then(|tag| be_i32(bytes, tag.data_start))
        .map(|v| v.max(0) as usize)
        .unwrap_or(mat_ncol)
        .max(1);
    if nrow * cols != flat.len() {
        return None;
    }
    let data: Vec<Vec<f64>> = flat.chunks(cols).map(|chunk| chunk.to_vec()).collect();
    let ctfkind = find_tag_in(contents, FIFF_MNE_CTF_COMP_KIND)
        .and_then(|tag| be_i32(bytes, tag.data_start))
        .unwrap_or(0) as u32;
    // MNE `_add_kind`: known CTF kinds map to grades 1/2/3.
    let kind = match ctfkind {
        0x4731_4252 => 1,
        0x4732_4252 => 2,
        0x4733_4252 => 3,
        other => other as i32,
    };
    let calibrated = find_tag_in(contents, FIFF_MNE_CTF_COMP_CALIBRATED)
        .and_then(|tag| be_i32(bytes, tag.data_start))
        .is_some_and(|value| value != 0);
    let row_names = find_tag_in(contents, FIFF_MNE_ROW_NAMES)
        .and_then(|tag| tag_string(tag, bytes))
        .map(|text| name_list(&text))
        .unwrap_or_default();
    let col_names = find_tag_in(contents, FIFF_MNE_COL_NAMES)
        .and_then(|tag| tag_string(tag, bytes))
        .map(|text| name_list(&text))
        .unwrap_or_default();
    Some(FifCompMatrix {
        kind,
        calibrated,
        row_names,
        col_names,
        data,
    })
}

/// Assemble a `FifProjector` from tags inside a `FIFFB_PROJ_ITEM` block.
fn build_projector(contents: &[Tag], bytes: &[u8]) -> Option<FifProjector> {
    let desc = find_tag_in(contents, FIFF_COMMENT)
        .or_else(|| find_tag_in(contents, FIFF_NAME))
        .and_then(|tag| tag_string(tag, bytes))
        .unwrap_or_default();
    let kind = find_tag_in(contents, FIFF_PROJ_ITEM_KIND)
        .and_then(|tag| be_i32(bytes, tag.data_start))
        .unwrap_or(0);
    let nvec = find_tag_in(contents, FIFF_PROJ_ITEM_NVEC)
        .and_then(|tag| be_i32(bytes, tag.data_start))
        .map(|v| v.max(0) as usize)
        .unwrap_or(0);
    let ch_names = find_tag_in(contents, FIFF_PROJ_ITEM_CH_NAME_LIST)
        .and_then(|tag| tag_string(tag, bytes))
        .map(|text| name_list(&text))
        .unwrap_or_default();
    let vectors_tag = find_tag_in(contents, FIFF_PROJ_ITEM_VECTORS)?;
    let (_nrow, ncol, flat) = matrix_payload(vectors_tag, bytes)?;
    if nvec == 0 || ncol != ch_names.len() || flat.len() % ncol != 0 {
        return None;
    }
    let rows = flat.len() / ncol;
    if rows != nvec {
        return None;
    }
    let vectors: Vec<Vec<f64>> = flat.chunks(ncol).map(|chunk| chunk.to_vec()).collect();
    let active = find_tag_in(contents, FIFF_MNE_PROJ_ITEM_ACTIVE)
        .and_then(|tag| be_i32(bytes, tag.data_start))
        .is_some_and(|value| value != 0);
    let explained_var =
        find_tag_in(contents, FIFF_MNE_ICA_PCA_EXPLAINED_VAR).and_then(|tag| tag_f64(tag, bytes));
    Some(FifProjector {
        desc,
        kind,
        active,
        nvec,
        ch_names,
        vectors,
        explained_var,
    })
}

/// Locate all data-buffer tags inside raw/continuous blocks, in file order.
fn data_buffer_tags(bytes: &[u8]) -> Result<Vec<Tag>, CoreError> {
    let tags = scan_tags(bytes)?;
    let mut buffers = Vec::new();
    let mut block_stack: Vec<i32> = Vec::new();
    for tag in tags {
        if let Some(id) = block_id(&tag, bytes) {
            block_stack.push(id);
            continue;
        }
        if tag.kind == FIFF_BLOCK_END {
            block_stack.pop();
            continue;
        }
        let in_raw = block_stack
            .iter()
            .any(|id| *id == FIFFB_RAW_DATA || *id == FIFFB_CONTINUOUS_DATA);
        if in_raw
            && tag.kind == FIFF_DATA_BUFFER
            && matches!(tag.ftype, FIFFT_FLOAT | FIFFT_DOUBLE)
            && tag.nbytes > 0
        {
            buffers.push(tag);
        }
    }
    if buffers.is_empty() {
        return Err(CoreError::InvalidInput(
            "FIF raw data buffer not found".into(),
        ));
    }
    Ok(buffers)
}

/// Stored compensation grade: modal `coil_type >> 16` across MEG channels
/// (MNE `get_current_comp`).
fn stored_comp_grade(info: &FifInfo) -> i32 {
    let mut counts: std::collections::BTreeMap<i32, usize> = std::collections::BTreeMap::new();
    for channel in &info.channel_info {
        if fif_kind_name(channel.kind) == "meg" {
            let grade = channel.coil_type >> 16;
            if grade >= 0 {
                *counts.entry(grade).or_default() += 1;
            }
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(grade, _)| grade)
        .unwrap_or(0)
}

/// Dense inverse via Gauss-Jordan with partial pivoting.
fn invert_matrix(matrix: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, CoreError> {
    let n = matrix.len();
    let mut aug = vec![vec![0.0f64; 2 * n]; n];
    for i in 0..n {
        aug[i][..n].copy_from_slice(&matrix[i]);
        aug[i][n + i] = 1.0;
    }
    for col in 0..n {
        let mut pivot = col;
        for row in col + 1..n {
            if aug[row][col].abs() > aug[pivot][col].abs() {
                pivot = row;
            }
        }
        if aug[pivot][col].abs() < 1e-12 {
            return Err(CoreError::InvalidInput(
                "singular compensation matrix".into(),
            ));
        }
        aug.swap(col, pivot);
        let div = aug[col][col];
        for v in aug[col].iter_mut() {
            *v /= div;
        }
        for row in 0..n {
            if row == col {
                continue;
            }
            let factor = aug[row][col];
            if factor == 0.0 {
                continue;
            }
            for k in 0..2 * n {
                aug[row][k] -= factor * aug[col][k];
            }
        }
    }
    Ok((0..n).map(|i| aug[i][n..].to_vec()).collect())
}

/// Calibrated coefficient matrix for one comp grade over `universe`
/// channel indices, replicating MNE `_calibrate_comp` + `_make_compensator`:
/// uncalibrated matrices get `data[i][j] *= rowcal_i * colcal_j` with
/// `rowcal = range*cal`, `colcal = 1/(range*cal)` of the named channels.
fn fif_comp_grade_matrix(
    info: &FifInfo,
    comp: &FifCompMatrix,
    universe: &[usize],
    pos: &std::collections::HashMap<usize, usize>,
) -> Vec<Vec<f64>> {
    let name_to_index: std::collections::HashMap<&str, usize> = info
        .channel_names
        .iter()
        .enumerate()
        .map(|(index, name)| (name.as_str(), index))
        .collect();
    let cal_product = |name: &str| -> f64 {
        name_to_index
            .get(name)
            .map(|&index| {
                let channel = &info.channel_info[index];
                let product = channel.range * channel.cal;
                if product.is_finite() && product != 0.0 {
                    product
                } else {
                    1.0
                }
            })
            .unwrap_or(1.0)
    };
    let size = universe.len();
    let mut matrix = vec![vec![0.0f64; size]; size];
    for (row_index, row_name) in comp.row_names.iter().enumerate() {
        let Some(&row_channel) = name_to_index.get(row_name.as_str()) else {
            continue;
        };
        let Some(&row_pos) = pos.get(&row_channel) else {
            continue;
        };
        let row_cal = if comp.calibrated {
            1.0
        } else {
            cal_product(row_name)
        };
        for (col_index, col_name) in comp.col_names.iter().enumerate() {
            let Some(value) = comp.data.get(row_index).and_then(|row| row.get(col_index)) else {
                continue;
            };
            let Some(&col_channel) = name_to_index.get(col_name.as_str()) else {
                continue;
            };
            let Some(&col_pos) = pos.get(&col_channel) else {
                continue;
            };
            let col_cal = if comp.calibrated {
                1.0
            } else {
                1.0 / cal_product(col_name)
            };
            matrix[row_pos][col_pos] += value * row_cal * col_cal;
        }
    }
    matrix
}

/// Apply FIF CTF compensation to channel-major data (rows = all channels,
/// raw units), replicating MNE `make_compensator(from_, to)`:
/// `comp = (I - C_to) * inv(I - C_from)` over the involved channel set.
fn apply_fif_compensation(
    info: &FifInfo,
    data: &mut [Vec<f64>],
    target_grade: i32,
) -> Result<(), CoreError> {
    if info.compensation.is_empty() {
        return Ok(());
    }
    let from_grade = stored_comp_grade(info);
    if from_grade == target_grade {
        return Ok(());
    }
    let name_to_index: std::collections::HashMap<&str, usize> = info
        .channel_names
        .iter()
        .enumerate()
        .map(|(index, name)| (name.as_str(), index))
        .collect();
    let mut involved: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
    for comp in &info.compensation {
        for name in comp.row_names.iter().chain(comp.col_names.iter()) {
            if let Some(&index) = name_to_index.get(name.as_str()) {
                involved.insert(index);
            }
        }
    }
    let universe: Vec<usize> = involved.into_iter().collect();
    if universe.is_empty() {
        return Ok(());
    }
    if universe.len() > 1024 {
        return Err(CoreError::InvalidInput(
            "compensation channel set too large".into(),
        ));
    }
    let pos: std::collections::HashMap<usize, usize> = universe
        .iter()
        .enumerate()
        .map(|(position, &channel)| (channel, position))
        .collect();
    let size = universe.len();
    let build = |grade: i32| -> Vec<Vec<f64>> {
        let mut matrix = vec![vec![0.0f64; size]; size];
        for comp in info.compensation.iter().filter(|comp| comp.kind == grade) {
            let part = fif_comp_grade_matrix(info, comp, &universe, &pos);
            for i in 0..size {
                for j in 0..size {
                    matrix[i][j] += part[i][j];
                }
            }
        }
        matrix
    };
    let ident = |scale: f64, subtract: &[Vec<f64>]| -> Vec<Vec<f64>> {
        let mut matrix = vec![vec![0.0f64; size]; size];
        for i in 0..size {
            for j in 0..size {
                matrix[i][j] = (if i == j { scale } else { 0.0 }) - subtract[i][j];
            }
        }
        matrix
    };
    let zeros = vec![vec![0.0f64; size]; size];
    let c_from = if from_grade != 0 {
        build(from_grade)
    } else {
        zeros.clone()
    };
    let c_to = if target_grade != 0 {
        build(target_grade)
    } else {
        zeros
    };
    let right = if from_grade != 0 {
        invert_matrix(&ident(1.0, &c_from))?
    } else {
        ident(1.0, &vec![vec![0.0; size]; size])
    };
    let left = ident(1.0, &c_to);
    let mut comp = vec![vec![0.0f64; size]; size];
    for i in 0..size {
        for k in 0..size {
            if left[i][k] == 0.0 {
                continue;
            }
            for j in 0..size {
                comp[i][j] += left[i][k] * right[k][j];
            }
        }
    }
    let n_times = data.first().map(Vec::len).unwrap_or(0);
    let mut compensated = vec![vec![0.0f64; n_times]; size];
    for i in 0..size {
        for j in 0..size {
            let weight = comp[i][j];
            if weight == 0.0 {
                continue;
            }
            let source = &data[universe[j]];
            let target = &mut compensated[i];
            for t in 0..n_times {
                target[t] += weight * source[t];
            }
        }
    }
    for (i, &channel) in universe.iter().enumerate() {
        data[channel] = std::mem::take(&mut compensated[i]);
    }
    Ok(())
}

/// Apply active FIF SSP projectors to channel-major data. Each projector's
/// vectors are orthonormalized (modified Gram-Schmidt) over the named
/// channel subset, then `P = I - sum(v v^T)` is applied; multiple active
/// projectors compose multiplicatively, matching MNE's SSP operator.
fn apply_fif_projectors(info: &FifInfo, data: &mut [Vec<f64>]) -> Result<(), CoreError> {
    let name_to_index: std::collections::HashMap<&str, usize> = info
        .channel_names
        .iter()
        .enumerate()
        .map(|(index, name)| (name.as_str(), index))
        .collect();
    let n_times = data.first().map(Vec::len).unwrap_or(0);
    for projector in info.projectors.iter().filter(|p| p.active) {
        // Channel subset for this projector, in vector order.
        let subset: Vec<usize> = projector
            .ch_names
            .iter()
            .filter_map(|name| name_to_index.get(name.as_str()).copied())
            .collect();
        if subset.is_empty() {
            continue;
        }
        if subset.len() > 1024 || projector.vectors.len() > 256 {
            return Err(CoreError::InvalidInput(
                "projector dimensions exceed limits".into(),
            ));
        }
        // Restrict vectors to the subset and orthonormalize.
        let pos: std::collections::HashMap<usize, usize> = subset
            .iter()
            .enumerate()
            .map(|(position, &channel)| (channel, position))
            .collect();
        let mut basis: Vec<Vec<f64>> = Vec::new();
        for vector in &projector.vectors {
            let mut v = vec![0.0f64; subset.len()];
            for (name_index, name) in projector.ch_names.iter().enumerate() {
                let Some(&channel) = name_to_index.get(name.as_str()) else {
                    continue;
                };
                let Some(&slot) = pos.get(&channel) else {
                    continue;
                };
                if let Some(value) = vector.get(name_index) {
                    v[slot] = *value;
                }
            }
            // Modified Gram-Schmidt against existing basis vectors.
            for b in &basis {
                let dot: f64 = v.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
                for (x, y) in v.iter_mut().zip(b.iter()) {
                    *x -= dot * y;
                }
            }
            let norm = v.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm > 1e-12 {
                for x in &mut v {
                    *x /= norm;
                }
                basis.push(v);
            }
        }
        if basis.is_empty() {
            continue;
        }
        // out[c] = data[c] - sum_b <b, data_subset> * b
        let mut projected = vec![vec![0.0f64; n_times]; subset.len()];
        for (slot, &channel) in subset.iter().enumerate() {
            projected[slot].copy_from_slice(&data[channel]);
        }
        for b in &basis {
            // coefficient per time sample: <b, data>
            let mut coeffs = vec![0.0f64; n_times];
            for (slot, _) in subset.iter().enumerate() {
                for t in 0..n_times {
                    coeffs[t] += b[slot] * projected[slot][t];
                }
            }
            for (slot, _) in subset.iter().enumerate() {
                for t in 0..n_times {
                    projected[slot][t] -= b[slot] * coeffs[t];
                }
            }
        }
        for (slot, &channel) in subset.iter().enumerate() {
            data[channel] = std::mem::take(&mut projected[slot]);
        }
    }
    Ok(())
}

fn fif_window(
    path: &Path,
    start_s: f64,
    end_s: f64,
    channel_names: &[String],
    max_samples: usize,
    compensation: Option<i32>,
    apply_projectors: bool,
) -> Result<(Vec<f64>, Vec<Vec<f64>>, usize), CoreError> {
    let info = inspect_fif(path)?;
    let duration = info.n_samples as f64 / info.sampling_frequency_hz;
    if start_s < 0.0 || end_s <= start_s || end_s > duration + 1e-9 {
        return Err(CoreError::InvalidInput("invalid FIF window".into()));
    }
    let mut indices = Vec::new();
    for name in channel_names {
        let index = info
            .channel_names
            .iter()
            .position(|candidate| candidate == name)
            .ok_or_else(|| CoreError::InvalidInput(format!("unknown FIF channel: {name}")))?;
        if !indices.contains(&index) {
            indices.push(index);
        }
    }
    if indices.is_empty() {
        indices = (0..info.n_channels).collect();
    }
    // Compensation/projectors mix channels, so they need the full set.
    if compensation.is_some() && info.compensation.is_empty() {
        return Err(CoreError::InvalidInput(
            "no CTF compensation matrices in this FIF file".into(),
        ));
    }
    let need_all = (compensation.is_some() && !info.compensation.is_empty())
        || (apply_projectors && info.projectors.iter().any(|p| p.active));
    let read_indices: Vec<usize> = if need_all {
        (0..info.n_channels).collect()
    } else {
        indices.clone()
    };
    let bytes = std::fs::read(path)?;
    let buffers = data_buffer_tags(&bytes)?;
    let start = ((start_s * info.sampling_frequency_hz).round() as usize).min(info.n_samples);
    let end = ((end_s * info.sampling_frequency_hz).round() as usize)
        .min(info.n_samples)
        .max(start);
    let native = end.saturating_sub(start);
    let mut factor = 1usize;
    while native / factor > max_samples {
        factor *= 2;
    }
    let nchan = info.n_channels;
    let mut times = Vec::new();
    let mut data = vec![Vec::new(); read_indices.len()];
    let mut sample = start;
    while sample < end {
        times.push(sample as f64 / info.sampling_frequency_hz);
        for (output, &channel) in read_indices.iter().enumerate() {
            let flat = sample * nchan + channel;
            // Buffers are contiguous segments of the sample-major stream.
            let mut cumulative = 0usize;
            let mut value = f64::NAN;
            for tag in &buffers {
                let elem = if tag.ftype == FIFFT_FLOAT { 4 } else { 8 };
                let count = tag.nbytes / elem;
                if flat >= cumulative && flat < cumulative + count {
                    let offset = tag.data_start + (flat - cumulative) * elem;
                    value = if elem == 8 {
                        be_f64(&bytes, offset).unwrap_or(f64::NAN)
                    } else {
                        be_f32(&bytes, offset).unwrap_or(f32::NAN) as f64
                    };
                    break;
                }
                cumulative += count;
            }
            data[output].push(value);
        }
        sample += factor;
    }
    if need_all {
        // data rows are in channel order (read_indices == 0..n_channels).
        if let Some(target) = compensation {
            apply_fif_compensation(&info, &mut data, target)?;
        }
        if apply_projectors {
            apply_fif_projectors(&info, &mut data)?;
        }
        let position: std::collections::HashMap<usize, usize> = read_indices
            .iter()
            .enumerate()
            .map(|(row, &channel)| (channel, row))
            .collect();
        data = indices
            .iter()
            .map(|channel| data[position[channel]].clone())
            .collect();
    }
    Ok((times, data, info.n_channels))
}

pub fn read_window(
    path: &Path,
    start_s: f64,
    end_s: f64,
    channel_names: &[String],
    max_samples: usize,
    compensation: Option<i32>,
    apply_projectors: bool,
) -> Result<MegWindow, CoreError> {
    let (times_s, data, _) = fif_window(
        path,
        start_s,
        end_s,
        channel_names,
        max_samples,
        compensation,
        apply_projectors,
    )?;
    let info = inspect_fif(path)?;
    let factor = if times_s.len() > 1 {
        let span = times_s[times_s.len() - 1] - times_s[0];
        ((times_s.len() - 1) as f64 / span.max(1e-9)).round() as usize
    } else {
        1
    };
    Ok(MegWindow {
        effective_sampling_frequency_hz: info.sampling_frequency_hz / factor.max(1) as f64,
        times_s,
        data,
    })
}

fn fif_kind_name(kind: i32) -> &'static str {
    match kind {
        1 | 301 => "meg",
        2 => "eeg",
        3 => "stim",
        201 => "misc",
        202 => "eog",
        302 => "misc",
        402 => "ecg",
        502 => "misc",
        602 => "misc",
        802 | 803 | 902 => "eeg",
        900 | 910 | 920 | 1100 | 1200 | 1300 | 1400 => "misc",
        _ => "other",
    }
}

fn fif_unit_name(unit: i32) -> &'static str {
    match unit {
        107 => "volt",
        112 => "tesla",
        201 => "tesla_per_metre",
        210 => "ampere",
        _ => "other",
    }
}

pub fn metadata(path: &Path) -> Result<(MegMetadataLike, usize), CoreError> {
    let info = inspect_fif(path)?;
    let channels = info
        .channel_info
        .iter()
        .map(|channel| MegChannel {
            name: channel.name.clone(),
            kind: fif_kind_name(channel.kind).into(),
            unit: fif_unit_name(channel.unit).into(),
            unit_mul: channel.cal,
            group_name: None,
            status: if info.bad_channels.contains(&channel.name) {
                "bad".into()
            } else {
                "good".into()
            },
            coil_type: if channel.coil_type != 0 {
                Some(format!("fiff_coil_{}", channel.coil_type))
            } else {
                None
            },
            position_mm: Some(channel.loc.to_vec()),
            orientation: None,
        })
        .collect();
    let duration = info.n_samples as f64 / info.sampling_frequency_hz;
    Ok((
        MegMetadataLike {
            sampling_frequency_hz: info.sampling_frequency_hz,
            channel_count: info.n_channels,
            duration_seconds: duration,
            channels,
            description: info.description,
        },
        info.n_samples,
    ))
}

pub struct MegMetadataLike {
    pub sampling_frequency_hz: f64,
    pub channel_count: usize,
    pub duration_seconds: f64,
    pub channels: Vec<MegChannel>,
    pub description: Option<String>,
}

// ---------------------------------------------------------------------------
// FIF writer
// ---------------------------------------------------------------------------

/// Channel descriptor for [`write_fif`]. `loc` follows the FIFF layout
/// `[ex | ey | ez | r0]` (12 floats, metres); coil centre goes in `r0`.
#[derive(Clone, Debug)]
pub struct FifWriteChannel {
    pub name: String,
    pub kind: i32,
    pub unit: i32,
    pub unit_mul: i32,
    pub cal: f64,
    pub range: f64,
    pub coil_type: i32,
    pub loc: [f64; 12],
}

struct FifWriter {
    bytes: Vec<u8>,
}

impl FifWriter {
    fn tag(&mut self, kind: i32, ftype: u32, payload: &[u8]) {
        self.bytes.extend_from_slice(&kind.to_be_bytes());
        self.bytes.extend_from_slice(&ftype.to_be_bytes());
        self.bytes
            .extend_from_slice(&(payload.len() as i32).to_be_bytes());
        // FIFFV_NEXT_SEQ: payload is followed immediately by the next tag.
        self.bytes.extend_from_slice(&0i32.to_be_bytes());
        self.bytes.extend_from_slice(payload);
    }
    fn block_start(&mut self, id: i32) {
        self.tag(FIFF_BLOCK_START, FIFFT_INT, &id.to_be_bytes());
    }
    fn block_end(&mut self, id: i32) {
        self.tag(FIFF_BLOCK_END, FIFFT_INT, &id.to_be_bytes());
    }
    fn int_tag(&mut self, kind: i32, value: i32) {
        self.tag(kind, FIFFT_INT, &value.to_be_bytes());
    }
    fn double_tag(&mut self, kind: i32, value: f64) {
        self.tag(kind, FIFFT_DOUBLE, &value.to_be_bytes());
    }
    fn string_tag(&mut self, kind: i32, value: &str) {
        let mut payload = value.as_bytes().to_vec();
        payload.push(0);
        self.tag(kind, FIFFT_STRING, &payload);
    }
}

/// Write `data` (channel-major, physical units) as a FIFF file that
/// [`inspect_fif`] and [`fif_window`] can read back. Payload is stored as
/// big-endian float32, sample-major interleaved across channels.
pub fn write_fif(
    path: &Path,
    sfreq: f64,
    channels: &[FifWriteChannel],
    data: &[Vec<f64>],
) -> Result<(), CoreError> {
    if channels.is_empty() || data.len() != channels.len() {
        return Err(CoreError::InvalidInput(
            "FIF write requires matching channel and data counts".into(),
        ));
    }
    let n_samples = data.first().map_or(0, Vec::len);
    if data.iter().any(|row| row.len() != n_samples) {
        return Err(CoreError::InvalidInput(
            "FIF write requires equal-length channel data".into(),
        ));
    }
    if !(sfreq.is_finite() && sfreq > 0.0) {
        return Err(CoreError::InvalidInput("invalid FIF sampling rate".into()));
    }
    if n_samples == 0 {
        return Err(CoreError::InvalidInput("nothing to write".into()));
    }
    let mut w = FifWriter { bytes: Vec::new() };
    // Compulsory header (matches MNE `start_file`): file-id struct, dir
    // pointer (-1 = no directory; readers fall back to sequential scan),
    // free list (-1). Sequential `next` offsets keep the tag chain valid.
    let mut id_struct = Vec::with_capacity(20);
    id_struct.extend_from_slice(&1i32.to_be_bytes()); // version
    id_struct.extend_from_slice(&0i32.to_be_bytes()); // machid
    id_struct.extend_from_slice(&0i32.to_be_bytes());
    id_struct.extend_from_slice(&0i32.to_be_bytes()); // secs
    id_struct.extend_from_slice(&0i32.to_be_bytes()); // usecs
    w.tag(100, 31, &id_struct); // FIFF_FILE_ID / FIFFT_ID_STRUCT
    w.int_tag(101, -1); // FIFF_DIR_POINTER: no directory
    w.int_tag(106, -1); // FIFF_FREE_LIST
                        // FIFFB_MEAS wraps meas info and raw data blocks.
    w.block_start(100); // FIFFB_MEAS
                        // FIFFB_MEAS_INFO
    w.block_start(FIFFB_MEAS_INFO);
    w.int_tag(FIFF_NCHAN, channels.len() as i32);
    w.double_tag(FIFF_SFREQ, sfreq);
    w.string_tag(212, "neuro.lab FIF export");
    // One FIFF_CH_INFO tag per channel (real FIF convention).
    for channel in channels {
        let mut record = vec![0u8; CH_INFO_SIZE];
        record[8..12].copy_from_slice(&channel.kind.to_be_bytes());
        record[12..16].copy_from_slice(&(channel.range as f32).to_be_bytes());
        record[16..20].copy_from_slice(&(channel.cal as f32).to_be_bytes());
        record[20..24].copy_from_slice(&channel.coil_type.to_be_bytes());
        for (i, value) in channel.loc.iter().enumerate() {
            record[24 + i * 4..28 + i * 4].copy_from_slice(&(*value as f32).to_be_bytes());
        }
        record[72..76].copy_from_slice(&channel.unit.to_be_bytes());
        record[76..80].copy_from_slice(&channel.unit_mul.to_be_bytes());
        let name = channel.name.as_bytes();
        let take = name.len().min(15);
        record[80..80 + take].copy_from_slice(&name[..take]);
        w.tag(FIFF_CH_INFO, FIFFT_CH_INFO_STRUCT, &record);
    }
    w.block_end(FIFFB_MEAS_INFO);
    // FIFFB_RAW_DATA with one contiguous sample-major buffer.
    w.block_start(FIFFB_RAW_DATA);
    w.int_tag(FIFF_FIRST_SAMPLE, 0);
    let mut payload = Vec::with_capacity(channels.len() * n_samples * 4);
    for sample in 0..n_samples {
        for row in data {
            payload.extend_from_slice(&(row[sample] as f32).to_be_bytes());
        }
    }
    w.tag(FIFF_DATA_BUFFER, FIFFT_FLOAT, &payload);
    w.block_end(FIFFB_RAW_DATA);
    w.block_end(100); // FIFFB_MEAS
                      // Closing FIFF_NOP (kind 108, FIFFT_VOID) with NEXT_NONE terminator.
    w.bytes.extend_from_slice(&108i32.to_be_bytes());
    w.bytes.extend_from_slice(&0u32.to_be_bytes());
    w.bytes.extend_from_slice(&0i32.to_be_bytes());
    w.bytes.extend_from_slice(&(-1i32).to_be_bytes());
    std::fs::write(path, &w.bytes)?;
    Ok(())
}
