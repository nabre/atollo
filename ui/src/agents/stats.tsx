// Plan limits and usage of each agent (Agents page), from
// tauri/src/stats: every limit as "% used" with the time it resets,
// tokens of today and of the last 7 days, sessions, last activity.
import { useCallback, useEffect, useRef, useState } from "react";
import {
  agentsStats,
  type AgentStats,
  type Level,
  type LimitWindow,
  type StatsReport,
  type Tokens,
  type Usage,
} from "../api";
import { locale, t } from "../i18n";
import { ActionButton } from "../common";
import { Badge } from "../tags";
import { Codicon, codicon } from "../workbench";

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

const TEXT: Record<Level, string> = {
  none: "",
  ok: "",
  warning: "text-vsc-warning",
  critical: "text-vsc-error",
};

function LimitBar({ w }: { w: LimitWindow }) {
  const p = w.percent ?? 0;
  return (
    <li className="space-y-1 rounded border border-vsc-border px-3 py-2 text-[12px]">
      <div className="flex items-baseline gap-2">
        <span className="min-w-0 flex-1 truncate" title={w.label}>
          {w.label}
        </span>
        <span className={`text-[16px] font-semibold tabular-nums ${TEXT[w.level]}`}>
          {w.percent === null ? "—" : `${String(p)}%`}
        </span>
      </div>
      <span
        role="meter"
        aria-label={w.label}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={p}
        aria-valuetext={t("{0}% usato", String(p))}
        className="block h-1.5 overflow-hidden rounded-full bg-vsc-border"
      >
        <span className={`block h-full rounded-full ${BAR[w.level]}`} style={{ width: `${String(p)}%` }} />
      </span>
      <p className="h-4 text-[11px] text-vsc-desc">{w.resetsAt !== null && t("si azzera {0}", relative(w.resetsAt))}</p>
    </li>
  );
}

/** A figure with its label, as a tile: the big number, today; the week under it. */
function Tile({ label, today, week, title }: { label: string; today: string; week: string; title?: string }) {
  return (
    <div className="min-w-28 rounded border border-vsc-border px-3 py-2" title={title}>
      <p className="text-[11px] text-vsc-desc">
        {label} · {t("Oggi")}
      </p>
      <p className="text-[16px] font-semibold tabular-nums">{today}</p>
      <p className="text-[11px] text-vsc-desc tabular-nums">
        {t("7 giorni")}: {week}
      </p>
    </div>
  );
}

function UsageView({ u }: { u: Usage }) {
  const max = Math.max(1, ...u.days.map((d) => (u.tokens ? d.total : d.sessions)));
  const weekday = new Intl.DateTimeFormat(locale(), { weekday: "short" });
  const models = Object.entries(u.models).sort((a, b) => b[1] - a[1]);
  return (
    <div className="space-y-2 text-[12px]">
      <span className="font-medium">{t("Consumi")}</span>
      <div className="flex flex-wrap items-stretch gap-2">
        {u.tokens && (
          <Tile
            label={t("Token")}
            today={count(u.today.total)}
            week={count(u.week.total)}
            title={`${t("Oggi")}: ${detail(u.today)}
${t("7 giorni")}: ${detail(u.week)}`}
          />
        )}
        <Tile label={t("Sessioni")} today={String(u.sessionsToday)} week={String(u.sessionsWeek)} />
        <figure
          className="m-0 flex items-end gap-1.5 rounded border border-vsc-border px-3 py-2"
          aria-label={u.tokens ? t("Token per giorno") : t("Sessioni per giorno")}
        >
          {u.days.map((d) => {
            const v = u.tokens ? d.total : d.sessions;
            const day = weekday.format(new Date(d.start));
            return (
              <span
                key={d.start}
                className="flex flex-col items-center gap-0.5"
                title={`${day}: ${u.tokens ? count(v) : String(v)}`}
              >
                <span className="flex h-11 w-3.5 items-end rounded-sm bg-vsc-input">
                  <span
                    className="block w-full rounded-sm bg-vsc-button"
                    style={{ height: `${String((v / max) * 100)}%` }}
                  />
                </span>
                <span className="text-[11px] text-vsc-desc">{day}</span>
              </span>
            );
          })}
        </figure>
      </div>
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
    <div className="space-y-4">
      <div className="space-y-2">
        <div className="flex flex-wrap items-center gap-2 text-[12px] text-vsc-desc">
          <span className="font-medium text-vsc-fg">{t("Limiti del piano")}</span>
          {l.plan && <Badge>{l.plan}</Badge>}
          {l.source && <span>· {l.source}</span>}
          {l.updated !== null && <span>· {t("letti {0}", relative(l.updated))}</span>}
        </div>
        {l.error && (
          <p className="flex items-center gap-2 text-[12px] text-vsc-desc">
            <Codicon name="warning" size={14} className="shrink-0 text-vsc-warning" />
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
          <ul className="m-0 grid list-none grid-cols-[repeat(auto-fill,minmax(14rem,1fr))] gap-2 p-0">
            {l.windows.map((w) => (
              <LimitBar key={w.label} w={w} />
            ))}
          </ul>
        )}
        {l.credits && (
          <p className="text-[12px]">
            {t(
              "Crediti extra: {0} di {1}",
              new Intl.NumberFormat(locale(), { style: "currency", currency: l.credits.currency }).format(
                l.credits.used,
              ),
              l.credits.limit === null
                ? "—"
                : new Intl.NumberFormat(locale(), { style: "currency", currency: l.credits.currency }).format(
                    l.credits.limit,
                  ),
            )}
          </p>
        )}
      </div>
      <UsageView u={s.usage} />
    </div>
  );
}

export function StatsRefresh({ loading, onRefresh }: { loading: boolean; onRefresh: () => void }) {
  return (
    <ActionButton
      icon={codicon("refresh")}
      label={t("Aggiorna i consumi")}
      busy={loading}
      title={t("Legge di nuovo limiti e consumi degli agenti")}
      onClick={onRefresh}
    />
  );
}
