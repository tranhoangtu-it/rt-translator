use serde::{Deserialize, Serialize};

/// Key point extracted from meeting transcript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPoint {
    pub topic: String,
    pub summary: String,
    pub timestamp: String,
}

/// Decision made during the meeting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub decision: String,
    pub rationale: Option<String>,
    pub timestamp: String,
}

/// Action item assigned during the meeting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub task: String,
    pub owner: Option<String>,
    pub deadline: Option<String>,
    pub priority: Option<String>, // "high" | "medium" | "low"
}

/// Risk identified during the meeting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub risk: String,
    pub impact: Option<String>,
    pub mitigation: Option<String>,
    pub timestamp: String,
}

/// Response from Ollama incremental note extraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalNotesResponse {
    pub key_points: Vec<KeyPoint>,
    pub decisions: Vec<Decision>,
    pub action_items: Vec<ActionItem>,
    pub risks: Vec<Risk>,
}

impl IncrementalNotesResponse {
    pub fn empty() -> Self {
        Self {
            key_points: vec![],
            decisions: vec![],
            action_items: vec![],
            risks: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.key_points.is_empty()
            && self.decisions.is_empty()
            && self.action_items.is_empty()
            && self.risks.is_empty()
    }

    pub fn merge(&mut self, other: IncrementalNotesResponse) {
        self.key_points.extend(other.key_points);
        self.decisions.extend(other.decisions);
        self.action_items.extend(other.action_items);
        self.risks.extend(other.risks);
    }

    pub fn count(&self) -> usize {
        self.key_points.len()
            + self.decisions.len()
            + self.action_items.len()
            + self.risks.len()
    }
}

/// Note category enum for DB storage.
#[derive(Debug, Clone, Serialize)]
pub enum NoteCategory {
    KeyPoint,
    Decision,
    ActionItem,
    Risk,
}

impl NoteCategory {
    pub fn as_str(&self) -> &str {
        match self {
            Self::KeyPoint => "key_point",
            Self::Decision => "decision",
            Self::ActionItem => "action_item",
            Self::Risk => "risk",
        }
    }
}

/// Event payload for notes-updated Tauri event.
#[derive(Debug, Clone, Serialize)]
pub struct NotesUpdatedPayload {
    pub meeting_id: i64,
    pub new_notes: IncrementalNotesResponse,
    pub total_count: usize,
    pub inserted_ids: Vec<i64>,
}

/// Event payload for notes-error Tauri event.
#[derive(Debug, Clone, Serialize)]
pub struct NotesErrorPayload {
    pub meeting_id: i64,
    pub error: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_empty_notes_response() {
        let notes = IncrementalNotesResponse::empty();
        assert!(notes.is_empty());
        assert_eq!(notes.count(), 0);
    }

    #[test]
    fn merges_notes_responses() {
        let mut notes = IncrementalNotesResponse::empty();
        let new_notes = IncrementalNotesResponse {
            key_points: vec![KeyPoint {
                topic: "Test".to_string(),
                summary: "Summary".to_string(),
                timestamp: "00:00:00".to_string(),
            }],
            decisions: vec![],
            action_items: vec![],
            risks: vec![],
        };
        notes.merge(new_notes);
        assert_eq!(notes.count(), 1);
    }

    #[test]
    fn category_as_str() {
        assert_eq!(NoteCategory::KeyPoint.as_str(), "key_point");
        assert_eq!(NoteCategory::Decision.as_str(), "decision");
        assert_eq!(NoteCategory::ActionItem.as_str(), "action_item");
        assert_eq!(NoteCategory::Risk.as_str(), "risk");
    }
}
