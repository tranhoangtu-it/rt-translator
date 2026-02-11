#![allow(deprecated)] // cpal 0.17 deprecates name() in favor of description()

use crate::audio::device;
use anyhow::{Context, Result};
use crossbeam::channel::{bounded, Receiver, Sender};
use cpal::traits::{DeviceTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

/// Manages dual-stream audio capture (mic + system loopback).
/// Audio callbacks run on dedicated OS threads; data flows through crossbeam channels.
pub struct AudioCaptureManager {
    mic_tx: Sender<Vec<f32>>,
    mic_rx: Receiver<Vec<f32>>,
    #[cfg(windows)]
    loopback_tx: Sender<Vec<u8>>,
    #[cfg(windows)]
    loopback_rx: Receiver<Vec<u8>>,
    output_tx: Option<Sender<Vec<u8>>>,
    stt_tx: Arc<Mutex<Option<Sender<Vec<f32>>>>>,
    is_running: Arc<AtomicBool>,
    mic_stream: Option<cpal::Stream>,
    mic_sample_rate: u32,
    mic_channels: u16,
}

impl AudioCaptureManager {
    pub fn new() -> Self {
        let (mic_tx, mic_rx) = bounded(100);
        #[cfg(windows)]
        let (loopback_tx, loopback_rx) = bounded(100);

        Self {
            mic_tx,
            mic_rx,
            #[cfg(windows)]
            loopback_tx,
            #[cfg(windows)]
            loopback_rx,
            output_tx: None,
            stt_tx: Arc::new(Mutex::new(None)),
            is_running: Arc::new(AtomicBool::new(false)),
            mic_stream: None,
            mic_sample_rate: 0,
            mic_channels: 0,
        }
    }

    /// Start capturing audio. Output bytes (PCM f32 LE) are sent to `output_tx`.
    pub fn start(&mut self, output_tx: Sender<Vec<u8>>) -> Result<()> {
        self.is_running.store(true, Ordering::SeqCst);
        self.output_tx = Some(output_tx);

        self.start_mic_stream()?;

        #[cfg(windows)]
        self.start_wasapi_loopback()?;

        #[cfg(not(windows))]
        tracing::warn!("System audio loopback not implemented on this platform");

        self.start_processor_thread()?;
        Ok(())
    }

    /// Set a sender for forwarding raw f32 audio to the STT pipeline.
    /// Thread-safe: the processor thread sees updates dynamically via Arc<Mutex>.
    pub fn set_stt_sender(&self, tx: Sender<Vec<f32>>) {
        if let Ok(mut guard) = self.stt_tx.lock() {
            *guard = Some(tx);
        }
    }

    /// Remove the STT sender (stops forwarding audio to STT).
    pub fn clear_stt_sender(&self) {
        if let Ok(mut guard) = self.stt_tx.lock() {
            *guard = None;
        }
    }

    /// Returns the mic sample rate and channel count for resampling purposes.
    pub fn mic_format(&self) -> (u32, u16) {
        (self.mic_sample_rate, self.mic_channels)
    }

    /// Stop all capture streams and processor thread.
    pub fn stop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);
        // Drop the cpal stream to release audio device resources
        if let Some(stream) = self.mic_stream.take() {
            drop(stream);
        }
        self.output_tx = None;
        self.clear_stt_sender();
        tracing::info!("Audio capture stopped");
    }

    fn start_mic_stream(&mut self) -> Result<()> {
        let device = device::get_default_input_device()?;
        let config = device
            .default_input_config()
            .context("Failed to get default input config")?;

        self.mic_sample_rate = config.sample_rate();
        self.mic_channels = config.channels();

        tracing::info!(
            "Mic: {} ch, {}Hz, {:?}",
            config.channels(),
            config.sample_rate(),
            config.sample_format()
        );

        let tx = self.mic_tx.clone();
        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if tx.try_send(data.to_vec()).is_err() {
                    tracing::warn!("Mic buffer full, dropping frame");
                }
            },
            |err| tracing::error!("Mic stream error: {}", err),
            None,
        )?;

        stream.play()?;
        self.mic_stream = Some(stream);
        tracing::info!("Mic stream started");
        Ok(())
    }

    #[cfg(windows)]
    fn start_wasapi_loopback(&self) -> Result<()> {
        let tx = self.loopback_tx.clone();
        let is_running = self.is_running.clone();

        thread::Builder::new()
            .name("wasapi-loopback".to_string())
            .spawn(move || {
                if let Err(e) = wasapi_loopback_thread(tx, is_running) {
                    tracing::error!("WASAPI loopback error: {}", e);
                }
            })?;

        tracing::info!("WASAPI loopback thread started");
        Ok(())
    }

    fn start_processor_thread(&self) -> Result<()> {
        let mic_rx = self.mic_rx.clone();
        let output_tx = self
            .output_tx
            .clone()
            .context("Output channel not set")?;
        let stt_tx = self.stt_tx.clone();
        let is_running = self.is_running.clone();

        #[cfg(windows)]
        let _loopback_rx = self.loopback_rx.clone();

        thread::Builder::new()
            .name("audio-processor".to_string())
            .spawn(move || {
                processor_thread(mic_rx, output_tx, stt_tx, is_running);
            })?;

        tracing::info!("Audio processor thread started");
        Ok(())
    }
}

