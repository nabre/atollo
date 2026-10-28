// The map of the in-app Guide (ui/src/guide/): its chapters and topics, in
// reading order. A topic says what it is called, what it covers, where it
// lives in the app and the words it answers to in the search; its text is in
// topics/, one file per chapter. The order follows the user: first steps,
// daily work, services, agents, tools, settings, carrying the kit, how it
// works inside, and help when something goes wrong.
import { tr } from "./lang";

export type GuideGroupId =
  "start" | "projects" | "services" | "agents" | "tools" | "settings" | "portable" | "inside" | "help";

export type GuideTopicId =
  // 1. First steps
  | "welcome"
  | "first-run"
  | "window"
  | "first-project"
  // 2. Projects
  | "project-new"
  | "project-page"
  | "project-run"
  | "project-versions"
  | "project-editor"
  | "project-git"
  | "project-cleanup"
  // 3. Databases, websites, terminals
  | "databases"
  | "project-db"
  | "websites"
  | "terminals"
  // 4. AI agents
  | "agents"
  | "agents-shared"
  | "agents-usage"
  | "models"
  // 5. Tools
  | "tools"
  | "tools-versions"
  | "tools-overview"
  // 6. Account and settings
  | "github"
  | "settings"
  | "network"
  | "frameworks"
  // 7. Carrying the kit
  | "eject"
  | "move"
  | "updates"
  // 8. How it works
  | "vm"
  | "isolation"
  | "disk-layout"
  | "lifecycle"
  | "sources"
  | "cli"
  // 9. Help
  | "troubleshooting"
  | "faq"
  | "glossary";

/** The pages a topic can open ("new" is the «Nuovo progetto» tab of Progetti). */
export type AppPageTarget =
  | "new"
  | "projects"
  | "agents"
  | "models"
  | "databases"
  | "servers"
  | "terminals"
  | "components"
  | "settings"
  | "eject";

export interface GuideTopic {
  id: GuideTopicId;
  group: GuideGroupId;
  title: () => string;
  subtitle: () => string;
  /** A codicon's name (`home`, `rocket`): the app draws it with its
   * font, the site with the SVG of @vscode/codicons. */
  icon: string;
  /** Extra words the search finds the topic by, in every language. */
  keywords: string;
  targetPage?: AppPageTarget;
}

export interface GuideGroup {
  id: GuideGroupId;
  title: () => string;
}

export const GUIDE_GROUPS: GuideGroup[] = [
  { id: "start", title: () => tr("Primi passi", "First steps", "Premiers pas", "Erste Schritte") },
  { id: "projects", title: () => tr("Progetti", "Projects", "Projets", "Projekte") },
  {
    id: "services",
    title: () =>
      tr(
        "Database, siti e terminali",
        "Databases, websites and terminals",
        "Bases de données, sites et terminaux",
        "Datenbanken, Websites und Terminals",
      ),
  },
  {
    id: "agents",
    title: () =>
      tr(
        "AI: agenti e modelli locali",
        "AI: agents and local models",
        "IA : agents et modèles locaux",
        "KI: Agenten und lokale Modelle",
      ),
  },
  {
    id: "tools",
    title: () => tr("Strumenti e linguaggi", "Tools and languages", "Outils et langages", "Werkzeuge und Sprachen"),
  },
  {
    id: "settings",
    title: () =>
      tr("Account e impostazioni", "Account and settings", "Compte et paramètres", "Konto und Einstellungen"),
  },
  {
    id: "portable",
    title: () =>
      tr("Portare il kit con sé", "Taking the kit with you", "Emporter le kit avec vous", "Das Kit mitnehmen"),
  },
  {
    id: "inside",
    title: () =>
      tr(
        "Come funziona dentro",
        "How it works inside",
        "Comment il fonctionne à l'intérieur",
        "Wie es innen funktioniert",
      ),
  },
  { id: "help", title: () => tr("Aiuto", "Help", "Aide", "Hilfe") },
];

