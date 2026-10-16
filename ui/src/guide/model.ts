// Structured model and reference catalogs for the in-app Guide (ui/src/guide/):
// architecture, disk layout tree, complete external sources & cryptographic
// verification table (36 entries), GitHub SSO, page-by-page feature tour,
// CLI reference and 12 step-by-step configuration procedures.
import type { IconType } from "react-icons";
import {
  VscBook,
  VscCheckAll,
  VscChecklist,
  VscCode,
  VscDatabase,
  VscDebugDisconnect,
  VscExtensions,
  VscFolderLibrary,
  VscGithub,
  VscGlobe,
  VscHubot,
  VscLayers,
  VscLock,
  VscPulse,
  VscServerProcess,
  VscSettingsGear,
  VscShield,
  VscSync,
  VscTable,
  VscTerminal,
  VscTools,
  VscVersions,
  VscWand,
} from "react-icons/vsc";
import { lang } from "../i18n";

/** Localized string helper for rich guide content (IT default, EN/FR/DE). */
export function tr(it: string, en?: string, fr?: string, de?: string): string {
  const l = lang();
  if (l === "en") return en ?? it;
  if (l === "fr") return fr ?? en ?? it;
  if (l === "de") return de ?? en ?? it;
  return it;
}

export type GuideGroupId = "arch" | "sources" | "github" | "features" | "recipes";

export type GuideTopicId =
  | "overview"
  | "disk-layout"
  | "lifecycle"
  | "supply-chain"
  | "sources-table"
  | "extra-sources"
  | "github-role"
  | "github-setup"
  | "feat-workbench"
  | "feat-projects"
  | "feat-agents"
  | "feat-services"
  | "feat-tools-cli"
  | "cfg-init-net"
  | "cfg-langs-agents"
  | "cfg-db-web"
  | "cfg-actions-orphans";

export type AppPageTarget =
  "new" | "projects" | "agents" | "databases" | "servers" | "components" | "settings" | "eject";

export interface GuideTopic {
  id: GuideTopicId;
  group: GuideGroupId;
  sectionCode: string;
  title: () => string;
  subtitle: () => string;
  icon: IconType;
  keywords: string;
  targetPage?: AppPageTarget;
}

export interface GuideGroup {
  id: GuideGroupId;
  title: () => string;
}

export const GUIDE_GROUPS: GuideGroup[] = [
  {
    id: "arch",
    title: () =>
      tr(
        "1. Architettura e Filosofia",
        "1. Architecture & Philosophy",
        "1. Architecture et Philosophie",
        "1. Architektur & Philosophie",
      ),
  },
  {
    id: "sources",
    title: () =>
      tr(
        "2. Fonti, Link e Sicurezza",
        "2. Sources, Links & Security",
        "2. Sources, Liens et Sécurité",
        "2. Quellen, Links & Sicherheit",
      ),
  },
  {
    id: "github",
    title: () =>
      tr(
        "3. Account GitHub Unificato",
        "3. Unified GitHub Account",
        "3. Compte GitHub Unifié",
        "3. Einheitliches GitHub-Konto",
      ),
  },
  {
    id: "features",
    title: () =>
      tr("4. Funzionalità e Pagine", "4. Features & Pages", "4. Fonctionnalités et Pages", "4. Funktionen & Seiten"),
  },
  {
    id: "recipes",
    title: () =>
      tr(
        "5. Configurazione Passo-Passo",
        "5. Step-by-Step Setup",
        "5. Configuration Pas à Pas",
        "5. Schritt-für-Schritt-Einrichtung",
      ),
  },
];

export const DEFAULT_GUIDE_TOPIC: GuideTopic = {
  id: "overview",
  group: "arch",
  sectionCode: "§ 1.1",
  title: () =>
    tr(
      "Principi di Portabilità e Isolamento",
      "Portability & Isolation Principles",
      "Principes de Portabilité et d'Isolation",
      "Portabilitäts- & Isolationsprinzipien",
    ),
  subtitle: () =>
    tr(
      "Zero installazione (asInvoker), ambiente pulito (env_clear), separazione netta tra projects\\ e data\\ e regola aurea dell'interfaccia.",
      "Zero install (asInvoker), clean environment (env_clear), strict projects\\ vs data\\ separation and UI golden rule.",
    ),
  icon: VscBook,
  keywords: "overview portabilita isolamento asinvoker env_clear path registro windows chiavetta usb filosofie",
};

