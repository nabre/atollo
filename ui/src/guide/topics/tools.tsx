// Chapter 5 of the Guide, «Strumenti e linguaggi»: the Tools page and its
// table, several versions of a tool, the overviews.
import { rich } from "../../i18n";
import { tr } from "../model";
import {
  Bullets,
  Callout,
  Code,
  Go,
  GuideShot,
  InBrief,
  P,
  Related,
  Section,
  See,
  Steps,
  Table,
  Under,
  type TopicTexts,
} from "../widgets";

export const TOOLS: TopicTexts = {
  tools: () => (
    <>
      <InBrief
        points={[
          tr(
            "La pagina Strumenti elenca tutti i componenti che il kit sa installare: agenti, linguaggi, strumenti, database.",
            "The Tools page lists all the components the kit can install: agents, languages, tools, databases.",
            "La page Outils liste tous les composants que le kit sait installer : agents, langages, outils, bases de données.",
            "Die Seite Werkzeuge listet alle Komponenten auf, die das Kit installieren kann: Agenten, Sprachen, Werkzeuge, Datenbanken.",
          ),
          tr(
            "Le icone delle righe agiscono subito: «Installa» e «Aggiorna» partono appena le premi.",
            "The icons on the rows act right away: “Install” and “Update” start as soon as you press them.",
            "Les icônes des lignes agissent tout de suite : « Installer » et « Mettre à jour » démarrent dès que vous appuyez dessus.",
            "Die Symbole in den Zeilen wirken sofort: „Installieren“ und „Aktualisieren“ starten, sobald du sie drückst.",
          ),
          tr(
            "Le caselle invece preparano una scelta, che si conferma con «Applica»: così si installano e tolgono più cose insieme.",
            "The checkboxes, instead, prepare a choice that you confirm with “Apply”: this way you install and remove several things at once.",
            "Les cases, elles, préparent un choix, que vous confirmez avec « Appliquer » : ainsi vous installez et retirez plusieurs choses à la fois.",
            "Die Kontrollkästchen bereiten dagegen eine Auswahl vor, die du mit „Übernehmen“ bestätigst: So installierst und entfernst du mehrere Dinge auf einmal.",
          ),
          tr(
            "Ogni riga mostra lo stato su disco con un'icona: non esiste, pre-installato, installato, da aggiornare, errore.",
            "Each row shows the status on disk with an icon: not present, pre-installed, installed, to update, error.",
            "Chaque ligne montre l'état sur le disque avec une icône : absent, pré-installé, installé, à mettre à jour, erreur.",
            "Jede Zeile zeigt den Status auf dem Datenträger mit einem Symbol: nicht vorhanden, vorinstalliert, installiert, zu aktualisieren, Fehler.",
          ),
        ]}
      />

      <GuideShot
        src="tools.webp"
        title={tr(
          "La tabella «Tutti gli strumenti»",
          "The “All tools” table",
          "Le tableau « Tous les outils »",
          "Die Tabelle „Alle Werkzeuge“",
        )}
        caption={tr(
          "Filtri in alto, una riga per componente con stato, versioni e azioni; in fondo la barra con «Aggiorna tutto» e «Applica».",
          "Filters at the top, one row per component with state, versions and actions; at the bottom the bar with “Update all” and “Apply”.",
          "Filtres en haut, une ligne par composant avec état, versions et actions ; en bas la barre avec « Tout mettre à jour » et « Appliquer ».",
          "Filter oben, eine Zeile pro Komponente mit Status, Versionen und Aktionen; unten die Leiste mit „Alle aktualisieren“ und „Übernehmen“.",
        )}
      />

      <Section
        title={tr(
          "Com'è fatta la pagina",
          "How the page is laid out",
          "Comment la page est organisée",
          "Wie die Seite aufgebaut ist",
        )}
      >
        <Table
          columns={[
            tr("Parte", "Part", "Partie", "Teil"),
            tr("Che cosa contiene", "What it contains", "Ce qu'elle contient", "Was er enthält"),
          ]}
          rows={[
            [
              tr("Cerca negli strumenti", "Search tools", "Rechercher dans les outils", "Werkzeuge durchsuchen"),
              tr(
                "Cerca nel nome, nella descrizione e nell'identificativo.",
                "Searches the name, the description and the identifier.",
                "Cherche dans le nom, la description et l'identifiant.",
                "Sucht im Namen, in der Beschreibung und in der Kennung.",
              ),
            ],
            [
              tr("Riquadro «Componenti»", "“Components” box", "Panneau « Composants »", "Bereich „Komponenten“"),
              tr(
                "Un albero con i sei tipi e i loro componenti, con quanti sono installati. Un clic apre la scheda del componente.",
                "A tree with the six kinds and their components, with how many are installed. A click opens the component's tab.",
                "Une arborescence avec les six types et leurs composants, avec le nombre d'installés. Un clic ouvre l'onglet du composant.",
                "Ein Baum mit den sechs Arten und ihren Komponenten, mit der Anzahl der installierten. Ein Klick öffnet die Registerkarte der Komponente.",
              ),
            ],
            [
              tr("Riquadro «Panoramica»", "“Overview” box", "Panneau « Vue d'ensemble »", "Bereich „Übersicht“"),
              rich(
                tr(
                  "Le viste d'insieme: «Tutti gli strumenti», «Matrice», «Estensioni dell'editor», «Relazioni», «Requisiti del PC» ({0}).",
                  "The overall views: “All tools”, “Grid”, “Editor extensions”, “Relations”, “PC requirements” ({0}).",
                  "Les vues d'ensemble : « Tous les outils », « Grille », « Extensions de l'éditeur », « Relations », « Configuration requise du PC » ({0}).",
                  "Die Gesamtansichten: „Alle Werkzeuge“, „Matrix“, „Erweiterungen des Editors“, „Beziehungen“, „PC-Voraussetzungen“ ({0}).",
                ),
                <See key="o" id="tools-overview" />,
              ),
            ],
            [
              tr(
                "Scheda «Tutti gli strumenti»",
                "“All tools” tab",
                "Onglet « Tous les outils »",
                "Registerkarte „Alle Werkzeuge“",
              ),
              tr(
                "La prima scheda, sempre aperta: la tabella di tutti i componenti.",
                "The first tab, always open: the table of all components.",
                "Le premier onglet, toujours ouvert : le tableau de tous les composants.",
                "Die erste Registerkarte, immer geöffnet: die Tabelle aller Komponenten.",
              ),
            ],
            [
              tr("Barra «Applica»", "“Apply” bar", "Barre « Appliquer »", "Leiste „Übernehmen“"),
              tr(
                "In fondo, sempre visibile: le modifiche in attesa, «Aggiorna tutto», «Applica».",
                "At the bottom, always visible: the pending changes, “Update all”, “Apply”.",
                "En bas, toujours visible : les modifications en attente, « Tout mettre à jour », « Appliquer ».",
                "Unten, immer sichtbar: die ausstehenden Änderungen, „Alle aktualisieren“, „Übernehmen“.",
              ),
            ],
          ]}
        />
        <Table
          columns={[
            tr("Tipo", "Type", "Type", "Typ"),
            tr("Che cosa contiene", "What it contains", "Ce qu'elle contient", "Was er enthält"),
          ]}
          rows={[
            [
              tr("Sistema ed editor", "System and editor", "Système et éditeur", "System und Editor"),
              tr(
                "Editor, PowerShell 7, GitHub CLI, Git: sempre installati.",
                "Editor, PowerShell 7, GitHub CLI, Git: always installed.",
                "Éditeur, PowerShell 7, GitHub CLI, Git : toujours installés.",
                "Editor, PowerShell 7, GitHub CLI, Git: immer installiert.",
              ),
            ],
            [
              tr("Agenti AI", "AI agents", "Agents IA", "KI-Agenten"),
              tr(
                "Claude Code, Codex, Antigravity: facoltativi, sempre all'ultima versione.",
                "Claude Code, Codex, Antigravity: optional, always at the latest version.",
                "Claude Code, Codex, Antigravity : facultatifs, toujours à la dernière version.",
                "Claude Code, Codex, Antigravity: optional, immer in der neuesten Version.",
              ),
            ],
            [
              tr("Linguaggi", "Languages", "Langages", "Sprachen"),
              tr(
                "Tutti i linguaggi e i runtime: Python, Node.js, PHP, Go, Java, Rust, Kotlin, .NET, Dart, Zig, Julia, Bun, Deno, C/C++, Ruby, R. Python, Node.js, PHP, Go e Java possono avere più versioni una accanto all'altra; gli altri restano all'ultima versione.",
                "All languages and runtimes: Python, Node.js, PHP, Go, Java, Rust, Kotlin, .NET, Dart, Zig, Julia, Bun, Deno, C/C++, Ruby, R. Python, Node.js, PHP, Go and Java can have several versions side by side; the others stay at the latest version.",
                "Tous les langages et environnements d'exécution : Python, Node.js, PHP, Go, Java, Rust, Kotlin, .NET, Dart, Zig, Julia, Bun, Deno, C/C++, Ruby, R. Python, Node.js, PHP, Go et Java peuvent avoir plusieurs versions côte à côte ; les autres restent à la dernière version.",
                "Alle Sprachen und Laufzeiten: Python, Node.js, PHP, Go, Java, Rust, Kotlin, .NET, Dart, Zig, Julia, Bun, Deno, C/C++, Ruby, R. Python, Node.js, PHP, Go und Java können mehrere Versionen nebeneinander haben; die anderen bleiben auf der neuesten Version.",
              ),
            ],
            [
              tr("Strumenti di sviluppo", "Development tools", "Outils de développement", "Entwicklungswerkzeuge"),
              tr(
                "uv, pnpm/yarn, CMake/Ninja, Android SDK, Typst, TinyTeX: una sola versione, sempre l'ultima; quelli che servono si installano da soli.",
                "uv, pnpm/yarn, CMake/Ninja, Android SDK, Typst, TinyTeX: a single version, always the latest; the ones that are needed install themselves.",
                "uv, pnpm/yarn, CMake/Ninja, Android SDK, Typst, TinyTeX : une seule version, toujours la dernière ; ceux qui sont nécessaires s'installent tout seuls.",
                "uv, pnpm/yarn, CMake/Ninja, Android SDK, Typst, TinyTeX: eine einzige Version, immer die neueste; die benötigten installieren sich von selbst.",
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
            ],
            [
              tr("Database", "Databases", "Bases de données", "Datenbanken"),
              tr(
                "MySQL, MongoDB (più versioni), phpMyAdmin, SQLite.",
                "MySQL, MongoDB (several versions), phpMyAdmin, SQLite.",
                "MySQL, MongoDB (plusieurs versions), phpMyAdmin, SQLite.",
                "MySQL, MongoDB (mehrere Versionen), phpMyAdmin, SQLite.",
              ),
            ],
          ]}
        />
        <Go page="components">
          {tr("Apri la pagina Strumenti", "Open the Tools page", "Ouvrir la page Outils", "Seite Werkzeuge öffnen")}
        </Go>
      </Section>

      <Section
        title={tr(
          "La tabella «Tutti gli strumenti»",
          "The “All tools” table",
          "Le tableau « Tous les outils »",
          "Die Tabelle „Alle Werkzeuge“",
        )}
      >
        <P>
          {tr(
            "Sopra la tabella ci sono i filtri: la ricerca, «Piattaforma», «Tipo» («Tutti i tipi» o uno dei sei) e «Stato» (Tutti, Installati, Non installati, Da applicare, Usati dai progetti), più i pulsanti «Espandi» e «Verifica». Le colonne: nome, versione, versioni su disco, dimensione, stato, progetti che lo usano, che cosa richiede e le azioni.",
            "Above the table are the filters: the search, “Platform”, “Type” (“All kinds” or one of the six) and “Status” (All, Installed, Not installed, To apply, Used by projects), plus the “Expand” and “Verify” buttons. The columns: name, version, versions on disk, size, status, projects that use it, what it requires and the actions.",
            "Au-dessus du tableau se trouvent les filtres : la recherche, « Plateforme », « Type » (« Tous les types » ou l'un des six) et « État » (Tous, Installés, Non installés, À appliquer, Utilisés par les projets), plus les boutons « Développer » et « Vérifier ». Les colonnes : nom, version, versions sur le disque, taille, état, projets qui l'utilisent, ce qu'il requiert et les actions.",
            "Über der Tabelle stehen die Filter: die Suche, „Plattform“, „Typ“ („Alle Typen“ oder eine der sechs) und „Status“ (Alle, Installiert, Nicht installiert, Anzuwenden, Von Projekten verwendet), dazu die Schaltflächen „Erweitern“ und „Prüfen“. Die Spalten: Name, Version, Versionen auf dem Datenträger, Größe, Status, Projekte, die es verwenden, was es benötigt, und die Aktionen.",
          )}
        </P>
        <P>
          {tr(
            "Le righe sono divise per tipo. Ogni tipo ha una riga di titolo: la freccia per chiuderlo o riaprirlo, il nome, quanti componenti contiene («3», oppure «2 di 5» quando un filtro ne nasconde qualcuno), la sua descrizione e una casella «Scegli tutti: …» che sceglie tutti i suoi componenti che si possono scegliere. Un clic sull'intestazione di una colonna ordina le righe dentro ogni tipo (all'inizio per nome).",
            "The rows are divided by kind. Each kind has a title row: the arrow to fold or unfold it, its name, how many components it holds (“3”, or “2 of 5” when a filter hides some), its description and a “Choose all: …” checkbox that chooses all its components that can be chosen. A click on a column header sorts the rows inside each kind (by name at first).",
            "Les lignes sont réparties par type. Chaque type a une ligne de titre : la flèche pour le replier ou le déplier, son nom, le nombre de composants qu'il contient (« 3 », ou « 2 sur 5 » quand un filtre en masque certains), sa description et une case « Tout choisir : … » qui choisit tous ses composants qui peuvent être choisis. Un clic sur l'en-tête d'une colonne trie les lignes à l'intérieur de chaque type (par nom au départ).",
            "Die Zeilen sind nach Art aufgeteilt. Jede Art hat eine Titelzeile: den Pfeil zum Ein- und Ausklappen, ihren Namen, wie viele Komponenten sie enthält („3“ oder „2 von 5“, wenn ein Filter einige ausblendet), ihre Beschreibung und ein Kästchen „Alle wählen: …“, das alle ihre wählbaren Komponenten wählt. Ein Klick auf eine Spaltenüberschrift sortiert die Zeilen innerhalb jeder Art (zu Beginn nach Name).",
          )}
        </P>
        <Table
          columns={[
            tr("Stato", "Status", "État", "Status"),
            tr("Icona", "Icon", "Icône", "Symbol"),
            tr("Significato", "Meaning", "Signification", "Bedeutung"),
          ]}
          rows={[
            [
              tr("non esiste", "not present", "absent", "nicht vorhanden"),
              tr("cerchio vuoto", "empty circle", "cercle vide", "leerer Kreis"),
              tr(
                "Non c'è niente su disco. Il suggerimento dice perché (non scelto, non disponibile su quel sistema…).",
                "There is nothing on disk. The tooltip says why (not chosen, not available on that system…).",
                "Il n'y a rien sur le disque. L'infobulle dit pourquoi (non choisi, non disponible sur ce système…).",
                "Auf dem Datenträger ist nichts vorhanden. Der Tooltip sagt, warum (nicht gewählt, auf diesem System nicht verfügbar…).",
              ),
            ],
            [
              tr("pre-installato", "pre-installed", "pré-installé", "vorinstalliert"),
              tr("pacchetto verde", "green package", "paquet vert", "grünes Paket"),
              tr(
                "Gli archivi sono già nel kit e l'installazione si completa al primo avvio su quel sistema.",
                "The archives are already in the kit and the installation completes at the first start on that system.",
                "Les archives sont déjà dans le kit et l'installation se termine au premier démarrage sur ce système.",
                "Die Archive sind schon im Kit, und die Installation wird beim ersten Start auf diesem System abgeschlossen.",
              ),
            ],
            [
              tr("installato", "installed", "installé", "installiert"),
              tr("spunta verde", "green check mark", "coche verte", "grünes Häkchen"),
              tr("Pronto all'uso.", "Ready to use.", "Prêt à l'emploi.", "Einsatzbereit."),
            ],
            [
              tr("da aggiornare", "to update", "à mettre à jour", "zu aktualisieren"),
              tr("triangolo giallo", "yellow triangle", "triangle jaune", "gelbes Dreieck"),
              tr(
                "Va reinstallato perché il kit è stato spostato e quel componente non lo segue (succede solo su macOS e Linux). Non vuol dire che esista una versione nuova.",
                "It must be reinstalled because the kit was moved and that component does not follow it (this happens only on macOS and Linux). It does not mean that a new version exists.",
                "Il doit être réinstallé parce que le kit a été déplacé et que ce composant ne le suit pas (cela n'arrive que sur macOS et Linux). Cela ne veut pas dire qu'une nouvelle version existe.",
                "Es muss neu installiert werden, weil das Kit verschoben wurde und diese Komponente nicht mitzieht (das passiert nur unter macOS und Linux). Es bedeutet nicht, dass es eine neue Version gibt.",
              ),
            ],
            [
              tr("errore", "error", "erreur", "Fehler"),
              tr("X rossa", "red X", "X rouge", "rotes X"),
              tr(
                "L'ultima installazione non è riuscita, oppure su disco manca una parte. Il suggerimento dice che cosa.",
                "The last installation failed, or a part is missing on disk. The tooltip says what.",
                "La dernière installation a échoué, ou une partie manque sur le disque. L'infobulle dit quoi.",
                "Die letzte Installation ist fehlgeschlagen, oder auf dem Datenträger fehlt ein Teil. Der Tooltip sagt, was.",
              ),
            ],
          ]}
        />
        <P>
          {tr(
            "Durante il lavoro compaiono altri quattro stati, solo sul sistema mostrato: «in corso» (con la percentuale del download), «in coda», «da installare» e «da rimuovere» (scelte non ancora applicate).",
            "While work is in progress, four more states appear, only on the system shown: “in progress” (with the download percentage), “queued”, “to install” and “to remove” (choices not yet applied).",
            "Pendant le travail apparaissent quatre autres états, uniquement sur le système affiché : « en cours » (avec le pourcentage du téléchargement), « en attente », « à installer » et « à supprimer » (choix pas encore appliqués).",
            "Während der Arbeit erscheinen vier weitere Zustände, nur auf dem angezeigten System: „läuft“ (mit dem Prozentsatz des Downloads), „wartet“, „zu installieren“ und „zu entfernen“ (noch nicht übernommene Auswahl).",
          )}
        </P>
        <Callout tone="tip">
          {tr(
            "Passa il mouse sull'icona di stato: il suggerimento dice componente, sistema, stato e il dettaglio (per esempio «download 42%» o «in uso: chiudi i programmi del kit e riprova»).",
            "Hover over the status icon: the tooltip gives the component, system, status and the detail (for example “download 42%” or “in use: close the kit programs and try again”).",
            "Passez la souris sur l'icône d'état : l'infobulle indique le composant, le système, l'état et le détail (par exemple « téléchargement 42% » ou « en cours d'utilisation : fermez les programmes du kit et réessayez »).",
            "Fahr mit der Maus über das Statussymbol: Der Tooltip nennt Komponente, System, Status und das Detail (zum Beispiel „Download 42%“ oder „in Verwendung: Schließe die Programme des Kits und versuche es erneut“).",
          )}
        </Callout>
      </Section>

      <Section
        title={tr(
          "Agire subito o scegliere e applicare",
          "Acting right away, or choosing and applying",
          "Agir tout de suite ou choisir et appliquer",
          "Sofort handeln oder auswählen und übernehmen",
        )}
      >
        <Table
          columns={[
            tr("Comando", "Command", "Commande", "Befehl"),
            tr("Dove", "Where", "Où", "Wo"),
            tr("Che cosa fa", "What it does", "Ce qu'il fait", "Was er tut"),
          ]}
          rows={[
            [
              tr("Installa (nuvola)", "Install (cloud)", "Installer (nuage)", "Installieren (Wolke)"),
              tr(
                "Riga di un componente non installato",
                "Row of a component that is not installed",
                "Ligne d'un composant non installé",
                "Zeile einer nicht installierten Komponente",
              ),
              tr(
                "Lo installa subito, all'ultima versione.",
                "Installs it right away, at the latest version.",
                "L'installe tout de suite, dans la dernière version.",
                "Installiert sie sofort in der neuesten Version.",
              ),
            ],
            [
              tr(
                "Installa una versione (+)",
                "Install a version (+)",
                "Installer une version (+)",
                "Eine Version installieren (+)",
              ),
              tr(
                "Riga di un linguaggio non installato",
                "Row of a language that is not installed",
                "Ligne d'un langage non installé",
                "Zeile einer nicht installierten Sprache",
              ),
              tr(
                "Lo installa subito nella versione che scegli dal menu.",
                "Installs it right away in the version you choose from the menu.",
                "L'installe tout de suite dans la version que vous choisissez dans le menu.",
                "Installiert sie sofort in der Version, die du im Menü wählst.",
              ),
            ],
            [
              tr("Aggiorna", "Update", "Mettre à jour", "Aktualisieren"),
              tr(
                "Riga di un componente installato",
                "Row of an installed component",
                "Ligne d'un composant installé",
                "Zeile einer installierten Komponente",
              ),
              tr(
                "Scarica subito la versione nuova, se c'è.",
                "Downloads the new version right away, if there is one.",
                "Télécharge tout de suite la nouvelle version, s'il y en a une.",
                "Lädt sofort die neue Version herunter, falls es eine gibt.",
              ),
            ],
            [
              tr(
                "Aggiungi una versione (+)",
                "Add a version (+)",
                "Ajouter une version (+)",
                "Eine Version hinzufügen (+)",
              ),
              tr(
                "Riga di un linguaggio installato",
                "Row of an installed language",
                "Ligne d'un langage installé",
                "Zeile einer installierten Sprache",
              ),
              tr(
                "Scarica un'altra versione accanto a quelle che hai.",
                "Downloads another version alongside the ones you have.",
                "Télécharge une autre version à côté de celles que vous avez.",
                "Lädt eine weitere Version neben den vorhandenen herunter.",
              ),
            ],
            [
              tr("Casella", "Checkbox", "Case à cocher", "Kontrollkästchen"),
              tr(
                "Ogni riga tranne quelle sempre installate; «Scegli tutti: …» nella riga del tipo",
                "Every row except those always installed; “Choose all: …” in the kind's row",
                "Chaque ligne sauf celles toujours installées ; « Tout choisir : … » dans la ligne du type",
                "Jede Zeile außer den immer installierten; „Alle wählen: …“ in der Zeile der Art",
              ),
              tr(
                "Segna il componente da installare o da togliere, in attesa di «Applica».",
                "Marks the component to install or remove, waiting for “Apply”.",
                "Marque le composant à installer ou à retirer, en attendant « Appliquer ».",
                "Markiert die Komponente zum Installieren oder Entfernen, bis du „Übernehmen“ drückst.",
              ),
            ],
            [
              tr("Applica", "Apply", "Appliquer", "Übernehmen"),
              tr("Barra in fondo", "Bar at the bottom", "Barre en bas", "Leiste unten"),
              tr(
                "Mostra il riepilogo e, con «Conferma», installa e toglie ciò che hai segnato.",
                "Shows the summary and, with “Confirm”, installs and removes what you marked.",
                "Affiche le récapitulatif et, avec « Confirmer », installe et retire ce que vous avez marqué.",
                "Zeigt die Zusammenfassung und installiert und entfernt mit „Bestätigen“, was du markiert hast.",
              ),
            ],
            [
              tr("Aggiorna tutto", "Update all", "Tout mettre à jour", "Alle aktualisieren"),
              tr("Barra in fondo", "Bar at the bottom", "Barre en bas", "Leiste unten"),
              tr(
                "Porta ogni componente installato all'ultima versione e ogni versione all'ultima correzione.",
                "Brings every installed component to the latest version and every version to the latest patch.",
                "Porte chaque composant installé à la dernière version et chaque version au dernier correctif.",
                "Bringt jede installierte Komponente auf die neueste Version und jede Version auf den neuesten Patch.",
              ),
            ],
          ]}
        />
        <Steps
          steps={[
            tr(
              "Spunta i componenti da aggiungere e togli la spunta a quelli da rimuovere. Le righe cambiate si colorano.",
              "Tick the components to add and untick the ones to remove. The changed rows are highlighted in color.",
              "Cochez les composants à ajouter et décochez ceux à supprimer. Les lignes modifiées se colorent.",
              "Hak die hinzuzufügenden Komponenten an und entferne den Haken bei denen, die entfernt werden sollen. Die geänderten Zeilen werden farbig markiert.",
            ),
            tr(
              "In fondo la barra dice per esempio «2 da installare (circa 180 MB), 1 da rimuovere».",
              "At the bottom the bar says, for example, “2 to install (about 180 MB), 1 to remove”.",
              "En bas, la barre indique par exemple « 2 à installer (environ 180 MB), 1 à supprimer ».",
              "Unten zeigt die Leiste zum Beispiel „2 zu installieren (etwa 180 MB), 1 zu entfernen“.",
            ),
            tr(
              "Premi «Applica»: compare «Applicare le modifiche?» con l'elenco di ciò che si installa e si toglie.",
              "Press “Apply”: “Apply the changes?” appears with the list of what will be installed and removed.",
              "Appuyez sur « Appliquer » : « Appliquer les modifications ? » apparaît avec la liste de ce qui s'installe et se retire.",
              "Drück „Übernehmen“: „Änderungen übernehmen?“ erscheint mit der Liste dessen, was installiert und entfernt wird.",
            ),
            tr(
              "Premi «Conferma». Prima si tolgono, poi si installano; lo stato scorre nella barra («Installazione 1/3: …»). «Annulla» ferma alla fine del passo in corso.",
              "Press “Confirm”. First things are removed, then installed; the progress runs in the bar (“Installing 1/3: …”). “Cancel” stops at the end of the current step.",
              "Appuyez sur « Confirmer ». D'abord les retraits, puis les installations ; l'état défile dans la barre (« Installation 1/3 : … »). « Annuler » arrête à la fin de l'étape en cours.",
              "Drück „Bestätigen“. Zuerst wird entfernt, dann installiert; der Fortschritt läuft in der Leiste („Installation 1/3: …“). „Abbrechen“ hält am Ende des laufenden Schritts an.",
            ),
          ]}
        />
        <Callout tone="warn">
          {tr(
            "«Aggiorna tutto» applica anche le scelte in attesa, senza la finestra di conferma. Se hai caselle cambiate che non vuoi applicare, rimettile com'erano prima.",
            "“Update all” also applies the pending choices, without the confirmation window. If you have changed checkboxes that you do not want to apply, put them back as they were first.",
            "« Tout mettre à jour » applique aussi les choix en attente, sans la fenêtre de confirmation. Si vous avez modifié des cases que vous ne voulez pas appliquer, remettez-les d'abord comme elles étaient.",
            "„Alle aktualisieren“ übernimmt auch die ausstehende Auswahl, ohne Bestätigungsfenster. Wenn du Kontrollkästchen geändert hast, die du nicht übernehmen möchtest, setze sie vorher zurück.",
          )}
        </Callout>
        <Bullets
          items={[
            tr(
              "Togliere un componente è possibile solo con la casella e «Applica», mai con un'icona: è un'operazione che vuole conferma.",
              "Removing a component is possible only with the checkbox and “Apply”, never with an icon: it is an operation that needs confirmation.",
              "Retirer un composant n'est possible qu'avec la case et « Appliquer », jamais avec une icône : c'est une opération qui demande une confirmation.",
              "Eine Komponente lässt sich nur mit dem Kontrollkästchen und „Übernehmen“ entfernen, nie mit einem Symbol: Dieser Vorgang erfordert eine Bestätigung.",
            ),
            tr(
              "Le dipendenze seguono in entrambi i sensi: scegliere Kotlin sceglie Java, togliere Java toglie Kotlin e Android. Le etichette delle righe lo spiegano («Richiede Java: si installa insieme», «Serve a Kotlin: resta installato finché serve»).",
              "Dependencies follow in both directions: choosing Kotlin chooses Java, removing Java removes Kotlin and Android. The labels on the rows explain it (“Needs Java: installed with it”, “Needed by Kotlin: stays installed while it is needed”).",
              "Les dépendances suivent dans les deux sens : choisir Kotlin choisit Java, retirer Java retire Kotlin et Android. Les étiquettes des lignes l'expliquent (« Nécessite Java : installé avec », « Nécessaire à Kotlin : reste installé tant qu'il le faut »).",
              "Abhängigkeiten wirken in beide Richtungen: Wer Kotlin wählt, wählt Java; wer Java entfernt, entfernt Kotlin und Android. Die Beschriftungen der Zeilen erklären es („Benötigt Java: wird mitinstalliert“, „Wird von Kotlin benötigt: bleibt installiert, solange es gebraucht wird“).",
            ),
            tr(
              "Un componente richiesto da un progetto ma non scelto ha un triangolo e l'etichetta «Serve ai progetti: … Non è scelto: sceglilo perché funzionino.»",
              "A component required by a project but not chosen has a triangle and the label “Needed by the projects: … Not chosen: choose it so they work.”",
              "Un composant requis par un projet mais non choisi a un triangle et l'étiquette « Nécessaire aux projets : … Non choisi : choisissez-le pour qu'ils fonctionnent. »",
              "Eine Komponente, die ein Projekt benötigt, die aber nicht gewählt ist, hat ein Dreieck und die Beschriftung „Wird von den Projekten benötigt: … Nicht ausgewählt: wähle es aus, damit sie funktionieren.“",
            ),
            tr(
              "Un componente cancellato a mano dal disco viene tolto dalla scelta all'apertura della pagina, con un avviso.",
              "A component deleted by hand from the disk is removed from the selection when the page opens, with a warning.",
              "Un composant supprimé à la main du disque est retiré du choix à l'ouverture de la page, avec un avertissement.",
              "Eine von Hand vom Datenträger gelöschte Komponente wird beim Öffnen der Seite aus der Auswahl entfernt, mit einer Warnung.",
            ),
          ]}
        />
      </Section>

      <Section
        title={tr(
          "La scheda di un componente",
          "A component's tab",
          "L'onglet d'un composant",
          "Die Registerkarte einer Komponente",
        )}
      >
        <P>
          {tr(
            "Un clic sul nome apre la scheda: in alto logo, descrizione, tipo, stato, dimensione e i pulsanti «Installa»/«Mantieni» e «Rimuovi»/«Non installare» (scelte da applicare), più «Sito ufficiale». Sotto, tre sezioni:",
            "A click on the name opens the tab: at the top the logo, description, kind, status, size and the buttons “Install”/“Keep” and “Remove”/“Do not install” (choices to apply), plus “Official site”. Below, three sections:",
            "Un clic sur le nom ouvre l'onglet : en haut le logo, la description, le type, l'état, la taille et les boutons « Installer »/« Conserver » et « Supprimer »/« Ne pas installer » (choix à appliquer), plus « Site officiel ». En dessous, trois sections :",
            "Ein Klick auf den Namen öffnet die Registerkarte: oben Logo, Beschreibung, Art, Status, Größe und die Schaltflächen „Installieren“/„Behalten“ und „Entfernen“/„Nicht installieren“ (zu übernehmende Auswahl), dazu „Offizielle Website“. Darunter drei Abschnitte:",
          )}
        </P>
        <Table
          columns={[
            tr("Sezione", "Section", "Section", "Abschnitt"),
            tr("Che cosa contiene", "What it contains", "Ce qu'elle contient", "Was er enthält"),
          ]}
          rows={[
            [
              tr("Versioni", "Versions", "Versions", "Versionen"),
              rich(
                tr(
                  "Solo per i linguaggi: le versioni su disco e quelle da scaricare ({0}).",
                  "Only for languages: the versions on disk and the ones to download ({0}).",
                  "Uniquement pour les langages : les versions sur le disque et celles à télécharger ({0}).",
                  "Nur für Sprachen: die Versionen auf dem Datenträger und die herunterladbaren ({0}).",
                ),
                <See key="v" id="tools-versions" />,
              ),
            ],
            [
              tr("Dettagli", "Details", "Détails", "Details"),
              tr(
                "Tipo, versione, dimensione, che cosa richiede, a chi serve, sito.",
                "Kind, version, size, what it requires, what needs it, site.",
                "Type, version, taille, ce qu'il requiert, à qui il sert, site.",
                "Typ, Version, Größe, was es benötigt, wer es braucht, Website.",
              ),
            ],
            [
              tr("Progetti", "Projects", "Projets", "Projekte"),
              tr(
                "I progetti che lo usano e, per i linguaggi, la versione scelta da ciascuno.",
                "The projects that use it and, for languages, the version each one chose.",
                "Les projets qui l'utilisent et, pour les langages, la version choisie par chacun.",
                "Die Projekte, die es verwenden, und bei Sprachen die von jedem gewählte Version.",
              ),
            ],
          ]}
        />
      </Section>

      <Section
        title={tr(
          "«Piattaforma», «Espandi» e «Verifica»",
          "“Platform”, “Expand” and “Verify”",
          "« Plateforme », « Développer » et « Vérifier »",
          "„Plattform“, „Erweitern“ und „Prüfen“",
        )}
      >
        <Bullets
          items={[
            tr(
              "«Piattaforma» sceglie per quale sistema guardare (e preparare) i componenti: Windows x64 o ARM64, macOS Apple Silicon o Intel, Linux x64 o ARM64. Lo stesso kit su una chiavetta si apre su tutti e sei: da qui puoi preparare i programmi per un altro sistema prima di portarci la chiavetta.",
              "“Platform” chooses which system to view (and prepare) the components for: Windows x64 or ARM64, macOS Apple Silicon or Intel, Linux x64 or ARM64. The same kit on a USB stick opens on all six: from here you can prepare the programs for another system before taking the stick there.",
              "« Plateforme » choisit pour quel système regarder (et préparer) les composants : Windows x64 ou ARM64, macOS Apple Silicon ou Intel, Linux x64 ou ARM64. Le même kit sur une clé USB s'ouvre sur les six : d'ici, vous pouvez préparer les programmes pour un autre système avant d'y apporter la clé.",
              "„Plattform“ wählt, für welches System du die Komponenten ansiehst (und vorbereitest): Windows x64 oder ARM64, macOS Apple Silicon oder Intel, Linux x64 oder ARM64. Dasselbe Kit auf einem USB-Stick öffnet sich auf allen sechs: Hier kannst du die Programme für ein anderes System vorbereiten, bevor du den Stick dorthin mitnimmst.",
            ),
            tr(
              "«Espandi» aggiunge una colonna di stato per ognuno dei sei sistemi.",
              "“Expand” adds a status column for each of the six systems.",
              "« Développer » ajoute une colonne d'état pour chacun des six systèmes.",
              "„Erweitern“ fügt für jedes der sechs Systeme eine Statusspalte hinzu.",
            ),
            tr(
              "«Verifica» chiede a ogni componente installato di rispondere con la sua versione, e aggiunge un segno ✓ o ⚠ nella cella dello stato.",
              "“Verify” asks every installed component to respond with its version, and adds a ✓ or ⚠ mark in the status cell.",
              "« Vérifier » demande à chaque composant installé de répondre avec sa version, et ajoute un signe ✓ ou ⚠ dans la cellule de l'état.",
              "„Prüfen“ fordert jede installierte Komponente auf, mit ihrer Version zu antworten, und fügt in der Statuszelle ein ✓ oder ⚠ hinzu.",
            ),
          ]}
        />
      </Section>

      <Under>
        <Bullets
          items={[
            tr(
              "La scelta è in data\\app\\components.txt, una per tutti i sistemi.",
              "The selection is in data\\app\\components.txt, one for all systems.",
              "Le choix est dans data\\app\\components.txt, un seul pour tous les systèmes.",
              "Die Auswahl steht in data\\app\\components.txt, eine für alle Systeme.",
            ),
            tr(
              "I programmi sono in data\\apps\\<sistema>\\<componente>; la versione installata in data\\app\\versions\\<sistema>\\<componente>.txt.",
              "The programs are in data\\apps\\<system>\\<component>; the installed version in data\\app\\versions\\<system>\\<component>.txt.",
              "Les programmes sont dans data\\apps\\<système>\\<composant> ; la version installée dans data\\app\\versions\\<système>\\<composant>.txt.",
              "Die Programme liegen in data\\apps\\<System>\\<Komponente>; die installierte Version in data\\app\\versions\\<System>\\<Komponente>.txt.",
            ),
            tr(
              "Un componente tolto viene tolto anche dalle cartelle degli altri sistemi che lo hanno su disco.",
              "A removed component is also removed from the folders of the other systems that have it on disk.",
              "Un composant retiré est aussi retiré des dossiers des autres systèmes qui l'ont sur le disque.",
              "Eine entfernte Komponente wird auch aus den Ordnern der anderen Systeme entfernt, die sie auf dem Datenträger haben.",
            ),
            tr(
              "Lo stato «da aggiornare» corrisponde a un file <componente>.reinstall, scritto quando il kit cambia cartella e un ambiente conda (R, Ruby, MongoDB e SQLite su Linux, make) non può seguirlo.",
              "The “to update” status corresponds to a <component>.reinstall file, written when the kit changes folder and a conda environment (R, Ruby, MongoDB and SQLite on Linux, make) cannot follow it.",
              "L'état « à mettre à jour » correspond à un fichier <composant>.reinstall, écrit quand le kit change de dossier et qu'un environnement conda (R, Ruby, MongoDB et SQLite sur Linux, make) ne peut pas le suivre.",
              "Der Status „zu aktualisieren“ entspricht einer Datei <Komponente>.reinstall, die geschrieben wird, wenn das Kit den Ordner wechselt und eine conda-Umgebung (R, Ruby, MongoDB und SQLite unter Linux, make) nicht mitziehen kann.",
            ),
          ]}
        />
      </Under>

      <Related ids={["tools-versions", "tools-overview", "sources"]} />
    </>
  ),

  "tools-versions": () => (
    <>
      <InBrief
        points={[
          tr(
            "I linguaggi e i server possono avere più versioni installate insieme, una per versione minore (8.3, 8.4…).",
            "Languages and servers can have several versions installed together, one per minor version (8.3, 8.4…).",
            "Les langages et les serveurs peuvent avoir plusieurs versions installées ensemble, une par version mineure (8.3, 8.4…).",
            "Sprachen und Server können mehrere Versionen gleichzeitig installiert haben, eine pro Nebenversion (8.3, 8.4…).",
          ),
          tr(
            "Il kit usa sempre la più recente su disco; ogni progetto può sceglierne un'altra nella sua pagina.",
            "The kit always uses the newest one on disk; each project can choose another one on its page.",
            "Le kit utilise toujours la plus récente sur le disque ; chaque projet peut en choisir une autre dans sa page.",
            "Das Kit verwendet immer die neueste auf dem Datenträger; jedes Projekt kann auf seiner Seite eine andere wählen.",
          ),
          tr(
            "Una versione usata da un progetto è «bloccata»: non si può togliere finché il progetto la usa.",
            "A version used by a project is “locked”: it cannot be removed while the project uses it.",
            "Une version utilisée par un projet est « verrouillée » : elle ne peut pas être retirée tant que le projet l'utilise.",
            "Eine von einem Projekt verwendete Version ist „gesperrt“: Sie lässt sich nicht entfernen, solange das Projekt sie verwendet.",
          ),
          tr(
            "«Aggiorna tutto» porta ogni versione all'ultima correzione (8.4.1 → 8.4.3) senza toglierne nessuna.",
            "“Update all” brings every version to the latest patch (8.4.1 → 8.4.3) without removing any.",
            "« Tout mettre à jour » porte chaque version au dernier correctif (8.4.1 → 8.4.3) sans en retirer aucune.",
            "„Alle aktualisieren“ bringt jede Version auf den neuesten Patch (8.4.1 → 8.4.3), ohne eine zu entfernen.",
          ),
        ]}
      />

      <Section
        title={tr(
          "Le versioni di un componente",
          "A component's versions",
          "Les versions d'un composant",
          "Die Versionen einer Komponente",
        )}
      >
        <P>
          {tr(
            "Nella sezione «Versioni» della scheda (o sotto la riga, nella tabella) c'è una riga per ogni versione su disco, per esempio «8.4 (8.4.3)», con delle etichette:",
            "In the tab's “Versions” section (or under the row, in the table) there is a row for each version on disk, for example “8.4 (8.4.3)”, with some labels:",
            "Dans la section « Versions » de l'onglet (ou sous la ligne, dans le tableau), il y a une ligne pour chaque version sur le disque, par exemple « 8.4 (8.4.3) », avec des étiquettes :",
            "Im Abschnitt „Versionen“ der Registerkarte (oder unter der Zeile in der Tabelle) gibt es eine Zeile für jede Version auf dem Datenträger, zum Beispiel „8.4 (8.4.3)“, mit Beschriftungen:",
          )}
        </P>
        <Table
          columns={[
            tr("Etichetta", "Label", "Étiquette", "Beschriftung"),
            tr("Significato", "Meaning", "Signification", "Bedeutung"),
          ]}
          rows={[
            [
              tr("più recente", "latest", "plus récente", "neueste"),
              tr(
                "È la versione più nuova offerta dalla fonte.",
                "It is the newest version offered by the source.",
                "C'est la version la plus récente proposée par la source.",
                "Es ist die neueste Version, die die Quelle anbietet.",
              ),
            ],
            [
              tr("bloccata (lucchetto)", "locked (padlock)", "verrouillée (cadenas)", "gesperrt (Schloss)"),
              tr(
                "Dei progetti la usano: il menu «Usata da» dice quali. Non si può rimuovere.",
                "Some projects use it: the “Used by” menu says which ones. It cannot be removed.",
                "Des projets l'utilisent : le menu « Utilisée par » dit lesquels. Elle ne peut pas être supprimée.",
                "Projekte verwenden sie: Das Menü „Verwendet von“ sagt, welche. Sie lässt sich nicht entfernen.",
              ),
            ],
            [
              tr("non per …", "not for …", "pas pour …", "nicht für …"),
              tr(
                "Un altro componente non è compatibile con questa versione (per esempio un'estensione che non la supporta).",
                "Another component is not compatible with this version (for example an extension that does not support it).",
                "Un autre composant n'est pas compatible avec cette version (par exemple une extension qui ne la prend pas en charge).",
                "Eine andere Komponente ist mit dieser Version nicht kompatibel (zum Beispiel eine Erweiterung, die sie nicht unterstützt).",
              ),
            ],
            [
              tr(
                "cestino «Rimuovi …»",
                "trash can “Remove …”",
                "corbeille « Supprimer … »",
                "Papierkorb „… entfernen“",
              ),
              tr(
                "Toglie quella versione. Compare solo se ce ne sono almeno due: l'ultima si toglie con la casella del componente.",
                "Removes that version. It appears only if there are at least two: the last one is removed with the component's checkbox.",
                "Retire cette version. Apparaît seulement s'il y en a au moins deux : la dernière se retire avec la case du composant.",
                "Entfernt diese Version. Erscheint nur, wenn es mindestens zwei gibt: Die letzte wird mit dem Kontrollkästchen der Komponente entfernt.",
              ),
            ],
          ]}
        />
        <Steps
          steps={[
            tr(
              "Per aggiungere una versione premi + «Aggiungi una versione di …»: il menu «Versioni da scaricare» le raggruppa (8.x, 7.x…).",
              "To add a version, press + “Add a version of …”: the “Versions to download” menu groups them (8.x, 7.x…).",
              "Pour ajouter une version, appuyez sur + « Ajouter une version de … » : le menu « Versions à télécharger » les regroupe (8.x, 7.x…).",
              "Um eine Version hinzuzufügen, drück + „Eine Version von … hinzufügen“: Das Menü „Herunterzuladende Versionen“ gruppiert sie (8.x, 7.x…).",
            ),
            tr(
              "Scegli la versione: il download parte e la riga compare in grigio finché non è pronta.",
              "Choose the version: the download starts and the row appears in grey until it is ready.",
              "Choisissez la version : le téléchargement démarre et la ligne apparaît en gris jusqu'à ce qu'elle soit prête.",
              "Wähle die Version: Der Download startet, und die Zeile erscheint grau, bis sie bereit ist.",
            ),
            tr(
              "Se l'elenco sembra vecchio, «Cerca altre versioni di …» lo chiede di nuovo alla fonte.",
              "If the list seems old, “Look for other versions of …” asks the source for it again.",
              "Si la liste semble ancienne, « Chercher d'autres versions de … » la redemande à la source.",
              "Wenn die Liste veraltet wirkt, fragt „Weitere Versionen von … suchen“ sie erneut bei der Quelle ab.",
            ),
          ]}
        />
        <Callout
          tone="info"
          title={tr(
            "Quale versione usa il kit",
            "Which version the kit uses",
            "Quelle version le kit utilise",
            "Welche Version das Kit verwendet",
          )}
        >
          {rich(
            tr(
              "Non c'è una versione «predefinita» da scegliere: il kit usa la più recente che ha su disco. Un progetto nuovo fissa quella che c'è alla sua creazione; per cambiarla vai nella pagina del progetto, «Componenti» › «Versioni» ({0}).",
              "There is no “default” version to choose: the kit uses the newest one it has on disk. A new project pins the one available when it is created; to change it, go to the project's page, “Components” › “Versions” ({0}).",
              "Il n'y a pas de version « par défaut » à choisir : le kit utilise la plus récente qu'il a sur le disque. Un nouveau projet fixe celle qui existe à sa création ; pour la changer, allez dans la page du projet, « Composants » › « Versions » ({0}).",
              "Es gibt keine „Standard“-Version zu wählen: Das Kit verwendet die neueste, die es auf dem Datenträger hat. Ein neues Projekt legt die bei seiner Erstellung vorhandene fest; um sie zu ändern, geh auf die Seite des Projekts, „Komponenten“ › „Versionen“ ({0}).",
            ),
            <See key="p" id="project-versions" />,
          )}
        </Callout>
      </Section>

      <Section title={tr("Casi particolari", "Special cases", "Cas particuliers", "Sonderfälle")}>
        <Bullets
          items={[
            tr(
              "Composer segue PHP: c'è una copia per ogni versione di PHP, compatibile con quella; si installa, si aggiorna e si toglie con PHP.",
              "Composer follows PHP: there is one copy for each PHP version, compatible with it; it is installed, updated and removed with PHP.",
              "Composer suit PHP : il y a une copie pour chaque version de PHP, compatible avec celle-ci ; il s'installe, se met à jour et se retire avec PHP.",
              "Composer folgt PHP: Es gibt eine Kopie für jede PHP-Version, die mit ihr kompatibel ist; es wird mit PHP installiert, aktualisiert und entfernt.",
            ),
            tr(
              "Maven e Gradle seguono Java e compaiono come etichette sulla riga della versione.",
              "Maven and Gradle follow Java and appear as labels on the version's row.",
              "Maven et Gradle suivent Java et apparaissent comme étiquettes sur la ligne de la version.",
              "Maven und Gradle folgen Java und erscheinen als Beschriftungen in der Zeile der Version.",
            ),
            tr(
              "MySQL e MongoDB: ogni versione ha i suoi dati e la sua porta; vedi la pagina Database.",
              "MySQL and MongoDB: each version has its own data and its own port; see the Databases page.",
              "MySQL et MongoDB : chaque version a ses données et son port ; voir la page Bases de données.",
              "MySQL und MongoDB: Jede Version hat ihre eigenen Daten und ihren eigenen Port; siehe die Seite Datenbanken.",
            ),
            tr(
              "In Impostazioni › «Versioni» si sceglie quale versione di Python e di PHP installare quando il componente si installa o si aggiorna, e il compilatore di Rust: non cambia la versione usata dal kit.",
              "In Settings › “Versions” you choose which version of Python and PHP to install when the component is installed or updated, and the Rust compiler: this does not change the version used by the kit.",
              "Dans Paramètres › « Versions », vous choisissez quelle version de Python et de PHP installer quand le composant s'installe ou se met à jour, et le compilateur de Rust : cela ne change pas la version utilisée par le kit.",
              "Unter Einstellungen › „Versionen“ wählst du, welche Version von Python und PHP installiert wird, wenn die Komponente installiert oder aktualisiert wird, sowie den Compiler von Rust: Das ändert nicht die vom Kit verwendete Version.",
            ),
          ]}
        />
      </Section>

      <Under>
        <Bullets
          items={[
            tr(
              "Ogni versione vive nella sua cartella: data\\apps\\<sistema>\\<componente>\\<versione minore>, con la versione completa nel file .atollo-version.",
              "Each version lives in its own folder: data\\apps\\<system>\\<component>\\<minor version>, with the full version in the .atollo-version file.",
              "Chaque version vit dans son dossier : data\\apps\\<système>\\<composant>\\<version mineure>, avec la version complète dans le fichier .atollo-version.",
              "Jede Version liegt in ihrem eigenen Ordner: data\\apps\\<System>\\<Komponente>\\<Nebenversion>, mit der vollständigen Version in der Datei .atollo-version.",
            ),
            tr(
              "Gli elenchi delle versioni offerte sono in data\\app\\version-lines.json, aggiornati quando hanno più di 24 ore.",
              "The lists of offered versions are in data\\app\\version-lines.json, refreshed when they are more than 24 hours old.",
              "Les listes des versions proposées sont dans data\\app\\version-lines.json, mises à jour quand elles ont plus de 24 heures.",
              "Die Listen der angebotenen Versionen stehen in data\\app\\version-lines.json und werden aktualisiert, wenn sie älter als 24 Stunden sind.",
            ),
            tr(
              "Su Windows PHP offre anche le ultime build delle versioni non più supportate (8.1, 8.0, 7.4…), utili per progetti vecchi; su macOS e Linux solo quelle con una build fissata dal kit.",
              "On Windows PHP also offers the last builds of versions no longer supported (8.1, 8.0, 7.4…), useful for old projects; on macOS and Linux only those with a build pinned by the kit.",
              "Sur Windows, PHP propose aussi les dernières builds des versions qui ne sont plus prises en charge (8.1, 8.0, 7.4…), utiles pour les anciens projets ; sur macOS et Linux, seulement celles qui ont une build fixée par le kit.",
              "Unter Windows bietet PHP auch die letzten Builds nicht mehr unterstützter Versionen an (8.1, 8.0, 7.4…), nützlich für alte Projekte; unter macOS und Linux nur die, für die das Kit einen Build festgelegt hat.",
            ),
          ]}
        />
      </Under>

      <Related ids={["project-versions", "tools", "databases"]} />
    </>
  ),

  "tools-overview": () => (
    <>
      <InBrief
        points={[
          tr(
            "«Matrice»: dove può stare ogni componente, sui sei sistemi che il kit supporta.",
            "“Grid”: where each component can be, on the six systems the kit supports.",
            "« Grille » : où chaque composant peut se trouver, sur les six systèmes que le kit prend en charge.",
            "„Matrix“: wo jede Komponente sein kann, auf den sechs Systemen, die das Kit unterstützt.",
          ),
          tr(
            "«Relazioni»: chi richiede chi e con quali versioni, e le versioni chieste dai progetti.",
            "“Relations”: what requires what and with which versions, and the versions the projects ask for.",
            "« Relations » : qui requiert quoi et avec quelles versions, et les versions demandées par les projets.",
            "„Beziehungen“: was was benötigt und mit welchen Versionen, und die von den Projekten verlangten Versionen.",
          ),
          tr(
            "«Estensioni dell'editor»: quali estensioni riceve ogni linguaggio.",
            "“Editor extensions”: which extensions each language receives.",
            "« Extensions de l'éditeur » : quelles extensions reçoit chaque langage.",
            "„Erweiterungen des Editors“: welche Erweiterungen jede Sprache erhält.",
          ),
          tr(
            "«Requisiti del PC»: i componenti di sistema che servono al PC.",
            "“PC requirements”: the system components the PC needs.",
            "« Configuration requise du PC » : les composants système dont le PC a besoin.",
            "„PC-Voraussetzungen“: die Systemkomponenten, die der PC braucht.",
          ),
        ]}
      />

      <Section title={tr("Matrice", "Grid", "Grille", "Matrix")}>
        <P>
          {tr(
            "Una tabella con un componente per riga, divisa negli stessi sei tipi della tabella «Tutti gli strumenti», e i sei sistemi in colonna. Ogni casella dice se il componente può stare su quel sistema: ✓ disponibile, ≈ in emulazione (la versione x64 su un PC ARM64), ↓ pre-installato, ◷ si completa al primo avvio, ✕ non disponibile; un punto verde vuol dire installato. Un clic su una casella mostra il motivo, la versione e il pulsante «Mostra questo sistema».",
            "A table with one component per row, split into the same six kinds as the “All tools” table, and the six systems as columns. Each cell says whether the component can be on that system: ✓ available, ≈ emulated (the x64 version on an ARM64 PC), ↓ pre-installed, ◷ completes at the first start, ✕ not available; a green dot means installed. A click on a cell shows the reason, the version and the “Show this system” button.",
            "Un tableau avec un composant par ligne, réparti dans les mêmes six types que le tableau « Tous les outils », et les six systèmes en colonnes. Chaque case dit si le composant peut se trouver sur ce système : ✓ disponible, ≈ en émulation (la version x64 sur un PC ARM64), ↓ pré-installé, ◷ se termine au premier démarrage, ✕ non disponible ; un point vert signifie installé. Un clic sur une case montre la raison, la version et le bouton « Afficher ce système ».",
            "Eine Tabelle mit einer Komponente pro Zeile, in dieselben sechs Arten aufgeteilt wie die Tabelle „Alle Werkzeuge“, und den sechs Systemen als Spalten. Jede Zelle sagt, ob die Komponente auf diesem System sein kann: ✓ verfügbar, ≈ in Emulation (die x64-Version auf einem ARM64-PC), ↓ vorinstalliert, ◷ wird beim ersten Start abgeschlossen, ✕ nicht verfügbar; ein grüner Punkt bedeutet installiert. Ein Klick auf eine Zelle zeigt den Grund, die Version und die Schaltfläche „Dieses System anzeigen“.",
          )}
        </P>
        <Callout tone="info">
          {tr(
            "La Matrice dice che cosa è possibile; la tabella «Tutti gli strumenti» dice che cosa c'è davvero su disco.",
            "The Grid says what is possible; the “All tools” table says what is really on disk.",
            "La Grille dit ce qui est possible ; le tableau « Tous les outils » dit ce qu'il y a vraiment sur le disque.",
            "Die Matrix sagt, was möglich ist; die Tabelle „Alle Werkzeuge“ sagt, was wirklich auf dem Datenträger ist.",
          )}
        </Callout>
      </Section>

      <Section title={tr("Relazioni", "Relations", "Relations", "Beziehungen")}>
        <P>
          {tr(
            "Due tabelle, ricalcolate quando cambia qualcosa (o con «Ricalcola le relazioni»):",
            "Two tables, recalculated when something changes (or with “Compute the relations again”):",
            "Deux tableaux, recalculés quand quelque chose change (ou avec « Recalculer les relations ») :",
            "Zwei Tabellen, die neu berechnet werden, wenn sich etwas ändert (oder mit „Beziehungen neu berechnen“):",
          )}
        </P>
        <Table
          columns={[
            tr("Tabella", "Table", "Tableau", "Tabelle"),
            tr("Che cosa dice", "What it says", "Ce qu'il dit", "Was sie aussagt"),
          ]}
          rows={[
            [
              tr("Matrice delle relazioni", "Relations matrix", "Matrice des relations", "Beziehungsmatrix"),
              tr(
                "Per ogni strumento che cosa richiede, con quali versioni, e se va bene («sì», «no: il kit usa …», «manca …»).",
                "For each tool, what it requires, with which versions, and whether it is fine (“yes”, “no: the kit uses …”, “… missing”).",
                "Pour chaque outil, ce qu'il requiert, avec quelles versions, et si ça va (« oui », « non : le kit utilise … », « … manquant »).",
                "Für jedes Werkzeug, was es benötigt, mit welchen Versionen und ob es passt („ja“, „nein: das Kit verwendet …“, „… fehlt“).",
              ),
            ],
            [
              tr(
                "Versioni chieste dai progetti",
                "Versions the projects ask for",
                "Versions demandées par les projets",
                "Von den Projekten verlangte Versionen",
              ),
              tr(
                "Per ogni progetto la versione che i suoi file chiedono, da quale file, e lo stato («usa …», «da scaricare: …»), con i pulsanti «Usa …» e «Scarica e usa …».",
                "For each project, the version its files ask for, from which file, and the status (“uses …”, “to download: …”), with the “Use …” and “Download and use …” buttons.",
                "Pour chaque projet, la version que ses fichiers demandent, depuis quel fichier, et l'état (« utilise … », « à télécharger : … »), avec les boutons « Utiliser … » et « Télécharger et utiliser … ».",
                "Für jedes Projekt die Version, die seine Dateien verlangen, aus welcher Datei, und der Status („verwendet …“, „herunterzuladen: …“), mit den Schaltflächen „… verwenden“ und „… herunterladen und verwenden“.",
              ),
            ],
          ]}
        />
      </Section>

      <Section
        title={tr(
          "Estensioni dell'editor",
          "Editor extensions",
          "Extensions de l'éditeur",
          "Erweiterungen des Editors",
        )}
      >
        <P>
          {rich(
            tr(
              "Una tabella in sola lettura: in riga le estensioni, in colonna i linguaggi e gli strumenti. ● vuol dire che l'estensione è attiva di default nei progetti con quel linguaggio, ○ che è facoltativa; ✓ che è installata. I linguaggi dei tuoi progetti sono in evidenza. Le estensioni si scelgono per progetto: {0}.",
              "A read-only table: the extensions as rows, the languages and tools as columns. ● means the extension is on by default in projects with that language, ○ that it is optional; ✓ that it is installed. The languages of your projects are highlighted. Extensions are chosen per project: {0}.",
              "Un tableau en lecture seule : les extensions en lignes, les langages et les outils en colonnes. ● signifie que l'extension est active par défaut dans les projets avec ce langage, ○ qu'elle est facultative ; ✓ qu'elle est installée. Les langages de vos projets sont mis en évidence. Les extensions se choisissent par projet : {0}.",
              "Eine schreibgeschützte Tabelle: die Erweiterungen als Zeilen, die Sprachen und Werkzeuge als Spalten. ● bedeutet, dass die Erweiterung in Projekten mit dieser Sprache standardmäßig aktiv ist, ○, dass sie optional ist; ✓, dass sie installiert ist. Die Sprachen deiner Projekte sind hervorgehoben. Die Erweiterungen werden pro Projekt gewählt: {0}.",
            ),
            <See key="e" id="project-editor" />,
          )}
        </P>
      </Section>

      <Section title={tr("Requisiti del PC", "PC requirements", "Configuration requise du PC", "PC-Voraussetzungen")}>
        <P>
          {tr(
            "Alcuni componenti hanno bisogno di qualcosa del sistema: Visual C++ 2015-2026 per PHP, Apache e MongoDB su Windows, WebView2 per la finestra dell'app, l'emulazione x64 di Windows 11 sui PC ARM64. Ogni riga dice per quali componenti serve e se è «presente», se «manca» o se «non serve».",
            "Some components need something from the system: Visual C++ 2015-2026 for PHP, Apache and MongoDB on Windows, WebView2 for the app window, the Windows 11 x64 emulation on ARM64 PCs. Each row says which components need it and whether it is “present”, “missing” or “not needed”.",
            "Certains composants ont besoin de quelque chose du système : Visual C++ 2015-2026 pour PHP, Apache et MongoDB sur Windows, WebView2 pour la fenêtre de l'application, l'émulation x64 de Windows 11 sur les PC ARM64. Chaque ligne dit pour quels composants c'est nécessaire et si c'est « présent », « manquant » ou « pas nécessaire ».",
            "Einige Komponenten brauchen etwas vom System: Visual C++ 2015-2026 für PHP, Apache und MongoDB unter Windows, WebView2 für das Fenster der App, die x64-Emulation von Windows 11 auf ARM64-PCs. Jede Zeile sagt, für welche Komponenten es nötig ist und ob es „vorhanden“ ist, „fehlt“ oder „nicht nötig“ ist.",
          )}
        </P>
      </Section>

      <Under>
        <Bullets
          items={[
            tr(
              "Le regole delle relazioni sono nel catalogo del kit (tauri/catalog/relations.json); il risultato in data\\app\\relations.json.",
              "The relation rules are in the kit's catalog (tauri/catalog/relations.json); the result in data\\app\\relations.json.",
              "Les règles des relations sont dans le catalogue du kit (tauri/catalog/relations.json) ; le résultat dans data\\app\\relations.json.",
              "Die Regeln der Beziehungen stehen im Katalog des Kits (tauri/catalog/relations.json), das Ergebnis in data\\app\\relations.json.",
            ),
            rich(
              tr(
                "File dei progetti che il kit legge per le versioni: {0} e altri.",
                "Project files the kit reads for the versions: {0} and others.",
                "Fichiers des projets que le kit lit pour les versions : {0} et d'autres.",
                "Projektdateien, die das Kit für die Versionen liest: {0} und weitere.",
              ),
              <Code key="f">
                package.json, .nvmrc, .python-version, pyproject.toml, composer.json, go.mod, global.json, pom.xml,
                Gemfile, Cargo.toml
              </Code>,
            ),
            tr(
              "Il catalogo delle estensioni (da Open VSX) è tauri/catalog/extensions.json.",
              "The extension catalog (from Open VSX) is tauri/catalog/extensions.json.",
              "Le catalogue des extensions (depuis Open VSX) est tauri/catalog/extensions.json.",
              "Der Katalog der Erweiterungen (von Open VSX) ist tauri/catalog/extensions.json.",
            ),
          ]}
        />
      </Under>

      <Related ids={["tools", "project-versions", "move"]} />
    </>
  ),
};
