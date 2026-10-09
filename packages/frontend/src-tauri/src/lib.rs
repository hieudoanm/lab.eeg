mod server;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_log::Builder::default()
      .level(log::LevelFilter::Info)
      .build())
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_notification::init())
    .setup(|app| {
      let app_handle = app.handle().clone();
      
      // Start the Next.js standalone server in the background
      #[cfg(not(debug_assertions))]
      {
        tauri::async_runtime::spawn(async move {
          if let Err(e) = crate::server::start_server(app_handle).await {
            log::error!("Failed to start Next.js server: {}", e);
          }
        });
      }
      
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while building tauri application");
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_build() {
    assert!(true);
  }
}