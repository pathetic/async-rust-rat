import React, { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";
import { getCurrent, WebviewWindow } from "@tauri-apps/api/window";
import { useParams } from "react-router-dom";

import {
  IconHandClick,
  IconKeyboard,
  IconAdjustmentsAlt,
} from "@tabler/icons-react";

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
      {displayId}
    </option>
  ));

  return (
    <div className="relative w-screen h-screen bg-black flex flex-col items-center p-0 m-0">
      <div className="fixed top-2 left-2 z-10 bg-primarybg bg-opacity-70 text-white p-2 rounded-full transition-opacity flex flex-col gap-2">
        <button
          className={`text-white p-2 rounded-full opacity-70 hover:opacity-100 transition-opacity cursor-pointer ${
            !showControls ? "bg-secondarybg" : "bg-white"
          }`}
          onClick={toggleControls}
          title={showControls ? "Hide Controls" : "Show Controls"}
        >
          <IconAdjustmentsAlt
            size={22}
            color={!showControls ? "white" : "black"}
          />
        </button>

        <button
          className={`p-2 rounded-full opacity-70 hover:opacity-100 transition-opacity cursor-pointer ${
            mouseControlEnabled ? "bg-white" : "bg-secondarybg"
          }`}
          onClick={toggleMouseControl}
          title={
            mouseControlEnabled
              ? "Disable Mouse Control"
              : "Enable Mouse Control"
          }
        >
          <IconHandClick
            size={22}
            color={mouseControlEnabled ? "black" : "white"}
          />
        </button>

        <button
          className={`p-2 rounded-full opacity-70 hover:opacity-100 transition-opacity cursor-pointer ${
            keyboardControlEnabled ? "bg-white" : "bg-secondarybg"
          }`}
          onClick={toggleKeyboardControl}
          title={
            keyboardControlEnabled
              ? "Disable Keyboard Control"
              : "Enable Keyboard Control"
          }
        >
          <IconKeyboard
            size={22}
            color={keyboardControlEnabled ? "black" : "white"}
          />
        </button>
      </div>

      {showControls && (
        <div className="absolute top-2 z-10 bg-primarybg bg-opacity-80 p-3 rounded-2xl flex flex-col items-center gap-3 text-white min-w-[300px]">
          <div className="flex w-full justify-between items-center">
            <h3 className="text-sm font-medium">Remote Desktop</h3>
            <span className="text-xs px-2 py-1 bg-white text-black rounded-2xl border border-accentx">
              {connectionStatus}
            </span>
          </div>

          <div className="grid grid-cols-2 gap-3 w-full">
            <div className="flex items-center rounded-full bg-secondarybg pl-3 border border-accentx h-9">
              <div className="shrink-0 text-base text-accentx select-none sm:text-sm/6">
                Display
              </div>
              <select
                className={`block w-16 py-0 pl-2 text-base placeholder:text-gray-400 bg-transparent focus:outline-none sm:text-sm/6 ${
                  streaming ? "text-accentx" : "text-white"
                }`}
                value={selectedDisplay}
                onChange={(e) => switchDisplay(Number(e.target.value))}
                disabled={streaming}
              >
                {displayOptions}
              </select>
            </div>

            <div className="flex items-center rounded-full bg-secondarybg pl-3 border border-accentx h-9">
              <div className="shrink-0 text-base text-accentx select-none sm:text-sm/6">
                Quality
              </div>
              <input
                type="number"
                className={`block w-16 py-0 pl-2 text-base placeholder:text-gray-400 bg-transparent focus:outline-none sm:text-sm/6 ${
                  streaming ? "text-accentx" : "text-white"
                }`}
                min="1"
                max="100"
                value={quality}
                onChange={(e) => setQuality(Number(e.target.value))}
                disabled={streaming}
              />
            </div>

            <div className="flex items-center rounded-full bg-secondarybg pl-3 border border-accentx h-9">
              <div className="shrink-0 text-base text-accentx select-none sm:text-sm/6">
                FPS:
              </div>
              <input
                type="number"
                className={`block w-16 py-0 pl-2 text-base placeholder:text-gray-400 bg-transparent focus:outline-none sm:text-sm/6 ${
                  streaming ? "text-accentx" : "text-white"
                }`}
                min="1"
                max="30"
                value={fps}
                onChange={(e) => setFps(Number(e.target.value))}
              />
            </div>

            <div className="flex items-end text-sm">
              {!streaming ? (
                <button
                  className="cursor-pointer rounded-full px-4 py-1.5 border border-accentx bg-active text-white hover:bg-white hover:text-black transition"
                  onClick={handleStartStreaming}
                >
                  Start Streaming
                </button>
              ) : (
                <button
                  className="cursor-pointer rounded-full px-4 py-1.5 border border-accentx bg-inactive text-white hover:bg-white hover:text-black transition"
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
