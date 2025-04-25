import { useEffect, ReactNode } from "react";
import { WindowWrapperProps } from "../../types";
import { useParams } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";

export interface WindowWrapperComponentProps
  extends Omit<WindowWrapperProps, "feature_cleanup"> {
  children: ReactNode;
  feature_cleanup: (params: Record<string, string | undefined>) => void;
}

export const WindowWrapper = ({
  feature_cleanup,
  children,
}: WindowWrapperComponentProps) => {
  const params = useParams();

  useEffect(() => {
    let unlistenCustomClose: (() => void) | undefined;

    const setup = async () => {
      unlistenCustomClose = await listen("close_window", async () => {
        try {
          feature_cleanup(params);
        } catch (e) {
          console.error("Feature cleanup error:", e);
        }
      });
    };

    setup();

    return () => {
      if (unlistenCustomClose) unlistenCustomClose();
      feature_cleanup(params);
    };
  }, []);

  return <>{children}</>;
};
