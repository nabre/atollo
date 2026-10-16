// Content renderer for every topic in the in-app Guide (ui/src/guide/).
// Uses VSCodium Workbench primitives (EditorSection, DataTable, Properties,
// Notice, Tag, Button, SearchBox) and custom Guide widgets (GuideShot,
// GuideCode, GuideCard, GuideRecipe, KitFoldersBar).
import { useMemo, useState, type ReactNode } from "react";
import {
  VscArrowRight,
  VscCheck,
  VscCloudDownload,
  VscCopy,
  VscDatabase,
  VscDebugDisconnect,
  VscFolderLibrary,
  VscGithub,
  VscGlobe,
  VscHubot,
  VscKey,
  VscLayers,
  VscLinkExternal,
  VscLock,
  VscPulse,
  VscServerProcess,
  VscSettingsGear,
  VscShield,
  VscSync,
  VscTools,
  VscVersions,
  VscWand,
} from "react-icons/vsc";
import { urlOpen, type AppInfo, type ComponentInfo } from "../api";
import { useCopy } from "../hooks";
import { t } from "../i18n";
import { ComponentLogo } from "../logos";
import { Tag } from "../tags";
import { Button, DataTable, EditorSection, Notice, Properties, SearchBox } from "../workbench";
import {
  CLI_COMMANDS,
  DISK_TREE,
  TOOL_SOURCES,
  tr,
  type AppPageTarget,
  type GuideTopicId,
  type SourceCategory,
} from "./model";
import { GuideCard, GuideCode, GuideRecipe, GuideShot, KitFoldersBar } from "./widgets";

export interface GuideSectionProps {
  topicId: GuideTopicId;
  info: AppInfo | null;
  components: ComponentInfo[] | null;
  onNavigate: (page: AppPageTarget) => void;
  onSelectTopic: (id: GuideTopicId) => void;
}

function SimpleTable({ columns, rows }: { columns: string[]; rows: { key: string; cells: ReactNode[] }[] }) {
  return (
    <div className="overflow-x-auto rounded border border-vsc-border">
      <DataTable columns={columns} className="w-full border-collapse text-[12px]">
        {rows.map((r) => (
          <tr key={r.key} className="border-t border-vsc-border hover:bg-vsc-list-hover/40">
            {r.cells.map((c, idx) => (
              <td key={idx} className="px-2.5 py-2 align-top">
                {c}
              </td>
            ))}
          </tr>
        ))}
      </DataTable>
    </div>
  );
}

export function GuideTopicContent({ topicId, info, components, onNavigate, onSelectTopic }: GuideSectionProps) {
  switch (topicId) {
    case "overview":
      return <TopicOverview info={info} onNavigate={onNavigate} onSelectTopic={onSelectTopic} />;
    case "disk-layout":
      return <TopicDiskLayout info={info} />;
    case "lifecycle":
      return <TopicLifecycle onNavigate={onNavigate} />;
    case "supply-chain":
      return <TopicSupplyChain onSelectTopic={onSelectTopic} />;
    case "sources-table":
      return <TopicSourcesTable components={components} onNavigate={onNavigate} />;
    case "extra-sources":
      return <TopicExtraSources />;
    case "github-role":
      return <TopicGithubRole onSelectTopic={onSelectTopic} />;
    case "github-setup":
      return <TopicGithubSetup onNavigate={onNavigate} />;
    case "feat-workbench":
      return <TopicFeatWorkbench onNavigate={onNavigate} />;
    case "feat-projects":
      return <TopicFeatProjects onNavigate={onNavigate} />;
    case "feat-agents":
      return <TopicFeatAgents onNavigate={onNavigate} />;
    case "feat-services":
      return <TopicFeatServices onNavigate={onNavigate} />;
    case "feat-tools-cli":
      return <TopicFeatToolsCli onNavigate={onNavigate} />;
    case "cfg-init-net":
      return <TopicCfgInitNet onNavigate={onNavigate} />;
    case "cfg-langs-agents":
      return <TopicCfgLangsAgents onNavigate={onNavigate} />;
    case "cfg-db-web":
      return <TopicCfgDbWeb onNavigate={onNavigate} />;
    case "cfg-actions-orphans":
      return <TopicCfgActionsOrphans onNavigate={onNavigate} />;
  }
}

// ---------------------------------------------------------------------------
// 1.1 Overview & Architectural Principles
// ---------------------------------------------------------------------------

function TopicOverview({
  info,
  onNavigate,
  onSelectTopic,
}: {
  info: AppInfo | null;
  onNavigate: (page: AppPageTarget) => void;
  onSelectTopic: (id: GuideTopicId) => void;
}) {
  return (
    <>
      <EditorSection
        id="guide-overview-mission"
        title={tr(
          "Cos'è Dev-Kit (NEXKORA) e quale problema risolve",
          "What Dev-Kit (NEXKORA) is and what problem it solves",
        )}
      >
        <p className="m-0 text-[12px] text-vsc-desc">
          {tr(
            "Ambiente di sviluppo integrato, 100% portabile e auto-contenuto per Windows 10/11 (x64 e ARM64), progettato per risiedere su un'unità esterna (SSD/USB) o in una singola cartella locale senza lasciare tracce sul PC ospite.",
            "Integrated, 100% portable and self-contained development environment for Windows 10/11 (x64 & ARM64), designed to live on an external drive (SSD/USB) or in a single local folder without leaving traces on the host PC.",
          )}
        </p>
        <Notice tone="info">
          <div className="space-y-1">
            <div className="font-semibold text-vsc-settings-header">
              {tr("Missione architetturale", "Architectural mission")}
            </div>
            <div>
              {tr(
                "Collegando l'unità a qualsiasi PC Windows, l'utente ritrova istantaneamente i propri progetti, tutti i linguaggi (anche in più versioni simultanee), i database attivi con i relativi dati, il server web, l'editor, il terminale, l'identità Git/GitHub e tre agenti IA (Claude Code, Codex, Antigravity) con memoria, skill e server MCP sincronizzati.",
                "By plugging the drive into any Windows PC, you immediately get back your projects, all languages (even multiple versions side by side), active databases with their data, web server, editor, terminal, Git/GitHub identity and three AI agents (Claude Code, Codex, Antigravity) with synchronized memory, skills and MCP servers.",
              )}
            </div>
          </div>
        </Notice>

        {info && (
          <Properties
            rows={[
              [tr("Versione kit attiva", "Active kit version"), <Tag key="v" tone="focus">{`v${info.version}`}</Tag>],
              [
                tr("Percorso radice corrente", "Current root path"),
                <code key="r" className="font-mono text-[12px] select-text">
                  {info.root}
                </code>,
              ],
              [
                tr("Cartella progetti", "Projects folder"),
                <code key="p" className="font-mono text-[12px] select-text">
                  {info.projects}
                </code>,
              ],
              [
                tr("File system rilevato", "Detected file system"),
                <Tag key="fs" tone="focus">
                  {info.diskFs ?? "NTFS"}
                </Tag>,
              ],
            ]}
          />
        )}
      </EditorSection>

      <EditorSection
        id="guide-overview-principles"
        title={tr("I 6 principi architetturali cardine", "The 6 core architectural principles")}
      >
        <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
          <GuideCard
            icon={VscLock}
            badge="Zero-Trace"
            title={tr("1. Zero tracce sul PC ospite", "1. Zero traces on the host PC")}
          >
            {tr(
              "Nessuna scrittura nel registro di Windows, nessuna dipendenza da %USERPROFILE%, %APPDATA% o %LOCALAPPDATA% del computer ospite. Circa 90 variabili d'ambiente vengono sovrascritte in memoria e confinate in <Radice>\\data\\.",
              "No writes to the Windows Registry, no dependency on host %USERPROFILE%, %APPDATA% or %LOCALAPPDATA%. Around 90 environment variables are overridden in memory and confined under <Root>\\data\\.",
            )}
          </GuideCard>

          <GuideCard
            icon={VscFolderLibrary}
            badge="Drive-Letter Agnostic"
            title={tr("2. Indipendenza dalla lettera di unità", "2. Drive-letter independence")}
          >
            {tr(
              "Tutti i percorsi nei file di configurazione sono ricalcolati dinamicamente ad ogni avvio a partire dalla posizione corrente di Dev-Kit.exe. Se la chiavetta passa da D:\\ a E:\\ su un altro PC, tutto continua a funzionare.",
              "All paths in configuration files are dynamically recomputed on every startup from Dev-Kit.exe's location. If the drive changes from D:\\ to E:\\ on another PC, everything keeps working seamlessly.",
            )}
          </GuideCard>

          <GuideCard
            icon={VscShield}
            badge="Supply-Chain"
            title={tr(
              "3. Fonti ufficiali e verifica crittografica",
              "3. Official sources & cryptographic verification",
            )}
            actionLabel={tr("Esplora le 36 fonti", "Explore all 36 sources")}
            onAction={() => {
              onSelectTopic("sources-table");
            }}
          >
            {tr(
              "Ogni strumento viene scaricato via HTTPS direttamente dal produttore ufficiale e verificato con SHA-256, SHA-512, SHA3-256, integrità npm sha512 o firma Authenticode prima di entrare in data\\apps\\.",
              "Every tool is downloaded over HTTPS directly from its official vendor and verified with SHA-256, SHA-512, SHA3-256, npm sha512 integrity or Authenticode signature before entering data\\apps\\.",
            )}
          </GuideCard>

          <GuideCard
            icon={VscVersions}
            badge="Multi-Version"
            title={tr("4. Multi-versione per singolo progetto", "4. Per-project multi-versioning")}
            actionLabel={tr("Gestisci Strumenti", "Manage Tools")}
            onAction={() => {
              onNavigate("components");
            }}
          >
            {tr(
              "Linguaggi e database supportano più linee major.minor installate contemporaneamente (es. PHP 7.4 e 8.4, Node 20 e 24, Python 3.11 e 3.13). Ogni progetto sceglie le proprie versioni nel file .devkit.json.",
              "Languages and databases support multiple major.minor lines installed side by side (e.g. PHP 7.4 & 8.4, Node 20 & 24, Python 3.11 & 3.13). Each project selects its own versions in .devkit.json.",
            )}
          </GuideCard>

          <GuideCard
            icon={VscHubot}
            badge="AI Sync"
            title={tr("5. Agenti IA sincronizzati", "5. Synchronized AI agents")}
            actionLabel={tr("Apri Agenti", "Open Agents")}
            onAction={() => {
              onNavigate("agents");
            }}
          >
            {tr(
              "Claude Code, Codex e Antigravity condividono un'unica fonte di verità in data\\agents\\ (AGENTS.md, skills\\, mcp.json), allineata automaticamente prima di ogni avvio.",
              "Claude Code, Codex and Antigravity share a single source of truth in data\\agents\\ (AGENTS.md, skills\\, mcp.json), automatically synchronized before every launch.",
            )}
          </GuideCard>

          <GuideCard
            icon={VscGithub}
            badge="GitHub SSO"
            title={tr("6. Identità GitHub unificata", "6. Unified GitHub identity")}
            actionLabel={tr("Guida GitHub", "GitHub Guide")}
            onAction={() => {
              onSelectTopic("github-role");
            }}
          >
            {tr(
              "Un singolo login OAuth Device Flow con GitHub CLI configura automaticamente firma dei commit Git, creazione progetti da template, backup cloud e quota API GitHub (5.000 req/ora).",
              "A single OAuth Device Flow login with GitHub CLI automatically configures Git commit authorship, project creation from templates, cloud backups and GitHub API rate limits (5,000 req/hour).",
            )}
          </GuideCard>
        </div>
      </EditorSection>

      <EditorSection
        id="guide-overview-quicklinks"
        title={tr("Percorsi rapidi nella guida", "Quick paths in the guide")}
      >
        <div className="flex flex-wrap gap-2">
          <Button
            variant="primary"
            onClick={() => {
              onSelectTopic("sources-table");
            }}
          >
            <span className="inline-flex items-center gap-1.5">
              <VscGlobe size={14} aria-hidden="true" />
              {tr("Tabella completa delle 36 fonti", "Complete table of all 36 sources")}
            </span>
          </Button>
          <Button
            variant="secondary"
            onClick={() => {
              onSelectTopic("github-setup");
            }}
          >
            <span className="inline-flex items-center gap-1.5">
              <VscGithub size={14} aria-hidden="true" />
              {tr("Configurare l'account GitHub", "Configure GitHub account")}
            </span>
          </Button>
          <Button
            variant="secondary"
            onClick={() => {
              onSelectTopic("feat-projects");
            }}
          >
            <span className="inline-flex items-center gap-1.5">
              <VscFolderLibrary size={14} aria-hidden="true" />
              {tr("Scheda progetto e sito web locale", "Project detail & local website")}
            </span>
          </Button>
          <Button
            variant="secondary"
            onClick={() => {
              onSelectTopic("cfg-init-net");
            }}
          >
            <span className="inline-flex items-center gap-1.5">
              <VscWand size={14} aria-hidden="true" />
              {tr("12 procedure di configurazione passo-passo", "12 step-by-step setup procedures")}
            </span>
          </Button>
        </div>
      </EditorSection>
    </>
  );
}

