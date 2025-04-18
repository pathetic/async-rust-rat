import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { ShellCommandType } from "../../types";
import { executeShellCommandCmd } from "../rat/RATCommands";
import { JSX } from "react";

export const getOutput = async (
  command: string,
  setCommand: React.Dispatch<React.SetStateAction<ShellCommandType[]>>,
  addr: string
): Promise<JSX.Element | string> => {
  return new Promise((resolve) => {
    switch (command.toLowerCase()) {
      case "!help":
        resolve(
          <div>
            Available commands: <br />
            <span className="text-primary ml-3">!clear</span> - Clear the
            terminal
          </div>
        );
        break;
      case "!clear":
        setCommand([]);
        resolve("");
        break;
      default:
        let output: string = "";
        let timer: number | null = null;
        let unlisten: UnlistenFn | undefined;

        listen("client_shellout", (event: any) => {
          if (event.payload.addr !== addr) {
            return;
          }
          output += event.payload.shell_output + "\n";
          if (timer !== undefined && timer) clearTimeout(timer);
          timer = setTimeout(() => {
            resolve(<div style={{ whiteSpace: "pre-wrap" }}>{output}</div>);
            if (unlisten) unlisten();
          }, 250);
        }).then((unlistenFn) => {
          unlisten = unlistenFn;
        });

        executeShellCommandCmd(addr, command).then(() => {
          timer = setTimeout(() => {
            resolve(<div style={{ whiteSpace: "pre-wrap" }}>{output}</div>);
            if (unlisten) unlisten();
          }, 250);
        });
    }
  });
};
