// «Matrice» of the Tools page (D-121): one row per component, the four
// systems of Docker Sandboxes (D-176) as columns with their readable names and marks, the states of
// the cells with the reason, the details on a click; «Piattaforma»
// remembered between sessions.
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, renderHook, screen, waitFor, within } from "@testing-library/react";
import { TARGETS, type ComponentInfo, type TargetGrid, type TargetProfile, type TargetToolStatus } from "@/ipc";
import { TargetMatrix } from "@/components/matrix";
import { useComponents, type ComponentsState } from "@/components/state";
import { cellState, targetLabel, targetName } from "@/components/targets";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

const component = (id: string, name: string, group: ComponentInfo["group"]): ComponentInfo => ({
  id,
  name,
  group,
  desc: "",
  mb: 10,
  chosen: false,
  installed: false,
  present: false,
  locked: false,
  version: null,
  available: true,
  unavailableReason: null,
  notNeeded: null,
  needs: [],
  versioned: false,
  home: "https://example.org",
  agent: null,
  leftover: false,
});

const LIST = [
  component("git", "Git", "required"),
  component("r", "R", "languages"),
  component("android", "Android SDK", "languages"),
  component("conpty", "ConPTY", "app"),
];

const status = (id: string, extra: Partial<TargetToolStatus> = {}): TargetToolStatus => ({
  id,
  name: id,
  available: true,
  chosen: false,
  installed: false,
  present: false,
  missing: null,
  version: null,
  needsReinstall: false,
  bootstrapStaged: false,
  firstStart: false,
  unavailableReason: null,
  notNeeded: null,
  viaVm: false,
  ...extra,
});

/** Not needed on that system: the system does its job (ConPTY on macOS
 * and Linux, D-177). */
const notNeeded = (id: string): TargetToolStatus =>
  status(id, { available: false, notNeeded: "il terminale dell'app usa la pty del sistema" });

/** This PC a Mac with Apple Silicon, the others not run here. The cells say
 *  what the catalog says: Android SDK missing on Linux ARM64. */
function grid(chosen: TargetGrid["chosen"] = null, vm: TargetGrid["vm"] = null): TargetGrid {
  const profiles: TargetProfile[] = TARGETS.map((id) => {
    const [os, arch] = id.split("-") as [TargetProfile["os"], TargetProfile["arch"]];
    return {
      id,
      os,
      arch,
      isHost: id === "darwin-arm64",
      executableOnHost: id === "darwin-arm64",
    };
  });
  const cells: Record<string, TargetToolStatus[]> = {
    "win32-x64": [
      status("git", { installed: true, version: "2.56.0", chosen: true }),
      status("r"),
      status("android"),
      status("conpty", { installed: true }),
    ],
    "darwin-arm64": [status("git", { installed: true }), status("r"), status("android"), notNeeded("conpty")],
    "linux-x64": [status("git", { bootstrapStaged: true }), status("r"), status("android"), notNeeded("conpty")],
    "linux-arm64": [
      notNeeded("conpty"),
      status("git", { firstStart: true }),
      status("r"),
      status("android", {
        available: false,
        unavailableReason: "Google non pubblica platform-tools e build-tools per Linux ARM64",
      }),
    ],
  };
  return {
    profiles,
    chosen,
    vm,
    targets: profiles.map(({ id, ...p }) => ({
      ...p,
      target: id,
      availableCount: 2,
      installedCount: 0,
      tools: cells[id] ?? [],
    })),
  };
}

function fakeState(extra: Partial<ComponentsState> = {}): ComponentsState {
  return {
    grid: grid(),
    gridError: null,
    list: LIST,
    target: undefined,
    host: "darwin-arm64",
    setTarget: vi.fn(),
    ...extra,
  } as unknown as ComponentsState;
}

