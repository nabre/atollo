// The ways to make a project from a base, in four groups (Web, CMS, App,
// Altro): the quick panel's blocks and the side bar's tree.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import type { FrameworkPiece } from "@/api";
import { basesByGroup, type Frameworks } from "@/frameworks";
import { ProjectsHome, type HomeHandlers } from "@/projects/home";
import { NewProjectTree } from "@/projects/new";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.reject(new Error("no backend")),
  isTauri: () => true,
}));

afterEach(cleanup);

const base = (id: string, name: string, category?: string): FrameworkPiece => ({
  id,
  kind: "base",
  name,
  desc: `${name}, a base`,
  home: "",
  ...(category ? { category } : {}),
});

const PIECES: FrameworkPiece[] = [
  base("laravel", "Laravel", "web-php"),
  base("vite", "Vite", "web-js"),
  base("docusaurus", "Docusaurus", "docs"),
  base("wordpress", "WordPress", "cms"),
  base("drupal", "Drupal", "cms"),
  base("electron", "Electron", "desktop"),
  base("android", "Android", "mobile"),
  base("jupyter", "Jupyter", "data"),
  base("mine", "Mine"),
  { id: "react", kind: "addon", name: "React", desc: "", home: "", on: { vite: {} } },
];

const fw = { view: null, pieces: PIECES } as unknown as Frameworks;

const names = (list: FrameworkPiece[]) => list.map((b) => b.id);

describe("basesByGroup", () => {
  it("puts the categories in Web, CMS, App and Altro, the empty groups left out", () => {
    const groups = basesByGroup(PIECES);
    expect(groups.map((g) => g.id)).toEqual(["web", "cms", "app", "other"]);
    const [web, cms, app, other] = groups;
    expect(names(web?.bases ?? [])).toEqual(["laravel", "vite", "docusaurus"]);
    expect(web?.categories.map((c) => c.id)).toEqual(["web-php", "web-js", "docs"]);
    expect(names(cms?.bases ?? [])).toEqual(["wordpress", "drupal"]);
    expect(names(app?.bases ?? [])).toEqual(["electron", "android"]);
    // A base made in Settings without a category is in «Altro».
    expect(names(other?.bases ?? [])).toEqual(["jupyter", "mine"]);
    expect(basesByGroup(PIECES.filter((p) => p.category === "cms")).map((g) => g.id)).toEqual(["cms"]);
  });
});

function home() {
  const handlers = {
    onNew: vi.fn(),
    onGithub: vi.fn(),
    onFolder: vi.fn(),
  } as unknown as HomeHandlers;
  render(<ProjectsHome projects={[]} runs={{}} busy={null} status={null} agents={null} fw={fw} handlers={handlers} />);
  return handlers;
}

describe("the quick panel's «Nuovo progetto»", () => {
  it("shows a block per group, its bases as buttons that open the wizard on them", () => {
    const handlers = home();
    const section = within(screen.getByRole("region", { name: "Nuovo progetto" }));
    const web = within(section.getByRole("region", { name: /^Web/ }));
    const cms = within(section.getByRole("region", { name: /^CMS/ }));
    const app = within(section.getByRole("region", { name: /^App/ }));
    expect(section.getByRole("region", { name: /^Altro/ })).toBeTruthy();
    // Web holds no CMS; App holds desktop and mobile, each under its heading.
    expect(web.queryByRole("button", { name: /WordPress/ })).toBeNull();
    expect(web.getByRole("group", { name: "Siti statici e documentazione" })).toBeTruthy();
    expect(
      within(app.getByRole("group", { name: "App desktop" })).getByRole("button", { name: /Electron/ }),
    ).toBeTruthy();
    expect(
      within(app.getByRole("group", { name: "App mobile" })).getByRole("button", { name: /Android/ }),
    ).toBeTruthy();
    // One category: its bases with no heading.
    expect(cms.queryByRole("group")).toBeNull();
    fireEvent.click(cms.getByRole("button", { name: /WordPress/ }));
    expect(handlers.onNew).toHaveBeenCalledWith("framework", "wordpress");
  });

  it("keeps the empty project and the Git address as two small links", () => {
    const handlers = home();
    fireEvent.click(screen.getByRole("button", { name: /Progetto vuoto/ }));
    expect(handlers.onNew).toHaveBeenCalledWith("empty");
    fireEvent.click(screen.getByRole("button", { name: /Da un indirizzo Git/ }));
    expect(handlers.onNew).toHaveBeenCalledWith("git");
  });
});

describe("NewProjectTree", () => {
  it("has the two ways, then the four groups with their counts", () => {
    render(<NewProjectTree fw={fw} way={null} base="" onSelect={vi.fn()} />);
    const tree = screen.getByRole("list", { name: "Nuovo progetto" });
    const rows = within(tree)
      .getAllByRole("button")
      .map((b) => b.textContent);
    expect(rows).toEqual(["Progetto vuoto", "Da un indirizzo Git", "Web3", "CMS2", "App2", "Altro2"]);
  });

  it("opens the branch holding the base shown, CMS straight on its bases", () => {
    const onSelect = vi.fn();
    const { rerender } = render(<NewProjectTree fw={fw} way="framework" base="vite" onSelect={onSelect} />);
    const tree = within(screen.getByRole("list", { name: "Nuovo progetto" }));
    expect(tree.getByRole("button", { name: /^Web\d+$/ }).getAttribute("aria-expanded")).toBe("true");
    expect(tree.getByRole("button", { name: /Web con JavaScript/ }).getAttribute("aria-expanded")).toBe("true");
    expect(tree.getByRole("button", { name: /Web con PHP/ }).getAttribute("aria-expanded")).toBe("false");
    expect(tree.getByRole("button", { name: "Vite" }).getAttribute("aria-current")).toBe("page");
    // A closed group: a click opens the wizard on its first base.
    fireEvent.click(tree.getByRole("button", { name: /^CMS\d+$/ }));
    expect(onSelect).toHaveBeenCalledWith("framework", "wordpress");
    rerender(<NewProjectTree fw={fw} way="framework" base="wordpress" onSelect={onSelect} />);
    expect(tree.getByRole("button", { name: "WordPress" }).getAttribute("aria-current")).toBe("page");
    // The group shown: a click closes it, the group then selected.
    fireEvent.click(tree.getByRole("button", { name: /^CMS\d+$/ }));
    expect(tree.queryByRole("button", { name: "WordPress" })).toBeNull();
    expect(tree.getByRole("button", { name: /^CMS\d+$/ }).getAttribute("aria-current")).toBe("page");
    // Each branch remembers being opened by hand.
    fireEvent.click(tree.getByRole("button", { name: /^App\d+$/ }));
    rerender(<NewProjectTree fw={fw} way="framework" base="electron" onSelect={onSelect} />);
    fireEvent.click(tree.getByRole("button", { name: /App mobile/ }));
    expect(tree.getByRole("button", { name: "Android" })).toBeTruthy();
    expect(tree.queryByRole("button", { name: "WordPress" })).toBeNull();
  });
});
