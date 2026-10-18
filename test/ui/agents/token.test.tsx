// "Collega con un token" (WP-4.2): offered for Claude Code on macOS only
// (tokenLink from Rust), not once signed in; runs claude setup-token, or
// saves a token pasted by hand.
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { AgentInfo } from "../../../ui/src/api";
import { TokenLink, tokenOffered } from "../../../ui/src/agents/token";

const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: unknown) => invoke(cmd, args),
  isTauri: () => true,
}));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

const CLAUDE = {
  id: "claude",
  name: "Claude Code",
  installed: true,
  version: "2.1.0",
  signedIn: false,
  signInKind: null,
  running: false,
  tokenLink: true,
} as AgentInfo;

const TOKEN = `sk-ant-oat01-${"a".repeat(90)}`;

async function innerInput(host: Element) {
  const field = host as HTMLElementTagNameMap["vscode-textfield"];
  await field.updateComplete;
  return field.wrappedElement;
}

describe("TokenLink", () => {
  it("is offered only where Rust says so and only while signed out", () => {
    expect(tokenOffered(CLAUDE)).toBe(true);
    expect(tokenOffered({ ...CLAUDE, tokenLink: false })).toBe(false);
    const { container, rerender } = render(
      <TokenLink agent={{ ...CLAUDE, tokenLink: false }} onDone={vi.fn()} />,
    );
    expect(container.textContent).toBe("");
    rerender(<TokenLink agent={{ ...CLAUDE, signedIn: true, signInKind: "token" }} onDone={vi.fn()} />);
    expect(container.textContent).toBe("");
    rerender(<TokenLink agent={CLAUDE} onDone={vi.fn()} />);
    expect(screen.getByRole("button", { name: "Collega con un token" })).toBeTruthy();
  });

  it("runs claude setup-token in the app's terminal", async () => {
    invoke.mockResolvedValue(undefined);
    const onDone = vi.fn();
    render(<TokenLink agent={CLAUDE} onDone={onDone} />);
    fireEvent.click(screen.getByRole("button", { name: "Collega con un token" }));
    await waitFor(() => {
      expect(onDone).toHaveBeenCalled();
    });
    expect(invoke).toHaveBeenCalledWith("agent_token_link", undefined);
  });

  it("saves a pasted token, trimmed, and empties the field", async () => {
    invoke.mockResolvedValue(undefined);
    const onDone = vi.fn();
    render(<TokenLink agent={CLAUDE} onDone={onDone} />);
    const save = screen.getByRole("button", { name: "Salva il token" });
    expect(save.getAttribute("aria-disabled") ?? save.getAttribute("disabled")).not.toBeNull();
    const host = screen.getByLabelText<HTMLElementTagNameMap["vscode-textfield"]>("Token di Claude Code");
    const input = await innerInput(host);
    expect(input.getAttribute("type")).toBe("password");
    act(() => {
      input.value = `  ${TOKEN}\n`;
      input.dispatchEvent(new Event("input", { bubbles: true, composed: true }));
    });
    fireEvent.click(screen.getByRole("button", { name: "Salva il token" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("agent_token_save", { token: TOKEN });
    });
    await waitFor(() => {
      expect(host.value).toBe("");
    });
    expect(onDone).toHaveBeenCalled();
  });

  it("shows why it failed", async () => {
    invoke.mockRejectedValue("Claude Code non è installato.");
    render(<TokenLink agent={CLAUDE} onDone={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: "Collega con un token" }));
    expect(await screen.findByText("Claude Code non è installato.")).toBeTruthy();
  });
});
