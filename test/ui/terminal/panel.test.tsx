// The terminal panel (every system, D-148) with a fake backend and a fake
// xterm.js: a session that opens shows the panel with the output kept by
// Rust then the chunks after it, typing goes back to the session, and
// closing a tab whose program still runs asks first.
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { PtyBackend, PtyChunk, PtyInfo } from "../../../ui/src/terminal";
import { useTerminals } from "../../../ui/src/terminal";
import { TerminalPanel } from "../../../ui/src/terminal/panel";
import { clipboardKey, sequencer } from "../../../ui/src/terminal/view";

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
    private onBinaryHandler: ((d: string) => void) | null = null;
    onBinary(handler: (d: string) => void) {
      this.onBinaryHandler = handler;
      return {
        dispose: () => {
          this.onBinaryHandler = null;
        },
      };
    }
    sendBinary(data: string) {
      this.onBinaryHandler?.(data);
    }
    unicode = { activeVersion: "6" };
    keyHandler: ((e: KeyboardEvent) => boolean) | null = null;
    attachCustomKeyEventHandler(handler: (e: KeyboardEvent) => boolean) {
      this.keyHandler = handler;
    }
    getSelection() {
      return "";
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
vi.mock("@xterm/addon-unicode11", () => ({
  Unicode11Addon: class {
    activate() {}
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
  const failures = { close: null as string | null };
  const sessions = vi.fn(() => Promise.resolve(start));
  const backend: PtyBackend = {
    sessions,
    backlog: (id) => Promise.resolve(backlogs.get(id) ?? { seq: 0, data: "" }),
    write: (id, data) => {
      calls.push(`write ${String(id)} ${data}`);
      return Promise.resolve();
    },
    writeBinary: (id, data) => {
      calls.push(`binary ${String(id)} ${Array.from(data, (c) => c.charCodeAt(0)).join(",")}`);
      return Promise.resolve();
    },
    resize: (id, cols, rows) => {
      calls.push(`resize ${String(id)} ${String(cols)}x${String(rows)}`);
      return Promise.resolve();
    },
    close: (id) => {
      calls.push(`close ${String(id)}`);
      return failures.close === null ? Promise.resolve() : Promise.reject(new Error(failures.close));
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
    failures,
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
    // Mouse reports that are not text reach the program as bytes.
    terms[0]?.sendBinary("\x1b[M \xff\xa0");
    expect(fake.calls).toContain("binary 1 27,91,77,32,255,160");
  });

  it("shows why an ended session's tab could not close", async () => {
    const fake = fakeBackend();
    render(<Harness backend={fake.backend} />);
    await waitFor(() => {
      expect(fake.sessions).toHaveBeenCalled();
    });
    fake.open({ ...BASH, running: false, code: 0 }, { seq: 0, data: "" });
    fake.failures.close = "terminal session 1 not found";
    fireEvent.click(screen.getByRole("button", { name: "Chiudi il terminale" }));
    expect(screen.queryByRole("alertdialog")).toBeNull();
    await waitFor(() => {
      expect(screen.getByText(/terminal session 1 not found/)).toBeTruthy();
    });
    // The tab stays; a later attempt that works clears the error.
    expect(screen.getByRole("tab", { name: "app · bash (terminato)" })).toBeTruthy();
    fake.failures.close = null;
    fireEvent.click(screen.getByRole("button", { name: "Chiudi il terminale" }));
    await waitFor(() => {
      expect(screen.getByText("hidden 0")).toBeTruthy();
    });
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

  it("shows ended a session whose exit came before its opening", async () => {
    const fake = fakeBackend();
    render(<Harness backend={fake.backend} />);
    await waitFor(() => {
      expect(fake.sessions).toHaveBeenCalled();
    });
    fake.exit({ ...BASH, running: false, code: 1 });
    fake.open(BASH, { seq: 0, data: "" });
    expect(screen.getByRole("tab", { name: "app · bash (terminato)" })).toBeTruthy();
    // No question: its program is gone.
    fireEvent.click(screen.getByRole("button", { name: "Chiudi il terminale" }));
    expect(screen.queryByRole("alertdialog")).toBeNull();
    await waitFor(() => {
      expect(fake.calls).toContain("close 1");
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

describe("clipboard keys off macOS (D-46, D-148)", () => {
  const key = (k: string, mods: { shift?: boolean; alt?: boolean; meta?: boolean; ctrl?: boolean } = {}) => ({
    type: "keydown",
    key: k,
    ctrlKey: mods.ctrl ?? true,
    shiftKey: mods.shift ?? false,
    altKey: mods.alt ?? false,
    metaKey: mods.meta ?? false,
  });

  it("copies a selection on Ctrl+C, else Ctrl+C goes to the program", () => {
    const copied: string[] = [];
    const copy = (s: string) => copied.push(s);
    expect(clipboardKey(key("c"), "", copy)).toBe(true);
    expect(clipboardKey(key("c"), "ls -la", copy)).toBe(false);
    expect(clipboardKey(key("C", { shift: true }), "x", copy)).toBe(false);
    // Ctrl+Shift+C never reaches the program, even without a selection.
    expect(clipboardKey(key("C", { shift: true }), "", copy)).toBe(false);
    expect(copied).toEqual(["ls -la", "x"]);
  });

  it("leaves Ctrl+V to the browser's paste and the other keys to xterm.js", () => {
    const copy = vi.fn();
    expect(clipboardKey(key("v"), "", copy)).toBe(false);
    expect(clipboardKey(key("V", { shift: true }), "", copy)).toBe(false);
    expect(clipboardKey(key("d"), "", copy)).toBe(true);
    expect(clipboardKey(key("c", { ctrl: false }), "sel", copy)).toBe(true);
    expect(clipboardKey(key("c", { alt: true }), "sel", copy)).toBe(true);
    expect(clipboardKey({ ...key("c"), type: "keyup" }, "sel", copy)).toBe(true);
    expect(copy).not.toHaveBeenCalled();
  });
});
