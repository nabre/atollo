// VSCodium's indicators: badges, the progress bar and spinning
// icons are vscode-elements components; the meter, the notices, the links
// and the state dots are drawn with the workbench's colors.
import type { ReactNode } from "react";
import { t } from "../i18n";
import { codicon } from "./codicon";

/** The round count of a view or a filter (vscode-badge, counter). */
export function CountBadge({ children, className = "" }: { children: ReactNode; className?: string }) {
  return (
    <vscode-badge variant="counter" className={className}>
      {children}
    </vscode-badge>
  );
}

/** The thin bar at the top of a view while it works (vscode-progress-bar,
 * the progress bit crossing it). */
export function ProgressBar({ active }: { active: boolean }) {
  if (!active) return null;
  return <vscode-progress-bar aria-label={t("Caricamento…")} className="shrink-0" />;
}

const METER = {
  none: "bg-vsc-desc",
  ok: "bg-vsc-success",
  warning: "bg-vsc-warning",
  error: "bg-vsc-error",
} as const;

/** How much of something is used (role meter), as a thin bar: the track
 * in the border color, the part used in its tone. Neither VSCodium nor
 * vscode-elements has one: vscode-progress-bar says that work goes on,
 * not a value. */
export function Meter({
  value,
  min = 0,
  max = 100,
  label,
  valueText,
  tone = "none",
  className = "",
}: {
  value: number;
  min?: number;
  max?: number;
  /** Its accessible name. */
  label: string;
  /** The value as read out ("40% used"). */
  valueText?: string | undefined;
  tone?: keyof typeof METER;
  className?: string;
}) {
  const shown = Math.min(max, Math.max(min, value));
  const width = max > min ? ((shown - min) / (max - min)) * 100 : 0;
  return (
    <span
      role="meter"
      aria-label={label}
      aria-valuemin={min}
      aria-valuemax={max}
      aria-valuenow={shown}
      aria-valuetext={valueText}
      className={`block h-vsc-size60 overflow-hidden rounded-vsc-circle bg-vsc-border ${className}`}
    >
      <span className={`block h-full rounded-vsc-circle ${METER[tone]}`} style={{ width: `${String(width)}%` }} />
    </span>
  );
}

/** A Codicon by name in its own component (vscode-icon), for a spinning
 * one; with `label` it is an image with that name, else hidden. The others
 * are Codicon's (codicon.tsx), in the page. */
export function Icon({
  name,
  size = 16,
  spin = false,
  label,
  className = "",
}: {
  name: string;
  size?: number;
  spin?: boolean;
  label?: string | undefined;
  className?: string;
}) {
  return (
    <vscode-icon
      name={name}
      size={size}
      spin={spin}
      role={label ? "img" : undefined}
      aria-label={label}
      aria-hidden={label ? undefined : "true"}
      className={className}
    />
  );
}

const NOTICE = {
  info: { icon: codicon("info"), color: "text-vsc-info" },
  warning: { icon: codicon("warning"), color: "text-vsc-warning" },
  error: { icon: codicon("error"), color: "text-vsc-error" },
  success: { icon: codicon("check"), color: "text-vsc-success" },
} as const;

/** A message with its severity icon, as in the editor's hovers and the
 * settings' validation. Errors are alerts, the rest status. */
export function Notice({
  tone,
  children,
  className = "",
}: {
  tone: keyof typeof NOTICE;
  children: ReactNode;
  className?: string;
}) {
  const { icon: Icon, color } = NOTICE[tone];
  return (
    <div role={tone === "error" ? "alert" : "status"} className={`flex items-start gap-1.5 ${className}`}>
      <Icon size={16} aria-hidden="true" className={`mt-px shrink-0 ${color}`} />
      <div className="min-w-0 select-text">{children}</div>
    </div>
  );
}

/** A text link (textLink.foreground, textLink.activeForeground on hover). */
export function Link({ onClick, children }: { onClick: () => void; children: ReactNode }) {
  return (
    <button type="button" onClick={onClick} className="text-vsc-link hover:text-vsc-link-active hover:underline">
      {children}
    </button>
  );
}

/** The tones of a Dot. */
export type DotTone = "ok" | "warn" | "err" | "off";

/** A state dot: green running, orange starting, red error, gray stopped. */
export function Dot({ tone, label }: { tone: DotTone; label?: string | undefined }) {
  const color = { ok: "bg-vsc-success", warn: "bg-vsc-warning", err: "bg-vsc-error", off: "bg-vsc-desc" }[tone];
  return (
    <span
      role={label ? "img" : undefined}
      aria-label={label}
      aria-hidden={label ? undefined : "true"}
      title={label}
      className={`inline-block size-2 shrink-0 rounded-full ${color}`}
    />
  );
}
