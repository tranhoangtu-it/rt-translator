use crate::audio::{list_devices, AudioCaptureManager, DeviceInfo};
use std::sync::{Arc, Mutex};
use tauri::ipc::Channel;
use tauri::State;

/// Application state for audio capture.
pub struct AudioState {
    pub manager: Arc<Mutex<Option<AudioCaptureManager>>>,
}

impl AudioState {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for AudioState {
    fn default() -> Self {
        Self::new()
    }
}

/// List all available audio devices (input + loopback).
#[tauri::command]
pub fn list_audio_devices() -> Result<Vec<DeviceInfo>, String> {
    list_devices().map_err(|e| format!("Failed to list devices: {}", e))
}

/// Start audio capture. Audio chunks (PCM f32 LE bytes) stream via `on_audio` channel.
#[tauri::command]
pub fn start_audio_capture(
    _device_id: Option<String>,
    on_audio: Channel<Vec<u8>>,
    state: State<AudioState>,
) -> Result<String, String> {
    let mut guard = state.manager.lock().unwrap();

    if guard.is_some() {
        return Err("Audio capture already running. Stop first.".to_string());
    }

    let mut manager = AudioCaptureManager::new();

    // Channel bridge: crossbeam → Tauri Channel
    let (output_tx, output_rx) = crossbeam::channel::bounded::<Vec<u8>>(100);

    manager
        .start(output_tx)
        .map_err(|e| format!("Failed to start capture: {}", e))?;

    *guard = Some(manager);
    let manager_ref = Arc::clone(&state.manager);

    // Forward audio from crossbeam channel to Tauri IPC channel.
    // When the frontend disconnects, auto-stop the capture manager.
    std::thread::Builder::new()
        .name("audio-ipc-bridge".to_string())
        .spawn(move || {
            while let Ok(chunk) = output_rx.recv() {
                if on_audio.send(chunk).is_err() {
                    tracing::warn!("Frontend channel closed, stopping capture");
                    if let Ok(mut guard) = manager_ref.lock() {
                        if let Some(mgr) = guard.as_mut() {
                            mgr.stop();
                        }
                        *guard = None;
                    }
                    break;
                }
            }
        })
        .map_err(|e| format!("Failed to start IPC bridge: {}", e))?;

    Ok("Audio capture started".to_string())
}

/// Stop audio capture.
#[tauri::command]
pub fn stop_audio_capture(state: State<AudioState>) -> Result<String, String> {
    let mut guard = state.manager.lock().unwrap();

    match guard.take() {
        Some(mut manager) => {
            manager.stop();
            Ok("Audio capture stopped".to_string())
        }
        None => Err("Audio capture not running".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_audio_state() {
        let state = AudioState::new();
        assert!(state.manager.lock().unwrap().is_none());
    }
}
