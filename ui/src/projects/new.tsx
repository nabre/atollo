// The ways to make a new project in the side bar of Projects: an empty
// folder, or a clone of a Git address.
import { useState, type SyntheticEvent } from "react";
import { VscCloudDownload } from "react-icons/vsc";
import { projectClone, projectCreate } from "../api";
import { useAction } from "../hooks";
import { ErrorLine } from "../common";
import { t } from "../i18n";
import { Button, TextField } from "../workbench";

export function NewProject({
  onDone,
  focus,
}: {
  onDone: (name: string) => void;
  /** Raised to put the focus on the name (the side bar's «Nuovo progetto»). */
  focus: number;
}) {
  const [name, setName] = useState("");
  const { busy, error, run } = useAction();
  const submit = (e: SyntheticEvent) => {
    e.preventDefault();
    run("create", async () => {
      onDone(await projectCreate(name));
    });
  };
  return (
    <form onSubmit={submit} className="space-y-1.5" aria-label={t("Vuoto")}>
      <label htmlFor="new-project-name" className="block font-semibold">
        {t("Vuoto")}
      </label>
      <p className="text-[12px] text-vsc-desc">{t("Nome della cartella")}</p>
      {/* Remounted on each request, so that it takes the focus again. */}
      <TextField
        key={focus}
        id="new-project-name"
        label={t("Vuoto")}
        autoFocus={focus > 0}
        value={name}
        onChange={setName}
        className="w-full"
      />
      {/* Required, as the native field was: nothing to submit while empty. */}
      <Button variant="primary" wide submit disabled={name === ""} busy={busy === "create"}>
        {t("Crea")}
      </Button>
      <ErrorLine error={error} />
    </form>
  );
}

export function CloneProject({ onDone }: { onDone: (name: string) => void }) {
  const [source, setSource] = useState("");
  const [name, setName] = useState("");
  const { busy, error, run } = useAction();
  const submit = (e: SyntheticEvent) => {
    e.preventDefault();
    run("clone", async () => {
      onDone(await projectClone(source, name.trim() === "" ? null : name));
    });
  };
  const sourceLabel = t("owner/repository di GitHub o indirizzo https://");
  const nameLabel = t("Nome della cartella (facoltativo)");
  return (
    <form onSubmit={submit} className="space-y-1.5" aria-label={t("Da un indirizzo Git")}>
      <p className="font-semibold">{t("Da un indirizzo Git")}</p>
      <label className="flex flex-col gap-1">
        <span className="text-[12px] text-vsc-desc">{sourceLabel}</span>
        <TextField label={sourceLabel} value={source} onChange={setSource} className="w-full" />
      </label>
      <label className="flex flex-col gap-1">
        <span className="text-[12px] text-vsc-desc">{nameLabel}</span>
        <TextField label={nameLabel} value={name} onChange={setName} className="w-full" />
      </label>
      <Button wide submit disabled={source === ""} busy={busy === "clone"}>
        <span className="inline-flex items-center gap-1">
          <VscCloudDownload size={14} aria-hidden="true" />
          {t("Clona")}
        </span>
      </Button>
      <ErrorLine error={error} />
    </form>
  );
}
