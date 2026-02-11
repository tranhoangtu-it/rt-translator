/// Configuration for energy-based Voice Activity Detection.
pub struct VadConfig {
    /// RMS threshold below which a frame is considered silence (default 0.02).
    pub rms_threshold: f32,
    /// Zero-crossing rate threshold (default 0.1).
    pub zcr_threshold: f32,
    /// How many consecutive silence frames before triggering SpeechEnd.
    pub silence_limit: usize,
}

impl Default for VadConfig {
    fn default() -> Self {
        // 500ms silence at 10ms frames = 50 frames
        Self {
            rms_threshold: 0.02,
            zcr_threshold: 0.1,
            silence_limit: 50,
        }
    }
}

/// Events produced by the VAD for each processed frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VadEvent {
    /// Frame contains speech.
    Speech,
    /// Frame is silence but threshold not yet reached.
    Silence,
    /// Sustained silence reached after speech — trigger STT inference.
    SpeechEnd,
}

/// Energy-based VAD using RMS + zero-crossing rate.
pub struct EnergyVad {
    config: VadConfig,
    silence_frames: usize,
    has_speech: bool,
}

impl EnergyVad {
    pub fn new(config: VadConfig) -> Self {
        Self {
            config,
            silence_frames: 0,
            has_speech: false,
        }
    }

    /// Process a frame of f32 samples. Returns a VadEvent.
    pub fn process_frame(&mut self, samples: &[f32]) -> VadEvent {
        if samples.is_empty() {
            return VadEvent::Silence;
        }

        let rms = compute_rms(samples);
        let zcr = compute_zcr(samples);

        let is_silence =
            rms < self.config.rms_threshold && zcr < self.config.zcr_threshold;

        if is_silence {
            self.silence_frames += 1;
            if self.has_speech && self.silence_frames >= self.config.silence_limit {
                self.has_speech = false;
                self.silence_frames = 0;
                VadEvent::SpeechEnd
            } else {
                VadEvent::Silence
            }
        } else {
            self.silence_frames = 0;
            self.has_speech = true;
            VadEvent::Speech
        }
    }

    pub fn reset(&mut self) {
        self.silence_frames = 0;
        self.has_speech = false;
    }
}

/// Accumulates speech samples until SpeechEnd, then yields the buffer.
pub struct SpeechBuffer {
    samples: Vec<f32>,
    sample_rate: u32,
    /// Safety cap: max duration in samples (default 30s).
    max_samples: usize,
}

impl SpeechBuffer {
    pub fn new(sample_rate: u32, max_duration_secs: u32) -> Self {
        Self {
            samples: Vec::with_capacity(sample_rate as usize * 5),
            sample_rate,
            max_samples: sample_rate as usize * max_duration_secs as usize,
        }
    }

    /// Append audio samples.
    pub fn push(&mut self, frame: &[f32]) {
        self.samples.extend_from_slice(frame);
    }

    /// Drain and return accumulated buffer if non-empty.
    pub fn take(&mut self) -> Option<Vec<f32>> {
        if self.samples.is_empty() {
            return None;
        }
        Some(std::mem::take(&mut self.samples))
    }

    /// Current buffer duration in milliseconds.
    pub fn duration_ms(&self) -> u64 {
        (self.samples.len() as u64 * 1000) / self.sample_rate as u64
    }

    /// Whether buffer exceeded the safety cap.
    pub fn is_full(&self) -> bool {
        self.samples.len() >= self.max_samples
    }

    pub fn clear(&mut self) {
        self.samples.clear();
    }
}

fn compute_rms(samples: &[f32]) -> f32 {
    let sum: f32 = samples.iter().map(|s| s * s).sum();
    (sum / samples.len() as f32).sqrt()
}

fn compute_zcr(samples: &[f32]) -> f32 {
    if samples.len() < 2 {
        return 0.0;
    }
    let crossings = samples
        .windows(2)
        .filter(|w| (w[0] * w[1]) < 0.0)
        .count();
    crossings as f32 / samples.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_detection_on_zeros() {
        let mut vad = EnergyVad::new(VadConfig::default());
        let silence = vec![0.0f32; 160];
        assert_eq!(vad.process_frame(&silence), VadEvent::Silence);
    }

    #[test]
    fn speech_detection_on_sine_wave() {
        let mut vad = EnergyVad::new(VadConfig::default());
        // Generate 160 samples of 440Hz sine at amplitude 0.5
        let samples: Vec<f32> = (0..160)
            .map(|i| 0.5 * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 16000.0).sin())
            .collect();
        assert_eq!(vad.process_frame(&samples), VadEvent::Speech);
    }

    #[test]
    fn speech_end_after_sustained_silence() {
        let config = VadConfig {
            silence_limit: 3,
            ..VadConfig::default()
        };
        let mut vad = EnergyVad::new(config);
        let speech: Vec<f32> = (0..160)
            .map(|i| 0.5 * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 16000.0).sin())
            .collect();
        let silence = vec![0.0f32; 160];

        // Speech frame first
        assert_eq!(vad.process_frame(&speech), VadEvent::Speech);
        // Then silence frames
        assert_eq!(vad.process_frame(&silence), VadEvent::Silence);
        assert_eq!(vad.process_frame(&silence), VadEvent::Silence);
        // Third silence frame hits limit -> SpeechEnd
        assert_eq!(vad.process_frame(&silence), VadEvent::SpeechEnd);
    }

    #[test]
    fn speech_buffer_accumulation() {
        let mut buf = SpeechBuffer::new(16000, 30);
        buf.push(&[0.1, 0.2, 0.3]);
        buf.push(&[0.4, 0.5]);
        assert_eq!(buf.duration_ms(), 0); // 5 samples at 16kHz < 1ms rounds to 0
        let taken = buf.take().unwrap();
        assert_eq!(taken.len(), 5);
        assert!(buf.take().is_none());
    }

    #[test]
    fn speech_buffer_max_duration_cap() {
        let mut buf = SpeechBuffer::new(16000, 1); // 1 second cap
        let chunk = vec![0.5f32; 16001];
        buf.push(&chunk);
        assert!(buf.is_full());
    }
}
