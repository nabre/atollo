// The Activity panel (D-76): at the bottom of every page, what is running or
// installing in the kit (projects' actions, database servers, the setup)
// and its output or error, in one place. Closed, it is one line of pills.
import { useEffect, useRef, useState } from "react";
import { VscChevronDown, VscChevronUp, VscDebugStop, VscGlobe, VscTrash } from "react-icons/vsc";
import { servicesList, serviceStop, urlOpen, type RunInfo, type ServiceInfo, type SetupSnapshot } from "./api";
import { t } from "./i18n";
import { ActionButton } from "./pages";
import { alive, stateText } from "./projects";
import { SetupProgress, setupStatusText } from "./setup";

type Selected = { kind: "run"; key: string } | { kind: "service"; id: string } | { kind: "setup" } | null;

function Dot({ tone }: { tone: "ok" | "warn" | "err" | "off" }) {
  const color = { ok: "text-vsc-success", warn: "text-vsc-warning", err: "text-vsc-error", off: "text-vsc-desc" }[tone];
  return (
    <span className={color} aria-hidden="true">
      ●
    </span>
  );
}

function runTone(r: RunInfo): "ok" | "warn" | "err" | "off" {
  if (r.state === "error") return "err";
  if (r.state === "starting") return "warn";
  return alive(r.state) ? "ok" : "off";
}

