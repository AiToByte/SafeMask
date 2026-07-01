/**
 * SafeMask Desktop — React entry point
 * Initializes React 19 + Zustand state management + global styles
 */

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";

// Global styles (Tailwind directives + custom classes)
import "./style.css";

const rootEl = document.getElementById("root");
if (!rootEl) throw new Error("Root element #root not found");

createRoot(rootEl).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
