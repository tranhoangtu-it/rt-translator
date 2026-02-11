pub mod memo_builder;
pub mod note_engine;
pub mod note_types;
pub mod ollama_summarizer;
pub mod prompt_templates;
mod summarizer; // Legacy skeleton

pub use memo_builder::MemoBuilder;
pub use note_engine::{NoteEngine, NoteEngineConfig, SegmentBuffer, SharedNoteEngine, TranscriptSegment};
pub use note_types::*;
pub use ollama_summarizer::OllamaSummarizer;
