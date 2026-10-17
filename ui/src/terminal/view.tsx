// One session of the app's terminal in xterm.js: what the program writes,
// with the output kept by Rust first (the view often attaches after the
// first prompt), then the chunks after it; the keys typed go back to the
// program, and the view's size in characters follows the panel's.
import { useEffect, useRef } from "react";
import { Terminal, type ITheme } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import type { PtyBackend, PtyChunk, PtyInfo } from "./api";
import { t } from "../i18n";

/** The editor's terminal colors (theme-vscode.css), read now: light and
 * dark follow the system, as the rest of the window. */
export function themeFromCss(style: CSSStyleDeclaration = getComputedStyle(document.documentElement)): ITheme {
  const theme: Record<string, string> = {};
  const colors: [keyof ITheme, string][] = [
    ["background", "panel-background"],
    ["foreground", "terminal-foreground"],
    ["cursor", "terminalCursor-foreground"],
    ["selectionBackground", "terminal-selectionBackground"],
    ["selectionInactiveBackground", "terminal-inactiveSelectionBackground"],
  ];
  for (const name of ANSI) {
    colors.push([(name.charAt(0).toLowerCase() + name.slice(1)) as keyof ITheme, `terminal-ansi${name}`]);
  }
  // A color the theme lacks stays xterm.js's own.
  for (const [key, name] of colors) {
    const value = style.getPropertyValue(`--vscode-${name}`).trim();
    if (value) theme[key] = value;
  }
  if (!theme.cursor && theme.foreground) theme.cursor = theme.foreground;
  return theme;
}

const ANSI = ["Black", "Red", "Green", "Yellow", "Blue", "Magenta", "Cyan", "White"].flatMap((c) => [c, `Bright${c}`]);

/** Writes the backlog, then only the chunks after it: events that arrive
 * while the backlog is read wait, and those already in it are dropped.
 * `note` writes a text of the view (the program ended) after the output
 * that came before it. */
export function sequencer(write: (data: string) => void) {
  let seen = -1;
  let queue: (PtyChunk | string)[] | null = [];
  const take = (c: PtyChunk | string) => {
    if (typeof c === "string") write(c);
    else if (c.seq > seen) {
      write(c.data);
      seen = c.seq;
    }
  };
  return {
    chunk(c: PtyChunk) {
      if (queue) queue.push(c);
      else take(c);
    },
    note(text: string) {
      if (queue) queue.push(text);
      else take(text);
    },
    backlog(b: PtyChunk) {
      if (!queue) return;
      write(b.data);
      seen = b.seq;
      const waiting = queue;
      queue = null;
      waiting.forEach(take);
    },
  };
}

export function TerminalView({
  session,
  backend,
  active,
}: {
  session: PtyInfo;
  backend: PtyBackend;
  /** The tab shown: it takes the focus; the others stay mounted, hidden. */
  active: boolean;
}) {
  const box = useRef<HTMLDivElement>(null);
  const term = useRef<Terminal | null>(null);
  const seqRef = useRef<ReturnType<typeof sequencer> | null>(null);
  const { id } = session;

  useEffect(() => {
    const el = box.current;
    if (!el) return;
    const style = getComputedStyle(document.documentElement);
    const x = new Terminal({
      fontFamily: style.getPropertyValue("--vscode-editor-font-family").trim() || "monospace",
      fontSize: 13,
      cursorBlink: true,
      scrollback: 5000,
      theme: themeFromCss(style),
    });
    term.current = x;
    const fit = new FitAddon();
    x.loadAddon(fit);
    x.open(el);

    let gone = false;
    let stop: (() => void) | null = null;
    const seq = sequencer((data) => {
      x.write(data);
    });
    seqRef.current = seq;
    backend
      .onOutput(id, (c) => {
        seq.chunk(c);
      })
      .then((unlisten) => {
        if (gone) {
          unlisten();
          return null;
        }
        stop = unlisten;
        return backend.backlog(id);
      })
      .then(
        (b) => {
          if (b && !gone) seq.backlog(b);
        },
        () => undefined,
      );

    const typed = x.onData((data) => {
      backend.write(id, data).catch(() => undefined);
    });
    let size = "";
    const resize = () => {
      // Hidden tabs have no size: measured again when shown.
      if (el.clientWidth === 0 || el.clientHeight === 0) return;
      fit.fit();
      const now = `${String(x.cols)}x${String(x.rows)}`;
      if (now === size) return;
      size = now;
      backend.resize(id, x.cols, x.rows).catch(() => undefined);
    };
    const observer = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(resize);
    observer?.observe(el);
    resize();
    // Light and dark follow the system while the session is open.
    const scheme = typeof window.matchMedia === "function" ? window.matchMedia("(prefers-color-scheme: dark)") : null;
    const retheme = () => {
      x.options.theme = themeFromCss();
    };
    scheme?.addEventListener("change", retheme);

    return () => {
      gone = true;
      stop?.();
      typed.dispose();
      observer?.disconnect();
      scheme?.removeEventListener("change", retheme);
      x.dispose();
      term.current = null;
      seqRef.current = null;
    };
  }, [id, backend]);

  useEffect(() => {
    if (active) term.current?.focus();
  }, [active]);

  const ended = !session.running;
  useEffect(() => {
    if (!ended) return;
    const code = session.code;
    const text = code === null ? t("[processo terminato]") : t("[processo terminato, codice {0}]", code);
    seqRef.current?.note(`\r\n\x1b[2m${text}\x1b[0m\r\n`);
  }, [ended]); // eslint-disable-line react-hooks/exhaustive-deps

  return (
    <div
      ref={box}
      role="group"
      aria-label={session.title}
      hidden={!active}
      className="min-h-0 min-w-0 flex-1 px-2 pt-1"
      data-testid={`pty-${String(id)}`}
    />
  );
}
