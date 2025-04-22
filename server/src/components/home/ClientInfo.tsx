import { RATClient } from "../../../types";
import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { takeScreenshotCmd } from "../../rat/RATCommands";
import {
  IconSquareRoundedX,
  IconCamera,
  IconPhotoScan,
  IconDeviceLaptop,
  IconDeviceDesktop,
  IconUser,
  IconServer,
  IconCpu,
  IconDeviceDesktopAnalytics,
  IconShieldCheck,
  IconDeviceSdCard,
  IconBrandWindows,
  IconBrandUbuntu,
} from "@tabler/icons-react";

export const ClientInfo = ({
  client,
  onClose,
}: {
  client: RATClient;
  onClose: () => void;
}) => {
  if (!client) return null;

  const [screenshot, setScreenshot] = useState<string | null>(null);
  const [isScreenshotLoading, setIsScreenshotLoading] = useState(false);
  const [activeSection, setActiveSection] = useState<string>("system");

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
      setIsScreenshotLoading(false);
    });
  }

  useEffect(() => {
    waitScreenshot();
  }, []);

  async function takeScreenshot(client: RATClient, display: number) {
    setIsScreenshotLoading(true);
    await takeScreenshotCmd(client.addr, display);
  }

  // Helper to determine OS icon
  const getOsIcon = () => {
    if (client.os.toLowerCase().includes("windows")) {
      return <IconBrandWindows size={20} className="text-blue-400" />;
    } else if (
      client.os.toLowerCase().includes("linux") ||
      client.os.toLowerCase().includes("ubuntu")
    ) {
      return <IconBrandUbuntu size={20} className="text-orange-400" />;
    } else {
      return <IconDeviceDesktop size={20} className="text-gray-400" />;
    }
  };

  return (
    <div className="relative bg-secondarybg p-4 rounded-xl text-white h-full flex flex-col">
      {/* Header */}
      <div className="flex justify-between items-center mb-4 border-b border-gray-700 pb-3">
        <h2 className="text-xl font-bold flex items-center gap-2">
          {getOsIcon()}
          <span className="truncate">{client.hostname}</span>
        </h2>
        <button
          onClick={onClose}
          className="text-accentx hover:text-white transition cursor-pointer"
          aria-label="Close details"
        >
          <IconSquareRoundedX size={28} />
        </button>
      </div>

      {/* Tabs */}
      <div className="flex mb-4 border-b border-gray-700">
        <button
          className={`py-2 px-4 flex items-center gap-1 ${
            activeSection === "system"
              ? "text-white border-b-2 border-accentx"
              : "text-gray-400 hover:text-white"
          }`}
          onClick={() => setActiveSection("system")}
        >
          <IconServer size={16} />
          <span>System</span>
        </button>
        <button
          className={`py-2 px-4 flex items-center gap-1 ${
            activeSection === "screenshot"
              ? "text-white border-b-2 border-accentx"
              : "text-gray-400 hover:text-white"
          }`}
          onClick={() => setActiveSection("screenshot")}
        >
          <IconCamera size={16} />
          <span>Screenshot</span>
        </button>
      </div>

      {/* Main content */}
      <div className="flex-1 overflow-y-auto">
        {activeSection === "system" && (
          <div className="space-y-4">
            {/* Connection info */}
            <div className="bg-primarybg rounded-lg p-3">
              <h3 className="text-accentx font-semibold mb-2 text-sm">
                CONNECTION
              </h3>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <div className="text-xs text-gray-400">Address</div>
                  <div className="font-mono text-sm">{client.addr}</div>
                </div>
                <div>
                  <div className="text-xs text-gray-400">Group</div>
                  <div className="text-sm">{client.group}</div>
                </div>
              </div>
            </div>

            {/* User info */}
            <div className="bg-primarybg rounded-lg p-3">
              <h3 className="text-accentx font-semibold mb-2 text-sm flex items-center gap-1">
                <IconUser size={16} />
                <span>USER</span>
              </h3>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <div className="text-xs text-gray-400">Username</div>
                  <div className="text-sm">{client.username}</div>
                </div>
                <div>
                  <div className="text-xs text-gray-400">Privileges</div>
                  <div className="text-sm flex items-center gap-1">
                    {client.is_elevated ? (
                      <>
                        <span className="text-green-400">Administrator</span>
                        <IconShieldCheck size={16} className="text-green-400" />
                      </>
                    ) : (
                      <span className="text-gray-300">Standard User</span>
                    )}
                  </div>
                </div>
              </div>
            </div>

            {/* System info */}
            <div className="bg-primarybg rounded-lg p-3">
              <h3 className="text-accentx font-semibold mb-2 text-sm flex items-center gap-1">
                <IconDeviceDesktopAnalytics size={16} />
                <span>SYSTEM</span>
              </h3>
              <div className="space-y-3">
                <div>
                  <div className="text-xs text-gray-400">Operating System</div>
                  <div className="text-sm flex items-center gap-1">
                    {getOsIcon()}
                    <span>{client.os}</span>
                  </div>
                </div>

                <div>
                  <div className="text-xs text-gray-400">CPU</div>
                  <div className="text-sm flex items-center gap-1">
                    <IconCpu size={16} className="text-blue-300" />
                    <span>{client.cpu}</span>
                  </div>
                </div>

                <div>
                  <div className="text-xs text-gray-400">RAM</div>
                  <div className="text-sm">{client.ram}</div>
                </div>

                <div>
                  <div className="text-xs text-gray-400">GPU</div>
                  <div className="text-sm">
                    {client.gpus.map((gpu, index) => (
                      <div key={index}>{gpu}</div>
                    ))}
                  </div>
                </div>

                <div>
                  <div className="text-xs text-gray-400">Storage</div>
                  <div className="text-sm flex items-center flex-wrap gap-2">
                    {client.storage.map((drive, index) => (
                      <div key={index} className="flex items-center gap-1">
                        <IconDeviceSdCard
                          size={16}
                          className="text-amber-300"
                        />
                        <span>{drive}</span>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </div>

            {/* Security info */}
            <div className="bg-primarybg rounded-lg p-3">
              <h3 className="text-accentx font-semibold mb-2 text-sm flex items-center gap-1">
                <IconShieldCheck size={16} />
                <span>SECURITY</span>
              </h3>
              <div>
                <div className="text-xs text-gray-400">Installed Antivirus</div>
                {client.installed_avs.length > 0 ? (
                  <div className="text-sm">
                    {client.installed_avs.map((av, index) => (
                      <div key={index} className="text-amber-300">
                        {av}
                      </div>
                    ))}
                  </div>
                ) : (
                  <div className="text-sm text-green-400">
                    No antivirus detected
                  </div>
                )}
              </div>
            </div>
          </div>
        )}

        {activeSection === "screenshot" && (
          <div className="space-y-4">
            {/* Screenshot display */}
            <div
              className={`w-full aspect-[16/9] border border-accentx rounded-xl flex items-center justify-center bg-primarybg overflow-hidden ${
                isScreenshotLoading ? "animate-pulse" : ""
              }`}
            >
              {screenshot ? (
                <img
                  src={`data:image/png;base64,${screenshot}`}
                  alt="Screenshot"
                  className="max-w-full max-h-full rounded-lg"
                />
              ) : (
                <div className="flex flex-col items-center justify-center text-gray-400">
                  <IconPhotoScan size={48} />
                  <p className="text-sm mt-2">No screenshot available</p>
                  {isScreenshotLoading && (
                    <p className="text-xs mt-1 text-accentx">Loading...</p>
                  )}
                </div>
              )}
            </div>

            {/* Display selection */}
            <div className="bg-primarybg rounded-lg p-3">
              <h3 className="text-accentx font-semibold mb-2 text-sm">
                DISPLAYS ({client.displays})
              </h3>
              <div className="flex flex-wrap gap-2">
                {client &&
                  [...Array(client.displays).keys()].map((index) => (
                    <button
                      key={index}
                      onClick={() => takeScreenshot(client, index)}
                      disabled={isScreenshotLoading}
                      className={`flex items-center gap-1 rounded-full px-3 py-1.5 border 
                        ${
                          isScreenshotLoading
                            ? "border-gray-600 text-gray-400 cursor-not-allowed"
                            : "border-accentx text-white hover:bg-accentx/30 transition"
                        }`}
                    >
                      <IconDeviceLaptop size={16} />
                      <span>Display {index}</span>
                    </button>
                  ))}
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
