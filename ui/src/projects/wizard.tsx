// The new-project wizard (D-83): a base (Laravel, Vite, Electron…), the
// additions that go with it (React, Tailwind CSS, a database…), the name (a
// new folder, or an existing project that is empty or has files but no Git),
// the variables of .env. The summary shows the kit's tools and the commands;
// «Crea il progetto» installs the tools, runs the commands with their output
// here and opens the project. A tab of the Projects editor, beside the
// projects' pages (explorer.tsx); the instructions are in Settings. The
// creation goes on on other pages and its progress is here again when the
// page comes back (frameworks/creation.ts, D-190); the choices stay too.
import { useEffect, useRef, useState } from "react";
import { errorText, fixOf, frameworksCancel, frameworksTargets, type EnvVar, type FrameworkTarget } from "../ipc";
import {
  AddonChoice,
  basesByCategory,
  groupOf,
  PieceLogo,
  PlanSummary,
  markSeen,
  startCreation,
  textOf,
  useCombination,
  useCreation,
  usePlan,
  type Frameworks,
} from "../frameworks";
import { ErrorLine } from "../shared/common";
import type { GuideTopicId } from "../guide";
import { useKept, useLoad } from "../shared/hooks";
import { t } from "../i18n";
import {
  Button,
  Codicon,
  codicon,
  EditorBody,
  EditorHeader,
  EditorSection,
  IconButton,
  Link,
  Notice,
  ProgressBar,
  Radio,
  TextField,
  ToggleButton,
} from "../workbench";

/** 12px text in a field (the component reads the variable). */
const SMALL = "[--vscode-font-size:12px]";

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
            placeholder="KEY"
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
            icon={codicon("close")}
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
          <Codicon name="add" size={12} />
          {t("Aggiungi una variabile")}
        </span>
      </Button>
    </div>
  );
}

