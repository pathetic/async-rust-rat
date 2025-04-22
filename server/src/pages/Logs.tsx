import { useContext, useState, useEffect, useMemo } from "react";
import { RATContext } from "../rat/RATContext";
import { Log } from "../../types";
import {
  IconAlertCircle,
  IconInfoCircle,
  IconCheck,
  IconX,
  IconCloudDownload,
  IconClockHour4,
  IconCloudUpload,
} from "@tabler/icons-react";
import { TableFilter } from "../components/TableFilter";

export const Logs = () => {
  const ratContext = useContext(RATContext);
  const serverLogs = ratContext?.serverLogs || [];
  const [searchTerm, setSearchTerm] = useState("");
  const [filter, setFilter] = useState<Record<string, boolean>>({});

  // Initialize filters based on event types in logs
  useEffect(() => {
    if (serverLogs && serverLogs.length > 0) {
      const eventTypes = new Set<string>();
      serverLogs.forEach((log: Log) => eventTypes.add(log.event_type));

      const initialFilters: Record<string, boolean> = {};
      eventTypes.forEach((type) => {
        initialFilters[type] = true;
      });

      setFilter(initialFilters);
    }
  }, [serverLogs]);

  // Filter logs based on search term and selected filters
  const filteredLogs = useMemo(() => {
    return serverLogs.filter((log: Log) => {
      const matchesSearch =
        searchTerm === "" ||
        log.event_type.toLowerCase().includes(searchTerm.toLowerCase()) ||
        log.message.toLowerCase().includes(searchTerm.toLowerCase()) ||
        log.time.toLowerCase().includes(searchTerm.toLowerCase());

      const passesFilter = filter[log.event_type] !== false;

      return matchesSearch && passesFilter;
    });
  }, [serverLogs, searchTerm, filter]);

  const getLogIcon = (eventType: string) => {
    switch (eventType.toLowerCase()) {
      case "server_stopped":
        return <IconAlertCircle className="text-red-500" size={20} />;
      case "warning":
        return <IconAlertCircle className="text-amber-500" size={20} />;
      case "server_started":
        return <IconInfoCircle className="text-blue-500" size={20} />;
      case "client_connected":
        return <IconCheck className="text-green-500" size={20} />;
      case "cmd_sent":
        return <IconCloudUpload className="text-purple-500" size={20} />;
      case "cmd_rcvd":
        return <IconCloudDownload className="text-orange-500" size={20} />;
      case "client_disconnected":
        return <IconX className="text-red-500" size={20} />;
      case "build_client":
        return <IconInfoCircle className="text-blue-400" size={20} />;
      case "build_finished":
        return <IconCheck className="text-green-500" size={20} />;
      case "build_failed":
        return <IconX className="text-red-500" size={20} />;
      default:
        return <IconInfoCircle className="text-gray-400" size={20} />;
    }
  };

  const getEventTypeClass = (eventType: string) => {
    switch (eventType.toLowerCase()) {
      case "server_stopped":
        return "text-red-400";
      case "warning":
        return "text-amber-400";
      case "server_started":
        return "text-blue-400";
      case "client_connected":
        return "text-green-400";
      case "cmd_sent":
        return "text-purple-400";
      case "cmd_rcvd":
        return "text-orange-400";
      case "client_disconnected":
        return "text-red-400";
      case "build_client":
        return "text-blue-400";
      case "build_finished":
        return "text-green-400";
      case "build_failed":
        return "text-red-400";
      default:
        return "text-gray-400";
    }
  };

  return (
    <div className="flex flex-col h-full w-full bg-primarybg overflow-hidden">
      <div className="flex flex-col">
        <div className="flex items-center justify-between"></div>

        <TableFilter
          searchTerm={searchTerm}
          setSearchTerm={setSearchTerm}
          searchPlaceholder="Search logs..."
          filters={filter}
          setFilters={setFilter}
        />
      </div>

      <div className="flex-1 overflow-auto rounded-2xl border bg-secondarybg">
        {filteredLogs.length > 0 ? (
          <table className="min-w-full bg-secondarybg divide-y divide-accentx">
            <thead className="bg-white/90">
              <tr>
                <th
                  scope="col"
                  className="px-6 py-3 font-semibold text-gray-900 text-left w-48"
                >
                  Time
                </th>
                <th
                  scope="col"
                  className="px-6 py-3 font-semibold text-gray-900 text-left w-32"
                >
                  Type
                </th>
                <th
                  scope="col"
                  className="px-6 py-3 font-semibold text-gray-900 text-left"
                >
                  Message
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-accentx">
              {filteredLogs.map((log: Log, index: number) => (
                <tr
                  key={`${log.time}-${index}`}
                  className="hover:bg-[#1f1f1f] transition-colors"
                >
                  <td className="px-6 py-3 whitespace-nowrap text-sm text-white flex items-center">
                    <IconClockHour4 size={16} className="mr-2 text-white" />
                    {log.time}
                  </td>
                  <td className="px-6 py-3 whitespace-nowrap">
                    <span
                      className={`bg-primarybg border border-accentx inline-flex items-center px-2.5 py-0.5 rounded-md text-xs font-medium ${getEventTypeClass(
                        log.event_type
                      )}`}
                    >
                      {getLogIcon(log.event_type)}
                      <span className="ml-1">{log.event_type}</span>
                    </span>
                  </td>
                  <td className="px-6 py-3 text-sm text-white">
                    {log.message}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          <div className="flex items-center justify-center h-64 text-white">
            {searchTerm || Object.values(filter).includes(false)
              ? "No logs match the current filters"
              : "No logs available"}
          </div>
        )}
      </div>
    </div>
  );
};
