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

export const App: React.FC = () => {
  return (
    <RATProvider>
      <Routes>
        <Route path="/reverse-proxy/:addr" element={<ReverseProxy />} />
        <Route path="/remote-shell/:addr" element={<ReverseShell />} />
        <Route path="/remote-desktop/:addr" element={<RemoteDesktop />} />
        <Route path="/process-viewer/:addr" element={<ProcessViewer />} />
        <Route path="/file-manager/:addr" element={<FileManager />} />
        <Route path="/" element={<Layout />}>
          <Route path="/" element={<Clients />} />
          <Route path="/logs" element={<Logs />} />
          <Route path="/settings" element={<Settings />} />
        </Route>
      </Routes>
    </RATProvider>
  );
};
