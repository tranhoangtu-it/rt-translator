use crate::notes::{ActionItem, Decision, IncrementalNotesResponse, KeyPoint, MemoBuilder, Risk};
use crate::storage::{note_store::NoteStore, NoteRecord, TranscriptDb};
use tauri::State;

/// Managed state for note operations.
pub struct NoteState {
    pub store: NoteStore,
    pub transcript_db: TranscriptDb,
}

/// Get all notes for a meeting, optionally filtered by type.
#[tauri::command]
pub async fn get_notes(
    meeting_id: i64,
    note_type: Option<String>,
    state: State<'_, NoteState>,
) -> Result<Vec<NoteRecord>, String> {
    state.store.get_notes(meeting_id, note_type.as_deref())
}

/// Update note content by ID (for user edits).
#[tauri::command]
pub async fn update_note(
    note_id: i64,
    content: String,
    state: State<'_, NoteState>,
) -> Result<(), String> {
    state.store.update_note(note_id, &content)
}

/// Delete a note by ID.
#[tauri::command]
pub async fn delete_note(note_id: i64, state: State<'_, NoteState>) -> Result<(), String> {
    state.store.delete_note(note_id)
}

/// Generate meeting memo from accumulated notes.
#[tauri::command]
pub async fn generate_memo(
    meeting_id: i64,
    ollama_provider: State<'_, crate::commands::TranslationState>,
    state: State<'_, NoteState>,
) -> Result<String, String> {
    // Get meeting metadata
    let meeting = state
        .transcript_db
        .get_meeting(meeting_id)
        .map_err(|e| format!("Failed to get meeting: {}", e))?;

    // Get all notes for meeting
    let note_records = state.store.get_notes(meeting_id, None)?;

    // Convert NoteRecords to IncrementalNotesResponse
    let notes = parse_notes_from_records(note_records)?;

    // Generate memo markdown
    let builder = MemoBuilder::new(ollama_provider.provider.clone());
    let memo = builder
        .generate_memo(&meeting.title, &meeting.started_at, &notes)
        .await
        .map_err(|e| format!("Failed to generate memo: {}", e))?;

    Ok(memo)
}

/// Export memo to file (saves to Downloads folder).
#[tauri::command]
pub async fn export_memo(
    app: tauri::AppHandle,
    meeting_id: i64,
    memo_content: String,
) -> Result<String, String> {
    use tauri::Manager;

    // Get Downloads folder path
    let downloads_dir = app
        .path()
        .download_dir()
        .map_err(|e| format!("Failed to get downloads dir: {}", e))?;

    // Generate filename with timestamp
    let filename = format!("meeting-memo-{}.md", meeting_id);
    let file_path = downloads_dir.join(&filename);

    // Write memo to file
    std::fs::write(&file_path, memo_content)
        .map_err(|e| format!("Failed to write memo: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

/// Helper to convert DB records to IncrementalNotesResponse.
fn parse_notes_from_records(
    records: Vec<NoteRecord>,
) -> Result<IncrementalNotesResponse, String> {
    let mut notes = IncrementalNotesResponse::empty();

    for record in records {
        match record.note_type.as_str() {
            "key_point" => {
                let kp: KeyPoint = serde_json::from_str(&record.content)
                    .map_err(|e| format!("Failed to parse key_point: {}", e))?;
                notes.key_points.push(kp);
            }
            "decision" => {
                let dec: Decision = serde_json::from_str(&record.content)
                    .map_err(|e| format!("Failed to parse decision: {}", e))?;
                notes.decisions.push(dec);
            }
            "action_item" => {
                let action: ActionItem = serde_json::from_str(&record.content)
                    .map_err(|e| format!("Failed to parse action_item: {}", e))?;
                notes.action_items.push(action);
            }
            "risk" => {
                let risk: Risk = serde_json::from_str(&record.content)
                    .map_err(|e| format!("Failed to parse risk: {}", e))?;
                notes.risks.push(risk);
            }
            _ => {
                tracing::warn!("Unknown note type: {}", record.note_type);
            }
        }
    }

    Ok(notes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex};

    fn create_test_note_state() -> NoteState {
        let conn = Arc::new(Mutex::new(Connection::open_in_memory().unwrap()));
        conn.lock()
            .unwrap()
            .execute_batch(
                "CREATE TABLE notes (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    meeting_id INTEGER NOT NULL,
                    note_type TEXT NOT NULL,
                    content TEXT NOT NULL,
                    created_at TEXT NOT NULL DEFAULT (datetime('now'))
                );
                CREATE TABLE meetings (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    title TEXT NOT NULL DEFAULT 'Untitled Meeting',
                    started_at TEXT NOT NULL DEFAULT (datetime('now')),
                    ended_at TEXT,
                    source_lang TEXT NOT NULL,
                    target_langs TEXT NOT NULL,
                    status TEXT NOT NULL
                );",
            )
            .unwrap();

        NoteState {
            store: NoteStore::new(conn.clone()),
            transcript_db: TranscriptDb::new(conn),
        }
    }

    #[test]
    fn note_state_compiles() {
        let _state = create_test_note_state();
    }

    #[test]
    fn parses_notes_from_records() {
        let records = vec![
            NoteRecord {
                id: Some(1),
                meeting_id: 1,
                note_type: "key_point".to_string(),
                content: r#"{"topic":"Test","summary":"Summary","timestamp":"00:01:00"}"#
                    .to_string(),
                created_at: "2026-02-10".to_string(),
            },
            NoteRecord {
                id: Some(2),
                meeting_id: 1,
                note_type: "decision".to_string(),
                content: r#"{"decision":"Test decision","timestamp":"00:02:00"}"#.to_string(),
                created_at: "2026-02-10".to_string(),
            },
        ];

        let notes = parse_notes_from_records(records).unwrap();
        assert_eq!(notes.key_points.len(), 1);
        assert_eq!(notes.decisions.len(), 1);
        assert_eq!(notes.count(), 2);
    }
}
