// The fields of a framework's settings: lines of commands, the project's
// actions, the kit's tools, the .env, the database.
import { useEffect, useRef, useState } from "react";
import type { EnvVar, FrameworkAction, FrameworkVariant } from "../api";
import { t } from "../i18n";
import { VscAdd } from "react-icons/vsc";
import { IconButton, Link } from "../workbench";
import { useComponents } from "./model";
import { ToolTag } from "./tags";

/** Lines of commands in a textarea, one per line. The text is kept as typed
 * and only replaced when the value changes from outside (Ripristina). */
export function LinesField({
  label,
  value,
  onChange,
  placeholder,
}: {
  label: string;
  value: string[] | undefined;
  onChange: (v: string[]) => void;
  placeholder?: string;
}) {
  const [text, setText] = useState((value ?? []).join("\n"));
  const sent = useRef(value);
  useEffect(() => {
    if (value !== sent.current) {
      sent.current = value;
      setText((value ?? []).join("\n"));
    }
  }, [value]);
  return (
    <label className="block">
      <span className="mb-1 block text-vsc-desc">{label}</span>
      <textarea
        value={text}
        placeholder={placeholder}
        spellCheck={false}
        wrap="off"
        rows={Math.min(8, Math.max(2, text.split("\n").length + 1))}
        onChange={(e) => {
          setText(e.target.value);
          const next = e.target.value.split("\n");
          sent.current = next;
          onChange(next);
        }}
        className="input w-full font-mono text-[12px]"
      />
    </label>
  );
}

const formatActions = (a: FrameworkAction[] | undefined) =>
  (a ?? []).map((x) => (x.name ? `${x.name}: ${x.command}` : x.command)).join("\n");

function parseActions(text: string): FrameworkAction[] {
  return text
    .split("\n")
    .filter((l) => l.trim())
    .map((l) => {
      const at = l.indexOf(":");
      return at < 0
        ? { name: "", command: l.trim() }
        : { name: l.slice(0, at).trim(), command: l.slice(at + 1).trim() };
    });
}

/** The actions saved in the project, one per line as «Nome: comando». */
export function ActionsField({
  value,
  onChange,
}: {
  value: FrameworkAction[] | undefined;
  onChange: (v: FrameworkAction[]) => void;
}) {
  const [text, setText] = useState(formatActions(value));
  const sent = useRef(value);
  useEffect(() => {
    if (value !== sent.current) {
      sent.current = value;
      setText(formatActions(value));
    }
  }, [value]);
  return (
    <label className="block">
      <span className="mb-1 block text-vsc-desc">{t("Azioni del progetto (Nome: comando)")}</span>
      <textarea
        value={text}
        placeholder="Avvia: npm run dev"
        spellCheck={false}
        wrap="off"
        rows={Math.min(6, Math.max(2, text.split("\n").length + 1))}
        onChange={(e) => {
          setText(e.target.value);
          const next = parseActions(e.target.value);
          sent.current = next;
          onChange(next);
        }}
        className="input w-full font-mono text-[12px]"
      />
    </label>
  );
}

/** The kit's tools as tags, with «+» to add one from the catalog. */
export function NeedsField({
  label,
  value,
  onChange,
}: {
  label: string;
  value: string[] | undefined;
  onChange: (v: string[]) => void;
}) {
  const components = useComponents();
  const [adding, setAdding] = useState(false);
  const ids = value ?? [];
  const others = components.filter((c) => !ids.includes(c.id) && c.group !== "app");
  return (
    <div>
      <span className="mb-1 block text-vsc-desc">{label}</span>
      <div className="flex flex-wrap items-center gap-1">
        {ids.map((id) => (
          <ToolTag
            key={id}
            id={id}
            components={components}
            onRemove={() => {
              onChange(ids.filter((x) => x !== id));
            }}
          />
        ))}
        {adding ? (
          <select
            autoFocus
            aria-label={t("Aggiungi uno strumento del kit")}
            value=""
            onBlur={() => {
              setAdding(false);
            }}
            onChange={(e) => {
              if (e.target.value) onChange([...ids, e.target.value]);
              setAdding(false);
            }}
            className="input"
          >
            <option value="">{t("scegli uno strumento…")}</option>
            {others.map((c) => (
              <option key={c.id} value={c.id}>
                {c.name}
              </option>
            ))}
          </select>
        ) : (
          <IconButton
            icon={VscAdd}
            label={t("Aggiungi uno strumento del kit")}
            onClick={() => {
              setAdding(true);
            }}
          />
        )}
      </div>
    </div>
  );
}

