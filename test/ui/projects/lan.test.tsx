// A project's site from the phone (N-6): the QR drawn from the app's
// modules, shown on demand, and nothing when «Rete locale» is off.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { LanLink } from "@/api";
import { LanAddress, QrCode, useLanLink } from "@/projects/lan";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

// 3 × 3: a dark corner and the centre.
const LINK: LanLink = { url: "http://192.168.1.20:7100/", qr: { width: 3, modules: "100010000" } };

describe("QrCode", () => {
  it("draws the dark modules inside the quiet zone, dark on white", () => {
    render(<QrCode qr={LINK.qr} label="QR" />);
    const svg = screen.getByRole("img", { name: "QR" });
    expect(svg.getAttribute("viewBox")).toBe("0 0 11 11");
    expect(svg.querySelector("rect")?.getAttribute("fill")).toBe("#ffffff");
    expect(svg.querySelector("path")?.getAttribute("d")).toBe("M4 4h1v1h-1zM5 5h1v1h-1z");
  });
});

describe("LanAddress", () => {
  it("shows the address and the QR on demand", () => {
    render(<LanAddress link={LINK} />);
    expect(screen.getByText("http://192.168.1.20:7100/")).toBeTruthy();
    expect(screen.queryByRole("img")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Mostra il QR" }));
    expect(screen.getByRole("img", { name: "QR di http://192.168.1.20:7100/" })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Nascondi il QR" }));
    expect(screen.queryByRole("img")).toBeNull();
  });
});

function Probe({ url }: { url: string | null }) {
  const link = useLanLink(url);
  return <span>{link ? link.url : "none"}</span>;
}

describe("useLanLink", () => {
  it("asks the app for a running site only", async () => {
    invoke.mockResolvedValue(LINK);
    const { rerender } = render(<Probe url={null} />);
    expect(screen.getByText("none")).toBeTruthy();
    expect(invoke).not.toHaveBeenCalled();
    rerender(<Probe url="http://localhost:7100/" />);
    expect(await screen.findByText("http://192.168.1.20:7100/")).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith("lan_link", { url: "http://localhost:7100/" });
    // Stopped: nothing, at once.
    rerender(<Probe url={null} />);
    await waitFor(() => {
      expect(screen.getByText("none")).toBeTruthy();
    });
  });

  it("shows nothing when «Rete locale» is off", async () => {
    invoke.mockResolvedValue(null);
    render(<Probe url="http://localhost:7100/" />);
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("lan_link", { url: "http://localhost:7100/" });
    });
    expect(screen.getByText("none")).toBeTruthy();
  });
});