// ---------------------------------------------------------------------------
// 1.2 Disk Layout & Portability
// ---------------------------------------------------------------------------

function TopicDiskLayout({ info }: { info: AppInfo | null }) {
  return (
    <>
      <EditorSection
        id="guide-disk-tree"
        title={tr("Albero delle cartelle del kit su disco", "Kit directory tree on disk")}
      >
        <p className="m-0 text-[12px] text-vsc-desc">
          {tr(
            "Tutto ciò che appartiene al kit vive dentro la cartella in cui si trova Dev-Kit.exe (indicata come <Radice>, ad esempio D:\\Dev-Kit\\). Clicca sui pulsanti qui sotto per aprire direttamente le cartelle in Esplora file.",
            "Everything belonging to the kit lives inside the folder where Dev-Kit.exe resides (denoted as <Root>, e.g. D:\\Dev-Kit\\). Click the buttons below to open folders directly in File Explorer.",
          )}
        </p>
        <KitFoldersBar />
        <SimpleTable
          columns={[tr("Percorso su disco", "Disk path"), tr("Ruolo e contenuto", "Role & contents")]}
          rows={DISK_TREE.map((node) => ({
            key: node.path,
            cells: [
              <code
                key="p"
                className="font-mono text-[12px] text-vsc-settings-header select-text"
                style={{ paddingLeft: `${String(node.depth * 14)}px` }}
              >
                {node.depth > 0 ? "├─ " : ""}
                {node.path}
              </code>,
              <span key="d" className="text-[12px] text-vsc-fg/90">
                {node.desc()}
              </span>,
            ],
          }))}
        />
      </EditorSection>

      <EditorSection
        id="guide-disk-env"
        title={tr(
          "Confinamento delle variabili d'ambiente (~90 variabili)",
          "Environment variable confinement (~90 variables)",
        )}
      >
        <SimpleTable
          columns={[
            tr("Categoria", "Category"),
            tr("Variabili principali", "Key variables"),
            tr("Destinazione nel kit", "Destination inside kit"),
          ]}
          rows={[
            {
              key: "home",
              cells: [
                <strong key="c">{tr("Profilo utente e XDG", "User profile & XDG")}</strong>,
                <code key="v" className="font-mono text-[11px]">
                  HOME, USERPROFILE, APPDATA, LOCALAPPDATA, XDG_*
                </code>,
                <code key="d" className="font-mono text-[11px]">
                  {"<Radice>\\data\\home\\, data\\appdata\\, data\\localappdata\\"}
                </code>,
              ],
            },
            {
              key: "tmp",
              cells: [
                <strong key="c">{tr("File temporanei", "Temporary files")}</strong>,
                <code key="v" className="font-mono text-[11px]">
                  TEMP, TMP, TMPDIR
                </code>,
                <code key="d" className="font-mono text-[11px]">
                  {"<Radice>\\data\\tmp\\"}
                </code>,
              ],
            },
            {
              key: "git",
              cells: [
                <strong key="c">Git, SSH & GitHub CLI</strong>,
                <code key="v" className="font-mono text-[11px]">
                  GIT_CONFIG_GLOBAL, GH_CONFIG_DIR, GIT_SSH_COMMAND
                </code>,
                <code key="d" className="font-mono text-[11px]">
                  {"<Radice>\\data\\home\\.gitconfig, data\\gh\\, data\\ssh\\"}
                </code>,
              ],
            },
            {
              key: "agents",
              cells: [
                <strong key="c">{tr("Agenti IA", "AI Agents")}</strong>,
                <code key="v" className="font-mono text-[11px]">
                  CLAUDE_CONFIG_DIR, CODEX_HOME, VSCODE_PORTABLE
                </code>,
                <code key="d" className="font-mono text-[11px]">
                  {"<Radice>\\data\\claude\\, data\\codex\\, data\\antigravity\\"}
                </code>,
              ],
            },
            {
              key: "langs",
              cells: [
                <strong key="c">{tr("Cache linguaggi e pacchetti", "Language & package caches")}</strong>,
                <code key="v" className="font-mono text-[11px]">
                  NPM_CONFIG_CACHE, PIP_CACHE_DIR, UV_CACHE_DIR, CARGO_HOME, GOPATH, GRADLE_USER_HOME, COMPOSER_HOME,
                  NUGET_PACKAGES
                </code>,
                <code key="d" className="font-mono text-[11px]">
                  {"<Radice>\\data\\npm-cache\\, pip-cache\\, uv\\, cargo\\, go\\, gradle\\, composer\\, nuget\\"}
                </code>,
              ],
            },
            {
              key: "privacy",
              cells: [
                <strong key="c">{tr("Telemetria disattivata", "Disabled telemetry")}</strong>,
                <code key="v" className="font-mono text-[11px]">
                  DOTNET_CLI_TELEMETRY_OPTOUT=1, NEXT_TELEMETRY_DISABLED=1, ASTRO_TELEMETRY_DISABLED=1
                </code>,
                <span key="d" className="text-[12px]">
                  {tr("Nessuna telemetria inviata a terze parti", "No telemetry sent to third parties")}
                </span>,
              ],
            },
          ]}
        />
        {info && (
          <GuideCode
            title={tr("Ispeziona l'ambiente completo da terminale", "Inspect full environment from terminal")}
            path={info.root}
            code={`.\\Dev-Kit.exe env | Out-Host`}
          />
        )}
      </EditorSection>
    </>
  );
}

// ---------------------------------------------------------------------------
// 1.3 Startup & Eject Lifecycle
// ---------------------------------------------------------------------------

function TopicLifecycle({ onNavigate }: { onNavigate: (page: AppPageTarget) => void }) {
  return (
    <>
      <EditorSection
        id="guide-lifecycle-start"
        title={tr("Sequenza di avvio automatico del kit", "Automatic kit startup sequence")}
      >
        <SimpleTable
          columns={[
            "#",
            tr("Fase di avvio", "Startup phase"),
            tr("Operazioni eseguite in automatico", "Operations performed automatically"),
          ]}
          rows={[
            {
              key: "1",
              cells: [
                <Tag key="n" tone="focus">
                  1
                </Tag>,
                <strong key="t">{tr("Rilevamento modalità", "Mode detection")}</strong>,
                <span key="d">
                  {tr(
                    "Senza argomenti apre la finestra grafica nativa (Tauri WebView2); con argomenti esegue il comando CLI richiesto e termina.",
                    "Without arguments opens the native GUI window (Tauri WebView2); with arguments runs the requested CLI subcommand and exits.",
                  )}
                </span>,
              ],
            },
            {
              key: "2",
              cells: [
                <Tag key="n" tone="focus">
                  2
                </Tag>,
                <strong key="t">{tr("Inizializzazione stato", "State initialization")}</strong>,
                <span key="d">
                  {tr(
                    "Individua la cartella di Dev-Kit.exe, crea le sottocartelle mancanti in data\\ e scrive i file shim (agy.cmd, ssh-wrapper.cmd, editor.cmd, gitconfig).",
                    "Detects Dev-Kit.exe's folder, creates any missing subfolders under data\\ and writes shim files (agy.cmd, ssh-wrapper.cmd, editor.cmd, gitconfig).",
                  )}
                </span>,
              ],
            },
            {
              key: "3",
              cells: [
                <Tag key="n" tone="focus">
                  3
                </Tag>,
                <strong key="t">{tr("Allineamento identità Git", "Git identity alignment")}</strong>,
                <span key="d">
                  {tr(
                    "Se GitHub CLI è autenticato, verifica ed eventualmente riallinea user.name e user.email nel .gitconfig del kit.",
                    "If GitHub CLI is authenticated, checks and realigns user.name and user.email in the kit's .gitconfig.",
                  )}
                </span>,
              ],
            },
            {
              key: "4",
              cells: [
                <Tag key="n" tone="focus">
                  4
                </Tag>,
                <strong key="t">{tr("Avvio pigro database", "Lazy database startup")}</strong>,
                <span key="d">
                  {tr(
                    "I motori MariaDB e MongoDB non rallentano l'apertura della finestra: partono automaticamente non appena apri la pagina Database o avvii un progetto che li usa.",
                    "MariaDB and MongoDB engines do not slow down window startup: they start automatically as soon as you open the Databases page or launch a project that uses them.",
                  )}
                </span>,
              ],
            },
            {
              key: "5",
              cells: [
                <Tag key="n" tone="focus">
                  5
                </Tag>,
                <strong key="t">{tr("Ripristino siti web", "Website restoration")}</strong>,
                <span key="d">
                  {tr(
                    "In background riavvia tutti i siti web dei progetti che avevano l'opzione Avvio automatico attiva prima dell'ultima chiusura.",
                    "In the background restarts all project websites that had Auto-start enabled before the last shutdown.",
                  )}
                </span>,
              ],
            },
          ]}
        />
      </EditorSection>

      <EditorSection
        id="guide-lifecycle-eject"
        title={tr("Sequenza di chiusura sicura e Rimuovi unità", "Safe shutdown & Eject drive sequence")}
      >
        <GuideShot
          src="15-rimuovi-unita.webp"
          title={tr("15 · Pagina Rimuovi unità e pulizia tracce", "15 · Eject drive & trace cleanup page")}
          caption={tr(
            "Riepilogo dei controlli effettuati prima della rimozione: arresto siti web e database, chiusura processi in data\\apps\\, salvataggio credenziali Antigravity cifrate DPAPI e rimozione delle chiavi orfane sul PC ospite.",
            "Summary of pre-eject checks: stopping websites and databases, closing processes under data\\apps\\, saving DPAPI-encrypted Antigravity credentials and removing orphan keys on the host PC.",
          )}
          example={tr(
            "Con un solo clic su «Arresta e rimuovi in sicurezza», il kit ferma il sito portale-clienti:8080, chiude MariaDB 11.4 e MongoDB 8.0 scaricando i buffer su disco, cifra il keyring e lascia l'unità D:\\ pronta da scollegare.",
            "With a single click on 'Stop and safely eject', the kit stops portale-clienti:8080, shuts down MariaDB 11.4 and MongoDB 8.0 flushing buffers to disk, encrypts the keyring and leaves drive D:\\ ready to unplug.",
          )}
        />
        <div>
          <Button
            variant="primary"
            onClick={() => {
              onNavigate("eject");
            }}
          >
            <span className="inline-flex items-center gap-1.5">
              <VscDebugDisconnect size={14} aria-hidden="true" />
              {tr("Vai a Rimuovi unità", "Go to Eject drive")}
            </span>
          </Button>
        </div>
      </EditorSection>
    </>
  );
}

// ---------------------------------------------------------------------------
// 2.1 Supply-Chain Security & Verification Pipeline
// ---------------------------------------------------------------------------

