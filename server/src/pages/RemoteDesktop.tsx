import React, { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";
import { getCurrent, WebviewWindow } from "@tauri-apps/api/window";
import { useParams } from "react-router-dom";

interface RemoteDesktopFramePayload {
  addr: string;
  timestamp: number;
  display: number;
  data: string;
}

export const RemoteDesktop: React.FC = () => {
  const { addr } = useParams();

  const canvasRef = useRef<HTMLCanvasElement>(null);
  const lastFrameRef = useRef<HTMLImageElement | null>(null);
  const imageSizeRef = useRef<{ width: number; height: number }>({
    width: 0,
    height: 0,
  });
  const [showControls, setShowControls] = useState(true);
  const [streaming, setStreaming] = useState(false);
  const [quality, setQuality] = useState(35);
  const [fps, setFps] = useState(10);
  const [displays, setDisplays] = useState<number[]>([]);
  const [selectedDisplay, setSelectedDisplay] = useState(0);
  const [connectionStatus, setConnectionStatus] =
    useState<string>("Ready to connect");
  const currentWindow = useRef<WebviewWindow | null>(null);
  const [mouseControlEnabled, setMouseControlEnabled] = useState(false);
  const [keyboardControlEnabled, setKeyboardControlEnabled] = useState(false);

  useEffect(() => {
    currentWindow.current = getCurrent();
  }, [addr]);

  useEffect(() => {
    lastFrameRef.current = new Image();

    const grabDisplayData = async () => {
      try {
        const clientInfo: any = await invoke("fetch_client", { addr });
        if (clientInfo && clientInfo.displays) {
          const displayArray = Array.from(
            { length: clientInfo.displays },
            (_, i) => i
          );
          setDisplays(displayArray);
        }
      } catch (error) {
        console.error("Failed to initialize window", error);
      }
    };

    grabDisplayData();

    const setupCleanup = async () => {
      try {
        const window = getCurrent();

        await window.listen("tauri://close-requested", async () => {
          if (streaming) {
            await stopStreamingAndCleanup();
          }
          window.close();
        });
      } catch (error) {
        console.error("Error setting up window close handler:", error);
      }
    };

    setupCleanup();

    return () => {
      if (streaming) {
        stopStreamingAndCleanup();
      }
    };
  }, []);

  useEffect(() => {
    const unlisten = listen("remote_desktop_frame", (event) => {
      const payload = event.payload as RemoteDesktopFramePayload;

      if (payload.addr !== addr) {
        return;
      }

      if (payload.display !== selectedDisplay) {
        return;
      }

      if (canvasRef.current && lastFrameRef.current) {
        const ctx = canvasRef.current.getContext("2d");
        if (ctx) {
          lastFrameRef.current.src = `data:image/jpeg;base64,${payload.data}`;

          lastFrameRef.current.onload = () => {
            if (ctx && canvasRef.current && lastFrameRef.current) {
              imageSizeRef.current = {
                width: lastFrameRef.current.width,
                height: lastFrameRef.current.height,
              };

              canvasRef.current.width = lastFrameRef.current.width;
              canvasRef.current.height = lastFrameRef.current.height;

              ctx.drawImage(
                lastFrameRef.current,
                0,
                0,
                lastFrameRef.current.width,
                lastFrameRef.current.height
              );
            }
          };
        }
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [selectedDisplay]);

  const stopStreamingAndCleanup = async () => {
    if (!streaming) return;

    try {
      await invoke("stop_remote_desktop", { addr });
      setStreaming(false);
      setConnectionStatus("Disconnected");
    } catch (error) {
      console.error("Error cleaning up streaming:", error);
    }
  };

  const handleStartStreaming = async () => {
    setConnectionStatus("Connecting...");
    try {
      await invoke("start_remote_desktop", {
        addr,
        display: selectedDisplay,
        quality,
        fps,
      });
      setStreaming(true);
      setConnectionStatus("Connected");
    } catch (error) {
      console.error("Error starting remote desktop:", error);
      setConnectionStatus(" ");
    }
  };

  const handleStopStreaming = async () => {
    await stopStreamingAndCleanup();
    setConnectionStatus("Ready to connect");
  };

  useEffect(() => {
    const handleBeforeUnload = () => {
      if (streaming) {
        invoke("stop_remote_desktop", { addr }).catch((err) =>
          console.error("Error stopping remote desktop on close:", err)
        );
      }
    };

    window.addEventListener("beforeunload", handleBeforeUnload);

    return () => {
      window.removeEventListener("beforeunload", handleBeforeUnload);
    };
  }, [addr, streaming]);

  const switchDisplay = async (newDisplay: number) => {
    if (streaming) {
      await invoke("stop_remote_desktop", { addr });

      setSelectedDisplay(newDisplay);

      await invoke("start_remote_desktop", {
        addr,
        display: newDisplay,
        quality,
        fps,
      });
    } else {
      setSelectedDisplay(newDisplay);
    }
  };

  const handleCanvasClick = async (
    event: React.MouseEvent<HTMLCanvasElement>
  ) => {
    event.preventDefault();

    if (!mouseControlEnabled || !canvasRef.current || !addr || !streaming)
      return;

    if (event.button !== 0 && event.button !== 2) {
      return;
    }

    const rect = canvasRef.current.getBoundingClientRect();

    const scaleFactorX = imageSizeRef.current.width / rect.width;
    const scaleFactorY = imageSizeRef.current.height / rect.height;

    const canvasX = event.clientX - rect.left;
    const canvasY = event.clientY - rect.top;

    const targetX = Math.round(canvasX * scaleFactorX);
    const targetY = Math.round(canvasY * scaleFactorY);

    try {
      await invoke("send_mouse_click", {
        addr,
        display: selectedDisplay,
        x: targetX,
        y: targetY,
        clickType: event.button,
      });
    } catch (error) {
      console.error("Error sending mouse click:", error);
    }
  };

  const toggleKeyboardControl = () => {
    setKeyboardControlEnabled(!keyboardControlEnabled);
  };

  const toggleControls = () => {
    setShowControls(!showControls);
  };

  const toggleMouseControl = () => {
    setMouseControlEnabled(!mouseControlEnabled);
  };

  const displayOptions = displays.map((displayId) => (
    <option key={displayId} value={displayId}>
      Display {displayId}
    </option>
  ));

  return (
    <div className="relative w-screen h-screen bg-black flex flex-col items-center p-0 m-0">
      <div className="fixed top-2 left-2 z-10 bg-gray-900 bg-opacity-70 text-white p-2 rounded-full transition-opacity flex flex-col gap-2">
        <button
          className={`text-white p-2 rounded-full opacity-70 hover:opacity-100 transition-opacity ${
            !showControls ? "bg-gray-600" : "bg-green-600"
          }`}
          onClick={toggleControls}
          title={showControls ? "Hide Controls" : "Show Controls"}
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            className="h-5 w-5"
            viewBox="0 0 20 20"
            fill="currentColor"
          >
            <path
              fillRule="evenodd"
              d="M11.49 3.17c-.38-1.56-2.6-1.56-2.98 0a1.532 1.532 0 01-2.286.948c-1.372-.836-2.942.734-2.106 2.106.54.886.061 2.042-.947 2.287-1.561.379-1.561 2.6 0 2.978a1.532 1.532 0 01.947 2.287c-.836 1.372.734 2.942 2.106 2.106a1.532 1.532 0 012.287.947c.379 1.561 2.6 1.561 2.978 0a1.533 1.533 0 012.287-.947c1.372.836 2.942-.734 2.106-2.106a1.533 1.533 0 01.947-2.287c1.561-.379 1.561-2.6 0-2.978a1.532 1.532 0 01-.947-2.287c.836-1.372-.734-2.942-2.106-2.106a1.532 1.532 0 01-2.287-.947zM10 13a3 3 0 100-6 3 3 0 000 6z"
              clipRule="evenodd"
            />
          </svg>
        </button>

        <button
          className={`p-2 rounded-full opacity-70 hover:opacity-100 transition-opacity ${
            mouseControlEnabled ? "bg-green-600" : "bg-gray-600"
          }`}
          onClick={toggleMouseControl}
          title={
            mouseControlEnabled
              ? "Disable Mouse Control"
              : "Enable Mouse Control"
          }
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            className="h-5 w-5"
            viewBox="0 0 20 20"
            fill="currentColor"
          >
            <path
              fillRule="evenodd"
              d="M6.672 1.911a1 1 0 10-1.932.518l.259.966a1 1 0 001.932-.518l-.26-.966zM2.429 4.74a1 1 0 10-.517 1.932l.966.259a1 1 0 00.517-1.932l-.966-.26zm8.814-.569a1 1 0 00-1.415-1.414l-.707.707a1 1 0 101.415 1.415l.707-.708zm-7.071 7.072l.707-.707A1 1 0 003.465 9.12l-.708.707a1 1 0 001.415 1.415zm3.2-5.171a1 1 0 00-1.3 1.3l4 10a1 1 0 001.823.075l1.38-2.759 3.018 3.02a1 1 0 001.414-1.415l-3.019-3.02 2.76-1.379a1 1 0 00-.076-1.822l-10-4z"
              clipRule="evenodd"
            />
          </svg>
        </button>

        <button
          className={`p-2 rounded-full opacity-70 hover:opacity-100 transition-opacity ${
            keyboardControlEnabled ? "bg-green-600" : "bg-gray-600"
          }`}
          onClick={toggleKeyboardControl}
          title={
            keyboardControlEnabled
              ? "Disable Keyboard Control"
              : "Enable Keyboard Control"
          }
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            className="h-5 w-5"
            viewBox="0 0 20 20"
            fill="currentColor"
          >
            <path
              fillRule="evenodd"
              d="M3 5a2 2 0 012-2h10a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2V5zm11 0H6v8h8V5zm-9 6h1v1H5v-1zm3 0h1v1H8v-1zm3 0h1v1h-1v-1z"
              clipRule="evenodd"
            />
          </svg>
        </button>
      </div>

      {showControls && (
        <div className="absolute top-2 z-10 bg-gray-800 bg-opacity-80 p-3 rounded-lg flex flex-col items-center gap-3 text-white min-w-[300px]">
          <div className="flex w-full justify-between items-center">
            <h3 className="text-sm font-medium">Remote Desktop Control</h3>
            <span className="text-xs px-2 py-1 rounded bg-gray-700">
              {connectionStatus}
            </span>
          </div>

          <div className="grid grid-cols-2 gap-3 w-full">
            <div>
              <label className="block text-xs font-medium mb-1">Display</label>
              <select
                className="bg-gray-700 text-white text-sm rounded px-2 py-1 w-full"
                value={selectedDisplay}
                onChange={(e) => switchDisplay(Number(e.target.value))}
                disabled={streaming}
              >
                {displayOptions}
              </select>
            </div>

            <div>
              <label className="block text-xs font-medium mb-1">Quality</label>
              <input
                type="number"
                className="bg-gray-700 text-white text-sm rounded px-2 py-1 w-full"
                min="1"
                max="100"
                value={quality}
                onChange={(e) => setQuality(Number(e.target.value))}
                disabled={streaming}
              />
            </div>

            <div>
              <label className="block text-xs font-medium mb-1">FPS</label>
              <input
                type="number"
                className="bg-gray-700 text-white text-sm rounded px-2 py-1 w-full"
                min="1"
                max="30"
                value={fps}
                onChange={(e) => setFps(Number(e.target.value))}
                disabled={streaming}
              />
            </div>

            <div className="flex items-end">
              {!streaming ? (
                <button
                  className="bg-green-600 hover:bg-green-700 text-white px-3 py-1 text-sm rounded w-full whitespace-nowrap"
                  onClick={handleStartStreaming}
                >
                  Start Streaming
                </button>
              ) : (
                <button
                  className="bg-red-600 hover:bg-red-700 text-white px-3 py-1 text-sm rounded w-full whitespace-nowrap"
                  onClick={handleStopStreaming}
                >
                  Stop Streaming
                </button>
              )}
            </div>
          </div>
        </div>
      )}

      <div className="flex-1 flex items-center justify-center w-full h-full">
        <canvas
          ref={canvasRef}
          className="max-h-full max-w-full"
          onMouseDown={(event) => handleCanvasClick(event)}
          onContextMenu={(event) => event.preventDefault()}
        />
      </div>
    </div>
  );
};
