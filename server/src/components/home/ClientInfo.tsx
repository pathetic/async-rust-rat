import { RATClient } from "../../../types";
import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { takeScreenshotCmd } from "../../rat/RATCommands";
import { IconSquareRoundedX } from "@tabler/icons-react";

export const ClientInfo = ({
  client,
  onClose,
}: {
  client: RATClient;
  onClose: () => void;
}) => {
  if (!client) return null;

  const [screenshot, setScreenshot] = useState<string | null>(null);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onClose]);

  async function waitScreenshot() {
    listen("client_screenshot", (event) => {
      setScreenshot(event.payload as string);
    });
  }

  useEffect(() => {
    waitScreenshot();
  }, []);

  async function takeScreenshot(client: RATClient, display: number) {
    await takeScreenshotCmd(client.addr, display);
  }

  return (
    <div className="relative bg-secondarybg p-4 rounded-xl text-white">
      {/* X button */}
      <button
        onClick={onClose}
        className="absolute top-3 right-3 text-accentx hover:text-white transition cursor-pointer"
        aria-label="Close details"
      >
        <IconSquareRoundedX size={32} />
      </button>

      <h2 className="text-lg font-bold mb-2">{client.addr}</h2>

      <div className="mt-4 mb-4">
        <div className="screenshot w-full aspect-[16/9] border border-accentx rounded-xl flex items-center justify-center">
          {screenshot ? (
            <img
              src={`data:image/png;base64,${screenshot}`}
              alt="Screenshot"
              className="h-full rounded-xl"
            />
          ) : (
            <div className="w-full h-full animate-pulse bg-accentx rounded-md" />
          )}
        </div>
      </div>

      <div className="flex flex-row gap-2 mb-2">
        {client &&
          [...Array(client.displays).keys()].map((index) => (
            <a
              key={index}
              onClick={() => takeScreenshot(client, index)}
              className="cursor-pointer rounded-full px-4 py-1.5 border border-accentx bg-secondarybg text-white hover:bg-white hover:text-black transition"
            >
              Capture Display {index}
            </a>
          ))}
      </div>

      <p>
        <span className="text-accentx">Group:</span> {client.group}
      </p>
      <p>
        <span className="text-accentx">OS:</span> {client.os}
      </p>
      <p>
        <span className="text-accentx">Username:</span> {client.username}
      </p>
      <p>
        <span className="text-accentx">Account Type:</span>{" "}
        {client.is_elevated ? "Admin" : "User"}
      </p>
      <p>
        <span className="text-accentx">CPU:</span> {client.cpu}
      </p>
      <p>
        <span className="text-accentx">GPU:</span>
        <br />
        {client.gpus}
      </p>
      <p>
        <span className="text-accentx">RAM:</span> {client.ram}
      </p>
      <p>
        <span className="text-accentx">Drives:</span> {client.storage}
      </p>

      <p>
        <span className="text-accentx">Installed AVs:</span>{" "}
        {client.installed_avs.length > 0
          ? client.installed_avs.join(", ")
          : "None"}
      </p>
    </div>
  );
};