function TopicSupplyChain({ onSelectTopic }: { onSelectTopic: (id: GuideTopicId) => void }) {
  return (
    <>
      <EditorSection
        id="guide-supply-chain"
        title={tr("Pipeline di sicurezza a 6 stadi per ogni download", "6-stage security pipeline for every download")}
      >
        <p className="m-0 text-[12px] text-vsc-desc">
          {tr(
            "Dev-Kit non include binari di terze parti pre-impacchettati e non si appoggia a mirror non ufficiali: ogni componente viene scaricato direttamente dal fornitore originario e verificato crittograficamente.",
            "Dev-Kit does not bundle pre-packaged third-party binaries and never relies on unofficial mirrors: every component is downloaded directly from its original vendor and cryptographically verified.",
          )}
        </p>
        <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
          <GuideCard
            icon={VscGlobe}
            badge="Stage 1 · HTTPS"
            title={tr("1. Interrogazione della fonte ufficiale", "1. Querying the official vendor source")}
          >
            {tr(
              "Il modulo src/components.rs interroga in HTTPS (TLS 1.2+) l'API o il manifesto ufficiale del fornitore (GitHub Releases API, npm Registry, nodejs.org, python-build-standalone, windows.php.net, ecc.).",
              "The src/components.rs module queries over HTTPS (TLS 1.2+) the vendor's official API or manifest (GitHub Releases API, npm Registry, nodejs.org, python-build-standalone, windows.php.net, etc.).",
            )}
          </GuideCard>

          <GuideCard
            icon={VscLock}
            badge="Stage 2 · Digest"
            title={tr("2. Estrazione dell'impronta crittografica", "2. Cryptographic digest extraction")}
          >
            {tr(
              "Prima di scaricare il file binario, il kit recupera l'impronta pubblicata dal fornitore: SHA-256 (64 hex), SHA-512 (128 hex), SHA3-256 (SQLite), integrità npm sha512-<base64> o firma Authenticode.",
              "Before downloading the binary archive, the kit fetches the vendor-published digest: SHA-256 (64 hex), SHA-512 (128 hex), SHA3-256 (SQLite), npm sha512-<base64> integrity or Authenticode signature.",
            )}
          </GuideCard>

          <GuideCard
            icon={VscCloudDownload}
            badge="Stage 3 · Resume"
            title={tr("3. Download con ripresa automatica (.part)", "3. Download with automatic resume (.part)")}
          >
            {tr(
              "L'archivio viene scaricato in data\\tmp\\_dl\\<nome>.part con supporto HTTP Range (206 Partial Content) fino a 4 tentativi automatici in caso di rete instabile.",
              "The archive is downloaded into data\\tmp\\_dl\\<name>.part with HTTP Range support (206 Partial Content) and up to 4 automatic retries on unstable networks.",
            )}
          </GuideCard>

          <GuideCard
            icon={VscShield}
            badge="Stage 4 · Hash Match"
            title={tr("4. Verifica crittografica in memoria", "4. In-memory cryptographic verification")}
          >
            {tr(
              "Al termine dello scaricamento, il kit ricalcola l'hash dell'intero file su disco e lo confronta byte per byte con quello atteso. Se non coincide, il file viene immediatamente eliminato e l'installazione bloccata.",
              "Once downloaded, the kit recomputes the hash of the entire file on disk and compares it byte-for-byte with the expected digest. On mismatch, the file is immediately deleted and installation aborted.",
            )}
          </GuideCard>

          <GuideCard
            icon={VscLayers}
            badge="Stage 5 · Atomic Swap"
            title={tr("5. Estrazione in staging e scambio atomico", "5. Staging extraction & atomic swap")}
          >
            {tr(
              "Gli archivi (.zip, .tar.gz, .tar.xz, .7z, .nupkg) vengono estratti in data\\apps\\_staging_<id>\\ (con 128 MB di stack dedicato per i dizionari LZMA2) e poi sostituiti atomicamente alla cartella finale.",
              "Archives (.zip, .tar.gz, .tar.xz, .7z, .nupkg) are extracted into data\\apps\\_staging_<id>\\ (with a dedicated 128 MB stack for LZMA2 dictionaries) and atomically swapped into the target folder.",
            )}
          </GuideCard>

          <GuideCard
            icon={VscCheck}
            badge="Stage 6 · Check & State"
            title={tr("6. Post-configurazione, test e registro", "6. Post-setup, smoke test & state record")}
          >
            {tr(
              "Vengono generati i file di supporto (es. php.ini con curl.cainfo, .portable, wrapper .cmd), viene eseguito il comando check e la versione installata viene salvata in data\\app\\state.json.",
              "Support files (e.g. php.ini with curl.cainfo, .portable, .cmd wrappers) are generated, the check command is executed, and the installed version is saved in data\\app\\state.json.",
            )}
          </GuideCard>
        </div>

        <div>
          <Button
            variant="primary"
            onClick={() => {
              onSelectTopic("sources-table");
            }}
          >
            <span className="inline-flex items-center gap-1.5">
              {tr("Consulta la tabella completa delle 36 fonti", "View the complete table of all 36 sources")}
              <VscArrowRight size={14} aria-hidden="true" />
            </span>
          </Button>
        </div>
      </EditorSection>
    </>
  );
}

// ---------------------------------------------------------------------------
// 2.2 Complete Table of all 36 Tool Sources (Interactive & Filterable)
// ---------------------------------------------------------------------------

function TopicSourcesTable({
  components,
  onNavigate,
}: {
  components: ComponentInfo[] | null;
  onNavigate: (page: AppPageTarget) => void;
}) {
  const [query, setQuery] = useState("");
  const [cat, setCat] = useState<"all" | SourceCategory>("all");

  const compMap = useMemo(() => {
    const m = new Map<string, ComponentInfo>();
    for (const c of components ?? []) m.set(c.id, c);
    return m;
  }, [components]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    return TOOL_SOURCES.filter((row) => {
      if (cat !== "all" && row.group !== cat) return false;
      if (!q) return true;
      return (
        row.id.toLowerCase().includes(q) ||
        row.name.toLowerCase().includes(q) ||
        row.home.toLowerCase().includes(q) ||
        row.metaSource.toLowerCase().includes(q) ||
        row.binarySource.toLowerCase().includes(q) ||
        row.verifyKind.toLowerCase().includes(q) ||
        row.targetPath.toLowerCase().includes(q)
      );
    });
  }, [query, cat]);

  const categories: { id: "all" | SourceCategory; label: string }[] = [
    { id: "all", label: `${tr("Tutti", "All")} (${String(TOOL_SOURCES.length)})` },
    { id: "agents", label: tr("Agenti IA", "AI Agents") },
    { id: "languages", label: tr("Linguaggi", "Languages") },
    { id: "databases", label: tr("Database", "Databases") },
    { id: "tools", label: tr("Strumenti", "Tools") },
    { id: "part", label: tr("Sotto-parti & SDK", "Sub-parts & SDKs") },
    { id: "app", label: tr("App & Editor", "App & Editor") },
    { id: "os", label: tr("Terminale", "Terminal") },
  ];

  return (
    <>
      <EditorSection
        id="guide-sources-table"
        title={tr(
          "Catalogo completo delle fonti esterne (36 componenti e sotto-parti)",
          "Complete catalog of external sources (36 components & sub-parts)",
        )}
        actions={
          <Button
            small
            variant="primary"
            onClick={() => {
              onNavigate("components");
            }}
          >
            <span className="inline-flex items-center gap-1">
              <VscTools size={12} aria-hidden="true" />
              {tr("Apri pagina Strumenti", "Open Tools page")}
            </span>
          </Button>
        }
      >
        <p className="m-0 text-[12px] text-vsc-desc">
          {tr(
            "Elenco minuzioso di tutti i link da cui Dev-Kit attinge gli strumenti, il metodo di verifica crittografica applicato, la cartella di destinazione in data\\apps\\ e lo stato attuale su questa unità.",
            "Detailed list of every URL from which Dev-Kit fetches tools, the cryptographic verification method applied, the destination folder in data\\apps\\ and the current status on this drive.",
          )}
        </p>

        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex flex-wrap items-center gap-1.5">
            {categories.map((c) => (
              <Button
                key={c.id}
                small
                variant={cat === c.id ? "primary" : "secondary"}
                onClick={() => {
                  setCat(c.id);
                }}
              >
                {c.label}
              </Button>
            ))}
          </div>
          <div className="w-64">
            <SearchBox
              value={query}
              onChange={setQuery}
              placeholder={tr("Filtra per nome, URL, hash, percorso...", "Filter by name, URL, hash, path...")}
            />
          </div>
        </div>

        <SimpleTable
          columns={[
            tr("Componente e Stato", "Component & Status"),
            tr("Endpoint metadati / versione", "Metadata / version endpoint"),
            tr("URL binario / archivio", "Binary / archive URL"),
            tr("Verifica integrità", "Integrity check"),
            tr("Percorso in data\\apps\\", "Path in data\\apps\\"),
          ]}
          rows={filtered.map((row) => {
            const live = compMap.get(row.id);
            const installedVer = live?.installed ? (live.version ?? tr("Installato", "Installed")) : null;
            return {
              key: row.id,
              cells: [
                <div key="name" className="space-y-1">
                  <div className="flex items-center gap-2">
                    <ComponentLogo id={row.logoId} size={16} />
                    <span className="font-semibold text-vsc-settings-header">{row.name}</span>
                  </div>
                  <div className="flex flex-wrap items-center gap-1.5 pl-6">
                    <code className="font-mono text-[10px] text-vsc-desc">{row.id}</code>
                    {installedVer ? (
                      <Tag tone="focus">{installedVer}</Tag>
                    ) : live ? (
                      <Tag>{tr("Non installato", "Not installed")}</Tag>
                    ) : (
                      <Tag>{tr("Automatico", "Automatic")}</Tag>
                    )}
                    <button
                      type="button"
                      onClick={() => {
                        void urlOpen(row.home);
                      }}
                      title={row.home}
                      className="inline-flex cursor-pointer items-center gap-0.5 border-0 bg-transparent p-0 text-[11px] text-vsc-link underline hover:opacity-80"
                    >
                      <span>{tr("Sito", "Site")}</span>
                      <VscLinkExternal size={10} aria-hidden="true" />
                    </button>
                  </div>
                </div>,
                <div key="meta" className="font-mono text-[11px] leading-snug break-all text-vsc-fg/90 select-text">
                  {row.metaSource}
                </div>,
                <div key="bin" className="font-mono text-[11px] leading-snug break-all text-vsc-desc select-text">
                  {row.binarySource}
                </div>,
                <div key="ver" className="space-y-0.5">
                  <Tag tone="focus">{row.verifyKind}</Tag>
                  <div className="text-[11px] leading-tight text-vsc-desc">{row.verifyDetail}</div>
                </div>,
                <code key="path" className="font-mono text-[11px] break-all text-vsc-fg select-text">
                  {row.targetPath}
                </code>,
              ],
            };
          })}
        />
      </EditorSection>
    </>
  );
}

// ---------------------------------------------------------------------------
// 2.3 Additional External Services & Endpoints
// ---------------------------------------------------------------------------

