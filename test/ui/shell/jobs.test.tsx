// The long work in sight (LEVELS.md §8.6): the status bar's item while a
// job runs (one, several, none), the VM's pill in each state with its
// name, a job's details in Activity (steps, error, «Annulla», «Riprova»),
// the card of a session waiting for its VM, and the jobs kept by the events.
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import type { IsolationStatus, Job } from "@/ipc";
import { vmPill, VmPillBadge } from "@/shell/isolation";
import { JobDetails, overallPercent, useJobs, VmPreparing } from "@/shell/jobs";
import { JobsItem, VmItem } from "@/shell/statusbar";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));
const events = vi.hoisted(() => new Map<string, (e: { payload: unknown }) => void>());
vi.mock("@tauri-apps/api/event", () => ({
  listen: (name: string, handler: (e: { payload: unknown }) => void) => {
    events.set(name, handler);
    return Promise.resolve(() => {
      events.delete(name);
    });
  },
}));

afterEach(() => {
  cleanup();
  invoke.mockReset();
  events.clear();
});

function job(id: string, extra: Partial<Job> = {}): Job {
  return {
    id,
    kind: "install",
    title: `Installo ${id}`,
    project: null,
    items: [id],
    step: null,
    steps: [],
    bytes: null,
    note: null,
    state: "running",
    error: null,
    created: 1,
    automatic: false,
    ...extra,
  };
}

const step = (index: number, total: number, label: string, state: Job["state"] = "running") => ({
  index,
  total,
  label,
  state,
  error: null,
});

describe("the status bar's jobs item", () => {
  it("shows the job running with its step, bar and percent, and opens it", () => {
    const onJob = vi.fn();
    const node = job("node", {
      title: "Installo Node.js",
      step: step(1, 1, "Node.js"),
      bytes: { done: 30, total: 120 },
    });
    render(<JobsItem jobs={[node]} onJob={onJob} />);
    const item = screen.getByRole("button", { name: /^Installo Node\.js · 1\/1 · Node\.js · 25%/ });
    expect(within(item).getByRole("progressbar").getAttribute("aria-valuenow")).toBe("25");
    expect(within(item).getByText("25%")).toBeTruthy();
    fireEvent.click(item);
    expect(onJob).toHaveBeenCalledWith("node");
  });

  it("counts several jobs and shows the running one first", () => {
    const jobs = [
      job("a", { state: "queued", created: 1, title: "Installo Python" }),
      job("b", { state: "running", created: 2, title: "Preparazione VM · shop", kind: "vm-prepare" }),
      job("c", { state: "queued", created: 3 }),
      job("d", { state: "done", created: 0 }),
    ];
    render(<JobsItem jobs={jobs} onJob={() => undefined} />);
    const item = screen.getByRole("button", { name: /^3 lavori in corso · Preparazione VM · shop/ });
    expect(within(item).getByText("3 lavori in corso:")).toBeTruthy();
    // No size known: the bar goes across, without a value.
    const bar = within(item).getByRole("progressbar");
    expect(bar.getAttribute("aria-valuenow")).toBeNull();
    expect(bar.getAttribute("aria-busy")).toBe("true");
  });

  it("is not there when nothing runs or waits", () => {
    const { container } = render(
      <JobsItem jobs={[job("x", { state: "done" }), job("y", { state: "error" })]} onJob={() => undefined} />,
    );
    expect(container.innerHTML).toBe("");
  });
});

const ON: IsolationStatus = { level: "vm", blockers: [], sbx: "sbx" };

describe("the VM's pill", () => {
  it("is green «VM» when ready", () => {
    const pill = vmPill(ON, []);
    expect([pill.tone, pill.text, pill.label]).toEqual(["ready", "VM", "Progetti nella VM"]);
  });

  it("is blue «VM…» with its percent while a VM is prepared", () => {
    const prep = job("p", { kind: "vm-prepare", title: "Preparazione VM · shop", step: step(3, 4, "Avvio la VM") });
    const pill = vmPill(ON, [prep]);
    expect(pill.tone).toBe("busy");
    expect(pill.percent).toBe(overallPercent(prep));
    expect(pill.text).toBe("VM… 50%");
    expect(pill.label).toContain("Preparazione VM · shop · 3/4 · Avvio la VM");
  });

  it("is red «VM ⚠» when the last preparation failed, green again after one went well", () => {
    const failed = job("f", { kind: "vm-prepare", state: "error", error: "sbx: no", created: 1 });
    expect(vmPill(ON, [failed])).toMatchObject({ tone: "error", text: "VM ⚠" });
    expect(vmPill(ON, [failed]).label).toContain("sbx: no");
    const fine = job("g", { kind: "vm-prepare", state: "done", created: 2 });
    expect(vmPill(ON, [failed, fine]).tone).toBe("ready");
  });

  it("is orange when the user can fix it, grey when this PC cannot have it, red on sbx's error", () => {
    const off = (codes: IsolationStatus["blockers"]): IsolationStatus => ({ level: "env", blockers: codes, sbx: null });
    expect(vmPill(off([{ code: "sbx_signed_out" }]), [])).toMatchObject({ tone: "fixable", text: "Senza VM" });
    expect(vmPill(off([{ code: "hypervisor_off" }, { code: "sbx_missing" }]), []).tone).toBe("fixable");
    expect(vmPill(off([{ code: "old_windows", build: 19045 }]), []).tone).toBe("off");
    expect(vmPill(off([{ code: "kit_disk", fs: "exFAT" }]), []).tone).toBe("off");
    expect(vmPill(off([{ code: "kvm_missing" }]), []).label).toContain("questo PC non la può avere");
    expect(vmPill(off([{ code: "sbx_failed", message: "x" }]), []).tone).toBe("error");
  });

  it("says its state with its text, not with the color alone", () => {
    const pill = vmPill(ON, [job("p", { kind: "vm-prepare", step: step(1, 4, "Strumenti") })]);
    render(<VmPillBadge pill={pill} />);
    const badge = screen.getByText("VM… 0%").parentElement as HTMLElement;
    expect(badge.dataset.tone).toBe("busy");
    expect(badge.className).toContain("bg-vsc-pill-busy");
  });

  it("is a button named with the state and what to do, which opens the VM's box", async () => {
    invoke.mockImplementation((c) =>
      Promise.resolve(
        c === "isolation_status"
          ? { level: "env", blockers: [{ code: "sbx_signed_out" }], sbx: "s" }
          : c === "vms_list"
            ? { vms: [], disk: null }
            : [],
      ),
    );
    render(<VmItem installing={false} jobs={[]} />);
    const button = await screen.findByRole("button", {
      name: "Senza VM, si può attivare: progetti e azioni sul PC senza agenti. Clic: cosa serve per attivare la VM",
    });
    expect(within(button).getByText("Senza VM")).toBeTruthy();
    fireEvent.click(button);
    expect(await screen.findByText("Lavori senza VM")).toBeTruthy();
  });
});

