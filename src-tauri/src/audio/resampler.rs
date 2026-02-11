use anyhow::Result;
use rubato::audioadapter::Adapter;
use rubato::{Fft, FixedSync, Resampler};

/// Wraps rubato's FFT resampler for audio rate conversion and channel downmix.
pub struct AudioResampler {
    resampler: Fft<f32>,
    input_channels: usize,
}

impl AudioResampler {
    /// Create a new resampler.
    /// `chunk_size` is the number of frames per channel per processing call.
    pub fn new(
        input_rate: u32,
        output_rate: u32,
        channels: usize,
        chunk_size: usize,
    ) -> Result<Self> {
        let resampler = Fft::<f32>::new(
            input_rate as usize,
            output_rate as usize,
            chunk_size,
            2, // sub_chunks
            channels,
            FixedSync::Input,
        )?;

        Ok(Self {
            resampler,
            input_channels: channels,
        })
    }

    /// Resample interleaved stereo input to mono output.
    /// Input: interleaved [L0, R0, L1, R1, ...] with `chunk_size` frames.
    /// Output: mono resampled samples.
    pub fn process_stereo_to_mono(&mut self, input: &[f32]) -> Result<Vec<f32>> {
        let frames = input.len() / 2;
        let left: Vec<f32> = input.iter().step_by(2).copied().collect();
        let right: Vec<f32> = input.iter().skip(1).step_by(2).copied().collect();
        let planar = vec![left, right];

        let resampled = self.resampler.process(
            &audioadapter_buffers::direct::SequentialSliceOfVecs::new(&planar, 2, frames)?,
            0,
            None,
        )?;

        // Average L+R channels to mono using Adapter trait
        let out_frames = resampled.frames();
        let mut mono = Vec::with_capacity(out_frames);
        for i in 0..out_frames {
            let l = resampled.read_sample(0, i).unwrap_or(0.0);
            let r = resampled.read_sample(1, i).unwrap_or(0.0);
            mono.push((l + r) / 2.0);
        }
        Ok(mono)
    }

    /// Resample mono input to mono output.
    pub fn process_mono(&mut self, input: &[f32]) -> Result<Vec<f32>> {
        let frames = input.len();
        let planar = vec![input.to_vec()];

        let resampled = self.resampler.process(
            &audioadapter_buffers::direct::SequentialSliceOfVecs::new(&planar, 1, frames)?,
            0,
            None,
        )?;

        let out_frames = resampled.frames();
        let mut mono = Vec::with_capacity(out_frames);
        for i in 0..out_frames {
            mono.push(resampled.read_sample(0, i).unwrap_or(0.0));
        }
        Ok(mono)
    }

    /// Number of input frames required per process call.
    pub fn input_frames_next(&self) -> usize {
        self.resampler.input_frames_next()
    }

    pub fn input_channels(&self) -> usize {
        self.input_channels
    }
}

/// Mix two equal-length audio buffers by averaging. Clamps to [-1.0, 1.0].
pub fn mix_audio(a: &[f32], b: &[f32]) -> Vec<f32> {
    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| ((x + y) / 2.0).clamp(-1.0, 1.0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mix_audio_averages_and_clamps() {
        let a = vec![0.5, -0.3, 1.0];
        let b = vec![0.2, 0.4, 0.8];
        let mixed = mix_audio(&a, &b);

        assert!((mixed[0] - 0.35).abs() < f32::EPSILON);
        assert!((mixed[1] - 0.05).abs() < f32::EPSILON);
        assert!((mixed[2] - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn mix_audio_clamps_overflow() {
        let a = vec![1.0];
        let b = vec![1.0];
        let mixed = mix_audio(&a, &b);
        assert_eq!(mixed[0], 1.0);
    }
}
