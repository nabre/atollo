// GitHub page: the kit's GitHub account and its repositories (port of
// nabre/dev-kit setup/extensions/github-repos). One sign-in for everything:
// this page, the kit's Git (editor, consoles, agents) and GitHub CLI, saved
// in data\gh (tauri/src/github.rs, DECISIONS.md D-70).
import { useCallback, useEffect, useMemo, useState, type SyntheticEvent } from "react";
import {
  VscAccount,
  VscArchive,
  VscCheck,
  VscChevronDown,
  VscChevronRight,
  VscClose,
  VscCloudDownload,
  VscCode,
  VscCopy,
  VscEdit,
  VscRepo,
  VscGlobe,
  VscKey,
  VscLinkExternal,
  VscLock,
  VscOrganization,
  VscRefresh,
  VscRepoForked,
  VscSignIn,
  VscSignOut,
  VscTrash,
} from "react-icons/vsc";
import type { IconType } from "react-icons";
import {
  editorOpen,
  githubClone,
  githubDelete,
  githubOpen,
  githubRename,
  githubRepos,
  githubSignIn,
  githubSignInCancel,
  githubSignOut,
  githubStatus,
  onGithub,
  type EditorStatus,
  type GithubRepo,
  type GithubRepos,
  type GithubStatus,
} from "./api";
import { ActionButton, ErrorLine, useAction } from "./pages";
import { Mark } from "./tags";
import { useSubscription } from "./hooks";
import { locale, rich, t } from "./i18n";

/** The account's state, kept current by the "github" events of the sign-in. */
export function useGithub() {
  const [status, setStatus] = useState<GithubStatus | null>(null);
  const reload = useCallback(() => {
    githubStatus().then(setStatus, () => {
      setStatus(null);
    });
  }, []);
  useEffect(reload, [reload]);
  useSubscription(onGithub, setStatus);
  return { status, setStatus, reload };
}

// Alphabetical whatever order GitHub answers in; app2 before app10.
const byName = (a: GithubRepo, b: GithubRepo) =>
  a.name.localeCompare(b.name, undefined, { sensitivity: "base", numeric: true }) ||
  a.fullName.localeCompare(b.fullName);

/** The GitHub account of the kit, in the Settings (D-76). */
export function GithubAccount() {
  const { status, setStatus, reload } = useGithub();
  const signIn = (deleteScope: boolean) => githubSignIn(deleteScope).then(setStatus);
  return <Account status={status} onSignIn={signIn} onChanged={reload} />;
}

/** The account's repositories, to clone one as a new project (D-76). */
export function GithubRepositories({ editor, onChanged }: { editor: EditorStatus | null; onChanged: () => void }) {
  const { status, setStatus } = useGithub();
  const [repos, setRepos] = useState<GithubRepos | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const signedIn = status?.signedIn ?? false;
  const account = signedIn ? (status?.login ?? "") : null;

  const load = useCallback(() => {
    githubRepos().then(
      (r) => {
        setLoading(false);
        setLoadError(null);
        setRepos(r);
      },
      (e: unknown) => {
        setLoading(false);
        setLoadError(String(e));
      },
    );
  }, []);
  // Signed in, or another account: the list of that account.
  useEffect(() => {
    if (account !== null) load();
  }, [account, load]);

  const signIn = (deleteScope: boolean) => githubSignIn(deleteScope).then(setStatus);
  // A list left by the account before is not shown.
  const shown = repos && (account === "" || repos.login === account) ? repos : null;

  return (
    <div className="space-y-3">
      {!signedIn && (
        <p className="text-vsc-desc">
          {status === null
            ? t("Caricamento…")
            : t("Collega il tuo account GitHub nelle Impostazioni per vedere qui i tuoi repository.")}
        </p>
      )}
      {signedIn && (
        <Repositories
          repos={shown}
          loading={loading}
          error={loadError}
          editor={editor}
          onReload={() => {
            setLoading(true);
            load();
          }}
          onRepos={setRepos}
          onSignIn={signIn}
          onEditor={onChanged}
        />
      )}
    </div>
  );
}

