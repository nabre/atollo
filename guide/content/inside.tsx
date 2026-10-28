// Chapter 8 of the Guide, «Come funziona dentro»: isolation, the folders on
// disk, background processes, download sources and checks, the command line.
import { rich, tr } from "../core/lang";
import { sourceCategories, sourceCounts, TOOL_SOURCES } from "./sources";
import {
  Bullets,
  Callout,
  Code,
  Flow,
  GuideCode,
  InBrief,
  KitFoldersBar,
  KitInfo,
  P,
  Related,
  Section,
  See,
  SourcesTable,
  Steps,
  Table,
  Under,
} from "../core/widgets";
import type { TopicTexts } from "../core/types";

/** The folders of the kit, one row each, indented by depth. */
function diskTree(): { path: string; depth: number; text: string }[] {
  return [
    {
      path: tr("<cartella del kit>\\", "<kit folder>\\", "<dossier du kit>\\", "<Kit-Ordner>\\"),
      depth: 0,
      text: tr(
        "La cartella che contiene l'eseguibile, per esempio D:\\Atollo.",
        "The folder that contains the executable, for example D:\\Atollo.",
        "Le dossier qui contient l'exécutable, par exemple D:\\Atollo.",
        "Der Ordner, der die ausführbare Datei enthält, zum Beispiel D:\\Atollo.",
      ),
    },
    {
      path: "Atollo.exe",
      depth: 1,
      text: tr(
        "L'unico programma: è l'app con la finestra e anche la riga di comando. Su macOS Atollo.app, su Linux l'AppImage.",
        "The only program: it is the app with the window and also the command line. On macOS Atollo.app, on Linux the AppImage.",
        "Le seul programme : c'est l'application avec la fenêtre et aussi la ligne de commande. Sur macOS Atollo.app, sur Linux l'AppImage.",
        "Das einzige Programm: Es ist die App mit dem Fenster und zugleich die Befehlszeile. Unter macOS Atollo.app, unter Linux das AppImage.",
      ),
    },
    {
      path: "projects\\",
      depth: 1,
      text: tr(
        "I tuoi progetti, una cartella ciascuno. È tuo: il kit non ci scrive senza che tu lo chieda.",
        "Your projects, one folder each. It is yours: the kit does not write there unless you ask.",
        "Vos projets, un dossier chacun. Il est à vous : le kit n'y écrit pas sans que vous le demandiez.",
        "Deine Projekte, je ein Ordner. Er gehört dir: Das Kit schreibt dort nichts, ohne dass du es verlangst.",
      ),
    },
    {
      path: "data\\",
      depth: 1,
      text: tr(
        "Tutto ciò che è del kit.",
        "Everything that belongs to the kit.",
        "Tout ce qui appartient au kit.",
        "Alles, was zum Kit gehört.",
      ),
    },
    {
      path: "app\\",
      depth: 2,
      text: tr(
        "Lo stato dell'app: settings.json (Impostazioni), secrets\\ (la chiave di Gemini, quella del server dei modelli locali e il token di Hugging Face, protetti), llm.json e llm\\presets.ini (il server dei modelli locali), ca-bundle.pem (i certificati, se le Impostazioni hanno un file PEM), components.txt (componenti scelti), versions\\ (versioni installate), logs\\, services.json e servers.json (database e server web, con le porte di ogni versione e di ogni pool), terminals.json, frameworks.json, root.txt (ultima cartella del kit), safe-directory.txt (l'ultimo safe.directory scritto dal kit).",
        "The app's state: settings.json (Settings), secrets\\ (the Gemini key, the local models server's key and the Hugging Face token, protected), llm.json and llm\\presets.ini (the local models server), ca-bundle.pem (the certificates, when Settings has a PEM file), components.txt (chosen components), versions\\ (installed versions), logs\\, services.json and servers.json (databases and web servers, with the ports of every version and every pool), terminals.json, frameworks.json, root.txt (last kit folder), safe-directory.txt (the last safe.directory the kit wrote).",
        "L'état de l'application : settings.json (Paramètres), secrets\\ (la clé de Gemini, celle du serveur des modèles locaux et le jeton Hugging Face, protégé), llm.json et llm\\presets.ini (le serveur des modèles locaux), ca-bundle.pem (les certificats, si les Paramètres ont un fichier PEM), components.txt (composants choisis), versions\\ (versions installées), logs\\, services.json et servers.json (bases de données et serveurs web, avec les ports de chaque version et de chaque pool), terminals.json, frameworks.json, root.txt (dernier dossier du kit), safe-directory.txt (le dernier safe.directory écrit par le kit).",
        "Der Zustand der App: settings.json (Einstellungen), secrets\\ (der Gemini-Schlüssel, der Schlüssel des Servers der lokalen Modelle und das Hugging-Face-Token, geschützt), llm.json und llm\\presets.ini (der Server der lokalen Modelle), ca-bundle.pem (die Zertifikate, wenn die Einstellungen eine PEM-Datei haben), components.txt (gewählte Komponenten), versions\\ (installierte Versionen), logs\\, services.json und servers.json (Datenbanken und Webserver, mit den Ports jeder Version und jedes Pools), terminals.json, frameworks.json, root.txt (letzter Kit-Ordner), safe-directory.txt (das letzte vom Kit geschriebene safe.directory).",
      ),
    },
    {
      path: "app\\db\\",
      depth: 3,
      text: tr(
        "Password di amministratore dei database e accessi di mysql. Protetta: la legge solo il tuo utente.",
        "Database administrator passwords and mysql logins. Protected: only your user can read it.",
        "Mots de passe d'administrateur des bases de données et accès de mysql. Protégé : seul votre utilisateur peut le lire.",
        "Administratorpasswörter der Datenbanken und Zugänge von mysql. Geschützt: Nur dein Benutzer kann ihn lesen.",
      ),
    },
    {
      path: "app\\diagnostics\\",
      depth: 3,
      text: tr(
        "Gli zip di «Esporta diagnostica» (atollo-diagnostica-<data-ora>.zip), da allegare a una segnalazione: segreti mascherati, restano sul PC.",
        "The zips of “Export diagnostics” (atollo-diagnostica-<date-time>.zip), to attach to a report: secrets masked, they stay on the PC.",
        "Les zip de « Exporter le diagnostic » (atollo-diagnostica-<date-heure>.zip), à joindre à un signalement : secrets masqués, ils restent sur le PC.",
        "Die Zips von „Diagnose exportieren“ (atollo-diagnostica-<Datum-Uhrzeit>.zip), zum Anhängen an eine Meldung: Geheimnisse maskiert, sie bleiben auf dem PC.",
      ),
    },
    {
      path: "app\\sandboxes\\",
      depth: 3,
      text: tr(
        "La descrizione di ogni sandbox delle VM (di chi è, cartelle montate, porte aperte, connessione al database leggibile solo da te) e secrets.json, i comandi di «GitHub nella VM».",
        "The description of each VM sandbox (whose it is, mounted folders, open ports, database connection readable only by you) and secrets.json, the commands of “GitHub in the VM”.",
        "La description de chaque sandbox des VM (à qui elle appartient, dossiers montés, ports ouverts, connexion à la base lisible par vous seul) et secrets.json, les commandes de « GitHub dans la VM ».",
        "Die Beschreibung jeder Sandbox der VMs (Besitzer, eingehängte Ordner, offene Ports, nur für dich lesbare Datenbankverbindung) und secrets.json, die Befehle von „GitHub in der VM“.",
      ),
    },
    {
      path: tr("apps\\<sistema>\\", "apps\\<system>\\", "apps\\<système>\\", "apps\\<System>\\"),
      depth: 2,
      text: tr(
        "I programmi, per sistema (win32-x64, darwin-arm64…), una cartella per componente e per versione. Con la VM attiva c'è anche linux-x64 (linux-arm64 sui Mac con Apple silicon): gli strumenti che la VM monta in sola lettura.",
        "The programs, by system (win32-x64, darwin-arm64…), one folder per component and per version. With the VM on there is also linux-x64 (linux-arm64 on Macs with Apple silicon): the tools the VM mounts read-only.",
        "Les programmes, par système (win32-x64, darwin-arm64…), un dossier par composant et par version. Avec la VM active, il y a aussi linux-x64 (linux-arm64 sur les Mac avec Apple silicon) : les outils que la VM monte en lecture seule.",
        "Die Programme, nach System (win32-x64, darwin-arm64…), ein Ordner pro Komponente und pro Version. Mit aktiver VM gibt es auch linux-x64 (linux-arm64 auf Macs mit Apple Silicon): die Werkzeuge, die die VM schreibgeschützt einhängt.",
      ),
    },
    {
      path: tr("projects\\<progetto>\\", "projects\\<project>\\", "projects\\<projet>\\", "projects\\<Projekt>\\"),
      depth: 2,
      text: tr(
        "I dati del kit per ogni progetto: dev-settings.json (versioni, estensioni, preferiti), ports.json (le porte delle sue azioni), configurazioni di Apache e Caddy (le tue direttive in apache\\conf.d\\*.conf e caddy\\conf.d\\*.caddy valgono sia per il server tutto suo sia in un pool), private\\ (password e copie dei database, protetta).",
        "The kit's data for each project: dev-settings.json (versions, extensions, favorites), ports.json (the ports of its actions), Apache and Caddy configurations (your directives in apache\\conf.d\\*.conf and caddy\\conf.d\\*.caddy apply both to its own server and in a pool), private\\ (database passwords and backups, protected).",
        "Les données du kit pour chaque projet : dev-settings.json (versions, extensions, favoris), ports.json (les ports de ses actions), configurations d'Apache et Caddy (vos directives dans apache\\conf.d\\*.conf et caddy\\conf.d\\*.caddy valent aussi bien pour son propre serveur que dans un pool), private\\ (mots de passe et copies des bases de données, protégé).",
        "Die Daten des Kits für jedes Projekt: dev-settings.json (Versionen, Erweiterungen, Favoriten), ports.json (die Ports seiner Aktionen), Konfigurationen von Apache und Caddy (deine Direktiven in apache\\conf.d\\*.conf und caddy\\conf.d\\*.caddy gelten für seinen eigenen Server wie auch in einem Pool), private\\ (Passwörter und Sicherungen der Datenbanken, geschützt).",
      ),
    },
    {
      path: tr(
        "db\\<mysql|mongodb>\\<versione>\\",
        "db\\<mysql|mongodb>\\<version>\\",
        "db\\<mysql|mongodb>\\<version>\\",
        "db\\<mysql|mongodb>\\<Version>\\",
      ),
      depth: 2,
      text: tr(
        "I dati dei database, una cartella per versione.",
        "The databases' data, one folder per version.",
        "Les données des bases de données, un dossier par version.",
        "Die Daten der Datenbanken, ein Ordner pro Version.",
      ),
    },
    {
      path: tr(
        "models\\gguf\\<proprietario>--<nome>\\",
        "models\\gguf\\<owner>--<name>\\",
        "models\\gguf\\<propriétaire>--<nom>\\",
        "models\\gguf\\<Eigentümer>--<Name>\\",
      ),
      depth: 2,
      text: tr(
        "I modelli locali (file GGUF), una cartella per repository di Hugging Face, una sola copia per Windows, macOS e Linux; atollo.json dice che cosa ha scaricato il kit.",
        "The local models (GGUF files), one folder per Hugging Face repository, a single copy for Windows, macOS and Linux; atollo.json says what the kit downloaded.",
        "Les modèles locaux (fichiers GGUF), un dossier par dépôt Hugging Face, une seule copie pour Windows, macOS et Linux ; atollo.json dit ce que le kit a téléchargé.",
        "Die lokalen Modelle (GGUF-Dateien), ein Ordner pro Hugging-Face-Repository, eine einzige Kopie für Windows, macOS und Linux; atollo.json sagt, was das Kit heruntergeladen hat.",
      ),
    },
    {
      path: "servers\\<pool>\\",
      depth: 2,
      text: tr(
        "Le configurazioni generate dei server web condivisi.",
        "The generated configurations of the shared web servers.",
        "Les configurations générées des serveurs web partagés.",
        "Die erzeugten Konfigurationen der gemeinsamen Webserver.",
      ),
    },
    {
      path: "agents\\",
      depth: 2,
      text: tr(
        "Ciò che gli agenti condividono: AGENTS.md, skills\\, mcp.json, e lo stato dell'allineamento in .sync\\.",
        "What the agents share: AGENTS.md, skills\\, mcp.json, and the sync state in .sync\\.",
        "Ce que les agents partagent : AGENTS.md, skills\\, mcp.json, et l'état de l'alignement dans .sync\\.",
        "Was die Agenten gemeinsam nutzen: AGENTS.md, skills\\, mcp.json und den Stand des Abgleichs in .sync\\.",
      ),
    },
    {
      path: "antigravity\\",
      depth: 2,
      text: tr(
        "L'accesso di Antigravity che un Atollo precedente ha conservato nel kit (keyring.json): solo letto, «Esci» lo toglie.",
        "The Antigravity sign-in an earlier Atollo kept in the kit (keyring.json): only read, “Sign out” removes it.",
        "La connexion d'Antigravity qu'un Atollo précédent a conservée dans le kit (keyring.json) : seulement lue, « Se déconnecter » la retire.",
        "Die Antigravity-Anmeldung, die ein früheres Atollo im Kit aufbewahrt hat (keyring.json): nur gelesen, „Abmelden“ entfernt sie.",
      ),
    },
    {
      path: "home\\",
      depth: 2,
      text: tr(
        "La «cartella utente» dei programmi del kit: .claude, .codex, .gemini, .pi (profili degli agenti), .gitconfig, .npmrc, AppData (su Windows). Qui stanno anche impostazioni, registri, immagini e dischi delle VM di Docker Sandboxes.",
        "The “user folder” of the kit's programs: .claude, .codex, .gemini, .pi (agent profiles), .gitconfig, .npmrc, AppData (on Windows). Docker Sandboxes' settings, logs, images and VM disks are here too.",
        "Le « dossier utilisateur » des programmes du kit : .claude, .codex, .gemini, .pi (profils des agents), .gitconfig, .npmrc, AppData (sur Windows). Ici se trouvent aussi les paramètres, journaux, images et disques des VM de Docker Sandboxes.",
        "Der „Benutzerordner“ der Programme des Kits: .claude, .codex, .gemini, .pi (Profile der Agenten), .gitconfig, .npmrc, AppData (unter Windows). Hier liegen auch Einstellungen, Logs, Images und VM-Festplatten von Docker Sandboxes.",
      ),
    },
    {
      path: "gh\\",
      depth: 2,
      text: tr(
        "L'accesso di GitHub CLI (hosts.yml).",
        "The GitHub CLI sign-in (hosts.yml).",
        "La connexion de GitHub CLI (hosts.yml).",
        "Die Anmeldung von GitHub CLI (hosts.yml).",
      ),
    },
    {
      path: "vscodium-server\\",
      depth: 2,
      text: tr(
        "Impostazioni ed estensioni del server dell'editor.",
        "Settings and extensions of the editor server.",
        "Paramètres et extensions du serveur de l'éditeur.",
        "Einstellungen und Erweiterungen des Editor-Servers.",
      ),
    },
    {
      path: "kit\\",
      depth: 2,
      text: tr(
        "File che il kit riscrive a ogni avvio: profilo della PowerShell, comandi di supporto (agy, mysql, gradle).",
        "Files the kit rewrites at every start: the PowerShell profile, helper commands (agy, mysql, gradle).",
        "Fichiers que le kit réécrit à chaque démarrage : profil de PowerShell, commandes d'appoint (agy, mysql, gradle).",
        "Dateien, die das Kit bei jedem Start neu schreibt: das PowerShell-Profil, Hilfsbefehle (agy, mysql, gradle).",
      ),
    },
    {
      path: "cache\\",
      depth: 2,
      text: tr(
        "Cache dei gestori di pacchetti (npm, pip, uv, Composer, Go, NuGet…) e delle risposte di GitHub. Si può cancellare.",
        "Cache of the package managers (npm, pip, uv, Composer, Go, NuGet…) and of GitHub's responses. It can be deleted.",
        "Cache des gestionnaires de paquets (npm, pip, uv, Composer, Go, NuGet…) et des réponses de GitHub. On peut l'effacer.",
        "Cache der Paketmanager (npm, pip, uv, Composer, Go, NuGet…) und der Antworten von GitHub. Kann gelöscht werden.",
      ),
    },
    {
      path: "python-user\\, npm-global\\, composer\\, go\\, gradle\\…",
      depth: 2,
      text: tr(
        "Pacchetti e strumenti installati globalmente con pip, npm, Composer, Go e gli altri.",
        "Packages and tools installed globally with pip, npm, Composer, Go and the others.",
        "Paquets et outils installés globalement avec pip, npm, Composer, Go et les autres.",
        "Pakete und Werkzeuge, die global mit pip, npm, Composer, Go und den anderen installiert wurden.",
      ),
    },
    {
      path: "tmp\\",
      depth: 2,
      text: tr(
        "File temporanei, quando le Impostazioni li vogliono nel kit.",
        "Temporary files, when Settings wants them in the kit.",
        "Fichiers temporaires, quand les Paramètres les veulent dans le kit.",
        "Temporäre Dateien, wenn die Einstellungen sie im Kit haben wollen.",
      ),
    },
    {
      path: "webview2\\",
      depth: 2,
      text: tr(
        "I dati della finestra dell'app.",
        "The data of the app window.",
        "Les données de la fenêtre de l'application.",
        "Die Daten des App-Fensters.",
      ),
    },
  ];
}

const CLI_EXAMPLE = `cd D:\\Atollo
.\\Atollo.exe components | Out-Host
.\\Atollo.exe prepare --only node,php --line php=8.3 | Out-Host
.\\Atollo.exe check | Out-Host`;

