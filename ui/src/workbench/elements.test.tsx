import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { VscodeTextfield } from "@vscode-elements/react-elements";
import "./elements";

afterEach(cleanup);

describe("vscode-elements", () => {
  it("links the Codicon stylesheet once, by the id vscode-icon looks for", () => {
    const links = document.querySelectorAll<HTMLLinkElement>("link#vscode-codicon-stylesheet");
    expect(links).toHaveLength(1);
    expect(links[0]?.rel).toBe("stylesheet");
  });

  it("renders a vscode-button with its name, class and clicks", async () => {
    const onClick = vi.fn();
    render(
      <vscode-button class="wide" onClick={onClick}>
        Salva
      </vscode-button>,
    );
    const button = await screen.findByRole("button", { name: "Salva" });
    expect(button.tagName).toBe("VSCODE-BUTTON");
    expect(button.getAttribute("class")).toBe("wide");
    expect(button.shadowRoot?.querySelector("slot")).not.toBeNull();
    fireEvent.click(button);
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("gives a wrapped text field's value to onChange", async () => {
    const onChange = vi.fn((e: Event) => (e.target as HTMLInputElement).value);
    render(<VscodeTextfield label="Nome" value="a" onChange={onChange} />);
    const field = document.querySelector("vscode-textfield");
    const input = await waitFor(() => {
      const el = field?.shadowRoot?.querySelector("input");
      if (!el) throw new Error("not rendered yet");
      return el;
    });
    await waitFor(() => {
      expect(input.value).toBe("a");
    });
    input.value = "ab";
    fireEvent.change(input);
    expect(onChange).toHaveBeenCalledTimes(1);
    expect(onChange.mock.results[0]?.value).toBe("ab");
  });
});
