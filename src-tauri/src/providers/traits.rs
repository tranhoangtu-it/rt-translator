use std::future::Future;

use super::ollama_types::ChatMessage;

/// Trait for LLM provider implementations.
/// Uses RPITIT (return position impl trait in trait) — no async_trait needed.
pub trait LlmProvider: Send + Sync {
    /// Non-streaming chat completion. Returns full response text.
    fn chat(
        &self,
        messages: Vec<ChatMessage>,
    ) -> impl Future<Output = anyhow::Result<String>> + Send;

    /// Streaming chat — calls `on_chunk` for each token chunk.
    /// Returns full accumulated response.
    fn chat_streaming(
        &self,
        messages: Vec<ChatMessage>,
        on_chunk: Box<dyn Fn(&str) + Send>,
    ) -> impl Future<Output = anyhow::Result<String>> + Send;

    /// Check if the provider is available/healthy.
    fn health_check(&self) -> impl Future<Output = anyhow::Result<bool>> + Send;

    /// Get the provider display name.
    fn name(&self) -> &str;
}
