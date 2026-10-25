// Chapter 1 of the Guide, «Primi passi»: what Atollo is, the first start,
// the window and a first project from start to finish.
import { rich, tr } from "../core/lang";
import { GUIDE_GROUPS, GUIDE_TOPICS, type AppPageTarget, type GuideTopicId } from "../core/model";
import {
  Bullets,
  Callout,
  Cards,
  Flow,
  GuideCard,
  InBrief,
  Key,
  KitInfo,
  Only,
  P,
  Related,
  Section,
  See,
  Steps,
  Table,
  Under,
  WindowSketch,
} from "../core/widgets";
import type { TopicTexts } from "../core/types";

/** The cards of the chapters, each opening its first topic. */
function ChapterMap() {
  return (
    <Cards>
      {GUIDE_GROUPS.map((g, i) => {
        const topics = GUIDE_TOPICS.filter((x) => x.group === g.id);
        const first = topics[0];
        if (!first) return null;
        return (
          <GuideCard
            key={g.id}
            icon={first.icon}
            badge={String(i + 1)}
            title={g.title()}
            actionLabel={tr("Inizia da qui", "Start here", "Commencer ici", "Hier beginnen")}
            to={{ topic: first.id }}
          >
            {topics.map((x) => x.title()).join(" · ")}
          </GuideCard>
        );
      })}
    </Cards>
  );
}

/** The pages of the activity bar, each with a button that opens it. */
function PageMap() {
  const pages: { icon: string; name: string; text: string; page?: AppPageTarget; topic: GuideTopicId }[] = [
    {
      icon: "folder-library",
      name: tr("Progetti", "Projects", "Projets", "Projekte"),
      text: tr(
        "Il centro del lavoro: l'elenco dei progetti, il «Pannello rapido», la creazione di progetti nuovi e la pagina di ogni progetto.",
        "The heart of the work: the list of projects, the “Quick panel”, creating new projects and the page of each project.",
        "Le cœur du travail : la liste des projets, le « Panneau rapide », la création de nouveaux projets et la page de chaque projet.",
        "Das Herz der Arbeit: die Liste der Projekte, die „Schnellübersicht“, das Erstellen neuer Projekte und die Seite jedes Projekts.",
      ),
      page: "projects",
      topic: "project-page",
    },
    {
      icon: "sparkle",
      name: tr("AI", "AI", "IA", "KI"),
      text: tr(
        "Tutta l'IA del kit in due riquadri. «Agenti»: accesso al tuo account, limiti del piano e consumi, e ciò che gli agenti condividono (istruzioni, skill, server MCP). «Modelli locali»: i modelli linguistici sul PC per Pi e la chat nel browser, il server llama.cpp, i download dal catalogo e da Hugging Face.",
        "All of the kit's AI in two panes. “Agents”: signing in to your account, plan limits and usage, and what the agents share (instructions, skills, MCP servers). “Local models”: the language models on the PC for Pi and the chat in the browser, the llama.cpp server, downloads from the catalog and from Hugging Face.",
        "Toute l'IA du kit en deux volets. « Agents » : connexion à votre compte, limites du forfait et consommation, et ce que les agents partagent (instructions, skills, serveurs MCP). « Modèles locaux » : les modèles de langage sur le PC pour Pi et le chat dans le navigateur, le serveur llama.cpp, les téléchargements depuis le catalogue et Hugging Face.",
        "Die ganze KI des Kits in zwei Bereichen. „Agenten“: Anmeldung bei deinem Konto, Limits des Plans und Verbrauch und das, was die Agenten teilen (Anweisungen, Skills, MCP-Server). „Lokale Modelle“: die Sprachmodelle auf dem PC für Pi und den Chat im Browser, der llama.cpp-Server, Downloads aus dem Katalog und von Hugging Face.",
      ),
      page: "agents",
      topic: "agents",
    },
    {
      icon: "database",
      name: tr("Database", "Databases", "Bases de données", "Datenbanken"),
      text: tr(
        "I server MySQL e MongoDB del kit: avvio, password, porte, versioni, phpMyAdmin e mongo-express.",
        "The kit's MySQL and MongoDB servers: starting, passwords, ports, versions, phpMyAdmin and mongo-express.",
        "Les serveurs MySQL et MongoDB du kit : démarrage, mots de passe, ports, versions, phpMyAdmin et mongo-express.",
        "Die MySQL- und MongoDB-Server des Kits: Start, Passwörter, Ports, Versionen, phpMyAdmin und mongo-express.",
      ),
      page: "databases",
      topic: "databases",
    },
    {
      icon: "globe",
      name: tr("Server Web", "Web Servers", "Serveurs web", "Webserver"),
      text: tr(
        "I server Caddy e Apache condivisi fra i progetti e i siti .localhost che servono.",
        "The Caddy and Apache servers shared among projects and the .localhost websites they serve.",
        "Les serveurs Caddy et Apache partagés entre les projets et les sites .localhost qu'ils servent.",
        "Die von den Projekten gemeinsam genutzten Caddy- und Apache-Server und die .localhost-Websites, die sie bereitstellen.",
      ),
      page: "servers",
      topic: "websites",
    },
    {
      icon: "terminal",
      name: tr("Terminali", "Terminals", "Terminaux", "Terminals"),
      text: tr(
        "Console con gli strumenti del kit, in qualsiasi cartella del PC.",
        "Consoles with the kit's tools, in any folder on the PC.",
        "Des consoles avec les outils du kit, dans n'importe quel dossier du PC.",
        "Konsolen mit den Werkzeugen des Kits, in jedem Ordner des PCs.",
      ),
      page: "terminals",
      topic: "terminals",
    },
    {
      icon: "extensions",
      name: tr("Strumenti", "Tools", "Outils", "Werkzeuge"),
      text: tr(
        "Linguaggi, agenti, database e programmi del kit: installare, aggiornare, togliere, più versioni.",
        "The kit's languages, agents, databases and programs: install, update, remove, several versions.",
        "Langages, agents, bases de données et programmes du kit : installer, mettre à jour, retirer, plusieurs versions.",
        "Sprachen, Agenten, Datenbanken und Programme des Kits: installieren, aktualisieren, entfernen, mehrere Versionen.",
      ),
      page: "components",
      topic: "tools",
    },
    {
      icon: "book",
      name: tr("Guida", "Guide", "Guide", "Anleitung"),
      text: tr(
        "Questa guida. Ogni pagina ha anche un'icona a forma di libro che apre l'argomento giusto.",
        "This guide. Every page also has a book-shaped icon that opens the right topic.",
        "Ce guide. Chaque page a aussi une icône en forme de livre qui ouvre le bon sujet.",
        "Diese Anleitung. Jede Seite hat außerdem ein Buchsymbol, das das passende Thema öffnet.",
      ),
      topic: "welcome",
    },
    {
      icon: "settings-gear",
      name: tr("Impostazioni", "Settings", "Paramètres", "Einstellungen"),
      text: tr(
        "Lingua, tema, rete, account GitHub e chi firma i commit, aggiornamenti, privacy e istruzioni dei framework, salvati da soli; «Il kit», con la sua salute, la sua copia e la diagnostica.",
        "Language, theme, network, GitHub account and who signs the commits, updates, privacy and framework instructions, saved by themselves; “The kit”, with its health, its copy and the diagnostics.",
        "Langue, thème, réseau, compte GitHub et qui signe les commits, mises à jour, confidentialité et instructions des frameworks, enregistrés tout seuls ; « Le kit », avec sa santé, sa copie et le diagnostic.",
        "Sprache, Design, Netzwerk, GitHub-Konto und wer die Commits signiert, Aktualisierungen, Datenschutz und Anweisungen der Frameworks, die sich von selbst speichern; „Das Kit“ mit seinem Zustand, seiner Kopie und der Diagnose.",
      ),
      page: "settings",
      topic: "settings",
    },
    {
      icon: "debug-disconnect",
      name: tr("Rimuovi l'unità", "Remove drive", "Retirer le lecteur", "Laufwerk entfernen"),
      text: tr(
        "Chiude tutto ciò che il kit ha aperto, prima di staccare la chiavetta.",
        "Closes everything the kit has opened, before you unplug the USB stick.",
        "Ferme tout ce que le kit a ouvert, avant de débrancher la clé.",
        "Schließt alles, was das Kit geöffnet hat, bevor du den USB-Stick abziehst.",
      ),
      page: "eject",
      topic: "eject",
    },
  ];
  return (
    <Cards>
      {pages.map((p) => (
        <GuideCard
          key={p.name}
          icon={p.icon}
          title={p.name}
          actionLabel={
            p.page
              ? tr("Apri la pagina", "Open the page", "Ouvrir la page", "Seite öffnen")
              : tr("Leggi", "Read", "Lire", "Lesen")
          }
          to={p.page ? { page: p.page } : { topic: p.topic }}
        >
          {p.text}{" "}
          <See id={p.topic}>{tr("Come funziona", "How it works", "Comment ça marche", "Wie es funktioniert")}</See>
        </GuideCard>
      ))}
    </Cards>
  );
}

