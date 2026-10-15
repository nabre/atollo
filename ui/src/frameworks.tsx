// The instructions of the new-project wizard in Settings (frameworks.rs,
// D-83): a tree in the side bar, one tab per base (Laravel, Vite,
// Electron…) and per addition (React, Tailwind CSS, NativePHP…), with the
// commands that install it, one per line, the kit's tools as tags with their
// logo, the variables of .env and the database. The official ones come with
// the app; a changed one shows the bar of a modified setting and goes back
// with «Ripristina». Also the plan of a combination, shown here to try the
// instructions and in the wizard.
import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";
import {
  componentsList,
  frameworksGet,
  frameworksPlan,
  frameworksSet,
  urlOpen,
  type ComponentInfo,
  type EnvVar,
  type FrameworkAction,
  type FrameworkPiece,
  type FrameworkPlan,
  type FrameworkVariant,
  type FrameworksView,
} from "./api";
import { ErrorLine } from "./pages";
import { t } from "./i18n";
import { VscAdd, VscBeaker, VscClose, VscExtensions, VscPackage } from "react-icons/vsc";
import { ComponentLogo, FrameworkLogo } from "./logos";
import { Tag } from "./tags";
import { CheckboxLabel, IconButton, Link, List, ListRow, Notice, SettingItem, SettingsGroup } from "./workbench";

/** The name of a slot of additions (one of each goes in a project). */
export function slotLabel(slot: string | undefined): string {
  switch (slot) {
    case "frontend":
      return t("Interfaccia");
    case "css":
      return t("Stili");
    case "desktop":
      return t("App desktop");
    case "mobile":
      return t("App mobile");
    case "tests":
      return t("Prove");
    case "admin":
      return t("Amministrazione");
    case "format":
      return t("Formattazione");
    case "database":
      return t("Database");
    case "bundler":
      return t("Sviluppo");
    case undefined:
    case "":
      return t("Altro");
    default:
      return slot;
  }
}

/** A description or note in the app's language (custom ones stay as written). */
export function textOf(view: FrameworksView | null, source: string | undefined): string {
  if (!source) return "";
  return view?.texts[source] ?? source;
}

export function bases(pieces: FrameworkPiece[]): FrameworkPiece[] {
  return pieces.filter((p) => p.kind === "base");
}

/** The additions that work with `base`. */
export function addonsFor(pieces: FrameworkPiece[], base: string): FrameworkPiece[] {
  return pieces.filter((p) => p.kind === "addon" && p.on?.[base] !== undefined);
}

const lines = (v: string[] | undefined) => (v ?? []).map((l) => l.trim()).filter(Boolean);

const envs = (v: EnvVar[] | undefined) =>
  (v ?? []).map((e) => ({ key: e.key.trim(), value: e.value.trim() })).filter((e) => e.key || e.value);

/** A piece as it is saved: trimmed, no empty lines (to compare). */
function cleaned(p: FrameworkPiece): string {
  const actions = (a: FrameworkAction[] | undefined) =>
    (a ?? []).map((x) => ({ name: x.name.trim(), command: x.command.trim() })).filter((x) => x.name || x.command);
  const variant = (v: FrameworkVariant) => ({
    needs: lines(v.needs),
    create: lines(v.create),
    steps: lines(v.steps),
    actions: actions(v.actions),
    env: envs(v.env),
    database: v.database?.trim() ?? "",
    note: v.note?.trim() ?? "",
  });
  const on = Object.keys(p.on ?? {})
    .sort()
    .map((k) => [k, variant(p.on?.[k] ?? {})]);
  return JSON.stringify([
    p.id,
    p.kind,
    p.name.trim(),
    p.desc.trim(),
    p.home.trim(),
    p.slot?.trim() ?? "",
    lines(p.needs),
    lines(p.create),
    lines(p.steps),
    actions(p.actions),
    envs(p.env),
    p.note?.trim() ?? "",
    on,
  ]);
}

