// The ways to make a new project: a tree in the side bar of Projects (an
// empty folder, a clone of a Git address, then the bases of frameworks and
// CMS in four groups, Web, CMS, App and Altro, by category), each opening
// its procedure in the «Nuovo progetto» tab of the editor.
import { useState, type ReactNode, type SyntheticEvent } from "react";
import { projectClone, projectCreate, type FrameworkPiece } from "../api";
import { basesByGroup, PieceLogo, textOf, type Frameworks, type GroupId } from "../frameworks";
import { useAction } from "../hooks";
import { ErrorLine } from "../common";
import { t } from "../i18n";
import { Button, Codicon, EditorBody, EditorHeader, List, ListRow, TextField } from "../workbench";

/** The procedure shown in the «Nuovo progetto» tab. */
export type NewWay = "empty" | "git" | "framework";

/** The wizard's tab among the projects': no folder name holds a NUL. Here,
 * not in wizard.tsx, which is read the first time the tab opens. */
export const WIZARD_TAB = "\u0000wizard";

export function wayLabel(way: NewWay): string {
  return { empty: t("Progetto vuoto"), git: t("Da un indirizzo Git"), framework: t("Da una base") }[way];
}

/** The icon of a group of bases. */
export const GROUP_ICON: Record<GroupId, string> = {
  web: "globe",
  cms: "layout",
  app: "device-desktop",
  other: "package",
};

/** The side bar's tree of the ways; then the groups of bases (Web, CMS,
 * App, Altro), their categories and bases. */
export function NewProjectTree({
  fw,
  way,
  base,
  onSelect,
}: {
  fw: Frameworks;
  /** The procedure shown, or null when the tab is not the active one. */
  way: NewWay | null;
  base: string;
  onSelect: (way: NewWay, base?: string) => void;
}) {
  /** Branches opened or closed by hand (`g:` a group, `c:` a category); the
   * others open while they hold the base shown. */
  const [branches, setBranches] = useState<Record<string, boolean>>({});
  const holds = (list: FrameworkPiece[]) => way === "framework" && list.some((b) => b.id === base);
  const isOpen = (key: string, list: FrameworkPiece[]) => branches[key] ?? holds(list);
  const flip = (key: string, open: boolean) => {
    setBranches((all) => ({ ...all, [key]: open }));
  };
  const baseRows = (list: FrameworkPiece[], depth: number) =>
    list.map((b) => (
      <ListRow
        key={b.id}
        depth={depth}
        icon={<PieceLogo piece={b} />}
        label={b.name || b.id}
        title={textOf(fw.view, b.desc)}
        selected={way === "framework" && base === b.id}
        current="page"
        onSelect={() => {
          onSelect("framework", b.id);
        }}
      />
    ));
  return (
    <List label={t("Nuovo progetto")}>
      <ListRow
        icon={<Codicon name="new-folder" size={16} />}
        label={wayLabel("empty")}
        selected={way === "empty"}
        current="page"
        onSelect={() => {
          onSelect("empty");
        }}
      />
      <ListRow
        icon={<Codicon name="repo-clone" size={16} />}
        label={wayLabel("git")}
        selected={way === "git"}
        current="page"
        onSelect={() => {
          onSelect("git");
        }}
      />
      {basesByGroup(fw.pieces).map((g) => {
        const key = `g:${g.id}`;
        const open = isOpen(key, g.bases);
        return (
          <ListRow
            key={g.id}
            twistie={open ? "open" : "closed"}
            icon={<Codicon name={GROUP_ICON[g.id]} size={16} />}
            label={g.label}
            description={String(g.bases.length)}
            selected={holds(g.bases) && !open}
            current="page"
            onSelect={() => {
              // Shown already: the branch opens and closes. Else the wizard
              // opens on its first base, the branch open.
              if (holds(g.bases)) {
                flip(key, !open);
              } else {
                flip(key, true);
                onSelect("framework", g.bases[0]?.id);
              }
            }}
          >
            <List label={g.label}>
              {g.categories.length === 1
                ? baseRows(g.bases, 1)
                : g.categories.map((c) => {
                    const ckey = `c:${c.id}`;
                    const shown = isOpen(ckey, c.bases);
                    return (
                      <ListRow
                        key={c.id}
                        depth={1}
                        twistie={shown ? "open" : "closed"}
                        label={c.label}
                        description={String(c.bases.length)}
                        onSelect={() => {
                          flip(ckey, !shown);
                        }}
                      >
                        <List label={c.label}>{baseRows(c.bases, 2)}</List>
                      </ListRow>
                    );
                  })}
            </List>
          </ListRow>
        );
      })}
    </List>
  );
}

