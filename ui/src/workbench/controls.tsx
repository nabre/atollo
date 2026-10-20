// VSCodium's controls. Text buttons, fields, dropdowns, badges, the progress
// bar, dividers and icons are the vscode-elements components (elements.ts),
// with their --vscode-* colors and VSCodium 1.135's sizes (the fields' in
// elements.ts, the buttons' here). Icon actions, checkboxes and radios
// stay native elements drawn as those components: their role and name must
// be in the page (light DOM), where the pages' <label for> reach them.
import {
  useEffect,
  useId,
  useLayoutEffect,
  useRef,
  type CSSProperties,
  type FocusEvent,
  type HTMLAttributes,
  type KeyboardEvent,
  type ReactNode,
} from "react";
import { flushSync } from "react-dom";
import type { IconType } from "react-icons";
import { VscodeOption, VscodeSingleSelect, VscodeTextarea, VscodeTextfield } from "@vscode-elements/react-elements";
import { t } from "../i18n";
import { useHost } from "./elements";
import { Codicon, codicon } from "./codicon";

type TextElement = HTMLElementTagNameMap["vscode-textfield" | "vscode-textarea"];
type FieldElement = TextElement | HTMLElementTagNameMap["vscode-single-select"];

/** The text of the <label> that names a field: <label for> its id, or the
 * label around it (without the field's own text, as a select's options). */
function labelText(el: HTMLElement): string {
  const label =
    (el.id ? [...document.getElementsByTagName("label")].find((l) => l.htmlFor === el.id) : undefined) ??
    el.closest("label");
  if (!label) return "";
  let text = "";
  for (const node of label.childNodes) if (!node.contains(el)) text += node.textContent ?? "";
  return text.replace(/\s+/g, " ").trim();
}

/** A field's accessible name. The focus goes to the input inside the
 * component, which a <label> outside can't name: its `label` property
 * names it, as vscode-label does. Without `label`, the text of the page's
 * <label> is taken. */
function useFieldLabel<E extends FieldElement>(label: string | undefined) {
  const ref = useRef<E>(null);
  useEffect(() => {
    const el = ref.current;
    if (el && label === undefined) el.label = labelText(el);
  });
  return ref;
}

/** The input/change handler of a text component, controlled as a native
 * input in React: the typed text goes to `onChange`, and when the page
 * keeps another value (it rejects or normalizes the text, so nothing
 * renders and the wrapper never sets `value` again) the field shows the
 * kept one at once. */
function useControlled(value: string, onChange: (v: string) => void) {
  const latest = useRef(value);
  useLayoutEffect(() => {
    latest.current = value;
  });
  return (e: Event) => {
    const el = e.currentTarget as TextElement;
    if (el.value === latest.current) return;
    flushSync(() => {
      onChange(el.value);
    });
    const kept = latest.current;
    if (el.value === kept) return;
    el.value = kept;
    // Lit compares with the value it drew last, not with the input's own.
    const inner = el.wrappedElement as HTMLInputElement | HTMLTextAreaElement | undefined;
    if (inner) inner.value = kept;
  };
}

/** The default button of a form, the one Enter clicks in its fields. */
function defaultButton(form: HTMLFormElement) {
  for (const el of form.elements)
    if ((el instanceof HTMLButtonElement || el instanceof HTMLInputElement) && el.type === "submit") return el;
  return undefined;
}

/** Enter in a vscode-textfield calls its form's requestSubmit() itself,
 * past the browser's implicit submission: stopped before it gets there
 * when the browser would not submit, with the form's default button
 * disabled (a busy Button) or while an IME composes. */
function guardEnter(e: KeyboardEvent<HTMLElement>) {
  if (e.key !== "Enter") return;
  const form = (e.currentTarget as HTMLElementTagNameMap["vscode-textfield"]).form;
  if (e.nativeEvent.isComposing || (form && defaultButton(form)?.disabled)) e.stopPropagation();
}

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
        className={["shrink-0", small ? SMALL_BUTTON : TEXT_BUTTON, className].join(" ")}
      >
        {busy ? <span className="animate-pulse">{children}</span> : children}
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
        "flex size-[22px] shrink-0 items-center justify-center rounded-[5px] text-vsc-icon disabled:opacity-40",
        "enabled:hover:bg-(--vscode-toolbar-hoverBackground) enabled:active:bg-(--vscode-toolbar-activeBackground)",
        checked
          ? "bg-(--vscode-inputOption-activeBackground) text-(--vscode-inputOption-activeForeground) outline-1 -outline-offset-1 outline-(--vscode-inputOption-activeBorder)"
          : "",
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

