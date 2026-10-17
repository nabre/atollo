// The Sito web form: the name it announces is the one Rust gave the site in
// its pool (name_sites), not `<project>.localhost` written by the page.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import type { ProjectAction, ProjectInfo } from "../../../ui/src/api";
import type { AttachedSite, WebServersView } from "../../../ui/src/api/servers";
import { SiteForm } from "../../../ui/src/projects/site";

const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: unknown) => invoke(cmd, args),
  isTauri: () => true,
}));

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
  nginx: null,
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
    nginxLines: [],
    phpLines: ["8.4"],
  };
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
});
