use crate::audio::resampler::AudioResampler;
use crate::audio::vad::{EnergyVad, SpeechBuffer, VadConfig, VadEvent};
use crate::notes::{SegmentBuffer, TranscriptSegment};
use crate::storage::TranscriptDb;
use crate::stt::whisper::SttEngine;
use crossbeam::channel::Receiver;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use tauri::Emitter;

/// Orchestrates VAD → STT → event emission pipeline.
/// Runs on a dedicated thread, receives f32 audio from AudioCaptureManager.
pub struct SttPipeline {
    is_running: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
}

/// Audio format info for resampling raw mic input to 16kHz mono.
pub struct MicFormat {
    pub sample_rate: u32,
    pub channels: u16,
}

impl SttPipeline {
    /// Start the pipeline on a dedicated thread.
    /// `audio_rx`: receives f32 audio chunks from the audio processor thread.
    /// `engine`: shared whisper-rs STT engine.
    /// `app`: Tauri AppHandle for emitting events.
    /// `mic_format`: mic sample rate + channels for resampling to 16kHz mono.
    pub fn start(
        audio_rx: Receiver<Vec<f32>>,
        engine: Arc<SttEngine>,
        app: tauri::AppHandle,
        mic_format: MicFormat,
        transcript_db: TranscriptDb,
        meeting_id: Arc<Mutex<Option<i64>>>,
        segment_buffer: SegmentBuffer,
    ) -> Self {
        let is_running = Arc::new(AtomicBool::new(true));
        let flag = is_running.clone();

        let handle = std::thread::Builder::new()
            .name("stt-pipeline".to_string())
            .spawn(move || {
                pipeline_loop(
                    audio_rx,
                    engine,
                    app,
                    flag,
                    mic_format,
                    transcript_db,
                    meeting_id,
                    segment_buffer,
                );
            })
            .expect("Failed to spawn stt-pipeline thread");

        tracing::info!("STT pipeline started");
        Self {
            is_running,
            thread_handle: Some(handle),
        }
    }

    /// Stop the pipeline and wait for the thread to exit.
    pub fn stop(mut self) {
        self.is_running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
        tracing::info!("STT pipeline stopped");
    }
}

impl Default for SttPipeline {
    fn default() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        }
    }
}

