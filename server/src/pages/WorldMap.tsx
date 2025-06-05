import { RATContext } from "../rat/RATContext";
import { useContext, useEffect, useMemo, useRef, useState } from "react";
import ReactECharts from "echarts-for-react";
import * as echarts from "echarts";

import worldmap from "../components/world/world.json";

export const WorldMap = () => {
  const { clientList } = useContext(RATContext)!;
  const mapRef = useRef<HTMLDivElement>(null);

  const [isMapRegistered, setIsMapRegistered] = useState(false);

  const clientCounts = useMemo(() => {
    const counts: Record<string, number> = {};
    clientList.forEach(({ data }) => {
      const code = data.country_code.toUpperCase();
      counts[code] = (counts[code] || 0) + 1;
      if (
        data.addr.includes("127.0.0.1") ||
        data.country_code == "LOCAL_HOST"
      ) {
        counts["Localhost"] = (counts["Localhost"] || 0) + 1;
      }
    });

    counts["Total Clients"] = clientList.length;
    return counts;
  }, [clientList]);

  const maxClients = useMemo(() => {
    const filteredCounts = Object.entries(clientCounts)
      .filter(([key]) => key !== "Total Clients")
      .filter(([key]) => key !== "Localhost")
      .map(([, count]) => count);

    return filteredCounts.length > 0 ? Math.max(...filteredCounts) : 1;
  }, [clientCounts]);

  useEffect(() => {
    if (!isMapRegistered) {
      echarts.registerMap("WORLD", worldmap as any);
      setIsMapRegistered(true);
    }
  }, []);

  const chartOptions = {
    backgroundColor: "#0e0e0e",
    series: [
      {
        name: "Connected Clients",
        type: "map",
        roam: true,
        map: "WORLD",
        data: Object.entries(clientCounts).map(([code, count]) => ({
          name: code,
          value: count,
        })),
        itemStyle: {
          areaColor: "rgba(23,23,23,0.1)",
          borderColor: "white",
        },
        label: {
          show: false,
        },
        emphasis: {
          label: {
            show: false,
          },
          itemStyle: {
            areaColor: "rgba(255,255,255,0.7)",
            borderColor: "white",
          },
        },
        selectedMode: false,
      },
    ],
    visualMap: {
      right: 10,
      bottom: 10,
      min: 0,
      max: maxClients,
      inRange: {
        color: ["rgba(20,71,230,0)", "rgba(20,71,230,1)"],
      },
      text: ["High", "Low"],
      textStyle: {
        color: "white",
      },
      calculable: true,
    },
    toolbox: {
      show: true,
      left: 5,
      top: 5,

      feature: {
        restore: {},
        saveAsImage: {},
      },
      iconStyle: {
        borderColor: "white",
      },
      textStyle: {
        color: "white",
      },
    },
    tooltip: {},
  };

  if (!isMapRegistered) {
    return <div>Loading map...</div>;
  }

  return (
    <>
      <div
        ref={mapRef}
        className="world-map relative w-full h-full bg-secondarybg world-map"
      >
        <ReactECharts
          option={chartOptions}
          style={{ width: "100%", height: "100%" }}
        />
      </div>
    </>
  );
};