/** The instructions as loaded and as edited, saved with the settings. */
export function useFrameworks() {
  const [view, setView] = useState<FrameworksView | null>(null);
  const [pieces, setPieces] = useState<FrameworkPiece[]>([]);
  const [dirty, setDirty] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const load = useCallback(() => {
    frameworksGet().then(
      (v) => {
        setView(v);
        setPieces(v.pieces);
        setDirty(false);
        setError(null);
      },
      (e: unknown) => {
        setError(String(e));
      },
    );
  }, []);
  const change = useCallback((next: FrameworkPiece[]) => {
    setPieces(next);
    setDirty(true);
  }, []);
  const save = useCallback(async () => {
    if (!dirty) return;
    await frameworksSet(pieces);
    load();
  }, [dirty, pieces, load]);
  return { view, pieces, dirty, error, load, change, save };
}

export type Frameworks = ReturnType<typeof useFrameworks>;

/** Lines of commands in a textarea, one per line. The text is kept as typed
 * and only replaced when the value changes from outside (Ripristina). */
function LinesField({
  label,
  value,
  onChange,
  placeholder,
}: {
  label: string;
  value: string[] | undefined;
  onChange: (v: string[]) => void;
  placeholder?: string;
}) {
  const [text, setText] = useState((value ?? []).join("\n"));
  const sent = useRef(value);
  useEffect(() => {
    if (value !== sent.current) {
      sent.current = value;
      setText((value ?? []).join("\n"));
    }
  }, [value]);
  return (
    <label className="block">
      <span className="mb-1 block text-vsc-desc">{label}</span>
      <textarea
        value={text}
        placeholder={placeholder}
        spellCheck={false}
        wrap="off"
        rows={Math.min(8, Math.max(2, text.split("\n").length + 1))}
        onChange={(e) => {
          setText(e.target.value);
          const next = e.target.value.split("\n");
          sent.current = next;
          onChange(next);
        }}
        className="input w-full font-mono text-[12px]"
      />
    </label>
  );
}

const formatActions = (a: FrameworkAction[] | undefined) =>
  (a ?? []).map((x) => (x.name ? `${x.name}: ${x.command}` : x.command)).join("\n");

function parseActions(text: string): FrameworkAction[] {
  return text
    .split("\n")
    .filter((l) => l.trim())
    .map((l) => {
      const at = l.indexOf(":");
      return at < 0
        ? { name: "", command: l.trim() }
        : { name: l.slice(0, at).trim(), command: l.slice(at + 1).trim() };
    });
}

/** The actions saved in the project, one per line as «Nome: comando». */
function ActionsField({
  value,
  onChange,
}: {
  value: FrameworkAction[] | undefined;
  onChange: (v: FrameworkAction[]) => void;
}) {
  const [text, setText] = useState(formatActions(value));
  const sent = useRef(value);
  useEffect(() => {
    if (value !== sent.current) {
      sent.current = value;
      setText(formatActions(value));
    }
  }, [value]);
  return (
    <label className="block">
      <span className="mb-1 block text-vsc-desc">{t("Azioni del progetto (Nome: comando)")}</span>
      <textarea
        value={text}
        placeholder="Avvia: npm run dev"
        spellCheck={false}
        wrap="off"
        rows={Math.min(6, Math.max(2, text.split("\n").length + 1))}
        onChange={(e) => {
          setText(e.target.value);
          const next = parseActions(e.target.value);
          sent.current = next;
          onChange(next);
        }}
        className="input w-full font-mono text-[12px]"
      />
    </label>
  );
}

let catalog: Promise<ComponentInfo[]> | null = null;

/** The kit's components, asked once: the names and logos of the tags. */
export function useComponents(): ComponentInfo[] {
  const [list, setList] = useState<ComponentInfo[]>([]);
  useEffect(() => {
    let gone = false;
    catalog ??= componentsList().catch(() => []);
    void catalog.then((l) => {
      if (!gone) setList(l);
    });
    return () => {
      gone = true;
    };
  }, []);
  return list;
}

