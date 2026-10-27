// What the workbench's fields and buttons share: a field's name from its
// <label>, a controlled value on a vscode-elements component, and Enter in
// a form. Only for the files of workbench/, not exported by index.ts.
import { useEffect, useLayoutEffect, useRef, type KeyboardEvent } from "react";
import { flushSync } from "react-dom";

export type TextElement = HTMLElementTagNameMap["vscode-textfield" | "vscode-textarea"];
export type FieldElement = TextElement | HTMLElementTagNameMap["vscode-single-select"];

/** The text of the <label> that names a field: <label for> its id, or the
 * label around it (without the field's own text, as a select's options). */
export function labelText(el: HTMLElement): string {
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
export function useFieldLabel<E extends FieldElement>(label: string | undefined) {
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
export function useControlled(value: string, onChange: (v: string) => void) {
  const latest = useRef(value);
  useLayoutEffect(() => {
    latest.current = value;
  });
  return (e: Event) => {
    const el = e.currentTarget as TextElement;
    if (el.value === latest.current) return;
    // flushSync renders now what a keystroke would render a microtask
    // later (an input event is discrete: React renders it at once anyway),
    // so it costs no render of its own; it lets the kept value be read
    // below, before the browser paints the rejected text or moves the
    // caret, which a later check could not. What a keystroke renders is
    // kept small elsewhere (the state outside App).
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
export function defaultButton(form: HTMLFormElement) {
  for (const el of form.elements)
    if ((el instanceof HTMLButtonElement || el instanceof HTMLInputElement) && el.type === "submit") return el;
  return undefined;
}

/** Enter in a vscode-textfield calls its form's requestSubmit() itself,
 * past the browser's implicit submission: stopped before it gets there
 * when the browser would not submit, with the form's default button
 * disabled (a busy Button) or while an IME composes. */
export function guardEnter(e: KeyboardEvent<HTMLElement>) {
  if (e.key !== "Enter") return;
  const form = (e.currentTarget as HTMLElementTagNameMap["vscode-textfield"]).form;
  if (e.nativeEvent.isComposing || (form && defaultButton(form)?.disabled)) e.stopPropagation();
}
