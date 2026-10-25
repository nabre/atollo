// The sessions of the app's terminal (every system, D-148) and the panel's
// state: kept by the main window while pages change, filled by the events
// of Rust. No xterm.js here, so the window loads it only with the panel.
import { useCallback, useEffect, useRef, useState } from "react";
import type { PtyBackend, PtyInfo } from "../ipc/pty";

/** The sessions and the panel's state; `enabled` false (the browser
 * preview's first render, before the app answers) reads and listens to
 * nothing. */
export function useTerminals(backend: PtyBackend, enabled: boolean) {
  const [sessions, setSessions] = useState<PtyInfo[]>([]);
  const [active, setActive] = useState<number | null>(null);
  const [shown, setShown] = useState(false);
  // Every exit told: an opening read after it (a program that ended at
  // once) still shows the session ended.
  const exits = useRef(new Map<number, PtyInfo>());

  const upsert = useCallback((info: PtyInfo) => {
    setSessions((list) =>
      list.some((s) => s.id === info.id) ? list.map((s) => (s.id === info.id ? info : s)) : [...list, info],
    );
  }, []);

  useEffect(() => {
    if (!enabled) return;
    let gone = false;
    const stops: (() => void)[] = [];
    const keep = (p: Promise<() => void>) => {
      p.then(
        (stop) => {
          if (gone) stop();
          else stops.push(stop);
        },
        () => undefined,
      );
    };
    keep(
      backend.onOpened((opened) => {
        const info = exits.current.get(opened.id) ?? opened;
        upsert(info);
        setActive(info.id);
        setShown(true);
      }),
    );
    // Only a tab still shown: a session closed here ends after its tab.
    keep(
      backend.onExit((info) => {
        exits.current.set(info.id, info);
        setSessions((list) => list.map((s) => (s.id === info.id ? info : s)));
      }),
    );
    // Into a window of its own, or back: a session docked again is shown.
    keep(
      backend.onMoved((info) => {
        upsert(info);
        if (!info.detached) {
          setActive(info.id);
          setShown(true);
        }
      }),
    );
    // Closed by its own window.
    keep(
      backend.onClosed((id) => {
        setSessions((list) => list.filter((s) => s.id !== id));
        setActive((a) => (a === id ? null : a));
      }),
    );
    backend.sessions().then(
      (list) => {
        if (gone) return;
        // Only the ones not known yet: an event read meanwhile is newer.
        setSessions((known) => [...known, ...list.filter((s) => !known.some((k) => k.id === s.id))]);
        const last = list.at(-1);
        if (last) setActive((a) => a ?? last.id);
      },
      () => undefined,
    );
    return () => {
      gone = true;
      stops.forEach((stop) => {
        stop();
      });
    };
  }, [backend, enabled, upsert]);

  /** Closes a session (its program ends) and drops its tab. */
  const close = useCallback(
    async (id: number) => {
      await backend.close(id);
      setSessions((list) => list.filter((s) => s.id !== id));
      // The panel then shows the last tab, and hides with none.
      setActive((a) => (a === id ? null : a));
    },
    [backend],
  );

  /** Into a window of its own: its tab leaves the panel when Rust says
   * so (`onMoved`). */
  const detach = useCallback((id: number) => backend.detach(id), [backend]);

  // The panel's tabs: the detached sessions are in their windows.
  const docked = sessions.filter((s) => !s.detached);
  return {
    sessions,
    docked,
    active,
    setActive,
    shown: shown && docked.length > 0,
    setShown,
    close,
    detach,
  };
}

export type Terminals = ReturnType<typeof useTerminals>;