/** A tool of the kit as a tag: its logo and its name in the catalog. */
export function ToolTag({
  id,
  components,
  onRemove,
}: {
  id: string;
  components: ComponentInfo[];
  onRemove?: () => void;
}) {
  const name = components.find((c) => c.id === id)?.name ?? id;
  return (
    <Tag icon={<ComponentLogo id={id} size={12} />} className="text-vsc-fg">
      {name}
      {onRemove && (
        <button
          type="button"
          aria-label={t("Togli {0}", name)}
          title={t("Togli {0}", name)}
          onClick={onRemove}
          className="-mr-1 flex size-4 items-center justify-center rounded hover:bg-vsc-toolbar-hover"
        >
          <VscClose size={12} aria-hidden="true" />
        </button>
      )}
    </Tag>
  );
}

/** The kit's tools as tags, with «+» to add one from the catalog. */
function NeedsField({
  label,
  value,
  onChange,
}: {
  label: string;
  value: string[] | undefined;
  onChange: (v: string[]) => void;
}) {
  const components = useComponents();
  const [adding, setAdding] = useState(false);
  const ids = value ?? [];
  const others = components.filter((c) => !ids.includes(c.id) && c.group !== "app");
  return (
    <div>
      <span className="mb-1 block text-vsc-desc">{label}</span>
      <div className="flex flex-wrap items-center gap-1">
        {ids.map((id) => (
          <ToolTag
            key={id}
            id={id}
            components={components}
            onRemove={() => {
              onChange(ids.filter((x) => x !== id));
            }}
          />
        ))}
        {adding ? (
          <select
            autoFocus
            aria-label={t("Aggiungi uno strumento del kit")}
            value=""
            onBlur={() => {
              setAdding(false);
            }}
            onChange={(e) => {
              if (e.target.value) onChange([...ids, e.target.value]);
              setAdding(false);
            }}
            className="input"
          >
            <option value="">{t("scegli uno strumento…")}</option>
            {others.map((c) => (
              <option key={c.id} value={c.id}>
                {c.name}
              </option>
            ))}
          </select>
        ) : (
          <IconButton
            icon={VscAdd}
            label={t("Aggiungi uno strumento del kit")}
            onClick={() => {
              setAdding(true);
            }}
          />
        )}
      </div>
    </div>
  );
}

const formatEnv = (v: EnvVar[] | undefined) =>
  (v ?? []).map((e) => (e.key ? `${e.key}=${e.value}` : e.value)).join("\n");

function parseEnv(text: string): EnvVar[] {
  return text
    .split("\n")
    .filter((l) => l.trim())
    .map((l) => {
      const at = l.indexOf("=");
      return at < 0 ? { key: l.trim(), value: "" } : { key: l.slice(0, at).trim(), value: l.slice(at + 1).trim() };
    });
}

/** The variables of .env, one per line as «CHIAVE=valore». */
function EnvField({ value, onChange }: { value: EnvVar[] | undefined; onChange: (v: EnvVar[]) => void }) {
  const [text, setText] = useState(formatEnv(value));
  const sent = useRef(value);
  useEffect(() => {
    if (value !== sent.current) {
      sent.current = value;
      setText(formatEnv(value));
    }
  }, [value]);
  return (
    <label className="block">
      <span className="mb-1 block text-vsc-desc">{t("Variabili del file .env (CHIAVE=valore)")}</span>
      <textarea
        value={text}
        placeholder="APP_NAME={name}"
        spellCheck={false}
        wrap="off"
        rows={Math.min(6, Math.max(2, text.split("\n").length + 1))}
        onChange={(e) => {
          setText(e.target.value);
          const next = parseEnv(e.target.value);
          sent.current = next;
          onChange(next);
        }}
        className="input w-full font-mono text-[12px]"
      />
    </label>
  );
}

/** The kit's database server where the project gets its database. */
function DatabaseField({ value, onChange }: { value: string | undefined; onChange: (v: string | undefined) => void }) {
  return (
    <label className="block">
      <span className="mb-1 block text-vsc-desc">{t("Database del kit per il progetto")}</span>
      <select
        value={value ?? ""}
        onChange={(e) => {
          onChange(e.target.value || undefined);
        }}
        className="input w-full"
      >
        <option value="">{t("Nessuno")}</option>
        <option value="mariadb">MariaDB (MySQL)</option>
        <option value="mongodb">MongoDB</option>
      </select>
    </label>
  );
}

