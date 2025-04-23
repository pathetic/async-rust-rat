import { useState } from "react";
import { testMessageBoxCmd, sendMessageBoxCmd } from "../../../rat/RATCommands";
import { IconMessage2, IconX } from "@tabler/icons-react";

export const MessageBoxModal = ({
  selectedClient,
}: {
  selectedClient: string;
}) => {
  const [messageBoxTitle, setMessageBoxTitle] = useState<string>("");
  const [messageBoxContent, setMessageBoxContent] = useState<string>("");
  const [messageBoxButton, setMessageBoxButton] =
    useState<string>("abort_retry_ignore");
  const [messageBoxIcon, setMessageBoxIcon] = useState<string>("error");

  return (
    <dialog id="message_box_modal" className="modal">
      <div className="modal-box w-[450px] bg-primarybg text-white border border-gray-700 rounded-xl shadow-xl backdrop-blur-sm transform transition-all duration-300 ease-out">
        <div className="flex justify-between items-center border-b border-gray-700 pb-3 mb-4">
          <h3 className="font-bold text-lg text-gray-200 flex items-center gap-2">
            <span className="p-1.5 rounded-md">
              <IconMessage2 size={24} className="text-accentx" />
            </span>
            Show MessageBox
          </h3>
          <form method="dialog">
            <button className="cursor-pointer p-1 rounded-md hover:bg-secondarybg transition-colors">
              <IconX size={20} className="text-gray-400 hover:text-white" />
            </button>
          </form>
        </div>

        <div className="space-y-4">
          <div className="form-control">
            <label className="text-sm font-medium text-gray-300 mb-1.5 ml-1">
              Title
            </label>
            <input
              type="text"
              placeholder="Message Box Title"
              className="mt-1 px-4 py-2.5 w-full text-sm bg-secondarybg rounded-lg border border-gray-700 focus:border-accentx focus:outline-none focus:ring-1 focus:ring-accentx transition-all"
              value={messageBoxTitle}
              onChange={(e) => setMessageBoxTitle(e.target.value)}
            />
          </div>

          <div className="form-control">
            <label className="text-sm font-medium text-gray-300 mb-1.5 ml-1">
              Message Content
            </label>
            <textarea
              placeholder="Content that will appear in the message box"
              className="mt-1 px-4 py-2.5 w-full text-sm bg-secondarybg rounded-lg border border-gray-700 focus:border-accentx focus:outline-none focus:ring-1 focus:ring-accentx transition-all min-h-[80px] resize-none"
              value={messageBoxContent}
              onChange={(e) => setMessageBoxContent(e.target.value)}
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="form-control">
              <label className="text-sm font-medium text-gray-300 mb-1.5 ml-1">
                Button Style
              </label>
              <select
                className="mt-1 px-4 py-2.5 w-full text-sm bg-secondarybg rounded-lg border border-gray-700 focus:border-accentx focus:outline-none appearance-none cursor-pointer"
                value={messageBoxButton}
                onChange={(e) => setMessageBoxButton(e.target.value)}
              >
                <option value="abort_retry_ignore">AbortRetryIgnore</option>
                <option value="ok">OK</option>
                <option value="ok_cancel">OKCancel</option>
                <option value="retry_cnacel">RetryCancel</option>
                <option value="yes_no">YesNo</option>
                <option value="yes_no_cancel">YesNoCancel</option>
              </select>
            </div>

            <div className="form-control">
              <label className="text-sm font-medium text-gray-300 mb-1.5 ml-1">
                Icon Type
              </label>
              <select
                className="mt-1 px-4 py-2.5 w-full text-sm bg-secondarybg rounded-lg border border-gray-700 focus:border-accentx focus:outline-none appearance-none cursor-pointer"
                value={messageBoxIcon}
                onChange={(e) => setMessageBoxIcon(e.target.value)}
              >
                <option value="error">Error</option>
                <option value="question">Question</option>
                <option value="warning">Warning</option>
                <option value="info">Information</option>
                <option value="asterisk">Asterisk</option>
              </select>
            </div>
          </div>
        </div>

        <div className="flex justify-end gap-3 mt-6">
          <button
            onClick={() =>
              testMessageBoxCmd(
                messageBoxTitle,
                messageBoxContent,
                messageBoxButton,
                messageBoxIcon
              )
            }
            className="px-4 py-2 text-sm font-medium rounded-lg bg-secondarybg text-gray-200 hover:bg-gray-700 hover:text-white border border-gray-700 hover:border-accentx transition-all duration-200 cursor-pointer"
          >
            Test Locally
          </button>
          <button
            onClick={() =>
              sendMessageBoxCmd(
                String(selectedClient),
                messageBoxTitle,
                messageBoxContent,
                messageBoxButton,
                messageBoxIcon
              )
            }
            className="px-4 py-2 text-sm font-medium rounded-lg bg-accentx text-primarybg hover:bg-white hover:text-accentx border border-accentx transition-all duration-200 cursor-pointer"
          >
            Send to Target
          </button>
        </div>
      </div>

      <form method="dialog" className="modal-backdrop">
        <button>close</button>
      </form>
    </dialog>
  );
};
