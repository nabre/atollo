// Editor extensions by language (tauri/src/editor_ext.rs): the kit's
// matrix, read only, in "Linguaggi e strumenti", and the extensions of a
// project in its panel: those of its languages, ticked unless removed, and
// those added by hand. Installed extensions work in the whole editor.
import { useEffect, useMemo, useState, type ReactNode, type SyntheticEvent } from "react";
import { VscAdd, VscCheck, VscTools } from "react-icons/vsc";
import {
  extensionsMatrix,
  projectExtensions,
  type ComponentInfo,
  type ExtensionsMatrix,
  type ProjectExtension,
  type ProjectInfo,
} from "./api";
import { t } from "./i18n";
import { ComponentLogo, LanguageLogo } from "./logos";
import { ActionButton, ErrorLine } from "./common";
import { useAction } from "./hooks";
import { DataTable } from "./workbench";

/** The matrix: one row per extension, one column per language or tool, then "all". */
export function ExtensionsMatrixSection({
  projects,
  components,
}: {
  projects: ProjectInfo[];
  /** A new list when the page reloads: the installed marks follow. */
  components: ComponentInfo[];
}) {
  const [matrix, setMatrix] = useState<ExtensionsMatrix | null>(null);
  useEffect(() => {
    extensionsMatrix().then(setMatrix, () => {
      setMatrix(null);
    });
  }, [components]);
  /** Languages and tools of the projects: their columns are highlighted. */
  const used = useMemo(() => new Set(projects.flatMap((p) => [...p.languages, ...p.requires])), [projects]);
  if (!matrix) return null;
  const columns: { key: string; name: string; tool: boolean }[] = [
    ...matrix.languages.map((l) => ({ key: l, name: l, tool: false })),
    ...matrix.tools.map(([id, name]) => ({ key: id, name, tool: true })),
  ];
  const componentName = (id: string) => components.find((c) => c.id === id)?.name ?? id;
  const installed = matrix.extensions.filter((e) => e.installed).length;
  const cell = "border-l border-vsc-border px-0 text-center";
  return (
    <section aria-labelledby="ext-matrix" className="border-t border-vsc-border">
      <header className="px-3 py-1.5">
        <h2 id="ext-matrix" className="text-[13px] font-semibold">
          {t("Estensioni dell'editor")}
        </h2>
        <p className="text-[12px] text-vsc-desc">
          {t(
            "NEXKORA abbina a ogni linguaggio le sue estensioni: un progetto riceve quelle dei suoi linguaggi e dal suo pannello puoi aggiungerne o toglierne. Le generali valgono per tutti.",
          )}
        </p>
      </header>
      <details>
        <summary className="cursor-pointer px-3 py-1.5 text-[12px] text-vsc-desc hover:bg-vsc-list-hover">
          {t("Matrice linguaggio-estensione: {0} estensioni, {1} installate", matrix.extensions.length, installed)}
        </summary>
        <p className="px-4 pb-2 text-[12px] text-vsc-desc">
          {t(
            "● predefinita nei progetti con quel linguaggio · ○ facoltativa, da aggiungere a un progetto · in evidenza i linguaggi dei tuoi progetti",
          )}
        </p>
        <div className="overflow-x-auto px-4 pb-3">
          <DataTable
            caption={t("Matrice linguaggio-estensione")}
            className="border-collapse text-[12px]"
            head={
              <>
                <th
                  scope="col"
                  className="sticky left-0 bg-vsc-bg pr-3 text-left align-bottom font-normal text-vsc-desc"
                >
                  {t("Estensione")}
                </th>
                {[...columns, { key: "", name: t("Generali"), tool: false }].map((c) => (
                  <th
                    key={c.key || "all"}
                    scope="col"
                    className={`${cell} h-28 w-6 align-bottom font-normal ${
                      used.has(c.key) ? "bg-vsc-focus/15 font-semibold text-vsc-fg" : "text-vsc-desc"
                    }`}
                  >
                    <span className="inline-flex rotate-180 items-center gap-1 py-1 [writing-mode:vertical-rl]">
                      <span className="rotate-180">
                        {c.tool ? <ComponentLogo id={c.key} size={12} /> : <LanguageLogo name={c.key} size={12} />}
                      </span>
                      {c.name}
                    </span>
                  </th>
                ))}
                <th scope="col" className="pl-3 text-left align-bottom font-normal text-vsc-desc">
                  <span className="sr-only">{t("Stato")}</span>
                </th>
              </>
            }
          >
            {matrix.extensions.map((e) => {
              const general = e.languages.length === 0 && e.tools.length === 0;
              const mark = (on: boolean) =>
                on ? (
                  <>
                    <span aria-hidden="true">{e.optional ? "○" : "●"}</span>
                    <span className="sr-only">{e.optional ? t("facoltativa") : t("sì")}</span>
                  </>
                ) : null;
              return (
                <tr key={e.id} className="border-t border-vsc-border hover:bg-vsc-list-hover">
                  <th scope="row" className="sticky left-0 bg-vsc-bg py-0.5 pr-3 text-left font-normal" title={e.id}>
                    {e.name}
                    {e.programs.length > 0 && (
                      <span className="ml-1 text-vsc-desc">
                        {t("(con {0})", e.programs.map(componentName).join(", "))}
                      </span>
                    )}
                  </th>
                  {columns.map((c) => (
                    <td key={c.key} className={`${cell} ${used.has(c.key) ? "bg-vsc-focus/15" : ""}`}>
                      {mark(c.tool ? e.tools.includes(c.key) : e.languages.includes(c.key))}
                    </td>
                  ))}
                  <td className={cell}>{mark(general)}</td>
                  <td className="pl-3 whitespace-nowrap">
                    {e.installed && (
                      <span className="text-vsc-success" title={t("installata")}>
                        <VscCheck size={12} aria-hidden="true" />
                        <span className="sr-only">{t("installata")}</span>
                      </span>
                    )}
                  </td>
                </tr>
              );
            })}
          </DataTable>
        </div>
      </details>
    </section>
  );
}

