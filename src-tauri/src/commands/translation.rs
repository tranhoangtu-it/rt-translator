use std::sync::Arc;

use tauri::{Emitter, State};
use tokio::sync::Semaphore;

use crate::commands::SttState;
use crate::providers::{ModelInfo, OllamaProvider};
use crate::translation::TranslationPipeline;

/// Application state for translation. Managed by Tauri.
/// Provider is shared via Arc across all translation tasks.
/// Semaphore caps concurrent Ollama requests to avoid GPU OOM.
pub struct TranslationState {
    pub provider: Arc<OllamaProvider>,
    pub semaphore: Arc<Semaphore>,
}

impl TranslationState {
    pub fn new(ollama_url: &str, model: &str) -> Self {
        Self {
            provider: Arc::new(OllamaProvider::new(ollama_url, model)),
            semaphore: Arc::new(Semaphore::new(3)),
        }
    }
}

impl Default for TranslationState {
    fn default() -> Self {
        Self::new("http://localhost:11434", "qwen2.5:3b")
    }
}

/// Check if Ollama server is reachable.
#[tauri::command]
pub async fn ollama_health_check(
    state: State<'_, TranslationState>,
) -> Result<bool, String> {
    use crate::providers::LlmProvider;
    state
        .provider
        .health_check()
        .await
        .map_err(|e| e.to_string())
}

/// Translate text to multiple target languages in parallel with streaming events.
/// Spawns background tasks (one per lang), returns segment_id immediately.
#[tauri::command]
pub async fn translate_text(
    app: tauri::AppHandle,
    state: State<'_, TranslationState>,
    stt_state: State<'_, SttState>,
    text: String,
    target_langs: Vec<String>,
    segment_id: String,
) -> Result<String, String> {
    if text.trim().is_empty() {
        return Err("Empty text".to_string());
    }
    if target_langs.is_empty() {
        return Err("No target languages specified".to_string());
    }

    let provider = state.provider.clone();
    let semaphore = state.semaphore.clone();
    let seg_id = segment_id.clone();
    let transcript_db = stt_state.transcript_db.clone();
    let meeting_id = stt_state.meeting_id.clone();

    // Spawn fan-out task: one sub-task per target language
    tauri::async_runtime::spawn(async move {
        let mut join_set = tokio::task::JoinSet::new();

        for lang in target_langs {
            let provider = provider.clone();
            let semaphore = semaphore.clone();
            let app = app.clone();
            let seg_id = seg_id.clone();
            let text = text.clone();
            let transcript_db = transcript_db.clone();
            let meeting_id = meeting_id.clone();

            join_set.spawn(async move {
                // Acquire semaphore permit (blocks if max concurrent reached)
                let _permit = match semaphore.acquire().await {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::error!("Semaphore closed for lang {}: {}", lang, e);
                        return;
                    }
                };

                let pipeline = TranslationPipeline::new(provider, &lang);
                // 30s timeout prevents hung Ollama requests from holding permits
                let translate_result = tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    pipeline.translate(&app, &seg_id, &text),
                )
                .await;

                match translate_result {
                    Err(_) => {
                        tracing::error!(
                            "Translation timed out for segment {} lang {}",
                            seg_id, lang
                        );
                    }
                    Ok(Err(e)) => {
                        tracing::error!(
                            "Translation failed for segment {} lang {}: {}",
                            seg_id, lang, e
                        );
                    }
                    Ok(Ok(result)) => {
                        tracing::info!(
                            "Translation complete: {} -> {}",
                            result.source_lang,
                            result.target_lang,
                        );

                        // Save to normalized translations table
                        if let Ok(guard) = meeting_id.lock() {
                            if let Some(mid) = *guard {
                                if let Ok(Some(transcript_id)) =
                                    transcript_db.get_transcript_id_by_segment(mid, &seg_id)
                                {
                                    if let Err(e) = transcript_db.insert_translation(
                                        transcript_id,
                                        &lang,
                                        &result.translated_text,
                                    ) {
                                        tracing::error!("Failed to save translation: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }

        // Await all parallel tasks, log panics
        while let Some(result) = join_set.join_next().await {
            if let Err(e) = result {
                tracing::error!("Translation task panicked: {}", e);
            }
        }
    });

    Ok(segment_id)
}

/// List all downloaded Ollama models.
#[tauri::command]
pub async fn list_ollama_models(
    state: State<'_, TranslationState>,
) -> Result<Vec<ModelInfo>, String> {
    state
        .provider
        .list_models()
        .await
        .map_err(|e| e.to_string())
}

/// Pull (download) an Ollama model with progress events.
#[tauri::command]
pub async fn pull_ollama_model(
    app: tauri::AppHandle,
    state: State<'_, TranslationState>,
    model_name: String,
) -> Result<String, String> {
    let provider = state.provider.clone();
    let name = model_name.clone();

    tauri::async_runtime::spawn(async move {
        let on_progress: Box<dyn Fn(&crate::providers::PullProgress) + Send> =
            Box::new(move |progress| {
                let _ = app.emit("ollama-pull-progress", progress);
            });

        match provider.pull_model(&name, on_progress).await {
            Ok(()) => tracing::info!("Model pull complete: {}", name),
            Err(e) => tracing::error!("Model pull failed: {}", e),
        }
    });

    Ok(format!("Pulling {}", model_name))
}

/// Delete an Ollama model.
#[tauri::command]
pub async fn delete_ollama_model(
    state: State<'_, TranslationState>,
    model_name: String,
) -> Result<String, String> {
    state
        .provider
        .delete_model(&model_name)
        .await
        .map_err(|e| e.to_string())?;
    Ok(format!("Deleted {}", model_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_translation_state() {
        let state = TranslationState::default();
        assert_eq!(state.provider.model(), "qwen2.5:3b");
    }
}
