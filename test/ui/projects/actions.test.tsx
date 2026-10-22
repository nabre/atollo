// The actions of Avvio written in panel.json (D-78): «Nuova azione» opens a
// form that saves the action, «Modifica» fills it with the chosen one (a
// detected action keeps its id: it is overridden), «Togli» asks first and
// only for an action of panel.json; the backend's refusal is shown.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import type { ProjectAction, ProjectInfo } from "@/api";
import { ProjectPage } from "@/projects/page";
import { actionId } from "@/projects/actionform";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

function action(over: Partial<ProjectAction>): ProjectAction {
  return {
    id: "npm:dev",
    label: "npm run dev",
    command: "npm run dev -- --port {port}",
    type: "web",
    folder: null,
    url: null,
    env: {},
    static: null,
    apache: null,
    caddy: null,
    custom: false,
    ...over,
  };
}

const DETECTED = action({ env: { NODE_ENV: "development" }, folder: "web" });
const CUSTOM = action({ id: "worker", label: "Worker", command: "node worker.js", type: "command", custom: true });

const PROJECT = {
  id: "shop",
  name: "shop",
  modified: 0,
  empty: false,
  missing: [],
  platform: "Vite",
  type: "web",
  reason: "package.json",
  languages: [],
  requires: [],
  actions: [DETECTED, CUSTOM],
  warnings: [],
  custom: true,
  default: "npm:dev",
  settings: {},
  extensions: [],
} satisfies ProjectInfo;

/** The page on Avvio; `save` answers project_action_save. */
function page(save: () => Promise<unknown> = () => Promise.resolve(null)) {
  invoke.mockImplementation((cmd) => {
    if (cmd === "project_action_save") return save();
    if (cmd === "project_action_remove") return Promise.resolve(null);
    if (cmd === "project_status") return Promise.resolve({ git: null, instructions: [] });
    if (cmd === "project_databases") return Promise.resolve(null);
    return Promise.reject(new Error(`unexpected ${cmd}`));
  });
  const onChanged = vi.fn();
  render(
    <ProjectPage
      p={PROJECT}
      agents={null}
      editor={null}
      runs={{}}
      logs={{}}
      ports={{ ports: {}, locked: [] }}
      busy={null}
      onStart={vi.fn()}
      onStop={vi.fn()}
      onPorts={vi.fn()}
      onShowLog={vi.fn()}
      onForget={vi.fn()}
      onChanged={onChanged}
    />,
  );
  return onChanged;
}

/** Types `value` into the field named `name` (a vscode-textfield or -textarea). */
function fill(form: HTMLElement, name: string, value: string) {
  fireEvent.input(within(form).getByLabelText(name), { target: { value } });
}

function field(form: HTMLElement, name: string): string {
  return within(form).getByLabelText<HTMLInputElement>(name).value;
}

function saved(): unknown {
  return invoke.mock.calls.find(([cmd]) => cmd === "project_action_save")?.[1];
}

