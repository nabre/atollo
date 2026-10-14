import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// The app window loads the built files from the exe; no CDN, no remote
// resources (CSP in src-tauri/tauri.conf.json).
export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  // Dependencies live in the root node_modules (npm workspace); keep the cache there too.
  cacheDir: "../node_modules/.vite",
  server: { host: "127.0.0.1", port: 5173, strictPort: true },
  build: { target: "es2022", outDir: "../dist", emptyOutDir: true, sourcemap: false },
  test: { environment: "jsdom", include: ["src/**/*.test.tsx", "src/**/*.test.ts"] },
});
