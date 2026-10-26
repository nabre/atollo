// The state of a tool on a system, as an icon: the columns of the systems
// in «Tutti gli strumenti» (./table.tsx). What is on disk takes five states
// only: missing, staged, installed, to update and error; why a tool is
// missing (not available there, chosen but not downloaded) goes in the
// tooltip, not in a state of its own. A tool not needed on a system (the
// system does its job, ConPTY on macOS and Linux, D-177) is never missing:
// it has a state of its own, with the reason in the tooltip. On the system shown it adds what the page knows only there: the preparation under way
// (queued, download) and the choice waiting for "Apply".
import type { IconType } from "react-icons";
import type { ComponentInfo, TargetToolStatus } from "../ipc";
import { t } from "../i18n";
import type { ItemState } from "./setup";
import { cellReason } from "./targets";
import { codicon } from "../workbench";

/** The states, in the order of the sort. */
export const TOOL_STATES = [
  "running",
  "queued",
  "toInstall",
  "toRemove",
  "error",
  "update",
  "installed",
  "staged",
  "missing",
  "notNeeded",
  "needsVm",
  "provided",
] as const;

export type ToolState = (typeof TOOL_STATES)[number];

const LOOK: Record<ToolState, { icon: IconType; tone: string; spin?: boolean }> = {
  running: { icon: codicon("loading"), tone: "text-vsc-link", spin: true },
  queued: { icon: codicon("history"), tone: "text-vsc-desc" },
  toInstall: { icon: codicon("cloud-download"), tone: "text-vsc-settings-modified" },
  toRemove: { icon: codicon("trash"), tone: "text-vsc-settings-modified" },
  error: { icon: codicon("error"), tone: "text-vsc-error" },
  update: { icon: codicon("warning"), tone: "text-vsc-warning" },
  installed: { icon: codicon("pass-filled"), tone: "text-vsc-success" },
  staged: { icon: codicon("package"), tone: "text-vsc-success" },
  missing: { icon: codicon("circle-large"), tone: "text-vsc-desc" },
  notNeeded: { icon: codicon("dash"), tone: "text-vsc-desc" },
  needsVm: { icon: codicon("vm"), tone: "text-vsc-desc" },
  provided: { icon: codicon("vm-active"), tone: "text-vsc-link" },
};

export function toolStateLabel(state: ToolState): string {
  return {
    running: t("in corso"),
    queued: t("in coda"),
    toInstall: t("da installare"),
    toRemove: t("da rimuovere"),
    error: t("errore"),
    update: t("da aggiornare"),
    installed: t("installato"),
    staged: t("pre-installato"),
    missing: t("non esiste"),
    notNeeded: t("non serve"),
    needsVm: t("serve la VM"),
    provided: t("fornito dalla VM"),
  }[state];
}

/** A tool's state on a system, and the detail of its tooltip. */
export interface ToolStatus {
  state: ToolState;
  detail: string | null;
}

/** The state on the system shown: the preparation first, then the choice
 * waiting for "Apply", then what is on disk or why it cannot be. */
export function shownStatus(
  c: ComponentInfo,
  s: TargetToolStatus | undefined,
  on: boolean,
  item: ItemState | null,
  target: string,
): ToolStatus {
  if (item?.kind === "running") {
    return {
      state: "running",
      detail: item.percent !== null ? t("download {0}%", item.percent) : item.note,
    };
  }
  if (item?.kind === "queued") return { state: "queued", detail: null };
  if (item?.kind === "problem") return { state: "error", detail: item.text };
  // An agent (LEVELS.md §7, Q11): an install for the PC goes with «Applica»;
  // Claude Code and Codex come with the VM; Pi and Antigravity need it.
  if (c.leftover) return { state: "toRemove", detail: leftoverText() };
  if (c.agent?.kind === "provided") return { state: "provided", detail: providedText(c.name) };
  if (c.agent?.kind === "needsVm") return { state: "needsVm", detail: c.agent.reason };
  if (on !== (c.chosen || c.locked)) return { state: on ? "toInstall" : "toRemove", detail: null };
  if (c.installed) return { state: s?.needsReinstall ? "update" : "installed", detail: null };
  if (s?.missing) return incomplete(s.missing);
  if (c.bootstrapStaged) return { state: "staged", detail: null };
  if (c.notNeeded !== null) return { state: "notNeeded", detail: c.notNeeded };
  if (c.unavailableReason !== null) return { state: "missing", detail: c.unavailableReason };
  if (s) return otherStatus(s, target);
  return { state: "missing", detail: null };
}

/** Why an agent's folder for the PC goes. */
export function leftoverText(): string {
  return t(
    "Installato per il PC da una versione precedente di Atollo: gli agenti girano solo nella VM, si toglie con «Applica».",
  );
}

/** What Claude Code's or Codex's row says. */
export function providedText(name: string): string {
  return t("{0} arriva con la sua sandbox di Docker Sandboxes: Atollo non lo scarica.", name);
}

/** Something on disk but not all: a download or an installation that did
 * not finish, with the first part missing. */
function incomplete(missing: string): ToolStatus {
  return { state: "error", detail: t("manca {0}", missing) };
}

/** The state on another system, from the grid (Rust `probe_all_targets`).
 * A tool that completes at its first start there is staged too: its files
 * are in the kit, waiting for that system. */
export function otherStatus(s: TargetToolStatus, target: string): ToolStatus {
  if (s.notNeeded !== null) return { state: "notNeeded", detail: s.notNeeded };
  if (!s.available) return { state: "missing", detail: cellReason(s) };
  if (s.installed) return { state: s.needsReinstall ? "update" : "installed", detail: null };
  if (s.missing) return incomplete(s.missing);
  if (s.bootstrapStaged || s.firstStart) return { state: "staged", detail: null };
  if (s.chosen && !s.present) {
    return { state: "missing", detail: t("nessun file in data/apps/{0}", target) };
  }
  return { state: "missing", detail: null };
}

/** The icon of a state, decorative: the cell around it carries the name. */
export function ToolStateIcon({ state }: { state: ToolState }) {
  const look = LOOK[state];
  const Icon = look.icon;
  return <Icon size={14} aria-hidden="true" className={`shrink-0 ${look.tone} ${look.spin ? "animate-spin" : ""}`} />;
}
