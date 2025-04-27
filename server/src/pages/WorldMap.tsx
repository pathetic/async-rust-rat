import { VectorMap } from "@south-paw/react-vector-maps";
import { RATContext } from "../rat/RATContext";
import { useContext, useMemo, useState, useRef } from "react";
import world from "./world.svg.json";

const TOOLTIP_WIDTH = 150; // Approximate tooltip size (adjust if needed)
const TOOLTIP_HEIGHT = 40;

const extendedWorldData = {
  ...world,
  layers: [
    ...world.layers,
    {
      id: "private",
      name: "Localhost",
      d: "M 10 20 L 25 10 L 40 20 L 40 40 L 10 40 Z",
    },
    {
      id: "total_clients",
      name: "Total Clients",
      d: "M 50 10 L 75 10 L 75 40 L 50 40 Z",
    },
  ],
};

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
    clientList.forEach(({ country_code }) => {
      const code = country_code.toUpperCase();
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
      fill: rgba(20,71,230, 1) !important;
    }
    `;

    styles += `
    .world-map svg path[id="total_clients"] {
      fill: rgba(255, 210, 48, 1) !important;
    }
    `;

    return styles;
  }, [clientCounts]);

  const layerProps = {
    onMouseEnter: (event: any) => {
      const countryName = event.target.attributes.name?.value;
      const countryId = event.target.attributes.id?.value?.toUpperCase();
      const count = clientCounts[countryId] || 0;

      const rect = mapRef.current?.getBoundingClientRect();
      if (rect) {
        let x = event.clientX - rect.left;
        let y = event.clientY - rect.top;

        // If tooltip would overflow right, move it to the left
        if (x + TOOLTIP_WIDTH > rect.width) {
          x = x - TOOLTIP_WIDTH - 10; // shift left with some padding
        } else {
          x = x + 10; // normal right side offset
        }

        // If tooltip would overflow bottom, move it above the cursor
        if (y + TOOLTIP_HEIGHT > rect.height) {
          y = y - TOOLTIP_HEIGHT - 10; // shift above
        } else {
          y = y + 10; // normal below offset
        }

        setTooltipPosition({ x, y });
      }

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
      const rect = mapRef.current?.getBoundingClientRect();
      if (rect) {
        let x = event.clientX - rect.left;
        let y = event.clientY - rect.top;

        // If tooltip would overflow right, move it to the left
        if (x + TOOLTIP_WIDTH > rect.width) {
          x = x - TOOLTIP_WIDTH - 10; // shift left with some padding
        } else {
          x = x + 10; // normal right side offset
        }

        // If tooltip would overflow bottom, move it above the cursor
        if (y + TOOLTIP_HEIGHT > rect.height) {
          y = y - TOOLTIP_HEIGHT - 10; // shift above
        } else {
          y = y + 10; // normal below offset
        }

        setTooltipPosition({ x, y });
      }
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
        {/* Inject generated dynamic styles */}

        <VectorMap
          {...extendedWorldData}
          layerProps={layerProps}
          style={{
            width: "100%",
            height: "100%",
            padding: "5px",
          }}
        />

        {tooltipContent && tooltipPosition && (
          <div
            className="absolute bg-black text-white text-xs rounded px-2 py-1 pointer-events-none"
            style={{
              top: tooltipPosition.y + 10,
              left: tooltipPosition.x + 10,
              zIndex: 1000,
            }}
          >
            {tooltipContent}
          </div>
        )}
      </div>
    </>
  );
};
