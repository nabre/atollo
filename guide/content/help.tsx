// Chapter 9 of the Guide, «Aiuto»: common problems with their message and
// remedy, frequent questions, a glossary.
import { rich, tr } from "../core/lang";
import { Bullets, Callout, Code, FilterList, InBrief, P, QA, Related, Section, See, Terms } from "../core/widgets";
import type { GuideProblem, TopicTexts } from "../core/types";

/** The problems by area: what the user sees (the message or the
 * symptom), what to do and where to read more. */
function problems(): { title: string; items: GuideProblem[] }[] {
  return [
    {
      title: tr(
        "Download e installazione",
        "Download and installation",
        "Téléchargement et installation",
        "Download und Installation",
      ),
      items: [
        {
          q: tr(
            "Un componente resta in «errore» con «non riuscito» e un messaggio di rete o di certificato.",
            "A component stays in “error” with “failed” and a network or certificate message.",
            "Un composant reste en « erreur » avec « échec » et un message de réseau ou de certificat.",
            "Eine Komponente bleibt auf „Fehler“ mit „fehlgeschlagen“ und einer Netzwerk- oder Zertifikatsmeldung.",
          ),
          a: tr(
            "Spesso è la rete: un proxy aziendale o l'ispezione HTTPS. Compila Impostazioni › Rete e riprova. Se non basta, usa il riquadro «Scaricalo dal browser» sotto la riga e consegna il file al kit.",
            "Often it is the network: a company proxy or HTTPS inspection. Fill in Settings › Network and try again. If that is not enough, use the “Download it in the browser” panel under the row and hand the file to the kit.",
            "C'est souvent le réseau : un proxy d'entreprise ou l'inspection HTTPS. Remplissez Paramètres › Réseau et réessayez. Si cela ne suffit pas, utilisez le panneau « Téléchargez-le dans le navigateur » sous la ligne et remettez le fichier au kit.",
            "Oft liegt es am Netzwerk: ein Firmen-Proxy oder die HTTPS-Untersuchung. Fülle Einstellungen › Netzwerk aus und versuche es erneut. Wenn das nicht reicht, verwende den Bereich „Im Browser herunterladen“ unter der Zeile und übergib die Datei dem Kit.",
          ),
          see: "network",
        },
        {
          q: tr(
            "«in uso: chiudi i programmi del kit e riprova»",
            "“in use: close the kit programs and try again”",
            "« en cours d'utilisation : fermez les programmes du kit et réessayez »",
            "„in Verwendung: Schließe die Programme des Kits und versuche es erneut“",
          ),
          a: tr(
            "Un programma del kit (un progetto avviato, una console, l'editor, un database) sta usando i file da sostituire. Fermalo, chiudi le console e riprova. Se non sai che cosa è aperto, la pagina «Rimuovi l'unità» lo elenca.",
            "A kit program (a running project, a console, the editor, a database) is using the files to be replaced. Stop it, close the consoles and try again. If you do not know what is open, the “Remove drive” page lists it.",
            "Un programme du kit (un projet lancé, une console, l'éditeur, une base de données) utilise les fichiers à remplacer. Arrêtez-le, fermez les consoles et réessayez. Si vous ne savez pas ce qui est ouvert, la page « Retirer le lecteur » le liste.",
            "Ein Programm des Kits (ein gestartetes Projekt, eine Konsole, der Editor, eine Datenbank) verwendet die zu ersetzenden Dateien. Stoppe es, schließe die Konsolen und versuche es erneut. Wenn du nicht weißt, was geöffnet ist, listet die Seite „Laufwerk entfernen“ es auf.",
          ),
          see: "eject",
        },
        {
          q: tr(
            "«C'è già un'installazione in corso.» oppure «Un'installazione è già in corso»",
            "“An installation is already running.” or “An installation is already in progress”",
            "« Une installation est déjà en cours. » ou « Une installation est déjà en cours »",
            "„Es läuft bereits eine Installation.“ oder „Es läuft bereits eine Installation“",
          ),
          a: tr(
            "Le installazioni e gli aggiornamenti chiesti dall'app non si rifiutano più: vanno in coda. Il messaggio resta quando togli una versione mentre si installa, o quando installa un'altra finestra di Atollo o il menu del kit: aspetta che finisca (la barra di stato mostra il passo) o annullala da Attività.",
            "Installations and updates asked from the app are no longer refused: they queue. The message remains when you remove a version while something installs, or when another Atollo window or the kit's menu is installing: wait for it to finish (the status bar shows the step) or cancel it from Activity.",
            "Les installations et mises à jour demandées depuis l'app ne sont plus refusées : elles attendent leur tour. Le message reste quand vous retirez une version pendant une installation, ou quand une autre fenêtre d'Atollo ou le menu du kit installe : attendez la fin (la barre d'état affiche l'étape) ou annulez depuis Activité.",
            "Installationen und Updates, die in der App angefordert werden, werden nicht mehr abgelehnt: Sie reihen sich ein. Die Meldung bleibt, wenn du während einer Installation eine Version entfernst oder wenn ein anderes Atollo-Fenster oder das Menü des Kits installiert: Warte, bis sie fertig ist (die Statusleiste zeigt den Schritt), oder brich sie in Aktivität ab.",
          ),
        },
        {
          q: tr(
            "Un componente è «da aggiornare» (triangolo giallo).",
            "A component is “to update” (yellow triangle).",
            "Un composant est « à mettre à jour » (triangle jaune).",
            "Eine Komponente ist „zu aktualisieren“ (gelbes Dreieck).",
          ),
          a: tr(
            "Su macOS e Linux, dopo uno spostamento del kit, alcuni componenti vanno reinstallati: il kit lo fa da solo all'avvio. Se resta così, premi «Aggiorna tutto» in fondo alla pagina Strumenti.",
            "On macOS and Linux, after the kit is moved, some components must be reinstalled: the kit does it by itself at start. If it stays that way, press “Update all” at the bottom of the Tools page.",
            "Sur macOS et Linux, après un déplacement du kit, certains composants doivent être réinstallés : le kit le fait tout seul au démarrage. Si cela reste ainsi, appuyez sur « Tout mettre à jour » en bas de la page Outils.",
            "Unter macOS und Linux müssen nach dem Verschieben des Kits einige Komponenten neu installiert werden: Das Kit erledigt das beim Start selbst. Wenn es so bleibt, klicke unten auf der Seite Werkzeuge auf „Alle aktualisieren“.",
          ),
          see: "move",
        },
        {
          q: tr(
            "Un programma si chiude da solo o manca una libreria, con il kit su una chiavetta.",
            "A program crashes or a library is missing, with the kit on a USB stick.",
            "Un programme plante ou une bibliothèque manque, avec le kit sur une clé USB.",
            "Ein Programm stürzt ab oder eine Bibliothek fehlt, mit dem Kit auf einem USB-Stick.",
          ),
          a: tr(
            "Un file del programma può essersi danneggiato sulla chiavetta. Premi «Verifica i file» nella pagina Strumenti (o esegui check --files): elenca le cartelle con file mancanti o cambiati dall'installazione, e «Reinstalla» le rimette alla stessa versione, senza toccare progetti e database.",
            "One of the program's files may be damaged on the stick. Press “Check the files” on the Tools page (or run check --files): it lists the folders with files missing or changed since the installation, and “Reinstall” puts them back at the same version, without touching projects and databases.",
            "Un fichier du programme peut s'être endommagé sur la clé. Appuyez sur « Vérifier les fichiers » dans la page Outils (ou lancez check --files) : la liste montre les dossiers avec des fichiers manquants ou modifiés depuis l'installation, et « Réinstaller » les remet à la même version, sans toucher aux projets ni aux bases de données.",
            "Eine Datei des Programms kann auf dem Stick beschädigt sein. Klicke auf der Seite Werkzeuge auf „Dateien prüfen“ (oder führe check --files aus): Die Liste zeigt die Ordner mit fehlenden oder seit der Installation geänderten Dateien, und „Neu installieren“ stellt sie in derselben Version wieder her, ohne Projekte und Datenbanken anzurühren.",
          ),
          see: "tools",
        },
      ],
    },
    {
      title: tr("Progetti e siti", "Projects and sites", "Projets et sites", "Projekte und Websites"),
      items: [
        {
          q: tr(
            "«Porta già assegnata a …», «La porta … è già del server/database/progetto …: scegline un'altra.», «Le porte da 7000 a 7099 sono dell'editor» o un programma che dice che la porta è occupata.",
            "“Port already assigned to …”, “Port … already belongs to server/database/project …: choose another one.”, “Ports 7000 to 7099 belong to the editor” or a program saying the port is taken.",
            "« Port déjà attribué à … », « Le port … appartient déjà au serveur/à la base de données/au projet … : choisissez-en un autre. », « Les ports 7000 à 7099 sont réservés à l'éditeur » ou un programme qui dit que le port est occupé.",
            "„Port bereits … zugewiesen“, „Port … gehört bereits zum Server/zur Datenbank/zum Projekt …: Wähle einen anderen.“, „Die Ports 7000 bis 7099 gehören dem Editor“ oder ein Programm, das meldet, dass der Port belegt ist.",
          ),
          a: tr(
            "Una porta è di una cosa sola: un'azione di un progetto, un pool della pagina Server Web, una versione di un database o l'editor (7000-7099). Il messaggio dice chi ce l'ha già: «Porta già assegnata a …» un'altra azione, «già del server …» un pool, «già del database …» una versione di MySQL o MongoDB, «già del progetto …» un'azione. Vale per la porta di un'azione, di un pool e di una versione del database. Se invece la porta è di un programma del PC, lo dice quel programma. Per un'azione svuota il campo «Porta» per tornare alla porta automatica; altrimenti scegline un'altra tra 1024 e 65535.",
            "A port belongs to one thing only: an action of a project, a pool of the Web Servers page, a version of a database or the editor (7000-7099). The message says who already has it: “Port already assigned to …” another action, “already belongs to server …” a pool, “already belongs to database …” a version of MySQL or MongoDB, “already belongs to project …” an action. It applies to the port of an action, of a pool and of a database version. If instead a program on the PC has the port, that program says so. For an action, empty the “Port” field to go back to the automatic port; otherwise choose another one between 1024 and 65535.",
            "Un port n'appartient qu'à une seule chose : une action d'un projet, un pool de la page Serveurs web, une version d'une base de données ou l'éditeur (7000-7099). Le message dit qui l'a déjà : « Port déjà attribué à … » une autre action, « appartient déjà au serveur … » un pool, « appartient déjà à la base de données … » une version de MySQL ou MongoDB, « appartient déjà au projet … » une action. Cela vaut pour le port d'une action, d'un pool et d'une version de la base de données. Si c'est un programme du PC qui a le port, c'est ce programme qui le dit. Pour une action, videz le champ « Port » pour revenir au port automatique ; sinon choisissez-en un autre entre 1024 et 65535.",
            "Ein Port gehört nur einem: einer Aktion eines Projekts, einem Pool der Seite Webserver, einer Version einer Datenbank oder dem Editor (7000-7099). Die Meldung sagt, wer ihn schon hat: „Port bereits … zugewiesen“ eine andere Aktion, „gehört bereits zum Server …“ ein Pool, „gehört bereits zur Datenbank …“ eine Version von MySQL oder MongoDB, „gehört bereits zum Projekt …“ eine Aktion. Das gilt für den Port einer Aktion, eines Pools und einer Datenbankversion. Hat dagegen ein Programm des PCs den Port, meldet es das selbst. Leere bei einer Aktion das Feld „Port“, um zum automatischen Port zurückzukehren; sonst wähle einen anderen zwischen 1024 und 65535.",
          ),
          see: "project-run",
        },
        {
          q: tr(
            "Un'azione si ferma subito con «errore (codice 1)».",
            "An action stops at once with “error (code 1)”.",
            "Une action s'arrête tout de suite avec « erreur (code 1) ».",
            "Eine Aktion stoppt sofort mit „Fehler (Code 1)“.",
          ),
          a: tr(
            "Leggi le ultime righe dell'output: di solito dicono che cosa manca. Casi frequenti: dipendenze non installate (esegui prima npm install o composer install), un componente mancante (sezione «Componenti» › «Installa ciò che manca»), una versione sbagliata (sezione «Versioni»).",
            "Read the last lines of the output: they usually say what is missing. Common cases: dependencies not installed (run npm install or composer install first), a missing component (“Components” section › “Install what is missing”), a wrong version (“Versions” section).",
            "Lisez les dernières lignes de la sortie : elles disent généralement ce qui manque. Cas fréquents : dépendances non installées (exécutez d'abord npm install ou composer install), un composant manquant (section « Composants » › « Installer ce qui manque »), une mauvaise version (section « Versions »).",
            "Lies die letzten Zeilen der Ausgabe: Meist sagen sie, was fehlt. Häufige Fälle: nicht installierte Abhängigkeiten (führe zuerst npm install oder composer install aus), eine fehlende Komponente (Abschnitt „Komponenten“ › „Fehlendes installieren“), eine falsche Version (Abschnitt „Versionen“).",
          ),
          see: "project-versions",
        },
        {
          q: tr(
            "«Azione non trovata: premi Aggiorna»",
            "“Action not found: press Refresh”",
            "« Action introuvable : cliquez sur Actualiser »",
            "„Aktion nicht gefunden: Klicke auf Aktualisieren“",
          ),
          a: tr(
            "Il progetto è cambiato (per esempio panel.json). Premi «Aggiorna» nella pagina del progetto.",
            "The project has changed (for example panel.json). Press “Refresh” on the project page.",
            "Le projet a changé (par exemple panel.json). Appuyez sur « Actualiser » dans la page du projet.",
            "Das Projekt hat sich geändert (zum Beispiel panel.json). Klicke auf der Seite des Projekts auf „Aktualisieren“.",
          ),
        },
        {
          q: tr(
            "«panel.json non è un JSON valido: ignorato.»",
            "“panel.json is not valid JSON: ignored.”",
            "« panel.json n'est pas un JSON valide : ignoré. »",
            "„panel.json ist kein gültiges JSON: ignoriert.“",
          ),
          a: tr(
            "C'è un errore di sintassi nel file (una virgola in più, una virgoletta mancante). Correggilo nell'editor e premi «Aggiorna».",
            "There is a syntax error in the file (an extra comma, a missing quote). Fix it in the editor and press “Refresh”.",
            "Il y a une erreur de syntaxe dans le fichier (une virgule en trop, un guillemet manquant). Corrigez-la dans l'éditeur et appuyez sur « Actualiser ».",
            "In der Datei ist ein Syntaxfehler (ein Komma zu viel, ein fehlendes Anführungszeichen). Korrigiere ihn im Editor und klicke auf „Aktualisieren“.",
          ),
          see: "project-run",
        },
        {
          q: tr(
            "«Il server condiviso … non è partito: guarda la pagina Server Web.»",
            "“The shared server … did not start: see the Web Servers page.”",
            "« Le serveur partagé … n'a pas démarré : voyez la page Serveurs web. »",
            "„Der gemeinsame Server … ist nicht gestartet: siehe die Seite Webserver.“",
          ),
          a: tr(
            "Apri la pagina Server Web, scegli il pool e guarda l'errore. Spesso la porta è occupata: cambia porta al profilo, oppure scegli «Server tutto suo» nella sezione «Sito web» del progetto.",
            "Open the Web Servers page, choose the pool and look at the error. Often the port is taken: change the profile's port, or choose “Its own server” in the project's “Website” section.",
            "Ouvrez la page Serveurs web, choisissez le pool et regardez l'erreur. Souvent le port est occupé : changez le port du profil, ou choisissez « Son propre serveur » dans la section « Site web » du projet.",
            "Öffne die Seite Webserver, wähle den Pool und sieh dir den Fehler an. Oft ist der Port belegt: Ändere den Port des Profils, oder wähle „Eigener Server“ im Abschnitt „Website“ des Projekts.",
          ),
          see: "websites",
        },
        {
          q: tr(
            "Il sito si apre ma mostra file vecchi o una pagina bianca.",
            "The site opens but shows old files or a blank page.",
            "Le site s'ouvre mais affiche d'anciens fichiers ou une page blanche.",
            "Die Website öffnet sich, zeigt aber alte Dateien oder eine leere Seite.",
          ),
          a: tr(
            "Controlla la cartella in «Servi come sito web la cartella» (per molti framework PHP è public) e ricarica la pagina del browser ignorando la cache (Ctrl+F5).",
            "Check the folder in “Folder to serve as a website” (for many PHP frameworks it is public) and reload the browser page bypassing the cache (Ctrl+F5).",
            "Vérifiez le dossier dans « Dossier à servir comme site web » (pour beaucoup de frameworks PHP, c'est public) et rechargez la page du navigateur en ignorant le cache (Ctrl+F5).",
            "Prüfe den Ordner unter „Als Website bereitzustellender Ordner“ (bei vielen PHP-Frameworks ist es public) und lade die Seite im Browser ohne Cache neu (Strg+F5).",
          ),
        },
        {
          q: tr(
            "L'editor non si accorge dei file cambiati.",
            "The editor does not notice changed files.",
            "L'éditeur ne s'aperçoit pas des fichiers modifiés.",
            "Der Editor bemerkt geänderte Dateien nicht.",
          ),
          a: tr(
            "Su Windows succede se una cartella del percorso del kit inizia con una cifra, con x o con u: l'app lo segnala con un avviso. Sposta il kit in un percorso diverso.",
            "On Windows this happens if a folder in the kit's path starts with a digit, with x or with u: the app flags it with a warning. Move the kit to a different path.",
            "Sur Windows, cela arrive si un dossier du chemin du kit commence par un chiffre, par x ou par u : l'application le signale par un avertissement. Déplacez le kit vers un autre chemin.",
            "Unter Windows passiert das, wenn ein Ordner im Pfad des Kits mit einer Ziffer, mit x oder mit u beginnt: Die App weist mit einem Hinweis darauf hin. Verschiebe das Kit in einen anderen Pfad.",
          ),
          see: "first-run",
        },
      ],
    },
    {
      title: tr("Database", "Databases", "Bases de données", "Datenbanken"),
      items: [
        {
          q: tr(
            "«La porta … è già usata da un altro programma: cambiala qui sotto.»",
            "“Port … is already used by another program: change it below.”",
            "« Le port … est déjà utilisé par un autre programme : changez-le ci-dessous. »",
            "„Port … wird bereits von einem anderen Programm verwendet: Ändere ihn unten.“",
          ),
          a: tr(
            "Sul PC gira un altro MySQL o MongoDB (installato a parte) sulla stessa porta. Cambia la «Porta» nella pagina del server (per esempio 3307) e avvialo di nuovo; poi riscrivi la connessione nel .env dei progetti con «Scrivi in .env».",
            "Another MySQL or MongoDB (installed separately) runs on the PC on the same port. Change the “Port” on the server page (for example 3307) and start it again; then rewrite the connection in the projects' .env with “Write to .env”.",
            "Un autre MySQL ou MongoDB (installé à part) tourne sur le PC sur le même port. Changez le « Port » dans la page du serveur (par exemple 3307) et démarrez-le de nouveau ; puis réécrivez la connexion dans le .env des projets avec « Écrire dans .env ».",
            "Auf dem PC läuft ein anderes MySQL oder MongoDB (separat installiert) auf demselben Port. Ändere den „Port“ auf der Seite des Servers (zum Beispiel 3307) und starte ihn erneut; schreib dann die Verbindung mit „In .env schreiben“ neu in die .env der Projekte.",
          ),
          see: "databases",
        },
        {
          q: tr(
            "Il server resta in «Errore» dopo l'avvio.",
            "The server stays in “Error” after starting.",
            "Le serveur reste en « Erreur » après le démarrage.",
            "Der Server bleibt nach dem Start auf „Fehler“.",
          ),
          a: tr(
            "La pagina del server mostra l'ultimo errore e il percorso del log (data\\app\\logs). Su Linux MySQL può chiedere le librerie libaio e libnuma: il messaggio dice il comando per installarle.",
            "The server page shows the last error and the path of the log (data\\app\\logs). On Linux MySQL may ask for the libaio and libnuma libraries: the message gives the command to install them.",
            "La page du serveur affiche la dernière erreur et le chemin du journal (data\\app\\logs). Sur Linux, MySQL peut demander les bibliothèques libaio et libnuma : le message indique la commande pour les installer.",
            "Die Seite des Servers zeigt den letzten Fehler und den Pfad des Logs (data\\app\\logs). Unter Linux kann MySQL die Bibliotheken libaio und libnuma verlangen: Die Meldung nennt den Befehl, um sie zu installieren.",
          ),
        },
        {
          q: tr(
            "Il progetto non si collega al database.",
            "The project does not connect to the database.",
            "Le projet ne se connecte pas à la base de données.",
            "Das Projekt verbindet sich nicht mit der Datenbank.",
          ),
          a: tr(
            "Controlla che il server sia «Avviato», poi usa «Scrivi in .env» nella sezione «Database» del progetto: il .env avrà utente, password e porta giusti. Se il progetto usa un'altra versione del server, la porta è diversa (3307…).",
            "Check that the server is “Running”, then use “Write to .env” in the project's “Databases” section: the .env will have the right user, password and port. If the project uses another version of the server, the port is different (3307…).",
            "Vérifiez que le serveur est « Démarré », puis utilisez « Écrire dans .env » dans la section « Bases de données » du projet : le .env aura l'utilisateur, le mot de passe et le port corrects. Si le projet utilise une autre version du serveur, le port est différent (3307…).",
            "Prüfe, ob der Server „Läuft“, und verwende dann „In .env schreiben“ im Abschnitt „Datenbanken“ des Projekts: Die .env enthält dann den richtigen Benutzer, das Passwort und den Port. Wenn das Projekt eine andere Version des Servers verwendet, ist der Port anders (3307…).",
          ),
          see: "project-db",
        },
      ],
    },
    {
      title: tr("Agenti, Git e GitHub", "Agents, Git and GitHub", "Agents, Git et GitHub", "Agenten, Git und GitHub"),
      items: [
        {
          q: tr(
            "«accesso scaduto» negli agenti.",
            "“sign-in expired” in the agents.",
            "« connexion expirée » dans les agents.",
            "„Anmeldung abgelaufen“ bei den Agenten.",
          ),
          a: tr(
            "Apri l'agente in un progetto: rinnova l'accesso, o te lo fa rifare.",
            "Open the agent in a project: it renews the sign-in, or has you sign in again.",
            "Ouvrez l'agent dans un projet : il renouvelle la connexion, ou vous la fait refaire.",
            "Öffne den Agenten in einem Projekt: Er erneuert die Anmeldung oder fordert dich auf, dich neu anzumelden.",
          ),
          see: "agents",
        },
        {
          q: tr(
            "Git dice «Please tell me who you are» al primo commit.",
            "Git says “Please tell me who you are” at the first commit.",
            "Git dit « Please tell me who you are » au premier commit.",
            "Git meldet „Please tell me who you are“ beim ersten Commit.",
          ),
          a: tr(
            "Scrivi nome ed e-mail in Impostazioni › «Account GitHub» › «Chi firma i commit», oppure accedi a GitHub: se sono vuoti, li riempie l'account. In un terminale del kit funziona anche git config --global user.name e user.email.",
            "Type your name and email in Settings › “GitHub account” › “Who signs the commits”, or sign in to GitHub: if they are empty, the account fills them. In a kit terminal git config --global user.name and user.email work too.",
            "Saisissez votre nom et votre e-mail dans Paramètres › « Compte GitHub » › « Qui signe les commits », ou connectez-vous à GitHub : s'ils sont vides, le compte les remplit. Dans un terminal du kit, git config --global user.name et user.email fonctionnent aussi.",
            "Gib Name und E-Mail unter Einstellungen › „GitHub-Konto“ › „Wer die Commits signiert“ ein oder melde dich bei GitHub an: Wenn sie leer sind, füllt das Konto sie aus. In einem Terminal des Kits funktionieren auch git config --global user.name und user.email.",
          ),
          see: "github",
        },
        {
          q: tr(
            "«GitHub ha rifiutato l'accesso: accedi di nuovo.»",
            "“GitHub refused the sign-in: sign in again.”",
            "« GitHub a refusé la connexion : connectez-vous de nouveau. »",
            "„GitHub hat die Anmeldung abgelehnt: Melde dich erneut an.“",
          ),
          a: tr(
            "Il token non è più valido (revocato o scaduto). In Impostazioni › «Account GitHub» premi «Esci» e poi «Accedi a GitHub».",
            "The token is no longer valid (revoked or expired). In Settings › “GitHub account” press “Sign out” and then “Sign in to GitHub”.",
            "Le jeton n'est plus valide (révoqué ou expiré). Dans Paramètres › « Compte GitHub », appuyez sur « Se déconnecter » puis sur « Se connecter à GitHub ».",
            "Das Token ist nicht mehr gültig (widerrufen oder abgelaufen). Klicke unter Einstellungen › „GitHub-Konto“ auf „Abmelden“ und dann auf „Bei GitHub anmelden“.",
          ),
          see: "github",
        },
      ],
    },
    {
      title: tr("Avvio dell'app", "App start", "Démarrage de l'application", "Start der App"),
      items: [
        {
          q: tr(
            "«Il disco del kit è in sola lettura: impossibile scrivere nella cartella …»",
            "“The kit's disk is read-only: cannot write to the folder …”",
            "« Le disque du kit est en lecture seule : impossible d'écrire dans le dossier … »",
            "„Das Laufwerk des Kits ist schreibgeschützt: In den Ordner … kann nicht geschrieben werden.“",
          ),
          a: tr(
            "La chiavetta è protetta da scrittura (a volte c'è un interruttore) o montata in sola lettura. Il kit deve poter scrivere nella sua cartella.",
            "The USB stick is write-protected (sometimes there is a switch) or mounted read-only. The kit must be able to write in its folder.",
            "La clé USB est protégée en écriture (il y a parfois un interrupteur) ou montée en lecture seule. Le kit doit pouvoir écrire dans son dossier.",
            "Der USB-Stick ist schreibgeschützt (manchmal gibt es einen Schalter) oder schreibgeschützt eingehängt. Das Kit muss in seinen Ordner schreiben können.",
          ),
        },
        {
          q: tr(
            "macOS: «macOS ha aperto Atollo da una copia temporanea in sola lettura (App Translocation)».",
            "macOS: “macOS opened Atollo from a temporary read-only copy (App Translocation)”.",
            "macOS : « macOS a ouvert Atollo depuis une copie temporaire en lecture seule (App Translocation) ».",
            "macOS: „macOS hat Atollo aus einer temporären, schreibgeschützten Kopie geöffnet (App Translocation)“.",
          ),
          a: tr(
            "Succede con le app scaricate e non autenticate. Il messaggio mostra un comando xattr: eseguilo nel Terminale e riapri Atollo.",
            "It happens with downloaded apps that are not notarized. The message shows an xattr command: run it in Terminal and reopen Atollo.",
            "Cela arrive avec les applications téléchargées et non notarisées. Le message affiche une commande xattr : exécutez-la dans le Terminal et rouvrez Atollo.",
            "Das passiert bei heruntergeladenen, nicht beglaubigten Apps. Die Meldung zeigt einen xattr-Befehl: Führe ihn im Terminal aus und öffne Atollo erneut.",
          ),
          see: "move",
        },
        {
          q: tr(
            "Linux: il disco è montato con noexec, o manca WebKitGTK.",
            "Linux: the disk is mounted with noexec, or WebKitGTK is missing.",
            "Linux : le disque est monté avec noexec, ou WebKitGTK manque.",
            "Linux: Das Laufwerk ist mit noexec eingehängt, oder WebKitGTK fehlt.",
          ),
          a: tr(
            "Il messaggio spiega che cosa fare: rimonta il disco con il permesso di esecuzione, o installa i pacchetti indicati.",
            "The message explains what to do: remount the disk with execute permission, or install the packages it names.",
            "Le message explique quoi faire : remontez le disque avec la permission d'exécution, ou installez les paquets indiqués.",
            "Die Meldung erklärt, was zu tun ist: Häng das Laufwerk mit Ausführungsrecht neu ein, oder installiere die angegebenen Pakete.",
          ),
        },
        {
          q: tr(
            "Windows: manca WebView2.",
            "Windows: WebView2 is missing.",
            "Windows : WebView2 manque.",
            "Windows: WebView2 fehlt.",
          ),
          a: tr(
            "WebView2 è il componente di Windows che disegna la finestra; c'è già in Windows 11 e quasi sempre in Windows 10. Se manca, Atollo mostra il collegamento per installarlo.",
            "WebView2 is the Windows component that draws the window; it is already in Windows 11 and almost always in Windows 10. If it is missing, Atollo shows the link to install it.",
            "WebView2 est le composant de Windows qui dessine la fenêtre ; il est déjà présent dans Windows 11 et presque toujours dans Windows 10. S'il manque, Atollo affiche le lien pour l'installer.",
            "WebView2 ist die Windows-Komponente, die das Fenster zeichnet; sie ist in Windows 11 bereits vorhanden und fast immer auch in Windows 10. Fehlt sie, zeigt Atollo den Link zum Installieren.",
          ),
        },
        {
          q: tr(
            "Avviso: «Su questo PC sono aperti programmi di un'altra copia del kit».",
            "Warning: “Programs from another copy of the kit are open on this PC”.",
            "Avertissement : « Des programmes d'une autre copie du kit sont ouverts sur ce PC ».",
            "Hinweis: „Auf diesem PC sind Programme einer anderen Kopie des Kits geöffnet“.",
          ),
          a: tr(
            "Solo un'informazione: un altro kit è aperto. Le due copie non condividono niente; Docker Sandboxes però ne serve uno alla volta per utente, quindi il secondo kit lavora «Senza VM» finché il primo tiene acceso il suo.",
            "Just information: another kit is open. The two copies share nothing; Docker Sandboxes, though, serves one at a time per user, so the second kit works with “No VM” while the first keeps its own running.",
            "Simple information : un autre kit est ouvert. Les deux copies ne partagent rien ; Docker Sandboxes, lui, n'en sert qu'un à la fois par utilisateur, donc le second kit travaille « Sans VM » tant que le premier garde le sien actif.",
            "Nur eine Information: Ein anderes Kit ist geöffnet. Die beiden Kopien teilen nichts; Docker Sandboxes bedient aber nur eines zur Zeit pro Benutzer, daher arbeitet das zweite Kit „Ohne VM“, solange das erste seines laufen lässt.",
          ),
          see: "vm",
        },
      ],
    },
    {
      title: tr(
        "La VM (Docker Sandboxes)",
        "The VM (Docker Sandboxes)",
        "La VM (Docker Sandboxes)",
        "Die VM (Docker Sandboxes)",
      ),
      items: [
        {
          q: tr(
            "«Senza VM» nella barra di stato.",
            "“No VM” in the status bar.",
            "« Sans VM » dans la barre d'état.",
            "„Ohne VM“ in der Statusleiste.",
          ),
          a: tr(
            "La VM dei progetti non può partire. Fai clic: la finestra elenca i motivi con che cosa fare («Lancia», «Copia», «Installa nel kit»); poi premi «Ricontrolla». Al primo avvio di solito manca solo l'accesso a Docker (sbx login).",
            "The projects' VM cannot start. Click it: the window lists the reasons with what to do (“Run”, “Copy”, “Install in the kit”); then press “Check again”. At the first start usually only the Docker sign-in (sbx login) is missing.",
            "La VM des projets ne peut pas démarrer. Cliquez dessus : la fenêtre liste les motifs avec quoi faire (« Lancer », « Copier », « Installer dans le kit ») ; puis appuyez sur « Vérifier à nouveau ». Au premier lancement, il ne manque en général que la connexion à Docker (sbx login).",
            "Die VM der Projekte kann nicht starten. Klicke darauf: Das Fenster listet die Gründe mit dem, was zu tun ist („Starten“, „Kopieren“, „Im Kit installieren“); drücke dann „Erneut prüfen“. Beim ersten Start fehlt meist nur die Anmeldung bei Docker (sbx login).",
          ),
          see: "vm",
        },
        {
          q: tr(
            "Un agente non parte.",
            "An agent does not start.",
            "Un agent ne démarre pas.",
            "Ein Agent startet nicht.",
          ),
          a: tr(
            "Gli agenti girano solo nella VM: Claude Code e Codex nelle loro sandbox, Pi e Antigravity nella VM del progetto. Senza VM nessun agente parte, mai sul PC al suo posto: i pulsanti sono disattivati e il loro suggerimento dice perché; risolvi prima i motivi di «Senza VM». Se manca la build per Linux di Pi o di Antigravity, installala dalla pagina Strumenti.",
            "The agents run only in the VM: Claude Code and Codex in their sandboxes, Pi and Antigravity in the project's VM. Without the VM no agent starts, never on the PC in its place: the buttons are disabled and their tooltip says why; fix the “No VM” reasons first. If the Linux build of Pi or Antigravity is missing, install it from the Tools page.",
            "Les agents ne tournent que dans la VM : Claude Code et Codex dans leurs sandbox, Pi et Antigravity dans la VM du projet. Sans VM, aucun agent ne démarre, jamais sur le PC à sa place : les boutons sont désactivés et leur infobulle dit pourquoi ; résolvez d'abord les motifs de « Sans VM ». S'il manque le build Linux de Pi ou d'Antigravity, installez-le depuis la page Outils.",
            "Die Agenten laufen nur in der VM: Claude Code und Codex in ihren Sandboxes, Pi und Antigravity in der VM des Projekts. Ohne VM startet kein Agent, nie stattdessen auf dem PC: Die Schaltflächen sind deaktiviert, und ihr Tooltip sagt warum; behebe zuerst die Gründe für „Ohne VM“. Fehlt der Linux-Build von Pi oder Antigravity, installiere ihn über die Seite Werkzeuge.",
          ),
          see: "vm",
        },
        {
          q: tr(
            "Nel terminale del kit claude, codex, pi o agy non partono.",
            "In the kit's terminal claude, codex, pi or agy do not start.",
            "Dans le terminal du kit, claude, codex, pi ou agy ne démarrent pas.",
            "Im Terminal des Kits starten claude, codex, pi oder agy nicht.",
          ),
          a: tr(
            "Nei terminali del PC questi comandi inoltrano l'agente alla VM del progetto: funzionano solo nella cartella di un progetto e con la VM attiva. Altrimenti dicono perché ed escono, perché un agente non gira mai sul PC. Apri il terminale nella cartella del progetto, oppure apri l'agente dalla pagina del progetto.",
            "In the PC's terminals these commands forward the agent to the project's VM: they work only in a project's folder and with the VM on. Otherwise they say why and exit, because an agent never runs on the PC. Open the terminal in the project's folder, or open the agent from the project's page.",
            "Dans les terminaux du PC, ces commandes transmettent l'agent à la VM du projet : elles ne fonctionnent que dans le dossier d'un projet et avec la VM active. Sinon, elles disent pourquoi et s'arrêtent, car un agent ne tourne jamais sur le PC. Ouvrez le terminal dans le dossier du projet, ou ouvrez l'agent depuis la page du projet.",
            "In den Terminals des PCs leiten diese Befehle den Agenten an die VM des Projekts weiter: Sie funktionieren nur im Ordner eines Projekts und mit aktiver VM. Sonst sagen sie warum und beenden sich, denn ein Agent läuft nie auf dem PC. Öffne das Terminal im Ordner des Projekts oder öffne den Agenten von der Seite des Projekts.",
          ),
          see: "terminals",
        },
        {
          q: tr(
            "Nell'editor manca il profilo di un agente.",
            "A profile of an agent is missing in the editor.",
            "Le profil d'un agent manque dans l'éditeur.",
            "Im Editor fehlt das Profil eines Agenten.",
          ),
          a: tr(
            "I terminali dell'editor girano sempre sul PC; i profili degli agenti ci sono solo con la VM, e l'editor li legge al suo avvio: quando la barra di stato dice «VM», premi «Riavvia l'editor». Anche i task girano sul PC, nell'ambiente sigillato, con o senza la VM.",
            "The editor's terminals always run on the PC; the agents' profiles are there only with the VM, and the editor reads them when it starts: when the status bar says “VM”, press “Restart the editor”. The tasks run on the PC too, in the sealed environment, with or without the VM.",
            "Les terminaux de l'éditeur tournent toujours sur le PC ; les profils des agents n'y sont qu'avec la VM, et l'éditeur les lit à son démarrage : quand la barre d'état indique « VM », appuyez sur « Redémarrer l'éditeur ». Les tâches tournent elles aussi sur le PC, dans l'environnement scellé, avec ou sans la VM.",
            "Die Terminals des Editors laufen immer auf dem PC; die Profile der Agenten gibt es nur mit der VM, und der Editor liest sie bei seinem Start: Wenn die Statusleiste „VM“ zeigt, drück „Editor neu starten“. Auch die Tasks laufen auf dem PC, in der versiegelten Umgebung, mit oder ohne VM.",
          ),
          see: "vm",
        },
        {
          q: tr(
            "«La sandbox … lavora in un'altra cartella: rimuovila in «Sandbox delle VM» …»",
            "“The sandbox … works in another folder: remove it in «VM sandboxes» …”",
            "« La sandbox … travaille dans un autre dossier : supprime-la dans «Sandbox des VM» … »",
            "„Die Sandbox … arbeitet in einem anderen Ordner: Entferne sie unter «Sandboxes der VMs» …“",
          ),
          a: tr(
            "Quando il kit cambia cartella Atollo rifà da solo la sandbox di un progetto; il messaggio resta per i casi che non sa risolvere. Apri la finestra della VM dalla barra di stato, premi «Rimuovi» e «Rimuovi davvero» sulla sandbox del progetto (si cancella il suo disco, non i file del progetto) e riapri: Atollo ne crea una nuova.",
            "When the kit changes folder Atollo makes a project's sandbox again by itself; the message remains for the cases it cannot solve. Open the VM's window from the status bar, press “Remove” and “Really remove” on the project's sandbox (its disk is deleted, not the project's files) and open again: Atollo makes a new one.",
            "Quand le kit change de dossier, Atollo refait tout seul la sandbox d'un projet ; le message reste pour les cas qu'il ne sait pas résoudre. Ouvrez la fenêtre de la VM depuis la barre d'état, appuyez sur « Supprimer » puis « Supprimer vraiment » sur la sandbox du projet (son disque est supprimé, pas les fichiers du projet) et rouvrez : Atollo en crée une nouvelle.",
            "Wechselt das Kit den Ordner, legt Atollo die Sandbox eines Projekts von selbst neu an; die Meldung bleibt für die Fälle, die es nicht lösen kann. Öffne das Fenster der VM über die Statusleiste, drücke „Entfernen“ und „Wirklich entfernen“ bei der Sandbox des Projekts (ihre Festplatte wird gelöscht, nicht die Dateien des Projekts) und öffne erneut: Atollo legt eine neue an.",
          ),
          see: "vm",
        },
        {
          q: tr(
            "«Docker Sandboxes deve ripartire per un'impostazione di sicurezza: …»",
            "“Docker Sandboxes must restart for a security setting: …”",
            "« Docker Sandboxes doit redémarrer pour un réglage de sécurité : … »",
            "„Docker Sandboxes muss für eine Sicherheitseinstellung neu starten: …“",
          ),
          a: tr(
            "Atollo lo fa ripartire solo quando nessuna sandbox è accesa. Chiudi gli agenti nella VM, poi «Ricontrolla» nella finestra della VM e riprova.",
            "Atollo restarts it only when no sandbox is running. Close the agents in the VM, then “Check again” in the VM's window and try again.",
            "Atollo ne le redémarre que lorsqu'aucune sandbox n'est allumée. Fermez les agents dans la VM, puis « Vérifier à nouveau » dans la fenêtre de la VM et réessayez.",
            "Atollo startet es nur neu, wenn keine Sandbox läuft. Schließe die Agenten in der VM, dann „Erneut prüfen“ im Fenster der VM und versuche es erneut.",
          ),
        },
      ],
    },
  ];
}