const formatEnv = (v: EnvVar[] | undefined) =>
  (v ?? []).map((e) => (e.key ? `${e.key}=${e.value}` : e.value)).join("\n");

function parseEnv(text: string): EnvVar[] {
  return text
    .split("\n")
    .filter((l) => l.trim())
    .map((l) => {
      const at = l.indexOf("=");
      return at < 0 ? { key: l.trim(), value: "" } : { key: l.slice(0, at).trim(), value: l.slice(at + 1).trim() };
    });
}

/** The variables of .env, one per line as «CHIAVE=valore». */
export function EnvField({ value, onChange }: { value: EnvVar[] | undefined; onChange: (v: EnvVar[]) => void }) {
  const [text, setText] = useState(formatEnv(value));
  const sent = useRef(value);
  useEffect(() => {
    if (value !== sent.current) {
      sent.current = value;
      setText(formatEnv(value));
    }
  }, [value]);
  return (
    <label className="block">
      <span className="mb-1 block text-vsc-desc">{t("Variabili del file .env (CHIAVE=valore)")}</span>
      <textarea
        value={text}
        placeholder="APP_NAME={name}"
        spellCheck={false}
        wrap="off"
        rows={Math.min(6, Math.max(2, text.split("\n").length + 1))}
        onChange={(e) => {
          setText(e.target.value);
          const next = parseEnv(e.target.value);
          sent.current = next;
          onChange(next);
        }}
        className="input w-full font-mono text-[12px]"
      />
    </label>
  );
}

/** The kit's database server where the project gets its database. */
function DatabaseField({ value, onChange }: { value: string | undefined; onChange: (v: string | undefined) => void }) {
  return (
    <label className="block">
      <span className="mb-1 block text-vsc-desc">{t("Database del kit per il progetto")}</span>
      <select
        value={value ?? ""}
        onChange={(e) => {
          onChange(e.target.value || undefined);
        }}
        className="input w-full"
      >
        <option value="">{t("Nessuno")}</option>
        <option value="mariadb">MariaDB (MySQL)</option>
        <option value="mongodb">MongoDB</option>
      </select>
    </label>
  );
}

export function TextField({ label, value, onChange }: { label: string; value: string; onChange: (v: string) => void }) {
  return (
    <label className="block">
      <span className="mb-1 block text-vsc-desc">{label}</span>
      <input
        value={value}
        onChange={(e) => {
          onChange(e.target.value);
        }}
        className="input w-full"
      />
    </label>
  );
}

/** What an addition does with one base. */
export function VariantFields({
  title,
  note,
  variant,
  onChange,
  onRemove,
}: {
  title: string;
  note: string;
  variant: FrameworkVariant;
  onChange: (v: FrameworkVariant) => void;
  onRemove: () => void;
}) {
  return (
    <fieldset className="space-y-2 rounded border border-vsc-border p-2.5">
      <legend className="px-1 font-semibold">{title}</legend>
      {note && <p className="text-vsc-desc">{note}</p>}
      <LinesField
        label={t("Crea il progetto al posto della base (facoltativo)")}
        value={variant.create}
        onChange={(create) => {
          onChange({ ...variant, create });
        }}
      />
      <LinesField
        label={t("Poi, nella cartella del progetto")}
        value={variant.steps}
        onChange={(steps) => {
          onChange({ ...variant, steps });
        }}
      />
      <ActionsField
        value={variant.actions}
        onChange={(actions) => {
          onChange({ ...variant, actions });
        }}
      />
      <EnvField
        value={variant.env}
        onChange={(env) => {
          onChange({ ...variant, env });
        }}
      />
      <DatabaseField
        value={variant.database}
        onChange={(database) => {
          const next = { ...variant };
          if (database) next.database = database;
          else delete next.database;
          onChange(next);
        }}
      />
      <NeedsField
        label={t("Altri strumenti del kit")}
        value={variant.needs}
        onChange={(needs) => {
          onChange({ ...variant, needs });
        }}
      />
      <Link onClick={onRemove}>{t("Togli questa base")}</Link>
    </fieldset>
  );
}
