mod commands;
mod workspace;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let workspace = neuro_store::LocalWorkspace::open(data_dir.join("workspace"))
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            app.manage(workspace::WorkspaceState(std::sync::Mutex::new(workspace)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::inspect_dataset,
            commands::run_eeg_analysis,
            commands::submit_eeg_analysis,
            commands::run_eeg_pipeline,
            commands::submit_eeg_pipeline,
            commands::export_eeg_edf,
            commands::export_meg_npy,
            commands::export_meg_csv,
            commands::export_meg_edf,
            commands::export_meg_fif,
            commands::export_meg_bids,
            commands::save_pipeline_preset,
            commands::list_pipeline_presets,
            commands::delete_pipeline_preset,
            commands::convert_dicom_to_nifti,
            commands::convert_dicom_series_to_nifti,
            commands::export_run_artifact,
            commands::get_eeg_signal_window,
            commands::inspect_meg_recording,
            commands::get_meg_window,
            commands::inspect_mri_volume,
            commands::get_mri_slice,
            commands::get_mri_projection,
            commands::normalize_mri_volume,
            commands::run_mri_operation,
            commands::submit_mri_operation,
            commands::get_mri_statistics,
            commands::get_mri_histogram,
            commands::run_meg_pipeline,
            commands::submit_meg_pipeline,
            commands::submit_meg_batch,
            commands::generate_meg_sample,
            commands::generate_eeg_sample,
            commands::validate_meg_file_set,
            commands::validate_eeg_file_set,
            workspace::list_projects,
            workspace::create_project,
            workspace::list_datasets,
            workspace::create_upload_session,
            workspace::append_upload_chunk,
            workspace::list_upload_sessions,
            workspace::abort_upload_session,
            workspace::finalize_upload_session,
            workspace::verify_upload_session,
            workspace::register_dataset,
            workspace::list_analysis_runs,
            workspace::get_analysis_run,
            workspace::list_artifacts,
            workspace::create_experiment,
            workspace::list_experiments,
            workspace::cancel_analysis_run,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Neuro Lab");
}
