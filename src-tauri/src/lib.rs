pub mod backend_config;
pub mod paper;
pub mod storage;
pub mod tool;

use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use tauri::Manager;

use backend_config::BackendConfig;
use paper::LocalPdfImporter;

struct AppState {
    config: Arc<Mutex<BackendConfig>>,
}

impl AppState {
    fn new(config: BackendConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
        }
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Tauri adapter for local-PDF ingestion. Storage and copy behavior live in
/// `paper::LocalPdfImporter`, so the UI has no filesystem implementation.
#[tauri::command]
fn import_local_pdf(
    state: tauri::State<'_, AppState>,
    source_path: String,
) -> Result<paper::LocalPdfImportReceipt, String> {
    let config = state
        .config
        .lock()
        .map_err(|_| "Storage configuration is unavailable.".to_string())?;
    LocalPdfImporter::new(config.storage.clone())
        .import_file(Path::new(&source_path))
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_storage_location(
    state: tauri::State<'_, AppState>,
) -> Result<backend_config::StorageLocation, String> {
    state
        .config
        .lock()
        .map_err(|_| "Storage configuration is unavailable.".to_string())
        .map(|config| config.storage_location())
}

#[tauri::command]
async fn migrate_storage_location(
    state: tauri::State<'_, AppState>,
    destination: String,
) -> Result<backend_config::StorageMigrationReceipt, String> {
    let config = Arc::clone(&state.config);
    tauri::async_runtime::spawn_blocking(move || {
        config
            .lock()
            .map_err(|_| "Storage configuration is unavailable.".to_string())?
            .migrate_storage_to(PathBuf::from(destination))
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|_| "Storage migration did not complete.".to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let settings_dir = app.path().app_local_data_dir()?;
            app.manage(AppState::new(BackendConfig::load(data_dir, settings_dir)?));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            import_local_pdf,
            get_storage_location,
            migrate_storage_location
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
