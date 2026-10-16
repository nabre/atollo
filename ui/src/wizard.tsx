// The new-project wizard (D-83): a base (Laravel, Vite, Electron…), the
// additions that go with it (React, Tailwind CSS, a database…), the name (a
// new folder, or an existing project that is empty or has files but no Git),
// the variables of .env. The summary shows the kit's tools and the commands;
// «Crea il progetto» installs the tools, runs the commands with their output
// here and opens the project. The instructions are in Settings.
import { useEffect, useRef, useState } from "react";
import { VscAdd, VscClose, VscWand } from "react-icons/vsc";
import {
  frameworksCancel,
  frameworksCreate,
  frameworksTargets,
  onFramework,
  type EnvVar,
  type FrameworkEvent,
  type FrameworkTarget,
} from "./api";
import {
  AddonChoice,
  bases,
  PieceLogo,
  PlanSummary,
  textOf,
  useCombination,
  useFrameworks,
  usePlan,
} from "./frameworks";
import { ErrorLine } from "./common";
import { useLoad, useSubscription } from "./hooks";
import { t } from "./i18n";
import { Button, IconButton, Link, Notice, ProgressBar, Radio, SingleEditor, TextField } from "./workbench";

const MAX_LOG = 600;

/** 12px text in a field (the component reads the variable). */
const SMALL = "[--vscode-font-size:12px]";

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="space-y-2">
      <h2 className="text-[15px] font-semibold text-vsc-settings-header">{title}</h2>
      {children}
    </section>
  );
}

type EnvRow = EnvVar & { added: boolean; index: number };

/** The variables of .env: those of the plan (value editable, removable) and
 * those added here. */
function EnvRows({
  rows,
  disabled,
  onValue,
  onRemove,
  onAdd,
}: {
  rows: EnvRow[];
  disabled: boolean;
  onValue: (row: EnvRow, patch: Partial<EnvVar>) => void;
  onRemove: (row: EnvRow) => void;
  onAdd: () => void;
}) {
  return (
    <div className="space-y-1.5">
      {rows.map((r) => (
        <div key={(r.added ? "+" : "") + String(r.index)} className="flex max-w-160 items-center gap-1.5">
          <TextField
            label={t("Variabile")}
            value={r.key}
            readOnly={!r.added}
            disabled={disabled}
            placeholder="CHIAVE"
            mono
            onChange={(key) => {
              onValue(r, { key });
            }}
            className={`w-55 ${SMALL}`}
          />
          <span className="text-vsc-desc">=</span>
          <TextField
            label={t("Valore di {0}", r.key)}
            value={r.value}
            disabled={disabled}
            mono
            onChange={(value) => {
              onValue(r, { value });
            }}
            className={`min-w-0 flex-1 ${SMALL}`}
          />
          <IconButton
            icon={VscClose}
            label={t("Togli {0}", r.key || t("Variabile"))}
            disabled={disabled}
            onClick={() => {
              onRemove(r);
            }}
          />
        </div>
      ))}
      <Button small disabled={disabled} onClick={onAdd}>
        <span className="inline-flex items-center gap-1">
          <VscAdd size={12} aria-hidden="true" />
          {t("Aggiungi una variabile")}
        </span>
      </Button>
    </div>
  );
}