/** A simple procedure in the editor: its header, its form. */
function WayEditor({
  way,
  icon,
  description,
  children,
}: {
  way: NewWay;
  icon: ReactNode;
  description: string;
  children: ReactNode;
}) {
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <EditorHeader
        icon={icon}
        title={t("Nuovo progetto")}
        meta={[<span key="way">{wayLabel(way)}</span>]}
        description={description}
      />
      <EditorBody>
        <div className="max-w-105">{children}</div>
      </EditorBody>
    </div>
  );
}

const BIG = "text-vsc-icon opacity-80";

export function EmptyProjectEditor({ onDone, focus }: { onDone: (name: string) => void; focus: number }) {
  return (
    <WayEditor
      way="empty"
      icon={<Codicon name="new-folder" size={56} className={BIG} />}
      description={t("Una cartella nuova e vuota in projects.")}
    >
      <NewProject onDone={onDone} focus={focus} />
    </WayEditor>
  );
}

export function GitProjectEditor({ onDone }: { onDone: (name: string) => void }) {
  return (
    <WayEditor
      way="git"
      icon={<Codicon name="repo-clone" size={56} className={BIG} />}
      description={t("Il repository si clona in una cartella di projects.")}
    >
      <CloneProject onDone={onDone} />
    </WayEditor>
  );
}

export function NewProject({
  onDone,
  focus,
}: {
  onDone: (name: string) => void;
  /** Raised to put the focus on the name (the side bar's «Nuovo progetto»). */
  focus: number;
}) {
  const [name, setName] = useState("");
  const { busy, error, run } = useAction();
  const submit = (e: SyntheticEvent) => {
    e.preventDefault();
    run("create", async () => {
      onDone(await projectCreate(name));
    });
  };
  return (
    <form onSubmit={submit} className="space-y-1.5" aria-label={t("Vuoto")}>
      <label htmlFor="new-project-name" className="block font-semibold">
        {t("Vuoto")}
      </label>
      <p className="text-[12px] text-vsc-desc">{t("Nome della cartella")}</p>
      {/* Remounted on each request, so that it takes the focus again. */}
      <TextField
        key={focus}
        id="new-project-name"
        label={t("Vuoto")}
        autoFocus={focus > 0}
        value={name}
        onChange={setName}
        className="w-full"
      />
      {/* Required, as the native field was: nothing to submit while empty. */}
      <Button variant="primary" wide submit disabled={name === ""} busy={busy === "create"}>
        {t("Crea")}
      </Button>
      <ErrorLine error={error} />
    </form>
  );
}

export function CloneProject({ onDone }: { onDone: (name: string) => void }) {
  const [source, setSource] = useState("");
  const [name, setName] = useState("");
  const { busy, error, run } = useAction();
  const submit = (e: SyntheticEvent) => {
    e.preventDefault();
    run("clone", async () => {
      onDone(await projectClone(source, name.trim() === "" ? null : name));
    });
  };
  const sourceLabel = t("owner/repository di GitHub o indirizzo https://");
  const nameLabel = t("Nome della cartella (facoltativo)");
  return (
    <form onSubmit={submit} className="space-y-1.5" aria-label={t("Da un indirizzo Git")}>
      <p className="font-semibold">{t("Da un indirizzo Git")}</p>
      <label className="flex flex-col gap-1">
        <span className="text-[12px] text-vsc-desc">{sourceLabel}</span>
        <TextField label={sourceLabel} value={source} onChange={setSource} className="w-full" />
      </label>
      <label className="flex flex-col gap-1">
        <span className="text-[12px] text-vsc-desc">{nameLabel}</span>
        <TextField label={nameLabel} value={name} onChange={setName} className="w-full" />
      </label>
      <Button wide submit disabled={source === ""} busy={busy === "clone"}>
        <span className="inline-flex items-center gap-1">
          <Codicon name="cloud-download" size={14} />
          {t("Clona")}
        </span>
      </Button>
      <ErrorLine error={error} />
    </form>
  );
}
