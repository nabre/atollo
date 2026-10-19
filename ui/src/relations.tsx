// The relations matrix (D-81) on the Tools page: the runtimes the PC must
// provide (with the required components), the relations that do not hold
// (in the row of the component), the lines another component cannot run on
// (in the list of versions), and the whole matrix with the versions the
// projects declare and what to choose or download for them.
import { useCallback, useEffect, useState } from "react";
import { VscCheck, VscCloudDownload, VscRefresh, VscServerEnvironment, VscWarning } from "react-icons/vsc";
import {
  projectVersionsSet,
  projectDbLine,
  relationsMatrix,
  versionsAdd,
  type RelationLink,
  type RelationNeed,
  type RelationRuntime,
  type RelationsMatrix,
  type SetupSnapshot,
} from "./api";
import { t } from "./i18n";
import { ComponentLogo } from "./logos";
import { ActionButton, ErrorLine } from "./common";
import { useAction } from "./hooks";
import { Tag } from "./tags";
import { DataTable } from "./workbench";

/** The matrix, read again when the preparation finishes a step or `key`
 * (the components on disk) changes; the backend rebuilds it only when what
 * it is made from changed. `target`: the system chosen on the Tools page
 * (D-121), whose components alone it relates; this PC's when omitted. */
export function useRelations(setup: SetupSnapshot | null, key: unknown, enabled: boolean, target?: string) {
  const [matrix, setMatrix] = useState<RelationsMatrix | null>(null);
  const load = useCallback(
    (force = false) => {
      if (!enabled) return;
      relationsMatrix(force, target).then(setMatrix, () => {
        setMatrix(null);
      });
    },
    [enabled, target],
  );
  const finished = setup?.steps.filter((s) => s.outcome !== null).length ?? 0;
  useEffect(() => {
    load();
  }, [load, key, finished]);
  return { matrix, load };
}

const component = (id: string) => id.split("/")[0] ?? id;

/** The relations of `id` (or its parts) that do not hold. */
export function LinkProblems({ id, links }: { id: string; links: RelationLink[] }) {
  const bad = links.filter((l) => component(l.from) === id && (l.status === "incompatible" || l.status === "missing"));
  return (
    <>
      {bad.map((l) => (
        <Tag
          key={`${l.from}>${l.to}`}
          tone="warning"
          className="ml-2"
          title={l.note ?? undefined}
          icon={<VscWarning size={11} aria-hidden="true" />}
        >
          {l.status === "missing"
            ? t("{0} richiede {1}", l.fromName, l.toName)
            : t("{0} richiede {1} {2}: il kit usa {3}", l.fromName, l.toName, l.range ?? "", l.uses ?? "")}
        </Tag>
      ))}
    </>
  );
}

/** Line of `to` → the components that cannot run on it. */
export function outsideOf(links: RelationLink[], to: string): Record<string, string[]> {
  const out: Record<string, string[]> = {};
  for (const l of links) {
    if (l.to !== to || l.status === "inactive") continue;
    for (const line of l.outside) out[line] = [...(out[line] ?? []), l.fromName];
  }
  return out;
}

/** The runtimes the PC must provide, with the required components. */
export function RuntimeRows({ runtimes }: { runtimes: RelationRuntime[] }) {
  return (
    <>
      {runtimes.map((r) => {
        const needed = r.neededBy.length > 0;
        const missing = needed && !r.present;
        return (
          <div
            key={r.id}
            data-setting
            className={`flex items-center gap-3 border-l-2 py-2 pr-[14px] pl-3 hover:bg-vsc-list-hover/50 ${
              missing ? "border-l-vsc-orange bg-vsc-orange/5" : "border-l-transparent"
            }`}
          >
            <span className="w-[18px] shrink-0" />
            <span className="flex w-5 shrink-0 justify-center text-vsc-desc">
              <VscServerEnvironment size={18} aria-hidden="true" />
            </span>
            <span className="min-w-0 flex-1 leading-[18px]">
              <span className="font-semibold text-vsc-settings-header">{r.name}</span>
              {r.note && <span className="ml-2 text-vsc-desc">{r.note}</span>}
              {needed && (
                <Tag dashed className="ml-2">
                  {t("per {0}", r.neededBy.join(", "))}
                </Tag>
              )}
            </span>
            <span className="w-16 shrink-0" />
            <span className="w-40 shrink-0 truncate text-right text-[12px]" title={r.detail ?? undefined}>
              {r.present ? (
                <span className="text-vsc-success">
                  <VscCheck className="mr-1 inline" size={12} aria-hidden="true" />
                  {r.detail ?? t("presente")}
                </span>
              ) : needed ? (
                <span className="text-vsc-orange">{t("manca")}</span>
              ) : (
                <span className="text-vsc-desc">{t("non serve")}</span>
              )}
            </span>
          </div>
        );
      })}
    </>
  );
}