const ID = /^[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$/;

interface ExtGroup {
  key: string;
  title: string;
  icon: ReactNode;
  items: ProjectExtension[];
}

/** The project's extensions grouped by what brings them: each of its
 * languages (alphabetical), its tools, then those added by hand. */
function groups(p: ProjectInfo, matrix: ExtensionsMatrix | null): ExtGroup[] {
  const byLanguage = new Map<string, ProjectExtension[]>();
  const tools: ProjectExtension[] = [];
  const added: ProjectExtension[] = [];
  const languages = [...p.languages].sort((a, b) => a.localeCompare(b, undefined, { sensitivity: "base" }));
  for (const x of p.extensions) {
    if (!x.byDefault) {
      added.push(x);
      continue;
    }
    const row = matrix?.extensions.find((e) => e.id === x.id);
    const language = languages.find((l) => row?.languages.includes(l));
    if (language) byLanguage.set(language, [...(byLanguage.get(language) ?? []), x]);
    else tools.push(x);
  }
  const out: ExtGroup[] = languages
    .filter((l) => byLanguage.has(l))
    .map((l) => ({ key: `l:${l}`, title: l, icon: <LanguageLogo name={l} />, items: byLanguage.get(l) ?? [] }));
  if (tools.length > 0) {
    out.push({ key: "tools", title: t("Strumenti"), icon: <VscTools size={14} aria-hidden="true" />, items: tools });
  }
  if (added.length > 0) {
    out.push({ key: "added", title: t("Aggiunte da te"), icon: <VscAdd size={14} aria-hidden="true" />, items: added });
  }
  return out;
}

/** The extensions of a project, as the editor's Extensions view: grouped
 * rows with a tick, name, id and state; one added by its id at the end. */
export function ProjectExtensions({ project: p, onChanged }: { project: ProjectInfo; onChanged: () => void }) {
  const { busy, error, run } = useAction(onChanged);
  const [matrix, setMatrix] = useState<ExtensionsMatrix | null>(null);
  const [adding, setAdding] = useState("");
  const [invalid, setInvalid] = useState(false);
  useEffect(() => {
    extensionsMatrix().then(setMatrix, () => {
      setMatrix(null);
    });
  }, []);
  const choice = p.settings.extensions ?? { added: [], removed: [] };
  const save = (key: string, added: string[], removed: string[]) => {
    run(key, () => projectExtensions(p.id, added, removed));
  };
  const toggle = (id: string, byDefault: boolean, on: boolean) => {
    if (byDefault) {
      const removed = on ? choice.removed.filter((x) => x !== id) : [...choice.removed, id];
      save(id, choice.added, removed);
    } else {
      save(id, on ? [...choice.added, id] : choice.added.filter((x) => x !== id), choice.removed);
    }
  };
  const add = (e: SyntheticEvent) => {
    e.preventDefault();
    const id = adding.trim().toLowerCase();
    if (!ID.test(id)) {
      setInvalid(true);
      return;
    }
    setInvalid(false);
    setAdding("");
    const known = p.extensions.find((x) => x.id === id);
    if (known?.byDefault) {
      save(
        "add",
        choice.added,
        choice.removed.filter((x) => x !== id),
      );
    } else if (!known) {
      save("add", [...choice.added, id], choice.removed);
    }
  };
  const suggestions = (matrix?.extensions ?? []).filter((e) => !p.extensions.some((x) => x.id === e.id));
  const listId = `ext-suggest-${p.id}`;
  const on = p.extensions.filter((x) => x.on);
  const toInstall = on.filter((x) => !x.installed).length;
  return (
    <div className="space-y-3">
      <p className="px-3 pt-2 text-[12px] text-vsc-desc">
        {t("{0} attive su {1}", on.length, p.extensions.length)}
        {toInstall > 0 && ` · ${t("{0} da installare", toInstall)}`}
        {" · "}
        {t("Le estensioni installate valgono in tutto l'editor; le nuove si installano quando l'editor è aperto.")}
      </p>
      {p.extensions.length === 0 && (
        <p className="px-3 text-vsc-desc">{t("Solo le generali: nessun linguaggio del progetto ne ha altre.")}</p>
      )}
      {groups(p, matrix).map((g) => (
        <section key={g.key} aria-labelledby={`ext-${p.id}-${g.key}`} className="border-t border-vsc-border">
          <h3
            id={`ext-${p.id}-${g.key}`}
            className="flex h-8 items-center gap-1.5 px-3 text-[12px] font-semibold text-vsc-sidebar-title"
          >
            {g.icon}
            {g.title}
            <span className="font-normal">({g.items.length})</span>
          </h3>
          <ul>
            {g.items.map((x) => {
              const inputId = `ext-${p.id}-${x.id}`;
              return (
                <li key={x.id} className="flex h-[26px] items-center gap-2 px-3 hover:bg-vsc-list-hover">
                  <input
                    id={inputId}
                    type="checkbox"
                    checked={x.on}
                    disabled={busy !== null}
                    onChange={(e) => {
                      toggle(x.id, x.byDefault, e.target.checked);
                    }}
                    className="size-3.5 shrink-0 accent-vsc-button"
                  />
                  <label
                    htmlFor={inputId}
                    className={`flex min-w-0 flex-1 items-baseline gap-2 ${x.on ? "" : "text-vsc-desc"}`}
                  >
                    <span className="truncate">{x.name}</span>
                    <span className="truncate font-mono text-[11px] text-vsc-desc">{x.id}</span>
                  </label>
                  <span className="shrink-0 text-[12px]">
                    {!x.on ? (
                      <span className="text-vsc-desc">{t("disattivata")}</span>
                    ) : x.installed ? (
                      <span className="flex items-center gap-1 text-vsc-success">
                        <VscCheck size={12} aria-hidden="true" />
                        {t("installata")}
                      </span>
                    ) : (
                      <span className="text-vsc-warning">{t("da installare")}</span>
                    )}
                  </span>
                </li>
              );
            })}
          </ul>
        </section>
      ))}
      <section aria-labelledby={`ext-${p.id}-add`} className="border-t border-vsc-border">
        <h3
          id={`ext-${p.id}-add`}
          className="flex h-8 items-center px-3 text-[12px] font-semibold text-vsc-sidebar-title"
        >
          {t("Aggiungi un'estensione")}
        </h3>
        <form
          onSubmit={add}
          className="flex flex-wrap items-center gap-2 px-3"
          aria-label={t("Aggiungi un'estensione")}
        >
          <input
            value={adding}
            list={listId}
            placeholder={t("editore.nome da Open VSX")}
            aria-label={t("Estensione da aggiungere")}
            aria-invalid={invalid}
            onChange={(e) => {
              setAdding(e.target.value);
              setInvalid(false);
            }}
            className="w-64 input text-[12px]"
          />
          <datalist id={listId}>
            {suggestions.map((e) => (
              <option key={e.id} value={e.id}>
                {e.name}
              </option>
            ))}
          </datalist>
          <ActionButton
            icon={VscAdd}
            label={t("Aggiungi")}
            busy={busy === "add"}
            disabled={adding.trim() === ""}
            onClick={() => undefined}
            submit
          />
        </form>
        <div className="px-3 pt-1">
          {invalid && (
            <ErrorLine error={t("Scrivi l'identificativo come editore.nome, per esempio ms-python.python.")} />
          )}
          <ErrorLine error={error} />
        </div>
      </section>
    </div>
  );
}
