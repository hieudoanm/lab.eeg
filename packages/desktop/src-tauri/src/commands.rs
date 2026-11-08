use neuro_core::{file_sha256_hex, validate_name, Dataset, Project, ProjectKind, ResourceId};
use neuro_eeg::{
    analyze_recording, channel_statistics, describe_npy, edf, extract_signal_window,
    load_recording, load_recording_with_events, EegAnalysisResult,
};
use neuro_meg::{
    inspect_recording as inspect_meg, read_window as read_meg_window, spectral::welch_band_powers,
};
use neuro_mri::{
    inspect_nifti, intensity_histogram, max_intensity_projection, slice as mri_slice,
    validate_limits, volume_statistics, zscore_normalize,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct InspectDatasetRequest {
    pub path: String,
    pub kind: ProjectKind,
    #[serde(default)]
    pub sampling_frequency_hz: Option<f64>,
    #[serde(default)]
    pub project_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InspectDatasetResponse {
    pub dataset: Dataset,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct EegAnalysisRequest {
    pub dataset_id: String,
    pub path: String,
    pub sampling_frequency_hz: f64,
    #[serde(default)]
    pub experiment_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SignalWindowRequest {
    pub dataset_id: String,
    pub path: String,
    pub start_seconds: f64,
    pub end_seconds: f64,
    pub channel_indices: Vec<usize>,
    pub sampling_frequency_hz: f64,
}

#[derive(Debug, Deserialize)]
pub struct MegWindowRequest {
    pub dataset_id: String,
    pub path: String,
    pub start_s: f64,
    pub end_s: f64,
    pub channel_names: Vec<String>,
    pub max_samples: usize,
    #[serde(default)]
    pub recording: Option<String>,
    #[serde(default)]
    pub compensation_grade: Option<i32>,
    #[serde(default)]
    pub apply_projectors: bool,
}

#[derive(Debug, Deserialize)]
pub struct MriSliceRequest {
    pub dataset_id: String,
    pub path: String,
    pub index: usize,
    #[serde(default = "default_plane")]
    pub plane: String,
}

fn default_plane() -> String {
    "axial".to_string()
}

fn write_run_manifest(
    workspace: &neuro_store::LocalWorkspace,
    run_id: &ResourceId,
    dataset_id: &ResourceId,
    pipeline: &str,
    parameters: &serde_json::Value,
    outputs: &[String],
) -> Result<neuro_core::Artifact, String> {
    let dataset = workspace
        .get_dataset(dataset_id.clone())
        .map_err(|error| error.to_string())?;
    let manifest = serde_json::json!({
        "schema_version": "1.0",
        "run_id": run_id.0,
        "pipeline": { "name": pipeline, "version": env!("CARGO_PKG_VERSION") },
        "inputs": dataset.map(|d| serde_json::json!([{
            "dataset_id": d.id.0,
            "name": d.name,
            "checksum_sha256": d.checksum_sha256,
            "size_bytes": d.size_bytes,
        }])),
        "parameters": parameters,
        "environment": {
            "app": "neuro.lab",
            "runtime": "tauri-rust",
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
        },
        "outputs": outputs,
        "status": "succeeded",
    });
    let payload = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;
    workspace
        .add_artifact(
            run_id.clone(),
            "run_manifest.json".to_string(),
            "application/json".to_string(),
            &payload,
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn inspect_dataset(request: InspectDatasetRequest) -> Result<InspectDatasetResponse, String> {
    let path = PathBuf::from(&request.path);
    let metadata = match request.kind {
        ProjectKind::Eeg => {
            if path
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("edf"))
            {
                serde_json::to_value(edf::inspect_edf(&path).map_err(|error| error.to_string())?)
                    .map_err(|error| error.to_string())?
            } else if path
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|ext| {
                    ["vhdr", "eeg", "vmrk"]
                        .iter()
                        .any(|candidate| ext.eq_ignore_ascii_case(candidate))
                })
                || (!path
                    .extension()
                    .and_then(|value| value.to_str())
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("npy"))
                    && neuro_eeg::brainvision::vhdr_path(&path).exists())
            {
                serde_json::to_value(
                    neuro_eeg::brainvision::inspect_brainvision(&path)
                        .map_err(|error| error.to_string())?,
                )
                .map_err(|error| error.to_string())?
            } else {
                let sampling_frequency_hz = request
                    .sampling_frequency_hz
                    .ok_or_else(|| "sampling_frequency_hz is required for EEG".to_string())?;
                serde_json::to_value(
                    describe_npy(&path, 512, 10_000_000, sampling_frequency_hz)
                        .map_err(|error| error.to_string())?,
                )
                .map_err(|error| error.to_string())?
            }
        }
        ProjectKind::Meg => match inspect_meg(&path) {
            Ok(metadata) => serde_json::to_value(metadata).map_err(|error| error.to_string())?,
            Err(error) => {
                let names: Vec<String> = std::fs::read_dir(&path)
                    .map(|entries| {
                        entries
                            .filter_map(Result::ok)
                            .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
                            .collect()
                    })
                    .unwrap_or_default();
                if let Some(format) = neuro_meg::formats::detect_format(&names) {
                    return Err(format!(
                            "detected {format} recording; decoding for this format is not yet ported ({error})"
                        ));
                }
                return Err(error.to_string());
            }
        },
        ProjectKind::Mri => {
            let info = inspect_nifti(&path).map_err(|error| error.to_string())?;
            validate_limits(&info, 1_000_000_000, 100.0).map_err(|error| error.to_string())?;
            serde_json::to_value(info).map_err(|error| error.to_string())?
        }
    };
    let bytes = if path.is_file() {
        std::fs::metadata(&path)
            .map_err(|error| error.to_string())?
            .len()
    } else {
        std::fs::read_dir(&path)
            .map_err(|error| error.to_string())?
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_file())
            .try_fold(0_u64, |total, entry| {
                entry
                    .metadata()
                    .map(|metadata| total + metadata.len())
                    .map_err(|error| error.to_string())
            })?
    };
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("dataset")
        .to_string();
    validate_name(&name).map_err(|error| error.to_string())?;
    let project_id = request
        .project_id
        .as_deref()
        .map(|value| {
            uuid::Uuid::parse_str(value)
                .map(ResourceId)
                .map_err(|error| error.to_string())
        })
        .transpose()?
        .unwrap_or_else(ResourceId::new);
    Ok(InspectDatasetResponse {
        dataset: Dataset {
            id: ResourceId::new(),
            project_id,
            name,
            path: path.to_string_lossy().into_owned(),
            format: format!("{:?}", request.kind).to_lowercase(),
            checksum_sha256: if path.is_file() {
                file_sha256_hex(&path).map_err(|error| error.to_string())?
            } else {
                neuro_core::sha256_hex(path.to_string_lossy().as_bytes())
            },
            size_bytes: bytes,
            version: 1,
            created_at: chrono::Utc::now(),
            updated_at: None,
        },
        metadata,
    })
}

#[tauri::command]
pub fn run_eeg_analysis(
    state: tauri::State<'_, crate::workspace::WorkspaceState>,
    request: EegAnalysisRequest,
) -> Result<serde_json::Value, String> {
    let parameters_json = serde_json::to_string(&serde_json::json!({
        "sampling_frequency_hz": request.sampling_frequency_hz,
    }))
    .map_err(|error| error.to_string())?;
    let (workspace, run, dataset_id) = begin_run(
        &state,
        &request.dataset_id,
        "eeg_descriptive",
        parameters_json,
        request.experiment_id.clone(),
    )?;
    execute_eeg_analysis(&workspace, &request, &run, &dataset_id)
}

#[tauri::command]
pub fn submit_eeg_analysis(
    state: tauri::State<'_, crate::workspace::WorkspaceState>,
    request: EegAnalysisRequest,
) -> Result<serde_json::Value, String> {
    let parameters_json = serde_json::to_string(&serde_json::json!({
        "sampling_frequency_hz": request.sampling_frequency_hz,
    }))
    .map_err(|error| error.to_string())?;
    let (workspace, run, dataset_id) = begin_run(
        &state,
        &request.dataset_id,
        "eeg_descriptive",
        parameters_json,
        request.experiment_id.clone(),
    )?;
    let run_id = run.id.0.to_string();
    std::thread::spawn(move || {
        let _ = execute_eeg_analysis(&workspace, &request, &run, &dataset_id);
    });
    Ok(serde_json::json!({ "run_id": run_id, "status": "running" }))
}

fn execute_eeg_analysis(
    workspace: &neuro_store::LocalWorkspace,
    request: &EegAnalysisRequest,
    run: &neuro_core::AnalysisRun,
    dataset_id: &neuro_core::ResourceId,
) -> Result<serde_json::Value, String> {
    let _ = workspace.update_run_progress(
        run.id.clone(),
        Some(0.1),
        Some("loading recording".to_string()),
    );
    let path = PathBuf::from(&request.path);
    let result = if path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("edf"))
    {
        load_recording(&path, request.sampling_frequency_hz).and_then(
            |(matrix, names, units, sfreq)| {
                let stats = channel_statistics(&matrix, &names, Some(&units))?;
                Ok(EegAnalysisResult {
                    dataset_id: request.dataset_id.clone(),
                    algorithm_version: "neuro-eeg-descriptive-v1".into(),
                    parameters: serde_json::json!({
                        "format": "edf",
                        "sampling_frequency_hz": sfreq,
                    }),
                    channels: stats,
                })
            },
        )
    } else {
        analyze_recording(&path, &request.dataset_id, request.sampling_frequency_hz)
    }
    .map_err(|error| error.to_string());
    let _ = workspace.update_run_progress(
        run.id.clone(),
        Some(0.9),
        Some("writing result".to_string()),
    );
    match result {
        Ok(analysis) => {
            let value = serde_json::to_value(&analysis).map_err(|error| error.to_string())?;
            let result_json = serde_json::to_string(&value).map_err(|e| e.to_string())?;
            let completed = workspace
                .complete_run(run.id.clone(), result_json)
                .map_err(|error| error.to_string())?;
            let payload = serde_json::to_vec_pretty(&value).map_err(|e| e.to_string())?;
            let artifact = workspace
                .add_artifact(
                    run.id.clone(),
                    "eeg_analysis".to_string(),
                    "application/json".to_string(),
                    &payload,
                )
                .map_err(|error| error.to_string())?;
            let _manifest = write_run_manifest(
                workspace,
                &run.id,
                dataset_id,
                "eeg_descriptive",
                &serde_json::json!({ "sampling_frequency_hz": request.sampling_frequency_hz }),
                &[artifact.name.clone()],
            )?;
            Ok(serde_json::json!({
                "run": completed,
                "artifact": artifact,
                "result": value,
            }))
        }
        Err(error) => {
            if error != "cancelled" {
                let _ = workspace.fail_run(run.id.clone(), error.clone());
            }
            Err(error)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct EegPipelineRequest {
    pub dataset_id: String,
    pub path: String,
    pub pipeline: serde_json::Value,
    #[serde(default)]
    pub experiment_id: Option<String>,
}

#[tauri::command]
pub fn run_eeg_pipeline(
    state: tauri::State<'_, crate::workspace::WorkspaceState>,
    request: EegPipelineRequest,
) -> Result<serde_json::Value, String> {
    let parameters_json =
        serde_json::to_string(&request.pipeline).map_err(|error| error.to_string())?;
    let (workspace, run, dataset_id) = begin_run(
        &state,
        &request.dataset_id,
        "eeg_pipeline",
        parameters_json,
        request.experiment_id.clone(),
    )?;
    execute_eeg_pipeline(&workspace, &request, &run, &dataset_id)
}

#[tauri::command]
pub fn submit_eeg_pipeline(
    state: tauri::State<'_, crate::workspace::WorkspaceState>,
    request: EegPipelineRequest,
) -> Result<serde_json::Value, String> {
    let parameters_json =
        serde_json::to_string(&request.pipeline).map_err(|error| error.to_string())?;
    let (workspace, run, dataset_id) = begin_run(
        &state,
        &request.dataset_id,
        "eeg_pipeline",
        parameters_json,
        request.experiment_id.clone(),
    )?;
    let run_id = run.id.0.to_string();
    std::thread::spawn(move || {
        let _ = execute_eeg_pipeline(&workspace, &request, &run, &dataset_id);
    });
    Ok(serde_json::json!({ "run_id": run_id, "status": "running" }))
}

fn execute_eeg_pipeline(
    workspace: &neuro_store::LocalWorkspace,
    request: &EegPipelineRequest,
    run: &neuro_core::AnalysisRun,
    dataset_id: &neuro_core::ResourceId,
) -> Result<serde_json::Value, String> {
    let _ = workspace.update_run_progress(
        run.id.clone(),
        Some(0.05),
        Some("loading recording".to_string()),
    );
    let result = (|| -> Result<serde_json::Value, String> {
        let path = PathBuf::from(&request.path);
        let (matrix, names, units, sfreq, events) =
            load_recording_with_events(&path, 250.0).map_err(|error| error.to_string())?;
        let _ = units;
        let run_id = run.id.clone();
        let cancel = || run_is_cancelled(workspace, &run_id);
        let progress = |fraction: f64, message: &str| {
            let _ = workspace.update_run_progress(
                run_id.clone(),
                Some(0.1 + 0.8 * fraction),
                Some(message.to_string()),
            );
        };
        let result = neuro_eeg::pipeline::run_pipeline(
            &request.pipeline,
            &matrix,
            sfreq,
            &names,
            &events,
            Some(&cancel),
            Some(&progress),
        )
        .map_err(|error| error.to_string())?;
        serde_json::to_value(&result).map_err(|error| error.to_string())
    })();
    let _ = workspace.update_run_progress(
        run.id.clone(),
        Some(0.95),
        Some("writing result".to_string()),
    );
    match result {
        Ok(value) => {
            let result_json = serde_json::to_string(&value).map_err(|e| e.to_string())?;
            let completed = workspace
                .complete_run(run.id.clone(), result_json)
                .map_err(|error| error.to_string())?;
            let payload = serde_json::to_vec_pretty(&value).map_err(|e| e.to_string())?;
            let artifact = workspace
                .add_artifact(
                    run.id.clone(),
                    "pipeline_result".to_string(),
                    "application/json".to_string(),
                    &payload,
                )
                .map_err(|error| error.to_string())?;
            let _manifest = write_run_manifest(
                workspace,
                &run.id,
                dataset_id,
                "eeg_pipeline",
                &request.pipeline,
                &[artifact.name.clone()],
            )?;
            Ok(serde_json::json!({
                "run": completed,
                "artifact": artifact,
                "result": value,
            }))
        }
        Err(error) => {
            if error != "cancelled" {
                let _ = workspace.fail_run(run.id.clone(), error.clone());
            }
            Err(error)
        }
    }
}

#[tauri::command]
pub fn get_eeg_signal_window(request: SignalWindowRequest) -> Result<serde_json::Value, String> {
    let _dataset_id = &request.dataset_id;
    let path = PathBuf::from(&request.path);
    let is_edf = path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("edf"));
    let (channel_names, channel_units, sfreq) = if is_edf {
        let info = edf::inspect_edf(&path).map_err(|error| error.to_string())?;
        let units = if info.units.iter().all(|unit| unit.is_empty()) {
            vec!["uV".to_string(); info.channel_names.len()]
        } else {
            info.units
        };
        (info.channel_names, units, info.sampling_frequency_hz)
    } else {
        let metadata = describe_npy(&path, 512, 10_000_000, request.sampling_frequency_hz)
            .map_err(|error| error.to_string())?;
        (
            metadata.channel_names,
            metadata.channel_units,
            metadata.sampling_frequency_hz,
        )
    };
    let window = extract_signal_window(
        &path,
        request.start_seconds,
        request.end_seconds,
        &request.channel_indices,
        &channel_names,
        Some(channel_units.as_slice()),
        sfreq,
        2_000,
        300.0,
    )
    .map_err(|error| error.to_string())?;
    serde_json::to_value(window).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn inspect_meg_recording(path: String) -> Result<serde_json::Value, String> {
    let metadata = inspect_meg(&PathBuf::from(path)).map_err(|error| error.to_string())?;
    serde_json::to_value(metadata).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_meg_window(request: MegWindowRequest) -> Result<serde_json::Value, String> {
    let _dataset_id = &request.dataset_id;
    let window = read_meg_window(
        &PathBuf::from(&request.path),
        request.start_s,
        request.end_s,
        &request.channel_names,
        request.max_samples,
        request.recording.as_deref(),
        request.compensation_grade,
        request.apply_projectors,
    )
    .map_err(|error| error.to_string())?;
    let mut bands = BTreeMap::new();
    bands.insert("delta".to_string(), (0.5, 4.0));
    bands.insert("theta".to_string(), (4.0, 8.0));
    bands.insert("alpha".to_string(), (8.0, 13.0));
    bands.insert("beta".to_string(), (13.0, 30.0));
    bands.insert("gamma".to_string(), (30.0, 80.0));
    let powers = welch_band_powers(&window, &bands, 256);
    let summary: BTreeMap<String, f64> = powers
        .into_iter()
        .map(|power| (power.band, power.power))
        .collect();
    Ok(serde_json::json!({ "window": window, "band_power": summary }))
}

#[tauri::command]
pub fn inspect_mri_volume(path: String) -> Result<serde_json::Value, String> {
    let info = inspect_nifti(&PathBuf::from(path)).map_err(|error| error.to_string())?;
    validate_limits(&info, 1_000_000_000, 100.0).map_err(|error| error.to_string())?;
    serde_json::to_value(info).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_mri_slice(request: MriSliceRequest) -> Result<serde_json::Value, String> {
    let _dataset_id = &request.dataset_id;
    let slice = mri_slice(&PathBuf::from(&request.path), &request.plane, request.index)
        .map_err(|error| error.to_string())?;
    serde_json::to_value(slice).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_mri_projection(path: String, axis: String) -> Result<serde_json::Value, String> {
    let projection =
        max_intensity_projection(&PathBuf::from(path), &axis).map_err(|error| error.to_string())?;
    serde_json::to_value(projection).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn normalize_mri_volume(path: String) -> Result<serde_json::Value, String> {
    let input = PathBuf::from(&path);
    let file_name = input
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("volume");
    let output = input.with_file_name(format!("{file_name}.zscore.nii.gz"));
    let info = zscore_normalize(&input, &output).map_err(|error| error.to_string())?;
    Ok(serde_json::json!({
        "output": output.to_string_lossy(),
        "info": info,
    }))
}

#[derive(Debug, Deserialize)]
pub struct MriPipelineStep {
    pub operation: String,
    #[serde(default)]
    pub params: serde_json::Value,
    #[serde(default)]
    pub mask_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MriOperationRequest {
    pub dataset_id: String,
    pub path: String,
    pub operation: String,
    #[serde(default)]
    pub params: serde_json::Value,
    #[serde(default)]
    pub mask_path: Option<String>,
    #[serde(default)]
    pub experiment_id: Option<String>,
    /// Optional multi-step chain; when present it takes precedence over
    /// `operation`/`params` and each step's output feeds the next.
    #[serde(default)]
    pub pipeline: Option<Vec<MriPipelineStep>>,
}

fn parse_optional_resource_id(value: &Option<String>) -> Result<Option<ResourceId>, String> {
    value
        .as_deref()
        .map(|raw| {
            uuid::Uuid::parse_str(raw)
                .map(ResourceId)
                .map_err(|error| error.to_string())
        })
        .transpose()
}

#[tauri::command]
pub fn export_eeg_edf(request: EegAnalysisRequest) -> Result<serde_json::Value, String> {
    let path = PathBuf::from(&request.path);
    let (matrix, names, units, sfreq) =
        load_recording(&path, request.sampling_frequency_hz).map_err(|error| error.to_string())?;
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("recording");
    let output = path.with_file_name(format!("{stem}.export.edf"));
    neuro_eeg::edf::write_edf(&output, &matrix, &names, &units, sfreq)
        .map_err(|error| error.to_string())?;
    Ok(serde_json::json!({ "output_path": output.to_string_lossy() }))
}

#[derive(Debug, Deserialize)]
pub struct MegExportRequest {
    pub path: String,
    #[serde(default)]
    pub start_s: f64,
    #[serde(default = "default_export_end")]
    pub end_s: f64,
    #[serde(default)]
    pub channel_names: Vec<String>,
    #[serde(default)]
    pub output_path: Option<String>,
    #[serde(default)]
    pub recording: Option<String>,
    #[serde(default)]
    pub compensation_grade: Option<i32>,
    #[serde(default)]
    pub apply_projectors: bool,
}

fn default_export_end() -> f64 {
    f64::MAX
}

#[tauri::command]
pub fn export_meg_npy(request: MegExportRequest) -> Result<serde_json::Value, String> {
    let path = PathBuf::from(&request.path);
    let metadata = neuro_meg::inspect_recording(&path).map_err(|error| error.to_string())?;
    let end_s = if request.end_s == f64::MAX {
        metadata.duration_seconds
    } else {
        request.end_s
    };
    let output = request
        .output_path
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| neuro_meg::export::default_output_path(&path));
    neuro_meg::export::export_window(
        &path,
        request.start_s,
        end_s,
        &request.channel_names,
        &output,
        request.recording.as_deref(),
        request.compensation_grade,
        request.apply_projectors,
    )
    .map_err(|error| error.to_string())?;
    Ok(serde_json::json!({ "output_path": output.to_string_lossy() }))
}

#[tauri::command]
pub fn export_meg_csv(request: MegExportRequest) -> Result<serde_json::Value, String> {
    let path = PathBuf::from(&request.path);
    let metadata = neuro_meg::inspect_recording(&path).map_err(|error| error.to_string())?;
    let end_s = if request.end_s == f64::MAX {
        metadata.duration_seconds
    } else {
        request.end_s
    };
    // CSV needs column names: fall back to the recording's channel list
    // (same order the readers emit when no selection is given).
    let channel_names = if request.channel_names.is_empty() {
        metadata
            .channels
            .iter()
            .map(|channel| channel.name.clone())
            .collect()
    } else {
        request.channel_names.clone()
    };
    let output = request
        .output_path
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| path.with_extension("meg.csv"));
    neuro_meg::export::export_window_csv(
        &path,
        request.start_s,
        end_s,
        &channel_names,
        &output,
        request.recording.as_deref(),
        request.compensation_grade,
        request.apply_projectors,
    )
    .map_err(|error| error.to_string())?;
    Ok(serde_json::json!({ "output_path": output.to_string_lossy() }))
}

#[tauri::command]
pub fn export_meg_edf(request: MegExportRequest) -> Result<serde_json::Value, String> {
    let path = PathBuf::from(&request.path);
    let metadata = neuro_meg::inspect_recording(&path).map_err(|error| error.to_string())?;
    let end_s = if request.end_s == f64::MAX {
        metadata.duration_seconds
    } else {
        request.end_s
    };
    let channel_names: Vec<String> = if request.channel_names.is_empty() {
        metadata
            .channels
            .iter()
            .map(|channel| channel.name.clone())
            .collect()
    } else {
        request.channel_names.clone()
    };
    let units: Vec<String> = metadata
        .channels
        .iter()
        .map(|channel| {
            if channel.unit.is_empty() {
                "T".to_string()
            } else {
                channel.unit.clone()
            }
        })
        .collect();
    let output = request
        .output_path
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| path.with_extension("meg.edf"));
    let window = neuro_meg::read_window(
        &path,
        request.start_s,
        end_s,
        &channel_names,
        usize::MAX / 4,
        request.recording.as_deref(),
        request.compensation_grade,
        request.apply_projectors,
    )
    .map_err(|error| error.to_string())?;
    neuro_eeg::edf::write_edf(
        &output,
        &window.data,
        &channel_names,
        &units,
        window.effective_sampling_frequency_hz,
    )
    .map_err(|error| error.to_string())?;
    Ok(serde_json::json!({ "output_path": output.to_string_lossy() }))
}

#[tauri::command]
pub fn export_meg_fif(request: MegExportRequest) -> Result<serde_json::Value, String> {
    let path = PathBuf::from(&request.path);
    let metadata = neuro_meg::inspect_recording(&path).map_err(|error| error.to_string())?;
    let end_s = if request.end_s == f64::MAX {
        metadata.duration_seconds
    } else {
        request.end_s
    };
    let channel_names: Vec<String> = if request.channel_names.is_empty() {
        metadata
            .channels
            .iter()
            .map(|channel| channel.name.clone())
            .collect()
    } else {
        request.channel_names.clone()
    };
    let output = request
        .output_path
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| path.with_extension("meg.fif"));
    neuro_meg::export::export_window_fif(
        &path,
        request.start_s,
        end_s,
        &channel_names,
        &output,
        request.recording.as_deref(),
        request.compensation_grade,
        request.apply_projectors,
    )
    .map_err(|error| error.to_string())?;
    Ok(serde_json::json!({ "output_path": output.to_string_lossy() }))
}

#[derive(Debug, Deserialize)]
pub struct PipelinePresetRequest {
    pub name: String,
    /// "mri", "eeg", or "meg".
    pub modality: String,
    pub steps: serde_json::Value,
}

#[tauri::command]
pub fn save_pipeline_preset(
    state: tauri::State<'_, crate::workspace::WorkspaceState>,
    request: PipelinePresetRequest,
) -> Result<serde_json::Value, String> {
    let workspace = state
        .0
        .lock()
        .map_err(|_| "workspace lock poisoned".to_string())?;
    let preset = workspace
        .save_pipeline_preset(request.name, request.modality, request.steps)
        .map_err(|error| error.to_string())?;
    Ok(serde_json::to_value(preset).map_err(|error| error.to_string())?)
}

#[tauri::command]
pub fn list_pipeline_presets(
    state: tauri::State<'_, crate::workspace::WorkspaceState>,
) -> Result<serde_json::Value, String> {
    let workspace = state
        .0
        .lock()
        .map_err(|_| "workspace lock poisoned".to_string())?;
    let presets = workspace
        .list_pipeline_presets()
        .map_err(|error| error.to_string())?;
    Ok(serde_json::to_value(presets).map_err(|error| error.to_string())?)
}

#[tauri::command]
pub fn delete_pipeline_preset(
    state: tauri::State<'_, crate::workspace::WorkspaceState>,
    preset_id: String,
) -> Result<(), String> {
    let preset_id = neuro_core::ResourceId(
        uuid::Uuid::parse_str(&preset_id).map_err(|error| error.to_string())?,
    );
    let workspace = state
        .0
        .lock()
        .map_err(|_| "workspace lock poisoned".to_string())?;
    workspace
        .delete_pipeline_preset(preset_id)
        .map_err(|error| error.to_string())
}

#[derive(Debug, Deserialize)]
pub struct MegBidsExportRequest {
    pub path: String,
    #[serde(default)]
    pub start_s: f64,
    #[serde(default = "default_export_end")]
    pub end_s: f64,
    #[serde(default)]
    pub channel_names: Vec<String>,
    /// Output directory for the BIDS dataset files.
    pub output_dir: String,
    #[serde(default)]
    pub dataset_name: Option<String>,
    #[serde(default)]
    pub recording: Option<String>,
    #[serde(default)]
    pub compensation_grade: Option<i32>,
    #[serde(default)]
    pub apply_projectors: bool,
}

#[tauri::command]
pub fn export_meg_bids(request: MegBidsExportRequest) -> Result<serde_json::Value, String> {
    let path = PathBuf::from(&request.path);
    let metadata = neuro_meg::inspect_recording(&path).map_err(|error| error.to_string())?;
    let end_s = if request.end_s == f64::MAX {
        metadata.duration_seconds
    } else {
        request.end_s
    };
    let dataset_name = request.dataset_name.unwrap_or_else(|| {
        path.file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("meg")
            .to_string()
    });
    let output_dir = PathBuf::from(&request.output_dir);
    neuro_meg::export::export_window_bids(
        &path,
        request.start_s,
        end_s,
        &request.channel_names,
        &output_dir,
        &dataset_name,
        request.recording.as_deref(),
        request.compensation_grade,
        request.apply_projectors,
    )
    .map_err(|error| error.to_string())?;
    Ok(serde_json::json!({ "output_dir": output_dir.to_string_lossy() }))
}

#[derive(Debug, Deserialize)]
pub struct DicomConvertRequest {
    pub path: String,
    #[serde(default)]
    pub output_path: Option<String>,
}

#[tauri::command]
pub fn convert_dicom_to_nifti(request: DicomConvertRequest) -> Result<serde_json::Value, String> {
    let input = PathBuf::from(&request.path);
    let output = request
        .output_path
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| input.with_extension("nii.gz"));
    let result =
        neuro_mri::dicom::dicom_to_nifti(&input, &output).map_err(|error| error.to_string())?;
    Ok(serde_json::to_value(result).map_err(|error| error.to_string())?)
}

#[tauri::command]
pub fn convert_dicom_series_to_nifti(
    request: DicomConvertRequest,
) -> Result<serde_json::Value, String> {
    let input = PathBuf::from(&request.path);
    let output = request
        .output_path
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| input.join("series.nii.gz"));
    let result = neuro_mri::dicom::dicom_series_to_nifti(&input, &output)
        .map_err(|error| error.to_string())?;
    Ok(serde_json::to_value(result).map_err(|error| error.to_string())?)
}

#[tauri::command]
pub fn validate_eeg_file_set(path: String) -> Result<serde_json::Value, String> {
    let (format, findings) = neuro_eeg::validate_recording(&PathBuf::from(path));
    Ok(serde_json::json!({ "format": format, "findings": findings }))
}

#[tauri::command]
pub fn export_run_artifact(
    state: tauri::State<'_, crate::workspace::WorkspaceState>,
    run_id: String,
    artifact_name: String,
) -> Result<serde_json::Value, String> {
    let run_id =
        neuro_core::ResourceId(uuid::Uuid::parse_str(&run_id).map_err(|error| error.to_string())?);
    let workspace = state
        .0
        .lock()
        .map_err(|_| "workspace lock poisoned".to_string())?;
    let artifacts = workspace
        .list_artifacts(run_id)
        .map_err(|error| error.to_string())?;
    let artifact = artifacts
        .iter()
        .find(|artifact| artifact.name == artifact_name)
        .ok_or_else(|| format!("artifact not found: {artifact_name}"))?;
    let source = PathBuf::from(&artifact.path);
    let exports_dir = workspace.root_dir().join("exports");
    std::fs::create_dir_all(&exports_dir).map_err(|error| error.to_string())?;
    let output = exports_dir.join(&artifact.name);
    std::fs::copy(&source, &output).map_err(|error| error.to_string())?;
    Ok(serde_json::json!({ "output_path": output.to_string_lossy() }))
}

fn begin_run(
    state: &tauri::State<'_, crate::workspace::WorkspaceState>,
    dataset_id: &str,
    pipeline: &str,
    parameters_json: String,
    experiment_id: Option<String>,
) -> Result<
    (
        neuro_store::LocalWorkspace,
        neuro_core::AnalysisRun,
        neuro_core::ResourceId,
    ),
    String,
> {
    use neuro_core::ResourceId;

    validate_name(dataset_id).map_err(|error| error.to_string())?;
    let dataset_id =
        ResourceId(uuid::Uuid::parse_str(dataset_id).map_err(|error| error.to_string())?);
    let experiment_id = parse_optional_resource_id(&experiment_id)?;
    let workspace = state
        .0
        .lock()
        .map_err(|_| "workspace lock poisoned".to_string())?;
    let run = workspace
        .create_run(
            dataset_id.clone(),
            pipeline.to_string(),
            parameters_json,
            experiment_id,
        )
        .map_err(|error| error.to_string())?;
    Ok((workspace.clone(), run, dataset_id))
}

fn run_is_cancelled(workspace: &neuro_store::LocalWorkspace, run_id: &ResourceId) -> bool {
    workspace
        .get_run(run_id.clone())
        .ok()
        .flatten()
        .is_some_and(|run| run.status == "cancelled")
}

fn default_mri_output(input: &PathBuf, suffix: &str) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("volume");
    input.with_file_name(format!("{stem}.{suffix}.nii.gz"))
}

#[tauri::command]
pub fn run_mri_operation(
    state: tauri::State<'_, crate::workspace::WorkspaceState>,
    request: MriOperationRequest,
) -> Result<serde_json::Value, String> {
    let pipeline_name = format!("mri_{}", request.operation);
    let parameters_json = serde_json::to_string(&serde_json::json!({
        "operation": request.operation,
        "params": request.params,
        "mask_path": request.mask_path,
    }))
    .map_err(|error| error.to_string())?;
    let (workspace, run, dataset_id) = begin_run(
        &state,
        &request.dataset_id,
        &pipeline_name,
        parameters_json,
        request.experiment_id.clone(),
    )?;
    execute_mri_operation(&workspace, &request, &run, &dataset_id, &pipeline_name)
}

#[tauri::command]
pub fn submit_mri_operation(
    state: tauri::State<'_, crate::workspace::WorkspaceState>,
    request: MriOperationRequest,
) -> Result<serde_json::Value, String> {
    let pipeline_name = format!("mri_{}", request.operation);
    let parameters_json = serde_json::to_string(&serde_json::json!({
        "operation": request.operation,
        "params": request.params,
        "mask_path": request.mask_path,
    }))
    .map_err(|error| error.to_string())?;
    let (workspace, run, dataset_id) = begin_run(
        &state,
        &request.dataset_id,
        &pipeline_name,
        parameters_json,
        request.experiment_id.clone(),
    )?;
    let run_id = run.id.0.to_string();
    std::thread::spawn(move || {
        let _ = execute_mri_operation(&workspace, &request, &run, &dataset_id, &pipeline_name);
    });
    Ok(serde_json::json!({ "run_id": run_id, "status": "running" }))
}

/// Run one MRI operation writing `output`; shared by single ops and
/// multi-step pipelines.
fn run_mri_step(
    operation: &str,
    params: &serde_json::Value,
    mask_path: Option<&str>,
    input: &Path,
    output: &Path,
) -> Result<serde_json::Value, neuro_core::CoreError> {
    use neuro_mri::operations;

    let f64_param = |key: &str, default: f64| {
        params
            .get(key)
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(default)
    };
    let opt_f64 = |key: &str| params.get(key).and_then(serde_json::Value::as_f64);
    match operation {
        "threshold" => operations::threshold(input, output, opt_f64("low"), opt_f64("high")),
        "gaussian_smooth" => operations::gaussian_smooth(input, output, f64_param("fwhm_mm", 4.0)),
        "resample" => {
            let spacing = params
                .get("spacing_mm")
                .and_then(serde_json::Value::as_array)
                .map(|values| {
                    [
                        values.first().and_then(|v| v.as_f64()).unwrap_or(1.0),
                        values.get(1).and_then(|v| v.as_f64()).unwrap_or(1.0),
                        values.get(2).and_then(|v| v.as_f64()).unwrap_or(1.0),
                    ]
                })
                .unwrap_or([1.0, 1.0, 1.0]);
            operations::resample(
                input,
                output,
                spacing,
                params
                    .get("method")
                    .and_then(|value| value.as_str())
                    .unwrap_or("trilinear"),
            )
        }
        "apply_mask" => {
            let mask = mask_path.ok_or_else(|| {
                neuro_core::CoreError::InvalidInput("mask_path is required for apply_mask".into())
            })?;
            operations::apply_mask(input, &PathBuf::from(mask), output)
        }
        "segment_threshold" => {
            operations::segment_threshold(input, output, opt_f64("low"), opt_f64("high"))
        }
        "bias_field_correct" => operations::bias_field_correct(
            input,
            output,
            f64_param("fwhm_mm", 60.0),
            params
                .get("iterations")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(2) as usize,
        ),
        "skull_strip" => operations::skull_strip(
            input,
            output,
            params.get("threshold").and_then(serde_json::Value::as_f64),
            params
                .get("erosion_mm")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(4.0),
        ),
        "register_rigid" | "register_affine" => {
            let moving = params
                .get("moving_path")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| {
                    neuro_core::CoreError::InvalidInput(format!(
                        "moving_path is required for {operation}"
                    ))
                })?;
            let metric = params
                .get("metric")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("ncc");
            let max_evals = params
                .get("max_evaluations")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(400) as usize;
            if operation == "register_affine" {
                operations::register_affine_with_metric(
                    input,
                    &PathBuf::from(moving),
                    output,
                    metric,
                    max_evals,
                )
            } else {
                operations::register_rigid_with_metric(
                    input,
                    &PathBuf::from(moving),
                    output,
                    metric,
                    max_evals,
                )
            }
        }
        other => Err(neuro_core::CoreError::InvalidInput(format!(
            "unsupported MRI operation: {other}"
        ))),
    }
}

fn execute_mri_operation(
    workspace: &neuro_store::LocalWorkspace,
    request: &MriOperationRequest,
    run: &neuro_core::AnalysisRun,
    dataset_id: &neuro_core::ResourceId,
    pipeline_name: &str,
) -> Result<serde_json::Value, String> {
    let _ = workspace.update_run_progress(
        run.id.clone(),
        Some(0.1),
        Some("loading volume".to_string()),
    );
    let input = PathBuf::from(&request.path);
    let result = if let Some(steps) = &request.pipeline {
        if steps.is_empty() || steps.len() > 32 {
            Err(neuro_core::CoreError::InvalidInput(
                "MRI pipeline must contain 1-32 steps".into(),
            ))
        } else {
            let mut current_input = input.clone();
            let mut step_results = Vec::with_capacity(steps.len());
            let mut chained = Ok(());
            for (index, step) in steps.iter().enumerate() {
                let fraction = 0.1 + 0.7 * (index as f64 / steps.len() as f64);
                let _ = workspace.update_run_progress(
                    run.id.clone(),
                    Some(fraction),
                    Some(format!("pipeline step {}: {}", index + 1, step.operation)),
                );
                let output = default_mri_output(&current_input, &step.operation);
                match run_mri_step(
                    &step.operation,
                    &step.params,
                    step.mask_path.as_deref(),
                    &current_input,
                    &output,
                ) {
                    Ok(stats) => {
                        step_results.push(serde_json::json!({
                            "operation": step.operation,
                            "output": output.file_name().and_then(|v| v.to_str()).unwrap_or(""),
                            "result": stats,
                        }));
                        current_input = output;
                    }
                    Err(error) => {
                        chained = Err(error);
                        break;
                    }
                }
            }
            chained.map(|_| {
                serde_json::json!({
                    "operation": "pipeline",
                    "n_steps": steps.len(),
                    "steps": step_results,
                })
            })
        }
    } else {
        run_mri_step(
            &request.operation,
            &request.params,
            request.mask_path.as_deref(),
            &input,
            &default_mri_output(
                &input,
                match request.operation.as_str() {
                    "threshold" => "threshold",
                    "gaussian_smooth" => "smooth",
                    "resample" => "resampled",
                    "apply_mask" => "masked",
                    "segment_threshold" => "mask",
                    "bias_field_correct" => "debias",
                    _ => "result",
                },
            ),
        )
    };
    let _ = workspace.update_run_progress(
        run.id.clone(),
        Some(0.9),
        Some("writing result".to_string()),
    );
    match result {
        Ok(stats) => {
            let value = serde_json::to_value(&stats).map_err(|error| error.to_string())?;
            let result_json = serde_json::to_string(&value).map_err(|e| e.to_string())?;
            workspace
                .complete_run(run.id.clone(), result_json)
                .map_err(|error| error.to_string())?;
            let payload = serde_json::to_vec_pretty(&value).map_err(|e| e.to_string())?;
            workspace
                .add_artifact(
                    run.id.clone(),
                    "operation_result".to_string(),
                    "application/json".to_string(),
                    &payload,
                )
                .map_err(|error| error.to_string())?;
            let _manifest = write_run_manifest(
                workspace,
                &run.id,
                dataset_id,
                pipeline_name,
                &serde_json::from_str(
                    &serde_json::to_string(&serde_json::json!({
                        "operation": request.operation,
                        "params": request.params,
                        "mask_path": request.mask_path,
                        "pipeline": request.pipeline.as_ref().map(|steps| {
                            steps.iter().map(|step| serde_json::json!({
                                "operation": step.operation,
                                "params": step.params,
                                "mask_path": step.mask_path,
                            })).collect::<Vec<_>>()
                        }),
                    }))
                    .unwrap_or_default(),
                )
                .unwrap_or(serde_json::Value::Null),
                &["operation_result".to_string()],
            )?;
            Ok(serde_json::json!({
                "run": workspace.list_runs(dataset_id.clone()).map_err(|e| e.to_string())?,
                "result": value,
            }))
        }
        Err(error) => {
            let message = error.to_string();
            if message != "cancelled" {
                let _ = workspace.fail_run(run.id.clone(), message);
            }
            Err(error.to_string())
        }
    }
}

#[tauri::command]
pub fn get_mri_statistics(path: String) -> Result<serde_json::Value, String> {
    let stats = volume_statistics(&PathBuf::from(path)).map_err(|error| error.to_string())?;
    serde_json::to_value(stats).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_mri_histogram(path: String, bins: usize) -> Result<serde_json::Value, String> {
    let histogram =
        intensity_histogram(&PathBuf::from(path), bins).map_err(|error| error.to_string())?;
    serde_json::to_value(histogram).map_err(|error| error.to_string())
}

#[derive(Debug, Deserialize)]
pub struct GenerateMegSampleRequest {
    pub directory: String,
    #[serde(default)]
    pub magnetometers: Option<usize>,
    #[serde(default)]
    pub gradiometers: Option<usize>,
    #[serde(default)]
    pub sampling_frequency_hz: Option<f64>,
    #[serde(default)]
    pub duration_seconds: Option<f64>,
}

#[tauri::command]
pub fn validate_meg_file_set(path: String) -> Result<serde_json::Value, String> {
    let dir = PathBuf::from(path);
    let names: Vec<String> = std::fs::read_dir(&dir)
        .map_err(|error| error.to_string())?
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
        .collect();
    let format = neuro_meg::formats::detect_format(&names)
        .ok_or_else(|| "could not detect a supported MEG format".to_string())?;
    // Map filenames to coarse roles for validation.
    let role_for = |name: &str| -> &'static str {
        let lower = name.to_lowercase();
        if lower.contains("header") || lower.contains("dataset_description") {
            "header"
        } else if lower.ends_with(".fif")
            || lower.ends_with(".bin")
            || lower.ends_with(".sqd")
            || lower.ends_with(".con")
        {
            "recording"
        } else if lower.contains("event") {
            "events"
        } else if lower.contains("coord") {
            "coordsystem"
        } else if lower.contains("calib") {
            "calibration"
        } else if lower.contains("crosstalk") {
            "crosstalk"
        } else {
            "companion"
        }
    };
    let files: Vec<neuro_meg::formats::FileDeclaration> = names
        .iter()
        .map(|name| neuro_meg::formats::FileDeclaration {
            name: name.clone(),
            role: role_for(name).to_string(),
            size_bytes: std::fs::metadata(dir.join(name))
                .map(|m| m.len())
                .unwrap_or(0),
        })
        .collect();
    let findings = neuro_meg::formats::validate_file_set(format, &files);
    Ok(serde_json::json!({
        "format": format,
        "findings": findings,
        "supported_formats": neuro_meg::formats::supported_formats(),
    }))
}

#[tauri::command]
pub fn generate_meg_sample(request: GenerateMegSampleRequest) -> Result<serde_json::Value, String> {
    let config = neuro_meg::sample::SampleConfig {
        magnetometers: request.magnetometers.unwrap_or(32),
        gradiometers: request.gradiometers.unwrap_or(64),
        auxiliary: 9,
        sampling_frequency_hz: request.sampling_frequency_hz.unwrap_or(250.0),
        duration_seconds: request.duration_seconds.unwrap_or(30.0),
        stimulus_interval_s: 1.5,
        seed: 42,
    };
    let result =
        neuro_meg::sample::generate_sample_recording(&PathBuf::from(&request.directory), &config)
            .map_err(|error| error.to_string())?;
    serde_json::to_value(result).map_err(|error| error.to_string())
}

#[derive(Debug, Deserialize)]
pub struct GenerateEegSampleRequest {
    pub directory: String,
    #[serde(default)]
    pub file_name: Option<String>,
    #[serde(default)]
    pub channels: Option<usize>,
    #[serde(default)]
    pub samples: Option<usize>,
    #[serde(default)]
    pub sampling_frequency_hz: Option<f64>,
}

#[tauri::command]
pub fn generate_eeg_sample(request: GenerateEegSampleRequest) -> Result<serde_json::Value, String> {
    let result = neuro_eeg::write_synthetic_recording(
        &PathBuf::from(&request.directory),
        request.file_name.as_deref().unwrap_or("synthetic_eeg"),
        request.channels.unwrap_or(16),
        request.samples.unwrap_or(10_000),
        request.sampling_frequency_hz.unwrap_or(250.0),
        42,
    )
    .map_err(|error| error.to_string())?;
    serde_json::to_value(result).map_err(|error| error.to_string())
}

#[derive(Debug, Deserialize)]
pub struct MegPipelineRequest {
    pub dataset_id: String,
    pub path: String,
    pub start_s: f64,
    pub end_s: f64,
    pub max_samples: usize,
    pub pipeline: serde_json::Value,
    #[serde(default)]
    pub experiment_id: Option<String>,
    #[serde(default)]
    pub recording: Option<String>,
    #[serde(default)]
    pub compensation_grade: Option<i32>,
    #[serde(default)]
    pub apply_projectors: bool,
}

#[derive(Debug, Deserialize)]
pub struct MegBatchRequest {
    pub dataset_ids: Vec<String>,
    pub start_s: f64,
    pub end_s: f64,
    pub max_samples: usize,
    pub pipeline: serde_json::Value,
    #[serde(default)]
    pub experiment_id: Option<String>,
}

/// Run the same MEG pipeline sequentially over multiple datasets in one
/// background worker. Each dataset gets its own AnalysisRun so per-run
/// progress, cancellation, artifacts, and manifests are preserved.
#[tauri::command]
pub fn submit_meg_batch(
    state: tauri::State<'_, crate::workspace::WorkspaceState>,
    request: MegBatchRequest,
) -> Result<serde_json::Value, String> {
    const MAX_BATCH: usize = 64;
    if request.dataset_ids.is_empty() || request.dataset_ids.len() > MAX_BATCH {
        return Err(format!("batch size must be between 1 and {MAX_BATCH}"));
    }
    let dataset_ids: Vec<ResourceId> = request
        .dataset_ids
        .iter()
        .map(|raw| {
            uuid::Uuid::parse_str(raw)
                .map(ResourceId)
                .map_err(|error| error.to_string())
        })
        .collect::<Result<_, _>>()?;
    let experiment_id = parse_optional_resource_id(&request.experiment_id)?;
    let workspace = state
        .0
        .lock()
        .map_err(|_| "workspace lock poisoned".to_string())?
        .clone();
    let queued = dataset_ids.len();
    std::thread::spawn(move || {
        for dataset_id in dataset_ids {
            let Ok(Some(dataset)) = workspace.get_dataset(dataset_id.clone()) else {
                continue;
            };
            let request = MegPipelineRequest {
                dataset_id: dataset_id.0.to_string(),
                path: dataset.path.clone(),
                start_s: request.start_s,
                end_s: request.end_s,
                max_samples: request.max_samples,
                pipeline: request.pipeline.clone(),
                experiment_id: experiment_id.as_ref().map(|id| id.0.to_string()),
                recording: None,
                compensation_grade: None,
                apply_projectors: false,
            };
            let parameters_json = match serde_json::to_string(&request.pipeline) {
                Ok(value) => value,
                Err(_) => continue,
            };
            let run = match workspace.create_run(
                dataset_id.clone(),
                "meg_pipeline".to_string(),
                parameters_json,
                experiment_id.clone(),
            ) {
                Ok(run) => run,
                Err(_) => continue,
            };
            let _ = execute_meg_pipeline(&workspace, &request, &run, &dataset_id);
        }
    });
    Ok(serde_json::json!({ "queued": queued, "status": "running" }))
}

#[tauri::command]
pub fn run_meg_pipeline(
    state: tauri::State<'_, crate::workspace::WorkspaceState>,
    request: MegPipelineRequest,
) -> Result<serde_json::Value, String> {
    let parameters_json =
        serde_json::to_string(&request.pipeline).map_err(|error| error.to_string())?;
    let (workspace, run, dataset_id) = begin_run(
        &state,
        &request.dataset_id,
        "meg_pipeline",
        parameters_json,
        request.experiment_id.clone(),
    )?;
    execute_meg_pipeline(&workspace, &request, &run, &dataset_id)
}

#[tauri::command]
pub fn submit_meg_pipeline(
    state: tauri::State<'_, crate::workspace::WorkspaceState>,
    request: MegPipelineRequest,
) -> Result<serde_json::Value, String> {
    let parameters_json =
        serde_json::to_string(&request.pipeline).map_err(|error| error.to_string())?;
    let (workspace, run, dataset_id) = begin_run(
        &state,
        &request.dataset_id,
        "meg_pipeline",
        parameters_json,
        request.experiment_id.clone(),
    )?;
    let run_id = run.id.0.to_string();
    std::thread::spawn(move || {
        let _ = execute_meg_pipeline(&workspace, &request, &run, &dataset_id);
    });
    Ok(serde_json::json!({ "run_id": run_id, "status": "running" }))
}

fn execute_meg_pipeline(
    workspace: &neuro_store::LocalWorkspace,
    request: &MegPipelineRequest,
    run: &neuro_core::AnalysisRun,
    dataset_id: &neuro_core::ResourceId,
) -> Result<serde_json::Value, String> {
    let _ = workspace.update_run_progress(
        run.id.clone(),
        Some(0.05),
        Some("loading recording".to_string()),
    );
    let result = (|| -> Result<serde_json::Value, String> {
        let path = PathBuf::from(&request.path);
        let metadata = inspect_meg(&path).map_err(|error| error.to_string())?;
        let channel_names: Vec<String> = metadata
            .channels
            .iter()
            .map(|channel| channel.name.clone())
            .collect();
        let window = read_meg_window(
            &path,
            request.start_s,
            request.end_s,
            &channel_names,
            request.max_samples,
            request.recording.as_deref(),
            request.compensation_grade,
            request.apply_projectors,
        )
        .map_err(|error| error.to_string())?;
        let run_id = run.id.clone();
        let cancel = || run_is_cancelled(workspace, &run_id);
        let progress = |fraction: f64, message: &str| {
            let _ = workspace.update_run_progress(
                run_id.clone(),
                Some(0.1 + 0.8 * fraction),
                Some(message.to_string()),
            );
        };
        let result = neuro_meg::pipeline::run_pipeline(
            &request.pipeline,
            &window,
            &metadata,
            &channel_names,
            Some(&cancel),
            Some(&progress),
        )
        .map_err(|error| error.to_string())?;
        serde_json::to_value(&result).map_err(|error| error.to_string())
    })();
    let _ = workspace.update_run_progress(
        run.id.clone(),
        Some(0.95),
        Some("writing result".to_string()),
    );
    match result {
        Ok(value) => {
            let result_json = serde_json::to_string(&value).map_err(|e| e.to_string())?;
            let completed = workspace
                .complete_run(run.id.clone(), result_json)
                .map_err(|error| error.to_string())?;
            let payload = serde_json::to_vec_pretty(&value).map_err(|e| e.to_string())?;
            let artifact = workspace
                .add_artifact(
                    run.id.clone(),
                    "pipeline_result".to_string(),
                    "application/json".to_string(),
                    &payload,
                )
                .map_err(|error| error.to_string())?;
            let _manifest = write_run_manifest(
                workspace,
                &run.id,
                dataset_id,
                "meg_pipeline",
                &request.pipeline,
                &[artifact.name.clone()],
            )?;
            Ok(serde_json::json!({
                "run": completed,
                "artifact": artifact,
                "result": value,
            }))
        }
        Err(error) => {
            if error != "cancelled" {
                let _ = workspace.fail_run(run.id.clone(), error.clone());
            }
            Err(error)
        }
    }
}

#[allow(dead_code)]
fn _project(kind: ProjectKind, name: String) -> Project {
    let now = chrono::Utc::now();
    Project {
        id: ResourceId::new(),
        name,
        description: String::new(),
        kind,
        created_at: now,
        updated_at: now,
    }
}
