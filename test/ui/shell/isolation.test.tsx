// The isolation notice (D-167): every reason the VM is off has a sentence
// and what to do, with the command of the PC's system; the box lists them
// all and asks again on "Ricontrolla".
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { IsolationBlocker } from "@/ipc";
import { apparmorCommand, explain, IsolationDialog } from "@/shell/isolation";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

const ALL: IsolationBlocker[] = [
  { code: "old_windows", build: 19045 },
  { code: "old_macos", version: "13.6" },
  { code: "distribution", name: "Fedora 41" },
  { code: "hypervisor_off" },
  { code: "kvm_missing" },
  { code: "kvm_denied" },
  { code: "apparmor", shim: "/media/u/KIT/data/apps/linux-x64/sandboxes/libexec/containerd-shim-nerdbox-v1" },
  { code: "sbx_missing" },
  { code: "sbx_signed_out" },
  { code: "sbx_failed", message: "daemon not running" },
  { code: "foreign_daemon" },
  { code: "kit_disk", fs: "exFAT" },
];

describe("explain", () => {
  it("says why and what to do for every reason", () => {
    for (const b of ALL) {
      const e = explain(b);
      expect(e.why.length, b.code).toBeGreaterThan(10);
      expect(e.fix.length, b.code).toBeGreaterThan(10);
    }
  });

  it("installs Docker Sandboxes in the kit, and gives the commands of the PC", () => {
    const e = explain({ code: "sbx_missing" });
    expect(e.install).toBe(true);
    expect(e.command).toBeUndefined();
    expect(explain({ code: "hypervisor_off" }).command).toContain("HypervisorPlatform");
    const profile = explain({ code: "apparmor", shim: "/k/shim" }).command ?? "";
    expect(profile).toContain('"/k/shim" flags=(unconfined)');
    expect(profile).toContain("apparmor_parser -r /etc/apparmor.d/atollo-sbx-shim");
    expect(profile).toContain("  userns,");
    expect(explain({ code: "sbx_signed_out" }).command).toBe("sbx login");
  });

  it("never builds a sudo command from a path that could add rules", () => {
    for (const shim of ['/k/a" { capability, } "/x', "/k/a\nb", "/k/*", "/k/a,b", "relative/shim"]) {
      expect(apparmorCommand(shim), shim).toBeNull();
      const e = explain({ code: "apparmor", shim });
      expect(e.command).toBeUndefined();
      expect(e.fix.length).toBeGreaterThan(10);
    }
    expect(apparmorCommand("/media/u/My Kit/shim")).toContain('"/media/u/My Kit/shim"');
  });

  it("says why the last installation of Docker Sandboxes failed", () => {
    const e = explain({ code: "sbx_missing", failed: "checksum mismatch" });
    expect(e.why).toContain("checksum mismatch");
    expect(e.install).toBe(true);
  });

  it("puts the details in the sentence", () => {
    expect(explain({ code: "old_windows", build: 19045 }).why).toContain("19045");
    expect(explain({ code: "sbx_failed", message: "boom" }).why).toContain("boom");
  });
});

describe("admin steps (D-178)", () => {
  it("never launches a command that asks for an administrator: shown to be copied", () => {
    for (const b of ALL) {
      const e = explain(b);
      if (e.admin) {
        expect(e.run, b.code).toBeUndefined();
        expect(e.command, b.code).toBeDefined();
      }
      if (e.run) expect(["login", "ls"]).toContain(e.run);
    }
    expect(explain({ code: "hypervisor_off" }).admin).toBe(true);
    expect(explain({ code: "kvm_denied" }).admin).toBe(true);
    expect(explain({ code: "apparmor", shim: "/k/shim" }).admin).toBe(true);
    expect(explain({ code: "sbx_signed_out" }).admin).toBeUndefined();
    expect(explain({ code: "sbx_missing" }).install).toBe(true);
  });

  it("says in the box that Atollo never runs them and checks again by itself", () => {
    render(
      <IsolationDialog
        status={{ level: "env", blockers: [{ code: "hypervisor_off" }, { code: "kvm_denied" }], sbx: null }}
        checking={false}
        onCheck={() => {}}
        onClose={() => {}}
      />,
    );
    expect(screen.queryByRole("button", { name: "Lancia" })).toBeNull();
    expect(screen.getAllByRole("button", { name: "Copia" })).toHaveLength(2);
    expect(screen.getAllByText(/Serve un amministratore: Atollo non lo esegue mai/)).toHaveLength(2);
  });

  it("says the rule without the VM: projects on the PC without agents, the editor's terminals and the agents only in the VM", () => {
    render(
      <IsolationDialog
        status={{ level: "env", blockers: [{ code: "sbx_signed_out" }], sbx: null }}
        checking={false}
        onCheck={() => {}}
        onClose={() => {}}
      />,
    );
    expect(screen.getByText(/girano sul PC con il tuo utente, senza gli agenti/)).toBeTruthy();
    expect(screen.getByText(/partono solo nella VM: senza, non si aprono/)).toBeTruthy();
    expect(screen.queryByText(/Serve un amministratore/)).toBeNull();
  });
});

