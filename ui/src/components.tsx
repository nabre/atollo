// "Linguaggi e strumenti": which agents, languages and tools the kit has.
// Same rules as the page of nabre/dev-kit: required components always on,
// dependencies follow both ways (Apache with PHP, Kotlin with Java), a
// component turned off is removed at the next installation. Also used as the
// first-start page, before anything optional is installed.
import { useCallback, useEffect, useMemo, useState } from "react";
import {
  VscCheck,
  VscCloudDownload,
  VscReferences,
  VscRefresh,
  VscTrash,
  VscTypeHierarchySub,
  VscWarning,
} from "react-icons/vsc";
import { componentsApply, componentsList, type ComponentInfo, type Group, type SetupSnapshot } from "./api";
import { locale, rich, t } from "./i18n";
import { ActionButton } from "./pages";
import { SetupProgress } from "./setup";

function groups(): { id: Group; title: string; hint: string }[] {
  return [
    { id: "required", title: t("Obbligatori"), hint: t("Sempre installati: l'editor, la shell del kit e GitHub CLI.") },
    { id: "agents", title: t("Agenti"), hint: t("Facoltativi: il kit funziona anche senza.") },
    {
      id: "languages",
      title: t("Linguaggi e strumenti"),
      hint: t(
        "Scegli quelli che usi: si aggiungono e tolgono quando vuoi. Quelli che servono a un agente o a un progetto si installano da soli.",
      ),
    },
  ];
}

/** Turns `id` on or off following the dependencies both ways. */
export function toggle(list: ComponentInfo[], chosen: Set<string>, id: string, on: boolean): Set<string> {
  const next = new Set(chosen);
  const visit = (cid: string, value: boolean) => {
    const c = list.find((x) => x.id === cid);
    if (!c || c.group === "required" || c.group === "app") return;
    if (value === next.has(cid)) return;
    if (value) {
      next.add(cid);
      c.needs.forEach((n) => {
        visit(n, true);
      });
    } else {
      next.delete(cid);
      list
        .filter((x) => x.needs.includes(cid))
        .forEach((x) => {
          visit(x.id, false);
        });
    }
  };
  visit(id, on);
  return next;
}

const chip =
  "ml-2 inline-flex items-center gap-1 rounded-sm border border-vsc-border px-1 align-middle text-[11px] leading-4 text-vsc-desc";

/** What `c` needs (installed with it) and, when chosen, what needs `c`. */
function Dependencies({ c, list, chosen }: { c: ComponentInfo; list: ComponentInfo[]; chosen: Set<string> }) {
  const name = (id: string) => list.find((x) => x.id === id)?.name ?? id;
  const usedBy = list.filter((x) => chosen.has(x.id) && x.needs.includes(c.id)).map((x) => x.name);
  return (
    <>
      {c.needs.map((n) => (
        <span key={n} className={chip} title={t("Richiede {0}: si installa insieme.", name(n))}>
          <VscTypeHierarchySub size={11} aria-hidden="true" />
          <span className="sr-only">{t("richiede")} </span>
          {name(n)}
        </span>
      ))}
      {usedBy.length > 0 && (
        <span
          className={`${chip} border-dashed`}
          title={t("Serve a {0}: resta installato finché serve.", usedBy.join(", "))}
        >
          <VscReferences size={11} aria-hidden="true" />
          {t("per {0}", usedBy.join(", "))}
        </span>
      )}
    </>
  );
}

function sizeText(mb: number): string {
  if (mb < 1000) return `${String(mb)} MB`;
  const gb = new Intl.NumberFormat(locale(), { minimumFractionDigits: 1, maximumFractionDigits: 1 }).format(mb / 1000);
  return `${gb} GB`;
}