function TopicExtraSources() {
  return (
    <>
      <EditorSection
        id="guide-extra-sources"
        title={tr(
          "Ulteriori servizi esterni interrogati dall'applicazione",
          "Additional external services queried by the application",
        )}
      >
        <p className="m-0 text-[12px] text-vsc-desc">
          {tr(
            "Oltre ai repository di download dei binari, Dev-Kit comunica esclusivamente con i seguenti endpoint per quote IA, autenticazione GitHub e galleria estensioni dell'editor.",
            "Beyond binary download repositories, Dev-Kit communicates exclusively with the following endpoints for AI quotas, GitHub authentication and the editor extension gallery.",
          )}
        </p>
        <SimpleTable
          columns={[
            tr("Servizio", "Service"),
            tr("Endpoint HTTPS", "HTTPS Endpoint"),
            tr("Scopo e funzionamento", "Purpose & behavior"),
          ]}
          rows={[
            {
              key: "gh-oauth",
              cells: [
                <strong key="s">GitHub OAuth & REST API</strong>,
                <code key="e" className="font-mono text-[11px] select-text">
                  https://github.com/login/device/code · https://api.github.com/user
                </code>,
                <span key="d">
                  {tr(
                    "Usato da gh.exe per il login Device Flow, la lettura del profilo e delle email (/user/emails), la creazione di repository da template e l'innalzamento del rate-limit da 60 a 5.000 richieste/ora.",
                    "Used by gh.exe for Device Flow login, reading profile and emails (/user/emails), creating repositories from templates and raising rate limits from 60 to 5,000 requests/hour.",
                  )}
                </span>,
              ],
            },
            {
              key: "claude-quota",
              cells: [
                <strong key="s">{tr("Quota Claude Code (Anthropic)", "Claude Code Quota (Anthropic)")}</strong>,
                <code key="e" className="font-mono text-[11px] select-text">
                  https://api.anthropic.com/api/oauth/usage · /api/oauth/profile
                </code>,
                <span key="d">
                  {tr(
                    "Interrogato dalla pagina Agenti usando il token OAuth locale (data\\claude\\.credentials.json) per mostrare percentuali di utilizzo a 5 ore, 7 giorni e crediti extra.",
                    "Queried by the Agents page using the local OAuth token (data\\claude\\.credentials.json) to display 5-hour, 7-day and extra-usage percentages.",
                  )}
                </span>,
              ],
            },
            {
              key: "codex-quota",
              cells: [
                <strong key="s">{tr("Quota Codex (OpenAI)", "Codex Quota (OpenAI)")}</strong>,
                <code key="e" className="font-mono text-[11px] select-text">
                  https://chatgpt.com/backend-api/wham/usage
                </code>,
                <span key="d">
                  {tr(
                    "Interrogato dalla pagina Agenti usando il token in data\\codex\\auth.json per mostrare la finestra primaria, settimanale e il saldo crediti.",
                    "Queried by the Agents page using the token in data\\codex\\auth.json to display primary window, weekly window and credit balance.",
                  )}
                </span>,
              ],
            },
            {
              key: "agy-quota",
              cells: [
                <strong key="s">{tr("Quota Antigravity (Google)", "Antigravity Quota (Google)")}</strong>,
                <code key="e" className="font-mono text-[11px] select-text">
                  https://127.0.0.1:&lt;porta&gt;/exa.language_server_pb.LanguageServerService/GetUserStatus
                </code>,
                <span key="d">
                  {tr(
                    "Interrogato in locale quando Antigravity è aperto, estraendo --csrf_token dal processo language_server_windows_x64.exe per leggere le quote per singolo modello IA.",
                    "Queried locally while Antigravity is running, extracting --csrf_token from language_server_windows_x64.exe to read per-model AI quotas.",
                  )}
                </span>,
              ],
            },
            {
              key: "openvsx",
              cells: [
                <strong key="s">Open VSX Registry (VSCodium)</strong>,
                <code key="e" className="font-mono text-[11px] select-text">
                  https://open-vsx.org/vscode/gallery · https://open-vsx.org/vscode/item
                </code>,
                <span key="d">
                  {tr(
                    "Marketplace open-source preconfigurato in VSCodium Reh Web per installare estensioni dell'editor direttamente in data\\vscodium-server\\data\\extensions\\.",
                    "Open-source marketplace preconfigured in VSCodium Reh Web to install editor extensions directly under data\\vscodium-server\\data\\extensions\\.",
                  )}
                </span>,
              ],
            },
          ]}
        />
      </EditorSection>
    </>
  );
}

// ---------------------------------------------------------------------------
// 3.1 Why GitHub Account is Essential (5 Roles)
// ---------------------------------------------------------------------------

function TopicGithubRole({ onSelectTopic }: { onSelectTopic: (id: GuideTopicId) => void }) {
  return (
    <>
      <EditorSection
        id="guide-github-role"
        title={tr(
          "Perché l'account GitHub è il pilastro dell'ecosistema Dev-Kit",
          "Why a GitHub account is the cornerstone of the Dev-Kit ecosystem",
        )}
      >
        <p className="m-0 text-[12px] text-vsc-desc">
          {tr(
            "Anche se puoi esplorare l'interfaccia senza login, collegare un account GitHub (gratuito) sblocca 5 meccanismi tecnici fondamentali che altrimenti sarebbero limitati o indisponibili.",
            "While you can browse the UI without logging in, connecting a (free) GitHub account unlocks 5 core technical mechanisms that would otherwise be rate-limited or unavailable.",
          )}
        </p>
        <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
          <GuideCard
            icon={VscPulse}
            badge="5.000 req/ora"
            title={tr(
              "1. Sblocco limiti API GitHub per scaricare gli strumenti",
              "1. Unlocks GitHub API rate limits for tool downloads",
            )}
          >
            {tr(
              "Oltre metà del catalogo (Git, uv, Ripgrep, fd, Bat, jq, Pandoc, Typst, CMake, Ninja, Rust Analyzer, Bun, Ruby, Adminer, Windows Terminal, ecc.) risolve le versioni tramite api.github.com. Senza login GitHub applica un limite di 60 richieste/ora per IP (HTTP 403 Rate limit exceeded); con l'account collegato il kit inietta automaticamente Authorization: Bearer <token> portando la soglia a 5.000 richieste/ora.",
              "Over half the catalog resolves versions via api.github.com. Without login, GitHub enforces a 60 req/hour limit per IP (HTTP 403 Rate limit exceeded); with a linked account, the kit automatically injects Authorization: Bearer <token>, raising the limit to 5,000 requests/hour.",
            )}
          </GuideCard>

          <GuideCard
            icon={VscWand}
            badge="Template"
            title={tr(
              "2. Creazione di nuovi progetti da Template GitHub",
              "2. Creating new projects from GitHub Templates",
            )}
          >
            {tr(
              "La procedura Nuovo Progetto clona il repository modello scelto (es. starter-kit), crea un nuovo repository privato o pubblico sotto il tuo account GitHub tramite gh repo create --template e configura automaticamente il remote origin.",
              "The New Project wizard clones the chosen template repository (e.g. starter-kit), creates a new private or public repository under your GitHub account via gh repo create --template, and automatically configures the origin remote.",
            )}
          </GuideCard>

          <GuideCard
            icon={VscKey}
            badge=".gitconfig"
            title={tr(
              "3. Configurazione automatica dell'identità Git nei commit",
              "3. Automatic Git commit authorship configuration",
            )}
          >
            {tr(
              "Al login il kit interroga /user e /user/emails e popola automaticamente user.name, user.email (inclusa l'email @users.noreply.github.com se l'email primaria è privata) e credential.helper = !gh auth git-credential in data\\home\\.gitconfig.",
              "Upon login, the kit queries /user and /user/emails and automatically populates user.name, user.email (including @users.noreply.github.com if primary email is private) and credential.helper = !gh auth git-credential in data\\home\\.gitconfig.",
            )}
          </GuideCard>

          <GuideCard
            icon={VscSync}
            badge="Push / Pull"
            title={tr(
              "4. Sincronizzazione e backup cloud a 1 clic dei progetti",
              "4. 1-click cloud synchronization & backup for projects",
            )}
          >
            {tr(
              "Nella scheda Git di ogni progetto i pulsanti «Pubblica su GitHub», «Salva e invia modifiche» e «Scarica aggiornamenti» eseguono commit, pull --rebase e push HTTPS senza mai chiedere password o chiavi SSH manuali.",
              "In each project's Git tab, the 'Publish to GitHub', 'Commit & Push' and 'Pull updates' buttons run commit, pull --rebase and HTTPS push without ever asking for manual passwords or SSH keys.",
            )}
          </GuideCard>
        </div>

        <GuideCard
          icon={VscLock}
          badge="data\gh\hosts.yml"
          title={tr(
            "5. Autenticazione HTTPS trasparente per terminali, editor e agenti IA",
            "5. Transparent HTTPS authentication for terminals, editor & AI agents",
          )}
          actionLabel={tr("Vedi come collegare l'account", "See how to connect your account")}
          onAction={() => {
            onSelectTopic("github-setup");
          }}
        >
          {tr(
            "Poiché il token è salvato in <Radice>\\data\\gh\\hosts.yml (tramite GH_CONFIG_DIR), spostando l'unità USB su un nuovo PC non dovrai mai rifare il login: qualsiasi git push o gh pr create lanciato da te o dagli agenti IA sarà già autenticato.",
            "Because the token is stored in <Root>\\data\\gh\\hosts.yml (via GH_CONFIG_DIR), moving your USB drive to a new PC never requires logging in again: any git push or gh pr create run by you or by AI agents is already authenticated.",
          )}
        </GuideCard>
      </EditorSection>
    </>
  );
}

// ---------------------------------------------------------------------------
// 3.2 Connecting GitHub Account (Device Flow & Generated .gitconfig)
// ---------------------------------------------------------------------------

function TopicGithubSetup({ onNavigate }: { onNavigate: (page: AppPageTarget) => void }) {
  return (
    <>
      <EditorSection
        id="guide-github-setup"
        title={tr(
          "Come collegare l'account GitHub (OAuth Device Flow)",
          "How to connect your GitHub account (OAuth Device Flow)",
        )}
      >
        <GuideRecipe
          number="GitHub SSO"
          title={tr(
            "Autenticazione con codice dispositivo a 8 caratteri",
            "Authentication with 8-character device code",
          )}
          whereUi={tr(
            "Wizard iniziale oppure Impostazioni → Account GitHub",
            "Initial Wizard or Settings → GitHub Account",
          )}
          whereFile="data\gh\hosts.yml · data\home\.gitconfig"
          targetPage="settings"
          targetLabel={tr("Apri Impostazioni → Account GitHub", "Open Settings → GitHub Account")}
          onNavigate={onNavigate}
          steps={[
            tr(
              "Se non possiedi ancora un account, clicca su «Crea un account gratuito su github.com» e registrati con il tuo indirizzo email.",
              "If you don't have an account yet, click 'Create a free account on github.com' and sign up with your email.",
            ),
            tr(
              "Nell'applicazione clicca sul pulsante «Collega account GitHub».",
              "In the application, click the 'Connect GitHub account' button.",
            ),
            tr(
              "Dev-Kit avvia in background gh auth login --hostname github.com --git-protocol https --web --scopes repo,read:org,user:email,workflow.",
              "Dev-Kit starts gh auth login --hostname github.com --git-protocol https --web --scopes repo,read:org,user:email,workflow in the background.",
            ),
            tr(
              "Sullo schermo compare un codice univoco a 8 caratteri (es. WDJB-4829), già copiato automaticamente negli appunti, e si apre https://github.com/login/device nel browser.",
              "A unique 8-character code (e.g. WDJB-4829) appears on screen, automatically copied to your clipboard, and https://github.com/login/device opens in your browser.",
            ),
            tr(
              "Incolla il codice nella pagina di GitHub e premi Authorize GitHub CLI: la finestra di Dev-Kit rileverà il completamento in tempo reale mostrando il tuo nome utente (@utente).",
              "Paste the code into the GitHub page and click Authorize GitHub CLI: Dev-Kit will detect completion in real time and display your username (@user).",
            ),
          ]}
          purpose={tr(
            "Autorizza i permessi repo (creazione/push repository pubblici e privati), read:org (lettura template aziendali), user:email (configurazione email autore commit) e workflow (aggiornamento GitHub Actions).",
            "Authorizes the scopes repo (create/push public and private repositories), read:org (read organization templates), user:email (configure commit author email) and workflow (update GitHub Actions).",
          )}
        />

        <GuideCode
          title={tr(
            "File .gitconfig generato e mantenuto automaticamente dal kit",
            ".gitconfig file automatically generated and maintained by the kit",
          )}
          path="<Radice>\data\home\.gitconfig"
          code={`[init]
    defaultBranch = main
[core]
    autocrlf = true
    longpaths = true
    editor = D:/Dev-Kit/data/kit/bin/editor.cmd
[safe]
    directory = *
[user]
    name = Mario Rossi
    email = mario.rossi@example.com
[credential "https://github.com"]
    helper = 
    helper = !'D:/Dev-Kit/data/apps/tools/gh.exe' auth git-credential
[credential "https://gist.github.com"]
    helper = 
    helper = !'D:/Dev-Kit/data/apps/tools/gh.exe' auth git-credential`}
        />
      </EditorSection>
    </>
  );
}

// ---------------------------------------------------------------------------
// 4.1 Workbench Layout & First-Run Wizard
// ---------------------------------------------------------------------------

