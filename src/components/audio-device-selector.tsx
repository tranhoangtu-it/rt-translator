import { useAudioCapture } from "@/hooks/use-audio-capture";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Mic } from "lucide-react";

export function AudioDeviceSelector() {
  const {
    devices,
    selectedDeviceId,
    isCapturing,
    error,
    startCapture,
    stopCapture,
    setSelectedDevice,
    fetchDevices,
  } = useAudioCapture();

  const inputDevices = devices.filter((d) => d.is_input);

  return (
    <div className="flex flex-col gap-4 rounded-lg border p-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Mic className="h-5 w-5 text-primary" />
          <h3 className="text-lg font-semibold">Audio Capture</h3>
          {isCapturing && (
            <div className="h-2 w-2 animate-pulse rounded-full bg-green-500" />
          )}
        </div>
        <button
          onClick={fetchDevices}
          className="cursor-pointer text-xs text-muted-foreground hover:underline"
          type="button"
        >
          Refresh devices
        </button>
      </div>

      {error && (
        <div className="rounded-md bg-red-50 p-3 text-sm text-red-700 dark:bg-red-950 dark:text-red-300">
          {error}
        </div>
      )}

      <div className="flex flex-col gap-2">
        <label htmlFor="audio-device" className="text-sm font-medium">
          Input Device
        </label>
        <Select
          value={selectedDeviceId ?? ""}
          onValueChange={(value) => setSelectedDevice(value || null)}
          disabled={isCapturing}
        >
          <SelectTrigger className="cursor-pointer">
            <SelectValue placeholder="Select audio device..." />
          </SelectTrigger>
          <SelectContent>
            {inputDevices.map((device) => (
              <SelectItem
                key={device.id}
                value={device.id}
                className="cursor-pointer"
              >
                {device.name} ({device.sample_rate}Hz, {device.channels}ch)
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      <div className="flex gap-2">
        <Button
          onClick={startCapture}
          disabled={isCapturing || !selectedDeviceId}
          variant="default"
          className="cursor-pointer"
        >
          Start Capture
        </Button>
        <Button
          onClick={stopCapture}
          disabled={!isCapturing}
          variant="outline"
          className="cursor-pointer"
        >
          Stop Capture
        </Button>
      </div>

      {isCapturing && (
        <p className="text-sm text-green-600 dark:text-green-400">
          Capturing audio... (check console for data)
        </p>
      )}
    </div>
  );
}
