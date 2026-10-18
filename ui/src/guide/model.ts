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

/** Localized string helper for rich guide content: the Italian source and
 * its English, French and German, all required (no silent fallback). */
export function tr(it: string, en: string, fr: string, de: string): string {
  const l = lang();
  if (l === "en") return en;
  if (l === "fr") return fr;
  if (l === "de") return de;
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
      "Zéro installation (asInvoker), environnement propre (env_clear), séparation stricte entre projects\\ et data\\ et règle d'or de l'interface.",
      "Null-Installation (asInvoker), saubere Umgebung (env_clear), strikte Trennung zwischen projects\\ und data\\ sowie goldene Regel der Oberfläche.",
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
        "Hiérarchie complète de la racine du kit, fichiers de configuration, dossiers protégés par DACL et procédure de sauvegarde rapide.",
        "Vollständige Verzeichnishierarchie des Kit-Stammordners, Konfigurationsdateien, DACL-geschützte Ordner und schnelle Backup-Strategie.",
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
        "Fermeture automatique des démons via Job Object Windows, instance unique par chemin et relocalisation automatique au changement de lettre USB.",
        "Automatisches Beenden von Hintergrundprozessen über Windows Job Object, Einzelinstanz pro Pfad und automatischer Umzug bei Wechsel des USB-Laufwerksbuchstabens.",
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
        "Pile TLS native de Windows (SChannel), téléchargements atomiques .part avec 3 tentatives et vérifications SHA-256, SHA-512, SHA3-256, npm et Authenticode.",
        "Nativer Windows-TLS-Stack (SChannel), atomare .part-Downloads mit 3 Versuchen sowie Prüfungen via SHA-256, SHA-512, SHA3-256, npm und Authenticode.",
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
        "Les 36 composants, langages, bases de données et runtimes : URL officielles, points de terminaison de métadonnées, binaires, méthode de vérification et dossier dans data\\apps\\.",
        "Alle 36 Komponenten, Sprachen, Datenbanken und Laufzeitumgebungen: offizielle URLs, Metadaten-Endpunkte, Binärdateien, Prüfmethode und Zielordner in data\\apps\\.",
      ),
    icon: VscTable,
    keywords:
      "tabella fonti link url download github api nodejs python php composer caddy apache mysql mongodb rust go java",
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
        "Open VSX, CodeLLDB, points de terminaison OAuth des 3 agents, mise à jour automatique de Dev-Kit.exe, repli HTML sans API et glisser-déposer manuel.",
        "Open VSX, CodeLLDB, OAuth-Nutzungsendpunkte der 3 Agenten, Selbstaktualisierung von Dev-Kit.exe, API-freier HTML-Fallback und manuelles Drag & Drop.",
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
        "Authentification unique (SSO) pour l'application, Git, l'éditeur, gh et les agents IA ; zéro trace dans le Gestionnaire d'identification Windows ; 5 000 req/h et .gitconfig portable.",
        "Single Sign-On für App, Git, Editor, gh und KI-Agenten; null Spuren in der Windows-Anmeldeinformationsverwaltung; 5.000 Anfragen/Std. und portables .gitconfig.",
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
        "Procédure pas à pas avec code à usage unique XXXX-XXXX, clonage depuis des dépôts personnels/d'organisation et permission delete_repo à la demande.",
        "Schritt-für-Schritt-Device-Flow mit Einmalcode XXXX-XXXX, Klonen aus persönlichen/Organisations-Repositorys und delete_repo-Berechtigung bei Bedarf.",
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
        "Barre de titre, bannières globales, barre d'état, panneau Activité en temps réel et Assistant Framework (Laravel, Symfony, Vite, Next.js, Electron, Tauri, Android).",
        "Titelleiste, globale Hinweise, Statusleiste, Echtzeit-Aktivitätsbereich und Framework-Assistent (Laravel, Symfony, Vite, Next.js, Electron, Tauri, Android).",
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
        "Esploratore progetti, scheda a 7 sezioni (Avvio, Sito web, Componenti, Database, Estensioni, Editor, Git), VSCodium Web e console del kit.",
        "Project explorer, 7-section detail view (Run, Website, Components, Database, Extensions, Editor, Git), VSCodium Web and kit consoles.",
        "Explorateur de projets, vue détaillée à 7 sections (Exécution, Site web, Composants, Base de données, Extensions, Éditeur, Git), VSCodium Web et consoles du kit.",
        "Projekt-Explorer, Detailansicht mit 7 Abschnitten (Start, Website, Komponenten, Datenbank, Erweiterungen, Editor, Git), VSCodium Web und Konsolen des Kits.",
      ),
    icon: VscFolderLibrary,
    keywords: "progetti scheda avvio sito web componenti database estensioni editor vscodium terminale console",
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
        "Claude Code, Codex et Antigravity : suivi des limites 5 h/semaine, graphique de jetons sur 7 jours, trousseau DPAPI et synchronisation de AGENTS.md, skills et MCP.",
        "Claude Code, Codex und Antigravity: 5-Std.-/Wochenlimits, 7-Tage-Token-Diagramm, DPAPI-Keyring und automatischer Abgleich von AGENTS.md, Skills und MCP.",
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
        "MySQL + phpMyAdmin con login monouso, MongoDB + esploratore Contenuto in sola lettura, e pool condivisi Caddy (predefinito) o Apache (per le regole .htaccess) con VirtualHost .localhost.",
        "MySQL + phpMyAdmin one-time token login, MongoDB + read-only Content browser, and shared Caddy (default) or Apache (for .htaccess rules) pools with .localhost VirtualHosts.",
        "MySQL + phpMyAdmin avec connexion à jeton unique, MongoDB + explorateur de contenu en lecture seule, et pools partagés Caddy (par défaut) ou Apache (pour les règles .htaccess) avec VirtualHosts .localhost.",
        "MySQL + phpMyAdmin mit Einmal-Token-Login, MongoDB + schreibgeschützter Inhaltsbrowser sowie geteilte Caddy- (Standard) oder Apache-Pools (für .htaccess-Regeln) mit .localhost-VirtualHosts.",
      ),
    icon: VscDatabase,
    keywords:
      "database mysql mongodb phpmyadmin dbbrowser server web caddy apache htaccess pool virtualhost localhost alias",
    targetPage: "databases",
  },
  {
    id: "feat-tools-cli",
    group: "features",
    sectionCode: "§ 4.8–4.11",
    title: () =>
      tr(
        "«Strumenti», «Impostazioni», «Rimuovi l'unità» e CLI",
        "“Tools”, “Settings”, “Remove drive” & CLI",
        "« Outils », « Paramètres », « Retirer le lecteur » et CLI",
        "„Werkzeuge“, „Einstellungen“, „Laufwerk entfernen“ & CLI",
      ),
    subtitle: () =>
      tr(
        "Gestione multi-versione con il lucchetto «bloccata», matrice delle Relazioni, albero di personalizzazione Framework, espulsione sicura e CLI Dev-Kit.exe.",
        "Multi-version management with the “locked” badge, Relations matrix, Framework customization tree, safe drive ejection and Dev-Kit.exe CLI.",
        "Gestion multi-version avec le cadenas « verrouillée », matrice des Relations, arbre de personnalisation des Frameworks, éjection sécurisée et CLI Dev-Kit.exe.",
        "Multi-Versions-Verwaltung mit dem Schloss „gesperrt“, Beziehungsmatrix, Framework-Anpassungsbaum, sicherem Auswerfen und Dev-Kit.exe-CLI.",
      ),
    icon: VscExtensions,
    keywords:
      "strumenti multi-versione bloccata locked relazioni estensioni impostazioni framework rimuovi unita eject cli powershell out-host",
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
        "Choix d'un chemin sûr pour le file watcher, formatage NTFS, emplacement des fichiers temporaires, proxys d'entreprise, certificats PEM et accès LAN.",
        "Sichere Pfadwahl für den File-Watcher, NTFS-Formatierung, Platzierung temporärer Dateien, Unternehmens-Proxys, PEM-Zertifikate und LAN-Zugriff.",
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
        "Chaîne d'outils Rust GNU vs MSVC, verrouillage de versions dans dev-settings.json (pin_newest), coexistence côte à côte, clé Gemini et serveurs MCP.",
        "Rust-Toolchain GNU vs. MSVC, Versionsfixierung in dev-settings.json (pin_newest), parallele Koexistenz, Gemini-API-Schlüssel und MCP-Server.",
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
        "7–8. Database Isolati, .env, Backup e Siti Caddy/Apache",
        "7–8. Isolated Databases, .env, Backups & Caddy/Apache Sites",
        "7–8. Bases Isolées, .env, Sauvegardes et Sites Caddy/Apache",
        "7–8. Isolierte Datenbanken, .env, Backups & Caddy/Apache-Sites",
      ),
    subtitle: () =>
      tr(
        "Creazione utente DB ristretto a 32 cifre hex, scrittura automatica in .env, dump/ripristino sicuro, domini .localhost e alias @altro-progetto.",
        "Restricted 32-hex DB user creation, one-click .env writing, safe dump/restore, .localhost domains and @other-project aliases.",
        "Création d'utilisateur de base de données restreint à 32 caractères hex, écriture automatique dans .env, sauvegarde/restauration sûre, domaines .localhost et alias @autre-projet.",
        "Erstellung eingeschränkter DB-Benutzer (32 Hex-Zeichen), automatisches Schreiben in .env, sicheres Dump/Restore, .localhost-Domains und @anderes-projekt-Aliase.",
      ),
    icon: VscGlobe,
    keywords:
      "database utente isolato .env backup sql archive.gz ripristino sito web caddy apache virtualhost localhost alias",
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
        "Actions personnalisées et verrouillage de ports, isolation des extensions par espace de travail (D-71), personnalisation des recettes de l'Assistant et gestion des Données sans projet.",
        "Benutzerdefinierte Aktionen und Port-Sperrung, Erweiterungsisolierung pro Arbeitsbereich (D-71), Anpassung der Assistenten-Rezepte und Verwaltung verwaister Projektdaten.",
      ),
    icon: VscChecklist,
    keywords:
      "panel.json ports.json azioni porta bloccata estensioni indexeddb settings.json frameworks.json dati senza progetto orfani",
    targetPage: "projects",
  },
];

