import { useState } from "react";
import { useParams } from "react-router-dom";
import { sendTrollCommand } from "../rat/RATCommands";
import { TrollCommand } from "../../types";
import {
  IconDeviceDesktop,
  IconDeviceDesktopOff,
  IconLayoutBottombar,
  IconLayoutBottombarCollapse,
  IconBellRinging,
  IconBellOff,
  IconArrowAutofitDown,
  IconTrash,
  IconMouse,
  IconMouseOff,
  IconScreenShare,
  IconScreenShareOff,
  IconVolume,
  IconVolumeOff,
  IconVolume3,
  IconVolume2,
} from "@tabler/icons-react";

enum TrollCommandType {
  HideDesktop = "HideDesktop",
  ShowDesktop = "ShowDesktop",
  HideTaskbar = "HideTaskbar",
  ShowTaskbar = "ShowTaskbar",
  HideNotify = "HideNotify",
  ShowNotify = "ShowNotify",
  FocusDesktop = "FocusDesktop",
  EmptyTrash = "EmptyTrash",
  RevertMouse = "RevertMouse",
  NormalMouse = "NormalMouse",
  MonitorOff = "MonitorOff",
  MonitorOn = "MonitorOn",
  MaxVolume = "MaxVolume",
  MinVolume = "MinVolume",
  MuteVolume = "MuteVolume",
  UnmuteVolume = "UnmuteVolume",
  SpeakText = "SpeakText",
}