export const START: TopicTexts = {
  welcome: ({ info }) => (
    <>
      <InBrief
        points={[
          tr(
            "Atollo è un ambiente di sviluppo completo che vive in una sola cartella: su una chiavetta, un disco esterno o il disco del PC.",
            "Atollo is a complete development environment that lives in a single folder: on a USB stick, an external disk or the PC's own disk.",
            "Atollo est un environnement de développement complet qui tient dans un seul dossier : sur une clé USB, un disque externe ou le disque du PC.",
            "Atollo ist eine vollständige Entwicklungsumgebung, die in einem einzigen Ordner lebt: auf einem USB-Stick, einer externen Festplatte oder der Festplatte des PCs.",
          ),
          tr(
            "Non si installa e non chiede diritti di amministratore: copi la cartella su un altro PC e ritrovi tutto, progetti compresi.",
            "It does not install and does not ask for administrator rights: copy the folder to another PC and you find everything again, projects included.",
            "Il ne s'installe pas et ne demande pas de droits d'administrateur : copiez le dossier sur un autre PC et vous retrouvez tout, projets compris.",
            "Es wird nicht installiert und verlangt keine Administratorrechte: Kopiere den Ordner auf einen anderen PC und du findest alles wieder, einschließlich der Projekte.",
          ),
          tr(
            "Il suo cuore è il configuratore di Docker Sandboxes: ogni progetto gira in una microVM sua, che vede solo la sua cartella, con terminale, azioni, sito e agenti. Senza la VM tutto gira sul PC, e la barra di stato lo dice.",
            "Its heart is the configurator of Docker Sandboxes: each project runs in a microVM of its own, which sees only its folder, with terminal, actions, website and agents. Without the VM everything runs on the PC, and the status bar says so.",
            "Son cœur est le configurateur de Docker Sandboxes : chaque projet tourne dans sa propre microVM, qui ne voit que son dossier, avec terminal, actions, site et agents. Sans la VM, tout tourne sur le PC, et la barre d'état le dit.",
            "Sein Herz ist der Konfigurator von Docker Sandboxes: Jedes Projekt läuft in einer eigenen microVM, die nur seinen Ordner sieht, mit Terminal, Aktionen, Website und Agenten. Ohne die VM läuft alles auf dem PC, und die Statusleiste sagt es.",
          ),
          tr(
            "Dentro trovi linguaggi in più versioni, database, server web, un editor, console, Git e GitHub, quattro agenti IA e i modelli linguistici locali.",
            "Inside you find languages in several versions, databases, web servers, an editor, consoles, Git and GitHub, four AI agents and local language models.",
            "À l'intérieur, vous trouvez des langages en plusieurs versions, des bases de données, des serveurs web, un éditeur, des consoles, Git et GitHub, quatre agents IA et des modèles de langage locaux.",
            "Darin findest du Sprachen in mehreren Versionen, Datenbanken, Webserver, einen Editor, Konsolen, Git und GitHub, vier KI-Agenten und lokale Sprachmodelle.",
          ),
          tr(
            "Questa guida va dalle cose più semplici a quelle più tecniche: puoi leggerla in ordine o saltare dove ti serve.",
            "This guide goes from the simplest things to the most technical ones: you can read it in order or jump to what you need.",
            "Ce guide va des choses les plus simples aux plus techniques : vous pouvez le lire dans l'ordre ou aller directement à ce qui vous intéresse.",
            "Diese Anleitung reicht von den einfachsten bis zu den technischsten Dingen: Du kannst sie der Reihe nach lesen oder direkt zu dem springen, was du brauchst.",
          ),
        ]}
      />

      <Section
        title={tr(
          "Che cos'è, in parole semplici",
          "What it is, in simple words",
          "Ce que c'est, en termes simples",
          "Was es ist, in einfachen Worten",
        )}
      >
        <P>
          {tr(
            "Immagina uno zaino da lavoro già pronto: dentro c'è tutto ciò che serve per scrivere programmi e siti, ordinato e funzionante. Lo porti con te, lo apri su qualsiasi computer e lavori, senza installare niente e senza lasciare disordine sul PC che ti ospita. Quando hai finito, lo chiudi e lo porti via.",
            "Picture a work backpack that is already packed: inside is everything you need to write programs and websites, tidy and working. You carry it with you, open it on any computer and work, without installing anything and without leaving a mess on the PC that hosts you. When you are done, you close it and take it away.",
            "Imaginez un sac de travail déjà prêt : à l'intérieur, il y a tout ce qu'il faut pour écrire des programmes et des sites, bien rangé et fonctionnel. Vous l'emportez avec vous, vous l'ouvrez sur n'importe quel ordinateur et vous travaillez, sans rien installer et sans laisser de désordre sur le PC qui vous accueille. Quand vous avez fini, vous le fermez et vous l'emportez.",
            "Stell dir einen fertig gepackten Arbeitsrucksack vor: Darin ist alles, was du zum Schreiben von Programmen und Websites brauchst, ordentlich und funktionsfähig. Du nimmst ihn mit, öffnest ihn auf einem beliebigen Computer und arbeitest, ohne etwas zu installieren und ohne Unordnung auf dem PC zu hinterlassen, der dich beherbergt. Wenn du fertig bist, schließt du ihn und nimmst ihn mit.",
          )}
        </P>
        <P>
          {tr(
            "Atollo è quello zaino. È un unico programma che sta in una cartella insieme a due sottocartelle: projects, dove vivono i tuoi progetti, e data, dove il kit tiene i suoi programmi, le impostazioni e i dati dei database. Funziona su Windows 10 e 11, macOS e Linux, con processori x64 e ARM64 (su ARM64 serve Windows 11).",
            "Atollo is that backpack. It is a single program that sits in a folder together with two subfolders: projects, where your projects live, and data, where the kit keeps its programs, settings and database data. It works on Windows 10 and 11, macOS and Linux, with x64 and ARM64 processors (on ARM64 Windows 11 is needed).",
            "Atollo est ce sac. C'est un seul programme qui se trouve dans un dossier avec deux sous-dossiers : projects, où vivent vos projets, et data, où le kit garde ses programmes, ses paramètres et les données des bases de données. Il fonctionne sous Windows 10 et 11, macOS et Linux, avec des processeurs x64 et ARM64 (sur ARM64, Windows 11 est nécessaire).",
            "Atollo ist dieser Rucksack. Es ist ein einziges Programm, das in einem Ordner zusammen mit zwei Unterordnern liegt: projects, wo deine Projekte leben, und data, wo das Kit seine Programme, Einstellungen und die Daten der Datenbanken aufbewahrt. Es läuft unter Windows 10 und 11, macOS und Linux, mit x64- und ARM64-Prozessoren (auf ARM64 braucht es Windows 11).",
          )}
        </P>
        <P>
          {rich(
            tr(
              "E ogni progetto ha la sua stanza chiusa: una microVM di Docker Sandboxes, dove girano il terminale, le azioni, il sito e gli agenti del progetto con gli strumenti del kit per Linux. Il codice di un progetto non vede i tuoi file né gli altri progetti; i database restano sul PC, raggiungibili solo dalla porta del progetto. La VM chiede Windows 11 x64, un Mac con Apple silicon o Ubuntu 24.04, e un account Docker gratuito: dove manca, Atollo lavora sul PC e lo dice. I dettagli: {0}.",
              "And each project has its own closed room: a Docker Sandboxes microVM, where the project's terminal, actions, website and agents run with the kit's tools for Linux. A project's code sees neither your files nor the other projects; the databases stay on the PC, reachable only through the project's port. The VM needs Windows 11 x64, a Mac with Apple silicon or Ubuntu 24.04, and a free Docker account: where it is missing, Atollo works on the PC and says so. The details: {0}.",
              "Et chaque projet a sa propre pièce fermée : une microVM de Docker Sandboxes, où tournent le terminal, les actions, le site et les agents du projet avec les outils du kit pour Linux. Le code d'un projet ne voit ni vos fichiers ni les autres projets ; les bases de données restent sur le PC, joignables seulement par le port du projet. La VM demande Windows 11 x64, un Mac avec Apple silicon ou Ubuntu 24.04, et un compte Docker gratuit : là où elle manque, Atollo travaille sur le PC et le dit. Les détails : {0}.",
              "Und jedes Projekt hat seinen eigenen geschlossenen Raum: eine microVM von Docker Sandboxes, in der Terminal, Aktionen, Website und Agenten des Projekts mit den Werkzeugen des Kits für Linux laufen. Der Code eines Projekts sieht weder deine Dateien noch die anderen Projekte; die Datenbanken bleiben auf dem PC, nur über den Port des Projekts erreichbar. Die VM braucht Windows 11 x64, einen Mac mit Apple Silicon oder Ubuntu 24.04 und ein kostenloses Docker-Konto: Wo sie fehlt, arbeitet Atollo auf dem PC und sagt es. Die Details: {0}.",
            ),
            <See key="v" id="vm" />,
          )}
        </P>
        <Callout tone="info" title={tr("A chi serve", "Who it is for", "À qui il s'adresse", "Für wen es gedacht ist")}>
          {tr(
            "A chi programma su più computer (casa, ufficio, laboratorio), a chi non può installare programmi sul PC di lavoro, a chi insegna o studia e vuole lo stesso ambiente per tutti, e a chi vuole provare un linguaggio o un framework senza sporcare il proprio sistema.",
            "For people who code on several computers (home, office, lab), people who cannot install programs on their work PC, people who teach or study and want the same environment for everyone, and people who want to try a language or a framework without cluttering their own system.",
            "À ceux qui programment sur plusieurs ordinateurs (maison, bureau, laboratoire), à ceux qui ne peuvent pas installer de programmes sur leur PC de travail, à ceux qui enseignent ou étudient et veulent le même environnement pour tous, et à ceux qui veulent essayer un langage ou un framework sans encombrer leur propre système.",
            "Für alle, die auf mehreren Computern programmieren (zu Hause, im Büro, im Labor), die auf dem Arbeits-PC keine Programme installieren dürfen, die unterrichten oder lernen und für alle dieselbe Umgebung wollen, und für alle, die eine Sprache oder ein Framework ausprobieren möchten, ohne das eigene System zu belasten.",
          )}
        </Callout>
      </Section>

      <Section
        title={tr(
          "Che cosa trovi dentro",
          "What you find inside",
          "Ce que vous trouvez à l'intérieur",
          "Was du darin findest",
        )}
      >
        <Table
          columns={[
            tr("Che cosa", "What", "Quoi", "Was"),
            tr("A che cosa serve", "What it is for", "À quoi ça sert", "Wozu es dient"),
            tr(
              "Dove si trova nell'app",
              "Where it is in the app",
              "Où le trouver dans l'application",
              "Wo es in der App ist",
            ),
          ]}
          rows={[
            [
              tr("Progetti", "Projects", "Projets", "Projekte"),
              tr(
                "Le tue cartelle di lavoro, ognuna con le sue azioni, versioni, database e sito.",
                "Your working folders, each with its own actions, versions, database and website.",
                "Vos dossiers de travail, chacun avec ses actions, versions, base de données et site.",
                "Deine Arbeitsordner, jeder mit eigenen Aktionen, Versionen, Datenbank und Website.",
              ),
              tr("Progetti", "Projects", "Projets", "Projekte"),
            ],
            [
              tr("La VM dei progetti", "The projects' VM", "La VM des projets", "Die VM der Projekte"),
              tr(
                "Docker Sandboxes: una microVM per ogni progetto e per ogni terminale del kit, che vede solo la sua cartella.",
                "Docker Sandboxes: a microVM for each project and each kit terminal, which sees only its folder.",
                "Docker Sandboxes : une microVM pour chaque projet et chaque terminal du kit, qui ne voit que son dossier.",
                "Docker Sandboxes: eine microVM für jedes Projekt und jedes Terminal des Kits, die nur ihren Ordner sieht.",
              ),
              tr(
                "Barra di stato › «VM» o «Senza VM»",
                "Status bar › “VM” or “No VM”",
                "Barre d'état › « VM » ou « Sans VM »",
                "Statusleiste › „VM“ oder „Ohne VM“",
              ),
            ],
            [
              tr("Linguaggi e strumenti", "Languages and tools", "Langages et outils", "Sprachen und Werkzeuge"),
              tr(
                "Python, Node.js, PHP, Java, .NET, Go, Rust e molti altri, anche più versioni insieme.",
                "Python, Node.js, PHP, Java, .NET, Go, Rust and many others, even several versions side by side.",
                "Python, Node.js, PHP, Java, .NET, Go, Rust et bien d'autres, même en plusieurs versions à la fois.",
                "Python, Node.js, PHP, Java, .NET, Go, Rust und viele andere, auch in mehreren Versionen gleichzeitig.",
              ),
              tr("Strumenti", "Tools", "Outils", "Werkzeuge"),
            ],
            [
              tr("Database", "Databases", "Bases de données", "Datenbanken"),
              tr(
                "Un server MySQL e uno MongoDB, solo per questo PC, con le password già pronte.",
                "A MySQL server and a MongoDB one, for this PC only, with passwords already set up.",
                "Un serveur MySQL et un serveur MongoDB, pour ce PC uniquement, avec des mots de passe déjà prêts.",
                "Ein MySQL- und ein MongoDB-Server, nur für diesen PC, mit bereits eingerichteten Passwörtern.",
              ),
              tr("Database", "Databases", "Bases de données", "Datenbanken"),
            ],
            [
              tr("Server web", "Web servers", "Serveurs web", "Webserver"),
              tr(
                "Caddy e Apache per vedere i tuoi siti su indirizzi come mio-sito.localhost.",
                "Caddy and Apache, to see your websites at addresses like mio-sito.localhost.",
                "Caddy et Apache pour voir vos sites à des adresses comme mio-sito.localhost.",
                "Caddy und Apache, um deine Websites unter Adressen wie mio-sito.localhost zu sehen.",
              ),
              tr("Server Web", "Web Servers", "Serveurs web", "Webserver"),
            ],
            [
              tr("Editor", "Editor", "Éditeur", "Editor"),
              tr(
                "VSCodium, la versione libera di Visual Studio Code, in una finestra di Atollo.",
                "VSCodium, the free version of Visual Studio Code, in an Atollo window.",
                "VSCodium, la version libre de Visual Studio Code, dans une fenêtre d'Atollo.",
                "VSCodium, die freie Version von Visual Studio Code, in einem Atollo-Fenster.",
              ),
              tr("Progetti › Editor", "Projects › Editor", "Projets › Éditeur", "Projekte › Editor"),
            ],
            [
              tr("Console", "Console", "Console", "Konsole"),
              tr(
                "PowerShell 7 con tutti gli strumenti del kit già pronti; con la VM, bash nella microVM con gli strumenti del kit per Linux.",
                "PowerShell 7 with all the kit's tools ready to use; with the VM, bash in the microVM with the kit's tools for Linux.",
                "PowerShell 7 avec tous les outils du kit déjà prêts ; avec la VM, bash dans la microVM avec les outils du kit pour Linux.",
                "PowerShell 7 mit allen Werkzeugen des Kits, sofort einsatzbereit; mit der VM bash in der microVM mit den Werkzeugen des Kits für Linux.",
              ),
              tr("Terminali", "Terminals", "Terminaux", "Terminals"),
            ],
            [
              tr("Git e GitHub", "Git and GitHub", "Git et GitHub", "Git und GitHub"),
              tr(
                "Git e GitHub CLI con un solo accesso al tuo account per tutto il kit.",
                "Git and GitHub CLI with a single sign-in to your account for the whole kit.",
                "Git et GitHub CLI avec une seule connexion à votre compte pour tout le kit.",
                "Git und GitHub CLI mit einer einzigen Anmeldung bei deinem Konto für das ganze Kit.",
              ),
              tr(
                "Impostazioni › Account GitHub",
                "Settings › GitHub account",
                "Paramètres › Compte GitHub",
                "Einstellungen › GitHub-Konto",
              ),
            ],
            [
              tr("Agenti IA", "AI agents", "Agents IA", "KI-Agenten"),
              tr(
                "Claude Code, Codex, Antigravity e Pi, con istruzioni, skill e server MCP condivisi.",
                "Claude Code, Codex, Antigravity and Pi, with shared instructions, skills and MCP servers.",
                "Claude Code, Codex, Antigravity et Pi, avec instructions, skills et serveurs MCP partagés.",
                "Claude Code, Codex, Antigravity und Pi, mit gemeinsamen Anweisungen, Skills und MCP-Servern.",
              ),
              tr("AI › Agenti", "AI › Agents", "IA › Agents", "KI › Agenten"),
            ],
            [
              tr("Modelli locali", "Local models", "Modèles locaux", "Lokale Modelle"),
              tr(
                "Modelli linguistici sul PC, senza rete e senza account, per Pi e per una chat nel browser.",
                "Language models on the PC, with no network and no account, for Pi and for a chat in the browser.",
                "Des modèles de langage sur le PC, sans réseau et sans compte, pour Pi et pour un chat dans le navigateur.",
                "Sprachmodelle auf dem PC, ohne Netzwerk und ohne Konto, für Pi und für einen Chat im Browser.",
              ),
              tr("AI › Modelli locali", "AI › Local models", "IA › Modèles locaux", "KI › Lokale Modelle"),
            ],
          ]}
        />
      </Section>

      <Section
        title={tr(
          "Tre idee da tenere a mente",
          "Three ideas to keep in mind",
          "Trois idées à garder en tête",
          "Drei Ideen, die du im Kopf behalten solltest",
        )}
      >
        <Cards>
          <GuideCard
            icon="folder-library"
            title={tr(
              "Tutto in una cartella",
              "Everything in one folder",
              "Tout dans un dossier",
              "Alles in einem Ordner",
            )}
          >
            {tr(
              "Il kit scrive solo dentro la sua cartella (con poche eccezioni dichiarate). Per fare una copia di tutto basta copiare la cartella; per toglierlo basta cancellarla.",
              "The kit writes only inside its own folder (with a few declared exceptions). To make a copy of everything, just copy the folder; to remove it, just delete it.",
              "Le kit n'écrit que dans son propre dossier (à quelques exceptions déclarées près). Pour faire une copie de tout, il suffit de copier le dossier ; pour l'enlever, il suffit de le supprimer.",
              "Das Kit schreibt nur in seinen eigenen Ordner (mit wenigen angegebenen Ausnahmen). Um alles zu sichern, kopierst du einfach den Ordner; um es zu entfernen, löschst du ihn einfach.",
            )}
          </GuideCard>
          <GuideCard
            icon="extensions"
            title={tr(
              "Il kit pensa agli strumenti",
              "The kit takes care of the tools",
              "Le kit s'occupe des outils",
              "Das Kit kümmert sich um die Werkzeuge",
            )}
          >
            {tr(
              "Quando un progetto o un agente ha bisogno di un programma che manca, il kit lo scarica dalla fonte ufficiale, lo verifica e lo mette al suo posto. Tu scegli, lui installa.",
              "When a project or an agent needs a program that is missing, the kit downloads it from the official source, checks it and puts it in its place. You choose, it installs.",
              "Quand un projet ou un agent a besoin d'un programme qui manque, le kit le télécharge depuis la source officielle, le vérifie et le met à sa place. Vous choisissez, il installe.",
              "Wenn ein Projekt oder ein Agent ein fehlendes Programm braucht, lädt das Kit es von der offiziellen Quelle herunter, prüft es und legt es an seinen Platz. Du wählst, das Kit installiert.",
            )}
          </GuideCard>
          <GuideCard
            icon="globe"
            title={tr("Solo su questo PC", "Only on this PC", "Uniquement sur ce PC", "Nur auf diesem PC")}
          >
            {tr(
              "Database e siti rispondono solo al computer su cui stai lavorando (indirizzo 127.0.0.1), con password generate dal kit. Nessuno dalla rete li vede, a meno che tu non lo chieda.",
              "Databases and websites answer only the computer you are working on (address 127.0.0.1), with passwords generated by the kit. Nobody on the network sees them, unless you ask for it.",
              "Les bases de données et les sites ne répondent qu'à l'ordinateur sur lequel vous travaillez (adresse 127.0.0.1), avec des mots de passe générés par le kit. Personne sur le réseau ne les voit, sauf si vous le demandez.",
              "Datenbanken und Websites antworten nur dem Computer, an dem du arbeitest (Adresse 127.0.0.1), mit vom Kit erzeugten Passwörtern. Niemand im Netzwerk sieht sie, es sei denn, du wünschst es.",
            )}
          </GuideCard>
        </Cards>
      </Section>

      <Section
        title={tr(
          "Come è fatta questa guida",
          "How this guide is organized",
          "Comment ce guide est organisé",
          "Wie diese Anleitung aufgebaut ist",
        )}
      >
        <P>
          {tr(
            "La guida ha nove capitoli. I primi tre servono a tutti: come partire, come lavorare ai progetti, come usare database e siti. Poi agenti, strumenti e impostazioni. Il capitolo «Come funziona dentro» è per chi vuole capire i dettagli tecnici; l'ultimo raccoglie problemi comuni, domande frequenti e un glossario.",
            "The guide has nine chapters. The first three are for everyone: how to get started, how to work on projects, how to use databases and websites. Then agents, tools and settings. The “How it works inside” chapter is for those who want to understand the technical details; the last one gathers common problems, frequently asked questions and a glossary.",
            "Le guide compte neuf chapitres. Les trois premiers servent à tout le monde : comment démarrer, comment travailler sur les projets, comment utiliser les bases de données et les sites. Ensuite les agents, les outils et les paramètres. Le chapitre « Comment il fonctionne à l'intérieur » est pour ceux qui veulent comprendre les détails techniques ; le dernier rassemble les problèmes courants, les questions fréquentes et un glossaire.",
            "Die Anleitung hat neun Kapitel. Die ersten drei sind für alle: wie man anfängt, wie man an Projekten arbeitet, wie man Datenbanken und Websites nutzt. Dann Agenten, Werkzeuge und Einstellungen. Das Kapitel „Wie es innen funktioniert“ ist für alle, die die technischen Details verstehen wollen; das letzte sammelt häufige Probleme, häufige Fragen und ein Glossar.",
          )}
        </P>
        <P>
          {tr(
            "In ogni argomento trovi: un riquadro «In breve» all'inizio, i passaggi da seguire, consigli e avvertenze, e una parte ripiegata «Sotto il cofano» con i dettagli tecnici (file, cartelle, comandi).",
            "In every topic you find: an “In short” box at the start, the steps to follow, tips and warnings, and a collapsed “Under the hood” part with the technical details (files, folders, commands).",
            "Dans chaque sujet, vous trouvez : un encadré « En bref » au début, les étapes à suivre, des conseils et des avertissements, et une partie repliée « Sous le capot » avec les détails techniques (fichiers, dossiers, commandes).",
            "In jedem Thema findest du: einen Kasten „Kurz gesagt“ am Anfang, die Schritte, die du befolgst, Tipps und Warnungen, und einen eingeklappten Teil „Unter der Haube“ mit den technischen Details (Dateien, Ordner, Befehle).",
          )}{" "}
          <Only in="app">
            {tr(
              "Il pulsante in alto a destra apre la pagina dell'app di cui l'argomento parla. La casella di ricerca a sinistra cerca nei titoli, nelle descrizioni e nelle parole chiave.",
              "The button at the top right opens the app page the topic is about. The search box on the left searches titles, descriptions and keywords.",
              "Le bouton en haut à droite ouvre la page de l'application dont parle le sujet. Le champ de recherche à gauche cherche dans les titres, les descriptions et les mots-clés.",
              "Die Schaltfläche oben rechts öffnet die Seite der App, um die es im Thema geht. Das Suchfeld links durchsucht Titel, Beschreibungen und Stichwörter.",
            )}
          </Only>
          <Only in="web">
            {tr(
              "Sotto il titolo di ogni argomento c'è la pagina dell'app di cui parla. La casella di ricerca sopra i capitoli cerca nei titoli, nelle descrizioni e nelle parole chiave.",
              "Under the title of every topic is the app page it is about. The search box above the chapters searches titles, descriptions and keywords.",
              "Sous le titre de chaque sujet figure la page de l'application dont il parle. Le champ de recherche au-dessus des chapitres cherche dans les titres, les descriptions et les mots-clés.",
              "Unter dem Titel jedes Themas steht die Seite der App, um die es geht. Das Suchfeld über den Kapiteln durchsucht Titel, Beschreibungen und Stichwörter.",
            )}
          </Only>
        </P>
        <ChapterMap />
      </Section>

      <KitInfo info={info} show="card" />

      <Related ids={["first-run", "window", "first-project", "vm"]} />
    </>
  ),

  "first-run": () => (
    <>
      <InBrief
        points={[
          tr(
            "Metti il programma di Atollo in una cartella vuota, dove vuoi tenere il kit, e aprilo.",
            "Put the Atollo program in an empty folder, where you want to keep the kit, and open it.",
            "Placez le programme d'Atollo dans un dossier vide, là où vous voulez garder le kit, et ouvrez-le.",
            "Leg das Atollo-Programm in einen leeren Ordner, in dem du das Kit aufbewahren möchtest, und öffne es.",
          ),
          tr(
            "La prima volta compare la pagina «Benvenuto»: scegli lingua e, se serve, proxy e certificati, poi agenti e linguaggi, e premi «Installa». Prima di «Installa» non si scarica niente.",
            "The first time, the “Welcome” page appears: choose the language and, if needed, proxy and certificates, then agents and languages, and press “Install”. Nothing is downloaded before “Install”.",
            "La première fois, la page « Bienvenue » apparaît : choisissez la langue et, si besoin, le proxy et les certificats, puis les agents et les langages, et appuyez sur « Installer ». Rien n'est téléchargé avant « Installer ».",
            "Beim ersten Mal erscheint die Seite „Willkommen“: Wähle die Sprache und, falls nötig, Proxy und Zertifikate, dann Agenten und Sprachen, und klicke auf „Installieren“. Vor „Installieren“ wird nichts heruntergeladen.",
          ),
          tr(
            "I componenti si scaricano in sottofondo dalle fonti ufficiali: puoi già usare l'app mentre arrivano.",
            "The components download in the background from the official sources: you can already use the app while they arrive.",
            "Les composants se téléchargent en arrière-plan depuis les sources officielles : vous pouvez déjà utiliser l'application pendant qu'ils arrivent.",
            "Die Komponenten werden im Hintergrund von den offiziellen Quellen heruntergeladen: Du kannst die App bereits nutzen, während sie ankommen.",
          ),
          tr(
            "Niente è definitivo: tutto si aggiunge o si toglie più tardi dalla pagina Strumenti.",
            "Nothing is final: everything can be added or removed later from the Tools page.",
            "Rien n'est définitif : tout s'ajoute ou se retire plus tard depuis la page Outils.",
            "Nichts ist endgültig: Alles lässt sich später auf der Seite Werkzeuge hinzufügen oder entfernen.",
          ),
        ]}
      />

      <Section
        title={tr(
          "1. Dove mettere il kit",
          "1. Where to put the kit",
          "1. Où placer le kit",
          "1. Wohin das Kit gehört",
        )}
      >
        <P>
          {tr(
            "Il kit sta dove sta il suo eseguibile: la cartella che lo contiene diventa la «cartella del kit» e lì nasceranno projects e data. Scegli quindi una cartella vuota e con un nome semplice, per esempio D:\\Atollo su una chiavetta o un disco esterno, oppure C:\\Atollo sul disco del PC.",
            "The kit lives where its executable is: the folder that contains it becomes the “kit folder”, and that is where projects and data will be created. So choose an empty folder with a simple name, for example D:\\Atollo on a USB stick or an external disk, or C:\\Atollo on the PC's disk.",
            "Le kit se trouve là où se trouve son exécutable : le dossier qui le contient devient le « dossier du kit » et c'est là que naîtront projects et data. Choisissez donc un dossier vide au nom simple, par exemple D:\\Atollo sur une clé ou un disque externe, ou C:\\Atollo sur le disque du PC.",
            "Das Kit befindet sich dort, wo seine Programmdatei liegt: Der Ordner, der sie enthält, wird zum „Ordner des Kits“, und dort entstehen projects und data. Wähle also einen leeren Ordner mit einem einfachen Namen, zum Beispiel D:\\Atollo auf einem USB-Stick oder einer externen Festplatte oder C:\\Atollo auf der Festplatte des PCs.",
          )}
        </P>
        <Bullets
          items={[
            tr(
              "Su Windows l'eseguibile si chiama Atollo.exe, oppure Atollo-<versione>-win32-x64.exe come lo scarichi dalla pagina delle release. Su macOS è Atollo.app, su Linux un file AppImage.",
              "On Windows the executable is called Atollo.exe, or Atollo-<version>-win32-x64.exe as you download it from the releases page. On macOS it is Atollo.app, on Linux an AppImage file.",
              "Sous Windows, l'exécutable s'appelle Atollo.exe, ou Atollo-<version>-win32-x64.exe tel que vous le téléchargez depuis la page des releases. Sous macOS, c'est Atollo.app, sous Linux un fichier AppImage.",
              "Unter Windows heißt die Programmdatei Atollo.exe oder Atollo-<Version>-win32-x64.exe, so wie du sie von der Release-Seite herunterlädst. Unter macOS ist es Atollo.app, unter Linux eine AppImage-Datei.",
            ),
            tr(
              "Spazio: la scelta proposta al primo avvio occupa circa 2,3 GB. Ogni linguaggio o versione in più aggiunge spazio, e i progetti con le loro dipendenze crescono nel tempo: su una chiavetta conviene avere almeno 16 GB liberi.",
              "Space: the choice suggested at the first start takes about 2.3 GB. Each extra language or version adds space, and projects with their dependencies grow over time: on a USB stick it is best to have at least 16 GB free.",
              "Espace : le choix proposé au premier lancement occupe environ 2,3 Go. Chaque langage ou version en plus ajoute de l'espace, et les projets avec leurs dépendances grossissent avec le temps : sur une clé, mieux vaut avoir au moins 16 Go libres.",
              "Speicherplatz: Die beim ersten Start vorgeschlagene Auswahl belegt etwa 2,3 GB. Jede weitere Sprache oder Version braucht zusätzlichen Platz, und Projekte mit ihren Abhängigkeiten wachsen mit der Zeit: Auf einem USB-Stick sollten mindestens 16 GB frei sein.",
            ),
            tr(
              "Velocità: un SSD esterno è molto più veloce di una chiavetta economica. Il kit funziona su entrambi, ma installare pacchetti con npm o Composer su una chiavetta lenta richiede pazienza.",
              "Speed: an external SSD is much faster than a cheap USB stick. The kit works on both, but installing packages with npm or Composer on a slow USB stick takes patience.",
              "Vitesse : un SSD externe est bien plus rapide qu'une clé bon marché. Le kit fonctionne sur les deux, mais installer des paquets avec npm ou Composer sur une clé lente demande de la patience.",
              "Geschwindigkeit: Eine externe SSD ist viel schneller als ein billiger USB-Stick. Das Kit funktioniert auf beiden, aber Pakete mit npm oder Composer auf einem langsamen USB-Stick zu installieren erfordert Geduld.",
            ),
          ]}
        />
        <Callout
          tone="warn"
          title={tr(
            "Due cose da evitare nel percorso",
            "Two things to avoid in the path",
            "Deux choses à éviter dans le chemin",
            "Zwei Dinge, die du im Pfad vermeiden solltest",
          )}
        >
          <Bullets
            items={[
              tr(
                "Su Windows, una cartella del percorso il cui nome comincia con una cifra, con la lettera x o con la lettera u (per esempio D:\\2024\\Atollo o D:\\utenti\\Atollo) impedisce all'editor di accorgersi dei file cambiati. Se succede, l'app lo segnala con un avviso: sposta il kit in un percorso diverso.",
                "On Windows, a folder in the path whose name starts with a digit, with the letter x or with the letter u (for example D:\\2024\\Atollo or D:\\utenti\\Atollo) stops the editor from noticing changed files. If that happens, the app warns you: move the kit to a different path.",
                "Sous Windows, un dossier du chemin dont le nom commence par un chiffre, par la lettre x ou par la lettre u (par exemple D:\\2024\\Atollo ou D:\\utenti\\Atollo) empêche l'éditeur de remarquer les fichiers modifiés. Si cela arrive, l'application l'indique par un avertissement : déplacez le kit vers un autre chemin.",
                "Unter Windows verhindert ein Ordner im Pfad, dessen Name mit einer Ziffer, mit dem Buchstaben x oder mit dem Buchstaben u beginnt (zum Beispiel D:\\2024\\Atollo oder D:\\utenti\\Atollo), dass der Editor geänderte Dateien bemerkt. Wenn das passiert, zeigt die App eine Warnung: Verschiebe das Kit in einen anderen Pfad.",
              ),
              tr(
                "Su macOS e Linux un disco montato senza permesso di esecuzione (noexec) ferma l'app: il kit deve poter avviare i suoi programmi.",
                "On macOS and Linux a disk mounted without execute permission (noexec) stops the app: the kit must be able to start its programs.",
                "Sous macOS et Linux, un disque monté sans permission d'exécution (noexec) bloque l'application : le kit doit pouvoir lancer ses programmes.",
                "Unter macOS und Linux stoppt eine ohne Ausführungsrecht eingebundene Festplatte (noexec) die App: Das Kit muss seine Programme starten können.",
              ),
            ]}
          />
        </Callout>
        <Under
          title={tr(
            "Quale file system scegliere",
            "Which file system to choose",
            "Quel système de fichiers choisir",
            "Welches Dateisystem du wählen solltest",
          )}
        >
          <P>
            {tr(
              "Su Windows il formato migliore è NTFS: è l'unico che permette al kit di proteggere le cartelle con le password (database, accessi) in modo che le legga solo il tuo utente, e di usare collegamenti al posto delle copie. Su exFAT e FAT32 il kit funziona, ma senza quella protezione; FAT32 inoltre non accetta file più grandi di 4 GB. Se usi la stessa chiavetta su Windows, macOS e Linux, exFAT è il formato che tutti leggono e scrivono: in quel caso tieni presente che i collegamenti diventano copie. Su exFAT e FAT32, però, la VM dei progetti non parte: i suoi dischi chiedono NTFS (o APFS ed ext4 su macOS e Linux).",
              "On Windows the best format is NTFS: it is the only one that lets the kit protect the folders holding passwords (databases, sign-ins) so that only your user can read them, and use links instead of copies. On exFAT and FAT32 the kit works, but without that protection; FAT32 also does not accept files larger than 4 GB. If you use the same USB stick on Windows, macOS and Linux, exFAT is the format all of them can read and write: in that case keep in mind that links become copies. On exFAT and FAT32, though, the projects' VM does not start: its disks need NTFS (or APFS and ext4 on macOS and Linux).",
              "Sous Windows, le meilleur format est NTFS : c'est le seul qui permet au kit de protéger les dossiers contenant des mots de passe (bases de données, connexions) pour que seul votre utilisateur puisse les lire, et d'utiliser des liens au lieu de copies. Sur exFAT et FAT32, le kit fonctionne, mais sans cette protection ; FAT32 n'accepte pas non plus les fichiers de plus de 4 Go. Si vous utilisez la même clé sous Windows, macOS et Linux, exFAT est le format que tous lisent et écrivent : dans ce cas, gardez à l'esprit que les liens deviennent des copies. Sur exFAT et FAT32, cependant, la VM des projets ne démarre pas : ses disques demandent NTFS (ou APFS et ext4 sur macOS et Linux).",
              "Unter Windows ist NTFS das beste Format: Es ist das einzige, mit dem das Kit die Ordner mit Passwörtern (Datenbanken, Anmeldungen) so schützen kann, dass nur dein Benutzer sie lesen kann, und Verknüpfungen statt Kopien verwenden kann. Auf exFAT und FAT32 funktioniert das Kit, aber ohne diesen Schutz; FAT32 akzeptiert außerdem keine Dateien über 4 GB. Wenn du denselben USB-Stick unter Windows, macOS und Linux verwendest, ist exFAT das Format, das alle lesen und schreiben können: Denk in diesem Fall daran, dass Verknüpfungen zu Kopien werden. Auf exFAT und FAT32 startet die VM der Projekte allerdings nicht: Ihre Festplatten brauchen NTFS (oder APFS und ext4 unter macOS und Linux).",
            )}
          </P>
          <P>
            {tr(
              "La pagina Impostazioni › «Il kit» mostra il disco su cui si trova il kit: file system e tipo (fisso, rimovibile, di rete).",
              "The Settings › “The kit” page shows the disk the kit is on: file system and type (fixed, removable, network).",
              "La page Paramètres › « Le kit » montre le disque sur lequel se trouve le kit : système de fichiers et type (fixe, amovible, réseau).",
              "Die Seite Einstellungen › „Das Kit“ zeigt die Festplatte, auf der sich das Kit befindet: Dateisystem und Typ (fest, Wechseldatenträger, Netzwerk).",
            )}
          </P>
        </Under>
      </Section>

      <Section
        title={tr(
          "2. La pagina «Benvenuto»",
          "2. The “Welcome” page",
          "2. La page « Bienvenue »",
          "2. Die Seite „Willkommen“",
        )}
      >
        <P>
          {tr(
            "Al primo avvio, al posto delle solite pagine, compare la scheda «Benvenuto», con il titolo «Benvenuto in Atollo». Spiega che agenti e linguaggi verranno scaricati dalle fonti ufficiali, verificati e messi nella cartella data del kit, senza installazione e senza diritti di amministratore.",
            "At the first start, instead of the usual pages, the “Welcome” tab appears, with the title “Welcome to Atollo”. It explains that agents and languages will be downloaded from the official sources, checked and placed in the kit's data folder, with no installation and no administrator rights.",
            "Au premier lancement, à la place des pages habituelles, l'onglet « Bienvenue » apparaît, avec le titre « Bienvenue dans Atollo ». Il explique que les agents et les langages seront téléchargés depuis les sources officielles, vérifiés et placés dans le dossier data du kit, sans installation et sans droits d'administrateur.",
            "Beim ersten Start erscheint statt der üblichen Seiten der Tab „Willkommen“ mit dem Titel „Willkommen bei Atollo“. Er erklärt, dass Agenten und Sprachen von den offiziellen Quellen heruntergeladen, geprüft und im Ordner data des Kits abgelegt werden, ohne Installation und ohne Administratorrechte.",
          )}
        </P>
        <P>
          {tr(
            "In cima ci sono due gruppi di impostazioni, salvate da sole mentre le cambi (come nella pagina Impostazioni, dove le ritrovi): «Prima di iniziare», con «Lingua», «Tema» e «Aggiornamento automatico», e «Rete», con proxy, certificati aggiuntivi e rete locale. Compila «Rete» solo se la tua rete passa da un proxy o ispeziona il traffico HTTPS: dato che niente si scarica prima di «Installa», anche i primi download li useranno.",
            "At the top there are two groups of settings, saved by themselves as you change them (as on the Settings page, where you will find them again): “Before you start”, with “Language”, “Theme” and “Automatic update”, and “Network”, with proxy, additional certificates and local network. Fill in “Network” only if your network goes through a proxy or inspects HTTPS traffic: since nothing is downloaded before “Install”, the very first downloads will use them too.",
            "En haut se trouvent deux groupes de paramètres, enregistrés tout seuls quand vous les changez (comme sur la page Paramètres, où vous les retrouvez) : « Avant de commencer », avec « Langue », « Thème » et « Mise à jour automatique », et « Réseau », avec le proxy, les certificats supplémentaires et le réseau local. Ne remplissez « Réseau » que si votre réseau passe par un proxy ou inspecte le trafic HTTPS : comme rien n'est téléchargé avant « Installer », même les premiers téléchargements les utiliseront.",
            "Oben stehen zwei Gruppen von Einstellungen, die sich beim Ändern von selbst speichern (wie auf der Seite Einstellungen, wo du sie wiederfindest): „Bevor du beginnst“ mit „Sprache“, „Design“ und „Automatisches Update“ und „Netzwerk“ mit Proxy, zusätzlichen Zertifikaten und lokalem Netzwerk. Fülle „Netzwerk“ nur aus, wenn dein Netzwerk über einen Proxy läuft oder den HTTPS-Verkehr untersucht: Da vor „Installieren“ nichts heruntergeladen wird, verwenden auch die ersten Downloads diese Werte.",
          )}
        </P>
        <P>
          {tr(
            "Sotto c'è l'elenco dei componenti, diviso in otto tipi (ognuno con la sua descrizione), con una casella di ricerca e un indice a sinistra. Ogni riga ha una casella da spuntare, il logo, il nome, una descrizione, la dimensione e delle etichette che dicono di che cosa ha bisogno o a che cosa serve.",
            "Below is the list of components, split into eight kinds (each with its description), with a search box and an index on the left. Each row has a checkbox, the logo, the name, a description, the size and some tags that say what it needs or what it is for.",
            "En dessous se trouve la liste des composants, divisée en huit types (chacun avec sa description), avec un champ de recherche et un index à gauche. Chaque ligne a une case à cocher, le logo, le nom, une description, la taille et des étiquettes qui disent de quoi il a besoin ou à quoi il sert.",
            "Darunter steht die Liste der Komponenten, in acht Arten aufgeteilt (jede mit ihrer Beschreibung), mit einem Suchfeld und einem Index links. Jede Zeile hat ein Kontrollkästchen, das Logo, den Namen, eine Beschreibung, die Größe und Etiketten, die sagen, was sie braucht oder wozu sie dient.",
          )}
        </P>
        <Table
          columns={[
            tr("Tipo", "Type", "Type", "Typ"),
            tr("Che cosa contiene", "What it contains", "Ce qu'il contient", "Was es enthält"),
            tr("Cosa scegliere", "What to choose", "Quoi choisir", "Was du wählen solltest"),
          ]}
          rows={[
            [
              tr("Sistema ed editor", "System and editor", "Système et éditeur", "System und Editor"),
              tr(
                "L'editor, PowerShell 7 (la console del kit), GitHub CLI, Docker Sandboxes (la VM dei progetti) e Git. Sotto, i requisiti del PC.",
                "The editor, PowerShell 7 (the kit's console), GitHub CLI, Docker Sandboxes (the projects' VM) and Git. Below, the PC requirements.",
                "L'éditeur, PowerShell 7 (la console du kit), GitHub CLI, Docker Sandboxes (la VM des projets) et Git. En dessous, les prérequis du PC.",
                "Der Editor, PowerShell 7 (die Konsole des Kits), GitHub CLI, Docker Sandboxes (die VM der Projekte) und Git. Darunter die Anforderungen an den PC.",
              ),
              tr(
                "L'editor, PowerShell 7, GitHub CLI e Docker Sandboxes sono sempre installati, insieme agli altri, quando premi «Installa»: le loro caselle sono bloccate. Docker Sandboxes solo dove Docker lo pubblica (non su Windows ARM64 né sui Mac Intel). Git si sceglie, e arriva comunque con Claude Code e Pi.",
                "The editor, PowerShell 7, GitHub CLI and Docker Sandboxes are always installed, together with the others, when you press “Install”: their checkboxes are locked. Docker Sandboxes only where Docker publishes it (not on Windows ARM64 nor on Intel Macs). Git is your choice, and it comes anyway with Claude Code and Pi.",
                "L'éditeur, PowerShell 7, GitHub CLI et Docker Sandboxes sont toujours installés, avec les autres, quand vous appuyez sur « Installer » : leurs cases sont verrouillées. Docker Sandboxes seulement là où Docker le publie (ni sur Windows ARM64 ni sur les Mac Intel). Git est à choisir, et il arrive de toute façon avec Claude Code et Pi.",
                "Der Editor, PowerShell 7, GitHub CLI und Docker Sandboxes werden immer installiert, zusammen mit den anderen, wenn du auf „Installieren“ klickst: Ihre Kontrollkästchen sind gesperrt. Docker Sandboxes nur dort, wo Docker es veröffentlicht (nicht unter Windows ARM64 und nicht auf Intel-Macs). Git wählst du selbst, und es kommt ohnehin mit Claude Code und Pi.",
              ),
            ],
            [
              tr("Agenti AI", "AI agents", "Agents IA", "KI-Agenten"),
              tr(
                "Claude Code, Codex, Antigravity e Pi.",
                "Claude Code, Codex, Antigravity and Pi.",
                "Claude Code, Codex, Antigravity et Pi.",
                "Claude Code, Codex, Antigravity und Pi.",
              ),
              tr(
                "Facoltativi. Tieni quelli di cui hai un account; servono un abbonamento o una chiave API del fornitore. Pi non è spuntato: spuntalo se vuoi un agente sui modelli locali, senza account.",
                "Optional. Keep the ones you have an account for; they require a subscription or an API key from the provider. Pi is not ticked: tick it if you want an agent on the local models, with no account.",
                "Facultatifs. Gardez ceux pour lesquels vous avez un compte ; il faut un abonnement ou une clé API du fournisseur. Pi n'est pas coché : cochez-le si vous voulez un agent sur les modèles locaux, sans compte.",
                "Optional. Behalte die, für die du ein Konto hast; sie erfordern ein Abonnement oder einen API-Schlüssel des Anbieters. Pi ist nicht angekreuzt: Kreuze ihn an, wenn du einen Agenten mit den lokalen Modellen willst, ohne Konto.",
              ),
            ],
            [
              tr("Modelli locali", "Local models", "Modèles locaux", "Lokale Modelle"),
              tr(
                "llama.cpp, il server dei modelli linguistici sul PC.",
                "llama.cpp, the server of the language models on the PC.",
                "llama.cpp, le serveur des modèles de langage sur le PC.",
                "llama.cpp, der Server der Sprachmodelle auf dem PC.",
              ),
              tr(
                "Facoltativo e non spuntato. Si installa anche da solo al primo avvio del server, dalla pagina AI, riquadro Modelli locali, dove si scaricano i modelli (molti GB).",
                "Optional and not ticked. It also installs itself the first time you start the server, from the AI page, Local models pane, where you download the models (many GB).",
                "Facultatif et non coché. Il s'installe aussi tout seul au premier démarrage du serveur, depuis la page IA, volet Modèles locaux, où se téléchargent les modèles (de nombreux Go).",
                "Optional und nicht angekreuzt. Er installiert sich auch von selbst beim ersten Start des Servers, auf der Seite KI im Bereich Lokale Modelle, wo die Modelle heruntergeladen werden (viele GB).",
              ),
            ],
            [
              tr("Linguaggi", "Languages", "Langages", "Sprachen"),
              tr(
                "Tutti i linguaggi e i runtime: Python, Node.js, PHP, Java, Go, .NET, Rust, Ruby, il compilatore C/C++ e gli altri. Python, Node.js, PHP, Go e Java possono avere più versioni; gli altri restano all'ultima.",
                "All languages and runtimes: Python, Node.js, PHP, Java, Go, .NET, Rust, Ruby, the C/C++ compiler and the others. Python, Node.js, PHP, Go and Java can have several versions; the others stay at the latest.",
                "Tous les langages et environnements d'exécution : Python, Node.js, PHP, Java, Go, .NET, Rust, Ruby, le compilateur C/C++ et les autres. Python, Node.js, PHP, Go et Java peuvent avoir plusieurs versions ; les autres restent à la dernière.",
                "Alle Sprachen und Laufzeiten: Python, Node.js, PHP, Java, Go, .NET, Rust, Ruby, der C/C++-Compiler und die anderen. Python, Node.js, PHP, Go und Java können mehrere Versionen haben; die anderen bleiben auf der neuesten.",
              ),
              tr(
                "Quelli che usi. Altri si aggiungono quando servono.",
                "The ones you use. Others can be added when needed.",
                "Ceux que vous utilisez. Les autres s'ajoutent quand il le faut.",
                "Die, die du verwendest. Weitere kommen hinzu, wenn sie gebraucht werden.",
              ),
            ],
            [
              tr("Strumenti di sviluppo", "Development tools", "Outils de développement", "Entwicklungswerkzeuge"),
              tr(
                "uv, pnpm e yarn, CMake e Ninja, Android SDK, Typst, TinyTeX: una sola versione, sempre l'ultima.",
                "uv, pnpm and yarn, CMake and Ninja, Android SDK, Typst, TinyTeX: a single version, always the latest.",
                "uv, pnpm et yarn, CMake et Ninja, Android SDK, Typst, TinyTeX : une seule version, toujours la dernière.",
                "uv, pnpm und yarn, CMake und Ninja, Android SDK, Typst, TinyTeX: eine einzige Version, immer die neueste.",
              ),
              tr(
                "Di solito niente: il kit li installa da solo quando un progetto o un agente li chiede.",
                "Usually nothing: the kit installs them by itself when a project or an agent asks for them.",
                "En général rien : le kit les installe tout seul quand un projet ou un agent les demande.",
                "Normalerweise nichts: Das Kit installiert sie selbst, wenn ein Projekt oder ein Agent sie anfordert.",
              ),
            ],
            [
              tr("Server web", "Web server", "Serveur web", "Webserver"),
              tr(
                "Apache e Caddy, per i siti dei progetti e i Server Web condivisi.",
                "Apache and Caddy, for the projects' sites and the shared Web Servers.",
                "Apache et Caddy, pour les sites des projets et les serveurs web partagés.",
                "Apache und Caddy, für die Websites der Projekte und die gemeinsamen Webserver.",
              ),
              tr(
                "Quello che usano i tuoi progetti; Apache è già nella scelta proposta.",
                "The one your projects use; Apache is already in the suggested choice.",
                "Celui qu'utilisent vos projets ; Apache est déjà dans le choix proposé.",
                "Den, den deine Projekte verwenden; Apache ist schon in der vorgeschlagenen Auswahl.",
              ),
            ],
            [
              tr("Database", "Databases", "Bases de données", "Datenbanken"),
              tr(
                "MySQL, MongoDB, SQLite; nel browser phpMyAdmin e mongo-express.",
                "MySQL, MongoDB, SQLite; in the browser phpMyAdmin and mongo-express.",
                "MySQL, MongoDB, SQLite ; dans le navigateur phpMyAdmin et mongo-express.",
                "MySQL, MongoDB, SQLite; im Browser phpMyAdmin und mongo-express.",
              ),
              tr(
                "Solo se ti servono. MySQL si installa anche al primo avvio del server.",
                "Only if you need them. MySQL is also installed the first time the server starts.",
                "Seulement si vous en avez besoin. MySQL s'installe aussi au premier démarrage du serveur.",
                "Nur wenn du sie brauchst. MySQL wird auch beim ersten Start des Servers installiert.",
              ),
            ],
          ]}
        />
        <P>
          {tr(
            "La scelta proposta comprende Claude Code, Codex e Antigravity (non Pi né llama.cpp), Python, PHP con Apache e i componenti di «Sistema ed editor», Git compreso. In tutto circa 2,3 GB. Quando spunti un componente, il kit spunta anche quelli di cui ha bisogno (per esempio Kotlin porta con sé Java, phpMyAdmin porta PHP e MySQL).",
            "The suggested choice includes Claude Code, Codex and Antigravity (not Pi or llama.cpp), Python, PHP with Apache and the “System and editor” components, Git included. About 2.3 GB in all. When you tick a component, the kit also ticks the ones it needs (for example Kotlin brings Java with it, phpMyAdmin brings PHP and MySQL).",
            "Le choix proposé comprend Claude Code, Codex et Antigravity (ni Pi ni llama.cpp), Python, PHP avec Apache et les composants de « Système et éditeur », Git compris. En tout, environ 2,3 Go. Quand vous cochez un composant, le kit coche aussi ceux dont il a besoin (par exemple Kotlin apporte Java, phpMyAdmin apporte PHP et MySQL).",
            "Die vorgeschlagene Auswahl umfasst Claude Code, Codex und Antigravity (weder Pi noch llama.cpp), Python, PHP mit Apache und die Komponenten von „System und Editor“, Git inklusive. Insgesamt etwa 2,3 GB. Wenn du eine Komponente ankreuzt, kreuzt das Kit auch die an, die sie braucht (zum Beispiel bringt Kotlin Java mit, phpMyAdmin bringt PHP und MySQL mit).",
          )}
        </P>
        <Steps
          steps={[
            tr(
              "In «Prima di iniziare» scegli la lingua e, se vuoi, il tema; se sei dietro un proxy aziendale, compila anche «Rete».",
              "In “Before you start” choose the language and, if you like, the theme; if you are behind a company proxy, also fill in “Network”.",
              "Dans « Avant de commencer », choisissez la langue et, si vous le souhaitez, le thème ; si vous êtes derrière un proxy d'entreprise, remplissez aussi « Réseau ».",
              "Wähle unter „Bevor du beginnst“ die Sprache und, wenn du willst, das Design; wenn du hinter einem Firmen-Proxy bist, fülle auch „Netzwerk“ aus.",
            ),
            tr(
              "Togli la spunta a ciò che non ti interessa e aggiungi ciò che usi.",
              "Untick what does not interest you and add what you use.",
              "Décochez ce qui ne vous intéresse pas et ajoutez ce que vous utilisez.",
              "Entferne das Häkchen bei allem, was dich nicht interessiert, und füge hinzu, was du verwendest.",
            ),
            tr(
              "Premi «Installa» in basso. Non c'è una conferma: la scelta viene salvata e lo scaricamento parte subito.",
              "Press “Install” at the bottom. There is no confirmation: the choice is saved and the download starts right away.",
              "Appuyez sur « Installer » en bas. Il n'y a pas de confirmation : le choix est enregistré et le téléchargement commence aussitôt.",
              "Klicke unten auf „Installieren“. Es gibt keine Bestätigung: Die Auswahl wird gespeichert und der Download beginnt sofort.",
            ),
            tr(
              "L'app lascia la pagina «Benvenuto» e mostra le pagine normali. Lo scaricamento continua in sottofondo: lo vedi nella barra di stato in basso e nel pannello «Attività».",
              "The app leaves the “Welcome” page and shows the normal pages. The download continues in the background: you see it in the status bar at the bottom and in the “Activity” panel.",
              "L'application quitte la page « Bienvenue » et affiche les pages normales. Le téléchargement continue en arrière-plan : vous le voyez dans la barre d'état en bas et dans le panneau « Activité ».",
              "Die App verlässt die Seite „Willkommen“ und zeigt die normalen Seiten. Der Download läuft im Hintergrund weiter: Du siehst ihn in der Statusleiste unten und im Bereich „Aktivität“.",
            ),
          ]}
        />
        <Callout
          tone="info"
          title={tr(
            "Poi, l'accesso a Docker",
            "Then, the Docker sign-in",
            "Ensuite, la connexion à Docker",
            "Dann die Anmeldung bei Docker",
          )}
        >
          {rich(
            tr(
              "Arrivato Docker Sandboxes, la barra di stato mostra «Senza VM» finché non accedi con un account Docker gratuito: fai clic sull'avviso, premi «Lancia» accanto a sbx login, accedi nel browser e poi «Ricontrolla». Se il PC non è pronto per la VM, la stessa finestra dice perché: {0}.",
              "Once Docker Sandboxes has arrived, the status bar shows “No VM” until you sign in with a free Docker account: click the notice, press “Run” next to sbx login, sign in in the browser and then “Check again”. If the PC is not ready for the VM, the same window tells why: {0}.",
              "Une fois Docker Sandboxes arrivé, la barre d'état affiche « Sans VM » jusqu'à ce que vous vous connectiez avec un compte Docker gratuit : cliquez sur l'avis, appuyez sur « Lancer » à côté de sbx login, connectez-vous dans le navigateur puis « Vérifier à nouveau ». Si le PC n'est pas prêt pour la VM, la même fenêtre dit pourquoi : {0}.",
              "Sobald Docker Sandboxes da ist, zeigt die Statusleiste „Ohne VM“, bis du dich mit einem kostenlosen Docker-Konto anmeldest: Klicke auf den Hinweis, drücke „Starten“ neben sbx login, melde dich im Browser an und dann „Erneut prüfen“. Ist der PC nicht bereit für die VM, sagt dasselbe Fenster, warum: {0}.",
            ),
            <See key="v" id="vm" />,
          )}
        </Callout>
        <Callout tone="tip">
          {tr(
            "Al primo avvio non si scelgono le versioni: di ogni linguaggio si installa la più recente. Le altre versioni si aggiungono dopo, dalla pagina Strumenti.",
            "At the first start you do not choose versions: for each language the latest one is installed. Other versions can be added later, from the Tools page.",
            "Au premier lancement, on ne choisit pas les versions : pour chaque langage, c'est la plus récente qui s'installe. Les autres versions s'ajoutent ensuite, depuis la page Outils.",
            "Beim ersten Start werden keine Versionen gewählt: Von jeder Sprache wird die neueste installiert. Weitere Versionen kommen später über die Seite Werkzeuge hinzu.",
          )}
        </Callout>
        <Callout
          tone="warn"
          title={tr("Fiducia nei progetti", "Trust in projects", "Confiance dans les projets", "Vertrauen in Projekte")}
        >
          {tr(
            "Quando apri un agente in un progetto da Atollo, la cartella del progetto diventa attendibile per Claude Code e Codex, che possono quindi eseguirvi comandi senza chiedere ogni volta. Apri un agente solo nei progetti di cui ti fidi.",
            "When you open an agent in a project from Atollo, the project's folder becomes trusted by Claude Code and Codex, which can then run commands in it without asking every time. Open an agent only in projects you trust.",
            "Quand vous ouvrez un agent dans un projet depuis Atollo, le dossier du projet devient fiable pour Claude Code et Codex, qui peuvent alors y exécuter des commandes sans demander à chaque fois. N'ouvrez un agent que dans les projets en lesquels vous avez confiance.",
            "Wenn du in einem Projekt einen Agenten aus Atollo öffnest, wird der Projektordner für Claude Code und Codex vertrauenswürdig, die darin dann Befehle ausführen können, ohne jedes Mal zu fragen. Öffne einen Agenten nur in Projekten, denen du vertraust.",
          )}
        </Callout>
      </Section>

      <Section
        title={tr(
          "3. Che cosa succede mentre scarica",
          "3. What happens while it downloads",
          "3. Ce qui se passe pendant le téléchargement",
          "3. Was beim Herunterladen passiert",
        )}
      >
        <Flow
          steps={[
            {
              title: tr("Scarica", "Download", "Télécharge", "Herunterladen"),
              text: tr(
                "Dalla fonte ufficiale del produttore, in HTTPS.",
                "From the manufacturer's official source, over HTTPS.",
                "Depuis la source officielle de l'éditeur, en HTTPS.",
                "Von der offiziellen Quelle des Herstellers, über HTTPS.",
              ),
            },
            {
              title: tr("Verifica", "Check", "Vérifie", "Prüfen"),
              text: tr(
                "Confronta l'impronta del file con quella pubblicata; se non coincide lo scarta.",
                "Compares the file's fingerprint with the published one; if they do not match, it discards it.",
                "Compare l'empreinte du fichier avec celle publiée ; si elles ne correspondent pas, il l'écarte.",
                "Vergleicht den Fingerabdruck der Datei mit dem veröffentlichten; stimmt er nicht überein, wird sie verworfen.",
              ),
            },
            {
              title: tr("Estrae", "Extract", "Extrait", "Entpacken"),
              text: tr(
                "In una cartella di lavoro temporanea.",
                "Into a temporary working folder.",
                "Dans un dossier de travail temporaire.",
                "In einen temporären Arbeitsordner.",
              ),
            },
            {
              title: tr("Sistema", "Set up", "Installe", "Einrichten"),
              text: tr(
                "Mette il programma in data\\apps e prova che risponda.",
                "Puts the program in data\\apps and checks that it responds.",
                "Met le programme dans data\\apps et vérifie qu'il répond.",
                "Legt das Programm in data\\apps ab und prüft, ob es antwortet.",
              ),
            },
          ]}
        />
        <P>
          {tr(
            "Ogni componente passa da questi quattro passi. Se un download non riesce (rete assente, proxy aziendale, sito del produttore irraggiungibile) il kit riprova tre volte; se ancora non va, la riga mostra l'errore e, quando possibile, un riquadro per scaricare il file con il browser e consegnarlo al kit.",
            "Every component goes through these four steps. If a download fails (no network, company proxy, manufacturer's site unreachable) the kit tries three times; if it still does not work, the row shows the error and, when possible, a box to download the file with the browser and hand it to the kit.",
            "Chaque composant passe par ces quatre étapes. Si un téléchargement échoue (pas de réseau, proxy d'entreprise, site de l'éditeur injoignable), le kit réessaie trois fois ; si cela ne marche toujours pas, la ligne affiche l'erreur et, quand c'est possible, un encadré pour télécharger le fichier avec le navigateur et le remettre au kit.",
            "Jede Komponente durchläuft diese vier Schritte. Wenn ein Download fehlschlägt (kein Netzwerk, Firmen-Proxy, Website des Herstellers nicht erreichbar), versucht es das Kit dreimal; klappt es dann immer noch nicht, zeigt die Zeile den Fehler und, wenn möglich, einen Kasten, um die Datei mit dem Browser herunterzuladen und dem Kit zu übergeben.",
          )}
        </P>
        <P>
          {rich(
            tr(
              "I dettagli su fonti e verifiche sono in {0}; se sei dietro un proxy aziendale leggi prima {1}.",
              "The details on sources and checks are in {0}; if you are behind a company proxy, read {1} first.",
              "Les détails sur les sources et les vérifications sont dans {0} ; si vous êtes derrière un proxy d'entreprise, lisez d'abord {1}.",
              "Die Details zu Quellen und Prüfungen stehen in {0}; wenn du hinter einem Firmen-Proxy bist, lies zuerst {1}.",
            ),
            <See key="s" id="sources" />,
            <See key="n" id="network" />,
          )}
        </P>
        <Under>
          <Bullets
            items={[
              tr(
                "La scelta dei componenti viene salvata in data\\app\\components.txt, una riga per componente (id=1 scelto, id=0 no).",
                "The choice of components is saved in data\\app\\components.txt, one line per component (id=1 chosen, id=0 not).",
                "Le choix des composants est enregistré dans data\\app\\components.txt, une ligne par composant (id=1 choisi, id=0 non).",
                "Die Auswahl der Komponenten wird in data\\app\\components.txt gespeichert, eine Zeile pro Komponente (id=1 gewählt, id=0 nicht).",
              ),
              tr(
                "Al primo avvio non si scarica niente, nemmeno i componenti obbligatori, finché non premi «Installa»: così proxy e certificati scelti nella pagina valgono già per i primi download. Dagli avvii successivi il kit controlla che ci siano i componenti obbligatori e li reinstalla da solo se mancano, senza chiedere.",
                "At the first start nothing is downloaded, not even the required components, until you press “Install”: this way the proxy and certificates chosen on the page already apply to the first downloads. From the following starts the kit checks that the required components are there and reinstalls them by itself if they are missing, without asking.",
                "Au premier lancement, rien n'est téléchargé, pas même les composants obligatoires, tant que vous n'appuyez pas sur « Installer » : ainsi le proxy et les certificats choisis sur la page s'appliquent déjà aux premiers téléchargements. Aux lancements suivants, le kit vérifie que les composants obligatoires sont présents et les réinstalle tout seul s'ils manquent, sans demander.",
                "Beim ersten Start wird nichts heruntergeladen, nicht einmal die erforderlichen Komponenten, bis du auf „Installieren“ klickst: So gelten Proxy und Zertifikate, die du auf der Seite gewählt hast, schon für die ersten Downloads. Bei den folgenden Starts prüft das Kit, ob die erforderlichen Komponenten vorhanden sind, und installiert sie selbst neu, wenn sie fehlen, ohne zu fragen.",
              ),
              tr(
                "Se il primo avvio viene interrotto, alla riapertura la pagina «Benvenuto» non ricompare se la scelta è già stata salvata: si riprende da Strumenti, dove le righe non scaricate mostrano lo stato «non esiste» o «errore».",
                "If the first start is interrupted, the “Welcome” page does not come back on reopening if the choice has already been saved: you pick up from Tools, where the rows not downloaded show the state “not present” or “error”.",
                "Si le premier lancement est interrompu, la page « Bienvenue » ne réapparaît pas à la réouverture si le choix a déjà été enregistré : on reprend depuis Outils, où les lignes non téléchargées affichent l'état « absent » ou « erreur ».",
                "Wird der erste Start unterbrochen, erscheint die Seite „Willkommen“ beim erneuten Öffnen nicht wieder, wenn die Auswahl bereits gespeichert ist: Du machst unter Werkzeuge weiter, wo nicht heruntergeladene Zeilen den Zustand „nicht vorhanden“ oder „Fehler“ zeigen.",
              ),
            ]}
          />
        </Under>
      </Section>

      <Related ids={["window", "first-project", "vm", "tools", "network"]} />
    </>
  ),

  window: () => (
    <>
      <InBrief
        points={[
          tr(
            "La finestra imita quella di Visual Studio Code: chi lo conosce si orienta subito.",
            "The window looks like Visual Studio Code's: if you know it, you will find your way right away.",
            "La fenêtre imite celle de Visual Studio Code : si vous le connaissez, vous vous y retrouvez tout de suite.",
            "Das Fenster ähnelt dem von Visual Studio Code: Wer es kennt, findet sich sofort zurecht.",
          ),
          tr(
            "A sinistra la barra delle attività cambia pagina; accanto, la barra laterale mostra elenchi e alberi.",
            "On the left the activity bar changes page; next to it, the sidebar shows lists and trees.",
            "À gauche, la barre d'activité change de page ; à côté, la barre latérale affiche des listes et des arborescences.",
            "Links wechselt die Aktivitätsleiste die Seite; daneben zeigt die Seitenleiste Listen und Bäume.",
          ),
          tr(
            "Al centro le schede; in basso la barra di stato e il pannello «Attività» con ciò che è in corso.",
            "In the center the tabs; at the bottom the status bar and the “Activity” panel with what is in progress.",
            "Au centre, les onglets ; en bas, la barre d'état et le panneau « Activité » avec ce qui est en cours.",
            "In der Mitte die Tabs; unten die Statusleiste und der Bereich „Aktivität“ mit dem, was gerade läuft.",
          ),
          rich(
            tr(
              "{0} apre un progetto per nome da qualsiasi pagina.",
              "{0} opens a project by name from any page.",
              "{0} ouvre un projet par son nom depuis n'importe quelle page.",
              "{0} öffnet ein Projekt über seinen Namen von jeder Seite aus.",
            ),
            <Key key="k">Ctrl+P</Key>,
          ),
          rich(
            tr(
              "{0} apre la palette dei comandi da qualsiasi pagina, anche da un terminale: avviare o fermare un progetto, aprirlo, lanciarci un agente, cambiare pagina.",
              "{0} opens the command palette from any page, even from a terminal: start or stop a project, open it, launch an agent in it, change page.",
              "{0} ouvre la palette de commandes depuis n'importe quelle page, même depuis un terminal : démarrer ou arrêter un projet, l'ouvrir, y lancer un agent, changer de page.",
              "{0} öffnet die Befehlspalette von jeder Seite aus, auch aus einem Terminal: ein Projekt starten oder stoppen, es öffnen, einen Agenten darin starten, die Seite wechseln.",
            ),
            <Key key="c">Ctrl+Shift+P</Key>,
          ),
        ]}
      />

      <Section
        title={tr(
          "Le parti della finestra",
          "The parts of the window",
          "Les parties de la fenêtre",
          "Die Teile des Fensters",
        )}
      >
        <WindowSketch
          title={tr("1 · Barra del titolo", "1 · Title bar", "1 · Barre de titre", "1 · Titelleiste")}
          activity={tr("2 · Barra delle attività", "2 · Activity bar", "2 · Barre d'activité", "2 · Aktivitätsleiste")}
          sidebar={tr("3 · Barra laterale", "3 · Sidebar", "3 · Barre latérale", "3 · Seitenleiste")}
          tabs={tr("4 · Area delle schede", "4 · Tab area", "4 · Zone des onglets", "4 · Tab-Bereich")}
          panel={tr(
            "5 · Pannello «Attività»",
            "5 · “Activity” panel",
            "5 · Panneau « Activité »",
            "5 · Bereich „Aktivität“",
          )}
          status={tr("6 · Barra di stato", "6 · Status bar", "6 · Barre d'état", "6 · Statusleiste")}
        />
        <Table
          columns={[
            tr("Parte", "Part", "Partie", "Teil"),
            tr("Che cosa contiene", "What it contains", "Ce qu'il contient", "Was es enthält"),
          ]}
          rows={[
            [
              tr("1 · Barra del titolo", "1 · Title bar", "1 · Barre de titre", "1 · Titelleiste"),
              tr(
                "Il nome Atollo e la pagina aperta; a destra riduci, ingrandisci e chiudi. Trascinala per spostare la finestra, doppio clic per ingrandirla.",
                "The name Atollo and the open page; on the right minimize, maximize and close. Drag it to move the window, double-click to maximize it.",
                "Le nom Atollo et la page ouverte ; à droite, réduire, agrandir et fermer. Faites-la glisser pour déplacer la fenêtre, double-cliquez pour l'agrandir.",
                "Der Name Atollo und die geöffnete Seite; rechts Minimieren, Maximieren und Schließen. Zieh sie, um das Fenster zu verschieben, ein Doppelklick maximiert es.",
              ),
            ],
            [
              tr("2 · Barra delle attività", "2 · Activity bar", "2 · Barre d'activité", "2 · Aktivitätsleiste"),
              tr(
                "Le icone delle pagine. In alto quelle di lavoro: Progetti, AI (agenti e modelli locali), Database, Server Web, Terminali, Strumenti. In basso: Guida, Impostazioni, Rimuovi l'unità. Un numero sull'icona di Progetti dice quante cose sono in esecuzione: azioni dei progetti, versioni dei database, pool dei server web e il server dei modelli locali.",
                "The page icons. At the top the working ones: Projects, AI (agents and local models), Databases, Web Servers, Terminals, Tools. At the bottom: Guide, Settings, Remove drive. A number on the Projects icon tells how many things are running: project actions, database versions, web server pools and the local models server.",
                "Les icônes des pages. En haut, celles de travail : Projets, IA (agents et modèles locaux), Bases de données, Serveurs web, Terminaux, Outils. En bas : Guide, Paramètres, Retirer le lecteur. Un nombre sur l'icône Projets indique combien de choses sont en cours d'exécution : actions de projet, versions des bases de données, pools de serveurs web et le serveur des modèles locaux.",
                "Die Symbole der Seiten. Oben die für die Arbeit: Projekte, KI (Agenten und lokale Modelle), Datenbanken, Webserver, Terminals, Werkzeuge. Unten: Anleitung, Einstellungen, Laufwerk entfernen. Eine Zahl auf dem Symbol von Projekte zeigt, wie viele Dinge laufen: Projektaktionen, Datenbankversionen, Webserver-Pools und der Server der lokalen Modelle.",
              ),
            ],
            [
              tr("3 · Barra laterale", "3 · Sidebar", "3 · Barre latérale", "3 · Seitenleiste"),
              tr(
                "Il titolo della pagina con le sue icone (tra cui il libro «Apri la guida di questa pagina»), spesso una casella di ricerca, e dei riquadri che si aprono e si chiudono con un clic sul titolo.",
                "The page title with its icons (including the book “Open the guide for this page”), often a search box, and panes that open and close with a click on their title.",
                "Le titre de la page avec ses icônes (dont le livre « Ouvrir le guide de cette page »), souvent un champ de recherche, et des volets qui s'ouvrent et se ferment d'un clic sur leur titre.",
                "Der Seitentitel mit seinen Symbolen (darunter das Buch „Anleitung für diese Seite öffnen“), oft ein Suchfeld, und Bereiche, die sich mit einem Klick auf ihren Titel öffnen und schließen.",
              ),
            ],
            [
              tr("4 · Area delle schede", "4 · Tab area", "4 · Zone des onglets", "4 · Tab-Bereich"),
              tr(
                "Ciò che hai scelto nella barra laterale si apre in una scheda. La prima scheda di alcune pagine è fissa: il «Pannello rapido» in Progetti, «Tutti gli strumenti» in Strumenti.",
                "What you choose in the sidebar opens in a tab. The first tab of some pages is fixed: the “Quick panel” in Projects, “All tools” in Tools.",
                "Ce que vous choisissez dans la barre latérale s'ouvre dans un onglet. Le premier onglet de certaines pages est fixe : le « Panneau rapide » dans Projets, « Tous les outils » dans Outils.",
                "Was du in der Seitenleiste auswählst, öffnet sich in einem Tab. Der erste Tab mancher Seiten ist fest: die „Schnellübersicht“ in Projekte, „Alle Werkzeuge“ in Werkzeuge.",
              ),
            ],
            [
              tr(
                "5 · Pannello «Attività»",
                "5 · “Activity” panel",
                "5 · Panneau « Activité »",
                "5 · Bereich „Aktivität“",
              ),
              tr(
                "Si apre dalla barra di stato. Elenca l'installazione in corso, le versioni dei database accese, i pool dei server web accesi, il server dei modelli locali e le azioni dei progetti, ognuna con il suo output; «Ferma tutto» le ferma tutte dopo una conferma.",
                "It opens from the status bar. It lists the installation in progress, the running database versions, the running web server pools, the local models server and the project actions, each with its output; “Stop everything” stops them all after a confirmation.",
                "Il s'ouvre depuis la barre d'état. Il liste l'installation en cours, les versions des bases de données allumées, les pools de serveurs web allumés, le serveur des modèles locaux et les actions des projets, chacune avec sa sortie ; « Tout arrêter » les arrête toutes après une confirmation.",
                "Er öffnet sich über die Statusleiste. Er listet die laufende Installation, die eingeschalteten Datenbankversionen, die eingeschalteten Webserver-Pools, den Server der lokalen Modelle und die Aktionen der Projekte auf, jede mit ihrer Ausgabe; „Alles stoppen“ beendet sie alle nach einer Bestätigung.",
              ),
            ],
            [
              tr("6 · Barra di stato", "6 · Status bar", "6 · Barre d'état", "6 · Statusleiste"),
              tr(
                "A sinistra la voce Atollo e il contatore di ciò che è in esecuzione e degli errori, che mostra o nasconde il pannello «Attività», e accanto «VM» o «Senza VM»; a destra i terminali aperti, lo stato dell'editor, il sistema (per esempio win32-x64) e, per ultima, la versione di Atollo: un clic cerca gli aggiornamenti, e accanto compare l'esito (un segno di spunta, la versione nuova o un avviso).",
                "On the left the Atollo item and the counter of what is running and of the errors, which shows or hides the “Activity” panel, and beside it “VM” or “No VM”; on the right the open terminals, the editor status, the system (for example win32-x64) and, last, the Atollo version: a click checks for updates, and the result appears beside it (a check mark, the new version or a warning).",
                "À gauche, l'élément Atollo et le compteur de ce qui est en cours d'exécution et des erreurs, qui affiche ou masque le panneau « Activité », et à côté « VM » ou « Sans VM » ; à droite, les terminaux ouverts, l'état de l'éditeur, le système (par exemple win32-x64) et, en dernier, la version d'Atollo : un clic recherche les mises à jour, et le résultat s'affiche à côté (une coche, la nouvelle version ou un avertissement).",
                "Links das Element Atollo und der Zähler dessen, was läuft, und der Fehler, der den Bereich „Aktivität“ ein- oder ausblendet, und daneben „VM“ oder „Ohne VM“; rechts die geöffneten Terminals, der Status des Editors, das System (zum Beispiel win32-x64) und als Letztes die Atollo-Version: Ein Klick sucht nach Updates, daneben erscheint das Ergebnis (ein Häkchen, die neue Version oder eine Warnung).",
              ),
            ],
          ]}
        />
      </Section>

      <Section title={tr("Le pagine", "The pages", "Les pages", "Die Seiten")}>
        <PageMap />
      </Section>

      <Section
        title={tr(
          "La barra di stato e il pannello «Attività»",
          "The status bar and the “Activity” panel",
          "La barre d'état et le panneau « Activité »",
          "Die Statusleiste und der Bereich „Aktivität“",
        )}
      >
        <Table
          columns={[
            tr("Elemento", "Item", "Élément", "Element"),
            tr("Che cosa dice", "What it says", "Ce qu'il indique", "Was es anzeigt"),
            tr("Che cosa fa al clic", "What a click does", "Ce que fait un clic", "Was ein Klick bewirkt"),
          ]}
          rows={[
            [
              tr("Atollo (a sinistra)", "Atollo (on the left)", "Atollo (à gauche)", "Atollo (links)"),
              tr(
                "Passandoci sopra mostra la cartella del kit.",
                "Hovering over it shows the kit folder.",
                "Au survol, il montre le dossier du kit.",
                "Beim Darüberfahren zeigt es den Ordner des Kits.",
              ),
              tr("—", "—", "—", "—"),
            ],
            [
              tr("▶ numero · ⨯ numero", "▶ number · ⨯ number", "▶ nombre · ⨯ nombre", "▶ Zahl · ⨯ Zahl"),
              tr(
                "Quante cose sono in esecuzione (azioni dei progetti, ogni versione dei database accesa, anche più d'una dello stesso database, i pool dei server web accesi e il server dei modelli locali) e quante azioni si sono fermate con un errore. Diventa rosso se c'è almeno un errore.",
                "How many things are running (project actions, every running database version, even several of the same database, the running web server pools and the local models server) and how many actions stopped with an error. It turns red if there is at least one error.",
                "Combien de choses sont en cours d'exécution (actions de projet, chaque version de base de données allumée, même plusieurs de la même base, les pools de serveurs web allumés et le serveur des modèles locaux) et combien d'actions se sont arrêtées sur une erreur. Il devient rouge s'il y a au moins une erreur.",
                "Wie viele Dinge laufen (Projektaktionen, jede eingeschaltete Datenbankversion, auch mehrere derselben Datenbank, die eingeschalteten Webserver-Pools und der Server der lokalen Modelle) und wie viele Aktionen mit einem Fehler beendet wurden. Es wird rot, wenn es mindestens einen Fehler gibt.",
              ),
              tr(
                "Apre o chiude il pannello «Attività».",
                "Opens or closes the “Activity” panel.",
                "Ouvre ou ferme le panneau « Activité ».",
                "Öffnet oder schließt den Bereich „Aktivität“.",
              ),
            ],
            [
              tr("VM · Senza VM", "VM · No VM", "VM · Sans VM", "VM · Ohne VM"),
              tr(
                "«VM» quando ogni progetto gira nella sua microVM; «Senza VM», sul colore di avviso, quando i progetti girano sul PC.",
                "“VM” when each project runs in its microVM; “No VM”, on the warning color, when the projects run on the PC.",
                "« VM » quand chaque projet tourne dans sa microVM ; « Sans VM », sur la couleur d'avertissement, quand les projets tournent sur le PC.",
                "„VM“, wenn jedes Projekt in seiner microVM läuft; „Ohne VM“, in der Warnfarbe, wenn die Projekte auf dem PC laufen.",
              ),
              tr(
                "Apre la finestra della VM: perché non è attiva e che cosa fare, «Ricontrolla» e le sandbox.",
                "Opens the VM's window: why it is not on and what to do, “Check again” and the sandboxes.",
                "Ouvre la fenêtre de la VM : pourquoi elle n'est pas active et quoi faire, « Vérifier à nouveau » et les sandbox.",
                "Öffnet das Fenster der VM: warum sie nicht aktiv ist und was zu tun ist, „Erneut prüfen“ und die Sandboxes.",
              ),
            ],
            [
              tr("Rotellina con testo", "Spinner with text", "Roue avec texte", "Drehsymbol mit Text"),
              tr(
                "Un'installazione di componenti in corso e il passo che sta facendo.",
                "A component installation in progress and the step it is on.",
                "Une installation de composants en cours et l'étape en train d'être faite.",
                "Eine laufende Installation von Komponenten und der Schritt, den sie gerade ausführt.",
              ),
              tr("—", "—", "—", "—"),
            ],
            [
              tr("Terminali aperti", "Open terminals", "Terminaux ouverts", "Geöffnete Terminals"),
              tr(
                "Quante sessioni ha il pannello dei terminali dell'app.",
                "How many sessions the app's terminal panel has.",
                "Combien de sessions compte le panneau des terminaux de l'application.",
                "Wie viele Sitzungen der Terminalbereich der App hat.",
              ),
              tr(
                "Mostra o nasconde i terminali.",
                "Shows or hides the terminals.",
                "Affiche ou masque les terminaux.",
                "Zeigt oder verbirgt die Terminals.",
              ),
            ],
            [
              tr("Editor attivo", "Editor running", "Éditeur actif", "Editor aktiv"),
              tr(
                "Il server dell'editor è acceso (passandoci sopra, il suo indirizzo).",
                "The editor server is on (hovering over it shows its address).",
                "Le serveur de l'éditeur est allumé (au survol, son adresse).",
                "Der Server des Editors läuft (beim Darüberfahren seine Adresse).",
              ),
              tr("—", "—", "—", "—"),
            ],
            [
              tr(
                "win32-x64, darwin-arm64…",
                "win32-x64, darwin-arm64…",
                "win32-x64, darwin-arm64…",
                "win32-x64, darwin-arm64…",
              ),
              tr(
                "Il sistema e il processore su cui gira il kit.",
                "The system and the processor the kit runs on.",
                "Le système et le processeur sur lesquels tourne le kit.",
                "Das System und der Prozessor, auf denen das Kit läuft.",
              ),
              tr("—", "—", "—", "—"),
            ],
            [
              tr("v0.x.y", "v0.x.y", "v0.x.y", "v0.x.y"),
              tr("La versione di Atollo.", "The Atollo version.", "La version d'Atollo.", "Die Atollo-Version."),
              tr("—", "—", "—", "—"),
            ],
          ]}
        />
        <P>
          {tr(
            "Nel pannello «Attività» ogni voce si seleziona per vederne i dettagli: per un'azione di progetto lo stato, l'indirizzo e l'output in diretta, con «Apri nel browser», «Ferma» o, se è finita, «Togli»; per un database il nome, l'indirizzo, eventuali errori, il file di log e «Ferma»; per un pool dei server web l'indirizzo, i VirtualHost collegati e «Ferma»; per i modelli locali l'indirizzo, i modelli in memoria e «Ferma». «Ferma tutto», in alto, chiede «Fermare le azioni dei progetti, i database, i modelli locali e i server web in esecuzione?» e poi ferma azioni, database, modelli locali e pool. Se non c'è niente compare «niente in corso». Il pannello si ridimensiona trascinandone il bordo; un doppio clic sul bordo lo riporta all'altezza iniziale.",
            "In the “Activity” panel you select each item to see its details: for a project action the status, the address and the live output, with “Open in the browser”, “Stop” or, if it has finished, “Remove”; for a database the name, the address, any errors, the log file and “Stop”; for a web server pool the address, the attached VirtualHosts and “Stop”; for the local models the address, the models in memory and “Stop”. “Stop everything”, at the top, asks “Stop the projects' actions, the databases, the local models and the web servers that are running?” and then stops actions, databases, local models and pools. If there is nothing, “nothing running” appears. You resize the panel by dragging its edge; a double-click on the edge brings it back to its initial height.",
            "Dans le panneau « Activité », chaque entrée se sélectionne pour en voir les détails : pour une action de projet, l'état, l'adresse et la sortie en direct, avec « Ouvrir dans le navigateur », « Arrêter » ou, si elle est terminée, « Retirer » ; pour une base de données, le nom, l'adresse, les éventuelles erreurs, le fichier de log et « Arrêter » ; pour un pool de serveurs web, l'adresse, les VirtualHosts rattachés et « Arrêter » ; pour les modèles locaux, l'adresse, les modèles en mémoire et « Arrêter ». « Tout arrêter », en haut, demande « Arrêter les actions des projets, les bases de données, les modèles locaux et les serveurs web en cours ? » puis arrête actions, bases de données, modèles locaux et pools. S'il n'y a rien, « rien en cours » s'affiche. Le panneau se redimensionne en faisant glisser son bord ; un double-clic sur le bord le ramène à sa hauteur initiale.",
            "Im Bereich „Aktivität“ wählst du jeden Eintrag aus, um seine Details zu sehen: bei einer Projektaktion den Status, die Adresse und die Live-Ausgabe, mit „Im Browser öffnen“, „Beenden“ oder, wenn sie fertig ist, „Entfernen“; bei einer Datenbank den Namen, die Adresse, eventuelle Fehler, die Logdatei und „Beenden“; bei einem Webserver-Pool die Adresse, die verbundenen VirtualHosts und „Beenden“; bei den lokalen Modellen die Adresse, die Modelle im Speicher und „Beenden“. „Alles stoppen“ oben fragt „Die laufenden Aktionen der Projekte, Datenbanken, lokalen Modelle und Webserver beenden?“ und beendet dann Aktionen, Datenbanken, lokale Modelle und Pools. Wenn nichts da ist, erscheint „nichts läuft“. Die Größe des Bereichs änderst du, indem du seinen Rand ziehst; ein Doppelklick auf den Rand stellt die ursprüngliche Höhe wieder her.",
          )}
        </P>
      </Section>

      <Section
        title={tr(
          "Scorciatoie e abitudini utili",
          "Useful shortcuts and habits",
          "Raccourcis et habitudes utiles",
          "Nützliche Tastenkürzel und Gewohnheiten",
        )}
      >
        <Table
          columns={[
            tr("Tasti", "Keys", "Touches", "Tasten"),
            tr("Che cosa fa", "What it does", "Ce qu'il fait", "Was es bewirkt"),
          ]}
          rows={[
            [
              <Key key="k">Ctrl+P</Key>,
              tr(
                "Apre «Apri un progetto» (⌘P su macOS): scrivi parte del nome, della piattaforma o del linguaggio. Invio apre la pagina del progetto, Maiusc+Invio avvia la sua azione predefinita, Esc chiude.",
                "Opens “Open a project” (⌘P on macOS): type part of the name, the platform or the language. Enter opens the project's page, Shift+Enter starts its default action, Esc closes.",
                "Ouvre « Ouvrir un projet » (⌘P sur macOS) : tapez une partie du nom, de la plateforme ou du langage. Entrée ouvre la page du projet, Maj+Entrée démarre son action par défaut, Échap ferme.",
                "Öffnet „Projekt öffnen“ (⌘P unter macOS): Gib einen Teil des Namens, der Plattform oder der Sprache ein. Eingabe öffnet die Seite des Projekts, Umschalt+Eingabe startet seine Standardaktion, Esc schließt.",
              ),
            ],
            [
              <Key key="c">Ctrl+Shift+P</Key>,
              tr(
                "Apre «Comandi», la palette dei comandi (⌘⇧P su macOS), da qualsiasi pagina e anche da un terminale. Offre «Avvia» o «Ferma» e «Apri» per ogni progetto, «Apri … nell'editor», ogni agente installato in un progetto, «Avvia» o «Ferma» per MySQL e MongoDB, «Vai a» per le altre pagine e, quando qualcosa è in corso, «Ferma tutto». Ogni parola scritta deve comparire, in qualsiasi ordine; ↑ e ↓ scelgono, Invio esegue, Esc chiude.",
                "Opens “Commands”, the command palette (⌘⇧P on macOS), from any page and also from a terminal. It offers “Start” or “Stop” and “Open” for each project, “Open … in the editor”, each installed agent in a project, “Start” or “Stop” for MySQL and MongoDB, “Go to” for the other pages and, when something is running, “Stop everything”. Every word you type must appear, in any order; ↑ and ↓ choose, Enter runs, Esc closes.",
                "Ouvre « Commandes », la palette de commandes (⌘⇧P sur macOS), depuis n'importe quelle page et aussi depuis un terminal. Elle propose « Démarrer » ou « Arrêter » et « Ouvrir » pour chaque projet, « Ouvrir … dans l'éditeur », chaque agent installé dans un projet, « Démarrer » ou « Arrêter » pour MySQL et MongoDB, « Aller à » pour les autres pages et, quand quelque chose tourne, « Tout arrêter ». Chaque mot tapé doit apparaître, dans n'importe quel ordre ; ↑ et ↓ choisissent, Entrée exécute, Échap ferme.",
                "Öffnet „Befehle“, die Befehlspalette (⌘⇧P unter macOS), von jeder Seite und auch aus einem Terminal. Sie bietet „Starten“ oder „Stoppen“ und „Öffnen“ für jedes Projekt, „… im Editor öffnen“, jeden installierten Agenten in einem Projekt, „Starten“ oder „Stoppen“ für MySQL und MongoDB, „Zu … wechseln“ für die anderen Seiten und, wenn etwas läuft, „Alles stoppen“. Jedes eingegebene Wort muss vorkommen, in beliebiger Reihenfolge; ↑ und ↓ wählen, Eingabe führt aus, Esc schließt.",
              ),
            ],
            [
              <Key key="e">Esc</Key>,
              tr(
                "Chiude le finestre di conferma e lo zoom delle immagini, e annulla le domande nella pagina (per esempio quella di «Ferma tutto» o della rimozione di un terminale).",
                "Closes confirmation dialogs and image zoom, and cancels the questions in the page (for example the one of “Stop everything” or of removing a terminal).",
                "Ferme les fenêtres de confirmation et le zoom des images, et annule les questions dans la page (par exemple celle de « Tout arrêter » ou de la suppression d'un terminal).",
                "Schließt Bestätigungsdialoge und die Bildvergrößerung und bricht die Fragen in der Seite ab (zum Beispiel die von „Alles stoppen“ oder beim Entfernen eines Terminals).",
              ),
            ],
          ]}
        />
        <Bullets
          items={[
            tr(
              "Passa il mouse sopra icone, pallini e etichette: quasi tutto ha un suggerimento che spiega che cosa è o che cosa fa.",
              "Hover the mouse over icons, dots and tags: almost everything has a tooltip that explains what it is or what it does.",
              "Passez la souris sur les icônes, les pastilles et les étiquettes : presque tout a une info-bulle qui explique ce que c'est ou ce que ça fait.",
              "Fahr mit der Maus über Symbole, Punkte und Etiketten: Fast alles hat einen Hinweis, der erklärt, was es ist oder was es tut.",
            ),
            tr(
              "Lingua e tema si cambiano in Impostazioni › Generale › «Lingua» e «Tema»: di base il tema chiaro o scuro segue quello del sistema operativo, e vale anche per l'editor e i terminali.",
              "The language and the theme are changed in Settings › General › “Language” and “Theme”: by default the light or dark theme follows the operating system's, and it applies to the editor and the terminals too.",
              "La langue et le thème se changent dans Paramètres › Général › « Langue » et « Thème » : par défaut, le thème clair ou sombre suit celui du système d'exploitation, et il vaut aussi pour l'éditeur et les terminaux.",
              "Sprache und Design änderst du unter Einstellungen › Allgemein › „Sprache“ und „Design“: Standardmäßig folgt das helle oder dunkle Design dem des Betriebssystems, und es gilt auch für den Editor und die Terminals.",
            ),
            tr(
              "L'icona a forma di libro nella barra laterale di ogni pagina apre l'argomento della guida che la spiega.",
              "The book-shaped icon in the sidebar of every page opens the guide topic that explains it.",
              "L'icône en forme de livre dans la barre latérale de chaque page ouvre le sujet du guide qui l'explique.",
              "Das Buchsymbol in der Seitenleiste jeder Seite öffnet das Thema der Anleitung, das sie erklärt.",
            ),
          ]}
        />
      </Section>

      <Related ids={["project-page", "lifecycle", "first-project"]} />
    </>
  ),

  "first-project": () => (
    <>
      <InBrief
        points={[
          tr(
            "Un esempio completo: un piccolo sito in TypeScript con Vite, dalla creazione al browser.",
            "A complete example: a small TypeScript website with Vite, from creation to the browser.",
            "Un exemple complet : un petit site en TypeScript avec Vite, de la création au navigateur.",
            "Ein vollständiges Beispiel: eine kleine TypeScript-Website mit Vite, von der Erstellung bis zum Browser.",
          ),
          tr(
            "Servono solo il componente Node.js, che il kit installa da solo se manca, e una connessione a Internet.",
            "All you need is the Node.js component, which the kit installs by itself if it is missing, and an Internet connection.",
            "Il suffit du composant Node.js, que le kit installe tout seul s'il manque, et d'une connexion à Internet.",
            "Du brauchst nur die Komponente Node.js, die das Kit selbst installiert, wenn sie fehlt, und eine Internetverbindung.",
          ),
          tr(
            "Alla fine saprai creare un progetto, avviarlo, aprirlo nel browser e modificarlo nell'editor.",
            "By the end you will know how to create a project, start it, open it in the browser and edit it in the editor.",
            "À la fin, vous saurez créer un projet, le démarrer, l'ouvrir dans le navigateur et le modifier dans l'éditeur.",
            "Am Ende weißt du, wie du ein Projekt erstellst, startest, im Browser öffnest und im Editor bearbeitest.",
          ),
        ]}
      />

      <Section
        title={tr("1. Crea il progetto", "1. Create the project", "1. Créez le projet", "1. Erstelle das Projekt")}
      >
        <Steps
          steps={[
            tr(
              "Apri la pagina «Progetti» dalla barra delle attività.",
              "Open the “Projects” page from the activity bar.",
              "Ouvrez la page « Projets » depuis la barre d'activité.",
              "Öffne die Seite „Projekte“ über die Aktivitätsleiste.",
            ),
            tr(
              "Nel «Pannello rapido», sotto «Nuovo progetto», nel riquadro «Web», fai clic su «Vite» (sotto «Web con JavaScript e TypeScript»). Si apre la scheda «Nuovo progetto».",
              "In the “Quick panel”, under “New project”, in the “Web” block, click “Vite” (under “Web with JavaScript and TypeScript”). The “New project” tab opens.",
              "Dans le « Panneau rapide », sous « Nouveau projet », dans le bloc « Web », cliquez sur « Vite » (sous « Web avec JavaScript et TypeScript »). L'onglet « Nouveau projet » s'ouvre.",
              "Klicke in der „Schnellübersicht“ unter „Neues Projekt“ im Block „Web“ auf „Vite“ (unter „Web mit JavaScript und TypeScript“). Der Tab „Neues Projekt“ öffnet sich.",
            ),
            tr(
              "In «1. Base» la scheda «Vite» («Sito o applicazione web in TypeScript») è già scelta.",
              "In “1. Base” the “Vite” card (“Website or web application in TypeScript”) is already chosen.",
              "Dans « 1. Base », la carte « Vite » (« Site ou application web en TypeScript ») est déjà choisie.",
              "In „1. Basis“ ist die Karte „Vite“ („Website oder Webanwendung in TypeScript“) bereits ausgewählt.",
            ),
            tr(
              "In «2. Aggiunte» non serve niente per questa prova.",
              "In “2. Additions” nothing is needed for this test.",
              "Dans « 2. Ajouts », rien n'est nécessaire pour cet essai.",
              "In „2. Ergänzungen“ brauchst du für diesen Versuch nichts.",
            ),
            tr(
              "In «3. Nome» scrivi primo-sito. Il nome diventa la cartella projects\\primo-sito.",
              "In “3. Name” type primo-sito. The name becomes the projects\\primo-sito folder.",
              "Dans « 3. Nom », tapez primo-sito. Le nom devient le dossier projects\\primo-sito.",
              "Gib in „3. Name“ primo-sito ein. Der Name wird zum Ordner projects\\primo-sito.",
            ),
            tr(
              "Guarda il «Riepilogo»: vedrai che il kit userà Node.js, eseguirà npm create vite nella cartella projects e poi npm install nel progetto, e salverà le azioni «Avvia» e «Build».",
              "Look at the “Summary”: you will see that the kit will use Node.js, run npm create vite in the projects folder and then npm install in the project, and save the “Start” and “Build” actions.",
              "Regardez le « Récapitulatif » : vous verrez que le kit utilisera Node.js, exécutera npm create vite dans le dossier projects puis npm install dans le projet, et enregistrera les actions « Démarrer » et « Build ».",
              "Sieh dir die „Zusammenfassung“ an: Du siehst, dass das Kit Node.js verwendet, npm create vite im Ordner projects und dann npm install im Projekt ausführt und die Aktionen „Starten“ und „Build“ speichert.",
            ),
            tr(
              "Premi «Crea il progetto». Nella parte «Avanzamento» scorrono i comandi e il loro output.",
              "Press “Create the project”. In the “Progress” part the commands and their output scroll by.",
              "Appuyez sur « Créer le projet ». Dans la partie « Progression », les commandes et leur sortie défilent.",
              "Klicke auf „Projekt erstellen“. Im Teil „Fortschritt“ laufen die Befehle und ihre Ausgabe durch.",
            ),
            tr(
              "Quando compare «Progetto primo-sito creato.» premi «Apri il progetto».",
              "When “Project primo-sito created.” appears, press “Open the project”.",
              "Quand « Projet primo-sito créé. » apparaît, appuyez sur « Ouvrir le projet ».",
              "Wenn „Projekt primo-sito erstellt.“ erscheint, klicke auf „Projekt öffnen“.",
            ),
          ]}
        />
        <Callout tone="info">
          {tr(
            "Se Node.js non era installato, il kit lo scarica prima di eseguire i comandi: lo vedi nelle prime righe dell'avanzamento («Installo gli strumenti del kit che servono…»).",
            "If Node.js was not installed, the kit downloads it before running the commands: you see it in the first lines of the progress (“Installing the kit tools needed…”).",
            "Si Node.js n'était pas installé, le kit le télécharge avant d'exécuter les commandes : vous le voyez dans les premières lignes de la progression (« Installation des outils du kit nécessaires… »).",
            "Wenn Node.js nicht installiert war, lädt das Kit es herunter, bevor es die Befehle ausführt: Du siehst es in den ersten Zeilen des Fortschritts („Benötigte Werkzeuge des Kits werden installiert…“).",
          )}
        </Callout>
      </Section>

      <Section
        title={tr(
          "2. Avvialo e aprilo nel browser",
          "2. Start it and open it in the browser",
          "2. Démarrez-le et ouvrez-le dans le navigateur",
          "2. Starte es und öffne es im Browser",
        )}
      >
        <Steps
          steps={[
            tr(
              "La pagina del progetto si apre sulla sezione «Avvio». In basso, l'elenco «Azioni» mostra ciò che il progetto sa fare: le azioni riconosciute dal file package.json (npm run dev, npm run build, npm install) e quelle della ricetta (Avvia, Build).",
              "The project page opens on the “Run” section. At the bottom, the “Actions” list shows what the project can do: the actions recognized from the package.json file (npm run dev, npm run build, npm install) and those from the recipe (Start, Build).",
              "La page du projet s'ouvre sur la section « Lancement ». En bas, la liste « Actions » montre ce que le projet sait faire : les actions reconnues à partir du fichier package.json (npm run dev, npm run build, npm install) et celles de la recette (Démarrer, Build).",
              "Die Seite des Projekts öffnet sich im Abschnitt „Start“. Unten zeigt die Liste „Aktionen“, was das Projekt kann: die aus der Datei package.json erkannten Aktionen (npm run dev, npm run build, npm install) und die aus dem Rezept (Starten, Build).",
            ),
            tr(
              "Premi ▶ accanto a «npm run dev». È un'azione web: il kit le assegna una porta, la avvia e aspetta che risponda. Lo stato passa da «in avvio» a «in ascolto».",
              "Press ▶ next to “npm run dev”. It is a web action: the kit assigns it a port, starts it and waits for it to respond. The status goes from “starting” to “listening”.",
              "Appuyez sur ▶ à côté de « npm run dev ». C'est une action web : le kit lui attribue un port, la démarre et attend qu'elle réponde. L'état passe de « démarrage » à « en écoute ».",
              "Klicke auf ▶ neben „npm run dev“. Es ist eine Web-Aktion: Das Kit weist ihr einen Port zu, startet sie und wartet, bis sie antwortet. Der Status wechselt von „startet“ zu „lauscht“.",
            ),
            tr(
              "Premi «Apri nel browser»: si apre il sito di prova di Vite all'indirizzo http://localhost:<porta>/, dove la porta è un numero tra 7100 e 7999 scelto dal kit e tenuto uguale a ogni avvio.",
              "Press “Open in the browser”: Vite's sample site opens at http://localhost:<port>/, where the port is a number between 7100 and 7999 chosen by the kit and kept the same at every start.",
              "Appuyez sur « Ouvrir dans le navigateur » : le site d'exemple de Vite s'ouvre à l'adresse http://localhost:<port>/, où le port est un nombre entre 7100 et 7999 choisi par le kit et conservé à chaque démarrage.",
              "Klicke auf „Im Browser öffnen“: Die Beispielseite von Vite öffnet sich unter http://localhost:<port>/, wobei der Port eine Zahl zwischen 7100 und 7999 ist, die das Kit wählt und bei jedem Start beibehält.",
            ),
            tr(
              "Vuoi che il grande pulsante «Avvia» in alto avvii sempre questa azione? Selezionala e premi la stella «Predefinita».",
              "Do you want the big “Start” button at the top to always start this action? Select it and press the “Default” star.",
              "Vous voulez que le grand bouton « Démarrer » en haut démarre toujours cette action ? Sélectionnez-la et appuyez sur l'étoile « Par défaut ».",
              "Soll die große Schaltfläche „Starten“ oben immer diese Aktion starten? Wähle sie aus und klicke auf den Stern „Standard“.",
            ),
          ]}
        />
        <Callout tone="info">
          {rich(
            tr(
              "Con la VM attiva («VM» nella barra di stato) npm run dev gira nella microVM del progetto, con il Node.js del kit per Linux, e la porta arriva su localhost del PC: per te non cambia niente. «Ferma» la ferma dentro la VM. I dettagli: {0}.",
              "With the VM on (“VM” in the status bar) npm run dev runs in the project's microVM, with the kit's Node.js for Linux, and the port reaches the PC's localhost: for you nothing changes. “Stop” stops it inside the VM. The details: {0}.",
              "Avec la VM active (« VM » dans la barre d'état), npm run dev tourne dans la microVM du projet, avec le Node.js du kit pour Linux, et le port arrive sur le localhost du PC : pour vous, rien ne change. « Arrêter » l'arrête dans la VM. Les détails : {0}.",
              "Mit aktiver VM („VM“ in der Statusleiste) läuft npm run dev in der microVM des Projekts, mit dem Node.js des Kits für Linux, und der Port kommt auf localhost des PCs an: Für dich ändert sich nichts. „Beenden“ beendet sie in der VM. Die Details: {0}.",
            ),
            <See key="v" id="vm" />,
          )}
        </Callout>
      </Section>

      <Section
        title={tr(
          "3. Modificalo nell'editor",
          "3. Edit it in the editor",
          "3. Modifiez-le dans l'éditeur",
          "3. Bearbeite es im Editor",
        )}
      >
        <Steps
          steps={[
            tr(
              "Premi «Editor» in alto nella pagina del progetto. Il kit avvia l'editor (la prima volta può richiedere qualche secondo) e apre una finestra di Atollo con la cartella del progetto.",
              "Press “Editor” at the top of the project page. The kit starts the editor (the first time it may take a few seconds) and opens an Atollo window with the project folder.",
              "Appuyez sur « Éditeur » en haut de la page du projet. Le kit démarre l'éditeur (la première fois, cela peut prendre quelques secondes) et ouvre une fenêtre d'Atollo avec le dossier du projet.",
              "Klicke oben auf der Seite des Projekts auf „Editor“. Das Kit startet den Editor (beim ersten Mal kann das einige Sekunden dauern) und öffnet ein Atollo-Fenster mit dem Projektordner.",
            ),
            tr(
              "Apri src/main.ts, cambia un testo e salva con Ctrl+S.",
              "Open src/main.ts, change some text and save with Ctrl+S.",
              "Ouvrez src/main.ts, changez un texte et enregistrez avec Ctrl+S.",
              "Öffne src/main.ts, ändere einen Text und speichere mit Ctrl+S.",
            ),
            tr(
              "Torna al browser: Vite aggiorna la pagina da solo.",
              "Go back to the browser: Vite updates the page by itself.",
              "Revenez au navigateur : Vite met la page à jour tout seul.",
              "Wechsle zurück zum Browser: Vite aktualisiert die Seite von selbst.",
            ),
            tr(
              "Per fermare il sito torna su Atollo e premi «Ferma» (o ■ accanto all'azione). Chiudendo Atollo si ferma comunque tutto.",
              "To stop the website go back to Atollo and press “Stop” (or ■ next to the action). Closing Atollo stops everything anyway.",
              "Pour arrêter le site, revenez sur Atollo et appuyez sur « Arrêter » (ou ■ à côté de l'action). Fermer Atollo arrête de toute façon tout.",
              "Um die Website zu beenden, wechsle zurück zu Atollo und klicke auf „Beenden“ (oder ■ neben der Aktion). Wenn du Atollo schließt, wird ohnehin alles beendet.",
            ),
          ]}
        />
      </Section>

      <Section title={tr("E adesso?", "What next?", "Et maintenant ?", "Und jetzt?")}>
        <Bullets
          items={[
            rich(
              tr(
                "Mettere il progetto su GitHub o clonarne uno: {0}.",
                "Putting the project on GitHub or cloning one: {0}.",
                "Mettre le projet sur GitHub ou en cloner un : {0}.",
                "Das Projekt auf GitHub stellen oder eines klonen: {0}.",
              ),
              <See key="g" id="github" />,
            ),
            rich(
              tr(
                "Dare al progetto un database: {0}.",
                "Giving the project a database: {0}.",
                "Donner une base de données au projet : {0}.",
                "Dem Projekt eine Datenbank geben: {0}.",
              ),
              <See key="d" id="project-db" />,
            ),
            rich(
              tr(
                "Farti aiutare da un agente IA nel progetto: {0}.",
                "Getting help from an AI agent in the project: {0}.",
                "Se faire aider par un agent IA dans le projet : {0}.",
                "Sich im Projekt von einem KI-Agenten helfen lassen: {0}.",
              ),
              <See key="a" id="agents" />,
            ),
            rich(
              tr(
                "Provare altre basi (Laravel, Django, WordPress, Astro…): {0}.",
                "Trying other bases (Laravel, Django, WordPress, Astro…): {0}.",
                "Essayer d'autres bases (Laravel, Django, WordPress, Astro…) : {0}.",
                "Andere Basen ausprobieren (Laravel, Django, WordPress, Astro…): {0}.",
              ),
              <See key="n" id="project-new" />,
            ),
          ]}
        />
      </Section>

      <Related ids={["project-new", "project-run", "project-editor"]} />
    </>
  ),
};
