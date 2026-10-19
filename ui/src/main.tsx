import "./freeze";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "@fontsource-variable/geist/wght.css";
import "@fontsource-variable/geist-mono/wght.css";
import "./theme/theme.css";
import { App } from "./App";

/** Geist and Geist Mono (theme-vscode.css) are fetched on first use; the
 *  terminal measures its cells once, when it opens, so both are loaded
 *  before the first render. Local files: a few milliseconds, never more
 *  than the cap. */
function fontsReady(): Promise<unknown> {
  if (!("fonts" in document)) return Promise.resolve();
  const load = Promise.allSettled([
    document.fonts.load('13px "Geist Variable"'),
    document.fonts.load('600 13px "Geist Variable"'),
    document.fonts.load('13px "Geist Mono Variable"'),
  ]);
  return Promise.race([load, new Promise((done) => setTimeout(done, 500))]);
}

const root = document.getElementById("root");
if (root) {
  void fontsReady().then(() => {
    createRoot(root).render(
      <StrictMode>
        <App />
      </StrictMode>,
    );
  });
}
