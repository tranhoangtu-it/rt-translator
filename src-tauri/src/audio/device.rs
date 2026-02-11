#![allow(deprecated)] // cpal 0.17 deprecates name() in favor of description()

use crate::audio::types::DeviceInfo;
use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait};

/// List all available audio input devices and loopback-capable output devices.
pub fn list_devices() -> Result<Vec<DeviceInfo>> {
    let mut devices = Vec::new();
    let host = cpal::default_host();

    // Input devices (microphones)
    if let Ok(input_devices) = host.input_devices() {
        for device in input_devices {
            let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
            let config = match device.default_input_config() {
                Ok(c) => c,
                Err(_) => continue,
            };

            devices.push(DeviceInfo {
                id: name.clone(),
                name,
                is_input: true,
                is_loopback: false,
                sample_rate: config.sample_rate(),
                channels: config.channels(),
            });
        }
    }

    // Output devices (for loopback capture on Windows)
    #[cfg(windows)]
    if let Ok(output_devices) = host.output_devices() {
        for device in output_devices {
            let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
            let config = match device.default_output_config() {
                Ok(c) => c,
                Err(_) => continue,
            };

            devices.push(DeviceInfo {
                id: format!("loopback:{}", name),
                name: format!("{} (Loopback)", name),
                is_input: false,
                is_loopback: true,
                sample_rate: config.sample_rate(),
                channels: config.channels(),
            });
        }
    }

    Ok(devices)
}

/// Get the default input device (microphone).
pub fn get_default_input_device() -> Result<cpal::Device> {
    let host = cpal::default_host();
    host.default_input_device()
        .ok_or_else(|| anyhow::anyhow!("No default input device found"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_devices_returns_ok() {
        let result = list_devices();
        assert!(result.is_ok());
    }
}
