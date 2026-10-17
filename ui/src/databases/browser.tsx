// The content of a running database server (DECISIONS.md D-79), read only:
// as VSCodium's explorer and an editor, the databases with their tables or
// collections on the left, one page of rows of the chosen one on the right.
// The server's own databases (mysql, admin…) under "Di sistema", closed.
import { useCallback, useEffect, useState } from "react";
import { VscChevronLeft, VscChevronRight, VscDatabase, VscEye, VscRefresh, VscTable } from "react-icons/vsc";
import { DB_PAGE, dbBrowseRows, dbBrowseTree, type DbDatabase, type DbRows, type ServiceInfo } from "../api";
import { locale, t } from "../i18n";
import { ErrorLine } from "../common";
import { DataTable, IconButton, List, ListRow, ProgressBar } from "../workbench";

type Engine = ServiceInfo["id"];

const number = (n: number) => new Intl.NumberFormat(locale()).format(n);

/** A value as text: null, binary data and Extended JSON made readable. */
function cell(v: unknown, engine: Engine): { text: string; dim?: boolean; num?: boolean } {
  if (v === null || v === undefined) return { text: engine === "mongodb" ? "null" : "NULL", dim: true };
  if (typeof v === "number") return { text: String(v), num: true };
  if (typeof v === "object" && !Array.isArray(v)) {
    const o = v as Record<string, unknown>;
    if (Object.keys(o).length === 1) {
      if (typeof o.$bytes === "number") return { text: t("{0} byte binari", number(o.$bytes)), dim: true };
      if (typeof o.$oid === "string") return { text: `ObjectId(${o.$oid})` };
      if (typeof o.$date === "string") return { text: o.$date };
      for (const k of ["$numberLong", "$numberDecimal", "$numberDouble", "$numberInt"]) {
        if (typeof o[k] === "string") return { text: o[k], num: true };
      }
    }
  }
  return { text: typeof v === "string" ? v : JSON.stringify(v) };
}

function TableView({
  engine,
  line,
  database,
  table,
}: {
  engine: Engine;
  line: string | null;
  database: string;
  table: string;
}) {
  const [offset, setOffset] = useState(0);
  const [page, setPage] = useState<DbRows | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [tick, setTick] = useState(0);
  // The request answered last: loading until it is the current one.
  const [answered, setAnswered] = useState<string | null>(null);
  const request = `${String(offset)}:${String(tick)}`;
  const loading = answered !== request;
  useEffect(() => {
    let alive = true;
    dbBrowseRows(engine, database, table, offset, line).then(
      (p) => {
        if (!alive) return;
        setPage(p);
        setError(null);
        setAnswered(request);
      },
      (e: unknown) => {
        if (!alive) return;
        setError(String(e));
        setAnswered(request);
      },
    );
    return () => {
      alive = false;
    };
  }, [engine, line, database, table, offset, request]);
  const total = page?.total ?? 0;
  const shown = page?.rows.length ?? 0;
  return (
    <div className="flex min-h-0 min-w-0 flex-1 flex-col">
      <div className="flex h-[35px] shrink-0 items-center gap-2 border-b border-vsc-border px-3">
        <VscTable size={16} aria-hidden="true" className="shrink-0" />
        <h4 className="min-w-0 truncate font-mono text-[13px] select-text">
          <span className="text-vsc-desc">{database}.</span>
          {table}
        </h4>
        <span className="flex-1" />
        {page && total > 0 && (
          <span className="text-[12px] text-vsc-desc tabular-nums">
            {t("{0}–{1} di {2}", number(offset + 1), number(offset + shown), number(total))}
          </span>
        )}
        <IconButton
          icon={VscChevronLeft}
          label={t("Pagina precedente")}
          disabled={offset === 0 || loading}
          onClick={() => {
            setOffset(Math.max(0, offset - DB_PAGE));
          }}
        />
        <IconButton
          icon={VscChevronRight}
          label={t("Pagina successiva")}
          disabled={offset + shown >= total || loading}
          onClick={() => {
            setOffset(offset + DB_PAGE);
          }}
        />
        <IconButton
          icon={VscRefresh}
          label={t("Aggiorna")}
          busy={loading}
          onClick={() => {
            setTick(tick + 1);
          }}
        />
      </div>
      <ProgressBar active={loading} />
      <div className="px-3 empty:hidden">
        <ErrorLine error={error} />
      </div>
      {page && page.rows.length === 0 && !error ? (
        <p className="p-3 text-vsc-desc">{t("Nessuna riga.")}</p>
      ) : (
        page && (
          // A new page starts at the top.
          <div key={page.offset} className="min-h-0 flex-1 overflow-auto">
            <DataTable
              className="border-separate border-spacing-0 font-mono text-[12px]"
              stickyHead
              head={page.columns.map((c) => (
                <th
                  key={c.name}
                  scope="col"
                  title={c.kind || undefined}
                  className="h-[26px] border-r border-b border-vsc-border px-2 text-left font-semibold whitespace-nowrap"
                >
                  {c.name}
                  {c.kind && <span className="ml-1.5 font-normal text-vsc-desc">{c.kind}</span>}
                </th>
              ))}
            >
              {page.rows.map((r, i) => (
                <tr key={offset + i} className="hover:bg-vsc-list-hover">
                  {page.columns.map((c, j) => {
                    const v = cell(r[j], engine);
                    return (
                      <td
                        key={c.name}
                        title={v.text.length > 40 ? v.text : undefined}
                        className={`h-[22px] max-w-[22rem] truncate border-r border-b border-vsc-border px-2 select-text ${
                          v.dim ? "text-vsc-desc italic" : ""
                        } ${v.num ? "text-right tabular-nums" : ""}`}
                      >
                        {v.text}
                      </td>
                    );
                  })}
                </tr>
              ))}
            </DataTable>
          </div>
        )
      )}
    </div>
  );
}

