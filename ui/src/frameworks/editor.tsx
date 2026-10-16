// One base or addition as a group of settings (a tab of Settings, or a
// search result), with the bar of a modified setting and «Ripristina».
import { urlOpen, type FrameworkPiece, type FrameworksView } from "../api";
import { ErrorLine } from "../common";
import { t } from "../i18n";
import { Link, Select, SettingItem, SettingsGroup } from "../workbench";
import { bases, cleaned, pieceTab, slotLabel, textOf, type Frameworks } from "./model";
import { ActionsField, DatabaseField, EnvField, LinesField, NeedsField, TextField, VariantFields } from "./fields";

function PieceItem({
  piece,
  official,
  all,
  view,
  onChange,
  onRemove,
}: {
  piece: FrameworkPiece;
  official: FrameworkPiece | undefined;
  all: FrameworkPiece[];
  view: FrameworksView | null;
  onChange: (p: FrameworkPiece) => void;
  onRemove: () => void;
}) {
  const modified = official ? cleaned(official) !== cleaned(piece) : true;
  const category = piece.kind === "base" ? t("Base") : slotLabel(piece.slot);
  const desc = textOf(view, piece.desc);
  const note = textOf(view, piece.note);
  const keywords = [piece.id, ...(piece.create ?? []), ...(piece.steps ?? [])].join(" ");
  const others = bases(all).filter((b) => piece.on?.[b.id] === undefined);
  const set = (patch: Partial<FrameworkPiece>) => {
    onChange({ ...piece, ...patch });
  };
  return (
    <SettingItem
      category={category}
      title={piece.name || piece.id}
      description={note ? `${desc} ${note}` : desc}
      modified={modified}
      keywords={keywords}
    >
      <div className="space-y-2.5">
        {!official && (
          <>
            <TextField
              label={t("Nome")}
              value={piece.name}
              onChange={(name) => {
                set({ name });
              }}
            />
            <TextField
              label={t("Descrizione")}
              value={piece.desc}
              onChange={(d) => {
                set({ desc: d });
              }}
            />
            {piece.kind === "addon" && (
              <TextField
                label={t("Gruppo (due aggiunte dello stesso gruppo non vanno insieme)")}
                value={piece.slot ?? ""}
                onChange={(slot) => {
                  set({ slot });
                }}
              />
            )}
            <TextField
              label={t("Pagina ufficiale (https://)")}
              value={piece.home}
              onChange={(home) => {
                set({ home });
              }}
            />
          </>
        )}
        {piece.kind === "base" ? (
          <>
            <NeedsField
              label={t("Strumenti del kit")}
              value={piece.needs}
              onChange={(needs) => {
                set({ needs });
              }}
            />
            <LinesField
              label={t("Crea il progetto (nella cartella projects)")}
              value={piece.create}
              onChange={(create) => {
                set({ create });
              }}
            />
            <LinesField
              label={t("Poi, nella cartella del progetto")}
              value={piece.steps}
              onChange={(steps) => {
                set({ steps });
              }}
            />
            <ActionsField
              value={piece.actions}
              onChange={(actions) => {
                set({ actions });
              }}
            />
            <EnvField
              value={piece.env}
              onChange={(env) => {
                set({ env });
              }}
            />
            <DatabaseField
              value={piece.database}
              onChange={(database) => {
                const next = { ...piece };
                if (database) next.database = database;
                else delete next.database;
                onChange(next);
              }}
            />
          </>
        ) : (
          <>
            {bases(all)
              .filter((b) => piece.on?.[b.id] !== undefined)
              .map((b) => {
                const variant = piece.on?.[b.id] ?? {};
                return (
                  <VariantFields
                    key={b.id}
                    title={t("Con {0}", b.name)}
                    note={textOf(view, variant.note)}
                    variant={variant}
                    onChange={(v) => {
                      set({ on: { ...piece.on, [b.id]: v } });
                    }}
                    onRemove={() => {
                      const on = { ...piece.on };
                      // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
                      delete on[b.id];
                      set({ on });
                    }}
                  />
                );
              })}
            {others.length > 0 && (
              <label className="flex items-center gap-2">
                <span className="text-vsc-desc">{t("Usala anche con")}</span>
                <Select
                  value=""
                  label={t("Usala anche con")}
                  onChange={(id) => {
                    if (id) set({ on: { ...piece.on, [id]: {} } });
                  }}
                  options={[
                    { value: "", label: t("scegli una base…") },
                    ...others.map((b) => ({ value: b.id, label: b.name })),
                  ]}
                />
              </label>
            )}
          </>
        )}
        <div className="flex flex-wrap gap-x-4 gap-y-1">
          {piece.home && (
            <Link
              onClick={() => {
                void urlOpen(piece.home);
              }}
            >
              {t("Pagina ufficiale")}
            </Link>
          )}
          {official && modified && (
            <Link
              onClick={() => {
                onChange(official);
              }}
            >
              {t("Ripristina le istruzioni ufficiali")}
            </Link>
          )}
          {!official && <Link onClick={onRemove}>{t("Togli")}</Link>}
        </div>
      </div>
    </SettingItem>
  );
}

/** Problems of the instructions file, above a framework's settings. */
function FrameworksProblems({ fw }: { fw: Frameworks }) {
  const problem = fw.view?.problem;
  if (!fw.error && !problem) return null;
  return (
    <div className="max-w-[880px] space-y-1 px-[14px] pb-2">
      <ErrorLine error={fw.error} />
      {problem && (
        <ErrorLine
          error={t("Le istruzioni in {0} non sono leggibili: uso quelle ufficiali e non modifico il file.", problem)}
        />
      )}
    </div>
  );
}

/** One base or addition as a group of settings (a tab, or a search result). */
export function PieceGroup({ fw, id, onRemoved }: { fw: Frameworks; id: string; onRemoved: () => void }) {
  const { view, pieces, change } = fw;
  const i = pieces.findIndex((p) => p.id === id);
  const piece = pieces[i];
  if (!piece) return null;
  const description =
    piece.kind === "base" ? (
      <>
        {t(
          "Le istruzioni del Nuovo progetto. Una riga = un comando, eseguito con PowerShell e gli strumenti del kit: la creazione nella cartella projects, il resto nella cartella del progetto.",
        )}{" "}
        {t("{name} = nome del progetto (tra virgolette), {package} = com.example. e il nome in minuscolo.")}
      </>
    ) : (
      t(
        "Si combinano con una base: per ciascuna base i loro comandi, dopo quelli della base. Se un'aggiunta crea il progetto al posto della base (uno starter kit), la creazione è la sua.",
      )
    );
  return (
    <SettingsGroup id={`s-${pieceTab(id)}`} title={piece.name || piece.id} description={description}>
      <FrameworksProblems fw={fw} />
      <PieceItem
        piece={piece}
        official={view?.official.find((p) => p.id === id)}
        all={pieces}
        view={view}
        onChange={(next) => {
          change(pieces.map((x, j) => (j === i ? next : x)));
        }}
        onRemove={() => {
          change(pieces.filter((_, j) => j !== i));
          onRemoved();
        }}
      />
    </SettingsGroup>
  );
}
