// The sessions of the app's terminal (macOS and Linux) and the panel's
// state: kept by the main window while pages change, filled by the events
// of Rust. No xterm.js here, so the window loads it only with the panel.
import { useCallback, useEffect, useState } from "react";
import type { PtyBackend, PtyInfo } from "./api";

/** The sessions and the panel's state; `enabled` false (Windows, or before
 * the system is known) reads and listens to nothing. */
export function useTerminals(backend: PtyBackend, enabled: boolean) {
  const [sessions, setSessions] = useState<PtyInfo[]>([]);
  const [active, setActive] = useState<number | null>(null);
  const [shown, setShown] = useState(false);

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
      backend.onOpened((info) => {
        upsert(info);
        setActive(info.id);
        setShown(true);
      }),
    );
    // Only a tab still shown: a session closed here ends after its tab.
    keep(
      backend.onExit((info) => {
        setSessions((list) => list.map((s) => (s.id === info.id ? info : s)));
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

  return {
    sessions,
    active,
    setActive,
    shown: shown && sessions.length > 0,
    setShown,
    close,
  };
}

export type Terminals = ReturnType<typeof useTerminals>;
