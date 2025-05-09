import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { sendInputBoxCmd } from "../rat/RATCommands";
import { listen } from "@tauri-apps/api/event";
import {
  IconSend,
  IconCheck,
  IconX,
  IconMessageDots,
  IconInputSearch,
  IconHistory,
  IconClock,
} from "@tabler/icons-react";

export const InputBox = () => {
  const { addr } = useParams();

  const [title, setTitle] = useState("");
  const [message, setMessage] = useState("");
  const [inputBoxResults, setInputBoxResults] = useState<
    Array<{ text: string; timestamp: string }>
  >([]);
  const [isSending, _setIsSending] = useState(false);

  const handleSendInputBox = async () => {
    if (!title.trim() || !message.trim() || isSending) return;

    try {
      await sendInputBoxCmd(addr, title, message);

      setTitle("");
      setMessage("");
    } catch (error) {
      console.error("Error sending input box:", error);
    }
  };

  useEffect(() => {
    const setupListener = async () => {
      const unlisten = await listen("inputbox_result", (event: any) => {
        if (event.payload.addr === addr) {
          const now = new Date();
          const timestamp = now.toLocaleTimeString();
          setInputBoxResults((prev) => [
            { text: event.payload.result, timestamp },
            ...prev,
          ]);
        }
      });

      return unlisten;
    };

    const unlistenPromise = setupListener();

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [addr]);

  return (
    <div className="bg-secondarybg text-white h-screen flex flex-col">
      <div className="sticky top-0 z-10 bg-secondarybg py-2 px-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <IconInputSearch size={20} className="text-accentx" />
            <h1 className="text-lg font-bold">Input Box</h1>
          </div>
        </div>
      </div>

      <div className="flex-1 p-3 flex flex-col gap-3 overflow-hidden">
        <div className="bg-primarybg rounded-lg p-3">
          <h3 className="text-accenttext font-semibold mb-2 text-xs flex items-center gap-1">
            <IconMessageDots size={14} />
            <span>SEND INPUT BOX</span>
          </h3>

          <div className="space-y-2">
            <div>
              <label className="block text-xs text-accenttext mb-1">
                Title
              </label>
              <input
                type="text"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                placeholder="Enter box title"
                className="w-full bg-secondarybg text-white border border-gray-600 rounded-lg p-1.5 text-sm focus:border-accentx focus:outline-none"
              />
            </div>

            <div>
              <label className="block text-xs text-accenttext mb-1">
                Message
              </label>
              <textarea
                value={message}
                onChange={(e) => setMessage(e.target.value)}
                placeholder="Enter message text"
                rows={3}
                className="w-full bg-secondarybg text-white border border-gray-600 rounded-lg p-1.5 text-sm focus:border-accentx focus:outline-none resize-none"
              ></textarea>
            </div>

            <div className="flex justify-end">
              <button
                onClick={handleSendInputBox}
                disabled={!title.trim() || !message.trim() || isSending}
                className={`flex items-center gap-1 py-1 px-3 rounded text-sm ${
                  !title.trim() || !message.trim() || isSending
                    ? "bg-accentx border border-accentx text-black !cursor-not-allowed"
                    : "bg-accentx text-white hover:bg-white hover:text-accentx border border-accentx cursor-pointer"
                } transition-colors`}
              >
                <IconSend size={14} />
                <span>Send</span>
              </button>
            </div>
          </div>
        </div>

        <div className="bg-primarybg rounded-lg p-3 flex-1 flex flex-col min-h-0">
          <h3 className="text-accenttext font-semibold mb-2 text-xs flex items-center gap-1">
            <IconHistory size={14} />
            <span>RESPONSE LOG</span>
          </h3>

          <div className="flex-1 overflow-y-auto pr-1 min-h-0">
            {inputBoxResults.length > 0 ? (
              <div className="space-y-2">
                {inputBoxResults.map((result, index) => (
                  <div
                    key={index}
                    className="border border-accentx rounded-lg p-2"
                  >
                    <div className="flex items-center gap-1 mb-1">
                      {result.text.toLowerCase() === "ok" ||
                      result.text.toLowerCase() === "yes" ? (
                        <IconCheck size={16} className="text-green-400" />
                      ) : result.text.toLowerCase() === "cancel" ||
                        result.text.toLowerCase() === "no" ? (
                        <IconX size={16} className="text-red-400" />
                      ) : (
                        <IconClock size={16} className="text-blue-400" />
                      )}
                      <span className="font-medium text-xs text-accenttext">
                        {result.timestamp}
                      </span>
                    </div>
                    <p className="text-xs text-white break-words whitespace-pre-wrap overflow-hidden">
                      {result.text}
                    </p>
                  </div>
                ))}
              </div>
            ) : (
              <div className="h-full flex items-center justify-center">
                <p className="text-gray-500 text-sm">No responses yet</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
