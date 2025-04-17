import { Routes, Route } from "react-router-dom";
import { Layout } from "./Layout";
import { Home } from "./pages/Home";
import { Server } from "./Server";
import { Clients } from "./pages/ClientList";
import { ClientView } from "./pages/ClientView";
import { FileManager } from "./pages/FileManager";
import { RemoteShell } from "./pages/RemoteShell";
import { ProcessList } from "./pages/ProcessList";
import { Settings } from "./pages/Settings";
import { RATProvider } from "./rat/RATProvider";
import { Toaster } from "react-hot-toast";
import RemoteDesktopWindow from "./pages/RemoteDesktopWindow";

export const App: React.FC = () => {
  return (
    <RATProvider>
      <Routes>
        <Route path="/" element={<Server />} />
        <Route
          path="/remote-desktop-window"
          element={<RemoteDesktopWindow />}
        />

        <Route path="/" element={<Layout />}>
          <Route path="/home" element={<Home />} />
          <Route path="/clients" element={<Clients />} />
          <Route path="/clients/:id" element={<ClientView />} />
          <Route path="/clients/:id/files" element={<FileManager />} />
          <Route path="/clients/:id/shell" element={<RemoteShell />} />
          <Route path="/clients/:id/process" element={<ProcessList />} />
          <Route path="/settings" element={<Settings />} />
        </Route>
      </Routes>
      <Toaster position="bottom-right" reverseOrder={false} />
    </RATProvider>
  );
};