function TopicFeatWorkbench({ onNavigate }: { onNavigate: (page: AppPageTarget) => void }) {
  return (
    <>
      <EditorSection
        id="guide-feat-workbench"
        title={tr("Struttura dell'interfaccia (VSCodium Workbench)", "Interface structure (VSCodium Workbench)")}
      >
        <SimpleTable
          columns={[
            tr("Area dell'interfaccia", "Interface area"),
            tr("Elementi e scorciatoie", "Elements & shortcuts"),
          ]}
          rows={[
            {
              key: "activity",
              cells: [
                <strong key="a">{tr("Barra delle attività (a sinistra)", "Activity Bar (left)")}</strong>,
                <span key="d">
                  {tr(
                    "In alto le 6 viste operative: Nuovo progetto, Progetti (con badge conteggio), Agenti, Database, Server Web, Strumenti. In basso le 3 viste di sistema: Guida (questa vista), Impostazioni, Rimuovi l'unità.",
                    "Top 6 operational views: New project, Projects (with count badge), Agents, Databases, Web Servers, Tools. Bottom 3 system views: Guide (this view), Settings, Eject drive.",
                  )}
                </span>,
              ],
            },
            {
              key: "sidebar",
              cells: [
                <strong key="a">{tr("Barra laterale contestuale", "Contextual Side Bar")}</strong>,
                <span key="d">
                  {tr(
                    "Affianca la barra delle attività nelle viste con elenchi (Progetti, Database, Strumenti, Guida) offrendo casella di ricerca e sezioni espandibili.",
                    "Sits next to the Activity Bar in list-based views (Projects, Databases, Tools, Guide) providing a search box and collapsible sections.",
                  )}
                </span>,
              ],
            },
            {
              key: "editor",
              cells: [
                <strong key="a">{tr("Area principale di lavoro", "Main Editor Area")}</strong>,
                <span key="d">
                  {tr(
                    "Ospita l'intestazione della risorsa selezionata, la barra delle schede (Tab) o i filtri e le sezioni operative.",
                    "Hosts the header of the selected resource, the tab bar or filters, and the operational sections.",
                  )}
                </span>,
              ],
            },
            {
              key: "status",
              cells: [
                <strong key="a">{tr("Barra di stato (in basso)", "Status Bar (bottom)")}</strong>,
                <span key="d">
                  {tr(
                    "Mostra lo stato del kit, l'editor VSCodium, le installazioni in corso e il pulsante per aprire il pannello Attività.",
                    "Shows kit status, VSCodium editor status, running installations and the button to toggle the Activity panel.",
                  )}
                </span>,
              ],
            },
          ]}
        />
      </EditorSection>

      <EditorSection
        id="guide-feat-wizard"
        title={tr("Procedura guidata di primo avvio (Wizard)", "First-Run Setup Wizard")}
      >
        <GuideShot
          src="01-wizard-benvenuto.webp"
          title={tr("01 · Wizard di primo avvio (5 passaggi)", "01 · First-Run Setup Wizard (5 steps)")}
          caption={tr(
            "Passaggi del Wizard: 1. Benvenuto e verifica spazio su disco · 2. Scelta componenti e linee di versione · 3. Collegamento account GitHub · 4. Installazione e verifica crittografica · 5. Riepilogo finale.",
            "Wizard steps: 1. Welcome & disk space check · 2. Component & version line selection · 3. GitHub account connection · 4. Download & cryptographic verification · 5. Final summary.",
          )}
          example={tr(
            "L'unità esterna D:\\Dev-Kit viene rilevata con 384 GB liberi (semaforo verde «Spazio disponibile più che sufficiente»).",
            "External drive D:\\Dev-Kit is detected with 384 GB free (green badge 'More than enough free space').",
          )}
        />
        <div>
          <Button
            variant="secondary"
            onClick={() => {
              onNavigate("settings");
            }}
          >
            <span className="inline-flex items-center gap-1.5">
              <VscSettingsGear size={14} aria-hidden="true" />
              {tr("Apri Impostazioni", "Open Settings")}
            </span>
          </Button>
        </div>
      </EditorSection>
    </>
  );
}

// ---------------------------------------------------------------------------
// 4.2 New Project & Projects Page
// ---------------------------------------------------------------------------

function TopicFeatProjects({ onNavigate }: { onNavigate: (page: AppPageTarget) => void }) {
  return (
    <>
      <EditorSection
        id="guide-feat-newproject"
        title={tr("Creazione di un Nuovo Progetto", "Creating a New Project")}
        actions={
          <Button
            small
            variant="primary"
            onClick={() => {
              onNavigate("new");
            }}
          >
            <span className="inline-flex items-center gap-1">
              <VscWand size={12} aria-hidden="true" />
              {tr("Apri Nuovo progetto", "Open New project")}
            </span>
          </Button>
        }
      >
        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
          <GuideShot
            src="03-nuovo-progetto-template.webp"
            title={tr("03 · Scelta del template e del nome progetto", "03 · Choosing template and project name")}
            caption={tr(
              "Selezione del modello starter-kit e validazione in tempo reale del nome della cartella.",
              "Selecting the starter-kit template and real-time folder name validation.",
            )}
            example={tr(
              "Digitando gestionale-fatture, il kit mostra l'anteprima del percorso finale D:\\Dev-Kit\\projects\\gestionale-fatture.",
              "Typing gestionale-fatture previews the target path D:\\Dev-Kit\\projects\\gestionale-fatture.",
            )}
          />
          <GuideShot
            src="04-nuovo-progetto-github.webp"
            title={tr("04 · Opzione Crea anche su GitHub", "04 · Also create on GitHub option")}
            caption={tr(
              "Attivando Crea anche su GitHub puoi scegliere visibilità Privato o Pubblico per creare automaticamente il repository remoto.",
              "Enabling Also create on GitHub lets you choose Private or Public visibility to automatically create the remote repository.",
            )}
            example={tr(
              "Il repository remoto mario-rossi/gestionale-fatture viene creato come Privato tramite gh repo create --template.",
              "Remote repository mario-rossi/gestionale-fatture is created as Private via gh repo create --template.",
            )}
          />
        </div>
      </EditorSection>

      <EditorSection
        id="guide-feat-projects"
        title={tr("Pagina Progetti e le schede operative del progetto", "Projects page & project sections")}
        actions={
          <Button
            small
            variant="primary"
            onClick={() => {
              onNavigate("projects");
            }}
          >
            <span className="inline-flex items-center gap-1">
              <VscFolderLibrary size={12} aria-hidden="true" />
              {tr("Apri Progetti", "Open Projects")}
            </span>
          </Button>
        }
      >
        <GuideShot
          src="02-progetti-elenco.webp"
          title={tr("02 · Elenco progetti e stato vuoto", "02 · Projects list & empty state")}
          caption={tr(
            "La barra laterale elenca tutte le sottocartelle di projects\\ con filtro istantaneo e indicatori dei linguaggi rilevati.",
            "The sidebar lists all subfolders under projects\\ with instant filtering and detected language tags.",
          )}
          example={tr(
            "Tre progetti reali in D:\\Dev-Kit\\projects\\: api-magazzino (Python + FastAPI), gestionale-fatture (Node.js + Next.js) e portale-clienti (PHP 8.4 + Node 22 + MariaDB + MongoDB).",
            "Three real projects in D:\\Dev-Kit\\projects\\: api-magazzino (Python + FastAPI), gestionale-fatture (Node.js + Next.js) and portale-clienti (PHP 8.4 + Node 22 + MariaDB + MongoDB).",
          )}
        />

        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
          <GuideShot
            src="05-progetto-avvio-log.webp"
            title={tr(
              "05 · Scheda 1 — Avvio (IDE, Terminali, Agenti, Azioni)",
              "05 · Tab 1 — Launch (IDE, Terminals, Agents, Actions)",
            )}
            caption={tr(
              "Apritori con 1 clic per Editor VSCodium, Terminali (PowerShell, Git Bash, CMD), Agenti IA (Claude, Codex, Antigravity) e Azioni personalizzate con log live.",
              "1-click launchers for VSCodium Editor, Terminals (PowerShell, Git Bash, CMD), AI Agents (Claude, Codex, Antigravity) and custom Actions with live logs.",
            )}
            example={tr(
              "Tre azioni pronte in portale-clienti: «Avvia dev server Vite» (npm run dev), «Esegui migrazioni DB» (php artisan migrate) e «Popola dati demo» (php artisan db:seed).",
              "Three ready actions in portale-clienti: 'Start Vite dev server' (npm run dev), 'Run DB migrations' (php artisan migrate) and 'Seed demo data' (php artisan db:seed).",
            )}
          />

          <GuideShot
            src="06-progetto-sito-web.webp"
            title={tr("06 · Scheda 2 — Sito web", "06 · Tab 2 — Website")}
            caption={tr(
              "Configurazione del server web dedicato al progetto con modalità, sottocartella pubblica (public), porta HTTP, HTTPS locale e Avvio automatico.",
              "Project-dedicated web server configuration with mode, public subfolder (public), HTTP port, local HTTPS and Auto-start.",
            )}
            example={tr(
              "Il progetto portale-clienti è attivo su http://localhost:8080 e https://localhost:8443 servendo D:\\Dev-Kit\\projects\\portale-clienti\\public con PHP 8.4.",
              "Project portale-clienti is active on http://localhost:8080 and https://localhost:8443 serving D:\\Dev-Kit\\projects\\portale-clienti\\public with PHP 8.4.",
            )}
          />

          <GuideShot
            src="07-progetto-componenti-versioni.webp"
            title={tr(
              "07 · Scheda 3 — Componenti e versioni (.devkit.json)",
              "07 · Tab 3 — Components & versions (.devkit.json)",
            )}
            caption={tr(
              "Selettori per fissare la linea major.minor di ogni linguaggio/database per questo specifico progetto e installazione automatica delle dipendenze (composer, npm, pip/uv, cargo).",
              "Dropdowns to pin the major.minor line of each language/database for this specific project and one-click dependency installation (composer, npm, pip/uv, cargo).",
            )}
            example={tr(
              "portale-clienti fissa Node.js 22 (invece della 24 globale) e PHP 8.4: aprendo un terminale in questo progetto, PATH punterà automaticamente a node\\22 e php\\8.4.",
              "portale-clienti pins Node.js 22 (instead of global 24) and PHP 8.4: opening a terminal in this project automatically points PATH to node\\22 and php\\8.4.",
            )}
          />

          <GuideShot
            src="08-progetto-database-env.webp"
            title={tr("08 · Scheda 4 — Database e variabili .env", "08 · Tab 4 — Databases & .env variables")}
            caption={tr(
              "Creazione istantanea di un database MariaDB/MongoDB omonimo (portale_clienti) e scrittura automatica delle stringhe di connessione nel file .env del progetto.",
              "Instant creation of a matching MariaDB/MongoDB database (portale_clienti) and automatic injection of connection strings into the project's .env file.",
            )}
            example={tr(
              "Con un clic su «Scrivi nel file .env», il kit compila DB_HOST=127.0.0.1, DB_PORT=3306, DB_DATABASE=portale_clienti, DATABASE_URL e MONGODB_URI.",
              "Clicking 'Write to .env file' populates DB_HOST=127.0.0.1, DB_PORT=3306, DB_DATABASE=portale_clienti, DATABASE_URL and MONGODB_URI.",
            )}
          />
        </div>

        <GuideShot
          src="09-progetto-git-mcp.webp"
          title={tr(
            "09 · Scheda 5 — Git, sincronizzazione GitHub e Server MCP di progetto",
            "09 · Tab 5 — Git, GitHub sync & project MCP servers",
          )}
          caption={tr(
            "Gestione del ramo corrente, file modificati, commit + push con un clic, pubblicazione di progetti locali su GitHub e aggiunta di server MCP dedicati al singolo progetto.",
            "Current branch management, modified files, one-click commit + push, publishing local projects to GitHub and adding project-specific MCP servers.",
          )}
          example={tr(
            "3 file modificati pronti per «Salva e invia modifiche» sul ramo main (sincronizzato con mario-rossi/portale-clienti) e server MCP sqlite-locale configurato nel progetto.",
            "3 modified files ready for 'Commit & Push' on branch main (synced with mario-rossi/portale-clienti) and a local sqlite-locale MCP server configured on the project.",
          )}
        />
      </EditorSection>
    </>
  );
}

// ---------------------------------------------------------------------------
// 4.3 AI Agents Page (Claude Code, Codex, Antigravity)
// ---------------------------------------------------------------------------