export const GUIDE_TOPICS: GuideTopic[] = [
  // 1. First steps
  {
    id: "welcome",
    group: "start",
    title: () => tr("Benvenuto in Atollo", "Welcome to Atollo", "Bienvenue dans Atollo", "Willkommen bei Atollo"),
    subtitle: () =>
      tr(
        "Che cos'è Atollo, che cosa trovi dentro e come è organizzata questa guida. Il punto da cui partire se è la prima volta.",
        "What Atollo is, what you find inside and how this guide is organized. The place to start if it is your first time.",
        "Ce qu'est Atollo, ce que vous y trouvez et comment ce guide est organisé. Le point de départ si c'est votre première fois.",
        "Was Atollo ist, was du darin findest und wie diese Anleitung aufgebaut ist. Der Ausgangspunkt, wenn du zum ersten Mal hier bist.",
      ),
    icon: "home",
    keywords:
      "welcome start overview introduzione cos'è portabile chiavetta usb kit introduction portable usb stick bienvenue introduction clé portable willkommen einführung portabel usb-stick",
  },
  {
    id: "first-run",
    group: "start",
    title: () => tr("Il primo avvio", "The first start", "Le premier lancement", "Der erste Start"),
    subtitle: () =>
      tr(
        "Dove mettere il kit, la pagina «Benvenuto», quali componenti scegliere e che cosa succede mentre si scaricano.",
        "Where to put the kit, the “Welcome” page, which components to choose and what happens while they download.",
        "Où placer le kit, la page « Bienvenue », quels composants choisir et ce qui se passe pendant leur téléchargement.",
        "Wohin das Kit gehört, die Seite „Willkommen“, welche Komponenten du wählst und was passiert, während sie heruntergeladen werden.",
      ),
    icon: "rocket",
    keywords:
      "first run benvenuto installa componenti scaricare download primo avvio setup first start install components welcome premier lancement installer composants télécharger erster start installieren komponenten herunterladen",
    targetPage: "components",
  },
  {
    id: "window",
    group: "start",
    title: () =>
      tr("Conoscere la finestra", "Getting to know the window", "Découvrir la fenêtre", "Das Fenster kennenlernen"),
    subtitle: () =>
      tr(
        "Barra delle attività, barra laterale, schede, barra di stato e pannello «Attività»: dove sono le cose e a che cosa servono.",
        "Activity bar, sidebar, tabs, status bar and the “Activity” panel: where things are and what they are for.",
        "Barre d'activité, barre latérale, onglets, barre d'état et panneau « Activité » : où se trouvent les choses et à quoi elles servent.",
        "Aktivitätsleiste, Seitenleiste, Tabs, Statusleiste und Bereich „Aktivität“: wo sich alles befindet und wozu es dient.",
      ),
    icon: "layout-sidebar-left",
    keywords:
      "window finestra barra attività stato pannello schede layout interfaccia ctrl+p ctrl+shift+p palette comandi palette dei comandi scorciatoie command palette commands shortcuts palette de commandes raccourcis befehlspalette befehle tastenkürzel activity bar status bar sidebar tabs panel fenêtre barre d'activité barre d'état panneau onglets fenster aktivitätsleiste statusleiste seitenleiste tabs bereich mostra nascondi pannello show hide panel afficher masquer panneau anzeigen ausblenden",
  },
  {
    id: "first-project",
    group: "start",
    title: () =>
      tr(
        "Il primo progetto, passo per passo",
        "The first project, step by step",
        "Le premier projet, pas à pas",
        "Das erste Projekt, Schritt für Schritt",
      ),
    subtitle: () =>
      tr(
        "Un percorso completo in dieci minuti: creare un progetto, avviarlo, vederlo nel browser e aprirlo nell'editor.",
        "A complete walkthrough in ten minutes: create a project, start it, see it in the browser and open it in the editor.",
        "Un parcours complet en dix minutes : créer un projet, le démarrer, le voir dans le navigateur et l'ouvrir dans l'éditeur.",
        "Ein vollständiger Durchgang in zehn Minuten: ein Projekt erstellen, starten, im Browser ansehen und im Editor öffnen.",
      ),
    icon: "debug-start",
    keywords:
      "tutorial esempio primo progetto passo passo vite laravel avviare browser example first project step by step start exemple premier projet pas à pas démarrer beispiel erstes projekt schritt für schritt starten",
    targetPage: "new",
  },

  // 2. Projects
  {
    id: "project-new",
    group: "projects",
    title: () => tr("Creare un progetto", "Creating a project", "Créer un projet", "Ein Projekt erstellen"),
    subtitle: () =>
      tr(
        "Le tre strade: una cartella vuota, un repository Git, o una base di framework o CMS con le sue aggiunte. Più i tuoi repository GitHub.",
        "The three ways: an empty folder, a Git repository, or a framework or CMS base with its additions. Plus your GitHub repositories.",
        "Les trois voies : un dossier vide, un dépôt Git, ou une base de framework ou de CMS avec ses ajouts. Plus vos dépôts GitHub.",
        "Die drei Wege: ein leerer Ordner, ein Git-Repository oder eine Framework- oder CMS-Basis mit ihren Ergänzungen. Dazu deine GitHub-Repositorys.",
      ),
    icon: "new-folder",
    keywords:
      "nuovo progetto wizard framework cms clona git vuoto base aggiunte laravel vite django wordpress le mie basi salva come base modello new project clone empty base additions my bases save as a base template nouveau projet cloner vide base ajouts mes bases enregistrer comme base modèle neues projekt klonen leer basis ergänzungen meine basen als basis speichern vorlage",
    targetPage: "new",
  },
  {
    id: "project-page",
    group: "projects",
    title: () =>
      tr(
        "La pagina Progetti e il «Pannello rapido»",
        "The Projects page and the “Quick panel”",
        "La page Projets et le « Panneau rapide »",
        "Die Seite Projekte und die „Schnellübersicht“",
      ),
    subtitle: () =>
      tr(
        "L'elenco dei progetti, il «Pannello rapido» con preferiti e recenti, l'apertura veloce con Ctrl+P, i comandi con Ctrl+Shift+P e le sette sezioni di un progetto.",
        "The list of projects, the “Quick panel” with favorites and recent ones, quick open with Ctrl+P, commands with Ctrl+Shift+P and the seven sections of a project.",
        "La liste des projets, le « Panneau rapide » avec les favoris et les récents, l'ouverture rapide avec Ctrl+P, les commandes avec Ctrl+Shift+P et les sept sections d'un projet.",
        "Die Liste der Projekte, die „Schnellübersicht“ mit Favoriten und zuletzt verwendeten, das schnelle Öffnen mit Ctrl+P, die Befehle mit Ctrl+Shift+P und die sieben Abschnitte eines Projekts.",
      ),
    icon: "folder-library",
    keywords:
      "progetti pannello rapido preferiti recenti quick open ctrl+p ctrl+shift+p palette comandi command palette avvia ferma start stop palette de commandes befehlspalette salva come base save as a base enregistrer comme base als basis speichern sezioni scheda progetto projects quick panel favorites recent sections projets panneau rapide favoris récents sections projekte schnellübersicht favoriten zuletzt abschnitte",
    targetPage: "projects",
  },
  {
    id: "project-run",
    group: "projects",
    title: () =>
      tr(
        "Avviare un progetto: azioni e porte",
        "Starting a project: actions and ports",
        "Démarrer un projet : actions et ports",
        "Ein Projekt starten: Aktionen und Ports",
      ),
    subtitle: () =>
      tr(
        "Che cos'è un'azione, come si avvia e si ferma, l'output, la porta, l'azione predefinita e come aggiungerne di tue nel file panel.json.",
        "What an action is, how to start and stop it, the output, the port, the default action and how to add your own in the panel.json file.",
        "Ce qu'est une action, comment la démarrer et l'arrêter, la sortie, le port, l'action par défaut et comment ajouter les vôtres dans le fichier panel.json.",
        "Was eine Aktion ist, wie man sie startet und beendet, die Ausgabe, der Port, die Standardaktion und wie du eigene in der Datei panel.json hinzufügst.",
      ),
    icon: "debug-start",
    keywords:
      "avvio azioni porta output predefinita panel.json avvia ferma comando npm run dev indirizzo dal telefono qr rete locale lan start actions port default stop command address phone qr code local network démarrer actions port par défaut arrêter commande adresse téléphone réseau local starten aktionen port standard beenden befehl adresse telefon lokales netzwerk",
    targetPage: "projects",
  },
  {
    id: "project-versions",
    group: "projects",
    title: () =>
      tr(
        "Componenti e versioni di un progetto",
        "Components and versions of a project",
        "Composants et versions d'un projet",
        "Komponenten und Versionen eines Projekts",
      ),
    subtitle: () =>
      tr(
        "Gli strumenti che servono a un progetto, la versione di ogni linguaggio scelta solo per lui e come il kit la riconosce dai file del progetto.",
        "The tools a project needs, the version of each language chosen just for it and how the kit recognizes it from the project's files.",
        "Les outils dont un projet a besoin, la version de chaque langage choisie pour lui seul et comment le kit la reconnaît à partir des fichiers du projet.",
        "Die Werkzeuge, die ein Projekt braucht, die nur für es gewählte Version jeder Sprache und wie das Kit sie an den Dateien des Projekts erkennt.",
      ),
    icon: "versions",
    keywords:
      "versioni componenti linguaggi node php python progetto dev-settings.json nvmrc versions components languages project versions composants langages projet versionen komponenten sprachen projekt",
    targetPage: "projects",
  },
  {
    id: "project-editor",
    group: "projects",
    title: () =>
      tr(
        "Editor, terminale ed estensioni",
        "Editor, terminal and extensions",
        "Éditeur, terminal et extensions",
        "Editor, Terminal und Erweiterungen",
      ),
    subtitle: () =>
      tr(
        "Aprire il progetto nell'editor o in una console del kit, scegliere le estensioni, le impostazioni dell'editor e i server MCP del progetto.",
        "Opening the project in the editor or in a kit console, choosing the extensions, the editor settings and the project's MCP servers.",
        "Ouvrir le projet dans l'éditeur ou dans une console du kit, choisir les extensions, les paramètres de l'éditeur et les serveurs MCP du projet.",
        "Das Projekt im Editor oder in einer Konsole des Kits öffnen, die Erweiterungen wählen, die Editor-Einstellungen und die MCP-Server des Projekts.",
      ),
    icon: "code",
    keywords:
      "editor vscodium terminale console estensioni open vsx impostazioni mcp progetto server pronti filesystem dbhub mysql mongodb editor terminal extensions settings ready servers éditeur terminal extensions paramètres serveurs prêts editor terminal erweiterungen einstellungen fertige server",
    targetPage: "projects",
  },
  {
    id: "project-git",
    group: "projects",
    title: () => tr("Git di un progetto", "A project's Git", "Le Git d'un projet", "Git eines Projekts"),
    subtitle: () =>
      tr(
        "Che cosa mostra la sezione «Git», dove si fanno commit e push e perché il kit legge lo stato senza eseguire Git.",
        "What the “Git” section shows, where you commit and push, and why the kit reads the status without running Git.",
        "Ce que montre la section « Git », où faire les commits et les push, et pourquoi le kit lit l'état sans exécuter Git.",
        "Was der Abschnitt „Git“ zeigt, wo Commits und Pushes gemacht werden und warum das Kit den Status liest, ohne Git auszuführen.",
      ),
    icon: "git-merge",
    keywords:
      "git ramo branch remoto commit push sincronizzato repository branch remote sync branche dépôt distant synchronisé branch remote synchron",
    targetPage: "projects",
  },
  {
    id: "project-cleanup",
    group: "projects",
    title: () =>
      tr(
        "Eliminare un progetto e i «Dati senza progetto»",
        "Deleting a project and the “Data without a project”",
        "Supprimer un projet et les « Données sans projet »",
        "Ein Projekt löschen und die „Daten ohne Projekt“",
      ),
    subtitle: () =>
      tr(
        "Che cosa si elimina e che cosa resta, e come ricollegare o pulire i dati di un progetto rinominato o cancellato fuori dall'app.",
        "What gets deleted and what stays, and how to relink or clean up the data of a project renamed or deleted outside the app.",
        "Ce qui est supprimé et ce qui reste, et comment relier ou nettoyer les données d'un projet renommé ou supprimé en dehors de l'application.",
        "Was gelöscht wird und was bleibt, und wie du die Daten eines außerhalb der App umbenannten oder gelöschten Projekts wieder verknüpfst oder aufräumst.",
      ),
    icon: "trash",
    keywords:
      "elimina progetto dati senza progetto orfani collega pulizia rinominato delete project orphan data link cleanup renamed supprimer projet données sans projet lier nettoyage renommé löschen projekt daten ohne projekt verknüpfen aufräumen umbenannt",
    targetPage: "projects",
  },

  // 3. Databases, websites, terminals
  {
    id: "databases",
    group: "services",
    title: () =>
      tr("I server di database", "The database servers", "Les serveurs de bases de données", "Die Datenbankserver"),
    subtitle: () =>
      tr(
        "MySQL e MongoDB del kit: avviarli, le password, le porte, più versioni insieme, phpMyAdmin, mongo-express e l'esploratore dei dati.",
        "The kit's MySQL and MongoDB: starting them, passwords, ports, several versions side by side, phpMyAdmin, mongo-express and the data explorer.",
        "MySQL et MongoDB du kit : les démarrer, les mots de passe, les ports, plusieurs versions ensemble, phpMyAdmin, mongo-express et l'explorateur de données.",
        "MySQL und MongoDB des Kits: sie starten, die Passwörter, die Ports, mehrere Versionen gleichzeitig, phpMyAdmin, mongo-express und der Daten-Explorer.",
      ),
    icon: "database",
    keywords:
      "database mysql mongodb phpmyadmin mongo-express porta password versione server dati database server port password version bases de données serveur port mot de passe version datenbank server port passwort version",
    targetPage: "databases",
  },
  {
    id: "project-db",
    group: "services",
    title: () =>
      tr(
        "Il database di un progetto",
        "A project's database",
        "La base de données d'un projet",
        "Die Datenbank eines Projekts",
      ),
    subtitle: () =>
      tr(
        "Un database e un utente solo per il progetto, la connessione scritta nel file .env, le copie di sicurezza e il ripristino.",
        "A database and a user just for the project, the connection written in the .env file, backups and restoring.",
        "Une base de données et un utilisateur rien que pour le projet, la connexion écrite dans le fichier .env, les sauvegardes et la restauration.",
        "Eine Datenbank und ein Benutzer nur für das Projekt, die in die Datei .env geschriebene Verbindung, Sicherungskopien und Wiederherstellung.",
      ),
    icon: "key",
    keywords:
      "database progetto utente password .env database_url db_host copie backup ripristino sqlite copia di sicurezza prima-del-ripristino prima-del-cambio-di-versione cambio versione copia blocco .env mongodb_uri stringa di connessione appunti copy .env block connection string clipboard copier le bloc chaîne de connexion presse-papiers block kopieren verbindungszeichenfolge zwischenablage project database user password backup restore safety copy version change base de données projet utilisateur mot de passe sauvegarde restauration projektdatenbank benutzer passwort sicherung wiederherstellung",
    targetPage: "projects",
  },
  {
    id: "websites",
    group: "services",
    title: () =>
      tr(
        "Siti web e indirizzi .localhost",
        "Websites and .localhost addresses",
        "Sites web et adresses .localhost",
        "Websites und .localhost-Adressen",
      ),
    subtitle: () =>
      tr(
        "Servire una cartella come sito: file statici, PHP, Caddy o Apache. I server condivisi della pagina «Server Web», i domini .localhost e gli alias.",
        "Serving a folder as a website: static files, PHP, Caddy or Apache. The shared servers of the “Web Servers” page, .localhost domains and aliases.",
        "Servir un dossier comme site : fichiers statiques, PHP, Caddy ou Apache. Les serveurs partagés de la page « Serveurs web », les domaines .localhost et les alias.",
        "Einen Ordner als Website bereitstellen: statische Dateien, PHP, Caddy oder Apache. Die gemeinsamen Server der Seite „Webserver“, die .localhost-Domains und die Aliasse.",
      ),
    icon: "globe",
    keywords:
      "sito web server caddy apache pool profilo localhost dominio alias php lan rete locale qr telefono website web server domain local network phone site web serveur domaine réseau local website webserver domain lokales netzwerk conf.d",
    targetPage: "servers",
  },
  {
    id: "terminals",
    group: "services",
    title: () => tr("Terminali del kit", "The kit's terminals", "Les terminaux du kit", "Terminals des Kits"),
    subtitle: () =>
      tr(
        "Console sul PC con gli strumenti del kit in qualsiasi cartella, senza gli agenti, e terminali tuoi con strumenti e versioni scelti.",
        "Consoles on the PC with the kit's tools in any folder, without the agents, and terminals of your own with the tools and versions you choose.",
        "Des consoles sur le PC avec les outils du kit dans n'importe quel dossier, sans les agents, et vos propres terminaux avec les outils et versions de votre choix.",
        "Konsolen auf dem PC mit den Werkzeugen des Kits in jedem Ordner, ohne die Agenten, und eigene Terminals mit Werkzeugen und Versionen deiner Wahl.",
      ),
    icon: "terminal",
    keywords:
      "terminali console powershell bash prompt strumenti versioni path terminals console tools versions terminaux console outils versions terminals konsole werkzeuge versionen",
    targetPage: "terminals",
  },

  // 4. AI agents
  {
    id: "agents",
    group: "agents",
    title: () =>
      tr(
        "Gli agenti: Claude Code, Codex, Antigravity, Pi",
        "The agents: Claude Code, Codex, Antigravity, Pi",
        "Les agents : Claude Code, Codex, Antigravity, Pi",
        "Die Agenten: Claude Code, Codex, Antigravity, Pi",
      ),
    subtitle: () =>
      tr(
        "Che cosa sono, come si installano, come si aprono in un progetto, l'accesso al proprio account e l'uscita.",
        "What they are, how to install them, how to open them in a project, signing in to your account and signing out.",
        "Ce qu'ils sont, comment les installer, comment les ouvrir dans un projet, la connexion à votre compte et la déconnexion.",
        "Was sie sind, wie man sie installiert, wie man sie in einem Projekt öffnet, die Anmeldung bei deinem Konto und die Abmeldung.",
      ),
    icon: "hubot",
    keywords:
      "pagina ai page ia ki agenti claude code codex antigravity gemini chiave di gemini gemini key pi earendil accesso login token chiave api agents sign in login api key agents connexion clé api agenten anmeldung api-schlüssel",
    targetPage: "agents",
  },
  {
    id: "agents-shared",
    group: "agents",
    title: () =>
      tr(
        "Istruzioni, skill e server MCP condivisi",
        "Shared instructions, skills and MCP servers",
        "Instructions, skills et serveurs MCP partagés",
        "Gemeinsame Anweisungen, Skills und MCP-Server",
      ),
    subtitle: () =>
      tr(
        "Un'unica cartella per istruire gli agenti insieme: AGENTS.md, le skill, i server MCP del kit e quelli di un progetto.",
        "A single folder to instruct all the agents at once: AGENTS.md, the skills, the kit's MCP servers and those of a project.",
        "Un seul dossier pour instruire les agents ensemble : AGENTS.md, les skills, les serveurs MCP du kit et ceux d'un projet.",
        "Ein einziger Ordner, um alle Agenten gemeinsam anzuleiten: AGENTS.md, die Skills, die MCP-Server des Kits und die eines Projekts.",
      ),
    icon: "sync",
    keywords:
      "agents.md claude.md skill mcp mcp.json sincronizzazione condiviso istruzioni conflitti server pronti github playwright fetch filesystem dbhub mysql mongodb token shared instructions skills sync conflicts ready servers instructions partagées synchronisation conflits serveurs prêts gemeinsame anweisungen synchronisierung konflikte fertige server",
    targetPage: "agents",
  },
  {
    id: "agents-usage",
    group: "agents",
    title: () =>
      tr(
        "Limiti e consumi degli agenti",
        "Agent limits and usage",
        "Limites et consommation des agents",
        "Limits und Verbrauch der Agenten",
      ),
    subtitle: () =>
      tr(
        "Quanto del piano hai usato, quando si azzera, e i token e le sessioni di oggi e degli ultimi sette giorni.",
        "How much of your plan you have used, when it resets, and the tokens and sessions of today and the last seven days.",
        "Quelle part de votre forfait vous avez utilisée, quand elle se réinitialise, et les tokens et sessions d'aujourd'hui et des sept derniers jours.",
        "Wie viel deines Plans du verbraucht hast, wann er zurückgesetzt wird, und die Tokens und Sitzungen von heute und den letzten sieben Tagen.",
      ),
    icon: "graph",
    keywords:
      "limiti quote consumi token sessioni piano 5 ore settimana statistiche limits quota usage tokens sessions plan week limites quota consommation tokens sessions forfait semaine limits kontingent verbrauch tokens sitzungen plan woche",
    targetPage: "agents",
  },
  {
    id: "models",
    group: "agents",
    title: () =>
      tr(
        "Modelli locali: llama.cpp e i GGUF",
        "Local models: llama.cpp and GGUF",
        "Modèles locaux : llama.cpp et les GGUF",
        "Lokale Modelle: llama.cpp und GGUF",
      ),
    subtitle: () =>
      tr(
        "Modelli linguistici sul PC, senza rete e senza account: il server, la chat nel browser, il modello di Pi, scaricare dal catalogo o da Hugging Face.",
        "Language models on the PC, with no network and no account: the server, the chat in the browser, Pi's model, downloading from the catalog or from Hugging Face.",
        "Des modèles de langage sur le PC, sans réseau et sans compte : le serveur, le chat dans le navigateur, le modèle de Pi, télécharger depuis le catalogue ou depuis Hugging Face.",
        "Sprachmodelle auf dem PC, ohne Netzwerk und ohne Konto: der Server, der Chat im Browser, das Modell von Pi, Herunterladen aus dem Katalog oder von Hugging Face.",
      ),
    icon: "chip",
    keywords:
      "pagina ai page ia ki modelli locali llm llama.cpp llama-server gguf hugging face quantizzazione qwen gpt-oss devstral pi chat gpu vulkan metal ram memoria video scarica sospendi riprendi token chiave local models quantization download pause resume video memory key modèles locaux quantification télécharger suspendre reprendre mémoire vidéo clé lokale modelle quantisierung herunterladen anhalten fortsetzen grafikspeicher schlüssel",
    targetPage: "models",
  },

  // 5. Tools
  {
    id: "tools",
    group: "tools",
    title: () => tr("La pagina Strumenti", "The Tools page", "La page Outils", "Die Seite Werkzeuge"),
    subtitle: () =>
      tr(
        "La tabella di tutti gli strumenti divisa negli otto tipi, i cinque stati, installare e aggiornare subito o scegliere e poi «Applica».",
        "The table of all the tools split into the eight kinds, the five states, installing and updating right away or choosing first and then “Apply”.",
        "Le tableau de tous les outils réparti en huit types, les cinq états, installer et mettre à jour tout de suite ou choisir puis « Appliquer ».",
        "Die Tabelle aller Werkzeuge, nach den acht Arten aufgeteilt, die fünf Zustände, sofort installieren und aktualisieren oder erst auswählen und dann „Übernehmen“.",
      ),
    icon: "extensions",
    keywords:
      "strumenti componenti tabella tipi sistema ed editor agenti ai modelli locali llama.cpp local models modèles locaux lokale modelle linguaggi strumenti di sviluppo server web database scegli tutti stati installa aggiorna rimuovi applica aggiorna tutto pre-installato verifica i file reinstalla reinstalla tutti manifesto file danneggiati chiavetta danneggiata check the files reinstall reinstall all manifest damaged files damaged usb stick vérifier les fichiers réinstaller tout réinstaller manifeste fichiers endommagés dateien prüfen neu installieren alle neu installieren manifest beschädigte dateien aggiornamenti versioni nuove controlla ora cosa cambia note di rilascio changelog tutto aggiornato updates new versions check now what changes release notes up to date mises à jour nouvelles versions vérifier maintenant ce qui change notes de version aktualisierungen neue versionen jetzt prüfen was sich ändert versionshinweise tools components table kinds types system and editor ai agents languages development tools web server databases choose all states install update remove apply outils composants tableau types système et éditeur agents ia langages outils de développement serveur web bases de données tout choisir états installer mettre à jour retirer appliquer werkzeuge komponenten tabelle arten typ system und editor ki-agenten sprachen entwicklungswerkzeuge webserver datenbanken alle wählen zustände installieren aktualisieren entfernen übernehmen",
    targetPage: "components",
  },
  {
    id: "tools-versions",
    group: "tools",
    title: () =>
      tr(
        "Più versioni dello stesso strumento",
        "Several versions of the same tool",
        "Plusieurs versions du même outil",
        "Mehrere Versionen desselben Werkzeugs",
      ),
    subtitle: () =>
      tr(
        "Tenere PHP 8.3 e 8.4 o Node 20 e 24 una accanto all'altra: aggiungere, rimuovere, versioni bloccate e quale usa il kit.",
        "Keeping PHP 8.3 and 8.4 or Node 20 and 24 side by side: adding, removing, pinned versions and which one the kit uses.",
        "Garder PHP 8.3 et 8.4 ou Node 20 et 24 côte à côte : ajouter, retirer, versions verrouillées et laquelle le kit utilise.",
        "PHP 8.3 und 8.4 oder Node 20 und 24 nebeneinander behalten: hinzufügen, entfernen, gesperrte Versionen und welche das Kit verwendet.",
      ),
    icon: "library",
    keywords:
      "versioni linee multi-versione aggiungi versione bloccata più recente composer versions lines multi-version add pinned latest versions lignes plusieurs versions ajouter verrouillée plus récente versionen linien mehrere versionen hinzufügen gesperrt neueste python php rust compilatore compiler compilateur msvc gnu",
    targetPage: "components",
  },
  {
    id: "tools-overview",
    group: "tools",
    title: () =>
      tr(
        "Panoramiche: matrice, relazioni, estensioni, requisiti",
        "Overviews: matrix, relations, extensions, requirements",
        "Vues d'ensemble : matrice, relations, extensions, prérequis",
        "Übersichten: Matrix, Beziehungen, Erweiterungen, Anforderungen",
      ),
    subtitle: () =>
      tr(
        "Le viste d'insieme della pagina Strumenti: i quattro sistemi, chi richiede chi, le estensioni per linguaggio e ciò che serve al PC.",
        "The overall views of the Tools page: the four systems, who requires whom, the extensions per language and what the PC needs.",
        "Les vues d'ensemble de la page Outils : les quatre systèmes, qui requiert qui, les extensions par langage et ce dont le PC a besoin.",
        "Die Gesamtansichten der Seite Werkzeuge: die vier Systeme, wer wen benötigt, die Erweiterungen je Sprache und was der PC braucht.",
      ),
    icon: "table",
    keywords:
      "matrice relazioni estensioni requisiti pc piattaforma sistemi windows macos linux arm64 matrix relations extensions requirements platform systems matrice relations extensions prérequis plateforme systèmes matrix beziehungen erweiterungen anforderungen plattform systeme",
    targetPage: "components",
  },

  // 6. Account and settings
  {
    id: "github",
    group: "settings",
    title: () =>
      tr(
        "L'account GitHub del kit",
        "The kit's GitHub account",
        "Le compte GitHub du kit",
        "Das GitHub-Konto des Kits",
      ),
    subtitle: () =>
      tr(
        "Perché collegarlo, come si accede, che cosa sblocca (repository, Git, GitHub CLI) e che cosa funziona anche senza.",
        "Why to connect it, how to sign in, what it unlocks (repositories, Git, GitHub CLI) and what works without it too.",
        "Pourquoi le connecter, comment se connecter, ce qu'il débloque (dépôts, Git, GitHub CLI) et ce qui fonctionne aussi sans lui.",
        "Warum du es verbinden solltest, wie du dich anmeldest, was es freischaltet (Repositorys, Git, GitHub CLI) und was auch ohne es funktioniert.",
      ),
    icon: "github",
    keywords:
      "github account accesso login gh codice dispositivo repository clona permesso delete_repo github account sign in device code permission compte connexion code appareil autorisation konto anmeldung gerätecode berechtigung git identity identità user.name user.email noreply firma commit signs safe.directory",
    targetPage: "settings",
  },
  {
    id: "settings",
    group: "settings",
    title: () =>
      tr(
        "Le Impostazioni, una per una",
        "The Settings, one by one",
        "Les Paramètres, un par un",
        "Die Einstellungen, eine nach der anderen",
      ),
    subtitle: () =>
      tr(
        "Salvataggio automatico e le sei sezioni: lingua e tema, file temporanei, rete, aggiornamento automatico, chi firma i commit, privacy e le informazioni sul kit.",
        "Automatic saving and the six sections: language and theme, temporary files, network, automatic updates, who signs the commits, privacy and the information about the kit.",
        "Enregistrement automatique et les six sections : langue et thème, fichiers temporaires, réseau, mise à jour automatique, qui signe les commits, confidentialité et les informations sur le kit.",
        "Automatisches Speichern und die sechs Abschnitte: Sprache und Design, temporäre Dateien, Netzwerk, automatische Aktualisierung, wer die Commits signiert, Datenschutz und die Informationen zum Kit.",
      ),
    icon: "settings-gear",
    keywords:
      "impostazioni lingua tema chiaro scuro theme light dark thème clair sombre design hell dunkel git identity identità user.name user.email temporanei aggiornamento aggiornamento automatico versioni dei componenti automatic update automatische aktualisierung il kit privacy telemetria salvataggio esporta diagnostica segnala un problema salute del kit copia del kit esporta importa zip copy of the kit export import copie du kit exporter importer kopie des kits exportieren importieren kit health santé du kit zustand des kits settings export diagnostics report a problem language temporary update privacy telemetry save paramètres langue temporaires mise à jour confidentialité télémétrie enregistrement einstellungen sprache temporär aktualisierung datenschutz telemetrie speichern",
    targetPage: "settings",
  },
  {
    id: "network",
    group: "settings",
    title: () =>
      tr(
        "Rete aziendale: proxy, certificati, rete locale",
        "Company network: proxy, certificates, local network",
        "Réseau d'entreprise : proxy, certificats, réseau local",
        "Firmennetzwerk: Proxy, Zertifikate, lokales Netzwerk",
      ),
    subtitle: () =>
      tr(
        "Quando servono, come compilarli e a che cosa si applicano. Più come far vedere i tuoi siti agli altri PC della rete.",
        "When you need them, how to fill them in and what they apply to. Plus how to let the other PCs on the network see your websites.",
        "Quand ils sont nécessaires, comment les remplir et à quoi ils s'appliquent. Plus comment rendre vos sites visibles aux autres PC du réseau.",
        "Wann du sie brauchst, wie du sie ausfüllst und wofür sie gelten. Dazu, wie andere PCs im Netzwerk deine Websites sehen können.",
      ),
    icon: "remote",
    keywords:
      "rete proxy https http certificati pem ca aziendale lan rete locale no_proxy qr telefono cellulare network proxy certificates company local network phone mobile qr code réseau proxy certificats entreprise réseau local téléphone netzwerk proxy zertifikate firma lokales netzwerk telefon handy",
    targetPage: "settings",
  },
  {
    id: "frameworks",
    group: "settings",
    title: () =>
      tr(
        "Le istruzioni dei framework",
        "The framework instructions",
        "Les instructions des frameworks",
        "Die Anweisungen der Frameworks",
      ),
    subtitle: () =>
      tr(
        "Modificare le ricette di «Nuovo progetto»: basi, aggiunte, comandi, variabili .env, azioni. E provare una combinazione prima di usarla.",
        "Editing the recipes of “New project”: bases, additions, commands, .env variables, actions. And trying a combination before using it.",
        "Modifier les recettes de « Nouveau projet » : bases, ajouts, commandes, variables .env, actions. Et essayer une combinaison avant de l'utiliser.",
        "Die Rezepte von „Neues Projekt“ bearbeiten: Basen, Ergänzungen, Befehle, .env-Variablen, Aktionen. Und eine Kombination ausprobieren, bevor du sie verwendest.",
      ),
    icon: "wand",
    keywords:
      "framework ricette istruzioni basi aggiunte prova una combinazione frameworks.json template mine le mie basi my bases mes bases meine basen framework recipes instructions bases additions try combination recettes instructions bases ajouts essayer combinaison rezepte anweisungen basen ergänzungen ausprobieren kombination",
    targetPage: "settings",
  },

  // 7. Carrying the kit
  {
    id: "eject",
    group: "portable",
    title: () =>
      tr(
        "Rimuovere l'unità in sicurezza",
        "Safely removing the drive",
        "Retirer le lecteur en toute sécurité",
        "Das Laufwerk sicher entfernen",
      ),
    subtitle: () =>
      tr(
        "Chiudere tutto quello che il kit ha aperto prima di staccare la chiavetta o il disco, senza perdere dati.",
        "Closing everything the kit has opened before unplugging the USB stick or the disk, without losing data.",
        "Fermer tout ce que le kit a ouvert avant de débrancher la clé ou le disque, sans perdre de données.",
        "Alles schließen, was das Kit geöffnet hat, bevor du den USB-Stick oder die Festplatte abziehst, ohne Daten zu verlieren.",
      ),
    icon: "debug-disconnect",
    keywords:
      "rimuovi unità espelli chiavetta usb sicura chiudi tutto remove drive eject usb stick safe close all retirer lecteur éjecter clé usb sécurité fermer tout laufwerk entfernen auswerfen usb-stick sicher alles schließen",
    targetPage: "eject",
  },
  {
    id: "move",
    group: "portable",
    title: () =>
      tr(
        "Su un altro PC, un'altra lettera, un'altra cartella",
        "On another PC, another letter, another folder",
        "Sur un autre PC, une autre lettre, un autre dossier",
        "Auf einem anderen PC, unter einem anderen Buchstaben, in einem anderen Ordner",
      ),
    subtitle: () =>
      tr(
        "Che cosa succede quando il kit cambia posto, e come si comporta sullo stesso disco aperto da Windows, macOS e Linux.",
        "What happens when the kit changes place, and how it behaves on the same disk opened from Windows, macOS and Linux.",
        "Ce qui se passe quand le kit change de place, et comment il se comporte sur le même disque ouvert depuis Windows, macOS et Linux.",
        "Was passiert, wenn das Kit seinen Ort wechselt, und wie es sich auf derselben Festplatte verhält, die unter Windows, macOS und Linux geöffnet wird.",
      ),
    icon: "layers",
    keywords:
      "spostare kit altro pc lettera unità cartella rilocazione macos linux windows stesso disco copia del kit importa move kit another pc drive letter folder relocation same disk copy of the kit import déplacer kit autre pc lettre lecteur dossier même disque verschieben kit anderer pc laufwerksbuchstabe ordner gleiche festplatte",
  },
  {
    id: "updates",
    group: "portable",
    title: () =>
      tr(
        "Aggiornamenti e copie di sicurezza",
        "Updates and backups",
        "Mises à jour et sauvegardes",
        "Aktualisierungen und Sicherungskopien",
      ),
    subtitle: () =>
      tr(
        "Aggiornare Atollo e i suoi strumenti, e che cosa copiare per avere un backup completo del tuo lavoro.",
        "Updating Atollo and its tools, and what to copy to have a complete backup of your work.",
        "Mettre à jour Atollo et ses outils, et ce qu'il faut copier pour avoir une sauvegarde complète de votre travail.",
        "Atollo und seine Werkzeuge aktualisieren, und was du kopieren musst, um eine vollständige Sicherung deiner Arbeit zu haben.",
      ),
    icon: "cloud-download",
    keywords:
      "aggiornamenti aggiorna atollo release backup copia sicurezza copia del kit esporta importa zip atollo-before-import copy of the kit export import copie du kit exporter importer kopie des kits exportieren importieren updates update backup copy mises à jour mettre à jour sauvegarde copie aktualisierungen aktualisieren sicherung kopie",
    targetPage: "settings",
  },

  // 8. How it works
  {
    id: "vm",
    group: "inside",
    title: () =>
      tr(
        "La VM dei progetti (Docker Sandboxes)",
        "The projects' VM (Docker Sandboxes)",
        "La VM des projets (Docker Sandboxes)",
        "Die VM der Projekte (Docker Sandboxes)",
      ),
    subtitle: () =>
      tr(
        "La microVM dove girano gli agenti, mentre il resto gira sul PC: che cosa serve su ogni sistema, il primo accesso, che cosa gira dove, l'avviso «Senza VM», GitHub nella VM e le sandbox.",
        "The microVM where the agents run, while the rest runs on the PC: what each system needs, the first sign-in, what runs where, the “No VM” notice, GitHub in the VM and the sandboxes.",
        "La microVM où tournent les agents, tandis que le reste tourne sur le PC : ce qu'il faut sur chaque système, la première connexion, ce qui tourne où, l'avis « Sans VM », GitHub dans la VM et les sandbox.",
        "Die microVM, in der die Agenten laufen, während der Rest auf dem PC läuft: was jedes System braucht, die erste Anmeldung, was wo läuft, der Hinweis „Ohne VM“, GitHub in der VM und die Sandboxes.",
      ),
    icon: "vm",
    keywords:
      "vm microvm docker sandboxes sbx sandbox isolamento senza vm sbx login account docker hypervisor platform kvm apparmor box rimuovi sandbox github nella vm requisiti virtualizzazione firmware annidata terminali del kit agenti pi antigravity isolation no vm sign in docker account remove sandbox github in the vm requirements virtualization nested kit terminals agents isolement sans vm connexion compte docker supprimer sandbox github dans la vm prérequis virtualisation imbriquée terminaux du kit isolierung ohne vm anmeldung docker-konto sandbox entfernen github in der vm voraussetzungen virtualisierung verschachtelt terminals des kits agenten",
  },
  {
    id: "isolation",
    group: "inside",
    title: () =>
      tr(
        "Portabile e isolato: che cosa vuol dire",
        "Portable and isolated: what it means",
        "Portable et isolé : ce que cela veut dire",
        "Portabel und isoliert: was das bedeutet",
      ),
    subtitle: () =>
      tr(
        "Perché il kit non si installa, non chiede diritti di amministratore e non tocca il PC che lo ospita; l'ambiente sigillato dei programmi del kit; le eccezioni.",
        "Why the kit does not install, does not ask for administrator rights and does not touch the PC that hosts it; the sealed environment of the kit's programs; the exceptions.",
        "Pourquoi le kit ne s'installe pas, ne demande pas de droits d'administrateur et ne touche pas au PC qui l'héberge ; l'environnement scellé des programmes du kit ; les exceptions.",
        "Warum sich das Kit nicht installiert, keine Administratorrechte verlangt und den PC, auf dem es läuft, nicht verändert; die versiegelte Umgebung der Programme des Kits; die Ausnahmen.",
      ),
    icon: "shield",
    keywords:
      "portabile isolamento variabili ambiente registro amministratore tracce env portable isolation environment variables registry administrator traces portable isolement variables d'environnement registre administrateur traces portabel isolierung umgebungsvariablen registrierung administrator spuren",
  },
  {
    id: "disk-layout",
    group: "inside",
    title: () =>
      tr(
        "Le cartelle del kit su disco",
        "The kit's folders on disk",
        "Les dossiers du kit sur le disque",
        "Die Ordner des Kits auf der Festplatte",
      ),
    subtitle: () =>
      tr(
        "projects e data: che cosa c'è in ogni cartella, quali file sono tuoi, quali del kit e quali sono protetti.",
        "projects and data: what is in each folder, which files are yours, which belong to the kit and which are protected.",
        "projects et data : ce qu'il y a dans chaque dossier, quels fichiers sont à vous, lesquels sont au kit et lesquels sont protégés.",
        "projects und data: was in jedem Ordner liegt, welche Dateien dir gehören, welche dem Kit und welche geschützt sind.",
      ),
    icon: "folder-library",
    keywords:
      "cartelle disco projects data apps home settings.json components.txt diagnostics struttura folders disk structure dossiers disque structure ordner festplatte struktur",
  },
  {
    id: "lifecycle",
    group: "inside",
    title: () =>
      tr("Processi in background", "Background processes", "Processus en arrière-plan", "Hintergrundprozesse"),
    subtitle: () =>
      tr(
        "Che cosa parte con l'app, che cosa si ferma quando la chiudi, e perché non restano programmi accesi a tua insaputa.",
        "What starts with the app, what stops when you close it, and why no programs stay running without your knowledge.",
        "Ce qui démarre avec l'application, ce qui s'arrête quand vous la fermez, et pourquoi aucun programme ne reste allumé à votre insu.",
        "Was mit der App startet, was beim Schließen beendet wird und warum keine Programme ohne dein Wissen weiterlaufen.",
      ),
    icon: "pulse",
    keywords:
      "processi background job avvio chiusura istanza singola server demoni processes background startup shutdown single instance daemons processus arrière-plan démarrage fermeture instance unique prozesse hintergrund start beenden einzelinstanz",
  },
  {
    id: "sources",
    group: "inside",
    title: () =>
      tr(
        "Da dove arrivano gli strumenti",
        "Where the tools come from",
        "D'où viennent les outils",
        "Woher die Werkzeuge stammen",
      ),
    subtitle: () =>
      tr(
        "Le fonti ufficiali di ogni componente, le verifiche crittografiche, il download a mano e la tabella completa delle fonti.",
        "The official sources of each component, the cryptographic checks, downloading by hand and the full table of sources.",
        "Les sources officielles de chaque composant, les vérifications cryptographiques, le téléchargement à la main et le tableau complet des sources.",
        "Die offiziellen Quellen jeder Komponente, die kryptografischen Prüfungen, der Download von Hand und die vollständige Tabelle der Quellen.",
      ),
    icon: "lock",
    keywords:
      "fonti download verifica sha-256 hash firma sicurezza tabella fonti a mano by hand sources download verification signature security sources téléchargement vérification signature sécurité quellen download prüfung signatur sicherheit",
    targetPage: "components",
  },
  {
    id: "cli",
    group: "inside",
    title: () => tr("La riga di comando", "The command line", "La ligne de commande", "Die Kommandozeile"),
    subtitle: () =>
      tr(
        "I comandi dell'eseguibile di Atollo per preparare, controllare e sincronizzare il kit senza aprire la finestra.",
        "The commands of the Atollo executable to prepare, check and sync the kit without opening the window.",
        "Les commandes de l'exécutable d'Atollo pour préparer, vérifier et synchroniser le kit sans ouvrir la fenêtre.",
        "Die Befehle der Atollo-Programmdatei, um das Kit vorzubereiten, zu prüfen und zu synchronisieren, ohne das Fenster zu öffnen.",
      ),
    icon: "symbol-keyword",
    keywords:
      "cli riga di comando prepare check check --kit check --files verifica i file salute del kit kit health components versions env sync models llm diagnostics diagnostica powershell command line ligne de commande kommandozeile befehlszeile",
  },

  // 9. Help
  {
    id: "troubleshooting",
    group: "help",
    title: () =>
      tr(
        "Quando qualcosa non va",
        "When something goes wrong",
        "Quand quelque chose ne va pas",
        "Wenn etwas nicht funktioniert",
      ),
    subtitle: () =>
      tr(
        "Su un PC nuovo, prima la «Salute del kit». Poi i problemi più comuni, il messaggio che vedi e che cosa fare: download, porte occupate, server che non partono, accessi scaduti. E come segnalare un problema con lo zip della diagnostica.",
        "On a new PC, “Kit health” first. Then the most common problems, the message you see and what to do: downloads, ports in use, servers that won't start, expired sign-ins. And how to report a problem with the diagnostics zip.",
        "Sur un nouveau PC, d'abord la « Santé du kit ». Puis les problèmes les plus courants, le message que vous voyez et quoi faire : téléchargements, ports occupés, serveurs qui ne démarrent pas, connexions expirées. Et comment signaler un problème avec le zip du diagnostic.",
        "Auf einem neuen PC zuerst der „Zustand des Kits“. Dann die häufigsten Probleme, die Meldung, die du siehst, und was zu tun ist: Downloads, belegte Ports, Server, die nicht starten, abgelaufene Anmeldungen. Und wie du ein Problem mit dem Diagnose-Zip meldest.",
      ),
    icon: "warning",
    keywords:
      "problemi errori risoluzione salute del kit kit health santé du kit zustand des kits pc nuovo new pc fat32 exfat noexec spazio libero free space percorso path permessi permissions takeown icacls chown librerie libraries libaio libnuma libicu libvulkan1 porta occupata programma si chiude libreria mancante chiavetta danneggiata verifica i file reinstalla program crashes missing library damaged usb stick check the files reinstall programme plante bibliothèque manquante vérifier les fichiers programm stürzt ab fehlende bibliothek dateien prüfen download fallito non parte errore segnala un problema segnalazione diagnostica esporta zip log issue github bug problems errors troubleshooting port in use download failed report a problem export diagnostics problèmes erreurs dépannage port occupé téléchargement échoué signaler un problème signalement exporter le diagnostic probleme fehler fehlerbehebung port belegt download fehlgeschlagen problem melden meldung diagnose exportieren",
  },
  {
    id: "faq",
    group: "help",
    title: () => tr("Domande frequenti", "Frequently asked questions", "Questions fréquentes", "Häufige Fragen"),
    subtitle: () =>
      tr(
        "Risposte brevi alle domande che tornano più spesso.",
        "Short answers to the questions that come up most often.",
        "Des réponses courtes aux questions qui reviennent le plus souvent.",
        "Kurze Antworten auf die Fragen, die am häufigsten gestellt werden.",
      ),
    icon: "question",
    keywords:
      "faq domande frequenti risposte frequently asked questions answers questions fréquentes réponses häufige fragen antworten",
  },
  {
    id: "glossary",
    group: "help",
    title: () => tr("Glossario", "Glossary", "Glossaire", "Glossar"),
    subtitle: () =>
      tr(
        "Le parole della guida spiegate in modo semplice, dalla A alla Z.",
        "The words of the guide explained simply, from A to Z.",
        "Les mots du guide expliqués simplement, de A à Z.",
        "Die Begriffe der Anleitung einfach erklärt, von A bis Z.",
      ),
    icon: "book",
    keywords:
      "glossario parole termini definizioni glossary words terms definitions glossaire mots termes définitions glossar begriffe definitionen",
  },
];

