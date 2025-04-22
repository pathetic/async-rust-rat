import { useState } from "react";
import { visitWebsiteCmd } from "../../../rat/RATCommands";

export const VisitWebsiteModal = ({
  selectedClient,
}: {
  selectedClient: string;
}) => {
  const [url, setUrl] = useState("");

  const handleVisitWebsite = () => {
    visitWebsiteCmd(selectedClient, url);
  };

  return (
    <dialog id="visit_website_modal" className="modal">
      <div className="modal-box bg-primarybg text-white w-80 border border-accentx rounded-2xl">
        <h3 className="font-bold text-lg">Visit Website</h3>
        <div className="form-control mt-4 flex flex-col">
          <label className="rounded-3xl pl-4 input input-bordered flex items-center gap-2 border-accentx bg-secondarybg">
            URL
            <input
              type="text"
              placeholder="https://example.com"
              className="grow"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
            />
          </label>

          <span
            onClick={() => handleVisitWebsite()}
            className="w-32 btn mt-4 bg-secondarybg text-white border border-accentx rounded-3xl hover:bg-white hover:text-black transition"
          >
            Visit Website
          </span>
        </div>
      </div>

      <form method="dialog" className="modal-backdrop">
        <button>close</button>
      </form>
    </dialog>
  );
};