// Re-export mic_format for use in commands
impl AudioCaptureManager {
    /// Check if capture is active.
    pub fn is_active(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
}

impl Default for AudioCaptureManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Forward mic PCM data as raw f32 LE bytes to the output channel,
/// and optionally fork a copy of f32 samples to the STT pipeline.
/// `stt_tx` is behind Arc<Mutex> so set_stt_sender/clear_stt_sender are visible dynamically.
fn processor_thread(
    mic_rx: Receiver<Vec<f32>>,
    output_tx: Sender<Vec<u8>>,
    stt_tx: Arc<Mutex<Option<Sender<Vec<f32>>>>>,
    is_running: Arc<AtomicBool>,
) {
    while is_running.load(Ordering::SeqCst) {
        match mic_rx.recv_timeout(std::time::Duration::from_millis(50)) {
            Ok(samples) => {
                // Fork: send f32 copy to STT pipeline if connected
                if let Ok(guard) = stt_tx.lock() {
                    if let Some(ref tx) = *guard {
                        let _ = tx.try_send(samples.clone());
                    }
                }

                // Convert to LE bytes for IPC bridge
                let bytes: Vec<u8> =
                    samples.iter().flat_map(|&s| s.to_le_bytes()).collect();

                if output_tx.try_send(bytes).is_err() {
                    tracing::warn!("Output buffer full, dropping frame");
                }
            }
            Err(crossbeam::channel::RecvTimeoutError::Timeout) => continue,
            Err(crossbeam::channel::RecvTimeoutError::Disconnected) => break,
        }
    }
    tracing::info!("Audio processor thread exiting");
}

/// WASAPI loopback capture thread (Windows only).
/// Gets the default Render device, initializes for Capture direction → WASAPI auto-sets loopback.
#[cfg(windows)]
fn wasapi_loopback_thread(
    tx: Sender<Vec<u8>>,
    is_running: Arc<AtomicBool>,
) -> Result<()> {
    use std::collections::VecDeque;
    use wasapi::*;

    // initialize_mta returns HRESULT; .ok() converts to WasapiRes
    initialize_mta()
        .ok()
        .map_err(|e| anyhow::anyhow!("COM init failed: {}", e))?;

    let enumerator = DeviceEnumerator::new()?;
    let device = enumerator.get_default_device(&Direction::Render)?;
    let mut audio_client = device.get_iaudioclient()?;

    // Get device's native format
    let mix_format = device.get_device_format()?;
    tracing::info!("WASAPI loopback format: {:?}", mix_format);

    let (def_time, _min_time) = audio_client.get_device_period()?;

    // EventsShared with Render device + Capture direction → loopback mode
    let mode = StreamMode::EventsShared {
        autoconvert: true,
        buffer_duration_hns: def_time,
    };

    audio_client.initialize_client(&mix_format, &Direction::Capture, &mode)?;

    let h_event = audio_client.set_get_eventhandle()?;
    let capture_client = audio_client.get_audiocaptureclient()?;
    audio_client.start_stream()?;

    tracing::info!("WASAPI loopback stream started");

    let mut sample_queue: VecDeque<u8> = VecDeque::with_capacity(16384);

    while is_running.load(Ordering::SeqCst) {
        // Read captured data into deque
        match capture_client.read_from_device_to_deque(&mut sample_queue) {
            Ok(_buffer_info) => {
                if !sample_queue.is_empty() {
                    let data: Vec<u8> = sample_queue.drain(..).collect();
                    if tx.try_send(data).is_err() {
                        tracing::warn!("Loopback buffer full, dropping frame");
                    }
                }
            }
            Err(e) => {
                tracing::warn!("WASAPI read error: {}", e);
            }
        }

        // Wait for next audio event (100ms timeout)
        let _ = h_event.wait_for_event(100);
    }

    audio_client.stop_stream()?;
    tracing::info!("WASAPI loopback thread exiting");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_manager() {
        let manager = AudioCaptureManager::new();
        assert!(!manager.is_running.load(Ordering::SeqCst));
    }

    #[test]
    fn stop_sets_flag() {
        let mut manager = AudioCaptureManager::new();
        manager.is_running.store(true, Ordering::SeqCst);
        manager.stop();
        assert!(!manager.is_running.load(Ordering::SeqCst));
    }
}
