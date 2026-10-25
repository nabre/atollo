// The settings saved as they change, with no Save button (D-137): each
// change waits a moment for the next keystroke, then goes to settings_set,
// one save at a time and always the latest values. A value refused by the
// check stays on screen with its error, and the next change tries again.
// Used by the Settings page and by the first-start page.
import { useCallback, useEffect, useRef, useState } from "react";
import { settingsGet, settingsSet, type Settings, type SettingsView } from "../ipc";
import { useLoad } from "../shared/hooks";

/** The wait after a change before saving it. */
const DELAY = 500;

export type SaveStatus = "idle" | "saving" | "saved";

/** Values that reach the programs through the kit's environment: the
 * editor already running keeps the old ones until it restarts. */
function envChanged(a: Settings, b: Settings): boolean {
  return (
    a.lang !== b.lang || a.programTemp !== b.programTemp || JSON.stringify(a.network) !== JSON.stringify(b.network)
  );
}

export function useSettingsAutosave(onSaved?: () => void) {
  const [s, setS] = useState<Settings | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<SaveStatus>("idle");
  /** A change saved that the running editor does not see yet. */
  const [needsRestart, setNeedsRestart] = useState(false);
  // The settings waiting to be saved. The Gemini key is not one of them: it
  // is saved from the AI page (D-165), null here keeps it.
  const pending = useRef<Settings | null>(null);
  const saved = useRef<Settings | null>(null);
  const timer = useRef<number | undefined>(undefined);
  const running = useRef(false);
  // The settings on screen, for changes made before React renders them.
  const current = useRef<Settings | null>(null);
  const onSavedRef = useRef(onSaved);
  useEffect(() => {
    onSavedRef.current = onSaved;
  }, [onSaved]);

  // The settings read: a read that works clears the error of the one before.
  // The values to edit and to compare with start from it.
  const read = useCallback(
    () =>
      settingsGet().then((v) => {
        setS(v.settings);
        current.current = v.settings;
        saved.current = v.settings;
        return v;
      }),
    [],
  );
  const {
    data: view,
    setData: setView,
    error: loadError,
    reload: load,
    retry,
  } = useLoad<SettingsView | null>(read, null, { onError: "show" });

  const flush = useCallback(async () => {
    window.clearTimeout(timer.current);
    if (running.current) return;
    running.current = true;
    while (pending.current) {
      const next = pending.current;
      pending.current = null;
      setStatus("saving");
      try {
        await settingsSet(next, null);
        if (saved.current && envChanged(saved.current, next)) setNeedsRestart(true);
        saved.current = next;
        setView((v) => (v ? { ...v, problem: null } : v));
        setError(null);
        setStatus("saved");
        onSavedRef.current?.();
      } catch (e: unknown) {
        setError(String(e));
        setStatus("idle");
      }
    }
    running.current = false;
  }, [setView]);

  // Saves what is waiting when the page closes.
  useEffect(
    () => () => {
      if (pending.current) void flush();
    },
    [flush],
  );

  const schedule = useCallback(
    (next: Settings) => {
      pending.current = next;
      window.clearTimeout(timer.current);
      timer.current = window.setTimeout(() => void flush(), DELAY);
    },
    [flush],
  );

  const change = useCallback(
    (make: (s: Settings) => Settings) => {
      if (!current.current) return;
      const next = make(current.current);
      current.current = next;
      setS(next);
      schedule(next);
    },
    [schedule],
  );
  const set = useCallback(
    (patch: Partial<Settings>) => {
      change((s) => ({ ...s, ...patch }));
    },
    [change],
  );
  const net = useCallback(
    (patch: Partial<Settings["network"]>) => {
      change((s) => ({ ...s, network: { ...s.network, ...patch } }));
    },
    [change],
  );
  return { view, s, loadError, error, status, needsRestart, set, net, load, retry };
}
