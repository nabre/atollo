// The lines of a tool as the lists show them, and the hooks that keep them
// current: the sources asked once a day, a reload after every download.
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { versionsAvailable, type SetupSnapshot, type VersionedTool } from "../ipc";

/** A list older than this (seconds) is asked again. */
const STALE = 24 * 60 * 60;

/** A full version and the parts that come with it, as the installers
 * write it: `8.4.13 (Composer 2.8.12)` → `8.4.13`, `["Composer 2.8.12"]`. */
export function splitVersion(version: string | null): { main: string | null; parts: string[] } {
  const m = version?.match(/^(.*?) \((.*)\)$/);
  return m ? { main: m[1] ?? null, parts: (m[2] ?? "").split(", ") } : { main: version, parts: [] };
}

/** The Composer that came with a version of PHP: `2.8.12`. */
export function composerOf(version: string | null): string | null {
  const part = splitVersion(version).parts.find((p) => p.startsWith("Composer "));
  return part ? part.slice("Composer ".length) : null;
}

/** `22.11` with its full version, as the lists show it. */
export function lineText(line: string, version: string | null): string {
  const main = splitVersion(version).main;
  return main && main !== line ? `${line} (${main})` : line;
}

/** The lines on disk: the default folder, then the saved ones. */
export function present(tool: VersionedTool): { line: string; label: string }[] {
  const out: { line: string; label: string }[] = [];
  if (tool.default) {
    out.push({
      line: tool.default.line,
      label: lineText(tool.default.line, tool.default.version),
    });
  }
  for (const s of tool.saved) out.push({ line: s.line, label: lineText(s.line, s.version) });
  return out;
}

export function isPresent(tool: VersionedTool, line: string): boolean {
  return tool.default?.line === line || tool.saved.some((s) => s.line === line);
}

/** The offered lines not on disk, by major version, newest first. */
export function offered(tool: VersionedTool): { major: string; lines: string[] }[] {
  if (!tool.downloadable) return [];
  const groups: { major: string; lines: string[] }[] = [];
  for (const line of tool.available) {
    if (isPresent(tool, line)) continue;
    const major = line.split(".")[0] ?? line;
    const last = groups.at(-1);
    if (last?.major === major) last.lines.push(line);
    else groups.push({ major, lines: [line] });
  }
  return groups;
}

/** The download of `line` of `tool` in the preparation task. */
export function stepOf(setup: SetupSnapshot | null, tool: VersionedTool, line: string) {
  return [...(setup?.steps ?? [])].reverse().find((s) => s.id === tool.id && s.name.endsWith(` ${line}`)) ?? null;
}

/** Asks the sources of the tools whose list is missing or older than a
 * day, once per view; `reload` after. */
export function useOffers(tools: VersionedTool[] | null, reload: () => void, target?: string) {
  const [checking, setChecking] = useState<ReadonlySet<string>>(new Set());
  const [failed, setFailed] = useState<Record<string, string>>({});
  const asked = useRef(new Set<string>());
  const refresh = useCallback(
    (ids: string[]) => {
      for (const id of ids) asked.current.add(id);
      setChecking((c) => new Set([...c, ...ids]));
      void Promise.allSettled(
        ids.map((id) =>
          versionsAvailable(id, target).then(
            () => {
              setFailed((f) => Object.fromEntries(Object.entries(f).filter(([k]) => k !== id)));
            },
            (e: unknown) => {
              setFailed((f) => ({ ...f, [id]: String(e) }));
            },
          ),
        ),
      ).then(() => {
        setChecking((c) => new Set([...c].filter((id) => !ids.includes(id))));
        reload();
      });
    },
    [reload, target],
  );
  useEffect(() => {
    if (tools === null) return;
    const now = Date.now() / 1000;
    const stale = tools
      // Installed or not: the menu to install a version needs it too.
      .filter((x) => x.downloadable && !asked.current.has(x.id))
      .filter((x) => x.checked === null || now - x.checked > STALE)
      .map((x) => x.id);
    if (stale.length > 0) refresh(stale);
  }, [tools, refresh]);
  // The same object while nothing changes: the rows of the Tools table
  // that get it render only then.
  return useMemo(() => ({ checking, failed, refresh }), [checking, failed, refresh]);
}

/** Reloads after every step the preparation task finishes. */
export function useReloadOnSteps(setup: SetupSnapshot | null, load: () => void) {
  const finished = setup?.steps.filter((s) => s.outcome !== null).length ?? 0;
  useEffect(() => {
    if (finished > 0) load();
  }, [finished, load]);
}
