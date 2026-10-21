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
    // What every window reads at start in chunks of its own, each under the
    // 500 kB warning and cached apart from the app's code: React, the
    // brands' logos (react-icons), the translation catalogs. The Guide's
    // chapters likewise, one chunk each, read together when it opens.
    rolldownOptions: {
      output: {
        codeSplitting: {
          groups: [
            { name: "react", test: /[\\/]node_modules[\\/](react|react-dom|scheduler)[\\/]/ },
            { name: "icons", test: /[\\/]node_modules[\\/]react-icons[\\/]/ },
            { name: "locales", test: /[\\/]src[\\/]locales[\\/]/ },
            {
              // Only the Guide's own modules: what they import from the app
              // stays where the app has it (never read at start through
              // them). The chapters' common parts get a chunk of their own,
              // so no two chunks import each other.
              name: (id: string) => {
                const topic = /[\\/]src[\\/]guide[\\/]topics[\\/](\w+)\.tsx$/.exec(id)?.[1];
                if (topic) return `guide-${topic}`;
                return /[\\/]src[\\/]guide[\\/](model|sources|widgets)\.tsx?$/.test(id) ? "guide-base" : null;
              },
              includeDependenciesRecursively: false,
            },
          ],
        },
      },
    },
  },
  test: {
    environment: "jsdom",
    // The tests live in the repository's test\ui, with the structure of src.
    dir: "../test/ui",
    include: ["**/*.test.tsx", "**/*.test.ts"],
    // What jsdom lacks for the vscode-elements components.
    setupFiles: ["../test/ui/setup.ts"],
    // The theme's CSS, VSCodium's reference and codicon.css, read as text by
    // theme.test.ts and codicon.test.tsx.
    css: { include: [/[\\/]theme[\\/]theme(-\w+)?\.css/, /workbench-ref\.css/, /[\\/]codicon\.css/] },
    // Resolved by Vite (with the conditions above), not by Node.
    server: { deps: { inline: [/[\\/]@vscode-elements[\\/]/, /[\\/]@lit[\\/]/, /[\\/]lit(-html|-element)?[\\/]/] } },
  },
}));