export function NewProjectWizard({
  fw,
  preset,
  onBase,
  onOpen,
  onCreated,
  onSettings,
  onGuide,
}: {
  /** The frameworks, read by the Projects page for its tree too. */
  fw: Frameworks;
  /** The base picked in the side bar's tree (`at` tells two picks apart). */
  preset: { base: string; at: number } | null;
  /** The base shown, for the tree's selection. */
  onBase: (base: string) => void;
  onOpen: (project: string) => void;
  /** A project was made: the list reads again. */
  onCreated: () => void;
  onSettings?: (() => void) | undefined;
  onGuide?: ((topic: GuideTopicId) => void) | undefined;
}) {
  const { view, pieces } = fw;
  const { base, setBase, addons, setAddons, toggle, available } = useCombination(pieces, "wizard");
  const { plan, error: planError } = usePlan(pieces, base, addons);
  useEffect(() => {
    if (preset) setBase(preset.base);
  }, [preset, setBase]);
  useEffect(() => {
    if (base) onBase(base);
  }, [base, onBase]);
  const [name, setName] = useKept("wizard.name", "");
  const [group, setGroup] = useState<{ id: string; base: string } | null>(null);
  /** Values changed (or null: removed) of the plan's variables, by key. */
  const [envEdits, setEnvEdits] = useKept<Record<string, string | null>>("wizard.envEdits", {});
  const [envAdded, setEnvAdded] = useKept<EnvVar[]>("wizard.envAdded", []);
  const creation = useCreation();
  const { running, log, created } = creation;
  const error = creation.error === null ? null : errorText(creation.error);
  // A refusal of the levels (D-179): what to do about it.
  const fix = creation.error === null ? null : fixOf(creation.error);
  const logBox = useRef<HTMLDivElement>(null);

  const { data: targets } = useLoad<FrameworkTarget[]>(frameworksTargets, [], {
    onError: "keep",
    key: `${String(creation.run)}:${String(running)}`,
  });

  // A creation running when the window was opened again: its choices.
  useEffect(() => {
    if (running && name === "" && creation.name !== "") {
      setBase(creation.base);
      setAddons(creation.addons);
      setName(creation.name);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- once, when the creation is known
  }, [creation.run, running]);
  // Done: the list reads again, the name is the project's now and the
  // next one starts empty.
  useEffect(() => {
    if (!running && creation.run !== creation.seen && (created !== null || creation.error !== null)) {
      markSeen(creation.run);
      if (created !== null) {
        setName("");
        setEnvEdits({});
        setEnvAdded([]);
        onCreated();
      }
    }
  }, [running, creation.run, creation.seen, creation.error, created, setName, setEnvEdits, setEnvAdded, onCreated]);
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
    startCreation(base, addons, target?.name ?? name.trim(), env);
  };

  /** The group of bases shown: the one picked while this base was chosen,
   * else the base's own. */
  const groups = basesByCategory(pieces);
  const shownGroup =
    groups.find((g) => group !== null && group.base === base && g.id === group.id) ??
    groups.find((g) => g.bases.some((b) => b.id === base)) ??
    groups[0];
  const ready = base !== "" && wanted !== "" && plan !== null && target?.state !== "git" && !running;
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <ProgressBar active={running} />
      <EditorHeader
        icon={<Codicon name="wand" size={56} className="text-vsc-icon opacity-80" />}
        title={t("Nuovo progetto")}
        meta={[<span key="what">{groupOf(pieces, base)?.label ?? t("Da una base")}</span>]}
        description={
          <>
            {t(
              "Scegli una base e le aggiunte: il kit installa gli strumenti che servono ed esegue i comandi ufficiali.",
            )}{" "}
            {onSettings && <Link onClick={onSettings}>{t("Modifica le istruzioni")}</Link>}
            {onSettings && onGuide && " · "}
            {onGuide && (
              <Link
                onClick={() => {
                  onGuide("project-new");
                }}
              >
                {t("Guida")}
              </Link>
            )}
          </>
        }
      />
      <EditorBody>
        <ErrorLine error={fw.error} />

        <EditorSection id="wizard-base" title={t("1. Base")}>
          {groups.length > 1 && (
            <div role="group" aria-label={t("Gruppi di basi")} className="flex flex-wrap gap-1.5">
              {groups.map((g) => (
                <ToggleButton
                  key={g.id}
                  disabled={running}
                  pressed={g.id === shownGroup?.id}
                  onClick={() => {
                    setGroup({ id: g.id, base });
                  }}
                >
                  {g.label}
                  <span className="pl-1 opacity-70">{g.bases.length}</span>
                </ToggleButton>
              ))}
            </div>
          )}
          <div
            role="radiogroup"
            aria-label={shownGroup ? t("Base: {0}", shownGroup.label) : t("Base")}
            className="grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-2"
          >
            {(shownGroup?.bases ?? []).map((b) => {
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
        </EditorSection>

        <EditorSection id="wizard-addons" title={t("2. Aggiunte")}>
          <AddonChoice
            view={view}
            base={base}
            available={available}
            addons={addons}
            toggle={toggle}
            disabled={running}
          />
        </EditorSection>

        <EditorSection id="wizard-name" title={t("3. Nome")}>
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
              {usable.map((x) => (
                <ToggleButton
                  key={x.name}
                  disabled={running}
                  pressed={target?.name === x.name}
                  title={x.state === "empty" ? t("Cartella vuota") : t("Cartella con file, senza Git")}
                  onClick={() => {
                    setName(x.name);
                  }}
                  // A folder with files: dashed, as it is not empty.
                  className={x.state === "files" ? "border-dashed" : ""}
                >
                  {x.name}
                  <span className="pl-1 opacity-70">{x.state === "empty" ? t("vuoto") : t("con file")}</span>
                </ToggleButton>
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
        </EditorSection>

        <EditorSection id="wizard-env" title={t("4. File .env")}>
          <p className="text-vsc-desc">
            {t(
              "Il kit le scrive nel file .env del progetto. {name} e {package} si riempiono con il nome, {password} con una password nuova.",
            )}
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
        </EditorSection>

        <EditorSection id="wizard-summary" title={t("Riepilogo")}>
          <PlanSummary view={view} plan={plan} error={planError} />
        </EditorSection>

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
          <EditorSection id="wizard-progress" title={t("Avanzamento")}>
            {running && (
              <p className="text-vsc-desc">
                {t("Progetto {0}", creation.name)}
                {" · "}
                {creation.step === 0
                  ? t("Installo gli strumenti del kit che servono…")
                  : t("Comando {0} di {1}", creation.step, creation.total)}
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
          </EditorSection>
        )}

        {error && (
          <div className="whitespace-pre-wrap">
            <ErrorLine error={error} fix={fix} />
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
      </EditorBody>
    </div>
  );
}
