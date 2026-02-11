use crate::providers::OllamaProvider;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::note_types::IncrementalNotesResponse;
use super::ollama_summarizer::OllamaSummarizer;

/// Transcript segment for note generation.
#[derive(Debug, Clone)]
pub struct TranscriptSegment {
    pub text: String,
    pub timestamp_ms: i64,
    pub segment_id: String,
}

/// Configuration for note engine trigger conditions.
#[derive(Debug, Clone)]
pub struct NoteEngineConfig {
    pub update_interval_secs: u64, // Default: 120 (2 min)
    pub segment_threshold: usize,   // Default: 10
    pub min_segments: usize,        // Default: 3
}

impl Default for NoteEngineConfig {
    fn default() -> Self {
        Self {
            update_interval_secs: 120,
            segment_threshold: 10,
            min_segments: 3,
        }
    }
}

/// Orchestrates incremental note generation from live transcripts.
pub struct NoteEngine {
    summarizer: OllamaSummarizer,
    config: NoteEngineConfig,
    accumulated_notes: IncrementalNotesResponse,
    pending_segments: Vec<TranscriptSegment>,
    last_update: Instant,
}

impl NoteEngine {
    pub fn new(provider: Arc<OllamaProvider>, base_url: String, config: NoteEngineConfig) -> Self {
        Self {
            summarizer: OllamaSummarizer::new(provider, base_url),
            config,
            accumulated_notes: IncrementalNotesResponse::empty(),
            pending_segments: Vec::new(),
            last_update: Instant::now(),
        }
    }

    /// Add new transcript segment to pending queue.
    pub fn add_segment(&mut self, segment: TranscriptSegment) {
        self.pending_segments.push(segment);
    }

    /// Check if should trigger note update (hybrid: time OR segment count).
    pub fn should_update(&self) -> bool {
        if self.pending_segments.len() < self.config.min_segments {
            return false; // Quality gate: need at least N segments
        }

        let time_trigger =
            self.last_update.elapsed() >= Duration::from_secs(self.config.update_interval_secs);
        let segment_trigger = self.pending_segments.len() >= self.config.segment_threshold;

        time_trigger || segment_trigger
    }

    /// Generate incremental notes from pending segments.
    /// Returns new notes extracted (empty if nothing new).
    pub async fn update_notes(&mut self) -> anyhow::Result<IncrementalNotesResponse> {
        if self.pending_segments.is_empty() {
            return Ok(IncrementalNotesResponse::empty());
        }

        tracing::info!(
            "Generating incremental notes from {} segments",
            self.pending_segments.len()
        );

        let new_notes = self
            .summarizer
            .generate_incremental_notes(&self.accumulated_notes, &self.pending_segments)
            .await?;

        // Merge new notes into accumulated
        self.accumulated_notes.merge(new_notes.clone());

        // Clear pending segments and reset timer
        self.pending_segments.clear();
        self.last_update = Instant::now();

        Ok(new_notes)
    }

    /// Get all accumulated notes (for meeting memo generation).
    pub fn get_accumulated_notes(&self) -> &IncrementalNotesResponse {
        &self.accumulated_notes
    }

    /// Reset engine for new meeting.
    pub fn reset(&mut self) {
        self.accumulated_notes = IncrementalNotesResponse::empty();
        self.pending_segments.clear();
        self.last_update = Instant::now();
    }
}

pub type SharedNoteEngine = Arc<tokio::sync::Mutex<Option<NoteEngine>>>;

/// Thread-safe segment buffer: STT pipeline pushes, note loop drains.
/// Uses std::sync::Mutex (not tokio) so STT thread can push without async.
pub type SegmentBuffer = Arc<std::sync::Mutex<Vec<TranscriptSegment>>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_note_engine() {
        let provider = Arc::new(OllamaProvider::default());
        let config = NoteEngineConfig::default();
        let engine = NoteEngine::new(provider, "http://localhost:11434".to_string(), config);
        assert_eq!(engine.pending_segments.len(), 0);
        assert!(engine.accumulated_notes.is_empty());
    }

    #[test]
    fn adds_segments() {
        let provider = Arc::new(OllamaProvider::default());
        let config = NoteEngineConfig::default();
        let mut engine = NoteEngine::new(provider, "http://localhost:11434".to_string(), config);

        engine.add_segment(TranscriptSegment {
            text: "Test".to_string(),
            timestamp_ms: 1000,
            segment_id: "seg-1".to_string(),
        });

        assert_eq!(engine.pending_segments.len(), 1);
    }

    #[test]
    fn trigger_logic_respects_min_segments() {
        let provider = Arc::new(OllamaProvider::default());
        let config = NoteEngineConfig {
            min_segments: 3,
            ..Default::default()
        };
        let mut engine = NoteEngine::new(provider, "http://localhost:11434".to_string(), config);

        // Add 2 segments (below threshold)
        engine.add_segment(TranscriptSegment {
            text: "Test 1".to_string(),
            timestamp_ms: 1000,
            segment_id: "seg-1".to_string(),
        });
        engine.add_segment(TranscriptSegment {
            text: "Test 2".to_string(),
            timestamp_ms: 2000,
            segment_id: "seg-2".to_string(),
        });

        assert!(!engine.should_update()); // Not enough segments
    }

    #[test]
    fn trigger_logic_segment_threshold() {
        let provider = Arc::new(OllamaProvider::default());
        let config = NoteEngineConfig {
            min_segments: 3,
            segment_threshold: 5,
            ..Default::default()
        };
        let mut engine = NoteEngine::new(provider, "http://localhost:11434".to_string(), config);

        // Add exactly threshold segments
        for i in 0..5 {
            engine.add_segment(TranscriptSegment {
                text: format!("Test {}", i),
                timestamp_ms: (i * 1000) as i64,
                segment_id: format!("seg-{}", i),
            });
        }

        assert!(engine.should_update()); // Meets segment threshold
    }

    #[test]
    fn resets_engine() {
        let provider = Arc::new(OllamaProvider::default());
        let config = NoteEngineConfig::default();
        let mut engine = NoteEngine::new(provider, "http://localhost:11434".to_string(), config);

        engine.add_segment(TranscriptSegment {
            text: "Test".to_string(),
            timestamp_ms: 1000,
            segment_id: "seg-1".to_string(),
        });

        engine.reset();
        assert_eq!(engine.pending_segments.len(), 0);
        assert!(engine.accumulated_notes.is_empty());
    }
}
