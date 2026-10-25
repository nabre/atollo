// The isolation notice (D-167): every reason the VM is off has a sentence
// and what to do, with the command of the PC's system; the box lists them
// all and asks again on "Ricontrolla".
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import type { IsolationBlocker } from "@/ipc";
import { explain, IsolationDialog, systemOf } from "@/shell/isolation";

afterEach(cleanup);

const ALL: IsolationBlocker[] = [
  { code: "old_windows", build: 19045 },
  { code: "processor" },
  { code: "old_macos", version: "13.6" },
  { code: "distribution", name: "Fedora 41" },
  { code: "hypervisor_off" },
  { code: "kvm_missing" },
  { code: "kvm_denied" },
  { code: "apparmor", shim: "/media/u/KIT/data/apps/linux-x64/sandboxes/libexec/containerd-shim-nerdbox-v1" },
  { code: "sbx_missing" },
  { code: "sbx_signed_out" },
  { code: "sbx_failed", message: "daemon not running" },
  { code: "pending" },
];

describe("explain", () => {
  it("says why and what to do for every reason, on every system", () => {
    for (const system of ["windows", "macos", "linux"] as const) {
      for (const b of ALL) {
        const e = explain(b, system);
        expect(e.why.length, b.code).toBeGreaterThan(10);
        expect(e.fix.length, b.code).toBeGreaterThan(10);
      }
    }
  });

  it("installs Docker Sandboxes in the kit, and gives the commands of the PC", () => {
    for (const system of ["windows", "macos", "linux"] as const) {
      const e = explain({ code: "sbx_missing" }, system);
      expect(e.install).toBe(true);
      expect(e.command).toBeUndefined();
    }
    expect(explain({ code: "hypervisor_off" }, "windows").command).toContain("HypervisorPlatform");
    const profile = explain({ code: "apparmor", shim: "/k/shim" }, "linux").command ?? "";
    expect(profile).toContain('"/k/shim" flags=(unconfined)');
    expect(profile).toContain("apparmor_parser -r /etc/apparmor.d/atollo-sbx-shim");
    expect(explain({ code: "sbx_signed_out" }, "linux").command).toBe("sbx login");
  });

  it("puts the details in the sentence", () => {
    expect(explain({ code: "old_windows", build: 19045 }, "windows").why).toContain("19045");
    expect(explain({ code: "sbx_failed", message: "boom" }, "windows").why).toContain("boom");
  });
});

describe("systemOf", () => {
  it("reads the kit's platform", () => {
    expect(systemOf("win32-x64")).toBe("windows");
    expect(systemOf("darwin-arm64")).toBe("macos");
    expect(systemOf("linux-x64")).toBe("linux");
  });
});

describe("IsolationDialog", () => {
  it("lists every reason with its command, and checks again", () => {
    const onCheck = vi.fn();
    render(
      <IsolationDialog
        status={{ level: "env", blockers: [{ code: "hypervisor_off" }, { code: "sbx_missing" }], sbx: null }}
        system="windows"
        checking={false}
        onCheck={onCheck}
        onClose={() => {}}
      />,
    );
    expect(screen.getByText("Lavori senza VM")).toBeTruthy();
    expect(screen.getByText("Perché la VM non è attiva")).toBeTruthy();
    expect(screen.getAllByRole("listitem")).toHaveLength(2);
    expect(screen.getByRole("button", { name: "Installa nel kit" })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Ricontrolla" }));
    expect(onCheck).toHaveBeenCalledOnce();
  });

  it("says the projects are in the VM when they are", () => {
    render(
      <IsolationDialog
        status={{ level: "vm", blockers: [], sbx: "sbx" }}
        system="macos"
        checking={false}
        onCheck={() => {}}
        onClose={() => {}}
      />,
    );
    expect(screen.getByText("Progetti nella VM")).toBeTruthy();
    expect(screen.queryByText("Perché la VM non è attiva")).toBeNull();
  });
});
