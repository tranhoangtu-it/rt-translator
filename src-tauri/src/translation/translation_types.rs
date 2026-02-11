use serde::{Deserialize, Serialize};

/// Result of a translation operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    pub source_text: String,
    pub translated_text: String,
    pub source_lang: String,
    pub target_lang: String,
    pub segment_id: String,
}

/// Emitted per streaming token chunk during translation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationUpdatePayload {
    /// ID linking to the source STT segment
    pub segment_id: String,
    /// Accumulated translation so far (grows with each chunk)
    pub text: String,
    /// Target language code
    pub target_lang: String,
    /// false while streaming, true on final chunk
    pub is_final: bool,
}

/// Emitted when translation fails.
#[derive(Debug, Clone, Serialize)]
pub struct TranslationErrorPayload {
    pub segment_id: String,
    pub error: String,
}
