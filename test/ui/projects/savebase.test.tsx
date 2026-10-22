// «Salva come base» (N-10): what would be copied, the entries left out,
// the name and the description sent, then where to find it.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { BaseEntry } from "@/api";
import { SaveBaseDialog } from "@/projects/savebase";
import { basesByGroup, categoryLabel } from "@/frameworks/model";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

const ENTRIES: BaseEntry[] = [
  { name: "public", dir: true, files: 8, bytes: 2048 },
  { name: "src", dir: true, files: 42, bytes: 4096 },
  { name: "composer.json", dir: false, files: 1, bytes: 1024 },
];

describe("SaveBaseDialog", () => {
  it("saves the project's files but those left out", async () => {
    invoke.mockImplementation((c) =>
      Promise.resolve(c === "project_base_preview" ? ENTRIES : c === "project_save_base" ? "mine-shop" : undefined),
    );
    render(<SaveBaseDialog project="shop" onClose={vi.fn()} />);
    expect(await screen.findByText("public/")).toBeTruthy();
    expect(screen.getByText("In tutto: 51 file, 7 KB")).toBeTruthy();
    fireEvent.click(screen.getByText("public/"));
    expect(screen.getByText("In tutto: 43 file, 5 KB")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Salva come base" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("project_save_base", {
        project: "shop",
        name: "shop",
        desc: "",
        skip: ["public"],
      });
    });
    expect(await screen.findByText(/Le mie basi/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Salva come base" })).toBeNull();
  });
});

describe("Le mie basi", () => {
  it("is the first group of the bases", () => {
    const base = (id: string, category: string) =>
      ({ id, kind: "base", name: id, category, create: [], steps: [], actions: [] }) as never;
    const groups = basesByGroup([base("laravel", "web-php"), base("mine-shop", "mine")]);
    expect(groups.map((g) => g.id)).toEqual(["mine", "web"]);
    expect(groups[0]?.label).toBe("Le mie basi");
    expect(categoryLabel("mine")).toBe("Le mie basi");
  });
});