export const GUIDE_TOPICS: GuideTopic[] = [
  // Group 1: Architecture
  DEFAULT_GUIDE_TOPIC,
  {
    id: "disk-layout",
    group: "arch",
    sectionCode: "§ 1.2",
    title: () =>
      tr(
        "Organizzazione su Disco (projects\\ e data\\)",
        "Disk Layout (projects\\ & data\\)",
        "Organisation sur Disque (projects\\ et data\\)",
        "Festplattenstruktur (projects\\ & data\\)",
      ),
    subtitle: () =>
      tr(
        "Gerarchia completa della radice del kit, file di configurazione, cartelle protette da DACL e procedura di backup rapido.",
        "Complete directory hierarchy of the kit root, configuration files, DACL-protected folders and quick backup strategy.",
      ),
    icon: VscLayers,
    keywords: "disco layout cartelle projects data apps settings panel.json dev-settings.json secrets.json dacl backup",
  },
  {
    id: "lifecycle",
    group: "arch",
    sectionCode: "§ 1.3",
    title: () =>
      tr(
        "Ciclo di Vita, Job Object e Rilocazione",
        "Process Lifecycle, Job Object & Relocation",
        "Cycle de Vie, Job Object et Relocalisation",
        "Lebenszyklus, Job Object & Umzug",
      ),
    subtitle: () =>
      tr(
        "Chiusura automatica dei demoni tramite Job Object di Windows, istanza singola per percorso e rilocazione automatica al cambio lettera USB.",
        "Automatic daemon cleanup via Windows Job Object, per-path single instance mutex and automatic relocation on USB drive letter change.",
      ),
    icon: VscPulse,
    keywords:
      "job object kill_on_job_close mutex istanza singola rilocazione root.txt old-root.txt spostamento lettera usb",
  },

  // Group 2: Sources & Verification
  {
    id: "supply-chain",
    group: "sources",
    sectionCode: "§ 2.1",
    title: () =>
      tr(
        "Approvvigionamento e 5 Verifiche Crittografiche",
        "Supply Chain & 5 Cryptographic Checks",
        "Approvisionnement et 5 Vérifications Cryptographiques",
        "Bereitstellung & 5 Kryptografische Prüfungen",
      ),
    subtitle: () =>
      tr(
        "Stack TLS nativo di Windows (SChannel), download atomici .part con 3 tentativi e verifiche SHA-256, SHA-512, SHA3-256, npm e Authenticode.",
        "Native Windows TLS (SChannel), atomic .part downloads with 3 retries and SHA-256, SHA-512, SHA3-256, npm integrity and Authenticode checks.",
      ),
    icon: VscShield,
    keywords:
      "tls schannel ureq sha256 sha512 sha3-256 authenticode winverifytrust npm integrity sicurezza verifica download",
  },
  {
    id: "sources-table",
    group: "sources",
    sectionCode: "§ 2.2",
    title: () =>
      tr(
        "Tabella Completa delle Fonti e dei Link",
        "Complete Table of Tool Sources & Links",
        "Tableau Complet des Sources et Liens",
        "Vollständige Tabelle der Werkzeugquellen & Links",
      ),
    subtitle: () =>
      tr(
        "Tutti i 36 componenti, linguaggi, database e runtime: URL ufficiali, endpoint metadati, binari, metodo di verifica e cartella in data\\apps\\.",
        "All 36 components, languages, databases and runtimes: official URLs, metadata endpoints, binaries, verification method and target folder.",
      ),
    icon: VscTable,
    keywords:
      "tabella fonti link url download github api nodejs python php composer apache nginx mariadb mongodb rust go java",
    targetPage: "components",
  },
  {
    id: "extra-sources",
    group: "sources",
    sectionCode: "§ 2.3–2.4",
    title: () =>
      tr(
        "Estensioni, Quote AI, Auto-Update e «By Hand»",
        "Extensions, AI Quotas, Auto-Update & “By Hand”",
        "Extensions, Quotas IA, Auto-Update et « By Hand »",
        "Erweiterungen, KI-Quoten, Auto-Update & „By Hand“",
      ),
    subtitle: () =>
      tr(
        "Open VSX, CodeLLDB, endpoint OAuth dei 3 agenti, auto-aggiornamento di Dev-Kit.exe, fallback HTML senza API e Drag & Drop manuale.",
        "Open VSX, CodeLLDB, OAuth usage endpoints for the 3 agents, Dev-Kit.exe self-update, API-free HTML fallback and manual Drag & Drop.",
      ),
    icon: VscCheckAll,
    keywords:
      "open vsx codelldb anthropic openai google cloudcode auto-update expanded_assets rate limit by hand drag drop",
    targetPage: "components",
  },

  // Group 3: GitHub
  {
    id: "github-role",
    group: "github",
    sectionCode: "§ 3.1–3.2",
    title: () =>
      tr(
        "Perché l'Account GitHub è Centrale (D-70)",
        "Why the GitHub Account is Central (D-70)",
        "Pourquoi le Compte GitHub est Central (D-70)",
        "Warum das GitHub-Konto Zentral ist (D-70)",
      ),
    subtitle: () =>
      tr(
        "Single Sign-On per App, Git, Editor, gh e Agenti AI; zero tracce nel Credential Manager di Windows; 5.000 richieste/ora e .gitconfig portabile.",
        "Single Sign-On for App, Git, Editor, gh and AI Agents; zero traces in Windows Credential Manager; 5,000 req/hr and portable .gitconfig.",
      ),
    icon: VscGithub,
    keywords: "github sso single sign-on hosts.yml gitconfig credential helper rate limit 5000 token sicurezza",
    targetPage: "settings",
  },
  {
    id: "github-setup",
    group: "github",
    sectionCode: "§ 3.3",
    title: () =>
      tr(
        "Configurazione GitHub e Permessi (workflow, delete_repo)",
        "GitHub Setup & Scopes (workflow, delete_repo)",
        "Configuration GitHub et Permissions (workflow, delete_repo)",
        "GitHub-Einrichtung & Berechtigungen (workflow, delete_repo)",
      ),
    subtitle: () =>
      tr(
        "Procedura passo-passo con codice monouso XXXX-XXXX, clonazione da repository personali/organizzazioni e permesso delete_repo su richiesta.",
        "Step-by-step Device flow with XXXX-XXXX code, cloning from personal/organization repos and on-demand delete_repo permission.",
      ),
    icon: VscLock,
    keywords: "github login device code workflow delete_repo clona repository organizzazione mario-rossi studio-rossi",
    targetPage: "projects",
  },

  // Group 4: Features & Pages
  {
    id: "feat-workbench",
    group: "features",
    sectionCode: "§ 4.1–4.2",
    title: () =>
      tr(
        "Finestra, Pannello Attività e «Nuovo progetto»",
        "Workbench, Activity Panel & “New Project”",
        "Fenêtre, Panneau Activité et « Nouveau projet »",
        "Hauptfenster, Aktivitätsbereich & „Neues Projekt“",
      ),
    subtitle: () =>
      tr(
        "Barra del titolo, avvisi globali, barra di stato, pannello Attività in tempo reale e Wizard Framework (Laravel, Symfony, Vite, Next.js, Electron, Tauri, Android).",
        "Title bar, global banners, status bar, live Activity panel and Framework Wizard (Laravel, Symfony, Vite, Next.js, Electron, Tauri, Android).",
      ),
    icon: VscWand,
    keywords:
      "finestra barra stato attivita wizard nuovo progetto laravel symfony vite nextjs electron tauri android react tailwind",
    targetPage: "new",
  },
  {
    id: "feat-projects",
    group: "features",
    sectionCode: "§ 4.3–4.4",
    title: () =>
      tr(
        "Pagina «Progetti», 7 Sezioni, Editor e Terminale",
        "“Projects” Page, 7 Sections, Editor & Terminal",
        "Page « Projets », 7 Sections, Éditeur et Terminal",
        "Seite „Projekte“, 7 Abschnitte, Editor & Terminal",
      ),
    subtitle: () =>
      tr(
        "Esploratore progetti, scheda a 7 sezioni (Avvio, Sito web, Componenti, Database, Estensioni, Editor, Git), VSCodium Web e named pipe del Terminale.",
        "Project explorer, 7-section detail view (Run, Website, Components, Database, Extensions, Editor, Git), VSCodium Web and Terminal named pipe.",
      ),
    icon: VscFolderLibrary,
    keywords:
      "progetti scheda avvio sito web componenti database estensioni editor vscodium terminale windows terminal named pipe",
    targetPage: "projects",
  },
  {
    id: "feat-agents",
    group: "features",
    sectionCode: "§ 4.5",
    title: () =>
      tr(
        "Pagina «Agenti», Quote e Sincronizzazione a 3 Vie",
        "“Agents” Page, Quotas & 3-Way Sync",
        "Page « Agents », Quotas et Synchronisation à 3 Voies",
        "Seite „Agenten“, Quoten & 3-Wege-Synchronisation",
      ),
    subtitle: () =>
      tr(
        "Claude Code, Codex e Antigravity: monitoraggio limiti 5h/settimana, grafico token 7 giorni, keyring DPAPI e allineamento di AGENTS.md, skills e MCP.",
        "Claude Code, Codex and Antigravity: 5h/weekly plan limits, 7-day token chart, DPAPI keyring and automatic sync of AGENTS.md, skills and MCP.",
      ),
    icon: VscHubot,
    keywords:
      "agenti claude code codex antigravity quote token statistiche dpapi keyring.json agents.md skills mcp.json",
    targetPage: "agents",
  },
  {
    id: "feat-services",
    group: "features",
    sectionCode: "§ 4.6–4.7",
    title: () =>
      tr(
        "Pagine «Database» e «Server Web» (.localhost)",
        "“Databases” & “Web Servers” (.localhost) Pages",
        "Pages « Bases de données » et « Serveurs Web » (.localhost)",
        "Seiten „Datenbanken“ & „Webserver“ (.localhost)",
      ),
    subtitle: () =>
      tr(
        "MariaDB + Adminer con login monouso, MongoDB + esploratore Contenuto in sola lettura, e pool condivisi Apache/Nginx con VirtualHost .localhost.",
        "MariaDB + Adminer one-time token login, MongoDB + read-only Content browser, and shared Apache/Nginx pools with .localhost VirtualHosts.",
      ),
    icon: VscDatabase,
    keywords: "database mariadb mongodb adminer dbbrowser server web apache nginx pool virtualhost localhost alias",
    targetPage: "databases",
  },
  {
    id: "feat-tools-cli",
    group: "features",
    sectionCode: "§ 4.8–4.11",
    title: () =>
      tr(
        "«Strumenti», «Impostazioni», «Rimuovi l'unità» e CLI",
        "“Tools”, “Settings”, “Remove Drive” & CLI",
        "« Outils », « Paramètres », « Retirer le lecteur » et CLI",
        "„Werkzeuge“, „Einstellungen“, „Laufwerk entfernen“ & CLI",
      ),
    subtitle: () =>
      tr(
        "Gestione multi-versione con lucchetto locked, matrice delle Relazioni, albero di personalizzazione Framework, espulsione sicura e CLI Dev-Kit.exe.",
        "Multi-version management with locked badge, Relations matrix, Framework customization tree, safe drive ejection and Dev-Kit.exe CLI.",
      ),
    icon: VscExtensions,
    keywords:
      "strumenti multi-versione locked relazioni estensioni impostazioni framework rimuovi unita eject cli powershell out-host",
    targetPage: "components",
  },

  // Group 5: Step-by-Step Configuration Recipes
  {
    id: "cfg-init-net",
    group: "recipes",
    sectionCode: "§ 5.1–5.3",
    title: () =>
      tr(
        "1–3. Disco, Generale (programTemp) e Rete (Proxy/CA/LAN)",
        "1–3. Disk, General (programTemp) & Network (Proxy/CA/LAN)",
        "1–3. Disque, Général (programTemp) et Réseau (Proxy/CA/LAN)",
        "1–3. Laufwerk, Allgemein (programTemp) & Netzwerk (Proxy/CA/LAN)",
      ),
    subtitle: () =>
      tr(
        "Scelta del percorso sicuro per il file watcher, formattazione NTFS, posizionamento dei temporanei, proxy aziendali, certificati PEM e accesso LAN.",
        "Safe path naming for the file watcher, NTFS formatting, temporary files placement, corporate proxies, PEM certificates and LAN access.",
      ),
    icon: VscSettingsGear,
    keywords:
      "configurazione primo avvio ntfs watcher programtemp automatico proxy https node_extra_ca_certs pem allowlan loopback",
    targetPage: "settings",
  },
  {
    id: "cfg-langs-agents",
    group: "recipes",
    sectionCode: "§ 5.4–5.6",
    title: () =>
      tr(
        "4–6. Compilatori, Versioni per Progetto e Agenti / MCP",
        "4–6. Compilers, Per-Project Versions & Agents / MCP",
        "4–6. Compilateurs, Versions par Projet et Agents / MCP",
        "4–6. Compiler, Projektversionen & Agenten / MCP",
      ),
    subtitle: () =>
      tr(
        "Toolchain Rust GNU vs MSVC, fissaggio versioni in dev-settings.json (pin_newest), coesistenza side-by-side, chiave Gemini e server MCP.",
        "Rust GNU vs MSVC toolchain, version pinning in dev-settings.json (pin_newest), side-by-side coexistence, Gemini API key and MCP servers.",
      ),
    icon: VscVersions,
    keywords:
      "compilatore rust gnu msvc python php versioni progetto dev-settings.json pin_newest side-by-side gemini_api_key mcp.json",
    targetPage: "agents",
  },
  {
    id: "cfg-db-web",
    group: "recipes",
    sectionCode: "§ 5.7–5.8",
    title: () =>
      tr(
        "7–8. Database Isolati, .env, Backup e Siti Apache/Nginx",
        "7–8. Isolated Databases, .env, Backups & Apache/Nginx Sites",
        "7–8. Bases Isolées, .env, Sauvegardes et Sites Apache/Nginx",
        "7–8. Isolierte Datenbanken, .env, Backups & Apache/Nginx-Sites",
      ),
    subtitle: () =>
      tr(
        "Creazione utente DB ristretto a 32 cifre hex, scrittura automatica in .env, dump/ripristino sicuro, domini .localhost e alias @altro-progetto.",
        "Restricted 32-hex DB user creation, one-click .env writing, safe dump/restore, .localhost domains and @other-project aliases.",
      ),
    icon: VscGlobe,
    keywords:
      "database utente isolato .env backup sql archive.gz ripristino sito web apache nginx virtualhost localhost alias",
    targetPage: "servers",
  },
  {
    id: "cfg-actions-orphans",
    group: "recipes",
    sectionCode: "§ 5.9–5.12",
    title: () =>
      tr(
        "9–12. Azioni (panel.json), Estensioni, Framework e Orfani",
        "9–12. Actions (panel.json), Extensions, Frameworks & Orphans",
        "9–12. Actions (panel.json), Extensions, Frameworks et Orphelins",
        "9–12. Aktionen (panel.json), Erweiterungen, Frameworks & Waisen",
      ),
    subtitle: () =>
      tr(
        "Azioni personalizzate e blocco porte, isolamento estensioni per area di lavoro (D-71), modifica ricette del Wizard e gestione Dati senza progetto.",
        "Custom actions and port locking, per-workspace extension isolation (D-71), Wizard recipe customization and Orphaned project data management.",
      ),
    icon: VscChecklist,
    keywords:
      "panel.json ports.json azioni porta bloccata estensioni indexeddb settings.json frameworks.json dati senza progetto orfani",
    targetPage: "projects",
  },
];