/** The filter box at the top of a list (Extensions, Settings), a
 * vscode-textfield: search icon on the left, the count on the right. It
 * filters as one types. */
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
  /** Also its accessible name. */
  placeholder: string;
  count?: string | undefined;
  /** The Settings editor's search: 28px. */
  large?: boolean;
  className?: string;
}) {
  const read = useControlled(value, onChange);
  return (
    <VscodeTextfield
      type="search"
      onKeyDownCapture={guardEnter}
      label={placeholder}
      // The name on the host too, for what reads the page's own tree.
      aria-label={placeholder}
      placeholder={placeholder}
      value={value}
      onInput={read}
      onChange={read}
      className={`w-full ${className}`}
    >
      {/* 26px high in the large box: the slot sets the field's height. */}
      <span
        slot="content-before"
        className={`flex items-center pl-[5px] text-vsc-placeholder ${large ? "h-[26px]" : ""}`}
      >
        <Codicon name="search" size={14} />
      </span>
      {count && (
        <span slot="content-after" className="pr-1">
          <CountBadge>{count}</CountBadge>
        </span>
      )}
    </VscodeTextfield>
  );
}

/** The editor's font in a field: the component reads this variable. */
const MONO = { "--vscode-font-family": "var(--vscode-editor-font-family)" } as CSSProperties;

/** A text field (vscode-textfield), 26px. Named by `label`, or by the
 * page's <label> for its `id` or around it. */
export function TextField({
  value,
  onChange,
  label,
  id,
  type = "text",
  placeholder,
  disabled = false,
  readOnly = false,
  invalid = false,
  inputMode,
  mono = false,
  autoFocus = false,
  pattern,
  autoComplete,
  title,
  onBlur,
  className = "",
  children,
}: {
  value: string;
  onChange: (v: string) => void;
  label?: string | undefined;
  id?: string | undefined;
  type?: "text" | "password" | "number" | "url" | "email" | "search";
  placeholder?: string | undefined;
  disabled?: boolean;
  readOnly?: boolean;
  /** Error colors (inputValidation), with aria-invalid. */
  invalid?: boolean;
  /** The keyboard of a touch screen, as the native attribute. */
  inputMode?: HTMLAttributes<HTMLInputElement>["inputMode"];
  /** The editor's font, for paths, commands and keys. */
  mono?: boolean;
  autoFocus?: boolean;
  pattern?: string | undefined;
  /** "off": no saved or suggested values (keys, tokens). */
  autoComplete?: "on" | "off" | undefined;
  /** Tooltip, and the description of the inner input. */
  title?: string | undefined;
  onBlur?: () => void;
  className?: string;
  /** Before or after the text, with slot="content-before" / "content-after". */
  children?: ReactNode;
}) {
  const ref = useFieldLabel<HTMLElementTagNameMap["vscode-textfield"]>(label);
  useAutoFocus(ref, autoFocus);
  const read = useControlled(value, onChange);
  // The focus goes to the input inside the component, which the component
  // doesn't give these: set on it once it is drawn.
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    let live = true;
    void el.updateComplete.then(() => {
      const input = el.wrappedElement as HTMLInputElement | undefined;
      if (!live || !input) return;
      if (invalid) input.setAttribute("aria-invalid", "true");
      else input.removeAttribute("aria-invalid");
      if (inputMode) input.setAttribute("inputmode", inputMode);
      else input.removeAttribute("inputmode");
      if (title) input.title = title;
      else input.removeAttribute("title");
    });
    return () => {
      live = false;
    };
  }, [ref, invalid, inputMode, title]);
  return (
    <VscodeTextfield
      ref={ref}
      onKeyDownCapture={guardEnter}
      id={id}
      type={type}
      label={label ?? ""}
      aria-label={label}
      aria-invalid={invalid ? "true" : undefined}
      placeholder={placeholder}
      value={value}
      disabled={disabled}
      readonly={readOnly}
      invalid={invalid}
      pattern={pattern}
      autocomplete={autoComplete}
      title={title}
      onInput={read}
      onChange={read}
      onBlur={onBlur && leaving(onBlur)}
      style={mono ? MONO : undefined}
      className={className}
    >
      {children}
    </VscodeTextfield>
  );
}