function TopicFeatAgents({ onNavigate }: { onNavigate: (page: AppPageTarget) => void }) {
  return (
    <>
      <EditorSection
        id="guide-feat-agents"
        title={tr(
          "Pagina Agenti: sincronizzazione, quote in tempo reale e statistiche",
          "Agents page: synchronization, real-time quotas & statistics",
        )}
        actions={
          <Button
            small
            variant="primary"
            onClick={() => {
              onNavigate("agents");
            }}
          >
            <span className="inline-flex items-center gap-1">
              <VscHubot size={12} aria-hidden="true" />
              {tr("Apri pagina Agenti", "Open Agents page")}
            </span>
          </Button>
        }
      >
        <GuideShot
          src="10-agenti-quote-statistiche.webp"
          title={tr(
            "10 · Pagina Agenti con quote live, statistiche 30 giorni e MCP condivisi",
            "10 · Agents page with live quotas, 30-day stats & shared MCP",
          )}
          caption={tr(
            "Pannello unificato con barre di avanzamento delle quote (finestra 5h, 7 giorni e modelli Antigravity), statistiche d'uso degli ultimi 30 giorni (sessioni, messaggi, token in/out, costo stimato in USD) ed elenco di skill e server MCP condivisi.",
            "Unified dashboard with quota progress bars (5h window, 7-day window and Antigravity models), 30-day usage statistics (sessions, messages, in/out tokens, estimated USD cost) and shared skills & MCP servers.",
          )}
          example={tr(
            "Claude Code (piano Max, 18% finestra 5h), Codex (piano Plus, 12% finestra 5h) e Antigravity (Google AI Pro) condividono 3 skill (architettura-pulita, migrazioni-sicure, test-automatici) e 2 server MCP (filesystem-kit, github-mcp).",
            "Claude Code (Max plan, 18% 5h window), Codex (Plus plan, 12% 5h window) and Antigravity (Google AI Pro) share 3 skills and 2 MCP servers.",
          )}
        />

        <SimpleTable
          columns={[
            tr("Elemento condiviso", "Shared element"),
            tr("Sorgente in data\\agents\\", "Source in data\\agents\\"),
            tr("Destinazioni sincronizzate automaticamente", "Automatically synchronized destinations"),
          ]}
          rows={[
            {
              key: "instructions",
              cells: [
                <strong key="e">{tr("Istruzioni globali", "Global instructions")}</strong>,
                <code key="s" className="font-mono text-[11px]">
                  data\agents\AGENTS.md
                </code>,
                <span key="d" className="text-[12px]">
                  <code>data\claude\CLAUDE.md</code> · <code>data\codex\AGENTS.md</code> ·{" "}
                  <code>data\home\.gemini\GEMINI.md</code>
                </span>,
              ],
            },
            {
              key: "skills",
              cells: [
                <strong key="e">{tr("Competenze (Skill)", "Skills")}</strong>,
                <code key="s" className="font-mono text-[11px]">
                  data\agents\skills\&lt;nome&gt;\SKILL.md
                </code>,
                <span key="d" className="text-[12px]">
                  <code>data\claude\skills\</code> · <code>data\codex\skills\</code> ·{" "}
                  <code>data\home\.gemini\antigravity\skills\</code>
                </span>,
              ],
            },
            {
              key: "mcp",
              cells: [
                <strong key="e">{tr("Server MCP globali", "Global MCP servers")}</strong>,
                <code key="s" className="font-mono text-[11px]">
                  data\agents\mcp.json
                </code>,
                <span key="d" className="text-[12px]">
                  <code>data\claude\.claude.json</code> · <code>data\codex\config.toml</code> ·{" "}
                  <code>data\home\.gemini\antigravity\mcp_config.json</code>
                </span>,
              ],
            },
          ]}
        />
      </EditorSection>
    </>
  );
}

// ---------------------------------------------------------------------------
// 4.4 Databases & Web Servers Pages
// ---------------------------------------------------------------------------

function TopicFeatServices({ onNavigate }: { onNavigate: (page: AppPageTarget) => void }) {
  return (
    <>
      <EditorSection
        id="guide-feat-databases"
        title={tr(
          "Pagina Database: MariaDB (con Adminer), MongoDB e SQLite",
          "Databases page: MariaDB (with Adminer), MongoDB & SQLite",
        )}
        actions={
          <Button
            small
            variant="primary"
            onClick={() => {
              onNavigate("databases");
            }}
          >
            <span className="inline-flex items-center gap-1">
              <VscDatabase size={12} aria-hidden="true" />
              {tr("Apri pagina Database", "Open Databases page")}
            </span>
          </Button>
        }
      >
        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
          <GuideShot
            src="11-database-mariadb-adminer.webp"
            title={tr("11 · MariaDB 11.4 + interfaccia web Adminer", "11 · MariaDB 11.4 + Adminer web interface")}
            caption={tr(
              "Controllo del motore MariaDB (127.0.0.1:3306, utente root), selettore di versione, avvio con 1 clic di Adminer con auto-login e backup/ripristino .sql.",
              "MariaDB engine control (127.0.0.1:3306, root user), version selector, 1-click Adminer launch with auto-login and .sql backup/restore.",
            )}
            example={tr(
              "Tre database applicativi presenti: api_magazzino (18,4 MB), gestionale_fatture (42,1 MB) e portale_clienti (9,6 MB), con Adminer attivo su 127.0.0.1:18306.",
              "Three application databases: api_magazzino (18.4 MB), gestionale_fatture (42.1 MB) and portale_clienti (9.6 MB), with Adminer active on 127.0.0.1:18306.",
            )}
          />

          <GuideShot
            src="11b-database-mongodb-esploratore.webp"
            title={tr(
              "11b · MongoDB 8.0 ed esploratore collezioni JSON",
              "11b · MongoDB 8.0 & JSON collection explorer",
            )}
            caption={tr(
              "Gestione del motore MongoDB (127.0.0.1:27017), backup/ripristino tramite mongodump/mongorestore (--archive --gzip) ed esploratore integrato di collezioni e documenti JSON.",
              "MongoDB engine management (127.0.0.1:27017), backup/restore via mongodump/mongorestore (--archive --gzip) and built-in JSON collection/document explorer.",
            )}
            example={tr(
              "Ispezione diretta della collezione sessioni_utenti (1.284 documenti) del database portale_clienti senza installare MongoDB Compass.",
              "Direct inspection of the sessioni_utenti collection (1,284 documents) inside the portale_clienti database without needing MongoDB Compass.",
            )}
          />
        </div>
      </EditorSection>

      <EditorSection
        id="guide-feat-servers"
        title={tr(
          "Pagina Server Web: pool VirtualHost e gestione multi-versione",
          "Web Servers page: VirtualHost pools & multi-version management",
        )}
        actions={
          <Button
            small
            variant="primary"
            onClick={() => {
              onNavigate("servers");
            }}
          >
            <span className="inline-flex items-center gap-1">
              <VscServerProcess size={12} aria-hidden="true" />
              {tr("Apri pagina Server Web", "Open Web Servers page")}
            </span>
          </Button>
        }
      >
        <GuideShot
          src="12-server-web-pool.webp"
          title={tr("12 · Pagina Server Web e pool multi-versione", "12 · Web Servers page & multi-version pool")}
          caption={tr(
            "Controllo globale dei pool server, URL cliccabili HTTP/HTTPS, stato per singolo progetto e visualizzatore dei log.",
            "Global server pool controls, clickable HTTP/HTTPS URLs, per-project status and live log viewer.",
          )}
          example={tr(
            "I siti portale-clienti (:8080/:8443 con PHP 8.4), gestionale-fatture (:8081/:8444) e sito-vetrina-legacy (con PHP 7.4) girano contemporaneamente senza conflitti.",
            "Sites portale-clienti (:8080/:8443 with PHP 8.4), gestionale-fatture (:8081/:8444) and sito-vetrina-legacy (with PHP 7.4) run simultaneously without conflicts.",
          )}
        />
      </EditorSection>
    </>
  );
}

// ---------------------------------------------------------------------------
// 4.5 Tools Page, Settings Page & CLI Commands
// ---------------------------------------------------------------------------

function TopicFeatToolsCli({ onNavigate }: { onNavigate: (page: AppPageTarget) => void }) {
  return (
    <>
      <EditorSection
        id="guide-feat-tools"
        title={tr(
          "Pagina Strumenti: catalogo componenti e matrice multi-versione",
          "Tools page: component catalog & multi-version matrix",
        )}
        actions={
          <Button
            small
            variant="primary"
            onClick={() => {
              onNavigate("components");
            }}
          >
            <span className="inline-flex items-center gap-1">
              <VscTools size={12} aria-hidden="true" />
              {tr("Apri pagina Strumenti", "Open Tools page")}
            </span>
          </Button>
        }
      >
        <GuideShot
          src="13-strumenti-catalogo-matrice.webp"
          title={tr("13 · Pagina Strumenti e matrice delle linee di versione", "13 · Tools page & version line matrix")}
          caption={tr(
            "Barra laterale divisa in categorie e scheda di dettaglio con matrice delle linee major.minor disponibili dalla fonte ufficiale (Installa, Rendi predefinita, Rimuovi).",
            "Sidebar divided into categories and detail view with the matrix of major.minor lines available from the official vendor (Install, Set default, Remove).",
          )}
          example={tr(
            "Scheda di PHP con due linee installate contemporaneamente: 8.4 (8.4.4, Predefinita globale) e 7.4 (7.4.33 EOL per progetti legacy), più 8.3 e 8.2 installabili con un clic.",
            "PHP detail view with two lines installed side by side: 8.4 (8.4.4, Global default) and 7.4 (7.4.33 EOL for legacy projects), plus 8.3 and 8.2 installable with one click.",
          )}
        />
      </EditorSection>

      <EditorSection
        id="guide-feat-settings"
        title={tr(
          "Pagina Impostazioni: aspetto, account GitHub, modelli, rete e manutenzione",
          "Settings page: appearance, GitHub account, templates, network & maintenance",
        )}
        actions={
          <Button
            small
            variant="primary"
            onClick={() => {
              onNavigate("settings");
            }}
          >
            <span className="inline-flex items-center gap-1">
              <VscSettingsGear size={12} aria-hidden="true" />
              {tr("Apri Impostazioni", "Open Settings")}
            </span>
          </Button>
        }
      >
        <GuideShot
          src="14-impostazioni-github-rete.webp"
          title={tr("14 · Pagina Impostazioni completa", "14 · Complete Settings page")}
          caption={tr(
            "Configurazione di Lingua (IT/EN/FR/DE), Account GitHub, Modelli di progetto, Rete (Proxy HTTP/HTTPS) e informazioni sul kit.",
            "Configuration of Language (IT/EN/FR/DE), GitHub Account, Project Templates, Network (HTTP/HTTPS Proxy) and kit details.",
          )}
          example={tr(
            "Account @mario-rossi collegato con semaforo verde «Tutti i permessi richiesti sono attivi» e verifica di rete superata su tutti i 9 endpoint (38–112 ms).",
            "Account @mario-rossi connected with green badge 'All required permissions are active' and network check passed across all 9 endpoints (38–112 ms).",
          )}
        />
      </EditorSection>

      <EditorSection
        id="guide-feat-cli"
        title={tr("Interfaccia a riga di comando (CLI) di Dev-Kit.exe", "Dev-Kit.exe Command-Line Interface (CLI)")}
      >
        <p className="m-0 text-[12px] text-vsc-desc">
          {tr(
            "Poiché Dev-Kit.exe usa il sottosistema grafico di Windows per non aprire finestre console nere all'avvio, da PowerShell aggiungi sempre | Out-Host per leggere l'output testuale.",
            "Because Dev-Kit.exe uses the Windows GUI subsystem to avoid flashing black console windows on startup, always pipe to | Out-Host in PowerShell to read text output.",
          )}
        </p>
        <SimpleTable
          columns={[
            tr("Comando PowerShell (con | Out-Host)", "PowerShell command (with | Out-Host)"),
            tr("Funzione", "Description"),
          ]}
          rows={CLI_COMMANDS.map((row) => ({
            key: row.cmd,
            cells: [
              <CopyCmdCell key="c" cmd={row.cmd} />,
              <span key="d" className="text-[12px] text-vsc-fg/90">
                {row.desc()}
              </span>,
            ],
          }))}
        />
      </EditorSection>
    </>
  );
}

function CopyCmdCell({ cmd }: { cmd: string }) {
  const { copied, copy } = useCopy(1200);
  return (
    <div className="flex items-center justify-between gap-2">
      <code className="font-mono text-[11px] text-vsc-settings-header select-text">{cmd}</code>
      <button
        type="button"
        onClick={() => {
          copy(cmd);
        }}
        title={t("Copia")}
        className="inline-flex shrink-0 cursor-pointer items-center gap-1 rounded border border-vsc-border bg-vsc-sidebar px-1.5 py-0.5 text-[10px] text-vsc-desc hover:text-vsc-fg"
      >
        {copied ? <VscCheck size={11} aria-hidden="true" /> : <VscCopy size={11} aria-hidden="true" />}
        <span>{copied ? t("Copiata") : t("Copia")}</span>
      </button>
    </div>
  );
}

// ---------------------------------------------------------------------------
// 5.1 - 5.3 Setup Recipes: Init, Proxy/Net, Templates
// ---------------------------------------------------------------------------