describe("a job in Activity", () => {
  it("lists its steps with the error, and retries a failed install", async () => {
    invoke.mockResolvedValue(undefined);
    const failed = job("node", {
      kind: "update",
      title: "Aggiorno Node.js",
      state: "error",
      error: "Node.js: checksum",
      steps: [{ ...step(1, 1, "Node.js", "error"), error: "checksum" }],
    });
    render(<JobDetails job={failed} />);
    expect(screen.getByText("Non riuscito")).toBeTruthy();
    expect(screen.getByText("checksum")).toBeTruthy();
    expect(screen.getByRole("alert").textContent).toBe("Node.js: checksum");
    fireEvent.click(screen.getByRole("button", { name: "Riprova" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("setup_start", { only: ["node"], updateAll: true });
    });
  });

  it("cancels an install that waits, never a VM's preparation", async () => {
    invoke.mockResolvedValue(true);
    render(<JobDetails job={job("python", { state: "queued" })} />);
    expect(screen.getAllByText("In coda").length).toBeGreaterThan(0);
    fireEvent.click(screen.getByRole("button", { name: "Annulla" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("job_cancel", { id: "python" });
    });
    cleanup();
    render(<JobDetails job={job("p", { kind: "vm-prepare", step: step(2, 4, "Database del progetto") })} />);
    expect(screen.queryByRole("button", { name: "Annulla" })).toBeNull();
    expect(screen.getByText("2/4 · Database del progetto")).toBeTruthy();
  });
});

describe("a session waiting for its VM", () => {
  it("shows the preparation of its project, with its step and bar", () => {
    invoke.mockResolvedValue([
      job("p", {
        kind: "vm-prepare",
        project: "shop",
        title: "Preparazione VM · shop",
        step: step(1, 4, "Strumenti per il Linux della VM"),
        note: "Node.js",
        bytes: { done: 1, total: 2 },
      }),
      job("q", { kind: "vm-prepare", project: "blog", title: "Preparazione VM · blog" }),
    ]);
    render(<VmPreparing project="shop" />);
    return waitFor(() => {
      const card = screen.getByRole("status", { name: "Preparazione della VM…" });
      expect(within(card).getByText("1/4 · Strumenti per il Linux della VM · Node.js")).toBeTruthy();
      expect(within(card).getByRole("progressbar").getAttribute("aria-valuenow")).toBe("50");
      expect(screen.queryByText("Preparazione VM · blog")).toBeNull();
    });
  });
});

function Titles() {
  const jobs = useJobs();
  return (
    <ul>
      {jobs.map((j) => (
        <li key={j.id}>{`${j.title}: ${j.state}`}</li>
      ))}
    </ul>
  );
}

describe("useJobs", () => {
  it("reads the jobs once, then follows the `job` events", async () => {
    invoke.mockResolvedValue([job("a", { title: "A" })]);
    render(<Titles />);
    await screen.findByText("A: running");
    await waitFor(() => {
      expect(events.has("job")).toBe(true);
    });
    act(() => {
      events.get("job")?.({ payload: job("a", { title: "A", state: "done" }) });
      events.get("job")?.({ payload: job("b", { title: "B", created: 5 }) });
    });
    // The events of a frame render together, at the next one.
    expect(screen.queryByText("A: done")).toBeNull();
    expect(await screen.findByText("A: done")).toBeTruthy();
    expect(screen.getByText("B: running")).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith("jobs_list", undefined);
  });

  it("is one subscription for every component, forgotten with the last one", async () => {
    invoke.mockResolvedValue([job("a", { title: "A" })]);
    const first = render(<Titles />);
    const second = render(<Titles />);
    await waitFor(() => {
      expect(screen.getAllByText("A: running")).toHaveLength(2);
    });
    expect(invoke).toHaveBeenCalledTimes(1);
    first.unmount();
    expect(events.has("job")).toBe(true);
    second.unmount();
    await waitFor(() => {
      expect(events.has("job")).toBe(false);
    });
    // Shown again: read again, nothing kept from before.
    invoke.mockResolvedValue([]);
    render(<Titles />);
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledTimes(2);
    });
    expect(screen.queryByText("A: running")).toBeNull();
  });
});
