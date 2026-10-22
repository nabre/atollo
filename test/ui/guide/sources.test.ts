import { describe, expect, it } from "vitest";
import catalog from "../../../tauri/catalog/components.json";
import { sourceCounts, TOOL_SOURCES } from "@guide/content/sources";

// The guide's table of sources lists the catalog (D-108): one row per
// component, plus the parts that have their own download (Composer, Maven…).
describe("guide sources table", () => {
  const ids = catalog.components.map((c) => c.id).sort();
  const rows = TOOL_SOURCES.filter((r) => r.group !== "part");

  it("has a row for every catalog component and none for another", () => {
    expect(rows.map((r) => r.id).sort()).toEqual(ids);
    expect(new Set(TOOL_SOURCES.map((r) => r.id)).size).toBe(TOOL_SOURCES.length);
  });

  it("counts its rows instead of a number written in the text", () => {
    expect(sourceCounts()).toEqual({
      components: ids.length,
      parts: TOOL_SOURCES.length - ids.length,
      all: TOOL_SOURCES.length,
    });
  });

  it("names the other source of macOS and Linux where it is not the same", () => {
    for (const id of ["git", "php", "apache", "cc", "ruby", "r", "mongodb", "sqlite"]) {
      expect(TOOL_SOURCES.find((r) => r.id === id)?.unix, id).toBeTruthy();
    }
    expect(TOOL_SOURCES.find((r) => r.id === "go")?.unix).toBeNull();
  });
});
