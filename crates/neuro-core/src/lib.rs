use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceId(pub Uuid);

impl ResourceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ResourceId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectKind {
    Eeg,
    Meg,
    Mri,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: ResourceId,
    pub name: String,
    pub description: String,
    pub kind: ProjectKind,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dataset {
    pub id: ResourceId,
    pub project_id: ResourceId,
    pub name: String,
    pub path: String,
    pub format: String,
    pub checksum_sha256: String,
    pub size_bytes: u64,
    #[serde(default = "default_version")]
    pub version: u32,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

fn default_version() -> u32 {
    1
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Experiment {
    pub id: ResourceId,
    pub project_id: ResourceId,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnalysisRun {
    pub id: ResourceId,
    pub dataset_id: ResourceId,
    #[serde(default)]
    pub experiment_id: Option<ResourceId>,
    pub pipeline: String,
    pub parameters_json: String,
    pub status: String,
    pub result_json: Option<String>,
    pub error: Option<String>,
    #[serde(default)]
    pub progress: Option<f64>,
    #[serde(default)]
    pub progress_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UploadSession {
    pub id: ResourceId,
    pub project_id: ResourceId,
    pub filename: String,
    pub total_bytes: u64,
    pub received_bytes: u64,
    pub status: String,
    pub checksum_sha256: Option<String>,
    /// Optional client-declared SHA-256 of the complete file, verified at
    /// finalize time.
    #[serde(default)]
    pub expected_sha256: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artifact {
    pub id: ResourceId,
    pub run_id: ResourceId,
    pub name: String,
    pub path: String,
    pub media_type: String,
    pub checksum_sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("cancelled")]
    Cancelled,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    hex::encode(digest)
}

pub fn file_sha256_hex(path: impl AsRef<std::path::Path>) -> Result<String, CoreError> {
    use std::io::Read;

    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

pub fn validate_name(name: &str) -> Result<(), CoreError> {
    if name.trim().is_empty() {
        return Err(CoreError::InvalidInput("name must not be empty".into()));
    }
    if name.len() > 200 {
        return Err(CoreError::InvalidInput("name is too long".into()));
    }
    Ok(())
}

pub mod ica;