export type SourceCategory = "required" | "app" | "agents" | "languages" | "tools" | "databases" | "part";

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
        "Dossier racine portable du kit (ex. D:\\Dev-Kit\\), déplaçable sur n'importe quel lecteur ou PC.",
        "Portabler Stammordner des Kits (z. B. D:\\Dev-Kit\\), auf jedes Laufwerk oder jeden PC verschiebbar.",
      ),
  },
  {
    path: "Dev-Kit.exe",
    depth: 1,
    desc: () =>
      tr(
        "Unico eseguibile nativo (Rust + Tauri + UI). Funge sia da interfaccia grafica sia da CLI.",
        "Single native executable (Rust + Tauri + UI). Acts as both GUI and CLI.",
        "Exécutable natif unique (Rust + Tauri + UI). Sert à la fois d'interface graphique et de CLI.",
        "Einzige native ausführbare Datei (Rust + Tauri + UI). Dient sowohl als grafische Oberfläche als auch als CLI.",
      ),
  },
  {
    path: "projects\\",
    depth: 1,
    desc: () =>
      tr(
        "Tutti i tuoi progetti di sviluppo: ogni sottocartella è rilevata automaticamente.",
        "All your development projects: every subfolder is automatically detected.",
        "Tous vos projets de développement : chaque sous-dossier est détecté automatiquement.",
        "Alle deine Entwicklungsprojekte: Jeder Unterordner wird automatisch erkannt.",
      ),
  },
  {
    path: "data\\app\\",
    depth: 1,
    desc: () =>
      tr(
        "Stato e impostazioni del kit (state.json, settings.json, frameworks, ports.json).",
        "Kit state and settings (state.json, settings.json, frameworks, ports.json).",
        "État et paramètres du kit (state.json, settings.json, frameworks, ports.json).",
        "Status und Einstellungen des Kits (state.json, settings.json, frameworks, ports.json).",
      ),
  },
  {
    path: "data\\apps\\",
    depth: 1,
    desc: () =>
      tr(
        "Binari estratti e verificati di tutti i 28 componenti e delle linee multi-versione.",
        "Extracted and verified binaries of all 28 components and multi-version lines.",
        "Binaires extraits et vérifiés des 28 composants et des lignes multi-versions.",
        "Extrahierte und geprüfte Binärdateien aller 28 Komponenten und Multi-Versions-Linien.",
      ),
  },
  {
    path: "data\\agents\\",
    depth: 1,
    desc: () =>
      tr(
        "Fonte di verità condivisa dei 3 agenti IA: AGENTS.md, skills\\ e mcp.json.",
        "Shared source of truth for the 3 AI agents: AGENTS.md, skills\\ and mcp.json.",
        "Source de vérité partagée des 3 agents IA : AGENTS.md, skills\\ et mcp.json.",
        "Gemeinsame Quelle der Wahrheit für die 3 KI-Agenten: AGENTS.md, skills\\ und mcp.json.",
      ),
  },
  {
    path: "data\\claude\\ · data\\codex\\ · data\\antigravity\\",
    depth: 1,
    desc: () =>
      tr(
        "Profili isolati, sessioni e keyring cifrato DPAPI (.keyring.dpapi) dei 3 agenti.",
        "Isolated profiles, sessions and DPAPI-encrypted keyring (.keyring.dpapi) for the 3 agents.",
        "Profils isolés, sessions et trousseau chiffré DPAPI (.keyring.dpapi) des 3 agents.",
        "Isolierte Profile, Sitzungen und DPAPI-verschlüsselter Keyring (.keyring.dpapi) der 3 Agenten.",
      ),
  },
  {
    path: "data\\db\\ · data\\projects\\<progetto>\\private\\backups\\",
    depth: 1,
    desc: () =>
      tr(
        "Dati fisici delle tabelle MySQL e collezioni MongoDB + backup esportati per progetto.",
        "Physical data of MySQL tables and MongoDB collections + per-project exported backups.",
        "Données physiques des tables MySQL et collections MongoDB + sauvegardes exportées par projet.",
        "Physische Daten der MySQL-Tabellen und MongoDB-Sammlungen + pro Projekt exportierte Backups.",
      ),
  },
  {
    path: "data\\gh\\ · data\\home\\.gitconfig · data\\ssh\\",
    depth: 1,
    desc: () =>
      tr(
        "Token OAuth di GitHub CLI (hosts.yml), configurazione Git globale e chiavi SSH portabili.",
        "GitHub CLI OAuth token (hosts.yml), global Git config and portable SSH keys.",
        "Jeton OAuth de GitHub CLI (hosts.yml), configuration Git globale et clés SSH portables.",
        "GitHub-CLI-OAuth-Token (hosts.yml), globale Git-Konfiguration und portable SSH-Schlüssel.",
      ),
  },
  {
    path: "data\\vscodium-server\\",
    depth: 1,
    desc: () =>
      tr(
        "Profilo utente, impostazioni ed estensioni Open VSX dell'editor VSCodium.",
        "User profile, settings and Open VSX extensions for the VSCodium editor.",
        "Profil utilisateur, paramètres et extensions Open VSX de l'éditeur VSCodium.",
        "Benutzerprofil, Einstellungen und Open-VSX-Erweiterungen des VSCodium-Editors.",
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

type LazyStr = string | (() => string);
function evalStr(v: LazyStr): string {
  return typeof v === "function" ? v() : v;
}
function defSource(r: {
  id: string;
  logoId: string;
  name: LazyStr;
  group: SourceCategory;
  home: string;
  metaSource: LazyStr;
  binarySource: LazyStr;
  verifyKind: ToolSourceRow["verifyKind"];
  verifyDetail: LazyStr;
  targetPath: LazyStr;
}): ToolSourceRow {
  return {
    id: r.id,
    logoId: r.logoId,
    get name() {
      return evalStr(r.name);
    },
    group: r.group,
    home: r.home,
    get metaSource() {
      return evalStr(r.metaSource);
    },
    get binarySource() {
      return evalStr(r.binarySource);
    },
    verifyKind: r.verifyKind,
    get verifyDetail() {
      return evalStr(r.verifyDetail);
    },
    get targetPath() {
      return evalStr(r.targetPath);
    },
  };
}

const lLine = () => tr("<linea>", "<line>", "<ligne>", "<Linie>");

export const TOOL_SOURCES: ToolSourceRow[] = [
  defSource({
    id: "pwsh",
    logoId: "pwsh",
    name: "PowerShell 7 (pwsh)",
    group: "required",
    home: "https://github.com/PowerShell/PowerShell",
    metaSource: "api.github.com/repos/PowerShell/PowerShell/releases/latest",
    binarySource: "PowerShell-<v>-win-(x64|arm64).zip",
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: () => `pwsh\\${lLine()}\\pwsh.exe`,
  }),
  defSource({
    id: "gh",
    logoId: "gh",
    name: "GitHub CLI (gh)",
    group: "required",
    home: "https://cli.github.com",
    metaSource: "api.github.com/repos/cli/cli/releases/latest",
    binarySource: "gh_<v>_windows_(amd64|arm64).zip",
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: () => `gh\\${lLine()}\\bin\\gh.exe`,
  }),
  defSource({
    id: "vscodium-server",
    logoId: "vscodium-server",
    name: "Editor (VSCodium Reh Web)",
    group: "app",
    home: "https://vscodium.com",
    metaSource: () =>
      tr(
        "Catalogo integrato (versione fissata 1.135.06055, D-58)",
        "Built-in catalog (pinned version 1.135.06055, D-58)",
        "Catalogue intégré (version figée 1.135.06055, D-58)",
        "Integrierter Katalog (fixierte Version 1.135.06055, D-58)",
      ),
    binarySource: "github.com/VSCodium/vscodium/releases/download/<v>/vscodium-reh-web-win32-x64-<v>.tar.gz",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 cablato in components.json",
        "SHA-256 hardcoded in components.json",
        "SHA-256 intégré dans components.json",
        "SHA-256 fest in components.json hinterlegt",
      ),
    targetPath: "vscodium-server\\",
  }),
  defSource({
    id: "claude",
    logoId: "claude",
    name: "Claude Code",
    group: "agents",
    home: "https://claude.com/product/claude-code",
    metaSource: "registry.npmjs.org/@anthropic-ai/claude-code-win32-<arch>/latest",
    binarySource: () =>
      tr(
        "Campo dist.tarball del pacchetto npm",
        "dist.tarball field of the npm package",
        "Champ dist.tarball du paquet npm",
        "Feld dist.tarball des npm-Pakets",
      ),
    verifyKind: "Authenticode + SHA",
    verifyDetail: () =>
      tr(
        "Integrità npm sha512 + firma Authenticode Anthropic",
        "npm sha512 integrity + Anthropic Authenticode signature",
        "Intégrité npm sha512 + signature Authenticode Anthropic",
        "npm-sha512-Integrität + Authenticode-Signatur von Anthropic",
      ),
    targetPath: () => `claude\\${lLine()}\\claude.exe`,
  }),
  defSource({
    id: "codex",
    logoId: "codex",
    name: "Codex (OpenAI)",
    group: "agents",
    home: "https://github.com/openai/codex",
    metaSource: "registry.npmjs.org/@openai/codex/latest → /<ver>-win32-<arch>",
    binarySource: () =>
      tr(
        "Campo dist.tarball del pacchetto specifico per architettura",
        "dist.tarball field of the architecture-specific package",
        "Champ dist.tarball du paquet spécifique à l'architecture",
        "Feld dist.tarball des architekturspezifischen Pakets",
      ),
    verifyKind: "npm sha512",
    verifyDetail: () =>
      tr(
        "Integrità npm dist.integrity (sha512-...)",
        "npm dist.integrity (sha512-...)",
        "Intégrité npm dist.integrity (sha512-...)",
        "npm-Integrität dist.integrity (sha512-...)",
      ),
    targetPath: () => `codex\\${lLine()}\\bin\\codex.exe`,
  }),
  defSource({
    id: "antigravity",
    logoId: "antigravity",
    name: "Antigravity (Google)",
    group: "agents",
    home: "https://antigravity.google",
    metaSource: "antigravity-cli-auto-updater-974169037036.us-central1.run.app/manifests/windows_(amd64|arm64).json",
    binarySource: () =>
      tr(
        "Campo url del manifesto JSON ufficiale",
        "url field of the official JSON manifest",
        "Champ url du manifeste JSON officiel",
        "Feld url des offiziellen JSON-Manifests",
      ),
    verifyKind: "SHA-512",
    verifyDetail: () =>
      tr(
        "SHA-512 (128 hex) dal campo sha512 del manifesto",
        "SHA-512 (128 hex) from the manifest sha512 field",
        "SHA-512 (128 hex) depuis le champ sha512 du manifeste",
        "SHA-512 (128 Hex) aus dem Feld sha512 des Manifests",
      ),
    targetPath: () => `antigravity\\${lLine()}\\agy.exe (via data\\kit\\bin\\agy.cmd)`,
  }),
  defSource({
    id: "git",
    logoId: "git",
    name: "Git + Git Bash (PortableGit)",
    group: "tools",
    home: "https://git-scm.com",
    metaSource: "api.github.com/repos/git-for-windows/git/releases/latest",
    binarySource: () =>
      tr(
        "PortableGit-<v>-(64-bit|arm64).7z.exe (estratto in Rust)",
        "PortableGit-<v>-(64-bit|arm64).7z.exe (extracted in Rust)",
        "PortableGit-<v>-(64-bit|arm64).7z.exe (extrait en Rust)",
        "PortableGit-<v>-(64-bit|arm64).7z.exe (in Rust entpackt)",
      ),
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: () =>
      tr(
        `git\\${lLine()}\\cmd\\git.exe e git\\${lLine()}\\bin\\bash.exe`,
        `git\\${lLine()}\\cmd\\git.exe and git\\${lLine()}\\bin\\bash.exe`,
        `git\\${lLine()}\\cmd\\git.exe et git\\${lLine()}\\bin\\bash.exe`,
        `git\\${lLine()}\\cmd\\git.exe und git\\${lLine()}\\bin\\bash.exe`,
      ),
  }),
  defSource({
    id: "python",
    logoId: "python",
    name: "Python + pip",
    group: "languages",
    home: "https://www.python.org",
    metaSource: "raw.githubusercontent.com/astral-sh/python-build-standalone/latest-release/latest-release.json",
    binarySource: "cpython-3.<minor>.<patch>+<date>-<triple>-install_only_stripped.tar.gz",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 dall'elenco ufficiale SHA256SUMS",
        "SHA-256 from the official SHA256SUMS list",
        "SHA-256 depuis la liste officielle SHA256SUMS",
        "SHA-256 aus der offiziellen SHA256SUMS-Liste",
      ),
    targetPath: () => `python\\${lLine()}\\python.exe (+ pip)`,
  }),
  defSource({
    id: "node",
    logoId: "node",
    name: "Node.js + npm",
    group: "languages",
    home: "https://nodejs.org",
    metaSource: () =>
      tr(
        "nodejs.org/dist/index.json (LTS pari >= 18 + ultima major)",
        "nodejs.org/dist/index.json (even LTS >= 18 + latest major)",
        "nodejs.org/dist/index.json (LTS paires >= 18 + dernière majeure)",
        "nodejs.org/dist/index.json (gerade LTS >= 18 + neueste Major)",
      ),
    binarySource: "nodejs.org/dist/<v>/node-<v>-win-<arch>.zip",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 da nodejs.org/dist/<v>/SHASUMS256.txt",
        "SHA-256 from nodejs.org/dist/<v>/SHASUMS256.txt",
        "SHA-256 depuis nodejs.org/dist/<v>/SHASUMS256.txt",
        "SHA-256 aus nodejs.org/dist/<v>/SHASUMS256.txt",
      ),
    targetPath: () => `node\\${lLine()}\\node.exe`,
  }),
  defSource({
    id: "uv",
    logoId: "uv",
    name: "uv (uv, uvx, uvw)",
    group: "tools",
    home: "https://docs.astral.sh/uv/",
    metaSource: "api.github.com/repos/astral-sh/uv/releases/latest",
    binarySource: "uv-<triple>.zip",
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: () => `uv\\${lLine()}\\uv.exe, uvx.exe, uvw.exe`,
  }),
  defSource({
    id: "php",
    logoId: "php",
    name: "PHP (Thread Safe)",
    group: "languages",
    home: "https://www.php.net",
    metaSource: () =>
      tr(
        "windows.php.net/downloads/releases/releases.json (+ archivi EOL 8.1..5.6)",
        "windows.php.net/downloads/releases/releases.json (+ EOL archives 8.1..5.6)",
        "windows.php.net/downloads/releases/releases.json (+ archives EOL 8.1..5.6)",
        "windows.php.net/downloads/releases/releases.json (+ EOL-Archive 8.1..5.6)",
      ),
    binarySource: "windows.php.net/downloads/releases/php-<v>-Win32-vs*-x64.zip",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 da releases.json (o tabella EOL verificata)",
        "SHA-256 from releases.json (or verified EOL table)",
        "SHA-256 depuis releases.json (ou table EOL vérifiée)",
        "SHA-256 aus releases.json (oder geprüfter EOL-Tabelle)",
      ),
    targetPath: () => `php\\${lLine()}\\php.exe`,
  }),
  defSource({
    id: "composer",
    logoId: "php",
    name: () => tr("↳ Parte: Composer", "↳ Part: Composer", "↳ Partie : Composer", "↳ Teil: Composer"),
    group: "part",
    home: "https://getcomposer.org",
    metaSource: () =>
      tr(
        "getcomposer.org/versions (canale stable o 2.2 LTS per PHP < 7.2.5)",
        "getcomposer.org/versions (stable channel or 2.2 LTS for PHP < 7.2.5)",
        "getcomposer.org/versions (canal stable ou 2.2 LTS pour PHP < 7.2.5)",
        "getcomposer.org/versions (Stable-Kanal oder 2.2 LTS für PHP < 7.2.5)",
      ),
    binarySource: "getcomposer.org/download/<v>/composer.phar",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 da <url>.sha256sum",
        "SHA-256 from <url>.sha256sum",
        "SHA-256 depuis <url>.sha256sum",
        "SHA-256 aus <url>.sha256sum",
      ),
    targetPath: () => `php\\${lLine()}\\composer.phar (+ composer.cmd)`,
  }),
  defSource({
    id: "cacert",
    logoId: "php",
    name: () =>
      tr(
        "↳ Parte: Bundle CA SSL (curl)",
        "↳ Part: SSL CA Bundle (curl)",
        "↳ Partie : Bundle CA SSL (curl)",
        "↳ Teil: SSL-CA-Bundle (curl)",
      ),
    group: "part",
    home: "https://curl.se",
    metaSource: "curl.se/ca/cacert.pem.sha256",
    binarySource: "curl.se/ca/cacert.pem",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 da cacert.pem.sha256",
        "SHA-256 from cacert.pem.sha256",
        "SHA-256 depuis cacert.pem.sha256",
        "SHA-256 aus cacert.pem.sha256",
      ),
    targetPath: () => `php\\${lLine()}\\extras\\ssl\\cacert.pem`,
  }),
  defSource({
    id: "vcruntime",
    logoId: "cc",
    name: () =>
      tr(
        "↳ Parte: Visual C++ Runtime",
        "↳ Part: Visual C++ Runtime",
        "↳ Partie : Visual C++ Runtime",
        "↳ Teil: Visual C++ Runtime",
      ),
    group: "part",
    home: "https://learn.microsoft.com",
    metaSource: () =>
      tr(
        "Estratto dal CAB vcRuntimeMinimum_amd64\\cab1.cab se assente",
        "Extracted from CAB vcRuntimeMinimum_amd64\\cab1.cab if missing",
        "Extrait du CAB vcRuntimeMinimum_amd64\\cab1.cab si absent",
        "Bei Fehlen aus CAB vcRuntimeMinimum_amd64\\cab1.cab entpackt",
      ),
    binarySource: "aka.ms/vc14/vc_redist.x64.exe",
    verifyKind: "Authenticode + SHA",
    verifyDetail: () =>
      tr(
        "Firma Authenticode O=Microsoft Corporation su exe e vcruntime140.dll",
        "Authenticode signature O=Microsoft Corporation on exe and vcruntime140.dll",
        "Signature Authenticode O=Microsoft Corporation sur exe et vcruntime140.dll",
        "Authenticode-Signatur O=Microsoft Corporation auf exe und vcruntime140.dll",
      ),
    targetPath: () =>
      tr(
        "tools\\vcruntime\\ (copiato in PHP, Apache, MongoDB)",
        "tools\\vcruntime\\ (copied into PHP, Apache, MongoDB)",
        "tools\\vcruntime\\ (copié dans PHP, Apache, MongoDB)",
        "tools\\vcruntime\\ (kopiert nach PHP, Apache, MongoDB)",
      ),
  }),
  defSource({
    id: "apache",
    logoId: "apache",
    name: "Apache httpd",
    group: "languages",
    home: "https://httpd.apache.org",
    metaSource: () =>
      tr(
        "apachelounge.com/download/ (scraping versione e checksum)",
        "apachelounge.com/download/ (version and checksum scraping)",
        "apachelounge.com/download/ (extraction de version et somme de contrôle)",
        "apachelounge.com/download/ (Auslesen von Version und Prüfsumme)",
      ),
    binarySource: "apachelounge.com/download/VS<nn>/binaries/httpd-<v>-Win64-VS<nn>.zip",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 da <url>.txt (SHA256-Checksum for:)",
        "SHA-256 from <url>.txt (SHA256-Checksum for:)",
        "SHA-256 depuis <url>.txt (SHA256-Checksum for:)",
        "SHA-256 aus <url>.txt (SHA256-Checksum for:)",
      ),
    targetPath: () => `apache\\${lLine()}\\bin\\httpd.exe`,
  }),
  defSource({
    id: "caddy",
    logoId: "caddy",
    name: "Caddy",
    group: "languages",
    home: "https://caddyserver.com",
    metaSource: "api.github.com/repos/caddyserver/caddy/releases?per_page=100",
    binarySource: () =>
      tr(
        "Asset caddy_<v>_windows_amd64.zip dalla release GitHub ufficiale",
        "caddy_<v>_windows_amd64.zip asset from the official GitHub release",
        "Asset caddy_<v>_windows_amd64.zip depuis la release GitHub officielle",
        "Asset caddy_<v>_windows_amd64.zip aus dem offiziellen GitHub-Release",
      ),
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "GitHub Release asset.digest (dalla 2.10.1), altrimenti SHA-512 da caddy_<v>_checksums.txt",
        "GitHub Release asset.digest (from 2.10.1), otherwise SHA-512 from caddy_<v>_checksums.txt",
        "GitHub Release asset.digest (depuis 2.10.1), sinon SHA-512 depuis caddy_<v>_checksums.txt",
        "GitHub Release asset.digest (ab 2.10.1), sonst SHA-512 aus caddy_<v>_checksums.txt",
      ),
    targetPath: () => `caddy\\${lLine()}\\caddy.exe`,
  }),
  defSource({
    id: "rust",
    logoId: "rust",
    name: "Rust (rustup, cargo, clippy)",
    group: "tools",
    home: "https://www.rust-lang.org",
    metaSource: "static.rust-lang.org/dist/channel-rust-stable.toml ([pkg.rustc])",
    binarySource: "static.rust-lang.org/rustup/dist/<triple>/rustup-init.exe",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 da <url>.sha256",
        "SHA-256 from <url>.sha256",
        "SHA-256 depuis <url>.sha256",
        "SHA-256 aus <url>.sha256",
      ),
    targetPath: () =>
      tr(
        "rust\\cargo\\bin\\rustc.exe e rustup.exe",
        "rust\\cargo\\bin\\rustc.exe and rustup.exe",
        "rust\\cargo\\bin\\rustc.exe et rustup.exe",
        "rust\\cargo\\bin\\rustc.exe und rustup.exe",
      ),
  }),
  defSource({
    id: "go",
    logoId: "go",
    name: "Go SDK",
    group: "languages",
    home: "https://go.dev",
    metaSource: "go.dev/dl/?mode=json (&include=all)",
    binarySource: "go.dev/dl/go<v>.windows-(amd64|arm64).zip",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 dal campo sha256 del JSON ufficiale",
        "SHA-256 from the sha256 field of the official JSON",
        "SHA-256 depuis le champ sha256 du JSON officiel",
        "SHA-256 aus dem Feld sha256 des offiziellen JSON",
      ),
    targetPath: () => `go\\${lLine()}\\bin\\go.exe`,
  }),
  defSource({
    id: "java",
    logoId: "java",
    name: "Java JDK (Eclipse Temurin)",
    group: "languages",
    home: "https://adoptium.net",
    metaSource: "api.adoptium.net/v3/info/available_releases + /v3/assets/latest/<major>/hotspot",
    binarySource: () =>
      tr(
        "Pacchetto .zip restituito dall'API Adoptium",
        ".zip package returned by the Adoptium API",
        "Paquet .zip renvoyé par l'API Adoptium",
        ".zip-Paket aus der Adoptium-API",
      ),
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 da binary.package.checksum",
        "SHA-256 from binary.package.checksum",
        "SHA-256 depuis binary.package.checksum",
        "SHA-256 aus binary.package.checksum",
      ),
    targetPath: () => `java\\${lLine()}\\bin\\java.exe`,
  }),
  defSource({
    id: "maven",
    logoId: "java",
    name: () => tr("↳ Parte: Apache Maven", "↳ Part: Apache Maven", "↳ Partie : Apache Maven", "↳ Teil: Apache Maven"),
    group: "part",
    home: "https://maven.apache.org",
    metaSource: "repo.maven.apache.org/maven2/org/apache/maven/apache-maven/maven-metadata.xml",
    binarySource: "apache-maven-<v>-bin.zip",
    verifyKind: "SHA-512",
    verifyDetail: () =>
      tr(
        "SHA-512 da <url>.sha512",
        "SHA-512 from <url>.sha512",
        "SHA-512 depuis <url>.sha512",
        "SHA-512 aus <url>.sha512",
      ),
    targetPath: "maven\\bin\\mvn.cmd",
  }),
  defSource({
    id: "gradle",
    logoId: "java",
    name: () => tr("↳ Parte: Gradle", "↳ Part: Gradle", "↳ Partie : Gradle", "↳ Teil: Gradle"),
    group: "part",
    home: "https://gradle.org",
    metaSource: "services.gradle.org/versions/current",
    binarySource: () =>
      tr(
        "Campo downloadUrl del JSON ufficiale",
        "downloadUrl field of the official JSON",
        "Champ downloadUrl du JSON officiel",
        "Feld downloadUrl des offiziellen JSON",
      ),
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 da checksumUrl del JSON",
        "SHA-256 from checksumUrl of the JSON",
        "SHA-256 depuis checksumUrl du JSON",
        "SHA-256 aus checksumUrl des JSON",
      ),
    targetPath: "gradle\\bin\\gradle.bat (via data\\kit\\bin\\gradle.cmd)",
  }),
  defSource({
    id: "kotlin",
    logoId: "kotlin",
    name: "Kotlin Compiler",
    group: "languages",
    home: "https://kotlinlang.org",
    metaSource: "api.github.com/repos/JetBrains/kotlin/releases?per_page=100",
    binarySource: "Asset kotlin-compiler-<v>.zip",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "GitHub asset.digest o <asset>.sha256",
        "GitHub asset.digest or <asset>.sha256",
        "GitHub asset.digest ou <asset>.sha256",
        "GitHub asset.digest oder <asset>.sha256",
      ),
    targetPath: () => `kotlin\\${lLine()}\\bin\\kotlinc.bat`,
  }),
  defSource({
    id: "android",
    logoId: "android",
    name: "Android SDK + Command-line Tools",
    group: "tools",
    home: "https://developer.android.com/studio",
    metaSource: () =>
      tr(
        "developer.android.com/studio (tabella commandlinetools-win-*_latest.zip)",
        "developer.android.com/studio (commandlinetools-win-*_latest.zip table)",
        "developer.android.com/studio (table commandlinetools-win-*_latest.zip)",
        "developer.android.com/studio (Tabelle commandlinetools-win-*_latest.zip)",
      ),
    binarySource: "dl.google.com/android/repository/commandlinetools-win-<build>_latest.zip",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 pubblicato nella tabella ufficiale Android",
        "SHA-256 published in the official Android table",
        "SHA-256 publié dans la table officielle Android",
        "SHA-256 aus der offiziellen Android-Tabelle",
      ),
    targetPath: () =>
      tr(
        "android\\cmdline-tools\\latest\\bin\\sdkmanager.bat e adb.exe",
        "android\\cmdline-tools\\latest\\bin\\sdkmanager.bat and adb.exe",
        "android\\cmdline-tools\\latest\\bin\\sdkmanager.bat et adb.exe",
        "android\\cmdline-tools\\latest\\bin\\sdkmanager.bat und adb.exe",
      ),
  }),
  defSource({
    id: "dotnet",
    logoId: "dotnet",
    name: ".NET SDK",
    group: "languages",
    home: "https://dotnet.microsoft.com",
    metaSource: "builds.dotnet.microsoft.com/dotnet/release-metadata/releases-index.json",
    binarySource: () =>
      tr(
        "Archivio .zip per rid=win-x64|win-arm64",
        ".zip archive for rid=win-x64|win-arm64",
        "Archive .zip pour rid=win-x64|win-arm64",
        ".zip-Archiv für rid=win-x64|win-arm64",
      ),
    verifyKind: "SHA-512",
    verifyDetail: () =>
      tr(
        "SHA-512 dal campo hash di releases.json",
        "SHA-512 from the hash field of releases.json",
        "SHA-512 depuis le champ hash de releases.json",
        "SHA-512 aus dem Feld hash von releases.json",
      ),
    targetPath: () =>
      tr(
        `dotnet\\${lLine()}\\dotnet.exe (con junction SDK condivise)`,
        `dotnet\\${lLine()}\\dotnet.exe (with shared SDK junctions)`,
        `dotnet\\${lLine()}\\dotnet.exe (avec jonctions SDK partagées)`,
        `dotnet\\${lLine()}\\dotnet.exe (mit geteilten SDK-Junctions)`,
      ),
  }),
  defSource({
    id: "dart",
    logoId: "dart",
    name: "Dart SDK",
    group: "languages",
    home: "https://dart.dev",
    metaSource: "storage.googleapis.com/dart-archive/channels/stable/release/latest/VERSION",
    binarySource: "dartsdk-windows-<arch>-release.zip",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 da <url>.sha256sum",
        "SHA-256 from <url>.sha256sum",
        "SHA-256 depuis <url>.sha256sum",
        "SHA-256 aus <url>.sha256sum",
      ),
    targetPath: () => `dart\\${lLine()}\\bin\\dart.exe`,
  }),
  defSource({
    id: "zig",
    logoId: "zig",
    name: "Zig",
    group: "languages",
    home: "https://ziglang.org",
    metaSource: "ziglang.org/download/index.json",
    binarySource: () =>
      tr(
        "Campo tarball per (x86_64|aarch64)-windows",
        "tarball field for (x86_64|aarch64)-windows",
        "Champ tarball pour (x86_64|aarch64)-windows",
        "Feld tarball für (x86_64|aarch64)-windows",
      ),
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 dal campo shasum del JSON",
        "SHA-256 from the shasum field of the JSON",
        "SHA-256 depuis le champ shasum du JSON",
        "SHA-256 aus dem Feld shasum des JSON",
      ),
    targetPath: () => `zig\\${lLine()}\\zig.exe`,
  }),
  defSource({
    id: "julia",
    logoId: "julia",
    name: "Julia",
    group: "languages",
    home: "https://julialang.org",
    metaSource: "julialang-s3.julialang.org/bin/versions.json",
    binarySource: () =>
      tr(
        "Archivio .zip per os=winnt, arch=x86_64",
        ".zip archive for os=winnt, arch=x86_64",
        "Archive .zip pour os=winnt, arch=x86_64",
        ".zip-Archiv für os=winnt, arch=x86_64",
      ),
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 dal campo sha256 di versions.json",
        "SHA-256 from the sha256 field of versions.json",
        "SHA-256 depuis le champ sha256 de versions.json",
        "SHA-256 aus dem Feld sha256 von versions.json",
      ),
    targetPath: () => `julia\\${lLine()}\\bin\\julia.exe`,
  }),
  defSource({
    id: "bun",
    logoId: "bun",
    name: "Bun",
    group: "languages",
    home: "https://bun.sh",
    metaSource: "api.github.com/repos/oven-sh/bun/releases",
    binarySource: "Asset bun-windows-(x64|aarch64).zip",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "GitHub asset.digest o SHASUMS256.txt",
        "GitHub asset.digest or SHASUMS256.txt",
        "GitHub asset.digest ou SHASUMS256.txt",
        "GitHub asset.digest oder SHASUMS256.txt",
      ),
    targetPath: () => `bun\\${lLine()}\\bun.exe`,
  }),
  defSource({
    id: "deno",
    logoId: "deno",
    name: "Deno",
    group: "languages",
    home: "https://deno.com",
    metaSource: "api.github.com/repos/denoland/deno/releases",
    binarySource: "Asset deno-(x86_64|aarch64)-pc-windows-msvc.zip",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "GitHub asset.digest o <asset>.sha256sum",
        "GitHub asset.digest or <asset>.sha256sum",
        "GitHub asset.digest ou <asset>.sha256sum",
        "GitHub asset.digest oder <asset>.sha256sum",
      ),
    targetPath: () => `deno\\${lLine()}\\deno.exe`,
  }),
  defSource({
    id: "typst",
    logoId: "typst",
    name: "Typst",
    group: "languages",
    home: "https://typst.app",
    metaSource: "api.github.com/repos/typst/typst/releases",
    binarySource: "Asset typst-(x86_64|aarch64)-pc-windows-msvc.zip",
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: () => `typst\\${lLine()}\\typst.exe`,
  }),
  defSource({
    id: "tinytex",
    logoId: "tinytex",
    name: "TinyTeX (LaTeX)",
    group: "tools",
    home: "https://yihui.org/tinytex/",
    metaSource: "api.github.com/repos/rstudio/tinytex-releases/releases/latest",
    binarySource: () =>
      tr(
        "Asset TinyTeX-1-windows-v*.exe (7z SFX estratto in Rust)",
        "Asset TinyTeX-1-windows-v*.exe (7z SFX extracted in Rust)",
        "Asset TinyTeX-1-windows-v*.exe (7z SFX extrait en Rust)",
        "Asset TinyTeX-1-windows-v*.exe (7z-SFX in Rust entpackt)",
      ),
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: () =>
      tr(
        "tinytex\\bin\\windows\\tlmgr.bat e pdftex.exe",
        "tinytex\\bin\\windows\\tlmgr.bat and pdftex.exe",
        "tinytex\\bin\\windows\\tlmgr.bat et pdftex.exe",
        "tinytex\\bin\\windows\\tlmgr.bat und pdftex.exe",
      ),
  }),
  defSource({
    id: "pnpm",
    logoId: "pnpm",
    name: "pnpm + yarn (corepack)",
    group: "tools",
    home: "https://pnpm.io",
    metaSource: () =>
      tr(
        "registry.npmjs.org/pnpm/latest e /corepack/latest",
        "registry.npmjs.org/pnpm/latest and /corepack/latest",
        "registry.npmjs.org/pnpm/latest et /corepack/latest",
        "registry.npmjs.org/pnpm/latest und /corepack/latest",
      ),
    binarySource: () =>
      tr(
        "Installato con npm-cli.js del Node.js del kit",
        "Installed via npm-cli.js of the kit's Node.js",
        "Installé avec npm-cli.js du Node.js du kit",
        "Installiert über npm-cli.js des Kit-Node.js",
      ),
    verifyKind: "npm sha512",
    verifyDetail: () =>
      tr(
        "Integrità del registro npm",
        "npm registry integrity",
        "Intégrité du registre npm",
        "Integrität der npm-Registry",
      ),
    targetPath: () =>
      tr(
        "data\\npm-global\\pnpm.cmd e yarn.cmd",
        "data\\npm-global\\pnpm.cmd and yarn.cmd",
        "data\\npm-global\\pnpm.cmd et yarn.cmd",
        "data\\npm-global\\pnpm.cmd und yarn.cmd",
      ),
  }),
  defSource({
    id: "cmake",
    logoId: "cmake",
    name: "CMake + Ninja",
    group: "tools",
    home: "https://cmake.org",
    metaSource: () =>
      tr(
        "GitHub API: Kitware/CMake e ninja-build/ninja (releases/latest)",
        "GitHub API: Kitware/CMake and ninja-build/ninja (releases/latest)",
        "API GitHub : Kitware/CMake et ninja-build/ninja (releases/latest)",
        "GitHub-API: Kitware/CMake und ninja-build/ninja (releases/latest)",
      ),
    binarySource: () =>
      tr(
        "cmake-*-windows-(x86_64|arm64).zip e ninja-win(arm64).zip",
        "cmake-*-windows-(x86_64|arm64).zip and ninja-win(arm64).zip",
        "cmake-*-windows-(x86_64|arm64).zip et ninja-win(arm64).zip",
        "cmake-*-windows-(x86_64|arm64).zip und ninja-win(arm64).zip",
      ),
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: () =>
      tr(
        `cmake\\${lLine()}\\bin\\cmake.exe e tools\\ninja.exe`,
        `cmake\\${lLine()}\\bin\\cmake.exe and tools\\ninja.exe`,
        `cmake\\${lLine()}\\bin\\cmake.exe et tools\\ninja.exe`,
        `cmake\\${lLine()}\\bin\\cmake.exe und tools\\ninja.exe`,
      ),
  }),
  defSource({
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
  }),
  defSource({
    id: "ruby",
    logoId: "ruby",
    name: "Ruby + Bundler",
    group: "languages",
    home: "https://www.ruby-lang.org",
    metaSource: "api.github.com/repos/oneclick/rubyinstaller2/releases?per_page=100",
    binarySource: "Asset rubyinstaller-<v>-(x64|arm).7z",
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: () => `ruby\\${lLine()}\\bin\\ruby.exe`,
  }),
  defSource({
    id: "r",
    logoId: "r",
    name: "R (conda-forge + OpenBLAS)",
    group: "languages",
    home: "https://www.r-project.org",
    metaSource: "api.anaconda.org/package/conda-forge/r-base + mamba-org/micromamba-releases",
    binarySource: () =>
      tr(
        "micromamba-win-64 + pacchetti r-base e libblas=*=*openblas",
        "micromamba-win-64 + r-base and libblas=*=*openblas packages",
        "micromamba-win-64 + paquets r-base et libblas=*=*openblas",
        "micromamba-win-64 + Pakete r-base und libblas=*=*openblas",
      ),
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 di micromamba + SHA-256 pacchetti conda-forge",
        "micromamba SHA-256 + conda-forge package SHA-256",
        "SHA-256 de micromamba + SHA-256 des paquets conda-forge",
        "SHA-256 von micromamba + SHA-256 der conda-forge-Pakete",
      ),
    targetPath: () =>
      tr(
        `r\\${lLine()}\\bin\\R.cmd e Rscript.cmd`,
        `r\\${lLine()}\\bin\\R.cmd and Rscript.cmd`,
        `r\\${lLine()}\\bin\\R.cmd et Rscript.cmd`,
        `r\\${lLine()}\\bin\\R.cmd und Rscript.cmd`,
      ),
  }),
  defSource({
    id: "mysql",
    logoId: "mysql",
    name: "MySQL Community Server",
    group: "databases",
    home: "https://www.mysql.com",
    metaSource: () =>
      tr(
        "Linee 8.4 e 9.7 LTS fissate nel catalogo (sha256ByLine)",
        "8.4 and 9.7 LTS lines pinned in the catalog (sha256ByLine)",
        "Lignes LTS 8.4 et 9.7 fixées dans le catalogue (sha256ByLine)",
        "LTS-Linien 8.4 und 9.7 im Katalog festgelegt (sha256ByLine)",
      ),
    binarySource: "cdn.mysql.com/Downloads/MySQL-<linea>/mysql-<v>-winx64.zip",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 fissato per linea e target dopo la firma GPG di Oracle",
        "SHA-256 pinned per line and target after Oracle's GPG signature",
        "SHA-256 fixé par ligne et cible après la signature GPG d'Oracle",
        "SHA-256 pro Linie und Ziel nach Oracles GPG-Signatur festgelegt",
      ),
    targetPath: () =>
      tr(
        `mysql\\${lLine()}\\bin\\mysqld.exe e mysql.exe`,
        `mysql\\${lLine()}\\bin\\mysqld.exe and mysql.exe`,
        `mysql\\${lLine()}\\bin\\mysqld.exe et mysql.exe`,
        `mysql\\${lLine()}\\bin\\mysqld.exe und mysql.exe`,
      ),
  }),
  defSource({
    id: "mongodb",
    logoId: "mongodb",
    name: "MongoDB + mongosh + Database Tools",
    group: "databases",
    home: "https://www.mongodb.com",
    metaSource: "downloads.mongodb.org/current.json, GitHub mongodb-js/mongosh, tools/db/release.json",
    binarySource: () =>
      tr(
        "Archivi .zip Windows x64 di mongod, mongosh e mongodump/mongorestore",
        "Windows x64 .zip archives of mongod, mongosh and mongodump/mongorestore",
        "Archives .zip Windows x64 de mongod, mongosh et mongodump/mongorestore",
        "Windows-x64-.zip-Archive von mongod, mongosh und mongodump/mongorestore",
      ),
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 dai JSON MongoDB + asset.digest GitHub per mongosh",
        "SHA-256 from MongoDB JSONs + GitHub asset.digest for mongosh",
        "SHA-256 depuis les JSON MongoDB + asset.digest GitHub pour mongosh",
        "SHA-256 aus den MongoDB-JSONs + GitHub asset.digest für mongosh",
      ),
    targetPath: () => `mongodb\\${lLine()}\\bin\\mongod.exe, mongosh\\, mongodb-tools\\`,
  }),
  defSource({
    id: "phpmyadmin",
    logoId: "phpmyadmin",
    name: () =>
      tr("phpMyAdmin (per MySQL)", "phpMyAdmin (for MySQL)", "phpMyAdmin (pour MySQL)", "phpMyAdmin (für MySQL)"),
    group: "databases",
    home: "https://www.phpmyadmin.net",
    metaSource: "www.phpmyadmin.net/home_page/version.json",
    binarySource: "files.phpmyadmin.net/phpMyAdmin/<v>/phpMyAdmin-<v>-all-languages.zip",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 dal file .sha256 ufficiale + controllo sintattico php -l",
        "SHA-256 from the official .sha256 file + php -l syntax check",
        "SHA-256 depuis le fichier .sha256 officiel + vérification syntaxique php -l",
        "SHA-256 aus der offiziellen .sha256-Datei + Syntaxprüfung php -l",
      ),
    targetPath: () => `phpmyadmin\\${lLine()}\\index.php`,
  }),
  defSource({
    id: "sqlite",
    logoId: "sqlite",
    name: "SQLite CLI",
    group: "databases",
    home: "https://sqlite.org",
    metaSource: () =>
      tr(
        "sqlite.org/download.html (righe CSV PRODUCT,...)",
        "sqlite.org/download.html (PRODUCT,... CSV rows)",
        "sqlite.org/download.html (lignes CSV PRODUCT,...)",
        "sqlite.org/download.html (CSV-Zeilen PRODUCT,...)",
      ),
    binarySource: () =>
      tr(
        "sqlite.org/<anno>/sqlite-tools-win-(x64|arm64)-<v>.zip",
        "sqlite.org/<year>/sqlite-tools-win-(x64|arm64)-<v>.zip",
        "sqlite.org/<année>/sqlite-tools-win-(x64|arm64)-<v>.zip",
        "sqlite.org/<Jahr>/sqlite-tools-win-(x64|arm64)-<v>.zip",
      ),
    verifyKind: "SHA3-256",
    verifyDetail: () =>
      tr(
        "SHA3-256 pubblicato nella riga PRODUCT della pagina ufficiale",
        "SHA3-256 published in the PRODUCT row of the official page",
        "SHA3-256 publié dans la ligne PRODUCT de la page officielle",
        "SHA3-256 aus der PRODUCT-Zeile der offiziellen Seite",
      ),
    targetPath: () => `sqlite\\${lLine()}\\sqlite3.exe`,
  }),
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
        "Affiche les ~90 variables d'environnement isolées calculées pour le kit.",
        "Gibt alle ~90 isolierten Umgebungsvariablen aus, die für das Kit berechnet wurden.",
      ),
  },
  {
    cmd: ".\\Dev-Kit.exe components | Out-Host",
    desc: () =>
      tr(
        "Elenca tutti i componenti del catalogo, se sono scelti e la versione installata su disco.",
        "Lists all catalog components, whether they are selected and the installed version on disk.",
        "Liste tous les composants du catalogue, s'ils sont sélectionnés et la version installée sur le disque.",
        "Listet alle Katalogkomponenten auf, ob sie ausgewählt sind und welche Version auf dem Datenträger installiert ist.",
      ),
  },
  {
    cmd: ".\\Dev-Kit.exe versions <id> | Out-Host",
    desc: () =>
      tr(
        "Interroga la fonte ufficiale e mostra le linee major.minor disponibili per un componente versionato (es. node, php, python).",
        "Queries the official source and shows available major.minor lines for a versioned component (e.g. node, php, python).",
        "Interroge la source officielle et affiche les lignes major.minor disponibles pour un composant versionné (ex. node, php, python).",
        "Fragt die offizielle Quelle ab und zeigt verfügbare major.minor-Linien für eine versionierte Komponente an (z. B. node, php, python).",
      ),
  },
  {
    cmd: ".\\Dev-Kit.exe check | Out-Host",
    desc: () =>
      tr(
        "Esegue il comando di verifica finale su tutti i componenti installati per accertarne l'integrità.",
        "Runs the verification command on all installed components to check their integrity.",
        "Exécute la commande de vérification finale sur tous les composants installés pour contrôler leur intégrité.",
        "Führt den abschließenden Prüfbefehl für alle installierten Komponenten aus, um deren Integrität sicherzustellen.",
      ),
  },
  {
    cmd: ".\\Dev-Kit.exe prepare [--only a,b] [--skip c] [--update-all] [--force] | Out-Host",
    desc: () =>
      tr(
        "Installa, aggiorna o rimuove da riga di comando i componenti scelti.",
        "Installs, updates or removes selected components from the command line.",
        "Installe, met à jour ou supprime en ligne de commande les composants sélectionnés.",
        "Installiert, aktualisiert oder entfernt ausgewählte Komponenten über die Befehlszeile.",
      ),
  },
  {
    cmd: ".\\Dev-Kit.exe sync [--project <cartella>] [--verbose] | Out-Host",
    desc: () =>
      tr(
        "Allinea istruzioni (AGENTS.md), skill e server MCP fra Claude Code, Codex e Antigravity.",
        "Synchronizes instructions (AGENTS.md), skills and MCP servers across Claude Code, Codex and Antigravity.",
        "Synchronise les instructions (AGENTS.md), les skills et les serveurs MCP entre Claude Code, Codex et Antigravity.",
        "Synchronisiert Anweisungen (AGENTS.md), Skills und MCP-Server zwischen Claude Code, Codex und Antigravity.",
      ),
  },
  {
    cmd: ".\\Dev-Kit.exe keyring restore|save | Out-Host",
    desc: () =>
      tr(
        "Ripristina o salva nel kit (con cifratura DPAPI) l'accesso di Antigravity (usato da agy.cmd).",
        "Restores or saves Antigravity's credentials into the kit (encrypted with DPAPI, used by agy.cmd).",
        "Restaure ou enregistre dans le kit (avec chiffrement DPAPI) les identifiants d'Antigravity (utilisé par agy.cmd).",
        "Stellt die Zugangsdaten von Antigravity im Kit wieder her oder speichert sie (DPAPI-verschlüsselt, verwendet von agy.cmd).",
      ),
  },
  {
    cmd: ".\\Dev-Kit.exe serve <cartella> <porta> | Out-Host",
    desc: () =>
      tr(
        "Avvia il server HTTP statico integrato sulla cartella e porta indicate.",
        "Starts the built-in static HTTP server on the given folder and port.",
        "Démarre le serveur HTTP statique intégré sur le dossier et le port indiqués.",
        "Startet den integrierten statischen HTTP-Server für den angegebenen Ordner und Port.",
      ),
  },
  {
    cmd: ".\\Dev-Kit.exe version | Out-Host",
    desc: () =>
      tr(
        "Mostra la versione dell'eseguibile Dev-Kit.exe.",
        "Shows the version of the Dev-Kit.exe executable.",
        "Affiche la version de l'exécutable Dev-Kit.exe.",
        "Zeigt die Version der ausführbaren Datei Dev-Kit.exe an.",
      ),
  },
  {
    cmd: ".\\Dev-Kit.exe help | Out-Host",
    desc: () =>
      tr(
        "Mostra il riepilogo di tutti i comandi CLI disponibili.",
        "Shows the summary of all available CLI commands.",
        "Affiche le récapitulatif de toutes les commandes CLI disponibles.",
        "Zeigt die Übersicht aller verfügbaren CLI-Befehle an.",
      ),
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