describe("target names and cells", () => {
  it("names the systems as people do, with what each is for this PC", () => {
    expect(targetName("darwin-arm64")).toBe("macOS Apple Silicon");
    expect(targetName("beos-x64")).toBe("beos-x64");
    const { profiles } = grid();
    expect(targetLabel("darwin-arm64", profiles)).toBe("macOS Apple Silicon (questo PC)");
    expect(targetLabel("linux-x64", profiles)).toBe("Linux x64 (non eseguibile qui)");
    expect(targetLabel("linux-x64", [])).toBe("Linux x64");
  });

  it("gives each cell one state, unavailable first", () => {
    expect(cellState(status("x"))).toBe("available");
    expect(cellState(status("x", { bootstrapStaged: true, firstStart: true }))).toBe("staged");
    expect(cellState(status("x", { available: false, bootstrapStaged: true }))).toBe("unavailable");
    // Not needed (D-177) before everything, the VM too: nothing to run.
    expect(cellState(notNeeded("x"))).toBe("notNeeded");
    expect(cellState({ ...notNeeded("x"), viaVm: true })).toBe("notNeeded");
  });
});

describe("TargetMatrix", () => {
  it("shows a row per component and the four systems in order, with their marks", () => {
    render(<TargetMatrix state={fakeState()} onOpen={vi.fn()} />);
    const table = screen.getByRole("table", { name: "Matrice componente-sistema" });
    const heads = within(table)
      .getAllByRole("columnheader")
      .map((h) => h.textContent);
    expect(heads.slice(1, 5).map((h) => h.split(/(?=win32|darwin|linux)/)[0])).toEqual([
      "Windows x64",
      "macOS Apple Silicon",
      "Linux x64",
      "Linux ARM64",
    ]);
    expect(heads[2]).toContain("questo PC");
    expect(heads[1]).toContain("non eseguibile qui");
    expect(heads[3]).toContain("non eseguibile qui");
    expect(within(table).getByRole("rowheader", { name: "Git" })).toBeTruthy();
    expect(within(table).getByRole("rowheader", { name: "R" })).toBeTruthy();
  });

  it("says each state, with the reason where it is not available", () => {
    render(<TargetMatrix state={fakeState()} onOpen={vi.fn()} />);
    expect(screen.getByRole("button", { name: "Git, Windows x64: disponibile" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Git, Linux x64: pre-installato" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Git, Linux ARM64: si completa al primo avvio" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Android SDK, macOS Apple Silicon: disponibile" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "R, macOS Apple Silicon: disponibile" })).toBeTruthy();
    expect(
      screen.getByRole("button", {
        name: "Android SDK, Linux ARM64: non disponibile (Google non pubblica platform-tools e build-tools per Linux ARM64)",
      }),
    ).toBeTruthy();
  });

  it("says a tool is not needed off Windows, with the reason, in the legend too (D-177)", () => {
    render(<TargetMatrix state={fakeState()} onOpen={vi.fn()} />);
    expect(screen.getByRole("button", { name: "ConPTY, Windows x64: disponibile" })).toBeTruthy();
    for (const system of ["macOS Apple Silicon", "Linux x64", "Linux ARM64"]) {
      expect(
        screen.getByRole("button", {
          name: `ConPTY, ${system}: non serve (il terminale dell'app usa la pty del sistema)`,
        }),
      ).toBeTruthy();
    }
    const legend = screen.getByRole("list", { name: "Legenda" });
    expect(within(legend).getByText("non serve")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: /^ConPTY, Linux x64/ }));
    const detail = screen.getByRole("region", { name: "ConPTY su Linux x64" });
    expect(detail.textContent).toContain("il terminale dell'app usa la pty del sistema");
    expect(detail.textContent).toContain("Su questo sistema non serve");
  });

  it("opens the details of a cell, and shows its system or the component from there", () => {
    const setTarget = vi.fn();
    const onOpen = vi.fn();
    render(<TargetMatrix state={fakeState({ setTarget })} onOpen={onOpen} />);
    fireEvent.click(screen.getByRole("button", { name: "Git, Windows x64: disponibile" }));
    const detail = screen.getByRole("region", { name: "Git su Windows x64" });
    expect(detail.textContent).toContain("2.56.0");
    expect(detail.textContent).toContain("non eseguibile qui");
    fireEvent.click(within(detail).getByRole("button", { name: "Mostra questo sistema" }));
    expect(setTarget).toHaveBeenCalledWith("win32-x64");
    fireEvent.click(within(detail).getByRole("button", { name: "Apri Git" }));
    expect(onOpen).toHaveBeenCalledWith("git");
    // The system shown already has nothing to show.
    fireEvent.click(screen.getByRole("button", { name: /^Git, macOS Apple Silicon/ }));
    const own = screen.getByRole("region", { name: "Git su macOS Apple Silicon" });
    expect(within(own).queryByRole("button", { name: "Mostra questo sistema" })).toBeNull();
  });

  it("marks the VM's column, and a tool of the PC's system the VM runs (D-174)", () => {
    const g = grid(null, "linux-arm64");
    const pc = g.targets.find((s) => s.target === "darwin-arm64");
    if (!pc) throw new Error("no column");
    pc.tools = pc.tools.map((x) => (x.id === "git" ? { ...x, viaVm: true } : x));
    render(<TargetMatrix state={fakeState({ grid: g })} onOpen={vi.fn()} />);
    const heads = within(screen.getByRole("table", { name: "Matrice componente-sistema" })).getAllByRole(
      "columnheader",
    );
    expect(heads.find((h) => h.textContent.includes("linux-arm64"))?.textContent).toContain("VM");
    expect(heads.find((h) => h.textContent.includes("linux-x64"))?.textContent).not.toContain("VM");
    expect(screen.getByRole("button", { name: "Git, macOS Apple Silicon: nella VM" })).toBeTruthy();
    expect(cellState(status("x", { viaVm: true, available: false }))).toBe("vm");
  });

  it("says when the grid cannot be read", () => {
    render(<TargetMatrix state={fakeState({ grid: null, gridError: "disco non leggibile" })} onOpen={vi.fn()} />);
    expect(screen.getByRole("alert").textContent).toContain("disco non leggibile");
  });
});

describe("«Piattaforma» between sessions", () => {
  function answer(chosen: TargetGrid["chosen"]) {
    invoke.mockImplementation((cmd) => {
      if (cmd === "target_profiles") return Promise.resolve(grid(chosen));
      if (cmd === "components_list") return Promise.resolve(LIST);
      return Promise.resolve([]);
    });
  }

  it("opens on the system chosen last, and saves this PC's as none", async () => {
    answer("linux-arm64");
    const { result } = renderHook(() => useComponents(null, false));
    await waitFor(() => {
      expect(result.current.list).not.toBeNull();
    });
    expect(result.current.target).toBe("linux-arm64");
    expect(invoke).toHaveBeenCalledWith("components_list", { target: "linux-arm64" });
    // The list is read once, for the system chosen last.
    expect(invoke.mock.calls.filter(([c]) => c === "components_list")).toHaveLength(1);

    act(() => {
      result.current.setTarget("win32-x64");
    });
    expect(invoke).toHaveBeenCalledWith("tools_target_set", { target: "win32-x64" });
    expect(result.current.target).toBe("win32-x64");
    act(() => {
      result.current.setTarget("darwin-arm64");
    });
    expect(invoke).toHaveBeenCalledWith("tools_target_set", { target: null });
    expect(result.current.target).toBeUndefined();
  });

  it("leaves the first-start page on this PC's system", async () => {
    answer("linux-arm64");
    const { result } = renderHook(() => useComponents(null, true));
    await waitFor(() => {
      expect(result.current.list).not.toBeNull();
    });
    expect(result.current.target).toBeUndefined();
    expect(invoke).not.toHaveBeenCalledWith("target_profiles", undefined);
  });
});
