// Editor extensions by language (src-tauri/src/editor_ext.rs): the kit's
// matrix, read only, in "Linguaggi e strumenti", and the extensions of a
// project in its panel: those of its languages, ticked unless removed, and
// those added by hand. Installed extensions work in the whole editor.
import { useEffect, useMemo, useState, type SyntheticEvent } from "react";
import { VscAdd, VscCheck } from "react-icons/vsc";
import {
  extensionsMatrix,
  projectExtensions,
  type ComponentInfo,
  type ExtensionsMatrix,
  type ProjectInfo,
} from "./api";
import { t } from "./i18n";
import { ComponentLogo, LanguageLogo } from "./logos";
import { ActionButton, ErrorLine, useAction } from "./pages";

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
            "Dev-Kit abbina a ogni linguaggio le sue estensioni: un progetto riceve quelle dei suoi linguaggi e dal suo pannello puoi aggiungerne o toglierne. Le generali valgono per tutti.",
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
          <table className="border-collapse text-[12px]">
            <caption className="sr-only">{t("Matrice linguaggio-estensione")}</caption>
            <thead>
              <tr>
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
              </tr>
            </thead>
            <tbody>
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
            </tbody>
          </table>
        </div>
      </details>
    </section>
  );
}

const ID = /^[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$/;

/** The extensions of a project: tick or untick, add one by its id. */
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
  return (
    <section aria-labelledby={`ext-${p.id}`} className="space-y-1.5">
      <h3 id={`ext-${p.id}`} className="text-[12px] text-vsc-desc">
        {t("Estensioni dell'editor")}
      </h3>
      {p.extensions.length === 0 ? (
        <p className="text-[12px] text-vsc-desc">
          {t("Solo le generali: nessun linguaggio del progetto ne ha altre.")}
        </p>
      ) : (
        <ul className="flex flex-wrap gap-1.5">
          {p.extensions.map((x) => {
            const inputId = `ext-${p.id}-${x.id}`;
            return (
              <li
                key={x.id}
                className={`flex items-center gap-1.5 rounded-sm border px-1.5 py-0.5 text-[12px] ${
                  x.on ? "border-vsc-border" : "border-dashed border-vsc-border text-vsc-desc"
                }`}
                title={x.id}
              >
                <input
                  id={inputId}
                  type="checkbox"
                  checked={x.on}
                  disabled={busy !== null}
                  onChange={(e) => {
                    toggle(x.id, x.byDefault, e.target.checked);
                  }}
                  className="size-3.5 accent-vsc-button"
                />
                <label htmlFor={inputId}>
                  {x.name}
                  {!x.byDefault && <span className="ml-1 text-vsc-desc">{t("(aggiunta)")}</span>}
                </label>
                {x.on &&
                  (x.installed ? (
                    <span className="text-vsc-success" title={t("installata")}>
                      <VscCheck size={11} aria-hidden="true" />
                      <span className="sr-only">{t("installata")}</span>
                    </span>
                  ) : (
                    <span className="text-[11px] text-vsc-desc">{t("da installare")}</span>
                  ))}
              </li>
            );
          })}
        </ul>
      )}
      <form onSubmit={add} className="flex flex-wrap items-center gap-2" aria-label={t("Aggiungi un'estensione")}>
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
        <span className="text-[12px] text-vsc-desc">
          {t("Le estensioni installate valgono in tutto l'editor; le nuove si installano quando l'editor è aperto.")}
        </span>
      </form>
      {invalid && <ErrorLine error={t("Scrivi l'identificativo come editore.nome, per esempio ms-python.python.")} />}
      <ErrorLine error={error} />
    </section>
  );
}
