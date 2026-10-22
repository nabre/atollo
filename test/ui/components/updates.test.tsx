// The components with a new version (N-9): each with its notes and
// «Aggiorna», «Controlla ora», read again when a preparation ends.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { SAMPLE_UPDATES, type Updates } from "@/api";
import { UpdatesPane } from "@/components/updates";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));
vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => undefined),
}));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

function pane(running = false) {
  return <UpdatesPane expanded onToggle={() => undefined} running={running} />;
}

describe("UpdatesPane", () => {
  it("lists the new versions with their notes and updates one", async () => {
    invoke.mockImplementation((c) => Promise.resolve(c === "updates_list" ? SAMPLE_UPDATES : undefined));
    render(pane());
    const gh = (await screen.findByText("GitHub CLI")).closest("li") as HTMLElement;
    expect(within(gh).getByText("2.100.0 → 2.103.0")).toBeTruthy();
    fireEvent.click(within(gh).getByText("Cosa cambia"));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("component_link_open", { id: "gh", kind: "notes" });
    });
    fireEvent.click(within(gh).getByRole("button", { name: "Aggiorna GitHub CLI" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("setup_start", { only: ["gh"], updateAll: true });
    });
    expect(screen.getByText(/^Controllato il /)).toBeTruthy();
  });

  it("checks now, and says when all is up to date or never checked", async () => {
    const never: Updates = { checked: null, items: [] };
    const fine: Updates = { checked: 1_800_000_000, items: [] };
    invoke.mockImplementation((c) => Promise.resolve(c === "updates_list" ? never : fine));
    render(pane());
    expect(await screen.findByText(/^Non ancora controllato/)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Controlla ora" }));
    expect(await screen.findByText("Tutto aggiornato.")).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith("updates_check", undefined);
  });

  it("reads the list again when a preparation ends", async () => {
    invoke.mockResolvedValue(SAMPLE_UPDATES);
    const { rerender } = render(pane(true));
    await screen.findByText("GitHub CLI");
    const reads = () => invoke.mock.calls.filter(([c]) => c === "updates_list").length;
    const before = reads();
    rerender(pane(false));
    await waitFor(() => {
      expect(reads()).toBe(before + 1);
    });
  });
});
