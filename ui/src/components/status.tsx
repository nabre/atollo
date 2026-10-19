// The state of a tool on a system, as an icon: the columns of the systems
// in «Tutti gli strumenti» (./table.tsx). On the system shown it adds what
// the page knows only there: the preparation under way (queued, download,
// failed) and the choice waiting for "Apply".
import type { IconType } from "react-icons";
import {
  VscCircleLarge,
  VscCircleSlash,
  VscCloud,
  VscCloudDownload,
  VscError,
  VscHistory,
  VscIssues,
  VscLoading,
  VscPackage,
  VscPassFilled,
  VscTrash,
  VscVm,
  VscWarning,
  VscWatch,
} from "react-icons/vsc";
import type { ComponentInfo, TargetToolStatus } from "../api";
import { t } from "../i18n";
import type { ItemState } from "../setup";
import { cellReason } from "./targets";

/** The states, in the order of the legend and of the sort. */
export const TOOL_STATES = [
  "running",
  "queued",
  "failed",
  "incomplete",
  "notDownloaded",
  "toInstall",
  "toRemove",
  "reinstall",
  "installed",
  "staged",
  "firstStart",
  "emulated",
  "missing",
  "unavailable",
] as const;

export type ToolState = (typeof TOOL_STATES)[number];

const LOOK: Record<ToolState, { icon: IconType; tone: string; spin?: boolean }> = {
  running: { icon: VscLoading, tone: "text-vsc-link", spin: true },
  queued: { icon: VscHistory, tone: "text-vsc-desc" },
  failed: { icon: VscError, tone: "text-vsc-error" },
  incomplete: { icon: VscIssues, tone: "text-vsc-error" },
  notDownloaded: { icon: VscCloud, tone: "text-vsc-warning" },
  toInstall: { icon: VscCloudDownload, tone: "text-vsc-settings-modified" },
  toRemove: { icon: VscTrash, tone: "text-vsc-settings-modified" },
  reinstall: { icon: VscWarning, tone: "text-vsc-warning" },
  installed: { icon: VscPassFilled, tone: "text-vsc-success" },
  staged: { icon: VscPackage, tone: "text-vsc-success" },
  firstStart: { icon: VscWatch, tone: "text-vsc-warning" },
  emulated: { icon: VscVm, tone: "text-vsc-link" },
  missing: { icon: VscCircleLarge, tone: "text-vsc-desc" },
  unavailable: { icon: VscCircleSlash, tone: "text-vsc-desc" },
};

export function toolStateLabel(state: ToolState): string {
  return {
    running: t("in corso"),
    queued: t("in coda"),
    failed: t("non riuscito"),
    incomplete: t("incompleto"),
    notDownloaded: t("scelto, non scaricato"),
    toInstall: t("da installare"),
    toRemove: t("da rimuovere"),
    reinstall: t("da reinstallare"),
    installed: t("installato"),
    staged: t("pre-scaricato"),
    firstStart: t("si completa al primo avvio"),
    emulated: t("in emulazione"),
    missing: t("non installato"),
    unavailable: t("non disponibile"),
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
  if (item?.kind === "problem") return { state: "failed", detail: item.text };
  if (on !== (c.chosen || c.locked)) return { state: on ? "toInstall" : "toRemove", detail: null };
  if (c.installed) return { state: s?.needsReinstall ? "reinstall" : "installed", detail: null };
  if (s?.missing) return incomplete(s.missing);
  if (c.bootstrapStaged) return { state: "staged", detail: null };
  if (c.noArm64) return { state: "unavailable", detail: t("non disponibile su PC ARM64") };
  if (c.unavailableReason !== null) return { state: "unavailable", detail: c.unavailableReason };
  if (s) return otherStatus(s, target);
  return { state: c.available ? "missing" : "unavailable", detail: null };
}

/** Something on disk but not all: a download or an installation that did
 * not finish, with the first part missing. */
function incomplete(missing: string): ToolStatus {
  return { state: "incomplete", detail: t("manca {0}", missing) };
}

/** The state on another system, from the grid (Rust `probe_all_targets`). */
export function otherStatus(s: TargetToolStatus, target: string): ToolStatus {
  if (!s.available) return { state: "unavailable", detail: cellReason(s) };
  if (s.installed) return { state: s.needsReinstall ? "reinstall" : "installed", detail: null };
  if (s.missing) return incomplete(s.missing);
  if (s.bootstrapStaged) return { state: "staged", detail: null };
  if (s.chosen && !s.present) {
    return { state: "notDownloaded", detail: t("nessun file in data/apps/{0}", target) };
  }
  if (s.firstStart) return { state: "firstStart", detail: null };
  if (s.emulated) return { state: "emulated", detail: null };
  return { state: "missing", detail: null };
}

/** The icon of a state, decorative: the cell around it carries the name. */
export function ToolStateIcon({ state }: { state: ToolState }) {
  const look = LOOK[state];
  const Icon = look.icon;
  return (
    <Icon size={14} aria-hidden="true" className={`inline shrink-0 ${look.tone} ${look.spin ? "animate-spin" : ""}`} />
  );
}
