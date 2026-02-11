use std::collections::HashMap;
use std::io::Write;

use tauri::State;

use crate::commands::SttState;
use crate::storage::{MeetingRecord, TranscriptRecord, TranslationRecord};

/// Export transcript for a meeting to a file in the specified format.
/// Reads from both `transcripts` table and `translations` table (multi-lang).
#[tauri::command]
pub async fn export_transcript(
    stt_state: State<'_, SttState>,
    meeting_id: i64,
    format: String,
    path: String,
) -> Result<String, String> {
    let meeting = stt_state.transcript_db.get_meeting(meeting_id)?;
    let transcripts = stt_state.transcript_db.get_meeting_transcripts(meeting_id)?;
    let all_translations = stt_state.transcript_db.get_meeting_translations(meeting_id)?;

    // Group translations by transcript_id for lookup
    let mut trans_map: HashMap<i64, Vec<&TranslationRecord>> = HashMap::new();
    for t in &all_translations {
        trans_map.entry(t.transcript_id).or_default().push(t);
    }

    let content = match format.as_str() {
        "txt" => format_txt(&meeting, &transcripts, &trans_map),
        "md" => format_md(&meeting, &transcripts, &trans_map),
        "json" => format_json(&meeting, &transcripts, &trans_map)?,
        _ => return Err(format!("Unsupported format: {}", format)),
    };

    let mut file =
        std::fs::File::create(&path).map_err(|e| format!("Failed to create file: {}", e))?;
    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write file: {}", e))?;

    tracing::info!("Exported transcript to {} ({})", path, format);
    Ok(path)
}

type TransMap<'a> = HashMap<i64, Vec<&'a TranslationRecord>>;

/// Plain text format with multi-lang translations.
fn format_txt(meeting: &MeetingRecord, transcripts: &[TranscriptRecord], trans_map: &TransMap) -> String {
    let mut out = String::new();
    out.push_str(&format!("Meeting: {}\n", meeting.title));
    out.push_str(&format!("Date: {}\n", meeting.started_at));
    if let Some(ref ended) = meeting.ended_at {
        out.push_str(&format!("Ended: {}\n", ended));
    }
    out.push_str(&format!(
        "Language: {} -> {}\n",
        meeting.source_lang, meeting.target_langs
    ));
    out.push_str("---\n\n");

    if transcripts.is_empty() {
        out.push_str("(No transcripts recorded)\n");
        return out;
    }

    for t in transcripts {
        out.push_str(&format!("[{}] {}\n", t.timestamp, t.text));
        if let Some(id) = t.id {
            if let Some(translations) = trans_map.get(&id) {
                for tr in translations {
                    out.push_str(&format!("    [{}] {}\n", tr.target_lang, tr.translated_text));
                }
            }
        }
        // Fallback to legacy column if no translations in new table
        if t.id.and_then(|id| trans_map.get(&id)).is_none() {
            if let Some(ref legacy) = t.translated_text {
                out.push_str(&format!("         {}\n", legacy));
            }
        }
        out.push('\n');
    }
    out
}

/// Markdown format with multi-lang translations.
fn format_md(meeting: &MeetingRecord, transcripts: &[TranscriptRecord], trans_map: &TransMap) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Meeting Transcript: {}\n\n", meeting.title));
    out.push_str(&format!("**Date:** {}", meeting.started_at));
    if let Some(ref ended) = meeting.ended_at {
        out.push_str(&format!(" - {}", ended));
    }
    out.push_str(&format!(
        "\n**Languages:** {} -> {}\n\n---\n\n",
        meeting.source_lang, meeting.target_langs
    ));

    if transcripts.is_empty() {
        out.push_str("*No transcripts recorded*\n");
        return out;
    }

    for t in transcripts {
        out.push_str(&format!("**[{}]** {}\n", t.timestamp, t.text));
        if let Some(id) = t.id {
            if let Some(translations) = trans_map.get(&id) {
                for tr in translations {
                    out.push_str(&format!("> **{}:** {}\n", tr.target_lang.to_uppercase(), tr.translated_text));
                }
            }
        }
        if t.id.and_then(|id| trans_map.get(&id)).is_none() {
            if let Some(ref legacy) = t.translated_text {
                out.push_str(&format!("> {}\n", legacy));
            }
        }
        out.push('\n');
    }
    out
}

