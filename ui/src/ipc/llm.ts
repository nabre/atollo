// The local models (tauri/src/llm/, D-151): the server, the catalog and the
// models on disk, downloads from Hugging Face. Rust:
// tauri/src/ipc/llm.rs.

import { act, engineText, query, subscribe } from "./core";
import { LLM_PULL } from "./events";
import type { ServiceState } from "./services";

export interface LlmInfo {
  installed: boolean;
  state: ServiceState;
  port: number;
  /** http://127.0.0.1:<port>: the chat in the browser and the API. */
  url: string;
  error: string | null;
  /** The models in memory now. */
  loaded: string[];
  autostart: boolean;
  /** The model Pi starts with, when the user chose one. */
  piModel: string | null;
}

export interface ModelFile {
  name: string;
  sha256: string;
  bytes: number;
}

export interface CatalogModel {
  id: string;
  name: string;
  repo: string;
  commit: string;
  files: ModelFile[];
  quant: string;
  ctx: number;
  /** The memory it needs with that context, about. */
  ramGb: number;
  /** It calls tools: an agent can use it. */
  tools: boolean;
  license: string;
  /** Translated by the engine. */
  desc: string;
  bytes: number;
}

export interface LocalModel {
  id: string;
  name: string;
  /** owner/repo, empty for a file put there by hand. */
  repo: string;
  folder: string;
  files: string[];
  mmproj: string | null;
  bytes: number;
  /** Bytes on disk of a download not finished. */
  downloaded: number;
  complete: boolean;
  catalog: string | null;
  ctx: number | null;
  byHand: boolean;
}

export interface Gpu {
  name: string;
  /** Bytes. */
  memory: number;
}

export interface ModelsView {
  catalog: CatalogModel[];
  local: LocalModel[];
  /** The downloads under way, by key (`<folder>/<id>`). */
  pulling: string[];
  folder: string;
  /** The kit drive's file system: FAT32 holds no file over 4 GiB. */
  fs: string | null;
  free: number | null;
  hardware: { ram: number | null; gpus: Gpu[] };
  hfToken: boolean;
}

/** One model of a Hugging Face repository: a quantization, all its parts. */
export interface HfChoice {
  id: string;
  label: string;
  quant: string | null;
  files: ModelFile[];
  bytes: number;
}

export interface HfRepo {
  repo: string;
  commit: string;
  license: string | null;
  choices: HfChoice[];
}

/** A step of a download (`llm-pull`). */
export interface PullEvent {
  key: string;
  done: number;
  total: number;
  state: "running" | "done" | "failed" | "cancelled";
  error: string | null;
}

export const SAMPLE_LLM: LlmInfo = {
  installed: true,
  state: "running",
  port: 9931,
  url: "http://127.0.0.1:9931",
  error: null,
  loaded: ["qwen2.5-coder-7b"],
  autostart: false,
  piModel: null,
};

const GB = 1e9;

