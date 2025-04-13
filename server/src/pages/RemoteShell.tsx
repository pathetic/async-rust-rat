import { useParams } from "react-router-dom";
import { useEffect, useState } from "react";
import { Ascii, Command, Header } from "../components/Shell";
import { manageShellCmd } from "../rat/RATCommands";

export const RemoteShell: React.FC = () => {
  const { id } = useParams();
  const [shellStatus, setShellStatus] = useState<string>("false");

  async function manage_shell(id: string, run: string) {
    const ok: string = await manageShellCmd(id, run);
    setShellStatus(ok);
  }

  useEffect(() => {
    manage_shell(String(id), "status");
  }, []);

  return (
    <div className="p-8 flex flex-1 flex-col overflow-auto w-full">
      <div className="flex flex-row gap-6 pb-4">
        <a
          onClick={() => manage_shell(String(id), "start")}
          className={`btn btn-success ${
            shellStatus == "true" ? "btn-disabled" : ""
          }`}
        >
          Start Shell
        </a>
        <a
          onClick={() => manage_shell(String(id), "stop")}
          className={`btn btn-error ${
            shellStatus == "true" ? "" : "btn-disabled"
          }`}
        >
          Stop Shell
        </a>
      </div>
      <div className="bg-neutral-800 text-slate-300 w-full h-screen overflow-x-hidden">
        <Ascii />
        <Header />
        <Command id={String(id)} shellStatus={shellStatus} />
      </div>
    </div>
  );
};
