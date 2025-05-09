import { VectorMap } from "@south-paw/react-vector-maps";
import { RATContext } from "../rat/RATContext";
import { useContext, useMemo, useState, useRef } from "react";
import {
  extendedWorldData,
  computePosition,
  ToolTip,
} from "../components/world/world";

export const WorldMap = () => {
  const { clientList } = useContext(RATContext)!;
  const [tooltipContent, setTooltipContent] = useState<string | null>(null);
  const [tooltipPosition, setTooltipPosition] = useState<{
    x: number;
    y: number;
  } | null>(null);
  const mapRef = useRef<HTMLDivElement>(null);

  const clientCounts = useMemo(() => {
    const counts: Record<string, number> = {};
    clientList.forEach(({ data }) => {
      const code = data.country_code.toUpperCase();
      counts[code] = (counts[code] || 0) + 1;
    });
    return counts;
  }, [clientList]);

  const generatedStyles = useMemo(() => {
    let styles = "";

    for (const [countryCode, count] of Object.entries(clientCounts)) {
      console.log(countryCode);
      const normalized = Math.min(count / 10, 1);
      const opacity = 0.2 + normalized * 0.8; // 0.2 to 1.0 opacity scaling
      styles += `
        .world-map svg path[id="${countryCode.toLowerCase()}"] {
          fill: rgba(20,71,230,${opacity.toFixed(2)});
        }
      `;
    }

    styles += `
    .world-map svg path[id="private"] {
      fill: rgba(255,255,255, 1) !important;
    }
    `;

    styles += `
    .world-map svg path[id="total_clients"] {
      fill: rgba(255, 255, 255, 1) !important;
    }
    `;

    return styles;
  }, [clientCounts]);

  const layerProps = {
    onMouseEnter: (event: any) => {
      const countryName = event.target.attributes.name?.value;
      const countryId = event.target.attributes.id?.value?.toUpperCase();
      const count = clientCounts[countryId] || 0;

      const { x, y } = computePosition(event, mapRef);
      setTooltipPosition({ x, y });

      if (countryName === "Total Clients") {
        setTooltipContent(
          `${countryName}: ${clientList.length} Client${
            clientList.length !== 1 ? "s" : ""
          }`
        );
      } else {
        setTooltipContent(
          `${countryName}: ${count} Client${count !== 1 ? "s" : ""}`
        );
      }
    },
    onMouseMove: (event: any) => {
      const { x, y } = computePosition(event, mapRef);
      setTooltipPosition({ x, y });
    },
    onMouseLeave: () => {
      setTooltipContent(null);
      setTooltipPosition(null);
    },
    onClick: (event: any) => {
      const countryName = event.target.attributes.name?.value;
      console.log("Clicked on", countryName);
    },
  };

  return (
    <>
      <style>{generatedStyles}</style>

      <div
        ref={mapRef}
        className="world-map relative w-full h-full bg-secondarybg world-map"
      >
        <VectorMap
          {...extendedWorldData}
          layerProps={layerProps}
          style={{
            width: "100%",
            height: "100%",
            padding: "5px",
          }}
        />

        <ToolTip
          tooltipContent={tooltipContent}
          tooltipPosition={tooltipPosition}
        />
      </div>
    </>
  );
};
