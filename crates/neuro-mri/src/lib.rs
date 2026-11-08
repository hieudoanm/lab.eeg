pub mod dicom;

pub mod operations;

use neuro_core::CoreError;
use nifti::{IntoNdArray, NiftiObject};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VolumeInfo {
    pub shape: [usize; 3],
    pub voxel_count: usize,
    pub spacing_mm: [f64; 3],
    pub datatype: String,
    pub affine: [[f64; 4]; 4],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SliceView {
    pub plane: String,
    pub index: usize,
    pub width: usize,
    pub height: usize,
    pub min_value: f64,
    pub max_value: f64,
    pub pixels: Vec<f64>,
}

pub fn inspect_nifti(path: &Path) -> Result<VolumeInfo, CoreError> {
    let object = nifti::ReaderOptions::new()
        .read_file(path)
        .map_err(|error| CoreError::InvalidInput(format!("invalid NIfTI volume: {error}")))?;
    let header = object.header();
    let shape = header.dim[1..4]
        .iter()
        .map(|value| *value as usize)
        .collect::<Vec<_>>();
    if shape.len() != 3 {
        return Err(CoreError::InvalidInput(
            "MRI volume must be three-dimensional".into(),
        ));
    }
    let dims = [shape[0], shape[1], shape[2]];
    let voxel_count = dims.iter().product::<usize>();
    if voxel_count == 0 {
        return Err(CoreError::InvalidInput("MRI volume is empty".into()));
    }
    let pixdim = header.pixdim;
    let affine = header.affine::<f64>();
    let mut affine_rows = [[0.0f64; 4]; 4];
    for row in 0..4 {
        for col in 0..4 {
            affine_rows[row][col] = affine[(row, col)];
        }
    }
    Ok(VolumeInfo {
        shape: dims,
        voxel_count,
        spacing_mm: [pixdim[1] as f64, pixdim[2] as f64, pixdim[3] as f64],
        datatype: format!("{:?}", header.datatype),
        affine: affine_rows,
    })
}

pub fn axial_slice(path: &Path, index: usize) -> Result<SliceView, CoreError> {
    let info = inspect_nifti(path)?;
    if index >= info.shape[2] {
        return Err(CoreError::InvalidInput(
            "slice index is outside the volume".into(),
        ));
    }
    let object = nifti::ReaderOptions::new()
        .read_file(path)
        .map_err(|error| CoreError::InvalidInput(format!("invalid NIfTI volume: {error}")))?;
    let volume = object
        .into_volume()
        .into_ndarray::<f32>()
        .map_err(|error| {
            CoreError::InvalidInput(format!("could not decode NIfTI volume: {error}"))
        })?;
    let mut pixels = Vec::with_capacity(info.shape[0] * info.shape[1]);
    let mut min_value = f64::INFINITY;
    let mut max_value = f64::NEG_INFINITY;
    for y in 0..info.shape[1] {
        for x in 0..info.shape[0] {
            let value = volume[[x, y, index]] as f64;
            pixels.push(value);
            min_value = min_value.min(value);
            max_value = max_value.max(value);
        }
    }
    Ok(SliceView {
        plane: "axial".into(),
        index,
        width: info.shape[0],
        height: info.shape[1],
        min_value,
        max_value,
        pixels,
    })
}

pub fn slice(path: &Path, plane: &str, index: usize) -> Result<SliceView, CoreError> {
    let info = inspect_nifti(path)?;
    let dimension = match plane {
        "axial" => 2,
        "coronal" => 1,
        "sagittal" => 0,
        _ => return Err(CoreError::InvalidInput("unsupported MRI plane".into())),
    };
    if index >= info.shape[dimension] {
        return Err(CoreError::InvalidInput(
            "slice index is outside the volume".into(),
        ));
    }
    let object = nifti::ReaderOptions::new()
        .read_file(path)
        .map_err(|error| CoreError::InvalidInput(format!("invalid NIfTI volume: {error}")))?;
    let volume = object
        .into_volume()
        .into_ndarray::<f32>()
        .map_err(|error| {
            CoreError::InvalidInput(format!("could not decode NIfTI volume: {error}"))
        })?;
    let (width, height) = match plane {
        "axial" => (info.shape[0], info.shape[1]),
        "coronal" => (info.shape[0], info.shape[2]),
        _ => (info.shape[1], info.shape[2]),
    };
    let mut pixels = Vec::with_capacity(width * height);
    let mut min_value = f64::INFINITY;
    let mut max_value = f64::NEG_INFINITY;
    for y in 0..height {
        for x in 0..width {
            let coords = match plane {
                "axial" => [x, y, index],
                "coronal" => [x, index, y],
                _ => [index, x, y],
            };
            let value = volume[coords] as f64;
            pixels.push(value);
            min_value = min_value.min(value);
            max_value = max_value.max(value);
        }
    }
    Ok(SliceView {
        plane: plane.to_string(),
        index,
        width,
        height,
        min_value,
        max_value,
        pixels,
    })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VolumeStatistics {
    pub voxel_count: usize,
    pub valid_sample_count: usize,
    pub min_value: f64,
    pub max_value: f64,
    pub mean: f64,
    pub std_dev: f64,
    pub non_finite_count: usize,
}

pub fn read_volume(path: &Path) -> Result<(ndarray::Array3<f32>, VolumeInfo), CoreError> {
    let info = inspect_nifti(path)?;
    let object = nifti::ReaderOptions::new()
        .read_file(path)
        .map_err(|error| CoreError::InvalidInput(format!("invalid NIfTI volume: {error}")))?;
    let volume = object
        .into_volume()
        .into_ndarray::<f32>()
        .map_err(|error| {
            CoreError::InvalidInput(format!("could not decode NIfTI volume: {error}"))
        })?;
    let array = volume
        .into_dimensionality::<ndarray::Ix3>()
        .map_err(|_| CoreError::InvalidInput("MRI volume must be 3D".into()))?;
    Ok((array, info))
}

pub fn volume_statistics(path: &Path) -> Result<VolumeStatistics, CoreError> {
    let (volume, info) = read_volume(path)?;
    let mut count = 0usize;
    let mut non_finite = 0usize;
    let mut min_value = f64::INFINITY;
    let mut max_value = f64::NEG_INFINITY;
    let mut sum = 0.0f64;
    let mut sum_sq = 0.0f64;
    for value in volume.iter() {
        let v = *value as f64;
        if !v.is_finite() {
            non_finite += 1;
            continue;
        }
        count += 1;
        min_value = min_value.min(v);
        max_value = max_value.max(v);
        sum += v;
        sum_sq += v * v;
    }
    if count == 0 {
        return Err(CoreError::InvalidInput(
            "MRI volume has no finite voxels".into(),
        ));
    }
    let mean = sum / count as f64;
    let variance = (sum_sq / count as f64 - mean * mean).max(0.0);
    Ok(VolumeStatistics {
        voxel_count: info.voxel_count,
        valid_sample_count: count,
        min_value,
        max_value,
        mean,
        std_dev: variance.sqrt(),
        non_finite_count: non_finite,
    })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IntensityHistogram {
    pub bins: Vec<f64>,
    pub counts: Vec<usize>,
}

pub fn intensity_histogram(path: &Path, bin_count: usize) -> Result<IntensityHistogram, CoreError> {
    if bin_count == 0 || bin_count > 1024 {
        return Err(CoreError::InvalidInput(
            "invalid histogram bin count".into(),
        ));
    }
    let (volume, _) = read_volume(path)?;
    let stats = volume_statistics(path)?;
    let mut counts = vec![0usize; bin_count];
    let width = (stats.max_value - stats.min_value) / bin_count as f64;
    for value in volume.iter() {
        let v = *value as f64;
        if !v.is_finite() {
            continue;
        }
        let idx = if width <= 0.0 {
            0
        } else {
            (((v - stats.min_value) / width).floor() as isize).clamp(0, bin_count as isize - 1)
                as usize
        };
        counts[idx] += 1;
    }
    let bins = (0..=bin_count)
        .map(|i| stats.min_value + width * i as f64)
        .collect();
    Ok(IntensityHistogram { bins, counts })
}

/// Writes a z-score normalized copy of the volume, preserving the source
/// spatial header (affine, spacing, datatype layout) where possible.
pub fn zscore_normalize(path: &Path, output: &Path) -> Result<VolumeInfo, CoreError> {
    let object = nifti::ReaderOptions::new()
        .read_file(path)
        .map_err(|error| CoreError::InvalidInput(format!("invalid NIfTI volume: {error}")))?;
    let header = object.header().clone();
    let stats = volume_statistics(path)?;
    if stats.std_dev <= 0.0 {
        return Err(CoreError::InvalidInput(
            "cannot normalize a constant volume".into(),
        ));
    }
    let (volume, _) = read_volume(path)?;
    let normalized = volume.mapv(|value| {
        let v = value as f64;
        if v.is_finite() {
            ((v - stats.mean) / stats.std_dev) as f32
        } else {
            f32::NAN
        }
    });
    nifti::writer::WriterOptions::new(output)
        .reference_header(&header)
        .write_nifti(&normalized)
        .map_err(|error| {
            CoreError::InvalidInput(format!("could not write NIfTI volume: {error}"))
        })?;
    inspect_nifti(output)
}

/// Maximum-intensity projection along one axis, returned as a 2D slice view.
pub fn max_intensity_projection(path: &Path, axis: &str) -> Result<SliceView, CoreError> {
    let info = inspect_nifti(path)?;
    let axis_index = match axis {
        "x" | "sagittal" => 0,
        "y" | "coronal" => 1,
        "z" | "axial" => 2,
        _ => {
            return Err(CoreError::InvalidInput(
                "unsupported projection axis".into(),
            ))
        }
    };
    let (volume, _) = read_volume(path)?;
    let (w, h) = match axis_index {
        0 => (info.shape[1], info.shape[2]),
        1 => (info.shape[0], info.shape[2]),
        _ => (info.shape[0], info.shape[1]),
    };
    let mut pixels = vec![f64::NEG_INFINITY; w * h];
    let mut min_value = f64::INFINITY;
    let mut max_value = f64::NEG_INFINITY;
    for z in 0..info.shape[2] {
        for y in 0..info.shape[1] {
            for x in 0..info.shape[0] {
                let value = volume[[x, y, z]] as f64;
                if !value.is_finite() {
                    continue;
                }
                min_value = min_value.min(value);
                max_value = max_value.max(value);
                let (u, v) = match axis_index {
                    0 => (y, z),
                    1 => (x, z),
                    _ => (x, y),
                };
                let idx = v * w + u;
                if value > pixels[idx] {
                    pixels[idx] = value;
                }
            }
        }
    }
    if min_value == f64::INFINITY {
        return Err(CoreError::InvalidInput(
            "MRI volume has no finite voxels".into(),
        ));
    }
    Ok(SliceView {
        plane: format!("mip_{axis}"),
        index: 0,
        width: w,
        height: h,
        min_value,
        max_value,
        pixels,
    })
}

pub fn validate_limits(
    info: &VolumeInfo,
    max_voxels: usize,
    max_spacing_mm: f64,
) -> Result<(), CoreError> {
    if info.voxel_count > max_voxels {
        return Err(CoreError::InvalidInput(
            "MRI volume exceeds voxel limit".into(),
        ));
    }
    if info
        .spacing_mm
        .iter()
        .any(|value| *value <= 0.0 || *value > max_spacing_mm)
    {
        return Err(CoreError::InvalidInput(
            "MRI spacing is outside supported bounds".into(),
        ));
    }
    Ok(())
}
