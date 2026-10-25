// The Avvio section of a project's page: its actions, the chosen one with
// its port and its output.
import { useEffect, useRef, useState } from "react";
import { runSetPort, type Ports, type ProjectAction, type ProjectInfo, type RunInfo } from "../ipc";
import { useAction } from "../shared/hooks";
import { ErrorLine, PORT_MAX, PORT_MIN, validPort } from "../shared/common";
import { t } from "../i18n";
import {
  CheckboxLabel,
  Codicon,
  codicon,
  Dot,
  IconButton,
  InlineConfirm,
  Link,
  List,
  ListRow,
  Notice,
  TextField,
} from "../workbench";
import { projectActionDefault, projectActionRemove, urlOpen } from "../ipc";
import { ActionButton } from "../shared/common";
import { ActionBar, EditorSection, Properties } from "../workbench";
import { ActionForm } from "./actionform";
import { alive, stateText } from "./runs";
import { KIND_ICON, kindText } from "./kind";
import { LanAddress, useLanLink } from "./lan";

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
            title={a.static ?? a.apache ?? a.caddy ?? a.command}
            selected={a.id === selected}
            onSelect={() => {
              onSelect(a.id);
            }}
            decorations={
              <>
                {p.default === a.id && (
                  <span role="img" aria-label={t("predefinita")} title={t("predefinita")} className="opacity-70">
                    <Codicon name="star-full" size={12} />
                  </span>
                )}
                {r && <Dot tone={r.state === "error" ? "err" : alive(r.state) ? "ok" : "off"} label={stateText(r)} />}
              </>
            }
            actions={
              alive(r?.state) ? (
                <IconButton
                  icon={codicon("debug-stop")}
                  label={t("Ferma")}
                  busy={busy === `stop:${key}`}
                  onClick={() => {
                    onStop(key);
                  }}
                />
              ) : (
                <IconButton
                  icon={codicon("debug-start")}
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
  onSaved,
}: {
  p: ProjectInfo;
  action: ProjectAction;
  runs: Record<string, RunInfo>;
  logs: Record<string, string[]>;
  ports: Ports;
  onPorts: (p: Ports) => void;
  onForget: (key: string) => void;
  onChanged: () => void;
  /** The action was saved or removed in panel.json: the project is read again. */
  onSaved: (id: string) => void;
}) {
  const { busy: working, error, run } = useAction(onChanged);
  /** The id of the action whose form or removal question is open (the
   * component stays when another action is chosen). */
  const [editing, setEditing] = useState<string | null>(null);
  const [removing, setRemoving] = useState<string | null>(null);
  const removal = useAction(() => {
    setRemoving(null);
    onSaved(a.id);
  });
  const key = `${p.id}|${a.id}`;
  // A site is changed in «Sito web»: its folder and server are not a command.
  const editable = a.id !== "site" && a.static === null && a.apache === null && !a.caddy;
  const r = runs[key];
  const isDefault = p.default === a.id;
  const lan = useLanLink(r?.url && r.state === "running" ? r.url : null);
  const rows: [string, React.ReactNode][] = [
    [t("Stato"), r ? stateText(r) : t("fermo")],
    [
      t("Comando"),
      <span key="cmd" className="font-mono text-[12px] break-all select-text">
        {a.static !== null
          ? t("file server del kit: {0}", a.static)
          : a.apache !== null
            ? t("Apache del progetto: {0}", a.apache)
            : a.caddy
              ? t("Caddy del progetto: {0}", a.caddy)
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
    if (lan) rows.push([t("Dal telefono"), <LanAddress link={lan} />]);
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
            icon={isDefault ? codicon("star-full") : codicon("star-empty")}
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
              icon={codicon("globe")}
              label={t("Apri nel browser")}
              onClick={() => {
                void urlOpen(r.url ?? "");
              }}
            />
          )}
          {r && !alive(r.state) && (
            <IconButton
              icon={codicon("trash")}
              label={t("Togli l'output")}
              onClick={() => {
                onForget(key);
              }}
            />
          )}
          {editable && (
            <IconButton
              icon={codicon("edit")}
              label={t("Modifica")}
              title={a.custom ? undefined : t("Modifica: la tua versione va in panel.json")}
              checked={editing === a.id}
              onClick={() => {
                setRemoving(null);
                setEditing(editing === a.id ? null : a.id);
              }}
            />
          )}
          {a.custom && (
            <IconButton
              icon={codicon("remove")}
              label={t("Togli")}
              title={t("Togli da panel.json")}
              busy={removal.busy === "remove"}
              onClick={() => {
                setEditing(null);
                setRemoving(a.id);
              }}
            />
          )}
        </ActionBar>
      }
    >
      {removing === a.id && (
        <InlineConfirm
          row
          question={t("Togliere {0} da panel.json?", a.label)}
          textClassName="text-vsc-warning"
          onCancel={() => {
            setRemoving(null);
          }}
          actions={
            <>
              <ActionButton
                icon={codicon("remove")}
                label={t("Togli")}
                primary
                busy={removal.busy === "remove"}
                onClick={() => {
                  removal.run("remove", () => projectActionRemove(p.id, a.id));
                }}
              />
              <ActionButton
                icon={codicon("close")}
                label={t("Annulla")}
                onClick={() => {
                  setRemoving(null);
                }}
              />
            </>
          }
        >
          {t("Se Atollo rileva un'azione con lo stesso id, torna quella rilevata.")}
        </InlineConfirm>
      )}
      <ErrorLine error={removal.error} />
      {editable && editing === a.id ? (
        <ActionForm
          p={p}
          action={a}
          onSaved={(id) => {
            setEditing(null);
            onSaved(id);
          }}
          onCancel={() => {
            setEditing(null);
          }}
        />
      ) : (
        <Properties rows={rows} />
      )}
      <ErrorLine error={error} />
      {r?.skippedAliases.map((url) => (
        <Notice key={url} tone="warning">
          {t(
            "L'alias {0} punta fuori dai progetti o a un collegamento, o la sua cartella contiene collegamenti o è troppo grande per controllarla: non servito",
            url,
          )}
        </Notice>
      ))}
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
    if (port !== null && !validPort(port)) {
      setError(t("Porta tra {0} e {1}", PORT_MIN, PORT_MAX));
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
