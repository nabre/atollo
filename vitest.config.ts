import { defineConfig } from "vitest/config";

// `npx vitest` from the repository's root runs the interface's tests only,
// with ui\vite.config.ts. Without this it collected every test file below
// the root: the copies under target\ and .claude\worktrees\ too. The tests'
// folder is named again from here: Vitest reads `dir` from the working
// directory, not from ui\.
export default defineConfig({
  test: {
    projects: [{ extends: "./ui/vite.config.ts", root: "ui", test: { dir: "test/ui" } }],
  },
});