function linkStatus(l: RelationLink): { text: string; tone: string } {
  switch (l.status) {
    case "ok":
      return { text: l.uses ? t("sì, con {0}", l.uses) : t("sì"), tone: "text-vsc-success" };
    case "incompatible":
      return { text: t("no: il kit usa {0}", l.uses ?? ""), tone: "text-vsc-orange" };
    case "missing":
      return { text: t("manca {0}", l.toName), tone: "text-vsc-orange" };
    case "optional":
      return { text: t("facoltativo, non installato"), tone: "text-vsc-desc" };
    case "inactive":
      return { text: t("{0} non installato", l.fromName), tone: "text-vsc-desc" };
  }
}

function needText(n: RelationNeed): string {
  switch (n.status) {
    case "ok":
      return t("usa {0}", n.uses ?? "");
    case "choose":
      return t("usa {0}: scegli {1}", n.uses ?? "", n.line ?? "");
    case "download":
      return t("da scaricare: {0}", n.line ?? "");
    case "unavailable":
      return t("nessuna versione del kit va bene");
    case "notInstalled":
      return n.line ? t("{0} non installato (va bene {1})", n.name, n.line) : t("{0} non installato", n.name);
    case "unknown":
      return t("versione non riconosciuta");
  }
}

/** What the kit does for a project's declared version: choose a line on
 * disk, or download one and choose it (`project_versions_set` downloads). */
function NeedAction({
  project,
  need,
  busy,
  running,
  onRun,
}: {
  project: string;
  need: RelationNeed;
  busy: boolean;
  running: boolean;
  onRun: (task: () => Promise<unknown>) => void;
}) {
  const line = need.line;
  if (!line || (need.status !== "choose" && need.status !== "download")) return null;
  return (
    <ActionButton
      icon={need.status === "download" ? VscCloudDownload : VscCheck}
      label={
        need.status === "download"
          ? need.server
            ? t("Scarica {0}", line)
            : t("Scarica e usa {0}", line)
          : t("Usa {0}", line)
      }
      small
      busy={busy}
      disabled={running}
      onClick={() => {
        onRun(() =>
          // A server: it downloads, or the project's database goes on that
          // version, on its own port beside the others (D-102).
          need.server
            ? need.status === "download"
              ? versionsAdd(need.component, line)
              : projectDbLine(project, need.component as "mysql" | "mongodb", line)
            : projectVersionsSet(project, need.component, line),
        );
      }}
    />
  );
}

/** The whole matrix: relations between components, then the projects'
 * declared versions. */
