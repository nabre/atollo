// The Avvio section of a project's page: its actions, the chosen one with
// its port and its output.
import { useEffect, useRef, useState } from "react";
import { VscGlobe } from "react-icons/vsc";
import { runSetPort, type Ports, type ProjectAction, type ProjectInfo, type RunInfo } from "../api";
import { useAction } from "../hooks";
import { ErrorLine } from "../common";
import { t } from "../i18n";
import { CheckboxLabel, Dot, IconButton, Link, List, ListRow, TextField } from "../workbench";
import { VscDebugStart, VscDebugStop, VscStarEmpty, VscStarFull, VscTrash } from "react-icons/vsc";
import { projectActionDefault, urlOpen } from "../api";
import { ActionBar, EditorSection, Properties } from "../workbench";
import { alive, stateText } from "./runs";
import { KIND_ICON, kindText } from "./kind";

/** The actions of the project, as a list: the row selects the action, its
 * inline button starts or stops it, the star marks the default one. */
export function ActionsList({
  p,
  runs,
  busy,
  selected,
  onSelect,
  onStart,
  onStop,
}: {
  p: ProjectInfo;
  runs: Record<string, RunInfo>;
  busy: string | null;
  selected: string | null;
  onSelect: (id: string) => void;
  onStart: (a: ProjectAction) => void;
  onStop: (key: string) => void;
}) {
  if (p.actions.length === 0) {
    return (
      <p className="px-2 text-vsc-desc">
        {t("Nessuna azione riconosciuta: aggiungine in panel.json o servi una cartella come sito web.")}
      </p>
    );
  }
  return (
    <List label={t("Azioni di {0}", p.name)}>
      {p.actions.map((a) => {
        const key = `${p.id}|${a.id}`;
        const r = runs[key];
        const Icon = KIND_ICON[a.type];
        return (
          <ListRow
            key={a.id}
            icon={<Icon size={16} aria-hidden="true" />}
            label={a.label}
            description={r ? stateText(r) : kindText(a.type)}
            title={a.static ?? a.apache ?? a.command}
            selected={a.id === selected}
            onSelect={() => {
              onSelect(a.id);
            }}
            decorations={
              <>
                {p.default === a.id && (
                  <span role="img" aria-label={t("predefinita")} title={t("predefinita")} className="opacity-70">
                    <VscStarFull size={12} aria-hidden="true" />
                  </span>
                )}
                {r && <Dot tone={r.state === "error" ? "err" : alive(r.state) ? "ok" : "off"} label={stateText(r)} />}
              </>
            }
            actions={
              alive(r?.state) ? (
                <IconButton
                  icon={VscDebugStop}
                  label={t("Ferma")}
                  busy={busy === `stop:${key}`}
                  onClick={() => {
                    onStop(key);
                  }}
                />
              ) : (
                <IconButton
                  icon={VscDebugStart}
                  label={t("Avvia")}
                  busy={busy === `run:${key}`}
                  onClick={() => {
                    onSelect(a.id);
                    onStart(a);
                  }}
                />
              )
            }
          />
        );
      })}
    </List>
  );
}

