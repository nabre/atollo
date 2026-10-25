import tseslint from "typescript-eslint";
import reactHooks from "eslint-plugin-react-hooks";

// The interface's sources (ui\src), the Guide's content (guide, D-164) and
// their tests (test\ui); each folder has its tsconfig.json.
export default tseslint.config(
  // guide/render/load.cjs is Node's CommonJS for the site, as scripts\.
  { ignores: ["dist", "node_modules", "tauri", "docs", "guide/render"] },
  ...tseslint.configs.strictTypeChecked,
  {
    files: ["ui/src/**/*.{ts,tsx}", "guide/**/*.{ts,tsx}", "test/ui/**/*.{ts,tsx}"],
    languageOptions: { parserOptions: { projectService: true, tsconfigRootDir: import.meta.dirname } },
    plugins: { "react-hooks": reactHooks },
    rules: { ...reactHooks.configs.recommended.rules },
  },
  // The IPC stays in ui/src/ipc (docs/app/ARCHITECTURE.md): no other file of
  // the interface reaches Tauri by itself.
  {
    files: ["ui/src/**/*.{ts,tsx}"],
    ignores: ["ui/src/ipc/**"],
    rules: {
      "no-restricted-imports": [
        "error",
        { patterns: [{ group: ["@tauri-apps/*"], message: "Tauri only through ui/src/ipc." }] },
      ],
    },
  },
  { files: ["eslint.config.mjs", "vitest.config.ts", "ui/vite.config.ts"], ...tseslint.configs.disableTypeChecked },
);