/** `line`: another version of the server, on its own port (PLAN.md §13.2);
 * null: the version the server runs. */
export function DbBrowser({ engine, line = null }: { engine: Engine; line?: string | null }) {
  const [tree, setTree] = useState<DbDatabase[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [open, setOpen] = useState<Set<string> | null>(null);
  const [systemOpen, setSystemOpen] = useState(false);
  const [chosen, setChosen] = useState<{ database: string; table: string } | null>(null);
  const load = useCallback(() => {
    dbBrowseTree(engine, line).then(
      (dbs) => {
        setTree(dbs);
        setError(null);
        setLoading(false);
        // At first the project databases open.
        setOpen((o) => o ?? new Set(dbs.filter((d) => !d.system).map((d) => d.name)));
      },
      (e: unknown) => {
        setError(String(e));
        setLoading(false);
      },
    );
  }, [engine, line]);
  useEffect(load, [load]);
  const refresh = () => {
    setLoading(true);
    load();
  };

  const toggle = (name: string) => {
    const next = new Set(open);
    if (next.has(name)) next.delete(name);
    else next.add(name);
    setOpen(next);
  };
  const row = (d: DbDatabase, depth: number) => (
    <ListRow
      key={d.name}
      icon={<VscDatabase size={14} aria-hidden="true" />}
      label={d.name}
      depth={depth}
      twistie={open?.has(d.name) ? "open" : "closed"}
      decorations={<span className="text-vsc-desc tabular-nums">{d.tables.length}</span>}
      onSelect={() => {
        toggle(d.name);
      }}
    >
      <ul className="m-0 list-none p-0">
        {d.tables.length === 0 && (
          <li className="h-[22px] text-[12px] leading-[22px] text-vsc-desc" style={{ paddingLeft: 40 + depth * 8 }}>
            {engine === "mongodb" ? t("Nessuna collezione") : t("Nessuna tabella")}
          </li>
        )}
        {d.tables.map((tb) => (
          <ListRow
            key={tb.name}
            depth={depth + 1}
            icon={tb.view ? <VscEye size={14} aria-hidden="true" /> : <VscTable size={14} aria-hidden="true" />}
            label={tb.name}
            title={tb.view ? t("{0} (vista)", tb.name) : tb.name}
            decorations={
              tb.rows !== null && (
                <span className="text-vsc-desc tabular-nums" title={t("Righe, stima")}>
                  {number(tb.rows)}
                </span>
              )
            }
            selected={chosen?.database === d.name && chosen.table === tb.name}
            onSelect={() => {
              setChosen({ database: d.name, table: tb.name });
            }}
          />
        ))}
      </ul>
    </ListRow>
  );
  const user = (tree ?? []).filter((d) => !d.system);
  const system = (tree ?? []).filter((d) => d.system);

  return (
    <div className="flex min-h-0 flex-1 border-t border-vsc-border">
      <div className="flex w-64 shrink-0 flex-col border-r border-vsc-border">
        <div className="flex h-[35px] shrink-0 items-center gap-2 pr-1 pl-3">
          <span className="min-w-0 flex-1 truncate text-[12px] font-semibold">{t("Database")}</span>
          <IconButton icon={VscRefresh} label={t("Aggiorna")} busy={loading} onClick={refresh} />
        </div>
        <div className="px-3 empty:hidden">
          <ErrorLine error={error} />
        </div>
        <div className="min-h-0 flex-1 overflow-y-auto px-1 pb-2">
          {tree === null ? (
            !error && <p className="px-2 text-vsc-desc">{t("Caricamento…")}</p>
          ) : (
            <List label={t("Contenuto del server")}>
              {user.map((d) => row(d, 0))}
              {system.length > 0 && (
                <ListRow
                  label={t("Di sistema")}
                  description={String(system.length)}
                  twistie={systemOpen ? "open" : "closed"}
                  onSelect={() => {
                    setSystemOpen(!systemOpen);
                  }}
                >
                  <ul className="m-0 list-none p-0">{system.map((d) => row(d, 1))}</ul>
                </ListRow>
              )}
            </List>
          )}
        </div>
      </div>
      {chosen ? (
        <TableView
          key={`${chosen.database}\u0000${chosen.table}`}
          engine={engine}
          line={line}
          database={chosen.database}
          table={chosen.table}
        />
      ) : (
        <div className="flex flex-1 items-center justify-center p-6 text-center text-vsc-desc">
          {engine === "mongodb"
            ? t("Scegli una collezione per vederne i documenti, in sola lettura.")
            : t("Scegli una tabella per vederne le righe, in sola lettura.")}
        </div>
      )}
    </div>
  );
}