export const Troll = () => {
  const { addr } = useParams();
  const [activeCommands, setActiveCommands] = useState<Record<string, boolean>>(
    {}
  );
  const [loading, setLoading] = useState<Record<string, boolean>>({});
  const [ttsText, setTtsText] = useState<string>("");

  const handleTrollCommand = async (command: TrollCommand) => {
    if (!addr) return;

    // Set loading state for this command
    setLoading((prev) => ({ ...prev, [command.type]: true }));

    try {
      await sendTrollCommand(addr, command);

      // For one-time actions, briefly show them as active then reset
      if (isOneTimeAction(command)) {
        setActiveCommands((prev) => ({ ...prev, [command.type]: true }));

        // Reset active state after a short delay
        setTimeout(() => {
          setActiveCommands((prev) => ({ ...prev, [command.type]: false }));
        }, 1000);
      }
      // For toggle commands, update the active state
      else if (isToggleCommand(command)) {
        setActiveCommands((prev) => {
          const newState = { ...prev };

          // If this is a "show" command, set its pair to inactive
          if (command.type.startsWith("Show")) {
            const hideCommand = command.type.replace("Show", "Hide");
            newState[hideCommand] = false;
            newState[command.type] = true;
          }
          // If this is a "hide" command, set its pair to inactive
          else if (command.type.startsWith("Hide")) {
            const showCommand = command.type.replace("Hide", "Show");
            newState[showCommand] = false;
            newState[command.type] = true;
          }
          // For other toggle commands
          else if (command.type === TrollCommandType.NormalMouse) {
            newState[TrollCommandType.RevertMouse] = false;
            newState[command.type] = true;
          } else if (command.type === TrollCommandType.RevertMouse) {
            newState[TrollCommandType.NormalMouse] = false;
            newState[command.type] = true;
          } else if (command.type === TrollCommandType.MonitorOn) {
            newState[TrollCommandType.MonitorOff] = false;
            newState[command.type] = true;
          } else if (command.type === TrollCommandType.MonitorOff) {
            newState[TrollCommandType.MonitorOn] = false;
            newState[command.type] = true;
          } else if (command.type === TrollCommandType.UnmuteVolume) {
            newState[TrollCommandType.MuteVolume] = false;
            newState[command.type] = true;
          } else if (command.type === TrollCommandType.MuteVolume) {
            newState[TrollCommandType.UnmuteVolume] = false;
            newState[command.type] = true;
          } else if (command.type === TrollCommandType.MaxVolume) {
            newState[TrollCommandType.MinVolume] = false;
            newState[TrollCommandType.MuteVolume] = false;
            newState[command.type] = true;
          } else if (command.type === TrollCommandType.MinVolume) {
            newState[TrollCommandType.MaxVolume] = false;
            newState[TrollCommandType.MuteVolume] = false;
            newState[command.type] = true;
          }

          return newState;
        });
      }
    } catch (error) {
      console.error("Failed to send troll command:", error);
    } finally {
      // Clear loading state
      setLoading((prev) => ({ ...prev, [command.type]: false }));
    }
  };

  const handleTrollCommandWithText = async (command: TrollCommand) => {
    if (!addr) return;

    // Set loading state for this command
    setLoading((prev) => ({ ...prev, [command.type]: true }));

    try {
      await sendTrollCommand(addr, {
        type: command.type,
        payload: ttsText,
      });
    } catch (error) {
      console.error("Failed to send troll command:", error);
    } finally {
      setLoading((prev) => ({ ...prev, [command.type]: false }));
    }
  };

  // Helper to determine if a command is part of a toggle pair
  const isToggleCommand = (command: TrollCommand): boolean => {
    return [
      TrollCommandType.HideDesktop,
      TrollCommandType.ShowDesktop,
      TrollCommandType.HideTaskbar,
      TrollCommandType.ShowTaskbar,
      TrollCommandType.HideNotify,
      TrollCommandType.ShowNotify,
      TrollCommandType.RevertMouse,
      TrollCommandType.NormalMouse,
      TrollCommandType.MonitorOff,
      TrollCommandType.MonitorOn,
      TrollCommandType.MuteVolume,
      TrollCommandType.UnmuteVolume,
      // These are not actual toggles, but we want to track their button state
      TrollCommandType.MaxVolume,
      TrollCommandType.MinVolume,
    ].includes(command.type);
  };

  // Helper to determine if a command is a one-time action rather than a toggle
  const isOneTimeAction = (command: TrollCommand): boolean => {
    return [
      TrollCommandType.FocusDesktop,
      TrollCommandType.EmptyTrash,
    ].includes(command.type);
  };

  // Get the appropriate CSS class for a button based on its state
  const getButtonClass = (
    command: TrollCommand,
    placeholder: boolean = false
  ): string => {
    if (placeholder) {
      return "bg-gray-800 text-gray-500 border-accentx cursor-not-allowed opacity-60";
    }

    if (loading[command.type]) {
      return "bg-gray-700 border-gray-600 text-gray-300 cursor-wait";
    }

    if (activeCommands[command.type]) {
      return "bg-green-900 border-green-700 text-white";
    }

    return "bg-secondarybg border-accentx text-white hover:bg-accentx hover:border-accentx transition-all duration-200";
  };

  return (
    <div className="p-3 flex flex-1 flex-col overflow-auto w-full bg-primarybg h-screen">
      {/* Header */}
      <div className="flex justify-between items-center mb-3">
        <div className="flex items-center gap-2">
          <svg
            className="text-accentx"
            width="20"
            height="20"
            viewBox="0 0 24 24"
            strokeWidth="1.5"
            stroke="currentColor"
            fill="none"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <path stroke="none" d="M0 0h24v24H0z" fill="none" />
            <path d="M8 9l3 3l-3 3" />
            <path d="M13 15l3 0" />
            <path d="M3 5a2 2 0 0 1 2 -2h14a2 2 0 0 1 2 2v14a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-14z" />
          </svg>
          <h1 className="text-xl font-medium text-white">Troll Panel</h1>
        </div>
      </div>

      {/* Warning */}
      <div className="bg-yellow-900/30 border border-yellow-800 rounded-lg p-2 mb-3 text-md">
        <div className="flex items-start">
          <svg
            className="text-yellow-500 mt-0.5 mr-2 flex-shrink-0"
            width="14"
            height="14"
            viewBox="0 0 24 24"
            strokeWidth="2"
            stroke="currentColor"
            fill="none"
          >
            <path stroke="none" d="M0 0h24v24H0z" fill="none"></path>
            <path d="M12 9v4"></path>
            <path d="M10.363 3.591l-8.106 13.534a1.914 1.914 0 0 0 1.636 2.871h16.214a1.914 1.914 0 0 0 1.636 -2.87l-8.106 -13.536a1.914 1.914 0 0 0 -3.274 0z"></path>
            <path d="M12 16h.01"></path>
          </svg>
          <div>
            <h3 className="text-yellow-500 font-medium text-md">
              Use with caution
            </h3>
            <p className="text-yellow-100/70 text-md mt-0.5">
              These commands manipulate the client's interface in real-time.
              Actions like hiding taskbars, controlling volume, and toggling
              displays are visible to the user and may alert them to your
              presence.
            </p>
          </div>
        </div>
      </div>

      {!addr ? (
        <div className="flex items-center justify-center h-32 bg-secondarybg border border-accentx rounded-lg">
          <p className="text-gray-400 text-md">No client selected</p>
        </div>
      ) : (
        <div className="grid grid-cols-2 lg:grid-cols-4 gap-2">
          {/* DESKTOP CONTROLS */}
          <div className="bg-secondarybg border border-accentx rounded-lg overflow-hidden">
            <div className="bg-primarybg border-b border-accentx px-2 py-1.5">
              <h3 className="text-white font-medium text-md">
                Desktop & Taskbar
              </h3>
            </div>
            <div className="p-2 space-y-1.5">
              <TrollButton
                title="Hide Desktop Icons"
                icon={<IconDeviceDesktopOff size={16} />}
                command={{ type: TrollCommandType.HideDesktop }}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommandType.HideDesktop]}
                loading={loading[TrollCommandType.HideDesktop]}
              />

              <TrollButton
                title="Show Desktop Icons"
                icon={<IconDeviceDesktop size={16} />}
                command={{ type: TrollCommandType.ShowDesktop }}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommandType.ShowDesktop]}
                loading={loading[TrollCommandType.ShowDesktop]}
              />

              <TrollButton
                title="Hide Taskbar"
                icon={<IconLayoutBottombarCollapse size={16} />}
                command={{ type: TrollCommandType.HideTaskbar }}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommandType.HideTaskbar]}
                loading={loading[TrollCommandType.HideTaskbar]}
              />

              <TrollButton
                title="Show Taskbar"
                icon={<IconLayoutBottombar size={16} />}
                command={{ type: TrollCommandType.ShowTaskbar }}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommandType.ShowTaskbar]}
                loading={loading[TrollCommandType.ShowTaskbar]}
              />
            </div>
          </div>

          {/* NOTIFICATION CONTROLS */}
          <div className="bg-secondarybg border border-accentx rounded-lg overflow-hidden">
            <div className="bg-primarybg border-b border-accentx px-2 py-1.5">
              <h3 className="text-white font-medium text-md">System Actions</h3>
            </div>
            <div className="p-2 space-y-1.5">
              <TrollButton
                title="Hide Notification Area"
                icon={<IconBellOff size={16} />}
                command={{ type: TrollCommandType.HideNotify }}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommandType.HideNotify]}
                loading={loading[TrollCommandType.HideNotify]}
              />

              <TrollButton
                title="Show Notification Area"
                icon={<IconBellRinging size={16} />}
                command={{ type: TrollCommandType.ShowNotify }}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommandType.ShowNotify]}
                loading={loading[TrollCommandType.ShowNotify]}
              />

              <TrollButton
                title="Focus Desktop"
                icon={<IconArrowAutofitDown size={16} />}
                command={{ type: TrollCommandType.FocusDesktop }}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommandType.FocusDesktop]}
                loading={loading[TrollCommandType.FocusDesktop]}
              />

              <TrollButton
                title="Empty Recycle Bin"
                icon={<IconTrash size={16} />}
                command={{ type: TrollCommandType.EmptyTrash }}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommandType.EmptyTrash]}
                loading={loading[TrollCommandType.EmptyTrash]}
              />
            </div>
          </div>

          {/* INPUT CONTROLS */}
          <div className="bg-secondarybg border border-accentx rounded-lg overflow-hidden">
            <div className="bg-primarybg border-b border-accentx px-2 py-1.5">
              <h3 className="text-white font-medium text-md">
                Mouse & Display
              </h3>
            </div>
            <div className="p-2 space-y-1.5">
              <TrollButton
                title="Invert Mouse Buttons"
                icon={<IconMouseOff size={16} />}
                command={{ type: TrollCommandType.RevertMouse }}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommandType.RevertMouse]}
                loading={loading[TrollCommandType.RevertMouse]}
              />

              <TrollButton
                title="Normal Mouse"
                icon={<IconMouse size={16} />}
                command={{ type: TrollCommandType.NormalMouse }}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommandType.NormalMouse]}
                loading={loading[TrollCommandType.NormalMouse]}
              />

              <TrollButton
                title="Turn Off Monitor"
                icon={<IconScreenShareOff size={16} />}
                command={{ type: TrollCommandType.MonitorOff }}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommandType.MonitorOff]}
                loading={loading[TrollCommandType.MonitorOff]}
              />

              <TrollButton
                title="Turn On Monitor"
                icon={<IconScreenShare size={16} />}
                command={{ type: TrollCommandType.MonitorOn }}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommandType.MonitorOn]}
                loading={loading[TrollCommandType.MonitorOn]}
              />
            </div>
          </div>

          {/* AUDIO CONTROLS */}
          <div className="bg-secondarybg border border-accentx rounded-lg overflow-hidden">
            <div className="bg-primarybg border-b border-accentx px-2 py-1.5">
              <h3 className="text-white font-medium text-md">Volume Control</h3>
            </div>
            <div className="p-2 space-y-1.5">
              <TrollButton
                title="Maximum Volume"
                icon={<IconVolume3 size={16} />}
                command={{ type: TrollCommandType.MaxVolume }}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommandType.MaxVolume]}
                loading={loading[TrollCommandType.MaxVolume]}
              />

              <TrollButton
                title="Minimum Volume"
                icon={<IconVolume2 size={16} />}
                command={{ type: TrollCommandType.MinVolume }}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommandType.MinVolume]}
                loading={loading[TrollCommandType.MinVolume]}
              />

              <TrollButton
                title="Mute Volume"
                icon={<IconVolumeOff size={16} />}
                command={{ type: TrollCommandType.MuteVolume }}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommandType.MuteVolume]}
                loading={loading[TrollCommandType.MuteVolume]}
              />

              <TrollButton
                title="Unmute Volume"
                icon={<IconVolume size={16} />}
                command={{ type: TrollCommandType.UnmuteVolume }}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommandType.UnmuteVolume]}
                loading={loading[TrollCommandType.UnmuteVolume]}
              />
            </div>
          </div>
        </div>
      )}

      {/* Text-to-Speech, Piano Tiles, and Beep Sounds */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-2 mt-4">
        {/* Text-to-Speech */}
        <div className="bg-secondarybg border border-accentx rounded-lg overflow-hidden">
          <div className="bg-primarybg border-b border-accentx px-2 py-1.5">
            <h3 className="text-white font-medium text-md">Text-to-Speech</h3>
          </div>
          <div className="p-2 space-y-1.5">
            <div className="flex gap-2">
              <input
                type="text"
                value={ttsText}
                onChange={(e) => setTtsText(e.target.value)}
                placeholder="Enter text to speak..."
                className="flex-1 p-2 bg-secondarybg text-white border border-accentx rounded"
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    handleTrollCommandWithText({
                      type: TrollCommandType.SpeakText,
                      payload: ttsText,
                    });
                  }
                }}
              />
              <button
                onClick={() =>
                  handleTrollCommandWithText({
                    type: TrollCommandType.SpeakText,
                    payload: ttsText,
                  })
                }
                disabled={
                  !ttsText.trim() || loading[TrollCommandType.SpeakText]
                }
                className={`p-2 rounded ${
                  !ttsText.trim() || loading[TrollCommandType.SpeakText]
                    ? "bg-gray-700 border-gray-600 text-gray-300 cursor-not-allowed"
                    : "bg-accentx border-accent hover:bg-accent text-white"
                }`}
              >
                {loading[TrollCommandType.SpeakText] ? (
                  <svg
                    className="animate-spin h-5 w-5 text-white"
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                  >
                    <circle
                      className="opacity-25"
                      cx="12"
                      cy="12"
                      r="10"
                      stroke="currentColor"
                      strokeWidth="4"
                    ></circle>
                    <path
                      className="opacity-75"
                      fill="currentColor"
                      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                    ></path>
                  </svg>
                ) : (
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    className="h-5 w-5"
                    viewBox="0 0 24 24"
                    strokeWidth="1.5"
                    stroke="currentColor"
                    fill="none"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  >
                    <path stroke="none" d="M0 0h24v24H0z" fill="none" />
                    <path d="M15 8a5 5 0 0 1 0 8" />
                    <path d="M17.7 5a9 9 0 0 1 0 14" />
                    <path d="M6 15h-2a1 1 0 0 1 -1 -1v-4a1 1 0 0 1 1 -1h2l3.5 -4.5a.8 .8 0 0 1 1.5 .5v14a.8 .8 0 0 1 -1.5 .5l-3.5 -4.5" />
                  </svg>
                )}
              </button>
            </div>
            <p className="text-gray-400 text-xs">
              Make the client's computer speak the text.
            </p>
          </div>
        </div>

        {/* Piano Tiles */}
        <div className="bg-secondarybg border border-accentx rounded-lg overflow-hidden">
          <div className="bg-primarybg border-b border-accentx px-2 py-1.5">
            <h3 className="text-white font-medium text-md">Piano Tiles</h3>
          </div>
          <div className="p-2">
            <div className="grid grid-cols-8 gap-1 mb-2">
              {[...Array(16)].map((_, index) => (
                <button
                  key={index}
                  className={`${
                    index % 2 === 0 ? "bg-white" : "bg-gray-900"
                  } h-12 ${
                    index % 2 === 0 ? "border-gray-300" : "border-gray-800"
                  } border rounded hover:opacity-80 transition-opacity`}
                  onClick={() => {
                    /* Piano tile click logic will go here */
                  }}
                ></button>
              ))}
            </div>
            <p className="text-gray-400 text-xs">
              Click keys to play sounds on the remote computer.
            </p>
          </div>
        </div>

        {/* Beep Command */}
        <div className="bg-secondarybg border border-accentx rounded-lg overflow-hidden">
          <div className="bg-primarybg border-b border-accentx px-2 py-1.5">
            <h3 className="text-white font-medium text-md">Beep Sound</h3>
          </div>
          <div className="p-2 space-y-1.5">
            <div className="grid grid-cols-2 gap-2">
              <div>
                <label className="text-gray-400 text-xs block mb-1">
                  Frequency (Hz)
                </label>
                <input
                  type="range"
                  min="100"
                  max="5000"
                  className="w-full"
                  // value={beepFrequency}
                  // onChange={(e) => setBeepFrequency(parseInt(e.target.value))}
                />
                <div className="flex justify-between mt-1">
                  <span className="text-xs text-gray-400">100</span>
                  <span className="text-xs text-gray-400">5000</span>
                </div>
              </div>
              <div>
                <label className="text-gray-400 text-xs block mb-1">
                  Duration (ms)
                </label>
                <input
                  type="number"
                  min="100"
                  max="5000"
                  // value={beepDuration}
                  // onChange={(e) => setBeepDuration(parseInt(e.target.value))}
                  className="w-full p-1.5 bg-secondarybg text-white border border-accentx rounded text-sm"
                  placeholder="Duration"
                />
              </div>
            </div>
            <button
              // onClick={handleBeepSubmit}
              className="w-full py-1 px-2 bg-accentx hover:bg-accent text-white rounded transition-all duration-200"
            >
              Play Beep
            </button>
          </div>
        </div>
      </div>

      {/* Credits */}
      <div className="mt-auto pt-4 border-t border-gray-800">
        <p className="text-xs text-gray-500">
          © AsyncRAT - All rights reserved.
        </p>
      </div>
    </div>
  );
};

