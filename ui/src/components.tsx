// "Linguaggi e strumenti": which agents, languages and tools the kit has.
// Same rules as the page of nabre/dev-kit: required components always on,
// dependencies follow both ways (Apache with PHP, Kotlin with Java), a
// component turned off is removed at the next installation. Also used as the
// first-start page, before anything optional is installed.
import { useCallback, useEffect, useMemo, useState } from "react";
import { VscCheck, VscCloudDownload, VscRefresh, VscTrash, VscWarning } from "react-icons/vsc";
import { componentsApply, componentsList, type ComponentInfo, type Group, type SetupSnapshot } from "./api";
import { ActionButton } from "./pages";
import { SetupProgress } from "./setup";

const GROUPS: { id: Group; title: string; hint: string }[] = [
  { id: "required", title: "Obbligatori", hint: "Sempre installati: servono al kit e agli agenti." },
  { id: "agents", title: "Agenti", hint: "Facoltativi: il kit funziona anche senza." },
  {
    id: "languages",
    title: "Linguaggi e strumenti",
    hint: "Scegli quelli che usi: si aggiungono e tolgono quando vuoi.",
  },
  { id: "optional", title: "Altro", hint: "" },
];

/** Turns `id` on or off following the dependencies both ways. */
export function toggle(list: ComponentInfo[], chosen: Set<string>, id: string, on: boolean): Set<string> {
  const next = new Set(chosen);
  const visit = (cid: string, value: boolean) => {
    const c = list.find((x) => x.id === cid);
    if (!c || c.group === "required") return;
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

function sizeText(mb: number): string {
  return mb >= 1000 ? `${(mb / 1000).toFixed(1).replace(".", ",")} GB` : `${String(mb)} MB`;
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
        setChosen(new Set(l.filter((c) => c.chosen || c.group === "required").map((c) => c.id)));
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
    const remove = items.filter((c) => !chosen.has(c.id) && c.installed);
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
  if (!list) return <p className="text-vsc-desc">Caricamento…</p>;

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
          <h2 className="text-[15px] font-semibold">Benvenuto in Dev-Kit</h2>
          <p>
            Scegli agenti e linguaggi: verranno scaricati dalle fonti ufficiali, verificati e messi nella cartella{" "}
            <span className="font-mono">data</span> del kit. Niente installazione, niente diritti di amministratore.
          </p>
          <p className="text-vsc-desc">
            Le cartelle in <span className="font-mono">projects</span> sono attendibili per Claude e Codex: apri lì solo
            codice di cui ti fidi.
          </p>
        </div>
      )}

      {setup && setup.steps.length > 0 && <SetupProgress snapshot={setup} />}

      {GROUPS.map((g) => {
        const items = list.filter((c) => c.group === g.id);
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
                const required = c.group === "required";
                const on = chosen.has(c.id);
                const inputId = `comp-${c.id}`;
                return (
                  <li key={c.id} className="flex items-center gap-3 px-4 py-1.5 hover:bg-vsc-list-hover">
                    <input
                      id={inputId}
                      type="checkbox"
                      checked={on}
                      disabled={required || running}
                      onChange={(e) => {
                        setChosen(toggle(list, chosen, c.id, e.target.checked));
                      }}
                      className="size-4 accent-vsc-button"
                    />
                    <label htmlFor={inputId} className="min-w-0 flex-1">
                      <span className="font-medium">{c.name}</span>
                      <span className="ml-2 text-vsc-desc">{c.desc}</span>
                      {c.needs.length > 0 && (
                        <span className="ml-2 text-[12px] text-vsc-desc">
                          (richiede {c.needs.map((n) => list.find((x) => x.id === n)?.name ?? n).join(", ")})
                        </span>
                      )}
                    </label>
                    {c.x64Only && (
                      <span className="text-[12px] text-vsc-desc" title="Su PC ARM64 gira in emulazione x64">
                        x64
                      </span>
                    )}
                    <span className="w-20 text-right text-[12px] text-vsc-desc">{sizeText(c.mb)}</span>
                    <span className="w-44 truncate text-right text-[12px]" title={c.version ?? undefined}>
                      {c.installed ? (
                        <span className="text-vsc-success">
                          <VscCheck className="mr-1 inline" size={12} aria-hidden="true" />
                          {c.version ?? "installato"}
                        </span>
                      ) : (
                        <span className="text-vsc-desc">non installato</span>
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
                `Da installare: ${plan.install.map((c) => c.name).join(", ")} (circa ${sizeText(plan.mb)}). `}
              {plan.remove.length > 0 && `Da rimuovere: ${plan.remove.map((c) => c.name).join(", ")}.`}
            </span>
            <ActionButton
              icon={VscCheck}
              label="Conferma"
              primary
              onClick={() => {
                apply(false);
              }}
            />
            <ActionButton
              icon={VscRefresh}
              label="Indietro"
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
              label={firstRun ? "Installa" : "Applica"}
              primary
              disabled={running || (!changed && !firstRun)}
              title={running ? "Un'installazione è già in corso" : undefined}
              onClick={() => {
                if (firstRun) apply(false);
                else setConfirm(true);
              }}
            />
            {!firstRun && (
              <ActionButton
                icon={VscRefresh}
                label="Aggiorna tutto"
                disabled={running}
                title={
                  running
                    ? "Un'installazione è già in corso"
                    : "Scarica le versioni nuove di tutto ciò che è installato"
                }
                onClick={() => {
                  apply(true);
                }}
              />
            )}
            <span className="text-vsc-desc">
              {changed
                ? `${String(plan.install.length)} da installare (${sizeText(plan.mb)}), ${String(plan.remove.length)} da rimuovere`
                : "Nessuna modifica"}
            </span>
          </>
        )}
      </div>
    </div>
  );
}
