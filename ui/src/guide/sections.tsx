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
          "Cos'è Atollo (Atollo) e quale problema risolve",
          "What Atollo (Atollo) is and what problem it solves",
          "Qu'est-ce que Atollo (Atollo) et quel problème il résout",
          "Was Atollo (Atollo) ist und welches Problem es löst",
        )}
      >
        <p className="m-0 text-[12px] text-vsc-desc">
          {tr(
            "Ambiente di sviluppo integrato, 100% portabile e auto-contenuto per Windows 10/11 (x64 e ARM64), progettato per risiedere su un'unità esterna (SSD/USB) o in una singola cartella locale senza lasciare tracce sul PC ospite.",
            "Integrated, 100% portable and self-contained development environment for Windows 10/11 (x64 & ARM64), designed to live on an external drive (SSD/USB) or in a single local folder without leaving traces on the host PC.",
            "Environnement de développement intégré, 100 % portable et autonome pour Windows 10/11 (x64 et ARM64), conçu pour résider sur un lecteur externe (SSD/USB) ou dans un seul dossier local sans laisser de traces sur le PC hôte.",
            "Integrierte, 100 % portable und eigenständige Entwicklungsumgebung für Windows 10/11 (x64 & ARM64), entwickelt für den Betrieb auf einem externen Laufwerk (SSD/USB) oder in einem einzelnen lokalen Ordner ohne Spuren auf dem Host-PC.",
          )}
        </p>
        <Notice tone="info">
          <div className="space-y-1">
            <div className="font-semibold text-vsc-settings-header">
              {tr(
                "Missione architetturale",
                "Architectural mission",
                "Mission architecturale",
                "Architektonische Mission",
              )}
            </div>
            <div>
              {tr(
                "Collegando l'unità a qualsiasi PC Windows, l'utente ritrova istantaneamente i propri progetti, tutti i linguaggi (anche in più versioni simultanee), i database attivi con i relativi dati, il server web, l'editor, il terminale, l'identità Git/GitHub e tre agenti IA (Claude Code, Codex, Antigravity) con memoria, skill e server MCP sincronizzati.",
                "By plugging the drive into any Windows PC, you immediately get back your projects, all languages (even multiple versions side by side), active databases with their data, web server, editor, terminal, Git/GitHub identity and three AI agents (Claude Code, Codex, Antigravity) with synchronized memory, skills and MCP servers.",
                "En branchant le lecteur sur n'importe quel PC Windows, l'utilisateur retrouve instantanément ses projets, tous les langages (même en plusieurs versions simultanées), ses bases de données actives avec leurs données, le serveur web, l'éditeur, le terminal, son identité Git/GitHub et trois agents IA (Claude Code, Codex, Antigravity) avec mémoire, skills et serveurs MCP synchronisés.",
                "Sobald das Laufwerk an einen beliebigen Windows-PC angeschlossen wird, stehen sofort alle Projekte, Sprachen (auch mehrere Versionen parallel), aktive Datenbanken samt Daten, Webserver, Editor, Terminal, Git/GitHub-Identität sowie drei KI-Agenten (Claude Code, Codex, Antigravity) mit synchronisiertem Speicher, Skills und MCP-Servern bereit.",
              )}
            </div>
          </div>
        </Notice>

        {info && (
          <Properties
            rows={[
              [
                tr("Versione kit attiva", "Active kit version", "Version active du kit", "Aktive Kit-Version"),
                <Tag key="v" tone="focus">{`v${info.version}`}</Tag>,
              ],
              [
                tr("Percorso radice corrente", "Current root path", "Chemin racine actuel", "Aktueller Stammpfad"),
                <code key="r" className="font-mono text-[12px] select-text">
                  {info.root}
                </code>,
              ],
              [
                tr("Cartella progetti", "Projects folder", "Dossier des projets", "Projektordner"),
                <code key="p" className="font-mono text-[12px] select-text">
                  {info.projects}
                </code>,
              ],
              [
                tr(
                  "File system rilevato",
                  "Detected file system",
                  "Système de fichiers détecté",
                  "Erkanntes Dateisystem",
                ),
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
        title={tr(
          "I 6 principi architetturali cardine",
          "The 6 core architectural principles",
          "Les 6 principes architecturaux fondamentaux",
          "Die 6 zentralen Architekturprinzipien",
        )}
      >
        <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
          <GuideCard
            icon={VscLock}
            badge="Zero-Trace"
            title={tr(
              "1. Zero tracce sul PC ospite",
              "1. Zero traces on the host PC",
              "1. Zéro trace sur le PC hôte",
              "1. Null Spuren auf dem Host-PC",
            )}
          >
            {tr(
              "Nessuna scrittura nel registro di Windows, nessuna dipendenza da %USERPROFILE%, %APPDATA% o %LOCALAPPDATA% del computer ospite. Circa 90 variabili d'ambiente vengono sovrascritte in memoria e confinate in <Radice>\\data\\.",
              "No writes to the Windows Registry, no dependency on host %USERPROFILE%, %APPDATA% or %LOCALAPPDATA%. Around 90 environment variables are overridden in memory and confined under <Root>\\data\\.",
              "Aucune écriture dans le registre Windows, aucune dépendance à %USERPROFILE%, %APPDATA% ou %LOCALAPPDATA% du PC hôte. Environ 90 variables d'environnement sont redéfinies en mémoire et confinées dans <Racine>\\data\\.",
              "Keine Einträge in der Windows-Registrierung, keine Abhängigkeit von %USERPROFILE%, %APPDATA% oder %LOCALAPPDATA% des Host-PCs. Rund 90 Umgebungsvariablen werden im Speicher überschrieben und in <Stamm>\\data\\ isoliert.",
            )}
          </GuideCard>

          <GuideCard
            icon={VscFolderLibrary}
            badge="Drive-Letter Agnostic"
            title={tr(
              "2. Indipendenza dalla lettera di unità",
              "2. Drive-letter independence",
              "2. Indépendance de la lettre de lecteur",
              "2. Unabhängigkeit vom Laufwerksbuchstaben",
            )}
          >
            {tr(
              "Tutti i percorsi nei file di configurazione sono ricalcolati dinamicamente ad ogni avvio a partire dalla posizione corrente di Atollo.exe. Se la chiavetta passa da D:\\ a E:\\ su un altro PC, tutto continua a funzionare.",
              "All paths in configuration files are dynamically recomputed on every startup from Atollo.exe's location. If the drive changes from D:\\ to E:\\ on another PC, everything keeps working seamlessly.",
              "Tous les chemins des fichiers de configuration sont recalculés dynamiquement à chaque démarrage depuis l'emplacement de Atollo.exe. Si la clé passe de D:\\ à E:\\ sur un autre PC, tout continue de fonctionner.",
              "Alle Pfade in Konfigurationsdateien werden bei jedem Start dynamisch aus dem Speicherort von Atollo.exe neu berechnet. Wechselt das Laufwerk an einem anderen PC von D:\\ zu E:\\, funktioniert alles nahtlos weiter.",
            )}
          </GuideCard>

          <GuideCard
            icon={VscShield}
            badge="Supply-Chain"
            title={tr(
              "3. Fonti ufficiali e verifica crittografica",
              "3. Official sources & cryptographic verification",
              "3. Sources officielles et vérification cryptographique",
              "3. Offizielle Quellen & kryptografische Prüfung",
            )}
            actionLabel={tr(
              "Esplora le 36 fonti",
              "Explore all 36 sources",
              "Explorer les 36 sources",
              "Alle 36 Quellen ansehen",
            )}
            onAction={() => {
              onSelectTopic("sources-table");
            }}
          >
            {tr(
              "Ogni strumento viene scaricato via HTTPS direttamente dal produttore ufficiale e verificato con SHA-256, SHA-512, SHA3-256, integrità npm sha512 o firma Authenticode prima di entrare in data\\apps\\.",
              "Every tool is downloaded over HTTPS directly from its official vendor and verified with SHA-256, SHA-512, SHA3-256, npm sha512 integrity or Authenticode signature before entering data\\apps\\.",
              "Chaque outil est téléchargé en HTTPS directement auprès de son éditeur officiel et vérifié par SHA-256, SHA-512, SHA3-256, intégrité npm sha512 ou signature Authenticode avant d'entrer dans data\\apps\\.",
              "Jedes Werkzeug wird per HTTPS direkt vom offiziellen Hersteller geladen und vor der Ablage in data\\apps\\ mittels SHA-256, SHA-512, SHA3-256, npm-sha512-Integrität oder Authenticode-Signatur geprüft.",
            )}
          </GuideCard>

          <GuideCard
            icon={VscVersions}
            badge="Multi-Version"
            title={tr(
              "4. Multi-versione per singolo progetto",
              "4. Per-project multi-versioning",
              "4. Multi-versions par projet",
              "4. Multi-Versionierung pro Projekt",
            )}
            actionLabel={tr("Gestisci Strumenti", "Manage Tools", "Gérer les Outils", "Werkzeuge verwalten")}
            onAction={() => {
              onNavigate("components");
            }}
          >
            {tr(
              "Linguaggi e database supportano più linee major.minor installate contemporaneamente (es. PHP 7.4 e 8.4, Node 20 e 24, Python 3.11 e 3.13). Ogni progetto sceglie le proprie versioni nel file data\\projects\\<progetto>\\dev-settings.json.",
              "Languages and databases support multiple major.minor lines installed side by side (e.g. PHP 7.4 & 8.4, Node 20 & 24, Python 3.11 & 3.13). Each project selects its own versions in data\\projects\\<project>\\dev-settings.json.",
              "Les langages et bases de données prennent en charge plusieurs lignes major.minor installées côte à côte (ex. PHP 7.4 et 8.4, Node 20 et 24, Python 3.11 et 3.13). Chaque projet choisit ses versions dans data\\projects\\<projet>\\dev-settings.json.",
              "Sprachen und Datenbanken unterstützen mehrere parallel installierte major.minor-Linien (z. B. PHP 7.4 & 8.4, Node 20 & 24, Python 3.11 & 3.13). Jedes Projekt wählt seine eigenen Versionen in data\\projects\\<projekt>\\dev-settings.json.",
            )}
          </GuideCard>

          <GuideCard
            icon={VscHubot}
            badge="AI Sync"
            title={tr(
              "5. Agenti IA sincronizzati",
              "5. Synchronized AI agents",
              "5. Agents IA synchronisés",
              "5. Synchronisierte KI-Agenten",
            )}
            actionLabel={tr("Apri Agenti", "Open Agents", "Ouvrir Agents", "Agenten öffnen")}
            onAction={() => {
              onNavigate("agents");
            }}
          >
            {tr(
              "Claude Code, Codex e Antigravity condividono un'unica fonte di verità in data\\agents\\ (AGENTS.md, skills\\, mcp.json), allineata automaticamente prima di ogni avvio.",
              "Claude Code, Codex and Antigravity share a single source of truth in data\\agents\\ (AGENTS.md, skills\\, mcp.json), automatically synchronized before every launch.",
              "Claude Code, Codex et Antigravity partagent une source unique de vérité dans data\\agents\\ (AGENTS.md, skills\\, mcp.json), alignée automatiquement avant chaque lancement.",
              "Claude Code, Codex und Antigravity teilen sich eine zentrale Quelle in data\\agents\\ (AGENTS.md, skills\\, mcp.json), die vor jedem Start automatisch abgeglichen wird.",
            )}
          </GuideCard>

          <GuideCard
            icon={VscGithub}
            badge="GitHub SSO"
            title={tr(
              "6. Identità GitHub unificata",
              "6. Unified GitHub identity",
              "6. Identité GitHub unifiée",
              "6. Einheitliche GitHub-Identität",
            )}
            actionLabel={tr("Guida GitHub", "GitHub Guide", "Guide GitHub", "GitHub-Anleitung")}
            onAction={() => {
              onSelectTopic("github-role");
            }}
          >
            {tr(
              "Un singolo login OAuth Device Flow con GitHub CLI configura automaticamente firma dei commit Git, creazione progetti da template, backup cloud e quota API GitHub (5.000 req/ora).",
              "A single OAuth Device Flow login with GitHub CLI automatically configures Git commit authorship, project creation from templates, cloud backups and GitHub API rate limits (5,000 req/hour).",
              "Une seule connexion OAuth Device Flow avec GitHub CLI configure automatiquement l'auteur des commits Git, la création de projets depuis des modèles, les sauvegardes cloud et le quota API GitHub (5 000 req/heure).",
              "Ein einziges OAuth-Device-Flow-Login über GitHub CLI konfiguriert automatisch Git-Commit-Autorenschaft, Projekterstellung aus Vorlagen, Cloud-Backups und das GitHub-API-Limit (5.000 Anfragen/Stunde).",
            )}
          </GuideCard>
        </div>
      </EditorSection>

      <EditorSection
        id="guide-overview-quicklinks"
        title={tr(
          "Percorsi rapidi nella guida",
          "Quick paths in the guide",
          "Accès rapides dans le guide",
          "Schnellzugriffe in der Anleitung",
        )}
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
              {tr(
                "Tabella completa delle 36 fonti",
                "Complete table of all 36 sources",
                "Tableau complet des 36 sources",
                "Vollständige Tabelle aller 36 Quellen",
              )}
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
              {tr(
                "Configurare l'account GitHub",
                "Configure GitHub account",
                "Configurer le compte GitHub",
                "GitHub-Konto einrichten",
              )}
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
              {tr(
                "Scheda progetto e sito web locale",
                "Project detail & local website",
                "Fiche projet et site web local",
                "Projektansicht & lokale Website",
              )}
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
              {tr(
                "12 procedure di configurazione passo-passo",
                "12 step-by-step setup procedures",
                "12 procédures de configuration pas à pas",
                "12 Schritt-für-Schritt-Einrichtungen",
              )}
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
        title={tr(
          "Albero delle cartelle del kit su disco",
          "Kit directory tree on disk",
          "Arborescence des dossiers du kit sur le disque",
          "Ordnerstruktur des Kits auf dem Datenträger",
        )}
      >
        <p className="m-0 text-[12px] text-vsc-desc">
          {tr(
            "Tutto ciò che appartiene al kit vive dentro la cartella in cui si trova Atollo.exe (indicata come <Radice>, ad esempio D:\\Atollo\\). Clicca sui pulsanti qui sotto per aprire direttamente le cartelle in Esplora file.",
            "Everything belonging to the kit lives inside the folder where Atollo.exe resides (denoted as <Root>, e.g. D:\\Atollo\\). Click the buttons below to open folders directly in File Explorer.",
            "Tout ce qui appartient au kit réside dans le dossier contenant Atollo.exe (indiqué comme <Racine>, ex. D:\\Atollo\\). Cliquez sur les boutons ci-dessous pour ouvrir directement les dossiers dans l'Explorateur de fichiers.",
            "Alles, was zum Kit gehört, befindet sich im Ordner von Atollo.exe (bezeichnet als <Stamm>, z. B. D:\\Atollo\\). Klicke auf die Schaltflächen unten, um die Ordner direkt im Datei-Explorer zu öffnen.",
          )}
        </p>
        <KitFoldersBar />
        <SimpleTable
          columns={[
            tr("Percorso su disco", "Disk path", "Chemin sur le disque", "Pfad auf dem Datenträger"),
            tr("Ruolo e contenuto", "Role & contents", "Rôle et contenu", "Rolle & Inhalt"),
          ]}
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
          "Confinement des variables d'environnement (~90 variables)",
          "Isolierung der Umgebungsvariablen (~90 Variablen)",
        )}
      >
        <SimpleTable
          columns={[
            tr("Categoria", "Category", "Catégorie", "Kategorie"),
            tr("Variabili principali", "Key variables", "Variables principales", "Wichtigste Variablen"),
            tr("Destinazione nel kit", "Destination inside kit", "Destination dans le kit", "Zielort im Kit"),
          ]}
          rows={[
            {
              key: "home",
              cells: [
                <strong key="c">
                  {tr(
                    "Profilo utente e XDG",
                    "User profile & XDG",
                    "Profil utilisateur et XDG",
                    "Benutzerprofil & XDG",
                  )}
                </strong>,
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
                <strong key="c">
                  {tr("File temporanei", "Temporary files", "Fichiers temporaires", "Temporäre Dateien")}
                </strong>,
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
                <strong key="c">{tr("Agenti IA", "AI Agents", "Agents IA", "KI-Agenten")}</strong>,
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
                <strong key="c">
                  {tr(
                    "Cache linguaggi e pacchetti",
                    "Language & package caches",
                    "Caches des langages et paquets",
                    "Sprach- & Paket-Caches",
                  )}
                </strong>,
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
                <strong key="c">
                  {tr(
                    "Telemetria disattivata",
                    "Disabled telemetry",
                    "Télémétrie désactivée",
                    "Deaktivierte Telemetrie",
                  )}
                </strong>,
                <code key="v" className="font-mono text-[11px]">
                  DOTNET_CLI_TELEMETRY_OPTOUT=1, NEXT_TELEMETRY_DISABLED=1, ASTRO_TELEMETRY_DISABLED=1
                </code>,
                <span key="d" className="text-[12px]">
                  {tr(
                    "Nessuna telemetria inviata a terze parti",
                    "No telemetry sent to third parties",
                    "Aucune télémétrie envoyée à des tiers",
                    "Keine Telemetriedaten an Dritte gesendet",
                  )}
                </span>,
              ],
            },
          ]}
        />
        {info && (
          <GuideCode
            title={tr(
              "Ispeziona l'ambiente completo da terminale",
              "Inspect full environment from terminal",
              "Inspecter l'environnement complet depuis le terminal",
              "Vollständige Umgebung im Terminal prüfen",
            )}
            path={info.root}
            code={`.\\Atollo.exe env | Out-Host`}
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
        title={tr(
          "Sequenza di avvio automatico del kit",
          "Automatic kit startup sequence",
          "Séquence de démarrage automatique du kit",
          "Automatische Startsequenz des Kits",
        )}
      >
        <SimpleTable
          columns={[
            "#",
            tr("Fase di avvio", "Startup phase", "Phase de démarrage", "Startphase"),
            tr(
              "Operazioni eseguite in automatico",
              "Operations performed automatically",
              "Opérations exécutées automatiquement",
              "Automatisch ausgeführte Vorgänge",
            ),
          ]}
          rows={[
            {
              key: "1",
              cells: [
                <Tag key="n" tone="focus">
                  1
                </Tag>,
                <strong key="t">
                  {tr("Rilevamento modalità", "Mode detection", "Détection du mode", "Moduserkennung")}
                </strong>,
                <span key="d">
                  {tr(
                    "Senza argomenti apre la finestra grafica nativa (Tauri WebView2); con argomenti esegue il comando CLI richiesto e termina.",
                    "Without arguments opens the native GUI window (Tauri WebView2); with arguments runs the requested CLI subcommand and exits.",
                    "Sans arguments, ouvre la fenêtre graphique native (Tauri WebView2) ; avec arguments, exécute la sous-commande CLI demandée et se termine.",
                    "Ohne Argumente wird das native GUI-Fenster (Tauri WebView2) geöffnet; mit Argumenten wird der angeforderte CLI-Befehl ausgeführt und beendet.",
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
                <strong key="t">
                  {tr(
                    "Inizializzazione stato",
                    "State initialization",
                    "Initialisation de l'état",
                    "Statusinitialisierung",
                  )}
                </strong>,
                <span key="d">
                  {tr(
                    "Individua la cartella di Atollo.exe, crea le sottocartelle mancanti in data\\ e scrive i file shim (agy.cmd, ssh-wrapper.cmd, editor.cmd, gitconfig).",
                    "Detects Atollo.exe's folder, creates any missing subfolders under data\\ and writes shim files (agy.cmd, ssh-wrapper.cmd, editor.cmd, gitconfig).",
                    "Détecte le dossier de Atollo.exe, crée les sous-dossiers manquants dans data\\ et écrit les fichiers shim (agy.cmd, ssh-wrapper.cmd, editor.cmd, gitconfig).",
                    "Ermittelt den Ordner von Atollo.exe, erstellt fehlende Unterordner in data\\ und schreibt die Shim-Dateien (agy.cmd, ssh-wrapper.cmd, editor.cmd, gitconfig).",
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
                <strong key="t">
                  {tr(
                    "Allineamento identità Git",
                    "Git identity alignment",
                    "Alignement de l'identité Git",
                    "Abgleich der Git-Identität",
                  )}
                </strong>,
                <span key="d">
                  {tr(
                    "Se GitHub CLI è autenticato, verifica ed eventualmente riallinea user.name e user.email nel .gitconfig del kit.",
                    "If GitHub CLI is authenticated, checks and realigns user.name and user.email in the kit's .gitconfig.",
                    "Si GitHub CLI est authentifié, vérifie et réaligne si besoin user.name et user.email dans le .gitconfig du kit.",
                    "Ist GitHub CLI authentifiziert, werden user.name und user.email in der .gitconfig des Kits geprüft und abgeglichen.",
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
                <strong key="t">
                  {tr(
                    "Avvio pigro database",
                    "Lazy database startup",
                    "Démarrage différé des bases de données",
                    "Verzögerter Datenbankstart",
                  )}
                </strong>,
                <span key="d">
                  {tr(
                    "I motori MariaDB e MongoDB non rallentano l'apertura della finestra: partono automaticamente non appena apri la pagina Database o avvii un progetto che li usa.",
                    "MariaDB and MongoDB engines do not slow down window startup: they start automatically as soon as you open the Databases page or launch a project that uses them.",
                    "Les moteurs MariaDB et MongoDB ne ralentissent pas l'ouverture de la fenêtre : ils démarrent automatiquement dès que vous ouvrez la page Bases de données ou lancez un projet qui les utilise.",
                    "Die Motoren MariaDB und MongoDB verlangsamen den Fensterstart nicht: Sie starten automatisch, sobald du die Seite Datenbanken öffnest oder ein entsprechendes Projekt startest.",
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
                <strong key="t">
                  {tr(
                    "Ripristino siti web",
                    "Website restoration",
                    "Restauration des sites web",
                    "Wiederherstellung der Websites",
                  )}
                </strong>,
                <span key="d">
                  {tr(
                    "In background riavvia tutti i siti web dei progetti che avevano l'opzione Avvio automatico attiva prima dell'ultima chiusura.",
                    "In the background restarts all project websites that had Auto-start enabled before the last shutdown.",
                    "En arrière-plan, redémarre tous les sites web des projets qui avaient l'option Démarrage automatique activée avant la dernière fermeture.",
                    "Startet im Hintergrund alle Projekt-Websites neu, bei denen vor dem letzten Beenden der automatische Start aktiviert war.",
                  )}
                </span>,
              ],
            },
          ]}
        />
      </EditorSection>

      <EditorSection
        id="guide-lifecycle-eject"
        title={tr(
          "Sequenza di chiusura sicura e «Rimuovi l'unità»",
          "Safe shutdown & “Remove drive” sequence",
          "Séquence de fermeture sécurisée et « Retirer le lecteur »",
          "Sicheres Beenden & „Laufwerk entfernen“",
        )}
      >
        <GuideShot
          src="15-rimuovi-unita.webp"
          title={tr(
            "15 · Pagina «Rimuovi l'unità» e pulizia tracce",
            "15 · “Remove drive” & trace cleanup page",
            "15 · Page « Retirer le lecteur » et nettoyage des traces",
            "15 · Seite „Laufwerk entfernen“ & Spurenbereinigung",
          )}
          caption={tr(
            "Riepilogo dei controlli effettuati prima della rimozione: arresto siti web e database, chiusura processi in data\\apps\\, salvataggio credenziali Antigravity cifrate DPAPI e rimozione delle chiavi orfane sul PC ospite.",
            "Summary of pre-eject checks: stopping websites and databases, closing processes under data\\apps\\, saving DPAPI-encrypted Antigravity credentials and removing orphan keys on the host PC.",
            "Récapitulatif des contrôles avant retrait : arrêt des sites web et bases de données, fermeture des processus dans data\\apps\\, sauvegarde des identifiants Antigravity chiffrés par DPAPI et suppression des clés orphelines sur le PC hôte.",
            "Übersicht der Prüfungen vor dem Entfernen: Stoppen von Websites und Datenbanken, Beenden aller Prozesse unter data\\apps\\, Speichern DPAPI-verschlüsselter Antigravity-Zugangsdaten und Entfernen verwaister Schlüssel auf dem Host-PC.",
          )}
          example={tr(
            "Con un solo clic su «Chiudi tutto e prepara la rimozione», il kit ferma il sito portale-clienti:8080, chiude MariaDB 11.4 e MongoDB 8.0 scaricando i buffer su disco, cifra il keyring e lascia l'unità D:\\ pronta da scollegare.",
            "With a single click on “Close everything and prepare for removal”, the kit stops portale-clienti:8080, shuts down MariaDB 11.4 and MongoDB 8.0 flushing buffers to disk, encrypts the keyring and leaves drive D:\\ ready to unplug.",
            "En un seul clic sur « Tout fermer et préparer le retrait », le kit arrête le site portale-clienti:8080, ferme MariaDB 11.4 et MongoDB 8.0 en vidant les tampons sur le disque, chiffre le trousseau et laisse le lecteur D:\\ prêt à être débranché.",
            "Mit einem einzigen Klick auf „Alles schließen und Entfernen vorbereiten“ stoppt das Kit portale-clienti:8080, fährt MariaDB 11.4 und MongoDB 8.0 inklusive Pufferspeicherung herunter, verschlüsselt den Keyring und macht Laufwerk D:\\ bereit zum Trennen.",
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
              {tr(
                "Vai a «Rimuovi l'unità»",
                "Go to “Remove drive”",
                "Aller à « Retirer le lecteur »",
                "Zu „Laufwerk entfernen“",
              )}
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
        title={tr(
          "Pipeline di sicurezza a 6 stadi per ogni download",
          "6-stage security pipeline for every download",
          "Pipeline de sécurité en 6 étapes pour chaque téléchargement",
          "6-stufige Sicherheitspipeline für jeden Download",
        )}
      >
        <p className="m-0 text-[12px] text-vsc-desc">
          {tr(
            "Atollo non include binari di terze parti pre-impacchettati e non si appoggia a mirror non ufficiali: ogni componente viene scaricato direttamente dal fornitore originario e verificato crittograficamente.",
            "Atollo does not bundle pre-packaged third-party binaries and never relies on unofficial mirrors: every component is downloaded directly from its original vendor and cryptographically verified.",
            "Atollo n'inclut aucun binaire tiers pré-empaqueté et ne dépend jamais de miroirs non officiels : chaque composant est téléchargé directement auprès du fournisseur d'origine et vérifié cryptographiquement.",
            "Atollo enthält keine vorab gepackten Drittanbieter-Binärdateien und nutzt niemals inoffizielle Mirrors: Jede Komponente wird direkt vom ursprünglichen Hersteller heruntergeladen und kryptografisch geprüft.",
          )}
        </p>
        <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
          <GuideCard
            icon={VscGlobe}
            badge="Stage 1 · HTTPS"
            title={tr(
              "1. Interrogazione della fonte ufficiale",
              "1. Querying the official vendor source",
              "1. Interrogation de la source officielle",
              "1. Abfrage der offiziellen Herstellerquelle",
            )}
          >
            {tr(
              "Il modulo src/components.rs interroga in HTTPS (TLS 1.2+) l'API o il manifesto ufficiale del fornitore (GitHub Releases API, npm Registry, nodejs.org, python-build-standalone, windows.php.net, ecc.).",
              "The src/components.rs module queries over HTTPS (TLS 1.2+) the vendor's official API or manifest (GitHub Releases API, npm Registry, nodejs.org, python-build-standalone, windows.php.net, etc.).",
              "Le module src/components.rs interroge en HTTPS (TLS 1.2+) l'API ou le manifeste officiel du fournisseur (API GitHub Releases, registre npm, nodejs.org, python-build-standalone, windows.php.net, etc.).",
              "Das Modul src/components.rs fragt per HTTPS (TLS 1.2+) die offizielle API oder das Manifest des Herstellers ab (GitHub Releases API, npm Registry, nodejs.org, python-build-standalone, windows.php.net usw.).",
            )}
          </GuideCard>

          <GuideCard
            icon={VscLock}
            badge="Stage 2 · Digest"
            title={tr(
              "2. Estrazione dell'impronta crittografica",
              "2. Cryptographic digest extraction",
              "2. Extraction de l'empreinte cryptographique",
              "2. Extraktion des kryptografischen Digests",
            )}
          >
            {tr(
              "Prima di scaricare il file binario, il kit recupera l'impronta pubblicata dal fornitore: SHA-256 (64 hex), SHA-512 (128 hex), SHA3-256 (SQLite), integrità npm sha512-<base64> o firma Authenticode.",
              "Before downloading the binary archive, the kit fetches the vendor-published digest: SHA-256 (64 hex), SHA-512 (128 hex), SHA3-256 (SQLite), npm sha512-<base64> integrity or Authenticode signature.",
              "Avant de télécharger le fichier binaire, le kit récupère l'empreinte publiée par le fournisseur : SHA-256 (64 hex), SHA-512 (128 hex), SHA3-256 (SQLite), intégrité npm sha512-<base64> ou signature Authenticode.",
              "Vor dem Herunterladen der Binärdatei ruft das Kit die vom Hersteller veröffentlichte Prüfsumme ab: SHA-256 (64 Hex), SHA-512 (128 Hex), SHA3-256 (SQLite), npm-sha512-<base64>-Integrität oder Authenticode-Signatur.",
            )}
          </GuideCard>

          <GuideCard
            icon={VscCloudDownload}
            badge="Stage 3 · Resume"
            title={tr(
              "3. Download con ripresa automatica (.part)",
              "3. Download with automatic resume (.part)",
              "3. Téléchargement avec reprise automatique (.part)",
              "3. Download mit automatischer Fortsetzung (.part)",
            )}
          >
            {tr(
              "L'archivio viene scaricato in data\\tmp\\_dl\\<nome>.part con supporto HTTP Range (206 Partial Content) fino a 4 tentativi automatici in caso di rete instabile.",
              "The archive is downloaded into data\\tmp\\_dl\\<name>.part with HTTP Range support (206 Partial Content) and up to 4 automatic retries on unstable networks.",
              "L'archive est téléchargée dans data\\tmp\\_dl\\<nom>.part avec prise en charge HTTP Range (206 Partial Content) et jusqu'à 4 tentatives automatiques en cas de réseau instable.",
              "Das Archiv wird nach data\\tmp\\_dl\\<Name>.part mit HTTP-Range-Unterstützung (206 Partial Content) und bis zu 4 automatischen Wiederholungsversuchen bei instabilem Netzwerk geladen.",
            )}
          </GuideCard>

          <GuideCard
            icon={VscShield}
            badge="Stage 4 · Hash Match"
            title={tr(
              "4. Verifica crittografica in memoria",
              "4. In-memory cryptographic verification",
              "4. Vérification cryptographique en mémoire",
              "4. Kryptografische Prüfung im Speicher",
            )}
          >
            {tr(
              "Al termine dello scaricamento, il kit ricalcola l'hash dell'intero file su disco e lo confronta byte per byte con quello atteso. Se non coincide, il file viene immediatamente eliminato e l'installazione bloccata.",
              "Once downloaded, the kit recomputes the hash of the entire file on disk and compares it byte-for-byte with the expected digest. On mismatch, the file is immediately deleted and installation aborted.",
              "Une fois le téléchargement terminé, le kit recalcule le hachage du fichier entier sur le disque et le compare octet par octet à la valeur attendue. En cas d'écart, le fichier est immédiatement supprimé et l'installation bloquée.",
              "Nach Abschluss des Downloads berechnet das Kit den Hash der gesamten Datei neu und vergleicht ihn Byte für Byte mit dem erwarteten Wert. Bei Abweichung wird die Datei sofort gelöscht und die Installation abgebrochen.",
            )}
          </GuideCard>

          <GuideCard
            icon={VscLayers}
            badge="Stage 5 · Atomic Swap"
            title={tr(
              "5. Estrazione in staging e scambio atomico",
              "5. Staging extraction & atomic swap",
              "5. Extraction en staging et échange atomique",
              "5. Entpacken im Staging-Ordner & atomarer Tausch",
            )}
          >
            {tr(
              "Gli archivi (.zip, .tar.gz, .tar.xz, .7z, .nupkg) vengono estratti in data\\apps\\_staging_<id>\\ (con 128 MB di stack dedicato per i dizionari LZMA2) e poi sostituiti atomicamente alla cartella finale.",
              "Archives (.zip, .tar.gz, .tar.xz, .7z, .nupkg) are extracted into data\\apps\\_staging_<id>\\ (with a dedicated 128 MB stack for LZMA2 dictionaries) and atomically swapped into the target folder.",
              "Les archives (.zip, .tar.gz, .tar.xz, .7z, .nupkg) sont extraites dans data\\apps\\_staging_<id>\\ (avec 128 Mo de pile dédiée pour les dictionnaires LZMA2) puis échangées atomiquement avec le dossier final.",
              "Archive (.zip, .tar.gz, .tar.xz, .7z, .nupkg) werden in data\\apps\\_staging_<id>\\ entpackt (mit 128 MB dediziertem Stack für LZMA2-Wörterbücher) und anschließend atomar in den Zielordner verschoben.",
            )}
          </GuideCard>

          <GuideCard
            icon={VscCheck}
            badge="Stage 6 · Check & State"
            title={tr(
              "6. Post-configurazione, test e registro",
              "6. Post-setup, smoke test & state record",
              "6. Post-configuration, test et registre d'état",
              "6. Nachkonfiguration, Funktionstest & Statusdatei",
            )}
          >
            {tr(
              "Vengono generati i file di supporto (es. php.ini con curl.cainfo, .portable, wrapper .cmd), viene eseguito il comando check e la versione installata viene salvata in data\\app\\state.json.",
              "Support files (e.g. php.ini with curl.cainfo, .portable, .cmd wrappers) are generated, the check command is executed, and the installed version is saved in data\\app\\state.json.",
              "Les fichiers de support (ex. php.ini avec curl.cainfo, .portable, wrappers .cmd) sont générés, la commande check est exécutée et la version installée est enregistrée dans data\\app\\state.json.",
              "Unterstützungsdateien (z. B. php.ini mit curl.cainfo, .portable, .cmd-Wrapper) werden erzeugt, der check-Befehl ausgeführt und die installierte Version in data\\app\\state.json gespeichert.",
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
              {tr(
                "Consulta la tabella completa delle 36 fonti",
                "View the complete table of all 36 sources",
                "Consulter le tableau complet des 36 sources",
                "Vollständige Tabelle aller 36 Quellen ansehen",
              )}
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
    { id: "all", label: `${tr("Tutti", "All", "Tous", "Alle")} (${String(TOOL_SOURCES.length)})` },
    { id: "agents", label: tr("Agenti IA", "AI Agents", "Agents IA", "KI-Agenten") },
    { id: "languages", label: tr("Linguaggi", "Languages", "Langages", "Sprachen") },
    { id: "databases", label: tr("Database", "Databases", "Bases de données", "Datenbanken") },
    { id: "tools", label: tr("Strumenti", "Tools", "Outils", "Werkzeuge") },
    { id: "part", label: tr("Sotto-parti & SDK", "Sub-parts & SDKs", "Sous-parties et SDK", "Teilkomponenten & SDKs") },
    { id: "app", label: tr("App & Editor", "App & Editor", "App et Éditeur", "App & Editor") },
  ];

  return (
    <>
      <EditorSection
        id="guide-sources-table"
        title={tr(
          "Catalogo completo delle fonti esterne (36 componenti e sotto-parti)",
          "Complete catalog of external sources (36 components & sub-parts)",
          "Catalogue complet des sources externes (36 composants et sous-parties)",
          "Vollständiger Katalog externer Quellen (36 Komponenten & Teilpakete)",
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
              {tr("Apri pagina Strumenti", "Open Tools page", "Ouvrir la page Outils", "Seite Werkzeuge öffnen")}
            </span>
          </Button>
        }
      >
        <p className="m-0 text-[12px] text-vsc-desc">
          {tr(
            "Elenco minuzioso di tutti i link da cui Atollo attinge gli strumenti, il metodo di verifica crittografica applicato, la cartella di destinazione in data\\apps\\ e lo stato attuale su questa unità.",
            "Detailed list of every URL from which Atollo fetches tools, the cryptographic verification method applied, the destination folder in data\\apps\\ and the current status on this drive.",
            "Liste détaillée de tous les liens depuis lesquels Atollo télécharge les outils, la méthode de vérification cryptographique appliquée, le dossier de destination dans data\\apps\\ et l'état actuel sur ce lecteur.",
            "Detaillierte Liste aller URLs, von denen Atollo Werkzeuge bezieht, die angewandte kryptografische Prüfmethode, der Zielordner in data\\apps\\ sowie der aktuelle Status auf diesem Laufwerk.",
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
              placeholder={tr(
                "Filtra per nome, URL, hash, percorso...",
                "Filter by name, URL, hash, path...",
                "Filtrer par nom, URL, hachage, chemin...",
                "Nach Name, URL, Hash, Pfad filtern...",
              )}
            />
          </div>
        </div>

        <SimpleTable
          columns={[
            tr("Componente e Stato", "Component & Status", "Composant et État", "Komponente & Status"),
            tr(
              "Endpoint metadati / versione",
              "Metadata / version endpoint",
              "Point de terminaison métadonnées / version",
              "Metadaten- / Versions-Endpunkt",
            ),
            tr("URL binario / archivio", "Binary / archive URL", "URL binaire / archive", "Binär- / Archiv-URL"),
            tr("Verifica integrità", "Integrity check", "Vérification d'intégrité", "Integritätsprüfung"),
            tr("Percorso in data\\apps\\", "Path in data\\apps\\", "Chemin dans data\\apps\\", "Pfad in data\\apps\\"),
          ]}
          rows={filtered.map((row) => {
            const live = compMap.get(row.id);
            const installedVer = live?.installed
              ? (live.version ?? tr("Installato", "Installed", "Installé", "Installiert"))
              : null;
            return {
              key: row.id,
              cells: [
                <div key="name" className="space-y-1">
                  <div className="flex items-center gap-2">
                    <ComponentLogo id={row.logoId} size={16} />
                    <span className="font-semibold text-vsc-settings-header">{row.name}</span>
                  </div>
                  <div className="flex flex-wrap items-center gap-1.5 pl-6">
                    <code className="font-mono text-[11px] text-vsc-desc">{row.id}</code>
                    {installedVer ? (
                      <Tag tone="focus">{installedVer}</Tag>
                    ) : live ? (
                      <Tag>{tr("Non installato", "Not installed", "Non installé", "Nicht installiert")}</Tag>
                    ) : (
                      <Tag>{tr("Automatico", "Automatic", "Automatique", "Automatisch")}</Tag>
                    )}
                    <button
                      type="button"
                      onClick={() => {
                        void urlOpen(row.home);
                      }}
                      title={row.home}
                      className="inline-flex cursor-pointer items-center gap-0.5 border-0 bg-transparent p-0 text-[11px] text-vsc-link underline hover:opacity-80"
                    >
                      <span>{tr("Sito", "Site", "Site", "Website")}</span>
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
          "Services externes supplémentaires interrogés par l'application",
          "Weitere von der Anwendung abgefragte externe Dienste",
        )}
      >
        <p className="m-0 text-[12px] text-vsc-desc">
          {tr(
            "Oltre ai repository di download dei binari, Atollo comunica esclusivamente con i seguenti endpoint per quote IA, autenticazione GitHub e galleria estensioni dell'editor.",
            "Beyond binary download repositories, Atollo communicates exclusively with the following endpoints for AI quotas, GitHub authentication and the editor extension gallery.",
            "Au-delà des dépôts de téléchargement de binaires, Atollo communique exclusivement avec les points de terminaison suivants pour les quotas IA, l'authentification GitHub et la galerie d'extensions de l'éditeur.",
            "Neben den Binär-Download-Repositorys kommuniziert Atollo ausschließlich mit den folgenden Endpunkten für KI-Quoten, GitHub-Authentifizierung und die Erweiterungsgalerie des Editors.",
          )}
        </p>
        <SimpleTable
          columns={[
            tr("Servizio", "Service", "Service", "Dienst"),
            tr("Endpoint HTTPS", "HTTPS Endpoint", "Point de terminaison HTTPS", "HTTPS-Endpunkt"),
            tr("Scopo e funzionamento", "Purpose & behavior", "Objectif et fonctionnement", "Zweck & Funktionsweise"),
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
                    "Utilisé par gh.exe pour la connexion Device Flow, la lecture du profil et des e-mails (/user/emails), la création de dépôts depuis des modèles et l'augmentation de la limite de 60 à 5 000 requêtes/heure.",
                    "Verwendet von gh.exe für das Device-Flow-Login, das Abrufen von Profil und E-Mails (/user/emails), das Erstellen von Repositorys aus Vorlagen und das Anheben des Rate-Limits von 60 auf 5.000 Anfragen/Stunde.",
                  )}
                </span>,
              ],
            },
            {
              key: "claude-quota",
              cells: [
                <strong key="s">
                  {tr(
                    "Quota Claude Code (Anthropic)",
                    "Claude Code Quota (Anthropic)",
                    "Quota Claude Code (Anthropic)",
                    "Claude-Code-Quote (Anthropic)",
                  )}
                </strong>,
                <code key="e" className="font-mono text-[11px] select-text">
                  https://api.anthropic.com/api/oauth/usage · /api/oauth/profile
                </code>,
                <span key="d">
                  {tr(
                    "Interrogato dalla pagina Agenti usando il token OAuth locale (data\\claude\\.credentials.json) per mostrare percentuali di utilizzo a 5 ore, 7 giorni e crediti extra.",
                    "Queried by the Agents page using the local OAuth token (data\\claude\\.credentials.json) to display 5-hour, 7-day and extra-usage percentages.",
                    "Interrogé par la page Agents à l'aide du jeton OAuth local (data\\claude\\.credentials.json) pour afficher les pourcentages d'utilisation sur 5 heures, 7 jours et les crédits supplémentaires.",
                    "Abgefragt von der Seite Agenten mit dem lokalen OAuth-Token (data\\claude\\.credentials.json) zur Anzeige der 5-Stunden-, 7-Tage- und Zusatz-Auslastung.",
                  )}
                </span>,
              ],
            },
            {
              key: "codex-quota",
              cells: [
                <strong key="s">
                  {tr("Quota Codex (OpenAI)", "Codex Quota (OpenAI)", "Quota Codex (OpenAI)", "Codex-Quote (OpenAI)")}
                </strong>,
                <code key="e" className="font-mono text-[11px] select-text">
                  https://chatgpt.com/backend-api/wham/usage
                </code>,
                <span key="d">
                  {tr(
                    "Interrogato dalla pagina Agenti usando il token in data\\codex\\auth.json per mostrare la finestra primaria, settimanale e il saldo crediti.",
                    "Queried by the Agents page using the token in data\\codex\\auth.json to display primary window, weekly window and credit balance.",
                    "Interrogé par la page Agents avec le jeton de data\\codex\\auth.json pour afficher la fenêtre principale, hebdomadaire et le solde de crédits.",
                    "Abgefragt von der Seite Agenten mit dem Token in data\\codex\\auth.json zur Anzeige des primären und wöchentlichen Zeitfensters sowie des Guthabens.",
                  )}
                </span>,
              ],
            },
            {
              key: "agy-quota",
              cells: [
                <strong key="s">
                  {tr(
                    "Quota Antigravity (Google)",
                    "Antigravity Quota (Google)",
                    "Quota Antigravity (Google)",
                    "Antigravity-Quote (Google)",
                  )}
                </strong>,
                <code key="e" className="font-mono text-[11px] select-text">
                  https://127.0.0.1:&lt;porta&gt;/exa.language_server_pb.LanguageServerService/GetUserStatus
                </code>,
                <span key="d">
                  {tr(
                    "Interrogato in locale quando Antigravity è aperto, estraendo --csrf_token dal processo language_server_windows_x64.exe per leggere le quote per singolo modello IA.",
                    "Queried locally while Antigravity is running, extracting --csrf_token from language_server_windows_x64.exe to read per-model AI quotas.",
                    "Interrogé localement lorsque Antigravity est ouvert, en extrayant --csrf_token du processus language_server_windows_x64.exe pour lire les quotas par modèle IA.",
                    "Lokal abgefragt, solange Antigravity läuft, wobei --csrf_token aus language_server_windows_x64.exe gelesen wird, um die Quoten pro KI-Modell anzuzeigen.",
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
                    "Marketplace open-source préconfiguré dans VSCodium Reh Web pour installer les extensions de l'éditeur directement dans data\\vscodium-server\\data\\extensions\\.",
                    "In VSCodium Reh Web vorkonfigurierter Open-Source-Marketplace zur direkten Installation von Editor-Erweiterungen unter data\\vscodium-server\\data\\extensions\\.",
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
          "Perché l'account GitHub è il pilastro dell'ecosistema Atollo",
          "Why a GitHub account is the cornerstone of the Atollo ecosystem",
          "Pourquoi le compte GitHub est le pilier de l'écosystème Atollo",
          "Warum das GitHub-Konto der Grundpfeiler des Atollo-Ökosystems ist",
        )}
      >
        <p className="m-0 text-[12px] text-vsc-desc">
          {tr(
            "Anche se puoi esplorare l'interfaccia senza login, collegare un account GitHub (gratuito) sblocca 5 meccanismi tecnici fondamentali che altrimenti sarebbero limitati o indisponibili.",
            "While you can browse the UI without logging in, connecting a (free) GitHub account unlocks 5 core technical mechanisms that would otherwise be rate-limited or unavailable.",
            "Même si vous pouvez explorer l'interface sans connexion, lier un compte GitHub (gratuit) débloque 5 mécanismes techniques fondamentaux qui seraient autrement limités ou indisponibles.",
            "Auch wenn die Oberfläche ohne Login erkundet werden kann, schaltet die Verknüpfung eines (kostenlosen) GitHub-Kontos 5 zentrale technische Mechanismen frei, die sonst begrenzt oder nicht verfügbar wären.",
          )}
        </p>
        <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
          <GuideCard
            icon={VscPulse}
            badge="5.000 req/ora"
            title={tr(
              "1. Sblocco limiti API GitHub per scaricare gli strumenti",
              "1. Unlocks GitHub API rate limits for tool downloads",
              "1. Déblocage des limites de l'API GitHub pour télécharger les outils",
              "1. Aufhebung des GitHub-API-Limits für Werkzeug-Downloads",
            )}
          >
            {tr(
              "Oltre metà del catalogo (Git, uv, Ripgrep, fd, Bat, jq, Pandoc, Typst, CMake, Ninja, Rust Analyzer, Bun, Ruby, ecc.) risolve le versioni tramite api.github.com. Senza login GitHub applica un limite di 60 richieste/ora per IP (HTTP 403 Rate limit exceeded); con l'account collegato il kit inietta automaticamente Authorization: Bearer <token> portando la soglia a 5.000 richieste/ora.",
              "Over half the catalog resolves versions via api.github.com. Without login, GitHub enforces a 60 req/hour limit per IP (HTTP 403 Rate limit exceeded); with a linked account, the kit automatically injects Authorization: Bearer <token>, raising the limit to 5,000 requests/hour.",
              "Plus de la moitié du catalogue résout les versions via api.github.com. Sans connexion, GitHub applique une limite de 60 requêtes/heure par IP (HTTP 403 Rate limit exceeded) ; avec un compte lié, le kit injecte automatiquement Authorization: Bearer <token>, portant le seuil à 5 000 requêtes/heure.",
              "Über die Hälfte des Katalogs ermittelt Versionen über api.github.com. Ohne Login gilt bei GitHub ein Limit von 60 Anfragen/Stunde pro IP (HTTP 403 Rate limit exceeded); mit verknüpftem Konto fügt das Kit automatisch Authorization: Bearer <token> hinzu und erhöht das Limit auf 5.000 Anfragen/Stunde.",
            )}
          </GuideCard>

          <GuideCard
            icon={VscWand}
            badge="Template"
            title={tr(
              "2. Creazione di nuovi progetti da Template GitHub",
              "2. Creating new projects from GitHub Templates",
              "2. Création de nouveaux projets depuis des modèles GitHub",
              "2. Erstellung neuer Projekte aus GitHub-Vorlagen",
            )}
          >
            {tr(
              "La procedura Nuovo Progetto clona il repository modello scelto (es. starter-kit), crea un nuovo repository privato o pubblico sotto il tuo account GitHub tramite gh repo create --template e configura automaticamente il remote origin.",
              "The New Project wizard clones the chosen template repository (e.g. starter-kit), creates a new private or public repository under your GitHub account via gh repo create --template, and automatically configures the origin remote.",
              "L'assistant Nouveau projet clone le dépôt modèle choisi (ex. starter-kit), crée un nouveau dépôt privé ou public sous votre compte GitHub via gh repo create --template et configure automatiquement le remote origin.",
              "Der Assistent Neues Projekt klont das gewählte Vorlagen-Repository (z. B. starter-kit), erstellt über gh repo create --template ein neues privates oder öffentliches Repository unter deinem GitHub-Konto und richtet automatisch das Remote origin ein.",
            )}
          </GuideCard>

          <GuideCard
            icon={VscKey}
            badge=".gitconfig"
            title={tr(
              "3. Configurazione automatica dell'identità Git nei commit",
              "3. Automatic Git commit authorship configuration",
              "3. Configuration automatique de l'identité Git dans les commits",
              "3. Automatische Einrichtung der Git-Commit-Identität",
            )}
          >
            {tr(
              "Al login il kit interroga /user e /user/emails e popola automaticamente user.name, user.email (inclusa l'email @users.noreply.github.com se l'email primaria è privata) e credential.helper = !gh auth git-credential in data\\home\\.gitconfig.",
              "Upon login, the kit queries /user and /user/emails and automatically populates user.name, user.email (including @users.noreply.github.com if primary email is private) and credential.helper = !gh auth git-credential in data\\home\\.gitconfig.",
              "À la connexion, le kit interroge /user et /user/emails et remplit automatiquement user.name, user.email (y compris l'adresse @users.noreply.github.com si l'e-mail principal est privé) et credential.helper = !gh auth git-credential dans data\\home\\.gitconfig.",
              "Beim Login fragt das Kit /user und /user/emails ab und trägt automatisch user.name, user.email (einschließlich @users.noreply.github.com bei privater Hauptadresse) sowie credential.helper = !gh auth git-credential in data\\home\\.gitconfig ein.",
            )}
          </GuideCard>

          <GuideCard
            icon={VscSync}
            badge="Push / Pull"
            title={tr(
              "4. Sincronizzazione e backup cloud a 1 clic dei progetti",
              "4. 1-click cloud synchronization & backup for projects",
              "4. Synchronisation et sauvegarde cloud en 1 clic des projets",
              "4. 1-Klick-Cloud-Synchronisation & Backup für Projekte",
            )}
          >
            {tr(
              "Nella scheda Git di ogni progetto i pulsanti «Pubblica su GitHub», «Salva e invia modifiche» e «Scarica aggiornamenti» eseguono commit, pull --rebase e push HTTPS senza mai chiedere password o chiavi SSH manuali.",
              "In each project's Git tab, the 'Publish to GitHub', 'Commit & Push' and 'Pull updates' buttons run commit, pull --rebase and HTTPS push without ever asking for manual passwords or SSH keys.",
              "Dans l'onglet Git de chaque projet, les boutons « Publier sur GitHub », « Enregistrer et envoyer » et « Télécharger les mises à jour » exécutent commit, pull --rebase et push HTTPS sans jamais demander de mot de passe ou de clé SSH manuelle.",
              "Auf der Registerkarte Git jedes Projekts führen die Schaltflächen „Auf GitHub veröffentlichen“, „Speichern und senden“ sowie „Aktualisierungen abrufen“ Commit, pull --rebase und HTTPS-Push aus, ohne nach Passwörtern oder manuellen SSH-Schlüsseln zu fragen.",
            )}
          </GuideCard>
        </div>

        <GuideCard
          icon={VscLock}
          badge="data\gh\hosts.yml"
          title={tr(
            "5. Autenticazione HTTPS trasparente per terminali, editor e agenti IA",
            "5. Transparent HTTPS authentication for terminals, editor & AI agents",
            "5. Authentification HTTPS transparente pour terminaux, éditeur et agents IA",
            "5. Transparente HTTPS-Authentifizierung für Terminals, Editor & KI-Agenten",
          )}
          actionLabel={tr(
            "Vedi come collegare l'account",
            "See how to connect your account",
            "Voir comment lier le compte",
            "So verknüpfst du dein Konto",
          )}
          onAction={() => {
            onSelectTopic("github-setup");
          }}
        >
          {tr(
            "Poiché il token è salvato in <Radice>\\data\\gh\\hosts.yml (tramite GH_CONFIG_DIR), spostando l'unità USB su un nuovo PC non dovrai mai rifare il login: qualsiasi git push o gh pr create lanciato da te o dagli agenti IA sarà già autenticato.",
            "Because the token is stored in <Root>\\data\\gh\\hosts.yml (via GH_CONFIG_DIR), moving your USB drive to a new PC never requires logging in again: any git push or gh pr create run by you or by AI agents is already authenticated.",
            "Comme le jeton est enregistré dans <Racine>\\data\\gh\\hosts.yml (via GH_CONFIG_DIR), déplacer la clé USB sur un nouveau PC ne nécessite jamais de se reconnecter : tout git push ou gh pr create lancé par vous ou par les agents IA est déjà authentifié.",
            "Da das Token in <Stamm>\\data\\gh\\hosts.yml (über GH_CONFIG_DIR) gespeichert ist, musst du dich beim Wechsel des USB-Laufwerks an einen neuen PC nie erneut anmelden: Jeder git push oder gh pr create von dir oder den KI-Agenten ist bereits authentifiziert.",
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
          "Comment lier votre compte GitHub (OAuth Device Flow)",
          "So verknüpfst du dein GitHub-Konto (OAuth Device Flow)",
        )}
      >
        <GuideRecipe
          number="GitHub SSO"
          title={tr(
            "Autenticazione con codice dispositivo a 8 caratteri",
            "Authentication with 8-character device code",
            "Authentification avec code d'appareil à 8 caractères",
            "Authentifizierung mit 8-stelligem Gerätecode",
          )}
          whereUi={tr(
            "Wizard iniziale oppure Impostazioni → Account GitHub",
            "Initial Wizard or Settings → GitHub Account",
            "Assistant initial ou Paramètres → Compte GitHub",
            "Start-Assistent oder Einstellungen → GitHub-Konto",
          )}
          whereFile="data\gh\hosts.yml · data\home\.gitconfig"
          targetPage="settings"
          targetLabel={tr(
            "Apri Impostazioni → Account GitHub",
            "Open Settings → GitHub Account",
            "Ouvrir Paramètres → Compte GitHub",
            "Einstellungen → GitHub-Konto öffnen",
          )}
          onNavigate={onNavigate}
          steps={[
            tr(
              "Se non possiedi ancora un account, clicca su «Crea un account gratuito su github.com» e registrati con il tuo indirizzo email.",
              "If you don't have an account yet, click 'Create a free account on github.com' and sign up with your email.",
              "Si vous n'avez pas encore de compte, cliquez sur « Créer un compte gratuit sur github.com » et inscrivez-vous avec votre adresse e-mail.",
              "Wenn du noch kein Konto hast, klicke auf „Kostenloses Konto auf github.com erstellen“ und registriere dich mit deiner E-Mail-Adresse.",
            ),
            tr(
              "Nell'applicazione clicca sul pulsante «Collega account GitHub».",
              "In the application, click the 'Connect GitHub account' button.",
              "Dans l'application, cliquez sur le bouton « Lier le compte GitHub ».",
              "Klicke in der Anwendung auf die Schaltfläche „GitHub-Konto verknüpfen“.",
            ),
            tr(
              "Atollo avvia in background gh auth login --hostname github.com --git-protocol https --web --scopes repo,read:org,user:email,workflow.",
              "Atollo starts gh auth login --hostname github.com --git-protocol https --web --scopes repo,read:org,user:email,workflow in the background.",
              "Atollo lance en arrière-plan gh auth login --hostname github.com --git-protocol https --web --scopes repo,read:org,user:email,workflow.",
              "Atollo startet im Hintergrund gh auth login --hostname github.com --git-protocol https --web --scopes repo,read:org,user:email,workflow.",
            ),
            tr(
              "Sullo schermo compare un codice univoco a 8 caratteri (es. WDJB-4829), già copiato automaticamente negli appunti, e si apre https://github.com/login/device nel browser.",
              "A unique 8-character code (e.g. WDJB-4829) appears on screen, automatically copied to your clipboard, and https://github.com/login/device opens in your browser.",
              "Un code unique à 8 caractères (ex. WDJB-4829) apparaît à l'écran, déjà copié automatiquement dans le presse-papiers, et https://github.com/login/device s'ouvre dans le navigateur.",
              "Auf dem Bildschirm erscheint ein eindeutiger 8-stelliger Code (z. B. WDJB-4829), der automatisch in die Zwischenablage kopiert wird, und https://github.com/login/device öffnet sich im Browser.",
            ),
            tr(
              "Incolla il codice nella pagina di GitHub e premi Authorize GitHub CLI: la finestra di Atollo rileverà il completamento in tempo reale mostrando il tuo nome utente (@utente).",
              "Paste the code into the GitHub page and click Authorize GitHub CLI: Atollo will detect completion in real time and display your username (@user).",
              "Collez le code sur la page GitHub et cliquez sur Authorize GitHub CLI : la fenêtre Atollo détectera la validation en temps réel et affichera votre nom d'utilisateur (@utilisateur).",
              "Füge den Code auf der GitHub-Seite ein und klicke auf Authorize GitHub CLI: Atollo erkennt den Abschluss in Echtzeit und zeigt deinen Benutzernamen (@benutzer) an.",
            ),
          ]}
          purpose={tr(
            "Autorizza i permessi repo (creazione/push repository pubblici e privati), read:org (lettura template aziendali), user:email (configurazione email autore commit) e workflow (aggiornamento GitHub Actions).",
            "Authorizes the scopes repo (create/push public and private repositories), read:org (read organization templates), user:email (configure commit author email) and workflow (update GitHub Actions).",
            "Autorise les permissions repo (création/push de dépôts publics et privés), read:org (lecture des modèles d'organisation), user:email (configuration de l'e-mail d'auteur des commits) et workflow (mise à jour des GitHub Actions).",
            "Autorisiert die Berechtigungen repo (Erstellen/Pushen öffentlicher und privater Repositorys), read:org (Lesen von Organisations-Vorlagen), user:email (E-Mail-Adresse für Commits) und workflow (Aktualisieren von GitHub Actions).",
          )}
        />

        <GuideCode
          title={tr(
            "File .gitconfig generato e mantenuto automaticamente dal kit",
            ".gitconfig file automatically generated and maintained by the kit",
            "Fichier .gitconfig généré et maintenu automatiquement par le kit",
            "Vom Kit automatisch erzeugte und gepflegte Datei .gitconfig",
          )}
          path="<Radice>\data\home\.gitconfig"
          code={`[init]
    defaultBranch = main
[core]
    autocrlf = true
    longpaths = true
    editor = D:/Atollo/data/kit/bin/editor.cmd
[safe]
    directory = *
[user]
    name = Mario Rossi
    email = mario.rossi@example.com
[credential "https://github.com"]
    helper = 
    helper = !gh auth git-credential
[credential "https://gist.github.com"]
    helper = 
    helper = !gh auth git-credential`}
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
        title={tr(
          "Struttura dell'interfaccia (VSCodium Workbench)",
          "Interface structure (VSCodium Workbench)",
          "Structure de l'interface (VSCodium Workbench)",
          "Aufbau der Oberfläche (VSCodium Workbench)",
        )}
      >
        <SimpleTable
          columns={[
            tr("Area dell'interfaccia", "Interface area", "Zone de l'interface", "Oberflächenbereich"),
            tr("Elementi e scorciatoie", "Elements & shortcuts", "Éléments et raccourcis", "Elemente & Kurzbefehle"),
          ]}
          rows={[
            {
              key: "activity",
              cells: [
                <strong key="a">
                  {tr(
                    "Barra delle attività (a sinistra)",
                    "Activity Bar (left)",
                    "Barre d'activité (à gauche)",
                    "Aktivitätsleiste (links)",
                  )}
                </strong>,
                <span key="d">
                  {tr(
                    "In alto le 6 viste operative: Nuovo progetto, Progetti (con badge conteggio), Agenti, Database, Server Web, Strumenti. In basso le 3 viste di sistema: Guida (questa vista), Impostazioni, Rimuovi l'unità.",
                    "Top 6 operational views: New project, Projects (with count badge), Agents, Databases, Web Servers, Tools. Bottom 3 system views: Guide (this view), Settings, Remove drive.",
                    "En haut les 6 vues opérationnelles : Nouveau projet, Projets (avec badge de comptage), Agents, Bases de données, Serveurs Web, Outils. En bas les 3 vues système : Guide (cette vue), Paramètres, Retirer le lecteur.",
                    "Oben die 6 Arbeitsansichten: Neues Projekt, Projekte (mit Zähler), Agenten, Datenbanken, Webserver, Werkzeuge. Unten die 3 Systemansichten: Anleitung (diese Ansicht), Einstellungen, Laufwerk entfernen.",
                  )}
                </span>,
              ],
            },
            {
              key: "sidebar",
              cells: [
                <strong key="a">
                  {tr(
                    "Barra laterale contestuale",
                    "Contextual Side Bar",
                    "Barre latérale contextuelle",
                    "Kontextabhängige Seitenleiste",
                  )}
                </strong>,
                <span key="d">
                  {tr(
                    "Affianca la barra delle attività nelle viste con elenchi (Progetti, Database, Strumenti, Guida) offrendo casella di ricerca e sezioni espandibili.",
                    "Sits next to the Activity Bar in list-based views (Projects, Databases, Tools, Guide) providing a search box and collapsible sections.",
                    "Accompagne la barre d'activité dans les vues à listes (Projets, Bases de données, Outils, Guide) avec champ de recherche et sections repliables.",
                    "Begleitet die Aktivitätsleiste in Listenansichten (Projekte, Datenbanken, Werkzeuge, Anleitung) mit Suchfeld und einklappbaren Abschnitten.",
                  )}
                </span>,
              ],
            },
            {
              key: "editor",
              cells: [
                <strong key="a">
                  {tr(
                    "Area principale di lavoro",
                    "Main Editor Area",
                    "Zone principale de travail",
                    "Hauptarbeitsbereich",
                  )}
                </strong>,
                <span key="d">
                  {tr(
                    "Ospita l'intestazione della risorsa selezionata, la barra delle schede (Tab) o i filtri e le sezioni operative.",
                    "Hosts the header of the selected resource, the tab bar or filters, and the operational sections.",
                    "Accueille l'en-tête de la ressource sélectionnée, la barre d'onglets ou les filtres ainsi que les sections opérationnelles.",
                    "Enthält die Kopfzeile der gewählten Ressource, die Registerkartenleiste oder Filter sowie die Arbeitsabschnitte.",
                  )}
                </span>,
              ],
            },
            {
              key: "status",
              cells: [
                <strong key="a">
                  {tr(
                    "Barra di stato (in basso)",
                    "Status Bar (bottom)",
                    "Barre d'état (en bas)",
                    "Statusleiste (unten)",
                  )}
                </strong>,
                <span key="d">
                  {tr(
                    "Mostra lo stato del kit, l'editor VSCodium, le installazioni in corso e il pulsante per aprire il pannello Attività.",
                    "Shows kit status, VSCodium editor status, running installations and the button to toggle the Activity panel.",
                    "Affiche l'état du kit, l'éditeur VSCodium, les installations en cours et le bouton pour ouvrir le panneau Activité.",
                    "Zeigt den Kit-Status, den VSCodium-Editor, laufende Installationen und die Schaltfläche für den Aktivitätsbereich.",
                  )}
                </span>,
              ],
            },
          ]}
        />
      </EditorSection>

      <EditorSection
        id="guide-feat-wizard"
        title={tr(
          "Procedura guidata di primo avvio (Wizard)",
          "First-Run Setup Wizard",
          "Assistant de premier démarrage (Wizard)",
          "Einrichtungsassistent beim ersten Start (Wizard)",
        )}
      >
        <GuideShot
          src="01-wizard-benvenuto.webp"
          title={tr(
            "01 · Wizard di primo avvio (5 passaggi)",
            "01 · First-Run Setup Wizard (5 steps)",
            "01 · Assistant de premier démarrage (5 étapes)",
            "01 · Einrichtungsassistent beim ersten Start (5 Schritte)",
          )}
          caption={tr(
            "Passaggi del Wizard: 1. Benvenuto e verifica spazio su disco · 2. Scelta componenti e linee di versione · 3. Collegamento account GitHub · 4. Installazione e verifica crittografica · 5. Riepilogo finale.",
            "Wizard steps: 1. Welcome & disk space check · 2. Component & version line selection · 3. GitHub account connection · 4. Download & cryptographic verification · 5. Final summary.",
            "Étapes de l'Assistant : 1. Bienvenue et vérification de l'espace disque · 2. Choix des composants et lignes de version · 3. Liaison du compte GitHub · 4. Installation et vérification cryptographique · 5. Récapitulatif final.",
            "Schritte des Assistenten: 1. Willkommen & Speicherplatzprüfung · 2. Auswahl von Komponenten & Versionslinien · 3. GitHub-Kontoverknüpfung · 4. Installation & kryptografische Prüfung · 5. Abschlussübersicht.",
          )}
          example={tr(
            "L'unità esterna D:\\Atollo viene rilevata con 384 GB liberi (semaforo verde «Spazio disponibile più che sufficiente»).",
            "External drive D:\\Atollo is detected with 384 GB free (green badge 'More than enough free space').",
            "Le lecteur externe D:\\Atollo est détecté avec 384 Go libres (badge vert « Espace disponible largement suffisant »).",
            "Das externe Laufwerk D:\\Atollo wird mit 384 GB freiem Speicher erkannt (grüne Anzeige „Mehr als genug freier Speicherplatz“).",
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
              {tr("Apri Impostazioni", "Open Settings", "Ouvrir Paramètres", "Einstellungen öffnen")}
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
        title={tr(
          "Creazione di un Nuovo Progetto",
          "Creating a New Project",
          "Création d'un Nouveau Projet",
          "Neues Projekt erstellen",
        )}
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
              {tr("Apri Nuovo progetto", "Open New project", "Ouvrir Nouveau projet", "Neues Projekt öffnen")}
            </span>
          </Button>
        }
      >
        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
          <GuideShot
            src="03-nuovo-progetto-template.webp"
            title={tr(
              "03 · Scelta del template e del nome progetto",
              "03 · Choosing template and project name",
              "03 · Choix du modèle et du nom du projet",
              "03 · Vorlage und Projektnamen wählen",
            )}
            caption={tr(
              "Selezione del modello starter-kit e validazione in tempo reale del nome della cartella.",
              "Selecting the starter-kit template and real-time folder name validation.",
              "Sélection du modèle starter-kit et validation en temps réel du nom de dossier.",
              "Auswahl der Vorlage starter-kit und Echtzeit-Validierung des Ordnernamens.",
            )}
            example={tr(
              "Digitando gestionale-fatture, il kit mostra l'anteprima del percorso finale D:\\Atollo\\projects\\gestionale-fatture.",
              "Typing gestionale-fatture previews the target path D:\\Atollo\\projects\\gestionale-fatture.",
              "En saisissant gestionale-fatture, le kit affiche l'aperçu du chemin final D:\\Atollo\\projects\\gestionale-fatture.",
              "Bei Eingabe von gestionale-fatture zeigt das Kit den Zielpfad D:\\Atollo\\projects\\gestionale-fatture in der Vorschau an.",
            )}
          />
          <GuideShot
            src="04-nuovo-progetto-github.webp"
            title={tr(
              "04 · Opzione Crea anche su GitHub",
              "04 · Also create on GitHub option",
              "04 · Option Créer aussi sur GitHub",
              "04 · Option Auch auf GitHub erstellen",
            )}
            caption={tr(
              "Attivando Crea anche su GitHub puoi scegliere visibilità Privato o Pubblico per creare automaticamente il repository remoto.",
              "Enabling Also create on GitHub lets you choose Private or Public visibility to automatically create the remote repository.",
              "En activant Créer aussi sur GitHub, vous pouvez choisir la visibilité Privé ou Public pour créer automatiquement le dépôt distant.",
              "Wenn Auch auf GitHub erstellen aktiviert ist, kannst du Privat oder Öffentlich wählen, um das Remote-Repository automatisch anzulegen.",
            )}
            example={tr(
              "Il repository remoto mario-rossi/gestionale-fatture viene creato come Privato tramite gh repo create --template.",
              "Remote repository mario-rossi/gestionale-fatture is created as Private via gh repo create --template.",
              "Le dépôt distant mario-rossi/gestionale-fatture est créé en Privé via gh repo create --template.",
              "Das Remote-Repository mario-rossi/gestionale-fatture wird über gh repo create --template als Privat erstellt.",
            )}
          />
        </div>
      </EditorSection>

      <EditorSection
        id="guide-feat-projects"
        title={tr(
          "Pagina Progetti e le schede operative del progetto",
          "Projects page & project sections",
          "Page Projets et sections opérationnelles du projet",
          "Seite Projekte & Arbeitsabschnitte des Projekts",
        )}
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
              {tr("Apri Progetti", "Open Projects", "Ouvrir Projets", "Projekte öffnen")}
            </span>
          </Button>
        }
      >
        <GuideShot
          src="02-progetti-elenco.webp"
          title={tr(
            "02 · Elenco progetti e stato vuoto",
            "02 · Projects list & empty state",
            "02 · Liste des projets et état vide",
            "02 · Projektliste & leerer Zustand",
          )}
          caption={tr(
            "La barra laterale elenca tutte le sottocartelle di projects\\ con filtro istantaneo e indicatori dei linguaggi rilevati.",
            "The sidebar lists all subfolders under projects\\ with instant filtering and detected language tags.",
            "La barre latérale liste tous les sous-dossiers de projects\\ avec filtrage instantané et badges des langages détectés.",
            "Die Seitenleiste listet alle Unterordner von projects\\ mit Sofortfilter und Tags der erkannten Sprachen auf.",
          )}
          example={tr(
            "Tre progetti reali in D:\\Atollo\\projects\\: api-magazzino (Python + FastAPI), gestionale-fatture (Node.js + Next.js) e portale-clienti (PHP 8.4 + Node 22 + MariaDB + MongoDB).",
            "Three real projects in D:\\Atollo\\projects\\: api-magazzino (Python + FastAPI), gestionale-fatture (Node.js + Next.js) and portale-clienti (PHP 8.4 + Node 22 + MariaDB + MongoDB).",
            "Trois projets réels dans D:\\Atollo\\projects\\ : api-magazzino (Python + FastAPI), gestionale-fatture (Node.js + Next.js) et portale-clienti (PHP 8.4 + Node 22 + MariaDB + MongoDB).",
            "Drei reale Projekte in D:\\Atollo\\projects\\: api-magazzino (Python + FastAPI), gestionale-fatture (Node.js + Next.js) und portale-clienti (PHP 8.4 + Node 22 + MariaDB + MongoDB).",
          )}
        />

        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
          <GuideShot
            src="05-progetto-avvio-log.webp"
            title={tr(
              "05 · Scheda 1 — Avvio (IDE, Terminali, Agenti, Azioni)",
              "05 · Tab 1 — Launch (IDE, Terminals, Agents, Actions)",
              "05 · Onglet 1 — Exécution (IDE, Terminaux, Agents, Actions)",
              "05 · Registerkarte 1 — Start (IDE, Terminals, Agenten, Aktionen)",
            )}
            caption={tr(
              "Apritori con 1 clic per Editor VSCodium, Terminali (PowerShell, Git Bash, CMD), Agenti IA (Claude, Codex, Antigravity) e Azioni personalizzate con log live.",
              "1-click launchers for VSCodium Editor, Terminals (PowerShell, Git Bash, CMD), AI Agents (Claude, Codex, Antigravity) and custom Actions with live logs.",
              "Lanceurs en 1 clic pour l'Éditeur VSCodium, les Terminaux (PowerShell, Git Bash, CMD), les Agents IA (Claude, Codex, Antigravity) et les Actions personnalisées avec journaux en direct.",
              "1-Klick-Starter für VSCodium-Editor, Terminals (PowerShell, Git Bash, CMD), KI-Agenten (Claude, Codex, Antigravity) und eigene Aktionen mit Live-Logs.",
            )}
            example={tr(
              "Tre azioni pronte in portale-clienti: «Avvia dev server Vite» (npm run dev), «Esegui migrazioni DB» (php artisan migrate) e «Popola dati demo» (php artisan db:seed).",
              "Three ready actions in portale-clienti: 'Start Vite dev server' (npm run dev), 'Run DB migrations' (php artisan migrate) and 'Seed demo data' (php artisan db:seed).",
              "Trois actions prêtes dans portale-clienti : « Démarrer le serveur Vite » (npm run dev), « Exécuter les migrations DB » (php artisan migrate) et « Peupler les données démo » (php artisan db:seed).",
              "Drei bereite Aktionen in portale-clienti: „Vite-Dev-Server starten“ (npm run dev), „DB-Migrationen ausführen“ (php artisan migrate) und „Demodaten einspielen“ (php artisan db:seed).",
            )}
          />

          <GuideShot
            src="06-progetto-sito-web.webp"
            title={tr(
              "06 · Scheda 2 — Sito web",
              "06 · Tab 2 — Website",
              "06 · Onglet 2 — Site web",
              "06 · Registerkarte 2 — Website",
            )}
            caption={tr(
              "Configurazione del server web dedicato al progetto con modalità, sottocartella pubblica (public), porta HTTP, HTTPS locale e Avvio automatico.",
              "Project-dedicated web server configuration with mode, public subfolder (public), HTTP port, local HTTPS and Auto-start.",
              "Configuration du serveur web dédié au projet avec mode, sous-dossier public (public), port HTTP, HTTPS local et Démarrage automatique.",
              "Konfiguration des projektspezifischen Webservers mit Modus, öffentlichem Unterordner (public), HTTP-Port, lokalem HTTPS und automatischem Start.",
            )}
            example={tr(
              "Il progetto portale-clienti è attivo su http://localhost:8080 e https://localhost:8443 servendo D:\\Atollo\\projects\\portale-clienti\\public con PHP 8.4.",
              "Project portale-clienti is active on http://localhost:8080 and https://localhost:8443 serving D:\\Atollo\\projects\\portale-clienti\\public with PHP 8.4.",
              "Le projet portale-clienti est actif sur http://localhost:8080 et https://localhost:8443 en servant D:\\Atollo\\projects\\portale-clienti\\public avec PHP 8.4.",
              "Das Projekt portale-clienti läuft auf http://localhost:8080 und https://localhost:8443 und stellt D:\\Atollo\\projects\\portale-clienti\\public mit PHP 8.4 bereit.",
            )}
          />

          <GuideShot
            src="07-progetto-componenti-versioni.webp"
            title={tr(
              "07 · Scheda 3 — Componenti e versioni (data\\projects\\<progetto>\\dev-settings.json)",
              "07 · Tab 3 — Components & versions (data\\projects\\<project>\\dev-settings.json)",
              "07 · Onglet 3 — Composants et versions (data\\projects\\<projet>\\dev-settings.json)",
              "07 · Registerkarte 3 — Komponenten & Versionen (data\\projects\\<projekt>\\dev-settings.json)",
            )}
            caption={tr(
              "Selettori per fissare la linea major.minor di ogni linguaggio/database per questo specifico progetto e installazione automatica delle dipendenze (composer, npm, pip/uv, cargo).",
              "Dropdowns to pin the major.minor line of each language/database for this specific project and one-click dependency installation (composer, npm, pip/uv, cargo).",
              "Sélecteurs pour fixer la ligne major.minor de chaque langage/base de données pour ce projet spécifique et installation en un clic des dépendances (composer, npm, pip/uv, cargo).",
              "Auswahlfelder zum Fixieren der major.minor-Linie jeder Sprache/Datenbank für dieses Projekt sowie 1-Klick-Installation der Abhängigkeiten (composer, npm, pip/uv, cargo).",
            )}
            example={tr(
              "portale-clienti fissa Node.js 22 (invece della 24 globale) e PHP 8.4: aprendo un terminale in questo progetto, PATH punterà automaticamente a node\\22 e php\\8.4.",
              "portale-clienti pins Node.js 22 (instead of global 24) and PHP 8.4: opening a terminal in this project automatically points PATH to node\\22 and php\\8.4.",
              "portale-clienti fixe Node.js 22 (au lieu de la 24 globale) et PHP 8.4 : en ouvrant un terminal dans ce projet, PATH pointera automatiquement vers node\\22 et php\\8.4.",
              "portale-clienti fixiert Node.js 22 (statt global 24) und PHP 8.4: Beim Öffnen eines Terminals in diesem Projekt zeigt PATH automatisch auf node\\22 und php\\8.4.",
            )}
          />

          <GuideShot
            src="08-progetto-database-env.webp"
            title={tr(
              "08 · Scheda 4 — Database e variabili .env",
              "08 · Tab 4 — Databases & .env variables",
              "08 · Onglet 4 — Bases de données et variables .env",
              "08 · Registerkarte 4 — Datenbanken & .env-Variablen",
            )}
            caption={tr(
              "Creazione istantanea di un database MariaDB/MongoDB omonimo (portale_clienti) e scrittura automatica delle stringhe di connessione nel file .env del progetto.",
              "Instant creation of a matching MariaDB/MongoDB database (portale_clienti) and automatic injection of connection strings into the project's .env file.",
              "Création instantanée d'une base MariaDB/MongoDB homonyme (portale_clienti) et écriture automatique des chaînes de connexion dans le fichier .env du projet.",
              "Sofortige Erstellung einer gleichnamigen MariaDB-/MongoDB-Datenbank (portale_clienti) und automatisches Eintragen der Verbindungsdaten in die .env-Datei des Projekts.",
            )}
            example={tr(
              "Con un clic su «Scrivi nel file .env», il kit compila DB_HOST=127.0.0.1, DB_PORT=3306, DB_DATABASE=portale_clienti, DATABASE_URL e MONGODB_URI.",
              "Clicking 'Write to .env file' populates DB_HOST=127.0.0.1, DB_PORT=3306, DB_DATABASE=portale_clienti, DATABASE_URL and MONGODB_URI.",
              "En cliquant sur « Écrire dans le fichier .env », le kit remplit DB_HOST=127.0.0.1, DB_PORT=3306, DB_DATABASE=portale_clienti, DATABASE_URL et MONGODB_URI.",
              "Mit einem Klick auf „In Datei .env schreiben“ trägt das Kit DB_HOST=127.0.0.1, DB_PORT=3306, DB_DATABASE=portale_clienti, DATABASE_URL und MONGODB_URI ein.",
            )}
          />
        </div>

        <GuideShot
          src="09-progetto-git-mcp.webp"
          title={tr(
            "09 · Scheda 5 — Git, sincronizzazione GitHub e Server MCP di progetto",
            "09 · Tab 5 — Git, GitHub sync & project MCP servers",
            "09 · Onglet 5 — Git, synchronisation GitHub et serveurs MCP du projet",
            "09 · Registerkarte 5 — Git, GitHub-Abgleich & Projekt-MCP-Server",
          )}
          caption={tr(
            "Gestione del ramo corrente, file modificati, commit + push con un clic, pubblicazione di progetti locali su GitHub e aggiunta di server MCP dedicati al singolo progetto.",
            "Current branch management, modified files, one-click commit + push, publishing local projects to GitHub and adding project-specific MCP servers.",
            "Gestion de la branche actuelle, fichiers modifiés, commit + push en un clic, publication de projets locaux sur GitHub et ajout de serveurs MCP dédiés au projet.",
            "Verwaltung des aktuellen Branches, geänderte Dateien, Commit + Push mit einem Klick, Veröffentlichen lokaler Projekte auf GitHub und Hinzufügen projektspezifischer MCP-Server.",
          )}
          example={tr(
            "3 file modificati pronti per «Salva e invia modifiche» sul ramo main (sincronizzato con mario-rossi/portale-clienti) e server MCP sqlite-locale configurato nel progetto.",
            "3 modified files ready for 'Commit & Push' on branch main (synced with mario-rossi/portale-clienti) and a local sqlite-locale MCP server configured on the project.",
            "3 fichiers modifiés prêts pour « Enregistrer et envoyer » sur la branche main (synchronisée avec mario-rossi/portale-clienti) et serveur MCP sqlite-locale configuré dans le projet.",
            "3 geänderte Dateien bereit für „Speichern und senden“ auf Branch main (synchronisiert mit mario-rossi/portale-clienti) und ein im Projekt konfigurierter MCP-Server sqlite-locale.",
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
          "Page Agents : synchronisation, quotas en temps réel et statistiques",
          "Seite Agenten: Synchronisation, Echtzeit-Quoten & Statistiken",
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
              {tr("Apri pagina Agenti", "Open Agents page", "Ouvrir la page Agents", "Seite Agenten öffnen")}
            </span>
          </Button>
        }
      >
        <GuideShot
          src="10-agenti-quote-statistiche.webp"
          title={tr(
            "10 · Pagina Agenti con quote live, statistiche 30 giorni e MCP condivisi",
            "10 · Agents page with live quotas, 30-day stats & shared MCP",
            "10 · Page Agents avec quotas en direct, statistiques sur 30 jours et MCP partagés",
            "10 · Seite Agenten mit Live-Quoten, 30-Tage-Statistik & geteilten MCP-Servern",
          )}
          caption={tr(
            "Pannello unificato con barre di avanzamento delle quote (finestra 5h, 7 giorni e modelli Antigravity), statistiche d'uso degli ultimi 30 giorni (sessioni, messaggi, token in/out, costo stimato in USD) ed elenco di skill e server MCP condivisi.",
            "Unified dashboard with quota progress bars (5h window, 7-day window and Antigravity models), 30-day usage statistics (sessions, messages, in/out tokens, estimated USD cost) and shared skills & MCP servers.",
            "Tableau de bord unifié avec barres de progression des quotas (fenêtre 5 h, 7 jours et modèles Antigravity), statistiques d'utilisation sur 30 jours (sessions, messages, jetons in/out, coût estimé en USD) et liste des skills et serveurs MCP partagés.",
            "Einheitliche Übersicht mit Quoten-Fortschrittsbalken (5-Std.-Fenster, 7-Tage-Fenster und Antigravity-Modelle), 30-Tage-Nutzungsstatistik (Sitzungen, Nachrichten, Ein-/Ausgabe-Tokens, geschätzte USD-Kosten) sowie geteilten Skills und MCP-Servern.",
          )}
          example={tr(
            "Claude Code (piano Max, 18% finestra 5h), Codex (piano Plus, 12% finestra 5h) e Antigravity (Google AI Pro) condividono 3 skill (architettura-pulita, migrazioni-sicure, test-automatici) e 2 server MCP (filesystem-kit, github-mcp).",
            "Claude Code (Max plan, 18% 5h window), Codex (Plus plan, 12% 5h window) and Antigravity (Google AI Pro) share 3 skills and 2 MCP servers.",
            "Claude Code (forfait Max, 18 % fenêtre 5 h), Codex (forfait Plus, 12 % fenêtre 5 h) et Antigravity (Google AI Pro) partagent 3 skills et 2 serveurs MCP (filesystem-kit, github-mcp).",
            "Claude Code (Max-Tarif, 18 % 5-Std.-Fenster), Codex (Plus-Tarif, 12 % 5-Std.-Fenster) und Antigravity (Google AI Pro) teilen sich 3 Skills und 2 MCP-Server (filesystem-kit, github-mcp).",
          )}
        />

        <SimpleTable
          columns={[
            tr("Elemento condiviso", "Shared element", "Élément partagé", "Geteiltes Element"),
            tr(
              "Sorgente in data\\agents\\",
              "Source in data\\agents\\",
              "Source dans data\\agents\\",
              "Quelle in data\\agents\\",
            ),
            tr(
              "Destinazioni sincronizzate automaticamente",
              "Automatically synchronized destinations",
              "Destinations synchronisées automatiquement",
              "Automatisch synchronisierte Ziele",
            ),
          ]}
          rows={[
            {
              key: "instructions",
              cells: [
                <strong key="e">
                  {tr("Istruzioni globali", "Global instructions", "Instructions globales", "Globale Anweisungen")}
                </strong>,
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
                <strong key="e">
                  {tr("Competenze (Skill)", "Skills", "Compétences (Skills)", "Fähigkeiten (Skills)")}
                </strong>,
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
                <strong key="e">
                  {tr("Server MCP globali", "Global MCP servers", "Serveurs MCP globaux", "Globale MCP-Server")}
                </strong>,
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
          "Pagina Database: MariaDB (con phpMyAdmin), MongoDB e SQLite",
          "Databases page: MariaDB (with phpMyAdmin), MongoDB & SQLite",
          "Page Bases de données : MariaDB (avec phpMyAdmin), MongoDB et SQLite",
          "Seite Datenbanken: MariaDB (mit phpMyAdmin), MongoDB & SQLite",
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
              {tr(
                "Apri pagina Database",
                "Open Databases page",
                "Ouvrir la page Bases de données",
                "Seite Datenbanken öffnen",
              )}
            </span>
          </Button>
        }
      >
        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
          <GuideShot
            src="11-database-mariadb-adminer.webp"
            title={tr(
              "11 · MariaDB 11.4 + interfaccia web phpMyAdmin",
              "11 · MariaDB 11.4 + phpMyAdmin web interface",
              "11 · MariaDB 11.4 + interface web phpMyAdmin",
              "11 · MariaDB 11.4 + phpMyAdmin-Weboberfläche",
            )}
            caption={tr(
              "Controllo del motore MariaDB (127.0.0.1:3306, utente root), selettore di versione, avvio con 1 clic di phpMyAdmin con auto-login e backup/ripristino .sql.",
              "MariaDB engine control (127.0.0.1:3306, root user), version selector, 1-click phpMyAdmin launch with auto-login and .sql backup/restore.",
              "Contrôle du moteur MariaDB (127.0.0.1:3306, utilisateur root), sélecteur de version, lancement en 1 clic de phpMyAdmin avec connexion automatique et sauvegarde/restauration .sql.",
              "Steuerung des MariaDB-Motors (127.0.0.1:3306, Benutzer root), Versionsauswahl, 1-Klick-Start von phpMyAdmin mit Auto-Login sowie .sql-Backup/Wiederherstellung.",
            )}
            example={tr(
              "Tre database applicativi presenti: api_magazzino (18,4 MB), gestionale_fatture (42,1 MB) e portale_clienti (9,6 MB), con phpMyAdmin attivo su 127.0.0.1:18306.",
              "Three application databases: api_magazzino (18.4 MB), gestionale_fatture (42.1 MB) and portale_clienti (9.6 MB), with phpMyAdmin active on 127.0.0.1:18306.",
              "Trois bases de données applicatives présentes : api_magazzino (18,4 Mo), gestionale_fatture (42,1 Mo) et portale_clienti (9,6 Mo), avec phpMyAdmin actif sur 127.0.0.1:18306.",
              "Drei Anwendungsdatenbanken vorhanden: api_magazzino (18,4 MB), gestionale_fatture (42,1 MB) und portale_clienti (9,6 MB), mit aktivem phpMyAdmin unter 127.0.0.1:18306.",
            )}
          />

          <GuideShot
            src="11b-database-mongodb-esploratore.webp"
            title={tr(
              "11b · MongoDB 8.0 ed esploratore collezioni JSON",
              "11b · MongoDB 8.0 & JSON collection explorer",
              "11b · MongoDB 8.0 et explorateur de collections JSON",
              "11b · MongoDB 8.0 & JSON-Sammlungsbrowser",
            )}
            caption={tr(
              "Gestione del motore MongoDB (127.0.0.1:27017), backup/ripristino tramite mongodump/mongorestore (--archive --gzip) ed esploratore integrato di collezioni e documenti JSON.",
              "MongoDB engine management (127.0.0.1:27017), backup/restore via mongodump/mongorestore (--archive --gzip) and built-in JSON collection/document explorer.",
              "Gestion du moteur MongoDB (127.0.0.1:27017), sauvegarde/restauration via mongodump/mongorestore (--archive --gzip) et explorateur intégré de collections et documents JSON.",
              "Verwaltung des MongoDB-Motors (127.0.0.1:27017), Backup/Wiederherstellung über mongodump/mongorestore (--archive --gzip) und integrierter Browser für JSON-Sammlungen und -Dokumente.",
            )}
            example={tr(
              "Ispezione diretta della collezione sessioni_utenti (1.284 documenti) del database portale_clienti senza installare MongoDB Compass.",
              "Direct inspection of the sessioni_utenti collection (1,284 documents) inside the portale_clienti database without needing MongoDB Compass.",
              "Inspection directe de la collection sessioni_utenti (1 284 documents) de la base portale_clienti sans installer MongoDB Compass.",
              "Direkte Einsicht in die Sammlung sessioni_utenti (1.284 Dokumente) der Datenbank portale_clienti ohne Installation von MongoDB Compass.",
            )}
          />
        </div>
      </EditorSection>

      <EditorSection
        id="guide-feat-servers"
        title={tr(
          "Pagina Server Web: pool VirtualHost e gestione multi-versione",
          "Web Servers page: VirtualHost pools & multi-version management",
          "Page Serveurs Web : pools VirtualHost et gestion multi-versions",
          "Seite Webserver: VirtualHost-Pools & Multi-Versions-Verwaltung",
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
              {tr(
                "Apri pagina Server Web",
                "Open Web Servers page",
                "Ouvrir la page Serveurs Web",
                "Seite Webserver öffnen",
              )}
            </span>
          </Button>
        }
      >
        <GuideShot
          src="12-server-web-pool.webp"
          title={tr(
            "12 · Pagina Server Web e pool multi-versione",
            "12 · Web Servers page & multi-version pool",
            "12 · Page Serveurs Web et pool multi-versions",
            "12 · Seite Webserver & Multi-Versions-Pool",
          )}
          caption={tr(
            "Controllo globale dei pool server, URL cliccabili HTTP/HTTPS, stato per singolo progetto e visualizzatore dei log.",
            "Global server pool controls, clickable HTTP/HTTPS URLs, per-project status and live log viewer.",
            "Contrôle global des pools de serveurs, URL HTTP/HTTPS cliquables, état par projet et visionneuse de journaux.",
            "Globale Steuerung der Server-Pools, anklickbare HTTP-/HTTPS-URLs, Status pro Projekt und Live-Log-Ansicht.",
          )}
          example={tr(
            "I siti portale-clienti (:8080/:8443 con PHP 8.4), gestionale-fatture (:8081/:8444) e sito-vetrina-legacy (con PHP 7.4) girano contemporaneamente senza conflitti.",
            "Sites portale-clienti (:8080/:8443 with PHP 8.4), gestionale-fatture (:8081/:8444) and sito-vetrina-legacy (with PHP 7.4) run simultaneously without conflicts.",
            "Les sites portale-clienti (:8080/:8443 avec PHP 8.4), gestionale-fatture (:8081/:8444) et sito-vetrina-legacy (avec PHP 7.4) tournent simultanément sans conflit.",
            "Die Websites portale-clienti (:8080/:8443 mit PHP 8.4), gestionale-fatture (:8081/:8444) und sito-vetrina-legacy (mit PHP 7.4) laufen gleichzeitig ohne Konflikte.",
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
          "Page Outils : catalogue des composants et matrice multi-versions",
          "Seite Werkzeuge: Komponentenkatalog & Multi-Versions-Matrix",
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
              {tr("Apri pagina Strumenti", "Open Tools page", "Ouvrir la page Outils", "Seite Werkzeuge öffnen")}
            </span>
          </Button>
        }
      >
        <p className="m-0 text-[12px] text-vsc-desc">
          {tr(
            "Ogni linea di versione mostra l'etichetta ",
            "Each version line displays the ",
            "Chaque ligne de version affiche l'étiquette ",
            "Jede Versionslinie zeigt das Abzeichen ",
          )}
          <Tag tone="focus">{t("più recente")}</Tag>
          {tr(
            " quando corrisponde alla più recente offerta dal fornitore e l'indicatore ",
            " badge when it matches the newest line offered by the vendor and the ",
            " lorsqu'elle correspond à la plus récente proposée par l'éditeur ainsi que l'indicateur ",
            ", wenn sie der neuesten vom Hersteller angebotenen Linie entspricht, sowie die Kennzeichnung ",
          )}
          <Tag>{t("bloccata")}</Tag>
          {tr(
            " quando è usata da uno o più progetti (impedendone la rimozione accidentale).",
            " indicator when it is pinned by one or more projects (preventing accidental removal).",
            " lorsqu'elle est utilisée par un ou plusieurs projets (empêchant toute suppression accidentelle).",
            ", wenn sie von einem oder mehreren Projekten verwendet wird (was ein versehentliches Entfernen verhindert).",
          )}
        </p>
        <GuideShot
          src="13-strumenti-catalogo-matrice.webp"
          title={tr(
            "13 · Pagina Strumenti e matrice delle linee di versione",
            "13 · Tools page & version line matrix",
            "13 · Page Outils et matrice des lignes de version",
            "13 · Seite Werkzeuge & Matrix der Versionslinien",
          )}
          caption={tr(
            "Barra laterale divisa in categorie e scheda di dettaglio con matrice delle linee major.minor disponibili dalla fonte ufficiale (Installa, Rendi predefinita, Rimuovi).",
            "Sidebar divided into categories and detail view with the matrix of major.minor lines available from the official vendor (Install, Set default, Remove).",
            "Barre latérale divisée en catégories et fiche détaillée avec la matrice des lignes major.minor disponibles auprès de la source officielle (Installer, Définir par défaut, Supprimer).",
            "In Kategorien unterteilte Seitenleiste und Detailansicht mit der Matrix der von der offiziellen Quelle verfügbaren major.minor-Linien (Installieren, Als Standard, Entfernen).",
          )}
          example={tr(
            "Scheda di PHP con due linee installate contemporaneamente: 8.4 (8.4.4, Predefinita globale) e 7.4 (7.4.33 EOL per progetti legacy), più 8.3 e 8.2 installabili con un clic.",
            "PHP detail view with two lines installed side by side: 8.4 (8.4.4, Global default) and 7.4 (7.4.33 EOL for legacy projects), plus 8.3 and 8.2 installable with one click.",
            "Fiche PHP avec deux lignes installées simultanément : 8.4 (8.4.4, Par défaut globale) et 7.4 (7.4.33 EOL pour projets anciens), plus 8.3 et 8.2 installables en un clic.",
            "PHP-Detailansicht mit zwei parallel installierten Linien: 8.4 (8.4.4, Globaler Standard) und 7.4 (7.4.33 EOL für Legacy-Projekte), plus 8.3 und 8.2 mit einem Klick installierbar.",
          )}
        />
      </EditorSection>

      <EditorSection
        id="guide-feat-settings"
        title={tr(
          "Pagina Impostazioni: aspetto, account GitHub, modelli, rete e manutenzione",
          "Settings page: appearance, GitHub account, templates, network & maintenance",
          "Page Paramètres : apparence, compte GitHub, modèles, réseau et maintenance",
          "Seite Einstellungen: Darstellung, GitHub-Konto, Vorlagen, Netzwerk & Wartung",
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
              {tr("Apri Impostazioni", "Open Settings", "Ouvrir Paramètres", "Einstellungen öffnen")}
            </span>
          </Button>
        }
      >
        <GuideShot
          src="14-impostazioni-github-rete.webp"
          title={tr(
            "14 · Pagina Impostazioni completa",
            "14 · Complete Settings page",
            "14 · Page Paramètres complète",
            "14 · Vollständige Seite Einstellungen",
          )}
          caption={tr(
            "Configurazione di Lingua (IT/EN/FR/DE), Account GitHub, Modelli di progetto, Rete (Proxy HTTP/HTTPS) e informazioni sul kit.",
            "Configuration of Language (IT/EN/FR/DE), GitHub Account, Project Templates, Network (HTTP/HTTPS Proxy) and kit details.",
            "Configuration de la Langue (IT/EN/FR/DE), du Compte GitHub, des Modèles de projet, du Réseau (Proxy HTTP/HTTPS) et des informations sur le kit.",
            "Konfiguration von Sprache (IT/EN/FR/DE), GitHub-Konto, Projektvorlagen, Netzwerk (HTTP-/HTTPS-Proxy) und Kit-Informationen.",
          )}
          example={tr(
            "Account @mario-rossi collegato con semaforo verde «Tutti i permessi richiesti sono attivi» e verifica di rete superata su tutti i 9 endpoint (38–112 ms).",
            "Account @mario-rossi connected with green badge 'All required permissions are active' and network check passed across all 9 endpoints (38–112 ms).",
            "Compte @mario-rossi lié avec voyant vert « Toutes les permissions requises sont actives » et test réseau réussi sur les 9 points de terminaison (38–112 ms).",
            "Konto @mario-rossi verbunden mit grüner Anzeige „Alle erforderlichen Berechtigungen sind aktiv“ und bestandener Netzwerkprüfung über alle 9 Endpunkte (38–112 ms).",
          )}
        />
      </EditorSection>

      <EditorSection
        id="guide-feat-cli"
        title={tr(
          "Interfaccia a riga di comando (CLI) di Atollo.exe",
          "Atollo.exe Command-Line Interface (CLI)",
          "Interface en ligne de commande (CLI) de Atollo.exe",
          "Befehlszeilenschnittstelle (CLI) von Atollo.exe",
        )}
      >
        <p className="m-0 text-[12px] text-vsc-desc">
          {tr(
            "Poiché Atollo.exe usa il sottosistema grafico di Windows per non aprire finestre console nere all'avvio, da PowerShell aggiungi sempre | Out-Host per leggere l'output testuale.",
            "Because Atollo.exe uses the Windows GUI subsystem to avoid flashing black console windows on startup, always pipe to | Out-Host in PowerShell to read text output.",
            "Comme Atollo.exe utilise le sous-système graphique de Windows pour éviter d'ouvrir des fenêtres de console noires au démarrage, ajoutez toujours | Out-Host dans PowerShell pour lire la sortie texte.",
            "Da Atollo.exe das grafische Windows-Subsystem nutzt, um beim Start keine schwarzen Konsolenfenster aufblitzen zu lassen, füge in PowerShell immer | Out-Host hinzu, um die Textausgabe zu lesen.",
          )}
        </p>
        <SimpleTable
          columns={[
            tr(
              "Comando PowerShell (con | Out-Host)",
              "PowerShell command (with | Out-Host)",
              "Commande PowerShell (avec | Out-Host)",
              "PowerShell-Befehl (mit | Out-Host)",
            ),
            tr("Funzione", "Description", "Fonction", "Funktion"),
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
        className="inline-flex shrink-0 cursor-pointer items-center gap-1 rounded border border-vsc-border bg-vsc-sidebar px-1.5 py-0.5 text-[11px] text-vsc-desc hover:text-vsc-fg"
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
          "Procédures 5.1 – 5.3 : Initialisation du lecteur, Réseau/Proxy et Modèles de projet",
          "Abläufe 5.1 – 5.3: Laufwerksinitialisierung, Netzwerk/Proxy & Projektvorlagen",
        )}
      >
        <GuideRecipe
          number="§ 5.1"
          title={tr(
            "Scelta dell'unità, inizializzazione del kit e primo install",
            "Drive selection, kit initialization & first install",
            "Choix du lecteur, initialisation du kit et première installation",
            "Laufwerksauswahl, Kit-Initialisierung & Erstinstallation",
          )}
          whereUi={tr(
            "Wizard di primo avvio (o pagina Strumenti)",
            "First-Run Wizard (or Tools page)",
            "Assistant de premier démarrage (ou page Outils)",
            "Einrichtungsassistent beim ersten Start (oder Seite Werkzeuge)",
          )}
          whereFile="data\app\state.json"
          targetPage="components"
          targetLabel={tr("Apri Strumenti", "Open Tools", "Ouvrir Outils", "Werkzeuge öffnen")}
          onNavigate={onNavigate}
          steps={[
            tr(
              "Formatta l'unità USB/SSD esterna in NTFS o exFAT (consigliato NTFS per prestazioni I/O su node_modules) e copia Atollo.exe in una cartella dedicata (es. D:\\Atollo\\Atollo.exe).",
              "Format your external USB/SSD drive as NTFS or exFAT (NTFS recommended for node_modules I/O performance) and copy Atollo.exe into a dedicated folder (e.g. D:\\Atollo\\Atollo.exe).",
              "Formatez le lecteur externe USB/SSD en NTFS ou exFAT (NTFS recommandé pour les performances E/S sur node_modules) et copiez Atollo.exe dans un dossier dédié (ex. D:\\Atollo\\Atollo.exe).",
              "Formatiere das externe USB-/SSD-Laufwerk mit NTFS oder exFAT (NTFS empfohlen für E/A-Leistung bei node_modules) und kopiere Atollo.exe in einen eigenen Ordner (z. B. D:\\Atollo\\Atollo.exe).",
            ),
            tr(
              "Avvia Atollo.exe e verifica nel Passo 1 del Wizard che lo spazio libero sia almeno 10–15 GB per un profilo completo.",
              "Launch Atollo.exe and check in Step 1 of the Wizard that free space is at least 10–15 GB for a full profile.",
              "Lancez Atollo.exe et vérifiez à l'Étape 1 de l'Assistant que l'espace libre est d'au moins 10 à 15 Go pour un profil complet.",
              "Starte Atollo.exe und prüfe in Schritt 1 des Assistenten, dass mindestens 10–15 GB freier Speicher für ein vollständiges Profil vorhanden sind.",
            ),
            tr(
              "Nel Passo 2 spunta gli strumenti desiderati e le linee major.minor di linguaggi e database.",
              "In Step 2, check the desired tools and major.minor lines for languages and databases.",
              "À l'Étape 2, cochez les outils souhaités et les lignes major.minor des langages et bases de données.",
              "Wähle in Schritt 2 die gewünschten Werkzeuge sowie die major.minor-Linien der Sprachen und Datenbanken aus.",
            ),
            tr(
              "Nel Passo 3 collega l'account GitHub e nel Passo 4 avvia lo scaricamento verificato crittograficamente.",
              "In Step 3, connect your GitHub account and in Step 4 start the cryptographically verified download.",
              "À l'Étape 3, liez votre compte GitHub et à l'Étape 4, lancez le téléchargement vérifié cryptographiquement.",
              "Verknüpfe in Schritt 3 dein GitHub-Konto und starte in Schritt 4 den kryptografisch geprüften Download.",
            ),
          ]}
          purpose={tr(
            "Crea l'intera gerarchia isolata data\\, genera i wrapper portabili e registra in data\\app\\state.json la mappa dei componenti scelti e delle versioni effettivamente installate.",
            "Creates the entire isolated data\\ hierarchy, generates portable wrappers and records in data\\app\\state.json the map of chosen components and installed versions.",
            "Crée toute la hiérarchie isolée data\\, génère les wrappers portables et enregistre dans data\\app\\state.json la carte des composants choisis et des versions installées.",
            "Erstellt die gesamte isolierte data\\-Hierarchie, erzeugt portable Wrapper und speichert in data\\app\\state.json die ausgewählten Komponenten und installierten Versionen.",
          )}
        />

        <GuideRecipe
          number="§ 5.2"
          title={tr(
            "Configurazione della rete, del Proxy aziendale e certificati CA",
            "Network, corporate Proxy & CA certificates configuration",
            "Configuration du réseau, du Proxy d'entreprise et des certificats CA",
            "Konfiguration von Netzwerk, Unternehmens-Proxy & CA-Zertifikaten",
          )}
          whereUi={tr("Impostazioni → Rete", "Settings → Network", "Paramètres → Réseau", "Einstellungen → Netzwerk")}
          whereFile={tr(
            "data\\app\\settings.json (sezione network)",
            "data\\app\\settings.json (network section)",
            "data\\app\\settings.json (section network)",
            "data\\app\\settings.json (Abschnitt network)",
          )}
          targetPage="settings"
          targetLabel={tr(
            "Configura Rete / Proxy",
            "Configure Network / Proxy",
            "Configurer Réseau / Proxy",
            "Netzwerk / Proxy konfigurieren",
          )}
          onNavigate={onNavigate}
          steps={[
            tr(
              "Apri Impostazioni (icona ingranaggio in basso a sinistra) e seleziona la voce «Rete».",
              "Open Settings (gear icon at bottom left) and select 'Network'.",
              "Ouvrez Paramètres (icône d'engrenage en bas à gauche) et sélectionnez « Réseau ».",
              "Öffne Einstellungen (Zahnradsymbol unten links) und wähle „Netzwerk“.",
            ),
            tr(
              "Se ti trovi dietro un proxy aziendale o scolastico, inserisci l'URL nei campi Proxy HTTPS / Proxy HTTP (es. http://utente:pass@proxy.azienda.it:8080) e gli eventuali certificati CA aggiuntivi.",
              "If behind a corporate or school proxy, enter the URL in HTTPS Proxy / HTTP Proxy and any additional CA certificates.",
              "Si vous êtes derrière un proxy d'entreprise ou scolaire, saisissez l'URL dans Proxy HTTPS / Proxy HTTP et les éventuels certificats CA supplémentaires.",
              "Wenn du dich hinter einem Unternehmens- oder Schul-Proxy befindest, gib die URL unter HTTPS-Proxy / HTTP-Proxy sowie etwaige zusätzliche CA-Zertifikate ein.",
            ),
            tr(
              "Clicca su Salva nella barra delle schede per applicare subito la configurazione a tutti i download e ai terminali.",
              "Click Save on the tab bar to immediately apply the configuration to all downloads and terminals.",
              "Cliquez sur Enregistrer dans la barre d'onglets pour appliquer immédiatement la configuration à tous les téléchargements et terminaux.",
              "Klicke in der Registerkartenleiste auf Speichern, um die Konfiguration sofort auf alle Downloads und Terminals anzuwenden.",
            ),
          ]}
          purpose={tr(
            "Propaga automaticamente HTTP_PROXY, HTTPS_PROXY, NO_PROXY e NODE_EXTRA_CA_CERTS sia al client HTTP interno di Rust sia a tutti i processi figli (git, gh, npm, pip, uv, composer, cargo, agenti IA).",
            "Automatically propagates HTTP_PROXY, HTTPS_PROXY, NO_PROXY and NODE_EXTRA_CA_CERTS both to Rust's internal HTTP client and to all child processes (git, gh, npm, pip, uv, composer, cargo, AI agents).",
            "Propage automatiquement HTTP_PROXY, HTTPS_PROXY, NO_PROXY et NODE_EXTRA_CA_CERTS aussi bien au client HTTP interne de Rust qu'à tous les processus enfants (git, gh, npm, pip, uv, composer, cargo, agents IA).",
            "Gibt HTTP_PROXY, HTTPS_PROXY, NO_PROXY und NODE_EXTRA_CA_CERTS automatisch sowohl an den internen Rust-HTTP-Client als auch an alle Kindprozesse (git, gh, npm, pip, uv, composer, cargo, KI-Agenten) weiter.",
          )}
        />

        <GuideRecipe
          number="§ 5.3"
          title={tr(
            "Configurazione dei Framework e Modelli di progetto",
            "Configuring Project Frameworks & Templates",
            "Configuration des Frameworks et Modèles de projet",
            "Konfiguration von Frameworks & Projektvorlagen",
          )}
          whereUi={tr(
            "Impostazioni → Istruzioni di Nuovo progetto",
            "Settings → New project instructions",
            "Paramètres → Instructions de Nouveau projet",
            "Einstellungen → Anweisungen für Neues Projekt",
          )}
          whereFile="data\app\frameworks\"
          targetPage="settings"
          targetLabel={tr(
            "Gestisci Framework in Impostazioni",
            "Manage Frameworks in Settings",
            "Gérer les Frameworks dans Paramètres",
            "Frameworks in Einstellungen verwalten",
          )}
          onNavigate={onNavigate}
          steps={[
            tr(
              "Vai in Impostazioni e osserva nella barra laterale la sezione dedicata ai Framework e componenti aggiuntivi di Nuovo progetto.",
              "Go to Settings and check the sidebar section dedicated to New project Frameworks and add-ons.",
              "Allez dans Paramètres et consultez dans la barre latérale la section dédiée aux Frameworks et ajouts de Nouveau projet.",
              "Wechsle zu Einstellungen und öffne in der Seitenleiste den Abschnitt für Frameworks und Ergänzungen von Neues Projekt.",
            ),
            tr(
              "Seleziona una base (es. Laravel, Vite, Electron, Next.js) o clicca sul pulsante + per aggiungere una nuova ricetta di creazione con comandi, strumenti richiesti e variabili .env predefinite.",
              "Select a base (e.g. Laravel, Vite, Electron, Next.js) or click + to add a new creation recipe with commands, required tools and default .env variables.",
              "Sélectionnez une base (ex. Laravel, Vite, Electron, Next.js) ou cliquez sur + pour ajouter une nouvelle recette de création avec commandes, outils requis et variables .env par défaut.",
              "Wähle eine Basis (z. B. Laravel, Vite, Electron, Next.js) oder klicke auf +, um ein neues Erstellungsrezept mit Befehlen, benötigten Werkzeugen und Standard-.env-Variablen hinzuzufügen.",
            ),
            tr(
              "Salva le modifiche: la ricetta comparirà immediatamente nella pagina Nuovo progetto.",
              "Save your changes: the recipe will immediately appear on the New project page.",
              "Enregistrez les modifications : la recette apparaîtra immédiatement sur la page Nouveau projet.",
              "Speichere die Änderungen: Das Rezept erscheint sofort auf der Seite Neues Projekt.",
            ),
          ]}
          purpose={tr(
            "Standardizza l'architettura iniziale dei progetti del team permettendo di istanziare nuovi applicativi completi di dipendenze e variabili .env con un solo clic.",
            "Standardizes initial project architecture across your team, allowing new applications complete with dependencies and .env variables to be created with a single click.",
            "Standardise l'architecture initiale des projets de l'équipe en permettant d'instancier de nouvelles applications complètes avec dépendances et variables .env en un seul clic.",
            "Standardisiert die Ausgangsarchitektur von Teamprojekten und ermöglicht das Erstellen neuer Anwendungen samt Abhängigkeiten und .env-Variablen mit einem einzigen Klick.",
          )}
        />
      </EditorSection>
    </>
  );
}

// ---------------------------------------------------------------------------
// 5.4 - 5.7 Setup Recipes: Languages (dev-settings.json), Agents, Skills, MCP
// ---------------------------------------------------------------------------

function TopicCfgLangsAgents({ onNavigate }: { onNavigate: (page: AppPageTarget) => void }) {
  return (
    <>
      <EditorSection
        id="guide-cfg-langs-agents"
        title={tr(
          "Procedure 5.4 – 5.7: Versioni linguaggi, Agenti IA, Skill e Server MCP",
          "Procedures 5.4 – 5.7: Language versions, AI Agents, Skills & MCP Servers",
          "Procédures 5.4 – 5.7 : Versions des langages, Agents IA, Skills et Serveurs MCP",
          "Abläufe 5.4 – 5.7: Sprachversionen, KI-Agenten, Skills & MCP-Server",
        )}
      >
        <GuideRecipe
          number="§ 5.4"
          title={tr(
            "Configurazione delle versioni dei linguaggi (globale vs per-progetto)",
            "Configuring language versions (global vs per-project)",
            "Configuration des versions des langages (globale vs par projet)",
            "Konfiguration der Sprachversionen (global vs. pro Projekt)",
          )}
          whereUi={tr(
            "Strumenti / Impostazioni → Versioni (globale) e Progetti → Componenti (per il singolo progetto)",
            "Tools / Settings → Versions (global) and Projects → Components (for a single project)",
            "Outils / Paramètres → Versions (globale) et Projets → Composants (pour un projet)",
            "Werkzeuge / Einstellungen → Versionen (global) und Projekte → Komponenten (pro Projekt)",
          )}
          whereFile="data\app\settings.json · data\projects\<id>\dev-settings.json"
          targetPage="components"
          targetLabel={tr("Apri Strumenti", "Open Tools", "Ouvrir Outils", "Werkzeuge öffnen")}
          onNavigate={onNavigate}
          steps={[
            tr(
              "Nella pagina Strumenti seleziona un componente multi-versione (Python, Node.js, PHP, .NET, Java, Go, Ruby, R, MariaDB o MongoDB) e installa una o più linee major.minor.",
              "In the Tools page, select a multi-version component (Python, Node.js, PHP, .NET, Java, Go, Ruby, R, MariaDB or MongoDB) and install one or more major.minor lines.",
              "Sur la page Outils, sélectionnez un composant multi-versions (Python, Node.js, PHP, .NET, Java, Go, Ruby, R, MariaDB ou MongoDB) et installez une ou plusieurs lignes major.minor.",
              "Wähle auf der Seite Werkzeuge eine Multi-Versions-Komponente (Python, Node.js, PHP, .NET, Java, Go, Ruby, R, MariaDB oder MongoDB) und installiere eine oder mehrere major.minor-Linien.",
            ),
            tr(
              "Imposta la versione predefinita del kit in Impostazioni → Versioni (oppure direttamente dalla scheda dello strumento).",
              "Set the kit's default version in Settings → Versions (or directly from the tool's detail tab).",
              "Définissez la version par défaut du kit dans Paramètres → Versions (ou directement depuis la fiche de l'outil).",
              "Lege die Standardversion des Kits unter Einstellungen → Versionen (oder direkt in der Werkzeugansicht) fest.",
            ),
            tr(
              "Per un progetto che richiede una versione diversa, apri Progetti → <progetto> → scheda «Componenti» e scegli la linea specifica per quel progetto.",
              "For a project requiring a different version, open Projects → <project> → 'Components' tab and pick the specific line for that project.",
              "Pour un projet nécessitant une version différente, ouvrez Projets → <projet> → onglet « Composants » et choisissez la ligne spécifique à ce projet.",
              "Für ein Projekt, das eine andere Version benötigt, öffne Projekte → <projekt> → Registerkarte „Komponenten“ und wähle die gewünschte Linie für dieses Projekt.",
            ),
          ]}
          purpose={tr(
            "Costruisce dinamicamente la variabile PATH per ogni progetto, anteponendo i binari delle versioni scelte per quel progetto e impostando automaticamente JAVA_HOME, GOROOT e COMPOSER_PHP.",
            "Dynamically constructs the PATH variable for each project, prepending binaries for the project's chosen versions and automatically setting JAVA_HOME, GOROOT and COMPOSER_PHP.",
            "Construit dynamiquement la variable PATH pour chaque projet, en plaçant en tête les binaires des versions choisies pour ce projet et en définissant automatiquement JAVA_HOME, GOROOT et COMPOSER_PHP.",
            "Erstellt dynamisch die PATH-Variable für jedes Projekt, stellt die Binärdateien der gewählten Projektversionen voran und setzt automatisch JAVA_HOME, GOROOT und COMPOSER_PHP.",
          )}
        />

        <GuideRecipe
          number="§ 5.5"
          title={tr(
            "Autenticazione dei 3 Agenti IA e salvataggio credenziali cifrate DPAPI",
            "Authenticating the 3 AI Agents & DPAPI encrypted credential storage",
            "Authentification des 3 Agents IA et sauvegarde des identifiants chiffrés DPAPI",
            "Authentifizierung der 3 KI-Agenten & DPAPI-verschlüsselte Speicherung",
          )}
          whereUi={tr(
            "Agenti (o Progetti → Avvio → pulsante Agente)",
            "Agents (or Projects → Launch → Agent button)",
            "Agents (ou Projets → Exécution → bouton Agent)",
            "Agenten (oder Projekte → Start → Agenten-Schaltfläche)",
          )}
          whereFile="data\claude\.credentials.json · data\codex\auth.json · data\antigravity\.keyring.dpapi"
          targetPage="agents"
          targetLabel={tr("Apri pagina Agenti", "Open Agents page", "Ouvrir la page Agents", "Seite Agenten öffnen")}
          onNavigate={onNavigate}
          steps={[
            tr(
              "Per Claude Code: avvialo da un progetto; al primo avvio esegui il login OAuth Anthropic nel browser. Il token viene salvato in data\\claude\\.credentials.json.",
              "For Claude Code: launch it from a project; on first run complete the Anthropic OAuth login in your browser. The token is saved in data\\claude\\.credentials.json.",
              "Pour Claude Code : lancez-le depuis un projet ; au premier démarrage, effectuez la connexion OAuth Anthropic dans le navigateur. Le jeton est enregistré dans data\\claude\\.credentials.json.",
              "Für Claude Code: Starte ihn aus einem Projekt; führe beim ersten Start das Anthropic-OAuth-Login im Browser durch. Das Token wird in data\\claude\\.credentials.json gespeichert.",
            ),
            tr(
              "Per Codex: avvialo da un progetto e completa l'accesso ChatGPT/OpenAI; il token viene salvato in data\\codex\\auth.json.",
              "For Codex: launch it from a project and complete the ChatGPT/OpenAI login; the token is saved in data\\codex\\auth.json.",
              "Pour Codex : lancez-le depuis un projet et terminez la connexion ChatGPT/OpenAI ; le jeton est enregistré dans data\\codex\\auth.json.",
              "Für Codex: Starte ihn aus einem Projekt und schließe die ChatGPT-/OpenAI-Anmeldung ab; das Token wird in data\\codex\\auth.json gespeichert.",
            ),
            tr(
              "Per Antigravity: avvialo tramite il pulsante Antigravity (che passa per data\\kit\\bin\\agy.cmd); alla chiusura, Atollo.exe keyring save esporta e cifra con Windows DPAPI il token in data\\antigravity\\.keyring.dpapi.",
              "For Antigravity: launch it via the Antigravity button (which runs data\\kit\\bin\\agy.cmd); on exit, Atollo.exe keyring save exports and encrypts the token with Windows DPAPI into data\\antigravity\\.keyring.dpapi.",
              "Pour Antigravity : lancez-le via le bouton Antigravity (qui passe par data\\kit\\bin\\agy.cmd) ; à la fermeture, Atollo.exe keyring save exporte et chiffre avec Windows DPAPI le jeton dans data\\antigravity\\.keyring.dpapi.",
              "Für Antigravity: Starte ihn über die Schaltfläche Antigravity (über data\\kit\\bin\\agy.cmd); beim Beenden exportiert und verschlüsselt Atollo.exe keyring save das Token mittels Windows DPAPI in data\\antigravity\\.keyring.dpapi.",
            ),
          ]}
          purpose={tr(
            "Mantiene le sessioni IA confinate sull'unità portabile e abilita il monitoraggio in tempo reale delle quote residue nella pagina Agenti.",
            "Keeps AI sessions confined to the portable drive and enables real-time remaining quota monitoring on the Agents page.",
            "Maintient les sessions IA confinées sur le lecteur portable et active le suivi en temps réel des quotas restants sur la page Agents.",
            "Hält KI-Sitzungen auf dem portablen Laufwerk isoliert und ermöglicht die Echtzeit-Überwachung der verbleibenden Quoten auf der Seite Agenten.",
          )}
        />

        <GuideRecipe
          number="§ 5.6"
          title={tr(
            "Configurazione delle Istruzioni globali (AGENTS.md) e delle Skill condivise",
            "Configuring global Instructions (AGENTS.md) & shared Skills",
            "Configuration des Instructions globales (AGENTS.md) et des Skills partagées",
            "Konfiguration globaler Anweisungen (AGENTS.md) & geteilter Skills",
          )}
          whereUi={tr(
            "Agenti → cartella condivisa data\\agents\\",
            "Agents → shared folder data\\agents\\",
            "Agents → dossier partagé data\\agents\\",
            "Agenten → geteilter Ordner data\\agents\\",
          )}
          whereFile="data\agents\AGENTS.md · data\agents\skills\<nome>\SKILL.md"
          targetPage="agents"
          targetLabel={tr("Apri Agenti", "Open Agents", "Ouvrir Agents", "Agenten öffnen")}
          onNavigate={onNavigate}
          steps={[
            tr(
              "Nella pagina Agenti clicca sull'icona «Apri la cartella condivisa» in alto nella barra laterale per aprire data\\agents\\.",
              "In the Agents page, click the 'Open shared folder' icon at the top of the sidebar to open data\\agents\\.",
              "Sur la page Agents, cliquez sur l'icône « Ouvrir le dossier partagé » en haut de la barre latérale pour ouvrir data\\agents\\.",
              "Klicke auf der Seite Agenten oben in der Seitenleiste auf das Symbol „Geteilten Ordner öffnen“, um data\\agents\\ zu öffnen.",
            ),
            tr(
              "Modifica il file AGENTS.md per definire le tue regole di codifica, lingua preferita e convenzioni architetturali valide per tutti gli agenti.",
              "Edit AGENTS.md to define your coding rules, preferred language and architectural conventions across all agents.",
              "Modifiez le fichier AGENTS.md pour définir vos règles de codage, votre langue préférée et vos conventions architecturales communes à tous les agents.",
              "Bearbeite die Datei AGENTS.md, um Programmierregeln, bevorzugte Sprache und Architekturkonventionen für alle Agenten festzulegen.",
            ),
            tr(
              "Aggiungi sottocartelle in data\\agents\\skills\\<nome>\\SKILL.md per condividere competenze specializzate: ad ogni avvio di un agente il kit le allinea automaticamente.",
              "Add subfolders in data\\agents\\skills\\<name>\\SKILL.md to share specialized skills: every time an agent starts, the kit synchronizes them automatically.",
              "Ajoutez des sous-dossiers dans data\\agents\\skills\\<nom>\\SKILL.md pour partager des compétences spécialisées : à chaque démarrage d'un agent, le kit les synchronise automatiquement.",
              "Füge Unterordner in data\\agents\\skills\\<Name>\\SKILL.md hinzu, um spezialisierte Fähigkeiten zu teilen: Bei jedem Start eines Agenten gleicht das Kit sie automatisch ab.",
            ),
          ]}
          purpose={tr(
            "Evita di dover mantenere 3 copie separate delle regole e delle skill: una sola modifica in data\\agents\\ istruisce contemporaneamente Claude Code, Codex e Antigravity.",
            "Eliminates maintaining 3 separate copies of rules and skills: a single edit in data\\agents\\ instructs Claude Code, Codex and Antigravity simultaneously.",
            "Évite de maintenir 3 copies séparées des règles et des skills : une seule modification dans data\\agents\\ instruit simultanément Claude Code, Codex et Antigravity.",
            "Erspart das Pflegen von 3 getrennten Kopien von Regeln und Skills: Eine einzige Änderung in data\\agents\\ gilt gleichzeitig für Claude Code, Codex und Antigravity.",
          )}
        />

        <GuideRecipe
          number="§ 5.7"
          title={tr(
            "Configurazione dei Server MCP condivisi",
            "Configuring shared MCP Servers",
            "Configuration des Serveurs MCP partagés",
            "Konfiguration geteilter MCP-Server",
          )}
          whereUi={tr(
            "Cartella condivisa data\\agents\\mcp.json",
            "Shared folder data\\agents\\mcp.json",
            "Dossier partagé data\\agents\\mcp.json",
            "Geteilter Ordner data\\agents\\mcp.json",
          )}
          whereFile="data\agents\mcp.json"
          targetPage="agents"
          targetLabel={tr("Vai ad Agenti", "Go to Agents", "Aller à Agents", "Zu Agenten")}
          onNavigate={onNavigate}
          steps={[
            tr(
              "Apri il file data\\agents\\mcp.json nella cartella condivisa degli agenti.",
              "Open data\\agents\\mcp.json in the shared agents folder.",
              "Ouvrez le fichier data\\agents\\mcp.json dans le dossier partagé des agents.",
              "Öffne die Datei data\\agents\\mcp.json im geteilten Ordner der Agenten.",
            ),
            tr(
              "Aggiungi dentro l'oggetto mcpServers i server desiderati specificando command e args.",
              "Add your desired servers inside the mcpServers object specifying command and args.",
              "Ajoutez dans l'objet mcpServers les serveurs souhaités en spécifiant command et args.",
              "Füge im Objekt mcpServers die gewünschten Server mit command und args hinzu.",
            ),
            tr(
              "All'avvio di un agente (o eseguendo .\\Atollo.exe sync), il kit traduce automaticamente la definizione nei 3 formati (.claude.json, config.toml di Codex e mcp_config.json di Antigravity).",
              "When launching an agent (or running .\\Atollo.exe sync), the kit automatically translates the definition into all 3 formats (.claude.json, Codex config.toml and Antigravity mcp_config.json).",
              "Au démarrage d'un agent (ou en exécutant .\\Atollo.exe sync), le kit traduit automatiquement la définition dans les 3 formats (.claude.json, config.toml de Codex et mcp_config.json d'Antigravity).",
              "Beim Start eines Agenten (oder über .\\Atollo.exe sync) übersetzt das Kit die Definition automatisch in alle 3 Formate (.claude.json, Codex config.toml und Antigravity mcp_config.json).",
            ),
          ]}
          purpose={tr(
            "Estende le capacità degli agenti IA permettendo loro di interrogare database locali, documentazione o strumenti esterni tramite il protocollo standard Model Context Protocol.",
            "Extends AI agent capabilities, allowing them to query local databases, documentation or external tools via the standard Model Context Protocol.",
            "Étend les capacités des agents IA en leur permettant d'interroger des bases de données locales, de la documentation ou des outils externes via le protocole standard Model Context Protocol.",
            "Erweitert die Fähigkeiten der KI-Agenten, sodass sie lokale Datenbanken, Dokumentationen oder externe Werkzeuge über das Model Context Protocol abfragen können.",
          )}
        >
          <GuideCode
            title={tr(
              "Esempio di configurazione MCP condivisa",
              "Example shared MCP configuration",
              "Exemple de configuration MCP partagée",
              "Beispiel einer geteilten MCP-Konfiguration",
            )}
            path="<Radice>\data\agents\mcp.json"
            code={`{
  "mcpServers": {
    "filesystem-kit": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "D:/Atollo/projects"]
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
          "Procédures 5.8 – 5.9 : Bases de données locales, injection .env et Serveur Web",
          "Abläufe 5.8 – 5.9: Lokale Datenbanken, .env-Eintragung & Webserver",
        )}
      >
        <GuideRecipe
          number="§ 5.8"
          title={tr(
            "Creazione di un Database di progetto e collegamento con .env",
            "Creating a project Database & linking with .env",
            "Création d'une Base de données de projet et liaison avec .env",
            "Erstellen einer Projektdatenbank & Verknüpfung mit .env",
          )}
          whereUi={tr(
            "Progetti → <progetto> → scheda Database (oppure pagina Database)",
            "Projects → <project> → Databases tab (or Databases page)",
            "Projets → <projet> → onglet Base de données (ou page Bases de données)",
            "Projekte → <projekt> → Registerkarte Datenbank (oder Seite Datenbanken)",
          )}
          whereFile="data\db\ · projects\<nome>\.env"
          targetPage="databases"
          targetLabel={tr(
            "Apri pagina Database",
            "Open Databases page",
            "Ouvrir la page Bases de données",
            "Seite Datenbanken öffnen",
          )}
          onNavigate={onNavigate}
          steps={[
            tr(
              "Apri il progetto in Progetti → <progetto> e seleziona la scheda «Database».",
              "Open your project in Projects → <project> and select the 'Databases' tab.",
              "Ouvrez le projet dans Projets → <projet> et sélectionnez l'onglet « Base de données ».",
              "Öffne das Projekt unter Projekte → <projekt> und wähle die Registerkarte „Datenbank“.",
            ),
            tr(
              "Crea o collega un database MariaDB, MongoDB o SQLite per il progetto: il motore parte automaticamente in locale su 127.0.0.1.",
              "Create or link a MariaDB, MongoDB or SQLite database for the project: the engine starts automatically on 127.0.0.1.",
              "Créez ou liez une base MariaDB, MongoDB ou SQLite pour le projet : le moteur démarre automatiquement en local sur 127.0.0.1.",
              "Erstelle oder verknüpfe eine MariaDB-, MongoDB- oder SQLite-Datenbank für das Projekt: Der Motor startet automatisch lokal auf 127.0.0.1.",
            ),
            tr(
              "Usa i pulsanti rapidi per sincronizzare le credenziali nel file .env del progetto o per esplorare tabelle e collezioni.",
              "Use the quick buttons to sync credentials into the project's .env file or browse tables and collections.",
              "Utilisez les boutons rapides pour synchroniser les identifiants dans le fichier .env du projet ou pour explorer tables et collections.",
              "Nutze die Schnellschaltflächen, um Zugangsdaten in die .env-Datei des Projekts zu übernehmen oder Tabellen und Sammlungen zu durchsuchen.",
            ),
          ]}
          purpose={tr(
            "Elimina gli errori di configurazione manuale delle porte o delle stringhe di connessione e conserva i dati fisici su disco dentro data\\db\\.",
            "Eliminates manual port or connection string typos and stores physical data on disk inside data\\db\\.",
            "Élimine les erreurs de configuration manuelle des ports ou chaînes de connexion et conserve les données physiques sur le disque dans data\\db\\.",
            "Vermeidet Tippfehler bei Ports oder Verbindungszeichenfolgen und speichert die physischen Daten auf dem Datenträger unter data\\db\\.",
          )}
        >
          <GuideCode
            title={tr(
              "Variabili .env tipiche di un progetto collegato",
              "Typical .env variables of a linked project",
              "Variables .env typiques d'un projet lié",
              "Typische .env-Variablen eines verknüpften Projekts",
            )}
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
            "Configuration du Site Web de projet et des Pools VirtualHost",
            "Konfiguration der Projekt-Website & VirtualHost-Pools",
          )}
          whereUi={tr(
            "Progetti → <progetto> → scheda Sito web (e pagina Server Web)",
            "Projects → <project> → Website tab (and Web Servers page)",
            "Projets → <projet> → onglet Site web (et page Serveurs Web)",
            "Projekte → <projekt> → Registerkarte Website (und Seite Webserver)",
          )}
          whereFile={tr(
            "data\\projects\\<id>\\dev-settings.json · pagina Server Web",
            "data\\projects\\<id>\\dev-settings.json · Web Servers page",
            "data\\projects\\<id>\\dev-settings.json · page Serveurs Web",
            "data\\projects\\<id>\\dev-settings.json · Seite Webserver",
          )}
          targetPage="servers"
          targetLabel={tr(
            "Apri pagina Server Web",
            "Open Web Servers page",
            "Ouvrir la page Serveurs Web",
            "Seite Webserver öffnen",
          )}
          onNavigate={onNavigate}
          steps={[
            tr(
              "Apri Progetti → <progetto> → scheda «Sito web».",
              "Open Projects → <project> → 'Website' tab.",
              "Ouvrez Projets → <projet> → onglet « Site web ».",
              "Öffne Projekte → <projekt> → Registerkarte „Website“.",
            ),
            tr(
              "Scegli il motore web (es. Apache, Nginx o server di sviluppo), la cartella pubblica (es. public) ed eventuali alias relativi.",
              "Choose the web engine (e.g. Apache, Nginx or dev server), the document root (e.g. public) and any relative aliases.",
              "Choisissez le moteur web (ex. Apache, Nginx ou serveur de développement), le dossier public (ex. public) et les éventuels alias relatifs.",
              "Wähle die Web-Engine (z. B. Apache, Nginx oder Entwicklungsserver), den öffentlichen Ordner (z. B. public) und eventuelle relative Aliase.",
            ),
            tr(
              "Nella pagina «Server Web» avvia o ricarica il pool corrispondente e apri il sito su <progetto>.localhost con un clic.",
              "In the 'Web Servers' page, start or reload the matching pool and open the site at <project>.localhost with one click.",
              "Sur la page « Serveurs Web », démarrez ou rechargez le pool correspondant et ouvrez le site sur <projet>.localhost en un clic.",
              "Starte oder lade auf der Seite „Webserver“ den entsprechenden Pool neu und öffne die Website unter <projekt>.localhost mit einem Klick.",
            ),
          ]}
          purpose={tr(
            "Raggruppa automaticamente i siti che condividono la stessa combinazione di motore e versione PHP in pool VirtualHost dedicati, rigenerando gli alias relativi se cambia la lettera di unità.",
            "Automatically groups sites sharing the same engine and PHP version into dedicated VirtualHost pools, regenerating relative aliases whenever the drive letter changes.",
            "Regroupe automatiquement les sites partageant la même combinaison de moteur et de version PHP dans des pools VirtualHost dédiés, en régénérant les alias relatifs si la lettre de lecteur change.",
            "Gruppiert Websites mit derselben Engine- und PHP-Versionskombination automatisch in eigenen VirtualHost-Pools und aktualisiert relative Aliase bei einem Wechsel des Laufwerksbuchstabens.",
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
          "Procédures 5.10 – 5.12 : Actions de lancement, Extensions de l'Éditeur et Maintenance",
          "Abläufe 5.10 – 5.12: Startaktionen, Editor-Erweiterungen & Wartung",
        )}
      >
        <GuideRecipe
          number="§ 5.10"
          title={tr(
            "Esecuzione e monitoraggio delle Azioni di avvio di un progetto",
            "Running & monitoring a project's Launch Actions",
            "Exécution et suivi des Actions de lancement d'un projet",
            "Ausführen & Überwachen der Startaktionen eines Projekts",
          )}
          whereUi={tr(
            "Progetti → <progetto> → scheda Avvio (e pannello Attività in basso)",
            "Projects → <project> → Launch tab (and bottom Activity panel)",
            "Projets → <projet> → onglet Exécution (et panneau Activité en bas)",
            "Projekte → <projekt> → Registerkarte Start (und Aktivitätsbereich unten)",
          )}
          whereFile="package.json / composer.json / data\projects\<id>\dev-settings.json"
          targetPage="projects"
          targetLabel={tr("Apri Progetti", "Open Projects", "Ouvrir Projets", "Projekte öffnen")}
          onNavigate={onNavigate}
          steps={[
            tr(
              "Apri Progetti → <progetto> → scheda «Avvio».",
              "Open Projects → <project> → 'Launch' tab.",
              "Ouvrez Projets → <projet> → onglet « Exécution ».",
              "Öffne Projekte → <projekt> → Registerkarte „Start“.",
            ),
            tr(
              "Seleziona una delle azioni rilevate dal progetto (es. dev server, build, test, migrazioni) e premi «Avvia».",
              "Select one of the detected project actions (e.g. dev server, build, test, migrations) and click 'Start'.",
              "Sélectionnez l'une des actions détectées du projet (ex. serveur de dév, build, tests, migrations) et cliquez sur « Démarrer ».",
              "Wähle eine der erkannten Projektaktionen (z. B. Dev-Server, Build, Tests, Migrationen) und klicke auf „Starten“.",
            ),
            tr(
              "Segui l'output in tempo reale nel riquadro dei log o apri il pannello «Attività» dalla barra di stato inferiore per controllare tutti i processi attivi.",
              "Follow real-time output in the log pane or open the 'Activity' panel from the bottom status bar to monitor all running processes.",
              "Suivez la sortie en temps réel dans le volet des journaux ou ouvrez le panneau « Activité » depuis la barre d'état inférieure pour contrôler tous les processus actifs.",
              "Verfolge die Ausgabe in Echtzeit im Log-Bereich oder öffne den Bereich „Aktivität“ über die untere Statusleiste, um alle laufenden Prozesse zu überwachen.",
            ),
          ]}
          purpose={tr(
            "Esegue i processi del progetto nell'ambiente isolato del kit con le versioni di linguaggio scelte per quel progetto, rilevando automaticamente le porte HTTP aperte.",
            "Runs project processes inside the kit's isolated environment with the language versions chosen for that project, automatically detecting open HTTP ports.",
            "Exécute les processus du projet dans l'environnement isolé du kit avec les versions de langage choisies pour ce projet, en détectant automatiquement les ports HTTP ouverts.",
            "Führt Projektprozesse in der isolierten Umgebung des Kits mit den für das Projekt gewählten Sprachversionen aus und erkennt geöffnete HTTP-Ports automatisch.",
          )}
        />

        <GuideRecipe
          number="§ 5.11"
          title={tr(
            "Gestione delle Estensioni VSCodium e delle impostazioni dell'Editor",
            "Managing VSCodium Extensions & Editor settings",
            "Gestion des Extensions VSCodium et des paramètres de l'Éditeur",
            "Verwaltung der VSCodium-Erweiterungen & Editor-Einstellungen",
          )}
          whereUi={tr(
            "Strumenti → Estensioni dell'editor (globale) e Progetti → Estensioni / Editor",
            "Tools → Editor extensions (global) and Projects → Extensions / Editor",
            "Outils → Extensions de l'éditeur (globale) et Projets → Extensions / Éditeur",
            "Werkzeuge → Editor-Erweiterungen (global) und Projekte → Erweiterungen / Editor",
          )}
          whereFile="data\vscodium-server\data\extensions\ · projects\<nome>\.vscode\settings.json"
          targetPage="components"
          targetLabel={tr("Apri Strumenti", "Open Tools", "Ouvrir Outils", "Werkzeuge öffnen")}
          onNavigate={onNavigate}
          steps={[
            tr(
              "In Strumenti → «Estensioni dell'editor» consulta la matrice che associa automaticamente le estensioni Open VSX ai linguaggi rilevati.",
              "In Tools → 'Editor extensions', inspect the matrix that automatically maps Open VSX extensions to detected languages.",
              "Dans Outils → « Extensions de l'éditeur », consultez la matrice qui associe automatiquement les extensions Open VSX aux langages détectés.",
              "Sieh dir unter Werkzeuge → „Editor-Erweiterungen“ die Matrix an, die Open-VSX-Erweiterungen automatisch den erkannten Sprachen zuordnet.",
            ),
            tr(
              "In Progetti → <progetto> → scheda «Estensioni» aggiungi o escludi estensioni specifiche per quel singolo progetto.",
              "In Projects → <project> → 'Extensions' tab, add or exclude specific extensions for that individual project.",
              "Dans Projets → <projet> → onglet « Extensions », ajoutez ou excluez des extensions spécifiques pour ce projet.",
              "Füge unter Projekte → <projekt> → Registerkarte „Erweiterungen“ bestimmte Erweiterungen für dieses einzelne Projekt hinzu oder schließe sie aus.",
            ),
            tr(
              "Nella scheda «Editor» del progetto personalizza tab size, formattazione al salvataggio e tema per quel progetto.",
              "In the project's 'Editor' tab, customize tab size, format-on-save and theme for that project.",
              "Dans l'onglet « Éditeur » du projet, personnalisez la taille des tabulations, le formatage à l'enregistrement et le thème pour ce projet.",
              "Passe auf der Registerkarte „Editor“ des Projekts Tabulatorgröße, Formatierung beim Speichern und Design für dieses Projekt an.",
            ),
          ]}
          purpose={tr(
            "Mantiene l'editor VSCodium leggero e mirato installando automaticamente da Open VSX solo le estensioni necessarie ai linguaggi effettivamente usati.",
            "Keeps the VSCodium editor fast and focused by automatically installing from Open VSX only the extensions needed for the languages actually used.",
            "Garde l'éditeur VSCodium léger et ciblé en installant automatiquement depuis Open VSX uniquement les extensions nécessaires aux langages réellement utilisés.",
            "Hält den VSCodium-Editor schlank und fokussiert, indem automatisch nur die für die tatsächlich genutzten Sprachen benötigten Erweiterungen von Open VSX installiert werden.",
          )}
        />

        <GuideRecipe
          number="§ 5.12"
          title={tr(
            "Aggiornamento del kit e rimozione sicura dell'unità",
            "Updating the kit & safely ejecting the drive",
            "Mise à jour du kit et retrait sécurisé du lecteur",
            "Aktualisieren des Kits & sicheres Entfernen des Laufwerks",
          )}
          whereUi={tr(
            "Impostazioni → Aggiornamenti / Il kit · Rimuovi l'unità",
            "Settings → Updates / The kit · Remove drive",
            "Paramètres → Mises à jour / Le kit · Retirer le lecteur",
            "Einstellungen → Updates / Das Kit · Laufwerk entfernen",
          )}
          whereFile="data\app\ · data\antigravity\.keyring.dpapi"
          targetPage="eject"
          targetLabel={tr(
            "Apri «Rimuovi l'unità»",
            "Open “Remove drive”",
            "Ouvrir « Retirer le lecteur »",
            "„Laufwerk entfernen“ öffnen",
          )}
          onNavigate={onNavigate}
          steps={[
            tr(
              "In Impostazioni → «Aggiornamenti» verifica la presenza di nuove versioni dell'applicazione e degli strumenti.",
              "In Settings → 'Updates', check for new versions of the application and tools.",
              "Dans Paramètres → « Mises à jour », vérifiez la disponibilité de nouvelles versions de l'application et des outils.",
              "Prüfe unter Einstellungen → „Updates“, ob neue Versionen der Anwendung und Werkzeuge verfügbar sind.",
            ),
            tr(
              "Quando hai terminato di lavorare, apri «Rimuovi l'unità» dalla barra sinistra in basso.",
              "When you finish working, open “Remove drive” from the bottom-left Activity Bar.",
              "Lorsque vous avez terminé de travailler, ouvrez « Retirer le lecteur » depuis la barre d'activité en bas à gauche.",
              "Wenn du mit der Arbeit fertig bist, öffne unten links in der Aktivitätsleiste „Laufwerk entfernen“.",
            ),
            tr(
              "Clicca su «Chiudi tutto e prepara la rimozione» per fermare ordinatamente database, siti web, editor e salvare nel kit l'accesso di Antigravity.",
              "Click “Close everything and prepare for removal” to cleanly stop databases, websites, editor and save Antigravity's login into the kit.",
              "Cliquez sur « Tout fermer et préparer le retrait » pour arrêter proprement bases de données, sites web, éditeur et enregistrer la session d'Antigravity dans le kit.",
              "Klicke auf „Alles schließen und Entfernen vorbereiten“, um Datenbanken, Websites und Editor geordnet zu beenden und den Antigravity-Zugang im Kit zu sichern.",
            ),
          ]}
          purpose={tr(
            "Garantisce l'integrità dei database (InnoDB e WiredTiger) e il confinamento totale delle credenziali prima dello scollegamento fisico dell'unità USB/SSD.",
            "Guarantees database integrity (InnoDB and WiredTiger) and complete credential confinement before physically unplugging the USB/SSD drive.",
            "Garantit l'intégrité des bases de données (InnoDB et WiredTiger) et le confinement total des identifiants avant le débranchement physique du lecteur USB/SSD.",
            "Gewährleistet die Integrität der Datenbanken (InnoDB und WiredTiger) und die vollständige Isolierung aller Zugangsdaten vor dem physischen Trennen des USB-/SSD-Laufwerks.",
          )}
        />
      </EditorSection>
    </>
  );
}
