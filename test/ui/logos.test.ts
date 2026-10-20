import { describe, expect, it } from "vitest";
import catalog from "../../tauri/catalog/components.json";
import { hasLogo } from "../../ui/src/logos";

// Every component of the catalog shows its logo in Tools, the matrix and
// the pages that list it (MySQL had none).
describe("component logos", () => {
  it("has a logo for every catalog component", () => {
    const missing = catalog.components.map((c) => c.id).filter((id) => !hasLogo(id));
    expect(missing).toEqual([]);
  });
});