export const SAMPLE_MODELS: ModelsView = {
  catalog: [
    {
      id: "qwen2.5-coder-7b",
      name: "Qwen2.5-Coder 7B",
      repo: "Qwen/Qwen2.5-Coder-7B-Instruct-GGUF",
      commit: "13fb94bfda8c8cf22497dc57b78f391a9acb426a",
      files: [],
      quant: "Q4_K_M",
      ctx: 32768,
      ramGb: 7,
      tools: false,
      license: "apache-2.0",
      desc: engineText("per il codice, leggero: entra in 8 GB di RAM o di memoria video", {
        en: "for code, light: fits in 8 GB of RAM or video memory",
        fr: "pour le code, léger : tient dans 8 Go de RAM ou de mémoire vidéo",
        de: "für Code, leicht: passt in 8 GB RAM oder Grafikspeicher",
      }),
      bytes: 4.68 * GB,
    },
    {
      id: "gpt-oss-20b",
      name: "gpt-oss 20B",
      repo: "ggml-org/gpt-oss-20b-GGUF",
      commit: "15dcfc7e7c608c6161507211ed43c5b590801b4b",
      files: [],
      quant: "MXFP4",
      ctx: 32768,
      ramGb: 15,
      tools: true,
      license: "apache-2.0",
      desc: engineText("il modello aperto di OpenAI, con ragionamento e strumenti: per 16-24 GB", {
        en: "OpenAI's open model, with reasoning and tools: for 16-24 GB",
        fr: "le modèle ouvert d'OpenAI, avec raisonnement et outils : pour 16-24 Go",
        de: "das offene Modell von OpenAI, mit Reasoning und Werkzeugen: für 16-24 GB",
      }),
      bytes: 12.1 * GB,
    },
    {
      id: "qwen3-coder-30b-a3b",
      name: "Qwen3-Coder 30B-A3B",
      repo: "unsloth/Qwen3-Coder-30B-A3B-Instruct-GGUF",
      commit: "b17cb02dd882d5b6ab62fc777ad2995f19668350",
      files: [],
      quant: "Q4_K_M",
      ctx: 32768,
      ramGb: 22,
      tools: true,
      license: "apache-2.0",
      desc: engineText(
        "per il codice e gli agenti; a esperti (3B attivi), veloce anche sulla CPU con molta RAM: 24 GB e più",
        {
          en: "for code and agents; mixture of experts (3B active), fast on the CPU too with plenty of RAM: 24 GB and more",
          fr: "pour le code et les agents ; à experts (3B actifs), rapide même sur le CPU avec beaucoup de RAM : 24 Go et plus",
          de: "für Code und Agenten; Mixture of Experts (3B aktiv), auch auf der CPU schnell mit viel RAM: 24 GB und mehr",
        },
      ),
      bytes: 18.6 * GB,
    },
  ],
  local: [
    {
      id: "qwen2.5-coder-7b",
      name: "Qwen2.5-Coder 7B",
      repo: "Qwen/Qwen2.5-Coder-7B-Instruct-GGUF",
      folder: "Qwen--Qwen2.5-Coder-7B-Instruct-GGUF",
      files: [
        "qwen2.5-coder-7b-instruct-q4_k_m-00001-of-00002.gguf",
        "qwen2.5-coder-7b-instruct-q4_k_m-00002-of-00002.gguf",
      ],
      mmproj: null,
      bytes: 4.68 * GB,
      downloaded: 0,
      complete: true,
      catalog: "qwen2.5-coder-7b",
      ctx: 32768,
      byHand: false,
    },
    {
      id: "gpt-oss-20b",
      name: "gpt-oss 20B",
      repo: "ggml-org/gpt-oss-20b-GGUF",
      folder: "ggml-org--gpt-oss-20b-GGUF",
      files: ["gpt-oss-20b-MXFP4.gguf"],
      mmproj: null,
      bytes: 12.1 * GB,
      downloaded: 3.4 * GB,
      complete: false,
      catalog: "gpt-oss-20b",
      ctx: 32768,
      byHand: false,
    },
  ],
  pulling: [],
  folder: "E:\\Atollo\\data\\models\\gguf",
  fs: "exFAT",
  free: 96 * GB,
  hardware: { ram: 32 * 1024 ** 3, gpus: [{ name: "NVIDIA GeForce RTX 4070", memory: 12 * 1024 ** 3 }] },
  hfToken: false,
};

export function llmStatus(): Promise<LlmInfo> {
  return query<LlmInfo>("llm_status", SAMPLE_LLM);
}

export function llmModels(): Promise<ModelsView> {
  return query<ModelsView>("llm_models", SAMPLE_MODELS);
}

/** Starts the server, installing llama.cpp first if needed. */
export function llmStart(): Promise<LlmInfo> {
  return query<LlmInfo>("llm_start", SAMPLE_LLM);
}

export function llmStop(): Promise<LlmInfo> {
  return query<LlmInfo>("llm_stop", { ...SAMPLE_LLM, state: "stopped", loaded: [] });
}

/** «Avvia con Atollo», and Pi's model ("": none chosen, Pi takes the
 * first model on disk). */
export function llmSet(change: { autostart?: boolean; piModel?: string }): Promise<LlmInfo> {
  const preview = { ...SAMPLE_LLM, ...change, piModel: change.piModel || SAMPLE_LLM.piModel };
  return query<LlmInfo>("llm_set", preview, change);
}

export function llmHfInspect(repo: string): Promise<HfRepo> {
  return query<HfRepo>(
    "llm_hf_inspect",
    {
      repo,
      commit: "0".repeat(40),
      license: "apache-2.0",
      choices: [
        { id: "model-q4_k_m", label: "model-Q4_K_M", quant: "Q4_K_M", files: [], bytes: 4.9 * GB },
        { id: "model-q8_0", label: "model-Q8_0", quant: "Q8_0", files: [], bytes: 8.5 * GB },
      ],
    },
    { repo },
  );
}

/** Downloads a model of the catalog, or `choice` of a repository; the id
 * of the model. Progress as `llm-pull` events. */
export function llmPull(what: { catalog: string } | { repo: string; choice: string }): Promise<string> {
  return query<string>("llm_pull", "catalog" in what ? what.catalog : what.choice, what);
}

/** Stops a download; what arrived is kept for the next time. */
export function llmPullCancel(key: string): Promise<void> {
  return act("llm_pull_cancel", { key });
}

export function llmRemove(id: string): Promise<ModelsView> {
  return query<ModelsView>("llm_remove", SAMPLE_MODELS, { id });
}

/** The Hugging Face token (gated models); empty: removed. */
export function llmHfTokenSet(token: string): Promise<void> {
  return act("llm_hf_token_set", { token });
}

/** The server's key, for the chat in the browser. */
export function llmKey(): Promise<string> {
  return query<string>("llm_key", "3f9c2a71d8e04b6a9c51e2d7f0a8b346");
}

export function onLlmPull(handler: (e: PullEvent) => void): Promise<() => void> {
  return subscribe<PullEvent>(LLM_PULL, handler);
}