interface TrollButtonProps {
  title: string;
  icon: React.ReactNode;
  command: TrollCommand;
  onClick: (command: TrollCommand) => void;
  active?: boolean;
  loading?: boolean;
  placeholder?: boolean;
}

const TrollButton = ({
  title,
  icon,
  command,
  onClick,
  active = false,
  loading = false,
  placeholder = false,
}: TrollButtonProps) => {
  return (
    <button
      className={`w-full flex items-center gap-2 px-2 py-1.5 rounded text-md border ${
        placeholder
          ? "bg-gray-800 text-gray-500 border-accentx opacity-60"
          : loading
          ? "bg-gray-700 border-gray-600 text-gray-300"
          : active
          ? "bg-green-900 border-green-700 text-white"
          : "bg-secondarybg border-accentx text-white hover:bg-accentx hover:border-accentx transition-all duration-200"
      } ${!placeholder && !loading ? "cursor-pointer" : "cursor-not-allowed"}`}
      onClick={() => !loading && !placeholder && onClick(command)}
      disabled={loading || placeholder}
    >
      {/* Icon with loading indicator */}
      <div className="flex-shrink-0">
        {loading ? (
          <div className="animate-spin">
            <svg
              className="w-3.5 h-3.5"
              viewBox="0 0 24 24"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                opacity="0.2"
                d="M12 2C6.47715 2 2 6.47715 2 12C2 17.5228 6.47715 22 12 22C17.5228 22 22 17.5228 22 12C22 6.47715 17.5228 2 12 2Z"
                fill="currentColor"
              />
              <path
                d="M12 22C17.5228 22 22 17.5228 22 12H18C18 15.3137 15.3137 18 12 18C8.68629 18 6 15.3137 6 12C6 8.68629 8.68629 6 12 6V2C6.47715 2 2 6.47715 2 12C2 17.5228 6.47715 22 12 22Z"
                fill="currentColor"
              />
            </svg>
          </div>
        ) : (
          icon
        )}
      </div>

      {/* Text content */}
      <span className="font-medium truncate">{title}</span>

      {placeholder && (
        <span className="ml-auto text-[10px] text-gray-500">Soon</span>
      )}
    </button>
  );
};
