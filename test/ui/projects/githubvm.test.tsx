// «GitHub nella VM» (D-172): off unless turned on for the project, since
// its code could then act as the user on GitHub; a click saves the choice.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { GithubInVm } from "@/projects/git";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

describe("GithubInVm", () => {
  it("is off by default, and a click gives the project's VM the access", async () => {
    invoke.mockImplementation((cmd: string) => Promise.resolve(cmd === "project_github_vm" ? false : undefined));
    render(<GithubInVm project="shop" />);
    const box = await screen.findByRole("checkbox");
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("project_github_vm", { project: "shop" });
    });
    expect((box as HTMLInputElement).checked).toBe(false);
    expect(screen.getByText(/accendilo solo per i progetti di cui ti fidi/)).toBeTruthy();
    fireEvent.click(box);
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("project_github_vm_set", { project: "shop", on: true });
    });
  });
});
