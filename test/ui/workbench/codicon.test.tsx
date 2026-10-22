import { afterEach, describe, expect, it } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import { Codicon, codicon } from "@/workbench/codicon";
import codiconCss from "../../../node_modules/@vscode/codicons/dist/codicon.css?raw";
import themeCss from "@/theme/theme.css?raw";

// The app's sources, as text.
const sources = import.meta.glob<string>("../../../ui/src/**/*.{ts,tsx}", {
  query: "?raw",
  import: "default",
  eager: true,
});

afterEach(cleanup);

/** The classes of an element, as a list. */
function classes(el: Element | null) {
  return (el?.getAttribute("class") ?? "").split(/\s+/).filter(Boolean);
}

describe("Codicon", () => {
  it("draws a glyph of the font, 16px by its token, hidden from assistive technology", () => {
    const { container } = render(<Codicon name="gear" className="text-vsc-desc" />);
    const icon = container.firstElementChild;
    expect(icon?.tagName).toBe("SPAN");
    expect(classes(icon)).toEqual(["codicon", "codicon-gear", "text-vsc-codicon", "text-vsc-desc"]);
    expect(icon?.getAttribute("aria-hidden")).toBe("true");
    expect(icon?.hasAttribute("role")).toBe(false);
    expect(icon?.getAttribute("style")).toBeNull();
  });

  it("takes VSCodium's compact token at 12px, its own font size at the others", () => {
    const { container } = render(
      <>
        <Codicon name="close" size={12} />
        <Codicon name="close" size={56} />
        <Codicon name="close" size="1em" />
      </>,
    );
    const [compact, big, text] = Array.from(container.children) as HTMLElement[];
    expect(classes(compact ?? null)).toContain("text-vsc-codicon-compact");
    expect(compact?.style.fontSize).toBe("");
    expect(classes(big ?? null)).not.toContain("text-vsc-codicon");
    expect(big?.style.fontSize).toBe("56px");
    expect(text?.style.fontSize).toBe("1em");
  });

  it("is an image with its name when it has a label", () => {
    render(<Codicon name="warning" size={12} label="Mancano componenti" title="Mancano componenti" />);
    const icon = screen.getByRole("img", { name: "Mancano componenti" });
    expect(classes(icon)).toContain("codicon-warning");
    expect(icon.hasAttribute("aria-hidden")).toBe(false);
    expect(icon.getAttribute("title")).toBe("Mancano componenti");
  });

  it("gives one icon component per name, with the props of react-icons", () => {
    expect(codicon("refresh")).toBe(codicon("refresh"));
    expect(codicon("refresh")).not.toBe(codicon("add"));
    const Refresh = codicon("refresh");
    render(<Refresh size={14} className="animate-spin" aria-label="Aggiorna" />);
    const icon = screen.getByRole("img", { name: "Aggiorna" });
    expect(classes(icon)).toEqual(["codicon", "codicon-refresh", "animate-spin"]);
    expect(icon.style.fontSize).toBe("14px");
  });
});

describe("the app's icons", () => {
  it("are the codicons only: nothing imports react-icons/vsc", () => {
    const users = Object.entries(sources)
      .filter(([, text]) => text.includes("react-icons/vsc"))
      .map(([file]) => file);
    expect(users).toEqual([]);
  });

  it("are all glyphs of the bundled codicon.css", () => {
    const names = new Set<string>();
    for (const text of Object.values(sources)) {
      for (const m of text.matchAll(/\bcodicon\("([\w-]+)"\)/g)) names.add(m[1] ?? "");
      for (const m of text.matchAll(/<Codicon\s+name="([\w-]+)"/g)) names.add(m[1] ?? "");
    }
    expect(names.size).toBeGreaterThan(90);
    const missing = [...names].filter((n) => !codiconCss.includes(`.codicon-${n}:before`));
    expect(missing).toEqual([]);
  });

  it("come from the font bundled with the app, in the base layer the utilities override", () => {
    expect(themeCss).toContain('@import "@vscode/codicons/dist/codicon.css" layer(base);');
    expect(codiconCss).toMatch(/@font-face\s*\{[^}]*font-family:\s*"codicon";[^}]*url\("\.\/codicon\.ttf/);
  });
});
