import React from "react";
import ReactDOM from "react-dom/client";
import { CaptionOverlayWindow } from "./caption-overlay-window";
import "./overlay-styles.css";

ReactDOM.createRoot(document.getElementById("overlay-root")!).render(
  <React.StrictMode>
    <CaptionOverlayWindow />
  </React.StrictMode>,
);