/** The action chosen in the list: what it runs, its port, its output. */
export function ActionDetail({
  p,
  action: a,
  runs,
  logs,
  ports,
  onPorts,
  onForget,
  onChanged,
}: {
  p: ProjectInfo;
  action: ProjectAction;
  runs: Record<string, RunInfo>;
  logs: Record<string, string[]>;
  ports: Ports;
  onPorts: (p: Ports) => void;
  onForget: (key: string) => void;
  onChanged: () => void;
}) {
  const { busy: working, error, run } = useAction(onChanged);
  const key = `${p.id}|${a.id}`;
  const r = runs[key];
  const isDefault = p.default === a.id;
  const rows: [string, React.ReactNode][] = [
    [t("Stato"), r ? stateText(r) : t("fermo")],
    [
      t("Comando"),
      <span key="cmd" className="font-mono text-[12px] break-all select-text">
        {a.static !== null
          ? t("file server del kit: {0}", a.static)
          : a.apache !== null
            ? t("Apache del progetto: {0}", a.apache)
            : a.command}
      </span>,
    ],
  ];
  if (a.folder) rows.push([t("Cartella"), <span className="font-mono text-[12px]">{a.folder}</span>]);
  if (a.type === "web") {
    rows.push([
      t("Porta"),
      // Remounted when the saved port changes: the field shows it.
      <PortField key={`${key}:${String(ports.ports[key] ?? "")}`} actionKey={key} ports={ports} onPorts={onPorts} />,
    ]);
  }
  if (r?.url && r.state === "running") {
    rows.push([
      t("Indirizzo"),
      <Link
        onClick={() => {
          void urlOpen(r.url ?? "");
        }}
      >
        {r.url}
      </Link>,
    ]);
  }
  if (a.custom) rows.push([t("Origine"), "panel.json"]);
  return (
    <EditorSection
      id={`action-${a.id}`}
      title={a.label}
      actions={
        <ActionBar>
          {/* The default action: the project's Avvia starts it. */}
          <IconButton
            icon={isDefault ? VscStarFull : VscStarEmpty}
            label={t("Predefinita")}
            title={t("La avvia il pulsante Avvia del progetto")}
            checked={isDefault}
            busy={working === "default"}
            onClick={() => {
              if (!isDefault) run("default", () => projectActionDefault(p.id, a.id));
            }}
          />
          {r?.url && r.state === "running" && (
            <IconButton
              icon={VscGlobe}
              label={t("Apri nel browser")}
              onClick={() => {
                void urlOpen(r.url ?? "");
              }}
            />
          )}
          {r && !alive(r.state) && (
            <IconButton
              icon={VscTrash}
              label={t("Togli l'output")}
              onClick={() => {
                onForget(key);
              }}
            />
          )}
        </ActionBar>
      }
    >
      <Properties rows={rows} />
      <ErrorLine error={error} />
      {r && <LogView title={a.label} lines={logs[key] ?? []} />}
    </EditorSection>
  );
}

/** The port of a web action: automatic (kept between starts) or chosen. */
function PortField({ actionKey, ports, onPorts }: { actionKey: string; ports: Ports; onPorts: (p: Ports) => void }) {
  const current = ports.ports[actionKey];
  const locked = ports.locked.includes(actionKey);
  const [value, setValue] = useState(current === undefined ? "" : String(current));
  const [error, setError] = useState<string | null>(null);
  const save = () => {
    const text = value.trim();
    const port = text === "" ? null : Number(text);
    if (port === (locked ? (current ?? null) : null)) return;
    if (port !== null && (!Number.isInteger(port) || port < 1024 || port > 65535)) {
      setError(t("Porta tra {0} e {1}", 1024, 65535));
      return;
    }
    runSetPort(actionKey, port).then(
      (p) => {
        setError(null);
        onPorts(p);
      },
      (e: unknown) => {
        setError(String(e));
      },
    );
  };
  return (
    // Enter in the field submits its form: the port is saved.
    <form
      onSubmit={(e) => {
        e.preventDefault();
        save();
      }}
    >
      <TextField
        label={t("Porta")}
        inputMode="numeric"
        title={
          locked
            ? t("Porta scelta da te")
            : t("Porta automatica, la stessa a ogni avvio. Scrivi un numero per sceglierla.")
        }
        placeholder={t("automatica")}
        value={value}
        invalid={error !== null}
        onChange={setValue}
        onBlur={save}
        className="w-24"
      />
      {error && (
        <p role="alert" className="text-[12px] text-vsc-error">
          {error}
        </p>
      )}
    </form>
  );
}

/** The output of an action, as the editor's Output view. */
function LogView({ title, lines }: { title: string; lines: string[] }) {
  const ref = useRef<HTMLPreElement>(null);
  const [follow, setFollow] = useState(true);
  useEffect(() => {
    const el = ref.current;
    if (el && follow) el.scrollTop = el.scrollHeight;
  }, [lines, follow]);
  return (
    <div className="overflow-hidden rounded border border-vsc-border">
      <div className="flex h-[26px] items-center gap-2 border-b border-vsc-border px-3 text-[11px] text-vsc-desc uppercase">
        <span className="flex-1 truncate">{t("Output di {0}", title)}</span>
        <span className="normal-case">
          <CheckboxLabel checked={follow} onChange={setFollow}>
            {t("Segui")}
          </CheckboxLabel>
        </span>
      </div>
      <pre
        ref={ref}
        tabIndex={0}
        aria-label={t("Output di {0}", title)}
        className="m-0 h-64 overflow-auto px-3 py-2 font-mono text-[12px] leading-[18px] whitespace-pre-wrap select-text"
      >
        {lines.join("\n")}
      </pre>
    </div>
  );
}
