import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// The app window loads the built files from the exe; no CDN, no remote
// resources (CSP in tauri/tauri.conf.json).
export default defineConfig(({ mode }) => ({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  // Dependencies live in the root node_modules (npm workspace); keep the cache there too.
  cacheDir: "../node_modules/.vite",
  server: { host: "127.0.0.1", port: 5173, strictPort: true },
  // Vitest resolves packages for Node: Lit and @lit/react would load their
  // server-rendering builds, which never set a property on an element.
  // Under jsdom they take their browser builds, as in the app.
  ...(mode === "test" && { resolve: { conditions: ["browser"] } }),
  build: {
    target: "es2022",
    outDir: "../dist",
    emptyOutDir: true,
    sourcemap: false,
  },
  test: {
    environment: "jsdom",
    // The tests live in the repository's test\ui, with the structure of src.
    dir: "../test/ui",
    include: ["**/*.test.tsx", "**/*.test.ts"],
    // What jsdom lacks for the vscode-elements components.
    setupFiles: ["../test/ui/setup.ts"],
    // The theme's generated CSS, read as text by theme.test.ts.
    css: { include: [/theme-(vscode|colors)\.css/] },
    // Resolved by Vite (with the conditions above), not by Node.
    server: { deps: { inline: [/[\\/]@vscode-elements[\\/]/, /[\\/]@lit[\\/]/, /[\\/]lit(-html|-element)?[\\/]/] } },
  },
}));
