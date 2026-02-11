/// Get the current app version.
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Health check for the backend.
#[tauri::command]
pub fn health_check() -> String {
    "ok".to_string()
}
