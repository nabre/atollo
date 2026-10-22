// The progress of a setup run: a component touched twice in one run (removed
// on another system then installed, two lines patched) is two steps.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import { SAMPLE_SETUP, type SetupStep } from "@/api";
import { SetupProgress } from "@/setup";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(null),
  isTauri: () => true,
}));

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

const step = (name: string): SetupStep => ({ id: "mysql", name, note: null, done: null, size: null, outcome: null });

describe("SetupProgress", () => {
  it("lists two steps of the same component, each once", () => {
    const warn = vi.spyOn(console, "error").mockImplementation(() => undefined);
    render(
      <SetupProgress
        snapshot={{ ...SAMPLE_SETUP, running: true, total: 2, steps: [step("MySQL 8.4"), step("MySQL 9.7")] }}
      />,
    );
    expect(screen.getByText("MySQL 8.4")).toBeTruthy();
    expect(screen.getByText("MySQL 9.7")).toBeTruthy();
    expect(warn.mock.calls.some((c) => String(c[0]).includes("same key"))).toBe(false);
  });
});
