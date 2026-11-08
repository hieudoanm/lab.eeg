use neuro_core::{Dataset, ProjectKind, ResourceId};
use neuro_store::LocalWorkspace;

fn dataset(project_id: ResourceId, name: &str) -> Dataset {
    Dataset {
        id: ResourceId::new(),
        project_id,
        name: name.to_string(),
        path: format!("/tmp/{name}"),
        format: "meg".to_string(),
        checksum_sha256: "abc".to_string(),
        size_bytes: 10,
        version: 1,
        created_at: chrono::Utc::now(),
        updated_at: None,
    }
}

#[test]
fn runs_and_artifacts_persist_across_reopen() {
    let dir = std::env::temp_dir().join(format!("neuro-store-test-{}", ResourceId::new().0));
    let workspace = LocalWorkspace::open(&dir).unwrap();
    let project = workspace
        .create_project("p".to_string(), ProjectKind::Meg, String::new())
        .unwrap();
    let dataset = workspace
        .register_dataset(dataset(project.id.clone(), "d"))
        .unwrap();

    let run = workspace
        .create_run(
            dataset.id.clone(),
            "meg_pipeline".to_string(),
            "{}".to_string(),
            None,
        )
        .unwrap();
    assert_eq!(run.status, "running");

    let completed = workspace
        .complete_run(run.id.clone(), "{\"ok\":true}".to_string())
        .unwrap();
    assert_eq!(completed.status, "succeeded");
    assert_eq!(completed.result_json.as_deref(), Some("{\"ok\":true}"));

    let artifact = workspace
        .add_artifact(
            run.id.clone(),
            "result".to_string(),
            "application/json".to_string(),
            b"{\"ok\":true}",
        )
        .unwrap();
    assert_eq!(artifact.checksum_sha256.len(), 64);
    assert!(std::path::Path::new(&artifact.path).exists());

    drop(workspace);
    let reopened = LocalWorkspace::open(&dir).unwrap();
    let runs = reopened.list_runs(dataset.id.clone()).unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].status, "succeeded");
    let artifacts = reopened.list_artifacts(run.id.clone()).unwrap();
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].name, "result");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn failed_run_records_error() {
    let dir = std::env::temp_dir().join(format!("neuro-store-test-{}", ResourceId::new().0));
    let workspace = LocalWorkspace::open(&dir).unwrap();
    let project = workspace
        .create_project("p".to_string(), ProjectKind::Eeg, String::new())
        .unwrap();
    let dataset = workspace
        .register_dataset(dataset(project.id.clone(), "d"))
        .unwrap();
    let run = workspace
        .create_run(
            dataset.id.clone(),
            "eeg_stats".to_string(),
            "{}".to_string(),
            None,
        )
        .unwrap();
    let failed = workspace
        .fail_run(run.id.clone(), "boom".to_string())
        .unwrap();
    assert_eq!(failed.status, "failed");
    assert_eq!(failed.error.as_deref(), Some("boom"));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn dataset_versioning_and_experiments() {
    let dir = std::env::temp_dir().join(format!("neuro-store-test-{}", ResourceId::new().0));
    let workspace = LocalWorkspace::open(&dir).unwrap();
    let project = workspace
        .create_project("p".to_string(), ProjectKind::Meg, String::new())
        .unwrap();

    let first = workspace
        .register_dataset(dataset(project.id.clone(), "d"))
        .unwrap();
    assert_eq!(first.version, 1);
    // Same path + checksum is idempotent.
    let again = workspace
        .register_dataset(dataset(project.id.clone(), "d"))
        .unwrap();
    assert_eq!(again.id, first.id);
    assert_eq!(again.version, 1);
    // Changed checksum bumps the version and keeps identity.
    let mut changed = dataset(project.id.clone(), "d");
    changed.checksum_sha256 = "def".to_string();
    let bumped = workspace.register_dataset(changed).unwrap();
    assert_eq!(bumped.id, first.id);
    assert_eq!(bumped.version, 2);
    assert_eq!(bumped.checksum_sha256, "def");

    let experiment = workspace
        .create_experiment(project.id.clone(), "exp".to_string(), String::new())
        .unwrap();
    let listed = workspace.list_experiments(project.id.clone()).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, experiment.id);

    let run = workspace
        .create_run(
            first.id.clone(),
            "meg_pipeline".to_string(),
            "{}".to_string(),
            Some(experiment.id.clone()),
        )
        .unwrap();
    assert_eq!(run.experiment_id, Some(experiment.id));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn cancel_run_transitions_running_to_cancelled() {
    let dir = std::env::temp_dir().join(format!("neuro-store-test-{}", ResourceId::new().0));
    let workspace = LocalWorkspace::open(&dir).unwrap();
    let project = workspace
        .create_project("p".to_string(), ProjectKind::Eeg, String::new())
        .unwrap();
    let dataset = workspace
        .register_dataset(dataset(project.id.clone(), "d"))
        .unwrap();
    let run = workspace
        .create_run(dataset.id.clone(), "p".to_string(), "{}".to_string(), None)
        .unwrap();
    let cancelled = workspace.cancel_run(run.id.clone()).unwrap();
    assert_eq!(cancelled.status, "cancelled");
    assert!(cancelled.completed_at.is_some());
    assert!(workspace.cancel_run(run.id.clone()).is_err());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn upload_session_resumes_across_reopen() {
    let dir = std::env::temp_dir().join(format!("neuro-store-test-{}", ResourceId::new().0));
    let workspace = LocalWorkspace::open(&dir).unwrap();
    let project = workspace
        .create_project("p".to_string(), ProjectKind::Eeg, String::new())
        .unwrap();
    let session = workspace
        .create_upload_session(project.id.clone(), "rec.npy".to_string(), 10, None)
        .unwrap();
    let mid = workspace
        .append_upload_chunk(session.id.clone(), b"hello")
        .unwrap();
    assert_eq!(mid.received_bytes, 5);
    drop(workspace);

    // Simulate a crash where the .part file received more bytes than the
    // workspace recorded: append 2 extra bytes directly to the part file.
    let part = dir.join("uploads").join(format!("{}.part", session.id.0));
    std::fs::write(&part, b"helloXY").unwrap();

    let reopened = LocalWorkspace::open(&dir).unwrap();
    let sessions = reopened.list_upload_sessions(project.id.clone()).unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].status, "open");
    assert_eq!(sessions[0].received_bytes, 5);

    // Next append reconciles the on-disk size (7) before writing.
    let updated = reopened
        .append_upload_chunk(session.id.clone(), b"123")
        .unwrap();
    assert_eq!(updated.received_bytes, 10);

    let (done, dataset) = reopened
        .finalize_upload_session(session.id.clone())
        .unwrap();
    assert_eq!(done.status, "completed");
    assert_eq!(dataset.size_bytes, 10);
    let bytes = std::fs::read(&dataset.path).unwrap();
    assert_eq!(&bytes, b"helloXY123");

    // Aborting a closed session is rejected.
    assert!(reopened.abort_upload_session(session.id.clone()).is_err());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn abort_upload_session_discards_open_upload() {
    let dir = std::env::temp_dir().join(format!("neuro-store-test-{}", ResourceId::new().0));
    let workspace = LocalWorkspace::open(&dir).unwrap();
    let project = workspace
        .create_project("p".to_string(), ProjectKind::Eeg, String::new())
        .unwrap();
    let session = workspace
        .create_upload_session(project.id.clone(), "rec.npy".to_string(), 10, None)
        .unwrap();
    workspace
        .append_upload_chunk(session.id.clone(), b"hello")
        .unwrap();
    let aborted = workspace.abort_upload_session(session.id.clone()).unwrap();
    assert_eq!(aborted.status, "aborted");
    assert!(!dir
        .join("uploads")
        .join(format!("{}.part", session.id.0))
        .exists());
    assert!(workspace
        .append_upload_chunk(session.id.clone(), b"x")
        .is_err());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn upload_session_verifies_expected_checksum() {
    let dir = std::env::temp_dir().join(format!("neuro-store-test-{}", ResourceId::new().0));
    let workspace = LocalWorkspace::open(&dir).unwrap();
    let project = workspace
        .create_project("p".to_string(), ProjectKind::Eeg, String::new())
        .unwrap();

    // Bad digest format is rejected up front.
    assert!(workspace
        .create_upload_session(
            project.id.clone(),
            "bad.npy".to_string(),
            5,
            Some("nothex".to_string())
        )
        .is_err());

    let good = neuro_core::sha256_hex(b"hello");
    let session = workspace
        .create_upload_session(
            project.id.clone(),
            "good.npy".to_string(),
            5,
            Some(good.clone()),
        )
        .unwrap();
    workspace
        .append_upload_chunk(session.id.clone(), b"hello")
        .unwrap();
    let report = workspace.verify_upload_session(session.id.clone()).unwrap();
    assert_eq!(report["size_matches"], true);
    assert_eq!(report["expected_match"], true);
    let (done, _) = workspace
        .finalize_upload_session(session.id.clone())
        .unwrap();
    assert_eq!(done.status, "completed");

    // A mismatched expected digest blocks finalization and leaves the
    // session open so the upload can be retried or aborted.
    let wrong = neuro_core::sha256_hex(b"world");
    let session = workspace
        .create_upload_session(project.id.clone(), "bad.npy".to_string(), 5, Some(wrong))
        .unwrap();
    workspace
        .append_upload_chunk(session.id.clone(), b"hello")
        .unwrap();
    assert!(workspace
        .finalize_upload_session(session.id.clone())
        .is_err());
    let sessions = workspace.list_upload_sessions(project.id.clone()).unwrap();
    let still_open = sessions.iter().find(|s| s.id == session.id).unwrap();
    assert_eq!(still_open.status, "open");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn upload_chunk_rejects_offset_mismatch() {
    let dir = std::env::temp_dir().join(format!("neuro-store-test-{}", ResourceId::new().0));
    let workspace = LocalWorkspace::open(&dir).unwrap();
    let project = workspace
        .create_project("p".to_string(), ProjectKind::Eeg, String::new())
        .unwrap();
    let session = workspace
        .create_upload_session(project.id.clone(), "rec.npy".to_string(), 10, None)
        .unwrap();
    workspace
        .append_upload_chunk_at(session.id.clone(), Some(0), b"hello")
        .unwrap();
    // A retried chunk claiming the wrong offset is rejected instead of
    // duplicating bytes.
    assert!(workspace
        .append_upload_chunk_at(session.id.clone(), Some(0), b"hello")
        .is_err());
    let updated = workspace
        .append_upload_chunk_at(session.id.clone(), Some(5), b"world")
        .unwrap();
    assert_eq!(updated.received_bytes, 10);
    std::fs::remove_dir_all(&dir).ok();
}
