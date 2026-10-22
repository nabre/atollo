// Chapter 8 of the Guide, «Come funziona dentro»: isolation, the folders on
// disk, background processes, download sources and checks, the command line.
import { useMemo, useState, type ReactNode } from "react";
import { urlOpen, type ComponentInfo } from "../../api";
import { rich } from "../../i18n";
import { ComponentLogo } from "../../logos";
import { Tag } from "../../tags";
import { Button, Codicon, SearchBox } from "../../workbench";
import { tr } from "../model";
import { sourceCounts, TOOL_SOURCES, type SourceCategory } from "../sources";
import {
  Bullets,
  Callout,
  Code,
  Flow,
  GuideCode,
  InBrief,
  KitFoldersBar,
  P,
  Related,
  Section,
  See,
  Steps,
  Table,
  Under,
  type TopicTexts,
} from "../widgets";

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
      path: tr("apps\\<sistema>\\", "apps\\<system>\\", "apps\\<système>\\", "apps\\<System>\\"),
      depth: 2,
      text: tr(
        "I programmi, per sistema (win32-x64, darwin-arm64…), una cartella per componente e per versione.",
        "The programs, by system (win32-x64, darwin-arm64…), one folder per component and per version.",
        "Les programmes, par système (win32-x64, darwin-arm64…), un dossier par composant et par version.",
        "Die Programme, nach System (win32-x64, darwin-arm64…), ein Ordner pro Komponente und pro Version.",
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
        "L'accesso di Antigravity conservato nel kit (keyring.json).",
        "The Antigravity sign-in kept in the kit (keyring.json).",
        "La connexion d'Antigravity conservée dans le kit (keyring.json).",
        "Die im Kit aufbewahrte Antigravity-Anmeldung (keyring.json).",
      ),
    },
    {
      path: "home\\",
      depth: 2,
      text: tr(
        "La «cartella utente» dei programmi del kit: .claude, .codex, .gemini, .pi (profili degli agenti), .gitconfig, .npmrc, AppData (su Windows).",
        "The “user folder” of the kit's programs: .claude, .codex, .gemini, .pi (agent profiles), .gitconfig, .npmrc, AppData (on Windows).",
        "Le « dossier utilisateur » des programmes du kit : .claude, .codex, .gemini, .pi (profils des agents), .gitconfig, .npmrc, AppData (sur Windows).",
        "Der „Benutzerordner“ der Programme des Kits: .claude, .codex, .gemini, .pi (Profile der Agenten), .gitconfig, .npmrc, AppData (unter Windows).",
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

/** The external sources of every component, with a filter and a search. */
function SourcesTable({ components }: { components: ComponentInfo[] | null }) {
  const [query, setQuery] = useState("");
  const [cat, setCat] = useState<"all" | SourceCategory>("all");
  const byId = useMemo(() => new Map((components ?? []).map((c) => [c.id, c])), [components]);
  const rows = useMemo(() => {
    const q = query.trim().toLowerCase();
    return TOOL_SOURCES.filter(
      (r) =>
        (cat === "all" || r.group === cat) &&
        (!q ||
          [r.id, r.name, r.home, r.metaSource, r.binarySource, r.verifyKind, r.targetPath].some((s) =>
            s.toLowerCase().includes(q),
          )),
    );
  }, [query, cat]);
  const cats: { id: "all" | SourceCategory; label: string }[] = [
    { id: "all", label: `${tr("Tutti", "All", "Tous", "Alle")} (${String(TOOL_SOURCES.length)})` },
    { id: "system", label: tr("Sistema ed editor", "System and editor", "Système et éditeur", "System und Editor") },
    { id: "agents", label: tr("Agenti AI", "AI agents", "Agents IA", "KI-Agenten") },
    { id: "ai", label: tr("Modelli locali", "Local models", "Modèles locaux", "Lokale Modelle") },
    { id: "languages", label: tr("Linguaggi", "Languages", "Langages", "Sprachen") },
    {
      id: "tools",
      label: tr("Strumenti di sviluppo", "Development tools", "Outils de développement", "Entwicklungswerkzeuge"),
    },
    { id: "web", label: tr("Server web", "Web servers", "Serveurs web", "Webserver") },
    { id: "databases", label: tr("Database", "Databases", "Bases de données", "Datenbanken") },
    {
      id: "dbviewers",
      label: tr(
        "Visualizzazione database",
        "Database viewers",
        "Visualisation des bases de données",
        "Datenbank-Ansicht",
      ),
    },
    { id: "part", label: tr("Parti", "Parts", "Parties", "Teile") },
  ];
  return (
    <>
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex flex-wrap items-center gap-1.5">
          {cats.map((c) => (
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
              "Filtra per nome, indirizzo, impronta…",
              "Filter by name, address, hash…",
              "Filtrer par nom, adresse, empreinte…",
              "Nach Name, Adresse, Hash filtern…",
            )}
          />
        </div>
      </div>
      <Table
        columns={[
          tr("Componente", "Component", "Composant", "Komponente"),
          tr("Versioni da", "Versions from", "Versions depuis", "Versionen von"),
          tr("File da", "Files from", "Fichiers depuis", "Dateien von"),
          tr("Verifica", "Check", "Vérification", "Prüfung"),
          tr(
            "In data\\apps\\<sistema>\\",
            "In data\\apps\\<system>\\",
            "Dans data\\apps\\<système>\\",
            "In data\\apps\\<System>\\",
          ),
        ]}
        rows={rows.map((r) => {
          const live = byId.get(r.id);
          const state: ReactNode = live?.installed ? (
            <Tag tone="focus">{live.version ?? tr("installato", "installed", "installé", "installiert")}</Tag>
          ) : live && (!live.available || live.noArm64 || live.unavailableReason !== null) ? (
            <Tag title={live.unavailableReason ?? undefined}>
              {tr("non disponibile qui", "not available here", "non disponible ici", "hier nicht verfügbar")}
            </Tag>
          ) : (
            <Tag>
              {live
                ? tr("non installato", "not installed", "non installé", "nicht installiert")
                : tr("con il suo componente", "with its component", "avec son composant", "mit seiner Komponente")}
            </Tag>
          );
          return [
            <div key="n" className="space-y-1">
              <div className="flex items-center gap-2">
                <ComponentLogo id={r.logoId} size={16} />
                <span className="font-semibold text-vsc-settings-header">{r.name}</span>
              </div>
              <div className="flex flex-wrap items-center gap-1.5 pl-6">
                {state}
                <button
                  type="button"
                  onClick={() => {
                    void urlOpen(r.home);
                  }}
                  title={r.home}
                  className="inline-flex items-center gap-0.5 text-[11px] text-vsc-link hover:underline"
                >
                  {tr("Sito", "Website", "Site", "Website")}
                  <Codicon name="link-external" size={10} />
                </button>
              </div>
            </div>,
            <div key="m" className="font-mono text-[11px] break-all select-text">
              {r.metaSource}
            </div>,
            <div key="b" className="space-y-1">
              <div className="font-mono text-[11px] break-all text-vsc-desc select-text">{r.binarySource}</div>
              {r.unix && <div className="text-[11px] select-text">{r.unix}</div>}
            </div>,
            <div key="v" className="space-y-0.5">
              <Tag tone="focus">{r.verifyKind}</Tag>
              <div className="text-[11px] text-vsc-desc">{r.verifyDetail}</div>
            </div>,
            <code key="p" className="font-mono text-[11px] break-all select-text">
              {r.targetPath}
            </code>,
          ];
        })}
      />
    </>
  );
}

const CLI_EXAMPLE = `cd D:\\Atollo
.\\Atollo.exe components | Out-Host
.\\Atollo.exe prepare --only node,php --line php=8.3 | Out-Host
.\\Atollo.exe check | Out-Host`;

export const INSIDE: TopicTexts = {
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
              "Su macOS: i dati della finestra in ~/Library/WebKit e, se non usi il token, l'accesso di Claude Code nel Portachiavi.",
              "On macOS: the window's data in ~/Library/WebKit and, if you do not use the token, the Claude Code sign-in in the Keychain.",
              "Sur macOS : les données de la fenêtre dans ~/Library/WebKit et, si vous n'utilisez pas le jeton, la connexion de Claude Code dans le Trousseau.",
              "Unter macOS: die Daten des Fensters in ~/Library/WebKit und, wenn du das Token nicht verwendest, die Anmeldung von Claude Code im Schlüsselbund.",
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
            "Atollo parte sempre come utente normale. Le uniche cose che chiedono un amministratore sono esterne al kit e facoltative: Visual Studio Build Tools per il compilatore MSVC di Rust, gli Xcode Command Line Tools su macOS per alcuni programmi C/C++.",
            "Atollo always starts as a normal user. The only things that need an administrator are outside the kit and optional: Visual Studio Build Tools for Rust's MSVC compiler, the Xcode Command Line Tools on macOS for some C/C++ programs.",
            "Atollo démarre toujours en tant qu'utilisateur normal. Les seules choses qui demandent un administrateur sont extérieures au kit et facultatives : Visual Studio Build Tools pour le compilateur MSVC de Rust, les Xcode Command Line Tools sur macOS pour certains programmes C/C++.",
            "Atollo startet immer als normaler Benutzer. Die einzigen Dinge, die einen Administrator verlangen, liegen außerhalb des Kits und sind optional: Visual Studio Build Tools für den MSVC-Compiler von Rust, die Xcode Command Line Tools unter macOS für einige C/C++-Programme.",
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

      <Related ids={["disk-layout", "lifecycle", "move"]} />
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
            <code
              key="p"
              className="font-mono text-[12px] text-vsc-settings-header select-text"
              style={{ paddingLeft: `${String(n.depth * 14)}px` }}
            >
              {n.path}
            </code>,
            n.text,
          ])}
        />
        {info && (
          <P>
            {rich(
              tr("Il tuo kit è in {0}.", "Your kit is in {0}.", "Votre kit est dans {0}.", "Dein Kit liegt in {0}."),
              <Code key="r">{info.root}</Code>,
            )}
          </P>
        )}
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
              "Dopo 10 secondi, le versioni offerte da ogni fonte, per i menu che ne scaricano una (al massimo una volta al giorno).",
              "After 10 seconds, the versions each source offers, for the menus that download one (at most once a day).",
              "Après 10 secondes, les versions proposées par chaque source, pour les menus qui en téléchargent une (au plus une fois par jour).",
              "Nach 10 Sekunden die Versionen, die jede Quelle anbietet, für die Menüs, die eine herunterladen (höchstens einmal am Tag).",
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
                "With “Stop”, “Stop all”, or on closing; a model leaves the memory after ten minutes without requests",
                "Avec « Arrêter », « Tout arrêter », ou à la fermeture ; un modèle quitte la mémoire après dix minutes sans requêtes",
                "Mit „Beenden“, „Alle beenden“ oder beim Schließen; ein Modell verlässt den Speicher nach zehn Minuten ohne Anfragen",
              ),
            ],
          ]}
        />
        <P>
          {tr(
            "Chiudere una console non spegne i database: restano accesi finché non li fermi o chiudi l'app. Il pannello Attività (dalla barra di stato) elenca le azioni dei progetti, ogni versione di database avviata, i pool attivi e il server dei modelli locali; «Ferma tutto» li ferma dopo una conferma.",
            "Closing a console does not shut down the databases: they stay on until you stop them or close the app. The Activity panel (from the status bar) lists the project actions, every running database version, the active pools and the local models server; “Stop all” stops them after a confirmation.",
            "Fermer une console n'arrête pas les bases de données : elles restent allumées jusqu'à ce que vous les arrêtiez ou fermiez l'application. Le panneau Activité (depuis la barre d'état) liste les actions des projets, chaque version de base de données démarrée, les pools actifs et le serveur des modèles locaux ; « Tout arrêter » les arrête après une confirmation.",
            "Das Schließen einer Konsole fährt die Datenbanken nicht herunter: Sie bleiben an, bis du sie stoppst oder die App schließt. Das Panel Aktivität (über die Statusleiste) listet die Aktionen der Projekte, jede gestartete Datenbankversion, die aktiven Pools und den Server der lokalen Modelle auf; „Alle beenden“ stoppt sie nach einer Bestätigung.",
          )}
        </P>
      </Section>

      <Section title={tr("Alla chiusura", "On closing", "À la fermeture", "Beim Schließen")}>
        <P>
          {rich(
            tr(
              "Se qualcosa è aperto, Atollo chiede «Chiudere Atollo?» con l'elenco; «Chiudi tutto» ferma le azioni, i server web e il server dei modelli locali e spegne con ordine i database, poi esce. Prima di staccare l'unità usa {0}.",
              "If something is open, Atollo asks “Close Atollo?” with the list; “Close everything” stops the actions, the web servers and the local models server and shuts down the databases neatly, then exits. Before unplugging the drive use {0}.",
              "Si quelque chose est ouvert, Atollo demande « Fermer Atollo ? » avec la liste ; « Tout fermer » arrête les actions, les serveurs web et le serveur des modèles locaux et éteint proprement les bases de données, puis quitte. Avant de débrancher le lecteur, utilisez {0}.",
              "Wenn etwas geöffnet ist, fragt Atollo „Atollo schließen?“ mit der Liste; „Alles schließen“ stoppt die Aktionen, die Webserver und den Server der lokalen Modelle, fährt die Datenbanken geordnet herunter und beendet sich dann. Bevor du das Laufwerk abziehst, verwende {0}.",
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
                "Python, Node.js, Go, Maven, Caddy, SQLite, l'editor",
                "Python, Node.js, Go, Maven, Caddy, SQLite, the editor",
                "Python, Node.js, Go, Maven, Caddy, SQLite, l'éditeur",
                "Python, Node.js, Go, Maven, Caddy, SQLite, der Editor",
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
                "I limiti dei piani degli agenti, con l'accesso salvato, solo in lettura e solo con la pagina Agenti aperta.",
                "The limits of the agents' plans, with the saved sign-in, read-only and only with the Agents page open.",
                "Les limites des forfaits des agents, avec la connexion enregistrée, en lecture seule et seulement avec la page Agents ouverte.",
                "Die Limits der Tarife der Agenten, mit der gespeicherten Anmeldung, nur lesend und nur bei geöffneter Seite Agenten.",
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
        <SourcesTable components={components} />
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
              <Code key="h">keyring restore | save</Code>,
              tr(
                "Usato dal kit attorno ad Antigravity: rimette o salva il suo accesso.",
                "Used by the kit around Antigravity: restores or saves its sign-in.",
                "Utilisé par le kit autour d'Antigravity : remet ou enregistre sa connexion.",
                "Vom Kit rund um Antigravity verwendet: stellt dessen Anmeldung wieder her oder speichert sie.",
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
                "Prepara per un altro sistema: win32-x64, win32-arm64, darwin-arm64, darwin-x64, linux-x64, linux-arm64.",
                "Prepares for another system: win32-x64, win32-arm64, darwin-arm64, darwin-x64, linux-x64, linux-arm64.",
                "Prépare pour un autre système : win32-x64, win32-arm64, darwin-arm64, darwin-x64, linux-x64, linux-arm64.",
                "Bereitet für ein anderes System vor: win32-x64, win32-arm64, darwin-arm64, darwin-x64, linux-x64, linux-arm64.",
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
