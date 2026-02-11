use crate::commands::AudioState;
use crate::notes::{
    NoteEngine, NoteEngineConfig, NotesErrorPayload, NotesUpdatedPayload, SegmentBuffer,
    SharedNoteEngine, TranscriptSegment,
};
use crate::storage::TranscriptDb;
use crate::stt::pipeline::MicFormat;
use crate::stt::{ModelManager, SttEngine, SttPipeline, DEFAULT_MODEL};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri::State;

/// Max note content length (2 KB) to prevent LLM output flooding DB.
const MAX_NOTE_CONTENT_LEN: usize = 2048;

/// Application state for STT (speech-to-text).
pub struct SttState {
    pub model_manager: ModelManager,
    pub engine: Mutex<Option<Arc<SttEngine>>>,
    pub pipeline: Mutex<Option<SttPipeline>>,
    pub meeting_id: Arc<Mutex<Option<i64>>>,
    pub transcript_db: TranscriptDb,
    pub note_engine: SharedNoteEngine,
    pub note_task_handle: Arc<Mutex<Option<tauri::async_runtime::JoinHandle<()>>>>,
    pub segment_buffer: SegmentBuffer,
}

impl SttState {
    pub fn new(app_data_dir: std::path::PathBuf) -> Self {
        let transcript_db = TranscriptDb::open(&app_data_dir)
            .expect("Failed to open transcript database");
        Self {
            model_manager: ModelManager::new(app_data_dir),
            engine: Mutex::new(None),
            pipeline: Mutex::new(None),
            meeting_id: Arc::new(Mutex::new(None)),
            transcript_db,
            note_engine: Arc::new(tokio::sync::Mutex::new(None)),
            note_task_handle: Arc::new(Mutex::new(None)),
            segment_buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelStatus {
    pub available: bool,
    pub model_name: String,
    pub file_size_mb: f64,
}

/// Check if default whisper model is downloaded.
#[tauri::command]
pub fn check_model_status(state: State<SttState>) -> Result<ModelStatus, String> {
    let name = DEFAULT_MODEL;
    Ok(ModelStatus {
        available: state.model_manager.is_model_available(name),
        model_name: name.to_string(),
        file_size_mb: state.model_manager.model_size_mb(name),
    })
}

/// Download default whisper model from HuggingFace CDN.
#[tauri::command]
pub async fn download_model(
    app: tauri::AppHandle,
    state: State<'_, SttState>,
) -> Result<String, String> {
    let path = state
        .model_manager
        .download_model(DEFAULT_MODEL, app)
        .await
        .map_err(|e| format!("Download failed: {}", e))?;

    Ok(path.to_string_lossy().to_string())
}

/// Start a meeting: load model, create STT pipeline, start audio capture with STT fork.
#[tauri::command]
pub fn start_meeting(
    src_lang: Option<String>,
    target_langs: Option<Vec<String>>,
    app: tauri::AppHandle,
    stt_state: State<SttState>,
    audio_state: State<AudioState>,
    translation_state: State<crate::commands::TranslationState>,
    note_state: State<crate::commands::NoteState>,
) -> Result<String, String> {
    // Check if pipeline already running
    {
        let guard = stt_state
            .pipeline
            .lock()
            .map_err(|e| format!("Pipeline lock poisoned: {}", e))?;
        if guard.is_some() {
            return Err("Meeting already in progress. Stop first.".to_string());
        }
    }

    // Ensure model is available
    if !stt_state.model_manager.is_model_available(DEFAULT_MODEL) {
        return Err("Whisper model not downloaded. Call download_model first.".to_string());
    }

    // Load engine if not already loaded
    {
        let mut engine_guard = stt_state
            .engine
            .lock()
            .map_err(|e| format!("Engine lock poisoned: {}", e))?;
        if engine_guard.is_none() {
            let model_path = stt_state.model_manager.model_path(DEFAULT_MODEL);
            let path_str = model_path
                .to_str()
                .ok_or_else(|| "Model path contains invalid characters".to_string())?;
            let lang = src_lang.clone();
            let engine = SttEngine::new(path_str, lang)?;
            *engine_guard = Some(Arc::new(engine));
        }
    }

    let engine = stt_state
        .engine
        .lock()
        .map_err(|e| format!("Engine lock poisoned: {}", e))?
        .clone()
        .ok_or("Engine not loaded")?;

    // Create crossbeam channel for audio → STT pipeline
    let (stt_tx, stt_rx) = crossbeam::channel::bounded::<Vec<f32>>(100);

    // Attach STT sender to audio capture manager and get mic format
    let mic_format = {
        let audio_guard = audio_state
            .manager
            .lock()
            .map_err(|e| format!("Audio lock poisoned: {}", e))?;
        if let Some(ref manager) = *audio_guard {
            manager.set_stt_sender(stt_tx);
            let (rate, channels) = manager.mic_format();
            MicFormat {
                sample_rate: if rate == 0 { 48000 } else { rate },
                channels: if channels == 0 { 1 } else { channels },
            }
        } else {
            return Err("Audio capture not running. Start audio capture first.".to_string());
        }
    };

    // Create meeting record in SQLite BEFORE starting pipeline
    // so that early STT segments can reference the meeting_id
    let tgt = target_langs.unwrap_or_else(|| vec!["vi".to_string()]).join(",");
    let db_meeting_id = stt_state
        .transcript_db
        .create_meeting(src_lang.as_deref().unwrap_or("en"), &tgt)
        .map_err(|e| format!("Failed to create meeting record: {}", e))?;

    {
        let mut mid_guard = stt_state
            .meeting_id
            .lock()
            .map_err(|e| format!("Meeting ID lock poisoned: {}", e))?;
        *mid_guard = Some(db_meeting_id);
    }

    // Clear segment buffer for fresh meeting
    {
        let mut buf = stt_state.segment_buffer.lock().map_err(|e| e.to_string())?;
        buf.clear();
    }

    // Start STT pipeline with segment buffer (not note engine directly)
    let pipeline = SttPipeline::start(
        stt_rx,
        engine,
        app.clone(),
        mic_format,
        stt_state.transcript_db.clone(),
        stt_state.meeting_id.clone(),
        stt_state.segment_buffer.clone(),
    );

    {
        let mut guard = stt_state
            .pipeline
            .lock()
            .map_err(|e| format!("Pipeline lock poisoned: {}", e))?;
        *guard = Some(pipeline);
    }

    // Initialize NoteEngine with OllamaProvider from TranslationState
    let ollama_provider = translation_state.provider.clone();
    let base_url = "http://localhost:11434".to_string(); // TODO: get from config
    let config = NoteEngineConfig::default();
    let engine = NoteEngine::new(ollama_provider, base_url, config);

    {
        let mut engine_guard = stt_state.note_engine.blocking_lock();
        *engine_guard = Some(engine);
    }

    // Fix #1: Abort any existing note task before spawning new one (restart safety)
    {
        let mut handle_guard = stt_state
            .note_task_handle
            .lock()
            .map_err(|e| format!("Note task handle lock poisoned: {}", e))?;
        if let Some(old_handle) = handle_guard.take() {
            old_handle.abort();
            tracing::warn!("Aborted stale note generation task from previous meeting");
        }
    }

    // Spawn note generation task with segment buffer
    let note_engine_clone = stt_state.note_engine.clone();
    let segment_buf_clone = stt_state.segment_buffer.clone();
    let note_store = note_state.store.clone();
    let app_clone = app.clone();
    let meeting_id_clone = db_meeting_id;

    let note_task = tauri::async_runtime::spawn(async move {
        run_note_generation_loop(
            note_engine_clone,
            segment_buf_clone,
            note_store,
            app_clone,
            meeting_id_clone,
        )
        .await;
    });

    {
        let mut handle_guard = stt_state
            .note_task_handle
            .lock()
            .map_err(|e| format!("Note task handle lock poisoned: {}", e))?;
        *handle_guard = Some(note_task);
    }

    tracing::info!("Meeting started: DB id={}", db_meeting_id);
    Ok(db_meeting_id.to_string())
}

/// Stop meeting: tear down STT pipeline, remove STT sender from audio.
#[tauri::command]
pub fn stop_meeting(
    stt_state: State<SttState>,
    audio_state: State<AudioState>,
) -> Result<String, String> {
    // Fix #3: Set engine to None FIRST (loop will break on next check),
    // then abort the task handle for immediate cancellation.
    {
        let mut engine_guard = stt_state.note_engine.blocking_lock();
        if let Some(engine) = engine_guard.as_mut() {
            engine.reset();
        }
        *engine_guard = None;
    }

    {
        let mut handle_guard = stt_state
            .note_task_handle
            .lock()
            .map_err(|e| format!("Note task handle lock poisoned: {}", e))?;
        if let Some(handle) = handle_guard.take() {
            handle.abort();
            tracing::info!("Note generation task aborted");
        }
    }

    // Clear segment buffer
    {
        let mut buf = stt_state.segment_buffer.lock().map_err(|e| e.to_string())?;
        buf.clear();
    }

    // Stop pipeline (takes ownership via .take() + .stop(self))
    {
        let mut guard = stt_state
            .pipeline
            .lock()
            .map_err(|e| format!("Pipeline lock poisoned: {}", e))?;
        match guard.take() {
            Some(pipeline) => pipeline.stop(),
            None => return Err("No meeting in progress".to_string()),
        }
    }

    // End meeting record in DB and clear meeting_id atomically
    {
        let mut mid_guard = stt_state
            .meeting_id
            .lock()
            .map_err(|e| format!("Meeting ID lock poisoned: {}", e))?;
        if let Some(mid) = mid_guard.take() {
            if let Err(e) = stt_state.transcript_db.end_meeting(mid) {
                tracing::error!("Failed to end meeting record: {}", e);
            }
        }
    }

    // Remove STT sender from audio manager
    {
        let audio_guard = audio_state
            .manager
            .lock()
            .map_err(|e| format!("Audio lock poisoned: {}", e))?;
        if let Some(ref manager) = *audio_guard {
            manager.clear_stt_sender();
        }
    }

    tracing::info!("Meeting stopped");
    Ok("Meeting stopped".to_string())
}

/// Async task loop for note generation.
/// Drains segment buffer, checks triggers, generates notes, saves to DB, emits events.
async fn run_note_generation_loop(
    note_engine: SharedNoteEngine,
    segment_buffer: SegmentBuffer,
    note_store: crate::storage::NoteStore,
    app: tauri::AppHandle,
    meeting_id: i64,
) {
    use tauri::Emitter;

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));

    loop {
        interval.tick().await;

        // Fix #2: Drain segment buffer (std::sync::Mutex — fast, no contention)
        let drained: Vec<TranscriptSegment> = {
            let mut buf = match segment_buffer.lock() {
                Ok(buf) => buf,
                Err(e) => {
                    tracing::error!("Segment buffer lock poisoned: {}", e);
                    break;
                }
            };
            std::mem::take(&mut *buf)
        };

        // Fix #3: Lock engine, feed segments, check trigger, generate — single scope
        let result = {
            let mut guard = note_engine.lock().await;
            let engine = match guard.as_mut() {
                Some(e) => e,
                None => {
                    tracing::debug!("Note engine stopped, exiting loop");
                    break;
                }
            };

            for seg in drained {
                engine.add_segment(seg);
            }

            if !engine.should_update() {
                continue;
            }

            engine.update_notes().await
        };
        // Engine lock released here — DB save + emit don't block segment feeding

        match result {
            Ok(new_notes) => {
                if new_notes.is_empty() {
                    tracing::debug!("No new notes extracted");
                    continue;
                }

                // Fix #5: validate + save to DB
                match save_notes_to_db(&note_store, meeting_id, &new_notes) {
                    Ok(inserted_ids) => {
                        let total_count = new_notes.count();
                        let payload = NotesUpdatedPayload {
                            meeting_id,
                            new_notes,
                            total_count,
                            inserted_ids,
                        };
                        let _ = app.emit("notes-updated", payload);
                        tracing::info!("Emitted notes-updated with {} notes", total_count);
                    }
                    Err(e) => {
                        tracing::error!("Failed to save notes: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Note generation failed: {}", e);
                // Fix #6: emit error event to frontend
                let _ = app.emit(
                    "notes-error",
                    NotesErrorPayload {
                        meeting_id,
                        error: e.to_string(),
                    },
                );
            }
        }
    }

    tracing::info!("Note generation loop stopped");
}

/// Save incremental notes to database. Returns inserted row IDs.
/// Fix #5: Truncates oversized LLM content before DB insert.
fn save_notes_to_db(
    store: &crate::storage::NoteStore,
    meeting_id: i64,
    notes: &crate::notes::IncrementalNotesResponse,
) -> Result<Vec<i64>, String> {
    let mut batch = Vec::new();

    for kp in &notes.key_points {
        let content = truncate_content(&serde_json::to_string(kp).map_err(|e| e.to_string())?);
        batch.push((meeting_id, "key_point".to_string(), content));
    }

    for dec in &notes.decisions {
        let content = truncate_content(&serde_json::to_string(dec).map_err(|e| e.to_string())?);
        batch.push((meeting_id, "decision".to_string(), content));
    }

    for action in &notes.action_items {
        let content =
            truncate_content(&serde_json::to_string(action).map_err(|e| e.to_string())?);
        batch.push((meeting_id, "action_item".to_string(), content));
    }

    for risk in &notes.risks {
        let content = truncate_content(&serde_json::to_string(risk).map_err(|e| e.to_string())?);
        batch.push((meeting_id, "risk".to_string(), content));
    }

    store.insert_notes_batch(batch)
}

/// Truncate note content to MAX_NOTE_CONTENT_LEN bytes (UTF-8 safe).
fn truncate_content(s: &str) -> String {
    if s.len() <= MAX_NOTE_CONTENT_LEN {
        s.to_string()
    } else {
        let mut end = MAX_NOTE_CONTENT_LEN;
        while !s.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        tracing::warn!("Truncating note content from {} to {} bytes", s.len(), end);
        s[..end].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_stt_state() {
        let state = SttState::new(std::path::PathBuf::from("/tmp"));
        assert!(state.pipeline.lock().unwrap().is_none());
        assert!(state.engine.lock().unwrap().is_none());
    }
}
