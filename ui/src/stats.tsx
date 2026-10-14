// Plan limits and usage of each agent (Agents page), from
// src-tauri/src/stats: every limit as "% used" with the time it resets,
// tokens of today and of the last 7 days, sessions, last activity.
import { useCallback, useEffect, useRef, useState } from "react";
import { VscRefresh, VscWarning } from "react-icons/vsc";
import {
  agentsStats,
  type AgentStats,
  type Level,
  type LimitWindow,
  type StatsReport,
  type Tokens,
  type Usage,
} from "./api";
import { locale, t } from "./i18n";
import { ActionButton } from "./pages";

/** Read again every 2 minutes, as often as the app asks the servers. */
const EVERY = 2 * 60 * 1000;

export function useStats(): {
  report: StatsReport | null;
  error: string | null;
  loading: boolean;
  refresh: () => void;
} {
  const [report, setReport] = useState<StatsReport | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const alive = useRef(true);
  const load = useCallback((force: boolean) => {
    agentsStats(force).then(
      (r) => {
        if (!alive.current) return;
        setReport(r);
        setError(null);
        setLoading(false);
      },
      (e: unknown) => {
        if (!alive.current) return;
        setError(String(e));
        setLoading(false);
      },
    );
  }, []);
  useEffect(() => {
    alive.current = true;
    load(false);
    const timer = window.setInterval(() => {
      load(false);
    }, EVERY);
    return () => {
      alive.current = false;
      window.clearInterval(timer);
    };
  }, [load]);
  return {
    report,
    error,
    loading,
    refresh: () => {
      setLoading(true);
      load(true);
    },
  };
}

const BAR: Record<Level, string> = {
  none: "bg-vsc-desc",
  ok: "bg-vsc-success",
  warning: "bg-vsc-warning",
  critical: "bg-vsc-error",
};

const count = (n: number) =>
  new Intl.NumberFormat(locale(), { notation: "compact", maximumFractionDigits: 1 }).format(n);

const detail = (k: Tokens) =>
  t("nuovi {0} · dalla cache {1} · in uscita {2}", count(k.input), count(k.cached), count(k.output));

/** "in 2 h", "3 days ago": the nearest unit. */
export function relative(ms: number, now = Date.now()): string {
  const f = new Intl.RelativeTimeFormat(locale(), { numeric: "auto" });
  const s = Math.round((ms - now) / 1000);
  const abs = Math.abs(s);
  if (abs < 60) return f.format(s, "second");
  if (abs < 3600) return f.format(Math.round(s / 60), "minute");
  if (abs < 48 * 3600) return f.format(Math.round(s / 3600), "hour");
  return f.format(Math.round(s / 86400), "day");
}

function LimitBar({ w }: { w: LimitWindow }) {
  const p = w.percent ?? 0;
  return (
    <li className="grid grid-cols-[minmax(8rem,14rem)_1fr_3.5rem] items-center gap-3 text-[12px]">
      <span className="truncate" title={w.label}>
        {w.label}
      </span>
      <span
        role="meter"
        aria-label={w.label}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={p}
        aria-valuetext={t("{0}% usato", String(p))}
        className="h-2 overflow-hidden rounded-sm bg-vsc-input"
      >
        <span className={`block h-full ${BAR[w.level]}`} style={{ width: `${String(p)}%` }} />
      </span>
      <span className="text-right tabular-nums">{w.percent === null ? "—" : `${String(p)}%`}</span>
      {w.resetsAt !== null && (
        <span className="col-start-2 col-end-4 -mt-1 text-[11px] text-vsc-desc">
          {t("si azzera {0}", relative(w.resetsAt))}
        </span>
      )}
    </li>
  );
}

