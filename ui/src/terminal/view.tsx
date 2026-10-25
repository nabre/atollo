// One session of the app's terminal in xterm.js: what the program writes,
// with the output kept by Rust first (the view often attaches after the
// first prompt, or after a move between the panel and a window), then the
// chunks after it; the keys typed go back to the program, and the view's
// size in characters follows its box's, in Geist Mono (font.ts).
import { useEffect, useRef } from "react";
import { Terminal, type ITheme } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { Unicode11Addon } from "@xterm/addon-unicode11";
import "@xterm/xterm/css/xterm.css";
import type { PtyBackend, PtyChunk, PtyInfo } from "../ipc/pty";
import { FONT_SIZE, monoFamily, monoLoaded, monoReady, remeasured, whenMonoLoads } from "./font";
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

/** Windows' browser engine (WebView2): the sessions run on ConPTY. */
const onWindows = () => typeof navigator !== "undefined" && /Windows/i.test(navigator.userAgent);
/** macOS copies and pastes with ⌘, which xterm.js leaves to the browser. */
const onMac = () => typeof navigator !== "undefined" && /Mac/i.test(navigator.userAgent);

/** The keys of Windows Terminal and VSCodium's terminal off macOS (D-46):
 * Ctrl+C copies when text is selected (else it goes to the program),
 * Ctrl+Shift+C copies, Ctrl+V and Ctrl+Shift+V paste (the browser's own
 * paste, which xterm.js reads). False: xterm.js leaves the key alone. */
export function clipboardKey(
  e: Pick<KeyboardEvent, "type" | "key" | "ctrlKey" | "shiftKey" | "altKey" | "metaKey">,
  selection: string,
  copy: (text: string) => void,
): boolean {
  if (e.type !== "keydown" || !e.ctrlKey || e.altKey || e.metaKey) return true;
  const key = e.key.toLowerCase();
  if (key === "v") return false;
  if (key !== "c" || (!e.shiftKey && !selection)) return true;
  if (selection) copy(selection);
  return false;
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
  // Read by a view that opens after the fonts: it takes the focus then.
  const activeRef = useRef(active);
  const { id } = session;

  useEffect(() => {
    const el = box.current;
    if (!el) return;
    let gone = false;
    let stop: (() => void) | null = null;
    let teardown: (() => void) | null = null;
    const family = monoFamily();
    // xterm.js measures its cell with the font it finds when it opens: it
    // waits for Geist Mono, which a new window reads from its files.
    const start = () => {
      if (gone) return;
      teardown = open(el, family);
    };
    if (monoLoaded(family)) start();
    else void monoReady(family).then(start);

    function open(el: HTMLDivElement, family: string) {
      const style = getComputedStyle(document.documentElement);
      const x = new Terminal({
        fontFamily: family,
        fontSize: FONT_SIZE,
        cursorBlink: true,
        scrollback: 5000,
        theme: themeFromCss(style),
        // The kit's ConPTY (D-148) reflows its own lines like the builds of
        // Windows after 21376: xterm.js then keeps its usual reflow.
        ...(onWindows() ? { windowsPty: { backend: "conpty" as const, buildNumber: 22000 } } : {}),
        // The Unicode 11 widths (`unicode` is a proposed API of xterm.js).
        allowProposedApi: true,
      });
      term.current = x;
      // Emoji and wide characters take the cells the program counted
      // (wcswidth: the kit's ConPTY measures the same way, D-148).
      x.loadAddon(new Unicode11Addon());
      x.unicode.activeVersion = "11";
      if (!onMac()) {
        x.attachCustomKeyEventHandler((e) =>
          clipboardKey(e, x.getSelection(), (text) => {
            navigator.clipboard.writeText(text).catch(() => undefined);
          }),
        );
      }
      const fit = new FitAddon();
      x.loadAddon(fit);
      x.open(el);
      // Opened with a fallback (the faces came after the cap): the cell is
      // measured again with Geist Mono, and the size follows.
      const late = monoLoaded(family)
        ? () => undefined
        : whenMonoLoads(family, () => {
            x.options.fontFamily = remeasured(x.options.fontFamily ?? family);
            resize();
          });

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
            if (!b || gone) return;
            seq.backlog(b);
            // A session that already wrote (moved from the panel to its
            // window or back): the program draws its screen again for this
            // view, with a size it did not have a moment ago.
            if (b.seq > 0 && x.rows > 1) {
              backend
                .resize(id, x.cols, x.rows - 1)
                .then(() => backend.resize(id, x.cols, x.rows))
                .catch(() => undefined);
            }
          },
          () => undefined,
        );

      const typed = x.onData((data) => {
        backend.write(id, data).catch(() => undefined);
      });
      const binary = x.onBinary((data) => {
        backend.writeBinary(id, data).catch(() => undefined);
      });
      let size = "";
      function resize() {
        // Hidden tabs have no size: measured again when shown.
        if (el.clientWidth === 0 || el.clientHeight === 0) return;
        fit.fit();
        const now = `${String(x.cols)}x${String(x.rows)}`;
        if (now === size) return;
        size = now;
        backend.resize(id, x.cols, x.rows).catch(() => undefined);
      }
      const observer = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(resize);
      observer?.observe(el);
      resize();
      // Light and dark follow the system while the session is open.
      const scheme = typeof window.matchMedia === "function" ? window.matchMedia("(prefers-color-scheme: dark)") : null;
      const retheme = () => {
        x.options.theme = themeFromCss();
      };
      scheme?.addEventListener("change", retheme);
      if (activeRef.current) x.focus();

      return () => {
        late();
        typed.dispose();
        binary.dispose();
        observer?.disconnect();
        scheme?.removeEventListener("change", retheme);
        x.dispose();
        term.current = null;
        seqRef.current = null;
      };
    }

    return () => {
      gone = true;
      stop?.();
      teardown?.();
    };
  }, [id, backend]);

  useEffect(() => {
    activeRef.current = active;
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
