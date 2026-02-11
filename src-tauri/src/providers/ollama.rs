use super::ollama_error::OllamaError;
use super::ollama_types::*;
use super::traits::LlmProvider;
use futures_util::StreamExt;

/// Ollama HTTP API provider (localhost:11434).
pub struct OllamaProvider {
    client: reqwest::Client,
    base_url: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(base_url: &str, model: &str) -> Self {
        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(5))
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            base_url: base_url.to_string(),
            model: model.to_string(),
        }
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    /// List all downloaded Ollama models.
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>, OllamaError> {
        let resp: TagsResponse = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(resp.models)
    }

    /// Pull (download) a model with streaming progress callback.
    pub async fn pull_model(
        &self,
        model_name: &str,
        on_progress: Box<dyn Fn(&PullProgress) + Send>,
    ) -> Result<(), OllamaError> {
        let request = PullRequest {
            model: model_name.to_string(),
            stream: true,
        };

        let response = self
            .client
            .post(format!("{}/api/pull", self.base_url))
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        let mut stream = response.bytes_stream();
        let mut buffer = Vec::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(OllamaError::Http)?;
            buffer.extend_from_slice(&chunk);

            while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                let line_bytes: Vec<u8> = buffer.drain(..=pos).collect();
                let line = String::from_utf8_lossy(&line_bytes);
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if let Ok(progress) = serde_json::from_str::<PullProgress>(trimmed) {
                    on_progress(&progress);
                    if progress.status == "success" {
                        return Ok(());
                    }
                }
            }
        }
        Err(OllamaError::StreamInterrupted)
    }

    /// Delete a model from Ollama.
    pub async fn delete_model(&self, model_name: &str) -> Result<(), OllamaError> {
        let resp = self
            .client
            .delete(format!("{}/api/delete", self.base_url))
            .json(&DeleteRequest {
                model: model_name.to_string(),
            })
            .send()
            .await?;

        if resp.status().as_u16() == 404 {
            return Err(OllamaError::ModelNotFound(model_name.to_string()));
        }
        resp.error_for_status()?;
        Ok(())
    }

    /// Core NDJSON streaming chat logic.
    async fn stream_ndjson_chat(
        &self,
        messages: Vec<ChatMessage>,
        on_chunk: Box<dyn Fn(&str) + Send>,
    ) -> Result<String, OllamaError> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            stream: true,
            options: Some(ChatOptions {
                temperature: 0.3,
                num_predict: 1024,
            }),
            keep_alive: Some("5m".to_string()),
        };

        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&request)
            .send()
            .await?;

        if response.status().as_u16() == 404 {
            return Err(OllamaError::ModelNotFound(self.model.clone()));
        }
        let response = response.error_for_status()?;

        let mut stream = response.bytes_stream();
        let mut buffer = Vec::new();
        let mut accumulated = String::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(OllamaError::Http)?;
            buffer.extend_from_slice(&chunk);

            // Process complete NDJSON lines
            while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
                let line_bytes: Vec<u8> = buffer.drain(..=newline_pos).collect();
                let line = String::from_utf8_lossy(&line_bytes);
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                match serde_json::from_str::<ChatResponse>(trimmed) {
                    Ok(parsed) => {
                        let content = &parsed.message.content;
                        if !content.is_empty() {
                            on_chunk(content);
                            accumulated.push_str(content);
                        }
                        if parsed.done {
                            return Ok(accumulated);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse NDJSON line: {}", e);
                    }
                }
            }
        }

        // Stream ended without done=true
        if accumulated.is_empty() {
            Err(OllamaError::StreamInterrupted)
        } else {
            Ok(accumulated)
        }
    }
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new("http://localhost:11434", "qwen2.5:3b")
    }
}

impl LlmProvider for OllamaProvider {
    async fn chat(&self, messages: Vec<ChatMessage>) -> anyhow::Result<String> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            stream: false,
            options: Some(ChatOptions {
                temperature: 0.3,
                num_predict: 1024,
            }),
            keep_alive: Some("5m".to_string()),
        };

        let response: ChatResponse = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&request)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(response.message.content)
    }

    async fn chat_streaming(
        &self,
        messages: Vec<ChatMessage>,
        on_chunk: Box<dyn Fn(&str) + Send>,
    ) -> anyhow::Result<String> {
        self.stream_ndjson_chat(messages, on_chunk)
            .await
            .map_err(Into::into)
    }

    async fn health_check(&self) -> anyhow::Result<bool> {
        match self.client.get(&self.base_url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    fn name(&self) -> &str {
        "Ollama"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_ollama_provider() {
        let provider = OllamaProvider::default();
        assert_eq!(provider.name(), "Ollama");
        assert_eq!(provider.base_url, "http://localhost:11434");
        assert_eq!(provider.model, "qwen2.5:3b");
    }

    #[test]
    fn creates_custom_provider() {
        let provider = OllamaProvider::new("http://remote:11434", "gemma2:2b");
        assert_eq!(provider.base_url, "http://remote:11434");
        assert_eq!(provider.model, "gemma2:2b");
    }
}