/// JSON format with multi-lang translations.
fn format_json(
    meeting: &MeetingRecord,
    transcripts: &[TranscriptRecord],
    trans_map: &TransMap,
) -> Result<String, String> {
    let export = serde_json::json!({
        "meeting": {
            "id": meeting.id,
            "title": meeting.title,
            "started_at": meeting.started_at,
            "ended_at": meeting.ended_at,
            "source_lang": meeting.source_lang,
            "target_langs": meeting.target_langs,
            "status": meeting.status,
        },
        "transcripts": transcripts.iter().map(|t| {
            let translations: HashMap<&str, &str> = t.id
                .and_then(|id| trans_map.get(&id))
                .map(|trs| trs.iter().map(|tr| (tr.target_lang.as_str(), tr.translated_text.as_str())).collect())
                .unwrap_or_default();
            serde_json::json!({
                "timestamp": t.timestamp,
                "text": t.text,
                "translations": translations,
                "translated_text": t.translated_text, // legacy fallback
                "speaker": t.speaker,
            })
        }).collect::<Vec<_>>(),
    });
    serde_json::to_string_pretty(&export).map_err(|e| format!("JSON serialization failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_meeting() -> MeetingRecord {
        MeetingRecord {
            id: Some(1),
            title: "Test Meeting".to_string(),
            started_at: "2026-02-10 22:00:00".to_string(),
            ended_at: Some("2026-02-10 22:30:00".to_string()),
            source_lang: "en".to_string(),
            target_langs: "vi,ja".to_string(),
            status: "stopped".to_string(),
        }
    }

    fn sample_transcripts() -> Vec<TranscriptRecord> {
        vec![TranscriptRecord {
            id: Some(1),
            meeting_id: 1,
            speaker: None,
            text: "Hello everyone".to_string(),
            translated_text: None,
            timestamp: "00:00:05".to_string(),
            is_final: true,
        }]
    }

    fn sample_trans_map() -> TransMap<'static> {
        // Use leaked static for test convenience
        let vi = Box::leak(Box::new(TranslationRecord {
            id: Some(1),
            transcript_id: 1,
            target_lang: "vi".to_string(),
            translated_text: "Xin chao moi nguoi".to_string(),
            created_at: None,
        }));
        let ja = Box::leak(Box::new(TranslationRecord {
            id: Some(2),
            transcript_id: 1,
            target_lang: "ja".to_string(),
            translated_text: "Mina-san konnichiwa".to_string(),
            created_at: None,
        }));
        let mut map: TransMap = HashMap::new();
        map.insert(1, vec![vi, ja]);
        map
    }

    #[test]
    fn txt_format_multi_lang() {
        let trans_map = sample_trans_map();
        let out = format_txt(&sample_meeting(), &sample_transcripts(), &trans_map);
        assert!(out.contains("Hello everyone"));
        assert!(out.contains("[vi] Xin chao moi nguoi"));
        assert!(out.contains("[ja] Mina-san konnichiwa"));
    }

    #[test]
    fn md_format_multi_lang() {
        let trans_map = sample_trans_map();
        let out = format_md(&sample_meeting(), &sample_transcripts(), &trans_map);
        assert!(out.contains("# Meeting Transcript"));
        assert!(out.contains("**VI:** Xin chao"));
        assert!(out.contains("**JA:** Mina-san"));
    }

    #[test]
    fn json_format_multi_lang() {
        let trans_map = sample_trans_map();
        let out = format_json(&sample_meeting(), &sample_transcripts(), &trans_map).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        let translations = &parsed["transcripts"][0]["translations"];
        assert_eq!(translations["vi"], "Xin chao moi nguoi");
        assert_eq!(translations["ja"], "Mina-san konnichiwa");
    }

    #[test]
    fn empty_transcripts_handled() {
        let trans_map: TransMap = HashMap::new();
        let out = format_txt(&sample_meeting(), &[], &trans_map);
        assert!(out.contains("No transcripts"));
    }
}
