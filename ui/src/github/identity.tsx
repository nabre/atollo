// Who signs the commits of the kit's Git (`user.name`, `user.email` of the
// kit's .gitconfig, D-137): filled from the GitHub account at sign-in, with
// GitHub's noreply address; changed here, each field saved when it loses
// the focus.
import { useState } from "react";
import { gitIdentityGet, gitIdentitySet, type GitIdentity } from "../api";
import { ErrorLine } from "../shared/common";
import { useAction, useLoad } from "../shared/hooks";
import { t } from "../i18n";
import { SettingItem, TextField } from "../workbench";

export function GitIdentitySetting() {
  const { data, setData } = useLoad<GitIdentity | null | undefined>(gitIdentityGet, undefined, { onError: "keep" });
  const [typed, setTyped] = useState<Partial<GitIdentity>>({});
  const { error, run } = useAction();
  if (data === undefined) return null;
  const title = t("Chi firma i commit");
  if (data === null) {
    return (
      <SettingItem title={title} keywords="git user.name user.email commit author">
        <p className="text-vsc-desc">{t("Git non è ancora nel kit: si installa con GitHub CLI.")}</p>
      </SettingItem>
    );
  }
  const value = { ...data, ...typed };
  const save = () => {
    if (value.name === data.name && value.email === data.email) return;
    run("identity", async () => {
      const saved = await gitIdentitySet(value);
      setData(saved);
      setTyped({});
    });
  };
  return (
    <SettingItem
      title={title}
      description={t(
        "Il nome e l'e-mail dei commit fatti con il Git del kit. All'accesso a GitHub, se sono vuoti, prendono il nome dell'account e il suo indirizzo «noreply», così l'e-mail vera resta privata.",
      )}
      keywords="git user.name user.email commit author identity"
      modified={data.name !== "" || data.email !== ""}
    >
      <div className="grid gap-2 sm:grid-cols-2">
        <TextField
          id="git-name"
          label={t("Nome")}
          value={value.name}
          autoComplete="off"
          onChange={(v) => {
            setTyped((x) => ({ ...x, name: v }));
          }}
          onBlur={save}
          className="w-full"
        />
        <TextField
          id="git-email"
          type="email"
          label={t("E-mail")}
          value={value.email}
          autoComplete="off"
          onChange={(v) => {
            setTyped((x) => ({ ...x, email: v }));
          }}
          onBlur={save}
          className="w-full"
        />
      </div>
      <ErrorLine error={error} />
    </SettingItem>
  );
}