function Account({
  status,
  onSignIn,
  onChanged,
}: {
  status: GithubStatus | null;
  onSignIn: (deleteScope: boolean) => Promise<void>;
  onChanged: () => void;
}) {
  const { busy, error, run } = useAction(onChanged);
  const [confirm, setConfirm] = useState(false);
  const [copied, setCopied] = useState(false);
  if (!status) return <p className="text-vsc-desc">{t("Caricamento…")}</p>;
  return (
    <section aria-labelledby="gh-account" className="space-y-2 border-t border-vsc-border px-3 py-2">
      <h2 id="gh-account" className="text-[13px] font-semibold">
        {t("Account GitHub del kit")}
      </h2>
      <p className="text-vsc-desc">
        {rich(
          t(
            "Un solo accesso per tutto il kit: questa pagina, Git (anche nell'editor, nelle console e negli agenti) e GitHub CLI. Resta nel kit, in {0}.",
          ),
          <span className="font-mono">data\gh</span>,
        )}
      </p>
      {status.signingIn ? (
        <div className="space-y-2" role="status">
          {status.code ? (
            <>
              <p>{t("Nella pagina di GitHub che si è aperta scrivi questo codice, poi autorizza GitHub CLI:")}</p>
              <p className="font-mono text-[22px] tracking-widest select-text">{status.code}</p>
            </>
          ) : (
            <p className="text-vsc-desc">{t("Accesso in preparazione…")}</p>
          )}
          <div className="flex flex-wrap gap-2">
            {status.code && (
              <>
                <ActionButton
                  icon={copied ? VscCheck : VscCopy}
                  label={copied ? t("Codice copiato") : t("Copia il codice")}
                  onClick={() => {
                    void navigator.clipboard.writeText(status.code ?? "").then(() => {
                      setCopied(true);
                    });
                  }}
                />
                <ActionButton
                  icon={VscLinkExternal}
                  label={t("Apri la pagina di GitHub")}
                  onClick={() => {
                    run("device", () => githubOpen(null));
                  }}
                />
              </>
            )}
            <ActionButton
              icon={VscClose}
              label={t("Annulla")}
              onClick={() => {
                run("cancel", githubSignInCancel);
              }}
            />
          </div>
        </div>
      ) : status.signedIn ? (
        <div className="flex flex-wrap items-center gap-2">
          <VscCheck className="text-vsc-success" size={16} aria-hidden="true" />
          <span>{status.login ? t("Accesso fatto come {0}", `@${status.login}`) : t("accesso fatto")}</span>
          <span className="ml-auto flex flex-wrap items-center gap-2">
            {confirm ? (
              <>
                <span>{t("Uscire da GitHub? Anche Git, l'editor e gli agenti del kit perdono l'accesso.")}</span>
                <ActionButton
                  icon={VscSignOut}
                  label={t("Esci")}
                  busy={busy === "out"}
                  onClick={() => {
                    setConfirm(false);
                    run("out", githubSignOut);
                  }}
                />
                <ActionButton
                  icon={VscClose}
                  label={t("Annulla")}
                  onClick={() => {
                    setConfirm(false);
                  }}
                />
              </>
            ) : (
              <ActionButton
                icon={VscSignOut}
                label={t("Esci")}
                title={t("Toglie l'accesso salvato nel kit")}
                onClick={() => {
                  setConfirm(true);
                }}
              />
            )}
          </span>
        </div>
      ) : (
        <div className="flex flex-wrap items-center gap-2">
          <ActionButton
            icon={VscSignIn}
            label={t("Accedi a GitHub")}
            primary
            busy={busy === "in"}
            onClick={() => {
              setCopied(false);
              run("in", () => onSignIn(false));
            }}
          />
          <span className="text-[12px] text-vsc-desc">
            {t("Si apre github.com nel browser: scrivi il codice che compare qui e autorizza GitHub CLI.")}
          </span>
        </div>
      )}
      <ErrorLine error={error ?? (status.signingIn ? null : status.error)} />
      {status.signedIn && (
        <p className="text-[12px] text-vsc-desc">
          {t(
            "Uscire dal kit non revoca le sessioni sul sito del fornitore: se perdi la chiavetta, revocale dalle impostazioni del tuo account.",
          )}
        </p>
      )}
    </section>
  );
}

