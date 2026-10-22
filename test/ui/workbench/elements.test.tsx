import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { VscodeTextfield } from "@/workbench/elements";

afterEach(cleanup);

// The app's sources, read as text.
const sources = import.meta.glob<string>(["../../../ui/src/**/*.ts", "../../../ui/src/**/*.tsx"], {
  query: "?raw",
  import: "default",
  eager: true,
});

// Every component the package has, by its folder (not imported here).
const components = Object.keys(import.meta.glob("../../../node_modules/@vscode-elements/elements/dist/*/index.js"))
  .map((path) => /dist\/(vscode-[a-z-]+)\/index\.js$/.exec(path)?.[1] ?? "")
  .filter(Boolean);

/** The tags the sources draw: raw <vscode-*> tags, the tag names in strings
 * (the wrappers' types, querySelector, restyle) and the wrappers. */
function usedTags(): Set<string> {
  const used = new Set<string>();
  for (const source of Object.values(sources)) {
    for (const m of source.matchAll(/<(vscode-[a-z-]+)[\s/>]|"(vscode-[a-z-]+)"/g)) used.add(m[1] ?? m[2] ?? "");
    for (const m of source.matchAll(/\bVscode([A-Z][A-Za-z]+)\b/g)) {
      used.add(`vscode-${(m[1] ?? "").replace(/[A-Z]/g, (c, i: number) => (i ? "-" : "") + c.toLowerCase())}`);
    }
  }
  return new Set([...used].filter((tag) => components.includes(tag)));
}

describe("vscode-elements", () => {
  it("registers the components the sources draw, and only those", () => {
    const used = usedTags();
    // The fields, the select and the button at least: the scan reads the sources.
    expect([...used]).toEqual(expect.arrayContaining(["vscode-button", "vscode-textfield", "vscode-single-select"]));
    const registered = components.filter((tag) => customElements.get(tag) !== undefined);
    expect([...used].filter((tag) => !registered.includes(tag))).toEqual([]);
    // Beyond those, only what they import themselves: the scrolling list of
    // the select's dropdown.
    expect(registered.filter((tag) => !used.has(tag))).toEqual(["vscode-scrollable"]);
  });

  it("imports each component on its own, never the packages' main files", () => {
    for (const [path, source] of Object.entries(sources)) {
      expect({ path, main: /["']@vscode-elements\/(react-)?elements["']/.test(source) }).toEqual({ path, main: false });
    }
  });

  it("links the Codicon stylesheet once, by the id vscode-icon looks for", () => {
    const links = document.querySelectorAll<HTMLLinkElement>("link#vscode-codicon-stylesheet");
    expect(links).toHaveLength(1);
    expect(links[0]?.rel).toBe("stylesheet");
    // Only for the shadow roots: the page has codicon.css in its base layer.
    expect(links[0]?.disabled).toBe(true);
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