function TextField({ label, value, onChange }: { label: string; value: string; onChange: (v: string) => void }) {
  return (
    <label className="block">
      <span className="mb-1 block text-vsc-desc">{label}</span>
      <input
        value={value}
        onChange={(e) => {
          onChange(e.target.value);
        }}
        className="input w-full"
      />
    </label>
  );
}

/** What an addition does with one base. */
function VariantFields({
  title,
  note,
  variant,
  onChange,
  onRemove,
}: {
  title: string;
  note: string;
  variant: FrameworkVariant;
  onChange: (v: FrameworkVariant) => void;
  onRemove: () => void;
}) {
  return (
    <fieldset className="space-y-2 rounded border border-vsc-border p-2.5">
      <legend className="px-1 font-semibold">{title}</legend>
      {note && <p className="text-vsc-desc">{note}</p>}
      <LinesField
        label={t("Crea il progetto al posto della base (facoltativo)")}
        value={variant.create}
        onChange={(create) => {
          onChange({ ...variant, create });
        }}
      />
      <LinesField
        label={t("Poi, nella cartella del progetto")}
        value={variant.steps}
        onChange={(steps) => {
          onChange({ ...variant, steps });
        }}
      />
      <ActionsField
        value={variant.actions}
        onChange={(actions) => {
          onChange({ ...variant, actions });
        }}
      />
      <EnvField
        value={variant.env}
        onChange={(env) => {
          onChange({ ...variant, env });
        }}
      />
      <DatabaseField
        value={variant.database}
        onChange={(database) => {
          const next = { ...variant };
          if (database) next.database = database;
          else delete next.database;
          onChange(next);
        }}
      />
      <NeedsField
        label={t("Altri strumenti del kit")}
        value={variant.needs}
        onChange={(needs) => {
          onChange({ ...variant, needs });
        }}
      />
      <Link onClick={onRemove}>{t("Togli questa base")}</Link>
    </fieldset>
  );
}

