// VSCodium's controls, with its sizes (workbench/browser, base/browser/ui):
// text buttons 26px high, action icons 22px, inputs 26px, checkboxes 18px,
// count badges, the 2px progress bar of the views, inline notices.
import { useId, type ReactNode } from "react";
import type { IconType } from "react-icons";
import { VscCheck, VscError, VscInfo, VscSearch, VscWarning } from "react-icons/vsc";

/** A text button (monaco-button): primary, secondary, or the small one of the
 * Extensions list rows. */
export function Button({
  children,
  onClick,
  variant = "secondary",
  small = false,
  busy = false,
  disabled = false,
  submit = false,
  wide = false,
  title,
  className = "",
}: {
  children: ReactNode;
  onClick?: () => void;
  variant?: "primary" | "secondary";
  /** 11px, 18px high, as "Install" in the Extensions view. */
  small?: boolean;
  busy?: boolean;
  disabled?: boolean;
  /** The submit button of its form (onClick is not needed). */
  submit?: boolean;
  /** As wide as its container, as the buttons of a welcome view. */
  wide?: boolean;
  title?: string | undefined;
  className?: string;
}) {
  return (
    <button
      type={submit ? "submit" : "button"}
      onClick={submit ? undefined : onClick}
      disabled={disabled || busy}
      aria-busy={busy}
      title={title}
      className={[
        "inline-flex shrink-0 items-center justify-center gap-1 rounded border whitespace-nowrap",
        "border-vsc-button-border disabled:cursor-not-allowed disabled:opacity-50",
        small ? "h-[18px] px-[5px] text-[11px] leading-4" : "min-h-[26px] px-[11px] py-1 leading-[18px]",
        wide ? "w-full" : "",
        variant === "primary"
          ? "bg-vsc-button text-vsc-button-fg enabled:hover:bg-vsc-button-hover"
          : "bg-vsc-button2 text-vsc-button2-fg enabled:hover:bg-vsc-button2-hover",
        className,
      ].join(" ")}
    >
      {busy ? <span className="animate-pulse">{children}</span> : children}
    </button>
  );
}

/** An icon action (action-item): 22px, a rounded hover, the label as its
 * accessible name and tooltip. */
export function IconButton({
  icon: Icon,
  label,
  onClick,
  busy = false,
  disabled = false,
  checked,
  title,
  className = "",
}: {
  icon: IconType;
  label: string;
  onClick: () => void;
  busy?: boolean;
  disabled?: boolean;
  /** A toggle action: pressed state. */
  checked?: boolean;
  title?: string | undefined;
  className?: string;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      title={title ?? label}
      aria-pressed={checked}
      aria-busy={busy}
      disabled={disabled || busy}
      onClick={onClick}
      className={[
        "flex size-[22px] shrink-0 items-center justify-center rounded text-vsc-icon",
        "enabled:hover:bg-vsc-toolbar-hover disabled:opacity-40",
        checked ? "bg-vsc-option-active outline-1 -outline-offset-1 outline-vsc-option-active-border" : "",
        className,
      ].join(" ")}
    >
      <Icon size={16} aria-hidden="true" className={busy ? "animate-pulse" : undefined} />
    </button>
  );
}

/** A row of icon actions (monaco-action-bar). */
export function ActionBar({ children, className = "" }: { children: ReactNode; className?: string }) {
  return (
    <div role="toolbar" className={`flex shrink-0 items-center gap-0.5 ${className}`}>
      {children}
    </div>
  );
}

/** The filter box at the top of a list (Extensions, Settings): search icon
 * on the left, the count on the right. */
export function SearchBox({
  value,
  onChange,
  placeholder,
  count,
  large = false,
  className = "",
}: {
  value: string;
  onChange: (v: string) => void;
  placeholder: string;
  count?: string | undefined;
  /** The Settings editor's search: 28px. */
  large?: boolean;
  className?: string;
}) {
  return (
    <div className={`relative flex items-center ${className}`}>
      <input
        type="search"
        aria-label={placeholder}
        placeholder={placeholder}
        value={value}
        onChange={(e) => {
          onChange(e.target.value);
        }}
        className={`input w-full pl-[26px] ${count ? "pr-24" : ""} ${large ? "h-[28px]! text-[13px]" : ""}`}
      />
      <VscSearch
        size={14}
        aria-hidden="true"
        className="pointer-events-none absolute left-[7px] text-vsc-placeholder"
      />
      {count && (
        <span className="pointer-events-none absolute right-2">
          <CountBadge>{count}</CountBadge>
        </span>
      )}
    </div>
  );
}

