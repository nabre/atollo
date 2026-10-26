// The four systems Docker Sandboxes runs on (D-176), as people call them
// («macOS Apple Silicon», not darwin-arm64), with what each one is for this PC: the «Piattaforma»
// selector of the Tools page and the columns of its grid (D-121).
import type { TargetProfile, TargetToolStatus } from "../ipc";
import { t } from "../i18n";

/** The readable name of a system id (`darwin-arm64`…); the id itself for
 * one the interface does not know. */
export function targetName(id: string): string {
  switch (id) {
    case "win32-x64":
      return t("Windows x64");
    case "darwin-arm64":
      return t("macOS Apple Silicon");
    case "linux-x64":
      return t("Linux x64");
    case "linux-arm64":
      return t("Linux ARM64");
    default:
      return id;
  }
}

/** What the system is for this PC: itself, run in emulation (the Rust
 * side's `emulatedOnHost`; of the four systems only the x64 builds of some
 * components run emulated, with Rosetta 2 on a Mac with Apple Silicon, and
 * those are marked per cell), or not run here at all (its kit is prepared,
 * it completes on that system). */
export function targetMark(p: TargetProfile): string {
  if (p.isHost) return t("questo PC");
  if (p.emulatedOnHost) return t("in emulazione");
  if (!p.executableOnHost) return t("non eseguibile qui");
  return "";
}

/** The name with its mark, for the options of «Piattaforma». */
export function targetLabel(id: string, profiles: TargetProfile[]): string {
  const p = profiles.find((x) => x.id === id);
  const mark = p ? targetMark(p) : "";
  return mark ? `${targetName(id)} (${mark})` : targetName(id);
}

/** The states of a cell of the grid, in the order of the legend. */
export const CELL_STATES = ["available", "vm", "emulated", "staged", "firstStart", "unavailable"] as const;

export type CellState = (typeof CELL_STATES)[number];

/** The state of a component on a system: not available (with the reason),
 * pre-downloaded, completing on the first start there, in emulation, run
 * in the VM with its build for Linux (the PC's system with the VM on,
 * D-174), or available. */
export function cellState(s: TargetToolStatus): CellState {
  if (s.viaVm) return "vm";
  if (!s.available) return "unavailable";
  if (s.bootstrapStaged) return "staged";
  if (s.firstStart) return "firstStart";
  if (s.emulated) return "emulated";
  return "available";
}

export function cellLabel(state: CellState): string {
  return {
    available: t("disponibile"),
    vm: t("nella VM"),
    emulated: t("in emulazione"),
    staged: t("pre-installato"),
    firstStart: t("si completa al primo avvio"),
    unavailable: t("non disponibile"),
  }[state];
}

/** What the legend says of each state. */
export function cellHint(state: CellState): string {
  return {
    available: t("Si installa e gira con la build per questo sistema."),
    vm: t("Con la VM attiva gira nella VM, con la build per Linux: su questo sistema non serve."),
    emulated: t("Si installa la build x64, che il sistema esegue in emulazione."),
    staged: t("Gli archivi sono già nel kit: l'installazione termina al primo avvio su quel sistema."),
    firstStart: t(
      "Preparato da questo PC, scarica solo gli archivi: l'installazione termina al primo avvio su quel sistema.",
    ),
    unavailable: t("Nessuna build per questo sistema: il motivo è nel dettaglio."),
  }[state];
}

/** Why a component is not available there, already translated. */
export function cellReason(s: TargetToolStatus): string | null {
  if (s.available) return null;
  return s.unavailableReason;
}
