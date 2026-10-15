// The account's repositories in the side bar of Projects, to clone one as a
// new project.
import { useCallback, useEffect, useMemo, useState } from "react";
import {
  VscAccount,
  VscArchive,
  VscCheck,
  VscChevronDown,
  VscChevronRight,
  VscCloudDownload,
  VscCode,
  VscRepo,
  VscGlobe,
  VscLinkExternal,
  VscLock,
  VscOrganization,
  VscRefresh,
  VscRepoForked,
} from "react-icons/vsc";
import type { IconType } from "react-icons";
import {
  editorOpen,
  githubClone,
  githubOpen,
  githubRepos,
  githubSignIn,
  type EditorStatus,
  type GithubRepo,
  type GithubRepos,
} from "../api";
import { useAction } from "../hooks";
import { ActionButton, ErrorLine } from "../common";
import { Mark } from "../tags";
import { t } from "../i18n";
import { useGithub } from "./account";
import { RepoDetails } from "./details";

// Alphabetical whatever order GitHub answers in; app2 before app10.
const byName = (a: GithubRepo, b: GithubRepo) =>
  a.name.localeCompare(b.name, undefined, { sensitivity: "base", numeric: true }) ||
  a.fullName.localeCompare(b.fullName);

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
