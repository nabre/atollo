// Lists, pane headers and scroll bars measured as VSCodium 1.135's modern
// UI, with the size tokens of theme-sizes.css: rows of 22px with the small
// radius, 4px in from a pane's sides, headers of 28px, 8px scroll bars in
// the theme's colors.
import { afterEach, describe, expect, it } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import themeCss from "../../../ui/src/theme/theme.css?raw";
import { DataTable, List, ListItem, ListRow, PaneStack, ViewPane } from "../../../ui/src/workbench";

afterEach(cleanup);

/** The classes of an element, as a list. */
function classes(el: Element | null) {
  return (el?.getAttribute("class") ?? "").split(/\s+/);
}

/** The row around a row's main button. */
function rowOf(name: string) {
  return screen.getByRole("button", { name }).parentElement;
}

describe("ListRow", () => {
  it("is a 22px row with the small radius, hovered when not selected", () => {
    render(
      <List label="Files">
        <ListRow label="a.txt" />
        <ListRow label="b.txt" selected />
      </List>,
    );
    const row = classes(rowOf("a.txt"));
    expect(row).toEqual(expect.arrayContaining(["h-vsc-row", "rounded-vsc-small", "hover:bg-vsc-list-hover"]));
    expect(row.some((c) => /^(h-\[|rounded$)/.test(c))).toBe(false);
    const selected = classes(rowOf("b.txt"));
    expect(selected).toEqual(
      expect.arrayContaining(["bg-vsc-list-inactive", "group-focus-within/list:bg-vsc-list-active"]),
    );
    expect(selected).not.toContain("hover:bg-vsc-list-hover");
  });

  it("indents 8px a level, with the twistie's room on a leaf", () => {
    render(
      <List label="Tree">
        <ListRow label="folder" twistie="open" depth={1}>
          <ListRow label="leaf" depth={2} />
        </ListRow>
      </List>,
    );
    const folder = screen.getByRole("button", { name: "folder" }).getAttribute("style") ?? "";
    expect(folder).toContain("calc(2 * var(--vscode-spacing-size80))");
    const leaf = screen.getByRole("button", { name: "leaf" }).getAttribute("style") ?? "";
    expect(leaf).toContain("calc(3 * var(--vscode-spacing-size80) + var(--vscode-spacing-size160))");
  });
});

describe("ListItem", () => {
  it("is the Extensions view's 62px row, 16px in, with a 36px icon", () => {
    render(
      <List label="Agents">
        <ListItem icon={<span />} name="Atollo" description="One" footer="kit" />
      </List>,
    );
    const button = screen.getByRole("button", { name: "Atollo" });
    const row = button.closest("li")?.firstElementChild ?? null;
    expect(classes(row)).toEqual(
      expect.arrayContaining(["h-vsc-row-extension", "pl-vsc-size160", "gap-vsc-size160", "rounded-vsc-small"]),
    );
    expect(classes(row?.firstElementChild ?? null)).toContain("size-vsc-size360");
  });
});

describe("ViewPane", () => {
  it("has a 28px header with the small radius, 4px in, hovered as a list row", () => {
    render(
      <PaneStack>
        <ViewPane title="Outline" expanded onToggle={() => {}} first>
          <List label="Symbols">
            <ListRow label="main" />
          </List>
        </ViewPane>
        <ViewPane title="Timeline" expanded={false} onToggle={() => {}}>
          <p>none</p>
        </ViewPane>
      </PaneStack>,
    );
    const header = classes(screen.getByRole("button", { name: "Outline" }).closest("h3"));
    expect(header).toEqual(
      expect.arrayContaining(["h-vsc-pane-header", "rounded-vsc-small", "mx-vsc-size40", "hover:bg-vsc-list-hover"]),
    );
    // A line above all but the first, 4px further in.
    expect(header.some((c) => c.startsWith("before:"))).toBe(false);
    const second = classes(screen.getByRole("button", { name: "Timeline" }).closest("h3"));
    expect(second).toEqual(expect.arrayContaining(["before:inset-x-vsc-size40", "before:bg-vsc-section-border"]));
    expect(classes(screen.getByRole("button", { name: "Outline" }))).toEqual(
      expect.arrayContaining(["text-vsc-label1", "font-vsc-semibold", "pl-vsc-size40"]),
    );
    // The rows 4px in, the scroll bar on the edge: the padding is the body's.
    const body = screen.getByRole("list", { name: "Symbols" }).parentElement;
    expect(classes(body)).toEqual(expect.arrayContaining(["overflow-y-auto", "px-vsc-size40"]));
    expect(classes(body?.closest("section")?.parentElement ?? null).some((c) => /^px-/.test(c))).toBe(false);
  });
});

describe("DataTable", () => {
  it("has the Keyboard Shortcuts editor's 30px header", () => {
    render(
      <DataTable columns={["Name", null]}>
        <tr>
          <td>a</td>
          <td />
        </tr>
      </DataTable>,
    );
    for (const th of screen.getAllByRole("columnheader")) {
      expect(classes(th)).toEqual(expect.arrayContaining(["h-vsc-table-header", "px-vsc-size80"]));
    }
  });
});

describe("scroll bars", () => {
  /** The declarations of a rule of theme.css. */
  function rule(selector: string) {
    const at = themeCss.indexOf(`${selector} {`);
    expect(at).toBeGreaterThanOrEqual(0);
    return themeCss.slice(at, themeCss.indexOf("}", at));
  }

  it("are the workbench's: 8px, a transparent track, the theme's slider colors", () => {
    expect(rule("::-webkit-scrollbar")).toContain("width: var(--vsc-size-scrollbar)");
    expect(rule("::-webkit-scrollbar")).toContain("height: var(--vsc-size-scrollbar)");
    expect(rule("::-webkit-scrollbar-corner")).toContain("background: transparent");
    const thumb = rule("::-webkit-scrollbar-thumb");
    expect(thumb).toContain("border-radius: var(--vscode-cornerRadius-small)");
    expect(thumb).toContain("background: var(--vsc-scrollbar)");
    expect(rule("::-webkit-scrollbar-thumb:hover")).toContain("var(--vsc-scrollbar-hover)");
    expect(rule("::-webkit-scrollbar-thumb:active")).toContain("var(--vsc-scrollbar-active)");
  });
});