/** A text area (vscode-textarea). Named as TextField. */
export function TextArea({
  value,
  onChange,
  label,
  id,
  rows = 4,
  placeholder,
  disabled = false,
  readOnly = false,
  mono = false,
  spellCheck,
  className = "",
}: {
  value: string;
  onChange: (v: string) => void;
  label?: string | undefined;
  id?: string | undefined;
  rows?: number;
  placeholder?: string | undefined;
  disabled?: boolean;
  readOnly?: boolean;
  /** The editor's font. */
  mono?: boolean;
  spellCheck?: boolean;
  className?: string;
}) {
  const ref = useFieldLabel<HTMLElementTagNameMap["vscode-textarea"]>(label);
  const read = useControlled(value, onChange);
  // A property (spellCheck as an attribute would read "false" as true).
  useEffect(() => {
    if (ref.current && spellCheck !== undefined) ref.current.spellcheck = spellCheck;
  }, [ref, spellCheck]);
  return (
    <VscodeTextarea
      ref={ref}
      id={id}
      label={label ?? ""}
      aria-label={label}
      rows={rows}
      resize="vertical"
      placeholder={placeholder}
      value={value}
      disabled={disabled}
      readonly={readOnly}
      monospace={mono}
      onInput={read}
      onChange={read}
      className={className}
    />
  );
}

/** Focuses a component once it has drawn its input. */
function useAutoFocus(ref: { current: (HTMLElement & { updateComplete: Promise<boolean> }) | null }, on: boolean) {
  useEffect(() => {
    const el = ref.current;
    if (!on || !el) return;
    let live = true;
    void el.updateComplete.then(() => {
      if (live) el.focus();
    });
    return () => {
      live = false;
    };
  }, [ref, on]);
}

/** onBlur for a component: only when the focus leaves it, not when it
 * moves inside its shadow root (a select's list). */
function leaving(onBlur: () => void) {
  return (e: FocusEvent<HTMLElement>) => {
    const to = e.relatedTarget;
    if (to instanceof Node && (to === e.currentTarget || e.currentTarget.contains(to))) return;
    onBlur();
  };
}

export interface SelectOption<T extends string> {
  value: T;
  label: string;
}

/** A labelled group of options: the group's name heads them, dimmed, and
 * is the description of each. */
export interface SelectGroup<T extends string> {
  group: string;
  options: SelectOption<T>[];
}

/** A dropdown (vscode-single-select), 26px as the inputs. `options` can mix
 * single options and groups, in order; `onChange` gets the chosen value.
 * Named as TextField. */
export function Select<T extends string>({
  value,
  onChange,
  options,
  label,
  id,
  disabled = false,
  autoFocus = false,
  onBlur,
  className = "",
}: {
  value: T;
  onChange: (v: T) => void;
  options: (SelectOption<T> | SelectGroup<T>)[];
  /** Accessible name when there is no <label> around or for it. */
  label?: string | undefined;
  id?: string | undefined;
  disabled?: boolean;
  autoFocus?: boolean;
  onBlur?: () => void;
  className?: string;
}) {
  const ref = useFieldLabel<HTMLElementTagNameMap["vscode-single-select"]>(label);
  useAutoFocus(ref, autoFocus);
  // A value no option has shows the first option, as a native select in
  // React: not the component's own fallback, its first item, which can be
  // a group's (disabled) header.
  const flat = options.flatMap((o) => ("group" in o ? o.options : [o]));
  const shown = flat.some((o) => o.value === value) ? value : flat[0]?.value;
  const option = (o: SelectOption<T>, description = "") => (
    <VscodeOption key={o.value} value={o.value} description={description} selected={o.value === shown}>
      {o.label}
    </VscodeOption>
  );
  return (
    <VscodeSingleSelect
      ref={ref}
      id={id}
      label={label ?? ""}
      aria-label={label}
      value={shown ?? ""}
      disabled={disabled}
      // Closed, the component still stops Escape (and the arrows, Space):
      // the page's own Esc (a dialog's cancel, a menu's close) gets it again
      // from the parent. Open, Escape only closes the list.
      onKeyDownCapture={(e) => {
        const el = e.currentTarget;
        if (e.key !== "Escape" || el.open) return;
        e.stopPropagation();
        (el.parentElement ?? document).dispatchEvent(
          new globalThis.KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true, composed: true }),
        );
      }}
      onChange={(e) => {
        onChange((e.currentTarget as FieldElement).value as T);
      }}
      onBlur={onBlur && leaving(onBlur)}
      className={className}
    >
      {options.flatMap((o) =>
        "group" in o
          ? [
              <VscodeOption key={`group:${o.group}`} value={`\u0000${o.group}`} disabled>
                {o.group}
              </VscodeOption>,
              ...o.options.map((g) => option(g, o.group)),
            ]
          : [option(o)],
      )}
    </VscodeSingleSelect>
  );
}

