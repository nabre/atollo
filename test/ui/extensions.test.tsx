// A project's extensions: one added by its id, picked from the
// suggestions of the kit's matrix (those the project does not have yet).
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { SAMPLE_MATRIX, SAMPLE_PROJECTS, type ProjectInfo } from "@/api";
import { ProjectExtensions } from "@/extensions";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

// The preview's static site: Prettier, Tailwind CSS and REST Client.
const [SITE] = SAMPLE_PROJECTS as [ProjectInfo, ...ProjectInfo[]];

describe("project extensions", () => {
  it("suggests the matrix's extensions the project lacks and adds the one taken", async () => {
    invoke.mockImplementation((cmd) => Promise.resolve(cmd === "extensions_matrix" ? SAMPLE_MATRIX : null));
    render(<ProjectExtensions project={SITE} onChanged={() => undefined} />);
    const host = screen.getByLabelText<HTMLElementTagNameMap["vscode-textfield"]>("Estensione da aggiungere");
    await host.updateComplete;
    const input = host.wrappedElement;
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("extensions_matrix", undefined);
    });
    act(() => {
      input.value = "git";
      input.dispatchEvent(new Event("input", { bubbles: true, composed: true }));
    });
    const options = await screen.findAllByRole("option");
    expect(options.map((o) => o.textContent)).toEqual(["mhutchie.git-graphGit Graph", "eamodio.gitlensGitLens"]);
    // Prettier is the project's already: never suggested.
    act(() => {
      input.value = "prettier";
      input.dispatchEvent(new Event("input", { bubbles: true, composed: true }));
    });
    expect(screen.queryByRole("listbox")).toBeNull();
    act(() => {
      input.value = "git";
      input.dispatchEvent(new Event("input", { bubbles: true, composed: true }));
    });
    fireEvent.keyDown(input, { key: "ArrowDown" });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(invoke).not.toHaveBeenCalledWith("project_extensions", expect.anything());
    expect(host.value).toBe("mhutchie.git-graph");
    fireEvent.click(screen.getByRole("button", { name: "Aggiungi" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("project_extensions", {
        project: SITE.id,
        added: ["mhutchie.git-graph"],
        removed: [],
      });
    });
  });
});
