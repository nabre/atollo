// The ways to make a new project: a tree in the side bar of Projects (an
// empty folder, a clone of a Git address, a framework or CMS and its bases
// by group: web with PHP, CMS, desktop…), each opening its procedure in the
// «Nuovo progetto» tab of the editor.
import { useState, type ReactNode, type SyntheticEvent } from "react";
import { projectClone, projectCreate } from "../api";
import { bases, basesByCategory, PieceLogo, textOf, type Frameworks } from "../frameworks";
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
  return { empty: t("Progetto vuoto"), git: t("Da un indirizzo Git"), framework: t("Da framework o CMS") }[way];
}

/** The side bar's tree of the ways; under the frameworks, their bases. */
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
  const [open, setOpen] = useState(true);
  /** Groups opened or closed by hand; the others open while they hold the base shown. */
  const [groups, setGroups] = useState<Record<string, boolean>>({});
  const list = bases(fw.pieces);
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
      <ListRow
        twistie={list.length > 0 ? (open ? "open" : "closed") : undefined}
        icon={<Codicon name="wand" size={16} />}
        label={wayLabel("framework")}
        description={list.length > 0 ? String(list.length) : undefined}
        selected={way === "framework" && !open}
        current="page"
        onSelect={() => {
          // Shown already: the branch opens and closes. Else the wizard
          // opens, the bases under it.
          if (way === "framework") setOpen(!open);
          else setOpen(true);
          onSelect("framework");
        }}
      >
        {list.length > 0 && (
          <List label={wayLabel("framework")}>
            {basesByCategory(fw.pieces).map((g) => {
              const holds = way === "framework" && g.bases.some((b) => b.id === base);
              const shown = groups[g.id] ?? holds;
              return (
                <ListRow
                  key={g.id}
                  depth={1}
                  twistie={shown ? "open" : "closed"}
                  label={g.label}
                  description={String(g.bases.length)}
                  onSelect={() => {
                    setGroups((all) => ({ ...all, [g.id]: !shown }));
                  }}
                >
                  <List label={g.label}>
                    {g.bases.map((b) => (
                      <ListRow
                        key={b.id}
                        depth={2}
                        icon={<PieceLogo piece={b} />}
                        label={b.name || b.id}
                        title={textOf(fw.view, b.desc)}
                        selected={way === "framework" && base === b.id}
                        current="page"
                        onSelect={() => {
                          onSelect("framework", b.id);
                        }}
                      />
                    ))}
                  </List>
                </ListRow>
              );
            })}
          </List>
        )}
      </ListRow>
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
