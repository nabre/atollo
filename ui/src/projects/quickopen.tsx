// Quick Open of Projects (Ctrl+P), as the editor's: a box at the top of the
// window, a project found by its name, platform or languages, Enter opens its
// page, Shift+Enter starts it; under the projects, the ways to make a new one.
import type { ProjectInfo, RunInfo } from "../api";
import { t } from "../i18n";
import { Palette, type PaletteItem } from "../palette";
import { codicon } from "../workbench";
import { mainAction } from "./home";
import { KIND_ICON, ProjectState } from "./kind";
import { wayLabel, type NewWay } from "./new";

export function QuickOpen({
  projects,
  runs,
  onOpen,
  onStart,
  onNew,
  onClose,
}: {
  projects: ProjectInfo[];
  runs: Record<string, RunInfo>;
  onOpen: (id: string) => void;
  onStart: (p: ProjectInfo) => void;
  onNew: (way: NewWay) => void;
  onClose: () => void;
}) {
  const items: PaletteItem[] = [
    ...[...projects]
      .sort(
        (a, b) => Math.max(b.modified, b.settings.lastOpened ?? 0) - Math.max(a.modified, a.settings.lastOpened ?? 0),
      )
      .map((p): PaletteItem => {
        const Icon = KIND_ICON[p.type];
        return {
          key: p.id,
          icon: <Icon size={16} aria-hidden="true" />,
          label: p.name,
          detail: p.platform,
          search: `${p.name} ${p.platform} ${p.languages.join(" ")}`,
          decoration: <ProjectState p={p} runs={runs} />,
          run: () => {
            onOpen(p.id);
          },
          alt: mainAction(p)
            ? () => {
                onStart(p);
              }
            : undefined,
        };
      }),
    ...(
      [
        ["empty", codicon("new-folder")],
        ["git", codicon("repo-clone")],
        ["framework", codicon("wand")],
      ] as const
    ).map(([way, Icon]): PaletteItem => ({
      key: `new:${way}`,
      icon: <Icon size={16} aria-hidden="true" />,
      label: t("Nuovo progetto"),
      detail: wayLabel(way),
      run: () => {
        onNew(way);
      },
    })),
  ];
  return (
    <Palette
      label={t("Apri un progetto")}
      placeholder={t("Nome del progetto, piattaforma o linguaggio")}
      listLabel={t("Progetti")}
      items={items}
      empty={t("Nessun progetto corrisponde alla ricerca.")}
      hint={t("Invio apre la pagina · Maiusc+Invio avvia")}
      onClose={onClose}
    />
  );
}
