import React, { useState, useEffect } from "react";
import { useNavigate, useLocation } from "react-router-dom";

export const Navigation: React.FC = () => {
  const [pageActive, setPageActive] = useState<string>("home");

  const navigate = useNavigate();
  const location = useLocation();

  useEffect(() => {
    const segments = location.pathname.split("/").filter(Boolean);
    const page = segments[0] || "home";
    setPageActive(page);
  }, [location]);

  function handleNavigation(page: string) {
    navigate(`/${page.toLowerCase()}`);
  }

  return (
    <div className="btm-nav bg-base-200">
      <button
        onClick={() => handleNavigation("home")}
        className={pageActive == "home" ? "active bg-base-300" : ""}
      >
        <i className="ri-home-line"></i>
        <span className="btm-nav-label">Home</span>
      </button>
      <button
        onClick={() => handleNavigation("clients")}
        className={pageActive == "clients" ? "active bg-base-300" : ""}
      >
        <i className="ri-user-line"></i>
        <span className="btm-nav-label">Clients</span>
      </button>
      <button
        onClick={() => handleNavigation("settings")}
        className={pageActive == "settings" ? "active bg-base-300" : ""}
      >
        <i className="ri-settings-3-line"></i>
        <span className="btm-nav-label">Settings</span>
      </button>
    </div>
  );
};
