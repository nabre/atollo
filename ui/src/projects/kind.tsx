// What a project is (how it starts, its languages) and its state, as the
// icons and marks of its row and of its page.
import type { IconType } from "react-icons";
import type { ProjectAction, ProjectInfo, RunInfo } from "../api";
import { t } from "../i18n";
import { LanguageIcon } from "../logos";
import { Mark } from "../tags";
import { Codicon, codicon, Dot } from "../workbench";
import { alive } from "./runs";

export function kindText(k: ProjectAction["type"]): string {
  return { web: t("sito web"), desktop: t("app con finestra"), command: t("comando") }[k];
}

export const KIND_ICON: Record<ProjectInfo["type"], IconType> = {
  web: codicon("globe"),
  desktop: codicon("window"),
  command: codicon("terminal"),
};

/** How the project starts (site, app with a window, command) and what it
 * was recognised as (Laravel, FastAPI, static site...). */
export function ProjectKind({ p }: { p: ProjectInfo }) {
  const Icon = KIND_ICON[p.type];
  return (
    <span className="flex min-w-0 items-center gap-1.5" title={`${kindText(p.type)} · ${p.reason}`}>
      <Mark label={kindText(p.type)}>
        <Icon size={14} aria-hidden="true" />
      </Mark>
      <span>{p.platform}</span>
    </span>
  );
}

/** Every language as its logo, in alphabetical order. */
export function LanguageIcons({ names }: { names: string[] }) {
  return (
    <span className="flex items-center gap-1.5">
      {[...names]
        .sort((a, b) => a.localeCompare(b, undefined, { sensitivity: "base" }))
        .map((l) => (
          <LanguageIcon key={l} name={l} />
        ))}
    </span>
  );
}

/** The project's state as the decoration of its row: running, or missing
 * components. */
export function ProjectState({ p, runs }: { p: ProjectInfo; runs: Record<string, RunInfo> }) {
  const live = Object.values(runs).filter((r) => r.project === p.id && alive(r.state));
  if (live.length > 0) {
    const text = live.some((r) => r.type === "web") ? t("in ascolto") : t("in esecuzione");
    return <Dot tone="ok" label={`${text}: ${live.map((r) => r.label).join(", ")}`} />;
  }
  if (p.missing.length > 0) {
    return (
      <Mark label={t("Mancano componenti: {0}", p.missing.join(", "))} className="text-vsc-warning">
        <Codicon name="warning" size={14} />
      </Mark>
    );
  }
  return null;
}