function glossary(): [string, string][] {
  return [
    [
      tr("Agente IA", "AI agent", "Agent IA", "KI-Agent"),
      tr(
        "Un assistente a riga di comando che lavora nella cartella del progetto: legge, modifica, esegue comandi. Nel kit: Claude Code, Codex, Antigravity, Pi.",
        "A command-line assistant that works in the project folder: it reads, edits and runs commands. In the kit: Claude Code, Codex, Antigravity, Pi.",
        "Un assistant en ligne de commande qui travaille dans le dossier du projet : il lit, modifie, exécute des commandes. Dans le kit : Claude Code, Codex, Antigravity, Pi.",
        "Ein Befehlszeilen-Assistent, der im Projektordner arbeitet: Er liest, ändert und führt Befehle aus. Im Kit: Claude Code, Codex, Antigravity, Pi.",
      ),
    ],
    [
      tr("Docker Sandboxes (sbx)", "Docker Sandboxes (sbx)", "Docker Sandboxes (sbx)", "Docker Sandboxes (sbx)"),
      tr(
        "Il programma di Docker che crea le microVM dei progetti. È un componente obbligatorio del kit, in data; Atollo usa solo quello del kit. Serve un account Docker gratuito (sbx login); Docker Desktop no.",
        "Docker's program that makes the projects' microVMs. It is a required component of the kit, in data; Atollo uses only the kit's. A free Docker account is needed (sbx login); Docker Desktop is not.",
        "Le programme de Docker qui crée les microVM des projets. C'est un composant obligatoire du kit, dans data ; Atollo n'utilise que celui du kit. Il faut un compte Docker gratuit (sbx login) ; Docker Desktop, non.",
        "Das Programm von Docker, das die microVMs der Projekte anlegt. Es ist eine Pflichtkomponente des Kits, in data; Atollo verwendet nur das des Kits. Nötig ist ein kostenloses Docker-Konto (sbx login), Docker Desktop nicht.",
      ),
    ],
    [
      tr("MicroVM (VM)", "MicroVM (VM)", "MicroVM (VM)", "MicroVM (VM)"),
      tr(
        "Un piccolo computer virtuale, con un Linux suo, che parte in pochi secondi. In Atollo ci girano gli agenti di un progetto: vede solo la cartella del progetto e gli strumenti del kit per Linux.",
        "A small virtual computer, with its own Linux, that starts in a few seconds. In Atollo a project's agents run in it: it sees only the project's folder and the kit's tools for Linux.",
        "Un petit ordinateur virtuel, avec son propre Linux, qui démarre en quelques secondes. Dans Atollo, les agents d'un projet y tournent : elle ne voit que le dossier du projet et les outils du kit pour Linux.",
        "Ein kleiner virtueller Computer mit eigenem Linux, der in wenigen Sekunden startet. In Atollo laufen darin die Agenten eines Projekts: Sie sieht nur den Ordner des Projekts und die Werkzeuge des Kits für Linux.",
      ),
    ],
    [
      tr("Sandbox", "Sandbox", "Sandbox", "Sandbox"),
      tr(
        "La microVM di Docker Sandboxes con il suo disco: una per progetto (dove gira anche Pi), una per Claude Code o Codex in ogni progetto. Si vedono e si rimuovono nella finestra della VM, sezione «Sandbox delle VM».",
        "The Docker Sandboxes microVM with its disk: one per project (where Pi runs too), one for Claude Code or Codex in each project. They are listed and removed in the VM's window, section “VM sandboxes”.",
        "La microVM de Docker Sandboxes avec son disque : une par projet (où tourne aussi Pi), une pour Claude Code ou Codex dans chaque projet. Elles se voient et se suppriment dans la fenêtre de la VM, section « Sandbox des VM ».",
        "Die microVM von Docker Sandboxes mit ihrer Festplatte: eine pro Projekt (in der auch Pi läuft), eine für Claude Code oder Codex in jedem Projekt. Sie werden im Fenster der VM angezeigt und entfernt, Abschnitt „Sandboxes der VMs“.",
      ),
    ],
    [
      tr("GGUF", "GGUF", "GGUF", "GGUF"),
      tr(
        "Il formato dei file dei modelli locali che usa llama.cpp: un file (a volte in parti) con i pesi del modello compressi secondo una «quantizzazione», per esempio Q4_K_M.",
        "The file format of the local models llama.cpp uses: a file (sometimes in parts) with the model's weights compressed according to a “quantization”, for example Q4_K_M.",
        "Le format des fichiers des modèles locaux qu'utilise llama.cpp : un fichier (parfois en parties) avec les poids du modèle compressés selon une « quantification », par exemple Q4_K_M.",
        "Das Dateiformat der lokalen Modelle, die llama.cpp verwendet: eine Datei (manchmal in Teilen) mit den Gewichten des Modells, komprimiert nach einer „Quantisierung“, zum Beispiel Q4_K_M.",
      ),
    ],
    [
      tr("Modelli locali", "Local models", "Modèles locaux", "Lokale Modelle"),
      tr(
        "Modelli linguistici che girano sul PC, senza rete e senza account, con il server llama.cpp del kit. Si scaricano dalla pagina AI, riquadro «Modelli locali», e li usano Pi e la chat nel browser.",
        "Language models that run on the PC, with no network and no account, with the kit's llama.cpp server. You download them from the AI page, “Local models” pane; Pi and the chat in the browser use them.",
        "Des modèles de langage qui tournent sur le PC, sans réseau et sans compte, avec le serveur llama.cpp du kit. Ils se téléchargent depuis la page IA, volet « Modèles locaux », et Pi et le chat dans le navigateur les utilisent.",
        "Sprachmodelle, die auf dem PC laufen, ohne Netzwerk und ohne Konto, mit dem llama.cpp-Server des Kits. Du lädst sie auf der Seite KI im Bereich „Lokale Modelle“ herunter; Pi und der Chat im Browser verwenden sie.",
      ),
    ],
    [
      tr("Pi", "Pi", "Pi", "Pi"),
      tr(
        "L'agente da terminale di Earendil (Mario Zechner), il quarto del kit: lavora con i modelli locali, senza account, o con un provider in cloud scelto con /login.",
        "The terminal agent by Earendil (Mario Zechner), the kit's fourth: it works with the local models, with no account, or with a cloud provider chosen with /login.",
        "L'agent de terminal d'Earendil (Mario Zechner), le quatrième du kit : il fonctionne avec les modèles locaux, sans compte, ou avec un fournisseur dans le cloud choisi avec /login.",
        "Der Terminal-Agent von Earendil (Mario Zechner), der vierte des Kits: Er arbeitet mit den lokalen Modellen, ohne Konto, oder mit einem mit /login gewählten Cloud-Anbieter.",
      ),
    ],
    [
      tr("Aggiunta", "Addition", "Ajout", "Ergänzung"),
      tr(
        "Nella procedura «Da una base», un pezzo in più da aggiungere alla base (Tailwind, React, un database, le prove).",
        "In the “From a base” wizard, an extra piece to add to the base (Tailwind, React, a database, tests).",
        "Dans l'assistant « À partir d'une base », un élément en plus à ajouter à la base (Tailwind, React, une base de données, les tests).",
        "Im Assistenten „Aus einer Basis“ ein zusätzlicher Baustein, der der Basis hinzugefügt wird (Tailwind, React, eine Datenbank, Tests).",
      ),
    ],
    [
      tr("Alias di percorso", "Path alias", "Alias de chemin", "Pfad-Alias"),
      tr(
        "Un indirizzo del sito che serve una cartella diversa, per esempio /media → storage/uploads.",
        "An address of the site that serves a different folder, for example /media → storage/uploads.",
        "Une adresse du site qui sert un dossier différent, par exemple /media → storage/uploads.",
        "Eine Adresse der Website, die einen anderen Ordner bereitstellt, zum Beispiel /media → storage/uploads.",
      ),
    ],
    [
      tr("Azione", "Action", "Action", "Aktion"),
      tr(
        "Un comando del progetto con un nome, che si avvia e si ferma dalla sezione «Avvio».",
        "A named project command, started and stopped from the “Run” section.",
        "Une commande du projet avec un nom, qu'on lance et arrête depuis la section « Lancement ».",
        "Ein Befehl des Projekts mit einem Namen, der im Abschnitt „Start“ gestartet und gestoppt wird.",
      ),
    ],
    [
      tr("Base", "Base", "Base", "Basis"),
      tr(
        "Il punto di partenza di un progetto creato con un framework o un CMS: Laravel, Vite, Django, WordPress…",
        "The starting point of a project created with a framework or a CMS: Laravel, Vite, Django, WordPress…",
        "Le point de départ d'un projet créé avec un framework ou un CMS : Laravel, Vite, Django, WordPress…",
        "Der Ausgangspunkt eines mit einem Framework oder CMS erstellten Projekts: Laravel, Vite, Django, WordPress…",
      ),
    ],
    [
      tr("Cartella del kit", "Kit folder", "Dossier du kit", "Kit-Ordner"),
      tr(
        "La cartella che contiene l'eseguibile di Atollo, projects e data.",
        "The folder that contains the Atollo executable, projects and data.",
        "Le dossier qui contient l'exécutable d'Atollo, projects et data.",
        "Der Ordner, der die ausführbare Datei von Atollo, projects und data enthält.",
      ),
    ],
    [
      tr("Chiave API", "API key", "Clé API", "API-Schlüssel"),
      tr(
        "Un codice segreto che permette di usare un servizio (per esempio un modello IA) pagando a consumo, senza abbonamento.",
        "A secret code that lets you use a service (for example an AI model) paying as you go, without a subscription.",
        "Un code secret qui permet d'utiliser un service (par exemple un modèle d'IA) en payant à l'usage, sans abonnement.",
        "Ein geheimer Code, mit dem du einen Dienst (zum Beispiel ein KI-Modell) nach Verbrauch bezahlt nutzen kannst, ohne Abonnement.",
      ),
    ],
    [
      tr("Clonare", "Clone", "Cloner", "Klonen"),
      tr(
        "Copiare un repository Git sul tuo computer, con tutta la sua storia.",
        "Copy a Git repository to your computer, with all its history.",
        "Copier un dépôt Git sur votre ordinateur, avec tout son historique.",
        "Ein Git-Repository mit seiner ganzen Geschichte auf deinen Computer kopieren.",
      ),
    ],
    [
      tr("Commit", "Commit", "Commit", "Commit"),
      tr(
        "Un salvataggio nella storia di Git: un insieme di modifiche con un messaggio.",
        "A save in Git's history: a set of changes with a message.",
        "Un enregistrement dans l'historique de Git : un ensemble de modifications avec un message.",
        "Ein Speicherpunkt in der Git-Geschichte: eine Reihe von Änderungen mit einer Nachricht.",
      ),
    ],
    [
      tr("Componente", "Component", "Composant", "Komponente"),
      tr(
        "Un programma che il kit sa installare: un linguaggio, un agente, un database, uno strumento.",
        "A program the kit can install: a language, an agent, a database, a tool.",
        "Un programme que le kit sait installer : un langage, un agent, une base de données, un outil.",
        "Ein Programm, das das Kit installieren kann: eine Sprache, ein Agent, eine Datenbank, ein Werkzeug.",
      ),
    ],
    [
      tr("Console (terminale)", "Console (terminal)", "Console (terminal)", "Konsole (Terminal)"),
      tr(
        "Una finestra in cui si scrivono comandi. Quella del kit è PowerShell 7 con gli strumenti del kit; su macOS e Linux, se PowerShell non è installato, è bash; gira sul PC, e lì claude, codex, pi e agy aprono l'agente nella VM del progetto. Anche il terminale di un progetto gira sul PC, con o senza la VM.",
        "A window where you type commands. The kit's one is PowerShell 7 with the kit's tools; on macOS and Linux, if PowerShell is not installed, it is bash; it runs on the PC, and there claude, codex, pi and agy open the agent in the project's VM. A project's terminal runs on the PC too, with or without the VM.",
        "Une fenêtre dans laquelle on tape des commandes. Celle du kit est PowerShell 7 avec les outils du kit ; sur macOS et Linux, si PowerShell n'est pas installé, c'est bash ; elle tourne sur le PC, et là claude, codex, pi et agy ouvrent l'agent dans la VM du projet. Le terminal d'un projet tourne lui aussi sur le PC, avec ou sans la VM.",
        "Ein Fenster, in das man Befehle eingibt. Die des Kits ist PowerShell 7 mit den Werkzeugen des Kits; unter macOS und Linux ist es bash, wenn PowerShell nicht installiert ist; sie läuft auf dem PC, und dort öffnen claude, codex, pi und agy den Agenten in der VM des Projekts. Auch das Terminal eines Projekts läuft auf dem PC, mit oder ohne VM.",
      ),
    ],
    [
      tr("Copia (backup)", "Backup", "Copie (sauvegarde)", "Sicherung (Backup)"),
      tr(
        "Un file con tutto il contenuto di un database, per rimetterlo com'era.",
        "A file with the whole content of a database, to put it back as it was.",
        "Un fichier avec tout le contenu d'une base de données, pour la remettre comme elle était.",
        "Eine Datei mit dem gesamten Inhalt einer Datenbank, um sie wieder in den alten Zustand zu bringen.",
      ),
    ],
    [
      tr(".env", ".env", ".env", ".env"),
      tr(
        "Un file di testo nel progetto con impostazioni e segreti (come la password del database), una riga CHIAVE=valore ciascuna. Non va messo in Git.",
        "A text file in the project with settings and secrets (like the database password), one KEY=value line each. It must not go into Git.",
        "Un fichier texte dans le projet avec des paramètres et des secrets (comme le mot de passe de la base de données), une ligne CLÉ=valeur chacun. Il ne doit pas aller dans Git.",
        "Eine Textdatei im Projekt mit Einstellungen und Geheimnissen (wie dem Datenbankpasswort), je eine Zeile SCHLÜSSEL=Wert. Sie gehört nicht in Git.",
      ),
    ],
    [
      tr("Estensione", "Extension", "Extension", "Erweiterung"),
      tr(
        "Un componente aggiuntivo dell'editor che lo rende capace di capire un linguaggio o di fare cose nuove.",
        "An add-on for the editor that makes it able to understand a language or do new things.",
        "Un module complémentaire de l'éditeur qui le rend capable de comprendre un langage ou de faire de nouvelles choses.",
        "Ein Zusatz für den Editor, der ihn eine Sprache verstehen oder neue Dinge tun lässt.",
      ),
    ],
    [
      tr("exFAT, FAT32, NTFS", "exFAT, FAT32, NTFS", "exFAT, FAT32, NTFS", "exFAT, FAT32, NTFS"),
      tr(
        "Formati dei dischi. NTFS (Windows) permette di proteggere le cartelle; exFAT è letto da Windows, macOS e Linux; FAT32 non accetta file oltre 4 GB.",
        "Disk formats. NTFS (Windows) lets you protect folders; exFAT is read by Windows, macOS and Linux; FAT32 does not accept files over 4 GB.",
        "Formats de disque. NTFS (Windows) permet de protéger les dossiers ; exFAT est lu par Windows, macOS et Linux ; FAT32 n'accepte pas les fichiers de plus de 4 Go.",
        "Dateisystemformate. NTFS (Windows) erlaubt es, Ordner zu schützen; exFAT wird von Windows, macOS und Linux gelesen; FAT32 akzeptiert keine Dateien über 4 GB.",
      ),
    ],
    [
      tr("Framework", "Framework", "Framework", "Framework"),
      tr(
        "Una struttura pronta per costruire un tipo di applicazione (Laravel per i siti PHP, Django per Python…).",
        "A ready-made structure for building a type of application (Laravel for PHP sites, Django for Python…).",
        "Une structure prête pour construire un type d'application (Laravel pour les sites PHP, Django pour Python…).",
        "Ein fertiges Gerüst, um eine Art von Anwendung zu bauen (Laravel für PHP-Websites, Django für Python…).",
      ),
    ],
    [
      tr("Git", "Git", "Git", "Git"),
      tr(
        "Il programma che tiene la storia delle modifiche di un progetto.",
        "The program that keeps the history of a project's changes.",
        "Le programme qui garde l'historique des modifications d'un projet.",
        "Das Programm, das die Geschichte der Änderungen eines Projekts festhält.",
      ),
    ],
    [
      tr("GitHub, GitHub CLI", "GitHub, GitHub CLI", "GitHub, GitHub CLI", "GitHub, GitHub CLI"),
      tr(
        "GitHub è il servizio online che ospita repository Git; GitHub CLI (gh) è il suo programma a riga di comando.",
        "GitHub is the online service that hosts Git repositories; GitHub CLI (gh) is its command-line program.",
        "GitHub est le service en ligne qui héberge des dépôts Git ; GitHub CLI (gh) est son programme en ligne de commande.",
        "GitHub ist der Online-Dienst, der Git-Repositorys hostet; GitHub CLI (gh) ist sein Befehlszeilenprogramm.",
      ),
    ],
    [
      tr("Impronta (hash)", "Hash", "Empreinte (hash)", "Hash (Prüfsumme)"),
      tr(
        "Un codice calcolato dal contenuto di un file: se il file cambia anche di un byte, l'impronta cambia. Serve a verificare che un download sia quello giusto.",
        "A code computed from a file's content: if the file changes by even one byte, the hash changes. It is used to verify that a download is the right one.",
        "Un code calculé à partir du contenu d'un fichier : si le fichier change ne serait-ce que d'un octet, l'empreinte change. Elle sert à vérifier qu'un téléchargement est le bon.",
        "Ein aus dem Inhalt einer Datei berechneter Code: Ändert sich die Datei auch nur um ein Byte, ändert sich der Hash. Er dient dazu zu prüfen, ob ein Download der richtige ist.",
      ),
    ],
    [
      tr("localhost, 127.0.0.1", "localhost, 127.0.0.1", "localhost, 127.0.0.1", "localhost, 127.0.0.1"),
      tr(
        "Il nome e l'indirizzo con cui un computer chiama sé stesso. Un nome che finisce in .localhost porta sempre a questo PC.",
        "The name and address a computer uses to call itself. A name ending in .localhost always leads to this PC.",
        "Le nom et l'adresse avec lesquels un ordinateur s'appelle lui-même. Un nom qui se termine par .localhost mène toujours à ce PC.",
        "Der Name und die Adresse, mit denen ein Computer sich selbst anspricht. Ein Name, der auf .localhost endet, führt immer zu diesem PC.",
      ),
    ],
    [
      tr("MCP (server MCP)", "MCP (MCP server)", "MCP (serveur MCP)", "MCP (MCP-Server)"),
      tr(
        "Model Context Protocol: un modo standard per dare agli agenti nuovi strumenti, attraverso un programma o un servizio.",
        "Model Context Protocol: a standard way to give agents new tools, through a program or a service.",
        "Model Context Protocol : une façon standard de donner aux agents de nouveaux outils, via un programme ou un service.",
        "Model Context Protocol: ein Standardweg, Agenten über ein Programm oder einen Dienst neue Werkzeuge zu geben.",
      ),
    ],
    [
      tr("Open VSX", "Open VSX", "Open VSX", "Open VSX"),
      tr(
        "Il catalogo libero di estensioni usato da VSCodium.",
        "The free extension catalog used by VSCodium.",
        "Le catalogue libre d'extensions utilisé par VSCodium.",
        "Der freie Erweiterungskatalog, den VSCodium verwendet.",
      ),
    ],
    [
      tr("PATH", "PATH", "PATH", "PATH"),
      tr(
        "L'elenco di cartelle in cui il sistema cerca i programmi quando scrivi un comando.",
        "The list of folders where the system looks for programs when you type a command.",
        "La liste des dossiers dans lesquels le système cherche les programmes quand vous tapez une commande.",
        "Die Liste der Ordner, in denen das System nach Programmen sucht, wenn du einen Befehl eingibst.",
      ),
    ],
    [
      tr("Pool", "Pool", "Pool", "Pool"),
      tr(
        "Un server web (Caddy o Apache) condiviso da più progetti, ognuno con il suo nome .localhost e la sua cartella conf.d.",
        "A web server (Caddy or Apache) shared by several projects, each with its own .localhost name and its own conf.d folder.",
        "Un serveur web (Caddy ou Apache) partagé par plusieurs projets, chacun avec son nom .localhost et son dossier conf.d.",
        "Ein Webserver (Caddy oder Apache), den mehrere Projekte gemeinsam nutzen, jedes mit seinem eigenen .localhost-Namen und seinem Ordner conf.d.",
      ),
    ],
    [
      tr("Porta", "Port", "Port", "Port"),
      tr(
        "Un numero che distingue i servizi sullo stesso computer: http://localhost:7100 e http://localhost:7101 sono due siti diversi. Nel kit ogni porta è di una cosa sola (un'azione, un pool, una versione di un database); 7000-7099 sono dell'editor.",
        "A number that tells services apart on the same computer: http://localhost:7100 and http://localhost:7101 are two different sites. In the kit each port belongs to one thing only (an action, a pool, a version of a database); 7000-7099 belong to the editor.",
        "Un numéro qui distingue les services sur un même ordinateur : http://localhost:7100 et http://localhost:7101 sont deux sites différents. Dans le kit, chaque port n'appartient qu'à une seule chose (une action, un pool, une version d'une base de données) ; 7000-7099 sont réservés à l'éditeur.",
        "Eine Nummer, die Dienste auf demselben Computer unterscheidet: http://localhost:7100 und http://localhost:7101 sind zwei verschiedene Websites. Im Kit gehört jeder Port nur einem (einer Aktion, einem Pool, einer Version einer Datenbank); 7000-7099 gehören dem Editor.",
      ),
    ],
    [
      tr("Profilo server", "Server profile", "Profil de serveur", "Serverprofil"),
      tr(
        "Un pool creato da te nella pagina Server Web, con le versioni e la porta che scegli.",
        "A pool you create on the Web Servers page, with the versions and port you choose.",
        "Un pool que vous créez dans la page Serveurs web, avec les versions et le port que vous choisissez.",
        "Ein Pool, den du auf der Seite Webserver anlegst, mit den Versionen und dem Port deiner Wahl.",
      ),
    ],
    [
      tr("Progetto", "Project", "Projet", "Projekt"),
      tr(
        "Una cartella dentro projects. Il kit ne riconosce il tipo dai file.",
        "A folder inside projects. The kit recognizes its type from the files.",
        "Un dossier dans projects. Le kit en reconnaît le type grâce aux fichiers.",
        "Ein Ordner in projects. Das Kit erkennt seine Art an den Dateien.",
      ),
    ],
    [
      tr("Proxy", "Proxy", "Proxy", "Proxy"),
      tr(
        "Un server intermedio da cui passa il traffico Internet, comune nelle reti aziendali.",
        "An intermediate server that Internet traffic goes through, common in company networks.",
        "Un serveur intermédiaire par lequel passe le trafic Internet, courant dans les réseaux d'entreprise.",
        "Ein Zwischenserver, über den der Internetverkehr läuft, üblich in Firmennetzwerken.",
      ),
    ],
    [
      tr("Push, pull", "Push, pull", "Push, pull", "Push, Pull"),
      tr(
        "Inviare i commit al repository remoto (push) e riceverne (pull).",
        "Sending commits to the remote repository (push) and receiving them (pull).",
        "Envoyer les commits au dépôt distant (push) et en recevoir (pull).",
        "Commits an das entfernte Repository senden (Push) und von dort empfangen (Pull).",
      ),
    ],
    [
      tr("Repository", "Repository", "Dépôt (repository)", "Repository"),
      tr(
        "Un progetto gestito con Git, con la sua storia. Può stare sul tuo computer e su GitHub.",
        "A project managed with Git, with its history. It can live on your computer and on GitHub.",
        "Un projet géré avec Git, avec son historique. Il peut être sur votre ordinateur et sur GitHub.",
        "Ein mit Git verwaltetes Projekt mit seiner Geschichte. Es kann auf deinem Computer und auf GitHub liegen.",
      ),
    ],
    [
      tr("Skill", "Skill", "Skill", "Skill"),
      tr(
        "Una procedura scritta che un agente carica quando serve, in una cartella con un file SKILL.md.",
        "A written procedure an agent loads when needed, in a folder with a SKILL.md file.",
        "Une procédure écrite qu'un agent charge quand il en a besoin, dans un dossier avec un fichier SKILL.md.",
        "Eine schriftliche Anleitung, die ein Agent bei Bedarf lädt, in einem Ordner mit einer Datei SKILL.md.",
      ),
    ],
    [
      tr("Token", "Token", "Jeton (token)", "Token"),
      tr(
        "Un codice segreto che dimostra un accesso già fatto (GitHub, Claude). Per i modelli IA, «token» è anche l'unità con cui si contano testo e consumi.",
        "A secret code that proves a sign-in already done (GitHub, Claude). For AI models, “token” is also the unit used to count text and usage.",
        "Un code secret qui prouve une connexion déjà faite (GitHub, Claude). Pour les modèles d'IA, « token » est aussi l'unité avec laquelle on compte le texte et la consommation.",
        "Ein geheimer Code, der eine bereits erfolgte Anmeldung belegt (GitHub, Claude). Bei KI-Modellen ist „Token“ auch die Einheit, in der Text und Verbrauch gezählt werden.",
      ),
    ],
    [
      tr("Versione minore", "Minor version", "Version mineure", "Nebenversion"),
      tr(
        "Le prime due parti di una versione: di PHP 8.4.3 la versione minore è 8.4. Il kit tiene una cartella per versione minore.",
        "The first two parts of a version: for PHP 8.4.3 the minor version is 8.4. The kit keeps one folder per minor version.",
        "Les deux premières parties d'une version : pour PHP 8.4.3, la version mineure est 8.4. Le kit garde un dossier par version mineure.",
        "Die ersten beiden Teile einer Version: Bei PHP 8.4.3 ist die Nebenversion 8.4. Das Kit hat einen Ordner pro Nebenversion.",
      ),
    ],
    [
      tr("VirtualHost", "VirtualHost", "VirtualHost", "VirtualHost"),
      tr(
        "Un sito dentro un server web, riconosciuto dal nome con cui lo chiami.",
        "A site inside a web server, recognized by the name you call it with.",
        "Un site dans un serveur web, reconnu grâce au nom avec lequel vous l'appelez.",
        "Eine Website in einem Webserver, erkannt an dem Namen, mit dem du sie aufrufst.",
      ),
    ],
    [
      tr("VSCodium", "VSCodium", "VSCodium", "VSCodium"),
      tr(
        "La versione libera di Visual Studio Code: l'editor del kit.",
        "The free version of Visual Studio Code: the kit's editor.",
        "La version libre de Visual Studio Code : l'éditeur du kit.",
        "Die freie Version von Visual Studio Code: der Editor des Kits.",
      ),
    ],
  ];
}

