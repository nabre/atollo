// Chapter 6 of the Guide, «Account e impostazioni»: the GitHub account, the
// Settings page section by section, the network, the framework recipes.
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
  P,
  Related,
  Section,
  See,
  Steps,
  Table,
  Under,
  type TopicTexts,
} from "../widgets";

export const SETTINGS: TopicTexts = {
  github: () => (
    <>
      <InBrief
        points={[
          tr(
            "Un solo accesso a GitHub vale per tutto il kit: la pagina Progetti, Git (anche nell'editor, nelle console e negli agenti) e GitHub CLI.",
            "One GitHub sign-in works for the whole kit: the Projects page, Git (in the editor, the consoles and the agents too) and GitHub CLI.",
            "Une seule connexion à GitHub vaut pour tout le kit : la page Projets, Git (y compris dans l'éditeur, les consoles et les agents) et GitHub CLI.",
            "Eine einzige GitHub-Anmeldung gilt für das ganze Kit: die Seite Projekte, Git (auch im Editor, in den Konsolen und in den Agenten) und GitHub CLI.",
          ),
          tr(
            "Si fa in Impostazioni › «Account GitHub» con «Accedi a GitHub» e un codice da scrivere nel browser.",
            "You do it in Settings › “GitHub account” with “Sign in to GitHub” and a code to type in the browser.",
            "Elle se fait dans Paramètres › « Compte GitHub » avec « Se connecter à GitHub » et un code à saisir dans le navigateur.",
            "Sie erfolgt unter Einstellungen › „GitHub-Konto“ mit „Bei GitHub anmelden“ und einem Code, den Sie im Browser eingeben.",
          ),
          tr(
            "Senza account il kit funziona lo stesso: cloni i repository pubblici e lavori in locale.",
            "Without an account the kit works all the same: you clone public repositories and work locally.",
            "Sans compte, le kit fonctionne quand même : vous clonez les dépôts publics et travaillez en local.",
            "Ohne Konto funktioniert das Kit trotzdem: Sie klonen öffentliche Repositorys und arbeiten lokal.",
          ),
          tr(
            "L'accesso resta nel kit: su un altro PC sei già collegato.",
            "The sign-in stays in the kit: on another PC you are already signed in.",
            "La connexion reste dans le kit : sur un autre PC, vous êtes déjà connecté.",
            "Die Anmeldung bleibt im Kit: Auf einem anderen PC sind Sie bereits angemeldet.",
          ),
        ]}
      />

      <Section title={tr("Che cosa sblocca", "What it unlocks", "Ce qu'elle débloque", "Was sie freischaltet")}>
        <Table
          columns={[
            tr("Con l'account", "With the account", "Avec le compte", "Mit dem Konto"),
            tr("Anche senza", "Even without", "Même sans", "Auch ohne"),
          ]}
          rows={[
            [
              tr(
                "L'elenco dei tuoi repository in Progetti, con clona, rinomina ed elimina",
                "The list of your repositories in Projects, with clone, rename and delete",
                "La liste de vos dépôts dans Projets, avec cloner, renommer et supprimer",
                "Die Liste Ihrer Repositorys unter Projekte, mit Klonen, Umbenennen und Löschen",
              ),
              tr(
                "Clonare repository pubblici con «Da un indirizzo Git»",
                "Cloning public repositories with “From a Git address”",
                "Cloner des dépôts publics avec « Depuis une adresse Git »",
                "Öffentliche Repositorys mit „Von einer Git-Adresse“ klonen",
              ),
            ],
            [
              tr(
                "Clonare repository privati",
                "Cloning private repositories",
                "Cloner des dépôts privés",
                "Private Repositorys klonen",
              ),
              tr(
                "Creare progetti, avviarli, database, siti, agenti, strumenti",
                "Creating projects, starting them, databases, sites, agents, tools",
                "Créer des projets, les lancer, bases de données, sites, agents, outils",
                "Projekte erstellen und starten, Datenbanken, Websites, Agenten, Werkzeuge",
              ),
            ],
            [
              tr(
                "git push e git pull verso GitHub senza password",
                "git push and git pull to GitHub without a password",
                "git push et git pull vers GitHub sans mot de passe",
                "git push und git pull zu GitHub ohne Passwort",
              ),
              tr(
                "Usare Git in locale (commit, rami)",
                "Using Git locally (commits, branches)",
                "Utiliser Git en local (commits, branches)",
                "Git lokal verwenden (Commits, Branches)",
              ),
            ],
            [
              tr(
                "I comandi gh nelle console, nell'editor e negli agenti",
                "The gh commands in the consoles, the editor and the agents",
                "Les commandes gh dans les consoles, l'éditeur et les agents",
                "Die gh-Befehle in den Konsolen, im Editor und in den Agenten",
              ),
              tr("—", "—", "—", "—"),
            ],
            [
              tr(
                "Più richieste all'API di GitHub per i download (5.000 l'ora invece di 60)",
                "More requests to the GitHub API for downloads (5,000 an hour instead of 60)",
                "Plus de requêtes à l'API de GitHub pour les téléchargements (5 000 par heure au lieu de 60)",
                "Mehr Anfragen an die GitHub-API für Downloads (5.000 pro Stunde statt 60)",
              ),
              tr(
                "Scaricare i componenti (con il limite più basso e qualche ripiego)",
                "Downloading the components (with the lower limit and a few fallbacks)",
                "Télécharger les composants (avec la limite plus basse et quelques solutions de repli)",
                "Komponenten herunterladen (mit dem niedrigeren Limit und einigen Ausweichlösungen)",
              ),
            ],
          ]}
        />
      </Section>

      <Section title={tr("Accedere", "Signing in", "Se connecter", "Anmelden")}>
        <Steps
          steps={[
            tr(
              "Apri Impostazioni › «Account GitHub».",
              "Open Settings › “GitHub account”.",
              "Ouvrez Paramètres › « Compte GitHub ».",
              "Öffnen Sie Einstellungen › „GitHub-Konto“.",
            ),
            tr(
              "Premi «Accedi a GitHub». Dopo «Accesso in preparazione…» compare un codice di otto caratteri (XXXX-XXXX) e si apre github.com nel browser.",
              "Press “Sign in to GitHub”. After “Preparing the sign-in…” an eight-character code appears (XXXX-XXXX) and github.com opens in the browser.",
              "Appuyez sur « Se connecter à GitHub ». Après « Préparation de la connexion… », un code de huit caractères apparaît (XXXX-XXXX) et github.com s'ouvre dans le navigateur.",
              "Klicken Sie auf „Bei GitHub anmelden“. Nach „Anmeldung wird vorbereitet…“ erscheint ein achtstelliger Code (XXXX-XXXX) und github.com öffnet sich im Browser.",
            ),
            tr(
              "Premi «Copia il codice», incollalo nella pagina di GitHub e autorizza GitHub CLI. Se la pagina non si è aperta, «Apri la pagina di GitHub».",
              "Press “Copy the code”, paste it into the GitHub page and authorize GitHub CLI. If the page did not open, use “Open the GitHub page”.",
              "Appuyez sur « Copier le code », collez-le dans la page de GitHub et autorisez GitHub CLI. Si la page ne s'est pas ouverte, utilisez « Ouvrir la page de GitHub ».",
              "Klicken Sie auf „Code kopieren“, fügen Sie ihn auf der GitHub-Seite ein und autorisieren Sie GitHub CLI. Wenn sich die Seite nicht geöffnet hat, verwenden Sie „GitHub-Seite öffnen“.",
            ),
            tr(
              "Torna su Atollo: compare «Accesso fatto come @tuo-nome».",
              "Go back to Atollo: “Signed in as @your-name” appears.",
              "Revenez sur Atollo : « Connecté en tant que @votre-nom » apparaît.",
              "Kehren Sie zu Atollo zurück: „Angemeldet als @ihr-name“ erscheint.",
            ),
          ]}
        />
        <P>
          {tr(
            "«Esci» toglie l'accesso dal kit dopo la conferma «Uscire da GitHub? Anche Git, l'editor e gli agenti del kit perdono l'accesso.»",
            "“Sign out” removes the sign-in from the kit after the confirmation “Sign out of GitHub? Git, the editor and the kit's agents lose the access too.”",
            "« Se déconnecter » retire la connexion du kit après la confirmation « Se déconnecter de GitHub ? Git, l'éditeur et les agents du kit perdent aussi l'accès. »",
            "„Abmelden“ entfernt die Anmeldung aus dem Kit nach der Bestätigung „Von GitHub abmelden? Auch Git, der Editor und die Agenten des Kits verlieren den Zugang.“",
          )}
        </P>
        <Callout tone="warn">
          {tr(
            "Il token di GitHub è salvato nel kit, in chiaro, perché ti segua su ogni PC. «Esci» non revoca la sessione su github.com: se perdi la chiavetta, revoca l'autorizzazione di GitHub CLI dalle impostazioni del tuo account GitHub (Applications).",
            "The GitHub token is saved in the kit, in plain text, so that it follows you to every PC. “Sign out” does not revoke the session on github.com: if you lose the USB stick, revoke the GitHub CLI authorization from your GitHub account settings (Applications).",
            "Le jeton GitHub est enregistré dans le kit, en clair, pour vous suivre sur chaque PC. « Se déconnecter » ne révoque pas la session sur github.com : si vous perdez la clé USB, révoquez l'autorisation de GitHub CLI depuis les paramètres de votre compte GitHub (Applications).",
            "Das GitHub-Token ist im Kit gespeichert, im Klartext, damit es Ihnen auf jeden PC folgt. „Abmelden“ widerruft die Sitzung auf github.com nicht: Wenn Sie den USB-Stick verlieren, widerrufen Sie die Autorisierung von GitHub CLI in den Einstellungen Ihres GitHub-Kontos (Applications).",
          )}
        </Callout>
        <Go page="settings">
          {tr("Apri le Impostazioni", "Open Settings", "Ouvrir les Paramètres", "Einstellungen öffnen")}
        </Go>
      </Section>

      <Section
        title={tr(
          "Il permesso di eliminare i repository",
          "The permission to delete repositories",
          "L'autorisation de supprimer les dépôts",
          "Die Berechtigung, Repositorys zu löschen",
        )}
      >
        <P>
          {tr(
            "L'accesso normale non comprende il permesso di eliminare repository. La prima volta che premi «Elimina da GitHub» su un tuo repository, il riquadro dice che manca il permesso e mostra «Concedi il permesso»: il kit rifà l'accesso chiedendo anche quello. Il codice da scrivere compare in Impostazioni › «Account GitHub». Poi premi di nuovo «Elimina da GitHub».",
            "The normal sign-in does not include the permission to delete repositories. The first time you press “Delete from GitHub” on one of your repositories, the panel says the permission is missing and shows “Grant the permission”: the kit signs in again, asking for that too. The code to type appears in Settings › “GitHub account”. Then press “Delete from GitHub” again.",
            "La connexion normale ne comprend pas l'autorisation de supprimer des dépôts. La première fois que vous appuyez sur « Supprimer de GitHub » sur l'un de vos dépôts, le panneau indique que l'autorisation manque et affiche « Accorder l'autorisation » : le kit refait la connexion en la demandant aussi. Le code à saisir apparaît dans Paramètres › « Compte GitHub ». Appuyez ensuite de nouveau sur « Supprimer de GitHub ».",
            "Die normale Anmeldung umfasst nicht die Berechtigung, Repositorys zu löschen. Wenn Sie zum ersten Mal bei einem Ihrer Repositorys auf „Von GitHub löschen“ klicken, meldet der Bereich, dass die Berechtigung fehlt, und zeigt „Berechtigung erteilen“: Das Kit meldet sich erneut an und fragt dabei auch danach. Der einzugebende Code erscheint unter Einstellungen › „GitHub-Konto“. Klicken Sie dann erneut auf „Von GitHub löschen“.",
          )}
        </P>
        <Callout tone="warn">
          {tr(
            "Eliminare un repository da GitHub non si può annullare: spariscono anche issue, pull request, wiki e release. La copia nel kit, se c'è, resta.",
            "Deleting a repository from GitHub cannot be undone: issues, pull requests, wiki and releases disappear too. The copy in the kit, if there is one, stays.",
            "La suppression d'un dépôt sur GitHub est irréversible : les issues, pull requests, wiki et releases disparaissent aussi. La copie dans le kit, s'il y en a une, reste.",
            "Das Löschen eines Repositorys von GitHub lässt sich nicht rückgängig machen: Auch Issues, Pull Requests, Wiki und Releases verschwinden. Die Kopie im Kit bleibt, falls vorhanden.",
          )}
        </Callout>
      </Section>

      <Section
        title={tr(
          "Il tuo nome nei commit",
          "Your name in commits",
          "Votre nom dans les commits",
          "Ihr Name in den Commits",
        )}
      >
        <P>
          {tr(
            "L'accesso serve per leggere e scrivere su GitHub, non dice a Git chi firma i commit. Imposta nome ed email una volta, in un terminale del kit:",
            "The sign-in is for reading and writing on GitHub; it does not tell Git who signs the commits. Set your name and email once, in a kit terminal:",
            "La connexion sert à lire et écrire sur GitHub ; elle ne dit pas à Git qui signe les commits. Définissez votre nom et votre e-mail une fois, dans un terminal du kit :",
            "Die Anmeldung dient zum Lesen und Schreiben auf GitHub; sie sagt Git nicht, wer die Commits signiert. Legen Sie Name und E-Mail einmal fest, in einem Terminal des Kits:",
          )}
        </P>
        <GuideCode
          title={tr(
            "Nome ed email per i commit",
            "Name and email for commits",
            "Nom et e-mail pour les commits",
            "Name und E-Mail für die Commits",
          )}
          code={'git config --global user.name "Nome Cognome"\ngit config --global user.email "tu@esempio.it"'}
        />
        <P>
          {tr(
            "Per non pubblicare la tua email puoi usare l'indirizzo noreply che GitHub ti assegna (lo trovi nelle impostazioni Email del tuo account).",
            "To keep your email private you can use the noreply address GitHub gives you (you find it in the Email settings of your account).",
            "Pour ne pas publier votre e-mail, vous pouvez utiliser l'adresse noreply que GitHub vous attribue (vous la trouvez dans les paramètres Email de votre compte).",
            "Um Ihre E-Mail nicht zu veröffentlichen, können Sie die noreply-Adresse verwenden, die GitHub Ihnen zuweist (Sie finden sie in den Email-Einstellungen Ihres Kontos).",
          )}
        </P>
      </Section>

      <Under>
        <Bullets
          items={[
            tr(
              "L'accesso esegue gh auth login --web --insecure-storage --scopes workflow: oltre ai permessi standard di GitHub CLI (repo, read:org, gist) chiede workflow; delete_repo solo con «Concedi il permesso».",
              "The sign-in runs gh auth login --web --insecure-storage --scopes workflow: besides the standard GitHub CLI permissions (repo, read:org, gist) it asks for workflow; delete_repo only with “Grant the permission”.",
              "La connexion exécute gh auth login --web --insecure-storage --scopes workflow : en plus des autorisations standard de GitHub CLI (repo, read:org, gist), elle demande workflow ; delete_repo seulement avec « Accorder l'autorisation ».",
              "Die Anmeldung führt gh auth login --web --insecure-storage --scopes workflow aus: Zusätzlich zu den Standardberechtigungen von GitHub CLI (repo, read:org, gist) fragt sie nach workflow; delete_repo nur mit „Berechtigung erteilen“.",
            ),
            tr(
              "Il token è in data\\gh\\hosts.yml (GH_CONFIG_DIR punta lì), mai nella Gestione credenziali di Windows.",
              "The token is in data\\gh\\hosts.yml (GH_CONFIG_DIR points there), never in the Windows Credential Manager.",
              "Le jeton est dans data\\gh\\hosts.yml (GH_CONFIG_DIR pointe là), jamais dans le Gestionnaire d'identification de Windows.",
              "Das Token liegt in data\\gh\\hosts.yml (GH_CONFIG_DIR zeigt dorthin), nie in der Windows-Anmeldeinformationsverwaltung.",
            ),
            tr(
              "Git usa lo stesso account attraverso un credential helper (gh auth git-credential) scritto nel file data\\home\\.gitconfig per github.com e gist.github.com, senza percorsi assoluti: sopravvive allo spostamento del kit. A ogni avvio il kit lo allinea allo stato di gh; un helper scritto da te per quegli indirizzi resta com'è.",
              "Git uses the same account through a credential helper (gh auth git-credential) written in the file data\\home\\.gitconfig for github.com and gist.github.com, without absolute paths: it survives moving the kit. At every start the kit aligns it with the state of gh; a helper you wrote yourself for those addresses stays as it is.",
              "Git utilise le même compte grâce à un credential helper (gh auth git-credential) écrit dans le fichier data\\home\\.gitconfig pour github.com et gist.github.com, sans chemins absolus : il survit au déplacement du kit. À chaque démarrage, le kit l'aligne sur l'état de gh ; un helper que vous avez écrit vous-même pour ces adresses reste tel quel.",
              "Git verwendet dasselbe Konto über einen Credential Helper (gh auth git-credential), der in die Datei data\\home\\.gitconfig für github.com und gist.github.com geschrieben wird, ohne absolute Pfade: Er übersteht das Verschieben des Kits. Bei jedem Start gleicht das Kit ihn an den Zustand von gh an; ein Helper, den Sie selbst für diese Adressen geschrieben haben, bleibt unverändert.",
            ),
            tr(
              "Per clonare un repository privato il token viene passato a Git solo per quella operazione, mai sulla riga di comando né nel file .git\\config.",
              "To clone a private repository the token is passed to Git only for that operation, never on the command line nor in the .git\\config file.",
              "Pour cloner un dépôt privé, le jeton est transmis à Git uniquement pour cette opération, jamais sur la ligne de commande ni dans le fichier .git\\config.",
              "Zum Klonen eines privaten Repositorys wird das Token nur für diesen Vorgang an Git übergeben, nie auf der Befehlszeile und nie in der Datei .git\\config.",
            ),
            tr(
              "Per i download il kit usa prima GITHUB_TOKEN o GH_TOKEN del PC, se ci sono, poi l'account del kit. Le risposte dell'API sono conservate in data\\cache\\github.",
              "For downloads the kit first uses the PC's GITHUB_TOKEN or GH_TOKEN, if present, then the kit's account. The API responses are kept in data\\cache\\github.",
              "Pour les téléchargements, le kit utilise d'abord GITHUB_TOKEN ou GH_TOKEN du PC, s'ils existent, puis le compte du kit. Les réponses de l'API sont conservées dans data\\cache\\github.",
              "Für Downloads verwendet das Kit zuerst GITHUB_TOKEN oder GH_TOKEN des PCs, falls vorhanden, dann das Konto des Kits. Die Antworten der API werden in data\\cache\\github aufbewahrt.",
            ),
            tr(
              "Il menu «Account» dell'editor non vede l'account del kit; push e pull nell'editor funzionano comunque, attraverso Git.",
              "The editor's “Accounts” menu does not see the kit's account; push and pull in the editor work anyway, through Git.",
              "Le menu « Comptes » de l'éditeur ne voit pas le compte du kit ; push et pull dans l'éditeur fonctionnent quand même, via Git.",
              "Das Menü „Konten“ des Editors sieht das Konto des Kits nicht; Push und Pull im Editor funktionieren trotzdem, über Git.",
            ),
          ]}
        />
      </Under>

      <Related ids={["project-new", "project-git", "sources"]} />
    </>
  ),

  settings: ({ info }) => (
    <>
      <InBrief
        points={[
          tr(
            "Le Impostazioni hanno sette sezioni nella barra laterale e, sotto, le istruzioni dei framework.",
            "Settings has seven sections in the sidebar and, below them, the framework instructions.",
            "Les Paramètres ont sept sections dans la barre latérale et, en dessous, les instructions des frameworks.",
            "Die Einstellungen haben sieben Abschnitte in der Seitenleiste und darunter die Anweisungen der Frameworks.",
          ),
          tr(
            "Le modifiche si salvano con «Salva» sulla barra delle schede.",
            "Changes are saved with “Save” on the tab bar.",
            "Les modifications s'enregistrent avec « Enregistrer » dans la barre des onglets.",
            "Änderungen werden mit „Speichern“ in der Registerkartenleiste gespeichert.",
          ),
          tr(
            "Valgono subito per l'app, le console e gli agenti; i terminali dell'editor dopo «Riavvia l'editor».",
            "They apply at once to the app, the consoles and the agents; to the editor's terminals after “Restart the editor”.",
            "Elles s'appliquent tout de suite à l'application, aux consoles et aux agents ; aux terminaux de l'éditeur après « Redémarrer l'éditeur ».",
            "Sie gelten sofort für die App, die Konsolen und die Agenten; für die Terminals des Editors nach „Editor neu starten“.",
          ),
        ]}
      />

      <GuideShot
        src="settings.webp"
        title={tr("La pagina Impostazioni", "The Settings page", "La page Paramètres", "Die Seite Einstellungen")}
        caption={tr(
          "Le sezioni a sinistra, sotto le istruzioni dei framework; «Salva» sulla barra delle schede.",
          "The sections on the left, the framework instructions below them; “Save” on the tab bar.",
          "Les sections à gauche, les instructions des frameworks en dessous ; « Enregistrer » sur la barre des onglets.",
          "Die Bereiche links, darunter die Framework-Anleitungen; „Speichern“ in der Registerleiste.",
        )}
      />

      <Section title={tr("Generale", "General", "Général", "Allgemein")}>
        <Table
          columns={[
            tr("Impostazione", "Setting", "Paramètre", "Einstellung"),
            tr("Che cosa fa", "What it does", "Ce qu'il fait", "Was sie bewirkt"),
          ]}
          rows={[
            [
              tr("Lingua", "Language", "Langue", "Sprache"),
              tr(
                "La lingua di Atollo: quella del sistema («Lingua di Windows»), italiano, inglese, francese o tedesco. Cambia subito.",
                "The language of Atollo: the system's (“Windows language”), Italian, English, French or German. It changes at once.",
                "La langue d'Atollo : celle du système (« Langue de Windows »), italien, anglais, français ou allemand. Elle change tout de suite.",
                "Die Sprache von Atollo: die des Systems („Windows-Sprache“), Italienisch, Englisch, Französisch oder Deutsch. Sie ändert sich sofort.",
              ),
            ],
            [
              tr(
                "Temporanei dei programmi",
                "Temporary files of programs",
                "Fichiers temporaires des programmes",
                "Temporäre Dateien der Programme",
              ),
              tr(
                "Dove i programmi del kit mettono i loro file temporanei. «Automatico» usa la cartella temporanea del sistema se il kit è su una chiavetta, un disco di rete o un disco FAT32/exFAT, altrimenti data\\tmp. «Nel kit» sempre data\\tmp; «Nel sistema» sempre la cartella del sistema (i file restano sul PC finché chiudi l'app).",
                "Where the kit's programs put their temporary files. “Automatic” uses the system's temporary folder if the kit is on a USB stick, a network drive or a FAT32/exFAT drive, otherwise data\\tmp. “In the kit” always data\\tmp; “In the system” always the system's folder (the files stay on the PC until you close the app).",
                "L'endroit où les programmes du kit mettent leurs fichiers temporaires. « Automatique » utilise le dossier temporaire du système si le kit est sur une clé USB, un disque réseau ou un disque FAT32/exFAT, sinon data\\tmp. « Dans le kit » toujours data\\tmp ; « Dans le système » toujours le dossier du système (les fichiers restent sur le PC jusqu'à la fermeture de l'application).",
                "Wo die Programme des Kits ihre temporären Dateien ablegen. „Automatisch“ verwendet den temporären Ordner des Systems, wenn das Kit auf einem USB-Stick, einem Netzlaufwerk oder einem FAT32/exFAT-Laufwerk liegt, sonst data\\tmp. „Im Kit“ immer data\\tmp; „Im System“ immer den Ordner des Systems (die Dateien bleiben auf dem PC, bis Sie die App schließen).",
              ),
            ],
            [
              tr("Aggiornamento automatico", "Automatic update", "Mise à jour automatique", "Automatisches Update"),
              tr(
                "«Scarica da solo le nuove versioni di Atollo»: la nuova versione sostituisce quella attuale quando chiudi l'app. Attivo di base.",
                "“Download new versions of Atollo automatically”: the new version replaces the current one when you close the app. On by default.",
                "« Télécharger automatiquement les nouvelles versions d'Atollo » : la nouvelle version remplace l'actuelle quand vous fermez l'application. Activé par défaut.",
                "„Neue Versionen von Atollo automatisch herunterladen“: Die neue Version ersetzt die aktuelle, wenn Sie die App schließen. Standardmäßig aktiv.",
              ),
            ],
            [
              tr(
                "Pacchetti per reinstallare senza rete",
                "Packages to reinstall without the network",
                "Paquets pour réinstaller sans réseau",
                "Pakete zum Neuinstallieren ohne Netzwerk",
              ),
              tr(
                "Solo su macOS e Linux: conserva nel kit i pacchetti dei componenti che vanno reinstallati quando il kit cambia cartella (R, Ruby, MongoDB, SQLite, make), così non serve la rete. «Libera» li cancella.",
                "Only on macOS and Linux: keeps in the kit the packages of the components that must be reinstalled when the kit changes folder (R, Ruby, MongoDB, SQLite, make), so the network is not needed. “Free” deletes them.",
                "Uniquement sur macOS et Linux : conserve dans le kit les paquets des composants à réinstaller quand le kit change de dossier (R, Ruby, MongoDB, SQLite, make), ainsi le réseau n'est pas nécessaire. « Libérer » les supprime.",
                "Nur unter macOS und Linux: Bewahrt im Kit die Pakete der Komponenten auf, die neu installiert werden müssen, wenn das Kit den Ordner wechselt (R, Ruby, MongoDB, SQLite, make), sodass kein Netzwerk nötig ist. „Freigeben“ löscht sie.",
              ),
            ],
          ]}
        />
        <Callout
          tone="tip"
          title={tr(
            "Perché i temporanei fuori dal kit su una chiavetta",
            "Why temporary files go outside the kit on a USB stick",
            "Pourquoi les fichiers temporaires hors du kit sur une clé USB",
            "Warum temporäre Dateien auf einem USB-Stick außerhalb des Kits liegen",
          )}
        >
          {tr(
            "I programmi scrivono e cancellano continuamente file temporanei: su una chiavetta questo rallenta tutto e la consuma. Sul disco del PC vanno molto più veloci, e il kit li cancella quando chiudi l'app.",
            "Programs constantly write and delete temporary files: on a USB stick this slows everything down and wears it out. On the PC's disk they are much faster, and the kit deletes them when you close the app.",
            "Les programmes écrivent et effacent sans cesse des fichiers temporaires : sur une clé USB, cela ralentit tout et l'use. Sur le disque du PC, ils vont beaucoup plus vite, et le kit les supprime quand vous fermez l'application.",
            "Programme schreiben und löschen ständig temporäre Dateien: Auf einem USB-Stick verlangsamt das alles und nutzt ihn ab. Auf der Festplatte des PCs geht das viel schneller, und das Kit löscht sie, wenn Sie die App schließen.",
          )}
        </Callout>
      </Section>

      <Section title={tr("Rete", "Network", "Réseau", "Netzwerk")}>
        <P>
          {rich(
            tr(
              "Proxy, certificati aggiuntivi e rete locale: {0}.",
              "Proxy, additional certificates and local network: {0}.",
              "Proxy, certificats supplémentaires et réseau local : {0}.",
              "Proxy, zusätzliche Zertifikate und lokales Netzwerk: {0}.",
            ),
            <See key="n" id="network" />,
          )}
        </P>
      </Section>

      <Section title={tr("Agenti", "Agents", "Agents", "Agenten")}>
        <P>
          {tr(
            "«Chiave API di Gemini (Antigravity senza account Google)»: incolla la chiave e premi «Salva». Il campo poi mostra «salvata: scrivi per sostituirla» e la chiave non viene più mostrata; «Togli la chiave» la cancella al prossimo salvataggio. Antigravity mostrerà «accesso con chiave API». La chiave è salvata in chiaro in data\\app\\settings.json e passata ai programmi del kit come GEMINI_API_KEY.",
            "“Gemini API key (Antigravity without a Google account)”: paste the key and press “Save”. The field then shows “saved: type to replace it” and the key is no longer shown; “Remove the key” deletes it at the next save. Antigravity will show “signed in with an API key”. The key is saved in plain text in data\\app\\settings.json and passed to the kit's programs as GEMINI_API_KEY.",
            "« Clé API Gemini (Antigravity sans compte Google) » : collez la clé et appuyez sur « Enregistrer ». Le champ affiche ensuite « enregistrée : saisissez-en une autre pour la remplacer » et la clé n'est plus affichée ; « Supprimer la clé » l'efface au prochain enregistrement. Antigravity affichera « connecté avec une clé API ». La clé est enregistrée en clair dans data\\app\\settings.json et transmise aux programmes du kit comme GEMINI_API_KEY.",
            "„Gemini-API-Schlüssel (Antigravity ohne Google-Konto)“: Fügen Sie den Schlüssel ein und klicken Sie auf „Speichern“. Das Feld zeigt dann „gespeichert: zum Ersetzen neu eingeben“ und der Schlüssel wird nicht mehr angezeigt; „Schlüssel entfernen“ löscht ihn beim nächsten Speichern. Antigravity zeigt dann „mit API-Schlüssel angemeldet“. Der Schlüssel wird im Klartext in data\\app\\settings.json gespeichert und den Programmen des Kits als GEMINI_API_KEY übergeben.",
          )}
        </P>
      </Section>

      <Section title={tr("Versioni", "Versions", "Versions", "Versionen")}>
        <Table
          columns={[
            tr("Impostazione", "Setting", "Paramètre", "Einstellung"),
            tr("Che cosa fa", "What it does", "Ce qu'il fait", "Was sie bewirkt"),
          ]}
          rows={[
            [
              tr("Python", "Python", "Python", "Python"),
              tr(
                "La versione di Python da installare (per esempio 3.13). Vuoto: la più recente.",
                "The Python version to install (for example 3.13). Empty: the latest.",
                "La version de Python à installer (par exemple 3.13). Vide : la plus récente.",
                "Die zu installierende Python-Version (zum Beispiel 3.13). Leer: die neueste.",
              ),
            ],
            [
              tr("PHP", "PHP", "PHP", "PHP"),
              tr(
                "La versione di PHP da installare (per esempio 8.4). Vuoto: la più recente.",
                "The PHP version to install (for example 8.4). Empty: the latest.",
                "La version de PHP à installer (par exemple 8.4). Vide : la plus récente.",
                "Die zu installierende PHP-Version (zum Beispiel 8.4). Leer: die neueste.",
              ),
            ],
            [
              tr("Compilatore di Rust", "Rust compiler", "Compilateur Rust", "Rust-Compiler"),
              tr(
                "GNU (tutto nel kit, la scelta di base) oppure MSVC, che richiede Visual Studio Build Tools installato nel PC.",
                "GNU (all in the kit, the default choice) or MSVC, which requires Visual Studio Build Tools installed on the PC.",
                "GNU (tout dans le kit, le choix par défaut) ou MSVC, qui nécessite Visual Studio Build Tools installé sur le PC.",
                "GNU (alles im Kit, die Standardwahl) oder MSVC, das auf dem PC installierte Visual Studio Build Tools erfordert.",
              ),
            ],
          ]}
        />
        <Callout tone="info">
          {rich(
            tr(
              "Valgono dalla prossima installazione o aggiornamento in Strumenti. Non scelgono la versione che il kit usa (sempre la più recente su disco): per tenere più versioni vedi {0}.",
              "They apply from the next installation or update in Tools. They do not choose the version the kit uses (always the latest on disk): to keep several versions see {0}.",
              "Ils s'appliquent à partir de la prochaine installation ou mise à jour dans Outils. Ils ne choisissent pas la version que le kit utilise (toujours la plus récente sur le disque) : pour garder plusieurs versions, voyez {0}.",
              "Sie gelten ab der nächsten Installation oder Aktualisierung unter Werkzeuge. Sie wählen nicht die Version, die das Kit verwendet (immer die neueste auf dem Datenträger): Um mehrere Versionen zu behalten, siehe {0}.",
            ),
            <See key="v" id="tools-versions" />,
          )}
        </Callout>
      </Section>

      <Section title={tr("Aggiornamenti", "Updates", "Mises à jour", "Updates")}>
        <P>
          {rich(
            tr(
              "Lo stato degli aggiornamenti di Atollo e il pulsante «Cerca aggiornamenti»: {0}.",
              "The state of Atollo's updates and the “Check for updates” button: {0}.",
              "L'état des mises à jour d'Atollo et le bouton « Rechercher des mises à jour » : {0}.",
              "Der Stand der Updates von Atollo und die Schaltfläche „Nach Updates suchen“: {0}.",
            ),
            <See key="u" id="updates" />,
          )}
        </P>
      </Section>

      <Section title={tr("Account GitHub", "GitHub account", "Compte GitHub", "GitHub-Konto")}>
        <P>
          {rich(
            tr(
              "L'accesso a GitHub per tutto il kit: {0}.",
              "The GitHub sign-in for the whole kit: {0}.",
              "La connexion à GitHub pour tout le kit : {0}.",
              "Die GitHub-Anmeldung für das ganze Kit: {0}.",
            ),
            <See key="g" id="github" />,
          )}
        </P>
      </Section>

      <Section title={tr("Il kit", "The kit", "Le kit", "Das Kit")}>
        <P>
          {tr(
            "Informazioni in sola lettura: cartella del kit, cartella dei progetti, cartella dei dati e dei programmi, disco del kit (file system e tipo: fisso, rimovibile, di rete), cartella temporanea dei programmi, piattaforma, versione di WebView2 e di Atollo. Se il disco ha qualcosa da segnalare compare un avviso.",
            "Read-only information: kit folder, projects folder, data and programs folder, the kit's disk (file system and type: fixed, removable, network), programs' temporary folder, platform, WebView2 and Atollo versions. If the disk has something to report, a warning appears.",
            "Informations en lecture seule : dossier du kit, dossier des projets, dossier des données et des programmes, disque du kit (système de fichiers et type : fixe, amovible, réseau), dossier temporaire des programmes, plateforme, versions de WebView2 et d'Atollo. Si le disque a quelque chose à signaler, un avertissement apparaît.",
            "Schreibgeschützte Informationen: Kit-Ordner, Projektordner, Ordner für Daten und Programme, Laufwerk des Kits (Dateisystem und Typ: fest, wechselbar, Netzwerk), temporärer Ordner der Programme, Plattform, Version von WebView2 und von Atollo. Wenn es zum Laufwerk etwas zu melden gibt, erscheint ein Hinweis.",
          )}
        </P>
        {info && <Code>{info.root}</Code>}
      </Section>

      <Section title={tr("Framework", "Frameworks", "Frameworks", "Frameworks")}>
        <P>
          {rich(
            tr(
              "Le istruzioni di «Nuovo progetto»: {0}.",
              "The instructions of “New project”: {0}.",
              "Les instructions de « Nouveau projet » : {0}.",
              "Die Anweisungen von „Neues Projekt“: {0}.",
            ),
            <See key="f" id="frameworks" />,
          )}
        </P>
      </Section>

      <Under>
        <Bullets
          items={[
            tr(
              "Le impostazioni sono in data\\app\\settings.json. Ogni salvataggio controlla i valori, scrive il file in modo sicuro (prima una copia, poi la sostituzione) e ricostruisce l'ambiente del kit.",
              "The settings are in data\\app\\settings.json. Every save checks the values, writes the file safely (first a copy, then the replacement) and rebuilds the kit's environment.",
              "Les paramètres sont dans data\\app\\settings.json. Chaque enregistrement contrôle les valeurs, écrit le fichier de façon sûre (d'abord une copie, puis le remplacement) et reconstruit l'environnement du kit.",
              "Die Einstellungen liegen in data\\app\\settings.json. Jedes Speichern prüft die Werte, schreibt die Datei sicher (zuerst eine Kopie, dann das Ersetzen) und baut die Umgebung des Kits neu auf.",
            ),
            tr(
              "Se il file non si può leggere, l'app usa i valori di base e non lo sovrascrive: «Le impostazioni in … non sono leggibili: uso quelle predefinite e non modifico il file.»",
              "If the file cannot be read, the app uses the default values and does not overwrite it: “The settings in … cannot be read: using the defaults and leaving the file as it is.”",
              "Si le fichier ne peut pas être lu, l'application utilise les valeurs par défaut et ne l'écrase pas : « Les paramètres de … sont illisibles : les valeurs par défaut sont utilisées et le fichier n'est pas modifié. »",
              "Wenn die Datei nicht gelesen werden kann, verwendet die App die Standardwerte und überschreibt sie nicht: „Die Einstellungen in … sind nicht lesbar: Es werden die Standardwerte verwendet, die Datei bleibt unverändert.“",
            ),
            tr(
              "La casella di ricerca delle Impostazioni cerca anche tra le basi e le aggiunte dei framework.",
              "The Settings search box also searches the framework bases and additions.",
              "Le champ de recherche des Paramètres cherche aussi parmi les bases et les ajouts des frameworks.",
              "Das Suchfeld der Einstellungen durchsucht auch die Basen und Ergänzungen der Frameworks.",
            ),
          ]}
        />
      </Under>

      <Related ids={["network", "updates", "frameworks"]} />
    </>
  ),

  network: () => (
    <>
      <InBrief
        points={[
          tr(
            "Servono solo se la rete passa da un proxy o ispeziona il traffico HTTPS, come in molte aziende e scuole.",
            "They are needed only if the network goes through a proxy or inspects HTTPS traffic, as in many companies and schools.",
            "Ils ne servent que si le réseau passe par un proxy ou inspecte le trafic HTTPS, comme dans beaucoup d'entreprises et d'écoles.",
            "Sie werden nur gebraucht, wenn das Netzwerk über einen Proxy läuft oder den HTTPS-Verkehr untersucht, wie in vielen Firmen und Schulen.",
          ),
          tr(
            "Valgono per i download del kit, per gli agenti e per gli strumenti nelle console.",
            "They apply to the kit's downloads, to the agents and to the tools in the consoles.",
            "Ils s'appliquent aux téléchargements du kit, aux agents et aux outils dans les consoles.",
            "Sie gelten für die Downloads des Kits, für die Agenten und für die Werkzeuge in den Konsolen.",
          ),
          tr(
            "«Rete locale» fa vedere i siti dei progetti agli altri PC della rete.",
            "“Local network” lets the other PCs on the network see the projects' sites.",
            "« Réseau local » rend les sites des projets visibles aux autres PC du réseau.",
            "„Lokales Netzwerk“ macht die Websites der Projekte für die anderen PCs im Netzwerk sichtbar.",
          ),
        ]}
      />

      <Section
        title={tr(
          "Capire se ti servono",
          "Working out whether you need them",
          "Savoir si vous en avez besoin",
          "Herausfinden, ob Sie sie brauchen",
        )}
      >
        <P>
          {tr(
            "Se a casa tutto si scarica e in ufficio i download falliscono con errori di connessione o di certificato, probabilmente la rete aziendale usa un proxy, oppure ispeziona il traffico HTTPS con un suo certificato. I valori giusti te li dà l'assistenza informatica; spesso sono gli stessi impostati nel browser aziendale.",
            "If everything downloads at home but at the office downloads fail with connection or certificate errors, the company network probably uses a proxy, or inspects HTTPS traffic with its own certificate. Your IT support can give you the right values; often they are the same as those set in the company browser.",
            "Si tout se télécharge à la maison mais qu'au bureau les téléchargements échouent avec des erreurs de connexion ou de certificat, le réseau de l'entreprise utilise probablement un proxy, ou inspecte le trafic HTTPS avec son propre certificat. Le support informatique vous donnera les bonnes valeurs ; ce sont souvent les mêmes que celles du navigateur de l'entreprise.",
            "Wenn zu Hause alles heruntergeladen wird, im Büro die Downloads aber mit Verbindungs- oder Zertifikatsfehlern scheitern, verwendet das Firmennetzwerk wahrscheinlich einen Proxy oder untersucht den HTTPS-Verkehr mit einem eigenen Zertifikat. Die richtigen Werte erhalten Sie von der IT-Abteilung; oft sind es dieselben wie im Browser der Firma.",
          )}
        </P>
      </Section>

      <Section title={tr("I campi", "The fields", "Les champs", "Die Felder")}>
        <Table
          columns={[
            tr("Campo", "Field", "Champ", "Feld"),
            tr("Esempio", "Example", "Exemple", "Beispiel"),
            tr("Note", "Notes", "Remarques", "Hinweise"),
          ]}
          rows={[
            [
              tr("Proxy HTTPS", "HTTPS proxy", "Proxy HTTPS", "HTTPS-Proxy"),
              <Code key="a">http://proxy.azienda.it:8080</Code>,
              tr(
                "Il più importante: lo usano anche i download di Atollo.",
                "The most important one: Atollo's downloads use it too.",
                "Le plus important : les téléchargements d'Atollo l'utilisent aussi.",
                "Das wichtigste: Auch die Downloads von Atollo verwenden ihn.",
              ),
            ],
            [
              tr("Proxy HTTP", "HTTP proxy", "Proxy HTTP", "HTTP-Proxy"),
              <Code key="b">http://proxy.azienda.it:8080</Code>,
              tr(
                "Per le connessioni non cifrate.",
                "For unencrypted connections.",
                "Pour les connexions non chiffrées.",
                "Für unverschlüsselte Verbindungen.",
              ),
            ],
            [
              tr("Senza proxy per", "No proxy for", "Pas de proxy pour", "Kein Proxy für"),
              <Code key="c">localhost,127.0.0.1,.azienda.it</Code>,
              tr(
                "Indirizzi da raggiungere direttamente, separati da virgola.",
                "Addresses to reach directly, separated by commas.",
                "Adresses à joindre directement, séparées par des virgules.",
                "Adressen, die direkt erreicht werden sollen, durch Kommas getrennt.",
              ),
            ],
            [
              tr(
                "Certificati aggiuntivi (file PEM)",
                "Additional certificates (PEM file)",
                "Certificats supplémentaires (fichier PEM)",
                "Zusätzliche Zertifikate (PEM-Datei)",
              ),
              <Code key="d">E:\certificati\azienda.pem</Code>,
              tr(
                "Il certificato dell'azienda, per i programmi come Node.js che non usano quelli del sistema.",
                "The company's certificate, for programs like Node.js that do not use the system's ones.",
                "Le certificat de l'entreprise, pour les programmes comme Node.js qui n'utilisent pas ceux du système.",
                "Das Zertifikat der Firma, für Programme wie Node.js, die nicht die des Systems verwenden.",
              ),
            ],
          ]}
        />
        <Steps
          steps={[
            tr(
              "Apri Impostazioni › «Rete» e compila i campi che ti servono.",
              "Open Settings › “Network” and fill in the fields you need.",
              "Ouvrez Paramètres › « Réseau » et remplissez les champs dont vous avez besoin.",
              "Öffnen Sie Einstellungen › „Netzwerk“ und füllen Sie die Felder aus, die Sie brauchen.",
            ),
            tr(
              "Premi «Salva». Se l'editor è aperto, premi anche «Riavvia l'editor».",
              "Press “Save”. If the editor is open, also press “Restart the editor”.",
              "Appuyez sur « Enregistrer ». Si l'éditeur est ouvert, appuyez aussi sur « Redémarrer l'éditeur ».",
              "Klicken Sie auf „Speichern“. Wenn der Editor geöffnet ist, klicken Sie auch auf „Editor neu starten“.",
            ),
            tr(
              "Torna in Strumenti e riprova il download che non riusciva.",
              "Go back to Tools and retry the download that was failing.",
              "Revenez dans Outils et relancez le téléchargement qui échouait.",
              "Kehren Sie zu Werkzeuge zurück und versuchen Sie den fehlgeschlagenen Download erneut.",
            ),
          ]}
        />
        <Bullets
          items={[
            tr(
              "Il proxy deve iniziare con http:// o https:// e non avere spazi; se serve l'utente: http://utente:password@host:porta.",
              "The proxy must start with http:// or https:// and have no spaces; if a user is needed: http://user:password@host:port.",
              "Le proxy doit commencer par http:// ou https:// et ne pas contenir d'espaces ; si un utilisateur est nécessaire : http://utilisateur:motdepasse@hôte:port.",
              "Der Proxy muss mit http:// oder https:// beginnen und darf keine Leerzeichen enthalten; wenn ein Benutzer nötig ist: http://benutzer:passwort@host:port.",
            ),
            tr(
              "Il file PEM deve esistere: il kit controlla il percorso al salvataggio.",
              "The PEM file must exist: the kit checks the path when you save.",
              "Le fichier PEM doit exister : le kit contrôle le chemin à l'enregistrement.",
              "Die PEM-Datei muss existieren: Das Kit prüft den Pfad beim Speichern.",
            ),
          ]}
        />
      </Section>

      <Section title={tr("Rete locale", "Local network", "Réseau local", "Lokales Netzwerk")}>
        <P>
          {tr(
            "La casella «Permetti ai server dei progetti di farsi raggiungere dagli altri PC della rete» apre i siti dei progetti alla rete: utile per provare un sito dal telefono. Dagli altri dispositivi si usa l'indirizzo IP di questo PC con la porta del sito. Database e phpMyAdmin restano raggiungibili solo da questo PC.",
            "The “Let other PCs on the network reach the project servers” checkbox opens the projects' sites to the network: useful to try a site from your phone. From the other devices you use this PC's IP address with the site's port. Databases and phpMyAdmin stay reachable only from this PC.",
            "La case « Permettre aux autres PC du réseau d'accéder aux serveurs des projets » ouvre les sites des projets au réseau : pratique pour essayer un site depuis le téléphone. Depuis les autres appareils, on utilise l'adresse IP de ce PC avec le port du site. Les bases de données et phpMyAdmin restent accessibles uniquement depuis ce PC.",
            "Das Kästchen „Anderen PCs im Netzwerk den Zugriff auf die Projektserver erlauben“ öffnet die Websites der Projekte für das Netzwerk: praktisch, um eine Website auf dem Telefon auszuprobieren. Von den anderen Geräten aus verwenden Sie die IP-Adresse dieses PCs mit dem Port der Website. Datenbanken und phpMyAdmin bleiben nur von diesem PC aus erreichbar.",
          )}
        </P>
        <Callout tone="warn">
          {tr(
            "Chiunque sia sulla stessa rete potrà vedere i tuoi siti di prova. Attivala su reti di cui ti fidi e spegnila quando hai finito. Windows può chiedere il permesso del firewall la prima volta.",
            "Anyone on the same network will be able to see your test sites. Turn it on only on networks you trust and turn it off when you are done. Windows may ask for the firewall permission the first time.",
            "Toute personne sur le même réseau pourra voir vos sites de test. Activez-la sur des réseaux de confiance et désactivez-la quand vous avez fini. Windows peut demander l'autorisation du pare-feu la première fois.",
            "Jeder im selben Netzwerk kann Ihre Testseiten sehen. Aktivieren Sie die Option in Netzwerken, denen Sie vertrauen, und schalten Sie sie aus, wenn Sie fertig sind. Windows kann beim ersten Mal nach der Erlaubnis der Firewall fragen.",
          )}
        </Callout>
      </Section>

      <Under>
        <Bullets
          items={[
            tr(
              "I valori finiscono nelle variabili HTTPS_PROXY, HTTP_PROXY, NO_PROXY e NODE_EXTRA_CA_CERTS di tutti i programmi del kit.",
              "The values end up in the HTTPS_PROXY, HTTP_PROXY, NO_PROXY and NODE_EXTRA_CA_CERTS variables of all the kit's programs.",
              "Les valeurs vont dans les variables HTTPS_PROXY, HTTP_PROXY, NO_PROXY et NODE_EXTRA_CA_CERTS de tous les programmes du kit.",
              "Die Werte landen in den Variablen HTTPS_PROXY, HTTP_PROXY, NO_PROXY und NODE_EXTRA_CA_CERTS aller Programme des Kits.",
            ),
            tr(
              "I download di Atollo usano il «Proxy HTTPS» (o, se vuoto, le variabili del PC o il proxy di sistema) e i certificati del sistema: per questo l'ispezione HTTPS aziendale funziona già, se il certificato aziendale è installato nel PC. Il file PEM serve a Node.js e agli strumenti che leggono NODE_EXTRA_CA_CERTS.",
              "Atollo's downloads use the “HTTPS proxy” (or, if empty, the PC's variables or the system proxy) and the system's certificates: that is why company HTTPS inspection already works, if the company certificate is installed on the PC. The PEM file is for Node.js and the tools that read NODE_EXTRA_CA_CERTS.",
              "Les téléchargements d'Atollo utilisent le « Proxy HTTPS » (ou, s'il est vide, les variables du PC ou le proxy du système) et les certificats du système : c'est pourquoi l'inspection HTTPS de l'entreprise fonctionne déjà, si le certificat de l'entreprise est installé sur le PC. Le fichier PEM sert à Node.js et aux outils qui lisent NODE_EXTRA_CA_CERTS.",
              "Die Downloads von Atollo verwenden den „HTTPS-Proxy“ (oder, wenn leer, die Variablen des PCs oder den System-Proxy) und die Zertifikate des Systems: Deshalb funktioniert die HTTPS-Untersuchung der Firma bereits, wenn das Firmenzertifikat auf dem PC installiert ist. Die PEM-Datei ist für Node.js und die Werkzeuge, die NODE_EXTRA_CA_CERTS lesen.",
            ),
            tr(
              "I file PAC (configurazione automatica del proxy) non sono supportati: chiedi l'indirizzo del proxy.",
              "PAC files (automatic proxy configuration) are not supported: ask for the proxy address.",
              "Les fichiers PAC (configuration automatique du proxy) ne sont pas pris en charge : demandez l'adresse du proxy.",
              "PAC-Dateien (automatische Proxy-Konfiguration) werden nicht unterstützt: Fragen Sie nach der Adresse des Proxys.",
            ),
            tr(
              "«Rete locale» imposta ATOLLO_ALLOW_LAN=1: i comandi dei progetti ascoltano su tutte le interfacce, e così il server di file e i pool di Caddy e Apache.",
              "“Local network” sets ATOLLO_ALLOW_LAN=1: the projects' commands listen on all interfaces, and so do the file server and the Caddy and Apache pools.",
              "« Réseau local » définit ATOLLO_ALLOW_LAN=1 : les commandes des projets écoutent sur toutes les interfaces, tout comme le serveur de fichiers et les pools de Caddy et Apache.",
              "„Lokales Netzwerk“ setzt ATOLLO_ALLOW_LAN=1: Die Befehle der Projekte lauschen auf allen Schnittstellen, ebenso der Dateiserver und die Pools von Caddy und Apache.",
            ),
          ]}
        />
      </Under>

      <Related ids={["sources", "troubleshooting", "websites"]} />
    </>
  ),

  frameworks: () => (
    <>
      <InBrief
        points={[
          tr(
            "Le basi e le aggiunte di «Nuovo progetto» sono ricette: elenchi di comandi che puoi leggere e modificare.",
            "The bases and additions of “New project” are recipes: lists of commands you can read and change.",
            "Les bases et les ajouts de « Nouveau projet » sont des recettes : des listes de commandes que vous pouvez lire et modifier.",
            "Die Basen und Ergänzungen von „Neues Projekt“ sind Rezepte: Listen von Befehlen, die Sie lesen und ändern können.",
          ),
          tr(
            "Le trovi in Impostazioni, nel riquadro «Framework».",
            "You find them in Settings, in the “Frameworks” panel.",
            "Vous les trouvez dans Paramètres, dans le panneau « Frameworks ».",
            "Sie finden sie unter Einstellungen, im Bereich „Frameworks“.",
          ),
          tr(
            "Puoi cambiare una ricetta ufficiale (e ripristinarla) o crearne di tue.",
            "You can change an official recipe (and restore it) or create your own.",
            "Vous pouvez modifier une recette officielle (et la rétablir) ou créer les vôtres.",
            "Sie können ein offizielles Rezept ändern (und wiederherstellen) oder eigene erstellen.",
          ),
          tr(
            "«Prova una combinazione» mostra che cosa farebbe una base con delle aggiunte, prima di usarla.",
            "“Try a combination” shows what a base with some additions would do, before you use it.",
            "« Essayer une combinaison » montre ce que ferait une base avec des ajouts, avant de l'utiliser.",
            "„Kombination ausprobieren“ zeigt, was eine Basis mit Ergänzungen tun würde, bevor Sie sie verwenden.",
          ),
        ]}
      />

      <Section
        title={tr(
          "Com'è fatta una ricetta",
          "What a recipe is made of",
          "Comment est faite une recette",
          "Wie ein Rezept aufgebaut ist",
        )}
      >
        <Flow
          steps={[
            {
              title: tr("Strumenti", "Tools", "Outils", "Werkzeuge"),
              text: tr(
                "I componenti del kit che servono (Node.js, PHP…).",
                "The kit components it needs (Node.js, PHP…).",
                "Les composants du kit nécessaires (Node.js, PHP…).",
                "Die benötigten Komponenten des Kits (Node.js, PHP…).",
              ),
            },
            {
              title: tr("Crea", "Create", "Créer", "Erstellen"),
              text: tr(
                "I comandi che creano il progetto, eseguiti nella cartella projects.",
                "The commands that create the project, run in the projects folder.",
                "Les commandes qui créent le projet, exécutées dans le dossier projects.",
                "Die Befehle, die das Projekt erstellen, ausgeführt im Ordner projects.",
              ),
            },
            {
              title: tr("Poi", "Then", "Ensuite", "Danach"),
              text: tr(
                "I comandi da eseguire nella cartella del progetto.",
                "The commands to run in the project's folder.",
                "Les commandes à exécuter dans le dossier du projet.",
                "Die Befehle, die im Ordner des Projekts ausgeführt werden.",
              ),
            },
            {
              title: tr("Contorno", "Extras", "Compléments", "Drumherum"),
              text: tr(
                "Database, variabili .env, azioni del progetto.",
                "Database, .env variables, project actions.",
                "Base de données, variables .env, actions du projet.",
                "Datenbank, .env-Variablen, Aktionen des Projekts.",
              ),
            },
          ]}
        />
        <P>
          {tr(
            "Ogni riga è un comando eseguito con la PowerShell e gli strumenti del kit. {name} diventa il nome del progetto (tra virgolette), {package} com.example. seguito dal nome in minuscolo.",
            "Each line is a command run with PowerShell and the kit's tools. {name} becomes the project's name (in quotes), {package} com.example. followed by the name in lowercase.",
            "Chaque ligne est une commande exécutée avec PowerShell et les outils du kit. {name} devient le nom du projet (entre guillemets), {package} com.example. suivi du nom en minuscules.",
            "Jede Zeile ist ein Befehl, der mit PowerShell und den Werkzeugen des Kits ausgeführt wird. {name} wird zum Namen des Projekts (in Anführungszeichen), {package} zu com.example. gefolgt vom Namen in Kleinbuchstaben.",
          )}
        </P>
      </Section>

      <Section
        title={tr(
          "Il riquadro «Framework»",
          "The “Frameworks” panel",
          "Le panneau « Frameworks »",
          "Der Bereich „Frameworks“",
        )}
      >
        <P>
          {tr(
            "L'albero ha le «Basi», divise per gruppo, e sotto ogni base le aggiunte che vanno con lei; poi le «Aggiunte» per tipo; in fondo «Prova una combinazione». In cima, le icone «Aggiungi una base» e «Aggiungi un'aggiunta».",
            "The tree has the “Bases”, divided by group, and under each base the additions that go with it; then the “Additions” by type; at the bottom “Try a combination”. At the top, the “Add a base” and “Add an addition” icons.",
            "L'arbre contient les « Bases », réparties par groupe, et sous chaque base les ajouts qui vont avec elle ; puis les « Ajouts » par type ; tout en bas « Essayer une combinaison ». En haut, les icônes « Ajouter une base » et « Ajouter un ajout ».",
            "Der Baum enthält die „Basen“, nach Gruppen geordnet, und unter jeder Basis die Ergänzungen, die zu ihr passen; dann die „Ergänzungen“ nach Art; ganz unten „Kombination ausprobieren“. Oben die Symbole „Basis hinzufügen“ und „Ergänzung hinzufügen“.",
          )}
        </P>
        <Table
          columns={[
            tr("Campo", "Field", "Champ", "Feld"),
            tr("Che cosa scrivere", "What to write", "Quoi écrire", "Was eintragen"),
          ]}
          rows={[
            [
              tr(
                "Nome, Descrizione, Pagina ufficiale",
                "Name, Description, Official page",
                "Nom, Description, Page officielle",
                "Name, Beschreibung, Offizielle Seite",
              ),
              tr(
                "Come compare nella procedura e il collegamento alla documentazione.",
                "How it appears in the wizard and the link to the documentation.",
                "Comment elle apparaît dans l'assistant et le lien vers la documentation.",
                "Wie es im Assistenten erscheint und der Link zur Dokumentation.",
              ),
            ],
            [
              tr("Strumenti del kit", "Kit tools", "Outils du kit", "Werkzeuge des Kits"),
              tr(
                "I componenti da installare prima («Aggiungi uno strumento del kit»).",
                "The components to install first (“Add a kit tool”).",
                "Les composants à installer d'abord (« Ajouter un outil du kit »).",
                "Die Komponenten, die zuerst installiert werden („Werkzeug des Kits hinzufügen“).",
              ),
            ],
            [
              tr(
                "Crea il progetto (nella cartella projects)",
                "Create the project (in the projects folder)",
                "Créer le projet (dans le dossier projects)",
                "Projekt erstellen (im Ordner projects)",
              ),
              tr(
                "Per esempio: npm create vite@latest {name} -- --template vanilla-ts",
                "For example: npm create vite@latest {name} -- --template vanilla-ts",
                "Par exemple : npm create vite@latest {name} -- --template vanilla-ts",
                "Zum Beispiel: npm create vite@latest {name} -- --template vanilla-ts",
              ),
            ],
            [
              tr(
                "Poi, nella cartella del progetto",
                "Then, in the project's folder",
                "Ensuite, dans le dossier du projet",
                "Dann, im Ordner des Projekts",
              ),
              tr(
                "Per esempio: npm install",
                "For example: npm install",
                "Par exemple : npm install",
                "Zum Beispiel: npm install",
              ),
            ],
            [
              tr(
                "Database del kit per il progetto",
                "Kit database for the project",
                "Base de données du kit pour le projet",
                "Datenbank des Kits für das Projekt",
              ),
              tr(
                "Nessuno, MySQL o MongoDB.",
                "None, MySQL or MongoDB.",
                "Aucune, MySQL ou MongoDB.",
                "Keine, MySQL oder MongoDB.",
              ),
            ],
            [
              tr(
                "Variabili del file .env (CHIAVE=valore)",
                ".env file variables (KEY=value)",
                "Variables du fichier .env (CLÉ=valeur)",
                "Variablen der Datei .env (SCHLÜSSEL=Wert)",
              ),
              tr(
                "Per esempio: APP_NAME={name}",
                "For example: APP_NAME={name}",
                "Par exemple : APP_NAME={name}",
                "Zum Beispiel: APP_NAME={name}",
              ),
            ],
            [
              tr(
                "Azioni del progetto (Nome: comando)",
                "Project actions (Name: command)",
                "Actions du projet (Nom : commande)",
                "Aktionen des Projekts (Name: Befehl)",
              ),
              tr(
                "Per esempio: Avvia: npm run dev",
                "For example: Start: npm run dev",
                "Par exemple : Démarrer : npm run dev",
                "Zum Beispiel: Starten: npm run dev",
              ),
            ],
          ]}
        />
        <P>
          {tr(
            "Un'aggiunta ha in più il «Gruppo» (due aggiunte dello stesso gruppo non vanno insieme), «Usala anche con» per le basi compatibili e, per ogni base, i comandi da aggiungere o «Crea il progetto al posto della base».",
            "An addition also has the “Group” (two additions of the same group do not go together), “Use it also with” for the compatible bases and, for each base, the commands to add or “Create the project instead of the base”.",
            "Un ajout a en plus le « Groupe » (deux ajouts du même groupe ne vont pas ensemble), « L'utiliser aussi avec » pour les bases compatibles et, pour chaque base, les commandes à ajouter ou « Créer le projet à la place de la base ».",
            "Eine Ergänzung hat zusätzlich die „Gruppe“ (zwei Ergänzungen derselben Gruppe passen nicht zusammen), „Auch verwenden mit“ für die kompatiblen Basen und, für jede Basis, die hinzuzufügenden Befehle oder „Projekt statt der Basis erstellen“.",
          )}
        </P>
        <Steps
          steps={[
            tr(
              "Scegli una base o un'aggiunta nell'albero e modifica i campi.",
              "Choose a base or an addition in the tree and edit the fields.",
              "Choisissez une base ou un ajout dans l'arbre et modifiez les champs.",
              "Wählen Sie im Baum eine Basis oder eine Ergänzung und bearbeiten Sie die Felder.",
            ),
            tr(
              "Premi «Salva» sulla barra delle schede. Una ricetta ufficiale cambiata mostra «Modificata».",
              "Press “Save” on the tab bar. A changed official recipe shows “Modified”.",
              "Appuyez sur « Enregistrer » dans la barre des onglets. Une recette officielle modifiée affiche « Modifiée ».",
              "Klicken Sie in der Registerkartenleiste auf „Speichern“. Ein geändertes offizielles Rezept zeigt „Geändert“.",
            ),
            tr(
              "Per tornare alla versione originale usa «Ripristina le istruzioni ufficiali»; una ricetta tua si toglie con «Togli».",
              "To go back to the original version use “Restore the official instructions”; a recipe of your own is removed with “Remove”.",
              "Pour revenir à la version d'origine, utilisez « Rétablir les instructions officielles » ; une recette à vous se retire avec « Retirer ».",
              "Um zur ursprünglichen Version zurückzukehren, verwenden Sie „Offizielle Anweisungen wiederherstellen“; ein eigenes Rezept entfernen Sie mit „Entfernen“.",
            ),
          ]}
        />
      </Section>

      <Section
        title={tr("Prova una combinazione", "Try a combination", "Essayer une combinaison", "Kombination ausprobieren")}
      >
        <P>
          {tr(
            "Scegli una base e delle aggiunte: la scheda mostra il piano completo che la procedura eseguirebbe (strumenti, comandi, database, .env, azioni), con le istruzioni come sono adesso, anche se non le hai ancora salvate. Non esegue niente.",
            "Choose a base and some additions: the tab shows the complete plan the wizard would run (tools, commands, database, .env, actions), with the instructions as they are now, even if you have not saved them yet. It runs nothing.",
            "Choisissez une base et des ajouts : l'onglet montre le plan complet que l'assistant exécuterait (outils, commandes, base de données, .env, actions), avec les instructions telles qu'elles sont maintenant, même si vous ne les avez pas encore enregistrées. Il n'exécute rien.",
            "Wählen Sie eine Basis und Ergänzungen: Die Registerkarte zeigt den vollständigen Plan, den der Assistent ausführen würde (Werkzeuge, Befehle, Datenbank, .env, Aktionen), mit den Anweisungen, wie sie jetzt sind, auch wenn Sie sie noch nicht gespeichert haben. Sie führt nichts aus.",
          )}
        </P>
      </Section>

      <Under>
        <Bullets
          items={[
            tr(
              "Le ricette ufficiali sono nel catalogo del kit (tauri/catalog/frameworks.json): 63 basi e 18 aggiunte.",
              "The official recipes are in the kit's catalog (tauri/catalog/frameworks.json): 63 bases and 18 additions.",
              "Les recettes officielles sont dans le catalogue du kit (tauri/catalog/frameworks.json) : 63 bases et 18 ajouts.",
              "Die offiziellen Rezepte stehen im Katalog des Kits (tauri/catalog/frameworks.json): 63 Basen und 18 Ergänzungen.",
            ),
            tr(
              "Le tue modifiche sono in data\\app\\frameworks.json, che contiene solo le differenze dalle ricette ufficiali.",
              "Your changes are in data\\app\\frameworks.json, which contains only the differences from the official recipes.",
              "Vos modifications sont dans data\\app\\frameworks.json, qui ne contient que les différences par rapport aux recettes officielles.",
              "Ihre Änderungen liegen in data\\app\\frameworks.json, die nur die Unterschiede zu den offiziellen Rezepten enthält.",
            ),
            tr(
              "Se quel file non si può leggere, il kit usa le ricette ufficiali e non lo tocca: «Le istruzioni in … non sono leggibili: uso quelle ufficiali e non modifico il file.»",
              "If that file cannot be read, the kit uses the official recipes and does not touch it: “The instructions in … cannot be read: using the official ones and leaving the file as it is.”",
              "Si ce fichier ne peut pas être lu, le kit utilise les recettes officielles et n'y touche pas : « Les instructions de … sont illisibles : j'utilise les officielles et je ne modifie pas le fichier. »",
              "Wenn diese Datei nicht gelesen werden kann, verwendet das Kit die offiziellen Rezepte und lässt sie unangetastet: „Die Anweisungen in … sind nicht lesbar: Die offiziellen werden verwendet, die Datei bleibt unverändert.“",
            ),
          ]}
        />
      </Under>

      <Related ids={["project-new", "settings"]} />
    </>
  ),
};
