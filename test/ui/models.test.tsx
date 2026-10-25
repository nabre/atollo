// The local models of the AI page (D-151, D-165): what fits the PC, a
// download with its progress and Pause, FAT32 refusing a file over 4 GB,
// Pi's model, the server's state after «Avvia».
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { SAMPLE_LLM, SAMPLE_MODELS, type LlmInfo, type ModelsView, type PullEvent } from "@/ipc/llm";
import { AiPage } from "@/ai";
import { fitOf } from "@/models";
import { invoke } from "@test/support/tauri";

const handlers = new Map<string, (e: { payload: unknown }) => void>();
vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));
vi.mock("@tauri-apps/api/event", () => ({
  listen: (event: string, h: (e: { payload: unknown }) => void) => {
    handlers.set(event, h);
    return Promise.resolve(() => handlers.delete(event));
  },
}));

afterEach(() => {
  cleanup();
  invoke.mockReset();
  handlers.clear();
});

function answer(view: ModelsView) {
  invoke.mockImplementation((cmd: string) => {
    switch (cmd) {
      case "llm_status":
        return Promise.resolve(SAMPLE_LLM);
      case "llm_models":
        return Promise.resolve(view);
      case "llm_pull":
        return new Promise(() => undefined);
      default:
        return Promise.resolve(null);
    }
  });
}

const GB = 1e9;
const GIB = 1024 ** 3;
const hw = (ram: number, gpu: number) => ({
  ram: ram * GIB,
  gpus: gpu ? [{ name: "GPU", memory: gpu * GIB }] : [],
});

describe("fitOf", () => {
  it("compares with the larger of the GPU and the RAM", () => {
    expect(fitOf(7, hw(16, 8))).toBe("ok");
    expect(fitOf(15, hw(16, 8))).toBe("tight");
    expect(fitOf(22, hw(16, 8))).toBe("no");
    expect(fitOf(22, hw(32, 0))).toBe("ok");
    expect(fitOf(7, { ram: null, gpus: [] })).toBe("unknown");
  });
});

describe("AiPage, local models", () => {
  it("shows the catalog with what fits, and a download's progress with Pause", async () => {
    answer({ ...SAMPLE_MODELS, local: [] });
    render(<AiPage focus="models" />);
    const row = (await screen.findByText("gpt-oss 20B", { exact: false })).closest("li");
    expect(row).not.toBeNull();
    const r = within(row as HTMLElement);
    expect(r.getByText("entra")).toBeTruthy();
    fireEvent.click(r.getByText("Scarica"));
    expect(invoke).toHaveBeenCalledWith("llm_pull", { catalog: "gpt-oss-20b" });
    const event: PullEvent = {
      key: "ggml-org--gpt-oss-20b-GGUF/gpt-oss-20b",
      done: 6 * GB,
      total: 12 * GB,
      state: "running",
      error: null,
    };
    act(() => {
      handlers.get("llm-pull")?.({ payload: event });
    });
    expect(r.getByRole("meter").getAttribute("aria-valuenow")).toBe("50");
    fireEvent.click(r.getByText("Sospendi"));
    expect(invoke).toHaveBeenCalledWith("llm_pull_cancel", { key: event.key });
  });

  it("on FAT32 refuses the models with a file over 4 GB and says why", async () => {
    const big = { name: "m.gguf", sha256: "0".repeat(64), bytes: 12 * GB };
    answer({
      ...SAMPLE_MODELS,
      fs: "FAT32",
      local: [],
      catalog: SAMPLE_MODELS.catalog.map((c) => ({ ...c, files: [big] })),
    });
    render(<AiPage focus="models" />);
    expect(await screen.findByText(/non tiene file oltre i 4 GB/)).toBeTruthy();
    expect(screen.getAllByText("non entra su FAT32").length).toBe(SAMPLE_MODELS.catalog.length);
    expect(screen.queryByText("Scarica")).toBeNull();
  });

  it("a download left halfway resumes, a model in the kit can be Pi's", async () => {
    answer(SAMPLE_MODELS);
    render(<AiPage focus="models" />);
    expect(await screen.findByText("Riprendi")).toBeTruthy();
    fireEvent.click(screen.getByText("Riprendi"));
    expect(invoke).toHaveBeenCalledWith("llm_pull", { catalog: "gpt-oss-20b" });
    expect(screen.getByText("Pi parte con")).toBeTruthy();
  });

  it("a status read from before «Avvia» does not undo the start", async () => {
    const stopped: LlmInfo = { ...SAMPLE_LLM, state: "stopped", loaded: [] };
    let state = stopped;
    let held: ((v: unknown) => void) | null = null;
    let hold = false;
    invoke.mockImplementation((cmd: string) => {
      switch (cmd) {
        case "llm_status":
          if (hold) {
            hold = false;
            return new Promise((resolve) => {
              held = resolve;
            });
          }
          return Promise.resolve(state);
        case "llm_models":
          return Promise.resolve(SAMPLE_MODELS);
        case "llm_start":
          state = SAMPLE_LLM;
          return Promise.resolve(SAMPLE_LLM);
        default:
          return Promise.resolve(null);
      }
    });
    render(<AiPage focus="models" />);
    await screen.findAllByText("Avvia");
    // A read of the status starts, and answers only after the start.
    hold = true;
    fireEvent.click(screen.getByRole("button", { name: "Aggiorna i modelli" }));
    fireEvent.click(screen.getAllByText("Avvia")[0] as HTMLElement);
    expect((await screen.findAllByText("Ferma")).length).toBeGreaterThan(0);
    await act(async () => {
      held?.(stopped);
      await Promise.resolve();
    });
    expect(screen.queryAllByText("Avvia")).toHaveLength(0);
  });
});
