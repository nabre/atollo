// The language of the Guide's text. The Guide knows no app: whoever draws
// it says which language is current (bindGuideLang), the app with its own
// lang(), the site once per language. tr() picks among the four texts at
// the moment it is called, so nothing may keep a tr() result from the time
// a module loads: one process draws the four languages one after another.
import { createElement, Fragment, type ReactNode } from "react";

export type Lang = "it" | "en" | "fr" | "de";

/** The languages of the Guide, Italian (the source) first. */
export const GUIDE_LANGS: readonly Lang[] = ["it", "en", "fr", "de"];

let get: () => Lang = () => "it";

/** Who tells the Guide its language: called once by each renderer. */
export function bindGuideLang(getter: () => Lang): void {
  get = getter;
}

/** The language the Guide is drawn in now. */
export function guideLang(): Lang {
  return get();
}

/** Localized string helper for rich guide content: the Italian source and
 * its English, French and German, all required (no silent fallback). */
export function tr(it: string, en: string, fr: string, de: string): string {
  const l = get();
  if (l === "en") return en;
  if (l === "fr") return fr;
  if (l === "de") return de;
  return it;
}

/** Italian text of a draft, waiting for its translations: every call becomes
 * a tr() before it ships (test/ui/guide/text.test.tsx). */
export function L(it: string): string {
  return it;
}

/** A text whose {0}, {1}… are elements, e.g. `rich(tr(…), <Code>data</Code>)`. */
export function rich(text: string, ...nodes: ReactNode[]): ReactNode {
  const parts = text.split(/\{(\d+)\}/).map((part, i) => (i % 2 ? nodes[Number(part)] : part));
  return createElement(Fragment, null, ...parts);
}