export type SourceCategory = "required" | "app" | "os" | "agents" | "languages" | "tools" | "databases" | "part";

export interface DiskTreeNode {
  path: string;
  depth: number;
  desc: () => string;
}

export const DISK_TREE: DiskTreeNode[] = [
  {
    path: "<Radice>\\",
    depth: 0,
    desc: () =>
      tr(
        "Cartella radice portabile del kit (es. D:\\Dev-Kit\\), spostabile su qualsiasi unità o PC.",
        "Portable root folder of the kit (e.g. D:\\Dev-Kit\\), movable across any drive or PC.",
      ),
  },
  {
    path: "Dev-Kit.exe",
    depth: 1,
    desc: () =>
      tr(
        "Unico eseguibile nativo (Rust + Tauri + UI). Funge sia da interfaccia grafica sia da CLI.",
        "Single native executable (Rust + Tauri + UI). Acts as both GUI and CLI.",
      ),
  },
  {
    path: "projects\\",
    depth: 1,
    desc: () =>
      tr(
        "Tutti i tuoi progetti di sviluppo: ogni sottocartella è rilevata automaticamente.",
        "All your development projects: every subfolder is automatically detected.",
      ),
  },
  {
    path: "data\\app\\",
    depth: 1,
    desc: () =>
      tr(
        "Stato e impostazioni del kit (state.json, settings.json, frameworks, ports.json).",
        "Kit state and settings (state.json, settings.json, frameworks, ports.json).",
      ),
  },
  {
    path: "data\\apps\\",
    depth: 1,
    desc: () =>
      tr(
        "Binari estratti e verificati di tutti i 28 componenti e delle linee multi-versione.",
        "Extracted and verified binaries of all 28 components and multi-version lines.",
      ),
  },
  {
    path: "data\\agents\\",
    depth: 1,
    desc: () =>
      tr(
        "Fonte di verità condivisa dei 3 agenti IA: AGENTS.md, skills\\ e mcp.json.",
        "Shared source of truth for the 3 AI agents: AGENTS.md, skills\\ and mcp.json.",
      ),
  },
  {
    path: "data\\claude\\ · data\\codex\\ · data\\antigravity\\",
    depth: 1,
    desc: () =>
      tr(
        "Profili isolati, sessioni e keyring cifrato DPAPI (.keyring.dpapi) dei 3 agenti.",
        "Isolated profiles, sessions and DPAPI-encrypted keyring (.keyring.dpapi) for the 3 agents.",
      ),
  },
  {
    path: "data\\db\\ · data\\backup\\",
    depth: 1,
    desc: () =>
      tr(
        "Dati fisici delle tabelle MariaDB e collezioni MongoDB + backup esportati.",
        "Physical data of MariaDB tables and MongoDB collections + exported backups.",
      ),
  },
  {
    path: "data\\gh\\ · data\\home\\.gitconfig · data\\ssh\\",
    depth: 1,
    desc: () =>
      tr(
        "Token OAuth di GitHub CLI (hosts.yml), configurazione Git globale e chiavi SSH portabili.",
        "GitHub CLI OAuth token (hosts.yml), global Git config and portable SSH keys.",
      ),
  },
  {
    path: "data\\vscodium-server\\",
    depth: 1,
    desc: () =>
      tr(
        "Profilo utente, impostazioni ed estensioni Open VSX dell'editor VSCodium.",
        "User profile, settings and Open VSX extensions for the VSCodium editor.",
      ),
  },
];

