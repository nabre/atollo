// The terminal panel of macOS and Linux with a fake backend and a fake
// xterm.js: a session that opens shows the panel with the output kept by
// Rust then the chunks after it, typing goes back to the session, and
// closing a tab whose program still runs asks first.
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { PtyBackend, PtyChunk, PtyInfo } from "../../../ui/src/terminal";
import { useTerminals } from "../../../ui/src/terminal";
import { TerminalPanel } from "../../../ui/src/terminal/panel";
import { sequencer } from "../../../ui/src/terminal/view";

// The fake xterm.js, made before the mocks (vi.mock is hoisted); every
// one made is kept, with what was written to it.
const { terms, FakeTerminal } = vi.hoisted(() => {
  const terms: InstanceType<typeof FakeTerminal>[] = [];
  class FakeTerminal {
    written = "";
    cols = 80;
    rows = 24;
    options: Record<string, unknown> = {};
    disposed = false;
    private onDataHandler: ((d: string) => void) | null = null;
    constructor(options: Record<string, unknown>) {
      this.options = options;
      terms.push(this);
    }
    loadAddon() {}
    open() {}
    focus() {}
    write(data: string) {
      this.written += data;
    }
    onData(handler: (d: string) => void) {
      this.onDataHandler = handler;
      return {
        dispose: () => {
          this.onDataHandler = null;
        },
      };
    }
    type(data: string) {
      this.onDataHandler?.(data);
    }
    dispose() {
      this.disposed = true;
    }
  }
  return { terms, FakeTerminal };
});

vi.mock("@xterm/xterm", () => ({ Terminal: FakeTerminal }));
vi.mock("@xterm/addon-fit", () => ({
  FitAddon: class {
    fit() {}
  },
}));

afterEach(() => {
  cleanup();
  terms.length = 0;
});

/** Rust, in memory: sessions, their backlogs and the events. */
function fakeBackend(start: PtyInfo[] = []) {
  const opened: ((i: PtyInfo) => void)[] = [];
  const exited: ((i: PtyInfo) => void)[] = [];
  const output = new Map<number, ((c: PtyChunk) => void)[]>();
  const backlogs = new Map<number, PtyChunk>();
  const calls: string[] = [];
  const sessions = vi.fn(() => Promise.resolve(start));
  const backend: PtyBackend = {
    sessions,
    backlog: (id) => Promise.resolve(backlogs.get(id) ?? { seq: 0, data: "" }),
    write: (id, data) => {
      calls.push(`write ${String(id)} ${data}`);
      return Promise.resolve();
    },
    resize: (id, cols, rows) => {
      calls.push(`resize ${String(id)} ${String(cols)}x${String(rows)}`);
      return Promise.resolve();
    },
    close: (id) => {
      calls.push(`close ${String(id)}`);
      return Promise.resolve();
    },
    onOpened: (h) => {
      opened.push(h);
      return Promise.resolve(() => undefined);
    },
    onExit: (h) => {
      exited.push(h);
      return Promise.resolve(() => undefined);
    },
    onOutput: (id, h) => {
      output.set(id, [...(output.get(id) ?? []), h]);
      return Promise.resolve(() => undefined);
    },
  };
  return {
    backend,
    sessions,
    calls,
    open(info: PtyInfo, backlog: PtyChunk) {
      backlogs.set(info.id, backlog);
      act(() => {
        opened.forEach((h) => {
          h(info);
        });
      });
    },
    out(id: number, chunk: PtyChunk) {
      act(() => {
        (output.get(id) ?? []).forEach((h) => {
          h(chunk);
        });
      });
    },
    exit(info: PtyInfo) {
      act(() => {
        exited.forEach((h) => {
          h(info);
        });
      });
    },
  };
}

function Harness({ backend, enabled = true }: { backend: PtyBackend; enabled?: boolean }) {
  const terminals = useTerminals(backend, enabled);
  return terminals.shown ? (
    <TerminalPanel terminals={terminals} backend={backend} height={200} />
  ) : (
    <p>{`hidden ${String(terminals.sessions.length)}`}</p>
  );
}

const BASH: PtyInfo = { id: 1, title: "app · bash", dir: "/k/projects/app", running: true, code: null };

