// A project's components and its Python environment (D-188).
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import { SAMPLE_PROJECTS, type ProjectInfo } from "@/ipc";
import { Needs } from "@/projects/needs";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

afterEach(cleanup);

const [SITE] = SAMPLE_PROJECTS as [ProjectInfo, ...ProjectInfo[]];

describe("project needs", () => {
  it("says where the Python environment is", () => {
    const { rerender } = render(<Needs p={{ ...SITE, localEnv: { state: "ready" } }} onChanged={() => undefined} />);
    expect(screen.getByText("Ambiente Python: .venv pronto.")).toBeTruthy();
    rerender(<Needs p={{ ...SITE, localEnv: { state: "failed", reason: "no uv" } }} onChanged={() => undefined} />);
    expect(screen.getByText("Ambiente Python: .venv non creato: no uv")).toBeTruthy();
  });

  it("says nothing for a project that is not Python's", () => {
    render(<Needs p={{ ...SITE, localEnv: null }} onChanged={() => undefined} />);
    expect(screen.queryByText(/Ambiente Python/)).toBeNull();
  });
});