/** A checkbox as VSCodium's: 18px, rounded, a check codicon. */
export function Checkbox({
  checked,
  onChange,
  label,
  id,
  disabled = false,
  className = "",
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
  /** Accessible name when there is no <label for>. */
  label?: string;
  id?: string;
  disabled?: boolean;
  className?: string;
}) {
  return (
    <span className={`relative inline-flex size-[18px] shrink-0 ${className}`}>
      <input
        id={id}
        type="checkbox"
        aria-label={label}
        checked={checked}
        disabled={disabled}
        onChange={(e) => {
          onChange(e.target.checked);
        }}
        className="peer size-[18px] cursor-pointer appearance-none rounded-[3px] border border-vsc-checkbox-border bg-vsc-checkbox focus-visible:outline-1 focus-visible:outline-offset-0 focus-visible:outline-vsc-focus disabled:cursor-not-allowed disabled:opacity-50"
      />
      <VscCheck
        size={16}
        aria-hidden="true"
        className="pointer-events-none absolute top-px left-px hidden text-vsc-checkbox-fg peer-checked:block"
      />
    </span>
  );
}

/** A checkbox with its text on the right (the boolean setting). */
export function CheckboxLabel({
  checked,
  onChange,
  disabled = false,
  children,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
  disabled?: boolean;
  children: ReactNode;
}) {
  const id = useId();
  return (
    <span className="flex items-start gap-2">
      <Checkbox id={id} checked={checked} onChange={onChange} disabled={disabled} />
      <label htmlFor={id} className="min-w-0 cursor-pointer leading-[18px]">
        {children}
      </label>
    </span>
  );
}

/** The round count of a view or a filter (badge.background). */
export function CountBadge({ children, className = "" }: { children: ReactNode; className?: string }) {
  return (
    <span
      className={`inline-flex h-[19px] min-w-[19px] items-center justify-center rounded-full bg-vsc-badge px-1.5 text-[10px] leading-none font-normal text-vsc-badge-fg ${className}`}
    >
      {children}
    </span>
  );
}

/** The thin bar at the top of a view while it works (progress-bit). */
export function ProgressBar({ active }: { active: boolean }) {
  if (!active) return null;
  return (
    <div role="progressbar" aria-busy="true" className="relative h-0.5 w-full shrink-0 overflow-hidden">
      <span className="absolute inset-y-0 w-[2%] animate-[progress_4s_linear_infinite] bg-vsc-progress" />
    </div>
  );
}

const NOTICE = {
  info: { icon: VscInfo, color: "text-vsc-info" },
  warning: { icon: VscWarning, color: "text-vsc-warning" },
  error: { icon: VscError, color: "text-vsc-error" },
  success: { icon: VscCheck, color: "text-vsc-success" },
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

/** A text link (textLink.foreground). */
export function Link({ onClick, children }: { onClick: () => void; children: ReactNode }) {
  return (
    <button type="button" onClick={onClick} className="text-vsc-link hover:underline">
      {children}
    </button>
  );
}

/** A state dot: green running, orange starting, red error, gray stopped. */
export function Dot({ tone, label }: { tone: "ok" | "warn" | "err" | "off"; label?: string | undefined }) {
  const color = { ok: "bg-vsc-success", warn: "bg-vsc-warning", err: "bg-vsc-error", off: "bg-vsc-desc" }[tone];
  return (
    <span
      role={label ? "img" : undefined}
      aria-label={label}
      aria-hidden={label ? undefined : true}
      title={label}
      className={`inline-block size-2 shrink-0 rounded-full ${color}`}
    />
  );
}