export function ActivityPanel({
  runs,
  logs,
  loadLog,
  onStop,
  onForget,
  setup,
}: {
  runs: Record<string, RunInfo>;
  logs: Record<string, string[]>;
  loadLog: (key: string) => void;
  onStop: (key: string) => void;
  onForget: (key: string) => void;
  setup: SetupSnapshot | null;
}) {
  const [open, setOpen] = useState(false);
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
  const running = list.filter((r) => alive(r.state)).length;
  const errors = list.filter((r) => r.state === "error").length + services.filter((s) => s.state === "error").length;

  // Nothing chosen: the newest error, else the newest action.
  const current: Selected =
    selected ??
    (list[0]
      ? { kind: "run", key: list[0].key }
      : liveServices[0]
        ? { kind: "service", id: liveServices[0].id }
        : null);
  const currentRun = current?.kind === "run" ? runs[current.key] : undefined;
  const currentService = current?.kind === "service" ? services.find((s) => s.id === current.id) : undefined;
  const lines = currentRun ? (logs[currentRun.key] ?? []) : [];

  useEffect(() => {
    if (open && currentRun) loadLog(currentRun.key);
  }, [open, currentRun?.key, loadLog]); // eslint-disable-line react-hooks/exhaustive-deps
  useEffect(() => {
    const el = logRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [lines.length]);

  const nothing = list.length === 0 && liveServices.length === 0 && !installing;

  return (
    <section aria-label={t("Attività")} className="border-t border-vsc-border bg-vsc-sidebar">
      <button
        type="button"
        aria-expanded={open}
        aria-controls="activity-body"
        onClick={() => {
          setOpen(!open);
        }}
        className="flex w-full flex-wrap items-center gap-x-4 gap-y-1 px-4 py-1 text-left text-[12px] hover:bg-vsc-list-hover"
      >
        <span className="font-semibold">{t("Attività")}</span>
        {nothing && <span className="text-vsc-desc">{t("niente in corso")}</span>}
        {running > 0 && (
          <span>
            <Dot tone="ok" /> {t("Progetti avviati: {0}", String(running))}
          </span>
        )}
        {liveServices.filter((s) => s.state === "running").length > 0 && (
          <span>
            <Dot tone="ok" />{" "}
            {t("Database avviati: {0}", String(liveServices.filter((s) => s.state === "running").length))}
          </span>
        )}
        {installing && (
          <span>
            <Dot tone="warn" /> {installing}
          </span>
        )}
        {errors > 0 && (
          <span className="text-vsc-error">
            <Dot tone="err" /> {t("Errori: {0}", String(errors))}
          </span>
        )}
        <span className="ml-auto text-vsc-desc">
          {open ? <VscChevronDown size={14} aria-hidden="true" /> : <VscChevronUp size={14} aria-hidden="true" />}
        </span>
      </button>
      {open && (
        <div
          id="activity-body"
          className="grid h-56 grid-cols-[minmax(12rem,18rem)_minmax(0,1fr)] border-t border-vsc-border"
        >
          <ul aria-label={t("Attività")} className="overflow-auto border-r border-vsc-border py-1">
            {nothing && <li className="px-3 py-1 text-vsc-desc">{t("niente in corso")}</li>}
            {installing && (
              <li>
                <button
                  type="button"
                  className={`flex w-full items-center gap-2 px-3 py-1 text-left hover:bg-vsc-list-hover ${current?.kind === "setup" ? "bg-vsc-list-active" : ""}`}
                  onClick={() => {
                    setSelected({ kind: "setup" });
                  }}
                >
                  <Dot tone="warn" />
                  <span className="truncate">{t("Installazione")}</span>
                </button>
              </li>
            )}
            {liveServices.map((s) => (
              <li key={s.id}>
                <button
                  type="button"
                  className={`flex w-full items-center gap-2 px-3 py-1 text-left hover:bg-vsc-list-hover ${current?.kind === "service" && current.id === s.id ? "bg-vsc-list-active" : ""}`}
                  onClick={() => {
                    setSelected({ kind: "service", id: s.id });
                  }}
                >
                  <Dot tone={s.state === "error" ? "err" : s.state === "running" ? "ok" : "warn"} />
                  <span className="truncate">{s.name}</span>
                </button>
              </li>
            ))}
            {list.map((r) => (
              <li key={r.key}>
                <button
                  type="button"
                  className={`flex w-full items-center gap-2 px-3 py-1 text-left hover:bg-vsc-list-hover ${current?.kind === "run" && current.key === r.key ? "bg-vsc-list-active" : ""}`}
                  onClick={() => {
                    setSelected({ kind: "run", key: r.key });
                    loadLog(r.key);
                  }}
                >
                  <Dot tone={runTone(r)} />
                  <span className="truncate">
                    {r.project}: {r.label}
                  </span>
                </button>
              </li>
            ))}
          </ul>
          <div className="flex min-w-0 flex-col">
            {currentRun && (
              <>
                <div className="flex flex-wrap items-center gap-2 border-b border-vsc-border px-3 py-1 text-[12px]">
                  <span className="min-w-0 flex-1 truncate">
                    {currentRun.project}: {currentRun.label} · {stateText(currentRun)}
                  </span>
                  {currentRun.url && currentRun.state === "running" && (
                    <ActionButton
                      icon={VscGlobe}
                      label={t("Apri nel browser")}
                      onClick={() => {
                        void urlOpen(currentRun.url ?? "");
                      }}
                    />
                  )}
                  {alive(currentRun.state) ? (
                    <ActionButton
                      icon={VscDebugStop}
                      label={t("Ferma")}
                      onClick={() => {
                        onStop(currentRun.key);
                      }}
                    />
                  ) : (
                    <ActionButton
                      icon={VscTrash}
                      label={t("Togli")}
                      onClick={() => {
                        setSelected(null);
                        onForget(currentRun.key);
                      }}
                    />
                  )}
                </div>
                <pre
                  ref={logRef}
                  tabIndex={0}
                  aria-label={t("Output di {0}", `${currentRun.project}: ${currentRun.label}`)}
                  className="m-0 min-h-0 flex-1 overflow-auto bg-vsc-bg px-3 py-2 font-mono text-[12px] leading-snug whitespace-pre-wrap select-text"
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
                    <ActionButton
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
                  {t("Log")}: <span className="font-mono">{currentService.log}</span>
                </p>
              </div>
            )}
            {current?.kind === "setup" && setup && (
              <div className="overflow-auto px-3 py-2">
                <SetupProgress snapshot={setup} />
              </div>
            )}
          </div>
        </div>
      )}
    </section>
  );
}
