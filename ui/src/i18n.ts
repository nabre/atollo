// Texts of the interface: Italian source, as in the Rust code
// (tauri/i18n/catalog.json), with English, French and German catalogs in
// locales/. `t("Testo {0}", x)` returns the text in the current language with
// {0}, {1}… replaced; a text missing from a catalog stays Italian (the test
// i18n.test.ts fails before that can ship).
import { createElement, Fragment, type ReactNode } from "react";

// One list of languages: the Guide's (a type, nothing loaded at start).
import type { Lang } from "@guide/core/lang";
export type { Lang };

export const LANGS: { id: Lang; name: string }[] = [
  { id: "it", name: "Italiano" },
  { id: "en", name: "English" },
  { id: "fr", name: "Français" },
  { id: "de", name: "Deutsch" },
];

type Catalog = Record<string, string>;

// Each catalog a chunk of its own, read when its language is chosen: a
// window reads only the one it shows.
const LOADERS: Record<Exclude<Lang, "it">, () => Promise<{ default: Catalog }>> = {
  en: () => import("./locales/en.json"),
  fr: () => import("./locales/fr.json"),
  de: () => import("./locales/de.json"),
};
const catalogs: Partial<Record<Exclude<Lang, "it">, Catalog>> = {};

/** A language's catalog, read once (Italian has none: the sources). */
export async function loadCatalog(lang: Exclude<Lang, "it">): Promise<Catalog> {
  return (catalogs[lang] ??= (await LOADERS[lang]()).default);
}

// The language last shown, remembered in the window's storage: main.tsx
// reads its catalog before the first render, before app_info answers.
const STORED = "lang";

/** The language last chosen in this window's storage, if any. */
export function storedLang(): Lang | undefined {
  try {
    const id = localStorage.getItem(STORED);
    return LANGS.find((l) => l.id === id)?.id;
  } catch {
    return undefined;
  }
}

let current: Lang = "it";

/** Chooses the language: lang() at once, t() in it once the promise
 * resolves (its catalog read); render the texts after that. */
export async function setLang(lang: Lang): Promise<void> {
  current = lang;
  document.documentElement.lang = lang;
  try {
    localStorage.setItem(STORED, lang);
  } catch {
    // No storage: the next start reads the catalog after app_info.
  }
  if (lang !== "it") await loadCatalog(lang);
}

export function lang(): Lang {
  return current;
}

/** Locale for dates and numbers. */
export function locale(): string {
  return { it: "it-IT", en: "en-GB", fr: "fr-FR", de: "de-DE" }[current];
}

export function t(source: string, ...args: (string | number)[]): string {
  const text = current === "it" ? source : (catalogs[current]?.[source] ?? source);
  return args.reduce<string>((out, arg, i) => out.split(`{${String(i)}}`).join(String(arg)), text);
}

/** A translated text whose {0}, {1}… are elements, e.g.
 * `rich(t(source), <code>data</code>)` with {0} in source. */
export function rich(text: string, ...nodes: ReactNode[]): ReactNode {
  const parts = text.split(/\{(\d+)\}/).map((part, i) => (i % 2 ? nodes[Number(part)] : part));
  return createElement(Fragment, null, ...parts);
}
