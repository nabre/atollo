// The ways to make a new project in the side bar of Projects: an empty
// folder, or a clone of a Git address.
import { useState, type SyntheticEvent } from "react";
import { VscCloudDownload } from "react-icons/vsc";
import { projectClone, projectCreate } from "../api";
import { useAction } from "../hooks";
import { ErrorLine } from "../common";
import { t } from "../i18n";
import { Button } from "../workbench";

export function NewProject({
  onDone,
  inputRef,
}: {
  onDone: (name: string) => void;
  inputRef: React.RefObject<HTMLInputElement | null>;
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
      <input
        id="new-project-name"
        ref={inputRef}
        required
        value={name}
        onChange={(e) => {
          setName(e.target.value);
        }}
        className="input w-full"
      />
      <Button variant="primary" wide submit busy={busy === "create"}>
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
  return (
    <form onSubmit={submit} className="space-y-1.5" aria-label={t("Da un indirizzo Git")}>
      <p className="font-semibold">{t("Da un indirizzo Git")}</p>
      <label className="flex flex-col gap-1">
        <span className="text-[12px] text-vsc-desc">{t("owner/repository di GitHub o indirizzo https://")}</span>
        <input
          required
          value={source}
          onChange={(e) => {
            setSource(e.target.value);
          }}
          className="w-full input"
        />
      </label>
      <label className="flex flex-col gap-1">
        <span className="text-[12px] text-vsc-desc">{t("Nome della cartella (facoltativo)")}</span>
        <input
          value={name}
          onChange={(e) => {
            setName(e.target.value);
          }}
          className="w-full input"
        />
      </label>
      <Button wide submit busy={busy === "clone"}>
        <VscCloudDownload size={14} aria-hidden="true" />
        {t("Clona")}
      </Button>
      <ErrorLine error={error} />
    </form>
  );
}
