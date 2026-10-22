// The Sito web form: the name it announces is the one Rust gave the site in
// its pool (name_sites), not `<project>.localhost` written by the page.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { ProjectAction, ProjectInfo } from "@/api";
import type { AttachedSite, WebServersView } from "@/api/servers";
import { SiteForm } from "@/projects/site";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

const SITE: ProjectAction = {
  id: "site",
  label: "sito web Apache (public)",
  command: "",
  type: "web",
  folder: null,
  url: null,
  env: {},
  static: null,
  apache: "public",
  caddy: null,
  server: null,
  domains: [],
  aliases: {},
  custom: true,
};

const PROJECT = { id: "My Shop", name: "My Shop", actions: [SITE] } as unknown as ProjectInfo;

function view(sites: AttachedSite[]): WebServersView {
  return {
    pools: [
      {
        id: "apache-2.4_php-8.4",
        name: "Apache 2.4 · PHP 8.4",
        engine: "apache",
        serverLine: "2.4",
        phpLine: "8.4",
        port: 8081,
        running: false,
        pid: null,
        manual: false,
        sites,
        confDir: "data\\servers\\apache-2.4_php-8.4",
        orphan: false,
        profile: null,
      },
    ],
    apacheLines: ["2.4"],
    caddyLines: [],
    phpLines: ["8.4"],
  };
}

/** The «con» select, once it shows `value` (the component settles after render). */
async function kindSelect(value: string): Promise<HTMLElementTagNameMap["vscode-single-select"]> {
  const kind = screen.getByLabelText<HTMLElementTagNameMap["vscode-single-select"]>("con");
  await waitFor(() => {
    expect(kind.value).toBe(value);
  });
  return kind;
}

describe("SiteForm", () => {
  it("announces the name the site has in its pool", async () => {
    const site: AttachedSite = {
      project: "My Shop",
      action: "site",
      folder: "public",
      serverName: "my-shop-2.localhost",
      domains: [],
      aliases: {},
      url: "http://my-shop-2.localhost:8081",
    };
    invoke.mockImplementation((cmd) =>
      cmd === "servers_list" ? Promise.resolve(view([site])) : Promise.reject(new Error(`unexpected ${cmd}`)),
    );
    render(<SiteForm project={PROJECT} onChanged={vi.fn()} />);
    expect(await screen.findByText(/su http:\/\/my-shop-2\.localhost\./)).toBeTruthy();
    expect(screen.getByText(/predefinito: my-shop-2\.localhost\)/)).toBeTruthy();
  });

  it("before the site is in a pool, the project's label", async () => {
    invoke.mockImplementation((cmd) =>
      cmd === "servers_list" ? Promise.resolve(view([])) : Promise.reject(new Error(`unexpected ${cmd}`)),
    );
    render(<SiteForm project={PROJECT} onChanged={vi.fn()} />);
    expect(await screen.findByText(/predefinito: my-shop\.localhost\)/)).toBeTruthy();
  });

  // The web servers of the kit: Caddy first, then Apache, and no other.
  it("offers Caddy first, then Apache, and no other web server", async () => {
    invoke.mockImplementation((cmd) =>
      cmd === "servers_list" ? Promise.resolve(view([])) : Promise.reject(new Error(`unexpected ${cmd}`)),
    );
    render(<SiteForm project={PROJECT} onChanged={vi.fn()} />);
    const kind = await kindSelect("apache");
    const kinds = [...kind.querySelectorAll("vscode-option")].map((o) => o.value);
    expect(kinds).toEqual(["static", "php", "caddy", "apache"]);
  });

  it("chosen, Caddy is saved as kind caddy, with the note on .htaccess", async () => {
    invoke.mockImplementation((cmd) => {
      if (cmd === "servers_list") return Promise.resolve(view([]));
      if (cmd === "project_site") return Promise.resolve(null);
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    const onChanged = vi.fn();
    render(<SiteForm project={PROJECT} onChanged={onChanged} />);
    // Apache, as saved: no note on .htaccess.
    expect(screen.queryByText(/Caddy non legge \.htaccess/)).toBeNull();
    const kind = await kindSelect("apache");
    fireEvent.change(kind, { target: { value: "caddy" } });
    expect(
      screen.getByText("Caddy non legge .htaccess: le regole di riscrittura si generano dal tipo di progetto."),
    ).toBeTruthy();
    expect(screen.getByText(/Servito da Caddy nel server condiviso/)).toBeTruthy();
    fireEvent.submit(screen.getByRole("form", { name: "Servi come sito web" }));
    await waitFor(() => {
      expect(onChanged).toHaveBeenCalled();
    });
    const call = invoke.mock.calls.find(([cmd]) => cmd === "project_site");
    expect(call?.[1]).toMatchObject({ project: "My Shop", folder: "public", kind: "caddy" });
  });

  it("a site served by Caddy opens with Caddy chosen", async () => {
    invoke.mockImplementation((cmd) =>
      cmd === "servers_list" ? Promise.resolve(view([])) : Promise.reject(new Error(`unexpected ${cmd}`)),
    );
    const caddySite = { ...SITE, apache: null, caddy: "web" };
    render(<SiteForm project={{ ...PROJECT, actions: [caddySite] }} onChanged={vi.fn()} />);
    await kindSelect("caddy");
    expect(screen.getByText(/Caddy non legge \.htaccess/)).toBeTruthy();
    expect(
      screen.getByLabelText<HTMLElementTagNameMap["vscode-textfield"]>("Servi come sito web la cartella").value,
    ).toBe("web");
  });
});