function PieceItem({
  piece,
  official,
  all,
  view,
  onChange,
  onRemove,
}: {
  piece: FrameworkPiece;
  official: FrameworkPiece | undefined;
  all: FrameworkPiece[];
  view: FrameworksView | null;
  onChange: (p: FrameworkPiece) => void;
  onRemove: () => void;
}) {
  const modified = official ? cleaned(official) !== cleaned(piece) : true;
  const category = piece.kind === "base" ? t("Base") : slotLabel(piece.slot);
  const desc = textOf(view, piece.desc);
  const note = textOf(view, piece.note);
  const keywords = [piece.id, ...(piece.create ?? []), ...(piece.steps ?? [])].join(" ");
  const others = bases(all).filter((b) => piece.on?.[b.id] === undefined);
  const set = (patch: Partial<FrameworkPiece>) => {
    onChange({ ...piece, ...patch });
  };
  return (
    <SettingItem
      category={category}
      title={piece.name || piece.id}
      description={note ? `${desc} ${note}` : desc}
      modified={modified}
      keywords={keywords}
    >
      <div className="space-y-2.5">
        {!official && (
          <>
            <TextField
              label={t("Nome")}
              value={piece.name}
              onChange={(name) => {
                set({ name });
              }}
            />
            <TextField
              label={t("Descrizione")}
              value={piece.desc}
              onChange={(d) => {
                set({ desc: d });
              }}
            />
            {piece.kind === "addon" && (
              <TextField
                label={t("Gruppo (due aggiunte dello stesso gruppo non vanno insieme)")}
                value={piece.slot ?? ""}
                onChange={(slot) => {
                  set({ slot });
                }}
              />
            )}
            <TextField
              label={t("Pagina ufficiale (https://)")}
              value={piece.home}
              onChange={(home) => {
                set({ home });
              }}
            />
          </>
        )}
        {piece.kind === "base" ? (
          <>
            <NeedsField
              label={t("Strumenti del kit")}
              value={piece.needs}
              onChange={(needs) => {
                set({ needs });
              }}
            />
            <LinesField
              label={t("Crea il progetto (nella cartella projects)")}
              value={piece.create}
              onChange={(create) => {
                set({ create });
              }}
            />
            <LinesField
              label={t("Poi, nella cartella del progetto")}
              value={piece.steps}
              onChange={(steps) => {
                set({ steps });
              }}
            />
            <ActionsField
              value={piece.actions}
              onChange={(actions) => {
                set({ actions });
              }}
            />
            <EnvField
              value={piece.env}
              onChange={(env) => {
                set({ env });
              }}
            />
          </>
        ) : (
          <>
            {bases(all)
              .filter((b) => piece.on?.[b.id] !== undefined)
              .map((b) => {
                const variant = piece.on?.[b.id] ?? {};
                return (
                  <VariantFields
                    key={b.id}
                    title={t("Con {0}", b.name)}
                    note={textOf(view, variant.note)}
                    variant={variant}
                    onChange={(v) => {
                      set({ on: { ...piece.on, [b.id]: v } });
                    }}
                    onRemove={() => {
                      const on = { ...piece.on };
                      // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
                      delete on[b.id];
                      set({ on });
                    }}
                  />
                );
              })}
            {others.length > 0 && (
              <label className="flex items-center gap-2">
                <span className="text-vsc-desc">{t("Usala anche con")}</span>
                <select
                  value=""
                  aria-label={t("Usala anche con")}
                  onChange={(e) => {
                    if (e.target.value) set({ on: { ...piece.on, [e.target.value]: {} } });
                  }}
                  className="input"
                >
                  <option value="">{t("scegli una base…")}</option>
                  {others.map((b) => (
                    <option key={b.id} value={b.id}>
                      {b.name}
                    </option>
                  ))}
                </select>
              </label>
            )}
          </>
        )}
        <div className="flex flex-wrap gap-x-4 gap-y-1">
          {piece.home && (
            <Link
              onClick={() => {
                void urlOpen(piece.home);
              }}
            >
              {t("Pagina ufficiale")}
            </Link>
          )}
          {official && modified && (
            <Link
              onClick={() => {
                onChange(official);
              }}
            >
              {t("Ripristina le istruzioni ufficiali")}
            </Link>
          )}
          {!official && <Link onClick={onRemove}>{t("Togli")}</Link>}
        </div>
      </div>
    </SettingItem>
  );
}

function newId(pieces: FrameworkPiece[], prefix: string): string {
  for (let i = 1; ; i++) {
    const id = `${prefix}-${String(i)}`;
    if (!pieces.some((p) => p.id === id)) return id;
  }
}

/** Adds a new base or addition and returns its id. */
export function addPiece(fw: Frameworks, kind: FrameworkPiece["kind"]): string {
  const id = newId(fw.pieces, kind);
  const piece: FrameworkPiece =
    kind === "base"
      ? { id, kind, name: t("Nuova base"), desc: "", home: "", create: ["<comando> {name}"] }
      : { id, kind, name: t("Nuova aggiunta"), desc: "", home: "", on: {} };
  fw.change([...fw.pieces, piece]);
  return id;
}

/** The logo of a base or addition; a generic icon for those without one. */
export function PieceLogo({ piece, size = 16 }: { piece: FrameworkPiece; size?: number }) {
  const Fallback = piece.kind === "base" ? VscPackage : VscExtensions;
  return <FrameworkLogo id={piece.id} size={size} fallback={<Fallback size={size} aria-hidden="true" />} />;
}

/** The tab id of a piece in the Settings page. */
export const pieceTab = (id: string) => `fw:${id}`;
export const TRY_TAB = "fw-try";

/** The side bar's tree of the frameworks: bases, additions by slot, the
 * trial of a combination. */
