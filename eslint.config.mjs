import tseslint from "typescript-eslint";
import reactHooks from "eslint-plugin-react-hooks";

// The interface's sources (ui\src) and their tests (test\ui); each folder has
// its tsconfig.json.
export default tseslint.config(
  { ignores: ["dist", "node_modules", "tauri", "docs"] },
  ...tseslint.configs.strictTypeChecked,
  {
    files: ["ui/src/**/*.{ts,tsx}", "test/ui/**/*.{ts,tsx}"],
    languageOptions: { parserOptions: { projectService: true, tsconfigRootDir: import.meta.dirname } },
    plugins: { "react-hooks": reactHooks },
    rules: { ...reactHooks.configs.recommended.rules },
  },
  { files: ["eslint.config.mjs", "ui/vite.config.ts"], ...tseslint.configs.disableTypeChecked },
);