describe("sequencer", () => {
  it("writes the backlog, then only the later chunks, then the notes", () => {
    let out = "";
    const s = sequencer((d) => {
      out += d;
    });
    s.chunk({ seq: 2, data: "b" });
    s.chunk({ seq: 3, data: "c" });
    s.note("[end]");
    expect(out).toBe("");
    s.backlog({ seq: 2, data: "ab" });
    expect(out).toBe("abc[end]");
    s.chunk({ seq: 3, data: "c" });
    s.chunk({ seq: 4, data: "d" });
    expect(out).toBe("abc[end]d");
  });
});

describe("TerminalPanel", () => {
  it("reads and listens to nothing when disabled (Windows)", () => {
    const fake = fakeBackend();
    render(<Harness backend={fake.backend} enabled={false} />);
    expect(fake.sessions).not.toHaveBeenCalled();
    expect(screen.getByText("hidden 0")).toBeTruthy();
  });

  it("shows a session that opens, with its output, and sends what is typed", async () => {
    const fake = fakeBackend();
    render(<Harness backend={fake.backend} />);
    await waitFor(() => {
      expect(fake.sessions).toHaveBeenCalled();
    });
    fake.open(BASH, { seq: 1, data: "$ " });
    expect(screen.getByRole("tab", { name: "app · bash" })).toBeTruthy();
    await waitFor(() => {
      expect(terms[0]?.written).toBe("$ ");
    });
    fake.out(1, { seq: 1, data: "$ " });
    fake.out(1, { seq: 2, data: "ls\r\n" });
    expect(terms[0]?.written).toBe("$ ls\r\n");
    terms[0]?.type("pwd\r");
    expect(fake.calls).toContain("write 1 pwd\r");
  });

  it("asks before closing a running session, not an ended one", async () => {
    const fake = fakeBackend([BASH]);
    render(<Harness backend={fake.backend} />);
    fake.open(BASH, { seq: 0, data: "" });
    fireEvent.click(screen.getByRole("button", { name: "Chiudi il terminale" }));
    expect(screen.getByRole("alertdialog")).toBeTruthy();
    expect(fake.calls).not.toContain("close 1");
    fireEvent.click(screen.getByRole("button", { name: "Annulla" }));
    expect(screen.queryByRole("alertdialog")).toBeNull();

    fake.exit({ ...BASH, running: false, code: 0 });
    expect(screen.getByRole("tab", { name: "app · bash (terminato)" })).toBeTruthy();
    await waitFor(() => {
      expect(terms[0]?.written).toContain("[processo terminato, codice 0]");
    });
    fireEvent.click(screen.getByRole("button", { name: "Chiudi il terminale" }));
    expect(screen.queryByRole("alertdialog")).toBeNull();
    await waitFor(() => {
      expect(fake.calls).toContain("close 1");
    });
    // The last tab gone: the panel hides.
    await waitFor(() => {
      expect(screen.getByText("hidden 0")).toBeTruthy();
    });
  });

  it("closes a running session once confirmed", async () => {
    const fake = fakeBackend();
    render(<Harness backend={fake.backend} />);
    await waitFor(() => {
      expect(fake.sessions).toHaveBeenCalled();
    });
    fake.open(BASH, { seq: 0, data: "" });
    fake.open({ ...BASH, id: 2, title: "app · claude" }, { seq: 0, data: "" });
    expect(screen.getByRole("tab", { name: "app · claude", selected: true })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Chiudi il terminale" }));
    const dialog = screen.getByRole("alertdialog");
    expect(dialog.textContent).toContain("app · claude");
    const confirm = Array.from(dialog.querySelectorAll("vscode-button, button")).find((b) =>
      b.textContent.includes("Chiudi il terminale"),
    );
    if (!confirm) throw new Error("no confirm button");
    fireEvent.click(confirm);
    await waitFor(() => {
      expect(fake.calls).toContain("close 2");
    });
    await waitFor(() => {
      expect(screen.getByRole("tab", { name: "app · bash", selected: true })).toBeTruthy();
    });
    // Its program ends after the tab closed: the tab does not come back.
    fake.exit({ ...BASH, id: 2, title: "app · claude", running: false, code: 0 });
    expect(screen.queryByRole("tab", { name: /app · claude/ })).toBeNull();
  });
});