describe("IsolationDialog", () => {
  it("lists every reason with its command, and checks again", () => {
    const onCheck = vi.fn();
    render(
      <IsolationDialog
        status={{ level: "env", blockers: [{ code: "hypervisor_off" }, { code: "sbx_missing" }], sbx: null }}
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

  it("launches sbx's commands in Atollo's terminal and closes, copies the PC's", async () => {
    invoke.mockResolvedValue(undefined);
    const onClose = vi.fn();
    render(
      <IsolationDialog
        status={{ level: "env", blockers: [{ code: "hypervisor_off" }, { code: "sbx_signed_out" }], sbx: "sbx" }}
        checking={false}
        onCheck={() => {}}
        onClose={onClose}
      />,
    );
    expect(screen.getAllByRole("button", { name: "Lancia" })).toHaveLength(1);
    expect(screen.getAllByRole("button", { name: "Copia" })).toHaveLength(1);
    fireEvent.click(screen.getByRole("button", { name: "Lancia" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("isolation_run", { fix: "login" });
    });
    await waitFor(() => {
      expect(onClose).toHaveBeenCalledOnce();
    });
  });

  it("lists Atollo's sandboxes with the room they take, and removes one after a second click", async () => {
    let vms = [
      { name: "atollo-p-shop-1", owner: { kind: "project", project: "shop" }, status: "running", lastUsed: null },
      { name: "atollo-a-claude-shop-2", owner: null, status: "stopped", lastUsed: null },
    ];
    invoke.mockImplementation((cmd: string) => {
      if (cmd === "vms_list") return Promise.resolve({ vms, disk: 3 * 1024 ** 3 });
      if (cmd === "vm_remove") {
        vms = vms.slice(1);
        return Promise.resolve(undefined);
      }
      return Promise.resolve(undefined);
    });
    render(
      <IsolationDialog
        status={{ level: "vm", blockers: [], sbx: "sbx" }}
        checking={false}
        onCheck={() => {}}
        onClose={() => {}}
      />,
    );
    expect(await screen.findByText("Progetto shop")).toBeTruthy();
    expect(screen.getByText("atollo-a-claude-shop-2")).toBeTruthy();
    expect(screen.getByText("accesa")).toBeTruthy();
    expect(screen.getByText("Spazio occupato nel kit: 3 GB")).toBeTruthy();
    const [first] = screen.getAllByRole("button", { name: "Rimuovi" });
    if (!first) throw new Error("no «Rimuovi»");
    fireEvent.click(first);
    expect(invoke).not.toHaveBeenCalledWith("vm_remove", expect.anything());
    fireEvent.click(screen.getByRole("button", { name: "Rimuovi davvero" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("vm_remove", { name: "atollo-p-shop-1" });
    });
    await waitFor(() => {
      expect(screen.queryByText("Progetto shop")).toBeNull();
    });
  });

  it("says the projects are in the VM when they are", () => {
    invoke.mockResolvedValue({ vms: [], disk: null });
    render(
      <IsolationDialog
        status={{ level: "vm", blockers: [], sbx: "sbx" }}
        checking={false}
        onCheck={() => {}}
        onClose={() => {}}
      />,
    );
    expect(screen.getByText("Progetti nella VM")).toBeTruthy();
    expect(screen.getByText(/I terminali del kit restano sul PC, senza gli agenti/)).toBeTruthy();
    expect(screen.queryByText("Perché la VM non è attiva")).toBeNull();
  });
});
