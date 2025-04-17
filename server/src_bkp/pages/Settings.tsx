import React, { useContext } from "react";
import { RATContext } from "../rat/RATContext";

export const Settings: React.FC = () => {
  const { setNotificationClient, notificationClient } = useContext(RATContext)!;

  return (
    <div className="p-8 flex flex-1 flex-row overflow-auto w-full">
      <div className="card bg-base-100 !min-w-[350px] h-[200px] shadow-xl border border-base-content">
        <div className="card-body gap-4">
          <h2 className="card-title">Frontend Settings</h2>
          <label className="cursor-pointer label bg-base-300 border border-base-content rounded-md px-4">
            <span className="label-text">Client Notifications</span>
            <input
              type="checkbox"
              checked={notificationClient}
              onChange={(e) => setNotificationClient(e.target.checked)}
              className="checkbox checkbox-accent"
            />
          </label>
        </div>
      </div>
    </div>
  );
};
