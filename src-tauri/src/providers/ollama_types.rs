use serde::{Deserialize, Serialize};

// --- Request types ---

#[derive(Debug, Clone, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ChatOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatOptions {
    pub temperature: f32,
    pub num_predict: u32,
}

#[derive(Debug, Serialize)]
pub struct PullRequest {
    pub model: String,
    pub stream: bool,
}

#[derive(Debug, Serialize)]
pub struct DeleteRequest {
    pub model: String,
}

// --- Response types ---

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub message: ChatMessageResponse,
    pub done: bool,
    #[serde(default)]
    pub total_duration: Option<u64>,
    #[serde(default)]
    pub eval_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ChatMessageResponse {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct TagsResponse {
    pub models: Vec<ModelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub size: u64,
    #[serde(default)]
    pub digest: Option<String>,
    #[serde(default)]
    pub details: Option<ModelDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDetails {
    #[serde(default)]
    pub parameter_size: Option<String>,
    #[serde(default)]
    pub quantization_level: Option<String>,
    #[serde(default)]
    pub family: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullProgress {
    pub status: String,
    #[serde(default)]
    pub total: Option<u64>,
    #[serde(default)]
    pub completed: Option<u64>,
}
