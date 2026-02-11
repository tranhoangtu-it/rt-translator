use serde::Serialize;
use std::sync::Arc;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// Result of a speech-to-text transcription.
#[derive(Debug, Clone, Serialize)]
pub struct TranscriptSegment {
    pub text: String,
    pub lang: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub is_final: bool,
}

/// Whisper-based speech-to-text engine.
/// Loads a ggml model file and runs inference via whisper.cpp FFI.
pub struct SttEngine {
    ctx: Arc<WhisperContext>,
    language: Option<String>,
}

impl SttEngine {
    /// Load a whisper model from the given file path.
    pub fn new(model_path: &str, language: Option<String>) -> Result<Self, String> {
        let ctx = WhisperContext::new_with_params(
            model_path,
            WhisperContextParameters::default(),
        )
        .map_err(|e| format!("Failed to load whisper model: {:?}", e))?;

        tracing::info!("Whisper model loaded: {}", model_path);

        Ok(Self {
            ctx: Arc::new(ctx),
            language,
        })
    }

    pub fn set_language(&mut self, lang: Option<String>) {
        self.language = lang;
    }

    /// Run transcription synchronously (call from dedicated thread, NOT tokio runtime).
    /// `audio` must be 16kHz mono f32. `base_time_ms` offsets segment timestamps.
    pub fn transcribe_sync(
        &self,
        audio: &[f32],
        base_time_ms: u64,
    ) -> Result<Vec<TranscriptSegment>, String> {
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        if let Some(ref lang) = self.language {
            params.set_language(Some(lang));
        }
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_no_timestamps(false);

        let mut state = self
            .ctx
            .create_state()
            .map_err(|e| format!("Failed to create whisper state: {:?}", e))?;

        state
            .full(params, audio)
            .map_err(|e| format!("Whisper inference failed: {:?}", e))?;

        let n_segments = state.full_n_segments();
        let lang_str = self
            .language
            .clone()
            .unwrap_or_else(|| "en".to_string());

        let mut segments = Vec::new();
        for i in 0..n_segments {
            let seg = state
                .get_segment(i)
                .ok_or_else(|| format!("Segment {} out of bounds", i))?;

            let text = seg
                .to_str_lossy()
                .map_err(|e| format!("Failed to get segment text: {:?}", e))?;

            let t0 = seg.start_timestamp().max(0) as u64;
            let t1 = seg.end_timestamp().max(0) as u64;

            segments.push(TranscriptSegment {
                text: text.trim().to_string(),
                lang: lang_str.clone(),
                start_ms: base_time_ms + t0 * 10, // centiseconds -> ms
                end_ms: base_time_ms + t1 * 10,
                is_final: true,
            });
        }

        Ok(segments)
    }

    /// Async wrapper: runs transcribe_sync on a blocking thread.
    pub async fn transcribe(
        &self,
        audio: Vec<f32>,
        base_time_ms: u64,
    ) -> Result<Vec<TranscriptSegment>, String> {
        let ctx = self.ctx.clone();
        let language = self.language.clone();

        tokio::task::spawn_blocking(move || {
            let engine_ref = SttEngine {
                ctx,
                language,
            };
            engine_ref.transcribe_sync(&audio, base_time_ms)
        })
        .await
        .map_err(|e| format!("Task join error: {:?}", e))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_creation_fails_with_bad_path() {
        let result = SttEngine::new("/nonexistent/model.bin", None);
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.contains("Failed to load"), "got: {}", err);
    }

    #[test]
    fn transcript_segment_serialization() {
        let seg = TranscriptSegment {
            text: "Hello world".to_string(),
            lang: "en".to_string(),
            start_ms: 1000,
            end_ms: 2000,
            is_final: true,
        };
        let json = serde_json::to_string(&seg).unwrap();
        assert!(json.contains("\"text\":\"Hello world\""));
        assert!(json.contains("\"start_ms\":1000"));
        assert!(json.contains("\"is_final\":true"));
    }
}
