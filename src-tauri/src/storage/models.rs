use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingRecord {
    pub id: Option<i64>,
    pub title: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub source_lang: String,
    pub target_langs: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptRecord {
    pub id: Option<i64>,
    pub meeting_id: i64,
    pub speaker: Option<String>,
    pub text: String,
    pub translated_text: Option<String>,
    pub timestamp: String,
    pub is_final: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteRecord {
    pub id: Option<i64>,
    pub meeting_id: i64,
    pub note_type: String,
    pub content: String,
    pub created_at: String,
}

/// A single translation row for a transcript in a specific target language.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationRecord {
    pub id: Option<i64>,
    pub transcript_id: i64,
    pub target_lang: String,
    pub translated_text: String,
    pub created_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_meeting_record() {
        let meeting = MeetingRecord {
            id: None,
            title: "Test Meeting".to_string(),
            started_at: "2026-02-10T00:00:00Z".to_string(),
            ended_at: None,
            source_lang: "en".to_string(),
            target_langs: "vi".to_string(),
            status: "idle".to_string(),
        };
        assert_eq!(meeting.title, "Test Meeting");
    }
}
