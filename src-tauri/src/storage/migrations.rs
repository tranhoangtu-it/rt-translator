use tauri_plugin_sql::{Migration, MigrationKind};

/// All database migrations, ordered by version.
pub fn get_migrations() -> Vec<Migration> {
    vec![migration_v1(), migration_v2(), migration_v3()]
}

/// V1: Initial schema -- meetings, transcripts, notes.
fn migration_v1() -> Migration {
    Migration {
        version: 1,
        description: "create_initial_tables",
        sql: r#"
            CREATE TABLE IF NOT EXISTS meetings (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                title       TEXT NOT NULL DEFAULT 'Untitled Meeting',
                started_at  TEXT NOT NULL DEFAULT (datetime('now')),
                ended_at    TEXT,
                source_lang TEXT NOT NULL DEFAULT 'auto',
                target_langs TEXT NOT NULL DEFAULT 'vi',
                status      TEXT NOT NULL DEFAULT 'idle'
                    CHECK(status IN ('idle', 'recording', 'paused', 'stopped')),
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS transcripts (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                meeting_id      INTEGER NOT NULL
                    REFERENCES meetings(id) ON DELETE CASCADE,
                speaker         TEXT,
                text            TEXT NOT NULL,
                translated_text TEXT,
                timestamp       TEXT NOT NULL DEFAULT (datetime('now')),
                is_final        INTEGER NOT NULL DEFAULT 0,
                created_at      TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_transcripts_meeting_id
                ON transcripts(meeting_id);

            CREATE TABLE IF NOT EXISTS notes (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                meeting_id  INTEGER NOT NULL
                    REFERENCES meetings(id) ON DELETE CASCADE,
                note_type   TEXT NOT NULL
                    CHECK(note_type IN ('key_point', 'decision', 'risk', 'action_item')),
                content     TEXT NOT NULL,
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_notes_meeting_id
                ON notes(meeting_id);

            PRAGMA foreign_keys = ON;
        "#,
        kind: MigrationKind::Up,
    }
}

/// V3: Normalized translations table for multi-target language support.
fn migration_v3() -> Migration {
    Migration {
        version: 3,
        description: "create_translations_table_multi_target",
        sql: r#"
            CREATE TABLE IF NOT EXISTS translations (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                transcript_id   INTEGER NOT NULL
                    REFERENCES transcripts(id) ON DELETE CASCADE,
                target_lang     TEXT NOT NULL,
                translated_text TEXT NOT NULL,
                created_at      TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(transcript_id, target_lang)
            );

            CREATE INDEX IF NOT EXISTS idx_translations_transcript_lang
                ON translations(transcript_id, target_lang);
        "#,
        kind: MigrationKind::Up,
    }
}

/// V2: Add segment_id to transcripts for matching STT segments to translations.
fn migration_v2() -> Migration {
    Migration {
        version: 2,
        description: "add_segment_id_to_transcripts",
        sql: r#"
            ALTER TABLE transcripts ADD COLUMN segment_id TEXT;
            CREATE INDEX IF NOT EXISTS idx_transcripts_segment_id
                ON transcripts(segment_id);
        "#,
        kind: MigrationKind::Up,
    }
}
