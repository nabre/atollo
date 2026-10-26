// The agents' rows of «Tutti gli strumenti» (LEVELS.md §7, Q11): Claude
// Code and Codex «fornito dalla VM», Pi and Antigravity «serve la VM» with
// the reason without it, an install for the PC «da rimuovere»; none of them
// can be ticked or installed there.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import type { ComponentInfo } from "@/ipc";
import { ComponentRow } from "@/components/rows";
import type { KitVersions } from "@/versions";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

afterEach(cleanup);

const REASON = "Gli agenti girano solo nella VM, che non è attiva: sbx non ha accesso a Docker.";

function agent(id: string, name: string, extra: Partial<ComponentInfo>): ComponentInfo {
  return {
    id,
    name,
    group: "agents",
    desc: "agente",
    mb: 100,
    chosen: true,
    installed: false,
    present: false,
    locked: false,
    version: null,
    available: false,
    unavailableReason: null,
    notNeeded: null,
    needs: [],
    versioned: false,
    home: "https://example.org",
    agent: null,
    leftover: false,
    ...extra,
  };
}

const KIT = { byId: new Map() } as unknown as KitVersions;

function row(c: ComponentInfo) {
  render(
    <ComponentRow
      c={c}
      list={[c]}
      chosen={new Set([c.id])}
      forProjects={[]}
      kit={KIT}
      setup={null}
      links={[]}
      onChange={() => undefined}
    />,
  );
  return screen.getByRole<HTMLInputElement>("checkbox");
}

describe("the agents' rows", () => {
  it("says «fornito dalla VM» for Claude Code and Codex", () => {
    const box = row(agent("claude", "Claude Code", { agent: { kind: "provided" } }));
    expect(box.disabled).toBe(true);
    expect(screen.getByText("fornito dalla VM").title).toContain("Docker Sandboxes");
  });

  it("says «serve la VM» with the reason for Pi without the VM", () => {
    const box = row(agent("pi", "Pi", { agent: { kind: "needsVm", reason: REASON } }));
    expect(box.disabled).toBe(true);
    expect(screen.getByText("serve la VM").title).toBe(REASON);
  });

  it("says an install for the PC goes with «Applica»", () => {
    row(agent("codex", "Codex", { agent: { kind: "provided" }, leftover: true, present: true }));
    expect(screen.getByText("da rimuovere").title).toContain("«Applica»");
  });
});
