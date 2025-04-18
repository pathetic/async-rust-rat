import { useParams } from "react-router-dom";
import { useEffect, useState } from "react";
import { Ascii, Command, Header } from "../components/Shell";
import { manageShellCmd } from "../rat/RATCommands";
import { getCurrent } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";

export const ReverseShell = () => {
  const { addr } = useParams();

  async function manage_shell(run: string) {
    await manageShellCmd(addr, run);
  }

  useEffect(() => {
    manage_shell("start");
  }, []);

  useEffect(() => {
    let cleanupFn: (() => void) | undefined;

    let window = getCurrent();

    listen("close_window", () => {
      window.close();
    }).then((unlisten) => {
      cleanupFn = unlisten;
    });

    return () => {
      if (cleanupFn) cleanupFn();
    };
  }, []);

  useEffect(() => {
    const handleBeforeUnload = () => {
      console.log("beforeunload");
      manage_shell("stop");
    };

    window.addEventListener("beforeunload", handleBeforeUnload);

    return () => {
      window.removeEventListener("beforeunload", handleBeforeUnload);
    };
  }, []);

  useEffect(() => {
    const cleanup = async () => {
      try {
        const window = getCurrent();

        await window.listen("tauri://close-requested", async () => {
          manage_shell("stop");
          window.close();
        });
      } catch (error) {
        console.error("Error setting up window close handler:", error);
      }
    };

    cleanup();

    return () => {
      manage_shell("stop");
    };
  }, []);

  return (
    <div className="reverse-shell bg-primarybg text-slate-300 w-full h-screen overflow-x-hidden">
      <Ascii />
      <Header />
      <Command addr={String(addr)} />
    </div>
  );
};
