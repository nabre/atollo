// VSCodium's fields: text fields, text areas and dropdowns are the
// vscode-elements components (elements.ts), with VSCodium 1.135's sizes;
// checkboxes and radios stay native elements drawn as those components, so
// the pages' <label for> reach them. A field with suggestions is in
// suggest.tsx.
import { useEffect, useId, type CSSProperties, type FocusEvent, type HTMLAttributes, type ReactNode } from "react";
import { VscodeOption, VscodeSingleSelect, VscodeTextarea, VscodeTextfield } from "./elements";
import { Codicon } from "./codicon";
import { CountBadge } from "./indicators";
import { guardEnter, useControlled, useFieldLabel, type FieldElement } from "./fields";

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
