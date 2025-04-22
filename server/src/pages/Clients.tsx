import React, { useEffect, useState, useContext, useMemo } from "react";
import { RATContext } from "../rat/RATContext";

import { ContextMenu } from "../components/ContextMenu";

import { RATClient, FilterCategories } from "../../types";

import { ContextMenuType } from "../../types";

import { ClientInfo } from "../components/home/ClientInfo";
import { ClientsTable } from "../components/home/ClientsTable";
import { VisitWebsiteModal } from "../components/home/modals/VisitWebsiteModal";
import { MessageBoxModal } from "../components/home/modals/MessageBoxModal";
import { TableFilter } from "../components/TableFilter";

export const Clients = () => {
  const [selectedClient, setSelectedClient] = useState<string>("");
  const [selectedClientDetails, setSelectedClientDetails] =
    useState<RATClient | null>(null);
  const { clientList } = useContext(RATContext)!;
  const [contextMenu, setContextMenu] = useState<ContextMenuType | null>(null);

  // Search and filter functionality
  const [searchTerm, setSearchTerm] = useState("");

  const [filters, setFilters] = useState<FilterCategories>({
    group: {},
    os: {},
    cpu: {},
    gpus: {},
  });
  const [activeFilterTab, setActiveFilterTab] =
    useState<keyof FilterCategories>("group");

  // Initialize filters based on client properties
  useEffect(() => {
    if (clientList && clientList.length > 0) {
      const groupValues = new Set<string>();
      const osValues = new Set<string>();
      const cpuValues = new Set<string>();
      const gpuValues = new Set<string>();

      clientList.forEach((client: RATClient) => {
        // Add values to respective sets
        groupValues.add(client.group);
        osValues.add(client.os);
        cpuValues.add(client.cpu);

        // Add each GPU to the set
        client.gpus.forEach((gpu) => {
          gpuValues.add(gpu);
        });
      });

      // Initialize filters with all options checked
      const initialFilters: FilterCategories = {
        group: {},
        os: {},
        cpu: {},
        gpus: {},
      };

      // Set initial values for each filter category
      groupValues.forEach((value) => {
        initialFilters.group[value] = true;
      });

      osValues.forEach((value) => {
        initialFilters.os[value] = true;
      });

      cpuValues.forEach((value) => {
        initialFilters.cpu[value] = true;
      });

      gpuValues.forEach((value) => {
        initialFilters.gpus[value] = true;
      });

      setFilters(initialFilters);
    }
  }, [clientList]);

  // Filter clients based on search term and selected filters
  const filteredClients = useMemo(() => {
    return clientList.filter((client: RATClient) => {
      // Check if client matches search term
      const matchesSearch =
        searchTerm === "" ||
        client.username.toLowerCase().includes(searchTerm.toLowerCase()) ||
        client.hostname.toLowerCase().includes(searchTerm.toLowerCase()) ||
        client.addr.toLowerCase().includes(searchTerm.toLowerCase()) ||
        client.os.toLowerCase().includes(searchTerm.toLowerCase()) ||
        client.cpu.toLowerCase().includes(searchTerm.toLowerCase()) ||
        client.ram.toLowerCase().includes(searchTerm.toLowerCase()) ||
        client.gpus.some((gpu) =>
          gpu.toLowerCase().includes(searchTerm.toLowerCase())
        );

      // Check if client passes all filters
      const passesGroupFilter = filters.group[client.group] !== false;
      const passesOsFilter = filters.os[client.os] !== false;
      const passesCpuFilter = filters.cpu[client.cpu] !== false;

      // Check if at least one GPU passes the filter
      const passesGpuFilter = client.gpus.some(
        (gpu) => filters.gpus[gpu] !== false
      );

      return (
        matchesSearch &&
        passesGroupFilter &&
        passesOsFilter &&
        passesCpuFilter &&
        passesGpuFilter
      );
    });
  }, [clientList, searchTerm, filters]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setContextMenu(null);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [contextMenu]);

  const handleContextMenu = (
    event: React.MouseEvent<HTMLDivElement, MouseEvent>,
    addr: string,
    clientFullName: string
  ) => {
    event.preventDefault();
    setSelectedClient(addr);
    setContextMenu({
      x: event.pageX,
      y: event.pageY,
      addr: addr,
      clientFullName,
    });
  };

  const handleClose = () => {
    setContextMenu(null);
  };

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      const target = event.target as HTMLElement;
      if (contextMenu && !target.closest(".context-menu")) {
        setContextMenu(null);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [contextMenu]);

  useEffect(() => {
    if (!clientList.some((client) => client.addr === selectedClient)) {
      setSelectedClient("");
      setSelectedClientDetails(null);
    }
  }, [clientList, selectedClient]);

  return (
    <>
      <MessageBoxModal selectedClient={selectedClient} />

      <VisitWebsiteModal selectedClient={selectedClient} />

      <div className="flex flex-col h-full">
        {/* Search and filter bar */}
        <TableFilter
          searchTerm={searchTerm}
          setSearchTerm={setSearchTerm}
          searchPlaceholder="Search clients..."
          filters={filters}
          setFilters={setFilters}
          filterCategories={["group", "os", "cpu", "gpus"]}
          activeFilterCategory={activeFilterTab}
          setActiveFilterCategory={(category) =>
            setActiveFilterTab(category as keyof FilterCategories)
          }
        />

        {/* Main content */}
        <div className="flex flex-row flex-1">
          <div className="flex-1 overflow-auto bg-secondarybg text-black rounded-2xl px-4 sm:px-6 lg:px-8">
            <div className="flow-root">
              <div className="-mx-4 -my-2 sm:-mx-6 lg:-mx-8">
                <div className="inline-block min-w-full py-2 align-middle">
                  <ClientsTable
                    filteredClients={filteredClients}
                    handleContextMenu={handleContextMenu}
                    setSelectedClientDetails={setSelectedClientDetails}
                    selectedClientDetails={selectedClientDetails}
                    searchTerm={searchTerm}
                    filters={filters}
                  />
                  {contextMenu && (
                    <ContextMenu
                      x={contextMenu.x}
                      y={contextMenu.y}
                      addr={contextMenu.addr}
                      onClose={handleClose}
                      clientFullName={contextMenu.clientFullName}
                    />
                  )}
                </div>
              </div>
            </div>
          </div>

          {/* Details panel */}
          <div
            className={`w-100 px-2 h-full ${
              selectedClientDetails ? "block" : "hidden"
            }`}
          >
            {selectedClientDetails && (
              <ClientInfo
                client={selectedClientDetails}
                onClose={() => setSelectedClientDetails(null)}
              />
            )}
          </div>
        </div>
      </div>
    </>
  );
};
