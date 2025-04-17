import { Routes, Route } from "react-router-dom";
import { Layout } from "./Layout";

import { RATProvider } from "./rat/RATProvider";

import { Logs } from "./pages/Logs";
import { Clients } from "./pages/Clients";
import { Settings } from "./pages/Settings";
import { RemoteDesktop } from "./pages/RemoteDesktop";
export const App: React.FC = () => {
  return (
    <RATProvider>
      <Routes>
        <Route path="/remote-desktop/:addr" element={<RemoteDesktop />} />
        <Route path="/" element={<Layout />}>
          <Route path="/" element={<Clients />} />
          <Route path="/logs" element={<Logs />} />
          <Route path="/settings" element={<Settings />} />
        </Route>
      </Routes>
    </RATProvider>
  );
};
