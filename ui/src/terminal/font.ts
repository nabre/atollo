// The terminal's monospace font in every window that shows one (the main
// window's panel, a detached session's window; D-149): Geist Mono, the
// editor's (theme-vscode.css). xterm.js measures its cell once, with the
// font it finds then: a fallback measured before Geist Mono arrives keeps
// its width, and every line is drawn on the wrong grid. So the faces are
// loaded before a view opens, and a view opened without them (the cap
// passed) measures again when they come.
import "@fontsource-variable/geist-mono/wght.css";

export const FONT_SIZE = 13;

/** The editor's monospace stack, read from the theme. */
export function monoFamily(style: CSSStyleDeclaration = getComputedStyle(document.documentElement)): string {
  return style.getPropertyValue("--vscode-editor-font-family").trim() || "monospace";
}

/** A character of each subset of Geist Mono a terminal draws (each is a
 * file of its own, read on first use): Latin, Latin extended, and the box
 * drawing and blocks of the programs' frames (U+2500-259F). */
const SAMPLE = "Wà ő ─│╭█";

/** Normal and bold: xterm.js draws bold text with the same cell. */
function faces(family: string): string[] {
  return [`${String(FONT_SIZE)}px ${family}`, `bold ${String(FONT_SIZE)}px ${family}`];
}

function fontSet(): FontFaceSet | null {
  return typeof document !== "undefined" && "fonts" in document ? document.fonts : null;
}

/** Whether the faces are ready now (no font loading API: nothing to wait). */
export function monoLoaded(family: string): boolean {
  const fonts = fontSet();
  return !fonts || faces(family).every((f) => fonts.check(f, SAMPLE));
}

/** Loads the faces; true once ready, false when `cap` ms passed first. */
export function monoReady(family: string, cap = 1500): Promise<boolean> {
  const fonts = fontSet();
  if (!fonts) return Promise.resolve(true);
  const load = Promise.allSettled(faces(family).map((f) => fonts.load(f, SAMPLE))).then(() => monoLoaded(family));
  return Promise.race([
    load,
    new Promise<boolean>((done) => {
      setTimeout(() => {
        done(false);
      }, cap);
    }),
  ]);
}

/** Calls `ready` once the faces are loaded, after the cap; the stop. */
export function whenMonoLoads(family: string, ready: () => void): () => void {
  const fonts = fontSet();
  if (!fonts) return () => undefined;
  const check = () => {
    if (!monoLoaded(family)) return;
    fonts.removeEventListener("loadingdone", check);
    ready();
  };
  fonts.addEventListener("loadingdone", check);
  return () => {
    fonts.removeEventListener("loadingdone", check);
  };
}

/** The same stack written another way (a trailing space): xterm.js
 * measures its cell again only when the option changes. */
export function remeasured(family: string): string {
  return family.endsWith(" ") ? family.trimEnd() : `${family} `;
}
