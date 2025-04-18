import React, { useContext, useEffect, useState, useRef } from "react";
import { useNavigate } from "react-router-dom";
import { RATContext } from "../rat/RATContext";
import { ContextMenuProps, SubMenuProps } from "../../types";
import {
  manageClientCmd,
  handleSystemCommandCmd,
  handleElevateCmd,
} from "../rat/RATCommands";

import {
  IconSettings2,
  IconDeviceDesktopCog,
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
        window: true,
      },
      {
        label: "File Manager",
        type: "file-manager",
        icon: <IconFolder size={24} />,
        window: true,
      },
      {
        label: "Reverse Shell",
        type: "reverse-shell",
        icon: <IconTerminal2 size={24} />,
        window: true,
      },
      {
        label: "Reverse Proxy",
        type: "reverse-proxy",
        icon: <IconNetwork size={24} />,
        window: true,
      },
      {
        label: "Process Viewer",
        type: "process-viewer",
        icon: <IconCpu2 size={24} />,
        window: true,
      },
      {
        label: "Visit Website",
        type: "visit-website",
        icon: <IconWorld size={24} />,
        navigate: false,
        modal: true,
        modalId: "visit_website_modal",
      },
      {
        label: "Elevate (UAC)",
        type: "elevate-privileges",
        icon: <IconShieldUp size={24} />,
        function: handleElevateCmd,
      },
      {
        label: "MessageBox",
        type: "show-message-box",
        icon: <IconMessage2 size={24} />,
        modal: true,
        modalId: "message_box_modal",
      },
    ],
    navigate: false,
  },
  {
    label: "System",
    icon: <IconDeviceDesktopCog size={24} />,
    options: [
      {
        label: "Shutdown",
        icon: <IconPower size={24} />,
        run: "shutdown",
        function: handleSystemCommandCmd,
      },
      {
        label: "Reboot",
        icon: <IconRotate size={24} />,
        run: "reboot",
        function: handleSystemCommandCmd,
      },
      {
        label: "Log Out",
        icon: <IconLogout2 size={24} />,
        run: "logout",
        function: handleSystemCommandCmd,
      },
    ],
    navigate: false,
  },
  {
    label: "Connection",
    icon: <IconPlugConnected size={24} />,
    options: [
      {
        label: "Reconnect",
        icon: <IconPlug size={24} />,
        run: "reconnect",
        function: manageClientCmd,
      },
      {
        label: "Disconnect",
        icon: <IconPlugConnectedX size={24} />,
        run: "disconnect",
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
  navigate,
  onClose,
}) => {
  const { openClientWindow } = useContext(RATContext)!;

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
            onClick={() => {
              if (item.window && typeof item.type === "string") {
                openClientWindow(addr, item.type, clientFullName);
              }
              if (item.navigate && typeof item.path === "string") {
                navigate(item.path.replace("[addr]", addr));
              }
              if (
                typeof item.function === "function" &&
                typeof item.run === "string"
              ) {
                item.function(String(addr), item.run);
              }
              if (item.modal && typeof item.modalId === "string") {
                (
                  document.getElementById(item.modalId) as HTMLDialogElement
                )?.showModal();
              }
              if (
                typeof item.function === "function" &&
                typeof item.run === "undefined"
              ) {
                item.function(String(addr));
              }
              onClose();
            }}
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
  const { setSelectedClient } = useContext(RATContext)!;
  const navigate = useNavigate();
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
              onMouseEnter={(e) => handleMouseEnter(index, e)}
            >
              {option.icon}
              {option.label}
              {!option.navigate && option.options && (
                <i className="ri-arrow-right-line ml-auto"></i>
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
            navigate={navigate}
            onClose={onClose}
          />
        </div>
      )}
    </>
  );
};
