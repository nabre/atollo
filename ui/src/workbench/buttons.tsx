// VSCodium's buttons: the text button is vscode-button (its sizes are
// VSCodium 1.135's, here); icon actions and toggles stay native buttons drawn
// as vscode-toolbar-button, so their role, name and state are in the page.
import { useEffect, useRef, type ReactNode } from "react";
import type { IconType } from "react-icons";
import { Codicon } from "./codicon";
import { useHost } from "./elements";

/** VSCodium 1.135's .monaco-text-button on the component's base: 4px 8px
 * around a 16px line of 12px text, 26px high with its border, 4px corners
 * (cornerRadius-small); as wide as its container (block) without the
 * component's 28px. */
const TEXT_BUTTON =
  "[&::part(base)]:min-h-0 [&::part(base)]:rounded-vsc-small [&::part(base)]:px-vsc-size80 [&::part(base)]:py-vsc-size40 [&::part(base)]:text-vsc-body-small [&::part(base)]:leading-(--vscode-spacing-size160)";
/** Its .small: 22px high, 6px at the sides, 11px text (3px above and below
 * a 14px line there; 2px around a 16px one here, the same height). */
const SMALL_BUTTON =
  "[&::part(base)]:min-h-0 [&::part(base)]:rounded-vsc-small [&::part(base)]:px-vsc-size60 [&::part(base)]:py-vsc-size20 [&::part(base)]:text-vsc-body-xsmall [&::part(base)]:leading-(--vscode-spacing-size160)";

/** A text button (vscode-button): primary, secondary, or the small one. */
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
  autoFocus = false,
}: {
  children: ReactNode;
  onClick?: () => void;
  variant?: "primary" | "secondary";
  /** 11px, 22px high (.monaco-text-button.small). */
  small?: boolean;
  busy?: boolean;
  disabled?: boolean;
  /** The submit button of its form (onClick is not needed). */
  submit?: boolean;
  /** As wide as its container, as the buttons of a welcome view. */
  wide?: boolean;
  title?: string | undefined;
  className?: string;
  /** Takes the focus when shown (the safe answer of a dialog). */
  autoFocus?: boolean;
}) {
  const off = disabled || busy;
  const [host, element] = useHost("button", true);
  const native = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    if (autoFocus) element.current?.focus();
  }, [autoFocus, element]);
  return (
    <>
      <vscode-button
        ref={host}
        secondary={variant === "secondary"}
        block={wide}
        disabled={off}
        aria-disabled={off ? "true" : undefined}
        aria-busy={busy ? "true" : undefined}
        title={title}
        // A custom element gets clicks even when disabled; the props say
        // what this button is now (the component's attribute catches up a
        // microtask later).
        onClick={() => {
          if (off) return;
          if (submit) native.current?.click();
          else onClick?.();
        }}
        // The component clicks on the keydown of Enter and Space, as a
        // native button but without its default handling: no page scroll
        // on Space, and one click per press, not one per key repeat.
        onKeyDownCapture={(e) => {
          if (e.key !== " " && e.key !== "Enter") return;
          if (e.key === " ") e.preventDefault();
          if (e.repeat) {
            e.preventDefault();
            e.stopPropagation();
          }
        }}
        // Busy, it keeps its full color (the component fades a disabled
        // button): the spinner says it works, not that it is off.
        className={["shrink-0", small ? SMALL_BUTTON : TEXT_BUTTON, busy ? "opacity-100" : "", className].join(" ")}
      >
        {busy ? (
          // VSCodium's spinning codicon-loading in place of the text, which
          // keeps its width (no shift) and stays the button's name.
          <span className="relative inline-flex items-center justify-center">
            <span className="opacity-0">{children}</span>
            <Codicon name="loading" size={small ? 12 : 16} className="absolute animate-spin" />
          </span>
        ) : (
          children
        )}
      </vscode-button>
      {/* The form's default button, a native one: it submits the clicks
          above, and Enter in the form's native fields, as the browser
          does. A TextField submits by itself, and checks this button's
          disabled state first (guardEnter). */}
      {submit && <button ref={native} type="submit" hidden disabled={off} tabIndex={-1} />}
    </>
  );
}

/** What a button that opens a menu says of it: open or not, and the id of
 * the menu (aria-controls while it is open). */
export interface MenuTrigger {
  open: boolean;
  id: string;
}

/** An icon action (vscode-toolbar-button's look: 22px, 5px corners, the
 * toolbar hover), with the label as its accessible name and tooltip. A
 * native button: the component has no disabled state, and its name would
 * sit inside its shadow root. */
export function IconButton({
  icon: Icon,
  label,
  onClick,
  busy = false,
  disabled = false,
  checked,
  menu,
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
  /** It opens a menu (aria-haspopup): whether it is open, and its id
   * (aria-expanded, aria-controls while it is open). */
  menu?: MenuTrigger | undefined;
  title?: string | undefined;
  className?: string;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      title={title ?? label}
      aria-pressed={checked}
      aria-haspopup={menu ? "menu" : undefined}
      aria-expanded={menu?.open}
      aria-controls={menu?.open ? menu.id : undefined}
      aria-busy={busy}
      disabled={disabled || busy}
      onClick={onClick}
      className={[
        // Busy, the spinner keeps its full color.
        "flex size-[22px] shrink-0 items-center justify-center rounded-[5px] text-vsc-icon",
        busy ? "" : "disabled:opacity-40",
        "enabled:hover:bg-(--vscode-toolbar-hoverBackground) enabled:active:bg-(--vscode-toolbar-activeBackground)",
        checked
          ? "bg-(--vscode-inputOption-activeBackground) text-(--vscode-inputOption-activeForeground) outline-1 -outline-offset-1 outline-(--vscode-inputOption-activeBorder)"
          : "",
        className,
      ].join(" ")}
    >
      {busy ? (
        // In place of the icon, the same size: VSCodium's busy action.
        <Codicon name="loading" size={16} className="animate-spin" />
      ) : (
        <Icon size={16} aria-hidden="true" />
      )}
    </button>
  );
}

/** A toggle with text (aria-pressed): one chip of a row of choices. Drawn
 * as VSCodium's input options (.monaco-custom-toggle) with the toggle
 * colors, pressed or not; a native button, as IconButton: vscode-button
 * has no pressed state. */
export function ToggleButton({
  pressed,
  onClick,
  disabled = false,
  title,
  className = "",
  children,
}: {
  pressed: boolean;
  onClick: () => void;
  disabled?: boolean;
  title?: string | undefined;
  className?: string;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      aria-pressed={pressed}
      title={title}
      disabled={disabled}
      onClick={onClick}
      className={[
        "rounded-vsc-small border px-vsc-size60 text-vsc-body-small leading-(--vscode-spacing-size200) disabled:opacity-40",
        "focus-visible:outline-1 focus-visible:-outline-offset-1 focus-visible:outline-vsc-focus",
        pressed
          ? "border-(--vscode-inputOption-activeBorder) bg-(--vscode-inputOption-activeBackground) text-(--vscode-inputOption-activeForeground)"
          : "border-vsc-border enabled:hover:bg-(--vscode-toolbar-hoverBackground)",
        className,
      ].join(" ")}
    >
      {children}
    </button>
  );
}

/** A row of icon actions (monaco-action-bar). */
export function ActionBar({
  children,
  className = "",
  label,
}: {
  children: ReactNode;
  className?: string;
  /** The toolbar's accessible name, when a page has more than one. */
  label?: string;
}) {
  return (
    <div role="toolbar" aria-label={label} className={`flex shrink-0 items-center gap-0.5 ${className}`}>
      {children}
    </div>
  );
}
