// The choice of the Tools page follows the needs the backend sends for the
// kit's target (C and C++ need Zig on macOS and Linux only), and the Xcode
// notice is read from the row of C and C++.
import { describe, expect, it } from "vitest";
import type { ComponentInfo } from "../../../ui/src/api";
import { toggle, xcodeMissing } from "../../../ui/src/components/model";

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
    x64Only: false,
    noArm64: false,
    unavailableReason: null,
    needs,
    needsWindows11: false,
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
    expect(xcodeMissing({ ...row("cc"), xcodeMissing: false } as ComponentInfo)).toBe(false);
    expect(xcodeMissing({ ...row("cc"), xcodeMissing: true } as ComponentInfo)).toBe(true);
  });
});
