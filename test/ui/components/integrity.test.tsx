// «Verifica i file» (N-5): the damaged folders with «Reinstalla» at their
// version, the ones without a manifest named, all well said once.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { SAMPLE_INTEGRITY, type ComponentIntegrity } from "../../../ui/src/api";
import { IntegrityReport, useIntegrity } from "../../../ui/src/components/integrity";

const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: unknown) => invoke(cmd, args),
  isTauri: () => true,
}));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

function Page({ target }: { target?: string }) {
  const state = useIntegrity(target);
  return (
    <>
      <button type="button" onClick={state.check}>
        check
      </button>
      <IntegrityReport state={state} running={false} />
    </>
  );
}

describe("IntegrityReport", () => {
  it("lists the damaged folders and reinstalls one at its version", async () => {
    invoke.mockImplementation((c) => Promise.resolve(c === "components_integrity" ? SAMPLE_INTEGRITY : undefined));
    render(<Page target="linux-x64" />);
    fireEvent.click(screen.getByRole("button", { name: "check" }));
    const report = await screen.findByRole("region", { name: "Verifica i file" });
    expect(invoke).toHaveBeenCalledWith("components_integrity", { target: "linux-x64" });
    const php = within(report).getByText("PHP 8.4").closest("li") as HTMLElement;
    expect(within(php).getByText(/1 file cambiati, 1 mancanti/)).toBeTruthy();
    expect(within(php).getByText(/php8ts\.dll/)).toBeTruthy();
    // Node is fine; Git has no manifest yet.
    expect(within(report).queryByText("Node.js 24.11")).toBeNull();
    expect(within(report).getByText(/Senza manifesto.*Git 2\.51/)).toBeTruthy();
    fireEvent.click(within(php).getByRole("button", { name: "Reinstalla" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("components_repair", {
        items: [{ id: "php", line: "8.4" }],
        target: "linux-x64",
      });
    });
    // Started: the report goes, the progress is the preparation's.
    await waitFor(() => {
      expect(screen.queryByRole("region", { name: "Verifica i file" })).toBeNull();
    });
  });

  it("says when nothing is damaged", async () => {
    const fine: ComponentIntegrity[] = SAMPLE_INTEGRITY.filter((i) => i.id === "node");
    invoke.mockResolvedValue(fine);
    render(<Page />);
    fireEvent.click(screen.getByRole("button", { name: "check" }));
    expect(await screen.findByText("Nessun file danneggiato: 1 cartelle verificate.")).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith("components_integrity", undefined);
    expect(screen.queryByRole("button", { name: "Reinstalla" })).toBeNull();
  });
});