function UsageView({ u }: { u: Usage }) {
  const max = Math.max(1, ...u.days.map((d) => (u.tokens ? d.total : d.sessions)));
  const weekday = new Intl.DateTimeFormat(locale(), { weekday: "short" });
  const models = Object.entries(u.models).sort((a, b) => b[1] - a[1]);
  return (
    <div className="flex flex-wrap items-end gap-x-6 gap-y-2 text-[12px]">
      <table className="text-left tabular-nums">
        <caption className="sr-only">{t("Consumi")}</caption>
        <thead className="text-vsc-desc">
          <tr>
            <td />
            <th scope="col" className="px-2 font-normal">
              {t("Oggi")}
            </th>
            <th scope="col" className="px-2 font-normal">
              {t("7 giorni")}
            </th>
          </tr>
        </thead>
        <tbody>
          {u.tokens && (
            <tr>
              <th scope="row" className="pr-2 font-normal text-vsc-desc">
                {t("Token")}
              </th>
              <td className="px-2" title={detail(u.today)}>
                {count(u.today.total)}
              </td>
              <td className="px-2" title={detail(u.week)}>
                {count(u.week.total)}
              </td>
            </tr>
          )}
          <tr>
            <th scope="row" className="pr-2 font-normal text-vsc-desc">
              {t("Sessioni")}
            </th>
            <td className="px-2">{u.sessionsToday}</td>
            <td className="px-2">{u.sessionsWeek}</td>
          </tr>
        </tbody>
      </table>
      <figure className="flex items-end gap-1" aria-label={u.tokens ? t("Token per giorno") : t("Sessioni per giorno")}>
        {u.days.map((d) => {
          const v = u.tokens ? d.total : d.sessions;
          const day = weekday.format(new Date(d.start));
          return (
            <span
              key={d.start}
              className="flex flex-col items-center gap-0.5"
              title={`${day}: ${u.tokens ? count(v) : String(v)}`}
            >
              <span className="flex h-10 w-3 items-end rounded-sm bg-vsc-input">
                <span
                  className="block w-full rounded-sm bg-vsc-button"
                  style={{ height: `${String((v / max) * 100)}%` }}
                />
              </span>
              <span className="text-[10px] text-vsc-desc">{day}</span>
            </span>
          );
        })}
      </figure>
      <div className="text-vsc-desc">
        {u.last !== null && <p>{t("Ultima attività {0}", relative(u.last))}</p>}
        {models.length > 0 && (
          <p className="truncate">
            {models
              .slice(0, 3)
              .map(([m]) => m)
              .join(", ")}
          </p>
        )}
        {!u.tokens && <p>{t("Antigravity non salva i token: solo le sessioni.")}</p>}
      </div>
    </div>
  );
}

/** Limits and usage of one agent. */
export function AgentUsage({ stats: s }: { stats: AgentStats }) {
  const l = s.limits;
  return (
    <div className="mt-3 space-y-2 border-t border-vsc-border pt-2">
      <div className="flex flex-wrap items-center gap-2 text-[12px] text-vsc-desc">
        <span className="font-medium text-vsc-fg">{t("Limiti del piano")}</span>
        {l.plan && <span className="rounded-sm bg-vsc-badge px-1.5 text-[11px] text-vsc-fg">{l.plan}</span>}
        {l.source && <span>· {l.source}</span>}
        {l.updated !== null && <span>· {t("letti {0}", relative(l.updated))}</span>}
      </div>
      {l.error && (
        <p className="flex items-center gap-2 text-[12px] text-vsc-desc">
          <VscWarning size={14} aria-hidden="true" className="shrink-0 text-vsc-warning" />
          <span className="select-text">{l.error}</span>
        </p>
      )}
      {l.note && <p className="text-[12px] text-vsc-desc">{l.note}</p>}
      {l.limited && l.retryAt !== null && (
        <p className="text-[12px] text-vsc-desc">
          {t("Il server chiede di aspettare: nuova lettura {0}.", relative(l.retryAt))}
        </p>
      )}
      {l.windows.length > 0 && (
        <ul className="max-w-xl space-y-2">
          {l.windows.map((w) => (
            <LimitBar key={w.label} w={w} />
          ))}
        </ul>
      )}
      {l.credits && (
        <p className="text-[12px]">
          {t(
            "Crediti extra: {0} di {1}",
            new Intl.NumberFormat(locale(), { style: "currency", currency: l.credits.currency }).format(l.credits.used),
            l.credits.limit === null
              ? "—"
              : new Intl.NumberFormat(locale(), { style: "currency", currency: l.credits.currency }).format(
                  l.credits.limit,
                ),
          )}
        </p>
      )}
      <UsageView u={s.usage} />
    </div>
  );
}

export function StatsRefresh({ loading, onRefresh }: { loading: boolean; onRefresh: () => void }) {
  return (
    <ActionButton
      icon={VscRefresh}
      label={t("Aggiorna i consumi")}
      busy={loading}
      title={t("Legge di nuovo limiti e consumi degli agenti")}
      onClick={onRefresh}
    />
  );
}
