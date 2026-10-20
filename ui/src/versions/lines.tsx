// The pieces of the Tools page and of the project page: the list of a
// tool's lines, the button that asks for more, the state of a chosen one.
import type { SetupSnapshot, VersionedTool } from "../api";
import { t } from "../i18n";
import { ActionButton } from "../common";
import { codicon, Link, Notice, Select } from "../workbench";
import { isPresent, offered, present, stepOf } from "./model";

/** The list of a tool's lines: those on disk, then those to download by
 * major version. `first` is an option before them (the project's "as the
 * kit"); a `value` in neither list is shown as it is. */
export function LineSelect({
  id,
  tool,
  value,
  first,
  label,
  disabled,
  onChange,
}: {
  id?: string;
  tool: VersionedTool;
  value: string;
  first?: { value: string; label: string };
  label?: string;
  disabled: boolean;
  onChange: (line: string) => void;
}) {
  const disk = present(tool);
  const groups = offered(tool);
  const known = value === first?.value || disk.some((o) => o.line === value) || tool.available.includes(value);
  return (
    <Select
      id={id}
      label={label}
      value={value}
      disabled={disabled}
      onChange={onChange}
      options={[
        ...(first ? [first] : []),
        { group: t("Scaricate"), options: disk.map((o) => ({ value: o.line, label: o.label })) },
        ...groups.map((g) => ({
          group: t("Da scaricare: {0}.x", g.major),
          options: g.lines.map((l) => ({ value: l, label: t("{0} (da scaricare)", l) })),
        })),
        ...(known ? [] : [{ value, label: t("{0} (non scaricata)", value) }]),
      ]}
      className="min-w-0 flex-1"
    />
  );
}

/** The button that asks the source for the lines it offers now. */
export function RefreshLines({
  tool,
  checking,
  disabled,
  onRefresh,
}: {
  tool: VersionedTool;
  checking: boolean;
  disabled: boolean;
  onRefresh: () => void;
}) {
  if (!tool.downloadable) return null;
  const when =
    tool.checked === null
      ? t("Elenco delle versioni mai scaricato.")
      : t("Elenco delle versioni del {0}.", new Date(tool.checked * 1000).toLocaleString());
  return (
    <ActionButton
      icon={codicon("refresh")}
      label={t("Cerca altre versioni di {0}", tool.name)}
      title={`${t("Cerca altre versioni di {0}", tool.name)}. ${when}`}
      compact
      busy={checking}
      disabled={disabled}
      onClick={onRefresh}
    />
  );
}

/** What happens to a chosen line not on disk: its download, its failure, or
 * a link to download it; until then the default folder (`fallback`) is used,
 * as versions/mod.rs does. */
export function LineState({
  tool,
  line,
  fallback,
  setup,
  error,
  onDownload,
  inList = false,
}: {
  tool: VersionedTool;
  line: string | null;
  fallback: string;
  setup: SetupSnapshot | null;
  error: string | undefined;
  onDownload: (line: string) => void;
  /** The list of the lines shows its download (Tools page). */
  inList?: boolean;
}) {
  if (error) {
    return (
      <Notice tone="warning" className="mt-1 text-[12px]">
        {t("Impossibile leggere le versioni disponibili: {0}", error)}
      </Notice>
    );
  }
  if (line === null || isPresent(tool, line)) return null;
  const step = stepOf(setup, tool, line);
  const waiting = setup?.running && setup.queue.some((q) => q.id === tool.id && q.name === `${tool.name} ${line}`);
  if (inList && (waiting || (step && step.outcome === null && setup?.running))) return null;
  if (step && step.outcome === null && setup?.running) {
    const percent = step.size && step.done !== null ? ` ${String(Math.floor((step.done * 100) / step.size))}%` : "";
    return (
      <Notice tone="info" className="mt-1 text-[12px]">
        {t("Download di {0} {1} in corso…", tool.name, line)}
        {percent}
      </Notice>
    );
  }
  const failure = step?.outcome?.kind === "failed" ? step.outcome.message : null;
  return (
    <Notice tone={failure ? "error" : "warning"} className="mt-1 text-[12px]">
      {failure
        ? t("Download di {0} {1} non riuscito: {2}", tool.name, line, failure)
        : t("{0} {1} non è scaricata: per ora si usa la {2}.", tool.name, line, fallback)}{" "}
      {tool.downloadable && !setup?.running && (
        <Link
          onClick={() => {
            onDownload(line);
          }}
        >
          {failure ? t("Riprova") : t("Scarica")}
        </Link>
      )}
    </Notice>
  );
}
