import React from "react";
import { WebviewWindow } from "@tauri-apps/api/window";

export type RATState = {
  running: boolean;
  port: string;
};

// Client window interface for tracking windows associated with clients
export interface ClientWindow {
  windowId: string;
  window: WebviewWindow;
  type: string;
  clientId: string;
}

export type RATContextType = {
  port: string;
  setPort: (port: string) => void;
  fetchClients: () => void;
  setRunning: (running: boolean) => void;
  running: boolean;
  clientList: Array<RATClient>;
  setSelectedClient: (client: string) => void;
  selectedClient: string;
  setNotificationClient: (notificationClient: boolean) => void;
  notificationClient: boolean;
  // Simple window opening function - no tracking
  openClientWindow: (
    clientId: string,
    type: string,
    url: string,
    title?: string
  ) => Promise<WebviewWindow | undefined>;
};

export interface RATProviderProps {
  children: React.ReactNode;
}

export type RATClient = {
  id: string;
  username: string;
  hostname: string;
  ip: string;
  os: string;
  cpu: string;
  ram: string;
  gpus: string[];
  storage: string[];
  displays: number;
  is_elevated: boolean;
  disconnected: boolean;
};

export type ContextMenuType = {
  x: number;
  y: number;
  id: string;
  clientFullName: string;
};

export interface ContextMenuProps {
  x: number;
  y: number;
  id: string;
  onClose: () => void;
  clientFullName: string;
}

export type MenuOptionType = {
  label: string;
  icon: React.ReactNode;
  navigate?: boolean;
  path?: string;
  options?: MenuOptionType[];
  run?: string;
  function?: (string?, string?) => void;
  modal?: boolean;
  modalId?: string;
};

interface SubMenuProps {
  items: MenuOptionType[];
  top: number;
  left: number;
  id: string;
  navigate: (string) => void;
  onClose: () => void;
}

export interface ShellCommandType {
  command: string;
  output: React.JSX.Element | string;
}

export interface CommandProps {
  id: string;
  shellStatus: string;
}

export type ProcessType = {
  pid: string;
  name: string;
};

export type FileType = {
  file_type: string;
  name: string;
};
