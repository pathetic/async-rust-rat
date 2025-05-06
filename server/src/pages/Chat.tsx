import { useState, useEffect } from "react";
import { useParams } from "react-router-dom";
import { sendChatMessageCmd } from "../rat/RATCommands";
import { listen } from "@tauri-apps/api/event";
import { IconSend, IconMessageDots } from "@tabler/icons-react";

export const Chat = () => {
  const { addr } = useParams();
  const [message, setMessage] = useState("");
  const [chatMessages, setChatMessages] = useState<
    Array<{ text: string; sender: string; timestamp: string }>
  >([]);

  const handleSendMessage = async () => {
    if (!message.trim()) return;

    try {
      // Add the admin message to the chat
      const now = new Date();
      const timestamp = now.toLocaleTimeString();

      setChatMessages((prev) => [
        ...prev,
        { text: message, sender: "Admin", timestamp },
      ]);

      // Send the message to the client
      await sendChatMessageCmd(addr, message);

      // Clear the input field
      setMessage("");
    } catch (error) {
      console.error("Error sending chat message:", error);
    }
  };

  // Handle Enter key press
  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
  };

  useEffect(() => {
    const setupListener = async () => {
      const unlisten = await listen("chat_message", (event: any) => {
        console.log(event);
        if (event.payload.addr === addr) {
          const now = new Date();
          const timestamp = now.toLocaleTimeString();

          setChatMessages((prev) => [
            ...prev,
            { text: event.payload.message, sender: "Client", timestamp },
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
      {/* Header */}
      <div className="sticky top-0 z-10 bg-secondarybg border-b border-gray-700 py-2 px-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <IconMessageDots size={20} className="text-accentx" />
            <h1 className="text-lg font-bold">Chat</h1>
          </div>
          <p className="text-gray-400 text-xs">{addr}</p>
        </div>
      </div>

      {/* Chat Messages */}
      <div className="flex-1 p-3 overflow-y-auto">
        {chatMessages.length > 0 ? (
          <div className="space-y-3">
            {chatMessages.map((msg, index) => (
              <div
                key={index}
                className={`flex flex-col ${
                  msg.sender === "Admin" ? "items-end" : "items-start"
                }`}
              >
                <div
                  className={`max-w-[80%] p-3 rounded-lg ${
                    msg.sender === "Admin"
                      ? "bg-accentx text-white"
                      : "bg-primarybg text-white"
                  }`}
                >
                  <p className="text-sm break-words whitespace-pre-wrap">
                    {msg.text}
                  </p>
                </div>
                <div className="flex items-center mt-1 text-xs text-gray-400">
                  <span className="font-semibold mr-1">{msg.sender}</span>
                  <span>{msg.timestamp}</span>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="h-full flex items-center justify-center">
            <p className="text-gray-500 text-sm">No messages yet</p>
          </div>
        )}
      </div>

      {/* Message Input */}
      <div className="p-3 border-t border-gray-700">
        <div className="flex gap-2">
          <input
            type="text"
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            onKeyDown={handleKeyPress}
            placeholder="Type a message..."
            className="flex-1 bg-primarybg text-white border border-gray-700 rounded-lg p-2 text-sm focus:border-accentx focus:outline-none"
          />
          <button
            onClick={handleSendMessage}
            disabled={!message.trim()}
            className={`p-2 rounded-lg ${
              !message.trim()
                ? "bg-gray-700 text-gray-400 cursor-not-allowed"
                : "bg-accentx text-white hover:bg-opacity-80"
            } transition-colors`}
          >
            <IconSend size={18} />
          </button>
        </div>
      </div>
    </div>
  );
};
