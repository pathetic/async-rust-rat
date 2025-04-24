import { useEffect, ReactNode } from "react";
import { WindowWrapperProps } from "../../types";
import { getCurrent } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { useParams } from "react-router-dom";

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
    let cleanupFn: (() => void) | undefined;

    let window = getCurrent();

    listen("close_window", () => {
      feature_cleanup(params);
      window.close();
    }).then((unlisten) => {
      cleanupFn = unlisten;
    });

    return () => {
      if (cleanupFn) cleanupFn();
    };
  }, []);

  useEffect(() => {
    const handleBeforeUnload = () => {
      feature_cleanup(params);
    };

    window.addEventListener("beforeunload", handleBeforeUnload);

    return () => {
      window.removeEventListener("beforeunload", handleBeforeUnload);
    };
  }, []);

  useEffect(() => {
    const cleanup = async () => {
      try {
        const window = getCurrent();

        await window.listen("tauri://close-requested", async () => {
          feature_cleanup(params);
          window.close();
        });
      } catch (error) {
        console.error("Error setting up window close handler:", error);
      }
    };

    cleanup();

    return () => {
      feature_cleanup(params);
    };
  }, []);

  return <>{children}</>;
};
