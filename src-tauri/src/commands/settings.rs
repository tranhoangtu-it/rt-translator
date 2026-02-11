use serde::Serialize;

/// App settings returned to frontend.
#[derive(Debug, Clone, Serialize)]
pub struct AppSettings {
    pub ollama_url: String,
    pub log_level: String,
}

/// Get current app settings.
#[tauri::command]
pub fn get_settings() -> AppSettings {
    AppSettings {
        ollama_url: std::env::var("RT_OLLAMA_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string()),
        log_level: std::env::var("RT_LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
    }
}
