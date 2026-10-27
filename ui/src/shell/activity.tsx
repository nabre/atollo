// The Activity panel (D-76): at the bottom of every page, as VSCodium's
// panel, what is running or installing in the kit (projects' actions,
// database servers, the server of the local models, web server pools, the
// jobs: installs, updates and VMs being prepared, LEVELS.md §8.6) and its
// output, its steps or its error.
// The status bar's Activity item opens and closes it. `useRunning` is what
// the status bar, the Projects badge and the panel count (UX-V5-01).
import { useEffect, useState } from "react";
import {
  llmStop,
  serverStop,
  serviceStop,
  urlOpen,
  type LlmInfo,
  type RunInfo,
  type ServiceInfo,
  type Job,
  type ServiceState,
  type SetupSnapshot,
  type WebServerPool,
} from "../ipc";
import { t } from "../i18n";
import { alive, RunOutputView, stateText, type RunLogs } from "../projects";
import { SetupProgress } from "../components/setup";
import { isActive, JobDetails, jobStateText, stepText } from "./jobs";
import {
  ActionBar,
  Button,
  codicon,
  Dot,
  type DotTone,
  IconButton,
  InlineConfirm,
  List,
  ListRow,
  Panel,
} from "../workbench";
import { ActionButton, ErrorLine } from "../shared/common";
import { useAction } from "../shared/hooks";
import { usePoll } from "../shared/poll";
import { llmPoll, serversPoll, servicesPoll } from "../shared/polls";

/** A version of a database server that is not stopped: the one it runs, or
 * another one on its own port (`line` set, D-102). */
export interface LiveLine {
  key: string;
  id: ServiceInfo["id"];
  line?: string;
  name: string;
  state: ServiceState;
  port: number;
  log: string;
  message: string | null;
}

/** The versions of the servers that are not stopped, the main one first. */
export function liveLines(services: ServiceInfo[]): LiveLine[] {
  return services.flatMap((s) => {
    const main: LiveLine[] =
      s.state === "stopped"
        ? []
        : [
            {
              key: s.id,
              id: s.id,
              name: s.line ? `${s.name} ${s.line}` : s.name,
              state: s.state,
              port: s.port,
              log: s.log,
              message: s.message,
            },
          ];
    const others = s.others
      .filter((o) => o.state !== "stopped")
      .map((o) => ({
        key: `${s.id}:${o.line}`,
        id: s.id,
        line: o.line,
        name: `${s.name} ${o.line}`,
        state: o.state,
        port: o.port,
        log: o.log,
        message: o.message,
      }));
    return [...main, ...others];
  });
}

const NO_POOLS: WebServerPool[] = [];

/** What runs in the kit, read every few seconds: the projects' live
 * actions, the database versions not stopped (each line, D-102), the
 * server of the local models (D-151) and the web server pools on; `count`
 * is how many, for the status bar and the Projects badge. The reads are
 * the polls the pages of the same things share (shared/polls). */
export function useRunning(runs: Record<string, RunInfo>) {
  const {
    data: services,
    setData: setServices,
    reload: reloadServices,
  } = usePoll<ServiceInfo[]>(servicesPoll, [], { onError: "keep", every: 4000 });
  const { data: servers, reload: reloadPools } = usePoll(serversPoll, null, { onError: "keep", every: 4000 });
  const { data: info, reload: reloadLlm } = usePoll(llmPoll, null, { onError: "keep", every: 4000 });
  const allPools = servers?.pools ?? NO_POOLS;
  const llm: LlmInfo | null = info && info.state !== "stopped" ? info : null;
  const liveRuns = Object.values(runs).filter((r) => alive(r.state));
  const lines = liveLines(services);
  const pools = allPools.filter((p) => p.running);
  return {
    liveRuns,
    lines,
    pools,
    llm,
    count: liveRuns.length + lines.length + pools.length + (llm ? 1 : 0),
    setServices,
    reloadServices,
    reloadPools,
    reloadLlm,
  };
}

