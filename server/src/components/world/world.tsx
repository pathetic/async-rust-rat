import world from "./world.svg.json";
import React from "react";

const TOOLTIP_WIDTH = 150;
const TOOLTIP_HEIGHT = 40;

export const extendedWorldData = {
  ...world,
  layers: [
    ...world.layers,
    {
      id: "private",
      name: "Localhost",
      d: `
        M77.912,28.014V6.881h-7.909v14.824L46.854,3.239L14.538,29.382l-0.002-0.002l-1.011,0.82l-3.275,2.649v0.011L0,41.188
        l4.429,5.45l5.821-4.729v49.079h73.725v-49.08l5.823,4.73l4.429-5.45L77.912,28.014z M73.638,82.837H58.179V57.028H36.045v25.809
        H20.586V41h53.053L73.638,82.837L73.638,82.837z
      `,
      transform: "translate(-25, 20) scale(0.34)",
    },
    {
      id: "total_clients",
      name: "Total Clients",
      d: `
        M47.561,0C25.928,0,8.39,6.393,8.39,14.283v11.72c0,7.891,17.538,14.282,39.171,14.282
        c21.632,0,39.17-6.392,39.17-14.282v-11.72C86.731,6.393,69.193,0,47.561,0z
    
        M47.561,47.115
        c-20.654,0-37.682-5.832-39.171-13.227c-0.071,0.353,0,19.355,0,19.355
        c0,7.892,17.538,14.283,39.171,14.283
        c21.632,0,39.17-6.393,39.17-14.283
        c0,0,0.044-19.003-0.026-19.355
        C85.214,41.284,68.214,47.115,47.561,47.115z
    
        M86.694,61.464
        c-1.488,7.391-18.479,13.226-39.133,13.226S9.875,68.854,8.386,61.464L8.39,80.82
        c0,7.891,17.538,14.282,39.171,14.282
        c21.632,0,39.17-6.393,39.17-14.282L86.694,61.464z
      `,
      transform: "translate(20, 20) scale(0.34)",
    },
  ],
};

export const computePosition = (
  event: any,
  mapRef: React.RefObject<HTMLDivElement | null>
) => {
  if (!mapRef.current) return { x: 0, y: 0 };
  const rect = mapRef.current.getBoundingClientRect();
  if (rect) {
    // Get the exact mouse position relative to the map container
    const mouseX = event.clientX - rect.left;
    const mouseY = event.clientY - rect.top;

    // Calculate the initial tooltip position (centered on the mouse)
    let x = mouseX - TOOLTIP_WIDTH / 2;
    let y = mouseY - TOOLTIP_HEIGHT / 2;

    // Handle horizontal overflow
    if (x < 0) {
      // If tooltip would overflow left, align with left edge
      x = 0;
    } else if (x + TOOLTIP_WIDTH > rect.width) {
      // If tooltip would overflow right, align with right edge
      x = rect.width - TOOLTIP_WIDTH;
    }

    // Handle vertical overflow
    if (y < 0) {
      // If tooltip would overflow top, position below the mouse
      y = mouseY + 10;
    } else if (y + TOOLTIP_HEIGHT > rect.height) {
      // If tooltip would overflow bottom, position above the mouse
      y = mouseY - TOOLTIP_HEIGHT - 10;
    }

    return { x, y };
  }
  return { x: 0, y: 0 };
};

export const ToolTip: React.FC<{
  tooltipContent: string | null;
  tooltipPosition: { x: number; y: number } | null;
}> = ({ tooltipContent, tooltipPosition }) => {
  if (!tooltipContent || !tooltipPosition) return null;
  return (
    <div
      className="absolute bg-black text-white text-xs rounded px-2 py-1 pointer-events-none"
      style={{
        top: tooltipPosition.y,
        left: tooltipPosition.x,
        zIndex: 1000,
      }}
    >
      {tooltipContent}
    </div>
  );
};
