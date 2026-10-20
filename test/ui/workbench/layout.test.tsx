// The sizes dragged with a sash: clamped at once, written to the browser's
// storage once the drag pauses, not at every pointer move. The panel's and
// the editor's tabs and a details editor's header and bar, measured as in
// VSCodium 1.135's modern UI with the size tokens.
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, renderHook, screen } from "@testing-library/react";
import { EditorHeader, EditorTabs, NavBar, Panel, useStoredSize } from "../../../ui/src/workbench/layout";

beforeEach(() => {
  vi.useFakeTimers();
  localStorage.clear();
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.useRealTimers();
});

describe("useStoredSize", () => {
  it("writes three quick changes once, after the pause", () => {
    const write = vi.spyOn(Storage.prototype, "setItem");
    const { result } = renderHook(() => useStoredSize("side", 300, 170, 640));
    act(() => {
      result.current[1](310);
    });
    act(() => {
      result.current[1]((v) => v + 5);
    });
    act(() => {
      result.current[1]((v) => v + 5);
    });
    expect(result.current[0]).toBe(320);
    expect(write).not.toHaveBeenCalled();
    act(() => {
      vi.advanceTimersByTime(150);
    });
    expect(write).toHaveBeenCalledTimes(1);
    expect(write).toHaveBeenCalledWith("size:side", "320");
  });

  it("clamps the size to its min and max", () => {
    const { result } = renderHook(() => useStoredSize("panel", 240, 120, 600));
    act(() => {
      result.current[1](10_000);
    });
    expect(result.current[0]).toBe(600);
    act(() => {
      result.current[1]((v) => v - 10_000);
    });
    expect(result.current[0]).toBe(120);
    act(() => {
      vi.advanceTimersByTime(150);
    });
    expect(localStorage.getItem("size:panel")).toBe("120");
  });

  it("clamps a stored size out of range", () => {
    localStorage.setItem("size:panel", "5000");
    const { result } = renderHook(() => useStoredSize("panel", 240, 120, 600));
    expect(result.current[0]).toBe(600);
  });

  it("writes the last size when it goes away before the pause", () => {
    const { result, unmount } = renderHook(() => useStoredSize("side", 300, 170, 640));
    act(() => {
      result.current[1](400);
    });
    unmount();
    expect(localStorage.getItem("size:side")).toBe("400");
  });

  it("still changes the size when the storage refuses it", () => {
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("full");
    });
    const { result } = renderHook(() => useStoredSize("side", 300, 170, 640));
    act(() => {
      result.current[1](350);
    });
    expect(() => {
      act(() => {
        vi.advanceTimersByTime(150);
      });
    }).not.toThrow();
    expect(result.current[0]).toBe(350);
  });
});

/** The classes of an element, as a list. */
const classes = (el: Element | null | undefined) => (el?.getAttribute("class") ?? "").split(/\s+/);

describe("EditorTabs", () => {
  const tabs = [
    { id: "a", label: "Uno" },
    { id: "b", label: "Due" },
    { id: "c", label: "Tabella", pinned: true },
  ];

  it("puts the tabs in a 32px bar, 24px pills on the list's colors", () => {
    render(<EditorTabs tabs={tabs} active="a" onSelect={vi.fn()} onClose={vi.fn()} />);
    expect(classes(screen.getByRole("tablist").parentElement)).toEqual(
      expect.arrayContaining(["h-vsc-tabs", "pl-vsc-size20"]),
    );
    const [label] = screen.getAllByRole("tab");
    const [on, off] = screen.getAllByRole("tab").map((tab) => tab.parentElement);
    for (const tab of [on, off]) {
      expect(classes(tab)).toEqual(
        expect.arrayContaining([
          "text-vsc-body1",
          "before:inset-x-vsc-size20",
          "before:inset-y-vsc-size40",
          "before:rounded-vsc-small",
          "hover:before:bg-vsc-list-hover",
        ]),
      );
    }
    expect(classes(on)).toEqual(expect.arrayContaining(["text-vsc-fg", "before:bg-vsc-list-inactive"]));
    // modernEditorTab.inactiveBackground is transparent, the text half the foreground.
    expect(classes(off)).toEqual(expect.arrayContaining(["text-vsc-fg/50", "hover:text-vsc-fg"]));
    expect(classes(off).some((c) => c.startsWith("before:bg-"))).toBe(false);
    expect(classes(label)).toEqual(expect.arrayContaining(["pl-vsc-size60", "pr-vsc-size80", "gap-vsc-size60"]));
  });

  it("puts the close button over the tab's right end, shown on hover or focus", () => {
    const onClose = vi.fn();
    render(<EditorTabs tabs={tabs} active="a" onSelect={vi.fn()} onClose={onClose} />);
    // The pinned tab has none.
    expect(screen.queryByRole("button", { name: "Chiudi Tabella" })).toBeNull();
    for (const name of ["Chiudi Uno", "Chiudi Due"]) {
      const close = screen.getByRole("button", { name });
      expect(classes(close)).toEqual(expect.arrayContaining(["size-vsc-size200", "rounded-vsc-small"]));
      const actions = close.parentElement;
      expect(classes(actions)).toEqual(
        expect.arrayContaining([
          "absolute",
          "right-vsc-size20",
          "inset-y-vsc-size40",
          "w-vsc-size240",
          "opacity-0",
          "group-hover/tab:opacity-100",
          "focus-within:opacity-100",
        ]),
      );
      // Over the label, not after it: the active tab's is hidden too.
      expect(classes(actions)).not.toContain("invisible");
    }
    // The active tab's on the selection color, the others' on the hover's.
    expect(classes(screen.getByRole("button", { name: "Chiudi Uno" }).parentElement)).toContain("bg-vsc-list-inactive");
    expect(classes(screen.getByRole("button", { name: "Chiudi Due" }).parentElement)).toContain("bg-vsc-list-hover");
    fireEvent.click(screen.getByRole("button", { name: "Chiudi Due" }));
    expect(onClose).toHaveBeenCalledWith("b");
  });
});