function Repositories({
  repos,
  loading,
  error,
  editor,
  onReload,
  onRepos,
  onSignIn,
  onEditor,
}: {
  repos: GithubRepos | null;
  loading: boolean;
  error: string | null;
  editor: EditorStatus | null;
  onReload: () => void;
  onRepos: (r: GithubRepos) => void;
  onSignIn: (deleteScope: boolean) => Promise<void>;
  onEditor: () => void;
}) {
  const [filter, setFilter] = useState("");
  const [open, setOpen] = useState<string | null>(null);
  const [note, setNote] = useState<string | null>(null);
  const { busy, error: actionError, run } = useAction();

  const groups = useMemo(() => {
    if (!repos) return [];
    const words = filter.toLowerCase().split(/\s+/).filter(Boolean);
    const shown = repos.repos
      .filter((r) => {
        const text = `${r.fullName} ${r.description} ${r.language}`.toLowerCase();
        return words.every((w) => text.includes(w));
      })
      .sort(byName);
    const owners = [...new Set(shown.map((r) => r.owner))].sort((a, b) =>
      a === repos.login ? -1 : b === repos.login ? 1 : a.localeCompare(b),
    );
    return owners.map((o) => ({ owner: o, repos: shown.filter((r) => r.owner === o) }));
  }, [repos, filter]);

  const update = (fullName: string, change: (r: GithubRepo) => GithubRepo | null) => {
    if (!repos) return;
    onRepos({
      ...repos,
      repos: repos.repos.flatMap((r) => {
        if (r.fullName !== fullName) return [r];
        const next = change(r);
        return next ? [next] : [];
      }),
    });
  };

  const clone = (r: GithubRepo) => {
    setNote(null);
    run(`clone:${r.fullName}`, async () => {
      const project = await githubClone(r.fullName);
      update(r.fullName, (x) => ({ ...x, local: project }));
      setNote(t("{0} clonato in projects\\{1}.", r.fullName, project));
    });
  };

  return (
    <section aria-labelledby="gh-repos" className="space-y-2">
      <div className="sticky top-0 z-10 -mt-1 flex items-center gap-1 bg-vsc-sidebar py-1">
        <h2 id="gh-repos" className="sr-only">
          {t("Repository")}
        </h2>
        <input
          type="search"
          aria-label={t("Cerca nei repository")}
          placeholder={t("Cerca nei repository")}
          value={filter}
          onChange={(e) => {
            setFilter(e.target.value);
          }}
          className="min-w-0 flex-1 input"
        />
        <ActionButton icon={VscRefresh} label={t("Aggiorna")} compact busy={loading} onClick={onReload} />
      </div>
      <ErrorLine error={error ?? actionError} />
      {note && (
        <p role="status" className="flex items-start gap-2">
          <VscCheck className="mt-0.5 shrink-0 text-vsc-success" size={16} aria-hidden="true" />
          <span className="select-text">{note}</span>
        </p>
      )}
      {repos === null ? (
        !error && <p className="text-vsc-desc">{t("Caricamento…")}</p>
      ) : repos.repos.length === 0 ? (
        <p className="text-vsc-desc">{t("Nessun repository in questo account.")}</p>
      ) : groups.length === 0 ? (
        <p className="text-vsc-desc">{t("Nessun repository corrisponde alla ricerca.")}</p>
      ) : (
        groups.map((g) => (
          <div key={g.owner} className="space-y-1">
            {groups.length > 1 || g.owner !== repos.login ? (
              <h3 className="flex items-center gap-2 text-[12px] text-vsc-desc">
                {g.owner === repos.login ? (
                  <VscAccount size={14} aria-hidden="true" />
                ) : (
                  <VscOrganization size={14} aria-hidden="true" />
                )}
                {g.owner} · {g.repos.length}
              </h3>
            ) : null}
            <ul aria-label={g.owner} className="-mx-3">
              {g.repos.map((r) => {
                const expanded = open === r.fullName;
                return (
                  <li key={r.fullName}>
                    <div
                      className={`group flex h-[22px] items-center gap-0.5 border-l-2 pr-1 pl-2.5 hover:bg-vsc-list-hover ${
                        r.local ? "border-l-vsc-success" : "border-l-transparent"
                      }`}
                    >
                      <button
                        type="button"
                        aria-expanded={expanded}
                        onClick={() => {
                          setOpen(expanded ? null : r.fullName);
                        }}
                        className="flex min-w-0 flex-1 items-center gap-1.5 text-left"
                      >
                        {expanded ? (
                          <VscChevronDown size={16} aria-hidden="true" className="shrink-0" />
                        ) : (
                          <VscChevronRight size={16} aria-hidden="true" className="shrink-0" />
                        )}
                        <RepoIcon repo={r} />
                        <span className={`truncate ${r.local ? "text-vsc-success" : ""}`}>{r.name}</span>
                        {r.local && <span className="sr-only">{t("clonato")}</span>}
                        <RepoFlags repo={r} />
                      </button>
                      {r.local ? (
                        <ActionButton
                          icon={VscCode}
                          label={t("Editor")}
                          ghost
                          compact
                          busy={busy === `editor:${r.fullName}`}
                          disabled={editor?.installed === false}
                          title={t("Apre {0} nell'editor", r.local)}
                          onClick={() => {
                            const project = r.local ?? "";
                            run(`editor:${r.fullName}`, async () => {
                              await editorOpen(project);
                              onEditor();
                            });
                          }}
                        />
                      ) : (
                        <ActionButton
                          icon={VscCloudDownload}
                          label={t("Clona")}
                          ghost
                          compact
                          busy={busy === `clone:${r.fullName}`}
                          title={t("Copia {0} in projects", r.fullName)}
                          onClick={() => {
                            clone(r);
                          }}
                        />
                      )}
                      <ActionButton
                        icon={VscLinkExternal}
                        label="GitHub"
                        compact
                        ghost
                        title={t("Apre {0} su github.com", r.fullName)}
                        onClick={() => {
                          run(`web:${r.fullName}`, () => githubOpen(r.fullName));
                        }}
                      />
                    </div>
                    {expanded && (
                      <RepoDetails
                        repo={r}
                        canDelete={repos.canDelete}
                        onRenamed={(renamed) => {
                          update(r.fullName, () => renamed);
                          setOpen(renamed.fullName);
                          setNote(
                            renamed.local
                              ? t(
                                  "{0} rinominato in {1}. La copia locale projects\\{2} ora punta al nuovo nome.",
                                  r.fullName,
                                  renamed.fullName,
                                  renamed.local,
                                )
                              : t("{0} rinominato in {1}.", r.fullName, renamed.fullName),
                          );
                        }}
                        onDeleted={() => {
                          update(r.fullName, () => null);
                          setOpen(null);
                          setNote(
                            r.local
                              ? t(
                                  "{0} eliminato da GitHub. La copia locale resta in projects\\{1}.",
                                  r.fullName,
                                  r.local,
                                )
                              : t("{0} eliminato da GitHub.", r.fullName),
                          );
                        }}
                        onSignIn={onSignIn}
                      />
                    )}
                  </li>
                );
              })}
            </ul>
          </div>
        ))
      )}
    </section>
  );
}

