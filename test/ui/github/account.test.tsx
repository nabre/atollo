// The kit's GitHub account in Settings: a status that cannot be read says
// why, with «Riprova», instead of loading for ever.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import type { GithubStatus } from "@/api";
import { GithubAccount } from "@/github/account";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));
vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => undefined),
}));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

const SIGNED_OUT: GithubStatus = {
  installed: true,
  signedIn: false,
  login: null,
  signingIn: false,
  code: null,
  error: null,
};

describe("GithubAccount", () => {
  it("shows a failed read with «Riprova», and the account once it is read", async () => {
    invoke.mockRejectedValueOnce("gh not answering");
    invoke.mockResolvedValue(SIGNED_OUT);
    render(<GithubAccount />);
    expect(await screen.findByText("gh not answering")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Riprova" }));
    expect(await screen.findByText("Account GitHub del kit")).toBeTruthy();
    expect(screen.queryByText("gh not answering")).toBeNull();
  });
});