export function FrameworksTree({
  fw,
  selected,
  onSelect,
}: {
  fw: Frameworks;
  selected: string | null;
  onSelect: (tab: string) => void;
}) {
  const { view, pieces } = fw;
  const [closed, setClosed] = useState<Set<string>>(() => new Set());
  const modified = (p: FrameworkPiece) => {
    const official = view?.official.find((o) => o.id === p.id);
    return official ? cleaned(official) !== cleaned(p) : true;
  };
  const row = (p: FrameworkPiece) => (
    <ListRow
      key={p.id}
      depth={1}
      icon={<PieceLogo piece={p} />}
      label={p.name || p.id}
      title={textOf(view, p.desc)}
      selected={selected === pieceTab(p.id)}
      current="page"
      onSelect={() => {
        onSelect(pieceTab(p.id));
      }}
      decorations={
        modified(p) ? (
          <span title={t("Modificata")} className="text-vsc-settings-modified">
            M
          </span>
        ) : undefined
      }
    />
  );
  const branch = (key: string, label: string, list: FrameworkPiece[]) => {
    if (list.length === 0) return null;
    const open = !closed.has(key);
    return (
      <ListRow
        key={key}
        twistie={open ? "open" : "closed"}
        label={label}
        description={String(list.length)}
        onSelect={() => {
          setClosed((c) => {
            const next = new Set(c);
            if (open) next.add(key);
            else next.delete(key);
            return next;
          });
        }}
      >
        <List label={label}>{list.map(row)}</List>
      </ListRow>
    );
  };
  const addons = pieces.filter((p) => p.kind === "addon");
  const slots = [...new Set(addons.map((p) => p.slot ?? ""))];
  return (
    <List label={t("Framework")}>
      {branch("bases", t("Basi"), bases(pieces))}
      {slots.map((s) =>
        branch(
          `slot:${s}`,
          t("Aggiunte: {0}", slotLabel(s)),
          addons.filter((p) => (p.slot ?? "") === s),
        ),
      )}
      <ListRow
        icon={<VscBeaker size={16} aria-hidden="true" />}
        label={t("Prova una combinazione")}
        selected={selected === TRY_TAB}
        current="page"
        onSelect={() => {
          onSelect(TRY_TAB);
        }}
      />
    </List>
  );
}

/** Problems of the instructions file, above a framework's settings. */
function FrameworksProblems({ fw }: { fw: Frameworks }) {
  const problem = fw.view?.problem;
  if (!fw.error && !problem) return null;
  return (
    <div className="max-w-[880px] space-y-1 px-[14px] pb-2">
      <ErrorLine error={fw.error} />
      {problem && (
        <ErrorLine
          error={t("Le istruzioni in {0} non sono leggibili: uso quelle ufficiali e non modifico il file.", problem)}
        />
      )}
    </div>
  );
}

/** One base or addition as a group of settings (a tab, or a search result). */
export function PieceGroup({ fw, id, onRemoved }: { fw: Frameworks; id: string; onRemoved: () => void }) {
  const { view, pieces, change } = fw;
  const i = pieces.findIndex((p) => p.id === id);
  const piece = pieces[i];
  if (!piece) return null;
  const description =
    piece.kind === "base" ? (
      <>
        {t(
          "Le istruzioni del Nuovo progetto. Una riga = un comando, eseguito con PowerShell e gli strumenti del kit: la creazione nella cartella projects, il resto nella cartella del progetto.",
        )}{" "}
        {t("{name} = nome del progetto (tra virgolette), {package} = com.example. e il nome in minuscolo.")}
      </>
    ) : (
      t(
        "Si combinano con una base: per ciascuna base i loro comandi, dopo quelli della base. Se un'aggiunta crea il progetto al posto della base (uno starter kit), la creazione è la sua.",
      )
    );
  return (
    <SettingsGroup id={`s-${pieceTab(id)}`} title={piece.name || piece.id} description={description}>
      <FrameworksProblems fw={fw} />
      <PieceItem
        piece={piece}
        official={view?.official.find((p) => p.id === id)}
        all={pieces}
        view={view}
        onChange={(next) => {
          change(pieces.map((x, j) => (j === i ? next : x)));
        }}
        onRemove={() => {
          change(pieces.filter((_, j) => j !== i));
          onRemoved();
        }}
      />
    </SettingsGroup>
  );
}

