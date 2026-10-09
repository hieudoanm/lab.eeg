use std::path::PathBuf;
use std::sync::Arc;
use axum::{
  body::Body,
  extract::{Path, State},
  http::{header, StatusCode},
  response::{IntoResponse, Response},
  Router,
};
use mime_guess::from_path;
use tauri::{Emitter, Manager};
use tokio::fs;
use tower_http::services::ServeDir;

#[derive(Clone)]
struct AppState {
  frontend_dir: PathBuf,
}

pub async fn start_server(app_handle: tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
  let resource_dir = app_handle
    .path()
    .resource_dir()
    .expect("Failed to get resource directory");

  // Next.js standalone output is at resources/.next/standalone
  let frontend_dir = resource_dir.join(".next/standalone");
  
  // Also check for the public directory
  let public_dir = resource_dir.join(".next/standalone/public");

  let state = Arc::new(AppState {
    frontend_dir: frontend_dir.clone(),
  });

  // Build the router
  let app = Router::new()
    // Serve static files from .next/standalone/public
    .nest_service("/_next/static", ServeDir::new(public_dir.join("_next/static")))
    .nest_service("/static", ServeDir::new(public_dir.join("static")))
    // Serve public assets
    .nest_service("/", ServeDir::new(public_dir.clone()))
    // Fallback to index.html for SPA routes
    .fallback(spa_fallback)
    .with_state(state);

  // Get port from environment or use random available port
  let port = std::env::var("TAURI_PORT")
    .ok()
    .and_then(|p| p.parse().ok())
    .unwrap_or(0);

  let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
  let actual_port = listener.local_addr()?.port();
  
  // Store the port for the frontend to know where to connect
  app_handle.emit("server-port", actual_port).ok();
  
  log::info!("Next.js standalone server running on port {}", actual_port);
  
  axum::serve(listener, app).await?;
  Ok(())
}

async fn spa_fallback(
  State(state): State<Arc<AppState>>,
  Path(path): Path<String>,
) -> Response {
  // Try to serve the file from the public directory
  let file_path = state.frontend_dir.join("public").join(&path);
  
  if file_path.exists() && file_path.is_file() {
    return serve_file(file_path).await;
  }

  // Try with .html extension
  let html_path = file_path.with_extension("html");
  if html_path.exists() && html_path.is_file() {
    return serve_file(html_path).await;
  }

  // Try index.html for SPA routes
  let index_path = state.frontend_dir.join("public/index.html");
  if index_path.exists() {
    return serve_file(index_path).await;
  }

  // Fallback
  (StatusCode::NOT_FOUND, "Not found").into_response()
}

async fn serve_file(path: PathBuf) -> Response {
  match fs::read(&path).await {
    Ok(content) => {
      let mime = from_path(&path).first_or_octet_stream();
      Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime.to_string())
        .body(Body::from(content))
        .unwrap()
    }
    Err(_) => (StatusCode::NOT_FOUND, "File not found").into_response(),
  }
}