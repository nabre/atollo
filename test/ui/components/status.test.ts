// The state of a tool on a system in «Tutti gli strumenti»: a tool not
// needed there (ConPTY on macOS and Linux, D-177) has a state of its own,
// with the reason, never "missing".
import { describe, expect, it } from "vitest";
import type { ComponentInfo, TargetToolStatus } from "@/ipc";
import { otherStatus, shownStatus, toolStateLabel } from "@/components/status";

const REASON = "il terminale dell'app usa la pty del sistema";

const component = (extra: Partial<ComponentInfo> = {}): ComponentInfo => ({
  id: "conpty",
  name: "ConPTY",
  group: "app",
  desc: "",
  mb: 3,
  chosen: true,
  installed: false,
  present: false,
  locked: true,
  version: null,
  available: false,
  unavailableReason: null,
  notNeeded: REASON,
  needs: [],
  versioned: false,
  home: "https://example.org",
  agent: null,
  leftover: false,
  ...extra,
});

const status = (extra: Partial<TargetToolStatus> = {}): TargetToolStatus => ({
  id: "conpty",
  name: "ConPTY",
  available: false,
  chosen: false,
  installed: false,
  present: false,
  missing: null,
  version: null,
  needsReinstall: false,
  bootstrapStaged: false,
  firstStart: false,
  unavailableReason: null,
  notNeeded: REASON,
  viaVm: false,
  ...extra,
});

describe("tool status", () => {
  it("says not needed with the reason, on the system shown and on the others", () => {
    expect(shownStatus(component(), status(), true, null, "linux-x64")).toEqual({
      state: "notNeeded",
      detail: REASON,
    });
    expect(otherStatus(status(), "darwin-arm64")).toEqual({ state: "notNeeded", detail: REASON });
    expect(toolStateLabel("notNeeded")).toBe("non serve");
  });

  it("keeps not available apart, with its own reason", () => {
    const why = "Google non pubblica platform-tools e build-tools per Linux ARM64";
    const off = status({ id: "android", notNeeded: null, unavailableReason: why });
    expect(otherStatus(off, "linux-arm64")).toEqual({ state: "missing", detail: why });
    const c = component({ id: "android", locked: false, chosen: false, notNeeded: null, unavailableReason: why });
    expect(shownStatus(c, off, false, null, "linux-arm64")).toEqual({ state: "missing", detail: why });
  });

  it("says «fornito dalla VM», «serve la VM» and an install for the PC to remove (LEVELS.md §7)", () => {
    const agent = (extra: Partial<ComponentInfo>) =>
      component({ id: "pi", name: "Pi", group: "agents", locked: false, notNeeded: null, ...extra });
    const reason = "Gli agenti girano solo nella VM, che non è attiva.";
    expect(shownStatus(agent({ agent: { kind: "needsVm", reason } }), undefined, true, null, "win32-x64")).toEqual({
      state: "needsVm",
      detail: reason,
    });
    const claude = shownStatus(
      agent({ id: "claude", name: "Claude Code", agent: { kind: "provided" } }),
      undefined,
      true,
      null,
      "win32-x64",
    );
    expect(claude.state).toBe("provided");
    expect(claude.detail).toContain("Docker Sandboxes");
    expect(shownStatus(agent({ leftover: true }), undefined, true, null, "win32-x64").state).toBe("toRemove");
    expect(toolStateLabel("needsVm")).toBe("serve la VM");
    expect(toolStateLabel("provided")).toBe("fornito dalla VM");
  });
});
