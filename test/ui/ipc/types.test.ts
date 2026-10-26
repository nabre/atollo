// The types of ./api say what Rust sends: the editor's status carries the
// extensions being installed (editor::Status), and the settings never carry
// the Gemini key (skipped when Rust serializes them).
import { describe, expect, it } from "vitest";
import { editorStatus, settingsGet, type EditorStatus, type Settings } from "@/ipc";

describe("api types", () => {
  it("reads the extensions of the editor's status", async () => {
    // As Rust serializes editor::Status.
    const fromRust: EditorStatus = {
      installed: true,
      running: true,
      port: 7000,
      extensions: { running: true, total: 3, done: 1, failed: ["ms-python.python"] },
    };
    expect(fromRust.extensions.failed).toEqual(["ms-python.python"]);
    expect((await editorStatus()).extensions).toEqual({ running: false, total: 0, done: 0, failed: [] });
  });

  it("takes settings without the Gemini key", async () => {
    const { settings } = await settingsGet();
    expect("geminiApiKey" in settings).toBe(false);
    const sent: Settings = { ...settings };
    expect(sent.geminiApiKey).toBeUndefined();
  });
});