function TopicCfgInitNet({ onNavigate }: { onNavigate: (page: AppPageTarget) => void }) {
  return (
    <>
      <EditorSection
        id="guide-cfg-init-net"
        title={tr(
          "Procedure 5.1 – 5.3: Inizializzazione unità, Rete/Proxy e Modelli di progetto",
          "Procedures 5.1 – 5.3: Drive initialization, Network/Proxy & Project templates",
        )}
      >
        <GuideRecipe
          number="§ 5.1"
          title={tr(
            "Scelta dell'unità, inizializzazione del kit e primo install",
            "Drive selection, kit initialization & first install",
          )}
          whereUi={tr("Wizard di primo avvio (o pagina Strumenti)", "First-Run Wizard (or Tools page)")}
          whereFile="data\app\state.json"
          targetPage="components"
          targetLabel={tr("Apri Strumenti", "Open Tools")}
          onNavigate={onNavigate}
          steps={[
            tr(
              "Formatta l'unità USB/SSD esterna in NTFS o exFAT (consigliato NTFS per prestazioni I/O su node_modules) e copia Dev-Kit.exe in una cartella dedicata (es. D:\\Dev-Kit\\Dev-Kit.exe).",
              "Format your external USB/SSD drive as NTFS or exFAT (NTFS recommended for node_modules I/O performance) and copy Dev-Kit.exe into a dedicated folder (e.g. D:\\Dev-Kit\\Dev-Kit.exe).",
            ),
            tr(
              "Avvia Dev-Kit.exe e verifica nel Passo 1 del Wizard che lo spazio libero sia almeno 10–15 GB per un profilo completo.",
              "Launch Dev-Kit.exe and check in Step 1 of the Wizard that free space is at least 10–15 GB for a full profile.",
            ),
            tr(
              "Nel Passo 2 spunta gli strumenti desiderati e le linee major.minor di linguaggi e database.",
              "In Step 2, check the desired tools and major.minor lines for languages and databases.",
            ),
            tr(
              "Nel Passo 3 collega l'account GitHub e nel Passo 4 avvia lo scaricamento verificato crittograficamente.",
              "In Step 3, connect your GitHub account and in Step 4 start the cryptographically verified download.",
            ),
          ]}
          purpose={tr(
            "Crea l'intera gerarchia isolata data\\, genera i wrapper portabili e registra in data\\app\\state.json la mappa dei componenti scelti e delle versioni effettivamente installate.",
            "Creates the entire isolated data\\ hierarchy, generates portable wrappers and records in data\\app\\state.json the map of chosen components and installed versions.",
          )}
        />

        <GuideRecipe
          number="§ 5.2"
          title={tr(
            "Configurazione della rete, del Proxy aziendale e certificati CA",
            "Network, corporate Proxy & CA certificates configuration",
          )}
          whereUi={tr("Impostazioni → Rete", "Settings → Network")}
          whereFile="data\app\settings.json (sezione network)"
          targetPage="settings"
          targetLabel={tr("Configura Rete / Proxy", "Configure Network / Proxy")}
          onNavigate={onNavigate}
          steps={[
            tr(
              "Apri Impostazioni (icona ingranaggio in basso a sinistra) e seleziona la voce «Rete».",
              "Open Settings (gear icon at bottom left) and select 'Network'.",
            ),
            tr(
              "Se ti trovi dietro un proxy aziendale o scolastico, inserisci l'URL nei campi Proxy HTTPS / Proxy HTTP (es. http://utente:pass@proxy.azienda.it:8080) e gli eventuali certificati CA aggiuntivi.",
              "If behind a corporate or school proxy, enter the URL in HTTPS Proxy / HTTP Proxy and any additional CA certificates.",
            ),
            tr(
              "Clicca su Salva nella barra delle schede per applicare subito la configurazione a tutti i download e ai terminali.",
              "Click Save on the tab bar to immediately apply the configuration to all downloads and terminals.",
            ),
          ]}
          purpose={tr(
            "Propaga automaticamente HTTP_PROXY, HTTPS_PROXY, NO_PROXY e NODE_EXTRA_CA_CERTS sia al client HTTP interno di Rust sia a tutti i processi figli (git, gh, npm, pip, uv, composer, cargo, agenti IA).",
            "Automatically propagates HTTP_PROXY, HTTPS_PROXY, NO_PROXY and NODE_EXTRA_CA_CERTS both to Rust's internal HTTP client and to all child processes (git, gh, npm, pip, uv, composer, cargo, AI agents).",
          )}
        />

        <GuideRecipe
          number="§ 5.3"
          title={tr("Configurazione dei Framework e Modelli di progetto", "Configuring Project Frameworks & Templates")}
          whereUi={tr("Impostazioni → Istruzioni di Nuovo progetto", "Settings → New project instructions")}
          whereFile="data\app\frameworks\"
          targetPage="settings"
          targetLabel={tr("Gestisci Framework in Impostazioni", "Manage Frameworks in Settings")}
          onNavigate={onNavigate}
          steps={[
            tr(
              "Vai in Impostazioni e osserva nella barra laterale la sezione dedicata ai Framework e componenti aggiuntivi di Nuovo progetto.",
              "Go to Settings and check the sidebar section dedicated to New project Frameworks and add-ons.",
            ),
            tr(
              "Seleziona una base (es. Laravel, Vite, Electron, Next.js) o clicca sul pulsante + per aggiungere una nuova ricetta di creazione con comandi, strumenti richiesti e variabili .env predefinite.",
              "Select a base (e.g. Laravel, Vite, Electron, Next.js) or click + to add a new creation recipe with commands, required tools and default .env variables.",
            ),
            tr(
              "Salva le modifiche: la ricetta comparirà immediatamente nella pagina Nuovo progetto.",
              "Save your changes: the recipe will immediately appear on the New project page.",
            ),
          ]}
          purpose={tr(
            "Standardizza l'architettura iniziale dei progetti del team permettendo di istanziare nuovi applicativi completi di dipendenze e variabili .env con un solo clic.",
            "Standardizes initial project architecture across your team, allowing new applications complete with dependencies and .env variables to be created with a single click.",
          )}
        />
      </EditorSection>
    </>
  );
}

// ---------------------------------------------------------------------------
// 5.4 - 5.7 Setup Recipes: Languages (.devkit.json), Agents, Skills, MCP
// ---------------------------------------------------------------------------

function TopicCfgLangsAgents({ onNavigate }: { onNavigate: (page: AppPageTarget) => void }) {
  return (
    <>
      <EditorSection
        id="guide-cfg-langs-agents"
        title={tr(
          "Procedure 5.4 – 5.7: Versioni linguaggi, Agenti IA, Skill e Server MCP",
          "Procedures 5.4 – 5.7: Language versions, AI Agents, Skills & MCP Servers",
        )}
      >
        <GuideRecipe
          number="§ 5.4"
          title={tr(
            "Configurazione delle versioni dei linguaggi (globale vs per-progetto)",
            "Configuring language versions (global vs per-project)",
          )}
          whereUi={tr(
            "Strumenti / Impostazioni → Versioni (globale) e Progetti → Componenti (per il singolo progetto)",
            "Tools / Settings → Versions (global) and Projects → Components (for a single project)",
          )}
          whereFile="data\app\settings.json · data\projects\<id>\dev-settings.json"
          targetPage="components"
          targetLabel={tr("Apri Strumenti", "Open Tools")}
          onNavigate={onNavigate}
          steps={[
            tr(
              "Nella pagina Strumenti seleziona un componente multi-versione (Python, Node.js, PHP, .NET, Java, Go, Ruby, R, MariaDB o MongoDB) e installa una o più linee major.minor.",
              "In the Tools page, select a multi-version component (Python, Node.js, PHP, .NET, Java, Go, Ruby, R, MariaDB or MongoDB) and install one or more major.minor lines.",
            ),
            tr(
              "Imposta la versione predefinita del kit in Impostazioni → Versioni (oppure direttamente dalla scheda dello strumento).",
              "Set the kit's default version in Settings → Versions (or directly from the tool's detail tab).",
            ),
            tr(
              "Per un progetto che richiede una versione diversa, apri Progetti → <progetto> → scheda «Componenti» e scegli la linea specifica per quel progetto.",
              "For a project requiring a different version, open Projects → <project> → 'Components' tab and pick the specific line for that project.",
            ),
          ]}
          purpose={tr(
            "Costruisce dinamicamente la variabile PATH per ogni progetto, anteponendo i binari delle versioni scelte per quel progetto e impostando automaticamente JAVA_HOME, GOROOT e COMPOSER_PHP.",
            "Dynamically constructs the PATH variable for each project, prepending binaries for the project's chosen versions and automatically setting JAVA_HOME, GOROOT and COMPOSER_PHP.",
          )}
        />

        <GuideRecipe
          number="§ 5.5"
          title={tr(
            "Autenticazione dei 3 Agenti IA e salvataggio credenziali cifrate DPAPI",
            "Authenticating the 3 AI Agents & DPAPI encrypted credential storage",
          )}
          whereUi={tr("Agenti (o Progetti → Avvio → pulsante Agente)", "Agents (or Projects → Launch → Agent button)")}
          whereFile="data\claude\.credentials.json · data\codex\auth.json · data\antigravity\.keyring.dpapi"
          targetPage="agents"
          targetLabel={tr("Apri pagina Agenti", "Open Agents page")}
          onNavigate={onNavigate}
          steps={[
            tr(
              "Per Claude Code: avvialo da un progetto; al primo avvio esegui il login OAuth Anthropic nel browser. Il token viene salvato in data\\claude\\.credentials.json.",
              "For Claude Code: launch it from a project; on first run complete the Anthropic OAuth login in your browser. The token is saved in data\\claude\\.credentials.json.",
            ),
            tr(
              "Per Codex: avvialo da un progetto e completa l'accesso ChatGPT/OpenAI; il token viene salvato in data\\codex\\auth.json.",
              "For Codex: launch it from a project and complete the ChatGPT/OpenAI login; the token is saved in data\\codex\\auth.json.",
            ),
            tr(
              "Per Antigravity: avvialo tramite il pulsante Antigravity (che passa per data\\kit\\bin\\agy.cmd); alla chiusura, Dev-Kit.exe keyring save esporta e cifra con Windows DPAPI il token in data\\antigravity\\.keyring.dpapi.",
              "For Antigravity: launch it via the Antigravity button (which runs data\\kit\\bin\\agy.cmd); on exit, Dev-Kit.exe keyring save exports and encrypts the token with Windows DPAPI into data\\antigravity\\.keyring.dpapi.",
            ),
          ]}
          purpose={tr(
            "Mantiene le sessioni IA confinate sull'unità portabile e abilita il monitoraggio in tempo reale delle quote residue nella pagina Agenti.",
            "Keeps AI sessions confined to the portable drive and enables real-time remaining quota monitoring on the Agents page.",
          )}
        />

        <GuideRecipe
          number="§ 5.6"
          title={tr(
            "Configurazione delle Istruzioni globali (AGENTS.md) e delle Skill condivise",
            "Configuring global Instructions (AGENTS.md) & shared Skills",
          )}
          whereUi={tr("Agenti → cartella condivisa data\\agents\\", "Agents → shared folder data\\agents\\")}
          whereFile="data\agents\AGENTS.md · data\agents\skills\<nome>\SKILL.md"
          targetPage="agents"
          targetLabel={tr("Apri Agenti", "Open Agents")}
          onNavigate={onNavigate}
          steps={[
            tr(
              "Nella pagina Agenti clicca sull'icona «Apri la cartella condivisa» in alto nella barra laterale per aprire data\\agents\\.",
              "In the Agents page, click the 'Open shared folder' icon at the top of the sidebar to open data\\agents\\.",
            ),
            tr(
              "Modifica il file AGENTS.md per definire le tue regole di codifica, lingua preferita e convenzioni architetturali valide per tutti gli agenti.",
              "Edit AGENTS.md to define your coding rules, preferred language and architectural conventions across all agents.",
            ),
            tr(
              "Aggiungi sottocartelle in data\\agents\\skills\\<nome>\\SKILL.md per condividere competenze specializzate: ad ogni avvio di un agente il kit le allinea automaticamente.",
              "Add subfolders in data\\agents\\skills\\<name>\\SKILL.md to share specialized skills: every time an agent starts, the kit synchronizes them automatically.",
            ),
          ]}
          purpose={tr(
            "Evita di dover mantenere 3 copie separate delle regole e delle skill: una sola modifica in data\\agents\\ istruisce contemporaneamente Claude Code, Codex e Antigravity.",
            "Eliminates maintaining 3 separate copies of rules and skills: a single edit in data\\agents\\ instructs Claude Code, Codex and Antigravity simultaneously.",
          )}
        />

        <GuideRecipe
          number="§ 5.7"
          title={tr("Configurazione dei Server MCP condivisi", "Configuring shared MCP Servers")}
          whereUi={tr("Cartella condivisa data\\agents\\mcp.json", "Shared folder data\\agents\\mcp.json")}
          whereFile="data\agents\mcp.json"
          targetPage="agents"
          targetLabel={tr("Vai ad Agenti", "Go to Agents")}
          onNavigate={onNavigate}
          steps={[
            tr(
              "Apri il file data\\agents\\mcp.json nella cartella condivisa degli agenti.",
              "Open data\\agents\\mcp.json in the shared agents folder.",
            ),
            tr(
              "Aggiungi dentro l'oggetto mcpServers i server desiderati specificando command e args.",
              "Add your desired servers inside the mcpServers object specifying command and args.",
            ),
            tr(
              "All'avvio di un agente (o eseguendo .\\Dev-Kit.exe sync), il kit traduce automaticamente la definizione nei 3 formati (.claude.json, config.toml di Codex e mcp_config.json di Antigravity).",
              "When launching an agent (or running .\\Dev-Kit.exe sync), the kit automatically translates the definition into all 3 formats (.claude.json, Codex config.toml and Antigravity mcp_config.json).",
            ),
          ]}
          purpose={tr(
            "Estende le capacità degli agenti IA permettendo loro di interrogare database locali, documentazione o strumenti esterni tramite il protocollo standard Model Context Protocol.",
            "Extends AI agent capabilities, allowing them to query local databases, documentation or external tools via the standard Model Context Protocol.",
          )}
        >
          <GuideCode
            title={tr("Esempio di configurazione MCP condivisa", "Example shared MCP configuration")}
            path="<Radice>\data\agents\mcp.json"
            code={`{
  "mcpServers": {
    "filesystem-kit": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "D:/Dev-Kit/projects"]
    },
    "github-mcp": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"]
    }
  }
}`}
          />
        </GuideRecipe>
      </EditorSection>
    </>
  );
}