describe("actions of Avvio", () => {
  it("derives the id from the name, unique in the project", () => {
    expect(actionId("Server di sviluppo", [])).toBe("server-di-sviluppo");
    expect(actionId("  Più Veloce!! ", [])).toBe("piu-veloce");
    expect(actionId("Worker", ["worker"])).toBe("worker-2");
    expect(actionId("!!!", [])).toBe("azione");
    expect(actionId("x".repeat(60), []).length).toBe(40);
  });

  it("«Nuova azione» saves the action of the form and reads the project again", async () => {
    const onChanged = page();
    fireEvent.click(screen.getByRole("button", { name: "Nuova azione" }));
    const form = screen.getByRole("form", { name: "Nuova azione" });
    fill(form, "Nome", "Server API");
    // The select, once it shows its value (the component settles after render).
    const kind = within(form).getByLabelText<HTMLElementTagNameMap["vscode-single-select"]>("Tipo");
    await waitFor(() => {
      expect(kind.value).toBe("web");
    });
    fireEvent.change(kind, { target: { value: "command" } });
    fill(form, "Comando", "  python -m api --port {port} ");
    fill(form, "Cartella", "api");
    fill(form, "Variabili", "DEBUG=1\n\nAPI_KEY=a=b");
    expect(within(form).getByText("Id: server-api")).toBeTruthy();
    // Indirizzo is only for a web action.
    expect(within(form).queryByLabelText("Indirizzo")).toBeNull();
    fireEvent.submit(form);
    await waitFor(() => {
      expect(onChanged).toHaveBeenCalled();
    });
    expect(saved()).toEqual({
      project: "shop",
      action: {
        id: "server-api",
        label: "Server API",
        command: "python -m api --port {port}",
        kind: "command",
        folder: "api",
        url: null,
        env: { DEBUG: "1", API_KEY: "a=b" },
      },
    });
    expect(screen.queryByRole("form", { name: "Nuova azione" })).toBeNull();
  });

  it("«Annulla» closes the form without saving", () => {
    page();
    fireEvent.click(screen.getByRole("button", { name: "Nuova azione" }));
    fireEvent.click(screen.getByRole("button", { name: "Annulla" }));
    expect(screen.queryByRole("form", { name: "Nuova azione" })).toBeNull();
    expect(saved()).toBeUndefined();
  });

  it("«Modifica» of a detected action fills the form and saves an override with its id", async () => {
    const onChanged = page();
    // The default action is the one shown; detected, it has no «Togli».
    const detail = screen.getByRole("region", { name: "npm run dev" });
    expect(within(detail).queryByRole("button", { name: "Togli" })).toBeNull();
    fireEvent.click(within(detail).getByRole("button", { name: "Modifica" }));
    const form = screen.getByRole("form", { name: "Modifica npm run dev" });
    expect(field(form, "Nome")).toBe("npm run dev");
    expect(field(form, "Comando")).toBe("npm run dev -- --port {port}");
    expect(field(form, "Cartella")).toBe("web");
    expect(field(form, "Variabili")).toBe("NODE_ENV=development");
    expect(within(form).getByText(/Azione rilevata/)).toBeTruthy();
    fill(form, "Nome", "Vite");
    fill(form, "Indirizzo", "http://localhost:{port}/app");
    fireEvent.submit(form);
    await waitFor(() => {
      expect(onChanged).toHaveBeenCalled();
    });
    expect(saved()).toEqual({
      project: "shop",
      action: {
        id: "npm:dev",
        label: "Vite",
        command: "npm run dev -- --port {port}",
        kind: "web",
        folder: "web",
        url: "http://localhost:{port}/app",
        env: { NODE_ENV: "development" },
      },
    });
  });

  it("«Togli» of an action of panel.json asks first, then removes it", async () => {
    const onChanged = page();
    const list = screen.getByRole("list", { name: "Azioni di shop" });
    fireEvent.click(within(list).getByRole("button", { name: /^Worker/ }));
    const detail = screen.getByRole("region", { name: "Worker" });
    fireEvent.click(within(detail).getByRole("button", { name: "Togli" }));
    const ask = screen.getByRole("alertdialog", { name: "Togliere Worker da panel.json?" });
    expect(invoke.mock.calls.some(([cmd]) => cmd === "project_action_remove")).toBe(false);
    fireEvent.click(within(ask).getByRole("button", { name: "Togli" }));
    await waitFor(() => {
      expect(onChanged).toHaveBeenCalled();
    });
    expect(invoke).toHaveBeenCalledWith("project_action_remove", { project: "shop", id: "worker" });
    expect(screen.queryByRole("alertdialog")).toBeNull();
  });

  it("shows the backend's refusal in the form, which stays open", async () => {
    const onChanged = page(() => Promise.reject(new Error("panel.json non è un JSON valido: correggilo prima.")));
    fireEvent.click(screen.getByRole("button", { name: "Nuova azione" }));
    const form = screen.getByRole("form", { name: "Nuova azione" });
    fill(form, "Nome", "Build");
    fill(form, "Comando", "npm run build");
    fireEvent.submit(form);
    expect((await within(form).findByRole("alert")).textContent).toContain(
      "panel.json non è un JSON valido: correggilo prima.",
    );
    expect(onChanged).not.toHaveBeenCalled();
    expect(screen.getByRole("form", { name: "Nuova azione" })).toBeTruthy();
  });

  it("a line of the variables without a name is not sent", () => {
    page();
    fireEvent.click(screen.getByRole("button", { name: "Nuova azione" }));
    const form = screen.getByRole("form", { name: "Nuova azione" });
    fill(form, "Nome", "Build");
    fill(form, "Comando", "npm run build");
    fill(form, "Variabili", "DEBUG=1\nsenza uguale");
    fireEvent.submit(form);
    expect(within(form).getByRole("alert").textContent).toContain("Riga 2 delle variabili");
    expect(saved()).toBeUndefined();
  });
});
