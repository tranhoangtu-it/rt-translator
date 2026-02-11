use std::sync::Arc;

use tauri::Emitter;

use crate::providers::{ChatMessage, LlmProvider, OllamaProvider};

use super::translation_types::{
    TranslationErrorPayload, TranslationResult, TranslationUpdatePayload,
};

/// Manages text translation via Ollama LLM provider.
/// Holds a shared provider reference and target language config.
pub struct TranslationPipeline {
    provider: Arc<OllamaProvider>,
    target_lang: String,
    system_prompt: String,
}

impl TranslationPipeline {
    pub fn new(provider: Arc<OllamaProvider>, target_lang: &str) -> Self {
        let system_prompt = build_system_prompt(target_lang);
        Self {
            provider,
            target_lang: target_lang.to_string(),
            system_prompt,
        }
    }

    /// Translate a single text segment with streaming events.
    /// Emits `translation-update` events for each partial chunk,
    /// and a final event with `is_final: true`.
    pub async fn translate(
        &self,
        app: &tauri::AppHandle,
        segment_id: &str,
        text: &str,
    ) -> anyhow::Result<TranslationResult> {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: self.system_prompt.clone(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: text.to_string(),
            },
        ];

        // Build streaming callback that emits partial translation events
        let app_clone = app.clone();
        let seg_id = segment_id.to_string();
        let tgt = self.target_lang.clone();
        let accumulated = Arc::new(std::sync::Mutex::new(String::new()));
        let acc = accumulated.clone();

        let on_chunk: Box<dyn Fn(&str) + Send> = Box::new(move |chunk: &str| {
            if let Ok(mut guard) = acc.lock() {
                guard.push_str(chunk);
                let current = guard.clone();
                let _ = app_clone.emit(
                    "translation-update",
                    TranslationUpdatePayload {
                        segment_id: seg_id.clone(),
                        text: current,
                        target_lang: tgt.clone(),
                        is_final: false,
                    },
                );
            }
        });

        // Run streaming translation
        let result: anyhow::Result<String> =
            self.provider.chat_streaming(messages, on_chunk).await;

        match result {
            Ok(full_text) => {
                // Emit final translation
                let _ = app.emit(
                    "translation-update",
                    TranslationUpdatePayload {
                        segment_id: segment_id.to_string(),
                        text: full_text.clone(),
                        target_lang: self.target_lang.clone(),
                        is_final: true,
                    },
                );

                Ok(TranslationResult {
                    source_text: text.to_string(),
                    translated_text: full_text,
                    source_lang: "auto".to_string(),
                    target_lang: self.target_lang.clone(),
                    segment_id: segment_id.to_string(),
                })
            }
            Err(e) => {
                let _ = app.emit(
                    "translation-error",
                    TranslationErrorPayload {
                        segment_id: segment_id.to_string(),
                        error: e.to_string(),
                    },
                );
                Err(e)
            }
        }
    }

    /// Update target language at runtime.
    pub fn set_target_lang(&mut self, lang: &str) {
        self.target_lang = lang.to_string();
        self.system_prompt = build_system_prompt(lang);
    }

    pub fn target_lang(&self) -> &str {
        &self.target_lang
    }
}

fn build_system_prompt(lang: &str) -> String {
    format!(
        "Translate to {}. Output only the translation. \
         No explanations. Preserve formatting.",
        lang
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_pipeline() {
        let provider = Arc::new(OllamaProvider::default());
        let pipeline = TranslationPipeline::new(provider, "vi");
        assert_eq!(pipeline.target_lang(), "vi");
        assert!(pipeline.system_prompt.contains("vi"));
    }

    #[test]
    fn updates_target_lang() {
        let provider = Arc::new(OllamaProvider::default());
        let mut pipeline = TranslationPipeline::new(provider, "vi");
        pipeline.set_target_lang("ja");
        assert_eq!(pipeline.target_lang(), "ja");
        assert!(pipeline.system_prompt.contains("ja"));
    }
}
