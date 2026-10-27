import "./shell/freeze";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "@fontsource-variable/geist/wght.css";
import "@fontsource-variable/geist-mono/wght.css";
import "./theme/theme.css";
import { App } from "./App";
import { setLang, storedLang } from "./i18n";

// The first render without waiting for the fonts: the terminal, the one
// view that measures them, waits for Geist Mono itself (terminal/view.tsx).
// The language last shown first, its catalog a local chunk (a few
// milliseconds): no text in Italian before app_info answers.
const root = document.getElementById("root");
if (root) {
  const last = storedLang();
  void (last ? setLang(last) : Promise.resolve())
    .catch(() => undefined)
    .then(() => {
      createRoot(root).render(
        <StrictMode>
          <App />
        </StrictMode>,
      );
    });
}
