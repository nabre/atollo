// The menus of a tool's versions: the one of "+" (lines to download) and
// the one of a locked line (the projects that use it) move the focus with
// the keyboard and give it back to their button on Esc.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import type { VersionedTool } from "@/ipc";
import { AddLine, ToolVersions, type KitVersions } from "@/versions";

afterEach(cleanup);

const tool: VersionedTool = {
  id: "php",
  name: "PHP",
  installed: true,
  default: { line: "8.4", version: "8.4.1" },
  saved: [],
  preferred: null,
  downloadable: true,
  available: ["8.4", "8.3", "8.2", "7.4"],
  checked: null,
  projects: { "8.4": ["shop", "blog"] },
};

const kit = {
  byId: new Map(),
  setTools: () => undefined,
  busy: null,
  error: null,
  last: null,
  run: () => undefined,
  offers: { checking: new Set<string>(), failed: {}, refresh: () => undefined },
} as unknown as KitVersions;

describe("versions menus", () => {
  it("moves among the lines to download with the arrows, Home and End", () => {
    const onAdd = vi.fn();
    render(<AddLine tool={tool} setup={null} busy={false} onAdd={onAdd} />);
    fireEvent.click(screen.getByRole("button", { name: "Aggiungi una versione di PHP" }));
    const menu = screen.getByRole("menu", { name: "Versioni da scaricare" });
    const items = screen.getAllByRole("menuitem");
    expect(items.map((i) => i.textContent)).toEqual(["PHP 8.3", "PHP 8.2", "PHP 7.4"]);
    // The first item has the focus at once.
    expect(document.activeElement).toBe(items[0]);
    fireEvent.keyDown(menu, { key: "ArrowDown" });
    expect(document.activeElement).toBe(items[1]);
    fireEvent.keyDown(menu, { key: "End" });
    expect(document.activeElement).toBe(items[2]);
    fireEvent.keyDown(menu, { key: "ArrowDown" });
    expect(document.activeElement).toBe(items[0]);
    fireEvent.keyDown(menu, { key: "ArrowUp" });
    expect(document.activeElement).toBe(items[2]);
    fireEvent.keyDown(menu, { key: "Home" });
    expect(document.activeElement).toBe(items[0]);
    fireEvent.click(items[0] as HTMLElement);
    expect(onAdd).toHaveBeenCalledWith("8.3");
  });

  it("says that + opens a menu, and gives it the focus back once a line is chosen", () => {
    const onAdd = vi.fn();
    render(<AddLine tool={tool} setup={null} busy={false} onAdd={onAdd} />);
    const plus = screen.getByRole("button", { name: "Aggiungi una versione di PHP" });
    expect(plus.getAttribute("aria-haspopup")).toBe("menu");
    expect(plus.getAttribute("aria-expanded")).toBe("false");
    expect(plus.hasAttribute("aria-controls")).toBe(false);
    fireEvent.click(plus);
    expect(plus.getAttribute("aria-expanded")).toBe("true");
    const menu = screen.getByRole("menu", { name: "Versioni da scaricare" });
    expect(plus.getAttribute("aria-controls")).toBe(menu.id);
    fireEvent.keyDown(menu, { key: "ArrowDown" });
    fireEvent.click(document.activeElement as HTMLElement);
    expect(onAdd).toHaveBeenCalledWith("8.2");
    expect(screen.queryByRole("menu")).toBeNull();
    expect(plus.getAttribute("aria-expanded")).toBe("false");
    expect(document.activeElement).toBe(plus);
  });

  it("closes the menu on Esc and gives the focus back to +", () => {
    render(<AddLine tool={tool} setup={null} busy={false} onAdd={() => undefined} />);
    const plus = screen.getByRole("button", { name: "Aggiungi una versione di PHP" });
    fireEvent.click(plus);
    const menu = screen.getByRole("menu");
    fireEvent.keyDown(menu, { key: "ArrowDown" });
    fireEvent.keyDown(document.activeElement as Element, { key: "Escape" });
    expect(screen.queryByRole("menu")).toBeNull();
    expect(document.activeElement).toBe(plus);
  });

  it("says that a locked line opens a menu of its projects", () => {
    render(<ToolVersions tool={tool} kit={kit} setup={null} />);
    const lock = screen.getByRole("button", { name: "PHP 8.4 bloccata: progetti che la usano" });
    // The badges speak the source language (Italian), as the rest of the row.
    expect(lock.textContent).toBe("bloccata");
    // The name holds the visible word (WCAG 2.5.3, label in name).
    expect(lock.getAttribute("aria-label")).toContain(lock.textContent);
    expect(screen.getByText("più recente")).toBeTruthy();
    expect(lock.getAttribute("aria-haspopup")).toBe("menu");
    expect(lock.getAttribute("aria-expanded")).toBe("false");
    fireEvent.click(lock);
    expect(lock.getAttribute("aria-expanded")).toBe("true");
    const menu = screen.getByRole("menu", { name: "PHP 8.4 bloccata: progetti che la usano" });
    expect(lock.getAttribute("aria-controls")).toBe(menu.id);
    const items = screen.getAllByRole("menuitem");
    expect(document.activeElement).toBe(items[0]);
    fireEvent.keyDown(menu, { key: "ArrowDown" });
    expect(document.activeElement).toBe(items[1]);
    fireEvent.keyDown(document.activeElement as Element, { key: "Escape" });
    expect(screen.queryByRole("menu")).toBeNull();
    expect(document.activeElement).toBe(lock);
  });
});