export type Running = ReturnType<typeof useRunning>;

type Selected =
  | { kind: "run"; key: string }
  | { kind: "service"; key: string }
  | { kind: "pool"; id: string }
  | { kind: "llm" }
  | { kind: "job"; id: string }
  | null;

function jobTone(j: Job): DotTone {
  if (j.state === "error") return "err";
  if (j.state === "done") return "ok";
  return isActive(j) ? "warn" : "off";
}

function runTone(r: RunInfo): DotTone {
  if (r.state === "error") return "err";
  if (r.state === "starting") return "warn";
  return alive(r.state) ? "ok" : "off";
}

export function ActivityPanel({
  height,
  runs,
  running,
  logs,
  loadLog,
  onStop,
  onForget,
  setup,
  jobs = [],
  focus = null,
  onClose,
}: {
  height: number;
  runs: Record<string, RunInfo>;
  running: Running;
  logs: RunLogs;
  loadLog: (key: string) => void;
  onStop: (key: string) => Promise<void>;
  onForget: (key: string) => void;
  setup: SetupSnapshot | null;
  /** The installs and the VMs' preparations. */
  jobs?: Job[];
  /** A job to show, chosen in the status bar (a new object each time). */
  focus?: { id: string } | null;
  onClose: () => void;
}) {
  const [selected, setSelected] = useState<Selected>(focus ? { kind: "job", id: focus.id } : null);
  // A job chosen again in the status bar is shown (during the render, as
  // React adjusts state to a new prop).
  const [shownFocus, setShownFocus] = useState(focus);
  if (focus !== shownFocus) {
    setShownFocus(focus);
    if (focus) setSelected({ kind: "job", id: focus.id });
  }
  const [confirmStop, setConfirmStop] = useState(false);
  // The stops, with why one failed: a process in use or a port held.
  const { busy, error, run } = useAction();
  const { liveRuns, lines: liveServices, pools, llm, setServices, reloadServices, reloadPools, reloadLlm } = running;

  const list = Object.values(runs).sort((a, b) => b.start - a.start);
  // The jobs going on, then the last five that ended (the newest first).
  const going = jobs.filter(isActive);
  const ended = jobs
    .filter((j) => !isActive(j))
    .slice(-5)
    .reverse();
  const shownJobs = [...going, ...ended];
  const installing = going.length > 0;

  // Nothing chosen: a job going on, else the newest action, else a
  // database, else a pool.
  const current: Selected =
    selected ??
    (going[0]
      ? { kind: "job", id: going[0].id }
      : list[0]
        ? { kind: "run", key: list[0].key }
        : liveServices[0]
          ? { kind: "service", key: liveServices[0].key }
          : llm
            ? { kind: "llm" }
            : pools[0]
              ? { kind: "pool", id: pools[0].id }
              : null);
  const currentRun = current?.kind === "run" ? runs[current.key] : undefined;
  const currentService = current?.kind === "service" ? liveServices.find((s) => s.key === current.key) : undefined;
  const currentPool = current?.kind === "pool" ? pools.find((p) => p.id === current.id) : undefined;

  useEffect(() => {
    if (currentRun) loadLog(currentRun.key);
  }, [currentRun?.key, loadLog]); // eslint-disable-line react-hooks/exhaustive-deps

  const currentJob = current?.kind === "job" ? jobs.find((j) => j.id === current.id) : undefined;
  const nothing =
    list.length === 0 &&
    liveServices.length === 0 &&
    pools.length === 0 &&
    !llm &&
    !installing &&
    shownJobs.length === 0;
  const count = list.length + liveServices.length + pools.length + (llm ? 1 : 0);
  const anyLive = liveRuns.length + liveServices.length + pools.length + (llm ? 1 : 0) > 0;

  /** Stops the actions, the pools, the database versions and the server of
   * the local models, then reads them again (the actions come back through
   * their events). What failed to stop is said, each on its line. */
  const stopAll = () => {
    setConfirmStop(false);
    run("all", async () => {
      const results = await Promise.allSettled([
        ...liveRuns.map((r) => onStop(r.key)),
        ...pools.map((p) => serverStop(p.id)),
        ...liveServices.map((s) => serviceStop(s.id, s.line)),
        ...(llm ? [llmStop()] : []),
      ]);
      reloadServices();
      reloadPools();
      reloadLlm();
      const failed = results.flatMap((r) => (r.status === "rejected" ? [String(r.reason)] : []));
      if (failed.length > 0) {
        const why: unknown = failed.join("\n");
        throw why;
      }
    });
  };

  return (
    <Panel
      height={height}
      tabs={[{ id: "activity", label: t("Attività"), badge: count > 0 ? count : undefined }]}
      active="activity"
      onSelect={() => undefined}
      actions={
        <>
          {anyLive && (
            <IconButton
              icon={codicon("stop-circle")}
              label={t("Ferma tutto")}
              busy={busy === "all"}
              onClick={() => {
                setConfirmStop(true);
              }}
            />
          )}
          <IconButton icon={codicon("close")} label={t("Chiudi il pannello")} onClick={onClose} />
        </>
      }
    >
      {nothing ? (
        <p className="px-5 py-1 text-vsc-desc">{t("niente in corso")}</p>
      ) : (
        <div className="flex min-w-0 flex-1 flex-col">
          {confirmStop && (
            <InlineConfirm
              row
              question={t("Fermare le azioni dei progetti, i database, i modelli locali e i server web in esecuzione?")}
              className="shrink-0 border-b border-vsc-border px-3 py-1"
              onCancel={() => {
                setConfirmStop(false);
              }}
              actions={
                <>
                  <ActionButton
                    icon={codicon("stop-circle")}
                    label={t("Ferma tutto")}
                    primary
                    small
                    onClick={stopAll}
                  />
                  <Button
                    small
                    onClick={() => {
                      setConfirmStop(false);
                    }}
                  >
                    {t("Annulla")}
                  </Button>
                </>
              }
            />
          )}
          {error && (
            <div className="shrink-0 border-b border-vsc-border px-3 py-1">
              <ErrorLine error={error} />
            </div>
          )}
          <div className="flex min-h-0 flex-1">
            <div className="w-[260px] shrink-0 overflow-auto border-r border-vsc-border">
              <List label={t("Attività")}>
                {shownJobs.map((j) => (
                  <ListRow
                    key={j.id}
                    icon={<Dot tone={jobTone(j)} />}
                    label={j.title}
                    description={isActive(j) ? stepText(j) : jobStateText(j)}
                    selected={current?.kind === "job" && current.id === j.id}
                    onSelect={() => {
                      setSelected({ kind: "job", id: j.id });
                    }}
                  />
                ))}
                {liveServices.map((s) => (
                  <ListRow
                    key={s.key}
                    icon={<Dot tone={s.state === "error" ? "err" : s.state === "running" ? "ok" : "warn"} />}
                    label={s.name}
                    description={`127.0.0.1:${String(s.port)}`}
                    selected={current?.kind === "service" && current.key === s.key}
                    onSelect={() => {
                      setSelected({ kind: "service", key: s.key });
                    }}
                  />
                ))}
                {llm && (
                  <ListRow
                    icon={<Dot tone={llm.state === "error" ? "err" : llm.state === "running" ? "ok" : "warn"} />}
                    label={t("Modelli locali")}
                    description={`127.0.0.1:${String(llm.port)}`}
                    selected={current?.kind === "llm"}
                    onSelect={() => {
                      setSelected({ kind: "llm" });
                    }}
                  />
                )}
                {pools.map((p) => (
                  <ListRow
                    key={p.id}
                    icon={<Dot tone="ok" />}
                    label={p.name}
                    description={`127.0.0.1:${String(p.port)}`}
                    selected={current?.kind === "pool" && current.id === p.id}
                    onSelect={() => {
                      setSelected({ kind: "pool", id: p.id });
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
                          icon={codicon("globe")}
                          label={t("Apri nel browser")}
                          onClick={() => {
                            void urlOpen(currentRun.url ?? "");
                          }}
                        />
                      )}
                      {alive(currentRun.state) ? (
                        <IconButton
                          icon={codicon("debug-stop")}
                          label={t("Ferma")}
                          busy={busy === `run:${currentRun.key}`}
                          onClick={() => {
                            run(`run:${currentRun.key}`, () => onStop(currentRun.key));
                          }}
                        />
                      ) : (
                        <IconButton
                          icon={codicon("clear-all")}
                          label={t("Togli")}
                          onClick={() => {
                            setSelected(null);
                            onForget(currentRun.key);
                          }}
                        />
                      )}
                    </ActionBar>
                  </div>
                  {/* Its own subscription: a line renders this view only,
                      and it follows the last line while the user is there. */}
                  <RunOutputView
                    logs={logs}
                    runKey={currentRun.key}
                    label={t("Output di {0}", `${currentRun.project}: ${currentRun.label}`)}
                    className="m-0 min-h-0 flex-1 overflow-auto px-3 pb-2 font-mono text-[12px] leading-[18px] whitespace-pre-wrap select-text"
                  />
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
                        icon={codicon("debug-stop")}
                        label={t("Ferma")}
                        busy={busy === `service:${currentService.key}`}
                        onClick={() => {
                          run(`service:${currentService.key}`, () =>
                            serviceStop(currentService.id, currentService.line).then((list) => {
                              setServices(list);
                              // A poll already on its way would bring back the
                              // state from before the stop: read again.
                              reloadServices();
                            }),
                          );
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
              {current?.kind === "llm" && llm && (
                <div className="space-y-2 overflow-auto px-3 py-2">
                  <div className="flex flex-wrap items-center gap-2">
                    <span className="flex-1">
                      {t("Modelli locali")} · 127.0.0.1:{String(llm.port)}
                    </span>
                    <IconButton
                      icon={codicon("debug-stop")}
                      label={t("Ferma")}
                      busy={busy === "llm"}
                      onClick={() => {
                        run("llm", () => llmStop().then(reloadLlm));
                      }}
                    />
                  </div>
                  <p className="text-[12px] text-vsc-desc">
                    {t("In memoria: {0}", llm.loaded.length ? llm.loaded.join(", ") : t("nessun modello"))}
                  </p>
                  {llm.error && (
                    <pre className="m-0 font-mono text-[12px] whitespace-pre-wrap text-vsc-error">{llm.error}</pre>
                  )}
                </div>
              )}
              {currentPool && (
                <div className="space-y-2 overflow-auto px-3 py-2">
                  <div className="flex flex-wrap items-center gap-2">
                    <span className="flex-1">
                      {currentPool.name} · 127.0.0.1:{String(currentPool.port)}
                    </span>
                    <IconButton
                      icon={codicon("debug-stop")}
                      label={t("Ferma")}
                      busy={busy === `pool:${currentPool.id}`}
                      onClick={() => {
                        run(`pool:${currentPool.id}`, () => serverStop(currentPool.id).then(reloadPools));
                      }}
                    />
                  </div>
                  <p className="text-[12px] text-vsc-desc">
                    {t("VirtualHost collegati")}:{" "}
                    {currentPool.sites.length === 0
                      ? t("nessuno")
                      : currentPool.sites.map((s) => s.serverName).join(", ")}
                  </p>
                </div>
              )}
              {currentJob && (
                <div className="overflow-auto">
                  <JobDetails job={currentJob} />
                  {/* An install's components, each with its outcome and
                      the download to do by hand when one failed. */}
                  {setup &&
                    (currentJob.kind === "install" || currentJob.kind === "update") &&
                    setup.job === currentJob.id && <SetupProgress snapshot={setup} />}
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </Panel>
  );
}