export function NewProjectPage({
  onOpen,
  onSettings,
  onGuide,
}: {
  onOpen: (project: string) => void;
  onSettings: () => void;
  onGuide?: (topic: "feat-projects") => void;
}) {
  const fw = useFrameworks();
  const { view, pieces, load } = fw;
  useEffect(load, [load]);
  const { base, setBase, addons, toggle, available } = useCombination(pieces);
  const { plan, error: planError } = usePlan(pieces, base, addons);
  const [name, setName] = useState("");
  /** Values changed (or null: removed) of the plan's variables, by key. */
  const [envEdits, setEnvEdits] = useState<Record<string, string | null>>({});
  const [envAdded, setEnvAdded] = useState<EnvVar[]>([]);
  const [running, setRunning] = useState(false);
  const [progress, setProgress] = useState<{ step: number; total: number }>({ step: 0, total: 0 });
  const [log, setLog] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [created, setCreated] = useState<string | null>(null);
  const logBox = useRef<HTMLDivElement>(null);

  const { data: targets, reload: loadTargets } = useLoad<FrameworkTarget[]>(frameworksTargets, [], {
    onError: "keep",
  });

  useSubscription(onFramework, (e: FrameworkEvent) => {
    if (e.command !== null || e.line === null) setProgress({ step: e.step, total: e.total });
    const text = e.command !== null ? `> ${e.command}` : e.line;
    if (text !== null) setLog((l) => [...l, text].slice(-MAX_LOG));
  });
  useEffect(() => {
    const box = logBox.current;
    if (box) box.scrollTop = box.scrollHeight;
  }, [log]);

  const envRows: EnvRow[] = [
    ...(plan?.env ?? []).flatMap((e, index) => {
      const edit = envEdits[e.key];
      return edit === null ? [] : [{ key: e.key, value: edit ?? e.value, added: false, index }];
    }),
    ...envAdded.map((e, index) => ({ ...e, added: true, index })),
  ];
  const env: EnvVar[] = envRows.filter((r) => r.key.trim()).map((r) => ({ key: r.key.trim(), value: r.value }));

  const wanted = name.trim().toLowerCase();
  const target = targets.find((x) => x.name.toLowerCase() === wanted);
  const usable = targets.filter((x) => x.state !== "git");

  const create = () => {
    setRunning(true);
    setError(null);
    setCreated(null);
    setLog([]);
    setProgress({ step: 0, total: 0 });
    frameworksCreate(base, addons, target?.name ?? name.trim(), env).then(
      (project) => {
        setRunning(false);
        setCreated(project);
        loadTargets();
      },
      (e: unknown) => {
        setRunning(false);
        setError(String(e));
        loadTargets();
      },
    );
  };

  const title = t("Nuovo progetto");
  const icon = <VscWand size={16} aria-hidden="true" />;
  const ready = base !== "" && wanted !== "" && plan !== null && target?.state !== "git" && !running;
  return (
    <SingleEditor title={title} icon={icon}>
      <ProgressBar active={running} />
      <div className="min-h-0 flex-1 overflow-y-auto">
        <div className="max-w-[960px] space-y-6 p-5">
          <p className="text-vsc-desc">
            {t(
              "Scegli una base e le aggiunte: il kit installa gli strumenti che servono ed esegue i comandi ufficiali.",
            )}{" "}
            <Link onClick={onSettings}>{t("Modifica le istruzioni")}</Link>
            {onGuide && (
              <>
                {" · "}
                <Link
                  onClick={() => {
                    onGuide("feat-projects");
                  }}
                >
                  {t("Guida")}
                </Link>
              </>
            )}
          </p>
          <ErrorLine error={fw.error} />

          <Section title={t("1. Base")}>
            <div
              role="radiogroup"
              aria-label={t("Base")}
              className="grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-2"
            >
              {bases(pieces).map((b) => {
                const on = b.id === base;
                const pick = () => {
                  setBase(b.id);
                };
                return (
                  // A card around a radio: the whole card picks it with the
                  // mouse; the keyboard moves on the radios (arrows).
                  <div
                    key={b.id}
                    onClick={running ? undefined : pick}
                    className={`rounded-md border p-3 ${
                      running ? "opacity-60" : "cursor-pointer hover:bg-vsc-list-hover"
                    } ${on ? "border-vsc-focus bg-vsc-list-inactive" : "border-vsc-border"}`}
                  >
                    <Radio name="wizard-base" checked={on} disabled={running} onSelect={pick} plain>
                      <span className="flex items-center gap-2 font-semibold">
                        <PieceLogo piece={b} size={18} />
                        {b.name}
                      </span>
                      {/* descriptionForeground is 4.4:1 on the dark selection: the foreground there */}
                      <span className={`mt-0.5 block font-normal ${on ? "text-vsc-fg" : "text-vsc-desc"}`}>
                        {textOf(view, b.desc)}
                      </span>
                    </Radio>
                  </div>
                );
              })}
            </div>
          </Section>

          <Section title={t("2. Aggiunte")}>
            <AddonChoice
              view={view}
              base={base}
              available={available}
              addons={addons}
              toggle={toggle}
              disabled={running}
            />
          </Section>

          <Section title={t("3. Nome")}>
            <TextField
              label={t("Nome del progetto")}
              value={name}
              disabled={running}
              placeholder={t("nome-del-progetto")}
              onChange={setName}
              className="w-full max-w-[420px]"
            />
            {usable.length > 0 && (
              <div className="flex flex-wrap items-center gap-1.5">
                <span className="text-vsc-desc">{t("Oppure usa un progetto esistente:")}</span>
                {/* Toggle chips (aria-pressed), drawn as the app's tags: no
                    vscode-elements component has a pressed state. */}
                {usable.map((x) => (
                  <button
                    key={x.name}
                    type="button"
                    disabled={running}
                    aria-pressed={target?.name === x.name}
                    title={x.state === "empty" ? t("Cartella vuota") : t("Cartella con file, senza Git")}
                    onClick={() => {
                      setName(x.name);
                    }}
                    className={`rounded-sm border px-1.5 text-[12px] leading-5 hover:bg-vsc-list-hover ${
                      target?.name === x.name ? "border-vsc-focus" : "border-vsc-border"
                    } ${x.state === "files" ? "border-dashed" : ""}`}
                  >
                    {x.name}
                    <span className="pl-1 text-vsc-desc">{x.state === "empty" ? t("vuoto") : t("con file")}</span>
                  </button>
                ))}
              </div>
            )}
            {target?.state === "empty" && (
              <Notice tone="info">{t("Il progetto {0} è vuoto: il framework si crea dentro.", target.name)}</Notice>
            )}
            {target?.state === "files" && (
              <Notice tone="warning">
                {t(
                  "Il progetto {0} contiene già dei file: quelli del framework si aggiungono, i file che ci sono già restano come sono.",
                  target.name,
                )}
              </Notice>
            )}
            {target?.state === "git" && (
              <Notice tone="error">{t("{0} è già un repository Git: scegli un altro nome.", target.name)}</Notice>
            )}
          </Section>

          <Section title={t("4. File .env")}>
            <p className="text-vsc-desc">
              {t("Il kit le scrive nel file .env del progetto. {name} e {package} si riempiono con il nome.")}
            </p>
            <EnvRows
              rows={envRows}
              disabled={running}
              onValue={(r, patch) => {
                if (r.added) {
                  setEnvAdded((list) => list.map((e, i) => (i === r.index ? { ...e, ...patch } : e)));
                } else if (patch.value !== undefined) {
                  const value = patch.value;
                  setEnvEdits((edits) => ({ ...edits, [r.key]: value }));
                }
              }}
              onRemove={(r) => {
                if (r.added) setEnvAdded((list) => list.filter((_, i) => i !== r.index));
                else setEnvEdits((edits) => ({ ...edits, [r.key]: null }));
              }}
              onAdd={() => {
                setEnvAdded((list) => [...list, { key: "", value: "" }]);
              }}
            />
          </Section>

          <Section title={t("Riepilogo")}>
            <PlanSummary view={view} plan={plan} error={planError} />
          </Section>

          <div className="flex items-center gap-2">
            <Button variant="primary" disabled={!ready} busy={running} onClick={create}>
              {t("Crea il progetto")}
            </Button>
            {running && (
              <Button
                onClick={() => {
                  void frameworksCancel();
                }}
              >
                {t("Annulla")}
              </Button>
            )}
          </div>

          {(running || log.length > 0) && (
            <Section title={t("Avanzamento")}>
              {running && (
                <p className="text-vsc-desc">
                  {progress.step === 0
                    ? t("Installo gli strumenti del kit che servono…")
                    : t("Comando {0} di {1}", progress.step, progress.total)}
                </p>
              )}
              <div
                ref={logBox}
                role="log"
                aria-label={t("Avanzamento")}
                className="max-h-[320px] overflow-auto rounded border border-vsc-border bg-vsc-input p-2 font-mono text-[12px] whitespace-pre select-text"
              >
                {log.map((l, i) => (
                  <div key={i} className={l.startsWith("> ") ? "font-semibold text-vsc-link" : undefined}>
                    {l}
                  </div>
                ))}
              </div>
            </Section>
          )}

          {error && (
            <div className="whitespace-pre-wrap">
              <ErrorLine error={error} />
            </div>
          )}
          {created && (
            <Notice tone="success">
              {t("Progetto {0} creato.", created)}{" "}
              <Link
                onClick={() => {
                  onOpen(created);
                }}
              >
                {t("Apri il progetto")}
              </Link>
            </Notice>
          )}
        </div>
      </div>
    </SingleEditor>
  );
}