/** The trial of a combination as a group of settings. */
export function TryGroup({ fw }: { fw: Frameworks }) {
  return (
    <SettingsGroup
      id="s-fw-try"
      title={t("Prova una combinazione")}
      description={t("Che cosa farebbe una combinazione, con le istruzioni come sono qui (anche se non salvate).")}
    >
      <SettingItem title={t("Prova una combinazione")} keywords="wizard framework">
        <TryPlan view={fw.view} pieces={fw.pieces} />
      </SettingItem>
    </SettingsGroup>
  );
}

/** Chooses a base and its additions (one per slot). */
export function useCombination(pieces: FrameworkPiece[]) {
  const [base, setBase] = useState<string>("");
  const [addons, setAddons] = useState<string[]>([]);
  const chosen = bases(pieces).some((b) => b.id === base) ? base : (bases(pieces)[0]?.id ?? "");
  const available = addonsFor(pieces, chosen);
  const picked = addons.filter((a) => available.some((p) => p.id === a));
  const toggle = (id: string, on: boolean) => {
    const slot = pieces.find((p) => p.id === id)?.slot;
    const sameSlot = (a: string) => slot !== undefined && slot !== "" && pieces.find((p) => p.id === a)?.slot === slot;
    setAddons(on ? [...picked.filter((a) => !sameSlot(a)), id] : picked.filter((a) => a !== id));
  };
  return { base: chosen, setBase, addons: picked, toggle, available };
}

