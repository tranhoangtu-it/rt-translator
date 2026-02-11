pub mod audio;
pub mod commands;
#[allow(dead_code)]
pub mod notes;
pub mod providers;
pub mod storage;
pub mod stt;
pub mod translation;

use commands::{
    get_app_version, get_settings, health_check,
    list_audio_devices, start_audio_capture, stop_audio_capture,
    check_model_status, download_model, start_meeting, stop_meeting,
    ollama_health_check, translate_text, list_ollama_models,
    pull_ollama_model, delete_ollama_model,
    export_transcript,
    open_overlay_window, close_overlay_window,
    get_notes, update_note, delete_note, generate_memo, export_memo,
    AudioState, SttState, TranslationState, NoteState,
};
use storage::NoteStore;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rt_translator_lib=info".into()),
        )
        .init();

    let migrations = storage::migrations::get_migrations();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:translator.db", migrations)
                .build(),
        )
        .manage(AudioState::new())
        .manage(TranslationState::default())
        .setup(|app| {
            use tauri::Manager;
            let app_data_dir = app.path().app_data_dir()?;
            let stt_state = SttState::new(app_data_dir.clone());

            // Share DB connection between SttState and NoteState
            let db_conn = stt_state.transcript_db.get_connection();
            let note_store = NoteStore::new(db_conn.clone());
            let note_state = NoteState {
                store: note_store,
                transcript_db: stt_state.transcript_db.clone(),
            };

            app.manage(stt_state);
            app.manage(note_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_version,
            health_check,
            get_settings,
            list_audio_devices,
            start_audio_capture,
            stop_audio_capture,
            check_model_status,
            download_model,
            start_meeting,
            stop_meeting,
            ollama_health_check,
            translate_text,
            list_ollama_models,
            pull_ollama_model,
            delete_ollama_model,
            export_transcript,
            open_overlay_window,
            close_overlay_window,
            get_notes,
            update_note,
            delete_note,
            generate_memo,
            export_memo,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