export function RelationsSection({
  matrix,
  setup,
  onReload,
}: {
  matrix: RelationsMatrix;
  setup: SetupSnapshot | null;
  onReload: (force?: boolean) => void;
}) {
  const reload = useCallback(() => {
    onReload();
  }, [onReload]);
  const { busy, error, run } = useAction(reload);
  const running = setup?.running ?? false;
  const active = matrix.links.filter((l) => l.status !== "inactive");
  const problems =
    active.filter((l) => l.status === "incompatible" || l.status === "missing").length +
    matrix.projects.flatMap((p) => p.needs).filter((n) => n.status !== "ok").length;
  const cell = "border-t border-vsc-border px-2 py-0.5 text-left align-top";
  return (
    <section aria-labelledby="relations" className="border-t border-vsc-border">
      <header className="flex items-center gap-2 px-3 py-1.5">
        <div className="min-w-0 flex-1">
          <h2 id="relations" className="text-[13px] font-semibold">
            {t("Relazioni")}
          </h2>
          <p className="text-[12px] text-vsc-desc">
            {t(
              "Cosa richiede ogni strumento e con quali versioni, e le versioni che chiedono i progetti. Si ricalcola solo quando cambia qualcosa.",
            )}
          </p>
        </div>
        <ActionButton
          icon={VscRefresh}
          label={t("Ricalcola le relazioni")}
          compact
          onClick={() => {
            onReload(true);
          }}
        />
      </header>
      <details>
        <summary className="cursor-pointer px-3 py-1.5 text-[12px] text-vsc-desc hover:bg-vsc-list-hover">
          {t("Matrice delle relazioni: {0} attive, {1} da sistemare", active.length, problems)}
        </summary>
        <div className="overflow-x-auto px-4 pb-3">
          <DataTable
            caption={t("Relazioni fra gli strumenti")}
            columns={[t("Strumento"), t("Richiede"), t("Versioni"), t("Stato")]}
            className="w-full border-collapse text-[12px]"
          >
            {matrix.links.map((l) => {
              const s = linkStatus(l);
              return (
                <tr key={`${l.from}>${l.to}`} className="hover:bg-vsc-list-hover" title={l.note ?? undefined}>
                  <th scope="row" className={`${cell} font-normal whitespace-nowrap`}>
                    <span className="inline-flex items-center gap-1.5">
                      <ComponentLogo id={component(l.from)} size={12} />
                      {l.fromName}
                    </span>
                  </th>
                  <td className={`${cell} whitespace-nowrap`}>
                    {l.toName}
                    {l.optional && <span className="ml-1 text-vsc-desc">{t("(facoltativo)")}</span>}
                  </td>
                  <td className={`${cell} font-mono`}>{l.range ?? "—"}</td>
                  <td className={`${cell} ${s.tone}`}>
                    {s.text}
                    {l.outside.length > 0 && l.status !== "inactive" && (
                      <span className="ml-1 text-vsc-desc">{t("(non con {0})", l.outside.join(", "))}</span>
                    )}
                  </td>
                </tr>
              );
            })}
          </DataTable>
          {matrix.projects.length > 0 && (
            <DataTable
              caption={t("Versioni chieste dai progetti")}
              captionVisible
              columns={[t("Progetto"), t("Chiede"), t("Stato"), null]}
              className="mt-3 w-full border-collapse text-[12px]"
            >
              {matrix.projects.flatMap((p) =>
                p.needs.map((n) => (
                  <tr key={`${p.project}:${n.source}`} className="hover:bg-vsc-list-hover">
                    <th scope="row" className={`${cell} font-normal`}>
                      {p.project}
                    </th>
                    <td className={cell}>
                      <span className="inline-flex items-center gap-1.5">
                        <ComponentLogo id={n.component} size={12} />
                        {n.name} <span className="font-mono">{n.declared}</span>
                      </span>
                      <span className="ml-1 text-vsc-desc">({n.source})</span>
                    </td>
                    <td className={`${cell} ${n.status === "ok" ? "text-vsc-success" : "text-vsc-orange"}`}>
                      {needText(n)}
                    </td>
                    <td className={`${cell} text-right`}>
                      <NeedAction
                        project={p.project}
                        need={n}
                        busy={busy === `${p.project}:${n.source}`}
                        running={running}
                        onRun={(task) => {
                          run(`${p.project}:${n.source}`, () => task().then(() => undefined));
                        }}
                      />
                    </td>
                  </tr>
                )),
              )}
            </DataTable>
          )}
          <ErrorLine error={error} />
          <p className="mt-2 text-[11px] text-vsc-desc">
            {t("Calcolata il {0} per {1}.", new Date(matrix.generated * 1000).toLocaleString(), matrix.platform)}
          </p>
        </div>
      </details>
    </section>
  );
}