/** The box of a native checkbox or radio, as vscode-checkbox draws it: 18px,
 * the Settings editor's checkbox colors, the focus border inside. */
const BOX =
  "peer size-[18px] cursor-pointer appearance-none border border-(--vscode-settings-checkboxBorder) bg-(--vscode-settings-checkboxBackground) focus-visible:outline-1 focus-visible:-outline-offset-1 focus-visible:outline-vsc-focus disabled:cursor-default";

/** A checkbox as vscode-checkbox: 18px, 3px corners, a check codicon. */
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
    <span className={`relative inline-flex size-[18px] shrink-0 has-disabled:opacity-40 ${className}`}>
      <input
        id={id}
        type="checkbox"
        aria-label={label}
        checked={checked}
        disabled={disabled}
        onChange={(e) => {
          onChange(e.target.checked);
        }}
        className={`${BOX} rounded-[3px]`}
      />
      <Codicon
        name="check"
        size={16}
        className="pointer-events-none absolute top-px left-px hidden text-(--vscode-settings-checkboxForeground) peer-checked:block"
      />
    </span>
  );
}

/** A checkbox with its text on the right (the boolean setting), 9px apart. */
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
    <span className="flex items-start gap-[9px]">
      <Checkbox id={id} checked={checked} onChange={onChange} disabled={disabled} />
      <label htmlFor={id} className="min-w-0 cursor-pointer leading-[18px] opacity-90">
        {children}
      </label>
    </span>
  );
}

/** A radio button with its text, as vscode-radio: 18px round, a dot. */
export function Radio({
  name,
  checked,
  onSelect,
  disabled = false,
  plain = false,
  children,
}: {
  /** The group: radios with one name move with the arrows. */
  name: string;
  checked: boolean;
  onSelect: () => void;
  disabled?: boolean;
  /** Content with its own look (a card's name and description): not
   * dimmed, its own line heights. */
  plain?: boolean;
  children: ReactNode;
}) {
  const id = useId();
  return (
    <span className="flex items-start gap-[9px]">
      <span className="relative inline-flex size-[18px] shrink-0 has-disabled:opacity-40">
        <input
          id={id}
          type="radio"
          name={name}
          checked={checked}
          disabled={disabled}
          onChange={onSelect}
          className={`${BOX} rounded-full`}
        />
        <span
          aria-hidden="true"
          className="pointer-events-none absolute top-[5px] left-[5px] hidden size-2 rounded-full bg-(--vscode-settings-checkboxForeground) peer-checked:block"
        />
      </span>
      <label htmlFor={id} className={`min-w-0 cursor-pointer ${plain ? "" : "leading-[18px] opacity-90"}`}>
        {children}
      </label>
    </span>
  );
}

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

/** A line between groups (vscode-divider). */
export function Divider({ className = "" }: { className?: string }) {
  const [ref] = useHost("separator");
  return <vscode-divider ref={ref} className={className} />;
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

/** A state dot: green running, orange starting, red error, gray stopped. */
export function Dot({ tone, label }: { tone: "ok" | "warn" | "err" | "off"; label?: string | undefined }) {
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
