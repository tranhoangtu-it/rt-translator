import { invoke, Channel } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef } from "react";
import { useAppStore } from "@/stores/app-store";
import type { DeviceInfo } from "@/types";

export function useAudioCapture() {
  const channelRef = useRef<Channel<number[]> | null>(null);
  const {
    audio,
    setAudioDevices,
    setCapturing,
    setAudioError,
    setSelectedDevice,
  } = useAppStore();

  const fetchDevices = useCallback(async () => {
    try {
      const devices = await invoke<DeviceInfo[]>("list_audio_devices");
      setAudioDevices(devices);

      // Auto-select first input device
      const firstInput = devices.find((d) => d.is_input);
      if (firstInput && !audio.selectedDeviceId) {
        setSelectedDevice(firstInput.id);
      }
    } catch (err) {
      setAudioError(`Failed to fetch devices: ${err}`);
    }
  }, [audio.selectedDeviceId, setAudioDevices, setAudioError, setSelectedDevice]);

  // Fetch devices on mount
  useEffect(() => {
    fetchDevices();
  }, [fetchDevices]);

  // Cleanup: stop capture when component unmounts
  useEffect(() => {
    return () => {
      invoke("stop_audio_capture").catch(() => {
        // Ignore errors — capture may not be running
      });
      channelRef.current = null;
    };
  }, []);

  const startCapture = useCallback(async () => {
    try {
      setAudioError(null);

      const channel = new Channel<number[]>();
      channelRef.current = channel;

      channel.onmessage = (audioBytes: number[]) => {
        const bytes = new Uint8Array(audioBytes);
        const floats = new Float32Array(bytes.buffer);
        console.log(
          `Audio chunk: ${floats.length} samples`,
          floats.slice(0, 5),
        );
      };

      await invoke("start_audio_capture", {
        deviceId: audio.selectedDeviceId,
        onAudio: channel,
      });

      setCapturing(true);
    } catch (err) {
      setAudioError(`Failed to start capture: ${err}`);
      setCapturing(false);
    }
  }, [audio.selectedDeviceId, setAudioError, setCapturing]);

  const stopCapture = useCallback(async () => {
    try {
      await invoke("stop_audio_capture");
      setCapturing(false);
      channelRef.current = null;
    } catch (err) {
      setAudioError(`Failed to stop capture: ${err}`);
    }
  }, [setAudioError, setCapturing]);

  return {
    devices: audio.devices,
    selectedDeviceId: audio.selectedDeviceId,
    isCapturing: audio.isCapturing,
    error: audio.error,
    fetchDevices,
    startCapture,
    stopCapture,
    setSelectedDevice,
  };
}
