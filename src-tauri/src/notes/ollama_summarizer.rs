use crate::providers::{ChatMessage, OllamaProvider};
use std::sync::Arc;

use super::note_engine::TranscriptSegment;
use super::note_types::IncrementalNotesResponse;
use super::prompt_templates::{build_user_prompt, get_json_schema, SYSTEM_PROMPT};

/// Ollama-based summarizer for incremental note generation.
pub struct OllamaSummarizer {
    provider: Arc<OllamaProvider>,
    base_url: String,
}

impl OllamaSummarizer {
    pub fn new(provider: Arc<OllamaProvider>, base_url: String) -> Self {
        Self { provider, base_url }
    }

    /// Generate incremental notes from recent transcript segments.
    /// Returns only NEW notes not present in existing_notes.
    pub async fn generate_incremental_notes(
        &self,
        existing_notes: &IncrementalNotesResponse,
        recent_segments: &[TranscriptSegment],
    ) -> anyhow::Result<IncrementalNotesResponse> {
        if recent_segments.is_empty() {
            return Ok(IncrementalNotesResponse::empty());
        }

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: SYSTEM_PROMPT.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: build_user_prompt(existing_notes, recent_segments),
            },
        ];

        // Call Ollama with JSON schema (non-streaming)
        let response = self.call_ollama_with_schema(messages).await?;

        // Parse JSON response
        let notes: IncrementalNotesResponse = serde_json::from_str(&response)
            .map_err(|e| anyhow::anyhow!("JSON parse error: {}", e))?;

        Ok(notes)
    }

    async fn call_ollama_with_schema(
        &self,
        messages: Vec<ChatMessage>,
    ) -> anyhow::Result<String> {
        // Use Ollama /api/chat endpoint with format field for JSON schema
        let url = format!("{}/api/chat", self.base_url);
        let schema = get_json_schema();

        let request = serde_json::json!({
            "model": self.provider.model(),
            "messages": messages,
            "temperature": 0.0, // Deterministic for structured output
            "stream": false,
            "format": schema,
        });

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .json(&request)
            .timeout(std::time::Duration::from_secs(60))
            .send()
            .await?
            .error_for_status()?;

        let json: serde_json::Value = response.json().await?;
        let content = json["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content in response"))?;

        Ok(content.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_ollama_summarizer() {
        let provider = Arc::new(OllamaProvider::default());
        let summarizer = OllamaSummarizer::new(provider, "http://localhost:11434".to_string());
        assert_eq!(summarizer.base_url, "http://localhost:11434");
    }
}