export const INSIDE: TopicTexts = {
  vm: () => (
    <>
      <InBrief
        points={[
          tr(
            "Il cuore di Atollo è il configuratore di Docker Sandboxes (sbx): gli agenti di ogni progetto girano in una microVM, con un Linux suo, e vedono solo la cartella del progetto.",
            "The heart of Atollo is the configurator of Docker Sandboxes (sbx): each project's agents run in a microVM, with its own Linux, and see only the project's folder.",
            "Le cœur d'Atollo est le configurateur de Docker Sandboxes (sbx) : les agents de chaque projet tournent dans une microVM, avec son propre Linux, et ne voient que le dossier du projet.",
            "Das Herz von Atollo ist der Konfigurator von Docker Sandboxes (sbx): Die Agenten jedes Projekts laufen in einer microVM, mit eigenem Linux, und sehen nur den Ordner des Projekts.",
          ),
          tr(
            "Due posti: sul PC tutto ciò che non è un agente, con o senza la VM (i terminali del kit e dell'editor, il codice del progetto: terminale del progetto, azioni, siti, task, creazione; i server web e i database); nella VM solo gli agenti (Claude Code, Codex, Pi, Antigravity).",
            "Two places: on the PC everything that is not an agent, with or without the VM (the kit's and the editor's terminals, the project's code: project terminal, actions, websites, tasks, creation; the web and database servers); in the VM only the agents (Claude Code, Codex, Pi, Antigravity).",
            "Deux endroits : sur le PC tout ce qui n'est pas un agent, avec ou sans la VM (les terminaux du kit et de l'éditeur, le code du projet : terminal du projet, actions, sites, tâches, création ; les serveurs web et de bases de données) ; dans la VM seulement les agents (Claude Code, Codex, Pi, Antigravity).",
            "Zwei Orte: auf dem PC alles, was kein Agent ist, mit oder ohne VM (die Terminals des Kits und des Editors, der Code des Projekts: Terminal des Projekts, Aktionen, Websites, Tasks, Anlegen; die Web- und Datenbankserver); in der VM nur die Agenten (Claude Code, Codex, Pi, Antigravity).",
          ),
          tr(
            "Docker Sandboxes è un componente obbligatorio del kit e Atollo usa solo quello: mai uno installato sul PC. Serve un account Docker gratuito, con «sbx login» nel terminale di Atollo.",
            "Docker Sandboxes is a required component of the kit and Atollo uses only that one: never one installed on the PC. A free Docker account is needed, with “sbx login” in Atollo's terminal.",
            "Docker Sandboxes est un composant obligatoire du kit et Atollo n'utilise que celui-là : jamais un autre installé sur le PC. Il faut un compte Docker gratuit, avec « sbx login » dans le terminal d'Atollo.",
            "Docker Sandboxes ist eine Pflichtkomponente des Kits, und Atollo verwendet nur diese: nie eine auf dem PC installierte. Nötig ist ein kostenloses Docker-Konto, mit „sbx login“ im Terminal von Atollo.",
          ),
          tr(
            "Quando la VM non può partire, la barra di stato dice «Senza VM»: un clic spiega perché e che cosa fare. Gli agenti sono disattivati, con il motivo; il codice dei progetti gira sul PC come sempre, con l'ambiente sigillato del kit.",
            "When the VM cannot start, the status bar says “No VM”: a click tells why and what to do. The agents are disabled, with the reason; the projects' code runs on the PC as always, with the kit's sealed environment.",
            "Quand la VM ne peut pas démarrer, la barre d'état indique « Sans VM » : un clic dit pourquoi et quoi faire. Les agents sont désactivés, avec le motif ; le code des projets tourne sur le PC comme toujours, avec l'environnement scellé du kit.",
            "Wenn die VM nicht starten kann, zeigt die Statusleiste „Ohne VM“: Ein Klick sagt, warum und was zu tun ist. Die Agenten sind deaktiviert, mit dem Grund; der Code der Projekte läuft wie immer auf dem PC, mit der versiegelten Umgebung des Kits.",
          ),
        ]}
      />

      <Section title={tr("Che cos'è", "What it is", "Ce que c'est", "Was das ist")}>
        <P>
          {tr(
            "Una microVM è un piccolo computer virtuale che parte in pochi secondi. Docker Sandboxes ne crea una per gli agenti di ogni progetto: dentro c'è un Linux con la cartella del progetto, in scrittura, e gli strumenti del kit per Linux, in sola lettura. Il resto del PC e del kit resta fuori: un agente, e ciò che esegue, non può leggere i tuoi documenti, le tue chiavi SSH o gli altri progetti. Il terminale, le azioni e i siti del progetto girano invece sul PC.",
            "A microVM is a small virtual computer that starts in a few seconds. Docker Sandboxes makes one for each project's agents: inside there is a Linux with the project's folder, writable, and the kit's tools for Linux, read-only. The rest of the PC and of the kit stays outside: an agent, and what it runs, cannot read your documents, your SSH keys or the other projects. The project's terminal, actions and websites run on the PC instead.",
            "Une microVM est un petit ordinateur virtuel qui démarre en quelques secondes. Docker Sandboxes en crée une pour les agents de chaque projet : dedans, un Linux avec le dossier du projet, en écriture, et les outils du kit pour Linux, en lecture seule. Le reste du PC et du kit reste dehors : un agent, et ce qu'il exécute, ne peut lire ni vos documents, ni vos clés SSH, ni les autres projets. Le terminal, les actions et les sites du projet tournent en revanche sur le PC.",
            "Eine microVM ist ein kleiner virtueller Computer, der in wenigen Sekunden startet. Docker Sandboxes legt eine für die Agenten jedes Projekts an: Darin ist ein Linux mit dem Ordner des Projekts, beschreibbar, und den Werkzeugen des Kits für Linux, schreibgeschützt. Der Rest des PCs und des Kits bleibt draußen: Ein Agent, und was er ausführt, kann weder deine Dokumente noch deine SSH-Schlüssel noch die anderen Projekte lesen. Terminal, Aktionen und Websites des Projekts laufen dagegen auf dem PC.",
          )}
        </P>
        <P>
          {tr(
            "Non c'è niente da accendere: il livello lo decide il PC. Atollo controlla lo stato all'avvio e poi ogni minuto; la barra di stato lo mostra con una pill: verde «VM» quando tutto è pronto, blu «VM…» con la percentuale mentre prepara la VM di un progetto (strumenti per Linux, database, la prima volta l'immagine della VM), arancione «Senza VM» quando puoi rimediare, grigia «Senza VM» quando questo PC non può averla, rossa «VM ⚠» quando una preparazione non è riuscita. La rete della VM passa dal proxy di Docker Sandboxes, con la politica che Atollo imposta (quella consigliata da Docker per lo sviluppo: servizi AI e registri di pacchetti).",
            "There is nothing to turn on: the PC decides the level. Atollo checks the status at start and then every minute; the status bar shows it with a pill: green “VM” when everything is ready, blue “VM…” with the percent while it prepares a project's VM (Linux tools, databases, the first time the VM's image), orange “No VM” when you can fix it, grey “No VM” when this PC cannot have it, red “VM ⚠” when a preparation failed. The VM's network goes through the Docker Sandboxes proxy, with the policy Atollo sets (the one Docker recommends for development: AI services and package registries).",
            "Il n'y a rien à activer : c'est le PC qui décide du niveau. Atollo vérifie l'état au démarrage puis chaque minute ; la barre d'état l'affiche avec une pastille : « VM » en vert quand tout est prêt, « VM… » en bleu avec le pourcentage pendant qu'il prépare la VM d'un projet (outils pour Linux, bases de données, la première fois l'image de la VM), « Sans VM » en orange quand vous pouvez y remédier, en gris quand ce PC ne peut pas l'avoir, « VM ⚠ » en rouge quand une préparation a échoué. Le réseau de la VM passe par le proxy de Docker Sandboxes, avec la politique qu'Atollo définit (celle que Docker conseille pour le développement : services d'IA et registres de paquets).",
            "Es gibt nichts einzuschalten: Die Stufe entscheidet der PC. Atollo prüft den Zustand beim Start und dann jede Minute; die Statusleiste zeigt sie als Pille: grün „VM“, wenn alles bereit ist, blau „VM…“ mit dem Prozentsatz, während die VM eines Projekts vorbereitet wird (Werkzeuge für Linux, Datenbanken, beim ersten Mal das Image der VM), orange „Ohne VM“, wenn du es beheben kannst, grau „Ohne VM“, wenn dieser PC sie nicht haben kann, rot „VM ⚠“, wenn eine Vorbereitung fehlgeschlagen ist. Das Netzwerk der VM läuft über den Proxy von Docker Sandboxes, mit der Richtlinie, die Atollo setzt (der von Docker für die Entwicklung empfohlenen: KI-Dienste und Paket-Registries).",
          )}
        </P>
      </Section>

      <Section
        title={tr(
          "Che cosa serve, sistema per sistema",
          "What it needs, system by system",
          "Ce qu'il faut, système par système",
          "Was nötig ist, System für System",
        )}
      >
        <Table
          columns={[
            tr("Sistema", "System", "Système", "System"),
            tr("Che cosa serve", "What it needs", "Ce qu'il faut", "Was nötig ist"),
            tr("Senza VM", "No VM", "Sans VM", "Ohne VM"),
          ]}
          rows={[
            [
              "Windows",
              tr(
                "Windows 11 a 64 bit (processore Intel o AMD) con Windows Hypervisor Platform attivo: si attiva una volta, da amministratore, poi si riavvia il PC.",
                "64-bit Windows 11 (Intel or AMD processor) with Windows Hypervisor Platform on: it is turned on once, as an administrator, then the PC is restarted.",
                "Windows 11 64 bits (processeur Intel ou AMD) avec Windows Hypervisor Platform activé : il s'active une fois, en administrateur, puis on redémarre le PC.",
                "Windows 11 mit 64 Bit (Intel- oder AMD-Prozessor) mit aktivierter Windows Hypervisor Platform: Sie wird einmal als Administrator eingeschaltet, dann wird der PC neu gestartet.",
              ),
              tr("Windows 10.", "Windows 10.", "Windows 10.", "Windows 10."),
            ],
            [
              "macOS",
              tr(
                "Un Mac con Apple silicon (M1 o successivo) e macOS 14 Sonoma o successivo.",
                "A Mac with Apple silicon (M1 or later) and macOS 14 Sonoma or later.",
                "Un Mac avec Apple silicon (M1 ou ultérieur) et macOS 14 Sonoma ou ultérieur.",
                "Ein Mac mit Apple Silicon (M1 oder neuer) und macOS 14 Sonoma oder neuer.",
              ),
              tr(
                "Le versioni di macOS precedenti alla 14.",
                "macOS versions before 14.",
                "Les versions de macOS antérieures à la 14.",
                "macOS-Versionen vor 14.",
              ),
            ],
            [
              "Linux",
              tr(
                "Ubuntu 24.04 o successivo; /dev/kvm presente (virtualizzazione accesa nel BIOS/UEFI) e il tuo utente nel gruppo kvm; dove Ubuntu limita i namespace utente, un profilo AppArmor per la microVM del kit.",
                "Ubuntu 24.04 or later; /dev/kvm present (virtualization on in the BIOS/UEFI) and your user in the kvm group; where Ubuntu restricts user namespaces, an AppArmor profile for the kit's microVM.",
                "Ubuntu 24.04 ou ultérieur ; /dev/kvm présent (virtualisation activée dans le BIOS/UEFI) et votre utilisateur dans le groupe kvm ; là où Ubuntu restreint les espaces de noms utilisateur, un profil AppArmor pour la microVM du kit.",
                "Ubuntu 24.04 oder neuer; /dev/kvm vorhanden (Virtualisierung im BIOS/UEFI eingeschaltet) und dein Benutzer in der Gruppe kvm; wo Ubuntu die Benutzer-Namespaces einschränkt, ein AppArmor-Profil für die microVM des Kits.",
              ),
              tr(
                "Le altre distribuzioni, comprese le derivate di Ubuntu.",
                "The other distributions, Ubuntu's derivatives included.",
                "Les autres distributions, y compris les dérivées d'Ubuntu.",
                "Die anderen Distributionen, Ableger von Ubuntu eingeschlossen.",
              ),
            ],
            [
              tr("Ogni sistema", "Every system", "Tous les systèmes", "Jedes System"),
              tr(
                "Docker Sandboxes nel kit (circa 470 MB in data, installato con i componenti obbligatori), la virtualizzazione accesa nel firmware (BIOS/UEFI; dentro una macchina virtuale, la virtualizzazione annidata), un account Docker gratuito (sbx login) e il kit su un disco NTFS, APFS o ext4.",
                "Docker Sandboxes in the kit (about 470 MB in data, installed with the required components), virtualization on in the firmware (BIOS/UEFI; inside a virtual machine, nested virtualization), a free Docker account (sbx login) and the kit on an NTFS, APFS or ext4 disk.",
                "Docker Sandboxes dans le kit (environ 470 Mo dans data, installé avec les composants obligatoires), la virtualisation activée dans le firmware (BIOS/UEFI ; dans une machine virtuelle, la virtualisation imbriquée), un compte Docker gratuit (sbx login) et le kit sur un disque NTFS, APFS ou ext4.",
                "Docker Sandboxes im Kit (etwa 470 MB in data, mit den Pflichtkomponenten installiert), die in der Firmware eingeschaltete Virtualisierung (BIOS/UEFI; in einer virtuellen Maschine die verschachtelte Virtualisierung), ein kostenloses Docker-Konto (sbx login) und das Kit auf einem NTFS-, APFS- oder ext4-Laufwerk.",
              ),
              tr(
                "Il kit su FAT32 o exFAT: i dischi delle VM non ci stanno o lo riempirebbero. La virtualizzazione spenta nel firmware, o una macchina virtuale senza virtualizzazione annidata.",
                "The kit on FAT32 or exFAT: the VMs' disks do not fit or would fill it. Virtualization off in the firmware, or a virtual machine without nested virtualization.",
                "Le kit sur FAT32 ou exFAT : les disques des VM n'y tiennent pas ou le rempliraient. La virtualisation désactivée dans le firmware, ou une machine virtuelle sans virtualisation imbriquée.",
                "Das Kit auf FAT32 oder exFAT: Die Festplatten der VMs passen nicht darauf oder würden es füllen. Die Virtualisierung in der Firmware ausgeschaltet, oder eine virtuelle Maschine ohne verschachtelte Virtualisierung.",
              ),
            ],
          ]}
        />
        <Callout
          tone="info"
          title={tr(
            "Niente Docker Desktop, niente amministratore",
            "No Docker Desktop, no administrator",
            "Pas de Docker Desktop, pas d'administrateur",
            "Kein Docker Desktop, kein Administrator",
          )}
        >
          {tr(
            "Docker Desktop non serve. Atollo non chiede mai diritti di amministratore: i passi che li richiedono (Hypervisor Platform, gruppo kvm, profilo AppArmor) li fai tu, una volta, con il comando che la finestra dell'avviso ti fa copiare.",
            "Docker Desktop is not needed. Atollo never asks for administrator rights: the steps that need them (Hypervisor Platform, kvm group, AppArmor profile) are yours to do, once, with the command the notice's window lets you copy.",
            "Docker Desktop n'est pas nécessaire. Atollo ne demande jamais de droits d'administrateur : les étapes qui en ont besoin (Hypervisor Platform, groupe kvm, profil AppArmor), c'est vous qui les faites, une fois, avec la commande que la fenêtre de l'avis vous fait copier.",
            "Docker Desktop wird nicht gebraucht. Atollo verlangt nie Administratorrechte: Die Schritte, die sie brauchen (Hypervisor Platform, Gruppe kvm, AppArmor-Profil), machst du selbst, einmal, mit dem Befehl, den dich das Fenster des Hinweises kopieren lässt.",
          )}
        </Callout>
        <Table
          columns={[tr("Chi", "Who", "Qui", "Wer"), tr("Che cosa", "What", "Quoi", "Was")]}
          rows={[
            [
              tr("Atollo, da solo", "Atollo, by itself", "Atollo, tout seul", "Atollo, von selbst"),
              tr(
                "Installa Docker Sandboxes nel kit (e se non riesce riprova ogni ora), avvia il suo demone, imposta la politica di rete e le impostazioni del demone, toglie le giunzioni che portano fuori dal kit, rifà le sandbox quando il kit cambia cartella.",
                "Installs Docker Sandboxes in the kit (and if it fails tries again every hour), starts its daemon, sets the network policy and the daemon's settings, removes the junctions that lead out of the kit, makes the sandboxes again when the kit moves to another folder.",
                "Installe Docker Sandboxes dans le kit (et s'il échoue, réessaie toutes les heures), démarre son démon, définit la politique réseau et les réglages du démon, retire les jonctions qui mènent hors du kit, refait les sandbox quand le kit change de dossier.",
                "Installiert Docker Sandboxes im Kit (und versucht es bei einem Fehlschlag jede Stunde erneut), startet seinen Daemon, setzt die Netzwerkrichtlinie und die Einstellungen des Daemons, entfernt die Verknüpfungen, die aus dem Kit hinausführen, legt die Sandboxes neu an, wenn das Kit den Ordner wechselt.",
              ),
            ],
            [
              tr("Tu, una volta", "You, once", "Vous, une fois", "Du, einmal"),
              tr(
                "sbx login con un account Docker gratuito, senza amministratore («Lancia» nella finestra dell'avviso).",
                "sbx login with a free Docker account, without an administrator (“Run” in the notice's window).",
                "sbx login avec un compte Docker gratuit, sans administrateur (« Lancer » dans la fenêtre de l'avis).",
                "sbx login mit einem kostenlosen Docker-Konto, ohne Administrator („Starten“ im Fenster des Hinweises).",
              ),
            ],
            [
              tr(
                "Tu, con un amministratore",
                "You, with an administrator",
                "Vous, avec un administrateur",
                "Du, mit einem Administrator",
              ),
              tr(
                "Windows Hypervisor Platform (poi un riavvio), il gruppo kvm (poi un nuovo accesso alla sessione), il profilo AppArmor sugli Ubuntu recenti. La finestra mostra il comando esatto con «Copia»: Atollo non lo lancia mai.",
                "Windows Hypervisor Platform (then a restart), the kvm group (then a new login to the session), the AppArmor profile on recent Ubuntu. The window shows the exact command with “Copy”: Atollo never runs it.",
                "Windows Hypervisor Platform (puis un redémarrage), le groupe kvm (puis une nouvelle ouverture de session), le profil AppArmor sur les Ubuntu récents. La fenêtre affiche la commande exacte avec « Copier » : Atollo ne la lance jamais.",
                "Windows Hypervisor Platform (dann ein Neustart), die Gruppe kvm (dann eine neue Anmeldung an der Sitzung), das AppArmor-Profil auf neueren Ubuntu-Versionen. Das Fenster zeigt den genauen Befehl mit „Kopieren“: Atollo führt ihn nie aus.",
              ),
            ],
            [
              tr("Nessuno", "No one", "Personne", "Niemand"),
              tr(
                "Windows 10, macOS prima del 14, Linux diverso da Ubuntu 24.04 o successivo, la virtualizzazione spenta nel firmware o una macchina virtuale senza virtualizzazione annidata, il kit su FAT32 o exFAT: lì la VM non può esistere, e i progetti lavorano «Senza VM».",
                "Windows 10, macOS before 14, Linux other than Ubuntu 24.04 or later, virtualization off in the firmware or a virtual machine without nested virtualization, the kit on FAT32 or exFAT: there the VM cannot exist, and the projects work with “No VM”.",
                "Windows 10, macOS avant la 14, Linux autre qu'Ubuntu 24.04 ou ultérieur, la virtualisation désactivée dans le firmware ou une machine virtuelle sans virtualisation imbriquée, le kit sur FAT32 ou exFAT : là, la VM ne peut pas exister, et les projets travaillent « Sans VM ».",
                "Windows 10, macOS vor 14, Linux außer Ubuntu 24.04 oder neuer, die Virtualisierung in der Firmware ausgeschaltet oder eine virtuelle Maschine ohne verschachtelte Virtualisierung, das Kit auf FAT32 oder exFAT: Dort kann die VM nicht existieren, und die Projekte arbeiten „Ohne VM“.",
              ),
            ],
          ]}
        />
      </Section>

      <Section title={tr("Il primo accesso", "The first sign-in", "La première connexion", "Die erste Anmeldung")}>
        <Steps
          steps={[
            tr(
              "Al primo avvio «Installa» scarica anche Docker Sandboxes, con gli altri componenti obbligatori; l'avanzamento è nella barra di stato.",
              "At the first start “Install” also downloads Docker Sandboxes, with the other required components; the progress is in the status bar.",
              "Au premier lancement, « Installer » télécharge aussi Docker Sandboxes, avec les autres composants obligatoires ; la progression est dans la barre d'état.",
              "Beim ersten Start lädt „Installieren“ auch Docker Sandboxes herunter, mit den anderen Pflichtkomponenten; der Fortschritt steht in der Statusleiste.",
            ),
            tr(
              "Finché non hai fatto l'accesso, la barra di stato mostra «Senza VM». Fai clic: nella finestra «Lavori senza VM», sotto «Perché la VM non è attiva», c'è il motivo «sbx non ha accesso a Docker» con il comando sbx login.",
              "Until you sign in, the status bar shows “No VM”. Click it: in the window “You are working without a VM”, under “Why the VM is not on”, there is the reason “sbx is not signed in to Docker” with the command sbx login.",
              "Tant que vous n'êtes pas connecté, la barre d'état affiche « Sans VM ». Cliquez dessus : dans la fenêtre « Vous travaillez sans VM », sous « Pourquoi la VM n'est pas active », il y a le motif « sbx n'est pas connecté à Docker » avec la commande sbx login.",
              "Solange du nicht angemeldet bist, zeigt die Statusleiste „Ohne VM“. Klicke darauf: Im Fenster „Du arbeitest ohne VM“ steht unter „Warum die VM nicht aktiv ist“ der Grund „sbx ist nicht bei Docker angemeldet“ mit dem Befehl sbx login.",
            ),
            tr(
              "Premi «Lancia»: Atollo apre sbx login in una scheda del suo terminale, con il Docker Sandboxes del kit. Accedi a Docker nel browser (o crea un account gratuito); un indirizzo che compare nel terminale si apre con un clic.",
              "Press “Run”: Atollo opens sbx login in a tab of its terminal, with the kit's Docker Sandboxes. Sign in to Docker in the browser (or create a free account); an address that appears in the terminal opens with a click.",
              "Appuyez sur « Lancer » : Atollo ouvre sbx login dans un onglet de son terminal, avec le Docker Sandboxes du kit. Connectez-vous à Docker dans le navigateur (ou créez un compte gratuit) ; une adresse qui apparaît dans le terminal s'ouvre d'un clic.",
              "Drücke „Starten“: Atollo öffnet sbx login in einem Tab seines Terminals, mit dem Docker Sandboxes des Kits. Melde dich im Browser bei Docker an (oder lege ein kostenloses Konto an); eine Adresse, die im Terminal erscheint, öffnet sich mit einem Klick.",
            ),
            tr(
              "Torna nella finestra dell'avviso e premi «Ricontrolla». Quando non resta nessun motivo, la finestra diventa «Progetti nella VM» e la barra di stato mostra «VM».",
              "Go back to the notice's window and press “Check again”. When no reason is left, the window becomes “Projects in the VM” and the status bar shows “VM”.",
              "Revenez à la fenêtre de l'avis et appuyez sur « Vérifier à nouveau ». Quand il ne reste aucun motif, la fenêtre devient « Projets dans la VM » et la barre d'état affiche « VM ».",
              "Geh zurück zum Fenster des Hinweises und drücke „Erneut prüfen“. Wenn kein Grund mehr übrig ist, wird das Fenster zu „Projekte in der VM“ und die Statusleiste zeigt „VM“.",
            ),
          ]}
        />
        <Callout tone="tip">
          {tr(
            "L'accesso a Docker vale per questo PC: Docker Sandboxes lo tiene nel portachiavi del sistema, non nel kit. Su un altro PC rifai sbx login, una volta.",
            "The Docker sign-in is for this PC: Docker Sandboxes keeps it in the system keychain, not in the kit. On another PC run sbx login again, once.",
            "La connexion à Docker vaut pour ce PC : Docker Sandboxes la garde dans le trousseau du système, pas dans le kit. Sur un autre PC, refaites sbx login, une fois.",
            "Die Anmeldung bei Docker gilt für diesen PC: Docker Sandboxes bewahrt sie im Schlüsselbund des Systems auf, nicht im Kit. Auf einem anderen PC führst du sbx login noch einmal aus, einmalig.",
          )}
        </Callout>
      </Section>

      <Section
        title={tr(
          "Che cosa gira dove: due posti, tre livelli",
          "What runs where: two places, three levels",
          "Ce qui tourne où : deux endroits, trois niveaux",
          "Was wo läuft: zwei Orte, drei Stufen",
        )}
      >
        <P>
          {tr(
            "Atollo dà a ogni programma un livello, secondo chi lo avvia, e il livello decide dove gira. Non si sceglie e non si cambia: lo stesso programma va sempre nello stesso posto. I posti sono due: il PC per i livelli 1 e 2, con o senza la VM; la VM solo per il livello 3, gli agenti.",
            "Atollo gives every program a level, by who starts it, and the level decides where it runs. You do not choose it and cannot change it: the same program always goes to the same place. There are two places: the PC for levels 1 and 2, with or without the VM; the VM only for level 3, the agents.",
            "Atollo donne à chaque programme un niveau, selon qui le lance, et le niveau décide où il tourne. On ne le choisit pas et on ne le change pas : le même programme va toujours au même endroit. Les endroits sont deux : le PC pour les niveaux 1 et 2, avec ou sans la VM ; la VM seulement pour le niveau 3, les agents.",
            "Atollo gibt jedem Programm eine Stufe, je nachdem, wer es startet, und die Stufe entscheidet, wo es läuft. Man wählt sie nicht und ändert sie nicht: Dasselbe Programm landet immer am selben Ort. Es gibt zwei Orte: den PC für die Stufen 1 und 2, mit oder ohne VM; die VM nur für Stufe 3, die Agenten.",
          )}
        </P>
        <Table
          columns={[
            tr("Livello", "Level", "Niveau", "Stufe"),
            tr("Che cosa", "What", "Quoi", "Was"),
            tr("Con la VM", "With the VM", "Avec la VM", "Mit VM"),
            tr("Senza VM", "No VM", "Sans VM", "Ohne VM"),
          ]}
          rows={[
            [
              tr("1 · Manuale", "1 · Manual", "1 · Manuel", "1 · Manuell"),
              tr(
                "I comandi che scrivi tu: i terminali del kit («Tutti gli strumenti» e i tuoi) e il terminale integrato dell'editor.",
                "The commands you type: the kit's terminals (“All tools” and your own) and the editor's integrated terminal.",
                "Les commandes que vous tapez : les terminaux du kit (« Tous les outils » et les vôtres) et le terminal intégré de l'éditeur.",
                "Die Befehle, die du tippst: die Terminals des Kits („Alle Werkzeuge“ und deine eigenen) und das integrierte Terminal des Editors.",
              ),
              tr(
                "Sul PC, con gli strumenti del kit (alle versioni del progetto nella sua cartella) e il tuo agente SSH per Git; claude, codex, pi e agy passano l'agente alla VM del progetto.",
                "On the PC, with the kit's tools (at the project's versions in its folder) and your SSH agent for Git; claude, codex, pi and agy hand the agent over to the project's VM.",
                "Sur le PC, avec les outils du kit (aux versions du projet dans son dossier) et votre agent SSH pour Git ; claude, codex, pi et agy confient l'agent à la VM du projet.",
                "Auf dem PC, mit den Werkzeugen des Kits (in den Versionen des Projekts in seinem Ordner) und deinem SSH-Agenten für Git; claude, codex, pi und agy übergeben den Agenten an die VM des Projekts.",
              ),
              tr(
                "Sul PC, uguale; claude, codex, pi e agy dicono perché non partono.",
                "On the PC, the same; claude, codex, pi and agy say why they do not start.",
                "Sur le PC, pareil ; claude, codex, pi et agy disent pourquoi ils ne démarrent pas.",
                "Auf dem PC, genauso; claude, codex, pi und agy sagen, warum sie nicht starten.",
              ),
            ],
            [
              tr("2 · Codice del progetto", "2 · Project code", "2 · Code du projet", "2 · Code des Projekts"),
              tr(
                "Il codice che arriva con un progetto: il terminale del progetto, le azioni (npm run dev…), i siti, i pool di Server Web, i task dell'editor, la creazione di un progetto da una base.",
                "The code that comes with a project: the project's terminal, the actions (npm run dev…), the websites, the Web Servers pools, the editor's tasks, creating a project from a base.",
                "Le code qui arrive avec un projet : le terminal du projet, les actions (npm run dev…), les sites, les pools de Serveurs web, les tâches de l'éditeur, la création d'un projet depuis une base.",
                "Der Code, der mit einem Projekt kommt: das Terminal des Projekts, die Aktionen (npm run dev…), die Websites, die Webserver-Pools, die Tasks des Editors, das Anlegen eines Projekts aus einer Basis.",
              ),
              tr(
                "Sul PC, come senza VM, con l'ambiente sigillato del kit.",
                "On the PC, as without the VM, with the kit's sealed environment.",
                "Sur le PC, comme sans VM, avec l'environnement scellé du kit.",
                "Auf dem PC, wie ohne VM, mit der versiegelten Umgebung des Kits.",
              ),
              tr(
                "Sul PC, con l'ambiente sigillato del kit e «Senza VM» nella barra di stato.",
                "On the PC, with the kit's sealed environment and “No VM” in the status bar.",
                "Sur le PC, avec l'environnement scellé du kit et « Sans VM » dans la barre d'état.",
                "Auf dem PC, mit der versiegelten Umgebung des Kits und „Ohne VM“ in der Statusleiste.",
              ),
            ],
            [
              tr("3 · Agenti", "3 · Agents", "3 · Agents", "3 · Agenten"),
              tr(
                "Programmi che decidono da soli che cosa eseguire: Claude Code, Codex, Pi, Antigravity, e le estensioni dell'editor che eseguono un agente.",
                "Programs that decide by themselves what to run: Claude Code, Codex, Pi, Antigravity, and the editor's extensions that run an agent.",
                "Des programmes qui décident seuls quoi exécuter : Claude Code, Codex, Pi, Antigravity, et les extensions de l'éditeur qui exécutent un agent.",
                "Programme, die selbst entscheiden, was sie ausführen: Claude Code, Codex, Pi, Antigravity und die Erweiterungen des Editors, die einen Agenten ausführen.",
              ),
              tr(
                "Solo nella VM: tutti gli agenti nella sandbox del progetto (atollo-p-…), con le loro build per Linux.",
                "Only in the VM: every agent in the project's sandbox (atollo-p-…), with their builds for Linux.",
                "Seulement dans la VM : tous les agents dans la sandbox du projet (atollo-p-…), avec leurs builds pour Linux.",
                "Nur in der VM: alle Agenten in der Sandbox des Projekts (atollo-p-…), mit ihren Builds für Linux.",
              ),
              tr(
                "Da nessuna parte: pulsanti, voci della tavolozza, profili dell'editor e installazioni sono disattivati, con il motivo.",
                "Nowhere: buttons, palette entries, editor profiles and installs are disabled, with the reason.",
                "Nulle part : boutons, entrées de la palette, profils de l'éditeur et installations sont désactivés, avec le motif.",
                "Nirgends: Schaltflächen, Einträge der Befehlspalette, Profile des Editors und Installationen sind deaktiviert, mit dem Grund.",
              ),
            ],
          ]}
        />
        <P>
          {tr(
            "Gli strumenti scelti nel kit si installano sempre sul PC; quelli che servono agli agenti anche nella loro build per Linux, alle versioni scelte dal progetto: se ne manca uno, Atollo lo installa in data\\apps\\linux-x64 (linux-arm64 sui Mac con Apple silicon) prima di aprire l'agente, con l'avanzamento nella barra di stato. Con la VM attiva la pagina Strumenti li installa per il PC e, nella stessa passata, per il Linux della VM; nella matrice la colonna del Linux porta il segno «VM». Gli strumenti per Linux si scaricano una volta nel kit, con lo SHA-256 verificato, e la VM li monta in sola lettura con il PATH alle versioni del progetto: viaggiano con il kit, funzionano senza rete, la VM parte subito con le stesse versioni verificate, non può cambiarli e non ha bisogno dei siti di download. I pacchetti del progetto (node_modules, vendor, .venv) li installa il progetto, nella sua cartella.",
            "The tools chosen in the kit are always installed on the PC; those the agents need also in their build for Linux, at the versions the project chose: when one is missing, Atollo installs it in data\\apps\\linux-x64 (linux-arm64 on Macs with Apple silicon) before opening the agent, with the progress in the status bar. With the VM on, the Tools page installs them for the PC and, in the same pass, for the VM's Linux; in the matrix the Linux column bears the “VM” mark. The tools for Linux are downloaded once into the kit, with the SHA-256 verified, and the VM mounts them read-only with the PATH set to the project's versions: they travel with the kit, work offline, the VM starts at once with the same verified versions, cannot change them and needs no download sites. The project's own packages (node_modules, vendor, .venv) are installed by the project, in its folder.",
            "Les outils choisis dans le kit s'installent toujours sur le PC ; ceux dont les agents ont besoin aussi dans leur build pour Linux, aux versions choisies par le projet : s'il en manque un, Atollo l'installe dans data\\apps\\linux-x64 (linux-arm64 sur les Mac avec Apple silicon) avant d'ouvrir l'agent, avec la progression dans la barre d'état. Avec la VM active, la page Outils les installe pour le PC et, dans le même passage, pour le Linux de la VM ; dans la matrice, la colonne de Linux porte la marque « VM ». Les outils pour Linux se téléchargent une fois dans le kit, avec le SHA-256 vérifié, et la VM les monte en lecture seule avec le PATH aux versions du projet : ils voyagent avec le kit, fonctionnent sans réseau, la VM démarre tout de suite avec les mêmes versions vérifiées, ne peut pas les modifier et n'a pas besoin des sites de téléchargement. Les paquets du projet (node_modules, vendor, .venv), c'est le projet qui les installe, dans son dossier.",
            "Die im Kit gewählten Werkzeuge werden immer auf dem PC installiert; die, die die Agenten brauchen, auch in ihrem Build für Linux, in den vom Projekt gewählten Versionen: Fehlt eines, installiert Atollo es in data\\apps\\linux-x64 (linux-arm64 auf Macs mit Apple Silicon), bevor der Agent geöffnet wird, mit dem Fortschritt in der Statusleiste. Mit aktiver VM installiert die Seite Werkzeuge sie für den PC und im selben Durchgang für das Linux der VM; in der Matrix trägt die Linux-Spalte das Zeichen „VM“. Die Werkzeuge für Linux werden einmal ins Kit heruntergeladen, mit geprüfter SHA-256, und die VM bindet sie schreibgeschützt ein, mit dem PATH auf die Versionen des Projekts: Sie reisen mit dem Kit, funktionieren ohne Netzwerk, die VM startet sofort mit denselben geprüften Versionen, kann sie nicht verändern und braucht keine Download-Seiten. Die Pakete des Projekts (node_modules, vendor, .venv) installiert das Projekt selbst, in seinem Ordner.",
          )}
        </P>
        <P>
          {tr(
            "Il database resta sul PC, nel server del kit: la VM ne raggiunge solo la porta, come l'utente del progetto (tutti i diritti sul suo database, nessuno sugli altri). Ai programmi della VM la connessione arriva con le stesse variabili del file .env (DB_HOST e le altre di Laravel, oppure DATABASE_URL o MONGODB_URI) e l'indirizzo host.docker.internal; il file .env del progetto non cambia. La password di amministratore dei database non entra mai nella VM.",
            "The database stays on the PC, in the kit's server: the VM reaches only its port, as the project's user (every right on its database, none on the others). The VM's programs get the connection through the same variables as the .env file (DB_HOST and Laravel's others, or DATABASE_URL or MONGODB_URI) and the address host.docker.internal; the project's .env file does not change. The databases' administrator password never enters the VM.",
            "La base de données reste sur le PC, dans le serveur du kit : la VM n'en atteint que le port, en tant qu'utilisateur du projet (tous les droits sur sa base, aucun sur les autres). Les programmes de la VM reçoivent la connexion par les mêmes variables que le fichier .env (DB_HOST et les autres de Laravel, ou DATABASE_URL ou MONGODB_URI) et l'adresse host.docker.internal ; le fichier .env du projet ne change pas. Le mot de passe d'administrateur des bases de données n'entre jamais dans la VM.",
            "Die Datenbank bleibt auf dem PC, im Server des Kits: Die VM erreicht nur ihren Port, als Benutzer des Projekts (alle Rechte an seiner Datenbank, keine an den anderen). Die Programme der VM erhalten die Verbindung über dieselben Variablen wie die Datei .env (DB_HOST und die anderen von Laravel, oder DATABASE_URL oder MONGODB_URI) und die Adresse host.docker.internal; die Datei .env des Projekts ändert sich nicht. Das Administratorpasswort der Datenbanken gelangt nie in die VM.",
          )}
        </P>
        <Bullets
          items={[
            tr(
              "Claude Code, Codex, Pi e Antigravity partono nella sandbox del progetto, ciascuno con la sua build per Linux del kit: tutti con gli strumenti del kit del progetto nel PATH e il suo database. L'accesso si fa nella VM, al primo avvio dell'agente; la chiave di Gemini delle impostazioni passa dal proxy di Docker Sandboxes, mai dentro la VM. Senza VM nessun agente parte, mai sul PC al posto della VM.",
              "Claude Code, Codex, Pi and Antigravity start in the project's sandbox, each with its kit's build for Linux: all with the project's kit tools on the PATH and its database. The sign-in happens in the VM, at the agent's first start; the Gemini key of the settings goes through the Docker Sandboxes proxy, never into the VM. Without the VM no agent starts, never on the PC in the VM's place.",
              "Claude Code, Codex, Pi et Antigravity démarrent dans la sandbox du projet, chacun avec son build pour Linux du kit : tous avec les outils du kit du projet dans le PATH et sa base de données. La connexion se fait dans la VM, au premier lancement de l'agent ; la clé Gemini des paramètres passe par le proxy de Docker Sandboxes, jamais dans la VM. Sans VM, aucun agent ne démarre, jamais sur le PC à la place de la VM.",
              "Claude Code, Codex, Pi und Antigravity starten in der Sandbox des Projekts, jeder mit seinem Build des Kits für Linux: alle mit den Werkzeugen des Kits für das Projekt im PATH und seiner Datenbank. Die Anmeldung erfolgt in der VM, beim ersten Start des Agenten; der Gemini-Schlüssel der Einstellungen geht über den Proxy von Docker Sandboxes, nie in die VM. Ohne VM startet kein Agent, nie stattdessen auf dem PC.",
            ),
            tr(
              "Un agente parte dalla pagina del progetto, dalla tavolozza dei comandi, dal suo profilo nel terminale dell'editor, oppure scrivendo claude, codex, pi o agy in un terminale del PC nella cartella del progetto: il comando chiede ad Atollo di avviarlo nella VM, nello stesso terminale. In un'altra cartella parte nella sandbox di quella cartella (D-187), mai nel kit, nel profilo, in una cartella di sistema o nella radice di un disco; senza VM dice perché ed esce.",
              "An agent starts from the project's page, from the command palette, from its profile in the editor's terminal, or by typing claude, codex, pi or agy in a terminal of the PC in the project's folder: the command asks Atollo to start it in the VM, in the same terminal. In another folder it starts in that folder's sandbox (D-187), never in the kit, the profile, a system folder or a drive's root; without the VM it says why and exits.",
              "Un agent démarre depuis la page du projet, depuis la palette de commandes, depuis son profil dans le terminal de l'éditeur, ou en tapant claude, codex, pi ou agy dans un terminal du PC dans le dossier du projet : la commande demande à Atollo de le lancer dans la VM, dans le même terminal. Dans un autre dossier, il démarre dans la sandbox de ce dossier (D-187), jamais dans le kit, le profil, un dossier système ou la racine d'un disque ; sans VM, elle dit pourquoi et s'arrête.",
              "Ein Agent startet von der Seite des Projekts, aus der Befehlspalette, über sein Profil im Terminal des Editors oder indem du claude, codex, pi oder agy in einem Terminal des PCs im Ordner des Projekts tippst: Der Befehl bittet Atollo, ihn in der VM zu starten, im selben Terminal. In einem anderen Ordner startet er in der Sandbox dieses Ordners (D-187), nie im Kit, im Profil, in einem Systemordner oder im Stammverzeichnis eines Laufwerks; ohne VM sagt er, warum, und beendet sich.",
            ),
            tr(
              "Nell'editor i terminali sono di livello 1 e si aprono sul PC; anche i task girano sul PC, sigillati, con o senza la VM; i profili degli agenti ci sono solo con la VM. Le estensioni che eseguono un agente (Claude Code, Codex, Cline…) non si installano: se ne trova una, Atollo la toglie all'avvio dell'editor e l'editor dice perché.",
              "In the editor the terminals are level 1 and open on the PC; the tasks run on the PC too, sealed, with or without the VM; the agents' profiles are there only with the VM. Extensions that run an agent (Claude Code, Codex, Cline…) are not installed: if it finds one, Atollo removes it at the editor's start and the editor says why.",
              "Dans l'éditeur, les terminaux sont de niveau 1 et s'ouvrent sur le PC ; les tâches tournent elles aussi sur le PC, scellées, avec ou sans la VM ; les profils des agents n'y sont qu'avec la VM. Les extensions qui exécutent un agent (Claude Code, Codex, Cline…) ne sont pas installées : s'il en trouve une, Atollo la retire au démarrage de l'éditeur et l'éditeur dit pourquoi.",
              "Im Editor sind die Terminals Stufe 1 und öffnen sich auf dem PC; auch die Tasks laufen auf dem PC, versiegelt, mit oder ohne VM; die Profile der Agenten gibt es nur mit der VM. Erweiterungen, die einen Agenten ausführen (Claude Code, Codex, Cline…), werden nicht installiert: Findet Atollo eine, entfernt es sie beim Start des Editors, und der Editor sagt, warum.",
            ),
            tr(
              "Ogni scheda del terminale dice dove gira: «PC» o «Agente · VM», con il nome della sandbox al passaggio del mouse. All'apertura una riga lo ripete: nei terminali del PC il livello, il terminale e il progetto; per un agente «[Atollo] Agente · VM: …».",
              "Every terminal tab says where it runs: “PC” or “Agent · VM”, with the sandbox's name on hover. On opening a line repeats it: in the PC's terminals the level, the terminal and the project; for an agent “[Atollo] Agente · VM: …”.",
              "Chaque onglet du terminal dit où il tourne : « PC » ou « Agent · VM », avec le nom de la sandbox au survol. À l'ouverture, une ligne le répète : dans les terminaux du PC le niveau, le terminal et le projet ; pour un agent « [Atollo] Agente · VM: … ».",
              "Jeder Tab des Terminals sagt, wo er läuft: „PC“ oder „Agent · VM“, mit dem Namen der Sandbox beim Überfahren mit der Maus. Beim Öffnen wiederholt es eine Zeile: in den Terminals des PCs die Stufe, das Terminal und das Projekt; für einen Agenten „[Atollo] Agente · VM: …“.",
            ),
          ]}
        />
      </Section>

      <Section
        title={tr(
          "Il pannello «VM del progetto»",
          "The “Project VM” panel",
          "Le panneau « VM du projet »",
          "Das Feld „VM des Projekts“",
        )}
      >
        <P>
          {tr(
            "Nella pagina del progetto, la sezione «VM del progetto» mostra la sandbox del progetto, dove girano tutti gli agenti: lo stato (accesa, spenta, non c'è), le cartelle montate (il progetto in lettura e scrittura, il resto in sola lettura), le porte del PC che raggiunge e gli agenti che ci girano; sotto, se GitHub è nella VM. Senza VM mostra il motivo e «Apri l'avviso VM».",
            "On the project's page, the “Project VM” section shows the project's sandbox, where every agent runs: its state (on, off, not there), the mounted folders (the project read-write, the rest read-only), the PC ports it reaches and the agents running in it; below, whether GitHub is in the VM. Without the VM it shows the reason and “Open the VM notice”.",
            "Sur la page du projet, la section « VM du projet » montre la sandbox du projet, où tournent tous les agents : son état (allumée, éteinte, absente), les dossiers montés (le projet en lecture et écriture, le reste en lecture seule), les ports du PC qu'elle atteint et les agents qui y tournent ; en dessous, si GitHub est dans la VM. Sans VM, elle montre le motif et « Ouvrir l'avis VM ».",
            "Auf der Seite des Projekts zeigt der Abschnitt „VM des Projekts“ die Sandbox des Projekts, in der alle Agenten laufen: ihren Zustand (an, aus, nicht vorhanden), die eingehängten Ordner (das Projekt mit Lese- und Schreibzugriff, der Rest schreibgeschützt), die Ports des PCs, die sie erreicht, und die Agenten, die darin laufen; darunter, ob GitHub in der VM ist. Ohne VM zeigt er den Grund und „VM-Hinweis öffnen“.",
          )}
        </P>
        <Table
          columns={[
            tr("Azione", "Action", "Action", "Aktion"),
            tr("Che cosa fa", "What it does", "Ce qu'elle fait", "Was sie tut"),
          ]}
          rows={[
            [
              tr("Verifica strumenti", "Check tools", "Vérifier les outils", "Werkzeuge prüfen"),
              tr(
                "Con la sandbox accesa, esegue nella VM il comando di versione di ogni strumento e lo confronta con la linea scelta: «✓ come scelto», «✗ atteso …, trovato …», «✗ manca», «✗ non è del kit», «non gira nella VM». Non accende mai una sandbox spenta.",
                "With the sandbox on, it runs each tool's version command in the VM and compares it with the chosen line: “✓ as chosen”, “✗ expected …, found …”, “✗ missing”, “✗ not the kit's”, “does not run in the VM”. It never turns on a sandbox that is off.",
                "Avec la sandbox allumée, elle exécute dans la VM la commande de version de chaque outil et la compare à la ligne choisie : « ✓ comme choisi », « ✗ attendu …, trouvé … », « ✗ manquant », « ✗ pas celui du kit », « ne tourne pas dans la VM ». Elle n'allume jamais une sandbox éteinte.",
                "Bei laufender Sandbox führt sie in der VM den Versionsbefehl jedes Werkzeugs aus und vergleicht ihn mit der gewählten Linie: „✓ wie gewählt“, „✗ erwartet …, gefunden …“, „✗ fehlt“, „✗ nicht vom Kit“, „läuft nicht in der VM“. Eine ausgeschaltete Sandbox schaltet sie nie ein.",
              ),
            ],
            [
              tr("Aggiorna strumenti", "Update tools", "Mettre à jour les outils", "Werkzeuge aktualisieren"),
              tr(
                "Prepara di nuovo la sandbox: gli strumenti per il Linux della VM alle linee del progetto, i database, le porte e le informazioni di atollo. Senza riavvio, se non serve.",
                "Prepares the sandbox again: the tools for the VM's Linux at the project's lines, the databases, the ports and atollo's information. Without a restart, unless one is needed.",
                "Prépare à nouveau la sandbox : les outils pour le Linux de la VM aux lignes du projet, les bases de données, les ports et les informations d'atollo. Sans redémarrage, sauf s'il le faut.",
                "Bereitet die Sandbox neu vor: die Werkzeuge für das Linux der VM in den Linien des Projekts, die Datenbanken, die Ports und die Angaben von atollo. Ohne Neustart, wenn keiner nötig ist.",
              ),
            ],
            [
              tr("Riavvia", "Restart", "Redémarrer", "Neu starten"),
              tr(
                "Ferma e riavvia la sandbox. Se ci girano agenti, prima li elenca e chiede: si fermano.",
                "Stops and starts the sandbox again. If agents run in it, it lists them first and asks: they stop.",
                "Arrête et redémarre la sandbox. Si des agents y tournent, elle les liste d'abord et demande : ils s'arrêtent.",
                "Stoppt die Sandbox und startet sie neu. Laufen darin Agenten, listet sie sie zuerst auf und fragt: Sie werden beendet.",
              ),
            ],
            [
              tr("Ricrea", "Make again", "Recréer", "Neu erstellen"),
              tr(
                "Toglie la sandbox e la rifà con gli strumenti attuali del kit. Chiede sempre conferma, dicendo che cosa si perde (la home della VM e le immagini che ha scaricato) e che cosa resta (il progetto, i suoi database, gli strumenti del kit e gli accessi che il kit conserva).",
                "Removes the sandbox and makes it again with the kit's current tools. It always asks first, saying what is lost (the VM's home and the images it downloaded) and what stays (the project, its databases, the kit's tools and the sign-ins the kit keeps).",
                "Retire la sandbox et la refait avec les outils actuels du kit. Elle demande toujours confirmation, en disant ce qui est perdu (le dossier personnel de la VM et les images qu'elle a téléchargées) et ce qui reste (le projet, ses bases de données, les outils du kit et les connexions que le kit conserve).",
                "Entfernt die Sandbox und erstellt sie mit den aktuellen Werkzeugen des Kits neu. Sie fragt immer zuerst und sagt, was verloren geht (das Home der VM und die Images, die sie heruntergeladen hat) und was bleibt (das Projekt, seine Datenbanken, die Werkzeuge des Kits und die Anmeldungen, die das Kit aufbewahrt).",
              ),
            ],
          ]}
        />
        <P>
          {tr(
            "Le stesse cose si fanno dal PC con la riga di comando, ad Atollo chiuso: atollo vm <progetto> mostra il rapporto, con --check verifica gli strumenti, con --restart, --refresh o --recreate agisce (--recreate chiede [s/N], salvo --yes), con --agent claude o codex sulla sandbox di quell'agente. Con Atollo aperto le azioni si fanno dal pannello.",
            "The same can be done from the PC with the command line, with Atollo closed: atollo vm <project> shows the report, with --check it checks the tools, with --restart, --refresh or --recreate it acts (--recreate asks [s/N], unless --yes), with --agent claude or codex on that agent's sandbox. With Atollo open the actions are done from the panel.",
            "On peut faire de même depuis le PC en ligne de commande, Atollo fermé : atollo vm <projet> montre le rapport, avec --check il vérifie les outils, avec --restart, --refresh ou --recreate il agit (--recreate demande [s/N], sauf --yes), avec --agent claude ou codex sur la sandbox de cet agent. Avec Atollo ouvert, les actions se font depuis le panneau.",
            "Dasselbe geht vom PC aus über die Befehlszeile, bei geschlossenem Atollo: atollo vm <Projekt> zeigt den Bericht, mit --check prüft es die Werkzeuge, mit --restart, --refresh oder --recreate handelt es (--recreate fragt [s/N], außer mit --yes), mit --agent claude oder codex auf der Sandbox dieses Agenten. Bei geöffnetem Atollo laufen die Aktionen über das Feld.",
          )}
        </P>
      </Section>

      <Section title={tr("atollo dentro la VM", "atollo inside the VM", "atollo dans la VM", "atollo in der VM")}>
        <P>
          {tr(
            "In ogni sandbox c'è un comando atollo che dice dove sei e che cosa hai, e non fa altro: atollo stampa il livello, la sandbox, il progetto, gli strumenti con le loro linee, i database con l'indirizzo host.docker.internal e l'utente (mai una password) e le porte del PC aperte per la sandbox; atollo --json lo stesso in JSON. Qualsiasi altra parola risponde che il kit si gestisce dal PC: dalla VM non si installa, non si toglie e non si legge nessun segreto del kit.",
            "In every sandbox there is an atollo command that tells where you are and what you have, and does nothing else: atollo prints the level, the sandbox, the project, the tools with their lines, the databases with the address host.docker.internal and the user (never a password) and the PC ports opened for the sandbox; atollo --json the same as JSON. Any other word answers that the kit is managed from the PC: from the VM nothing is installed or removed and no secret of the kit is read.",
            "Dans chaque sandbox il y a une commande atollo qui dit où vous êtes et ce que vous avez, et ne fait rien d'autre : atollo affiche le niveau, la sandbox, le projet, les outils avec leurs lignes, les bases de données avec l'adresse host.docker.internal et l'utilisateur (jamais un mot de passe) et les ports du PC ouverts pour la sandbox ; atollo --json la même chose en JSON. Tout autre mot répond que le kit se gère depuis le PC : depuis la VM on n'installe rien, on ne retire rien et on ne lit aucun secret du kit.",
            "In jeder Sandbox gibt es einen Befehl atollo, der sagt, wo du bist und was du hast, und sonst nichts tut: atollo gibt die Stufe, die Sandbox, das Projekt, die Werkzeuge mit ihren Linien, die Datenbanken mit der Adresse host.docker.internal und dem Benutzer (nie ein Passwort) und die für die Sandbox geöffneten Ports des PCs aus; atollo --json dasselbe als JSON. Jedes andere Wort antwortet, dass das Kit vom PC aus verwaltet wird: Aus der VM wird nichts installiert oder entfernt und kein Geheimnis des Kits gelesen.",
          )}
        </P>
        <P>
          {tr(
            "Una VM non può chiedere al PC di aprire la sandbox di un agente: dentro la VM claude, codex, pi e agy dicono da dove avviare gli agenti (la pagina del progetto, l'editor, o il loro nome scritto in un terminale del PC nella cartella del progetto).",
            "A VM cannot ask the PC to open an agent's sandbox: inside the VM claude, codex, pi and agy say where to start the agents from (the project's page, the editor, or their name typed in a terminal of the PC in the project's folder).",
            "Une VM ne peut pas demander au PC d'ouvrir la sandbox d'un agent : dans la VM, claude, codex, pi et agy disent d'où lancer les agents (la page du projet, l'éditeur, ou leur nom tapé dans un terminal du PC dans le dossier du projet).",
            "Eine VM kann den PC nicht bitten, die Sandbox eines Agenten zu öffnen: In der VM sagen claude, codex, pi und agy, von wo die Agenten zu starten sind (die Seite des Projekts, der Editor oder ihr Name, in einem Terminal des PCs im Ordner des Projekts getippt).",
          )}
        </P>
      </Section>

      <Section
        title={tr(
          "L'avviso «Senza VM» e come risolverlo",
          "The “No VM” notice and how to fix it",
          "L'avis « Sans VM » et comment le résoudre",
          "Der Hinweis „Ohne VM“ und wie man ihn behebt",
        )}
      >
        <P>
          {tr(
            "«Senza VM» sta nella barra di stato, sempre in vista: arancione quando puoi rimediare, grigio quando questo PC non può avere la VM; il nome del pulsante e il suggerimento lo dicono a parole. Un clic apre la finestra «Lavori senza VM»: che cosa vuol dire e, sotto «Perché la VM non è attiva», un motivo per riga, ognuno con che cosa fare e, quando c'è, il comando.",
            "“No VM” is in the status bar, always in sight: orange when you can fix it, grey when this PC cannot have the VM; the button's name and its tooltip say so in words. A click opens the window “You are working without a VM”: what it means and, under “Why the VM is not on”, one reason per row, each with what to do and, when there is one, the command.",
            "« Sans VM » est dans la barre d'état, toujours visible : en orange quand vous pouvez y remédier, en gris quand ce PC ne peut pas avoir la VM ; le nom du bouton et son infobulle le disent en mots. Un clic ouvre la fenêtre « Vous travaillez sans VM » : ce que cela veut dire et, sous « Pourquoi la VM n'est pas active », un motif par ligne, chacun avec quoi faire et, quand il y en a une, la commande.",
            "„Ohne VM“ steht in der Statusleiste, immer sichtbar: orange, wenn du es beheben kannst, grau, wenn dieser PC die VM nicht haben kann; Name und Tooltip der Schaltfläche sagen es in Worten. Ein Klick öffnet das Fenster „Du arbeitest ohne VM“: was das bedeutet und, unter „Warum die VM nicht aktiv ist“, ein Grund pro Zeile, jeder mit dem, was zu tun ist, und, wenn es einen gibt, dem Befehl.",
          )}
        </P>
        <Table
          columns={[
            tr("Motivo", "Reason", "Motif", "Grund"),
            tr("Che cosa fare", "What to do", "Quoi faire", "Was zu tun ist"),
          ]}
          rows={[
            [
              tr(
                "Sistema non supportato (Windows prima dell'11, macOS prima del 14, Linux diverso da Ubuntu 24.04 o successivo)",
                "System not supported (Windows before 11, macOS before 14, Linux other than Ubuntu 24.04 or later)",
                "Système non pris en charge (Windows avant la 11, macOS avant la 14, Linux autre qu'Ubuntu 24.04 ou ultérieur)",
                "System nicht unterstützt (Windows vor 11, macOS vor 14, Linux außer Ubuntu 24.04 oder neuer)",
              ),
              tr(
                "Aggiorna il sistema, se si può; altrimenti i progetti restano sul PC. È l'unico motivo mostrato: gli altri non servirebbero.",
                "Update the system if you can; otherwise the projects stay on the PC. It is the only reason shown: the others would not help.",
                "Mettez le système à jour si possible ; sinon, les projets restent sur le PC. C'est le seul motif affiché : les autres ne serviraient à rien.",
                "Aktualisiere das System, wenn möglich; sonst bleiben die Projekte auf dem PC. Es ist der einzige angezeigte Grund: Die anderen würden nicht helfen.",
              ),
            ],
            [
              tr(
                "Windows Hypervisor Platform non è attivo",
                "Windows Hypervisor Platform is not on",
                "Windows Hypervisor Platform n'est pas activé",
                "Windows Hypervisor Platform ist nicht aktiv",
              ),
              tr(
                "«Copia» il comando Enable-WindowsOptionalFeature, eseguilo in PowerShell come amministratore, una volta sola, poi riavvia il PC.",
                "“Copy” the Enable-WindowsOptionalFeature command, run it in PowerShell as an administrator, just once, then restart the PC.",
                "Avec « Copier », copiez la commande Enable-WindowsOptionalFeature, exécutez-la dans PowerShell en administrateur, une seule fois, puis redémarrez le PC.",
                "Kopiere mit „Kopieren“ den Befehl Enable-WindowsOptionalFeature, führe ihn in PowerShell als Administrator aus, nur einmal, und starte dann den PC neu.",
              ),
            ],
            [
              tr("/dev/kvm non c'è", "There is no /dev/kvm", "/dev/kvm est absent", "/dev/kvm fehlt"),
              tr(
                "Attiva la virtualizzazione (Intel VT-x o AMD-V) nel BIOS/UEFI e riavvia il PC.",
                "Turn on virtualization (Intel VT-x or AMD-V) in the BIOS/UEFI and restart the PC.",
                "Activez la virtualisation (Intel VT-x ou AMD-V) dans le BIOS/UEFI et redémarrez le PC.",
                "Schalte die Virtualisierung (Intel VT-x oder AMD-V) im BIOS/UEFI ein und starte den PC neu.",
              ),
            ],
            [
              tr(
                "Il tuo utente non può usare /dev/kvm",
                "Your user cannot use /dev/kvm",
                "Votre utilisateur ne peut pas utiliser /dev/kvm",
                "Dein Benutzer kann /dev/kvm nicht verwenden",
              ),
              tr(
                "«Copia» sudo usermod -aG kvm $USER, eseguilo in un terminale del PC, poi esci e rientra nella sessione.",
                "“Copy” sudo usermod -aG kvm $USER, run it in a terminal of the PC, then log out and back in.",
                "Avec « Copier », copiez sudo usermod -aG kvm $USER, exécutez-le dans un terminal du PC, puis fermez et rouvrez la session.",
                "Kopiere mit „Kopieren“ sudo usermod -aG kvm $USER, führe es in einem Terminal des PCs aus, melde dich dann ab und wieder an.",
              ),
            ],
            [
              tr(
                "AppArmor non lascia partire la microVM del kit",
                "AppArmor does not let the kit's microVM start",
                "AppArmor empêche la microVM du kit de démarrer",
                "AppArmor lässt die microVM des Kits nicht starten",
              ),
              tr(
                "«Copia» il comando sudo che crea il profilo, una volta (di nuovo se il kit cambia cartella). Se la cartella del kit ha caratteri che un profilo non accetta, il comando non c'è: sposta il kit in una cartella con un nome semplice.",
                "“Copy” the sudo command that creates the profile, once (again if the kit changes folder). If the kit's folder has characters a profile does not accept, there is no command: move the kit to a folder with a plain name.",
                "Avec « Copier », copiez la commande sudo qui crée le profil et exécutez-la une fois (à nouveau si le kit change de dossier). Si le dossier du kit contient des caractères qu'un profil n'accepte pas, il n'y a pas de commande : déplacez le kit dans un dossier au nom simple.",
                "Kopiere mit „Kopieren“ den sudo-Befehl, der das Profil anlegt, und führe ihn einmal aus (erneut, wenn das Kit den Ordner wechselt). Hat der Ordner des Kits Zeichen, die ein Profil nicht annimmt, gibt es keinen Befehl: Verschiebe das Kit in einen Ordner mit einfachem Namen.",
              ),
            ],
            [
              tr(
                "Docker Sandboxes (sbx) non è nel kit",
                "Docker Sandboxes (sbx) is not in the kit",
                "Docker Sandboxes (sbx) n'est pas dans le kit",
                "Docker Sandboxes (sbx) ist nicht im Kit",
              ),
              tr(
                "«Installa nel kit»: lo installa come la pagina Strumenti, con l'avanzamento nella barra di stato; alla fine lo stato si ricontrolla da solo. Se l'installazione non riesce, l'avviso ne dice il motivo accanto al pulsante, che riprova subito; Atollo riprova anche da solo ogni ora finché il kit non ha il suo sbx.",
                "“Install in the kit”: it installs it as the Tools page does, with the progress in the status bar; at the end the status is checked again by itself. If the installation fails, the notice gives the reason beside the button, which tries again at once; Atollo also tries again by itself every hour while the kit has no sbx of its own.",
                "« Installer dans le kit » : l'installe comme la page Outils, avec la progression dans la barre d'état ; à la fin, l'état est revérifié tout seul. Si l'installation échoue, l'avis en donne la raison à côté du bouton, qui réessaie tout de suite ; Atollo réessaie aussi tout seul toutes les heures tant que le kit n'a pas son sbx.",
                "„Im Kit installieren“: installiert es wie die Seite Werkzeuge, mit dem Fortschritt in der Statusleiste; am Ende wird der Zustand von selbst erneut geprüft. Schlägt die Installation fehl, nennt der Hinweis den Grund neben der Schaltfläche, die es sofort erneut versucht; Atollo versucht es außerdem jede Stunde von selbst, solange das Kit kein eigenes sbx hat.",
              ),
            ],
            [
              tr(
                "sbx non ha accesso a Docker",
                "sbx is not signed in to Docker",
                "sbx n'est pas connecté à Docker",
                "sbx ist nicht bei Docker angemeldet",
              ),
              tr(
                "«Lancia» apre sbx login nel terminale di Atollo (il primo accesso, sopra).",
                "“Run” opens sbx login in Atollo's terminal (the first sign-in, above).",
                "« Lancer » ouvre sbx login dans le terminal d'Atollo (la première connexion, ci-dessus).",
                "„Starten“ öffnet sbx login im Terminal von Atollo (die erste Anmeldung, oben).",
              ),
            ],
            [
              tr(
                "sbx ha risposto con un errore",
                "sbx answered with an error",
                "sbx a répondu par une erreur",
                "sbx hat mit einem Fehler geantwortet",
              ),
              tr(
                "«Lancia» apre sbx ls nel terminale di Atollo, per vedere il problema.",
                "“Run” opens sbx ls in Atollo's terminal, to see the problem.",
                "« Lancer » ouvre sbx ls dans le terminal d'Atollo, pour voir le problème.",
                "„Starten“ öffnet sbx ls im Terminal von Atollo, um das Problem zu sehen.",
              ),
            ],
            [
              tr(
                "Sul PC è già attivo un altro Docker Sandboxes",
                "Another Docker Sandboxes is already running on the PC",
                "Un autre Docker Sandboxes est déjà actif sur le PC",
                "Auf dem PC läuft bereits ein anderes Docker Sandboxes",
              ),
              tr(
                "Quello del PC o di un altro kit: le VM del kit finirebbero nei suoi dati. Se non ti serve, «Copia» sbx daemon stop, eseguilo, poi «Ricontrolla». Atollo non lo ferma mai da solo.",
                "The PC's or another kit's: the kit's VMs would end up in its data. If you do not need it, “Copy” sbx daemon stop, run it, then “Check again”. Atollo never stops it by itself.",
                "Celui du PC ou d'un autre kit : les VM du kit finiraient dans ses données. Si vous n'en avez pas besoin, copiez sbx daemon stop avec « Copier », exécutez-le, puis « Vérifier à nouveau ». Atollo ne l'arrête jamais de lui-même.",
                "Das des PCs oder eines anderen Kits: Die VMs des Kits landeten in dessen Daten. Wenn du es nicht brauchst, kopiere sbx daemon stop mit „Kopieren“, führe es aus, dann „Erneut prüfen“. Atollo hält es nie von selbst an.",
              ),
            ],
            [
              tr(
                "Il kit è su un disco FAT32 o exFAT",
                "The kit is on a FAT32 or exFAT disk",
                "Le kit est sur un disque FAT32 ou exFAT",
                "Das Kit liegt auf einem FAT32- oder exFAT-Laufwerk",
              ),
              tr(
                "Sposta il kit su un disco NTFS (APFS o ext4 su macOS e Linux) e riaprilo da lì: lì i dischi delle VM, fino a 30 GB ciascuno, occupano solo ciò che si scrive.",
                "Move the kit to an NTFS disk (APFS or ext4 on macOS and Linux) and open it again from there: there the VMs' disks, up to 30 GB each, take only what is written.",
                "Déplacez le kit sur un disque NTFS (APFS ou ext4 sur macOS et Linux) et rouvrez-le depuis là : les disques des VM, jusqu'à 30 Go chacun, n'y occupent que ce qui est écrit.",
                "Verschiebe das Kit auf ein NTFS-Laufwerk (APFS oder ext4 unter macOS und Linux) und öffne es von dort: Dort belegen die Festplatten der VMs, bis zu 30 GB je VM, nur, was geschrieben wird.",
              ),
            ],
          ]}
        />
        <P>
          {tr(
            "«Lancia» c'è solo per i comandi di sbx: Atollo li apre nel suo terminale, con il Docker Sandboxes del kit. I comandi del PC hanno «Copia», perché chiedono un amministratore e Atollo non li esegue mai. «Ricontrolla» chiede di nuovo lo stato, subito.",
            "“Run” is there only for sbx's commands: Atollo opens them in its terminal, with the kit's Docker Sandboxes. The PC's commands have “Copy”, because they need an administrator and Atollo never runs them. “Check again” asks for the status again, right away.",
            "« Lancer » n'existe que pour les commandes de sbx : Atollo les ouvre dans son terminal, avec le Docker Sandboxes du kit. Les commandes du PC ont « Copier », car elles demandent un administrateur et Atollo ne les exécute jamais. « Vérifier à nouveau » redemande l'état, tout de suite.",
            "„Starten“ gibt es nur für die Befehle von sbx: Atollo öffnet sie in seinem Terminal, mit dem Docker Sandboxes des Kits. Die Befehle des PCs haben „Kopieren“, weil sie einen Administrator brauchen und Atollo sie nie ausführt. „Erneut prüfen“ fragt den Zustand sofort neu ab.",
          )}
        </P>
        <P>
          {rich(
            tr(
              "Il terminale, le azioni e i siti dei progetti girano sul PC con il tuo utente, con o senza la VM: possono leggere i tuoi file e usare la rete. Il kit ne sigilla l'ambiente (niente variabili verso il tuo profilo vero o le tue chiavi SSH; MySQL con l'utente del progetto, mai l'amministratore; nessun agente), ma è un filtro, non un muro: {0}. Per questo gli agenti girano solo nella VM, e senza VM non partono.",
              "The projects' terminal, actions and websites run on the PC as your user, with or without the VM: they can read your files and use the network. The kit seals their environment (no variables leading to your real profile or your SSH keys; MySQL as the project's user, never the administrator; no agents), but it is a filter, not a wall: {0}. This is why the agents run only in the VM, and without the VM they do not start.",
              "Le terminal, les actions et les sites des projets tournent sur le PC avec votre utilisateur, avec ou sans la VM : ils peuvent lire vos fichiers et utiliser le réseau. Le kit scelle leur environnement (pas de variables menant à votre vrai profil ou à vos clés SSH ; MySQL avec l'utilisateur du projet, jamais l'administrateur ; aucun agent), mais c'est un filtre, pas un mur : {0}. C'est pourquoi les agents ne tournent que dans la VM, et sans VM ils ne démarrent pas.",
              "Terminal, Aktionen und Websites der Projekte laufen auf dem PC mit deinem Benutzer, mit oder ohne VM: Sie können deine Dateien lesen und das Netzwerk nutzen. Das Kit versiegelt ihre Umgebung (keine Variablen zu deinem echten Profil oder deinen SSH-Schlüsseln; MySQL mit dem Benutzer des Projekts, nie dem Administrator; keine Agenten), aber das ist ein Filter, keine Mauer: {0}. Deshalb laufen die Agenten nur in der VM, und ohne VM starten sie nicht.",
            ),
            <See key="i" id="isolation" />,
          )}
        </P>
      </Section>

      <Section title={tr("GitHub nella VM", "GitHub in the VM", "GitHub dans la VM", "GitHub in der VM")}>
        <P>
          {tr(
            "Di serie la VM di un progetto non usa il tuo accesso a GitHub: push e gh, dalla VM, non arrivano al tuo account. Per un progetto di cui ti fidi puoi accenderlo:",
            "By default a project's VM does not use your GitHub access: push and gh, from the VM, do not reach your account. For a project you trust you can turn it on:",
            "Par défaut, la VM d'un projet n'utilise pas votre accès GitHub : push et gh, depuis la VM, n'atteignent pas votre compte. Pour un projet de confiance, vous pouvez l'activer :",
            "Standardmäßig nutzt die VM eines Projekts deinen GitHub-Zugang nicht: Push und gh erreichen aus der VM dein Konto nicht. Für ein Projekt, dem du vertraust, kannst du ihn einschalten:",
          )}
        </P>
        <Steps
          steps={[
            tr(
              "Collega il kit a GitHub in Impostazioni › Account GitHub, se non l'hai già fatto.",
              "Connect the kit to GitHub in Settings › GitHub account, if you have not already.",
              "Reliez le kit à GitHub dans Paramètres › Compte GitHub, si ce n'est pas déjà fait.",
              "Verbinde das Kit mit GitHub unter Einstellungen › GitHub-Konto, falls noch nicht geschehen.",
            ),
            tr(
              "Nella pagina del progetto apri la sezione «Git»; in fondo c'è il riquadro «GitHub nella VM».",
              "On the project's page open the “Git” section; at the bottom there is the “GitHub in the VM” box.",
              "Sur la page du projet, ouvrez la section « Git » ; en bas se trouve l'encadré « GitHub dans la VM ».",
              "Öffne auf der Seite des Projekts den Abschnitt „Git“; unten steht der Kasten „GitHub in der VM“.",
            ),
            tr(
              "Spunta «Il progetto nella VM usa il tuo accesso a GitHub del kit (push, gh)». Vale dal prossimo avvio della sua VM; togliendo la spunta l'accesso si toglie subito.",
              "Tick “The project in the VM uses your GitHub access of the kit (push, gh)”. It applies from the next start of its VM; unticking it removes the access right away.",
              "Cochez « Le projet dans la VM utilise ton accès GitHub du kit (push, gh) ». Cela vaut dès le prochain démarrage de sa VM ; en décochant, l'accès est retiré aussitôt.",
              "Setze das Häkchen bei „Das Projekt in der VM nutzt deinen GitHub-Zugang des Kits (push, gh)“. Es gilt ab dem nächsten Start seiner VM; ohne Häkchen wird der Zugang sofort entfernt.",
            ),
          ]}
        />
        <Callout
          tone="warn"
          title={tr(
            "Solo per i progetti di cui ti fidi",
            "Only for projects you trust",
            "Seulement pour les projets de confiance",
            "Nur für Projekte, denen du vertraust",
          )}
        >
          {tr(
            "Il token non entra nella VM: il proxy di Docker Sandboxes lo aggiunge alle richieste verso GitHub. Ma un agente, o il codice che esegue (anche un npm install), potrebbe agire su GitHub a tuo nome. La scelta vale solo per la sandbox di quel progetto, mai per i terminali del kit.",
            "The token never enters the VM: the Docker Sandboxes proxy adds it to the requests to GitHub. But an agent, or the code it runs (even an npm install), could act on GitHub on your behalf. The choice applies only to that project's sandbox, never to the kit's terminals.",
            "Le jeton n'entre jamais dans la VM : le proxy de Docker Sandboxes l'ajoute aux requêtes vers GitHub. Mais un agent, ou le code qu'il exécute (même un npm install), pourrait agir sur GitHub en votre nom. Le choix ne vaut que pour la sandbox de ce projet, jamais pour les terminaux du kit.",
            "Das Token gelangt nie in die VM: Der Proxy von Docker Sandboxes fügt es den Anfragen an GitHub hinzu. Aber ein Agent oder der Code, den er ausführt (auch ein npm install), könnte auf GitHub in deinem Namen handeln. Die Wahl gilt nur für die Sandbox dieses Projekts, nie für die Terminals des Kits.",
          )}
        </Callout>
      </Section>

      <Section
        title={tr(
          "Le sandbox e lo spazio che occupano",
          "The sandboxes and the room they take",
          "Les sandbox et l'espace qu'elles occupent",
          "Die Sandboxes und ihr Platzbedarf",
        )}
      >
        <P>
          {tr(
            "La finestra che si apre dalla barra di stato (anche quando dice «VM») ha, con Docker Sandboxes nel kit, la sezione «Sandbox delle VM»: in cima lo «Spazio occupato nel kit», lo spazio vero sul disco; poi ogni sandbox di Atollo con di chi è (un progetto o un suo agente), «accesa» o «spenta», e «Rimuovi».",
            "The window that opens from the status bar (even when it says “VM”) has, with Docker Sandboxes in the kit, the “VM sandboxes” section: at the top the “Space taken in the kit”, the real room on the disk; then each of Atollo's sandboxes with whose it is (a project or one of its agents), “running” or “stopped”, and “Remove”.",
            "La fenêtre qui s'ouvre depuis la barre d'état (même quand elle indique « VM ») a, avec Docker Sandboxes dans le kit, la section « Sandbox des VM » : en haut l'« Espace occupé dans le kit », l'espace réel sur le disque ; puis chaque sandbox d'Atollo avec à qui elle appartient (un projet ou l'un de ses agents), « allumée » ou « éteinte », et « Supprimer ».",
            "Das Fenster, das sich aus der Statusleiste öffnet (auch wenn sie „VM“ zeigt), hat mit Docker Sandboxes im Kit den Abschnitt „Sandboxes der VMs“: oben der „Belegte Platz im Kit“, der echte Platz auf dem Laufwerk; dann jede Sandbox von Atollo mit ihrem Besitzer (ein Projekt oder einer seiner Agenten), „läuft“ oder „gestoppt“, und „Entfernen“.",
          )}
        </P>
        <Bullets
          items={[
            tr(
              "«Rimuovi», poi «Rimuovi davvero», cancella la sandbox con il suo disco (la sua home e /tmp); i file del progetto restano. Al prossimo uso Atollo ne crea una nuova.",
              "“Remove”, then “Really remove”, deletes the sandbox with its disk (its home and /tmp); the project's files stay. At the next use Atollo makes a new one.",
              "« Supprimer », puis « Supprimer vraiment », efface la sandbox avec son disque (son dossier personnel et /tmp) ; les fichiers du projet restent. À la prochaine utilisation, Atollo en crée une nouvelle.",
              "„Entfernen“, dann „Wirklich entfernen“, löscht die Sandbox mit ihrer Festplatte (ihr Home und /tmp); die Dateien des Projekts bleiben. Bei der nächsten Verwendung legt Atollo eine neue an.",
            ),
            tr(
              "Eliminare un progetto toglie anche la sua sandbox, con gli accessi dei suoi agenti.",
              "Deleting a project also removes its sandbox, with its agents' sign-ins.",
              "Supprimer un projet retire aussi sa sandbox, avec les connexions de ses agents.",
              "Ein Projekt zu löschen entfernt auch seine Sandbox, samt den Anmeldungen seiner Agenten.",
            ),
            tr(
              "I dischi delle VM arrivano fino a circa 30 GB ciascuno, ma su NTFS, APFS ed ext4 occupano solo ciò che si scrive.",
              "The VMs' disks reach up to about 30 GB each, but on NTFS, APFS and ext4 they take only what is written.",
              "Les disques des VM vont jusqu'à environ 30 Go chacun, mais sur NTFS, APFS et ext4 ils n'occupent que ce qui est écrit.",
              "Die Festplatten der VMs reichen bis etwa 30 GB je VM, belegen auf NTFS, APFS und ext4 aber nur, was geschrieben wird.",
            ),
            tr(
              "Se il kit cambia lettera o cartella, la sandbox di un progetto si rifà da sola nella cartella nuova, con una riga nel log: file, database, strumenti e accessi stanno nel kit e restano; si perdono solo la home della VM e le immagini scaricate.",
              "If the kit changes letter or folder, a project's sandbox is made again by itself in the new folder, with a line in the log: files, databases, tools and sign-ins are in the kit and stay; only the VM's home and the downloaded images are lost.",
              "Si le kit change de lettre ou de dossier, la sandbox d'un projet est refaite toute seule dans le nouveau dossier, avec une ligne dans le journal : fichiers, bases de données, outils et connexions sont dans le kit et restent ; seuls le dossier personnel de la VM et les images téléchargées sont perdus.",
              "Wechselt das Kit Buchstaben oder Ordner, wird die Sandbox eines Projekts von selbst im neuen Ordner neu angelegt, mit einer Zeile im Log: Dateien, Datenbanken, Werkzeuge und Anmeldungen liegen im Kit und bleiben; verloren gehen nur das Home der VM und die heruntergeladenen Images.",
            ),
            tr(
              "«Copia del kit» non copia immagini e dischi delle VM (decine di GB): le sandbox si rifanno da sole dove la copia viene aperta.",
              "“Copy of the kit” does not copy the VMs' images and disks (tens of GB): the sandboxes are made again by themselves where the copy is opened.",
              "« Copie du kit » ne copie pas les images et les disques des VM (des dizaines de Go) : les sandbox sont refaites toutes seules là où la copie est ouverte.",
              "„Kopie des Kits“ kopiert die Images und Festplatten der VMs nicht (Dutzende GB): Die Sandboxes werden dort, wo die Kopie geöffnet wird, von selbst neu angelegt.",
            ),
          ]}
        />
      </Section>

      <Section
        title={tr(
          "Le azioni, i siti e i nuovi progetti, sul PC",
          "Actions, websites and new projects, on the PC",
          "Les actions, les sites et les nouveaux projets, sur le PC",
          "Aktionen, Websites und neue Projekte, auf dem PC",
        )}
      >
        <Bullets
          items={[
            tr(
              "Le azioni, i siti (con un server tutto suo o in un pool della pagina Server Web), i task dell'editor e la creazione di un progetto da una base (il comando di un framework, come composer create-project o npm create) girano sul PC, con l'ambiente sigillato del kit, con o senza la VM.",
              "The actions, the websites (with their own server or in a pool of the Web Servers page), the editor's tasks and creating a project from a base (a framework's command, such as composer create-project or npm create) run on the PC, with the kit's sealed environment, with or without the VM.",
              "Les actions, les sites (avec leur propre serveur ou dans un pool de la page Serveurs web), les tâches de l'éditeur et la création d'un projet depuis une base (la commande d'un framework, comme composer create-project ou npm create) tournent sur le PC, avec l'environnement scellé du kit, avec ou sans la VM.",
              "Die Aktionen, die Websites (mit eigenem Server oder in einem Pool der Seite Webserver), die Tasks des Editors und das Anlegen eines Projekts aus einer Basis (der Befehl eines Frameworks, etwa composer create-project oder npm create) laufen auf dem PC, mit der versiegelten Umgebung des Kits, mit oder ohne VM.",
            ),
            tr(
              "Accendere la VM non cambia niente per loro: porte, indirizzi .localhost, alias e rete locale funzionano come senza VM. La VM serve solo agli agenti, che lavorano sugli stessi file del progetto.",
              "Turning the VM on changes nothing for them: ports, .localhost addresses, aliases and the local network work as without the VM. The VM is only for the agents, which work on the same files of the project.",
              "Activer la VM ne change rien pour eux : ports, adresses .localhost, alias et réseau local fonctionnent comme sans VM. La VM ne sert qu'aux agents, qui travaillent sur les mêmes fichiers du projet.",
              "Die VM einzuschalten ändert für sie nichts: Ports, .localhost-Adressen, Aliasse und das lokale Netzwerk funktionieren wie ohne VM. Die VM ist nur für die Agenten da, die an denselben Dateien des Projekts arbeiten.",
            ),
          ]}
        />
      </Section>

      <Section
        title={tr(
          "Che cosa non va ancora nella VM",
          "What does not go into the VM yet",
          "Ce qui ne passe pas encore dans la VM",
          "Was noch nicht in die VM geht",
        )}
      >
        <Callout tone="warn">
          <Bullets
            items={[
              tr(
                "I server MCP del kit e del progetto per gli agenti nella VM: i loro comandi sono programmi del PC, e l'agente nella VM per ora non li ha.",
                "The kit's and the project's MCP servers for the agents in the VM: their commands are programs of the PC, and the agent in the VM does not have them for now.",
                "Les serveurs MCP du kit et du projet pour les agents dans la VM : leurs commandes sont des programmes du PC, et l'agent dans la VM ne les a pas pour l'instant.",
                "Die MCP-Server des Kits und des Projekts für die Agenten in der VM: Ihre Befehle sind Programme des PCs, und der Agent in der VM hat sie vorerst nicht.",
              ),
            ]}
          />
        </Callout>
      </Section>

      <Section
        title={tr(
          "Che cosa resta fuori dal kit",
          "What stays outside the kit",
          "Ce qui reste hors du kit",
          "Was außerhalb des Kits bleibt",
        )}
      >
        <P>
          {tr(
            "Sandbox, dischi, immagini, impostazioni e registri di Docker Sandboxes stanno in data e viaggiano con il kit. Atollo ferma Docker Sandboxes quando si chiude e prima di «Rimuovi l'unità». Restano fuori, perché Docker Sandboxes o il sistema non lasciano altro:",
            "Docker Sandboxes' sandboxes, disks, images, settings and logs are in data and travel with the kit. Atollo stops Docker Sandboxes when it closes and before “Remove drive”. What stays outside, because Docker Sandboxes or the system allow nothing else:",
            "Les sandbox, disques, images, paramètres et journaux de Docker Sandboxes sont dans data et voyagent avec le kit. Atollo arrête Docker Sandboxes à sa fermeture et avant « Retirer le lecteur ». Restent dehors, parce que Docker Sandboxes ou le système ne permettent rien d'autre :",
            "Sandboxes, Festplatten, Images, Einstellungen und Logs von Docker Sandboxes liegen in data und reisen mit dem Kit. Atollo beendet Docker Sandboxes beim Schließen und vor „Laufwerk entfernen“. Außerhalb bleiben, weil Docker Sandboxes oder das System nichts anderes zulassen:",
          )}
        </P>
        <Bullets
          items={[
            tr(
              "L'hypervisor (Windows Hypervisor Platform, /dev/kvm), attivato una volta da te come amministratore.",
              "The hypervisor (Windows Hypervisor Platform, /dev/kvm), turned on once by you as an administrator.",
              "L'hyperviseur (Windows Hypervisor Platform, /dev/kvm), activé une fois par vous en administrateur.",
              "Der Hypervisor (Windows Hypervisor Platform, /dev/kvm), einmal von dir als Administrator eingeschaltet.",
            ),
            tr(
              "L'accesso a Docker (sbx login), nel portachiavi del sistema, una volta per PC.",
              "The Docker sign-in (sbx login), in the system keychain, once per PC.",
              "La connexion à Docker (sbx login), dans le trousseau du système, une fois par PC.",
              "Die Anmeldung bei Docker (sbx login), im Schlüsselbund des Systems, einmal pro PC.",
            ),
            tr(
              "Per «GitHub nella VM», nel portachiavi del sistema (su Windows la Gestione credenziali) il comando che dà il token alla sandbox: il comando, non il token.",
              "For “GitHub in the VM”, in the system keychain (on Windows the Credential Manager) the command that gives the token to the sandbox: the command, not the token.",
              "Pour « GitHub dans la VM », dans le trousseau du système (sur Windows le Gestionnaire d'identification) la commande qui donne le jeton à la sandbox : la commande, pas le jeton.",
              "Für „GitHub in der VM“ im Schlüsselbund des Systems (unter Windows die Anmeldeinformationsverwaltung) der Befehl, der der Sandbox das Token gibt: der Befehl, nicht das Token.",
            ),
            tr(
              "Una giunzione sboxd-<id> e una cartella gvisor nella cartella temporanea del PC, per i percorsi brevi del servizio di Docker Sandboxes.",
              "A junction sboxd-<id> and a gvisor folder in the PC's temporary folder, for the short paths of the Docker Sandboxes service.",
              "Une jonction sboxd-<id> et un dossier gvisor dans le dossier temporaire du PC, pour les chemins courts du service de Docker Sandboxes.",
              "Eine Verknüpfung sboxd-<id> und ein Ordner gvisor im temporären Ordner des PCs, für die kurzen Pfade des Dienstes von Docker Sandboxes.",
            ),
            tr(
              "Su Ubuntu, il profilo AppArmor della microVM del kit.",
              "On Ubuntu, the AppArmor profile of the kit's microVM.",
              "Sur Ubuntu, le profil AppArmor de la microVM du kit.",
              "Unter Ubuntu das AppArmor-Profil der microVM des Kits.",
            ),
          ]}
        />
      </Section>

      <Under>
        <Bullets
          items={[
            tr(
              "Nomi delle sandbox: atollo-p-<progetto>-<hash> per un progetto, dove girano tutti i suoi agenti. L'hash viene dal nome esatto: due progetti non condividono mai una sandbox. I terminali del kit non hanno sandbox: girano sul PC. Le sandbox atollo-a-… e atollo-t-… di una versione precedente di Atollo si rimuovono da sole.",
              "Sandbox names: atollo-p-<project>-<hash> for a project, where all its agents run. The hash comes from the exact name: two projects never share a sandbox. The kit's terminals have no sandbox: they run on the PC. The atollo-a-… and atollo-t-… sandboxes of an earlier Atollo are removed by themselves.",
              "Noms des sandbox : atollo-p-<projet>-<hash> pour un projet, où tournent tous ses agents. Le hash vient du nom exact : deux projets ne partagent jamais une sandbox. Les terminaux du kit n'ont pas de sandbox : ils tournent sur le PC. Les sandbox atollo-a-… et atollo-t-… d'une version précédente d'Atollo se suppriment d'elles-mêmes.",
              "Namen der Sandboxes: atollo-p-<Projekt>-<Hash> für ein Projekt, in der alle seine Agenten laufen. Der Hash stammt vom genauen Namen: Zwei Projekte teilen sich nie eine Sandbox. Die Terminals des Kits haben keine Sandbox: Sie laufen auf dem PC. Die Sandboxes atollo-a-… und atollo-t-… einer früheren Version von Atollo werden von selbst entfernt.",
            ),
            tr(
              "data\\app\\sandboxes\\<sandbox>\\ è la descrizione di ogni sandbox (spec.yaml, atollo.json con di chi è, le cartelle montate e le porte aperte, db.env con la connessione al database, leggibile solo da te, e guest, la cartella che la VM vede in sola lettura, con atollo); secrets.json annota i comandi di GitHub e della chiave di Gemini registrati.",
              "data\\app\\sandboxes\\<sandbox>\\ is each sandbox's description (spec.yaml, atollo.json with whose it is, the mounted folders and the open ports, db.env with the database connection, readable only by you, and guest, the folder the VM sees read-only, with atollo); secrets.json notes the GitHub and Gemini key commands registered.",
              "data\\app\\sandboxes\\<sandbox>\\ est la description de chaque sandbox (spec.yaml, atollo.json avec à qui elle appartient, les dossiers montés et les ports ouverts, db.env avec la connexion à la base, lisible par vous seul, et guest, le dossier que la VM voit en lecture seule, avec atollo) ; secrets.json note les commandes GitHub et de la clé Gemini enregistrées.",
              "data\\app\\sandboxes\\<sandbox>\\ ist die Beschreibung jeder Sandbox (spec.yaml, atollo.json mit ihrem Besitzer, die eingehängten Ordner und die offenen Ports, db.env mit der Datenbankverbindung, nur für dich lesbar, und guest, der Ordner, den die VM schreibgeschützt sieht, mit atollo); secrets.json vermerkt die registrierten Befehle für GitHub und den Gemini-Schlüssel.",
            ),
            tr(
              "data\\app\\levels\\requests\\ tiene per un attimo le richieste dei task e dei profili degli agenti dell'editor e dei comandi degli agenti scritti in un terminale del PC (data\\kit\\forward, con i client data\\kit\\launch.ps1 e launch.sh): Atollo risponde a ognuna decidendo dove gira; si cancellano a ogni avvio dell'app e nessuna VM le vede.",
              "data\\app\\levels\\requests\\ briefly holds the requests of the editor's tasks and agent profiles and of the agents' commands typed in a terminal of the PC (data\\kit\\forward, with the clients data\\kit\\launch.ps1 and launch.sh): Atollo answers each one by deciding where it runs; they are cleared at every start of the app and no VM sees them.",
              "data\\app\\levels\\requests\\ garde un instant les requêtes des tâches et des profils d'agents de l'éditeur et des commandes d'agents tapées dans un terminal du PC (data\\kit\\forward, avec les clients data\\kit\\launch.ps1 et launch.sh) : Atollo répond à chacune en décidant où elle tourne ; elles sont effacées à chaque démarrage de l'application et aucune VM ne les voit.",
              "data\\app\\levels\\requests\\ enthält kurz die Anfragen der Tasks und Agentenprofile des Editors und der in einem Terminal des PCs getippten Agentenbefehle (data\\kit\\forward, mit den Clients data\\kit\\launch.ps1 und launch.sh): Atollo beantwortet jede, indem es entscheidet, wo sie läuft; sie werden bei jedem Start der App gelöscht, und keine VM sieht sie.",
            ),
            tr(
              "Docker Sandboxes sta in data\\apps\\<sistema>\\sandboxes; configurazione, accesso e registri in data\\home (su Windows data\\home\\AppData\\Local\\DockerSandboxes), immagini e dischi nella cartella di stato del suo servizio, anch'essa nel kit.",
              "Docker Sandboxes is in data\\apps\\<system>\\sandboxes; settings, sign-in state and logs in data\\home (on Windows data\\home\\AppData\\Local\\DockerSandboxes), images and disks in its service's state folder, in the kit as well.",
              "Docker Sandboxes est dans data\\apps\\<système>\\sandboxes ; configuration, état de connexion et journaux dans data\\home (sur Windows data\\home\\AppData\\Local\\DockerSandboxes), images et disques dans le dossier d'état de son service, lui aussi dans le kit.",
              "Docker Sandboxes liegt in data\\apps\\<System>\\sandboxes; Konfiguration, Anmeldestatus und Logs in data\\home (unter Windows data\\home\\AppData\\Local\\DockerSandboxes), Images und Festplatten im Zustandsordner seines Dienstes, ebenfalls im Kit.",
            ),
            tr(
              "Il servizio di Docker Sandboxes del kit parte con l'inoltro dell'agente SSH spento, senza browser nella VM, senza invio di diagnostica e senza telemetria.",
              "The kit's Docker Sandboxes service starts with SSH agent forwarding off, no browser in the VM, no diagnostics upload and no telemetry.",
              "Le service Docker Sandboxes du kit démarre avec le transfert de l'agent SSH désactivé, sans navigateur dans la VM, sans envoi de diagnostics et sans télémétrie.",
              "Der Docker-Sandboxes-Dienst des Kits startet mit ausgeschalteter SSH-Agent-Weiterleitung, ohne Browser in der VM, ohne Hochladen von Diagnosen und ohne Telemetrie.",
            ),
            tr(
              "Nella VM partono solo gli agenti, con gli strumenti del kit per Linux in testa al PATH; nessuna porta della VM si pubblica sul PC. Nella VM D:\\a\\b diventa /d/a/b; su macOS e Linux il percorso resta lo stesso.",
              "Only the agents start in the VM, with the kit's tools for Linux first on the PATH; no port of the VM is published on the PC. In the VM D:\\a\\b becomes /d/a/b; on macOS and Linux the path stays the same.",
              "Seuls les agents démarrent dans la VM, avec les outils du kit pour Linux en tête du PATH ; aucun port de la VM n'est publié sur le PC. Dans la VM, D:\\a\\b devient /d/a/b ; sur macOS et Linux, le chemin reste le même.",
              "In der VM starten nur die Agenten, mit den Werkzeugen des Kits für Linux vorne im PATH; kein Port der VM wird auf dem PC veröffentlicht. In der VM wird D:\\a\\b zu /d/a/b; unter macOS und Linux bleibt der Pfad gleich.",
            ),
          ]}
        />
      </Under>

      <Related ids={["isolation", "terminals", "project-git", "troubleshooting"]} />
    </>
  ),

  isolation: () => (
    <>
      <InBrief
        points={[
          tr(
            "Atollo non si installa, non scrive nel registro e non chiede diritti di amministratore.",
            "Atollo does not install itself, does not write to the registry and does not ask for administrator rights.",
            "Atollo ne s'installe pas, n'écrit pas dans le registre et ne demande pas de droits d'administrateur.",
            "Atollo wird nicht installiert, schreibt nicht in die Registry und verlangt keine Administratorrechte.",
          ),
          tr(
            "I programmi del kit partono in un ambiente costruito da zero, che punta solo a cartelle del kit.",
            "The kit's programs start in an environment built from scratch, which points only to kit folders.",
            "Les programmes du kit démarrent dans un environnement construit de zéro, qui ne pointe que vers des dossiers du kit.",
            "Die Programme des Kits starten in einer von Grund auf neu aufgebauten Umgebung, die nur auf Ordner des Kits zeigt.",
          ),
          tr(
            "Così non vedono i programmi del PC e il PC non vede loro: niente conflitti, niente tracce.",
            "So they do not see the PC's programs and the PC does not see them: no conflicts, no traces.",
            "Ainsi, ils ne voient pas les programmes du PC et le PC ne les voit pas : pas de conflits, pas de traces.",
            "So sehen sie die Programme des PCs nicht und der PC sieht sie nicht: keine Konflikte, keine Spuren.",
          ),
          tr(
            "Poche eccezioni sono dichiarate: la cartella temporanea del sistema e, per qualche secondo, il portachiavi.",
            "A few exceptions are declared: the system's temporary folder and, for a few seconds, the keychain.",
            "Quelques exceptions sont déclarées : le dossier temporaire du système et, pendant quelques secondes, le trousseau.",
            "Wenige Ausnahmen sind offen angegeben: der temporäre Ordner des Systems und, für einige Sekunden, der Schlüsselbund.",
          ),
          tr(
            "L'ambiente è un filtro, non un muro: il confine vero è la VM, dove girano gli agenti. I programmi dei progetti, che girano sul PC con o senza la VM, ricevono in più un ambiente sigillato.",
            "The environment is a filter, not a wall: the real boundary is the VM, where the agents run. The projects' programs, which run on the PC with or without the VM, also get a sealed environment.",
            "L'environnement est un filtre, pas un mur : la vraie frontière est la VM, où tournent les agents. Les programmes des projets, qui tournent sur le PC avec ou sans la VM, reçoivent en plus un environnement scellé.",
            "Die Umgebung ist ein Filter, keine Mauer: Die echte Grenze ist die VM, in der die Agenten laufen. Die Programme der Projekte, die mit oder ohne VM auf dem PC laufen, erhalten zusätzlich eine versiegelte Umgebung.",
          ),
        ]}
      />

      <Section
        title={tr(
          "L'idea, con un esempio",
          "The idea, with an example",
          "L'idée, avec un exemple",
          "Die Idee, an einem Beispiel",
        )}
      >
        <P>
          {tr(
            "Sul PC di un collega c'è già Node.js 18 installato, con i suoi pacchetti globali e le sue impostazioni di npm. Se apri una console del kit e scrivi node --version, rispondi Node 24 del kit, non il 18 del PC: la console del kit non sa nemmeno che l'altro esiste. E quando installi un pacchetto globale, finisce nella cartella del kit, non nel profilo del collega.",
            "A colleague's PC already has Node.js 18 installed, with its global packages and its npm settings. If you open a kit console and type node --version, the kit's Node 24 answers, not the PC's 18: the kit console does not even know the other one exists. And when you install a global package, it ends up in the kit folder, not in your colleague's profile.",
            "Sur le PC d'un collègue, Node.js 18 est déjà installé, avec ses paquets globaux et ses paramètres npm. Si vous ouvrez une console du kit et tapez node --version, c'est Node 24 du kit qui répond, pas le 18 du PC : la console du kit ne sait même pas que l'autre existe. Et quand vous installez un paquet global, il atterrit dans le dossier du kit, pas dans le profil du collègue.",
            "Auf dem PC eines Kollegen ist bereits Node.js 18 installiert, mit seinen globalen Paketen und seinen npm-Einstellungen. Wenn du eine Konsole des Kits öffnest und node --version eingibst, antwortet Node 24 des Kits, nicht die 18 des PCs: Die Konsole des Kits weiß nicht einmal, dass die andere existiert. Und wenn du ein globales Paket installierst, landet es im Kit-Ordner, nicht im Profil des Kollegen.",
          )}
        </P>
        <P>
          {tr(
            "Questo succede perché ogni programma avviato dal kit riceve un ambiente costruito da zero: il kit prende dal PC solo una manciata di variabili di sistema (nome del computer, cartelle di Windows, lingua…) e imposta tutte le altre in modo che puntino dentro data.",
            "This happens because every program started by the kit receives an environment built from scratch: the kit takes only a handful of system variables from the PC (computer name, Windows folders, language…) and sets all the others so that they point inside data.",
            "Cela se produit parce que chaque programme lancé par le kit reçoit un environnement construit de zéro : le kit ne prend du PC qu'une poignée de variables système (nom de l'ordinateur, dossiers de Windows, langue…) et définit toutes les autres pour qu'elles pointent dans data.",
            "Das liegt daran, dass jedes vom Kit gestartete Programm eine von Grund auf neu aufgebaute Umgebung erhält: Das Kit übernimmt vom PC nur eine Handvoll Systemvariablen (Computername, Windows-Ordner, Sprache…) und setzt alle anderen so, dass sie in data zeigen.",
          )}
        </P>
      </Section>

      <Section
        title={tr("Che cosa viene reindirizzato", "What is redirected", "Ce qui est redirigé", "Was umgeleitet wird")}
      >
        <Table
          columns={[
            tr("Gruppo", "Group", "Groupe", "Gruppe"),
            tr("Variabili principali", "Main variables", "Variables principales", "Wichtigste Variablen"),
            tr("Dove portano", "Where they point", "Où elles mènent", "Wohin sie zeigen"),
          ]}
          rows={[
            [
              tr("Cartella utente", "User folder", "Dossier utilisateur", "Benutzerordner"),
              <Code key="a">HOME, USERPROFILE, APPDATA, LOCALAPPDATA</Code>,
              <Code key="b">data\home</Code>,
            ],
            [
              tr("File temporanei", "Temporary files", "Fichiers temporaires", "Temporäre Dateien"),
              <Code key="c">TEMP, TMP (TMPDIR su macOS e Linux)</Code>,
              tr(
                "data\\tmp\\system o la cartella temporanea del sistema, secondo le Impostazioni",
                "data\\tmp\\system or the system's temporary folder, depending on Settings",
                "data\\tmp\\system ou le dossier temporaire du système, selon les Paramètres",
                "data\\tmp\\system oder der temporäre Ordner des Systems, je nach Einstellungen",
              ),
            ],
            [
              tr("Agenti", "Agents", "Agents", "Agenten"),
              <Code key="d">CLAUDE_CONFIG_DIR, CODEX_HOME, PI_CODING_AGENT_DIR</Code>,
              <Code key="e">data\home\.claude, data\home\.codex, data\home\.pi\agent</Code>,
            ],
            [
              tr("GitHub CLI", "GitHub CLI", "GitHub CLI", "GitHub CLI"),
              <Code key="f">GH_CONFIG_DIR</Code>,
              <Code key="g">data\gh</Code>,
            ],
            [
              tr("Cache dei pacchetti", "Package caches", "Caches des paquets", "Paket-Caches"),
              <Code key="h">
                NPM_CONFIG_CACHE, PIP_CACHE_DIR, UV_CACHE_DIR, COMPOSER_CACHE_DIR, GOCACHE, NUGET_PACKAGES
              </Code>,
              <Code key="i">data\cache\…</Code>,
            ],
            [
              tr("Pacchetti globali", "Global packages", "Paquets globaux", "Globale Pakete"),
              <Code key="j">
                NPM_CONFIG_PREFIX, PYTHONUSERBASE, COMPOSER_HOME, GOPATH, GRADLE_USER_HOME, CARGO_HOME
              </Code>,
              <Code key="k">data\npm-global, data\python-user, data\composer, data\go, data\gradle…</Code>,
            ],
            [
              tr("Linguaggi", "Languages", "Langages", "Sprachen"),
              <Code key="l">JAVA_HOME, DOTNET_ROOT, PHPRC, RUSTUP_HOME…</Code>,
              tr(
                "Le cartelle dei programmi del kit, alle versioni in uso",
                "The folders of the kit's programs, at the versions in use",
                "Les dossiers des programmes du kit, aux versions utilisées",
                "Die Ordner der Programme des Kits, in den verwendeten Versionen",
              ),
            ],
            [
              tr("Certificati", "Certificates", "Certificats", "Zertifikate"),
              <Code key="o">
                SSL_CERT_FILE, REQUESTS_CA_BUNDLE, PIP_CERT, CURL_CA_BUNDLE, GIT_SSL_CAINFO, CARGO_HTTP_CAINFO,
                NODE_EXTRA_CA_CERTS
              </Code>,
              tr(
                "Solo con un file PEM in Impostazioni › Rete: data\\app\\ca-bundle.pem (i certificati del sistema più quelli del file); Node.js riceve il file PEM",
                "Only with a PEM file in Settings › Network: data\\app\\ca-bundle.pem (the system's certificates plus those of the file); Node.js gets the PEM file",
                "Seulement avec un fichier PEM dans Paramètres › Réseau : data\\app\\ca-bundle.pem (les certificats du système plus ceux du fichier) ; Node.js reçoit le fichier PEM",
                "Nur mit einer PEM-Datei unter Einstellungen › Netzwerk: data\\app\\ca-bundle.pem (die Zertifikate des Systems plus die der Datei); Node.js erhält die PEM-Datei",
              ),
            ],
            [
              tr(
                "Niente telemetria e aggiornamenti automatici",
                "No telemetry and no automatic updates",
                "Pas de télémétrie ni de mises à jour automatiques",
                "Keine Telemetrie und keine automatischen Updates",
              ),
              <Code key="m">
                DOTNET_CLI_TELEMETRY_OPTOUT, POWERSHELL_TELEMETRY_OPTOUT, GOTELEMETRY=off, DISABLE_UPDATES
              </Code>,
              tr(
                "Spenti: gli strumenti non mandano dati e non si aggiornano da soli",
                "Off: the tools send no data and do not update themselves",
                "Désactivées : les outils n'envoient pas de données et ne se mettent pas à jour tout seuls",
                "Ausgeschaltet: Die Werkzeuge senden keine Daten und aktualisieren sich nicht selbst",
              ),
            ],
            [
              tr("PATH", "PATH", "PATH", "PATH"),
              <Code key="n">PATH</Code>,
              tr(
                "Prima le cartelle del kit, poi solo le cartelle di sistema (System32 su Windows, /usr/bin e simili su macOS e Linux)",
                "First the kit folders, then only the system folders (System32 on Windows, /usr/bin and similar on macOS and Linux)",
                "D'abord les dossiers du kit, puis uniquement les dossiers système (System32 sur Windows, /usr/bin et similaires sur macOS et Linux)",
                "Zuerst die Ordner des Kits, dann nur die Systemordner (System32 unter Windows, /usr/bin und ähnliche unter macOS und Linux)",
              ),
            ],
          ]}
        />
        <P>
          {rich(
            tr(
              "Per vedere l'ambiente completo apri una console nella cartella del kit e usa il comando {0} (le chiavi e le password sono mascherate con ***).",
              "To see the complete environment open a console in the kit folder and use the command {0} (keys and passwords are masked with ***).",
              "Pour voir l'environnement complet, ouvrez une console dans le dossier du kit et utilisez la commande {0} (les clés et les mots de passe sont masqués par ***).",
              "Um die vollständige Umgebung zu sehen, öffnest du eine Konsole im Kit-Ordner und verwendest den Befehl {0} (Schlüssel und Passwörter werden mit *** maskiert).",
            ),
            <Code key="e">.\Atollo.exe env | Out-Host</Code>,
          )}
        </P>
      </Section>

      <Section
        title={tr(
          "L'ambiente sigillato dei progetti",
          "The projects' sealed environment",
          "L'environnement scellé des projets",
          "Die versiegelte Umgebung der Projekte",
        )}
      >
        <P>
          {tr(
            "Il terminale di un progetto, le sue azioni, i suoi siti e i suoi task, sul PC con o senza la VM, e i terminali del kit e dell'editor ricevono l'ambiente del kit sigillato, senza i programmi degli agenti (al loro posto, gli inoltri verso la VM): un progetto riceve da sé solo ciò che è suo.",
            "A project's terminal, actions, websites and tasks, on the PC with or without the VM, and the kit's and the editor's terminals get the kit's environment sealed, without the agents' programs (in their place, the forwarders to the VM): a project receives from it only what is its own.",
            "Le terminal d'un projet, ses actions, ses sites et ses tâches, sur le PC avec ou sans la VM, et les terminaux du kit et de l'éditeur reçoivent l'environnement du kit scellé, sans les programmes des agents (à leur place, les relais vers la VM) : un projet n'en reçoit que ce qui est à lui.",
            "Das Terminal eines Projekts, seine Aktionen, Websites und Tasks, auf dem PC mit oder ohne VM, und die Terminals des Kits und des Editors erhalten die Umgebung des Kits versiegelt, ohne die Programme der Agenten (an ihrer Stelle die Weiterleitungen in die VM): Ein Projekt bekommt daraus nur, was ihm gehört.",
          )}
        </P>
        <Bullets
          items={[
            tr(
              "Fuori le variabili che portano al tuo profilo vero (CP_USER_PROFILE) e, su Linux, al portachiavi della sessione (DBUS_SESSION_BUS_ADDRESS); il codice dei progetti perde anche le tue chiavi SSH (SSH_AUTH_SOCK), che restano solo nei terminali dove scrivi tu, per Git.",
              "Out go the variables leading to your real profile (CP_USER_PROFILE) and, on Linux, to the session keyring (DBUS_SESSION_BUS_ADDRESS); the projects' code also loses your SSH keys (SSH_AUTH_SOCK), which stay only in the terminals where you type, for Git.",
              "Dehors les variables qui mènent à votre vrai profil (CP_USER_PROFILE) et, sur Linux, au trousseau de la session (DBUS_SESSION_BUS_ADDRESS) ; le code des projets perd aussi vos clés SSH (SSH_AUTH_SOCK), qui ne restent que dans les terminaux où vous tapez, pour Git.",
              "Draußen bleiben die Variablen, die zu deinem echten Profil (CP_USER_PROFILE) und unter Linux zum Schlüsselbund der Sitzung (DBUS_SESSION_BUS_ADDRESS) führen; der Code der Projekte verliert auch deine SSH-Schlüssel (SSH_AUTH_SOCK), die nur in den Terminals bleiben, in denen du tippst, für Git.",
            ),
            tr(
              "Il client mysql entra come l'utente del progetto, sul suo database, mai come amministratore: l'accesso di amministratore resta ai server del kit e a phpMyAdmin.",
              "The mysql client signs in as the project's user, on its database, never as the administrator: the administrator login stays with the kit's servers and phpMyAdmin.",
              "Le client mysql se connecte en tant qu'utilisateur du projet, sur sa base, jamais en administrateur : l'accès administrateur reste aux serveurs du kit et à phpMyAdmin.",
              "Der mysql-Client meldet sich als Benutzer des Projekts an, auf dessen Datenbank, nie als Administrator: Die Administrator-Anmeldung bleibt bei den Servern des Kits und bei phpMyAdmin.",
            ),
            tr(
              "Restano l'accesso di GitHub CLI del kit (serve a Git su HTTPS) e la sessione grafica, perché un progetto può aprire finestre.",
              "The kit's GitHub CLI sign-in stays (Git over HTTPS needs it), and so does the graphical session, since a project can open windows.",
              "Restent la connexion de GitHub CLI du kit (Git en HTTPS en a besoin) et la session graphique, car un projet peut ouvrir des fenêtres.",
              "Es bleiben die Anmeldung von GitHub CLI des Kits (Git über HTTPS braucht sie) und die grafische Sitzung, denn ein Projekt kann Fenster öffnen.",
            ),
            tr(
              "Nella VM non arriva nulla di questo ambiente: solo gli strumenti del kit per Linux e le variabili delle versioni del progetto.",
              "Nothing of this environment reaches the VM: only the kit's tools for Linux and the variables of the project's versions.",
              "Rien de cet environnement n'arrive dans la VM : seulement les outils du kit pour Linux et les variables des versions du projet.",
              "Nichts von dieser Umgebung gelangt in die VM: nur die Werkzeuge des Kits für Linux und die Variablen der Versionen des Projekts.",
            ),
            tr(
              "Quando l'app legge un progetto con Git, il .git/config del progetto non può avviare programmi suoi (core.fsmonitor spento).",
              "When the app reads a project with Git, the project's .git/config cannot start programs of its own (core.fsmonitor off).",
              "Quand l'application lit un projet avec Git, le .git/config du projet ne peut pas lancer ses propres programmes (core.fsmonitor désactivé).",
              "Wenn die App ein Projekt mit Git liest, kann die .git/config des Projekts keine eigenen Programme starten (core.fsmonitor aus).",
            ),
          ]}
        />
        <P>
          {rich(
            tr(
              "Sul PC questo è tutto: i programmi dei progetti girano con il tuo utente e leggono ciò che leggi tu, con o senza la VM. Gli agenti invece girano nella VM e vedono solo la cartella del progetto: {0}.",
              "On the PC this is all: the projects' programs run as your user and read what you read, with or without the VM. The agents instead run in the VM and see only the project's folder: {0}.",
              "Sur le PC, c'est tout : les programmes des projets tournent avec votre utilisateur et lisent ce que vous lisez, avec ou sans la VM. Les agents, eux, tournent dans la VM et ne voient que le dossier du projet : {0}.",
              "Auf dem PC ist das alles: Die Programme der Projekte laufen mit deinem Benutzer und lesen, was du liest, mit oder ohne VM. Die Agenten dagegen laufen in der VM und sehen nur den Ordner des Projekts: {0}.",
            ),
            <See key="v" id="vm" />,
          )}
        </P>
      </Section>

      <Section
        title={tr("Che cosa resta sul PC", "What stays on the PC", "Ce qui reste sur le PC", "Was auf dem PC bleibt")}
      >
        <P>
          {tr(
            "Il kit non scrive nel registro di Windows, non cambia il PATH del sistema, non crea servizi né operazioni pianificate. Le eccezioni, dichiarate:",
            "The kit does not write to the Windows registry, does not change the system PATH, and creates no services or scheduled tasks. The exceptions, declared:",
            "Le kit n'écrit pas dans le registre de Windows, ne modifie pas le PATH du système, ne crée ni services ni tâches planifiées. Les exceptions, déclarées :",
            "Das Kit schreibt nicht in die Windows-Registry, ändert nicht den PATH des Systems und legt weder Dienste noch geplante Aufgaben an. Die Ausnahmen, offen angegeben:",
          )}
        </P>
        <Bullets
          items={[
            tr(
              "La cartella temporanea del sistema, in Atollo\\<id del kit>: area di lavoro dei download, file consegnati a mano, temporanei dei programmi. Viene pulita alla chiusura e ai successivi avvii.",
              "The system's temporary folder, in Atollo\\<kit id>: the downloads' work area, files handed over manually, programs' temporary files. It is cleaned on closing and at the following starts.",
              "Le dossier temporaire du système, dans Atollo\\<id du kit> : zone de travail des téléchargements, fichiers remis à la main, fichiers temporaires des programmes. Il est nettoyé à la fermeture et aux démarrages suivants.",
              "Der temporäre Ordner des Systems, in Atollo\\<Kit-ID>: Arbeitsbereich der Downloads, manuell übergebene Dateien, temporäre Dateien der Programme. Er wird beim Schließen und bei den folgenden Starts bereinigt.",
            ),
            tr(
              "Mentre Antigravity è aperto, il suo accesso sta nel portachiavi del sistema (su Windows solo per la sessione); alla chiusura torna nel kit.",
              "While Antigravity is open, its sign-in is in the system keychain (on Windows only for the session); on closing it goes back into the kit.",
              "Tant qu'Antigravity est ouvert, sa connexion se trouve dans le trousseau du système (sur Windows seulement pour la session) ; à la fermeture, elle revient dans le kit.",
              "Solange Antigravity geöffnet ist, liegt seine Anmeldung im Schlüsselbund des Systems (unter Windows nur für die Sitzung); beim Schließen kehrt sie ins Kit zurück.",
            ),
            tr(
              "Una cartella vuota che la PowerShell crea sempre in %LOCALAPPDATA%\\Microsoft\\PowerShell.",
              "An empty folder that PowerShell always creates in %LOCALAPPDATA%\\Microsoft\\PowerShell.",
              "Un dossier vide que PowerShell crée toujours dans %LOCALAPPDATA%\\Microsoft\\PowerShell.",
              "Ein leerer Ordner, den PowerShell immer in %LOCALAPPDATA%\\Microsoft\\PowerShell anlegt.",
            ),
            tr(
              "Se esegui gh auth login a mano in una console, GitHub CLI può salvare l'accesso nella Gestione credenziali di Windows: accedi invece dalle Impostazioni.",
              "If you run gh auth login manually in a console, GitHub CLI may save the sign-in in the Windows Credential Manager: sign in from Settings instead.",
              "Si vous exécutez gh auth login à la main dans une console, GitHub CLI peut enregistrer la connexion dans le Gestionnaire d'identification de Windows : connectez-vous plutôt depuis les Paramètres.",
              "Wenn du gh auth login manuell in einer Konsole ausführst, kann GitHub CLI die Anmeldung in der Windows-Anmeldeinformationsverwaltung speichern: Melde dich stattdessen über die Einstellungen an.",
            ),
            tr(
              "Su macOS: i dati della finestra in ~/Library/WebKit.",
              "On macOS: the window's data in ~/Library/WebKit.",
              "Sur macOS : les données de la fenêtre dans ~/Library/WebKit.",
              "Unter macOS: die Daten des Fensters in ~/Library/WebKit.",
            ),
            tr(
              "Per la VM dei progetti: l'hypervisor attivato da te, l'accesso a Docker nel portachiavi, una giunzione e una cartella nella cartella temporanea del sistema e, su Ubuntu, un profilo AppArmor (l'elenco completo nell'argomento sulla VM).",
              "For the projects' VM: the hypervisor you turned on, the Docker sign-in in the keychain, a junction and a folder in the system's temporary folder and, on Ubuntu, an AppArmor profile (the full list in the topic about the VM).",
              "Pour la VM des projets : l'hyperviseur que vous avez activé, la connexion à Docker dans le trousseau, une jonction et un dossier dans le dossier temporaire du système et, sur Ubuntu, un profil AppArmor (la liste complète dans le sujet sur la VM).",
              "Für die VM der Projekte: der von dir eingeschaltete Hypervisor, die Docker-Anmeldung im Schlüsselbund, eine Verknüpfung und ein Ordner im temporären Ordner des Systems und unter Ubuntu ein AppArmor-Profil (die vollständige Liste im Thema über die VM).",
            ),
          ]}
        />
        <Callout
          tone="info"
          title={tr(
            "Diritti di amministratore",
            "Administrator rights",
            "Droits d'administrateur",
            "Administratorrechte",
          )}
        >
          {tr(
            "Atollo parte sempre come utente normale e non esegue mai un comando da amministratore. Le uniche cose che lo chiedono sono esterne al kit e le fai tu, una volta: attivare l'hypervisor per la VM (o, su Linux, il gruppo kvm e il profilo AppArmor), Visual Studio Build Tools per il compilatore MSVC di Rust, gli Xcode Command Line Tools su macOS per alcuni programmi C/C++.",
            "Atollo always starts as a normal user and never runs an administrator command. The only things that need one are outside the kit and are yours to do, once: turning on the hypervisor for the VM (or, on Linux, the kvm group and the AppArmor profile), Visual Studio Build Tools for Rust's MSVC compiler, the Xcode Command Line Tools on macOS for some C/C++ programs.",
            "Atollo démarre toujours en tant qu'utilisateur normal et n'exécute jamais de commande d'administrateur. Les seules choses qui en demandent un sont extérieures au kit et c'est vous qui les faites, une fois : activer l'hyperviseur pour la VM (ou, sur Linux, le groupe kvm et le profil AppArmor), Visual Studio Build Tools pour le compilateur MSVC de Rust, les Xcode Command Line Tools sur macOS pour certains programmes C/C++.",
            "Atollo startet immer als normaler Benutzer und führt nie einen Administratorbefehl aus. Die einzigen Dinge, die einen verlangen, liegen außerhalb des Kits und machst du selbst, einmal: den Hypervisor für die VM einschalten (oder unter Linux die Gruppe kvm und das AppArmor-Profil), Visual Studio Build Tools für den MSVC-Compiler von Rust, die Xcode Command Line Tools unter macOS für einige C/C++-Programme.",
          )}
        </Callout>
      </Section>

      <Under>
        <Bullets
          items={[
            tr(
              "L'ambiente parte vuoto e copia dal PC circa 40 variabili di sistema (ComSpec, SystemRoot, PROCESSOR_ARCHITECTURE, LANG, DISPLAY…) più le variabili del proxy; il kit ne imposta un centinaio. Atollo env su Windows stampa circa 130 righe.",
              "The environment starts empty and copies about 40 system variables from the PC (ComSpec, SystemRoot, PROCESSOR_ARCHITECTURE, LANG, DISPLAY…) plus the proxy variables; the kit sets about a hundred. Atollo env on Windows prints about 130 lines.",
              "L'environnement part vide et copie depuis le PC environ 40 variables système (ComSpec, SystemRoot, PROCESSOR_ARCHITECTURE, LANG, DISPLAY…) plus les variables du proxy ; le kit en définit une centaine. Atollo env sur Windows affiche environ 130 lignes.",
              "Die Umgebung startet leer und übernimmt vom PC etwa 40 Systemvariablen (ComSpec, SystemRoot, PROCESSOR_ARCHITECTURE, LANG, DISPLAY…) plus die Proxy-Variablen; das Kit setzt etwa hundert. Atollo env gibt unter Windows etwa 130 Zeilen aus.",
            ),
            tr(
              "Ogni programma figlio parte con l'ambiente svuotato e poi riempito con quello del kit: niente filtra dal PC.",
              "Every child program starts with the environment emptied and then filled with the kit's: nothing leaks in from the PC.",
              "Chaque programme enfant démarre avec l'environnement vidé puis rempli avec celui du kit : rien ne filtre du PC.",
              "Jedes Kindprogramm startet mit geleerter Umgebung, die dann mit der des Kits gefüllt wird: Vom PC sickert nichts durch.",
            ),
            tr(
              "Anche il processo dell'app reindirizza la propria cartella utente e i temporanei, prima di avviare qualsiasi cosa.",
              "The app's process too redirects its own user folder and temporary files, before starting anything.",
              "Le processus de l'application redirige lui aussi son propre dossier utilisateur et ses fichiers temporaires, avant de lancer quoi que ce soit.",
              "Auch der Prozess der App leitet seinen eigenen Benutzerordner und die temporären Dateien um, bevor er irgendetwas startet.",
            ),
            tr(
              "Le variabili che contengono KEY, TOKEN, SECRET o PASSWORD sono mascherate nel comando env e nel log.",
              "Variables containing KEY, TOKEN, SECRET or PASSWORD are masked in the env command and in the log.",
              "Les variables qui contiennent KEY, TOKEN, SECRET ou PASSWORD sont masquées dans la commande env et dans le journal.",
              "Variablen, die KEY, TOKEN, SECRET oder PASSWORD enthalten, werden im Befehl env und im Log maskiert.",
            ),
          ]}
        />
      </Under>

      <Related ids={["vm", "disk-layout", "lifecycle", "move"]} />
    </>
  ),

  "disk-layout": ({ info }) => (
    <>
      <InBrief
        points={[
          tr(
            "Tutto sta nella cartella del kit: l'eseguibile, projects (tuo) e data (del kit).",
            "Everything is in the kit folder: the executable, projects (yours) and data (the kit's).",
            "Tout se trouve dans le dossier du kit : l'exécutable, projects (à vous) et data (au kit).",
            "Alles liegt im Kit-Ordner: die ausführbare Datei, projects (deiner) und data (der des Kits).",
          ),
          tr(
            "In data i programmi sono separati per sistema; il resto è in comune fra Windows, macOS e Linux.",
            "In data the programs are separated by system; the rest is shared between Windows, macOS and Linux.",
            "Dans data, les programmes sont séparés par système ; le reste est commun à Windows, macOS et Linux.",
            "In data sind die Programme nach System getrennt; der Rest ist Windows, macOS und Linux gemeinsam.",
          ),
          tr(
            "Le cartelle con password sono leggibili solo dal tuo utente (sui dischi NTFS).",
            "The folders with passwords can be read only by your user (on NTFS drives).",
            "Les dossiers avec des mots de passe ne sont lisibles que par votre utilisateur (sur les disques NTFS).",
            "Die Ordner mit Passwörtern sind nur für deinen Benutzer lesbar (auf NTFS-Laufwerken).",
          ),
        ]}
      />

      <Section title={tr("Le cartelle", "The folders", "Les dossiers", "Die Ordner")}>
        <KitFoldersBar />
        <Table
          columns={[
            tr("Cartella o file", "Folder or file", "Dossier ou fichier", "Ordner oder Datei"),
            tr("Che cosa contiene", "What it contains", "Ce qu'il contient", "Was er enthält"),
          ]}
          rows={diskTree().map((n) => [
            <Code key="p" indent={n.depth}>
              {n.path}
            </Code>,
            n.text,
          ])}
        />
        <KitInfo
          info={info}
          show="root"
          text={tr("Il tuo kit è in {0}.", "Your kit is in {0}.", "Votre kit est dans {0}.", "Dein Kit liegt in {0}.")}
        />
      </Section>

      <Section
        title={tr(
          "Che cosa è tuo e che cosa è del kit",
          "What is yours and what is the kit's",
          "Ce qui est à vous et ce qui est au kit",
          "Was dir gehört und was dem Kit",
        )}
      >
        <Table
          columns={[
            tr(
              "Puoi modificare a mano",
              "You can edit by hand",
              "Vous pouvez modifier à la main",
              "Du kannst von Hand ändern",
            ),
            tr(
              "Meglio lasciare al kit",
              "Better left to the kit",
              "Mieux vaut laisser au kit",
              "Besser dem Kit überlassen",
            ),
          ]}
          rows={[
            [
              tr(
                "Tutto in projects; data\\agents (istruzioni, skill, mcp.json); data\\home\\.gitconfig",
                "Everything in projects; data\\agents (instructions, skills, mcp.json); data\\home\\.gitconfig",
                "Tout dans projects ; data\\agents (instructions, skills, mcp.json) ; data\\home\\.gitconfig",
                "Alles in projects; data\\agents (Anweisungen, Skills, mcp.json); data\\home\\.gitconfig",
              ),
              tr(
                "data\\app, data\\apps, data\\kit, data\\servers, data\\db: li gestisce il kit, e alcuni vengono riscritti a ogni avvio",
                "data\\app, data\\apps, data\\kit, data\\servers, data\\db: the kit manages them, and some are rewritten at every start",
                "data\\app, data\\apps, data\\kit, data\\servers, data\\db : le kit les gère, et certains sont réécrits à chaque démarrage",
                "data\\app, data\\apps, data\\kit, data\\servers, data\\db: Das Kit verwaltet sie, und einige werden bei jedem Start neu geschrieben",
              ),
            ],
          ]}
        />
        <Callout tone="warn">
          {tr(
            "Non cancellare a mano le cartelle dei database (data\\db) o le cartelle private dei progetti: perderesti dati e password. Per togliere un componente usa la pagina Strumenti; per pulire i dati di un progetto usa «Dati senza progetto».",
            "Do not delete the database folders (data\\db) or the projects' private folders by hand: you would lose data and passwords. To remove a component use the Tools page; to clean up a project's data use “Data without a project”.",
            "N'effacez pas à la main les dossiers des bases de données (data\\db) ni les dossiers privés des projets : vous perdriez données et mots de passe. Pour retirer un composant, utilisez la page Outils ; pour nettoyer les données d'un projet, utilisez « Données sans projet ».",
            "Lösche die Ordner der Datenbanken (data\\db) oder die privaten Ordner der Projekte nicht von Hand: Du würdest Daten und Passwörter verlieren. Um eine Komponente zu entfernen, verwende die Seite Werkzeuge; um die Daten eines Projekts zu bereinigen, verwende „Daten ohne Projekt“.",
          )}
        </Callout>
      </Section>

      <Under>
        <Bullets
          items={[
            tr(
              "Cartelle protette (solo il tuo utente, su NTFS; permessi 700 su macOS e Linux): data\\app\\db, data\\app\\secrets, data\\projects\\<progetto>\\private, data\\agents\\claude, data\\tmp quando è la cartella temporanea di Claude.",
              "Protected folders (only your user, on NTFS; permissions 700 on macOS and Linux): data\\app\\db, data\\app\\secrets, data\\projects\\<project>\\private, data\\agents\\claude, data\\tmp when it is Claude's temporary folder.",
              "Dossiers protégés (seulement votre utilisateur, sur NTFS ; permissions 700 sur macOS et Linux) : data\\app\\db, data\\app\\secrets, data\\projects\\<projet>\\private, data\\agents\\claude, data\\tmp quand c'est le dossier temporaire de Claude.",
              "Geschützte Ordner (nur dein Benutzer, auf NTFS; Berechtigungen 700 unter macOS und Linux): data\\app\\db, data\\app\\secrets, data\\projects\\<Projekt>\\private, data\\agents\\claude, data\\tmp, wenn es der temporäre Ordner von Claude ist.",
            ),
            tr(
              "Su exFAT e FAT32 i permessi non esistono: il kit funziona, ma quelle cartelle non sono protette.",
              "On exFAT and FAT32 permissions do not exist: the kit works, but those folders are not protected.",
              "Sur exFAT et FAT32, les permissions n'existent pas : le kit fonctionne, mais ces dossiers ne sont pas protégés.",
              "Unter exFAT und FAT32 gibt es keine Berechtigungen: Das Kit funktioniert, aber diese Ordner sind nicht geschützt.",
            ),
            tr(
              "I file di configurazione vengono scritti in modo sicuro: prima un file temporaneo, poi la sostituzione, così un'interruzione non lascia file a metà.",
              "Configuration files are written safely: first a temporary file, then the replacement, so an interruption does not leave half-written files.",
              "Les fichiers de configuration sont écrits de façon sûre : d'abord un fichier temporaire, puis le remplacement, ainsi une interruption ne laisse pas de fichiers à moitié écrits.",
              "Konfigurationsdateien werden sicher geschrieben: zuerst eine temporäre Datei, dann das Ersetzen, damit eine Unterbrechung keine halb geschriebenen Dateien hinterlässt.",
            ),
            tr(
              "data\\kit viene riscritta a ogni avvio: non metterci file tuoi.",
              "data\\kit is rewritten at every start: do not put your own files there.",
              "data\\kit est réécrit à chaque démarrage : n'y mettez pas vos propres fichiers.",
              "data\\kit wird bei jedem Start neu geschrieben: Leg dort keine eigenen Dateien ab.",
            ),
          ]}
        />
      </Under>

      <Related ids={["isolation", "updates", "project-cleanup"]} />
    </>
  ),

  lifecycle: () => (
    <>
      <InBrief
        points={[
          tr(
            "Quando apri Atollo partono solo le cose che hai chiesto: i database e i server con «Avvia con Atollo».",
            "When you open Atollo only the things you asked for start: the databases and servers with “Start with Atollo”.",
            "Quand vous ouvrez Atollo, seules les choses que vous avez demandées démarrent : les bases de données et les serveurs avec « Démarrer avec Atollo ».",
            "Wenn du Atollo öffnest, starten nur die Dinge, die du verlangt hast: die Datenbanken und Server mit „Mit Atollo starten“.",
          ),
          tr(
            "Ogni programma avviato dal kit è legato all'app: se l'app si chiude, anche nel modo peggiore, si chiudono con lei.",
            "Every program started by the kit is tied to the app: if the app closes, even in the worst way, they close with it.",
            "Chaque programme lancé par le kit est lié à l'application : si l'application se ferme, même de la pire façon, ils se ferment avec elle.",
            "Jedes vom Kit gestartete Programm ist an die App gebunden: Wenn die App sich schließt, selbst auf die schlimmste Weise, schließen sie sich mit ihr.",
          ),
          tr(
            "Una sola finestra per cartella del kit: aprirla di nuovo porta in primo piano quella esistente.",
            "Only one window per kit folder: opening it again brings the existing one to the front.",
            "Une seule fenêtre par dossier du kit : la rouvrir met au premier plan celle qui existe.",
            "Nur ein Fenster pro Kit-Ordner: Ein erneutes Öffnen holt das vorhandene in den Vordergrund.",
          ),
        ]}
      />

      <Section title={tr("All'avvio", "At start", "Au démarrage", "Beim Start")}>
        <Flow
          steps={[
            {
              title: tr("Cartelle", "Folders", "Dossiers", "Ordner"),
              text: tr(
                "Crea projects e data se mancano e controlla di poter scrivere.",
                "Creates projects and data if they are missing and checks that it can write.",
                "Crée projects et data s'ils manquent et vérifie qu'il peut écrire.",
                "Legt projects und data an, falls sie fehlen, und prüft, ob es schreiben kann.",
              ),
            },
            {
              title: tr("Posto", "Place", "Emplacement", "Ort"),
              text: tr(
                "Si accorge se il kit è stato spostato e lo annota.",
                "Notices if the kit has been moved and takes note.",
                "S'aperçoit si le kit a été déplacé et le note.",
                "Bemerkt, ob das Kit verschoben wurde, und vermerkt es.",
              ),
            },
            {
              title: tr("Ambiente", "Environment", "Environnement", "Umgebung"),
              text: tr(
                "Legge le Impostazioni e costruisce l'ambiente del kit.",
                "Reads Settings and builds the kit's environment.",
                "Lit les Paramètres et construit l'environnement du kit.",
                "Liest die Einstellungen und baut die Umgebung des Kits auf.",
              ),
            },
            {
              title: tr("Finestra", "Window", "Fenêtre", "Fenster"),
              text: tr(
                "Una sola per cartella; poi apre la finestra.",
                "Only one per folder; then it opens the window.",
                "Une seule par dossier ; puis ouvre la fenêtre.",
                "Nur eines pro Ordner; dann öffnet es das Fenster.",
              ),
            },
            {
              title: tr("Sottofondo", "Background", "Arrière-plan", "Hintergrund"),
              text: tr(
                "Avvia ciò che è segnato «Avvia con Atollo», allinea gli agenti e Git, controlla aggiornamenti.",
                "Starts what is marked “Start with Atollo”, syncs the agents and Git, checks for updates.",
                "Lance ce qui est marqué « Démarrer avec Atollo », aligne les agents et Git, vérifie les mises à jour.",
                "Startet, was mit „Mit Atollo starten“ markiert ist, gleicht die Agenten und Git ab, sucht nach Updates.",
              ),
            },
          ]}
        />
        <Bullets
          items={[
            tr(
              "I server di database e i profili dei server web con «Avvia con Atollo» partono subito dopo l'apertura.",
              "Database servers and web server profiles with “Start with Atollo” start right after opening.",
              "Les serveurs de bases de données et les profils de serveurs web avec « Démarrer avec Atollo » démarrent juste après l'ouverture.",
              "Datenbankserver und Webserver-Profile mit „Mit Atollo starten“ starten direkt nach dem Öffnen.",
            ),
            tr(
              "I componenti obbligatori mancanti e quelli da reinstallare vengono installati da soli; al primo avvio no: lì si scarica tutto solo con «Installa» della pagina «Benvenuto».",
              "Missing required components and those to reinstall are installed automatically; not at the first start: there everything is downloaded only with “Install” on the “Welcome” page.",
              "Les composants obligatoires manquants et ceux à réinstaller sont installés automatiquement ; pas au premier lancement : là, tout n'est téléchargé qu'avec « Installer » de la page « Bienvenue ».",
              "Fehlende Pflichtkomponenten und solche, die neu installiert werden müssen, werden automatisch installiert; beim ersten Start nicht: Dort wird alles erst mit „Installieren“ auf der Seite „Willkommen“ heruntergeladen.",
            ),
            tr(
              "Istruzioni, skill e server MCP degli agenti vengono allineati; se il kit è stato spostato, i percorsi vengono aggiornati.",
              "The agents' instructions, skills and MCP servers are synced; if the kit has been moved, the paths are updated.",
              "Les instructions, skills et serveurs MCP des agents sont alignés ; si le kit a été déplacé, les chemins sont mis à jour.",
              "Anweisungen, Skills und MCP-Server der Agenten werden abgeglichen; wenn das Kit verschoben wurde, werden die Pfade aktualisiert.",
            ),
            tr(
              "Git segue la cartella del kit (safe.directory) e l'account GitHub: se «Chi firma i commit» è vuoto, nome e indirizzo privato vengono dall'account.",
              "Git follows the kit folder (safe.directory) and the GitHub account: if “Who signs the commits” is empty, the name and private address come from the account.",
              "Git suit le dossier du kit (safe.directory) et le compte GitHub : si « Qui signe les commits » est vide, le nom et l'adresse privée viennent du compte.",
              "Git folgt dem Kit-Ordner (safe.directory) und dem GitHub-Konto: Wenn „Wer die Commits signiert“ leer ist, kommen Name und private Adresse vom Konto.",
            ),
            tr(
              "Dopo 10 secondi, le versioni offerte da ogni fonte, per i menu che ne scaricano una (al massimo una volta al giorno); poi, con «Aggiornamento automatico» attivo, l'ultima versione di ogni componente installato, per il riquadro «Aggiornamenti» di Strumenti (anche questa al massimo una volta al giorno).",
              "After 10 seconds, the versions each source offers, for the menus that download one (at most once a day); then, with “Automatic update” on, the latest version of each installed component, for the “Updates” pane of Tools (also at most once a day).",
              "Après 10 secondes, les versions proposées par chaque source, pour les menus qui en téléchargent une (au plus une fois par jour) ; puis, avec « Mise à jour automatique » activée, la dernière version de chaque composant installé, pour le volet « Mises à jour » d'Outils (au plus une fois par jour aussi).",
              "Nach 10 Sekunden die Versionen, die jede Quelle anbietet, für die Menüs, die eine herunterladen (höchstens einmal am Tag); danach, mit „Automatisches Update“ an, die neueste Version jeder installierten Komponente, für den Bereich „Aktualisierungen“ von Werkzeuge (ebenfalls höchstens einmal am Tag).",
            ),
            tr(
              "Dopo 20 secondi, se attivo, il controllo degli aggiornamenti di Atollo (spegnere «Aggiornamento automatico» prima lo ferma).",
              "After 20 seconds, if turned on, the check for Atollo updates (turning “Automatic update” off before then stops it).",
              "Après 20 secondes, si elle est activée, la vérification des mises à jour d'Atollo (désactiver « Mise à jour automatique » avant l'annule).",
              "Nach 20 Sekunden, falls aktiviert, die Prüfung auf Updates von Atollo (wenn du „Automatisches Update“ vorher ausschaltest, wird sie gestoppt).",
            ),
          ]}
        />
        <Callout tone="info">
          {tr(
            "Aprire la pagina Database non avvia i database: partono con «Avvia», con «Avvia con Atollo» o quando un progetto ne ha bisogno (per creare il suo database o fare una copia).",
            "Opening the Databases page does not start the databases: they start with “Start”, with “Start with Atollo” or when a project needs them (to create its database or make a backup).",
            "Ouvrir la page Bases de données ne démarre pas les bases de données : elles démarrent avec « Démarrer », avec « Démarrer avec Atollo » ou quand un projet en a besoin (pour créer sa base de données ou faire une copie).",
            "Das Öffnen der Seite Datenbanken startet die Datenbanken nicht: Sie starten mit „Starten“, mit „Mit Atollo starten“ oder wenn ein Projekt sie braucht (um seine Datenbank anzulegen oder eine Sicherung zu erstellen).",
          )}
        </Callout>
      </Section>

      <Section title={tr("Mentre lavori", "While you work", "Pendant que vous travaillez", "Während du arbeitest")}>
        <Table
          columns={[
            tr("Che cosa", "What", "Quoi", "Was"),
            tr("Quando parte", "When it starts", "Quand ça démarre", "Wann es startet"),
            tr("Quando si ferma", "When it stops", "Quand ça s'arrête", "Wann es stoppt"),
          ]}
          rows={[
            [
              tr("Azioni dei progetti", "Project actions", "Actions des projets", "Aktionen der Projekte"),
              tr("Con «Avvia» o ▶", "With “Start” or ▶", "Avec « Démarrer » ou ▶", "Mit „Starten“ oder ▶"),
              tr(
                "Con «Ferma», o alla chiusura dell'app",
                "With “Stop”, or when the app closes",
                "Avec « Arrêter », ou à la fermeture de l'application",
                "Mit „Beenden“ oder beim Schließen der App",
              ),
            ],
            [
              tr("Server web condivisi", "Shared web servers", "Serveurs web partagés", "Gemeinsame Webserver"),
              tr(
                "Con il primo progetto che li usa, dalla pagina Server Web o con «Avvia con Atollo»",
                "With the first project that uses them, from the Web Servers page or with “Start with Atollo”",
                "Avec le premier projet qui les utilise, depuis la page Serveurs web ou avec « Démarrer avec Atollo »",
                "Mit dem ersten Projekt, das sie nutzt, über die Seite Webserver oder mit „Mit Atollo starten“",
              ),
              tr(
                "Con l'ultimo progetto, salvo se avviati dalla pagina Server Web o con «Mantieni attivo anche senza progetti»; con «Ferma»; alla chiusura",
                "With the last project, unless started from the Web Servers page or with “Keep running even without projects”; with “Stop”; on closing",
                "Avec le dernier projet, sauf s'ils ont été démarrés depuis la page Serveurs web ou avec « Garder actif même sans projets » ; avec « Arrêter » ; à la fermeture",
                "Mit dem letzten Projekt, außer wenn über die Seite Webserver gestartet oder mit „Auch ohne Projekte aktiv lassen“; mit „Beenden“; beim Schließen",
              ),
            ],
            [
              tr("Database", "Databases", "Bases de données", "Datenbanken"),
              tr(
                "Con «Avvia», «Avvia con Atollo» o quando un progetto li chiede",
                "With “Start”, “Start with Atollo” or when a project asks for them",
                "Avec « Démarrer », « Démarrer avec Atollo » ou quand un projet les demande",
                "Mit „Starten“, „Mit Atollo starten“ oder wenn ein Projekt sie anfordert",
              ),
              tr(
                "Con «Ferma», o alla chiusura (arresto ordinato)",
                "With “Stop”, or on closing (orderly shutdown)",
                "Avec « Arrêter », ou à la fermeture (arrêt propre)",
                "Mit „Beenden“ oder beim Schließen (geordnetes Herunterfahren)",
              ),
            ],
            [
              tr("Editor", "Editor", "Éditeur", "Editor"),
              tr(
                "Con il primo «Editor»",
                "With the first “Editor”",
                "Avec le premier « Éditeur »",
                "Mit dem ersten „Editor“",
              ),
              tr(
                "Alla chiusura dell'app",
                "When the app closes",
                "À la fermeture de l'application",
                "Beim Schließen der App",
              ),
            ],
            [
              tr("Console e agenti", "Consoles and agents", "Consoles et agents", "Konsolen und Agenten"),
              tr(
                "Con «Terminale», «Apri», i loghi degli agenti",
                "With “Terminal”, “Open”, the agent logos",
                "Avec « Terminal », « Ouvrir », les logos des agents",
                "Mit „Terminal“, „Öffnen“, den Logos der Agenten",
              ),
              tr(
                "Quando le chiudi, o alla chiusura dell'app",
                "When you close them, or when the app closes",
                "Quand vous les fermez, ou à la fermeture de l'application",
                "Wenn du sie schließt oder beim Schließen der App",
              ),
            ],
            [
              tr("VM (Docker Sandboxes)", "VM (Docker Sandboxes)", "VM (Docker Sandboxes)", "VM (Docker Sandboxes)"),
              tr(
                "Lo stato si controlla all'avvio e ogni minuto; una microVM parte con il primo terminale, azione o agente che la usa",
                "The status is checked at start and every minute; a microVM starts with the first terminal, action or agent that uses it",
                "L'état est vérifié au démarrage puis chaque minute ; une microVM démarre avec le premier terminal, action ou agent qui l'utilise",
                "Der Zustand wird beim Start und jede Minute geprüft; eine microVM startet mit dem ersten Terminal, der ersten Aktion oder dem ersten Agenten, der sie nutzt",
              ),
              tr(
                "Alla chiusura dell'app e con «Rimuovi l'unità», insieme al servizio di Docker Sandboxes del kit",
                "When the app closes and with “Remove drive”, together with the kit's Docker Sandboxes service",
                "À la fermeture de l'application et avec « Retirer le lecteur », avec le service Docker Sandboxes du kit",
                "Beim Schließen der App und mit „Laufwerk entfernen“, zusammen mit dem Docker-Sandboxes-Dienst des Kits",
              ),
            ],
            [
              tr(
                "Server dei modelli locali",
                "Local models server",
                "Serveur des modèles locaux",
                "Server der lokalen Modelle",
              ),
              tr(
                "Con «Avvia», «Avvia con Atollo» o aprendo Pi su un modello locale",
                "With “Start”, “Start with Atollo” or by opening Pi on a local model",
                "Avec « Démarrer », « Démarrer avec Atollo » ou en ouvrant Pi sur un modèle local",
                "Mit „Starten“, „Mit Atollo starten“ oder durch Öffnen von Pi mit einem lokalen Modell",
              ),
              tr(
                "Con «Ferma», «Ferma tutto», o alla chiusura; un modello lascia la memoria dopo dieci minuti senza richieste",
                "With “Stop”, “Stop everything”, or on closing; a model leaves the memory after ten minutes without requests",
                "Avec « Arrêter », « Tout arrêter », ou à la fermeture ; un modèle quitte la mémoire après dix minutes sans requêtes",
                "Mit „Beenden“, „Alles stoppen“ oder beim Schließen; ein Modell verlässt den Speicher nach zehn Minuten ohne Anfragen",
              ),
            ],
          ]}
        />
        <P>
          {tr(
            "Chiudere una console non spegne i database: restano accesi finché non li fermi o chiudi l'app. Il pannello Attività (dalla barra di stato) elenca le azioni dei progetti, ogni versione di database avviata, i pool attivi e il server dei modelli locali; «Ferma tutto» li ferma dopo una conferma.",
            "Closing a console does not shut down the databases: they stay on until you stop them or close the app. The Activity panel (from the status bar) lists the project actions, every running database version, the active pools and the local models server; “Stop everything” stops them after a confirmation.",
            "Fermer une console n'arrête pas les bases de données : elles restent allumées jusqu'à ce que vous les arrêtiez ou fermiez l'application. Le panneau Activité (depuis la barre d'état) liste les actions des projets, chaque version de base de données démarrée, les pools actifs et le serveur des modèles locaux ; « Tout arrêter » les arrête après une confirmation.",
            "Das Schließen einer Konsole fährt die Datenbanken nicht herunter: Sie bleiben an, bis du sie stoppst oder die App schließt. Das Panel Aktivität (über die Statusleiste) listet die Aktionen der Projekte, jede gestartete Datenbankversion, die aktiven Pools und den Server der lokalen Modelle auf; „Alles stoppen“ stoppt sie nach einer Bestätigung.",
          )}
        </P>
      </Section>

      <Section title={tr("Alla chiusura", "On closing", "À la fermeture", "Beim Schließen")}>
        <P>
          {rich(
            tr(
              "Se qualcosa è aperto, Atollo chiede «Chiudere Atollo?» con l'elenco; «Chiudi tutto» ferma le azioni, i server web e il server dei modelli locali e spegne con ordine i database e Docker Sandboxes con le microVM, poi esce. Prima di staccare l'unità usa {0}.",
              "If something is open, Atollo asks “Close Atollo?” with the list; “Close everything” stops the actions, the web servers and the local models server and shuts down the databases and Docker Sandboxes with the microVMs neatly, then exits. Before unplugging the drive use {0}.",
              "Si quelque chose est ouvert, Atollo demande « Fermer Atollo ? » avec la liste ; « Tout fermer » arrête les actions, les serveurs web et le serveur des modèles locaux et éteint proprement les bases de données et Docker Sandboxes avec les microVM, puis quitte. Avant de débrancher le lecteur, utilisez {0}.",
              "Wenn etwas geöffnet ist, fragt Atollo „Atollo schließen?“ mit der Liste; „Alles schließen“ stoppt die Aktionen, die Webserver und den Server der lokalen Modelle, fährt die Datenbanken und Docker Sandboxes mit den microVMs geordnet herunter und beendet sich dann. Bevor du das Laufwerk abziehst, verwende {0}.",
            ),
            <See key="e" id="eject" />,
          )}
        </P>
      </Section>

      <Under>
        <Bullets
          items={[
            tr(
              "Su Windows tutti i programmi del kit vivono in un Job Object con l'opzione «chiudi alla chiusura»: se l'app termina in qualsiasi modo, Windows chiude anche loro. Editor e azioni hanno un Job loro, dentro quello dell'app.",
              "On Windows all the kit's programs live in a Job Object with the “kill on close” option: if the app ends in any way, Windows closes them too. The editor and actions have their own Job, inside the app's.",
              "Sur Windows, tous les programmes du kit vivent dans un Job Object avec l'option « fermer à la fermeture » : si l'application se termine de quelque façon que ce soit, Windows les ferme aussi. L'éditeur et les actions ont leur propre Job, à l'intérieur de celui de l'application.",
              "Unter Windows leben alle Programme des Kits in einem Job Object mit der Option „beim Schließen beenden“: Wenn die App auf irgendeine Weise endet, schließt Windows auch sie. Editor und Aktionen haben einen eigenen Job innerhalb dessen der App.",
            ),
            tr(
              "Su macOS e Linux un piccolo processo di guardia si accorge se l'app muore e chiude i gruppi di processi del kit (SIGTERM, poi SIGKILL dopo 5 secondi).",
              "On macOS and Linux a small watchdog process notices if the app dies and closes the kit's process groups (SIGTERM, then SIGKILL after 5 seconds).",
              "Sur macOS et Linux, un petit processus de surveillance s'aperçoit si l'application meurt et ferme les groupes de processus du kit (SIGTERM, puis SIGKILL après 5 secondes).",
              "Unter macOS und Linux bemerkt ein kleiner Wächterprozess, wenn die App abstürzt, und schließt die Prozessgruppen des Kits (SIGTERM, dann SIGKILL nach 5 Sekunden).",
            ),
            tr(
              "L'istanza unica usa un identificativo ricavato dalla cartella del kit (un mutex su Windows, un lock su macOS e Linux): due kit diversi non si disturbano.",
              "The single instance uses an identifier derived from the kit folder (a mutex on Windows, a lock on macOS and Linux): two different kits do not get in each other's way.",
              "L'instance unique utilise un identifiant tiré du dossier du kit (un mutex sur Windows, un verrou sur macOS et Linux) : deux kits différents ne se gênent pas.",
              "Die Einzelinstanz verwendet eine aus dem Kit-Ordner abgeleitete Kennung (ein Mutex unter Windows, ein Lock unter macOS und Linux): Zwei verschiedene Kits stören sich nicht.",
            ),
            tr(
              "Ogni porta ha un solo proprietario: un pool (di base da 8081 a 8999), una versione di database, un'azione di un progetto (di base da 7100 a 7999), il server dei modelli locali (9931 e seguenti); da 7000 a 7099 sono dell'editor. Una porta già presa viene rifiutata con il nome di chi la tiene (per esempio «La porta 3306 è già del database MySQL 8.4: scegline un'altra.»), e la porta automatica di un nuovo pool le evita tutte.",
              "Every port has only one owner: a pool (by default 8081 to 8999), a database version, a project action (by default 7100 to 7999), the local models server (9931 and the following ones); 7000 to 7099 belong to the editor. A port already taken is refused with the name of whoever holds it (for example “Port 3306 already belongs to database MySQL 8.4: choose another one.”), and a new pool's automatic port avoids them all.",
              "Chaque port n'a qu'un seul propriétaire : un pool (par défaut de 8081 à 8999), une version de base de données, une action d'un projet (par défaut de 7100 à 7999), le serveur des modèles locaux (9931 et les suivants) ; de 7000 à 7099, ils sont à l'éditeur. Un port déjà pris est refusé avec le nom de celui qui le tient (par exemple « Le port 3306 appartient déjà à la base de données MySQL 8.4 : choisissez-en un autre. »), et le port automatique d'un nouveau pool les évite tous.",
              "Jeder Port hat nur einen Besitzer: einen Pool (standardmäßig 8081 bis 8999), eine Datenbankversion, eine Aktion eines Projekts (standardmäßig 7100 bis 7999), den Server der lokalen Modelle (9931 und folgende); 7000 bis 7099 gehören dem Editor. Ein bereits vergebener Port wird mit dem Namen dessen abgelehnt, der ihn hält (zum Beispiel „Port 3306 gehört bereits zur Datenbank MySQL 8.4: Wähle einen anderen.“), und der automatische Port eines neuen Pools meidet sie alle.",
            ),
            tr(
              "Ogni versione di un database tiene la sua porta, anche quando cambi la versione del server; due avvii della stessa versione fanno partire un solo server, sulla stessa porta.",
              "Every version of a database keeps its own port, also when you change the server's version; two starts of the same version run only one server, on the same port.",
              "Chaque version d'une base de données garde son propre port, même quand vous changez la version du serveur ; deux démarrages de la même version ne lancent qu'un seul serveur, sur le même port.",
              "Jede Version einer Datenbank behält ihren eigenen Port, auch wenn du die Version des Servers wechselst; zwei Starts derselben Version starten nur einen Server, auf demselben Port.",
            ),
            tr(
              "Browser ed Esplora file vengono aperti fuori dal Job: restano aperti anche dopo la chiusura di Atollo.",
              "The browser and File Explorer are opened outside the Job: they stay open even after Atollo closes.",
              "Le navigateur et l'Explorateur de fichiers sont ouverts hors du Job : ils restent ouverts même après la fermeture d'Atollo.",
              "Browser und Datei-Explorer werden außerhalb des Jobs geöffnet: Sie bleiben auch nach dem Schließen von Atollo offen.",
            ),
            tr(
              "Alla chiusura, se c'è un aggiornamento pronto, l'eseguibile viene sostituito.",
              "On closing, if an update is ready, the executable is replaced.",
              "À la fermeture, si une mise à jour est prête, l'exécutable est remplacé.",
              "Beim Schließen wird die ausführbare Datei ersetzt, wenn ein Update bereitsteht.",
            ),
          ]}
        />
      </Under>

      <Related ids={["eject", "isolation", "databases"]} />
    </>
  ),

  sources: ({ components }) => (
    <>
      <InBrief
        points={[
          tr(
            "Ogni componente arriva dalla fonte ufficiale del suo produttore, in HTTPS: niente mirror, niente copie di terzi.",
            "Every component comes from its maker's official source, over HTTPS: no mirrors, no third-party copies.",
            "Chaque composant vient de la source officielle de son éditeur, en HTTPS : pas de miroirs, pas de copies de tiers.",
            "Jede Komponente kommt aus der offiziellen Quelle ihres Herstellers, über HTTPS: keine Mirrors, keine Kopien Dritter.",
          ),
          tr(
            "Ogni file viene verificato con la sua impronta crittografica prima di essere usato; se non coincide, viene scartato.",
            "Every file is verified with its cryptographic hash before it is used; if it does not match, it is discarded.",
            "Chaque fichier est vérifié avec son empreinte cryptographique avant d'être utilisé ; si elle ne correspond pas, il est écarté.",
            "Jede Datei wird vor der Verwendung mit ihrem kryptografischen Hash geprüft; stimmt er nicht überein, wird sie verworfen.",
          ),
          tr(
            "Se un download non riesce, puoi scaricare il file con il browser e consegnarlo al kit, che lo verifica allo stesso modo.",
            "If a download fails, you can download the file with the browser and hand it to the kit, which verifies it the same way.",
            "Si un téléchargement échoue, vous pouvez télécharger le fichier avec le navigateur et le remettre au kit, qui le vérifie de la même façon.",
            "Wenn ein Download fehlschlägt, kannst du die Datei mit dem Browser herunterladen und dem Kit übergeben, das sie auf dieselbe Weise prüft.",
          ),
        ]}
      />

      <Section
        title={tr(
          "Il percorso di un download",
          "The path of a download",
          "Le parcours d'un téléchargement",
          "Der Weg eines Downloads",
        )}
      >
        <Flow
          steps={[
            {
              title: tr("Chiede", "Request", "Demande", "Anfrage"),
              text: tr(
                "Qual è l'ultima versione e qual è la sua impronta, alla fonte ufficiale.",
                "Which is the latest version and what its hash is, from the official source.",
                "À la source officielle, quelle est la dernière version et quelle est son empreinte.",
                "Welches die neueste Version ist und wie ihr Hash lautet, bei der offiziellen Quelle.",
              ),
            },
            {
              title: tr("Scarica", "Download", "Téléchargement", "Download"),
              text: tr(
                "In un file .part nella cartella di lavoro; tre tentativi.",
                "Into a .part file in the work folder; three attempts.",
                "Dans un fichier .part dans le dossier de travail ; trois tentatives.",
                "In eine .part-Datei im Arbeitsordner; drei Versuche.",
              ),
            },
            {
              title: tr("Verifica", "Check", "Vérification", "Prüfung"),
              text: tr(
                "Calcola l'impronta del file e la confronta con quella attesa.",
                "Computes the file's hash and compares it with the expected one.",
                "Calcule l'empreinte du fichier et la compare avec celle attendue.",
                "Berechnet den Hash der Datei und vergleicht ihn mit dem erwarteten.",
              ),
            },
            {
              title: tr("Estrae", "Extract", "Extraction", "Entpacken"),
              text: tr(
                "zip, tar.gz, tar.xz, 7z o file singolo.",
                "zip, tar.gz, tar.xz, 7z or single file.",
                "zip, tar.gz, tar.xz, 7z ou fichier unique.",
                "zip, tar.gz, tar.xz, 7z oder Einzeldatei.",
              ),
            },
            {
              title: tr("Sostituisce", "Replace", "Remplacement", "Ersetzen"),
              text: tr(
                "Mette la cartella nuova al posto della vecchia, tutta insieme.",
                "Puts the new folder in place of the old one, all at once.",
                "Met le nouveau dossier à la place de l'ancien, d'un seul coup.",
                "Setzt den neuen Ordner an die Stelle des alten, alles auf einmal.",
              ),
            },
            {
              title: tr("Prova", "Test", "Test", "Test"),
              text: tr(
                "Esegue il programma per controllare che risponda.",
                "Runs the program to check that it responds.",
                "Exécute le programme pour contrôler qu'il répond.",
                "Führt das Programm aus, um zu prüfen, ob es antwortet.",
              ),
            },
          ]}
        />
        <Table
          columns={[
            tr("Verifica", "Check", "Vérification", "Prüfung"),
            tr("Che cosa vuol dire", "What it means", "Ce que cela veut dire", "Was es bedeutet"),
            tr("Esempi", "Examples", "Exemples", "Beispiele"),
          ]}
          rows={[
            [
              tr(
                "SHA-256, SHA-512, SHA3-256",
                "SHA-256, SHA-512, SHA3-256",
                "SHA-256, SHA-512, SHA3-256",
                "SHA-256, SHA-512, SHA3-256",
              ),
              tr(
                "L'impronta del file deve essere quella pubblicata dal produttore (o fissata nel catalogo del kit).",
                "The file's hash must be the one published by the maker (or pinned in the kit's catalog).",
                "L'empreinte du fichier doit être celle publiée par l'éditeur (ou fixée dans le catalogue du kit).",
                "Der Hash der Datei muss der vom Hersteller veröffentlichte sein (oder der im Katalog des Kits festgelegte).",
              ),
              tr(
                "Python, Node.js, Go, Maven, Caddy, SQLite, l'editor, Pi, llama.cpp, i modelli locali",
                "Python, Node.js, Go, Maven, Caddy, SQLite, the editor, Pi, llama.cpp, the local models",
                "Python, Node.js, Go, Maven, Caddy, SQLite, l'éditeur, Pi, llama.cpp, les modèles locaux",
                "Python, Node.js, Go, Maven, Caddy, SQLite, der Editor, Pi, llama.cpp, die lokalen Modelle",
              ),
            ],
            [
              tr("npm sha512", "npm sha512", "npm sha512", "npm sha512"),
              tr(
                "L'impronta di integrità del registro npm.",
                "The integrity hash of the npm registry.",
                "L'empreinte d'intégrité du registre npm.",
                "Der Integritäts-Hash der npm-Registry.",
              ),
              tr("Claude Code, Codex", "Claude Code, Codex", "Claude Code, Codex", "Claude Code, Codex"),
            ],
            [
              tr(
                "Authenticode (solo Windows)",
                "Authenticode (Windows only)",
                "Authenticode (Windows uniquement)",
                "Authenticode (nur Windows)",
              ),
              tr(
                "In più, la firma digitale del produttore sul programma, con la verifica della revoca.",
                "In addition, the maker's digital signature on the program, with the revocation check.",
                "En plus, la signature numérique de l'éditeur sur le programme, avec la vérification de la révocation.",
                "Zusätzlich die digitale Signatur des Herstellers auf dem Programm, mit Prüfung auf Widerruf.",
              ),
              tr(
                "Claude Code, Visual C++ runtime",
                "Claude Code, Visual C++ runtime",
                "Claude Code, Visual C++ runtime",
                "Claude Code, Visual C++ runtime",
              ),
            ],
          ]}
        />
        <Callout tone="tip">
          {tr(
            "Ogni parte di un componente (per esempio PHP e il suo Composer) viene scaricata e verificata prima di toccare quella installata: se qualcosa va storto, la versione che avevi resta com'era.",
            "Every part of a component (for example PHP and its Composer) is downloaded and verified before touching the installed one: if something goes wrong, the version you had stays as it was.",
            "Chaque partie d'un composant (par exemple PHP et son Composer) est téléchargée et vérifiée avant de toucher à celle installée : si quelque chose se passe mal, la version que vous aviez reste telle quelle.",
            "Jeder Teil einer Komponente (zum Beispiel PHP und sein Composer) wird heruntergeladen und geprüft, bevor der installierte angetastet wird: Wenn etwas schiefgeht, bleibt die Version, die du hattest, wie sie war.",
          )}
        </Callout>
      </Section>

      <Section
        title={tr(
          "Quando un download non riesce: «a mano»",
          "When a download fails: “manually”",
          "Quand un téléchargement échoue : « à la main »",
          "Wenn ein Download fehlschlägt: „von Hand“",
        )}
      >
        <P>
          {tr(
            "Se dopo tre tentativi il download fallisce (proxy che blocca, sito irraggiungibile), sotto la riga del componente compare un riquadro:",
            "If the download fails after three attempts (a blocking proxy, an unreachable site), a panel appears under the component's row:",
            "Si le téléchargement échoue après trois tentatives (proxy qui bloque, site injoignable), un panneau apparaît sous la ligne du composant :",
            "Wenn der Download nach drei Versuchen fehlschlägt (blockierender Proxy, nicht erreichbare Website), erscheint unter der Zeile der Komponente ein Bereich:",
          )}
        </P>
        <Steps
          steps={[
            tr(
              "Fai clic su «Scaricalo dal browser: <nome del file>»: si apre l'indirizzo esatto nel browser.",
              "Click “Download it in the browser: <file name>”: the exact address opens in the browser.",
              "Cliquez sur « Téléchargez-le dans le navigateur : <nom du fichier> » : l'adresse exacte s'ouvre dans le navigateur.",
              "Klicke auf „Im Browser herunterladen: <Dateiname>“: Die genaue Adresse öffnet sich im Browser.",
            ),
            tr(
              "Quando il file è scaricato, trascinalo nel riquadro o usa «sfoglia…».",
              "When the file has downloaded, drag it onto the panel or use “browse…”.",
              "Quand le fichier est téléchargé, faites-le glisser dans le panneau ou utilisez « parcourez… ».",
              "Wenn die Datei heruntergeladen ist, ziehe sie in den Bereich oder verwende „durchsuchen…“.",
            ),
            tr(
              "Il kit lo conserva e lo usa solo se la sua impronta è quella pubblicata; l'installazione riparte da sola («Aggiunto …: nuovo tentativo.»).",
              "The kit keeps it and uses it only if its hash is the published one; the installation restarts by itself (“Added …: trying again.”).",
              "Le kit le conserve et ne l'utilise que si son empreinte est celle publiée ; l'installation reprend toute seule (« … ajouté : nouvel essai. »).",
              "Das Kit bewahrt sie auf und verwendet sie nur, wenn ihr Hash der veröffentlichte ist; die Installation startet von selbst neu („… hinzugefügt: neuer Versuch.“).",
            ),
          ]}
        />
        <P>
          {tr(
            "Il riquadro compare solo quando l'impronta del file è nota. I file consegnati e non usati vengono cancellati dopo 30 giorni.",
            "The panel appears only when the file's hash is known. Files handed over and not used are deleted after 30 days.",
            "Le panneau n'apparaît que quand l'empreinte du fichier est connue. Les fichiers remis et non utilisés sont supprimés après 30 jours.",
            "Der Bereich erscheint nur, wenn der Hash der Datei bekannt ist. Übergebene, nicht verwendete Dateien werden nach 30 Tagen gelöscht.",
          )}
        </P>
      </Section>

      <Section
        title={tr(
          "GitHub e i limiti dell'API",
          "GitHub and the API limits",
          "GitHub et les limites de l'API",
          "GitHub und die Grenzen der API",
        )}
      >
        <P>
          {rich(
            tr(
              "Molti componenti sono pubblicati su GitHub. Senza accesso, l'API di GitHub consente 60 richieste l'ora; con l'account del kit ({0}) o con una variabile GITHUB_TOKEN del PC, 5.000. Il kit conserva le risposte e, se il limite è raggiunto, legge l'ultima versione direttamente dalle pagine di GitHub.",
              "Many components are published on GitHub. Without a sign-in, the GitHub API allows 60 requests an hour; with the kit's account ({0}) or with a GITHUB_TOKEN variable on the PC, 5,000. The kit keeps the responses and, if the limit is reached, reads the latest version straight from the GitHub pages.",
              "Beaucoup de composants sont publiés sur GitHub. Sans connexion, l'API de GitHub permet 60 requêtes par heure ; avec le compte du kit ({0}) ou avec une variable GITHUB_TOKEN du PC, 5 000. Le kit conserve les réponses et, si la limite est atteinte, lit la dernière version directement dans les pages de GitHub.",
              "Viele Komponenten werden auf GitHub veröffentlicht. Ohne Anmeldung erlaubt die GitHub-API 60 Anfragen pro Stunde; mit dem Konto des Kits ({0}) oder mit einer Variable GITHUB_TOKEN des PCs 5.000. Das Kit bewahrt die Antworten auf und liest, wenn das Limit erreicht ist, die neueste Version direkt von den GitHub-Seiten.",
            ),
            <See key="g" id="github" />,
          )}
        </P>
      </Section>

      <Section
        title={tr(
          "Altri servizi che l'app contatta",
          "Other services the app contacts",
          "Autres services que l'application contacte",
          "Weitere Dienste, die die App kontaktiert",
        )}
      >
        <Table
          columns={[tr("Servizio", "Service", "Service", "Dienst"), tr("Perché", "Why", "Pourquoi", "Warum")]}
          rows={[
            [
              <Code key="a">api.github.com/repos/nabre/atollo/releases</Code>,
              tr(
                "Gli aggiornamenti di Atollo.",
                "Atollo's updates.",
                "Les mises à jour d'Atollo.",
                "Die Updates von Atollo.",
              ),
            ],
            [
              <Code key="b">open-vsx.org</Code>,
              tr(
                "Le estensioni dell'editor.",
                "The editor's extensions.",
                "Les extensions de l'éditeur.",
                "Die Erweiterungen des Editors.",
              ),
            ],
            [
              <Code key="c">api.anthropic.com · chatgpt.com · cloudcode-pa.googleapis.com</Code>,
              tr(
                "I limiti dei piani degli agenti, con l'accesso salvato, solo in lettura e solo con la pagina AI aperta.",
                "The limits of the agents' plans, with the saved sign-in, read-only and only with the AI page open.",
                "Les limites des forfaits des agents, avec la connexion enregistrée, en lecture seule et seulement avec la page IA ouverte.",
                "Die Limits der Tarife der Agenten, mit der gespeicherten Anmeldung, nur lesend und nur bei geöffneter Seite KI.",
              ),
            ],
            [
              <Code key="h">huggingface.co</Code>,
              tr(
                "I modelli locali: la ricerca dei file GGUF e i download, che huggingface.co reindirizza alla sua CDN. Solo quando scarichi un modello; il token, se c'è, va solo a huggingface.co.",
                "The local models: the search for GGUF files and the downloads, which huggingface.co redirects to its CDN. Only when you download a model; the token, if any, goes only to huggingface.co.",
                "Les modèles locaux : la recherche des fichiers GGUF et les téléchargements, que huggingface.co redirige vers son CDN. Seulement quand vous téléchargez un modèle ; le jeton, s'il y en a un, ne va qu'à huggingface.co.",
                "Die lokalen Modelle: die Suche nach GGUF-Dateien und die Downloads, die huggingface.co an sein CDN weiterleitet. Nur wenn du ein Modell herunterlädst; das Token, falls vorhanden, geht nur an huggingface.co.",
              ),
            ],
            [
              <Code key="g">github.com/ggml-org/llama.cpp · github.com/earendil-works/pi</Code>,
              tr(
                "Le release di llama.cpp e di Pi, come per gli altri componenti pubblicati su GitHub (sono nella tabella delle fonti qui sotto).",
                "The releases of llama.cpp and Pi, as for the other components published on GitHub (they are in the table of sources below).",
                "Les releases de llama.cpp et de Pi, comme pour les autres composants publiés sur GitHub (ils sont dans le tableau des sources ci-dessous).",
                "Die Releases von llama.cpp und Pi, wie bei den anderen auf GitHub veröffentlichten Komponenten (sie stehen in der Tabelle der Quellen unten).",
              ),
            ],
            [
              <Code key="i">github.com/nabre/atollo/issues</Code>,
              tr(
                "«Segnala un problema» apre qui una segnalazione nuova nel browser, con versione e sistema nell'indirizzo. Lo apri tu, nel tuo browser: l'app non manda niente, e nulla arriva a GitHub finché non crei la segnalazione.",
                "“Report a problem” opens a new issue here in the browser, with version and system in the address. You open it, in your browser: the app sends nothing, and nothing reaches GitHub until you create the issue.",
                "« Signaler un problème » ouvre ici une nouvelle issue dans le navigateur, avec la version et le système dans l'adresse. C'est vous qui l'ouvrez, dans votre navigateur : l'application n'envoie rien, et rien n'arrive à GitHub tant que vous ne créez pas l'issue.",
                "„Problem melden“ öffnet hier ein neues Issue im Browser, mit Version und System in der Adresse. Du öffnest es in deinem Browser: Die App sendet nichts, und nichts erreicht GitHub, bevor du das Issue erstellst.",
              ),
            ],
            [
              <Code key="d">registry.npmjs.org · pypi.org · packagist.org…</Code>,
              tr(
                "Solo quando lo chiedono i comandi dei progetti e dei framework.",
                "Only when the commands of projects and frameworks ask for it.",
                "Seulement quand les commandes des projets et des frameworks le demandent.",
                "Nur wenn die Befehle der Projekte und Frameworks es verlangen.",
              ),
            ],
          ]}
        />
      </Section>

      <Section
        title={tr(
          "Tabella completa delle fonti ({0} componenti e {1} parti)",
          "Complete table of sources ({0} components and {1} parts)",
          "Tableau complet des sources ({0} composants et {1} parties)",
          "Vollständige Tabelle der Quellen ({0} Komponenten und {1} Teile)",
        )
          .replace("{0}", String(sourceCounts().components))
          .replace("{1}", String(sourceCounts().parts))}
      >
        <P>
          {tr(
            "Per ogni componente: da dove il kit legge le versioni, da dove scarica il file, come lo verifica e dove lo mette. Le colonne descrivono Windows; dove macOS e Linux usano un'altra fonte, è scritta sotto l'indirizzo.",
            "For each component: where the kit reads the versions from, where it downloads the file from, how it verifies it and where it puts it. The columns describe Windows; where macOS and Linux use another source, it is written under the address.",
            "Pour chaque composant : d'où le kit lit les versions, d'où il télécharge le fichier, comment il le vérifie et où il le met. Les colonnes décrivent Windows ; quand macOS et Linux utilisent une autre source, elle est écrite sous l'adresse.",
            "Für jede Komponente: woher das Kit die Versionen liest, woher es die Datei herunterlädt, wie es sie prüft und wohin es sie legt. Die Spalten beschreiben Windows; wo macOS und Linux eine andere Quelle verwenden, steht sie unter der Adresse.",
          )}
        </P>
        <SourcesTable rows={TOOL_SOURCES} categories={sourceCategories()} components={components} />
      </Section>

      <Under>
        <Bullets
          items={[
            tr(
              "Cartella di lavoro: <temporanei del sistema>\\Atollo\\<id del kit>\\setup, oppure data\\tmp\\setup se il disco di sistema ha meno di 4 GB liberi o non si può scrivere.",
              "Work folder: <system temp>\\Atollo\\<kit id>\\setup, or data\\tmp\\setup if the system disk has less than 4 GB free or cannot be written to.",
              "Dossier de travail : <temporaires du système>\\Atollo\\<id du kit>\\setup, ou data\\tmp\\setup si le disque système a moins de 4 Go libres ou n'est pas accessible en écriture.",
              "Arbeitsordner: <System-Temp>\\Atollo\\<Kit-ID>\\setup, oder data\\tmp\\setup, wenn das Systemlaufwerk weniger als 4 GB frei hat oder nicht beschreibbar ist.",
            ),
            tr(
              "Tre tentativi a 5 e 10 secondi di distanza; il download riparte da capo a ogni tentativo.",
              "Three attempts 5 and 10 seconds apart; the download starts over at every attempt.",
              "Trois tentatives à 5 et 10 secondes d'intervalle ; le téléchargement repart de zéro à chaque tentative.",
              "Drei Versuche im Abstand von 5 und 10 Sekunden; der Download beginnt bei jedem Versuch von vorn.",
            ),
            tr(
              "Connessioni con il proxy delle Impostazioni e con i certificati del sistema più quelli di «Certificati aggiuntivi (file PEM)»: l'ispezione HTTPS di un firewall funziona se il suo certificato è installato nel PC o è in quel file.",
              "Connections use the proxy of Settings and the system's certificates plus those of “Additional certificates (PEM file)”: a firewall's HTTPS inspection works if its certificate is installed on the PC or is in that file.",
              "Connexions avec le proxy des Paramètres et les certificats du système plus ceux de « Certificats supplémentaires (fichier PEM) » : l'inspection HTTPS d'un pare-feu fonctionne si son certificat est installé sur le PC ou se trouve dans ce fichier.",
              "Verbindungen mit dem Proxy der Einstellungen und den Zertifikaten des Systems plus denen von „Zusätzliche Zertifikate (PEM-Datei)“: Die HTTPS-Untersuchung einer Firewall funktioniert, wenn ihr Zertifikat auf dem PC installiert ist oder in dieser Datei steht.",
            ),
            tr(
              "La versione installata viene scritta in data\\app\\versions\\<sistema>\\<componente>.txt.",
              "The installed version is written in data\\app\\versions\\<system>\\<component>.txt.",
              "La version installée est écrite dans data\\app\\versions\\<système>\\<composant>.txt.",
              "Die installierte Version wird in data\\app\\versions\\<System>\\<Komponente>.txt geschrieben.",
            ),
          ]}
        />
      </Under>

      <Related ids={["tools", "network", "github"]} />
    </>
  ),

  cli: () => (
    <>
      <InBrief
        points={[
          tr(
            "L'eseguibile di Atollo è anche un programma a riga di comando: con un comando, fa il lavoro e esce, senza aprire la finestra.",
            "Atollo's executable is also a command-line program: given a command, it does the job and exits, without opening the window.",
            "L'exécutable d'Atollo est aussi un programme en ligne de commande : avec une commande, il fait le travail et quitte, sans ouvrir la fenêtre.",
            "Die ausführbare Datei von Atollo ist auch ein Befehlszeilenprogramm: Mit einem Befehl erledigt sie die Arbeit und beendet sich, ohne das Fenster zu öffnen.",
          ),
          tr(
            "Utile per preparare un kit per un altro sistema, controllare i componenti, o vedere l'ambiente.",
            "Useful to prepare a kit for another system, check the components, or see the environment.",
            "Utile pour préparer un kit pour un autre système, contrôler les composants ou voir l'environnement.",
            "Nützlich, um ein Kit für ein anderes System vorzubereiten, die Komponenten zu prüfen oder die Umgebung anzusehen.",
          ),
          tr(
            "Funziona anche mentre l'app è aperta.",
            "It works even while the app is open.",
            "Fonctionne aussi pendant que l'application est ouverte.",
            "Funktioniert auch, während die App geöffnet ist.",
          ),
        ]}
      />

      <Section title={tr("Come si usa", "How to use it", "Comment l'utiliser", "So verwendest du es")}>
        <P>
          {tr(
            "Apri una console nella cartella del kit. Su Windows, in PowerShell, aggiungi | Out-Host perché l'output compaia nella console (l'eseguibile è un programma con finestra); nel Prompt dei comandi non serve. Su macOS e Linux usa il percorso del programma (per esempio ./Atollo.AppImage).",
            "Open a console in the kit folder. On Windows, in PowerShell, add | Out-Host so the output appears in the console (the executable is a windowed program); in Command Prompt it is not needed. On macOS and Linux use the program's path (for example ./Atollo.AppImage).",
            "Ouvrez une console dans le dossier du kit. Sur Windows, dans PowerShell, ajoutez | Out-Host pour que la sortie apparaisse dans la console (l'exécutable est un programme à fenêtre) ; dans l'Invite de commandes, ce n'est pas nécessaire. Sur macOS et Linux, utilisez le chemin du programme (par exemple ./Atollo.AppImage).",
            "Öffne eine Konsole im Kit-Ordner. Unter Windows fügst du in PowerShell | Out-Host hinzu, damit die Ausgabe in der Konsole erscheint (die ausführbare Datei ist ein Programm mit Fenster); in der Eingabeaufforderung ist das nicht nötig. Unter macOS und Linux verwendest du den Pfad des Programms (zum Beispiel ./Atollo.AppImage).",
          )}
        </P>
        <GuideCode title={tr("Esempio", "Example", "Exemple", "Beispiel")} code={CLI_EXAMPLE} />
      </Section>

      <Section title={tr("I comandi", "The commands", "Les commandes", "Die Befehle")}>
        <Table
          columns={[
            tr("Comando", "Command", "Commande", "Befehl"),
            tr("Che cosa fa", "What it does", "Ce qu'elle fait", "Was er tut"),
          ]}
          rows={[
            [
              <Code key="a">{`env [--project ${tr("<nome>", "<name>", "<nom>", "<Name>")}]`}</Code>,
              tr(
                "Stampa l'ambiente del kit (segreti mascherati); con un progetto, con le sue versioni.",
                "Prints the kit's environment (secrets masked); with a project, with its versions.",
                "Affiche l'environnement du kit (secrets masqués) ; avec un projet, avec ses versions.",
                "Gibt die Umgebung des Kits aus (Geheimnisse maskiert); mit einem Projekt, mit dessen Versionen.",
              ),
            ],
            [
              <Code key="b">{`components [--target ${tr("<sistema>", "<system>", "<système>", "<System>")}]`}</Code>,
              tr(
                "Elenca i componenti: scelto o no, gruppo, versione installata o perché non è disponibile.",
                "Lists the components: chosen or not, group, installed version or why it is not available.",
                "Liste les composants : choisi ou non, groupe, version installée ou raison de l'indisponibilité.",
                "Listet die Komponenten auf: gewählt oder nicht, Gruppe, installierte Version oder warum sie nicht verfügbar ist.",
              ),
            ],
            [
              <Code key="c">versions [&lt;id&gt;]</Code>,
              tr(
                "Senza id: le versioni su disco degli strumenti installati. Con un id: chiede alla fonte le versioni offerte e le salva.",
                "Without id: the versions on disk of the installed tools. With an id: asks the source for the versions offered and saves them.",
                "Sans id : les versions sur disque des outils installés. Avec un id : demande à la source les versions proposées et les enregistre.",
                "Ohne ID: die Versionen der installierten Werkzeuge auf dem Datenträger. Mit einer ID: fragt die Quelle nach den angebotenen Versionen und speichert sie.",
              ),
            ],
            [
              <Code key="d">check</Code>,
              tr(
                "Chiede a ogni componente installato la sua versione e segna ✓ o ✗.",
                "Asks every installed component for its version and marks ✓ or ✗.",
                "Demande à chaque composant installé sa version et marque ✓ ou ✗.",
                "Fragt jede installierte Komponente nach ihrer Version und markiert ✓ oder ✗.",
              ),
            ],
            [
              <Code key="d2">check --kit</Code>,
              tr(
                "La salute del kit: disco, spazio, percorso, permessi, librerie, come «Salute del kit» nelle Impostazioni. Una riga per controllo (√, ! o x) con sotto che cosa fare; finisce con 1 se un controllo è un errore.",
                "The kit's health: drive, space, path, permissions, libraries, like “Kit health” in Settings. One line per check (√, ! or x) with what to do under it; ends with 1 if a check is an error.",
                "La santé du kit : disque, espace, chemin, permissions, bibliothèques, comme « Santé du kit » dans les Paramètres. Une ligne par contrôle (√, ! ou x) avec en dessous ce qu'il faut faire ; se termine par 1 si un contrôle est une erreur.",
                "Der Zustand des Kits: Laufwerk, Platz, Pfad, Rechte, Bibliotheken, wie „Zustand des Kits“ in den Einstellungen. Eine Zeile pro Prüfung (√, ! oder x), darunter, was zu tun ist; endet mit 1, wenn eine Prüfung ein Fehler ist.",
              ),
            ],
            [
              <Code key="d3">check --files</Code>,
              tr(
                "I file dei programmi, come «Verifica i file» nella pagina Strumenti: una riga per cartella (√ a posto, x danneggiata con i percorsi mancanti o cambiati, - senza manifesto); finisce con 1 se una cartella è danneggiata.",
                "The programs' files, like “Check the files” on the Tools page: one line per folder (√ fine, x damaged with the paths missing or changed, - without a manifest); ends with 1 if a folder is damaged.",
                "Les fichiers des programmes, comme « Vérifier les fichiers » dans la page Outils : une ligne par dossier (√ en ordre, x endommagé avec les chemins manquants ou modifiés, - sans manifeste) ; se termine par 1 si un dossier est endommagé.",
                "Die Dateien der Programme, wie „Dateien prüfen“ auf der Seite Werkzeuge: eine Zeile pro Ordner (√ in Ordnung, x beschädigt mit den fehlenden oder geänderten Pfaden, - ohne Manifest); endet mit 1, wenn ein Ordner beschädigt ist.",
              ),
            ],
            [
              <Code key="e">{`prepare ${tr("[opzioni]", "[options]", "[options]", "[Optionen]")}`}</Code>,
              tr(
                "Installa, aggiorna e toglie i componenti secondo la scelta. Le opzioni sono nella tabella sotto.",
                "Installs, updates and removes components according to the choice. The options are in the table below.",
                "Installe, met à jour et retire les composants selon le choix. Les options sont dans le tableau ci-dessous.",
                "Installiert, aktualisiert und entfernt Komponenten gemäß der Auswahl. Die Optionen stehen in der Tabelle unten.",
              ),
            ],
            [
              <Code key="f">{`sync [--project ${tr("<cartella>", "<folder>", "<dossier>", "<Ordner>")}] [--verbose]`}</Code>,
              tr(
                "Allinea istruzioni, skill e server MCP degli agenti.",
                "Syncs the agents' instructions, skills and MCP servers.",
                "Aligne les instructions, skills et serveurs MCP des agents.",
                "Gleicht Anweisungen, Skills und MCP-Server der Agenten ab.",
              ),
            ],
            [
              <Code key="g">{`serve ${tr("<cartella>", "<folder>", "<dossier>", "<Ordner>")} ${tr("<porta>", "<port>", "<port>", "<Port>")}`}</Code>,
              tr(
                "Serve una cartella come sito statico (il server di file del kit).",
                "Serves a folder as a static site (the kit's file server).",
                "Sert un dossier comme site statique (le serveur de fichiers du kit).",
                "Stellt einen Ordner als statische Website bereit (der Dateiserver des Kits).",
              ),
            ],
            [
              <Code key="h">{`vm ${tr("<progetto>", "<project>", "<projet>", "<Projekt>")} [--check | --restart | --refresh | --recreate [--yes]] [--agent claude|codex]`}</Code>,
              tr(
                "Il rapporto della VM del progetto e le sue azioni, come il pannello «VM del progetto»; le azioni solo ad Atollo chiuso.",
                "The report of the project's VM and its actions, as the “Project VM” panel; the actions only with Atollo closed.",
                "Le rapport de la VM du projet et ses actions, comme le panneau « VM du projet » ; les actions seulement Atollo fermé.",
                "Der Bericht der VM des Projekts und ihre Aktionen, wie das Feld „VM des Projekts“; die Aktionen nur bei geschlossenem Atollo.",
              ),
            ],
            [
              <Code key="m">models</Code>,
              tr(
                "I modelli locali: quelli del catalogo e quelli scaricati.",
                "The local models: those of the catalog and those downloaded.",
                "Les modèles locaux : ceux du catalogue et ceux téléchargés.",
                "Die lokalen Modelle: die des Katalogs und die heruntergeladenen.",
              ),
            ],
            [
              <Code key="n">{`models pull ${tr("<modello>", "<model>", "<modèle>", "<Modell>")}`}</Code>,
              tr(
                "Scarica un id del catalogo o un modello di Hugging Face (proprietario/nome:quantizzazione); riprende se interrotto.",
                "Downloads a catalog id or a Hugging Face model (owner/name:quantization); resumes if interrupted.",
                "Télécharge un id du catalogue ou un modèle de Hugging Face (propriétaire/nom:quantification) ; reprend s'il est interrompu.",
                "Lädt eine Katalog-ID oder ein Hugging-Face-Modell herunter (Eigentümer/Name:Quantisierung); setzt nach einer Unterbrechung fort.",
              ),
            ],
            [
              <Code key="o">models rm &lt;id&gt;</Code>,
              tr(
                "Cancella un modello scaricato.",
                "Deletes a downloaded model.",
                "Supprime un modèle téléchargé.",
                "Löscht ein heruntergeladenes Modell.",
              ),
            ],
            [
              <Code key="p">llm start | stop | status</Code>,
              tr(
                "Il server dei modelli locali (chat e agenti): lo avvia, lo ferma o dice se risponde. status non manda la chiave a chi ascolta sulla porta.",
                "The local models server (chat and agents): starts it, stops it or says whether it answers. status does not send the key to whatever listens on the port.",
                "Le serveur des modèles locaux (chat et agents) : le démarre, l'arrête ou dit s'il répond. status n'envoie pas la clé à ce qui écoute sur le port.",
                "Der Server der lokalen Modelle (Chat und Agenten): startet ihn, stoppt ihn oder sagt, ob er antwortet. status sendet den Schlüssel nicht an das, was auf dem Port lauscht.",
              ),
            ],
            [
              <Code key="q">diagnostics</Code>,
              tr(
                "Crea lo zip da allegare a una segnalazione (segreti mascherati) in data\\app\\diagnostics, come «Esporta diagnostica» nelle Impostazioni, e ne stampa il percorso.",
                "Makes the zip to attach to a problem report (secrets masked) in data\\app\\diagnostics, like “Export diagnostics” in Settings, and prints its path.",
                "Crée le zip à joindre à un signalement (secrets masqués) dans data\\app\\diagnostics, comme « Exporter le diagnostic » dans les Paramètres, et en affiche le chemin.",
                "Erstellt das Zip für eine Problemmeldung (Geheimnisse maskiert) in data\\app\\diagnostics, wie „Diagnose exportieren“ in den Einstellungen, und gibt seinen Pfad aus.",
              ),
            ],
            [
              <Code key="i">version · help</Code>,
              tr(
                "La versione (anche --version, -V) e l'aiuto (anche --help, -h, /?).",
                "The version (also --version, -V) and the help (also --help, -h, /?).",
                "La version (aussi --version, -V) et l'aide (aussi --help, -h, /?).",
                "Die Version (auch --version, -V) und die Hilfe (auch --help, -h, /?).",
              ),
            ],
          ]}
        />
        <Table
          columns={[
            tr("Opzione di prepare", "prepare option", "Option de prepare", "Option von prepare"),
            tr("Effetto", "Effect", "Effet", "Wirkung"),
          ]}
          rows={[
            [
              <Code key="a">--only a,b</Code>,
              tr(
                "Solo questi componenti (più ciò di cui hanno bisogno), senza togliere gli altri.",
                "Only these components (plus what they need), without removing the others.",
                "Seulement ces composants (plus ce dont ils ont besoin), sans retirer les autres.",
                "Nur diese Komponenten (plus das, was sie brauchen), ohne die anderen zu entfernen.",
              ),
            ],
            [
              <Code key="b">--skip a,b</Code>,
              tr(
                "Tutti tranne questi (gli obbligatori non si saltano).",
                "All except these (the required ones cannot be skipped).",
                "Tous sauf ceux-ci (les obligatoires ne peuvent pas être sautés).",
                "Alle außer diesen (die Pflichtkomponenten werden nicht übersprungen).",
              ),
            ],
            [
              <Code key="c">--update-all</Code>,
              tr(
                "Aggiorna anche ciò che è già installato.",
                "Also updates what is already installed.",
                "Met aussi à jour ce qui est déjà installé.",
                "Aktualisiert auch, was bereits installiert ist.",
              ),
            ],
            [
              <Code key="d">--force</Code>,
              tr(
                "Reinstalla anche se la versione è già quella.",
                "Reinstalls even if the version is already that one.",
                "Réinstalle même si la version est déjà celle-là.",
                "Installiert neu, auch wenn die Version schon dieselbe ist.",
              ),
            ],
            [
              <Code key="e">--line id=x.y</Code>,
              tr(
                "Aggiunge quella versione accanto alle altre (si può ripetere).",
                "Adds that version next to the others (can be repeated).",
                "Ajoute cette version à côté des autres (peut être répété).",
                "Fügt diese Version neben den anderen hinzu (kann wiederholt werden).",
              ),
            ],
            [
              <Code key="f">{`--target ${tr("<sistema>", "<system>", "<système>", "<System>")}`}</Code>,
              tr(
                "Prepara per un altro sistema: win32-x64, darwin-arm64, linux-x64, linux-arm64.",
                "Prepares for another system: win32-x64, darwin-arm64, linux-x64, linux-arm64.",
                "Prépare pour un autre système : win32-x64, darwin-arm64, linux-x64, linux-arm64.",
                "Bereitet für ein anderes System vor: win32-x64, darwin-arm64, linux-x64, linux-arm64.",
              ),
            ],
          ]}
        />
        <Callout tone="warn">
          {tr(
            "Senza --only, prepare toglie i componenti non scelti, anche dalle cartelle degli altri sistemi. Se non sei sicuro, guarda prima components.",
            "Without --only, prepare removes the components not chosen, from the other systems' folders too. If you are not sure, look at components first.",
            "Sans --only, prepare retire les composants non choisis, y compris des dossiers des autres systèmes. Si vous n'êtes pas sûr, regardez d'abord components.",
            "Ohne --only entfernt prepare die nicht gewählten Komponenten, auch aus den Ordnern der anderen Systeme. Wenn du unsicher bist, sieh dir zuerst components an.",
          )}
        </Callout>
      </Section>

      <Under>
        <Table
          columns={[
            tr("Codice di uscita", "Exit code", "Code de sortie", "Exit-Code"),
            tr("Significato", "Meaning", "Signification", "Bedeutung"),
          ]}
          rows={[
            [<Code key="a">0</Code>, tr("Tutto bene.", "All good.", "Tout va bien.", "Alles in Ordnung.")],
            [
              <Code key="b">1</Code>,
              tr(
                "prepare o check: qualcosa non è riuscito.",
                "prepare or check: something failed.",
                "prepare ou check : quelque chose a échoué.",
                "prepare oder check: Etwas ist fehlgeschlagen.",
              ),
            ],
            [
              <Code key="c">2</Code>,
              tr(
                "Comando sconosciuto (viene stampato anche l'aiuto).",
                "Unknown command (the help is printed too).",
                "Commande inconnue (l'aide est aussi affichée).",
                "Unbekannter Befehl (die Hilfe wird ebenfalls ausgegeben).",
              ),
            ],
            [
              <Code key="d">3</Code>,
              tr(
                "prepare: è già in corso un'altra preparazione dello stesso kit.",
                "prepare: another preparation of the same kit is already running.",
                "prepare : une autre préparation du même kit est déjà en cours.",
                "prepare: Eine andere Vorbereitung desselben Kits läuft bereits.",
              ),
            ],
          ]}
        />
        <P>
          {tr(
            "Dalla riga di comando prepare usa GITHUB_TOKEN o GH_TOKEN della console, se ci sono, per alzare il limite dell'API di GitHub; non usa l'account del kit. Se un download fallisce stampa l'indirizzo da scaricare a mano e la cartella in cui metterlo.",
            "From the command line prepare uses the console's GITHUB_TOKEN or GH_TOKEN, if present, to raise the GitHub API limit; it does not use the kit's account. If a download fails it prints the address to download by hand and the folder to put it in.",
            "En ligne de commande, prepare utilise GITHUB_TOKEN ou GH_TOKEN de la console, s'ils existent, pour relever la limite de l'API de GitHub ; il n'utilise pas le compte du kit. Si un téléchargement échoue, il affiche l'adresse à télécharger à la main et le dossier où le mettre.",
            "Auf der Befehlszeile verwendet prepare GITHUB_TOKEN oder GH_TOKEN der Konsole, falls vorhanden, um das Limit der GitHub-API anzuheben; das Konto des Kits verwendet es nicht. Wenn ein Download fehlschlägt, gibt es die Adresse zum manuellen Herunterladen und den Ordner aus, in den die Datei gehört.",
          )}
        </P>
      </Under>

      <Related ids={["tools", "move", "isolation"]} />
    </>
  ),
};
