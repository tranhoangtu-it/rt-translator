use thiserror::Error;

#[derive(Error, Debug)]
pub enum OllamaError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Streaming interrupted")]
    StreamInterrupted,
}
