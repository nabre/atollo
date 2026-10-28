// The levels in the pages (D-179, LEVELS.md §4.1, §8.3): a refused command's
// error is read as its text and its fix, the error line shows the fix the
// app can act on (the VM's notice), and `useAction` keeps it.
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, renderHook, screen } from "@testing-library/react";
import { errorText, fixOf, isRefused, type RefusedError } from "@/ipc/levels";
import { ErrorLine } from "@/shared/common";
import { useAction } from "@/shared/hooks";
import { labelText, OPEN_VM_NOTICE } from "@/shared/levels";

afterEach(() => {
  cleanup();
});

const AGENTS: RefusedError = {
  message: "Gli agenti partono solo nella VM del progetto, e la VM non è attiva.",
  refusal: { code: "agents_need_vm", blockers: [] },
  fix: "open_vm_notice",
};

describe("a refused command's error", () => {
  it("is read as its message and its fix; any other error as its text", () => {
    expect(isRefused(AGENTS)).toBe(true);
    expect(errorText(AGENTS)).toBe(AGENTS.message);
    expect(fixOf(AGENTS)).toBe("open_vm_notice");
    expect(isRefused("Progetto non trovato")).toBe(false);
    expect(errorText("Progetto non trovato")).toBe("Progetto non trovato");
    expect(fixOf("Progetto non trovato")).toBeNull();
    expect(errorText(new Error("disco pieno"))).toBe("Error: disco pieno");
    expect(isRefused({ message: "x" })).toBe(false);
  });

  it("shows the refusal with the action of its fix: the VM's notice", () => {
    const opened = vi.fn();
    window.addEventListener(OPEN_VM_NOTICE, opened);
    render(<ErrorLine error={AGENTS.message} fix={AGENTS.fix} />);
    expect(screen.getByRole("alert").textContent).toContain(AGENTS.message);
    fireEvent.click(screen.getByRole("button", { name: "Apri l'avviso VM" }));
    expect(opened).toHaveBeenCalledTimes(1);
    window.removeEventListener(OPEN_VM_NOTICE, opened);
  });

  it("shows a pending restart of the daemon with the VM's notice too", () => {
    render(<ErrorLine error="Docker Sandboxes deve ripartire" fix="restart_daemon" />);
    expect(screen.getByRole("button", { name: "Apri l'avviso VM" })).toBeTruthy();
  });

  it("says the other fixes in its text only", () => {
    for (const fix of ["open_from_project", "use_local_disk", "none"] as const) {
      const { unmount } = render(<ErrorLine error="rifiutato" fix={fix} />);
      expect(screen.getByRole("alert").textContent).toBe("rifiutato");
      expect(screen.queryByRole("button")).toBeNull();
      unmount();
    }
  });

  it("is kept by useAction with its fix, cleared by the next run", async () => {
    const { result } = renderHook(() => useAction());
    await act(async () => {
      // What Tauri rejects with: the command's error as Rust sent it, not an Error.
      // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors
      result.current.run("terminal", () => Promise.reject(AGENTS));
      await Promise.resolve();
    });
    expect(result.current.error).toBe(AGENTS.message);
    expect(result.current.fix).toBe("open_vm_notice");
    await act(async () => {
      result.current.run("terminal", () => Promise.reject(new Error("altro")));
      await Promise.resolve();
    });
    expect(result.current.error).toBe("Error: altro");
    expect(result.current.fix).toBeNull();
  });
});

describe("a terminal's label (8.3)", () => {
  it("names the level and the place", () => {
    expect(labelText("pc")).toBe("PC");
    expect(labelText("agentVm")).toBe("Agente · VM");
  });
});