export interface ToolSourceRow {
  id: string;
  logoId: string;
  name: string;
  group: SourceCategory;
  home: string;
  metaSource: string;
  binarySource: string;
  verifyKind: "SHA-256" | "SHA-512" | "SHA3-256" | "npm sha512" | "Authenticode + SHA";
  verifyDetail: string;
  targetPath: string;
}

export const TOOL_SOURCES: ToolSourceRow[] = [
  {
    id: "pwsh",
    logoId: "pwsh",
    name: "PowerShell 7 (pwsh)",
    group: "required",
    home: "https://github.com/PowerShell/PowerShell",
    metaSource: "api.github.com/repos/PowerShell/PowerShell/releases/latest",
    binarySource: "PowerShell-<v>-win-(x64|arm64).zip",
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: "pwsh\\pwsh.exe",
  },
  {
    id: "gh",
    logoId: "gh",
    name: "GitHub CLI (gh)",
    group: "required",
    home: "https://cli.github.com",
    metaSource: "api.github.com/repos/cli/cli/releases/latest",
    binarySource: "gh_<v>_windows_(amd64|arm64).zip",
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: "tools\\gh.exe",
  },
  {
    id: "vscodium-server",
    logoId: "vscodium-server",
    name: "Editor (VSCodium Reh Web)",
    group: "app",
    home: "https://vscodium.com",
    metaSource: "Catalogo integrato (versione fissata 1.135.06055, D-58)",
    binarySource: "github.com/VSCodium/vscodium/releases/download/<v>/vscodium-reh-web-win32-x64-<v>.tar.gz",
    verifyKind: "SHA-256",
    verifyDetail: "SHA-256 cablato in components.json",
    targetPath: "vscodium-server\\",
  },
  {
    id: "terminal",
    logoId: "terminal",
    name: "Windows Terminal Portabile",
    group: "os",
    home: "https://github.com/microsoft/terminal",
    metaSource: "api.github.com/repos/microsoft/terminal/releases/latest",
    binarySource: "Microsoft.WindowsTerminal_<v>_(x64|arm64).zip",
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: "terminal\\WindowsTerminal.exe (+ .portable)",
  },
  {
    id: "claude",
    logoId: "claude",
    name: "Claude Code",
    group: "agents",
    home: "https://claude.com/product/claude-code",
    metaSource: "registry.npmjs.org/@anthropic-ai/claude-code-win32-<arch>/latest",
    binarySource: "Campo dist.tarball del pacchetto npm",
    verifyKind: "Authenticode + SHA",
    verifyDetail: "Integrità npm sha512 + firma Authenticode Anthropic",
    targetPath: "claude\\claude.exe",
  },
  {
    id: "codex",
    logoId: "codex",
    name: "Codex (OpenAI)",
    group: "agents",
    home: "https://github.com/openai/codex",
    metaSource: "registry.npmjs.org/@openai/codex/latest → /<ver>-win32-<arch>",
    binarySource: "Campo dist.tarball del pacchetto specifico per architettura",
    verifyKind: "npm sha512",
    verifyDetail: "Integrità npm dist.integrity (sha512-...)",
    targetPath: "codex\\bin\\codex.exe",
  },
  {
    id: "antigravity",
    logoId: "antigravity",
    name: "Antigravity (Google)",
    group: "agents",
    home: "https://antigravity.google",
    metaSource: "antigravity-cli-auto-updater-974169037036.us-central1.run.app/manifests/windows_(amd64|arm64).json",
    binarySource: "Campo url del manifesto JSON ufficiale",
    verifyKind: "SHA-512",
    verifyDetail: "SHA-512 (128 hex) dal campo sha512 del manifesto",
    targetPath: "antigravity\\agy.exe (via data\\kit\\bin\\agy.cmd)",
  },
  {
    id: "git",
    logoId: "git",
    name: "Git + Git Bash (PortableGit)",
    group: "tools",
    home: "https://git-scm.com",
    metaSource: "api.github.com/repos/git-for-windows/git/releases/latest",
    binarySource: "PortableGit-<v>-(64-bit|arm64).7z.exe (estratto in Rust)",
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: "git\\cmd\\git.exe e git\\bin\\bash.exe",
  },
  {
    id: "python",
    logoId: "python",
    name: "Python + pip",
    group: "languages",
    home: "https://www.python.org",
    metaSource: "raw.githubusercontent.com/astral-sh/python-build-standalone/latest-release/latest-release.json",
    binarySource: "cpython-3.<minor>.<patch>+<date>-<triple>-install_only_stripped.tar.gz",
    verifyKind: "SHA-256",
    verifyDetail: "SHA-256 dall'elenco ufficiale SHA256SUMS",
    targetPath: "python\\<linea>\\python.exe (+ pip)",
  },
  {
    id: "node",
    logoId: "node",
    name: "Node.js + npm",
    group: "languages",
    home: "https://nodejs.org",
    metaSource: "nodejs.org/dist/index.json (LTS pari >= 18 + ultima major)",
    binarySource: "nodejs.org/dist/<v>/node-<v>-win-<arch>.zip",
    verifyKind: "SHA-256",
    verifyDetail: "SHA-256 da nodejs.org/dist/<v>/SHASUMS256.txt",
    targetPath: "node\\<linea>\\node.exe",
  },
  {
    id: "uv",
    logoId: "uv",
    name: "uv (uv, uvx, uvw)",
    group: "tools",
    home: "https://docs.astral.sh/uv/",
    metaSource: "api.github.com/repos/astral-sh/uv/releases/latest",
    binarySource: "uv-<triple>.zip",
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: "tools\\uv.exe, tools\\uvx.exe, tools\\uvw.exe",
  },
  {
    id: "php",
    logoId: "php",
    name: "PHP (Thread Safe)",
    group: "languages",
    home: "https://www.php.net",
    metaSource: "windows.php.net/downloads/releases/releases.json (+ archivi EOL 8.1..5.6)",
    binarySource: "windows.php.net/downloads/releases/php-<v>-Win32-vs*-x64.zip",
    verifyKind: "SHA-256",
    verifyDetail: "SHA-256 da releases.json (o tabella EOL verificata)",
    targetPath: "php\\<linea>\\php.exe",
  },
  {
    id: "composer",
    logoId: "php",
    name: "↳ Parte: Composer",
    group: "part",
    home: "https://getcomposer.org",
    metaSource: "getcomposer.org/versions (canale stable o 2.2 LTS per PHP < 7.2.5)",
    binarySource: "getcomposer.org/download/<v>/composer.phar",
    verifyKind: "SHA-256",
    verifyDetail: "SHA-256 da <url>.sha256sum",
    targetPath: "php\\<linea>\\composer.phar (+ composer.cmd)",
  },
  {
    id: "cacert",
    logoId: "php",
    name: "↳ Parte: Bundle CA SSL (curl)",
    group: "part",
    home: "https://curl.se",
    metaSource: "curl.se/ca/cacert.pem.sha256",
    binarySource: "curl.se/ca/cacert.pem",
    verifyKind: "SHA-256",
    verifyDetail: "SHA-256 da cacert.pem.sha256",
    targetPath: "php\\<linea>\\extras\\ssl\\cacert.pem",
  },
  {
    id: "vcruntime",
    logoId: "cc",
    name: "↳ Parte: Visual C++ Runtime",
    group: "part",
    home: "https://learn.microsoft.com",
    metaSource: "Estratto dal CAB vcRuntimeMinimum_amd64\\cab1.cab se assente",
    binarySource: "aka.ms/vc14/vc_redist.x64.exe",
    verifyKind: "Authenticode + SHA",
    verifyDetail: "Firma Authenticode O=Microsoft Corporation su exe e vcruntime140.dll",
    targetPath: "tools\\vcruntime\\ (copiato in PHP, Apache, MongoDB)",
  },
  {
    id: "apache",
    logoId: "apache",
    name: "Apache httpd",
    group: "languages",
    home: "https://httpd.apache.org",
    metaSource: "apachelounge.com/download/ (scraping versione e checksum)",
    binarySource: "apachelounge.com/download/VS<nn>/binaries/httpd-<v>-Win64-VS<nn>.zip",
    verifyKind: "SHA-256",
    verifyDetail: "SHA-256 da <url>.txt (SHA256-Checksum for:)",
    targetPath: "apache\\<linea>\\bin\\httpd.exe",
  },
  {
    id: "nginx",
    logoId: "nginx",
    name: "Nginx",
    group: "languages",
    home: "https://nginx.org",
    metaSource: "api.github.com/repos/nginx/nginx/releases?per_page=100",
    binarySource: "Asset nginx-<v>.zip dalla release GitHub ufficiale",
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: "nginx\\<linea>\\nginx.exe",
  },
  {
    id: "rust",
    logoId: "rust",
    name: "Rust (rustup, cargo, clippy)",
    group: "tools",
    home: "https://www.rust-lang.org",
    metaSource: "static.rust-lang.org/dist/channel-rust-stable.toml ([pkg.rustc])",
    binarySource: "static.rust-lang.org/rustup/dist/<triple>/rustup-init.exe",
    verifyKind: "SHA-256",
    verifyDetail: "SHA-256 da <url>.sha256",
    targetPath: "rust\\cargo\\bin\\rustc.exe e rustup.exe",
  },
  {
    id: "go",
    logoId: "go",
    name: "Go SDK",
    group: "languages",
    home: "https://go.dev",
    metaSource: "go.dev/dl/?mode=json (&include=all)",
    binarySource: "go.dev/dl/go<v>.windows-(amd64|arm64).zip",
    verifyKind: "SHA-256",
    verifyDetail: "SHA-256 dal campo sha256 del JSON ufficiale",
    targetPath: "go\\<linea>\\bin\\go.exe",
  },
  {
    id: "java",
    logoId: "java",
    name: "Java JDK (Eclipse Temurin)",
    group: "languages",
    home: "https://adoptium.net",
    metaSource: "api.adoptium.net/v3/info/available_releases + /v3/assets/latest/<major>/hotspot",
    binarySource: "Pacchetto .zip restituito dall'API Adoptium",
    verifyKind: "SHA-256",
    verifyDetail: "SHA-256 da binary.package.checksum",
    targetPath: "java\\<linea>\\bin\\java.exe",
  },
  {
    id: "maven",
    logoId: "java",
    name: "↳ Parte: Apache Maven",
    group: "part",
    home: "https://maven.apache.org",
    metaSource: "repo.maven.apache.org/maven2/org/apache/maven/apache-maven/maven-metadata.xml",
    binarySource: "apache-maven-<v>-bin.zip",
    verifyKind: "SHA-512",
    verifyDetail: "SHA-512 da <url>.sha512",
    targetPath: "maven\\bin\\mvn.cmd",
  },
  {
    id: "gradle",
    logoId: "java",
    name: "↳ Parte: Gradle",
    group: "part",
    home: "https://gradle.org",
    metaSource: "services.gradle.org/versions/current",
    binarySource: "Campo downloadUrl del JSON ufficiale",
    verifyKind: "SHA-256",
    verifyDetail: "SHA-256 da checksumUrl del JSON",
    targetPath: "gradle\\bin\\gradle.bat (via data\\kit\\bin\\gradle.cmd)",
  },
  {
    id: "kotlin",
    logoId: "kotlin",
    name: "Kotlin Compiler",
    group: "languages",
    home: "https://kotlinlang.org",
    metaSource: "api.github.com/repos/JetBrains/kotlin/releases?per_page=100",
    binarySource: "Asset kotlin-compiler-<v>.zip",
    verifyKind: "SHA-256",
    verifyDetail: "GitHub asset.digest o <asset>.sha256",
    targetPath: "kotlin\\<linea>\\bin\\kotlinc.bat",
  },
  {
    id: "android",
    logoId: "android",
    name: "Android SDK + Command-line Tools",
    group: "tools",
    home: "https://developer.android.com/studio",
    metaSource: "developer.android.com/studio (tabella commandlinetools-win-*_latest.zip)",
    binarySource: "dl.google.com/android/repository/commandlinetools-win-<build>_latest.zip",
    verifyKind: "SHA-256",
    verifyDetail: "SHA-256 pubblicato nella tabella ufficiale Android",
    targetPath: "android\\cmdline-tools\\latest\\bin\\sdkmanager.bat e adb.exe",
  },
  {
    id: "dotnet",
    logoId: "dotnet",
    name: ".NET SDK",
    group: "languages",
    home: "https://dotnet.microsoft.com",
    metaSource: "builds.dotnet.microsoft.com/dotnet/release-metadata/releases-index.json",
    binarySource: "Archivio .zip per rid=win-x64|win-arm64",
    verifyKind: "SHA-512",
    verifyDetail: "SHA-512 dal campo hash di releases.json",
    targetPath: "dotnet\\<linea>\\dotnet.exe (con junction SDK condivise)",
  },
  {
    id: "dart",
    logoId: "dart",
    name: "Dart SDK",
    group: "languages",
    home: "https://dart.dev",
    metaSource: "storage.googleapis.com/dart-archive/channels/stable/release/latest/VERSION",
    binarySource: "dartsdk-windows-<arch>-release.zip",
    verifyKind: "SHA-256",
    verifyDetail: "SHA-256 da <url>.sha256sum",
    targetPath: "dart\\<linea>\\bin\\dart.exe",
  },
  {
    id: "zig",
    logoId: "zig",
    name: "Zig",
    group: "languages",
    home: "https://ziglang.org",
    metaSource: "ziglang.org/download/index.json",
    binarySource: "Campo tarball per (x86_64|aarch64)-windows",
    verifyKind: "SHA-256",
    verifyDetail: "SHA-256 dal campo shasum del JSON",
    targetPath: "zig\\<linea>\\zig.exe",
  },
  {
    id: "julia",
    logoId: "julia",
    name: "Julia",
    group: "languages",
    home: "https://julialang.org",
    metaSource: "julialang-s3.julialang.org/bin/versions.json",
    binarySource: "Archivio .zip per os=winnt, arch=x86_64",
    verifyKind: "SHA-256",
    verifyDetail: "SHA-256 dal campo sha256 di versions.json",
    targetPath: "julia\\<linea>\\bin\\julia.exe",
  },
  {
    id: "bun",
    logoId: "bun",
    name: "Bun",
    group: "languages",
    home: "https://bun.sh",
    metaSource: "api.github.com/repos/oven-sh/bun/releases",
    binarySource: "Asset bun-windows-(x64|aarch64).zip",
    verifyKind: "SHA-256",
    verifyDetail: "GitHub asset.digest o SHASUMS256.txt",
    targetPath: "bun\\<linea>\\bun.exe",
  },
  {
    id: "deno",
    logoId: "deno",
    name: "Deno",
    group: "languages",
    home: "https://deno.com",
    metaSource: "api.github.com/repos/denoland/deno/releases",
    binarySource: "Asset deno-(x86_64|aarch64)-pc-windows-msvc.zip",
    verifyKind: "SHA-256",
    verifyDetail: "GitHub asset.digest o <asset>.sha256sum",
    targetPath: "deno\\<linea>\\deno.exe",
  },
  {
    id: "typst",
    logoId: "typst",
    name: "Typst",
    group: "languages",
    home: "https://typst.app",
    metaSource: "api.github.com/repos/typst/typst/releases",
    binarySource: "Asset typst-(x86_64|aarch64)-pc-windows-msvc.zip",
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: "typst\\<linea>\\typst.exe",
  },
  {
    id: "tinytex",
    logoId: "tinytex",
    name: "TinyTeX (LaTeX)",
    group: "tools",
    home: "https://yihui.org/tinytex/",
    metaSource: "api.github.com/repos/rstudio/tinytex-releases/releases/latest",
    binarySource: "Asset TinyTeX-1-windows-v*.exe (7z SFX estratto in Rust)",
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: "tinytex\\bin\\windows\\tlmgr.bat e pdftex.exe",
  },
  {
    id: "pnpm",
    logoId: "pnpm",
    name: "pnpm + yarn (corepack)",
    group: "tools",
    home: "https://pnpm.io",
    metaSource: "registry.npmjs.org/pnpm/latest e /corepack/latest",
    binarySource: "Installato con npm-cli.js del Node.js del kit",
    verifyKind: "npm sha512",
    verifyDetail: "Integrità del registro npm",
    targetPath: "data\\npm-global\\pnpm.cmd e yarn.cmd",
  },
  {
    id: "cmake",
    logoId: "cmake",
    name: "CMake + Ninja",
    group: "tools",
    home: "https://cmake.org",
    metaSource: "GitHub API: Kitware/CMake e ninja-build/ninja (releases/latest)",
    binarySource: "cmake-*-windows-(x86_64|arm64).zip e ninja-win(arm64).zip",
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: "cmake\\bin\\cmake.exe e tools\\ninja.exe",
  },
  {
    id: "cc",
    logoId: "cc",
    name: "C/C++ (llvm-mingw)",
    group: "tools",
    home: "https://github.com/mstorsjo/llvm-mingw",
    metaSource: "api.github.com/repos/mstorsjo/llvm-mingw/releases/latest",
    binarySource: "Asset llvm-mingw-*-ucrt-(x86_64|aarch64).zip",
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: "cc\\bin\\clang.exe (+ make.cmd)",
  },
  {
    id: "ruby",
    logoId: "ruby",
    name: "Ruby + Bundler",
    group: "languages",
    home: "https://www.ruby-lang.org",
    metaSource: "api.github.com/repos/oneclick/rubyinstaller2/releases?per_page=100",
    binarySource: "Asset rubyinstaller-<v>-(x64|arm).7z",
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: "ruby\\<linea>\\bin\\ruby.exe",
  },
  {
    id: "r",
    logoId: "r",
    name: "R (conda-forge + OpenBLAS)",
    group: "languages",
    home: "https://www.r-project.org",
    metaSource: "api.anaconda.org/package/conda-forge/r-base + mamba-org/micromamba-releases",
    binarySource: "micromamba-win-64 + pacchetti r-base e libblas=*=*openblas",
    verifyKind: "SHA-256",
    verifyDetail: "SHA-256 di micromamba + SHA-256 pacchetti conda-forge",
    targetPath: "r\\<linea>\\bin\\R.cmd e Rscript.cmd",
  },
  {
    id: "mariadb",
    logoId: "mariadb",
    name: "MariaDB Server",
    group: "databases",
    home: "https://mariadb.org",
    metaSource: "downloads.mariadb.org/rest-api/mariadb/ (linee Stable LTS)",
    binarySource: "downloads.mariadb.org/rest-api/mariadb/<v>/mariadb-<v>-winx64.zip",
    verifyKind: "SHA-256",
    verifyDetail: "SHA-256 dal campo checksum.sha256sum dell'API",
    targetPath: "mariadb\\<linea>\\bin\\mariadbd.exe e mariadb.exe",
  },
  {
    id: "mongodb",
    logoId: "mongodb",
    name: "MongoDB + mongosh + Database Tools",
    group: "databases",
    home: "https://www.mongodb.com",
    metaSource: "downloads.mongodb.org/current.json, GitHub mongodb-js/mongosh, tools/db/release.json",
    binarySource: "Archivi .zip Windows x64 di mongod, mongosh e mongodump/mongorestore",
    verifyKind: "SHA-256",
    verifyDetail: "SHA-256 dai JSON MongoDB + asset.digest GitHub per mongosh",
    targetPath: "mongodb\\<linea>\\bin\\mongod.exe, mongosh\\, mongodb-tools\\",
  },
  {
    id: "adminer",
    logoId: "adminer",
    name: "Adminer (per MariaDB)",
    group: "databases",
    home: "https://www.adminer.org",
    metaSource: "api.github.com/repos/vrana/adminer/releases/latest",
    binarySource: "Asset adminer-<v>-mysql.php",
    verifyKind: "SHA-256",
    verifyDetail: "GitHub asset.digest + controllo sintattico php -l",
    targetPath: "adminer\\adminer.php",
  },
  {
    id: "sqlite",
    logoId: "sqlite",
    name: "SQLite CLI",
    group: "databases",
    home: "https://sqlite.org",
    metaSource: "sqlite.org/download.html (righe CSV PRODUCT,...)",
    binarySource: "sqlite.org/<anno>/sqlite-tools-win-(x64|arm64)-<v>.zip",
    verifyKind: "SHA3-256",
    verifyDetail: "SHA3-256 pubblicato nella riga PRODUCT della pagina ufficiale",
    targetPath: "sqlite\\sqlite3.exe",
  },
];

