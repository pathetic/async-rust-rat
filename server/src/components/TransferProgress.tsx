import React, { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { save } from "@tauri-apps/plugin-dialog";
import { writeFile } from "@tauri-apps/plugin-fs";

interface TransferItem {
  id: string;
  filename: string;
  addr: string;
  status: string;
  loaded: number;
  total: number;
  speed: string;
  path?: string;
}

const base64ToUint8Array = (base64: string) => {
  const binary = window.atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
};

const formatBytes = (bytes: number) => {
  if (bytes >= 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  }
  if (bytes >= 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${bytes} B`;
};

export const TransferProgress: React.FC = () => {
  const [transfers, setTransfers] = useState<TransferItem[]>([]);

  useEffect(() => {
    let unlistenStart: (() => void) | null = null;
    let unlistenComplete: (() => void) | null = null;

    const setupListeners = async () => {
      const currentWindow = await getCurrentWindow();
      if (currentWindow.label !== "main") {
        return;
      }

      unlistenStart = await listen("file_transfer_start", (event: any) => {
        const payload = event.payload as any;
        setTransfers((prev) => {
          const existing = prev.find((item) => item.id === payload.id);
          if (existing) {
            return prev.map((item) =>
              item.id === payload.id
                ? {
                    ...item,
                    status: payload.status || item.status,
                    total: payload.total || item.total,
                  }
                : item
            );
          }

          return [
            {
              id: payload.id,
              filename: payload.filename,
              addr: payload.addr,
              status: "Transferring",
              loaded: payload.loaded || 0,
              total: payload.total || 0,
              speed: "…",
            },
            ...prev,
          ];
        });
      });

      unlistenComplete = await listen("file_transfer_complete", async (event: any) => {
        const payload = event.payload as any;
        const bytes = base64ToUint8Array(payload.data);
        setTransfers((prev) => {
          const existing = prev.find((item) => item.id === payload.id);
          const next = prev.map((item) =>
            item.id === payload.id
              ? {
                  ...item,
                  status: "Completed",
                  loaded: payload.total || bytes.length,
                  total: payload.total || bytes.length,
                  speed: `${payload.speed ? payload.speed : 0} B/s`,
                }
              : item
          );
          if (!existing) {
            next.unshift({
              id: payload.id,
              filename: payload.filename,
              addr: payload.addr,
              status: "Completed",
              loaded: payload.total || bytes.length,
              total: payload.total || bytes.length,
              speed: `${payload.speed ? payload.speed : 0} B/s`,
            });
          }
          return next;
        });

        if (payload.data && payload.filename) {
          const selected = await save({ defaultPath: payload.filename });
          if (selected) {
            try {
              await writeFile(selected, bytes);
              setTransfers((prev) =>
                prev.map((item) =>
                  item.id === payload.id ? { ...item, path: selected } : item
                )
              );
            } catch (error) {
              console.error("Failed to save transferred file:", error);
            }
          }
        }

        setTimeout(() => {
          setTransfers((prev) => prev.filter((item) => item.id !== payload.id));
        }, 10000);
      });
    };

    setupListeners();

    return () => {
      unlistenStart?.();
      unlistenComplete?.();
    };
  }, []);

  if (transfers.length === 0) {
    return null;
  }

  return (
    <div className="fixed bottom-6 right-6 z-50 space-y-3 w-[320px]">
      {transfers.map((transfer) => {
        const progress = transfer.total ? Math.min((transfer.loaded / transfer.total) * 100, 100) : 0;
        return (
          <div
            key={transfer.id}
            className="rounded-2xl border border-accentx bg-secondarybg p-4 text-white shadow-xl"
          >
            <div className="flex items-center justify-between gap-2">
              <div>
                <p className="font-semibold">{transfer.filename}</p>
                <p className="text-xs text-gray-400">{transfer.addr}</p>
              </div>
              <span className="text-xs text-accentx">{transfer.status}</span>
            </div>
            <div className="mt-3 bg-[#15202b] h-2 rounded-full overflow-hidden">
              <div
                className="h-full bg-green-500"
                style={{ width: `${progress}%` }}
              />
            </div>
            <div className="mt-2 flex items-center justify-between text-xs text-gray-300">
              <span>{formatBytes(transfer.loaded)} / {formatBytes(transfer.total)}</span>
              <span>{transfer.speed}</span>
            </div>
            {transfer.path && (
              <div className="mt-2 text-[11px] text-gray-400 break-all">
                saved to {transfer.path}
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
};
