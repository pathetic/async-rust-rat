import { useState } from "react";
import { visitWebsiteCmd } from "../../../rat/RATCommands";
import { IconLink, IconSend, IconWorld, IconX } from "@tabler/icons-react";

export const VisitWebsiteModal = ({
  selectedClient,
}: {
  selectedClient: string;
}) => {
  const [url, setUrl] = useState("");
  const [isValidUrl, setIsValidUrl] = useState(true);

  const validateUrl = (input: string) => {
    if (!input) return true; // Empty is fine, just don't allow to proceed
    try {
      new URL(input);
      return true;
    } catch (e) {
      return false;
    }
  };

  const handleUrlChange = (value: string) => {
    setUrl(value);
    setIsValidUrl(validateUrl(value));
  };

  const handleVisitWebsite = () => {
    if (!url || !isValidUrl) return;
    visitWebsiteCmd(selectedClient, url);

    const modal = document.getElementById(
      "visit_website_modal"
    ) as HTMLDialogElement;
    if (modal) modal.close();
  };

  return (
    <dialog id="visit_website_modal" className="modal">
      <div className="modal-box w-[450px] bg-primarybg text-white border border-accentx rounded-xl shadow-xl backdrop-blur-sm transform transition-all duration-300 ease-out">
        <div className="flex justify-between items-center border-b border-accentx pb-3 mb-4">
          <h3 className="font-bold text-lg text-gray-200 flex items-center gap-2">
            <span className="p-1.5 rounded-md">
              <IconWorld size={24} className="text-accentx" />
            </span>
            Visit Website
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
              Target URL
            </label>
            <div className="relative mt-3">
              <div className="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none">
                <IconLink size={18} className="text-gray-400" />
              </div>
              <input
                type="text"
                placeholder="https://example.com"
                className={`pl-10 pr-4 py-2.5 w-full text-sm bg-secondarybg rounded-lg border ${
                  isValidUrl
                    ? "border-gray-600 focus:border-accentx"
                    : "border-red-500"
                } focus:outline-none focus:ring-1 ${
                  isValidUrl ? "focus:ring-accentx" : "focus:ring-red-500"
                } transition-all`}
                value={url}
                onChange={(e) => handleUrlChange(e.target.value)}
              />
            </div>
            {!isValidUrl && (
              <p className="text-xs text-red-400 mt-1 ml-1">
                Please enter a valid URL (e.g., https://example.com)
              </p>
            )}
          </div>

          <div className="text-xs text-accenttext">
            <p>
              Make sure to include the full URL with http:// or https:// prefix.
            </p>
          </div>
        </div>

        <div className="flex justify-end gap-3 mt-6">
          <form method="dialog">
            <button className="px-4 py-2 text-sm font-medium rounded-lg bg-secondarybg text-gray-200 hover:bg-white hover:text-accentx border border-accentx hover:border-accentx transition-all duration-200 cursor-pointer">
              Cancel
            </button>
          </form>
          <button
            onClick={handleVisitWebsite}
            disabled={!url || !isValidUrl}
            className={`px-4 py-2 text-sm font-medium rounded-lg flex items-center gap-2 cursor-pointer transition-all duration-200 ${
              url && isValidUrl
                ? "bg-accentx text-white hover:bg-white hover:text-accentx border border-accentx"
                : "bg-accentx border border-accentx text-black !cursor-not-allowed"
            }`}
          >
            <IconSend size={16} />
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