export interface CliCommandRow {
  cmd: string;
  desc: () => string;
}

export const CLI_COMMANDS: CliCommandRow[] = [
  {
    cmd: ".\\Dev-Kit.exe env | Out-Host",
    desc: () =>
      tr(
        "Stampa tutte le ~90 variabili d'ambiente isolate calcolate per il kit.",
        "Prints all ~90 isolated environment variables computed for the kit.",
      ),
  },
  {
    cmd: ".\\Dev-Kit.exe components | Out-Host",
    desc: () =>
      tr(
        "Elenca tutti i componenti del catalogo, se sono scelti e la versione installata su disco.",
        "Lists all catalog components, whether they are selected and the installed version on disk.",
      ),
  },
  {
    cmd: ".\\Dev-Kit.exe versions <id> | Out-Host",
    desc: () =>
      tr(
        "Interroga la fonte ufficiale e mostra le linee major.minor disponibili per un componente versionato (es. node, php, python).",
        "Queries the official source and shows available major.minor lines for a versioned component (e.g. node, php, python).",
      ),
  },
  {
    cmd: ".\\Dev-Kit.exe check | Out-Host",
    desc: () =>
      tr(
        "Esegue il comando di verifica finale su tutti i componenti installati per accertarne l'integrità.",
        "Runs the verification command on all installed components to check their integrity.",
      ),
  },
  {
    cmd: ".\\Dev-Kit.exe prepare [--only a,b] [--skip c] [--update-all] [--force] | Out-Host",
    desc: () =>
      tr(
        "Installa, aggiorna o rimuove da riga di comando i componenti scelti.",
        "Installs, updates or removes selected components from the command line.",
      ),
  },
  {
    cmd: ".\\Dev-Kit.exe sync [--project <cartella>] [--verbose] | Out-Host",
    desc: () =>
      tr(
        "Allinea istruzioni (AGENTS.md), skill e server MCP fra Claude Code, Codex e Antigravity.",
        "Synchronizes instructions (AGENTS.md), skills and MCP servers across Claude Code, Codex and Antigravity.",
      ),
  },
  {
    cmd: ".\\Dev-Kit.exe keyring restore|save | Out-Host",
    desc: () =>
      tr(
        "Ripristina o salva nel kit (con cifratura DPAPI) l'accesso di Antigravity (usato da agy.cmd).",
        "Restores or saves Antigravity's credentials into the kit (encrypted with DPAPI, used by agy.cmd).",
      ),
  },
  {
    cmd: ".\\Dev-Kit.exe serve <cartella> <porta> | Out-Host",
    desc: () =>
      tr(
        "Avvia il server HTTP statico integrato sulla cartella e porta indicate.",
        "Starts the built-in static HTTP server on the given folder and port.",
      ),
  },
  {
    cmd: ".\\Dev-Kit.exe version | Out-Host",
    desc: () =>
      tr("Mostra la versione dell'eseguibile Dev-Kit.exe.", "Shows the version of the Dev-Kit.exe executable."),
  },
  {
    cmd: ".\\Dev-Kit.exe help | Out-Host",
    desc: () =>
      tr("Mostra il riepilogo di tutti i comandi CLI disponibili.", "Shows the summary of all available CLI commands."),
  },
];

export {
  VscBook,
  VscCheckAll,
  VscChecklist,
  VscCode,
  VscDatabase,
  VscDebugDisconnect,
  VscExtensions,
  VscFolderLibrary,
  VscGithub,
  VscGlobe,
  VscHubot,
  VscLayers,
  VscLock,
  VscPulse,
  VscServerProcess,
  VscSettingsGear,
  VscShield,
  VscSync,
  VscTable,
  VscTerminal,
  VscTools,
  VscVersions,
  VscWand,
};
