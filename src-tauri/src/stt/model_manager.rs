use anyhow::{Context, Result};
use std::path::PathBuf;
use tauri::Emitter;

/// Default whisper model for STT inference.
pub const DEFAULT_MODEL: &str = "ggml-base.bin";

/// Hugging Face CDN base URL for whisper.cpp models.
const HF_MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";

/// Manages whisper model files (download, path resolution, status check).
pub struct ModelManager {
    models_dir: PathBuf,
}

impl ModelManager {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let models_dir = app_data_dir.join("models").join("whisper");
        Self { models_dir }
    }

    /// Full path for a model file.
    pub fn model_path(&self, model_name: &str) -> PathBuf {
        self.models_dir.join(model_name)
    }

    /// Check if model file exists and is non-empty.
    pub fn is_model_available(&self, model_name: &str) -> bool {
        let path = self.model_path(model_name);
        path.exists() && path.metadata().map(|m| m.len() > 0).unwrap_or(false)
    }

    /// File size in MB, or 0 if missing.
    pub fn model_size_mb(&self, model_name: &str) -> f64 {
        let path = self.model_path(model_name);
        path.metadata()
            .map(|m| m.len() as f64 / (1024.0 * 1024.0))
            .unwrap_or(0.0)
    }

    /// Download model from Hugging Face CDN with progress events.
    pub async fn download_model(
        &self,
        model_name: &str,
        app: tauri::AppHandle,
    ) -> Result<PathBuf> {
        use futures_util::StreamExt;

        // Ensure directory exists
        std::fs::create_dir_all(&self.models_dir)
            .context("Failed to create models directory")?;

        let url = format!("{}/{}", HF_MODEL_URL, model_name);
        let dest = self.model_path(model_name);
        let tmp = dest.with_extension("bin.tmp");

        tracing::info!("Downloading model from: {}", url);

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .send()
            .await
            .context("Failed to start download")?
            .error_for_status()
            .context("Download returned error status")?;

        let total = response.content_length().unwrap_or(0);
        let mut stream = response.bytes_stream();
        let mut file = std::fs::File::create(&tmp)
            .context("Failed to create temp file")?;
        let mut downloaded: u64 = 0;

        use std::io::Write;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Download stream error")?;
            file.write_all(&chunk)
                .context("Failed to write chunk")?;
            downloaded += chunk.len() as u64;

            // Emit progress every ~1MB
            if downloaded % (1024 * 1024) < chunk.len() as u64 {
                let _ = app.emit(
                    "model-download-progress",
                    serde_json::json!({
                        "model": model_name,
                        "downloaded": downloaded,
                        "total": total,
                    }),
                );
            }
        }

        // Rename temp to final
        std::fs::rename(&tmp, &dest).context("Failed to finalize model file")?;
        tracing::info!("Model downloaded: {:?} ({:.1} MB)", dest, downloaded as f64 / 1_048_576.0);

        Ok(dest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_path_resolution() {
        let mgr = ModelManager::new(PathBuf::from("/tmp/app"));
        let path = mgr.model_path("ggml-base.bin");
        assert!(path.ends_with("models/whisper/ggml-base.bin"));
    }

    #[test]
    fn missing_model_not_available() {
        let mgr = ModelManager::new(PathBuf::from("/tmp/nonexistent"));
        assert!(!mgr.is_model_available("ggml-base.bin"));
    }
}
