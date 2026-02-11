use crate::storage::models::{MeetingRecord, TranscriptRecord, TranslationRecord};
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Thread-safe handle to the SQLite database for transcript operations.
#[derive(Clone)]
pub struct TranscriptDb {
    conn: Arc<Mutex<Connection>>,
}

impl TranscriptDb {
    /// Create a new TranscriptDb from an existing connection Arc.
    /// Used to share the same connection across multiple stores.
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Get a clone of the internal connection Arc for sharing with other stores.
    pub fn get_connection(&self) -> Arc<Mutex<Connection>> {
        self.conn.clone()
    }
}

impl TranscriptDb {
    /// Open (or create) the translator.db in the given app data directory.
    pub fn open(app_data_dir: &Path) -> Result<Self, String> {
        let db_path = app_data_dir.join("translator.db");
        let conn =
            Connection::open(&db_path).map_err(|e| format!("Failed to open DB: {}", e))?;
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")
            .map_err(|e| format!("Failed to set pragmas: {}", e))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Create a new meeting record. Returns the meeting_id.
    pub fn create_meeting(&self, source_lang: &str, target_langs: &str) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO meetings (source_lang, target_langs, status) VALUES (?1, ?2, 'recording')",
            params![source_lang, target_langs],
        )
        .map_err(|e| format!("Failed to create meeting: {}", e))?;
        Ok(conn.last_insert_rowid())
    }

    /// Insert a transcript row for a finalized STT segment.
    pub fn insert_transcript(
        &self,
        meeting_id: i64,
        text: &str,
        segment_id: &str,
        timestamp_ms: i64,
    ) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let ts = format_ms_to_timestamp(timestamp_ms);
        conn.execute(
            "INSERT INTO transcripts (meeting_id, text, segment_id, timestamp, is_final) \
             VALUES (?1, ?2, ?3, ?4, 1)",
            params![meeting_id, text, segment_id, ts],
        )
        .map_err(|e| format!("Failed to insert transcript: {}", e))?;
        Ok(conn.last_insert_rowid())
    }

    /// Update translated_text for a transcript matched by segment_id.
    pub fn update_transcript_translation(
        &self,
        meeting_id: i64,
        segment_id: &str,
        translated_text: &str,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE transcripts SET translated_text = ?1 \
             WHERE meeting_id = ?2 AND segment_id = ?3",
            params![translated_text, meeting_id, segment_id],
        )
        .map_err(|e| format!("Failed to update translation: {}", e))?;
        Ok(())
    }

    /// Mark meeting as stopped with ended_at timestamp.
    pub fn end_meeting(&self, meeting_id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE meetings SET status = 'stopped', ended_at = datetime('now') WHERE id = ?1",
            params![meeting_id],
        )
        .map_err(|e| format!("Failed to end meeting: {}", e))?;
        Ok(())
    }

    /// Get meeting metadata by ID.
    pub fn get_meeting(&self, meeting_id: i64) -> Result<MeetingRecord, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT id, title, started_at, ended_at, source_lang, target_langs, status \
             FROM meetings WHERE id = ?1",
            params![meeting_id],
            |row| {
                Ok(MeetingRecord {
                    id: Some(row.get(0)?),
                    title: row.get(1)?,
                    started_at: row.get(2)?,
                    ended_at: row.get(3)?,
                    source_lang: row.get(4)?,
                    target_langs: row.get(5)?,
                    status: row.get(6)?,
                })
            },
        )
        .map_err(|e| format!("Meeting not found: {}", e))
    }

    /// Get all final transcripts for a meeting, ordered by timestamp.
    pub fn get_meeting_transcripts(
        &self,
        meeting_id: i64,
    ) -> Result<Vec<TranscriptRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, meeting_id, speaker, text, translated_text, timestamp, is_final \
                 FROM transcripts WHERE meeting_id = ?1 AND is_final = 1 \
                 ORDER BY id ASC",
            )
            .map_err(|e| format!("Query prepare failed: {}", e))?;

        let rows = stmt
            .query_map(params![meeting_id], |row| {
                Ok(TranscriptRecord {
                    id: Some(row.get(0)?),
                    meeting_id: row.get(1)?,
                    speaker: row.get(2)?,
                    text: row.get(3)?,
                    translated_text: row.get(4)?,
                    timestamp: row.get(5)?,
                    is_final: row.get::<_, i32>(6)? != 0,
                })
            })
            .map_err(|e| format!("Query failed: {}", e))?;

        let mut transcripts = Vec::new();
        for row in rows {
            transcripts.push(row.map_err(|e| format!("Row read failed: {}", e))?);
        }
        Ok(transcripts)
    }

    /// Insert or replace a translation for a transcript in a specific target language.
    pub fn insert_translation(
        &self,
        transcript_id: i64,
        target_lang: &str,
        translated_text: &str,
    ) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO translations (transcript_id, target_lang, translated_text) \
             VALUES (?1, ?2, ?3)",
            params![transcript_id, target_lang, translated_text],
        )
        .map_err(|e| format!("Failed to insert translation: {}", e))?;
        Ok(conn.last_insert_rowid())
    }

    /// Look up transcript row ID by meeting_id + segment_id.
    pub fn get_transcript_id_by_segment(
        &self,
        meeting_id: i64,
        segment_id: &str,
    ) -> Result<Option<i64>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        match conn.query_row(
            "SELECT id FROM transcripts WHERE meeting_id = ?1 AND segment_id = ?2",
            params![meeting_id, segment_id],
            |row| row.get(0),
        ) {
            Ok(id) => Ok(Some(id)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Query failed: {}", e)),
        }
    }

    /// Get all translations for a specific transcript.
    pub fn get_translations_for_transcript(
        &self,
        transcript_id: i64,
    ) -> Result<Vec<TranslationRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, transcript_id, target_lang, translated_text, created_at \
                 FROM translations WHERE transcript_id = ?1",
            )
            .map_err(|e| format!("Prepare failed: {}", e))?;

        let rows = stmt
            .query_map(params![transcript_id], |row| {
                Ok(TranslationRecord {
                    id: Some(row.get(0)?),
                    transcript_id: row.get(1)?,
                    target_lang: row.get(2)?,
                    translated_text: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| format!("Query failed: {}", e))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Row read failed: {}", e))
    }

    /// Get all translations for a meeting, joined with transcripts, ordered by transcript ID.
    pub fn get_meeting_translations(
        &self,
        meeting_id: i64,
    ) -> Result<Vec<TranslationRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT t.id, t.transcript_id, t.target_lang, t.translated_text, t.created_at \
                 FROM translations t \
                 JOIN transcripts tr ON t.transcript_id = tr.id \
                 WHERE tr.meeting_id = ?1 \
                 ORDER BY tr.id ASC, t.target_lang ASC",
            )
            .map_err(|e| format!("Prepare failed: {}", e))?;

        let rows = stmt
            .query_map(params![meeting_id], |row| {
                Ok(TranslationRecord {
                    id: Some(row.get(0)?),
                    transcript_id: row.get(1)?,
                    target_lang: row.get(2)?,
                    translated_text: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| format!("Query failed: {}", e))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Row read failed: {}", e))
    }
}

/// Convert milliseconds offset to HH:MM:SS string for display.
/// This is a module-level helper used by TranscriptDb methods.
fn format_ms_to_timestamp(ms: i64) -> String {
    let total_secs = ms / 1000;
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_ms_converts_correctly() {
        assert_eq!(format_ms_to_timestamp(0), "00:00:00");
        assert_eq!(format_ms_to_timestamp(61000), "00:01:01");
        assert_eq!(format_ms_to_timestamp(3661000), "01:01:01");
    }
}
