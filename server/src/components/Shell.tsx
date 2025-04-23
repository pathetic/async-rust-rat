import React, { useState, useRef, useEffect } from "react";
import { keybindings } from "../utils/keybindings";
import { ShellCommandType, CommandProps } from "../../types";

export const Ascii: React.FC = () => {
  return (
    <div>
      <pre className="text-blue-400 text-left font-bold mb-2 drop-shadow-xl selection:bg-yellow-900 selection:text-white">
        {`
    __________                       __                _________.__           .__  .__   
    \\______   \\ ____   _____   _____/  |_  ____       /   _____/|  |__   ____ |  | |  |  
     |       _// __ \\ /     \\ /  _ \\   __\\/ __ \\      \\_____  \\ |  |  \\_/ __ \\|  | |  |  
     |    |   \\  ___/|  Y Y  (  <_> )  | \\  ___/      /        \\|   Y  \\  ___/|  |_|  |__
     |____|_  /\\___  >__|_|  /\\____/|__|  \\___  >    /_______  /|___|  /\\___  >____/____/
            \\/     \\/      \\/                 \\/             \\/      \\/     \\/              
              `}
      </pre>
    </div>
  );
};

export const Header: React.FC = () => {
  return (
    <div className="font-mono text-left ml-10 mb-5  selection:bg-yellow-900">
      Welcome to Remote Shell!
      <br />
      Type <span className="text-blue-400">!help</span> to get a list of
      available client-side commands. <br />
    </div>
  );
};

export const Command: React.FC<CommandProps> = ({ addr }) => {
  const [command, setCommand] = useState<ShellCommandType[]>([]);
  const [currentCommand, setCurrentCommand] = useState<string>("");
  const [upArrowKeyPressed, setUpArrowKeyPressed] = useState<number>(0);
  const inputRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    if (inputRef.current) {
      inputRef.current.scrollIntoView({
        behavior: "smooth",
        block: "nearest",
        inline: "start",
      });
      inputRef.current.focus();
    }
  }, [command]);

  useEffect(() => {
    const handleClick = (event: MouseEvent) => {
      // Skip focusing if text is being selected
      const selectedText = window.getSelection()?.toString();
      if (selectedText && selectedText.length > 0) {
        return;
      }

      if (
        inputRef.current &&
        !inputRef.current.contains(event.target as Node)
      ) {
        inputRef.current.focus();
      }
    };

    document.addEventListener("click", handleClick);

    return () => {
      document.removeEventListener("click", handleClick);
    };
  }, []);

  console.log(command);

  return (
    <div className="reverse-shell">
      {command.map((item, index) => (
        <div key={index}>
          <div className="flex flex-row mb-1">
            <div className="text-green-400 font-bold ml-6 selection:bg-yellow-900">
              remote@shell~$
            </div>
            <div className="ml-2 font-mono selection:bg-yellow-900">
              {item.command}
            </div>
          </div>
          <div
            className="font-mono text-left ml-12 mb-1 selection:bg-yellow-900 mr-10"
            id="output-content"
          >
            {item.output && typeof item.output === "string"
              ? item.output.substring(item.output.indexOf("\n") + 1)
              : item.output}
          </div>
        </div>
      ))}
      <div className="flex flex-row mb-4">
        <div className="text-green-400 font-bold ml-6 selection:bg-yellow-900">
          remote@shell~$
        </div>
        <input
          className="bg-transparent outline-none border-none font-mono ml-2  w-2/3"
          type="text"
          ref={inputRef}
          value={currentCommand}
          autoFocus={true}
          onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
            setCurrentCommand(e.target.value)
          }
          onKeyDown={(e: React.KeyboardEvent<HTMLInputElement>) => {
            keybindings(
              e,
              setCommand,
              setUpArrowKeyPressed,
              setCurrentCommand,
              currentCommand,
              command,
              addr,
              upArrowKeyPressed
            );
          }}
        />
      </div>
    </div>
  );
};
