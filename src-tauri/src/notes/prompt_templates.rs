use super::note_engine::TranscriptSegment;
use super::note_types::IncrementalNotesResponse;

pub const SYSTEM_PROMPT: &str = r#"You are a professional meeting note-taker. Extract structured information from meeting transcripts.

IMPORTANT RULES:
1. Only extract NEW information not present in existing notes
2. Output ONLY in JSON format (no markdown, no extra text)
3. If no new information, return empty arrays
4. For action items, infer owner/deadline from context clues (e.g., "John will finish by Friday")
5. Be concise: max 1-2 sentences per bullet
6. DO NOT repeat information already in existing notes"#;

/// Build user prompt with existing notes context and recent transcript segments.
pub fn build_user_prompt(
    existing_notes: &IncrementalNotesResponse,
    recent_segments: &[TranscriptSegment],
) -> String {
    let notes_json = serde_json::to_string_pretty(existing_notes).unwrap_or_default();
    let transcript = format_transcript_segments(recent_segments);

    format!(
        r#"EXISTING NOTES:
{}

RECENT TRANSCRIPT (last 2-3 minutes):
{}

Extract ONLY new information and output as JSON matching the schema."#,
        notes_json, transcript
    )
}

fn format_transcript_segments(segments: &[TranscriptSegment]) -> String {
    segments
        .iter()
        .map(|s| format!("[{}] {}", format_timestamp(s.timestamp_ms), s.text))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn format_timestamp(ms: i64) -> String {
    let total_secs = ms / 1000;
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

/// Get JSON schema for Ollama structured output.
pub fn get_json_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "key_points": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "topic": {"type": "string"},
                        "summary": {"type": "string"},
                        "timestamp": {"type": "string"}
                    },
                    "required": ["topic", "summary", "timestamp"]
                }
            },
            "decisions": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "decision": {"type": "string"},
                        "rationale": {"type": "string"},
                        "timestamp": {"type": "string"}
                    },
                    "required": ["decision", "timestamp"]
                }
            },
            "action_items": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "task": {"type": "string"},
                        "owner": {"type": "string"},
                        "deadline": {"type": "string"},
                        "priority": {"type": "string", "enum": ["high", "medium", "low"]}
                    },
                    "required": ["task"]
                }
            },
            "risks": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "risk": {"type": "string"},
                        "impact": {"type": "string"},
                        "mitigation": {"type": "string"},
                        "timestamp": {"type": "string"}
                    },
                    "required": ["risk", "timestamp"]
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_timestamp_correctly() {
        assert_eq!(format_timestamp(0), "00:00:00");
        assert_eq!(format_timestamp(61000), "00:01:01");
        assert_eq!(format_timestamp(3661000), "01:01:01");
    }

    #[test]
    fn builds_user_prompt() {
        let existing = IncrementalNotesResponse::empty();
        let segments = vec![TranscriptSegment {
            text: "Test segment".to_string(),
            timestamp_ms: 1000,
            segment_id: "seg-1".to_string(),
        }];
        let prompt = build_user_prompt(&existing, &segments);
        assert!(prompt.contains("EXISTING NOTES"));
        assert!(prompt.contains("RECENT TRANSCRIPT"));
        assert!(prompt.contains("Test segment"));
    }

    #[test]
    fn json_schema_valid() {
        let schema = get_json_schema();
        assert!(schema["properties"]["key_points"].is_object());
        assert!(schema["properties"]["decisions"].is_object());
        assert!(schema["properties"]["action_items"].is_object());
        assert!(schema["properties"]["risks"].is_object());
    }
}
