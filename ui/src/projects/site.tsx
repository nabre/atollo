// The Sito web section of a project's page: a folder of it served on its
// own port or attached to a pooled Caddy/Apache VirtualHost.
import { useState, type SyntheticEvent } from "react";
import { projectSite, serversList, type ProjectInfo, type SiteKind } from "../api";
import { ActionButton, ErrorLine } from "../common";
import { useAction, useLoad } from "../hooks";
import { t } from "../i18n";
import { codicon, Select, TextField } from "../workbench";

interface AliasRow {
  url: string;
  target: string;
}

/** A `.localhost` label as Rust's `host_label` (servers/mod.rs) writes it:
 * only for a site not in a pool yet, whose name Rust has not given. */
function hostLabel(name: string): string {
  const label = name
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 50)
    .replace(/-+$/, "");
  return label || "site";
}

function initialAliases(aliases: Record<string, string> | undefined): AliasRow[] {
  if (!aliases) return [];
  return Object.entries(aliases).map(([url, target]) => ({ url, target }));
}

/** "Servi come sito web": a folder of the project served on its own port or VirtualHost. */
export function SiteForm({ project, onChanged }: { project: ProjectInfo; onChanged: () => void }) {
  const site = project.actions.find((a) => a.id === "site");
  const [folder, setFolder] = useState(site?.static ?? site?.apache ?? site?.caddy ?? ".");
  const [kind, setKind] = useState<SiteKind>(
    site
      ? site.static !== null
        ? "static"
        : site.apache !== null
          ? "apache"
          : site.caddy
            ? "caddy"
            : "php"
      : "static",
  );
  const [server, setServer] = useState<string>(site?.server ?? "auto");
  const [domainsText, setDomainsText] = useState<string>((site?.domains ?? []).join(", "));
  const [aliases, setAliases] = useState<AliasRow[]>(() => initialAliases(site?.aliases));
  const { data: servers } = useLoad(serversList, null, { onError: { fallback: null } });
  const { busy, error, run } = useAction(onChanged);

  const isWebServer = kind === "apache" || kind === "caddy";
  const matchingPools = (servers?.pools ?? []).filter((p) => p.engine === kind);
  // The name Rust gave the site in its pool (`name_sites`: lowercase labels,
  // `-2` when taken); before the site is saved, the project's own label.
  const siteName =
    (servers?.pools ?? []).flatMap((p) => p.sites).find((s) => s.project === project.id && s.action === "site")
      ?.serverName ?? `${hostLabel(project.id)}.localhost`;

  const submit = (e: SyntheticEvent) => {
    e.preventDefault();
    const domains = domainsText
      .split(",")
      .map((d) => d.trim())
      .filter((d) => d.length > 0);
    const aliasMap: Record<string, string> = {};
    for (const row of aliases) {
      const u = row.url.trim();
      const target = row.target.trim();
      if (u.length > 0 || target.length > 0) {
        aliasMap[u] = target;
      }
    }
    run("site", () =>
      projectSite(project.id, folder, kind, {
        server: server === "auto" ? null : server,
        domains,
        aliases: aliasMap,
      }),
    );
  };

  return (
    <form onSubmit={submit} className="flex flex-col gap-3" aria-label={t("Servi come sito web")}>
      <div className="flex flex-wrap items-end gap-2">
        <label className="flex flex-col gap-1">
          <span className="text-vsc-desc">{t("Servi come sito web la cartella")}</span>
          <TextField
            label={t("Servi come sito web la cartella")}
            value={folder}
            onChange={setFolder}
            mono
            className="w-64"
          />
        </label>
        <label className="flex flex-col gap-1">
          <span className="text-vsc-desc">{t("con")}</span>
          <Select<SiteKind>
            label={t("con")}
            value={kind}
            onChange={setKind}
            options={[
              { value: "static", label: t("file statici") },
              { value: "php", label: t("PHP (server integrato)") },
              { value: "caddy", label: t("Caddy") },
              { value: "apache", label: t("Apache") },
            ]}
          />
        </label>
        {isWebServer && (
          <label className="flex flex-col gap-1">
            <span className="text-vsc-desc">{t("Profilo server")}</span>
            <Select<string>
              label={t("Profilo server")}
              value={server}
              onChange={setServer}
              options={[
                { value: "auto", label: t("Automatico (pool per versione)") },
                { value: "dedicated", label: t("Server tutto suo (porta del progetto)") },
                ...matchingPools.map((p) => ({
                  value: p.id,
                  label: `${p.name} (:${String(p.port)})`,
                })),
              ]}
            />
          </label>
        )}
        <ActionButton
          icon={codicon("globe")}
          label={t("Salva")}
          busy={busy === "site"}
          onClick={() => undefined}
          submit
        />
        {site && (
          <ActionButton
            icon={codicon("globe")}
            label={t("Togli il sito")}
            busy={busy === "none"}
            onClick={() => {
              run("none", () => projectSite(project.id, ".", "none"));
            }}
          />
        )}
      </div>

      {isWebServer && (
        <div className="flex flex-col gap-2 border-t border-vsc-border pt-2">
          <label className="flex flex-col gap-1">
            <span className="text-vsc-desc">
              {t("Domini .localhost aggiuntivi (separati da virgola; predefinito: {0})", siteName)}
            </span>
            <TextField
              label={t("Domini .localhost aggiuntivi")}
              value={domainsText}
              onChange={setDomainsText}
              placeholder={`api.${siteName}, admin.${siteName}`}
              mono
              className="w-full max-w-xl"
            />
          </label>

          <div className="flex flex-col gap-1.5">
            <div className="flex items-center gap-2">
              <span className="text-vsc-desc">
                {t("Alias di percorso portabili (es. /media → storage/uploads o @altro-progetto/public)")}
              </span>
              <ActionButton
                icon={codicon("add")}
                label={t("Aggiungi alias")}
                onClick={() => {
                  setAliases((prev) => [...prev, { url: "/media", target: "storage" }]);
                }}
              />
            </div>
            {aliases.map((row, idx) => (
              <div key={idx} className="flex flex-wrap items-center gap-2">
                <TextField
                  label={t("Percorso URL")}
                  value={row.url}
                  onChange={(v) => {
                    setAliases((prev) => prev.map((r, i) => (i === idx ? { ...r, url: v } : r)));
                  }}
                  placeholder="/media"
                  mono
                  className="w-40"
                />
                <span className="text-vsc-desc">→</span>
                <TextField
                  label={t("Cartella relativa")}
                  value={row.target}
                  onChange={(v) => {
                    setAliases((prev) => prev.map((r, i) => (i === idx ? { ...r, target: v } : r)));
                  }}
                  placeholder="storage/uploads"
                  mono
                  className="w-64"
                />
                <ActionButton
                  icon={codicon("trash")}
                  label={t("Rimuovi")}
                  onClick={() => {
                    setAliases((prev) => prev.filter((_, i) => i !== idx));
                  }}
                />
              </div>
            ))}
          </div>

          <p className="w-full text-[12px] text-vsc-desc">
            {server !== "dedicated"
              ? t(
                  "Servito da {0} nel server condiviso delle versioni del progetto (o nel profilo scelto), su http://{1}.",
                  kind === "apache" ? "Apache" : "Caddy",
                  siteName,
                )
              : kind === "apache"
                ? t(
                    "Un Apache solo per questo progetto, sulla sua porta, con .htaccess e il PHP del kit. La configurazione è in data\\projects\\{0}\\apache; le tue direttive vanno in conf.d.",
                    project.id,
                  )
                : t("Un Caddy solo per questo progetto, sulla sua porta, con il PHP del kit.")}
          </p>
          {kind === "caddy" && (
            <p className="w-full text-[12px] text-vsc-desc">
              {t("Caddy non legge .htaccess: le regole di riscrittura si generano dal tipo di progetto.")}
            </p>
          )}
        </div>
      )}

      <div className="w-full empty:hidden">
        <ErrorLine error={error} />
      </div>
    </form>
  );
}