describe("EditorHeader", () => {
  it("draws a 128px icon, 20px from the top and the left, 12px above the bar", () => {
    render(
      <EditorHeader
        icon={<svg data-testid="icon" width={56} height={56} />}
        title="PHP"
        meta={["8.4", "Installato"]}
        description="Il linguaggio."
        actions={<button type="button">Aggiorna</button>}
      />,
    );
    expect(classes(screen.getByRole("banner"))).toEqual(
      expect.arrayContaining(["pt-vsc-size200", "pb-vsc-size120", "px-vsc-size200", "gap-vsc-size120"]),
    );
    const icon = screen.getByTestId("icon").parentElement;
    expect(classes(icon)).toEqual(
      expect.arrayContaining([
        "size-vsc-extension-icon",
        "[&>svg]:size-full",
        "[&>.codicon]:text-(length:--vsc-size-extension-icon)!",
      ]),
    );
    // The header's text 14px, the name on a 30px line, the facts' bars 14px
    // from each side.
    expect(classes(screen.getByRole("banner"))).toContain("text-(length:--vsc-size-extension-header)");
    expect(classes(screen.getByRole("heading", { name: "PHP" }))).toEqual(
      expect.arrayContaining(["text-vsc-heading1", "leading-(--vsc-size-extension-name)"]),
    );
    expect(classes(screen.getByText("8.4"))).not.toContain("border-l");
    expect(classes(screen.getByText("Installato"))).toEqual(
      expect.arrayContaining(["border-l", "ml-vsc-extension-fact", "pl-vsc-extension-fact"]),
    );
    expect(classes(screen.getByText("Il linguaggio."))).toContain("mt-vsc-size100");
    expect(classes(screen.getByRole("button", { name: "Aggiorna" }).parentElement)).toContain("mt-vsc-size100");
  });
});

describe("NavBar", () => {
  it("is 36px, 20px in, its items 11px uppercase and 10px wide on each side", () => {
    const onSelect = vi.fn();
    render(
      <NavBar
        label="Sezioni"
        items={[
          { id: "a", label: "Panoramica" },
          { id: "b", label: "Versioni" },
        ]}
        active="a"
        onSelect={onSelect}
      />,
    );
    const nav = screen.getByRole("navigation", { name: "Sezioni" });
    // A panelSection.border line under it, as .navbar's.
    expect(classes(nav)).toEqual(
      expect.arrayContaining(["h-vsc-size360", "pl-vsc-size200", "border-b", "border-vsc-panel-section-border"]),
    );
    const on = screen.getByRole("button", { name: "Panoramica" });
    const off = screen.getByRole("button", { name: "Versioni" });
    for (const item of [on, off]) {
      expect(classes(item)).toEqual(expect.arrayContaining(["px-vsc-size100", "text-vsc-body-xsmall", "uppercase"]));
    }
    expect(on.getAttribute("aria-current")).toBe("page");
    expect(classes(on)).toContain("after:bg-vsc-panel-title-border");
    expect(classes(off)).toContain("text-vsc-panel-title-inactive");
    fireEvent.click(off);
    expect(onSelect).toHaveBeenCalledWith("b");
  });
});

describe("Panel", () => {
  it("draws its tabs as VSCodium's modern ones, with the size tokens", () => {
    const onSelect = vi.fn();
    render(
      <Panel
        tabs={[
          { id: "activity", label: "Attività" },
          { id: "output", label: "Output" },
        ]}
        active="activity"
        onSelect={onSelect}
        height={200}
      >
        <p>content</p>
      </Panel>,
    );
    const list = screen.getByRole("tablist");
    expect(classes(list.parentElement)).toEqual(
      expect.arrayContaining(["h-vsc-title", "pl-vsc-size20", "pr-vsc-size40"]),
    );
    const on = screen.getByRole("tab", { name: "Attività" });
    const off = screen.getByRole("tab", { name: "Output" });
    expect(on.getAttribute("aria-selected")).toBe("true");
    // 10px beside the label; the fill a 24px pill 2px in from the sides.
    for (const tab of [on, off]) {
      expect(classes(tab)).toEqual(
        expect.arrayContaining([
          "px-vsc-size100",
          "before:inset-x-vsc-size20",
          "before:inset-y-vsc-size40",
          "before:rounded-vsc-small",
        ]),
      );
    }
    expect(classes(on)).toContain("before:bg-vsc-list-inactive");
    expect(classes(off)).toContain("hover:before:bg-vsc-list-hover");
    // No underline: the modern tabs drop the panelTitle.activeBorder.
    expect(classes(on).some((c) => c.startsWith("after:"))).toBe(false);
    fireEvent.click(off);
    expect(onSelect).toHaveBeenCalledWith("output");
  });
});
