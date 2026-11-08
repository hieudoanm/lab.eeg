use neuro_core::{
    AnalysisRun, Artifact, Dataset, Experiment, Project, ProjectKind, ResourceId, UploadSession,
};
use neuro_store::LocalWorkspace;
use serde::Deserialize;
use std::sync::Mutex;
use tauri::State;

pub struct WorkspaceState(pub Mutex<LocalWorkspace>);

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub kind: ProjectKind,
    #[serde(default)]
    pub description: String,
}

#[tauri::command]
pub fn list_projects(state: State<'_, WorkspaceState>) -> Result<Vec<Project>, String> {
    state
        .0
        .lock()
        .map_err(|_| "workspace lock poisoned".to_string())?
        .list_projects()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_project(
    state: State<'_, WorkspaceState>,
    request: CreateProjectRequest,
) -> Result<Project, String> {
    state
        .0
        .lock()
        .map_err(|_| "workspace lock poisoned".to_string())?
        .create_project(request.name, request.kind, request.description)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_datasets(
    state: State<'_, WorkspaceState>,
    project_id: String,
    query: Option<String>,
    sort: Option<String>,
) -> Result<Vec<Dataset>, String> {
    let project_id =
        ResourceId(uuid::Uuid::parse_str(&project_id).map_err(|error| error.to_string())?);
    let mut datasets = state
        .0
        .lock()
        .map_err(|_| "workspace lock poisoned".to_string())?
        .list_datasets(project_id)
        .map_err(|error| error.to_string())?;
    if let Some(query) = query.filter(|value| !value.trim().is_empty()) {
        let needle = query.trim().to_lowercase();
        datasets.retain(|dataset| {
            dataset.name.to_lowercase().contains(&needle)
                || dataset.path.to_lowercase().contains(&needle)
                || dataset.format.to_lowercase().contains(&needle)
        });
    }
    match sort.as_deref() {
        Some("name") => datasets.sort_by(|a, b| a.name.cmp(&b.name)),
        Some("newest") => datasets.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
        Some("version") => datasets.sort_by(|a, b| b.version.cmp(&a.version)),
        _ => datasets.sort_by(|a, b| a.name.cmp(&b.name)),
    }
    Ok(datasets)
}

#[derive(Debug, Deserialize)]
pub struct CreateUploadSessionRequest {
    pub project_id: String,
    pub filename: String,
    pub total_bytes: u64,
    #[serde(default)]
    pub expected_sha256: Option<String>,
}

#[tauri::command]
pub fn create_upload_session(
    state: State<'_, WorkspaceState>,
    request: CreateUploadSessionRequest,
) -> Result<UploadSession, String> {
    let project_id =
        ResourceId(uuid::Uuid::parse_str(&request.project_id).map_err(|error| error.to_string())?);
    state
        .0
        .lock()
        .map_err(|_| "workspace lock poisoned".to_string())?
        .create_upload_session(
            project_id,
            request.filename,
            request.total_bytes,
            request.expected_sha256,
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn append_upload_chunk(
    state: State<'_, WorkspaceState>,
    session_id: String,
    data: Vec<u8>,
    offset: Option<u64>,
) -> Result<UploadSession, String> {
    let session_id =
        ResourceId(uuid::Uuid::parse_str(&session_id).map_err(|error| error.to_string())?);
    state
        .0
        .lock()
        .map_err(|_| "workspace lock poisoned".to_string())?
        .append_upload_chunk_at(session_id, offset, &data)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_upload_sessions(
    state: State<'_, WorkspaceState>,
    project_id: String,
) -> Result<Vec<UploadSession>, String> {
    let project_id =
        ResourceId(uuid::Uuid::parse_str(&project_id).map_err(|error| error.to_string())?);
    state
        .0
        .lock()
        .map_err(|_| "workspace lock poisoned".to_string())?
        .list_upload_sessions(project_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn abort_upload_session(
    state: State<'_, WorkspaceState>,
    session_id: String,
) -> Result<UploadSession, String> {
    let session_id =
        ResourceId(uuid::Uuid::parse_str(&session_id).map_err(|error| error.to_string())?);
    state
        .0
        .lock()
        .map_err(|_| "workspace lock poisoned".to_string())?
        .abort_upload_session(session_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn verify_upload_session(
    state: State<'_, WorkspaceState>,
    session_id: String,
) -> Result<serde_json::Value, String> {
    let session_id =
        ResourceId(uuid::Uuid::parse_str(&session_id).map_err(|error| error.to_string())?);
    state
        .0
        .lock()
        .map_err(|_| "workspace lock poisoned".to_string())?
        .verify_upload_session(session_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn finalize_upload_session(
    state: State<'_, WorkspaceState>,
    session_id: String,
) -> Result<serde_json::Value, String> {
    let session_id =
        ResourceId(uuid::Uuid::parse_str(&session_id).map_err(|error| error.to_string())?);
    let (session, dataset) = state
        .0
        .lock()
        .map_err(|_| "workspace lock poisoned".to_string())?
        .finalize_upload_session(session_id)
        .map_err(|error| error.to_string())?;
    Ok(serde_json::json!({ "session": session, "dataset": dataset }))
}

#[tauri::command]
pub fn register_dataset(
    state: State<'_, WorkspaceState>,
    dataset: Dataset,
) -> Result<Dataset, String> {
    state
        .0
        .lock()
        .map_err(|_| "workspace lock poisoned".to_string())?
        .register_dataset(dataset)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_analysis_runs(
    state: State<'_, WorkspaceState>,
    dataset_id: String,
) -> Result<Vec<AnalysisRun>, String> {
    let dataset_id =
        ResourceId(uuid::Uuid::parse_str(&dataset_id).map_err(|error| error.to_string())?);
    state
        .0
        .lock()
        .map_err(|_| "workspace lock poisoned".to_string())?
        .list_runs(dataset_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_analysis_run(
    state: State<'_, WorkspaceState>,
    dataset_id: String,
    run_id: String,
) -> Result<Option<AnalysisRun>, String> {
    let dataset_id =
        ResourceId(uuid::Uuid::parse_str(&dataset_id).map_err(|error| error.to_string())?);
    let run_id = ResourceId(uuid::Uuid::parse_str(&run_id).map_err(|error| error.to_string())?);
    let runs = state
        .0
        .lock()
        .map_err(|_| "workspace lock poisoned".to_string())?
        .list_runs(dataset_id)
        .map_err(|error| error.to_string())?;
    Ok(runs.into_iter().find(|run| run.id == run_id))
}

#[derive(Debug, Deserialize)]
pub struct CreateExperimentRequest {
    pub project_id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[tauri::command]
pub fn create_experiment(
    state: State<'_, WorkspaceState>,
    request: CreateExperimentRequest,
) -> Result<Experiment, String> {
    let project_id =
        ResourceId(uuid::Uuid::parse_str(&request.project_id).map_err(|error| error.to_string())?);
    state
        .0
        .lock()
        .map_err(|_| "workspace lock poisoned".to_string())?
        .create_experiment(project_id, request.name, request.description)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_experiments(
    state: State<'_, WorkspaceState>,
    project_id: String,
) -> Result<Vec<Experiment>, String> {
    let project_id =
        ResourceId(uuid::Uuid::parse_str(&project_id).map_err(|error| error.to_string())?);
    state
        .0
        .lock()
        .map_err(|_| "workspace lock poisoned".to_string())?
        .list_experiments(project_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn cancel_analysis_run(
    state: State<'_, WorkspaceState>,
    run_id: String,
) -> Result<AnalysisRun, String> {
    let run_id = ResourceId(uuid::Uuid::parse_str(&run_id).map_err(|error| error.to_string())?);
    state
        .0
        .lock()
        .map_err(|_| "workspace lock poisoned".to_string())?
        .cancel_run(run_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_artifacts(
    state: State<'_, WorkspaceState>,
    run_id: String,
) -> Result<Vec<Artifact>, String> {
    let run_id = ResourceId(uuid::Uuid::parse_str(&run_id).map_err(|error| error.to_string())?);
    state
        .0
        .lock()
        .map_err(|_| "workspace lock poisoned".to_string())?
        .list_artifacts(run_id)
        .map_err(|error| error.to_string())
}
