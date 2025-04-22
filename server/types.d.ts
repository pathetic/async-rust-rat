import React from "react";
import { WebviewWindow } from "@tauri-apps/api/window";

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
  ) => Promise<WebviewWindow | undefined>;
  serverLogs: Array<Log>;
};

export interface RATProviderProps {
  children: React.ReactNode;
}

export type RATClient = {
  uuidv4: string;
  addr: string;
  group: string;
  addr: string;
  username: string;
  hostname: string;
  os: string;
  cpu: string;
  ram: string;
  gpus: string[];
  storage: string[];
  displays: number;
  is_elevated: boolean;
  disconnected: boolean;
  installed_avs: string[];
};

export type ClientWindowType = {
  id: string;
  addr: string;
  window: WebviewWindow;
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
