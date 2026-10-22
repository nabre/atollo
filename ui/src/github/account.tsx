// The kit's GitHub account in Settings: sign-in with a code shown here,
// sign-out.
import { useState } from "react";
import {
  githubOpen,
  githubSignIn,
  githubSignInCancel,
  githubSignOut,
  githubStatus,
  onGithub,
  type GithubStatus,
} from "../api";
import { useAction, useCopy, useLoad } from "../shared/hooks";
import { ActionButton, ErrorLine, LoadError } from "../shared/common";
import { useSubscription } from "../shared/hooks";
import { rich, t } from "../i18n";
import { Codicon, codicon, InlineConfirm } from "../workbench";

/** The account's state, kept current by the "github" events of the sign-in;
 * `error` is why it could not be read, `retry` reads it again. */
export function useGithub() {
  const {
    data: status,
    error,
    setData: setStatus,
    reload,
    retry,
  } = useLoad<GithubStatus | null>(githubStatus, null, { onError: "show" });
  useSubscription(onGithub, setStatus);
  return { status, error, setStatus, reload, retry };
}

/** The GitHub account of the kit, in the Settings (D-76). */
export function GithubAccount() {
  const { status, error, setStatus, reload, retry } = useGithub();
  const signIn = (deleteScope: boolean) => githubSignIn(deleteScope).then(setStatus);
  // Not read: why, with «Riprova», instead of a «Caricamento…» that never ends.
  if (!status && error) return <LoadError error={error} onRetry={retry} className="mx-3 my-2" />;
  return <Account status={status} onSignIn={signIn} onChanged={reload} />;
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
  const { copied, copy, reset: resetCopied } = useCopy();
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
                  icon={copied ? codicon("check") : codicon("copy")}
                  label={copied ? t("Codice copiato") : t("Copia il codice")}
                  onClick={() => {
                    copy(status.code ?? "");
                  }}
                />
                <ActionButton
                  icon={codicon("link-external")}
                  label={t("Apri la pagina di GitHub")}
                  onClick={() => {
                    run("device", () => githubOpen(null));
                  }}
                />
              </>
            )}
            <ActionButton
              icon={codicon("close")}
              label={t("Annulla")}
              onClick={() => {
                run("cancel", githubSignInCancel);
              }}
            />
          </div>
        </div>
      ) : status.signedIn ? (
        <div className="flex flex-wrap items-center gap-2">
          <Codicon name="check" className="text-vsc-success" size={16} />
          <span>{status.login ? t("Accesso fatto come {0}", `@${status.login}`) : t("accesso fatto")}</span>
          <span className="ml-auto flex flex-wrap items-center gap-2">
            {confirm ? (
              <InlineConfirm
                row
                question={t("Uscire da GitHub? Anche Git, l'editor e gli agenti del kit perdono l'accesso.")}
                onCancel={() => {
                  setConfirm(false);
                }}
                actions={
                  <>
                    <ActionButton
                      icon={codicon("sign-out")}
                      label={t("Esci")}
                      busy={busy === "out"}
                      onClick={() => {
                        setConfirm(false);
                        run("out", githubSignOut);
                      }}
                    />
                    <ActionButton
                      icon={codicon("close")}
                      label={t("Annulla")}
                      onClick={() => {
                        setConfirm(false);
                      }}
                    />
                  </>
                }
              />
            ) : (
              <ActionButton
                icon={codicon("sign-out")}
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
            icon={codicon("sign-in")}
            label={t("Accedi a GitHub")}
            primary
            busy={busy === "in"}
            onClick={() => {
              resetCopied();
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