export const HELP: TopicTexts = {
  troubleshooting: () => (
    <>
      <InBrief
        points={[
          tr(
            "Cerca il messaggio che vedi nella casella qui sotto: la guida ne raccoglie i più comuni, con il rimedio.",
            "Search for the message you see in the box below: the guide collects the most common ones, with the remedy.",
            "Cherchez le message que vous voyez dans le champ ci-dessous : le guide rassemble les plus courants, avec la solution.",
            "Such die Meldung, die du siehst, im Feld unten: Die Anleitung sammelt die häufigsten, mit der Abhilfe.",
          ),
          tr(
            "Quasi ogni errore ha un suggerimento al passaggio del mouse e una riga di output o di log che dice di più.",
            "Almost every error has a tooltip on mouse-over and a line of output or log that tells you more.",
            "Presque chaque erreur a une info-bulle au survol de la souris et une ligne de sortie ou de journal qui en dit plus.",
            "Fast jeder Fehler hat einen Tooltip beim Darüberfahren mit der Maus und eine Zeile in der Ausgabe oder im Log, die mehr verrät.",
          ),
          tr(
            "Il log dell'app è in data\\app\\logs\\app.log; quelli dei server accanto.",
            "The app's log is in data\\app\\logs\\app.log; the servers' logs are next to it.",
            "Le journal de l'application est dans data\\app\\logs\\app.log ; ceux des serveurs à côté.",
            "Das Log der App liegt in data\\app\\logs\\app.log; die der Server daneben.",
          ),
        ]}
      />
      <Section title={tr("Salute del kit", "Kit health", "Santé du kit", "Zustand des Kits")}>
        <P>
          {tr(
            "Su un PC nuovo, o con il kit su un'altra chiavetta, guarda prima Impostazioni › «Il kit» › «Salute del kit»: una riga per controllo, verde se va bene, gialla per un avviso, rossa per un errore, e sotto che cosa fare. «Controlla di nuovo» rifà i controlli dopo una correzione.",
            "On a new PC, or with the kit on another stick, look first at Settings › “The kit” › “Kit health”: one line per check, green when fine, yellow for a warning, red for an error, and under it what to do. “Check again” runs the checks again after a fix.",
            "Sur un nouveau PC, ou avec le kit sur une autre clé, regardez d'abord Paramètres › « Le kit » › « Santé du kit » : une ligne par contrôle, verte si tout va bien, jaune pour un avertissement, rouge pour une erreur, et en dessous ce qu'il faut faire. « Vérifier à nouveau » refait les contrôles après une correction.",
            "Auf einem neuen PC, oder mit dem Kit auf einem anderen Stick, sieh zuerst unter Einstellungen › „Das Kit“ › „Zustand des Kits“ nach: eine Zeile pro Prüfung, grün wenn alles passt, gelb für eine Warnung, rot für einen Fehler, und darunter, was zu tun ist. „Erneut prüfen“ wiederholt die Prüfungen nach einer Korrektur.",
          )}
        </P>
        <Bullets
          items={[
            tr(
              "Il disco. FAT32 tiene file fino a 4 GB (niente modelli locali né copie di database grandi) e niente collegamenti: per un kit su chiavetta è meglio exFAT, o NTFS se resta su Windows. Su macOS e Linux un disco montato noexec (o FAT32 con showexec su Linux) non lascia partire i programmi: la riga dà il comando per rimontarlo. Su FAT32 ed exFAT la VM dei progetti non parte (motivo nell'avviso «Senza VM»).",
              "The drive. FAT32 holds files up to 4 GB (no large local models or database copies) and no links: for a kit on a stick exFAT is better, or NTFS if it stays on Windows. On macOS and Linux a drive mounted noexec (or FAT32 with showexec on Linux) lets no program start: the line gives the command to remount it. On FAT32 and exFAT the projects' VM does not start (reason in the “No VM” notice).",
              "Le disque. FAT32 accepte des fichiers jusqu'à 4 Go (pas de grands modèles locaux ni de grandes copies de bases de données) et aucun lien : pour un kit sur clé, exFAT est préférable, ou NTFS s'il reste sous Windows. Sous macOS et Linux, un disque monté noexec (ou FAT32 avec showexec sous Linux) ne laisse démarrer aucun programme : la ligne donne la commande pour le remonter. Sur FAT32 et exFAT, la VM des projets ne démarre pas (motif dans l'avis « Sans VM »).",
              "Das Laufwerk. FAT32 nimmt Dateien bis 4 GB (keine großen lokalen Modelle oder Datenbankkopien) und keine Links: Für ein Kit auf einem Stick ist exFAT besser, oder NTFS, wenn es unter Windows bleibt. Unter macOS und Linux lässt ein mit noexec eingehängtes Laufwerk (oder FAT32 mit showexec unter Linux) kein Programm starten: Die Zeile nennt den Befehl, um es neu einzuhängen. Auf FAT32 und exFAT startet die VM der Projekte nicht (Grund im Hinweis „Ohne VM“).",
            ),
            tr(
              "Lo spazio libero: sotto 5 GB un avviso, sotto 1 GB un errore (installazioni, download e copie dei database non riescono). Libera spazio o sposta il kit su un disco più grande.",
              "The free space: under 5 GB a warning, under 1 GB an error (installs, downloads and database copies fail). Free up space or move the kit to a larger drive.",
              "L'espace libre : sous 5 Go un avertissement, sous 1 Go une erreur (installations, téléchargements et copies des bases de données échouent). Libérez de l'espace ou déplacez le kit sur un disque plus grand.",
              "Der freie Platz: unter 5 GB eine Warnung, unter 1 GB ein Fehler (Installationen, Downloads und Datenbankkopien schlagen fehl). Schaffe Platz oder verschiebe das Kit auf ein größeres Laufwerk.",
            ),
            tr(
              "Il percorso del kit: &, %, !, ^, apici e lettere accentate o fuori dall'alfabeto latino confondono cmd, PowerShell, Maven e Gradle. Sposta il kit in una cartella con solo lettere, cifre, trattini e spazi.",
              "The kit's path: &, %, !, ^, quotes and accented or non-Latin letters confuse cmd, PowerShell, Maven and Gradle. Move the kit to a folder with only letters, digits, hyphens and spaces.",
              "Le chemin du kit : &, %, !, ^, les apostrophes et guillemets, les lettres accentuées ou hors de l'alphabet latin perturbent cmd, PowerShell, Maven et Gradle. Déplacez le kit dans un dossier avec seulement des lettres, des chiffres, des tirets et des espaces.",
              "Der Pfad des Kits: &, %, !, ^, Anführungszeichen sowie Umlaute und Buchstaben außerhalb des lateinischen Alphabets bringen cmd, PowerShell, Maven und Gradle durcheinander. Verschiebe das Kit in einen Ordner mit nur Buchstaben, Ziffern, Bindestrichen und Leerzeichen.",
            ),
            tr(
              "File resi privati da un altro account, spesso su un altro PC: il kit non li legge (impostazioni, chiavi, accessi degli agenti). La riga dà il comando per riprenderli, da eseguire in un terminale da amministratore: takeown e icacls su Windows, sudo chown su macOS e Linux. L'app non lo esegue da sé perché servono i diritti di amministratore.",
              "Files made private by another account, often on another PC: the kit cannot read them (settings, keys, the agents' sign-ins). The line gives the command to take them back, to run in an administrator terminal: takeown and icacls on Windows, sudo chown on macOS and Linux. The app does not run it itself because it needs administrator rights.",
              "Des fichiers rendus privés par un autre compte, souvent sur un autre PC : le kit ne peut pas les lire (paramètres, clés, connexions des agents). La ligne donne la commande pour les récupérer, à lancer dans un terminal administrateur : takeown et icacls sous Windows, sudo chown sous macOS et Linux. L'application ne la lance pas elle-même car il faut les droits d'administrateur.",
              "Von einem anderen Konto privat gemachte Dateien, oft auf einem anderen PC: Das Kit kann sie nicht lesen (Einstellungen, Schlüssel, Anmeldungen der Agenten). Die Zeile nennt den Befehl, um sie zurückzuholen, auszuführen in einem Administrator-Terminal: takeown und icacls unter Windows, sudo chown unter macOS und Linux. Die App führt ihn nicht selbst aus, weil Administratorrechte nötig sind.",
            ),
            tr(
              "Su Linux, le librerie del sistema: libaio e libnuma per MySQL, libicu per .NET, libvulkan1 per la scheda video dei modelli locali (senza, restano sulla CPU). La riga dà il comando per installarle, per esempio con apt.",
              "On Linux, the system's libraries: libaio and libnuma for MySQL, libicu for .NET, libvulkan1 for the graphics card of the local models (without it they stay on the CPU). The line gives the command to install them, for example with apt.",
              "Sous Linux, les bibliothèques du système : libaio et libnuma pour MySQL, libicu pour .NET, libvulkan1 pour la carte graphique des modèles locaux (sans elle, ils restent sur le CPU). La ligne donne la commande pour les installer, par exemple avec apt.",
              "Unter Linux die Bibliotheken des Systems: libaio und libnuma für MySQL, libicu für .NET, libvulkan1 für die Grafikkarte der lokalen Modelle (ohne sie bleiben sie auf der CPU). Die Zeile nennt den Befehl, um sie zu installieren, zum Beispiel mit apt.",
            ),
            rich(
              tr(
                "Dalla riga di comando gli stessi controlli li fa {0}, che finisce con 1 se uno è un errore.",
                "From the command line {0} runs the same checks, and ends with 1 if one is an error.",
                "En ligne de commande, {0} fait les mêmes contrôles, et se termine par 1 si l'un d'eux est une erreur.",
                "In der Kommandozeile macht {0} dieselben Prüfungen und endet mit 1, wenn eine davon ein Fehler ist.",
              ),
              <Code key="k">.\Atollo.exe check --kit | Out-Host</Code>,
            ),
          ]}
        />
      </Section>
      <FilterList
        groups={problems()}
        placeholder={tr(
          "Cerca un messaggio o un problema",
          "Search for a message or a problem",
          "Chercher un message ou un problème",
          "Nach einer Meldung oder einem Problem suchen",
        )}
      />
      <Section
        title={tr(
          "Se il problema non è qui",
          "If the problem is not here",
          "Si le problème n'est pas ici",
          "Wenn das Problem nicht hier steht",
        )}
      >
        <Bullets
          items={[
            tr(
              "Leggi l'output dell'azione (sezione «Avvio») o il log del server: l'ultima riga di solito dice la causa.",
              "Read the action's output (“Run” section) or the server's log: the last line usually gives the cause.",
              "Lisez la sortie de l'action (section « Lancement ») ou le journal du serveur : la dernière ligne donne généralement la cause.",
              "Lies die Ausgabe der Aktion (Abschnitt „Start“) oder das Log des Servers: Die letzte Zeile nennt meist die Ursache.",
            ),
            tr(
              "Chiudi Atollo e riaprilo: molte cose (ambiente, agenti, file del kit) vengono rigenerate a ogni avvio.",
              "Close Atollo and open it again: many things (environment, agents, kit files) are regenerated at every start.",
              "Fermez Atollo et rouvrez-le : beaucoup de choses (environnement, agents, fichiers du kit) sont régénérées à chaque démarrage.",
              "Schließe Atollo und öffne es erneut: Vieles (Umgebung, Agenten, Dateien des Kits) wird bei jedem Start neu erzeugt.",
            ),
            rich(
              tr(
                "Controlla i componenti con «Verifica» nella pagina Strumenti, o con {0}.",
                "Check the components with “Verify” on the Tools page, or with {0}.",
                "Contrôlez les composants avec « Vérifier » dans la page Outils, ou avec {0}.",
                "Prüfe die Komponenten mit „Prüfen“ auf der Seite Werkzeuge oder mit {0}.",
              ),
              <Code key="c">.\Atollo.exe check | Out-Host</Code>,
            ),
            tr(
              "Se ancora non va, segnalalo: la sezione qui sotto dice come.",
              "If it still does not work, report it: the section below says how.",
              "Si cela ne marche toujours pas, signalez-le : la section ci-dessous explique comment.",
              "Wenn es immer noch nicht klappt, melde es: Der Abschnitt unten erklärt, wie.",
            ),
          ]}
        />
      </Section>
      <Section title={tr("Segnalare un problema", "Reporting a problem", "Signaler un problème", "Ein Problem melden")}>
        <P>
          {tr(
            "In Impostazioni › «Il kit» › «Segnalare un problema» ci sono due pulsanti, da usare in quest'ordine.",
            "In Settings › “The kit” › “Reporting a problem” there are two buttons, to use in this order.",
            "Dans Paramètres › « Le kit » › « Signaler un problème », il y a deux boutons, à utiliser dans cet ordre.",
            "Unter Einstellungen › „Das Kit“ › „Ein Problem melden“ gibt es zwei Schaltflächen, in dieser Reihenfolge zu verwenden.",
          )}
        </P>
        <Bullets
          items={[
            rich(
              tr(
                "«Esporta diagnostica» crea sul PC lo zip atollo-diagnostica-<data-ora>.zip in data\\app\\diagnostics: versione e sistema, disco del kit (file system, tipo, spazio libero), RAM, ambiente del kit, la verifica dei programmi, i componenti, le porte e la parte finale di ogni log. «Apri la cartella» lo mostra. Dalla riga di comando fa lo stesso {0}.",
                "“Export diagnostics” makes the zip atollo-diagnostica-<date-time>.zip on the PC in data\\app\\diagnostics: version and system, the kit's disk (file system, kind, free space), RAM, the kit's environment, the programs' check, the components, the ports and the end of every log. “Open folder” shows it. From the command line {0} does the same.",
                "« Exporter le diagnostic » crée sur le PC le zip atollo-diagnostica-<date-heure>.zip dans data\\app\\diagnostics : version et système, disque du kit (système de fichiers, type, espace libre), RAM, environnement du kit, la vérification des programmes, les composants, les ports et la fin de chaque journal. « Ouvrir le dossier » l'affiche. En ligne de commande, {0} fait la même chose.",
                "„Diagnose exportieren“ erstellt auf dem PC das Zip atollo-diagnostica-<Datum-Uhrzeit>.zip in data\\app\\diagnostics: Version und System, Laufwerk des Kits (Dateisystem, Art, freier Platz), RAM, Umgebung des Kits, die Prüfung der Programme, die Komponenten, die Ports und das Ende jedes Logs. „Ordner öffnen“ zeigt es. In der Kommandozeile macht {0} dasselbe.",
              ),
              <Code key="d">.\Atollo.exe diagnostics | Out-Host</Code>,
            ),
            tr(
              "Prima di scrivere lo zip, ogni segreto che il kit conserva (le chiavi di data\\app\\secrets, il token di Claude Code, le password dei database, gli accessi degli agenti, il token di GitHub) diventa *** ovunque compaia; la cartella del tuo profilo diventa ~, il tuo nome utente e il nome del PC ***. Aprilo e guardalo comunque prima di allegarlo: un progetto può scrivere nei log cose che il kit non conosce.",
              "Before the zip is written, every secret the kit keeps (the keys of data\\app\\secrets, Claude Code's token, the database passwords, the agents' sign-ins, the GitHub token) becomes *** wherever it appears; your profile folder becomes ~, your user name and the PC's name ***. Open it and look at it anyway before attaching it: a project can write things into the logs that the kit does not know.",
              "Avant l'écriture du zip, chaque secret que le kit conserve (les clés de data\\app\\secrets, le jeton de Claude Code, les mots de passe des bases de données, les connexions des agents, le jeton GitHub) devient *** partout où il apparaît ; le dossier de votre profil devient ~, votre nom d'utilisateur et le nom du PC ***. Ouvrez-le et regardez-le quand même avant de le joindre : un projet peut écrire dans les journaux des choses que le kit ne connaît pas.",
              "Bevor das Zip geschrieben wird, wird jedes Geheimnis, das das Kit aufbewahrt (die Schlüssel in data\\app\\secrets, das Token von Claude Code, die Datenbankpasswörter, die Anmeldungen der Agenten, das GitHub-Token), überall durch *** ersetzt; dein Profilordner wird zu ~, dein Benutzername und der Name des PCs zu ***. Öffne es trotzdem und sieh es dir an, bevor du es anhängst: Ein Projekt kann Dinge in die Logs schreiben, die das Kit nicht kennt.",
            ),
            tr(
              "«Segnala un problema» apre nel browser una segnalazione nuova su github.com/nabre/atollo con il modulo dei problemi: versione di Atollo e sistema (kit, build di Windows, file system e tipo del disco) sono già scritti. Racconta che cosa è successo e i passi per rifarlo, e trascina lo zip nella segnalazione.",
              "“Report a problem” opens a new issue on github.com/nabre/atollo in the browser with the bug form: Atollo's version and the system (kit, Windows build, the disk's file system and kind) are already filled in. Tell what happened and the steps to reproduce it, and drag the zip into the issue.",
              "« Signaler un problème » ouvre dans le navigateur une nouvelle issue sur github.com/nabre/atollo avec le formulaire des problèmes : la version d'Atollo et le système (kit, build de Windows, système de fichiers et type du disque) sont déjà remplis. Racontez ce qui s'est passé et les étapes pour le reproduire, et faites glisser le zip dans l'issue.",
              "„Problem melden“ öffnet im Browser ein neues Issue auf github.com/nabre/atollo mit dem Fehlerformular: Version von Atollo und System (Kit, Windows-Build, Dateisystem und Art des Laufwerks) sind schon eingetragen. Beschreibe, was passiert ist und die Schritte, um es nachzustellen, und zieh das Zip in das Issue.",
            ),
            tr(
              "L'app non manda niente: lo zip resta sul PC finché non lo alleghi tu, e la segnalazione arriva a GitHub solo quando premi «Create» (serve un account GitHub). Da lì valgono le regole sulla privacy di GitHub, e la segnalazione è pubblica.",
              "The app sends nothing: the zip stays on the PC until you attach it, and the issue reaches GitHub only when you press “Create” (a GitHub account is needed). From then on GitHub's privacy rules apply, and the issue is public.",
              "L'application n'envoie rien : le zip reste sur le PC tant que vous ne le joignez pas, et l'issue n'arrive à GitHub que lorsque vous appuyez sur « Create » (il faut un compte GitHub). À partir de là, les règles de confidentialité de GitHub s'appliquent, et l'issue est publique.",
              "Die App sendet nichts: Das Zip bleibt auf dem PC, bis du es anhängst, und das Issue erreicht GitHub erst, wenn du auf „Create“ klickst (ein GitHub-Konto ist nötig). Ab dann gelten die Datenschutzregeln von GitHub, und das Issue ist öffentlich.",
            ),
          ]}
        />
      </Section>
      <Related ids={["faq", "vm", "network", "lifecycle"]} />
    </>
  ),

  faq: () => (
    <>
      <InBrief
        points={[
          tr(
            "Risposte brevi; ognuna rimanda all'argomento che spiega di più.",
            "Short answers; each one points to the topic that explains more.",
            "Des réponses brèves ; chacune renvoie au sujet qui en explique davantage.",
            "Kurze Antworten; jede verweist auf das Thema, das mehr erklärt.",
          ),
        ]}
      />
      <Section title={tr("Domande frequenti", "Frequently asked questions", "Questions fréquentes", "Häufige Fragen")}>
        <QA
          q={tr(
            "Funziona senza Internet?",
            "Does it work without the Internet?",
            "Est-ce que ça fonctionne sans Internet ?",
            "Funktioniert es ohne Internet?",
          )}
        >
          <P>
            {tr(
              "Sì, per tutto ciò che è già installato: progetti, database, siti, editor, console. Servono Internet i download dei componenti, i comandi che scaricano pacchetti (npm, Composer, pip) e gli agenti IA, che parlano con i server del fornitore.",
              "Yes, for everything already installed: projects, databases, sites, editor, consoles. The Internet is needed for component downloads, for commands that download packages (npm, Composer, pip) and for the AI agents, which talk to the provider's servers.",
              "Oui, pour tout ce qui est déjà installé : projets, bases de données, sites, éditeur, consoles. Internet est nécessaire pour les téléchargements des composants, les commandes qui téléchargent des paquets (npm, Composer, pip) et les agents IA, qui parlent avec les serveurs du fournisseur.",
              "Ja, für alles, was bereits installiert ist: Projekte, Datenbanken, Websites, Editor, Konsolen. Internet brauchen die Downloads der Komponenten, die Befehle, die Pakete herunterladen (npm, Composer, pip), und die KI-Agenten, die mit den Servern des Anbieters sprechen.",
            )}
          </P>
        </QA>
        <QA q={tr("Quanto costa?", "How much does it cost?", "Combien ça coûte ?", "Was kostet es?")}>
          <P>
            {tr(
              "Atollo è gratuito e open source (licenza MIT), come i componenti che scarica. Gli agenti IA richiedono un account o una chiave API del fornitore, che può essere a pagamento.",
              "Atollo is free and open source (MIT license), like the components it downloads. The AI agents require an account or an API key from the provider, which may be paid.",
              "Atollo est gratuit et open source (licence MIT), comme les composants qu'il télécharge. Les agents IA nécessitent un compte ou une clé API du fournisseur, qui peut être payant.",
              "Atollo ist kostenlos und Open Source (MIT-Lizenz), wie die Komponenten, die es herunterlädt. Die KI-Agenten erfordern ein Konto oder einen API-Schlüssel des Anbieters, der kostenpflichtig sein kann.",
            )}
          </P>
        </QA>
        <QA
          q={tr(
            "Il mio codice va da qualche parte?",
            "Does my code go anywhere?",
            "Mon code va-t-il quelque part ?",
            "Geht mein Code irgendwohin?",
          )}
        >
          <P>
            {tr(
              "Il kit non manda niente: niente telemetria, e gli strumenti hanno la telemetria spenta. Quando usi un agente IA, però, il codice che gli fai leggere viene inviato al fornitore del modello (Anthropic, OpenAI, Google), secondo le sue condizioni. Il riassunto è in Impostazioni › «Privacy», con il link «Leggi l'informativa sulla privacy».",
              "The kit sends nothing: no telemetry, and the tools have telemetry turned off. When you use an AI agent, however, the code you have it read is sent to the model's provider (Anthropic, OpenAI, Google), under its terms. The summary is in Settings › “Privacy”, with the link “Read the privacy policy”.",
              "Le kit n'envoie rien : pas de télémétrie, et les outils ont la télémétrie désactivée. Quand vous utilisez un agent IA, en revanche, le code que vous lui faites lire est envoyé au fournisseur du modèle (Anthropic, OpenAI, Google), selon ses conditions. Le résumé se trouve dans Paramètres › « Confidentialité », avec le lien « Lire la politique de confidentialité ».",
              "Das Kit sendet nichts: keine Telemetrie, und bei den Werkzeugen ist die Telemetrie ausgeschaltet. Wenn du einen KI-Agenten verwendest, wird der Code, den du ihn lesen lässt, allerdings an den Anbieter des Modells gesendet (Anthropic, OpenAI, Google), gemäß dessen Bedingungen. Die Zusammenfassung steht unter Einstellungen › „Datenschutz“, mit dem Link „Datenschutzerklärung lesen“.",
            )}
          </P>
        </QA>
        <QA
          q={tr(
            "Come disinstallo Atollo?",
            "How do I uninstall Atollo?",
            "Comment désinstaller Atollo ?",
            "Wie deinstalliere ich Atollo?",
          )}
        >
          <P>
            {tr(
              "Chiudi l'app e cancella la cartella del kit (dopo aver messo al sicuro i progetti che ti servono). Non ci sono voci di registro né programmi installati da togliere. La cartella temporanea del sistema viene pulita alla chiusura.",
              "Close the app and delete the kit folder (after putting the projects you need somewhere safe). There are no registry entries or installed programs to remove. The system's temporary folder is cleaned on closing.",
              "Fermez l'application et supprimez le dossier du kit (après avoir mis à l'abri les projets dont vous avez besoin). Il n'y a ni entrées de registre ni programmes installés à retirer. Le dossier temporaire du système est nettoyé à la fermeture.",
              "Schließe die App und lösche den Kit-Ordner (nachdem du die Projekte, die du brauchst, in Sicherheit gebracht hast). Es gibt keine Registry-Einträge und keine installierten Programme zu entfernen. Der temporäre Ordner des Systems wird beim Schließen bereinigt.",
            )}
          </P>
        </QA>
        <QA
          q={tr(
            "Posso tenere i progetti fuori da projects?",
            "Can I keep projects outside projects?",
            "Puis-je garder les projets hors de projects ?",
            "Kann ich Projekte außerhalb von projects aufbewahren?",
          )}
        >
          <P>
            {rich(
              tr(
                "La pagina Progetti mostra solo le cartelle dentro projects. Per lavorare con gli strumenti del kit su una cartella qualsiasi del PC usa la pagina Terminali ({0}), che apre console ovunque.",
                "The Projects page shows only the folders inside projects. To work with the kit's tools on any folder of the PC use the Terminals page ({0}), which opens consoles anywhere.",
                "La page Projets n'affiche que les dossiers dans projects. Pour travailler avec les outils du kit sur n'importe quel dossier du PC, utilisez la page Terminaux ({0}), qui ouvre des consoles partout.",
                "Die Seite Projekte zeigt nur die Ordner in projects. Um mit den Werkzeugen des Kits in einem beliebigen Ordner des PCs zu arbeiten, verwende die Seite Terminals ({0}), die überall Konsolen öffnet.",
              ),
              <See key="t" id="terminals" />,
            )}
          </P>
        </QA>
        <QA
          q={tr(
            "Posso usare un altro editor?",
            "Can I use another editor?",
            "Puis-je utiliser un autre éditeur ?",
            "Kann ich einen anderen Editor verwenden?",
          )}
        >
          <P>
            {tr(
              "I progetti sono cartelle normali: puoi aprirle con qualsiasi editor. Ma gli strumenti del kit (Node, PHP…) e le versioni del progetto sono pronti solo nelle console e nell'editor del kit, perché è il kit a preparare il loro ambiente.",
              "Projects are normal folders: you can open them with any editor. But the kit's tools (Node, PHP…) and the project's versions are ready only in the kit's consoles and editor, because it is the kit that prepares their environment.",
              "Les projets sont des dossiers normaux : vous pouvez les ouvrir avec n'importe quel éditeur. Mais les outils du kit (Node, PHP…) et les versions du projet ne sont prêts que dans les consoles et l'éditeur du kit, car c'est le kit qui prépare leur environnement.",
              "Projekte sind normale Ordner: Du kannst sie mit jedem Editor öffnen. Aber die Werkzeuge des Kits (Node, PHP…) und die Versionen des Projekts stehen nur in den Konsolen und im Editor des Kits bereit, weil das Kit ihre Umgebung vorbereitet.",
            )}
          </P>
        </QA>
        <QA
          q={tr(
            "Atollo sostituisce Docker?",
            "Does Atollo replace Docker?",
            "Atollo remplace-t-il Docker ?",
            "Ersetzt Atollo Docker?",
          )}
        >
          <P>
            {tr(
              "No, e Docker Desktop non serve. I programmi del kit sono nativi, scaricati dalle fonti ufficiali, senza amministratore; se un progetto ha un compose.yaml o un Dockerfile, il kit li legge per suggerire le versioni giuste (PHP, Node, MySQL…). Docker Sandboxes, un componente obbligatorio del kit, dà agli agenti di ogni progetto una microVM, dove girano con gli strumenti del kit per Linux: isola gli agenti (domanda qui sotto), non sostituisce i programmi del kit, che girano sul PC.",
              "No, and Docker Desktop is not needed. The kit's programs are native, downloaded from the official sources, without an administrator; if a project has a compose.yaml or a Dockerfile, the kit reads them to suggest the right versions (PHP, Node, MySQL…). Docker Sandboxes, a required component of the kit, gives each project's agents a microVM, where they run with the kit's tools for Linux: it isolates the agents (question below), it does not replace the kit's programs, which run on the PC.",
              "Non, et Docker Desktop n'est pas nécessaire. Les programmes du kit sont natifs, téléchargés depuis les sources officielles, sans administrateur ; si un projet a un compose.yaml ou un Dockerfile, le kit les lit pour suggérer les bonnes versions (PHP, Node, MySQL…). Docker Sandboxes, un composant obligatoire du kit, donne aux agents de chaque projet une microVM, où ils tournent avec les outils du kit pour Linux : il isole les agents (question ci-dessous), il ne remplace pas les programmes du kit, qui tournent sur le PC.",
              "Nein, und Docker Desktop wird nicht gebraucht. Die Programme des Kits sind nativ, aus den offiziellen Quellen geladen, ohne Administrator; wenn ein Projekt eine compose.yaml oder ein Dockerfile hat, liest das Kit sie, um die richtigen Versionen vorzuschlagen (PHP, Node, MySQL…). Docker Sandboxes, eine Pflichtkomponente des Kits, gibt den Agenten jedes Projekts eine microVM, in der sie mit den Werkzeugen des Kits für Linux laufen: Es isoliert die Agenten (Frage unten), es ersetzt die Programme des Kits nicht, die auf dem PC laufen.",
            )}
          </P>
        </QA>
        <QA
          q={tr(
            "Cosa vuol dire «Senza VM» nella barra di stato?",
            "What does “No VM” in the status bar mean?",
            "Que signifie « Sans VM » dans la barre d'état ?",
            "Was bedeutet „Ohne VM“ in der Statusleiste?",
          )}
        >
          <P>
            {rich(
              tr(
                "Che nessun agente parte (i loro pulsanti dicono perché): gli agenti girano solo nella VM, dove vedono solo la cartella del progetto. Il resto lavora come sempre: i programmi dei progetti (terminale, azioni, server) girano sul PC con il tuo utente, con o senza la VM, e possono leggere i tuoi file; il kit ne sigilla l'ambiente (niente chiavi SSH, niente login di amministratore di MySQL: ogni progetto entra con il suo utente), ma non è un muro. Un clic sull'avviso dice perché la VM non è attiva e che cosa fare, con «Lancia», «Copia», «Installa nel kit» e «Ricontrolla»: di solito servono Windows 11 con Hypervisor Platform, un Mac con Apple silicon e macOS 14, o Ubuntu 24.04 con KVM, più un account Docker («sbx login» nel terminale di Atollo). Tutto in {0}.",
                "That no agent starts (their buttons say why): the agents run only in the VM, where they see only the project's folder. The rest works as always: the projects' programs (terminal, actions, servers) run on the PC as your user, with or without the VM, and can read your files; the kit seals their environment (no SSH keys, no MySQL administrator login: each project signs in as its own user), but that is no wall. A click on the notice tells why the VM is off and what to do, with “Run”, “Copy”, “Install in the kit” and “Check again”: usually Windows 11 with Hypervisor Platform, a Mac with Apple silicon and macOS 14, or Ubuntu 24.04 with KVM, plus a Docker account (“sbx login” in Atollo's terminal). All of it in {0}.",
                "Qu'aucun agent ne démarre (leurs boutons disent pourquoi) : les agents ne tournent que dans la VM, où ils ne voient que le dossier du projet. Le reste fonctionne comme toujours : les programmes des projets (terminal, actions, serveurs) tournent sur le PC avec votre utilisateur, avec ou sans la VM, et peuvent lire vos fichiers ; le kit scelle leur environnement (pas de clés SSH, pas de login administrateur de MySQL : chaque projet se connecte avec son utilisateur), mais ce n'est pas un mur. Un clic sur l'avis dit pourquoi la VM n'est pas active et quoi faire, avec « Lancer », « Copier », « Installer dans le kit » et « Vérifier à nouveau » : en général il faut Windows 11 avec Hypervisor Platform, un Mac Apple silicon sous macOS 14, ou Ubuntu 24.04 avec KVM, plus un compte Docker (« sbx login » dans le terminal d'Atollo). Tout est dans {0}.",
                "Dass kein Agent startet (ihre Schaltflächen sagen warum): Die Agenten laufen nur in der VM, wo sie nur den Ordner des Projekts sehen. Der Rest arbeitet wie immer: Die Programme der Projekte (Terminal, Aktionen, Server) laufen auf dem PC mit deinem Benutzer, mit oder ohne VM, und können deine Dateien lesen; das Kit versiegelt ihre Umgebung (keine SSH-Schlüssel, keine Administrator-Anmeldung von MySQL: Jedes Projekt meldet sich mit seinem Benutzer an), aber das ist keine Mauer. Ein Klick auf den Hinweis sagt, warum die VM nicht aktiv ist und was zu tun ist, mit „Starten“, „Kopieren“, „Im Kit installieren“ und „Erneut prüfen“: Meist braucht es Windows 11 mit Hypervisor Platform, einen Mac mit Apple Silicon und macOS 14 oder Ubuntu 24.04 mit KVM, dazu ein Docker-Konto („sbx login“ im Terminal von Atollo). Alles in {0}.",
              ),
              <See key="v" id="vm" />,
            )}
          </P>
        </QA>
        <QA
          q={tr(
            "Posso avere due kit?",
            "Can I have two kits?",
            "Puis-je avoir deux kits ?",
            "Kann ich zwei Kits haben?",
          )}
        >
          <P>
            {tr(
              "Sì: ogni cartella è un kit indipendente, con i suoi progetti, accessi e impostazioni, e due kit possono essere aperti insieme. Lo stesso kit, invece, si apre una volta sola. Solo un kit alla volta ha la VM: Docker Sandboxes ne serve uno per utente, e l'altro lavora «Senza VM».",
              "Yes: each folder is an independent kit, with its own projects, sign-ins and settings, and two kits can be open together. The same kit, however, opens only once. One kit at a time has the VM: Docker Sandboxes serves one per user, and the other works with “No VM”.",
              "Oui : chaque dossier est un kit indépendant, avec ses projets, connexions et paramètres, et deux kits peuvent être ouverts ensemble. Le même kit, en revanche, ne s'ouvre qu'une fois. Un seul kit à la fois a la VM : Docker Sandboxes en sert un par utilisateur, et l'autre travaille « Sans VM ».",
              "Ja: Jeder Ordner ist ein eigenständiges Kit mit eigenen Projekten, Anmeldungen und Einstellungen, und zwei Kits können gleichzeitig geöffnet sein. Dasselbe Kit öffnet sich dagegen nur einmal. Die VM hat jeweils nur ein Kit: Docker Sandboxes bedient eines pro Benutzer, das andere arbeitet „Ohne VM“.",
            )}
          </P>
        </QA>
        <QA
          q={tr(
            "Funziona su un disco di rete?",
            "Does it work on a network drive?",
            "Est-ce que ça fonctionne sur un disque réseau ?",
            "Funktioniert es auf einem Netzlaufwerk?",
          )}
        >
          <P>
            {tr(
              "Sì, ma è più lento: i programmi leggono molti piccoli file. Con «Temporanei dei programmi» su «Automatico», i file temporanei vanno comunque sul disco del PC.",
              "Yes, but it is slower: the programs read many small files. With “Temporary files of programs” set to “Automatic”, temporary files go to the PC's disk anyway.",
              "Oui, mais c'est plus lent : les programmes lisent beaucoup de petits fichiers. Avec « Fichiers temporaires des programmes » sur « Automatique », les fichiers temporaires vont de toute façon sur le disque du PC.",
              "Ja, aber es ist langsamer: Die Programme lesen viele kleine Dateien. Mit „Temporäre Dateien der Programme“ auf „Automatisch“ landen die temporären Dateien trotzdem auf der Festplatte des PCs.",
            )}
          </P>
        </QA>
        <QA
          q={tr(
            "Quanto spazio serve?",
            "How much space does it need?",
            "Combien d'espace faut-il ?",
            "Wie viel Speicherplatz wird gebraucht?",
          )}
        >
          <P>
            {tr(
              "La scelta iniziale occupa circa 2,3 GB. Ogni linguaggio aggiunge da qualche decina a qualche centinaio di MB (l'Android SDK e TinyTeX di più), ogni versione in più altrettanto. I progetti con le loro dipendenze (node_modules, vendor) crescono nel tempo. La pagina Strumenti mostra la dimensione di ogni componente.",
              "The initial selection takes about 2.3 GB. Each language adds from a few tens to a few hundred MB (the Android SDK and TinyTeX more), each extra version as much again. Projects with their dependencies (node_modules, vendor) grow over time. The Tools page shows the size of each component.",
              "La sélection initiale occupe environ 2,3 Go. Chaque langage ajoute de quelques dizaines à quelques centaines de Mo (l'Android SDK et TinyTeX davantage), chaque version supplémentaire autant. Les projets avec leurs dépendances (node_modules, vendor) grossissent avec le temps. La page Outils affiche la taille de chaque composant.",
              "Die anfängliche Auswahl belegt etwa 2,3 GB. Jede Sprache fügt einige zehn bis einige hundert MB hinzu (das Android SDK und TinyTeX mehr), jede weitere Version ebenso viel. Projekte mit ihren Abhängigkeiten (node_modules, vendor) wachsen mit der Zeit. Die Seite Werkzeuge zeigt die Größe jeder Komponente.",
            )}
          </P>
        </QA>
        <QA
          q={tr(
            "Gli strumenti si aggiornano da soli?",
            "Do the tools update themselves?",
            "Les outils se mettent-ils à jour tout seuls ?",
            "Aktualisieren sich die Werkzeuge selbst?",
          )}
        >
          <P>
            {rich(
              tr(
                "Atollo sì (se lasci attivo l'aggiornamento automatico); i componenti no, li aggiorni tu dalla pagina Strumenti. Vedi {0}.",
                "Atollo does (if you leave automatic update on); the components do not, you update them from the Tools page. See {0}.",
                "Atollo oui (si vous laissez la mise à jour automatique activée) ; les composants non, c'est vous qui les mettez à jour depuis la page Outils. Voyez {0}.",
                "Atollo ja (wenn du das automatische Update aktiviert lässt); die Komponenten nicht, die aktualisierst du selbst auf der Seite Werkzeuge. Siehe {0}.",
              ),
              <See key="u" id="updates" />,
            )}
          </P>
        </QA>
        <QA
          q={tr(
            "Perché la pagina AI non apre gli agenti?",
            "Why doesn't the AI page open the agents?",
            "Pourquoi la page IA n'ouvre-t-elle pas les agents ?",
            "Warum öffnet die Seite KI die Agenten nicht?",
          )}
        >
          <P>
            {tr(
              "Perché un agente lavora sempre in un progetto: si apre dalla pagina Progetti, con il pulsante che porta il logo dell'agente. La pagina AI, riquadro Agenti, serve per accesso, limiti e ciò che gli agenti condividono; si installano da Strumenti.",
              "Because an agent always works in a project: you open it from the Projects page, with the button that shows the agent's logo. The AI page, Agents pane, is for sign-in, limits and what the agents share; you install them from Tools.",
              "Parce qu'un agent travaille toujours dans un projet : il s'ouvre depuis la page Projets, avec le bouton qui porte le logo de l'agent. La page IA, volet Agents, sert pour la connexion, les limites et ce que les agents partagent ; ils s'installent depuis Outils.",
              "Weil ein Agent immer in einem Projekt arbeitet: Er wird auf der Seite Projekte mit der Schaltfläche geöffnet, die das Logo des Agenten trägt. Die Seite KI, Bereich Agenten, dient für Anmeldung, Limits und das, was die Agenten teilen; installiert werden sie über Werkzeuge.",
            )}
          </P>
        </QA>
        <Callout tone="tip">
          {rich(
            tr(
              "Non trovi la risposta? Prova {0} o la casella di ricerca della guida.",
              "Can't find the answer? Try {0} or the guide's search box.",
              "Vous ne trouvez pas la réponse ? Essayez {0} ou le champ de recherche du guide.",
              "Keine Antwort gefunden? Versuch {0} oder das Suchfeld der Anleitung.",
            ),
            <See key="p" id="troubleshooting" />,
          )}
        </Callout>
      </Section>
      <Related ids={["troubleshooting", "glossary", "welcome"]} />
    </>
  ),

  glossary: () => (
    <>
      <InBrief
        points={[
          tr(
            "Le parole che tornano nella guida e nell'app, spiegate senza dare niente per scontato.",
            "The words that come up in the guide and the app, explained without taking anything for granted.",
            "Les mots qui reviennent dans le guide et dans l'application, expliqués sans rien tenir pour acquis.",
            "Die Wörter, die in der Anleitung und in der App vorkommen, erklärt, ohne etwas vorauszusetzen.",
          ),
        ]}
      />
      <Section title={tr("Dalla A alla Z", "From A to Z", "De A à Z", "Von A bis Z")}>
        <Terms items={glossary().sort((a, b) => a[0].localeCompare(b[0]))} />
      </Section>
      <Related ids={["welcome", "faq"]} />
    </>
  ),
};
