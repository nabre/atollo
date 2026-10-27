import "./shell/freeze";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "@fontsource-variable/geist/wght.css";
import "@fontsource-variable/geist-mono/wght.css";
import "./theme/theme.css";
import { App } from "./App";

// The first render without waiting for the fonts: the terminal, the one
// view that measures them, waits for Geist Mono itself (terminal/view.tsx).
const root = document.getElementById("root");
if (root) {
  createRoot(root).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
}
