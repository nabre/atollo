// The Componenti section of a project's page: the kit's components it
// needs, and how it was recognised.
import type { ProjectInfo } from "../api";
import { useAction } from "../hooks";
import { ActionButton, ErrorLine } from "../common";
import { t } from "../i18n";
import { List, ListRow } from "../workbench";
import { VscDebugStart } from "react-icons/vsc";
import { projectInstallMissing } from "../api";
import { ComponentLogo } from "../logos";
import { EditorSection, Notice } from "../workbench";

/** Names of the kit's components (tauri/catalog/components.json). */
export const COMPONENT_NAMES: Record<string, string> = {
  claude: "Claude Code",
  codex: "Codex",
  antigravity: "Antigravity",
  git: "Git",
  python: "Python",
  node: "Node.js",
  uv: "uv",
  gh: "GitHub CLI",
  php: "PHP",
  apache: "Apache",
  nginx: "Nginx",
  rust: "Rust",
  go: "Go",
  java: "Java",
  kotlin: "Kotlin",
  android: "Android SDK",
  dotnet: ".NET",
  dart: "Dart",
  zig: "Zig",
  julia: "Julia",
  bun: "Bun",
  deno: "Deno",
  typst: "Typst",
  tinytex: "TinyTeX",
  pnpm: "pnpm, yarn",
  cmake: "CMake, Ninja",
  cc: "C, C++",
  ruby: "Ruby",
  r: "R",
  mariadb: "MariaDB",
  mongodb: "MongoDB",
  sqlite: "SQLite",
  pwsh: "PowerShell 7",
  terminal: "Windows Terminal",
  "vscodium-server": "Editor",
};

/** The components the project needs, with "Installa ciò che manca". */
export function Needs({ p, onChanged }: { p: ProjectInfo; onChanged: () => void }) {
  const { busy, error, run } = useAction(onChanged);
  return (
    <>
      <EditorSection
        id="need"
        title={t("Componenti")}
        actions={
          p.missing.length > 0 && (
            <ActionButton
              icon={VscDebugStart}
              label={t("Installa ciò che manca")}
              primary
              busy={busy === "install"}
              onClick={() => {
                run("install", () => projectInstallMissing(p.id));
              }}
            />
          )
        }
      >
        {p.requires.length === 0 ? (
          <p className="text-vsc-desc">{t("Nessun componente del kit.")}</p>
        ) : (
          <List label={t("Componenti")} className="-mx-2">
            {p.requires.map((id) => {
              const missing = p.missing.includes(id);
              return (
                <ListRow
                  key={id}
                  icon={<ComponentLogo id={id} />}
                  label={COMPONENT_NAMES[id] ?? id}
                  decorations={
                    <span className={missing ? "text-vsc-warning" : "text-vsc-success"}>
                      {missing ? t("da installare") : t("installato")}
                    </span>
                  }
                />
              );
            })}
          </List>
        )}
        <ErrorLine error={error} />
      </EditorSection>
      <EditorSection id="need-why" title={t("Come è stato riconosciuto")}>
        <p className="select-text">{p.reason}</p>
        {p.warnings.map((w) => (
          <Notice key={w} tone="warning">
            {w}
          </Notice>
        ))}
      </EditorSection>
    </>
  );
}