export const DEFAULT_GUIDE_TOPIC: GuideTopic = GUIDE_TOPICS[0] as GuideTopic;

/** The topic of an id (the first one for an unknown id). */
export function topicOf(id: GuideTopicId): GuideTopic {
  return GUIDE_TOPICS.find((x) => x.id === id) ?? DEFAULT_GUIDE_TOPIC;
}

/** The topic that describes a page of the app: where a link to that page
 * leads when there is no app (the site). */
export function pageTopic(page: AppPageTarget): GuideTopicId {
  const topics: Record<AppPageTarget, GuideTopicId> = {
    new: "project-new",
    projects: "project-page",
    agents: "agents",
    models: "models",
    databases: "databases",
    servers: "websites",
    terminals: "terminals",
    components: "tools",
    settings: "settings",
    eject: "eject",
  };
  return topics[page];
}

/** A page of the app by the name it has there. */
export function pageName(page: AppPageTarget): string {
  switch (page) {
    case "new":
      return tr("Nuovo progetto", "New project", "Nouveau projet", "Neues Projekt");
    case "projects":
      return tr("Progetti", "Projects", "Projets", "Projekte");
    // Agents and local models are the two panes of the AI page (D-165).
    case "agents":
      return tr("AI › Agenti", "AI › Agents", "IA › Agents", "KI › Agenten");
    case "models":
      return tr("AI › Modelli locali", "AI › Local models", "IA › Modèles locaux", "KI › Lokale Modelle");
    case "databases":
      return tr("Database", "Databases", "Bases de données", "Datenbanken");
    case "servers":
      return tr("Server Web", "Web Servers", "Serveurs web", "Webserver");
    case "terminals":
      return tr("Terminali", "Terminals", "Terminaux", "Terminals");
    case "components":
      return tr("Strumenti", "Tools", "Outils", "Werkzeuge");
    case "settings":
      return tr("Impostazioni", "Settings", "Paramètres", "Einstellungen");
    case "eject":
      return tr("Rimuovi l'unità", "Remove drive", "Retirer le lecteur", "Laufwerk entfernen");
  }
}

/** A chapter's title with its number: "3. Database, siti e terminali". */
export function chapterTitle(id: GuideGroupId): string {
  const i = GUIDE_GROUPS.findIndex((g) => g.id === id);
  return `${String(i + 1)}. ${GUIDE_GROUPS[i]?.title() ?? ""}`;
}

/** The chapter number of a group (1-based) and a topic's "n.m" code. */
export function sectionCode(topic: GuideTopic): string {
  const g = GUIDE_GROUPS.findIndex((x) => x.id === topic.group) + 1;
  const n = GUIDE_TOPICS.filter((x) => x.group === topic.group).indexOf(topic) + 1;
  return `${String(g)}.${String(n)}`;
}
