// The Settings editor as VSCodium 1.135's: the table of contents, the
// groups' titles and the bar of a changed setting, measured with the size
// tokens; the search hides what does not match.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import { SettingItem, SettingsEditor, SettingsGroup } from "../../../ui/src/workbench";

afterEach(cleanup);

/** The classes of an element, as a list. */
const classes = (el: Element | null | undefined) => (el?.getAttribute("class") ?? "").split(/\s+/);

function editor(search = "") {
  return render(
    <SettingsEditor
      toc={[
        { id: "g-a", label: "Rete" },
        { id: "g-b", label: "Editor" },
      ]}
      search={search}
      onSearch={vi.fn()}
      placeholder="Cerca"
    >
      <SettingsGroup id="g-a" title="Rete">
        <SettingItem title="Proxy" description="Il proxy." modified />
      </SettingsGroup>
      <SettingsGroup id="g-b" title="Editor">
        <SettingItem title="Font" description="La dimensione." />
      </SettingsGroup>
    </SettingsEditor>,
  );
}

describe("SettingsEditor", () => {
  it("lists the groups in a 200px table, the current one bold in the header color", () => {
    editor();
    const toc = screen.getByRole("navigation", { name: "Cerca" });
    expect(classes(toc)).toEqual(expect.arrayContaining(["w-vsc-settings-toc", "pl-vsc-size240"]));
    const current = screen.getByRole("button", { name: "Rete" });
    const other = screen.getByRole("button", { name: "Editor" });
    expect(current.getAttribute("aria-current")).toBe("location");
    expect(classes(current)).toEqual(expect.arrayContaining(["h-vsc-row", "font-bold", "text-vsc-settings-header"]));
    expect(classes(other)).toEqual(
      expect.arrayContaining(["h-vsc-row", "text-vsc-fg/90", "hover:text-vsc-settings-header/70"]),
    );
  });

  it("puts the header 14px from the top and the tree 24px in, no line between", () => {
    editor();
    const body = screen.getByRole("navigation", { name: "Cerca" }).parentElement;
    expect(classes(body?.previousElementSibling)).toEqual(
      expect.arrayContaining(["mt-vsc-size100", "pt-vsc-size40", "px-vsc-size240"]),
    );
    expect(classes(body)).toEqual(expect.arrayContaining(["mt-vsc-size100", "pt-vsc-size40"]));
    expect(classes(body)).not.toContain("border-t");
    expect(classes(body?.lastElementChild)).toContain("px-vsc-size240");
  });

  it("titles a group at 26px, 10px around and 15px on the left", () => {
    editor();
    const title = screen.getByRole("heading", { name: "Rete" });
    expect(classes(title)).toEqual(
      expect.arrayContaining(["text-vsc-heading1", "font-semibold", "p-vsc-size100", "pl-vsc-settings-title"]),
    );
  });

  it("marks a changed setting with a 2px bar, 5px in, 15px under the top and 18px over the bottom", () => {
    editor();
    const [changed, plain] = document.querySelectorAll("[data-setting]");
    const bar = changed?.querySelector(":scope > span[aria-hidden]");
    expect(classes(bar)).toEqual(
      expect.arrayContaining([
        "absolute",
        "w-vsc-size20",
        "left-vsc-modified-left",
        "top-vsc-modified-top",
        "bottom-vsc-modified-bottom",
        "bg-vsc-settings-modified",
      ]),
    );
    expect(plain?.querySelector(":scope > span[aria-hidden]")).toBeNull();
    // No border on the whole item: the bar stops short of its ends.
    for (const item of [changed, plain]) {
      expect(classes(item).some((c) => c.startsWith("border-l"))).toBe(false);
      expect(classes(item)).toContain("rounded-vsc-medium");
    }
  });

  it("hides the settings and the groups the search does not find", () => {
    editor("proxy");
    expect(document.querySelectorAll("[data-setting]")).toHaveLength(1);
    expect(screen.getByText("Proxy")).toBeTruthy();
    expect(screen.queryByText("Font")).toBeNull();
  });
});
