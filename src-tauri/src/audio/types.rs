use serde::{Deserialize, Serialize};

/// Information about an audio device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub is_input: bool,
    pub is_loopback: bool,
    pub sample_rate: u32,
    pub channels: u16,
}

/// Audio capture configuration.
#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub target_sample_rate: u32,
    pub target_channels: u16,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            target_sample_rate: 16000,
            target_channels: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = AudioConfig::default();
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.target_sample_rate, 16000);
        assert_eq!(config.channels, 2);
        assert_eq!(config.target_channels, 1);
    }
}