// ---------------------------------------------------------------------------
// 5.8 - 5.9 Setup Recipes: Databases & .env, Web Server
// ---------------------------------------------------------------------------

function TopicCfgDbWeb({ onNavigate }: { onNavigate: (page: AppPageTarget) => void }) {
  return (
    <>
      <EditorSection
        id="guide-cfg-db-web"
        title={tr(
          "Procedure 5.8 – 5.9: Database locali, iniezione .env e Server Web",
          "Procedures 5.8 – 5.9: Local databases, .env injection & Web Server",
        )}
      >
        <GuideRecipe
          number="§ 5.8"
          title={tr(
            "Creazione di un Database di progetto e collegamento con .env",
            "Creating a project Database & linking with .env",
          )}
          whereUi={tr(
            "Progetti → <progetto> → scheda Database (oppure pagina Database)",
            "Projects → <project> → Databases tab (or Databases page)",
          )}
          whereFile="data\db\ · projects\<nome>\.env"
          targetPage="databases"
          targetLabel={tr("Apri pagina Database", "Open Databases page")}
          onNavigate={onNavigate}
          steps={[
            tr(
              "Apri il progetto in Progetti → <progetto> e seleziona la scheda «Database».",
              "Open your project in Projects → <project> and select the 'Databases' tab.",
            ),
            tr(
              "Crea o collega un database MariaDB, MongoDB o SQLite per il progetto: il motore parte automaticamente in locale su 127.0.0.1.",
              "Create or link a MariaDB, MongoDB or SQLite database for the project: the engine starts automatically on 127.0.0.1.",
            ),
            tr(
              "Usa i pulsanti rapidi per sincronizzare le credenziali nel file .env del progetto o per esplorare tabelle e collezioni.",
              "Use the quick buttons to sync credentials into the project's .env file or browse tables and collections.",
            ),
          ]}
          purpose={tr(
            "Elimina gli errori di configurazione manuale delle porte o delle stringhe di connessione e conserva i dati fisici su disco dentro data\\db\\.",
            "Eliminates manual port or connection string typos and stores physical data on disk inside data\\db\\.",
          )}
        >
          <GuideCode
            title={tr("Variabili .env tipiche di un progetto collegato", "Typical .env variables of a linked project")}
            path="projects\portale-clienti\.env"
            code={`DB_CONNECTION=mysql
DB_HOST=127.0.0.1
DB_PORT=3306
DB_DATABASE=portale_clienti
DB_USERNAME=root
DB_PASSWORD=
DATABASE_URL="mysql://root@127.0.0.1:3306/portale_clienti"
MONGODB_URI="mongodb://127.0.0.1:27017/portale_clienti"`}
          />
        </GuideRecipe>

        <GuideRecipe
          number="§ 5.9"
          title={tr(
            "Configurazione del Sito Web di progetto e dei Pool VirtualHost",
            "Configuring a project Website & VirtualHost Pools",
          )}
          whereUi={tr(
            "Progetti → <progetto> → scheda Sito web (e pagina Server Web)",
            "Projects → <project> → Website tab (and Web Servers page)",
          )}
          whereFile="data\projects\<id>\dev-settings.json · pagina Server Web"
          targetPage="servers"
          targetLabel={tr("Apri pagina Server Web", "Open Web Servers page")}
          onNavigate={onNavigate}
          steps={[
            tr("Apri Progetti → <progetto> → scheda «Sito web».", "Open Projects → <project> → 'Website' tab."),
            tr(
              "Scegli il motore web (es. Apache, Nginx o server di sviluppo), la cartella pubblica (es. public) ed eventuali alias relativi.",
              "Choose the web engine (e.g. Apache, Nginx or dev server), the document root (e.g. public) and any relative aliases.",
            ),
            tr(
              "Nella pagina «Server Web» avvia o ricarica il pool corrispondente e apri il sito su <progetto>.localhost con un clic.",
              "In the 'Web Servers' page, start or reload the matching pool and open the site at <project>.localhost with one click.",
            ),
          ]}
          purpose={tr(
            "Raggruppa automaticamente i siti che condividono la stessa combinazione di motore e versione PHP in pool VirtualHost dedicati, rigenerando gli alias relativi se cambia la lettera di unità.",
            "Automatically groups sites sharing the same engine and PHP version into dedicated VirtualHost pools, regenerating relative aliases whenever the drive letter changes.",
          )}
        />
      </EditorSection>
    </>
  );
}

// ---------------------------------------------------------------------------
// 5.10 - 5.12 Setup Recipes: Custom Actions, Orphans & Complete Settings
// ---------------------------------------------------------------------------

function TopicCfgActionsOrphans({ onNavigate }: { onNavigate: (page: AppPageTarget) => void }) {
  return (
    <>
      <EditorSection
        id="guide-cfg-actions-orphans"
        title={tr(
          "Procedure 5.10 – 5.12: Azioni di avvio, Estensioni Editor e Manutenzione",
          "Procedures 5.10 – 5.12: Launch Actions, Editor Extensions & Maintenance",
        )}
      >
        <GuideRecipe
          number="§ 5.10"
          title={tr(
            "Esecuzione e monitoraggio delle Azioni di avvio di un progetto",
            "Running & monitoring a project's Launch Actions",
          )}
          whereUi={tr(
            "Progetti → <progetto> → scheda Avvio (e pannello Attività in basso)",
            "Projects → <project> → Launch tab (and bottom Activity panel)",
          )}
          whereFile="package.json / composer.json / data\projects\<id>\dev-settings.json"
          targetPage="projects"
          targetLabel={tr("Apri Progetti", "Open Projects")}
          onNavigate={onNavigate}
          steps={[
            tr("Apri Progetti → <progetto> → scheda «Avvio».", "Open Projects → <project> → 'Launch' tab."),
            tr(
              "Seleziona una delle azioni rilevate dal progetto (es. dev server, build, test, migrazioni) e premi «Avvia».",
              "Select one of the detected project actions (e.g. dev server, build, test, migrations) and click 'Start'.",
            ),
            tr(
              "Segui l'output in tempo reale nel riquadro dei log o apri il pannello «Attività» dalla barra di stato inferiore per controllare tutti i processi attivi.",
              "Follow real-time output in the log pane or open the 'Activity' panel from the bottom status bar to monitor all running processes.",
            ),
          ]}
          purpose={tr(
            "Esegue i processi del progetto nell'ambiente isolato del kit con le versioni di linguaggio scelte per quel progetto, rilevando automaticamente le porte HTTP aperte.",
            "Runs project processes inside the kit's isolated environment with the language versions chosen for that project, automatically detecting open HTTP ports.",
          )}
        />

        <GuideRecipe
          number="§ 5.11"
          title={tr(
            "Gestione delle Estensioni VSCodium e delle impostazioni dell'Editor",
            "Managing VSCodium Extensions & Editor settings",
          )}
          whereUi={tr(
            "Strumenti → Estensioni dell'editor (globale) e Progetti → Estensioni / Editor",
            "Tools → Editor extensions (global) and Projects → Extensions / Editor",
          )}
          whereFile="data\vscodium-server\data\extensions\ · projects\<nome>\.vscode\settings.json"
          targetPage="components"
          targetLabel={tr("Apri Strumenti", "Open Tools")}
          onNavigate={onNavigate}
          steps={[
            tr(
              "In Strumenti → «Estensioni dell'editor» consulta la matrice che associa automaticamente le estensioni Open VSX ai linguaggi rilevati.",
              "In Tools → 'Editor extensions', inspect the matrix that automatically maps Open VSX extensions to detected languages.",
            ),
            tr(
              "In Progetti → <progetto> → scheda «Estensioni» aggiungi o escludi estensioni specifiche per quel singolo progetto.",
              "In Projects → <project> → 'Extensions' tab, add or exclude specific extensions for that individual project.",
            ),
            tr(
              "Nella scheda «Editor» del progetto personalizza tab size, formattazione al salvataggio e tema per quel progetto.",
              "In the project's 'Editor' tab, customize tab size, format-on-save and theme for that project.",
            ),
          ]}
          purpose={tr(
            "Mantiene l'editor VSCodium leggero e mirato installando automaticamente da Open VSX solo le estensioni necessarie ai linguaggi effettivamente usati.",
            "Keeps the VSCodium editor fast and focused by automatically installing from Open VSX only the extensions needed for the languages actually used.",
          )}
        />

        <GuideRecipe
          number="§ 5.12"
          title={tr(
            "Aggiornamento del kit e rimozione sicura dell'unità",
            "Updating the kit & safely ejecting the drive",
          )}
          whereUi={tr(
            "Impostazioni → Aggiornamenti / Il kit · Rimuovi l'unità",
            "Settings → Updates / The kit · Eject drive",
          )}
          whereFile="data\app\ · data\antigravity\.keyring.dpapi"
          targetPage="eject"
          targetLabel={tr("Apri Rimuovi l'unità", "Open Eject drive")}
          onNavigate={onNavigate}
          steps={[
            tr(
              "In Impostazioni → «Aggiornamenti» verifica la presenza di nuove versioni dell'applicazione e degli strumenti.",
              "In Settings → 'Updates', check for new versions of the application and tools.",
            ),
            tr(
              "Quando hai terminato di lavorare, apri «Rimuovi l'unità» dalla barra sinistra in basso.",
              "When you finish working, open 'Eject drive' from the bottom-left Activity Bar.",
            ),
            tr(
              "Clicca su «Chiudi tutto e prepara la rimozione» per fermare ordinatamente database, siti web, editor e salvare nel kit l'accesso di Antigravity.",
              "Click 'Close all and prepare removal' to cleanly stop databases, websites, editor and save Antigravity's login into the kit.",
            ),
          ]}
          purpose={tr(
            "Garantisce l'integrità dei database (InnoDB e WiredTiger) e il confinamento totale delle credenziali prima dello scollegamento fisico dell'unità USB/SSD.",
            "Guarantees database integrity (InnoDB and WiredTiger) and complete credential confinement before physically unplugging the USB/SSD drive.",
          )}
        />
      </EditorSection>
    </>
  );
}