/** The plan of a combination, asked again when it changes. */
export function usePlan(pieces: FrameworkPiece[], base: string, addons: string[]) {
  const [plan, setPlan] = useState<FrameworkPlan | null>(null);
  const [error, setError] = useState<string | null>(null);
  const key = JSON.stringify([base, addons]);
  useEffect(() => {
    if (!base) return;
    let gone = false;
    const timer = window.setTimeout(() => {
      frameworksPlan(pieces, base, addons).then(
        (p) => {
          if (gone) return;
          setPlan(p);
          setError(null);
        },
        (e: unknown) => {
          if (gone) return;
          setPlan(null);
          setError(String(e));
        },
      );
    }, 150);
    return () => {
      gone = true;
      window.clearTimeout(timer);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps -- key stands for base and addons
  }, [pieces, key]);
  return { plan, error };
}

/** The additions of a base, grouped by slot, as checkboxes. */
export function AddonChoice({
  view,
  base,
  available,
  addons,
  toggle,
  disabled = false,
}: {
  view: FrameworksView | null;
  base: string;
  available: FrameworkPiece[];
  addons: string[];
  toggle: (id: string, on: boolean) => void;
  disabled?: boolean;
}) {
  if (available.length === 0) return <p className="text-vsc-desc">{t("Nessuna aggiunta per questa base.")}</p>;
  const slots = [...new Set(available.map((p) => p.slot ?? ""))];
  return (
    <div className="space-y-3">
      {slots.map((slot) => (
        <div key={slot}>
          <div className="mb-1 text-[11px] font-semibold tracking-wide text-vsc-desc uppercase">{slotLabel(slot)}</div>
          <div className="space-y-1.5">
            {available
              .filter((p) => (p.slot ?? "") === slot)
              .map((p) => {
                const note = textOf(view, p.on?.[base]?.note);
                return (
                  <CheckboxLabel
                    key={p.id}
                    checked={addons.includes(p.id)}
                    disabled={disabled}
                    onChange={(on) => {
                      toggle(p.id, on);
                    }}
                  >
                    <span className="inline-flex translate-y-0.5 pr-1">
                      <PieceLogo piece={p} size={14} />
                    </span>
                    <span className="font-semibold">{p.name}</span>
                    <span className="text-vsc-desc">
                      {" "}
                      - {textOf(view, p.desc)}
                      {note && ` ${note}`}
                    </span>
                  </CheckboxLabel>
                );
              })}
          </div>
        </div>
      ))}
    </div>
  );
}

function Commands({ title, lines: list }: { title: string; lines: string[] }) {
  if (list.length === 0) return null;
  return (
    <div>
      <div className="mb-1 text-vsc-desc">{title}</div>
      <ol className="m-0 list-decimal space-y-0.5 pl-6 font-mono text-[12px] break-all select-text">
        {list.map((l, i) => (
          <li key={String(i) + l}>{l}</li>
        ))}
      </ol>
    </div>
  );
}

/** What a combination does: tools, database, .env, commands, actions, notes. */
export function PlanView({
  view,
  plan,
  error,
}: {
  view: FrameworksView | null;
  plan: FrameworkPlan | null;
  error: string | null;
}): ReactNode {
  return <PlanDetails view={view} plan={plan} error={error} showEnv />;
}

function PlanDetails({
  view,
  plan,
  error,
  showEnv,
}: {
  view: FrameworksView | null;
  plan: FrameworkPlan | null;
  error: string | null;
  showEnv: boolean;
}): ReactNode {
  const components = useComponents();
  if (error) return <ErrorLine error={error} />;
  if (!plan) return null;
  return (
    <div className="space-y-2.5">
      <div className="flex flex-wrap items-center gap-1">
        <span className="pr-1 text-vsc-desc">{t("Strumenti del kit")}:</span>
        {plan.needs.length === 0 ? "-" : plan.needs.map((id) => <ToolTag key={id} id={id} components={components} />)}
      </div>
      {plan.database && (
        <div className="flex flex-wrap items-center gap-1">
          <span className="pr-1 text-vsc-desc">{t("Database del progetto")}:</span>
          <ToolTag id={plan.database} components={components} />
        </div>
      )}
      {showEnv && plan.env.length > 0 && (
        <div>
          <div className="mb-1 text-vsc-desc">{t("File .env")}</div>
          <ul className="m-0 list-none space-y-0.5 p-0 font-mono text-[12px] select-text">
            {plan.env.map((e) => (
              <li key={e.key}>
                {e.key}={e.value}
              </li>
            ))}
          </ul>
        </div>
      )}
      <Commands title={t("Nella cartella projects")} lines={plan.create} />
      <Commands title={t("Nella cartella del progetto")} lines={plan.steps} />
      {plan.actions.length > 0 && (
        <div>
          <div className="mb-1 text-vsc-desc">{t("Azioni del progetto")}</div>
          <ul className="m-0 list-none space-y-0.5 p-0 text-[12px]">
            {plan.actions.map((a) => (
              <li key={a.name}>
                <span className="font-semibold">{a.name}</span>{" "}
                <span className="font-mono text-vsc-desc">{a.command}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
      {plan.notes.map((n) => (
        <Notice key={n} tone="info">
          {textOf(view, n)}
        </Notice>
      ))}
    </div>
  );
}

/** The plan without the .env, which the wizard shows as fields of its own. */
export function PlanSummary(props: {
  view: FrameworksView | null;
  plan: FrameworkPlan | null;
  error: string | null;
}): ReactNode {
  return <PlanDetails {...props} showEnv={false} />;
}

function TryPlan({ view, pieces }: { view: FrameworksView | null; pieces: FrameworkPiece[] }) {
  const { base, setBase, addons, toggle, available } = useCombination(pieces);
  const { plan, error } = usePlan(pieces, base, addons);
  return (
    <div className="space-y-3">
      <select
        aria-label={t("Base")}
        value={base}
        onChange={(e) => {
          setBase(e.target.value);
        }}
        className="input w-full"
      >
        {bases(pieces).map((b) => (
          <option key={b.id} value={b.id}>
            {b.name}
          </option>
        ))}
      </select>
      <AddonChoice view={view} base={base} available={available} addons={addons} toggle={toggle} />
      <PlanView view={view} plan={plan} error={error} />
    </div>
  );
}