/// Main pipeline loop: resample → VAD frames → accumulate speech → STT on silence → emit events.
fn pipeline_loop(
    audio_rx: Receiver<Vec<f32>>,
    engine: Arc<SttEngine>,
    app: tauri::AppHandle,
    is_running: Arc<AtomicBool>,
    mic_format: MicFormat,
    transcript_db: TranscriptDb,
    meeting_id: Arc<Mutex<Option<i64>>>,
    segment_buffer: SegmentBuffer,
) {
    let mut vad = EnergyVad::new(VadConfig::default());
    let mut buffer = SpeechBuffer::new(16000, 30);
    let mut segment_counter: u32 = 0;
    let start_time = std::time::Instant::now();

    // Setup resampler if mic isn't already 16kHz mono
    let needs_resample = mic_format.sample_rate != 16000 || mic_format.channels != 1;
    let chunk_size = 1024_usize;
    let mut resampler = if needs_resample {
        match AudioResampler::new(
            mic_format.sample_rate,
            16000,
            mic_format.channels as usize,
            chunk_size,
        ) {
            Ok(r) => {
                tracing::info!(
                    "STT resampler: {}Hz {}ch -> 16kHz mono",
                    mic_format.sample_rate,
                    mic_format.channels,
                );
                Some(r)
            }
            Err(e) => {
                tracing::error!("Failed to create resampler: {}", e);
                return;
            }
        }
    } else {
        tracing::info!("Mic already 16kHz mono, no resampling needed");
        None
    };

    // Resampler input buffer: accumulate enough frames for a full chunk
    let input_chunk_samples = chunk_size * mic_format.channels as usize;
    let mut resample_buf: Vec<f32> = Vec::with_capacity(input_chunk_samples * 2);

    // Process audio in 10ms frames (160 samples at 16kHz)
    const FRAME_SIZE: usize = 160;
    let mut frame_buf: Vec<f32> = Vec::with_capacity(FRAME_SIZE * 4);

    while is_running.load(Ordering::SeqCst) {
        match audio_rx.recv_timeout(std::time::Duration::from_millis(50)) {
            Ok(samples) => {
                // Resample to 16kHz mono if needed, otherwise pass through
                let mono_16k = if let Some(ref mut rs) = resampler {
                    resample_buf.extend_from_slice(&samples);
                    let mut out = Vec::new();
                    while resample_buf.len() >= input_chunk_samples {
                        let chunk: Vec<f32> =
                            resample_buf.drain(..input_chunk_samples).collect();
                        let resampled = if mic_format.channels >= 2 {
                            rs.process_stereo_to_mono(&chunk)
                        } else {
                            rs.process_mono(&chunk)
                        };
                        match resampled {
                            Ok(data) => out.extend(data),
                            Err(e) => {
                                tracing::warn!("Resample error: {}", e);
                                continue;
                            }
                        }
                    }
                    out
                } else {
                    samples
                };

                frame_buf.extend_from_slice(&mono_16k);

                // Process complete frames
                while frame_buf.len() >= FRAME_SIZE {
                    let frame: Vec<f32> =
                        frame_buf.drain(..FRAME_SIZE).collect();

                    let event = vad.process_frame(&frame);
                    match event {
                        VadEvent::Speech => {
                            buffer.push(&frame);
                        }
                        VadEvent::Silence => {
                            // Keep padding a bit of silence for natural trailing
                        }
                        VadEvent::SpeechEnd => {
                            if let Some(audio) = buffer.take() {
                                let base_ms = start_time.elapsed().as_millis() as u64;
                                run_stt_and_emit(
                                    &engine,
                                    &app,
                                    &audio,
                                    base_ms,
                                    &mut segment_counter,
                                    &transcript_db,
                                    &meeting_id,
                                    &segment_buffer,
                                );
                            }
                        }
                    }

                    // Safety cap: force STT if buffer too long
                    if buffer.is_full() {
                        tracing::warn!("Speech buffer at max cap, forcing STT");
                        if let Some(audio) = buffer.take() {
                            let base_ms = start_time.elapsed().as_millis() as u64;
                            run_stt_and_emit(
                                &engine,
                                &app,
                                &audio,
                                base_ms,
                                &mut segment_counter,
                                &transcript_db,
                                &meeting_id,
                                &segment_buffer,
                            );
                        }
                        vad.reset();
                    }
                }
            }
            Err(crossbeam::channel::RecvTimeoutError::Timeout) => continue,
            Err(crossbeam::channel::RecvTimeoutError::Disconnected) => {
                tracing::info!("STT audio channel disconnected");
                break;
            }
        }
    }

    // Process any remaining buffer
    if let Some(audio) = buffer.take() {
        let base_ms = start_time.elapsed().as_millis() as u64;
        run_stt_and_emit(
            &engine,
            &app,
            &audio,
            base_ms,
            &mut segment_counter,
            &transcript_db,
            &meeting_id,
            &segment_buffer,
        );
    }

    tracing::info!("STT pipeline loop exiting");
}

/// Run whisper inference, emit results as Tauri events, and insert into DB.
fn run_stt_and_emit(
    engine: &SttEngine,
    app: &tauri::AppHandle,
    audio: &[f32],
    base_time_ms: u64,
    segment_counter: &mut u32,
    transcript_db: &TranscriptDb,
    meeting_id: &Arc<Mutex<Option<i64>>>,
    segment_buffer: &SegmentBuffer,
) {
    if audio.is_empty() {
        return;
    }

    tracing::debug!(
        "Running STT on {} samples ({:.1}s)",
        audio.len(),
        audio.len() as f64 / 16000.0,
    );

    match engine.transcribe_sync(audio, base_time_ms) {
        Ok(segments) => {
            for seg in segments {
                *segment_counter += 1;
                // Build canonical segment_id — single source of truth
                let seg_id = format!("seg-{}-{}", *segment_counter, seg.start_ms);

                let payload = serde_json::json!({
                    "text": seg.text,
                    "language": seg.lang,
                    "start_ms": seg.start_ms,
                    "end_ms": seg.end_ms,
                    "is_final": seg.is_final,
                    "segment_id": seg_id,
                });

                if let Err(e) = app.emit("stt-partial", payload) {
                    tracing::warn!("Failed to emit stt-partial: {}", e);
                }

                // Insert final segments into DB and feed to NoteEngine
                if seg.is_final {
                    if let Ok(guard) = meeting_id.lock() {
                        if let Some(mid) = *guard {
                            if let Err(e) = transcript_db.insert_transcript(
                                mid,
                                &seg.text,
                                &seg_id,
                                seg.start_ms as i64,
                            ) {
                                tracing::error!("Failed to insert transcript: {}", e);
                            }
                        }
                    }

                    // Feed segment to buffer (std::sync::Mutex — fast, no async)
                    if let Ok(mut buf) = segment_buffer.lock() {
                        buf.push(TranscriptSegment {
                            text: seg.text.clone(),
                            timestamp_ms: seg.start_ms as i64,
                            segment_id: seg_id.clone(),
                        });
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!("STT inference failed: {}", e);
        }
    }
}