export function ComponentsPage({
  setup,
  firstRun = false,
  onDone,
}: {
  setup: SetupSnapshot | null;
  firstRun?: boolean;
  onDone?: () => void;
}) {
  const [list, setList] = useState<ComponentInfo[] | null>(null);
  const [chosen, setChosen] = useState<Set<string>>(new Set());
  const [error, setError] = useState<string | null>(null);
  const [confirm, setConfirm] = useState(false);
  const running = setup?.running ?? false;

  const load = useCallback(() => {
    componentsList().then(
      (l) => {
        setList(l);
        setChosen(new Set(l.filter((c) => c.chosen || c.group === "required" || c.group === "app").map((c) => c.id)));
      },
      (e: unknown) => {
        setError(String(e));
      },
    );
  }, []);
  useEffect(load, [load]);
  // Reload when an installation ends: versions and "installed" change.
  useEffect(() => {
    if (setup && !setup.running && setup.steps.length > 0) load();
  }, [setup, load]);

  const plan = useMemo(() => {
    const items = list ?? [];
    const install = items.filter((c) => chosen.has(c.id) && !c.installed && c.available);
    const remove = items.filter((c) => !chosen.has(c.id) && c.present);
    return { install, remove, mb: install.reduce((n, c) => n + c.mb, 0) };
  }, [list, chosen]);

  if (error) {
    return (
      <p role="alert" className="flex items-center gap-2 text-vsc-error">
        <VscWarning size={16} aria-hidden="true" />
        {error}
      </p>
    );
  }
  if (!list) return <p className="text-vsc-desc">{t("Caricamento…")}</p>;

  const apply = (updateAll: boolean) => {
    setConfirm(false);
    componentsApply([...chosen], updateAll).then(
      () => onDone?.(),
      (e: unknown) => {
        setError(String(e));
      },
    );
  };
  const changed = plan.install.length > 0 || plan.remove.length > 0;

  return (
    <div className="max-w-4xl space-y-5">
      {firstRun && (
        <div className="space-y-1">
          <h2 className="text-[15px] font-semibold">{t("Benvenuto in Dev-Kit")}</h2>
          <p>
            {rich(
              t(
                "Scegli agenti e linguaggi: verranno scaricati dalle fonti ufficiali, verificati e messi nella cartella {0} del kit. Niente installazione, niente diritti di amministratore.",
              ),
              <span className="font-mono">data</span>,
            )}
          </p>
          <p className="text-vsc-desc">
            {rich(
              t("Le cartelle in {0} sono attendibili per Claude e Codex: apri lì solo codice di cui ti fidi."),
              <span className="font-mono">projects</span>,
            )}
          </p>
        </div>
      )}

      {setup && setup.steps.length > 0 && <SetupProgress snapshot={setup} />}

      {groups().map((g) => {
        // The editor (part of the app) is listed with the required ones.
        const items = list.filter((c) => c.group === g.id || (g.id === "required" && c.group === "app"));
        if (items.length === 0) return null;
        return (
          <section key={g.id} aria-labelledby={`group-${g.id}`} className="rounded border border-vsc-border">
            <header className="border-b border-vsc-border bg-vsc-sidebar px-4 py-2">
              <h2 id={`group-${g.id}`} className="text-[13px] font-semibold">
                {g.title}
              </h2>
              {g.hint && <p className="text-[12px] text-vsc-desc">{g.hint}</p>}
            </header>
            <ul className="divide-y divide-vsc-border">
              {items.map((c) => {
                const required = c.group === "required" || c.group === "app";
                const on = chosen.has(c.id);
                const inputId = `comp-${c.id}`;
                return (
                  <li key={c.id} className="flex items-center gap-3 px-4 py-1.5 hover:bg-vsc-list-hover">
                    <input
                      id={inputId}
                      type="checkbox"
                      checked={on}
                      disabled={required || running || c.noArm64 || (!c.available && !on)}
                      onChange={(e) => {
                        setChosen(toggle(list, chosen, c.id, e.target.checked));
                      }}
                      className="size-4 accent-vsc-button"
                    />
                    <label htmlFor={inputId} className="min-w-0 flex-1">
                      <span className="font-medium">{c.name}</span>
                      <span className="ml-2 text-vsc-desc">{c.desc}</span>
                      <Dependencies c={c} list={list} chosen={chosen} />
                    </label>
                    {c.x64Only && (
                      <span className="text-[12px] text-vsc-desc" title={t("Su PC ARM64 gira in emulazione x64")}>
                        x64
                      </span>
                    )}
                    <span className="w-20 text-right text-[12px] text-vsc-desc">{sizeText(c.mb)}</span>
                    <span className="w-44 truncate text-right text-[12px]" title={c.version ?? undefined}>
                      {c.installed ? (
                        <span className="text-vsc-success">
                          <VscCheck className="mr-1 inline" size={12} aria-hidden="true" />
                          {c.version ?? t("installato")}
                        </span>
                      ) : c.noArm64 ? (
                        <span className="text-vsc-desc">{t("non disponibile su PC ARM64")}</span>
                      ) : (
                        <span className="text-vsc-desc">{t("non installato")}</span>
                      )}
                    </span>
                  </li>
                );
              })}
            </ul>
          </section>
        );
      })}

      <div className="sticky bottom-0 flex flex-wrap items-center gap-3 border-t border-vsc-border bg-vsc-bg py-3">
        {confirm ? (
          <>
            <span role="status">
              {plan.install.length > 0 &&
                t("Da installare: {0} (circa {1}).", plan.install.map((c) => c.name).join(", "), sizeText(plan.mb)) +
                  " "}
              {plan.remove.length > 0 && t("Da rimuovere: {0}.", plan.remove.map((c) => c.name).join(", "))}
            </span>
            <ActionButton
              icon={VscCheck}
              label={t("Conferma")}
              primary
              onClick={() => {
                apply(false);
              }}
            />
            <ActionButton
              icon={VscRefresh}
              label={t("Indietro")}
              onClick={() => {
                setConfirm(false);
              }}
            />
          </>
        ) : (
          <>
            <ActionButton
              icon={
                firstRun ? VscCloudDownload : plan.remove.length > 0 && plan.install.length === 0 ? VscTrash : VscCheck
              }
              label={firstRun ? t("Installa") : t("Applica")}
              primary
              disabled={running || (!changed && !firstRun)}
              title={running ? t("Un'installazione è già in corso") : undefined}
              onClick={() => {
                if (firstRun) apply(false);
                else setConfirm(true);
              }}
            />
            {!firstRun && (
              <ActionButton
                icon={VscRefresh}
                label={t("Aggiorna tutto")}
                disabled={running}
                title={
                  running
                    ? t("Un'installazione è già in corso")
                    : t("Scarica le versioni nuove di tutto ciò che è installato")
                }
                onClick={() => {
                  apply(true);
                }}
              />
            )}
            <span className="text-vsc-desc">
              {changed
                ? t(
                    "{0} da installare ({1}), {2} da rimuovere",
                    plan.install.length,
                    sizeText(plan.mb),
                    plan.remove.length,
                  )
                : t("Nessuna modifica")}
            </span>
          </>
        )}
      </div>
    </div>
  );
}
