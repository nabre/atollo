// The kit's GitHub account in Settings: sign-in with a code shown here,
// sign-out.
import { useState } from "react";
import { VscCheck, VscClose, VscCopy, VscLinkExternal, VscSignIn, VscSignOut } from "react-icons/vsc";
import {
  githubOpen,
  githubSignIn,
  githubSignInCancel,
  githubSignOut,
  githubStatus,
  onGithub,
  type GithubStatus,
} from "../api";
import { useAction, useCopy, useLoad } from "../hooks";
import { ActionButton, ErrorLine } from "../common";
import { useSubscription } from "../hooks";
import { rich, t } from "../i18n";
import { InlineConfirm } from "../workbench";

/** The account's state, kept current by the "github" events of the sign-in. */
export function useGithub() {
  const {
    data: status,
    setData: setStatus,
    reload,
  } = useLoad<GithubStatus | null>(githubStatus, null, { onError: { fallback: null } });
  useSubscription(onGithub, setStatus);
  return { status, setStatus, reload };
}

/** The GitHub account of the kit, in the Settings (D-76). */
export function GithubAccount() {
  const { status, setStatus, reload } = useGithub();
  const signIn = (deleteScope: boolean) => githubSignIn(deleteScope).then(setStatus);
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
                  icon={copied ? VscCheck : VscCopy}
                  label={copied ? t("Codice copiato") : t("Copia il codice")}
                  onClick={() => {
                    copy(status.code ?? "");
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
              <InlineConfirm
                row
                question={t("Uscire da GitHub? Anche Git, l'editor e gli agenti del kit perdono l'accesso.")}
                onCancel={() => {
                  setConfirm(false);
                }}
                actions={
                  <>
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
                }
              />
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