/** A repository, or a fork; green when it is cloned in projects. */
function RepoIcon({ repo }: { repo: GithubRepo }) {
  const Icon = repo.fork ? VscRepoForked : VscRepo;
  const label = repo.fork ? t("fork") : undefined;
  return (
    <span
      role={label ? "img" : undefined}
      aria-label={label}
      title={label}
      className={`inline-flex shrink-0 ${repo.local ? "text-vsc-success" : "text-vsc-desc"}`}
    >
      <Icon size={16} aria-hidden="true" />
    </span>
  );
}

/** Private or public, and archived: small marks after the name. */
function RepoFlags({ repo }: { repo: GithubRepo }) {
  const flags: { icon: IconType; label: string }[] = [
    repo.private ? { icon: VscLock, label: t("privato") } : { icon: VscGlobe, label: t("pubblico") },
    ...(repo.archived ? [{ icon: VscArchive, label: t("archiviato") }] : []),
  ];
  return (
    <>
      {flags.map(({ icon: Icon, label }) => (
        <Mark key={label} label={label} className="text-vsc-desc">
          <Icon size={12} aria-hidden="true" />
        </Mark>
      ))}
    </>
  );
}

function RepoDetails({
  repo: r,
  canDelete,
  onRenamed,
  onDeleted,
  onSignIn,
}: {
  repo: GithubRepo;
  canDelete: boolean | null;
  onRenamed: (r: GithubRepo) => void;
  onDeleted: () => void;
  onSignIn: (deleteScope: boolean) => Promise<void>;
}) {
  const [mode, setMode] = useState<"rename" | "delete" | "permission" | null>(null);
  const [name, setName] = useState(r.name);
  const [copied, setCopied] = useState(false);
  const { busy, error, run } = useAction();

  const rename = (e: SyntheticEvent) => {
    e.preventDefault();
    const next = name.trim();
    if (next === r.name) {
      setMode(null);
      return;
    }
    run("rename", async () => {
      const done = await githubRename(r.fullName, next);
      setMode(null);
      onRenamed({ ...done.repo, local: done.local });
    });
  };
  const remove = () => {
    if (canDelete === false) {
      setMode("permission");
      return;
    }
    run("delete", async () => {
      const result = await githubDelete(r.fullName);
      if (result === "deleted") onDeleted();
      else setMode("permission");
    });
  };

  return (
    <div className="space-y-1.5 py-1.5 pr-3 pl-8">
      <p className="select-text">
        <span className="font-semibold">{r.fullName}</span>
        {r.description && <span className="text-vsc-desc"> — {r.description}</span>}
      </p>
      <p className="text-[12px] text-vsc-desc">
        {[
          r.private ? t("Privato") : t("Pubblico"),
          r.language,
          `★ ${String(r.stars)}`,
          r.pushedAt
            ? t(
                "ultimo push {0}",
                new Intl.DateTimeFormat(locale(), { dateStyle: "medium", timeStyle: "short" }).format(
                  new Date(r.pushedAt),
                ),
              )
            : "",
          r.local ? t("clonato in projects\\{0}", r.local) : t("non clonato"),
        ]
          .filter(Boolean)
          .join(" · ")}
      </p>
      <div className="flex flex-wrap gap-2">
        <ActionButton
          icon={copied ? VscCheck : VscCopy}
          label={copied ? t("Indirizzo copiato") : t("Copia l'indirizzo per Git")}
          onClick={() => {
            void navigator.clipboard.writeText(`${r.htmlUrl}.git`).then(() => {
              setCopied(true);
            });
          }}
        />
        {r.admin && (
          <>
            <ActionButton
              icon={VscEdit}
              label={t("Rinomina")}
              onClick={() => {
                setName(r.name);
                setMode(mode === "rename" ? null : "rename");
              }}
            />
            <ActionButton
              icon={VscTrash}
              label={t("Elimina da GitHub")}
              onClick={() => {
                setMode(mode === "delete" ? null : "delete");
              }}
            />
          </>
        )}
      </div>
      {mode === "rename" && (
        <form onSubmit={rename} className="flex flex-wrap items-end gap-2" aria-label={t("Rinomina {0}", r.fullName)}>
          <label className="flex flex-col gap-1">
            <span className="text-[12px] text-vsc-desc">{t("Nuovo nome del repository su GitHub")}</span>
            <input
              autoFocus
              required
              pattern="[A-Za-z0-9._\-]+"
              value={name}
              onChange={(e) => {
                setName(e.target.value);
              }}
              className="w-64 input"
            />
          </label>
          <ActionButton
            icon={VscEdit}
            label={t("Rinomina")}
            primary
            submit
            busy={busy === "rename"}
            onClick={() => undefined}
          />
          <ActionButton
            icon={VscClose}
            label={t("Annulla")}
            onClick={() => {
              setMode(null);
            }}
          />
          <p className="basis-full text-[12px] text-vsc-desc">
            {t(
              "GitHub reindirizza i vecchi indirizzi. La copia locale, se c'è, punta al nuovo nome; la sua cartella in projects resta com'è.",
            )}
          </p>
        </form>
      )}
      {mode === "delete" && (
        <div role="alertdialog" aria-label={t("Eliminare {0} da GitHub?", r.fullName)} className="space-y-2">
          <p className="text-vsc-warning">
            {t(
              "Eliminare {0} da GitHub? Non si può annullare: anche issue, pull request, wiki e release vengono eliminate.",
              r.fullName,
            )}
            {r.local && ` ${t("La copia locale in projects\\{0} resta.", r.local)}`}
          </p>
          <div className="flex flex-wrap gap-2">
            <ActionButton icon={VscTrash} label={t("Elimina")} busy={busy === "delete"} onClick={remove} />
            <ActionButton
              icon={VscClose}
              label={t("Annulla")}
              onClick={() => {
                setMode(null);
              }}
            />
          </div>
        </div>
      )}
      {mode === "permission" && (
        <div role="status" className="space-y-2">
          <p>
            {t(
              "L'accesso del kit non ha il permesso di eliminare i repository. Accedi di nuovo concedendolo, poi premi ancora «Elimina da GitHub».",
            )}
          </p>
          <ActionButton
            icon={VscKey}
            label={t("Concedi il permesso")}
            primary
            busy={busy === "grant"}
            onClick={() => {
              run("grant", async () => {
                await onSignIn(true);
                setMode(null);
                window.scrollTo({ top: 0 });
              });
            }}
          />
        </div>
      )}
      <ErrorLine error={error} />
    </div>
  );
}
