// The choice of the Tools page follows the needs the backend sends for the
// kit's target (C and C++ need Zig on macOS and Linux only), and the Xcode
// notice is read from the row of C and C++.
import { describe, expect, it } from "vitest";
import catalog from "../../../tauri/catalog/components.json";
import type { ComponentInfo, Group } from "@/ipc";
import { KIND_OF, KIND_OF_GROUP, KINDS, groups, kindOf, toggle, xcodeMissing } from "@/components/model";

function row(id: string, needs: string[] = [], more: Partial<ComponentInfo> = {}): ComponentInfo {
  return {
    id,
    name: id,
    group: "languages",
    desc: "",
    mb: 1,
    chosen: false,
    installed: false,
    present: false,
    locked: false,
    version: null,
    available: true,
    unavailableReason: null,
    needs,
    versioned: false,
    home: "https://example.org",
    ...more,
  };
}

describe("toggle", () => {
  it("follows the needs of the kit's target both ways", () => {
    // A Linux kit: the backend says C and C++ need Zig.
    const linux = [row("zig"), row("cc", ["zig"])];
    const on = toggle(linux, new Set(), "cc", true);
    expect([...on].sort()).toEqual(["cc", "zig"]);
    expect([...toggle(linux, on, "zig", false)]).toEqual([]);
    // A Windows x64 kit: no need, Zig stays as it is.
    const windows = [row("zig"), row("cc")];
    expect([...toggle(windows, new Set(), "cc", true)]).toEqual(["cc"]);
    expect([...toggle(windows, new Set(["cc", "zig"]), "zig", false)]).toEqual(["cc"]);
  });
});

describe("xcodeMissing", () => {
  it("is true only when the backend says so", () => {
    expect(xcodeMissing(row("cc"))).toBe(false);
    expect(xcodeMissing({ ...row("cc"), xcodeMissing: false })).toBe(false);
    expect(xcodeMissing({ ...row("cc"), xcodeMissing: true })).toBe(true);
  });
});

// The Tools page, the first-start page and the matrix group the tools as
// the site does (site/matrix.mjs): same kinds, same exceptions.
describe("kinds", () => {
  it("are the site's", async () => {
    // @ts-expect-error -- the site's plain JavaScript module, typed by the cast.
    const site = (await import("../../../site/matrix.mjs")) as {
      KINDS: string[];
      KIND_OF: Record<string, string>;
      KIND_OF_GROUP: Record<string, string>;
    };
    expect([...KINDS]).toEqual(site.KINDS);
    expect(KIND_OF).toEqual(site.KIND_OF);
    expect(KIND_OF_GROUP).toEqual(site.KIND_OF_GROUP);
  });

  it("put every catalog component in exactly one section", () => {
    const sections = groups();
    for (const c of catalog.components) {
      const info = { ...c, group: c.group as Group } as unknown as ComponentInfo;
      expect(
        sections.filter((g) => g.has(info)).map((g) => g.id),
        c.id,
      ).toHaveLength(1);
    }
    expect(kindOf({ id: "git", group: "languages" })).toBe("system");
    expect(kindOf({ id: "caddy", group: "languages" })).toBe("web");
    expect(kindOf({ id: "rust", group: "languages" })).toBe("languages");
  });

  // D-147: «Database» holds the servers only; the tools that show them in
  // the browser have their own section right after it.
  it("put the database viewers after the servers", () => {
    const ids = (kind: string) =>
      catalog.components
        .filter((c) => kindOf({ id: c.id, group: c.group as Group }) === kind)
        .map((c) => c.id)
        .sort();
    expect(ids("databases")).toEqual(["mongodb", "mysql", "sqlite"]);
    expect(ids("dbviewers")).toEqual(["mongo-express", "phpmyadmin"]);
    expect(KINDS.indexOf("dbviewers")).toBe(KINDS.indexOf("databases") + 1);
    expect(groups().find((g) => g.id === "dbviewers")?.title).toBe("Visualizzazione database");
  });
});
