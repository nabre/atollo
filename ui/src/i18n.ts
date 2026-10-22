// Texts of the interface: Italian source, as in the Rust code
// (tauri/i18n/catalog.json), with English, French and German catalogs in
// locales/. `t("Testo {0}", x)` returns the text in the current language with
// {0}, {1}… replaced; a text missing from a catalog stays Italian (the test
// i18n.test.ts fails before that can ship).
import { createElement, Fragment, type ReactNode } from "react";
import en from "./locales/en.json";
import fr from "./locales/fr.json";
import de from "./locales/de.json";

// One list of languages: the Guide's (a type, nothing loaded at start).
import type { Lang } from "@guide/core/lang";
export type { Lang };

export const LANGS: { id: Lang; name: string }[] = [
  { id: "it", name: "Italiano" },
  { id: "en", name: "English" },
  { id: "fr", name: "Français" },
  { id: "de", name: "Deutsch" },
];

export const CATALOGS: Record<Exclude<Lang, "it">, Record<string, string>> = { en, fr, de };

let current: Lang = "it";

export function setLang(lang: Lang) {
  current = lang;
  document.documentElement.lang = lang;
}

export function lang(): Lang {
  return current;
}

/** Locale for dates and numbers. */
export function locale(): string {
  return { it: "it-IT", en: "en-GB", fr: "fr-FR", de: "de-DE" }[current];
}

export function t(source: string, ...args: (string | number)[]): string {
  const text = current === "it" ? source : (CATALOGS[current][source] ?? source);
  return args.reduce<string>((out, arg, i) => out.split(`{${String(i)}}`).join(String(arg)), text);
}

/** A translated text whose {0}, {1}… are elements, e.g.
 * `rich(t(source), <code>data</code>)` with {0} in source. */
export function rich(text: string, ...nodes: ReactNode[]): ReactNode {
  const parts = text.split(/\{(\d+)\}/).map((part, i) => (i % 2 ? nodes[Number(part)] : part));
  return createElement(Fragment, null, ...parts);
}
