// A repository opened in the list: its facts, its address, rename and
// delete on GitHub.
import { useState, type SyntheticEvent } from "react";
import { VscCheck, VscClose, VscCopy, VscEdit, VscKey, VscTrash } from "react-icons/vsc";
import { githubDelete, githubRename, type GithubRepo } from "../api";
import { useAction, useCopy } from "../hooks";
import { ActionButton, ErrorLine } from "../common";
import { locale, t } from "../i18n";
import { InlineConfirm } from "../workbench";

export function RepoDetails({
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
  const { copied, copy } = useCopy();
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
            copy(`${r.htmlUrl}.git`);
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
        <InlineConfirm
          label={t("Eliminare {0} da GitHub?", r.fullName)}
          actions={
            <>
              <ActionButton icon={VscTrash} label={t("Elimina")} busy={busy === "delete"} onClick={remove} />
              <ActionButton
                icon={VscClose}
                label={t("Annulla")}
                onClick={() => {
                  setMode(null);
                }}
              />
            </>
          }
        >
          <p className="text-vsc-warning">
            {t(
              "Eliminare {0} da GitHub? Non si può annullare: anche issue, pull request, wiki e release vengono eliminate.",
              r.fullName,
            )}
            {r.local && ` ${t("La copia locale in projects\\{0} resta.", r.local)}`}
          </p>
        </InlineConfirm>
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
