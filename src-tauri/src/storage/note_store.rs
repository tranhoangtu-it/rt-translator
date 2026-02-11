use crate::storage::models::NoteRecord;
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};

/// Storage operations for meeting notes.
#[derive(Clone)]
pub struct NoteStore {
    conn: Arc<Mutex<Connection>>,
}

impl NoteStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Insert a note entry for a meeting.
    pub fn insert_note(
        &self,
        meeting_id: i64,
        note_type: &str,
        content: &str,
    ) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO notes (meeting_id, note_type, content) VALUES (?1, ?2, ?3)",
            params![meeting_id, note_type, content],
        )
        .map_err(|e| format!("Failed to insert note: {}", e))?;
        Ok(conn.last_insert_rowid())
    }

    /// Get all notes for a meeting, optionally filtered by type.
    pub fn get_notes(
        &self,
        meeting_id: i64,
        note_type_filter: Option<&str>,
    ) -> Result<Vec<NoteRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let (sql, params_vec): (String, Vec<Box<dyn rusqlite::ToSql>>) =
            if let Some(nt) = note_type_filter {
                (
                    "SELECT id, meeting_id, note_type, content, created_at FROM notes \
                     WHERE meeting_id = ?1 AND note_type = ?2 ORDER BY created_at ASC"
                        .to_string(),
                    vec![Box::new(meeting_id), Box::new(nt.to_string())],
                )
            } else {
                (
                    "SELECT id, meeting_id, note_type, content, created_at FROM notes \
                     WHERE meeting_id = ?1 ORDER BY created_at ASC"
                        .to_string(),
                    vec![Box::new(meeting_id)],
                )
            };

        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|b| b.as_ref()).collect();

        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params_refs.as_slice(), |row| {
                Ok(NoteRecord {
                    id: row.get(0)?,
                    meeting_id: row.get(1)?,
                    note_type: row.get(2)?,
                    content: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// Update note content by ID (for user edits).
    pub fn update_note(&self, note_id: i64, content: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE notes SET content = ?1 WHERE id = ?2",
            params![content, note_id],
        )
        .map_err(|e| format!("Failed to update note: {}", e))?;
        Ok(())
    }

    /// Delete a note by ID.
    pub fn delete_note(&self, note_id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM notes WHERE id = ?1", params![note_id])
            .map_err(|e| format!("Failed to delete note: {}", e))?;
        Ok(())
    }

    /// Delete all notes for a meeting (used in reset/cleanup).
    pub fn delete_meeting_notes(&self, meeting_id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM notes WHERE meeting_id = ?1",
            params![meeting_id],
        )
        .map_err(|e| format!("Failed to delete meeting notes: {}", e))?;
        Ok(())
    }

    /// Batch insert notes (for incremental updates). Returns inserted row IDs.
    pub fn insert_notes_batch(
        &self,
        notes: Vec<(i64, String, String)>, // (meeting_id, note_type, content)
    ) -> Result<Vec<i64>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let tx = conn
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;

        let mut ids = Vec::with_capacity(notes.len());
        for (meeting_id, note_type, content) in notes {
            tx.execute(
                "INSERT INTO notes (meeting_id, note_type, content) VALUES (?1, ?2, ?3)",
                params![meeting_id, &note_type, &content],
            )
            .map_err(|e| format!("Batch insert failed: {}", e))?;
            ids.push(tx.last_insert_rowid());
        }

        tx.commit()
            .map_err(|e| format!("Transaction commit failed: {}", e))?;
        Ok(ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_store() -> NoteStore {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                meeting_id INTEGER NOT NULL,
                note_type TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )
        .unwrap();
        NoteStore::new(Arc::new(Mutex::new(conn)))
    }

    #[test]
    fn inserts_note() {
        let store = create_test_store();
        let id = store
            .insert_note(1, "key_point", r#"{"topic":"Test"}"#)
            .unwrap();
        assert_eq!(id, 1);
    }

    #[test]
    fn gets_notes_for_meeting() {
        let store = create_test_store();
        store
            .insert_note(1, "key_point", r#"{"topic":"Test 1"}"#)
            .unwrap();
        store
            .insert_note(1, "decision", r#"{"decision":"Test 2"}"#)
            .unwrap();
        store
            .insert_note(2, "key_point", r#"{"topic":"Other meeting"}"#)
            .unwrap();

        let notes = store.get_notes(1, None).unwrap();
        assert_eq!(notes.len(), 2);
    }

    #[test]
    fn filters_notes_by_type() {
        let store = create_test_store();
        store
            .insert_note(1, "key_point", r#"{"topic":"Test 1"}"#)
            .unwrap();
        store
            .insert_note(1, "decision", r#"{"decision":"Test 2"}"#)
            .unwrap();

        let notes = store.get_notes(1, Some("key_point")).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].note_type, "key_point");
    }

    #[test]
    fn updates_note() {
        let store = create_test_store();
        let id = store
            .insert_note(1, "key_point", r#"{"topic":"Old"}"#)
            .unwrap();
        store.update_note(id, r#"{"topic":"New"}"#).unwrap();

        let notes = store.get_notes(1, None).unwrap();
        assert!(notes[0].content.contains("New"));
    }

    #[test]
    fn deletes_note() {
        let store = create_test_store();
        let id = store
            .insert_note(1, "key_point", r#"{"topic":"Test"}"#)
            .unwrap();
        store.delete_note(id).unwrap();

        let notes = store.get_notes(1, None).unwrap();
        assert_eq!(notes.len(), 0);
    }

    #[test]
    fn batch_inserts_notes_returns_ids() {
        let store = create_test_store();
        let batch = vec![
            (1, "key_point".to_string(), r#"{"topic":"Test 1"}"#.to_string()),
            (1, "decision".to_string(), r#"{"decision":"Test 2"}"#.to_string()),
        ];
        let ids = store.insert_notes_batch(batch).unwrap();
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0], 1);
        assert_eq!(ids[1], 2);

        let notes = store.get_notes(1, None).unwrap();
        assert_eq!(notes.len(), 2);
    }
}
