use neuro_core::{
    AnalysisRun, Artifact, CoreError, Dataset, Experiment, Project, ProjectKind, ResourceId,
    UploadSession,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct WorkspaceFile {
    #[serde(default)]
    projects: Vec<Project>,
    #[serde(default)]
    datasets: Vec<Dataset>,
    #[serde(default)]
    runs: Vec<AnalysisRun>,
    #[serde(default)]
    artifacts: Vec<Artifact>,
    #[serde(default)]
    experiments: Vec<Experiment>,
    #[serde(default)]
    upload_sessions: Vec<UploadSession>,
    #[serde(default)]
    pipeline_presets: Vec<PipelinePreset>,
}

/// A saved pipeline definition (e.g. an MRI step chain) for reuse.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PipelinePreset {
    pub id: ResourceId,
    pub name: String,
    /// Owning modality: "mri", "eeg", or "meg".
    pub modality: String,
    /// Serialized pipeline steps (shape is modality-specific).
    pub steps: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct LocalWorkspace {
    root: PathBuf,
}

impl LocalWorkspace {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, CoreError> {
        let root = root.as_ref().to_path_buf();
        std::fs::create_dir_all(root.join("artifacts"))?;
        std::fs::create_dir_all(root.join("uploads"))?;
        Ok(Self { root })
    }

    pub fn root_dir(&self) -> PathBuf {
        self.root.clone()
    }

    fn file_path(&self) -> PathBuf {
        self.root.join("workspace.json")
    }

    fn load(&self) -> Result<WorkspaceFile, CoreError> {
        let path = self.file_path();
        if !path.exists() {
            return Ok(WorkspaceFile::default());
        }
        let text = std::fs::read_to_string(path)?;
        serde_json::from_str(&text).map_err(CoreError::from)
    }

    fn save(&self, workspace: &WorkspaceFile) -> Result<(), CoreError> {
        let path = self.file_path();
        let temp = self.root.join("workspace.json.tmp");
        std::fs::write(&temp, serde_json::to_vec_pretty(workspace)?)?;
        std::fs::rename(temp, path)?;
        Ok(())
    }

    pub fn list_projects(&self) -> Result<Vec<Project>, CoreError> {
        Ok(self.load()?.projects)
    }

    pub fn create_project(
        &self,
        name: String,
        kind: ProjectKind,
        description: String,
    ) -> Result<Project, CoreError> {
        neuro_core::validate_name(&name)?;
        let mut workspace = self.load()?;
        let now = chrono::Utc::now();
        let project = Project {
            id: ResourceId::new(),
            name,
            description,
            kind,
            created_at: now,
            updated_at: now,
        };
        workspace.projects.push(project.clone());
        self.save(&workspace)?;
        Ok(project)
    }

    pub fn get_dataset(&self, dataset_id: ResourceId) -> Result<Option<Dataset>, CoreError> {
        Ok(self
            .load()?
            .datasets
            .into_iter()
            .find(|dataset| dataset.id == dataset_id))
    }

    pub fn list_datasets(&self, project_id: ResourceId) -> Result<Vec<Dataset>, CoreError> {
        Ok(self
            .load()?
            .datasets
            .into_iter()
            .filter(|dataset| dataset.project_id == project_id)
            .collect())
    }

    pub fn register_dataset(&self, mut dataset: Dataset) -> Result<Dataset, CoreError> {
        let mut workspace = self.load()?;
        if !workspace
            .projects
            .iter()
            .any(|project| project.id == dataset.project_id)
        {
            return Err(CoreError::NotFound("project not found".into()));
        }
        if let Some(existing) = workspace.datasets.iter_mut().find(|existing| {
            existing.project_id == dataset.project_id && existing.path == dataset.path
        }) {
            if existing.checksum_sha256 != dataset.checksum_sha256 {
                existing.version += 1;
                existing.checksum_sha256 = dataset.checksum_sha256.clone();
                existing.size_bytes = dataset.size_bytes;
                existing.name = dataset.name.clone();
                existing.updated_at = Some(chrono::Utc::now());
                let updated = existing.clone();
                self.save(&workspace)?;
                return Ok(updated);
            }
            return Ok(existing.clone());
        }
        dataset.version = 1;
        workspace.datasets.push(dataset.clone());
        self.save(&workspace)?;
        Ok(dataset)
    }

    pub fn list_runs(&self, dataset_id: ResourceId) -> Result<Vec<AnalysisRun>, CoreError> {
        Ok(self
            .load()?
            .runs
            .into_iter()
            .filter(|run| run.dataset_id == dataset_id)
            .collect())
    }

    pub fn create_experiment(
        &self,
        project_id: ResourceId,
        name: String,
        description: String,
    ) -> Result<Experiment, CoreError> {
        neuro_core::validate_name(&name)?;
        let mut workspace = self.load()?;
        if !workspace
            .projects
            .iter()
            .any(|project| project.id == project_id)
        {
            return Err(CoreError::NotFound("project not found".into()));
        }
        let experiment = Experiment {
            id: ResourceId::new(),
            project_id,
            name,
            description,
            created_at: chrono::Utc::now(),
        };
        workspace.experiments.push(experiment.clone());
        self.save(&workspace)?;
        Ok(experiment)
    }

    pub fn list_experiments(&self, project_id: ResourceId) -> Result<Vec<Experiment>, CoreError> {
        Ok(self
            .load()?
            .experiments
            .into_iter()
            .filter(|experiment| experiment.project_id == project_id)
            .collect())
    }

    pub fn create_run(
        &self,
        dataset_id: ResourceId,
        pipeline: String,
        parameters_json: String,
        experiment_id: Option<ResourceId>,
    ) -> Result<AnalysisRun, CoreError> {
        let mut workspace = self.load()?;
        if !workspace
            .datasets
            .iter()
            .any(|dataset| dataset.id == dataset_id)
        {
            return Err(CoreError::NotFound("dataset not found".into()));
        }
        let run = AnalysisRun {
            id: ResourceId::new(),
            dataset_id,
            experiment_id,
            pipeline,
            parameters_json,
            status: "running".into(),
            result_json: None,
            error: None,
            progress: None,
            progress_message: None,
            created_at: chrono::Utc::now(),
            completed_at: None,
        };
        workspace.runs.push(run.clone());
        self.save(&workspace)?;
        Ok(run)
    }

    pub fn complete_run(
        &self,
        run_id: ResourceId,
        result_json: String,
    ) -> Result<AnalysisRun, CoreError> {
        let mut workspace = self.load()?;
        let run = workspace
            .runs
            .iter_mut()
            .find(|run| run.id == run_id)
            .ok_or_else(|| CoreError::NotFound("analysis run not found".into()))?;
        if run.status == "cancelled" {
            let current = run.clone();
            return Ok(current);
        }
        run.status = "succeeded".into();
        run.result_json = Some(result_json);
        run.completed_at = Some(chrono::Utc::now());
        let updated = run.clone();
        self.save(&workspace)?;
        Ok(updated)
    }

    pub fn get_run(&self, run_id: ResourceId) -> Result<Option<AnalysisRun>, CoreError> {
        Ok(self.load()?.runs.into_iter().find(|run| run.id == run_id))
    }

    pub fn cancel_run(&self, run_id: ResourceId) -> Result<AnalysisRun, CoreError> {
        let mut workspace = self.load()?;
        let run = workspace
            .runs
            .iter_mut()
            .find(|run| run.id == run_id)
            .ok_or_else(|| CoreError::NotFound("analysis run not found".into()))?;
        if run.status != "running" {
            return Err(CoreError::InvalidInput(
                "only running analyses can be cancelled".into(),
            ));
        }
        run.status = "cancelled".into();
        run.completed_at = Some(chrono::Utc::now());
        let updated = run.clone();
        self.save(&workspace)?;
        Ok(updated)
    }

    pub fn fail_run(&self, run_id: ResourceId, error: String) -> Result<AnalysisRun, CoreError> {
        let mut workspace = self.load()?;
        let run = workspace
            .runs
            .iter_mut()
            .find(|run| run.id == run_id)
            .ok_or_else(|| CoreError::NotFound("analysis run not found".into()))?;
        if run.status == "cancelled" {
            let current = run.clone();
            return Ok(current);
        }
        run.status = "failed".into();
        run.error = Some(error);
        run.completed_at = Some(chrono::Utc::now());
        let updated = run.clone();
        self.save(&workspace)?;
        Ok(updated)
    }

    pub fn update_run_progress(
        &self,
        run_id: ResourceId,
        progress: Option<f64>,
        progress_message: Option<String>,
    ) -> Result<(), CoreError> {
        let mut workspace = self.load()?;
        let run = workspace
            .runs
            .iter_mut()
            .find(|run| run.id == run_id)
            .ok_or_else(|| CoreError::NotFound("analysis run not found".into()))?;
        if run.status != "running" {
            return Ok(());
        }
        run.progress = progress;
        run.progress_message = progress_message;
        self.save(&workspace)
    }

    pub fn list_artifacts(&self, run_id: ResourceId) -> Result<Vec<Artifact>, CoreError> {
        Ok(self
            .load()?
            .artifacts
            .into_iter()
            .filter(|artifact| artifact.run_id == run_id)
            .collect())
    }

    pub fn artifact_path(&self, artifact_id: ResourceId) -> PathBuf {
        self.root
            .join("artifacts")
            .join(format!("{}.json", artifact_id.0))
    }

    pub fn add_artifact(
        &self,
        run_id: ResourceId,
        name: String,
        media_type: String,
        payload: &[u8],
    ) -> Result<Artifact, CoreError> {
        let mut workspace = self.load()?;
        if !workspace.runs.iter().any(|run| run.id == run_id) {
            return Err(CoreError::NotFound("analysis run not found".into()));
        }
        let artifact = Artifact {
            id: ResourceId::new(),
            run_id,
            name: name.clone(),
            path: String::new(),
            media_type,
            checksum_sha256: neuro_core::sha256_hex(payload),
            size_bytes: payload.len() as u64,
        };
        let path = self.artifact_path(artifact.id.clone());
        std::fs::write(&path, payload)?;
        let mut stored = artifact.clone();
        stored.path = path.to_string_lossy().into_owned();
        workspace.artifacts.push(stored.clone());
        self.save(&workspace)?;
        Ok(stored)
    }

    pub fn upload_part_path(&self, session_id: &ResourceId) -> PathBuf {
        self.root
            .join("uploads")
            .join(format!("{}.part", session_id.0))
    }

    pub fn create_upload_session(
        &self,
        project_id: ResourceId,
        filename: String,
        total_bytes: u64,
        expected_sha256: Option<String>,
    ) -> Result<UploadSession, CoreError> {
        const MAX_UPLOAD_BYTES: u64 = 8 * 1024 * 1024 * 1024;
        if total_bytes == 0 || total_bytes > MAX_UPLOAD_BYTES {
            return Err(CoreError::InvalidInput(
                "upload size must be between 1 byte and 8 GiB".into(),
            ));
        }
        neuro_core::validate_name(&filename)?;
        if let Some(expected) = &expected_sha256 {
            let valid = expected.len() == 64
                && expected
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase());
            if !valid {
                return Err(CoreError::InvalidInput(
                    "expected_sha256 must be a lowercase hex SHA-256 digest".into(),
                ));
            }
        }
        let mut workspace = self.load()?;
        if !workspace
            .projects
            .iter()
            .any(|project| project.id == project_id)
        {
            return Err(CoreError::NotFound("project not found".into()));
        }
        let session = UploadSession {
            id: ResourceId::new(),
            project_id,
            filename,
            total_bytes,
            received_bytes: 0,
            status: "open".into(),
            checksum_sha256: None,
            expected_sha256,
            created_at: chrono::Utc::now(),
            completed_at: None,
        };
        workspace.upload_sessions.push(session.clone());
        self.save(&workspace)?;
        Ok(session)
    }

    pub fn append_upload_chunk(
        &self,
        session_id: ResourceId,
        data: &[u8],
    ) -> Result<UploadSession, CoreError> {
        self.append_upload_chunk_at(session_id, None, data)
    }

    /// Append a chunk, optionally asserting the expected byte offset so a
    /// retried request cannot silently duplicate data.
    pub fn append_upload_chunk_at(
        &self,
        session_id: ResourceId,
        expected_offset: Option<u64>,
        data: &[u8],
    ) -> Result<UploadSession, CoreError> {
        const MAX_CHUNK_BYTES: usize = 16 * 1024 * 1024;
        if data.is_empty() || data.len() > MAX_CHUNK_BYTES {
            return Err(CoreError::InvalidInput(
                "upload chunk must be between 1 byte and 16 MiB".into(),
            ));
        }
        let mut workspace = self.load()?;
        let session = workspace
            .upload_sessions
            .iter_mut()
            .find(|session| session.id == session_id)
            .ok_or_else(|| CoreError::NotFound("upload session not found".into()))?;
        if session.status != "open" {
            return Err(CoreError::InvalidInput("upload session is closed".into()));
        }
        // Reconcile against the on-disk .part file so a crash between the
        // file write and the workspace save cannot corrupt the offset.
        let part_path = self.upload_part_path(&session_id);
        if let Ok(meta) = std::fs::metadata(&part_path) {
            let on_disk = meta.len().min(session.total_bytes);
            if on_disk != session.received_bytes {
                session.received_bytes = on_disk;
            }
        }
        if let Some(expected) = expected_offset {
            if expected != session.received_bytes {
                return Err(CoreError::InvalidInput(format!(
                    "upload offset mismatch: expected {} but session is at {}",
                    expected, session.received_bytes
                )));
            }
        }
        if session.received_bytes + data.len() as u64 > session.total_bytes {
            return Err(CoreError::InvalidInput(
                "upload chunk exceeds declared size".into(),
            ));
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&part_path)?;
        use std::io::Write;
        file.write_all(data)?;
        session.received_bytes += data.len() as u64;
        let updated = session.clone();
        self.save(&workspace)?;
        Ok(updated)
    }

    pub fn list_upload_sessions(
        &self,
        project_id: ResourceId,
    ) -> Result<Vec<UploadSession>, CoreError> {
        Ok(self
            .load()?
            .upload_sessions
            .into_iter()
            .filter(|session| session.project_id == project_id)
            .collect())
    }

    pub fn abort_upload_session(&self, session_id: ResourceId) -> Result<UploadSession, CoreError> {
        let mut workspace = self.load()?;
        let session = workspace
            .upload_sessions
            .iter_mut()
            .find(|session| session.id == session_id)
            .ok_or_else(|| CoreError::NotFound("upload session not found".into()))?;
        if session.status != "open" {
            return Err(CoreError::InvalidInput("upload session is closed".into()));
        }
        session.status = "aborted".into();
        session.completed_at = Some(chrono::Utc::now());
        let _ = std::fs::remove_file(self.upload_part_path(&session_id));
        let updated = session.clone();
        self.save(&workspace)?;
        Ok(updated)
    }

    pub fn save_pipeline_preset(
        &self,
        name: String,
        modality: String,
        steps: serde_json::Value,
    ) -> Result<PipelinePreset, CoreError> {
        neuro_core::validate_name(&name)?;
        if !matches!(modality.as_str(), "mri" | "eeg" | "meg") {
            return Err(CoreError::InvalidInput("unknown preset modality".into()));
        }
        let steps_json = serde_json::to_string(&steps)?;
        if steps_json.len() > 1_000_000 {
            return Err(CoreError::InvalidInput("preset too large".into()));
        }
        let mut workspace = self.load()?;
        if let Some(existing) = workspace
            .pipeline_presets
            .iter_mut()
            .find(|preset| preset.name == name && preset.modality == modality)
        {
            existing.steps = steps;
            existing.created_at = chrono::Utc::now();
            let updated = existing.clone();
            self.save(&workspace)?;
            return Ok(updated);
        }
        let preset = PipelinePreset {
            id: ResourceId::new(),
            name,
            modality,
            steps,
            created_at: chrono::Utc::now(),
        };
        workspace.pipeline_presets.push(preset.clone());
        self.save(&workspace)?;
        Ok(preset)
    }

    pub fn list_pipeline_presets(&self) -> Result<Vec<PipelinePreset>, CoreError> {
        Ok(self.load()?.pipeline_presets)
    }

    pub fn delete_pipeline_preset(&self, preset_id: ResourceId) -> Result<(), CoreError> {
        let mut workspace = self.load()?;
        let before = workspace.pipeline_presets.len();
        workspace
            .pipeline_presets
            .retain(|preset| preset.id != preset_id);
        if workspace.pipeline_presets.len() == before {
            return Err(CoreError::NotFound("pipeline preset not found".into()));
        }
        self.save(&workspace)
    }

    /// Recompute the on-disk `.part` hash and compare it against the session
    /// metadata — used to verify a resumed upload is still intact.
    pub fn verify_upload_session(
        &self,
        session_id: ResourceId,
    ) -> Result<serde_json::Value, CoreError> {
        let workspace = self.load()?;
        let session = workspace
            .upload_sessions
            .iter()
            .find(|session| session.id == session_id)
            .ok_or_else(|| CoreError::NotFound("upload session not found".into()))?;
        let part_path = self.upload_part_path(&session_id);
        let meta = std::fs::metadata(&part_path).ok();
        let on_disk = meta.as_ref().map(|meta| meta.len()).unwrap_or(0);
        let checksum = neuro_core::file_sha256_hex(&part_path).ok();
        let expected_match = match (&session.expected_sha256, &checksum) {
            (Some(expected), Some(actual)) if on_disk == session.total_bytes => {
                Some(expected == actual)
            }
            _ => None,
        };
        Ok(serde_json::json!({
            "session_id": session.id.0,
            "status": session.status,
            "received_bytes": session.received_bytes,
            "on_disk_bytes": on_disk,
            "total_bytes": session.total_bytes,
            "size_matches": on_disk == session.received_bytes.min(session.total_bytes),
            "part_sha256": checksum,
            "expected_sha256": session.expected_sha256,
            "expected_match": expected_match,
        }))
    }

    pub fn finalize_upload_session(
        &self,
        session_id: ResourceId,
    ) -> Result<(UploadSession, Dataset), CoreError> {
        let mut workspace = self.load()?;
        let session_index = workspace
            .upload_sessions
            .iter()
            .position(|session| session.id == session_id)
            .ok_or_else(|| CoreError::NotFound("upload session not found".into()))?;
        {
            let session = &workspace.upload_sessions[session_index];
            if session.status != "open" {
                return Err(CoreError::InvalidInput("upload session is closed".into()));
            }
            if session.received_bytes != session.total_bytes {
                return Err(CoreError::InvalidInput("upload is incomplete".into()));
            }
        }
        let part_path = self.upload_part_path(&session_id);
        let bytes = std::fs::read(&part_path)?;
        let checksum = neuro_core::sha256_hex(&bytes);
        let expected = workspace.upload_sessions[session_index]
            .expected_sha256
            .clone();
        if let Some(expected) = expected {
            if expected != checksum {
                return Err(CoreError::InvalidInput(
                    "upload integrity check failed: .part checksum does not match expected SHA-256"
                        .into(),
                ));
            }
        }
        let final_dir = self.root.join("uploads").join(session_id.0.to_string());
        std::fs::create_dir_all(&final_dir)?;
        let final_path = final_dir.join(&workspace.upload_sessions[session_index].filename);
        std::fs::rename(&part_path, &final_path)?;
        let session = &mut workspace.upload_sessions[session_index];
        session.status = "completed".into();
        session.checksum_sha256 = Some(checksum.clone());
        session.completed_at = Some(chrono::Utc::now());
        let project_id = session.project_id.clone();
        let filename = session.filename.clone();
        let size = session.total_bytes;
        let stem = Path::new(&filename)
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("upload")
            .to_string();
        let format = Path::new(&filename)
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("bin")
            .to_lowercase();
        let dataset = Dataset {
            id: ResourceId::new(),
            project_id,
            name: stem,
            path: final_path.to_string_lossy().into_owned(),
            format,
            checksum_sha256: checksum,
            size_bytes: size,
            version: 1,
            created_at: chrono::Utc::now(),
            updated_at: None,
        };
        workspace.datasets.push(dataset.clone());
        let updated_session = session.clone();
        self.save(&workspace)?;
        Ok((updated_session, dataset))
    }
}
