import { useEffect, useState } from "react";
import { ProcessType } from "../../types";
import { processListCmd, killProcessCmd } from "../rat/RATCommands";
import { listen } from "@tauri-apps/api/event";
import { useParams } from "react-router-dom";
import { IconRefresh, IconProgressX } from "@tabler/icons-react";

export const ProcessViewer: React.FC = () => {
  const { addr } = useParams();
  const [processes, setProcesses] = useState<ProcessType[] | null>(null);
  const [processFilter, setProcessFilter] = useState("");

  useEffect(() => {
    const unlisten = listen("process_list", (event: any) => {
      console.log(event.payload);
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
      }
    });

    fetchProcessList();

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  async function fetchProcessList() {
    await processListCmd(addr);
  }

  return (
    <div className="p-8 flex flex-1 flex-col overflow-auto w-full bg-primaryx h-screen">
      <div className="flex flex-row gap-4 pb-4 w-[30%]">
        <a
          className="btn btn-active bg-secondarybg text-white no-animation flex items-center gap-2 hover:bg-white hover:text-black rounded-2xl border border-accentx"
          onClick={fetchProcessList}
        >
          <IconRefresh size={20} />
          Refresh
        </a>
        <div>
          <label className="input flex items-center gap-2 w-[100%] bg-secondarybg text-white border border-accentx rounded-2xl">
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
      <div className="overflow-x-auto bg-secondarybg rounded-2xl h-full border-accentx border">
        <table className="processlist table table-zebra ">
          <thead>
            <tr className="text-white">
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
                          className="btn btn-active no-animation bg-inactive rounded-2xl border border-accentx"
                          onClick={() =>
                            killProcessCmd(
                              addr,
                              parseInt(process.pid),
                              process.name
                            )
                          }
                        >
                          <IconProgressX size={20} />
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
