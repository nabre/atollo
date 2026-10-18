// The bar under the Tools page, as the editor's notification of a pending
// change: what Apply installs and removes, the installation running, "Update
// all"; Apply asks first what it will do.
import { useState } from "react";
import { VscCheck, VscCircleSlash, VscCloudDownload, VscRefresh, VscTrash } from "react-icons/vsc";
import { setupCancel, type SetupSnapshot } from "../api";
import { t } from "../i18n";
import { ActionButton } from "../common";
import { setupStatusText } from "../setup";
import { InlineConfirm } from "../workbench";
import type { KitVersions } from "../versions";
import { linesOf, sizeText } from "./model";
import type { ComponentsState } from "./state";

export function ApplyBar({
  state,
  kit,
  setup,
  firstRun = false,
}: {
  state: ComponentsState;
  kit: KitVersions;
  setup: SetupSnapshot | null;
  firstRun?: boolean;
}) {
  const [confirm, setConfirm] = useState(false);
  const [cancelError, setCancelError] = useState<string | null>(null);
  const { plan, apply } = state;
  const running = setup?.running ?? false;
  const changed = plan.install.length > 0 || plan.remove.length > 0;
  const go = (updateAll: boolean) => {
    setConfirm(false);
    apply(updateAll);
  };
  return (
    <div className="flex shrink-0 flex-wrap items-center gap-2 border-t border-vsc-border px-5 py-2">
      {confirm ? (
        <InlineConfirm
          row
          question={t("Applicare le modifiche?")}
          className="w-full"
          textClassName="min-w-0 flex-1"
          onCancel={() => {
            setConfirm(false);
          }}
          actions={
            <>
              <ActionButton
                icon={VscCheck}
                label={t("Conferma")}
                primary
                onClick={() => {
                  go(false);
                }}
              />
              <ActionButton
                icon={VscRefresh}
                label={t("Indietro")}
                onClick={() => {
                  setConfirm(false);
                }}
              />
            </>
          }
        >
          {plan.install.length > 0 &&
            t("Da installare: {0} (circa {1}).", plan.install.map((c) => c.name).join(", "), sizeText(plan.mb)) + " "}
          {plan.remove.length > 0 &&
            t(
              "Da rimuovere: {0}.",
              plan.remove
                .map((c) => {
                  const n = linesOf(kit.byId.get(c.id)).length;
                  return n > 1 ? `${c.name} (${t("{0} versioni", n)})` : c.name;
                })
                .join(", "),
            )}
        </InlineConfirm>
      ) : (
        <>
          <span
            role="status"
            className={`min-w-0 flex-1 ${setup?.error || cancelError ? "text-vsc-error" : "text-vsc-desc"}`}
          >
            {running
              ? setupStatusText(setup)
              : (setup?.error ?? cancelError)
                ? (setup?.error ?? cancelError)
                : changed
                  ? t(
                      "{0} da installare ({1}), {2} da rimuovere",
                      plan.install.length,
                      sizeText(plan.mb),
                      plan.remove.length,
                    )
                  : t("Nessuna modifica")}
          </span>
          {running && (
            <ActionButton
              icon={VscCircleSlash}
              label={setup?.cancelling ? t("Annullamento…") : t("Annulla")}
              disabled={setup?.cancelling ?? false}
              onClick={() => {
                setCancelError(null);
                setupCancel().catch((e: unknown) => {
                  setCancelError(String(e));
                });
              }}
            />
          )}
          {!firstRun && (
            <ActionButton
              icon={VscRefresh}
              label={t("Aggiorna tutto")}
              disabled={running}
              title={
                running
                  ? t("Un'installazione è già in corso")
                  : t("Scarica le versioni nuove di tutto ciò che è installato")
              }
              onClick={() => {
                go(true);
              }}
            />
          )}
          <ActionButton
            icon={
              firstRun ? VscCloudDownload : plan.remove.length > 0 && plan.install.length === 0 ? VscTrash : VscCheck
            }
            label={firstRun ? t("Installa") : t("Applica")}
            primary
            disabled={running || (!changed && !firstRun)}
            title={running ? t("Un'installazione è già in corso") : undefined}
            onClick={() => {
              if (firstRun) go(false);
              else setConfirm(true);
            }}
          />
        </>
      )}
    </div>
  );
}
