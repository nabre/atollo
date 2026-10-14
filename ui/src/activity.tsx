// The Activity panel (D-76): at the bottom of every page, as VSCodium's
// panel, what is running or installing in the kit (projects' actions,
// database servers, the setup) and its output or error. The status bar's
// Activity item opens and closes it.
import { useEffect, useRef, useState } from "react";
import { VscClearAll, VscClose, VscDebugStop, VscGlobe } from "react-icons/vsc";
import { servicesList, serviceStop, urlOpen, type RunInfo, type ServiceInfo, type SetupSnapshot } from "./api";
import { t } from "./i18n";
import { alive, stateText } from "./projects";
import { SetupProgress, setupStatusText } from "./setup";
import { ActionBar, Dot, IconButton, List, ListRow, Panel } from "./workbench";

type Selected = { kind: "run"; key: string } | { kind: "service"; id: string } | { kind: "setup" } | null;

function runTone(r: RunInfo): "ok" | "warn" | "err" | "off" {
  if (r.state === "error") return "err";
  if (r.state === "starting") return "warn";
  return alive(r.state) ? "ok" : "off";
}

export function ActivityPanel({
  height,
  runs,
  logs,
  loadLog,
  onStop,
  onForget,
  setup,
  onClose,
}: {
  height: number;
  runs: Record<string, RunInfo>;
  logs: Record<string, string[]>;
  loadLog: (key: string) => void;
  onStop: (key: string) => void;
  onForget: (key: string) => void;
  setup: SetupSnapshot | null;
  onClose: () => void;
}) {
  const [selected, setSelected] = useState<Selected>(null);
  const [services, setServices] = useState<ServiceInfo[]>([]);
  const logRef = useRef<HTMLPreElement>(null);

  useEffect(() => {
    const read = () => {
      servicesList().then(setServices, () => undefined);
    };
    read();
    const timer = window.setInterval(read, 4000);
    return () => {
      window.clearInterval(timer);
    };
  }, []);

  const list = Object.values(runs).sort((a, b) => b.start - a.start);
  const liveServices = services.filter((s) => s.state !== "stopped");
  const installing = setupStatusText(setup);

  // Nothing chosen: the installation, else the newest action, else a database.
  const current: Selected =
    selected ??
    (installing
      ? { kind: "setup" }
      : list[0]
        ? { kind: "run", key: list[0].key }
        : liveServices[0]
          ? { kind: "service", id: liveServices[0].id }
          : null);
  const currentRun = current?.kind === "run" ? runs[current.key] : undefined;
  const currentService = current?.kind === "service" ? services.find((s) => s.id === current.id) : undefined;
  const lines = currentRun ? (logs[currentRun.key] ?? []) : [];

  useEffect(() => {
    if (currentRun) loadLog(currentRun.key);
  }, [currentRun?.key, loadLog]); // eslint-disable-line react-hooks/exhaustive-deps
  useEffect(() => {
    const el = logRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [lines.length]);

  const nothing = list.length === 0 && liveServices.length === 0 && !installing;
  const count = list.length + liveServices.length;

  return (
    <Panel
      height={height}
      tabs={[{ id: "activity", label: t("Attività"), badge: count > 0 ? count : undefined }]}
      active="activity"
      onSelect={() => undefined}
      actions={<IconButton icon={VscClose} label={t("Chiudi il pannello")} onClick={onClose} />}
    >
      {nothing ? (
        <p className="px-5 py-1 text-vsc-desc">{t("niente in corso")}</p>
      ) : (
        <>
          <div className="w-[260px] shrink-0 overflow-auto border-r border-vsc-border">
            <List label={t("Attività")}>
              {installing && (
                <ListRow
                  icon={<Dot tone="warn" />}
                  label={t("Installazione")}
                  selected={current?.kind === "setup"}
                  onSelect={() => {
                    setSelected({ kind: "setup" });
                  }}
                />
              )}
              {liveServices.map((s) => (
                <ListRow
                  key={s.id}
                  icon={<Dot tone={s.state === "error" ? "err" : s.state === "running" ? "ok" : "warn"} />}
                  label={s.name}
                  description={`127.0.0.1:${String(s.port)}`}
                  selected={current?.kind === "service" && current.id === s.id}
                  onSelect={() => {
                    setSelected({ kind: "service", id: s.id });
                  }}
                />
              ))}
              {list.map((r) => (
                <ListRow
                  key={r.key}
                  icon={<Dot tone={runTone(r)} />}
                  label={`${r.project}: ${r.label}`}
                  description={stateText(r)}
                  selected={current?.kind === "run" && current.key === r.key}
                  onSelect={() => {
                    setSelected({ kind: "run", key: r.key });
                    loadLog(r.key);
                  }}
                />
              ))}
            </List>
          </div>
          <div className="flex min-w-0 flex-1 flex-col">
            {currentRun && (
              <>
                <div className="flex h-[30px] shrink-0 items-center gap-2 pr-2 pl-3 text-[12px]">
                  <span className="min-w-0 flex-1 truncate text-vsc-desc">
                    {currentRun.project}: {currentRun.label} · {stateText(currentRun)}
                    {currentRun.url && currentRun.state === "running" && ` · ${currentRun.url}`}
                  </span>
                  <ActionBar>
                    {currentRun.url && currentRun.state === "running" && (
                      <IconButton
                        icon={VscGlobe}
                        label={t("Apri nel browser")}
                        onClick={() => {
                          void urlOpen(currentRun.url ?? "");
                        }}
                      />
                    )}
                    {alive(currentRun.state) ? (
                      <IconButton
                        icon={VscDebugStop}
                        label={t("Ferma")}
                        onClick={() => {
                          onStop(currentRun.key);
                        }}
                      />
                    ) : (
                      <IconButton
                        icon={VscClearAll}
                        label={t("Togli")}
                        onClick={() => {
                          setSelected(null);
                          onForget(currentRun.key);
                        }}
                      />
                    )}
                  </ActionBar>
                </div>
                <pre
                  ref={logRef}
                  tabIndex={0}
                  aria-label={t("Output di {0}", `${currentRun.project}: ${currentRun.label}`)}
                  className="m-0 min-h-0 flex-1 overflow-auto px-3 pb-2 font-mono text-[12px] leading-[18px] whitespace-pre-wrap select-text"
                >
                  {lines.join("\n")}
                </pre>
              </>
            )}
            {currentService && (
              <div className="space-y-2 overflow-auto px-3 py-2">
                <div className="flex flex-wrap items-center gap-2">
                  <span className="flex-1">
                    {currentService.name} · 127.0.0.1:{String(currentService.port)}
                  </span>
                  {currentService.state === "running" && (
                    <IconButton
                      icon={VscDebugStop}
                      label={t("Ferma")}
                      onClick={() => {
                        serviceStop(currentService.id).then(setServices, () => undefined);
                      }}
                    />
                  )}
                </div>
                {currentService.message && (
                  <pre className="m-0 font-mono text-[12px] whitespace-pre-wrap text-vsc-error">
                    {currentService.message}
                  </pre>
                )}
                <p className="text-[12px] text-vsc-desc">
                  {t("Log")}: <span className="font-mono select-text">{currentService.log}</span>
                </p>
              </div>
            )}
            {current?.kind === "setup" && setup && (
              <div className="overflow-auto">
                <SetupProgress snapshot={setup} />
              </div>
            )}
          </div>
        </>
      )}
    </Panel>
  );
}
