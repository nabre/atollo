// Chapter 2 of the Guide, «Progetti»: creating projects, the Projects page,
// actions and ports, versions, editor and terminal, Git, deleting.
import { rich } from "../../i18n";
import { tr } from "../model";
import {
  Bullets,
  Callout,
  Code,
  Flow,
  GuideCode,
  GuideShot,
  Go,
  InBrief,
  Key,
  P,
  Related,
  Section,
  See,
  Steps,
  Table,
  Under,
  type TopicTexts,
} from "../widgets";

const PANEL_EXAMPLE = `{
  "default": "dev",
  "actions": [
    {
      "id": "dev",
      "label": "Avvia tutto",
      "type": "web",
      "command": "npm run dev -- --port {port}",
      "folder": "frontend",
      "env": { "API_URL": "http://localhost:8000" }
    },
    {
      "id": "test",
      "label": "Prove",
      "type": "command",
      "command": "php artisan test"
    }
  ]
}`;

export const PROJECTS: TopicTexts = {
  "project-new": () => (
    <>
      <InBrief
        points={[
          tr(
            "Un progetto è semplicemente una cartella dentro projects: il kit la riconosce da sola.",
            "A project is simply a folder inside projects: the kit recognizes it by itself.",
            "Un projet est simplement un dossier dans projects : le kit le reconnaît tout seul.",
            "Ein Projekt ist einfach ein Ordner in projects: Das Kit erkennt ihn von selbst.",
          ),
          tr(
            "Per crearne uno ci sono tre strade: «Progetto vuoto», «Da un indirizzo Git» e «Da framework o CMS». Più l'elenco dei tuoi repository GitHub.",
            "There are three ways to create one: “Empty project”, “From a Git address” and “From a framework or CMS”. Plus the list of your GitHub repositories.",
            "Il y a trois voies pour en créer un : « Projet vide », « Depuis une adresse Git » et « À partir d'un framework ou d'un CMS ». Plus la liste de vos dépôts GitHub.",
            "Es gibt drei Wege, eines zu erstellen: „Leeres Projekt“, „Von einer Git-Adresse“ und „Aus Framework oder CMS“. Dazu die Liste deiner GitHub-Repositorys.",
          ),
          tr(
            "Con un framework, il kit installa gli strumenti che servono ed esegue i comandi ufficiali del framework.",
            "With a framework, the kit installs the tools needed and runs the framework's official commands.",
            "Avec un framework, le kit installe les outils nécessaires et exécute les commandes officielles du framework.",
            "Mit einem Framework installiert das Kit die benötigten Werkzeuge und führt die offiziellen Befehle des Frameworks aus.",
          ),
          tr(
            "Puoi anche copiare a mano una cartella in projects: compare nell'elenco dopo «Aggiorna».",
            "You can also copy a folder into projects by hand: it appears in the list after “Refresh”.",
            "Vous pouvez aussi copier un dossier à la main dans projects : il apparaît dans la liste après « Actualiser ».",
            "Du kannst auch einen Ordner von Hand nach projects kopieren: Er erscheint nach „Aktualisieren“ in der Liste.",
          ),
        ]}
      />

      <GuideShot
        src="project-new.webp"
        title={tr(
          "La procedura «Da framework o CMS»",
          "The “From a framework or CMS” procedure",
          "La procédure « À partir d'un framework ou d'un CMS »",
          "Der Ablauf „Aus Framework oder CMS“",
        )}
        caption={tr(
          "Base, aggiunte, nome, file .env e riepilogo, uno sotto l'altro. A sinistra l'albero delle tre strade.",
          "Base, add-ons, name, .env file and summary, one below the other. On the left, the tree of the three ways.",
          "Base, ajouts, nom, fichier .env et récapitulatif, l'un sous l'autre. À gauche, l'arbre des trois voies.",
          "Basis, Ergänzungen, Name, .env-Datei und Zusammenfassung untereinander. Links der Baum der drei Wege.",
        )}
      />

      <Section
        title={tr(
          "Dove si crea un progetto",
          "Where you create a project",
          "Où créer un projet",
          "Wo du ein Projekt erstellst",
        )}
      >
        <P>
          {tr(
            "Tutte le strade portano alla scheda «Nuovo progetto» della pagina Progetti, che si apre in uno di questi modi:",
            "All roads lead to the “New project” tab of the Projects page, which opens in one of these ways:",
            "Toutes les voies mènent à l'onglet « Nouveau projet » de la page Projets, qui s'ouvre de l'une de ces façons :",
            "Alle Wege führen zum Tab „Neues Projekt“ der Seite Projekte, der sich auf eine dieser Arten öffnet:",
          )}
        </P>
        <Bullets
          items={[
            tr(
              "dal «Pannello rapido», con le tre caselle sotto «Nuovo progetto»;",
              "from the “Quick panel”, with the three boxes under “New project”;",
              "depuis le « Panneau rapide », avec les trois cases sous « Nouveau projet » ;",
              "über die „Schnellübersicht“, mit den drei Kästen unter „Neues Projekt“;",
            ),
            tr(
              "dalle icone in cima alla barra laterale: «Nuovo progetto» (cartella con il più) e «Nuovo progetto da framework o CMS» (bacchetta);",
              "from the icons at the top of the sidebar: “New project” (folder with a plus) and “New project from a framework or CMS” (wand);",
              "depuis les icônes en haut de la barre latérale : « Nouveau projet » (dossier avec un plus) et « Nouveau projet à partir d'un framework ou d'un CMS » (baguette) ;",
              "über die Symbole oben in der Seitenleiste: „Neues Projekt“ (Ordner mit Plus) und „Neues Projekt aus Framework oder CMS“ (Zauberstab);",
            ),
            tr(
              "dal riquadro «Nuovo progetto» della barra laterale, un albero con le tre strade; sotto «Da framework o CMS» ci sono i gruppi e le basi: un clic su una base apre la procedura con quella base già scelta;",
              "from the “New project” pane of the sidebar, a tree with the three ways; under “From a framework or CMS” are the groups and the bases: a click on a base opens the wizard with that base already chosen;",
              "depuis le volet « Nouveau projet » de la barre latérale, une arborescence avec les trois voies ; sous « À partir d'un framework ou d'un CMS » se trouvent les groupes et les bases : un clic sur une base ouvre l'assistant avec cette base déjà choisie ;",
              "über den Bereich „Neues Projekt“ der Seitenleiste, einen Baum mit den drei Wegen; unter „Aus Framework oder CMS“ stehen die Gruppen und die Basen: Ein Klick auf eine Basis öffnet den Assistenten mit dieser Basis bereits ausgewählt;",
            ),
            rich(
              tr(
                "da {0}, che in fondo all'elenco ha le tre voci «Nuovo progetto».",
                "from {0}, which has the three “New project” entries at the bottom of the list.",
                "depuis {0}, qui a en bas de la liste les trois entrées « Nouveau projet ».",
                "über {0}, das am Ende der Liste die drei Einträge „Neues Projekt“ hat.",
              ),
              <Key key="k">Ctrl+P</Key>,
            ),
          ]}
        />
        <P>
          {tr(
            "Un progetto creato o clonato compare subito anche nella pagina «Server Web».",
            "A created or cloned project also appears at once on the “Web Servers” page.",
            "Un projet créé ou cloné apparaît aussitôt aussi dans la page « Serveurs web ».",
            "Ein erstelltes oder geklontes Projekt erscheint sofort auch auf der Seite „Webserver“.",
          )}
        </P>
        <Go page="new">
          {tr("Apri «Nuovo progetto»", "Open “New project”", "Ouvrir « Nouveau projet »", "„Neues Projekt“ öffnen")}
        </Go>
      </Section>

      <Section
        title={tr(
          "Strada 1 · «Progetto vuoto»",
          "Way 1 · “Empty project”",
          "Voie 1 · « Projet vide »",
          "Weg 1 · „Leeres Projekt“",
        )}
      >
        <P>
          {tr(
            "Una cartella nuova e vuota in projects, da riempire come vuoi (anche copiandoci dei file).",
            "A new, empty folder in projects, to fill as you like (even by copying files into it).",
            "Un nouveau dossier vide dans projects, à remplir comme vous voulez (y compris en y copiant des fichiers).",
            "Ein neuer, leerer Ordner in projects, den du nach Belieben füllst (auch durch Hineinkopieren von Dateien).",
          )}
        </P>
        <Steps
          steps={[
            tr(
              "Scrivi il nome della cartella nel campo «Vuoto».",
              "Type the folder name in the “Empty” field.",
              "Tapez le nom du dossier dans le champ « Vide ».",
              "Gib den Namen des Ordners in das Feld „Leer“ ein.",
            ),
            tr(
              "Premi «Crea». Si apre la pagina del nuovo progetto.",
              "Press “Create”. The new project's page opens.",
              "Appuyez sur « Créer ». La page du nouveau projet s'ouvre.",
              "Klicke auf „Erstellen“. Die Seite des neuen Projekts öffnet sich.",
            ),
          ]}
        />
        <P>
          {tr(
            'Il nome non può contenere \\ / : * ? " < > |, non può iniziare o finire con un punto e non può essere un nome riservato di Windows (come CON o NUL). Se esiste già una cartella con quel nome, il kit lo dice.',
            'The name cannot contain \\ / : * ? " < > |, cannot start or end with a dot and cannot be a reserved Windows name (like CON or NUL). If a folder with that name already exists, the kit tells you.',
            'Le nom ne peut pas contenir \\ / : * ? " < > |, ne peut pas commencer ni finir par un point et ne peut pas être un nom réservé de Windows (comme CON ou NUL). Si un dossier de ce nom existe déjà, le kit vous le dit.',
            'Der Name darf \\ / : * ? " < > | nicht enthalten, nicht mit einem Punkt beginnen oder enden und kein reservierter Windows-Name sein (wie CON oder NUL). Wenn bereits ein Ordner mit diesem Namen existiert, sagt dir das Kit das.',
          )}
        </P>
      </Section>

      <Section
        title={tr(
          "Strada 2 · «Da un indirizzo Git»",
          "Way 2 · “From a Git address”",
          "Voie 2 · « Depuis une adresse Git »",
          "Weg 2 · „Von einer Git-Adresse“",
        )}
      >
        <P>
          {tr(
            "Copia (clona) un repository Git esistente in una cartella di projects.",
            "Copies (clones) an existing Git repository into a folder in projects.",
            "Copie (clone) un dépôt Git existant dans un dossier de projects.",
            "Kopiert (klont) ein bestehendes Git-Repository in einen Ordner in projects.",
          )}
        </P>
        <Steps
          steps={[
            tr(
              "Nel primo campo scrivi owner/repository per un repository GitHub (per esempio laravel/laravel), oppure l'indirizzo completo https:// di qualsiasi server Git.",
              "In the first field type owner/repository for a GitHub repository (for example laravel/laravel), or the full https:// address of any Git server.",
              "Dans le premier champ, tapez owner/repository pour un dépôt GitHub (par exemple laravel/laravel), ou l'adresse complète https:// de n'importe quel serveur Git.",
              "Gib im ersten Feld owner/repository für ein GitHub-Repository ein (zum Beispiel laravel/laravel) oder die vollständige https://-Adresse eines beliebigen Git-Servers.",
            ),
            tr(
              "Nel secondo campo, facoltativo, il nome della cartella; se lo lasci vuoto è l'ultima parte dell'indirizzo.",
              "In the second field, optional, the folder name; if you leave it empty it is the last part of the address.",
              "Dans le second champ, facultatif, le nom du dossier ; si vous le laissez vide, c'est la dernière partie de l'adresse.",
              "Im zweiten Feld, optional, der Name des Ordners; wenn du es leer lässt, ist es der letzte Teil der Adresse.",
            ),
            tr("Premi «Clona».", "Press “Clone”.", "Appuyez sur « Cloner ».", "Klicke auf „Klonen“."),
          ]}
        />
        <P>
          {tr(
            "Dopo la copia il kit guarda i file del progetto, installa i componenti che servono e sceglie le versioni che il progetto dichiara (per esempio la versione di Node scritta in .nvmrc o quella di PHP in composer.json), scaricandole se mancano.",
            "After the copy the kit looks at the project's files, installs the components needed and picks the versions the project declares (for example the Node version written in .nvmrc or the PHP one in composer.json), downloading them if missing.",
            "Après la copie, le kit examine les fichiers du projet, installe les composants nécessaires et choisit les versions que le projet déclare (par exemple la version de Node écrite dans .nvmrc ou celle de PHP dans composer.json), en les téléchargeant si elles manquent.",
            "Nach dem Kopieren sieht sich das Kit die Dateien des Projekts an, installiert die benötigten Komponenten und wählt die Versionen, die das Projekt angibt (zum Beispiel die in .nvmrc eingetragene Node-Version oder die PHP-Version in composer.json), und lädt sie herunter, wenn sie fehlen.",
          )}
        </P>
        <Callout tone="info">
          {tr(
            "Sono accettati solo indirizzi https://, non ssh. Per un repository privato di GitHub serve l'accesso con l'account GitHub del kit; per i repository pubblici no. Se la copia non riesce, la cartella viene tolta e compare l'ultima riga dell'errore di Git.",
            "Only https:// addresses are accepted, not ssh. A private GitHub repository needs you to be signed in with the kit's GitHub account; public repositories do not. If the copy fails, the folder is removed and the last line of Git's error appears.",
            "Seules les adresses https:// sont acceptées, pas ssh. Pour un dépôt GitHub privé, il faut être connecté avec le compte GitHub du kit ; pour les dépôts publics, non. Si la copie échoue, le dossier est retiré et la dernière ligne de l'erreur de Git s'affiche.",
            "Nur https://-Adressen werden akzeptiert, kein ssh. Für ein privates GitHub-Repository musst du mit dem GitHub-Konto des Kits angemeldet sein; für öffentliche Repositorys nicht. Wenn das Kopieren fehlschlägt, wird der Ordner entfernt und die letzte Zeile des Git-Fehlers erscheint.",
          )}
        </Callout>
      </Section>

      <Section
        title={tr(
          "Strada 3 · «Da framework o CMS»",
          "Way 3 · “From a framework or CMS”",
          "Voie 3 · « À partir d'un framework ou d'un CMS »",
          "Weg 3 · „Aus Framework oder CMS“",
        )}
      >
        <P>
          {tr(
            "Una procedura guidata che crea un progetto partendo da una base (un framework come Laravel, Django o Vite, un CMS come WordPress, un generatore di siti, un'app desktop o mobile…) e da eventuali aggiunte (Tailwind, React, un database, gli strumenti di prova…). Il kit installa gli strumenti che servono ed esegue i comandi ufficiali del framework, gli stessi che troveresti nella sua documentazione.",
            "A wizard that creates a project starting from a base (a framework like Laravel, Django or Vite, a CMS like WordPress, a site generator, a desktop or mobile app…) and from optional additions (Tailwind, React, a database, testing tools…). The kit installs the tools needed and runs the framework's official commands, the same ones you would find in its documentation.",
            "Un assistant qui crée un projet à partir d'une base (un framework comme Laravel, Django ou Vite, un CMS comme WordPress, un générateur de sites, une application de bureau ou mobile…) et d'éventuels ajouts (Tailwind, React, une base de données, les outils de test…). Le kit installe les outils nécessaires et exécute les commandes officielles du framework, les mêmes que vous trouveriez dans sa documentation.",
            "Ein Assistent, der ein Projekt ausgehend von einer Basis (einem Framework wie Laravel, Django oder Vite, einem CMS wie WordPress, einem Website-Generator, einer Desktop- oder Mobil-App…) und von eventuellen Ergänzungen (Tailwind, React, eine Datenbank, Testwerkzeuge…) erstellt. Das Kit installiert die benötigten Werkzeuge und führt die offiziellen Befehle des Frameworks aus, dieselben, die du in seiner Dokumentation finden würdest.",
          )}
        </P>
        <Table
          columns={[
            tr("Passo", "Step", "Étape", "Schritt"),
            tr("Che cosa fai", "What you do", "Ce que vous faites", "Was du tust"),
          ]}
          rows={[
            [
              tr("1. Base", "1. Base", "1. Base", "1. Basis"),
              tr(
                "Scegli un gruppo con le etichette in alto (Web con PHP, Web con JavaScript e TypeScript, Web con Python, Web con altri linguaggi, CMS, Siti statici e documentazione, App desktop, App mobile, Dati e documenti, Riga di comando e librerie) e poi la scheda della base.",
                "Choose a group with the tags at the top (Web with PHP, Web with JavaScript and TypeScript, Web with Python, Web with other languages, CMS, Static sites and documentation, Desktop app, Mobile app, Data and documents, Command line and libraries) and then the base's card.",
                "Choisissez un groupe avec les étiquettes en haut (Web avec PHP, Web avec JavaScript et TypeScript, Web avec Python, Web avec d'autres langages, CMS, Sites statiques et documentation, Application de bureau, Application mobile, Données et documents, Ligne de commande et bibliothèques) puis la carte de la base.",
                "Wähle eine Gruppe mit den Etiketten oben (Web mit PHP, Web mit JavaScript und TypeScript, Web mit Python, Web mit anderen Sprachen, CMS, Statische Websites und Dokumentation, Desktop-App, Mobile App, Daten und Dokumente, Kommandozeile und Bibliotheken) und dann die Karte der Basis.",
              ),
            ],
            [
              tr("2. Aggiunte", "2. Additions", "2. Ajouts", "2. Ergänzungen"),
              tr(
                "Spunta le aggiunte che vuoi. Sono divise per tipo (Interfaccia, Stili, Prove, Database…): di ogni tipo se ne sceglie una sola, e sceglierne un'altra sostituisce la prima.",
                "Tick the additions you want. They are divided by type (Interface, Styles, Tests, Databases…): you choose only one of each type, and choosing another replaces the first.",
                "Cochez les ajouts que vous voulez. Ils sont classés par type (Interface, Styles, Tests, Bases de données…) : on en choisit un seul par type, et en choisir un autre remplace le premier.",
                "Kreuze die gewünschten Ergänzungen an. Sie sind nach Typ unterteilt (Oberfläche, Stile, Tests, Datenbanken…): Von jedem Typ wählst du nur eine, und die Wahl einer anderen ersetzt die erste.",
              ),
            ],
            [
              tr("3. Nome", "3. Name", "3. Nom", "3. Name"),
              tr(
                "Scrivi il nome del progetto. In alternativa usa un progetto esistente vuoto («vuoto») o con dei file ma senza Git («con file»): i file del framework si aggiungono e quelli che ci sono restano come sono. Un repository Git non si può usare.",
                "Type the project name. Alternatively use an existing project that is empty (“empty”) or has files but no Git (“with files”): the framework's files are added and the ones already there stay as they are. A Git repository cannot be used.",
                "Tapez le nom du projet. Vous pouvez aussi utiliser un projet existant vide (« vide ») ou avec des fichiers mais sans Git (« avec des fichiers ») : les fichiers du framework s'ajoutent et ceux qui y sont restent tels quels. Un dépôt Git ne peut pas être utilisé.",
                "Gib den Namen des Projekts ein. Alternativ verwendest du ein bestehendes Projekt, das leer ist („leer“) oder Dateien, aber kein Git hat („mit Dateien“): Die Dateien des Frameworks kommen hinzu, und die vorhandenen bleiben, wie sie sind. Ein Git-Repository kann nicht verwendet werden.",
              ),
            ],
            [
              tr("4. File .env", "4. .env file", "4. Fichier .env", "4. Datei .env"),
              tr(
                "Le variabili che il kit scriverà nel file .env del progetto. {name} diventa il nome del progetto, {package} un identificativo come com.example.nome, {password} una password nuova. Puoi cambiare i valori, togliere righe o aggiungerne con «Aggiungi una variabile».",
                "The variables the kit will write in the project's .env file. {name} becomes the project name, {package} an identifier like com.example.nome, {password} a new password. You can change the values, remove lines or add some with “Add a variable”.",
                "Les variables que le kit écrira dans le fichier .env du projet. {name} devient le nom du projet, {package} un identifiant comme com.example.nome, {password} un nouveau mot de passe. Vous pouvez changer les valeurs, retirer des lignes ou en ajouter avec « Ajouter une variable ».",
                "Die Variablen, die das Kit in die Datei .env des Projekts schreibt. {name} wird zum Namen des Projekts, {package} zu einer Kennung wie com.example.nome, {password} zu einem neuen Passwort. Du kannst die Werte ändern, Zeilen entfernen oder mit „Variable hinzufügen“ welche hinzufügen.",
              ),
            ],
            [
              tr("Riepilogo", "Summary", "Récapitulatif", "Zusammenfassung"),
              tr(
                "Che cosa succederà: gli strumenti del kit, il database del progetto, i comandi nella cartella projects, quelli nella cartella del progetto e le azioni che il progetto avrà.",
                "What will happen: the kit tools, the project database, the commands in the projects folder, those in the project folder and the actions the project will have.",
                "Ce qui va se passer : les outils du kit, la base de données du projet, les commandes dans le dossier projects, celles dans le dossier du projet et les actions qu'aura le projet.",
                "Was passieren wird: die Werkzeuge des Kits, die Datenbank des Projekts, die Befehle im Ordner projects, die im Projektordner und die Aktionen, die das Projekt haben wird.",
              ),
            ],
          ]}
        />
        <Steps
          steps={[
            tr(
              "Controlla il «Riepilogo» e premi «Crea il progetto».",
              "Check the “Summary” and press “Create the project”.",
              "Vérifiez le « Récapitulatif » et appuyez sur « Créer le projet ».",
              "Prüfe die „Zusammenfassung“ und klicke auf „Projekt erstellen“.",
            ),
            tr(
              "Nella parte «Avanzamento» vedi «Installo gli strumenti del kit che servono…», poi «Comando 1 di N» con l'output di ogni comando.",
              "In the “Progress” part you see “Installing the kit tools needed…”, then “Command 1 of N” with the output of each command.",
              "Dans la partie « Progression », vous voyez « Installation des outils du kit nécessaires… », puis « Commande 1 sur N » avec la sortie de chaque commande.",
              "Im Teil „Fortschritt“ siehst du „Benötigte Werkzeuge des Kits werden installiert…“, dann „Befehl 1 von N“ mit der Ausgabe jedes Befehls.",
            ),
            tr(
              "Alla fine compare «Progetto … creato.» con «Apri il progetto». Se un comando fallisce, la procedura si ferma e mostra l'errore.",
              "At the end “Project … created.” appears with “Open the project”. If a command fails, the wizard stops and shows the error.",
              "À la fin, « Projet … créé. » apparaît avec « Ouvrir le projet ». Si une commande échoue, l'assistant s'arrête et affiche l'erreur.",
              "Am Ende erscheint „Projekt … erstellt.“ mit „Projekt öffnen“. Wenn ein Befehl fehlschlägt, hält der Assistent an und zeigt den Fehler.",
            ),
          ]}
        />
        <Flow
          steps={[
            {
              title: tr("Strumenti", "Tools", "Outils", "Werkzeuge"),
              text: tr(
                "Installa ciò che la base e le aggiunte chiedono (Node, PHP, Composer…).",
                "Installs what the base and the additions ask for (Node, PHP, Composer…).",
                "Installe ce que la base et les ajouts demandent (Node, PHP, Composer…).",
                "Installiert, was die Basis und die Ergänzungen verlangen (Node, PHP, Composer…).",
              ),
            },
            {
              title: tr("Creazione", "Creation", "Création", "Erstellung"),
              text: tr(
                "Esegue i comandi di creazione nella cartella projects.",
                "Runs the creation commands in the projects folder.",
                "Exécute les commandes de création dans le dossier projects.",
                "Führt die Erstellungsbefehle im Ordner projects aus.",
              ),
            },
            {
              title: tr("Database", "Databases", "Bases de données", "Datenbanken"),
              text: tr(
                "Se previsto, crea database e utente e scrive la connessione in .env.",
                "If planned, creates the database and user and writes the connection in .env.",
                "Si c'est prévu, crée la base de données et l'utilisateur et écrit la connexion dans .env.",
                "Falls vorgesehen, erstellt es Datenbank und Benutzer und schreibt die Verbindung in .env.",
              ),
            },
            {
              title: tr("Rifinitura", "Finishing", "Finitions", "Feinschliff"),
              text: tr(
                "Esegue i passi nella cartella del progetto (installazioni, migrazioni).",
                "Runs the steps in the project folder (installations, migrations).",
                "Exécute les étapes dans le dossier du projet (installations, migrations).",
                "Führt die Schritte im Projektordner aus (Installationen, Migrationen).",
              ),
            },
            {
              title: tr("Azioni", "Actions", "Actions", "Aktionen"),
              text: tr(
                "Salva le azioni della ricetta nel file panel.json del progetto.",
                "Saves the recipe's actions in the project's panel.json file.",
                "Enregistre les actions de la recette dans le fichier panel.json du projet.",
                "Speichert die Aktionen des Rezepts in der Datei panel.json des Projekts.",
              ),
            },
          ]}
        />
        <Callout tone="tip">
          {rich(
            tr(
              "Le ricette si possono modificare o se ne possono aggiungere di tue: {0}.",
              "The recipes can be edited, or you can add your own: {0}.",
              "Les recettes peuvent être modifiées, ou vous pouvez ajouter les vôtres : {0}.",
              "Die Rezepte lassen sich bearbeiten, oder du fügst eigene hinzu: {0}.",
            ),
            <See key="f" id="frameworks" />,
          )}
        </Callout>
        <Under>
          <Bullets
            items={[
              tr(
                "Si crea un progetto alla volta. «Annulla», durante l'avanzamento, ferma il comando in corso.",
                "One project is created at a time. “Cancel”, during the progress, stops the running command.",
                "On crée un projet à la fois. « Annuler », pendant la progression, arrête la commande en cours.",
                "Es wird jeweils ein Projekt erstellt. „Abbrechen“ während des Fortschritts beendet den laufenden Befehl.",
              ),
              tr(
                "Ogni riga della ricetta è un comando eseguito con la PowerShell del kit e con l'ambiente del kit, quindi con le versioni degli strumenti del kit.",
                "Each line of the recipe is a command run with the kit's PowerShell and the kit's environment, so with the versions of the kit's tools.",
                "Chaque ligne de la recette est une commande exécutée avec le PowerShell du kit et l'environnement du kit, donc avec les versions des outils du kit.",
                "Jede Zeile des Rezepts ist ein Befehl, der mit der PowerShell des Kits und der Umgebung des Kits ausgeführt wird, also mit den Versionen der Werkzeuge des Kits.",
              ),
              tr(
                "Se usi un progetto esistente con dei file, il framework viene generato in una cartella temporanea del sistema e poi spostato nel progetto: i file che ci sono già restano, e nell'avanzamento compare «Già nel progetto, lasciato com'è».",
                "If you use an existing project with files, the framework is generated in a temporary system folder and then moved into the project: the files already there stay, and the progress shows “Already in the project, left as it is”.",
                "Si vous utilisez un projet existant avec des fichiers, le framework est généré dans un dossier temporaire du système puis déplacé dans le projet : les fichiers déjà présents restent, et la progression affiche « Déjà dans le projet, laissé tel quel ».",
                "Wenn du ein bestehendes Projekt mit Dateien verwendest, wird das Framework in einem temporären Ordner des Systems erzeugt und dann ins Projekt verschoben: Die vorhandenen Dateien bleiben, und im Fortschritt erscheint „Schon im Projekt, unverändert gelassen“.",
              ),
              tr(
                "Se il piano prevede MySQL o MongoDB, il kit avvia il server, crea database e utente del progetto e passa ai comandi le variabili ATOLLO_DB_HOST, ATOLLO_DB_PORT, ATOLLO_DB_NAME, ATOLLO_DB_USER, ATOLLO_DB_PASSWORD e ATOLLO_DB_URL. La password generata dal kit prende il posto di una vuota che arriva dalla base (per esempio DB_PASSWORD= nel suo .env.example).",
                "If the plan includes MySQL or MongoDB, the kit starts the server, creates the project's database and user and passes the variables ATOLLO_DB_HOST, ATOLLO_DB_PORT, ATOLLO_DB_NAME, ATOLLO_DB_USER, ATOLLO_DB_PASSWORD and ATOLLO_DB_URL to the commands. The password the kit generates takes the place of an empty one that comes from the base (for example DB_PASSWORD= in its .env.example).",
                "Si le plan prévoit MySQL ou MongoDB, le kit démarre le serveur, crée la base de données et l'utilisateur du projet et passe aux commandes les variables ATOLLO_DB_HOST, ATOLLO_DB_PORT, ATOLLO_DB_NAME, ATOLLO_DB_USER, ATOLLO_DB_PASSWORD et ATOLLO_DB_URL. Le mot de passe généré par le kit prend la place d'un mot de passe vide venu de la base (par exemple DB_PASSWORD= dans son .env.example).",
                "Wenn der Plan MySQL oder MongoDB vorsieht, startet das Kit den Server, erstellt Datenbank und Benutzer des Projekts und übergibt den Befehlen die Variablen ATOLLO_DB_HOST, ATOLLO_DB_PORT, ATOLLO_DB_NAME, ATOLLO_DB_USER, ATOLLO_DB_PASSWORD und ATOLLO_DB_URL. Das vom Kit erzeugte Passwort tritt an die Stelle eines leeren, das aus der Basis kommt (zum Beispiel DB_PASSWORD= in ihrer .env.example).",
              ),
              tr(
                "Alla fine, se c'è un file .env che Git prenderebbe, il kit aggiunge /.env al file .gitignore.",
                "At the end, if there is a .env file that Git would pick up, the kit adds /.env to the .gitignore file.",
                "À la fin, s'il y a un fichier .env que Git prendrait en compte, le kit ajoute /.env au fichier .gitignore.",
                "Am Ende fügt das Kit /.env zur Datei .gitignore hinzu, wenn es eine Datei .env gibt, die Git erfassen würde.",
              ),
              tr(
                "Le azioni della ricetta vanno nel file panel.json del progetto con un identificativo che comincia con fw- (per esempio fw-avvia).",
                "The recipe's actions go into the project's panel.json file with an identifier that starts with fw- (for example fw-avvia).",
                "Les actions de la recette vont dans le fichier panel.json du projet avec un identifiant qui commence par fw- (par exemple fw-avvia).",
                "Die Aktionen des Rezepts kommen in die Datei panel.json des Projekts, mit einer Kennung, die mit fw- beginnt (zum Beispiel fw-avvia).",
              ),
              tr(
                "Alcune basi richiedono componenti particolari: le app Android vogliono il componente Android SDK; Laravel con MongoDB vuole l'estensione mongodb di PHP, che il kit non installa.",
                "Some bases require particular components: Android apps want the Android SDK component; Laravel with MongoDB wants PHP's mongodb extension, which the kit does not install.",
                "Certaines bases demandent des composants particuliers : les applications Android veulent le composant Android SDK ; Laravel avec MongoDB veut l'extension mongodb de PHP, que le kit n'installe pas.",
                "Manche Basen erfordern besondere Komponenten: Android-Apps brauchen die Komponente Android SDK; Laravel mit MongoDB braucht die PHP-Erweiterung mongodb, die das Kit nicht installiert.",
              ),
            ]}
          />
        </Under>
      </Section>

      <Section
        title={tr(
          "I tuoi repository GitHub",
          "Your GitHub repositories",
          "Vos dépôts GitHub",
          "Deine GitHub-Repositorys",
        )}
      >
        <P>
          {tr(
            "Se hai collegato l'account GitHub, il riquadro «Dai tuoi repository GitHub» della barra laterale elenca i tuoi repository e quelli delle organizzazioni di cui fai parte, raggruppati per proprietario, con una casella «Cerca nei repository».",
            "If you have connected the GitHub account, the “From your GitHub repositories” pane of the sidebar lists your repositories and those of the organizations you belong to, grouped by owner, with a “Search the repositories” box.",
            "Si vous avez connecté le compte GitHub, le volet « Depuis vos dépôts GitHub » de la barre latérale liste vos dépôts et ceux des organisations dont vous faites partie, regroupés par propriétaire, avec un champ « Rechercher dans les dépôts ».",
            "Wenn du das GitHub-Konto verbunden hast, listet der Bereich „Aus deinen GitHub-Repositorys“ der Seitenleiste deine Repositorys und die der Organisationen auf, denen du angehörst, nach Eigentümer gruppiert, mit einem Feld „Repositorys durchsuchen“.",
          )}
        </P>
        <Bullets
          items={[
            tr(
              "«Clona» copia il repository in projects (nella cartella con il suo nome, oppure proprietario-nome se il nome è già preso). Un repository già clonato ha il nome in verde e il pulsante «Editor».",
              "“Clone” copies the repository into projects (into the folder with its name, or owner-name if the name is already taken). An already cloned repository has its name in green and the “Editor” button.",
              "« Cloner » copie le dépôt dans projects (dans le dossier portant son nom, ou propriétaire-nom si le nom est déjà pris). Un dépôt déjà cloné a son nom en vert et le bouton « Éditeur ».",
              "„Klonen“ kopiert das Repository nach projects (in den Ordner mit seinem Namen oder Eigentümer-Name, wenn der Name schon vergeben ist). Ein bereits geklontes Repository hat seinen Namen in Grün und die Schaltfläche „Editor“.",
            ),
            tr(
              "«GitHub» apre il repository nel browser.",
              "“GitHub” opens the repository in the browser.",
              "« GitHub » ouvre le dépôt dans le navigateur.",
              "„GitHub“ öffnet das Repository im Browser.",
            ),
            tr(
              "Con un clic sulla riga si aprono i dettagli: privato o pubblico, linguaggio, stelle, ultimo push, dove è clonato, «Copia l'indirizzo per Git» e, per i repository che amministri, «Rinomina» ed «Elimina da GitHub».",
              "A click on the row opens the details: private or public, language, stars, last push, where it is cloned, “Copy the Git address” and, for the repositories you administer, “Rename” and “Delete from GitHub”.",
              "Un clic sur la ligne ouvre les détails : privé ou public, langage, étoiles, dernier push, où il est cloné, « Copier l'adresse pour Git » et, pour les dépôts que vous administrez, « Renommer » et « Supprimer de GitHub ».",
              "Ein Klick auf die Zeile öffnet die Details: privat oder öffentlich, Sprache, Sterne, letzter Push, wo es geklont ist, „Git-Adresse kopieren“ und, für Repositorys, die du verwaltest, „Umbenennen“ und „Von GitHub löschen“.",
            ),
          ]}
        />
        <Callout tone="info">
          {rich(
            tr(
              "Come «Da un indirizzo Git», dopo la copia il kit installa i componenti che servono e sceglie le versioni che il progetto dichiara. Tutto sull'account in {0}.",
              "As with “From a Git address”, after the copy the kit installs the components needed and picks the versions the project declares. All about the account in {0}.",
              "Comme pour « Depuis une adresse Git », après la copie le kit installe les composants nécessaires et choisit les versions que le projet déclare. Tout sur le compte dans {0}.",
              "Wie bei „Von einer Git-Adresse“ installiert das Kit nach dem Kopieren die nötigen Komponenten und wählt die Versionen, die das Projekt angibt. Alles zum Konto in {0}.",
            ),
            <See key="g" id="github" />,
          )}
        </Callout>
      </Section>

      <Related ids={["first-project", "project-page", "frameworks", "github"]} />
    </>
  ),

  "project-page": () => (
    <>
      <InBrief
        points={[
          tr(
            "Nella barra laterale trovi l'elenco dei progetti; con un clic se ne apre la pagina in una scheda.",
            "In the sidebar you find the list of projects; a click opens a project's page in a tab.",
            "Dans la barre latérale se trouve la liste des projets ; un clic ouvre la page d'un projet dans un onglet.",
            "In der Seitenleiste findest du die Liste der Projekte; ein Klick öffnet die Seite eines Projekts in einem Tab.",
          ),
          tr(
            "Il «Pannello rapido» raccoglie ciò che è avviato, i preferiti, i recenti e ciò che va sistemato.",
            "The “Quick panel” gathers what is running, the favorites, the recent ones and what needs fixing.",
            "Le « Panneau rapide » rassemble ce qui est démarré, les favoris, les récents et ce qui est à corriger.",
            "Die „Schnellübersicht“ sammelt, was gestartet ist, die Favoriten, die zuletzt verwendeten und was zu beheben ist.",
          ),
          tr(
            "La pagina di un progetto ha in alto i pulsanti principali e sotto sette sezioni.",
            "A project's page has the main buttons at the top and seven sections below.",
            "La page d'un projet a en haut les boutons principaux et en dessous sept sections.",
            "Die Seite eines Projekts hat oben die wichtigsten Schaltflächen und darunter sieben Abschnitte.",
          ),
        ]}
      />

      <GuideShot
        src="projects-home.webp"
        title={tr("Il «Pannello rapido»", "The “Quick panel”", "Le « Panneau rapide »", "Die „Schnellübersicht“")}
        caption={tr(
          "Recenti, da sistemare e le tre strade per un progetto nuovo. A sinistra l'elenco dei progetti.",
          "Recent projects, things to fix and the three ways to a new project. On the left, the list of projects.",
          "Récents, à corriger et les trois voies pour un nouveau projet. À gauche, la liste des projets.",
          "Zuletzt verwendet, zu beheben und die drei Wege zu einem neuen Projekt. Links die Projektliste.",
        )}
      />

      <Section title={tr("La barra laterale", "The sidebar", "La barre latérale", "Die Seitenleiste")}>
        <P>
          {tr(
            "In cima, le icone: «Nuovo progetto», «Nuovo progetto da framework o CMS», «Apri un progetto (Ctrl+P)», «Aggiorna», «Apri la cartella» (apre projects) e il libro della guida. Sotto, quattro riquadri:",
            "At the top, the icons: “New project”, “New project from a framework or CMS”, “Open a project (Ctrl+P)”, “Refresh”, “Open folder” (opens projects) and the guide's book. Below, four panes:",
            "En haut, les icônes : « Nouveau projet », « Nouveau projet à partir d'un framework ou d'un CMS », « Ouvrir un projet (Ctrl+P) », « Actualiser », « Ouvrir le dossier » (ouvre projects) et le livre du guide. En dessous, quatre volets :",
            "Oben die Symbole: „Neues Projekt“, „Neues Projekt aus Framework oder CMS“, „Projekt öffnen (Strg+P)“, „Aktualisieren“, „Ordner öffnen“ (öffnet projects) und das Buch der Anleitung. Darunter vier Bereiche:",
          )}
        </P>
        <Table
          columns={[
            tr("Riquadro", "Pane", "Volet", "Bereich"),
            tr("Che cosa contiene", "What it contains", "Ce qu'il contient", "Was er enthält"),
          ]}
          rows={[
            [
              tr("Progetti", "Projects", "Projets", "Projekte"),
              tr(
                "Tutti i progetti in ordine alfabetico, con il numero in alto. Le cartelle che iniziano con un punto e i collegamenti non contano. Oltre sei progetti compare la casella «Cerca nei progetti».",
                "All the projects in alphabetical order, with the count at the top. Folders starting with a dot and links do not count. With more than six projects the “Search projects” box appears.",
                "Tous les projets par ordre alphabétique, avec leur nombre en haut. Les dossiers qui commencent par un point et les liens ne comptent pas. Au-delà de six projets apparaît le champ « Rechercher dans les projets ».",
                "Alle Projekte in alphabetischer Reihenfolge, mit der Anzahl oben. Ordner, die mit einem Punkt beginnen, und Verknüpfungen zählen nicht. Bei mehr als sechs Projekten erscheint das Feld „Projekte durchsuchen“.",
              ),
            ],
            [
              tr("Dati senza progetto", "Data without a project", "Données sans projet", "Daten ohne Projekt"),
              rich(
                tr(
                  "Compare solo se il kit ha dati di progetti che non sono più in projects. Vedi {0}.",
                  "It appears only if the kit has data of projects that are no longer in projects. See {0}.",
                  "N'apparaît que si le kit a des données de projets qui ne sont plus dans projects. Voir {0}.",
                  "Erscheint nur, wenn das Kit Daten von Projekten hat, die nicht mehr in projects sind. Siehe {0}.",
                ),
                <See key="c" id="project-cleanup" />,
              ),
            ],
            [
              tr("Nuovo progetto", "New project", "Nouveau projet", "Neues Projekt"),
              tr(
                "L'albero delle tre strade per creare un progetto, con le basi dei framework.",
                "The tree of the three ways to create a project, with the framework bases.",
                "L'arborescence des trois voies pour créer un projet, avec les bases des frameworks.",
                "Der Baum der drei Wege zum Erstellen eines Projekts, mit den Basen der Frameworks.",
              ),
            ],
            [
              tr(
                "Dai tuoi repository GitHub",
                "From your GitHub repositories",
                "Depuis vos dépôts GitHub",
                "Aus deinen GitHub-Repositorys",
              ),
              tr(
                "I repository del tuo account, da clonare.",
                "The repositories of your account, ready to clone.",
                "Les dépôts de votre compte, à cloner.",
                "Die Repositorys deines Kontos, zum Klonen.",
              ),
            ],
          ]}
        />
        <P>
          {tr(
            "Ogni riga di progetto ha un'icona che dice che tipo di progetto è: un globo per un sito web, una finestra per un'app con finestra, un terminale per un programma a riga di comando. Un pallino verde vuol dire che qualcosa è avviato; un triangolo giallo che mancano dei componenti. Passando il mouse sulla riga compaiono i pulsanti ▶ «Avvia» (o ■ «Ferma»), «Editor», «Terminale» e «Agente».",
            "Each project row has an icon that says what kind of project it is: a globe for a website, a window for an app with a window, a terminal for a command-line program. A green dot means something is running; a yellow triangle that components are missing. Hovering over the row shows the buttons ▶ “Start” (or ■ “Stop”), “Editor”, “Terminal” and “Agent”.",
            "Chaque ligne de projet a une icône qui indique de quel type de projet il s'agit : un globe pour un site web, une fenêtre pour une application avec fenêtre, un terminal pour un programme en ligne de commande. Une pastille verte veut dire que quelque chose est démarré ; un triangle jaune qu'il manque des composants. Au survol de la ligne apparaissent les boutons ▶ « Démarrer » (ou ■ « Arrêter »), « Éditeur », « Terminal » et « Agent ».",
            "Jede Projektzeile hat ein Symbol, das zeigt, welche Art von Projekt es ist: ein Globus für eine Website, ein Fenster für eine App mit Fenster, ein Terminal für ein Kommandozeilenprogramm. Ein grüner Punkt bedeutet, dass etwas gestartet ist; ein gelbes Dreieck, dass Komponenten fehlen. Beim Darüberfahren erscheinen die Schaltflächen ▶ „Starten“ (oder ■ „Beenden“), „Editor“, „Terminal“ und „Agent“.",
          )}
        </P>
      </Section>

      <Section
        title={tr("Il «Pannello rapido»", "The “Quick panel”", "Le « Panneau rapide »", "Die „Schnellübersicht“")}
      >
        <P>
          {tr(
            "È la prima scheda della pagina, sempre presente. Mostra soltanto le parti che hanno qualcosa da mostrare:",
            "It is the first tab of the page, always there. It only shows the parts that have something to show:",
            "C'est le premier onglet de la page, toujours présent. Il n'affiche que les parties qui ont quelque chose à montrer :",
            "Sie ist der erste Tab der Seite, immer vorhanden. Sie zeigt nur die Teile, die etwas anzuzeigen haben:",
          )}
        </P>
        <Table
          columns={[
            tr("Parte", "Part", "Partie", "Teil"),
            tr("Che cosa mostra", "What it shows", "Ce qu'elle montre", "Was er zeigt"),
          ]}
          rows={[
            [
              tr("In esecuzione", "Running", "En cours d'exécution", "Läuft"),
              tr(
                "Le azioni avviate in questa sessione, con l'indirizzo e i pulsanti «Apri nel browser», «Vedi l'output», «Ferma».",
                "The actions started in this session, with the address and the “Open in the browser”, “Show the output”, “Stop” buttons.",
                "Les actions démarrées dans cette session, avec l'adresse et les boutons « Ouvrir dans le navigateur », « Voir la sortie », « Arrêter ».",
                "Die in dieser Sitzung gestarteten Aktionen, mit der Adresse und den Schaltflächen „Im Browser öffnen“, „Ausgabe anzeigen“, „Beenden“.",
              ),
            ],
            [
              tr("Preferiti", "Favorites", "Favoris", "Favoriten"),
              tr(
                "I progetti che hai segnato con la stella («Aggiungi … ai preferiti»), come schede con i loro linguaggi e pulsanti.",
                "The projects you have marked with the star (“Add … to favorites”), as cards with their languages and buttons.",
                "Les projets que vous avez marqués d'une étoile (« Ajouter … aux favoris »), sous forme de cartes avec leurs langages et leurs boutons.",
                "Die Projekte, die du mit dem Stern markiert hast („… zu den Favoriten hinzufügen“), als Karten mit ihren Sprachen und Schaltflächen.",
              ),
            ],
            [
              tr("Recenti", "Recent", "Récents", "Zuletzt verwendet"),
              tr(
                "I sei progetti usati o modificati più di recente, escluso chi è già tra i preferiti.",
                "The six projects used or changed most recently, excluding those already among the favorites.",
                "Les six projets utilisés ou modifiés le plus récemment, sauf ceux qui sont déjà dans les favoris.",
                "Die sechs zuletzt verwendeten oder geänderten Projekte, ohne die, die schon unter den Favoriten sind.",
              ),
            ],
            [
              tr("Da sistemare", "Needs attention", "À corriger", "Zu beheben"),
              tr(
                "Le azioni fermate con un errore («Vedi l'output») e i progetti a cui mancano componenti («Installa», che apre la sezione Componenti del progetto).",
                "The actions stopped with an error (“Show the output”) and the projects missing components (“Install”, which opens the project's Components section).",
                "Les actions arrêtées sur une erreur (« Voir la sortie ») et les projets auxquels il manque des composants (« Installer », qui ouvre la section Composants du projet).",
                "Die mit einem Fehler beendeten Aktionen („Ausgabe anzeigen“) und die Projekte, denen Komponenten fehlen („Installieren“, das den Abschnitt Komponenten des Projekts öffnet).",
              ),
            ],
            [
              tr("Nuovo progetto", "New project", "Nouveau projet", "Neues Projekt"),
              tr(
                "Le tre strade per creare un progetto, più i collegamenti ai repository GitHub, alla cartella dei progetti e alla guida.",
                "The three ways to create a project, plus the links to GitHub repositories, the projects folder and the guide.",
                "Les trois voies pour créer un projet, plus les liens vers les dépôts GitHub, le dossier des projets et le guide.",
                "Die drei Wege zum Erstellen eines Projekts, dazu die Links zu den GitHub-Repositorys, zum Projektordner und zur Anleitung.",
              ),
            ],
          ]}
        />
      </Section>

      <Section
        title={tr(
          "Aprire un progetto per nome",
          "Opening a project by name",
          "Ouvrir un projet par son nom",
          "Ein Projekt über seinen Namen öffnen",
        )}
      >
        <P>
          {rich(
            tr(
              "Da qualsiasi pagina premi {0} (⌘P su macOS): compare «Apri un progetto». Scrivi parte del nome, della piattaforma o di un linguaggio; i progetti sono in ordine di uso recente. {1} apre la pagina, {2} avvia subito l'azione predefinita.",
              "From any page press {0} (⌘P on macOS): “Open a project” appears. Type part of the name, the platform or a language; projects are sorted by recent use. {1} opens the page, {2} immediately starts the default action.",
              "Depuis n'importe quelle page, appuyez sur {0} (⌘P sur macOS) : « Ouvrir un projet » apparaît. Tapez une partie du nom, de la plateforme ou d'un langage ; les projets sont triés par utilisation récente. {1} ouvre la page, {2} démarre aussitôt l'action par défaut.",
              "Drück auf jeder Seite {0} (⌘P unter macOS): „Projekt öffnen“ erscheint. Gib einen Teil des Namens, der Plattform oder einer Sprache ein; die Projekte sind nach letzter Verwendung sortiert. {1} öffnet die Seite, {2} startet sofort die Standardaktion.",
            ),
            <Key key="p">Ctrl+P</Key>,
            <Key key="i">{tr("Invio", "Enter", "Entrée", "Enter")}</Key>,
            <Key key="s">{tr("Maiusc+Invio", "Shift+Enter", "Maj+Entrée", "Umschalt+Enter")}</Key>,
          )}
        </P>
      </Section>

      <Section
        title={tr("La pagina di un progetto", "A project's page", "La page d'un projet", "Die Seite eines Projekts")}
      >
        <P>
          {tr(
            "In alto ci sono il nome, il tipo e la piattaforma riconosciuta (per esempio «Laravel · sito web»), i loghi dei linguaggi, il ramo Git e, se mancano componenti, un avviso «Mancano: …». Poi i pulsanti:",
            "At the top are the name, the type and the recognized platform (for example “Laravel · website”), the language logos, the Git branch and, if components are missing, a “Missing: …” warning. Then the buttons:",
            "En haut se trouvent le nom, le type et la plateforme reconnue (par exemple « Laravel · site web »), les logos des langages, la branche Git et, s'il manque des composants, un avertissement « Manquants : … ». Puis les boutons :",
            "Oben stehen der Name, der Typ und die erkannte Plattform (zum Beispiel „Laravel · Website“), die Logos der Sprachen, der Git-Branch und, wenn Komponenten fehlen, ein Hinweis „Fehlend: …“. Dann die Schaltflächen:",
          )}
        </P>
        <Table
          columns={[
            tr("Pulsante", "Button", "Bouton", "Schaltfläche"),
            tr("Che cosa fa", "What it does", "Ce qu'il fait", "Was sie bewirkt"),
          ]}
          rows={[
            [
              tr("Avvia / Ferma", "Start / Stop", "Démarrer / Arrêter", "Starten / Beenden"),
              tr(
                "Avvia l'azione predefinita del progetto, o la ferma se è in corso.",
                "Starts the project's default action, or stops it if it is running.",
                "Démarre l'action par défaut du projet, ou l'arrête si elle est en cours.",
                "Startet die Standardaktion des Projekts oder beendet sie, wenn sie läuft.",
              ),
            ],
            [
              tr("Editor", "Editor", "Éditeur", "Editor"),
              tr(
                "Apre il progetto nell'editor, in una finestra di Atollo.",
                "Opens the project in the editor, in an Atollo window.",
                "Ouvre le projet dans l'éditeur, dans une fenêtre d'Atollo.",
                "Öffnet das Projekt im Editor, in einem Atollo-Fenster.",
              ),
            ],
            [
              tr("Terminale", "Terminal", "Terminal", "Terminal"),
              tr(
                "Apre una console del kit nella cartella del progetto, con le sue versioni.",
                "Opens a kit console in the project folder, with its versions.",
                "Ouvre une console du kit dans le dossier du projet, avec ses versions.",
                "Öffnet eine Konsole des Kits im Projektordner, mit seinen Versionen.",
              ),
            ],
            [
              tr(
                "Loghi degli agenti e «Agente»",
                "Agent logos and “Agent”",
                "Logos des agents et « Agent »",
                "Logos der Agenten und „Agent“",
              ),
              tr(
                "Scegli un agente con il suo logo e aprilo nel progetto.",
                "Choose an agent by its logo and open it in the project.",
                "Choisissez un agent par son logo et ouvrez-le dans le projet.",
                "Wähle einen Agenten über sein Logo und öffne ihn im Projekt.",
              ),
            ],
            [
              tr("Apri la cartella", "Open folder", "Ouvrir le dossier", "Ordner öffnen"),
              tr(
                "Apre la cartella del progetto in Esplora file (o Finder).",
                "Opens the project folder in File Explorer (or Finder).",
                "Ouvre le dossier du projet dans l'Explorateur de fichiers (ou le Finder).",
                "Öffnet den Projektordner im Datei-Explorer (oder Finder).",
              ),
            ],
            [
              tr("Aggiorna", "Refresh", "Actualiser", "Aktualisieren"),
              tr(
                "Rilegge il progetto: utile dopo aver cambiato a mano panel.json o i file del progetto.",
                "Rereads the project: useful after changing panel.json or the project's files by hand.",
                "Relit le projet : utile après avoir modifié à la main panel.json ou les fichiers du projet.",
                "Liest das Projekt neu ein: nützlich, nachdem du panel.json oder die Dateien des Projekts von Hand geändert hast.",
              ),
            ],
          ]}
        />
        <P>
          {tr(
            "Sotto, le sette sezioni:",
            "Below, the seven sections:",
            "En dessous, les sept sections :",
            "Darunter die sieben Abschnitte:",
          )}
        </P>
        <Table
          columns={[
            tr("Sezione", "Section", "Section", "Abschnitt"),
            tr("A che cosa serve", "What it is for", "À quoi elle sert", "Wozu er dient"),
            tr("Argomento", "Topic", "Sujet", "Thema"),
          ]}
          rows={[
            [
              tr("Avvio", "Run", "Lancement", "Start"),
              tr(
                "Le azioni del progetto, il loro stato, la porta e l'output.",
                "The project's actions, their status, the port and the output.",
                "Les actions du projet, leur état, le port et la sortie.",
                "Die Aktionen des Projekts, ihr Status, der Port und die Ausgabe.",
              ),
              <See key="r" id="project-run" />,
            ],
            [
              tr("Sito web", "Website", "Site web", "Website"),
              tr(
                "Servire una cartella del progetto come sito, con file statici, PHP, Caddy o Apache.",
                "Serving a project folder as a website, with static files, PHP, Caddy or Apache.",
                "Servir un dossier du projet comme site, avec des fichiers statiques, PHP, Caddy ou Apache.",
                "Einen Ordner des Projekts als Website bereitstellen, mit statischen Dateien, PHP, Caddy oder Apache.",
              ),
              <See key="w" id="websites" />,
            ],
            [
              tr("Componenti", "Components", "Composants", "Komponenten"),
              tr(
                "Gli strumenti che servono, come è stato riconosciuto il progetto, le versioni.",
                "The tools needed, how the project was recognized, the versions.",
                "Les outils nécessaires, comment le projet a été reconnu, les versions.",
                "Die benötigten Werkzeuge, wie das Projekt erkannt wurde, die Versionen.",
              ),
              <See key="v" id="project-versions" />,
            ],
            [
              tr("Database", "Databases", "Bases de données", "Datenbanken"),
              tr(
                "Database e utente del progetto, la connessione nel file .env, le copie.",
                "The project's database and user, the connection in the .env file, the copies.",
                "La base de données et l'utilisateur du projet, la connexion dans le fichier .env, les copies.",
                "Datenbank und Benutzer des Projekts, die Verbindung in der Datei .env, die Kopien.",
              ),
              <See key="d" id="project-db" />,
            ],
            [
              tr("Estensioni", "Extensions", "Extensions", "Erweiterungen"),
              tr(
                "Le estensioni dell'editor attive in questo progetto.",
                "The editor extensions active in this project.",
                "Les extensions de l'éditeur actives dans ce projet.",
                "Die in diesem Projekt aktiven Erweiterungen des Editors.",
              ),
              <See key="e" id="project-editor" />,
            ],
            [
              tr("Editor", "Editor", "Éditeur", "Editor"),
              tr(
                "Impostazioni dell'editor e server MCP del progetto.",
                "Editor settings and the project's MCP servers.",
                "Paramètres de l'éditeur et serveurs MCP du projet.",
                "Einstellungen des Editors und MCP-Server des Projekts.",
              ),
              <See key="m" id="project-editor" />,
            ],
            [
              tr("Git", "Git", "Git", "Git"),
              tr(
                "Remoto, ramo e sincronizzazione, in sola lettura.",
                "Remote, branch and sync, read-only.",
                "Dépôt distant, branche et synchronisation, en lecture seule.",
                "Remote, Branch und Synchronisierung, schreibgeschützt.",
              ),
              <See key="g" id="project-git" />,
            ],
          ]}
        />
        <Callout
          tone="info"
          title={tr(
            "Come il kit riconosce un progetto",
            "How the kit recognizes a project",
            "Comment le kit reconnaît un projet",
            "Wie das Kit ein Projekt erkennt",
          )}
        >
          {tr(
            "Il kit guarda i file della cartella: package.json vuol dire Node.js (e con dentro vite o next, un sito web), composer.json PHP, artisan Laravel, pyproject.toml Python, Cargo.toml Rust, go.mod Go e così via. Da lì ricava il tipo, la piattaforma, i linguaggi, i componenti che servono e le azioni. La sezione «Componenti» › «Come è stato riconosciuto» dice da quali file è partito. Il riconoscimento si può correggere con il file panel.json.",
            "The kit looks at the files in the folder: package.json means Node.js (and with vite or next inside, a website), composer.json PHP, artisan Laravel, pyproject.toml Python, Cargo.toml Rust, go.mod Go and so on. From there it works out the type, the platform, the languages, the components needed and the actions. The “Components” › “How it was recognized” section says which files it started from. The recognition can be corrected with the panel.json file.",
            "Le kit examine les fichiers du dossier : package.json veut dire Node.js (et avec vite ou next dedans, un site web), composer.json PHP, artisan Laravel, pyproject.toml Python, Cargo.toml Rust, go.mod Go, et ainsi de suite. Il en déduit le type, la plateforme, les langages, les composants nécessaires et les actions. La section « Composants » › « Comment il a été reconnu » indique de quels fichiers il est parti. La reconnaissance peut être corrigée avec le fichier panel.json.",
            "Das Kit sieht sich die Dateien im Ordner an: package.json bedeutet Node.js (und mit vite oder next darin eine Website), composer.json PHP, artisan Laravel, pyproject.toml Python, Cargo.toml Rust, go.mod Go und so weiter. Daraus leitet es den Typ, die Plattform, die Sprachen, die benötigten Komponenten und die Aktionen ab. Der Abschnitt „Komponenten“ › „Wie es erkannt wurde“ sagt, von welchen Dateien es ausgegangen ist. Die Erkennung lässt sich mit der Datei panel.json korrigieren.",
          )}
        </Callout>
      </Section>

      <Related ids={["project-run", "project-new", "window"]} />
    </>
  ),

  "project-run": () => (
    <>
      <InBrief
        points={[
          tr(
            "Un'azione è un comando del progetto con un nome: «npm run dev», «php artisan serve», il sito web…",
            "An action is a project command with a name: “npm run dev”, “php artisan serve”, the website…",
            "Une action est une commande du projet avec un nom : « npm run dev », « php artisan serve », le site web…",
            "Eine Aktion ist ein Befehl des Projekts mit einem Namen: „npm run dev“, „php artisan serve“, die Website…",
          ),
          tr(
            "Le azioni web ricevono una porta dal kit, che aspetta che rispondano e ti dà l'indirizzo da aprire.",
            "Web actions get a port from the kit, which waits for them to respond and gives you the address to open.",
            "Les actions web reçoivent un port du kit, qui attend qu'elles répondent et vous donne l'adresse à ouvrir.",
            "Web-Aktionen erhalten vom Kit einen Port; das Kit wartet, bis sie antworten, und gibt dir die Adresse zum Öffnen.",
          ),
          tr(
            "La stella «Predefinita» sceglie l'azione del pulsante «Avvia».",
            "The “Default” star picks the action of the “Start” button.",
            "L'étoile « Par défaut » choisit l'action du bouton « Démarrer ».",
            "Der Stern „Standard“ legt die Aktion der Schaltfläche „Starten“ fest.",
          ),
          tr(
            "Le azioni si riconoscono da sole; le tue si aggiungono con «Nuova azione» e finiscono nel file panel.json del progetto.",
            "Actions are recognized by themselves; you add your own with “New action” and they end up in the project's panel.json file.",
            "Les actions se reconnaissent toutes seules ; les vôtres s'ajoutent avec « Nouvelle action » et vont dans le fichier panel.json du projet.",
            "Aktionen werden von selbst erkannt; deine eigenen fügst du mit „Neue Aktion“ hinzu, sie landen in der Datei panel.json des Projekts.",
          ),
        ]}
      />

      <GuideShot
        src="project-run.webp"
        title={tr(
          "La pagina di un progetto, sezione «Avvio»",
          "A project's page, “Run” section",
          "La page d'un projet, section « Lancement »",
          "Die Seite eines Projekts, Bereich „Start“",
        )}
        caption={tr(
          "In alto i pulsanti Avvia, Editor, Terminale e Agente; sotto, l'azione scelta con stato, comando e porta, e l'elenco delle azioni.",
          "At the top the Start, Editor, Terminal and Agent buttons; below, the chosen action with state, command and port, and the list of actions.",
          "En haut les boutons Démarrer, Éditeur, Terminal et Agent ; en dessous, l'action choisie avec état, commande et port, et la liste des actions.",
          "Oben die Schaltflächen Starten, Editor, Terminal und Agent; darunter die gewählte Aktion mit Status, Befehl und Port sowie die Liste der Aktionen.",
        )}
      />

      <Section
        title={tr("La sezione «Avvio»", "The “Run” section", "La section « Lancement »", "Der Abschnitt „Start“")}
      >
        <P>
          {tr(
            "In alto c'è il dettaglio dell'azione scelta (all'inizio quella predefinita), sotto l'elenco «Azioni». Un clic su una riga la sceglie e ne mostra l'output; ▶ e ■ sulla riga la avviano e la fermano.",
            "At the top is the detail of the chosen action (at first the default one), below the “Actions” list. A click on a row selects it and shows its output; ▶ and ■ on the row start and stop it.",
            "En haut se trouve le détail de l'action choisie (au début, celle par défaut), en dessous la liste « Actions ». Un clic sur une ligne la choisit et affiche sa sortie ; ▶ et ■ sur la ligne la démarrent et l'arrêtent.",
            "Oben steht das Detail der gewählten Aktion (anfangs die Standardaktion), darunter die Liste „Aktionen“. Ein Klick auf eine Zeile wählt sie aus und zeigt ihre Ausgabe; ▶ und ■ in der Zeile starten und beenden sie.",
          )}
        </P>
        <Table
          columns={[
            tr("Voce", "Item", "Élément", "Eintrag"),
            tr("Che cosa dice", "What it says", "Ce qu'il indique", "Was er anzeigt"),
          ]}
          rows={[
            [
              tr("Stato", "Status", "État", "Status"),
              tr(
                "fermo, in avvio, in ascolto (azione web pronta), in esecuzione, terminato, fermato, errore (con il codice di uscita).",
                "stopped, starting, listening (web action ready), running, exited, stopped, error (with the exit code).",
                "arrêté, démarrage, en écoute (action web prête), en cours d'exécution, terminé, arrêté, erreur (avec le code de sortie).",
                "gestoppt, startet, lauscht (Web-Aktion bereit), läuft, beendet, gestoppt, Fehler (mit dem Exit-Code).",
              ),
            ],
            [
              tr("Comando", "Command", "Commande", "Befehl"),
              tr(
                "Il comando eseguito, oppure il server che serve il sito (file server del kit, Apache o Caddy).",
                "The command run, or the server that serves the website (the kit's file server, Apache or Caddy).",
                "La commande exécutée, ou le serveur qui sert le site (serveur de fichiers du kit, Apache ou Caddy).",
                "Der ausgeführte Befehl oder der Server, der die Website bereitstellt (Dateiserver des Kits, Apache oder Caddy).",
              ),
            ],
            [
              tr("Cartella", "Folder", "Dossier", "Ordner"),
              tr(
                "Dove viene eseguito il comando.",
                "Where the command is run.",
                "Où la commande est exécutée.",
                "Wo der Befehl ausgeführt wird.",
              ),
            ],
            [
              tr("Porta", "Port", "Port", "Port"),
              tr(
                "Solo per le azioni web: «automatica» o un numero scelto da te.",
                "Only for web actions: “automatic” or a number you choose.",
                "Seulement pour les actions web : « automatique » ou un numéro choisi par vous.",
                "Nur für Web-Aktionen: „automatisch“ oder eine von dir gewählte Zahl.",
              ),
            ],
            [
              tr("Indirizzo", "Address", "Adresse", "Adresse"),
              tr(
                "Mentre l'azione è in ascolto, l'indirizzo da aprire (un collegamento).",
                "While the action is listening, the address to open (a link).",
                "Pendant que l'action est en écoute, l'adresse à ouvrir (un lien).",
                "Während die Aktion lauscht, die zu öffnende Adresse (ein Link).",
              ),
            ],
            [
              tr("Origine", "Source", "Origine", "Herkunft"),
              tr(
                "panel.json, se l'azione è scritta a mano nel progetto.",
                "panel.json, if the action is written by hand in the project.",
                "panel.json, si l'action est écrite à la main dans le projet.",
                "panel.json, wenn die Aktion von Hand im Projekt geschrieben ist.",
              ),
            ],
          ]}
        />
        <P>
          {tr(
            "Sotto il dettaglio c'è il riquadro «Output di …» con la casella «Segui», che tiene in vista le ultime righe. Il kit conserva le ultime 3000 righe di ogni azione; le sue note compaiono come righe che iniziano con [Atollo]. I pulsanti in alto: la stella «Predefinita», «Apri nel browser» (mentre un'azione web è in ascolto) e «Togli l'output» (quando l'azione è finita).",
            "Below the detail is the “Output of …” pane with the “Follow” checkbox, which keeps the last lines in view. The kit keeps the last 3000 lines of each action; its notes appear as lines starting with [Atollo]. The buttons at the top: the “Default” star, “Open in the browser” (while a web action is listening) and “Clear the output” (when the action has finished).",
            "Sous le détail se trouve le volet « Sortie de … » avec la case « Suivre », qui garde les dernières lignes en vue. Le kit conserve les 3000 dernières lignes de chaque action ; ses notes apparaissent comme des lignes qui commencent par [Atollo]. Les boutons en haut : l'étoile « Par défaut », « Ouvrir dans le navigateur » (pendant qu'une action web est en écoute) et « Effacer la sortie » (quand l'action est terminée).",
            "Unter dem Detail steht der Bereich „Ausgabe von …“ mit dem Kästchen „Folgen“, das die letzten Zeilen im Blick behält. Das Kit bewahrt die letzten 3000 Zeilen jeder Aktion auf; seine Hinweise erscheinen als Zeilen, die mit [Atollo] beginnen. Die Schaltflächen oben: der Stern „Standard“, „Im Browser öffnen“ (während eine Web-Aktion lauscht) und „Ausgabe entfernen“ (wenn die Aktion beendet ist).",
          )}
        </P>
      </Section>

      <Section
        title={tr(
          "Che cosa succede quando premi «Avvia»",
          "What happens when you press “Start”",
          "Ce qui se passe quand vous appuyez sur « Démarrer »",
          "Was passiert, wenn du auf „Starten“ klickst",
        )}
      >
        <Flow
          steps={[
            {
              title: tr("Rilegge", "Rereads", "Relit", "Neu einlesen"),
              text: tr(
                "Riconosce di nuovo il progetto: panel.json potrebbe essere cambiato.",
                "Recognizes the project again: panel.json might have changed.",
                "Reconnaît de nouveau le projet : panel.json a pu changer.",
                "Erkennt das Projekt erneut: panel.json könnte sich geändert haben.",
              ),
            },
            {
              title: tr("Installa", "Installs", "Installe", "Installieren"),
              text: tr(
                "Se all'azione manca un componente, lo installa prima.",
                "If the action is missing a component, it installs it first.",
                "S'il manque un composant à l'action, il l'installe d'abord.",
                "Fehlt der Aktion eine Komponente, installiert es sie zuerst.",
              ),
            },
            {
              title: tr("Prepara", "Prepares", "Prépare", "Vorbereiten"),
              text: tr(
                "Ambiente del kit, versioni del progetto, porta, variabili dell'azione.",
                "The kit's environment, the project's versions, port, the action's variables.",
                "Environnement du kit, versions du projet, port, variables de l'action.",
                "Umgebung des Kits, Versionen des Projekts, Port, Variablen der Aktion.",
              ),
            },
            {
              title: tr("Avvia", "Starts", "Démarre", "Starten"),
              text: tr(
                "Esegue il comando nella cartella dell'azione.",
                "Runs the command in the action's folder.",
                "Exécute la commande dans le dossier de l'action.",
                "Führt den Befehl im Ordner der Aktion aus.",
              ),
            },
            {
              title: tr("Aspetta", "Waits", "Attend", "Warten"),
              text: tr(
                "Per un'azione web, finché la porta risponde: allora è «in ascolto».",
                "For a web action, until the port responds: then it is “listening”.",
                "Pour une action web, jusqu'à ce que le port réponde : elle est alors « en écoute ».",
                "Bei einer Web-Aktion, bis der Port antwortet: Dann hat sie den Status „lauscht“.",
              ),
            },
          ]}
        />
        <Bullets
          items={[
            tr(
              "Il browser non si apre da solo: usa «Apri nel browser» (qui, nel «Pannello rapido» o nel pannello «Attività»). L'indirizzo è di solito http://localhost:<porta>/.",
              "The browser does not open by itself: use “Open in the browser” (here, in the “Quick panel” or in the “Activity” panel). The address is usually http://localhost:<port>/.",
              "Le navigateur ne s'ouvre pas tout seul : utilisez « Ouvrir dans le navigateur » (ici, dans le « Panneau rapide » ou dans le panneau « Activité »). L'adresse est en général http://localhost:<port>/.",
              "Der Browser öffnet sich nicht von selbst: Verwende „Im Browser öffnen“ (hier, in der „Schnellübersicht“ oder im Bereich „Aktivität“). Die Adresse ist meist http://localhost:<Port>/.",
            ),
            tr(
              "Se il programma ignora la porta assegnata e ne stampa un'altra nell'output, il kit segue quella.",
              "If the program ignores the assigned port and prints another one in the output, the kit follows that one.",
              "Si le programme ignore le port attribué et en affiche un autre dans la sortie, le kit suit celui-là.",
              "Wenn das Programm den zugewiesenen Port ignoriert und einen anderen in der Ausgabe ausgibt, folgt das Kit diesem.",
            ),
            tr(
              "«Ferma» chiude l'intero albero di processi dell'azione. Chiudendo Atollo si fermano tutte le azioni.",
              "“Stop” closes the action's whole process tree. Closing Atollo stops all actions.",
              "« Arrêter » ferme toute l'arborescence de processus de l'action. Fermer Atollo arrête toutes les actions.",
              "„Beenden“ schließt den gesamten Prozessbaum der Aktion. Wenn du Atollo schließt, werden alle Aktionen beendet.",
            ),
            tr(
              "Le azioni avviate compaiono anche nel pannello «Attività» e nel contatore della barra di stato; nel pannello «Ferma tutto», dopo una conferma, le ferma insieme ai database e ai server web in esecuzione.",
              "Running actions also appear in the “Activity” panel and in the status bar counter; in the panel “Stop all”, after a confirmation, stops them together with the running databases and web servers.",
              "Les actions démarrées apparaissent aussi dans le panneau « Activité » et dans le compteur de la barre d'état ; dans le panneau, « Tout arrêter », après une confirmation, les arrête avec les bases de données et les serveurs web en cours d'exécution.",
              "Gestartete Aktionen erscheinen auch im Bereich „Aktivität“ und im Zähler der Statusleiste; im Bereich beendet „Alle beenden“ sie nach einer Bestätigung zusammen mit den laufenden Datenbanken und Webservern.",
            ),
          ]}
        />
      </Section>

      <Section title={tr("La porta", "The port", "Le port", "Der Port")}>
        <P>
          {tr(
            "Ogni azione web ha una porta tutta sua, scelta dal kit tra 7100 e 7999 la prima volta e poi tenuta uguale a ogni avvio: così l'indirizzo non cambia. Se ti serve un numero preciso, scrivilo nel campo «Porta» e premi Invio (o esci dal campo); svuotandolo torni alla porta automatica.",
            "Each web action has a port of its own, chosen by the kit between 7100 and 7999 the first time and then kept the same at every start: that way the address does not change. If you need a specific number, type it in the “Port” field and press Enter (or leave the field); emptying it takes you back to the automatic port.",
            "Chaque action web a son propre port, choisi par le kit entre 7100 et 7999 la première fois puis conservé à chaque démarrage : ainsi l'adresse ne change pas. Si vous avez besoin d'un numéro précis, tapez-le dans le champ « Port » et appuyez sur Entrée (ou quittez le champ) ; en le vidant, vous revenez au port automatique.",
            "Jede Web-Aktion hat ihren eigenen Port, den das Kit beim ersten Mal zwischen 7100 und 7999 wählt und dann bei jedem Start beibehält: So ändert sich die Adresse nicht. Wenn du eine bestimmte Nummer brauchst, gib sie in das Feld „Port“ ein und drück Eingabe (oder verlass das Feld); wenn du es leerst, kehrst du zum automatischen Port zurück.",
          )}
        </P>
        <Bullets
          items={[
            tr(
              "La porta deve essere tra 1024 e 65535.",
              "The port must be between 1024 and 65535.",
              "Le port doit être compris entre 1024 et 65535.",
              "Der Port muss zwischen 1024 und 65535 liegen.",
            ),
            tr(
              "Le porte da 7000 a 7099 sono dell'editor: una di queste viene rifiutata con «Le porte da 7000 a 7099 sono dell'editor».",
              "Ports 7000 to 7099 belong to the editor: one of them is refused with “Ports 7000 to 7099 belong to the editor”.",
              "Les ports de 7000 à 7099 sont réservés à l'éditeur : l'un d'eux est refusé avec « Les ports 7000 à 7099 sont réservés à l'éditeur ».",
              "Die Ports 7000 bis 7099 gehören dem Editor: Einer davon wird mit „Die Ports 7000 bis 7099 gehören dem Editor“ abgelehnt.",
            ),
            tr(
              "Una porta già usata da un'altra azione viene rifiutata con «Porta già assegnata a …», seguito da progetto → azione.",
              "A port already used by another action is refused with “Port already assigned to …”, followed by project → action.",
              "Un port déjà utilisé par une autre action est refusé avec « Port déjà attribué à … », suivi de projet → action.",
              "Ein bereits von einer anderen Aktion verwendeter Port wird mit „Port bereits … zugewiesen“ abgelehnt, mit Projekt → Aktion.",
            ),
            tr(
              "Viene rifiutata anche una porta che ha già un pool dei server web o una versione di un database, e il messaggio dice di chi è: «La porta … è già del server …: scegline un'altra.» oppure «La porta … è già del database …: scegline un'altra.» (per esempio MySQL 8.4).",
              "A port that a web server pool or a database version already has is refused too, and the message names its owner: “Port … already belongs to server …: choose another one.” or “Port … already belongs to database …: choose another one.” (for example MySQL 8.4).",
              "Un port qu'a déjà un pool de serveurs web ou une version d'une base de données est refusé aussi, et le message dit à qui il est : « Le port … appartient déjà au serveur … : choisissez-en un autre. » ou « Le port … appartient déjà à la base de données … : choisissez-en un autre. » (par exemple MySQL 8.4).",
              "Auch ein Port, den schon ein Pool der Webserver oder eine Version einer Datenbank hat, wird abgelehnt, und die Meldung nennt den Besitzer: „Port … gehört bereits zum Server …: Wähle einen anderen.“ oder „Port … gehört bereits zur Datenbank …: Wähle einen anderen.“ (zum Beispiel MySQL 8.4).",
            ),
          ]}
        />
      </Section>

      <Section title={tr("L'azione predefinita", "The default action", "L'action par défaut", "Die Standardaktion")}>
        <P>
          {tr(
            "È quella che parte con il pulsante «Avvia» della pagina, con ▶ nella riga del progetto e con Maiusc+Invio in «Apri un progetto». Per cambiarla scegli l'azione nell'elenco e premi la stella «Predefinita». Quando servi il progetto come sito web, il sito diventa da solo l'azione predefinita.",
            "It is the one that starts with the page's “Start” button, with ▶ in the project row and with Shift+Enter in “Open a project”. To change it, choose the action in the list and press the “Default” star. When you serve the project as a website, the website becomes the default action by itself.",
            "C'est celle qui démarre avec le bouton « Démarrer » de la page, avec ▶ dans la ligne du projet et avec Maj+Entrée dans « Ouvrir un projet ». Pour la changer, choisissez l'action dans la liste et appuyez sur l'étoile « Par défaut ». Quand vous servez le projet comme site web, le site devient tout seul l'action par défaut.",
            "Es ist die Aktion, die mit der Schaltfläche „Starten“ der Seite, mit ▶ in der Projektzeile und mit Umschalt+Eingabe in „Projekt öffnen“ startet. Um sie zu ändern, wähle die Aktion in der Liste und klicke auf den Stern „Standard“. Wenn du das Projekt als Website bereitstellst, wird die Website von selbst zur Standardaktion.",
          )}
        </P>
      </Section>

      <Section
        title={tr(
          "Aggiungere, modificare e togliere azioni",
          "Adding, editing and removing actions",
          "Ajouter, modifier et retirer des actions",
          "Aktionen hinzufügen, bearbeiten und entfernen",
        )}
      >
        <Steps
          steps={[
            tr(
              "Nella sezione «Avvio» premi «Nuova azione», accanto al titolo «Azioni».",
              "In the “Run” section press “New action”, next to the “Actions” heading.",
              "Dans la section « Lancement », appuyez sur « Nouvelle action », à côté du titre « Actions ».",
              "Klicke im Bereich „Start“ auf „Neue Aktion“, neben der Überschrift „Aktionen“.",
            ),
            tr(
              "Compila «Nome», «Tipo» («Sito o server web», «App con finestra» o «Comando») e «Comando»; se serve, «Cartella» (relativa al progetto), «Indirizzo» (solo per un sito) e «Variabili», una NOME=valore per riga. Nel comando, {port} diventa la porta dell'azione.",
              "Fill in “Name”, “Type” (“Website or web server”, “Windowed app” or “Command”) and “Command”; if needed, “Folder” (relative to the project), “Address” (websites only) and “Variables”, one NAME=value per line. In the command, {port} becomes the action's port.",
              "Remplissez « Nom », « Type » (« Site ou serveur web », « Application avec fenêtre » ou « Commande ») et « Commande » ; si besoin, « Dossier » (relatif au projet), « Adresse » (sites seulement) et « Variables », une NOM=valeur par ligne. Dans la commande, {port} devient le port de l'action.",
              "Fülle „Name“, „Typ“ („Website oder Webserver“, „App mit Fenster“ oder „Befehl“) und „Befehl“ aus; bei Bedarf „Ordner“ (relativ zum Projekt), „Adresse“ (nur für Websites) und „Variablen“, eine NAME=Wert pro Zeile. Im Befehl wird {port} zum Port der Aktion.",
            ),
            tr(
              "Premi «Salva». L'azione compare nell'elenco; l'identificativo si ricava dal nome.",
              "Press “Save”. The action appears in the list; its id comes from the name.",
              "Appuyez sur « Enregistrer ». L'action apparaît dans la liste ; son identifiant vient du nom.",
              "Klicke auf „Speichern“. Die Aktion erscheint in der Liste; ihre ID ergibt sich aus dem Namen.",
            ),
          ]}
        />
        <Bullets
          items={[
            tr(
              "«Modifica», sull'azione scelta, riapre lo stesso modulo. Su un'azione riconosciuta salva una tua versione con lo stesso identificativo, che prende il suo posto.",
              "“Edit”, on the chosen action, opens the same form again. On a recognized action it saves your own version with the same id, which takes its place.",
              "« Modifier », sur l'action choisie, rouvre le même formulaire. Sur une action reconnue, il enregistre votre version avec le même identifiant, qui prend sa place.",
              "„Bearbeiten“ öffnet für die gewählte Aktion dasselbe Formular. Bei einer erkannten Aktion speichert es deine Version mit derselben ID, die an ihre Stelle tritt.",
            ),
            tr(
              "«Togli» c'è solo per le azioni tue e chiede conferma; se esiste un'azione riconosciuta con lo stesso identificativo, torna quella.",
              "“Remove” exists only for your own actions and asks for confirmation; if a recognized action has the same id, that one comes back.",
              "« Retirer » n'existe que pour vos actions et demande confirmation ; si une action reconnue a le même identifiant, elle revient.",
              "„Entfernen“ gibt es nur für deine eigenen Aktionen und fragt nach; hat eine erkannte Aktion dieselbe ID, kommt sie zurück.",
            ),
            tr(
              "Il sito web si modifica nella sezione «Sito web», non qui.",
              "The website is edited in the “Website” section, not here.",
              "Le site web se modifie dans la section « Site web », pas ici.",
              "Die Website bearbeitest du im Bereich „Website“, nicht hier.",
            ),
          ]}
        />
      </Section>

      <Section title={tr("Il file panel.json", "The panel.json file", "Le fichier panel.json", "Die Datei panel.json")}>
        <P>
          {tr(
            "Le azioni tue, la stella e il sito web stanno nel file panel.json nella cartella del progetto. Puoi anche scriverlo a mano e premere «Aggiorna». Il file sta nel progetto, quindi va in Git e vale per chiunque usi il progetto con Atollo.",
            "Your own actions, the star and the website live in the panel.json file in the project folder. You can also write it by hand and press “Refresh”. The file lives in the project, so it goes into Git and applies to anyone who uses the project with Atollo.",
            "Vos actions, l'étoile et le site web se trouvent dans le fichier panel.json dans le dossier du projet. Vous pouvez aussi l'écrire à la main et appuyer sur « Actualiser ». Le fichier est dans le projet, donc il va dans Git et vaut pour quiconque utilise le projet avec Atollo.",
            "Deine eigenen Aktionen, der Stern und die Website stehen in der Datei panel.json im Projektordner. Du kannst sie auch von Hand schreiben und auf „Aktualisieren“ klicken. Die Datei liegt im Projekt, kommt also in Git und gilt für alle, die das Projekt mit Atollo verwenden.",
          )}
        </P>
        <GuideCode
          title={tr("Esempio di panel.json", "Example panel.json", "Exemple de panel.json", "Beispiel für panel.json")}
          path={"projects\\gestionale\\panel.json"}
          code={PANEL_EXAMPLE}
        />
        <Table
          columns={[
            tr("Chiave", "Key", "Clé", "Schlüssel"),
            tr("Significato", "Meaning", "Signification", "Bedeutung"),
          ]}
          rows={[
            [
              <Code key="i">id</Code>,
              tr(
                "Identificativo dell'azione: minuscole, cifre e - _ : . (al massimo 40 caratteri).",
                "Identifier of the action: lowercase letters, digits and - _ : . (40 characters at most).",
                "Identifiant de l'action : minuscules, chiffres et - _ : . (40 caractères au maximum).",
                "Kennung der Aktion: Kleinbuchstaben, Ziffern und - _ : . (höchstens 40 Zeichen).",
              ),
            ],
            [
              <Code key="l">label</Code>,
              tr(
                "Il nome che vedi nell'elenco.",
                "The name you see in the list.",
                "Le nom que vous voyez dans la liste.",
                "Der Name, den du in der Liste siehst.",
              ),
            ],
            [
              <Code key="t">type</Code>,
              tr(
                "web (un server con porta), desktop (un'app con finestra) o command (un comando che fa qualcosa e finisce).",
                "web (a server with a port), desktop (an app with a window) or command (a command that does something and finishes).",
                "web (un serveur avec un port), desktop (une application avec fenêtre) ou command (une commande qui fait quelque chose et se termine).",
                "web (ein Server mit Port), desktop (eine App mit Fenster) oder command (ein Befehl, der etwas tut und endet).",
              ),
            ],
            [
              <Code key="c">command</Code>,
              tr(
                "Il comando. {port} viene sostituito con la porta dell'azione.",
                "The command. {port} is replaced with the action's port.",
                "La commande. {port} est remplacé par le port de l'action.",
                "Der Befehl. {port} wird durch den Port der Aktion ersetzt.",
              ),
            ],
            [
              <Code key="f">folder</Code>,
              tr(
                "La sottocartella in cui eseguirlo (facoltativa).",
                "The subfolder to run it in (optional).",
                "Le sous-dossier où l'exécuter (facultatif).",
                "Der Unterordner, in dem er ausgeführt wird (optional).",
              ),
            ],
            [
              <Code key="u">url</Code>,
              tr(
                "L'indirizzo da aprire, se non è http://localhost:{port}/ (facoltativo).",
                "The address to open, if it is not http://localhost:{port}/ (optional).",
                "L'adresse à ouvrir, si ce n'est pas http://localhost:{port}/ (facultatif).",
                "Die zu öffnende Adresse, wenn sie nicht http://localhost:{port}/ ist (optional).",
              ),
            ],
            [
              <Code key="e">env</Code>,
              tr(
                "Variabili d'ambiente in più solo per questa azione (facoltativo).",
                "Extra environment variables just for this action (optional).",
                "Variables d'environnement supplémentaires pour cette action seulement (facultatif).",
                "Zusätzliche Umgebungsvariablen nur für diese Aktion (optional).",
              ),
            ],
            [
              <Code key="d">default</Code>,
              tr(
                "In cima al file: l'id dell'azione predefinita.",
                "At the top of the file: the id of the default action.",
                "En tête du fichier : l'id de l'action par défaut.",
                "Oben in der Datei: die id der Standardaktion.",
              ),
            ],
            [
              <Code key="r">replace</Code>,
              tr(
                "In cima al file: true per togliere tutte le azioni riconosciute e tenere solo le tue.",
                "At the top of the file: true to remove all the recognized actions and keep only yours.",
                "En tête du fichier : true pour retirer toutes les actions reconnues et ne garder que les vôtres.",
                "Oben in der Datei: true, um alle erkannten Aktionen zu entfernen und nur deine zu behalten.",
              ),
            ],
          ]}
        />
        <Callout tone="info">
          {tr(
            "Un'azione con lo stesso id di una riconosciuta la sostituisce. Se il file non è un JSON valido, il kit lo ignora e lo dice in «Componenti» › «Come è stato riconosciuto»; correggilo prima di usare stella, sito web o «Nuova azione», che scrivono nello stesso file.",
            "An action with the same id as a recognized one replaces it. If the file is not valid JSON, the kit ignores it and says so in “Components” › “How it was recognized”; fix it before using the star, the website or “New action”, which write to the same file.",
            "Une action avec le même id qu'une action reconnue la remplace. Si le fichier n'est pas un JSON valide, le kit l'ignore et le signale dans « Composants » › « Comment il a été reconnu » ; corrigez-le avant d'utiliser l'étoile, le site web ou « Nouvelle action », qui écrivent dans le même fichier.",
            "Eine Aktion mit derselben id wie eine erkannte ersetzt diese. Ist die Datei kein gültiges JSON, ignoriert das Kit sie und meldet das unter „Komponenten“ › „Wie es erkannt wurde“; korrigiere sie, bevor du Stern, Website oder „Neue Aktion“ verwendest, die in dieselbe Datei schreiben.",
          )}
        </Callout>
        <Under>
          <Bullets
            items={[
              tr(
                "Il comando viene eseguito con cmd.exe su Windows e /bin/sh su macOS e Linux, nella cartella dell'azione, con l'ambiente del kit e le versioni del progetto.",
                "The command is run with cmd.exe on Windows and /bin/sh on macOS and Linux, in the action's folder, with the kit's environment and the project's versions.",
                "La commande est exécutée avec cmd.exe sous Windows et /bin/sh sous macOS et Linux, dans le dossier de l'action, avec l'environnement du kit et les versions du projet.",
                "Der Befehl wird unter Windows mit cmd.exe und unter macOS und Linux mit /bin/sh ausgeführt, im Ordner der Aktion, mit der Umgebung des Kits und den Versionen des Projekts.",
              ),
              tr(
                "Le azioni web ricevono le variabili PORT (e ROCKET_PORT) con la porta; tutte ricevono BROWSER=none, PYTHONUNBUFFERED=1 e ATOLLO_LANG. Se il progetto ha un database MySQL del kit e il suo .env non dice un'altra porta, ricevono anche DB_PORT.",
                "Web actions get the PORT (and ROCKET_PORT) variables with the port; all of them get BROWSER=none, PYTHONUNBUFFERED=1 and ATOLLO_LANG. If the project has a kit MySQL database and its .env does not name another port, they also get DB_PORT.",
                "Les actions web reçoivent les variables PORT (et ROCKET_PORT) avec le port ; toutes reçoivent BROWSER=none, PYTHONUNBUFFERED=1 et ATOLLO_LANG. Si le projet a une base de données MySQL du kit et que son .env n'indique pas un autre port, elles reçoivent aussi DB_PORT.",
                "Web-Aktionen erhalten die Variablen PORT (und ROCKET_PORT) mit dem Port; alle erhalten BROWSER=none, PYTHONUNBUFFERED=1 und ATOLLO_LANG. Wenn das Projekt eine MySQL-Datenbank des Kits hat und seine .env keinen anderen Port angibt, erhalten sie auch DB_PORT.",
              ),
              tr(
                "Le porte assegnate e quelle scelte da te sono in data\\projects\\<progetto>\\ports.json, fuori dal progetto.",
                "The assigned ports and the ones you chose are in data\\projects\\<project>\\ports.json, outside the project.",
                "Les ports attribués et ceux que vous avez choisis sont dans data\\projects\\<projet>\\ports.json, en dehors du projet.",
                "Die zugewiesenen und die von dir gewählten Ports stehen in data\\projects\\<Projekt>\\ports.json, außerhalb des Projekts.",
              ),
              tr(
                "Ogni azione gira in un suo gruppo di processi (un Job Object su Windows): «Ferma» chiude anche i processi figli, per esempio il server avviato da npm.",
                "Each action runs in its own process group (a Job Object on Windows): “Stop” also closes the child processes, for example the server started by npm.",
                "Chaque action tourne dans son propre groupe de processus (un Job Object sous Windows) : « Arrêter » ferme aussi les processus enfants, par exemple le serveur lancé par npm.",
                "Jede Aktion läuft in einer eigenen Prozessgruppe (einem Job Object unter Windows): „Beenden“ schließt auch die Kindprozesse, zum Beispiel den von npm gestarteten Server.",
              ),
              tr(
                "Nei file vecchi sono ancora accettate le chiavi in italiano (azioni, etichetta, comando, cartella, predefinita) e {porta} al posto di {port}.",
                "In old files the Italian keys (azioni, etichetta, comando, cartella, predefinita) and {porta} instead of {port} are still accepted.",
                "Dans les anciens fichiers, les clés en italien (azioni, etichetta, comando, cartella, predefinita) et {porta} à la place de {port} sont encore acceptées.",
                "In alten Dateien werden die italienischen Schlüssel (azioni, etichetta, comando, cartella, predefinita) und {porta} statt {port} weiterhin akzeptiert.",
              ),
            ]}
          />
        </Under>
      </Section>

      <Related ids={["websites", "project-page", "troubleshooting"]} />
    </>
  ),

  "project-versions": () => (
    <>
      <InBrief
        points={[
          tr(
            "La sezione «Componenti» dice quali strumenti del kit servono al progetto e li installa con un clic.",
            "The “Components” section says which kit tools the project needs and installs them with one click.",
            "La section « Composants » indique quels outils du kit sont nécessaires au projet et les installe en un clic.",
            "Der Abschnitt „Komponenten“ sagt, welche Werkzeuge des Kits das Projekt braucht, und installiert sie mit einem Klick.",
          ),
          tr(
            "In «Versioni» scegli, solo per questo progetto, la versione di ogni linguaggio: per esempio Node 20 qui e Node 24 altrove.",
            "In “Versions” you choose, for this project only, the version of each language: for example Node 20 here and Node 24 elsewhere.",
            "Dans « Versions », vous choisissez, pour ce projet seulement, la version de chaque langage : par exemple Node 20 ici et Node 24 ailleurs.",
            "Unter „Versionen“ wählst du nur für dieses Projekt die Version jeder Sprache: zum Beispiel Node 20 hier und Node 24 anderswo.",
          ),
          tr(
            "Un progetto nuovo tiene le versioni più recenti che c'erano quando è stato creato, finché non le cambi.",
            "A new project keeps the latest versions that were there when it was created, until you change them.",
            "Un nouveau projet garde les versions les plus récentes présentes au moment de sa création, jusqu'à ce que vous les changiez.",
            "Ein neues Projekt behält die neuesten Versionen, die bei seiner Erstellung vorhanden waren, bis du sie änderst.",
          ),
          tr(
            "La scelta vale per azioni, terminale, agenti, editor e server web del progetto.",
            "The choice applies to the project's actions, terminal, agents, editor and web server.",
            "Le choix vaut pour les actions, le terminal, les agents, l'éditeur et le serveur web du projet.",
            "Die Auswahl gilt für Aktionen, Terminal, Agenten, Editor und Webserver des Projekts.",
          ),
        ]}
      />

      <GuideShot
        src="project-versions.webp"
        title={tr(
          "La sezione «Componenti»",
          "The “Components” section",
          "La section « Composants »",
          "Der Bereich „Komponenten“",
        )}
        caption={tr(
          "I componenti che servono al progetto, come è stato riconosciuto e le versioni dei suoi linguaggi.",
          "The components the project needs, how it was recognized and the versions of its languages.",
          "Les composants nécessaires au projet, comment il a été reconnu et les versions de ses langages.",
          "Die Komponenten, die das Projekt braucht, wie es erkannt wurde und die Versionen seiner Sprachen.",
        )}
      />

      <Section
        title={tr("Componenti che servono", "Components needed", "Composants nécessaires", "Benötigte Komponenten")}
      >
        <P>
          {tr(
            "La parte «Componenti» elenca gli strumenti del kit di cui il progetto ha bisogno, ciascuno «installato» o «da installare». Se manca qualcosa, «Installa ciò che manca» li installa e aspetta che abbiano finito. La parte «Come è stato riconosciuto» spiega da quali file il kit ha capito il tipo di progetto, con eventuali avvisi (per esempio «Dipendenze non installate: esegui prima npm install»).",
            "The “Components” part lists the kit tools the project needs, each “installed” or “to install”. If something is missing, “Install what is missing” installs them and waits for them to finish. The “How it was recognized” part explains from which files the kit worked out the project type, with any warnings (for example “Dependencies not installed: run npm install first”).",
            "La partie « Composants » liste les outils du kit dont le projet a besoin, chacun « installé » ou « à installer ». S'il manque quelque chose, « Installer ce qui manque » les installe et attend qu'ils aient terminé. La partie « Comment il a été reconnu » explique à partir de quels fichiers le kit a compris le type de projet, avec d'éventuels avertissements (par exemple « Dépendances non installées : exécutez d'abord npm install »).",
            "Der Teil „Komponenten“ listet die Werkzeuge des Kits auf, die das Projekt braucht, jedes „installiert“ oder „zu installieren“. Fehlt etwas, installiert „Fehlendes installieren“ sie und wartet, bis sie fertig sind. Der Teil „Wie es erkannt wurde“ erklärt, aus welchen Dateien das Kit den Projekttyp abgeleitet hat, mit eventuellen Warnungen (zum Beispiel „Abhängigkeiten nicht installiert: Führe zuerst npm install aus“).",
          )}
        </P>
        <Callout tone="info">
          {tr(
            "«Installa ciò che manca» installa i componenti del kit (Node.js, PHP, Python…), non le dipendenze del progetto: per quelle esegui npm install, composer install o simili, con un'azione o nel terminale del progetto.",
            "“Install what is missing” installs the kit's components (Node.js, PHP, Python…), not the project's dependencies: for those run npm install, composer install or similar, with an action or in the project terminal.",
            "« Installer ce qui manque » installe les composants du kit (Node.js, PHP, Python…), pas les dépendances du projet : pour celles-ci, exécutez npm install, composer install ou équivalent, avec une action ou dans le terminal du projet.",
            "„Fehlendes installieren“ installiert die Komponenten des Kits (Node.js, PHP, Python…), nicht die Abhängigkeiten des Projekts: Für diese führst du npm install, composer install oder Ähnliches aus, mit einer Aktion oder im Terminal des Projekts.",
          )}
        </Callout>
      </Section>

      <Section
        title={tr(
          "Scegliere le versioni del progetto",
          "Choosing the project's versions",
          "Choisir les versions du projet",
          "Die Versionen des Projekts wählen",
        )}
      >
        <P>
          {tr(
            "La parte «Versioni» ha due gruppi: «Linguaggi attivi nel progetto», aperto, e «Linguaggi non presenti nel progetto», chiuso, per impostare altri linguaggi da usare nel terminale o negli script.",
            "The “Versions” part has two groups: “Languages active in the project”, open, and “Languages not present in the project”, closed, to set other languages to use in the terminal or in scripts.",
            "La partie « Versions » a deux groupes : « Langages actifs dans le projet », ouvert, et « Langages absents du projet », fermé, pour régler d'autres langages à utiliser dans le terminal ou dans les scripts.",
            "Der Teil „Versionen“ hat zwei Gruppen: „Im Projekt aktive Sprachen“, geöffnet, und „Im Projekt nicht vorhandene Sprachen“, geschlossen, um weitere Sprachen für das Terminal oder Skripte einzustellen.",
          )}
        </P>
        <Steps
          steps={[
            tr(
              "Apri il menu accanto al linguaggio.",
              "Open the menu next to the language.",
              "Ouvrez le menu à côté du langage.",
              "Öffne das Menü neben der Sprache.",
            ),
            tr(
              "La prima voce, «La più recente (…)», lascia decidere al kit: userà sempre la versione più recente presente su disco.",
              "The first entry, “The latest (…)”, lets the kit decide: it will always use the newest version on disk.",
              "La première entrée, « La plus récente (…) », laisse le kit décider : il utilisera toujours la version la plus récente présente sur le disque.",
              "Der erste Eintrag, „Die neueste (…)“, überlässt die Entscheidung dem Kit: Es verwendet immer die neueste Version auf der Festplatte.",
            ),
            tr(
              "Sotto «Scaricate» ci sono le versioni già nel kit; sotto «Da scaricare: 8.x» quelle che la fonte offre e il kit non ha ancora.",
              "Under “Downloaded” are the versions already in the kit; under “To download: 8.x” those the source offers and the kit does not have yet.",
              "Sous « Téléchargées » se trouvent les versions déjà dans le kit ; sous « À télécharger : 8.x » celles que la source propose et que le kit n'a pas encore.",
              "Unter „Heruntergeladen“ stehen die Versionen, die schon im Kit sind; unter „Herunterzuladen: 8.x“ die, die die Quelle anbietet und das Kit noch nicht hat.",
            ),
            tr(
              "Scegli una versione: se non è scaricata parte subito il download («Download di … in corso…»). Finché non è pronta il progetto usa la più recente («… non è scaricata: per ora si usa la …», con «Scarica» o «Riprova»).",
              "Choose a version: if it is not downloaded, the download starts right away (“Downloading …”). Until it is ready the project uses the newest one (“… is not downloaded: … is used for now”, with “Download” or “Retry”).",
              "Choisissez une version : si elle n'est pas téléchargée, le téléchargement démarre aussitôt (« Téléchargement de … en cours… »). Tant qu'elle n'est pas prête, le projet utilise la plus récente (« … n'est pas téléchargée : … est utilisée pour l'instant », avec « Télécharger » ou « Réessayer »).",
              "Wähle eine Version: Ist sie nicht heruntergeladen, startet der Download sofort („… wird heruntergeladen…“). Bis sie bereit ist, verwendet das Projekt die neueste („… ist nicht heruntergeladen: vorerst wird … verwendet“, mit „Herunterladen“ oder „Erneut versuchen“).",
            ),
            tr(
              "Il pulsante con le frecce, «Cerca altre versioni di …», chiede di nuovo alla fonte l'elenco delle versioni.",
              "The button with the arrows, “Look for other versions of …”, asks the source again for the list of versions.",
              "Le bouton avec les flèches, « Chercher d'autres versions de … », redemande à la source la liste des versions.",
              "Die Schaltfläche mit den Pfeilen, „Weitere Versionen von … suchen“, fragt die Quelle erneut nach der Liste der Versionen.",
            ),
          ]}
        />
        <P>
          {tr(
            "Sotto il menu il kit può mostrare dei suggerimenti: «Richiesto da package.json: >=20» se il progetto dichiara una versione, oppure «Dai container del progetto (Compose o Dockerfile): php 8.3» con il pulsante «Usa 8.3».",
            "Below the menu the kit can show some suggestions: “Required by package.json: >=20” if the project declares a version, or “From the project's containers (Compose or Dockerfile): php 8.3” with the “Use 8.3” button.",
            "Sous le menu, le kit peut afficher des suggestions : « Requis par package.json : >=20 » si le projet déclare une version, ou « D'après les conteneurs du projet (Compose ou Dockerfile) : php 8.3 » avec le bouton « Utiliser 8.3 ».",
            "Unter dem Menü kann das Kit Vorschläge zeigen: „Benötigt von package.json: >=20“, wenn das Projekt eine Version angibt, oder „Aus den Containern des Projekts (Compose oder Dockerfile): php 8.3“ mit der Schaltfläche „8.3 verwenden“.",
          )}
        </P>
        <Callout
          tone="tip"
          title={tr(
            "Quando vale la nuova versione",
            "When the new version takes effect",
            "Quand la nouvelle version s'applique",
            "Wann die neue Version gilt",
          )}
        >
          {tr(
            "Per le azioni, il terminale del progetto e gli agenti dal prossimo avvio; per i terminali dell'editor da quelli aperti dopo la modifica; per il sito web servito da Caddy o Apache dal prossimo avvio del sito (il server sceglie il PHP del progetto).",
            "For the actions, the project terminal and the agents from the next start; for the editor's terminals from those opened after the change; for the website served by Caddy or Apache from the next start of the website (the server picks the project's PHP).",
            "Pour les actions, le terminal du projet et les agents, dès le prochain démarrage ; pour les terminaux de l'éditeur, dès ceux ouverts après la modification ; pour le site web servi par Caddy ou Apache, dès le prochain démarrage du site (le serveur choisit le PHP du projet).",
            "Für die Aktionen, das Terminal des Projekts und die Agenten ab dem nächsten Start; für die Terminals des Editors ab denen, die nach der Änderung geöffnet werden; für die von Caddy oder Apache bereitgestellte Website ab dem nächsten Start der Website (der Server wählt das PHP des Projekts).",
          )}
        </Callout>
      </Section>

      <Section
        title={tr(
          "Da dove vengono le versioni di partenza",
          "Where the starting versions come from",
          "D'où viennent les versions de départ",
          "Woher die Ausgangsversionen kommen",
        )}
      >
        <Bullets
          items={[
            tr(
              "Un progetto creato dall'app (vuoto o con un framework) fissa le versioni più recenti presenti in quel momento: un aggiornamento futuro del kit non cambia sotto i piedi un progetto che funziona.",
              "A project created by the app (empty or with a framework) pins the latest versions present at that moment: a future kit update does not change things under the feet of a project that works.",
              "Un projet créé par l'application (vide ou avec un framework) fixe les versions les plus récentes présentes à ce moment-là : une future mise à jour du kit ne change pas sous ses pieds un projet qui fonctionne.",
              "Ein von der App erstelltes Projekt (leer oder mit einem Framework) legt die zu diesem Zeitpunkt neuesten vorhandenen Versionen fest: Eine künftige Aktualisierung des Kits verändert ein funktionierendes Projekt nicht unbemerkt.",
            ),
            tr(
              "Un progetto clonato «Da un indirizzo Git» usa le versioni che il suo codice chiede, se il kit le riconosce: .nvmrc, .node-version, .python-version, pyproject.toml, composer.json, .php-version, go.mod, global.json, .ruby-version, pom.xml e altri.",
              "A project cloned “From a Git address” uses the versions its code asks for, if the kit recognizes them: .nvmrc, .node-version, .python-version, pyproject.toml, composer.json, .php-version, go.mod, global.json, .ruby-version, pom.xml and others.",
              "Un projet cloné « Depuis une adresse Git » utilise les versions que son code demande, si le kit les reconnaît : .nvmrc, .node-version, .python-version, pyproject.toml, composer.json, .php-version, go.mod, global.json, .ruby-version, pom.xml et d'autres.",
              "Ein „Von einer Git-Adresse“ geklontes Projekt verwendet die Versionen, die sein Code verlangt, wenn das Kit sie erkennt: .nvmrc, .node-version, .python-version, pyproject.toml, composer.json, .php-version, go.mod, global.json, .ruby-version, pom.xml und andere.",
            ),
            rich(
              tr(
                "Il riepilogo di chi chiede che cosa è nella vista «Relazioni» della pagina Strumenti: {0}.",
                "The summary of who asks for what is in the “Relations” view of the Tools page: {0}.",
                "Le récapitulatif de qui demande quoi se trouve dans la vue « Relations » de la page Outils : {0}.",
                "Die Übersicht, wer was verlangt, steht in der Ansicht „Beziehungen“ der Seite Werkzeuge: {0}.",
              ),
              <See key="o" id="tools-overview" />,
            ),
          ]}
        />
        <Under>
          <Bullets
            items={[
              tr(
                "La scelta viene salvata in data\\projects\\<progetto>\\dev-settings.json, alla voce versions; la prima voce del menu la toglie.",
                "The choice is saved in data\\projects\\<project>\\dev-settings.json, under versions; the first menu entry removes it.",
                "Le choix est enregistré dans data\\projects\\<projet>\\dev-settings.json, sous la clé versions ; la première entrée du menu le retire.",
                "Die Auswahl wird in data\\projects\\<Projekt>\\dev-settings.json unter versions gespeichert; der erste Menüeintrag entfernt sie.",
              ),
              tr(
                "Il kit scrive anche versions.json e versions.txt nella stessa cartella: li leggono il profilo della PowerShell e quello di Git Bash per mettere nel PATH le versioni giuste quando un terminale si apre in quel progetto.",
                "The kit also writes versions.json and versions.txt in the same folder: the PowerShell and Git Bash profiles read them to put the right versions in the PATH when a terminal opens in that project.",
                "Le kit écrit aussi versions.json et versions.txt dans le même dossier : les profils de PowerShell et de Git Bash les lisent pour mettre dans le PATH les bonnes versions quand un terminal s'ouvre dans ce projet.",
                "Das Kit schreibt außerdem versions.json und versions.txt in denselben Ordner: Die Profile von PowerShell und Git Bash lesen sie, um die richtigen Versionen in den PATH zu setzen, wenn sich ein Terminal in diesem Projekt öffnet.",
              ),
              tr(
                "Oltre al PATH, il kit imposta JAVA_HOME, DOTNET_ROOT e CP_PHP sulle versioni scelte.",
                "Besides the PATH, the kit sets JAVA_HOME, DOTNET_ROOT and CP_PHP to the chosen versions.",
                "En plus du PATH, le kit règle JAVA_HOME, DOTNET_ROOT et CP_PHP sur les versions choisies.",
                "Neben dem PATH setzt das Kit JAVA_HOME, DOTNET_ROOT und CP_PHP auf die gewählten Versionen.",
              ),
              tr(
                "La versione del server di database del progetto si sceglie nella sezione «Database», non qui.",
                "The version of the project's database server is chosen in the “Databases” section, not here.",
                "La version du serveur de bases de données du projet se choisit dans la section « Bases de données », pas ici.",
                "Die Version des Datenbankservers des Projekts wählst du im Abschnitt „Datenbanken“, nicht hier.",
              ),
            ]}
          />
        </Under>
      </Section>

      <Related ids={["tools-versions", "project-db", "tools-overview"]} />
    </>
  ),

  "project-editor": () => (
    <>
      <InBrief
        points={[
          tr(
            "«Editor» apre il progetto in VSCodium, dentro una finestra di Atollo, con tutti gli strumenti del kit.",
            "“Editor” opens the project in VSCodium, inside an Atollo window, with all the kit's tools.",
            "« Éditeur » ouvre le projet dans VSCodium, dans une fenêtre d'Atollo, avec tous les outils du kit.",
            "„Editor“ öffnet das Projekt in VSCodium, in einem Atollo-Fenster, mit allen Werkzeugen des Kits.",
          ),
          tr(
            "«Terminale» apre una console del kit nella cartella del progetto.",
            "“Terminal” opens a kit console in the project folder.",
            "« Terminal » ouvre une console du kit dans le dossier du projet.",
            "„Terminal“ öffnet eine Konsole des Kits im Projektordner.",
          ),
          tr(
            "Le estensioni si scelgono per progetto; quelle installate valgono per tutto l'editor.",
            "Extensions are chosen per project; the installed ones apply to the whole editor.",
            "Les extensions se choisissent par projet ; celles qui sont installées valent pour tout l'éditeur.",
            "Erweiterungen werden pro Projekt gewählt; die installierten gelten für den ganzen Editor.",
          ),
          tr(
            "Nella sezione «Editor» imposti rientro, formattazione, cartelle nascoste e server MCP del progetto.",
            "In the “Editor” section you set indentation, formatting, hidden folders and the project's MCP servers.",
            "Dans la section « Éditeur », vous réglez l'indentation, la mise en forme, les dossiers masqués et les serveurs MCP du projet.",
            "Im Abschnitt „Editor“ stellst du Einrückung, Formatierung, ausgeblendete Ordner und die MCP-Server des Projekts ein.",
          ),
        ]}
      />

      <Section title={tr("L'editor", "The editor", "L'éditeur", "Der Editor")}>
        <P>
          {tr(
            "L'editor del kit è VSCodium, la versione libera di Visual Studio Code. Il pulsante «Editor» (nella pagina del progetto, nella riga dell'elenco o nel «Pannello rapido») avvia il server dell'editor la prima volta e apre una finestra di Atollo sul progetto. Nella barra di stato compare «Editor attivo».",
            "The kit's editor is VSCodium, the free version of Visual Studio Code. The “Editor” button (on the project page, in the list row or in the “Quick panel”) starts the editor server the first time and opens an Atollo window on the project. “Editor running” appears in the status bar.",
            "L'éditeur du kit est VSCodium, la version libre de Visual Studio Code. Le bouton « Éditeur » (dans la page du projet, dans la ligne de la liste ou dans le « Panneau rapide ») démarre le serveur de l'éditeur la première fois et ouvre une fenêtre d'Atollo sur le projet. Dans la barre d'état apparaît « Éditeur actif ».",
            "Der Editor des Kits ist VSCodium, die freie Version von Visual Studio Code. Die Schaltfläche „Editor“ (auf der Seite des Projekts, in der Zeile der Liste oder in der „Schnellübersicht“) startet beim ersten Mal den Server des Editors und öffnet ein Atollo-Fenster mit dem Projekt. In der Statusleiste erscheint „Editor aktiv“.",
          )}
        </P>
        <Bullets
          items={[
            tr(
              "I terminali dell'editor hanno già l'ambiente del kit. Su Windows i profili sono «Atollo» (PowerShell, predefinito), «Atollo - Git Bash» e «Atollo - cmd»; su macOS e Linux «Atollo», «Atollo - Bash» e, su macOS, «Atollo - zsh».",
              "The editor's terminals already have the kit's environment. On Windows the profiles are “Atollo” (PowerShell, default), “Atollo - Git Bash” and “Atollo - cmd”; on macOS and Linux “Atollo”, “Atollo - Bash” and, on macOS, “Atollo - zsh”.",
              "Les terminaux de l'éditeur ont déjà l'environnement du kit. Sous Windows, les profils sont « Atollo » (PowerShell, par défaut), « Atollo - Git Bash » et « Atollo - cmd » ; sous macOS et Linux « Atollo », « Atollo - Bash » et, sous macOS, « Atollo - zsh ».",
              "Die Terminals des Editors haben bereits die Umgebung des Kits. Unter Windows heißen die Profile „Atollo“ (PowerShell, Standard), „Atollo - Git Bash“ und „Atollo - cmd“; unter macOS und Linux „Atollo“, „Atollo - Bash“ und, unter macOS, „Atollo - zsh“.",
            ),
            tr(
              "Commit, push, confronto delle modifiche: tutto ciò che fa Git nell'editor funziona con l'account GitHub del kit.",
              "Commit, push, comparing changes: everything Git does in the editor works with the kit's GitHub account.",
              "Commit, push, comparaison des modifications : tout ce que fait Git dans l'éditeur fonctionne avec le compte GitHub du kit.",
              "Commit, Push, Vergleich der Änderungen: Alles, was Git im Editor tut, funktioniert mit dem GitHub-Konto des Kits.",
            ),
            tr(
              "Il menu «Account» dell'editor e le sue funzioni «Pubblica su GitHub» e «Clona da GitHub» non vedono l'account del kit: usa Git, la pagina Progetti o GitHub CLI.",
              "The editor's “Accounts” menu and its “Publish to GitHub” and “Clone from GitHub” functions do not see the kit's account: use Git, the Projects page or GitHub CLI.",
              "Le menu « Comptes » de l'éditeur et ses fonctions « Publier sur GitHub » et « Cloner depuis GitHub » ne voient pas le compte du kit : utilisez Git, la page Projets ou GitHub CLI.",
              "Das Menü „Konten“ des Editors und seine Funktionen „In GitHub veröffentlichen“ und „Aus GitHub klonen“ sehen das Konto des Kits nicht: Verwende Git, die Seite Projekte oder GitHub CLI.",
            ),
          ]}
        />
        <Callout tone="info">
          {tr(
            "Se con l'editor aperto cambi nelle Impostazioni la lingua, la rete (per esempio il proxy) o i temporanei dei programmi, l'app propone «Riavvia l'editor», perché i terminali dell'editor usino i nuovi valori.",
            "If, with the editor open, you change the language, the network (for example the proxy) or the programs' temporary files in Settings, the app suggests “Restart the editor”, so that the editor's terminals use the new values.",
            "Si, avec l'éditeur ouvert, vous changez dans les Paramètres la langue, le réseau (par exemple le proxy) ou les fichiers temporaires des programmes, l'application propose « Redémarrer l'éditeur », pour que les terminaux de l'éditeur utilisent les nouvelles valeurs.",
            "Wenn du bei geöffnetem Editor in den Einstellungen die Sprache, das Netzwerk (zum Beispiel den Proxy) oder die temporären Dateien der Programme änderst, schlägt die App „Editor neu starten“ vor, damit die Terminals des Editors die neuen Werte verwenden.",
          )}
        </Callout>
      </Section>

      <Section
        title={tr(
          "Il terminale del progetto",
          "The project terminal",
          "Le terminal du projet",
          "Das Terminal des Projekts",
        )}
      >
        <P>
          {tr(
            "«Terminale» apre una console del kit nella cartella del progetto: su Windows una finestra di console classica con la PowerShell 7 del kit; su macOS e Linux una scheda del pannello dei terminali dentro Atollo. Gli strumenti sono quelli del kit, alle versioni scelte per il progetto. Il prompt mostra Atollo e la cartella.",
            "“Terminal” opens a kit console in the project folder: on Windows a classic console window with the kit's PowerShell 7; on macOS and Linux a tab of the terminal panel inside Atollo. The tools are the kit's, at the versions chosen for the project. The prompt shows Atollo and the folder.",
            "« Terminal » ouvre une console du kit dans le dossier du projet : sous Windows, une fenêtre de console classique avec le PowerShell 7 du kit ; sous macOS et Linux, un onglet du panneau des terminaux dans Atollo. Les outils sont ceux du kit, aux versions choisies pour le projet. L'invite affiche Atollo et le dossier.",
            "„Terminal“ öffnet eine Konsole des Kits im Projektordner: unter Windows ein klassisches Konsolenfenster mit der PowerShell 7 des Kits; unter macOS und Linux einen Tab des Terminalbereichs in Atollo. Die Werkzeuge sind die des Kits, in den für das Projekt gewählten Versionen. Die Eingabeaufforderung zeigt Atollo und den Ordner.",
          )}
        </P>
        <P>
          {rich(
            tr(
              "Per console in altre cartelle o con strumenti scelti da te c'è la pagina Terminali: {0}.",
              "For consoles in other folders or with tools you choose there is the Terminals page: {0}.",
              "Pour des consoles dans d'autres dossiers ou avec des outils de votre choix, il y a la page Terminaux : {0}.",
              "Für Konsolen in anderen Ordnern oder mit Werkzeugen deiner Wahl gibt es die Seite Terminals: {0}.",
            ),
            <See key="t" id="terminals" />,
          )}
        </P>
      </Section>

      <Section
        title={tr(
          "La sezione «Estensioni»",
          "The “Extensions” section",
          "La section « Extensions »",
          "Der Abschnitt „Erweiterungen“",
        )}
      >
        <P>
          {tr(
            "Atollo abbina a ogni linguaggio le sue estensioni dell'editor, prese da Open VSX (il catalogo libero di estensioni). Un progetto riceve quelle dei suoi linguaggi: la sezione le mostra in gruppi (uno per linguaggio, poi «Strumenti» e «Aggiunte da te»), con lo stato «installata», «da installare» o «disattivata».",
            "Atollo pairs each language with its editor extensions, taken from Open VSX (the free extension catalog). A project gets those of its languages: the section shows them in groups (one per language, then “Tools” and “Added by you”), with the status “installed”, “to install” or “off”.",
            "Atollo associe à chaque langage ses extensions de l'éditeur, prises sur Open VSX (le catalogue libre d'extensions). Un projet reçoit celles de ses langages : la section les affiche en groupes (un par langage, puis « Outils » et « Ajoutées par vous »), avec l'état « installée », « à installer » ou « désactivée ».",
            "Atollo ordnet jeder Sprache ihre Editor-Erweiterungen zu, aus Open VSX (dem freien Erweiterungskatalog). Ein Projekt erhält die seiner Sprachen: Der Abschnitt zeigt sie in Gruppen (eine pro Sprache, dann „Werkzeuge“ und „Von dir hinzugefügt“), mit dem Status „installiert“, „zu installieren“ oder „deaktiviert“.",
          )}
        </P>
        <Steps
          steps={[
            tr(
              "Togli la spunta a un'estensione per disattivarla in questo progetto, o rimettila.",
              "Untick an extension to turn it off in this project, or tick it again.",
              "Décochez une extension pour la désactiver dans ce projet, ou recochez-la.",
              "Entferne das Häkchen bei einer Erweiterung, um sie in diesem Projekt zu deaktivieren, oder setz es wieder.",
            ),
            tr(
              "Per aggiungerne una che non c'è, scrivi il suo identificativo editore.nome (per esempio ms-python.python) nel campo «Aggiungi un'estensione» e premi «Aggiungi».",
              "To add one that is not there, type its publisher.name identifier (for example ms-python.python) in the “Add an extension” field and press “Add”.",
              "Pour en ajouter une qui n'y est pas, tapez son identifiant editeur.nom (par exemple ms-python.python) dans le champ « Ajouter une extension » et appuyez sur « Ajouter ».",
              "Um eine hinzuzufügen, die fehlt, gib ihre Kennung herausgeber.name (zum Beispiel ms-python.python) in das Feld „Erweiterung hinzufügen“ ein und klicke auf „Hinzufügen“.",
            ),
          ]}
        />
        <Callout tone="info">
          {tr(
            "Le estensioni installate valgono in tutto l'editor; quelle nuove si installano quando l'editor è aperto. In cima, il riepilogo dice quante sono attive e quante da installare.",
            "Installed extensions apply to the whole editor; new ones are installed when the editor is open. At the top, the summary says how many are active and how many are to install.",
            "Les extensions installées valent pour tout l'éditeur ; les nouvelles s'installent quand l'éditeur est ouvert. En haut, le récapitulatif indique combien sont actives et combien sont à installer.",
            "Installierte Erweiterungen gelten für den ganzen Editor; neue werden installiert, wenn der Editor geöffnet ist. Oben sagt die Zusammenfassung, wie viele aktiv und wie viele zu installieren sind.",
          )}
        </Callout>
      </Section>

      <Section
        title={tr("La sezione «Editor»", "The “Editor” section", "La section « Éditeur »", "Der Abschnitt „Editor“")}
      >
        <P>{tr("Due riquadri.", "Two panes.", "Deux volets.", "Zwei Bereiche.")}</P>
        <Table
          columns={[
            tr("Impostazione", "Setting", "Paramètre", "Einstellung"),
            tr("Valori", "Values", "Valeurs", "Werte"),
          ]}
          rows={[
            [
              tr(
                "Formatta al salvataggio",
                "Format on save",
                "Formater à l'enregistrement",
                "Beim Speichern formatieren",
              ),
              tr(
                "come l'editor, sì, no",
                "like the editor, yes, no",
                "comme l'éditeur, oui, non",
                "wie der Editor, ja, nein",
              ),
            ],
            [
              tr("Rientro", "Indentation", "Indentation", "Einrückung"),
              tr(
                "come l'editor, 2 spazi, 4 spazi, tabulazione",
                "like the editor, 2 spaces, 4 spaces, tab",
                "comme l'éditeur, 2 espaces, 4 espaces, tabulation",
                "wie der Editor, 2 Leerzeichen, 4 Leerzeichen, Tabulator",
              ),
            ],
            [
              tr(
                "Cartelle nascoste nell'editor",
                "Folders hidden in the editor",
                "Dossiers masqués dans l'éditeur",
                "Im Editor ausgeblendete Ordner",
              ),
              tr(
                "Una per riga, per esempio **/vendor o **/node_modules: non compaiono nell'elenco dei file.",
                "One per line, for example **/vendor or **/node_modules: they do not appear in the file list.",
                "Un par ligne, par exemple **/vendor ou **/node_modules : ils n'apparaissent pas dans la liste des fichiers.",
                "Einer pro Zeile, zum Beispiel **/vendor oder **/node_modules: Sie erscheinen nicht in der Dateiliste.",
              ),
            ],
          ]}
        />
        <P>
          {tr(
            "«Salva» le scrive nel file .vscode\\settings.json del progetto; le altre impostazioni del file restano. Se il file contiene commenti o non è JSON semplice, Atollo non lo riscrive e ti chiede di modificarlo nell'editor.",
            "“Save” writes them in the project's .vscode\\settings.json file; the other settings in the file stay. If the file contains comments or is not plain JSON, Atollo does not rewrite it and asks you to edit it in the editor.",
            "« Enregistrer » les écrit dans le fichier .vscode\\settings.json du projet ; les autres paramètres du fichier restent. Si le fichier contient des commentaires ou n'est pas du JSON simple, Atollo ne le réécrit pas et vous demande de le modifier dans l'éditeur.",
            "„Speichern“ schreibt sie in die Datei .vscode\\settings.json des Projekts; die anderen Einstellungen der Datei bleiben. Enthält die Datei Kommentare oder ist sie kein einfaches JSON, schreibt Atollo sie nicht neu und bittet dich, sie im Editor zu bearbeiten.",
          )}
        </P>
        <P>
          {rich(
            tr(
              "Il secondo riquadro, «Server MCP del progetto», aggiunge server MCP che valgono solo in questo progetto, oltre a quelli del kit. Come funzionano e come si aggiungono: {0}.",
              "The second pane, “Project MCP servers”, adds MCP servers that apply only in this project, on top of the kit's. How they work and how to add them: {0}.",
              "Le second volet, « Serveurs MCP du projet », ajoute des serveurs MCP qui valent seulement dans ce projet, en plus de ceux du kit. Comment ils fonctionnent et comment les ajouter : {0}.",
              "Der zweite Bereich, „MCP-Server des Projekts“, fügt MCP-Server hinzu, die nur in diesem Projekt gelten, zusätzlich zu denen des Kits. Wie sie funktionieren und wie man sie hinzufügt: {0}.",
            ),
            <See key="a" id="agents-shared" />,
          )}
        </P>
      </Section>

      <Related ids={["terminals", "agents-shared", "tools-overview"]} />
    </>
  ),

  "project-git": () => (
    <>
      <InBrief
        points={[
          tr(
            "La sezione «Git» mostra lo stato del repository: remoto, ramo, collegamento al remoto, sincronizzazione.",
            "The “Git” section shows the repository status: remote, branch, link to the remote, sync.",
            "La section « Git » montre l'état du dépôt : dépôt distant, branche, lien avec le dépôt distant, synchronisation.",
            "Der Abschnitt „Git“ zeigt den Status des Repositorys: Remote, Branch, Verbindung zum Remote, Synchronisierung.",
          ),
          tr(
            "È in sola lettura: commit, push e l'elenco delle modifiche si fanno nell'editor, in un terminale o con un agente.",
            "It is read-only: commits, pushes and the list of changes are done in the editor, in a terminal or with an agent.",
            "Elle est en lecture seule : les commits, les push et la liste des modifications se font dans l'éditeur, dans un terminal ou avec un agent.",
            "Er ist schreibgeschützt: Commits, Pushes und die Liste der Änderungen erledigst du im Editor, in einem Terminal oder mit einem Agenten.",
          ),
          tr(
            "Git usa l'account GitHub del kit senza chiedere password.",
            "Git uses the kit's GitHub account without asking for passwords.",
            "Git utilise le compte GitHub du kit sans demander de mot de passe.",
            "Git verwendet das GitHub-Konto des Kits, ohne nach Passwörtern zu fragen.",
          ),
        ]}
      />

      <Section title={tr("Che cosa mostra", "What it shows", "Ce qu'elle montre", "Was er zeigt")}>
        <Table
          columns={[
            tr("Voce", "Item", "Élément", "Eintrag"),
            tr("Che cosa dice", "What it says", "Ce qu'il indique", "Was er anzeigt"),
          ]}
          rows={[
            [
              tr("Remoto", "Remote", "Dépôt distant", "Remote"),
              tr(
                "L'indirizzo del repository remoto (origin), o «nessuno».",
                "The address of the remote repository (origin), or “none”.",
                "L'adresse du dépôt distant (origin), ou « aucun ».",
                "Die Adresse des Remote-Repositorys (origin) oder „keiner“.",
              ),
            ],
            [
              tr("Ramo", "Branch", "Branche", "Branch"),
              tr(
                "Il ramo corrente; «nessun ramo» se sei su un commit preciso.",
                "The current branch; “no branch” if you are on a specific commit.",
                "La branche courante ; « aucune branche » si vous êtes sur un commit précis.",
                "Der aktuelle Branch; „kein Branch“, wenn du auf einem bestimmten Commit bist.",
              ),
            ],
            [
              tr("Remoto del ramo", "Remote branch", "Branche distante", "Remote-Branch"),
              tr(
                "Il ramo remoto collegato, o «ramo non collegato al remoto».",
                "The linked remote branch, or “branch not linked to a remote”.",
                "La branche distante liée, ou « branche non liée au dépôt distant ».",
                "Der verbundene Remote-Branch oder „Branch nicht mit Remote verbunden“.",
              ),
            ],
            [
              tr("Sincronizzato", "In sync", "Synchronisé", "Synchron"),
              tr(
                "«sì, allo stesso commit», «no: ci sono commit da inviare o da ricevere», oppure «non si sa». Si riferisce all'ultimo aggiornamento dal remoto (fetch).",
                "“yes, at the same commit”, “no: there are commits to push or to pull”, or “unknown”. It refers to the last update from the remote (fetch).",
                "« oui, au même commit », « non : il y a des commits à envoyer ou à recevoir », ou « inconnu ». Cela se rapporte à la dernière mise à jour depuis le dépôt distant (fetch).",
                "„ja, beim selben Commit“, „nein: Es gibt Commits zum Senden oder Empfangen“ oder „unbekannt“. Es bezieht sich auf die letzte Aktualisierung vom Remote (fetch).",
              ),
            ],
          ]}
        />
        <P>
          {tr(
            "«Apri sul web» apre il repository nel browser (per indirizzi https e per GitHub via ssh). Se la cartella non è un repository, la sezione lo dice.",
            "“Open on the web” opens the repository in the browser (for https addresses and for GitHub via ssh). If the folder is not a repository, the section says so.",
            "« Ouvrir sur le web » ouvre le dépôt dans le navigateur (pour les adresses https et pour GitHub via ssh). Si le dossier n'est pas un dépôt, la section le dit.",
            "„Im Web öffnen“ öffnet das Repository im Browser (für https-Adressen und für GitHub über ssh). Ist der Ordner kein Repository, sagt der Abschnitt das.",
          )}
        </P>
      </Section>

      <Section
        title={tr(
          "Dove si lavora con Git",
          "Where you work with Git",
          "Où travailler avec Git",
          "Wo du mit Git arbeitest",
        )}
      >
        <Bullets
          items={[
            tr(
              "Nell'editor: il pannello «Controllo del codice sorgente» per commit, push, pull, rami e confronti.",
              "In the editor: the “Source Control” panel for commits, push, pull, branches and comparisons.",
              "Dans l'éditeur : le panneau « Contrôle de code source » pour les commits, push, pull, branches et comparaisons.",
              "Im Editor: der Bereich „Quellcodeverwaltung“ für Commits, Push, Pull, Branches und Vergleiche.",
            ),
            tr(
              "Nel terminale del progetto o dell'editor: i comandi git e gh, già autenticati.",
              "In the project or editor terminal: the git and gh commands, already authenticated.",
              "Dans le terminal du projet ou de l'éditeur : les commandes git et gh, déjà authentifiées.",
              "Im Terminal des Projekts oder des Editors: die Befehle git und gh, bereits angemeldet.",
            ),
            tr(
              "Con un agente: Claude Code, Codex e Antigravity possono fare commit, creare rami e aprire pull request.",
              "With an agent: Claude Code, Codex and Antigravity can commit, create branches and open pull requests.",
              "Avec un agent : Claude Code, Codex et Antigravity peuvent faire des commits, créer des branches et ouvrir des pull requests.",
              "Mit einem Agenten: Claude Code, Codex und Antigravity können Commits machen, Branches erstellen und Pull Requests öffnen.",
            ),
          ]}
        />
        <Callout
          tone="warn"
          title={tr(
            "Prima del primo commit",
            "Before the first commit",
            "Avant le premier commit",
            "Vor dem ersten Commit",
          )}
        >
          {tr(
            "Git vuole un nome e un'e-mail per i commit. Se il kit ha accesso a GitHub li prende dall'account (con l'indirizzo noreply di GitHub); altrimenti scrivili in Impostazioni › «Account GitHub» › «Chi firma i commit». Vengono salvati nel file .gitconfig del kit e ti seguono su ogni PC.",
            "Git needs a name and an email for commits. If the kit is signed in to GitHub it takes them from the account (with GitHub's noreply address); otherwise type them in Settings › “GitHub account” › “Who signs the commits”. They are saved in the kit's .gitconfig file and follow you on every PC.",
            "Git a besoin d'un nom et d'une adresse e-mail pour les commits. Si le kit est connecté à GitHub, il les prend du compte (avec l'adresse noreply de GitHub) ; sinon, saisissez-les dans Paramètres › « Compte GitHub » › « Qui signe les commits ». Ils sont enregistrés dans le fichier .gitconfig du kit et vous suivent sur chaque PC.",
            "Git braucht einen Namen und eine E-Mail für Commits. Wenn das Kit bei GitHub angemeldet ist, nimmt es sie aus dem Konto (mit der noreply-Adresse von GitHub); sonst gib sie unter Einstellungen › „GitHub-Konto“ › „Wer die Commits signiert“ ein. Sie werden in der Datei .gitconfig des Kits gespeichert und begleiten dich auf jeden PC.",
          )}
        </Callout>
        <Under>
          <P>
            {tr(
              "Atollo legge lo stato direttamente dai file della cartella .git (HEAD, config, riferimenti), senza eseguire Git nel progetto: così non può far partire hook o programmi del repository solo perché hai aperto la pagina.",
              "Atollo reads the status directly from the files in the .git folder (HEAD, config, refs), without running Git in the project: that way it cannot start the repository's hooks or programs just because you opened the page.",
              "Atollo lit l'état directement dans les fichiers du dossier .git (HEAD, config, références), sans exécuter Git dans le projet : ainsi, il ne peut pas lancer des hooks ou des programmes du dépôt simplement parce que vous avez ouvert la page.",
              "Atollo liest den Status direkt aus den Dateien des Ordners .git (HEAD, config, Referenzen), ohne Git im Projekt auszuführen: So kann es keine Hooks oder Programme des Repositorys starten, nur weil du die Seite geöffnet hast.",
            )}
          </P>
        </Under>
      </Section>

      <Related ids={["github", "project-editor", "agents"]} />
    </>
  ),

  "project-cleanup": () => (
    <>
      <InBrief
        points={[
          tr(
            "Il kit tiene alcuni dati di ogni progetto fuori dal progetto, in data\\projects: impostazioni, porte, password dei database, copie.",
            "The kit keeps some data of each project outside the project, in data\\projects: settings, ports, database passwords, copies.",
            "Le kit garde certaines données de chaque projet en dehors du projet, dans data\\projects : paramètres, ports, mots de passe des bases de données, copies.",
            "Das Kit bewahrt einige Daten jedes Projekts außerhalb des Projekts auf, in data\\projects: Einstellungen, Ports, Passwörter der Datenbanken, Kopien.",
          ),
          tr(
            "Dall'app si elimina solo un progetto con la cartella vuota; per gli altri cancelli la cartella tu.",
            "From the app you can only delete a project with an empty folder; for the others you delete the folder yourself.",
            "Depuis l'application, on ne supprime qu'un projet dont le dossier est vide ; pour les autres, c'est vous qui supprimez le dossier.",
            "In der App lässt sich nur ein Projekt mit leerem Ordner löschen; bei den anderen löschst du den Ordner selbst.",
          ),
          tr(
            "Quando la cartella sparisce, i suoi dati diventano «Dati senza progetto»: li ricolleghi o li elimini.",
            "When the folder disappears, its data becomes “Data without a project”: you relink it or delete it.",
            "Quand le dossier disparaît, ses données deviennent des « Données sans projet » : vous les reliez ou les supprimez.",
            "Wenn der Ordner verschwindet, werden seine Daten zu „Daten ohne Projekt“: Du verknüpfst sie neu oder löschst sie.",
          ),
          tr(
            "I dati senza progetto non vengono mai eliminati da soli.",
            "Data without a project is never deleted by itself.",
            "Les données sans projet ne sont jamais supprimées toutes seules.",
            "Daten ohne Projekt werden nie von selbst gelöscht.",
          ),
        ]}
      />

      <Section
        title={tr(
          "Dove sono i dati di un progetto",
          "Where a project's data is",
          "Où sont les données d'un projet",
          "Wo die Daten eines Projekts liegen",
        )}
      >
        <Table
          columns={[
            tr(
              "Dentro il progetto (va in Git)",
              "Inside the project (goes into Git)",
              "Dans le projet (va dans Git)",
              "Im Projekt (kommt in Git)",
            ),
            tr(
              "Fuori, in data\\projects\\<progetto> (resta nel kit)",
              "Outside, in data\\projects\\<project> (stays in the kit)",
              "En dehors, dans data\\projects\\<projet> (reste dans le kit)",
              "Außerhalb, in data\\projects\\<Projekt> (bleibt im Kit)",
            ),
          ]}
          rows={[
            [
              tr(
                "Il tuo codice, panel.json, .env, .vscode\\settings.json, .mcp.json",
                "Your code, panel.json, .env, .vscode\\settings.json, .mcp.json",
                "Votre code, panel.json, .env, .vscode\\settings.json, .mcp.json",
                "Dein Code, panel.json, .env, .vscode\\settings.json, .mcp.json",
              ),
              tr(
                "Versioni scelte, estensioni, preferiti (dev-settings.json), porte (ports.json), configurazioni di Apache e Caddy, password e nomi dei database (private), copie dei database (private\\backups)",
                "Chosen versions, extensions, favorites (dev-settings.json), ports (ports.json), Apache and Caddy configurations, database passwords and names (private), database copies (private\\backups)",
                "Versions choisies, extensions, favoris (dev-settings.json), ports (ports.json), configurations d'Apache et de Caddy, mots de passe et noms des bases de données (private), copies des bases de données (private\\backups)",
                "Gewählte Versionen, Erweiterungen, Favoriten (dev-settings.json), Ports (ports.json), Konfigurationen von Apache und Caddy, Passwörter und Namen der Datenbanken (private), Kopien der Datenbanken (private\\backups)",
              ),
            ],
          ]}
        />
      </Section>

      <Section title={tr("Eliminare un progetto", "Deleting a project", "Supprimer un projet", "Ein Projekt löschen")}>
        <P>
          {tr(
            "Se la cartella del progetto è vuota, la sezione «Avvio» mostra «Elimina la cartella vuota». Per un progetto con dei file, il kit non elimina niente al posto tuo: cancella o sposta la cartella in Esplora file, poi premi «Aggiorna»; i suoi dati compaiono in «Dati senza progetto».",
            "If the project folder is empty, the “Run” section shows “Delete the empty folder”. For a project with files, the kit does not delete anything for you: delete or move the folder in File Explorer, then press “Refresh”; its data appears in “Data without a project”.",
            "Si le dossier du projet est vide, la section « Lancement » affiche « Supprimer le dossier vide ». Pour un projet avec des fichiers, le kit ne supprime rien à votre place : supprimez ou déplacez le dossier dans l'Explorateur de fichiers, puis appuyez sur « Actualiser » ; ses données apparaissent dans « Données sans projet ».",
            "Ist der Projektordner leer, zeigt der Abschnitt „Start“ „Leeren Ordner löschen“. Bei einem Projekt mit Dateien löscht das Kit nichts für dich: Lösche oder verschiebe den Ordner im Datei-Explorer und klicke dann auf „Aktualisieren“; seine Daten erscheinen in „Daten ohne Projekt“.",
          )}
        </P>
        <P>
          {tr(
            "La finestra «Eliminare …?» elenca che cosa sparirà (il progetto sparisce subito anche dalla pagina «Server Web»):",
            "The “Delete …?” window lists what will disappear (the project also disappears at once from the “Web Servers” page):",
            "La fenêtre « Supprimer … ? » liste ce qui va disparaître (le projet disparaît aussitôt aussi de la page « Serveurs web ») :",
            "Das Fenster „… löschen?“ listet auf, was verschwinden wird (das Projekt verschwindet sofort auch von der Seite „Webserver“):",
          )}
        </P>
        <Bullets
          items={[
            tr(
              "la cartella vuota e le impostazioni del kit per quel progetto;",
              "the empty folder and the kit's settings for that project;",
              "le dossier vide et les paramètres du kit pour ce projet ;",
              "der leere Ordner und die Einstellungen des Kits für dieses Projekt;",
            ),
            tr(
              "il database e il suo utente su MySQL o MongoDB (un server fermo viene avviato apposta);",
              "the database and its user on MySQL or MongoDB (a stopped server is started for the purpose);",
              "la base de données et son utilisateur sur MySQL ou MongoDB (un serveur arrêté est démarré exprès) ;",
              "die Datenbank und ihr Benutzer auf MySQL oder MongoDB (ein gestoppter Server wird eigens dafür gestartet);",
            ),
            tr(
              "le copie dei database, con «Apri la cartella delle copie» per salvarle prima altrove.",
              "the database copies, with “Open the copies folder” to save them elsewhere first.",
              "les copies des bases de données, avec « Ouvrir le dossier des copies » pour les mettre ailleurs avant.",
              "die Kopien der Datenbanken, mit „Ordner der Kopien öffnen“, um sie vorher anderswo zu sichern.",
            ),
          ]}
        />
        <P>
          {tr(
            "Se l'eliminazione di un database non riesce, non viene eliminato nient'altro; compare allora anche «Elimina senza i database», che lascia database e utenti sui server.",
            "If deleting a database fails, nothing else is deleted; “Delete without the databases” then also appears, which leaves databases and users on the servers.",
            "Si la suppression d'une base de données échoue, rien d'autre n'est supprimé ; apparaît alors aussi « Supprimer sans les bases de données », qui laisse les bases de données et les utilisateurs sur les serveurs.",
            "Wenn das Löschen einer Datenbank fehlschlägt, wird nichts anderes gelöscht; dann erscheint auch „Ohne die Datenbanken löschen“, das Datenbanken und Benutzer auf den Servern lässt.",
          )}
        </P>
        <Callout tone="warn">
          {tr(
            "Le copie dei database vengono eliminate insieme al progetto. Se ti servono, copiale prima fuori dal kit.",
            "The database copies are deleted together with the project. If you need them, copy them outside the kit first.",
            "Les copies des bases de données sont supprimées avec le projet. Si vous en avez besoin, copiez-les d'abord hors du kit.",
            "Die Kopien der Datenbanken werden zusammen mit dem Projekt gelöscht. Wenn du sie brauchst, kopiere sie vorher aus dem Kit heraus.",
          )}
        </Callout>
      </Section>

      <Section
        title={tr(
          "I «Dati senza progetto»",
          "The “Data without a project”",
          "Les « Données sans projet »",
          "Die „Daten ohne Projekt“",
        )}
      >
        <P>
          {tr(
            "Se rinomini o cancelli la cartella di un progetto fuori dall'app, il kit si ritrova dei dati senza progetto. Il riquadro «Dati senza progetto (N)» li elenca, con che cosa contengono: impostazioni, porte, database, copie.",
            "If you rename or delete a project's folder outside the app, the kit ends up with data without a project. The “Data without a project (N)” pane lists it, with what it contains: settings, ports, databases, copies.",
            "Si vous renommez ou supprimez le dossier d'un projet en dehors de l'application, le kit se retrouve avec des données sans projet. Le volet « Données sans projet (N) » les liste, avec ce qu'elles contiennent : paramètres, ports, bases de données, copies.",
            "Wenn du den Ordner eines Projekts außerhalb der App umbenennst oder löschst, hat das Kit Daten ohne Projekt. Der Bereich „Daten ohne Projekt (N)“ listet sie auf, mit ihrem Inhalt: Einstellungen, Ports, Datenbanken, Kopien.",
          )}
        </P>
        <Table
          columns={[
            tr("Pulsante", "Button", "Bouton", "Schaltfläche"),
            tr("Che cosa fa", "What it does", "Ce qu'il fait", "Was sie bewirkt"),
          ]}
          rows={[
            [
              tr("Collega a…", "Link to…", "Lier à…", "Verknüpfen mit…"),
              tr(
                "Sposta impostazioni, porte, database e copie a un progetto che scegli nel menu «Progetto». Database e utente mantengono il vecchio nome, così il file .env continua a funzionare.",
                "Moves settings, ports, databases and copies to a project you choose in the “Project” menu. Database and user keep the old name, so the .env file keeps working.",
                "Déplace paramètres, ports, bases de données et copies vers un projet que vous choisissez dans le menu « Projet ». La base de données et l'utilisateur gardent l'ancien nom, ainsi le fichier .env continue de fonctionner.",
                "Verschiebt Einstellungen, Ports, Datenbanken und Kopien zu einem Projekt, das du im Menü „Projekt“ wählst. Datenbank und Benutzer behalten den alten Namen, sodass die Datei .env weiter funktioniert.",
              ),
            ],
            [
              tr("Elimina", "Delete", "Supprimer", "Löschen"),
              tr(
                "Elimina quei dati, database compresi, con la stessa finestra di conferma dell'eliminazione di un progetto.",
                "Deletes that data, databases included, with the same confirmation window as when deleting a project.",
                "Supprime ces données, bases de données comprises, avec la même fenêtre de confirmation que pour la suppression d'un projet.",
                "Löscht diese Daten, einschließlich der Datenbanken, mit demselben Bestätigungsfenster wie beim Löschen eines Projekts.",
              ),
            ],
          ]}
        />
        <Callout
          tone="info"
          title={tr(
            "Quando «Collega a…» non si può usare",
            "When “Link to…” cannot be used",
            "Quand « Lier à… » ne peut pas être utilisé",
            "Wann „Verknüpfen mit…“ nicht verwendet werden kann",
          )}
        >
          {tr(
            "Il progetto di destinazione può avere al massimo le sue impostazioni (versioni, preferito, ultimo agente): quelle dei dati collegati prendono il loro posto. Se ha già porte scelte, server, un database o delle copie, il collegamento viene rifiutato, per non mescolare i dati di due progetti.",
            "The target project may have at most its settings (versions, favorite, last agent): those of the linked data take their place. If it already has chosen ports, servers, a database or copies, the link is refused, so the data of two projects never mix.",
            "Le projet de destination peut avoir au plus ses paramètres (versions, favori, dernier agent) : ceux des données liées prennent leur place. S'il a déjà des ports choisis, des serveurs, une base de données ou des copies, la liaison est refusée, pour ne pas mélanger les données de deux projets.",
            "Das Zielprojekt darf höchstens seine Einstellungen haben (Versionen, Favorit, letzter Agent): Die der verknüpften Daten treten an ihre Stelle. Hat es schon gewählte Ports, Server, eine Datenbank oder Kopien, wird die Verknüpfung abgelehnt, damit sich die Daten zweier Projekte nie mischen.",
          )}
        </Callout>
        <Callout tone="tip">
          {tr(
            "Per rinominare un progetto senza perdere niente: rinomina la cartella in Esplora file, torna su Atollo, premi «Aggiorna» e usa «Collega a…» sui suoi vecchi dati, scegliendo il progetto con il nome nuovo.",
            "To rename a project without losing anything: rename the folder in File Explorer, go back to Atollo, press “Refresh” and use “Link to…” on its old data, choosing the project with the new name.",
            "Pour renommer un projet sans rien perdre : renommez le dossier dans l'Explorateur de fichiers, revenez sur Atollo, appuyez sur « Actualiser » et utilisez « Lier à… » sur ses anciennes données, en choisissant le projet au nouveau nom.",
            "Um ein Projekt umzubenennen, ohne etwas zu verlieren: Benenne den Ordner im Datei-Explorer um, wechsle zurück zu Atollo, klicke auf „Aktualisieren“ und verwende „Verknüpfen mit…“ für seine alten Daten, mit dem Projekt unter dem neuen Namen als Ziel.",
          )}
        </Callout>
      </Section>

      <Related ids={["project-db", "disk-layout", "updates"]} />
    </>
  ),
};
