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
            "Sie erfolgt unter Einstellungen › „GitHub-Konto“ mit „Bei GitHub anmelden“ und einem Code, den du im Browser eingibst.",
          ),
          tr(
            "Senza account il kit funziona lo stesso: cloni i repository pubblici e lavori in locale.",
            "Without an account the kit works all the same: you clone public repositories and work locally.",
            "Sans compte, le kit fonctionne quand même : vous clonez les dépôts publics et travaillez en local.",
            "Ohne Konto funktioniert das Kit trotzdem: Du klonst öffentliche Repositorys und arbeitest lokal.",
          ),
          tr(
            "L'accesso resta nel kit: su un altro PC sei già collegato.",
            "The sign-in stays in the kit: on another PC you are already signed in.",
            "La connexion reste dans le kit : sur un autre PC, vous êtes déjà connecté.",
            "Die Anmeldung bleibt im Kit: Auf einem anderen PC bist du bereits angemeldet.",
          ),
          tr(
            "Con l'accesso, nome ed e-mail dei commit si riempiono dall'account, con l'indirizzo noreply di GitHub; li cambi in «Chi firma i commit».",
            "With the sign-in, the commits' name and email are filled from the account, with GitHub's noreply address; you change them in “Who signs the commits”.",
            "Avec la connexion, le nom et l'e-mail des commits se remplissent depuis le compte, avec l'adresse noreply de GitHub ; vous les changez dans « Qui signe les commits ».",
            "Mit der Anmeldung werden Name und E-Mail der Commits aus dem Konto gefüllt, mit der noreply-Adresse von GitHub; du änderst sie unter „Wer die Commits signiert“.",
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
                "Die Liste deiner Repositorys unter Projekte, mit Klonen, Umbenennen und Löschen",
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
              "Öffne Einstellungen › „GitHub-Konto“.",
            ),
            tr(
              "Premi «Accedi a GitHub». Dopo «Accesso in preparazione…» compare un codice di otto caratteri (XXXX-XXXX) e si apre github.com nel browser.",
              "Press “Sign in to GitHub”. After “Preparing the sign-in…” an eight-character code appears (XXXX-XXXX) and github.com opens in the browser.",
              "Appuyez sur « Se connecter à GitHub ». Après « Préparation de la connexion… », un code de huit caractères apparaît (XXXX-XXXX) et github.com s'ouvre dans le navigateur.",
              "Klicke auf „Bei GitHub anmelden“. Nach „Anmeldung wird vorbereitet…“ erscheint ein achtstelliger Code (XXXX-XXXX) und github.com öffnet sich im Browser.",
            ),
            tr(
              "Premi «Copia il codice», incollalo nella pagina di GitHub e autorizza GitHub CLI. Se la pagina non si è aperta, «Apri la pagina di GitHub».",
              "Press “Copy the code”, paste it into the GitHub page and authorize GitHub CLI. If the page did not open, use “Open the GitHub page”.",
              "Appuyez sur « Copier le code », collez-le dans la page de GitHub et autorisez GitHub CLI. Si la page ne s'est pas ouverte, utilisez « Ouvrir la page de GitHub ».",
              "Klicke auf „Code kopieren“, füge ihn auf der GitHub-Seite ein und autorisiere GitHub CLI. Wenn sich die Seite nicht geöffnet hat, verwende „GitHub-Seite öffnen“.",
            ),
            tr(
              "Torna su Atollo: compare «Accesso fatto come @tuo-nome».",
              "Go back to Atollo: “Signed in as @your-name” appears.",
              "Revenez sur Atollo : « Connecté en tant que @votre-nom » apparaît.",
              "Kehre zu Atollo zurück: „Angemeldet als @dein-name“ erscheint.",
            ),
          ]}
        />
        <P>
          {tr(
            "«Esci» toglie l'accesso dal kit dopo la conferma «Uscire da GitHub? Anche Git, l'editor e gli agenti del kit perdono l'accesso.»; «Annulla» o Esc chiudono la domanda.",
            "“Sign out” removes the sign-in from the kit after the confirmation “Sign out of GitHub? Git, the editor and the kit's agents lose the access too.”; “Cancel” or Esc closes the question.",
            "« Se déconnecter » retire la connexion du kit après la confirmation « Se déconnecter de GitHub ? Git, l'éditeur et les agents du kit perdent aussi l'accès. » ; « Annuler » ou Échap ferme la question.",
            "„Abmelden“ entfernt die Anmeldung aus dem Kit nach der Bestätigung „Von GitHub abmelden? Auch Git, der Editor und die Agenten des Kits verlieren den Zugang.“; „Abbrechen“ oder Esc schließt die Frage.",
          )}
        </P>
        <Callout tone="warn">
          {tr(
            "Il token di GitHub è salvato nel kit, in chiaro, perché ti segua su ogni PC. «Esci» non revoca la sessione su github.com: se perdi la chiavetta, revoca l'autorizzazione di GitHub CLI dalle impostazioni del tuo account GitHub (Applications).",
            "The GitHub token is saved in the kit, in plain text, so that it follows you to every PC. “Sign out” does not revoke the session on github.com: if you lose the USB stick, revoke the GitHub CLI authorization from your GitHub account settings (Applications).",
            "Le jeton GitHub est enregistré dans le kit, en clair, pour vous suivre sur chaque PC. « Se déconnecter » ne révoque pas la session sur github.com : si vous perdez la clé USB, révoquez l'autorisation de GitHub CLI depuis les paramètres de votre compte GitHub (Applications).",
            "Das GitHub-Token ist im Kit gespeichert, im Klartext, damit es dir auf jeden PC folgt. „Abmelden“ widerruft die Sitzung auf github.com nicht: Wenn du den USB-Stick verlierst, widerrufe die Autorisierung von GitHub CLI in den Einstellungen deines GitHub-Kontos (Applications).",
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
            "Die normale Anmeldung umfasst nicht die Berechtigung, Repositorys zu löschen. Wenn du zum ersten Mal bei einem deiner Repositorys auf „Von GitHub löschen“ klickst, meldet der Bereich, dass die Berechtigung fehlt, und zeigt „Berechtigung erteilen“: Das Kit meldet sich erneut an und fragt dabei auch danach. Der einzugebende Code erscheint unter Einstellungen › „GitHub-Konto“. Klicke dann erneut auf „Von GitHub löschen“.",
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
          "Dein Name in den Commits",
        )}
      >
        <P>
          {tr(
            "In Impostazioni › «Account GitHub», sotto l'account, «Chi firma i commit» ha due campi, «Nome» ed «E-mail»: sono il nome e l'e-mail dei commit fatti con il Git del kit. Ognuno si salva quando il campo perde il focus.",
            "In Settings › “GitHub account”, below the account, “Who signs the commits” has two fields, “Name” and “Email”: they are the name and email of the commits made with the kit's Git. Each one is saved when the field loses focus.",
            "Dans Paramètres › « Compte GitHub », sous le compte, « Qui signe les commits » a deux champs, « Nom » et « E-mail » : ce sont le nom et l'e-mail des commits faits avec le Git du kit. Chacun s'enregistre quand le champ perd le focus.",
            "Unter Einstellungen › „GitHub-Konto“ hat „Wer die Commits signiert“ unter dem Konto zwei Felder, „Name“ und „E-Mail“: Das sind Name und E-Mail der Commits, die mit dem Git des Kits gemacht werden. Jedes wird gespeichert, wenn das Feld den Fokus verliert.",
          )}
        </P>
        <Bullets
          items={[
            tr(
              "Se sono vuoti tutti e due e il kit ha accesso a GitHub, Atollo li riempie da solo, all'accesso e a ogni avvio: il nome dell'account (o il login) e l'indirizzo privato di GitHub, id+login@users.noreply.github.com. Così la tua e-mail vera resta privata.",
              "If both are empty and the kit is signed in to GitHub, Atollo fills them by itself, at sign-in and at every start: the account's name (or login) and GitHub's private address, id+login@users.noreply.github.com. That way your real email stays private.",
              "S'ils sont vides tous les deux et que le kit est connecté à GitHub, Atollo les remplit tout seul, à la connexion et à chaque démarrage : le nom du compte (ou le login) et l'adresse privée de GitHub, id+login@users.noreply.github.com. Ainsi votre vraie adresse e-mail reste privée.",
              "Wenn beide leer sind und das Kit bei GitHub angemeldet ist, füllt Atollo sie von selbst aus, bei der Anmeldung und bei jedem Start: den Namen des Kontos (oder den Login) und die private Adresse von GitHub, id+login@users.noreply.github.com. So bleibt deine echte E-Mail privat.",
            ),
            tr(
              "Puoi scrivere altro nome ed e-mail quando vuoi. Se svuoti tutti e due, vengono tolti; al prossimo avvio, se il kit ha ancora accesso a GitHub, tornano quelli dell'account.",
              "You can type another name and email whenever you like. If you empty both, they are removed; at the next start, if the kit is still signed in to GitHub, the account's come back.",
              "Vous pouvez saisir un autre nom et une autre adresse quand vous voulez. Si vous videz les deux, ils sont supprimés ; au prochain démarrage, si le kit est toujours connecté à GitHub, ceux du compte reviennent.",
              "Du kannst jederzeit einen anderen Namen und eine andere E-Mail eingeben. Wenn du beide leerst, werden sie entfernt; beim nächsten Start kommen die des Kontos zurück, wenn das Kit noch bei GitHub angemeldet ist.",
            ),
            tr(
              "Un valore che non va bene mostra «Nome non valido.» o «Indirizzo e-mail non valido.». Se Git non è ancora nel kit compare «Git non è ancora nel kit: si installa con GitHub CLI.».",
              "A value that is not valid shows “Invalid name.” or “Invalid email address.”. If Git is not in the kit yet, “Git is not in the kit yet: it is installed with GitHub CLI.” appears.",
              "Une valeur non valable affiche « Nom non valide. » ou « Adresse e-mail non valide. ». Si Git n'est pas encore dans le kit, « Git n'est pas encore dans le kit : il s'installe avec GitHub CLI. » apparaît.",
              "Ein ungültiger Wert zeigt „Ungültiger Name.“ oder „Ungültige E-Mail-Adresse.“. Wenn Git noch nicht im Kit ist, erscheint „Git ist noch nicht im Kit: Es wird mit GitHub CLI installiert.“.",
            ),
          ]}
        />
        <P>
          {tr(
            "Sono i valori user.name e user.email di Git: in un terminale del kit funziona anche git config --global.",
            "They are Git's user.name and user.email values: in a kit terminal git config --global works too.",
            "Ce sont les valeurs user.name et user.email de Git : dans un terminal du kit, git config --global fonctionne aussi.",
            "Das sind die Werte user.name und user.email von Git: In einem Terminal des Kits funktioniert auch git config --global.",
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
              "Git verwendet dasselbe Konto über einen Credential Helper (gh auth git-credential), der in die Datei data\\home\\.gitconfig für github.com und gist.github.com geschrieben wird, ohne absolute Pfade: Er übersteht das Verschieben des Kits. Bei jedem Start gleicht das Kit ihn an den Zustand von gh an; ein Helper, den du selbst für diese Adressen geschrieben hast, bleibt unverändert.",
            ),
            tr(
              "«Chi firma i commit» legge e scrive git config --global user.name e user.email, cioè il file data\\home\\.gitconfig del kit. Per riempirli dall'account basta una chiamata a gh api user.",
              "“Who signs the commits” reads and writes git config --global user.name and user.email, that is the kit's data\\home\\.gitconfig file. Filling them from the account takes one gh api user call.",
              "« Qui signe les commits » lit et écrit git config --global user.name et user.email, c'est-à-dire le fichier data\\home\\.gitconfig du kit. Les remplir depuis le compte demande un seul appel à gh api user.",
              "„Wer die Commits signiert“ liest und schreibt git config --global user.name und user.email, also die Datei data\\home\\.gitconfig des Kits. Um sie aus dem Konto zu füllen, genügt ein Aufruf von gh api user.",
            ),
            tr(
              "Git si fida solo dei repository nella cartella del kit: il .gitconfig del kit ha safe.directory = <cartella del kit>/* (per esempio D:/Atollo/*), riscritto a ogni avvio perché segua il kit quando cambia cartella; i valori aggiunti da te restano. Prima era safe.directory = *, cioè ogni repository del PC, anche di altri utenti.",
              "Git trusts only the repositories in the kit's folder: the kit's .gitconfig has safe.directory = <kit folder>/* (for example D:/Atollo/*), rewritten at every start so it follows the kit when it changes folder; values you added stay. Before it was safe.directory = *, that is every repository on the PC, even other users'.",
              "Git ne fait confiance qu'aux dépôts du dossier du kit : le .gitconfig du kit a safe.directory = <dossier du kit>/* (par exemple D:/Atollo/*), réécrit à chaque démarrage pour suivre le kit quand il change de dossier ; les valeurs que vous avez ajoutées restent. Avant, c'était safe.directory = *, c'est-à-dire tous les dépôts du PC, même ceux d'autres utilisateurs.",
              "Git vertraut nur den Repositorys im Ordner des Kits: Die .gitconfig des Kits hat safe.directory = <Kit-Ordner>/* (zum Beispiel D:/Atollo/*), bei jedem Start neu geschrieben, damit es dem Kit folgt, wenn es den Ordner wechselt; Werte, die du hinzugefügt hast, bleiben. Vorher war es safe.directory = *, also jedes Repository auf dem PC, auch das anderer Benutzer.",
            ),
            tr(
              "Per un repository fuori dal kit che appartiene a un altro utente, Git avvisa di nuovo «dubious ownership». Se ti fidi, in un terminale del kit: git config --global --add safe.directory <cartella>.",
              "For a repository outside the kit that belongs to another user, Git again warns about “dubious ownership”. If you trust it, in a kit terminal: git config --global --add safe.directory <folder>.",
              "Pour un dépôt hors du kit qui appartient à un autre utilisateur, Git avertit de nouveau « dubious ownership ». Si vous lui faites confiance, dans un terminal du kit : git config --global --add safe.directory <dossier>.",
              "Bei einem Repository außerhalb des Kits, das einem anderen Benutzer gehört, warnt Git wieder vor „dubious ownership“. Wenn du ihm vertraust, in einem Terminal des Kits: git config --global --add safe.directory <Ordner>.",
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
            "Le Impostazioni hanno sette sezioni nella barra laterale (Generale, Rete, Agenti, Aggiornamenti, Account GitHub, Privacy, Il kit) e, sotto, le istruzioni dei framework.",
            "Settings has seven sections in the sidebar (General, Network, Agents, Updates, GitHub account, Privacy, The kit) and, below them, the framework instructions.",
            "Les Paramètres ont sept sections dans la barre latérale (Général, Réseau, Agents, Mises à jour, Compte GitHub, Confidentialité, Le kit) et, en dessous, les instructions des frameworks.",
            "Die Einstellungen haben sieben Abschnitte in der Seitenleiste (Allgemein, Netzwerk, Agenten, Updates, GitHub-Konto, Datenschutz, Das Kit) und darunter die Anweisungen der Frameworks.",
          ),
          tr(
            "Non c'è un pulsante «Salva»: ogni modifica si salva da sola mezzo secondo dopo, e la barra delle schede mostra «Salvataggio…» e poi «Salvato.».",
            "There is no “Save” button: every change saves itself half a second later, and the tab bar shows “Saving…” and then “Saved.”.",
            "Il n'y a pas de bouton « Enregistrer » : chaque modification s'enregistre toute seule une demi-seconde plus tard, et la barre des onglets affiche « Enregistrement… » puis « Enregistré. ».",
            "Es gibt keine Schaltfläche „Speichern“: Jede Änderung speichert sich eine halbe Sekunde später von selbst, und die Registerkartenleiste zeigt „Wird gespeichert…“ und dann „Gespeichert.“.",
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
          "Le sezioni a sinistra, sotto le istruzioni dei framework; lo stato del salvataggio sulla barra delle schede.",
          "The sections on the left, the framework instructions below them; the saving status on the tab bar.",
          "Les sections à gauche, les instructions des frameworks en dessous ; l'état de l'enregistrement sur la barre des onglets.",
          "Die Abschnitte links, darunter die Anweisungen der Frameworks; der Speicherstatus in der Registerkartenleiste.",
        )}
      />

      <Section
        title={tr(
          "Come si salvano le modifiche",
          "How changes are saved",
          "Comment les modifications sont enregistrées",
          "Wie Änderungen gespeichert werden",
        )}
      >
        <Bullets
          items={[
            tr(
              "Ogni modifica si salva e si applica da sola, circa mezzo secondo dopo l'ultima battuta: sulla barra delle schede compare «Salvataggio…», poi «Salvato.».",
              "Every change is saved and applied by itself, about half a second after the last keystroke: the tab bar shows “Saving…”, then “Saved.”.",
              "Chaque modification est enregistrée et appliquée toute seule, environ une demi-seconde après la dernière frappe : la barre des onglets affiche « Enregistrement… », puis « Enregistré. ».",
              "Jede Änderung wird etwa eine halbe Sekunde nach dem letzten Tastendruck von selbst gespeichert und angewendet: Die Registerkartenleiste zeigt „Wird gespeichert…“, dann „Gespeichert.“.",
            ),
            tr(
              "Se un valore non va bene (per esempio un file PEM che non esiste), compare il suo errore e il valore resta sullo schermo: il kit riprova alla modifica successiva.",
              "If a value is not valid (for example a PEM file that does not exist), its error appears and the value stays on screen: the kit tries again at the next change.",
              "Si une valeur n'est pas valable (par exemple un fichier PEM qui n'existe pas), son erreur apparaît et la valeur reste à l'écran : le kit réessaie à la modification suivante.",
              "Wenn ein Wert nicht gültig ist (zum Beispiel eine PEM-Datei, die es nicht gibt), erscheint sein Fehler und der Wert bleibt auf dem Bildschirm: Das Kit versucht es bei der nächsten Änderung erneut.",
            ),
            tr(
              "Anche le istruzioni dei framework si salvano da sole, poco meno di un secondo dopo l'ultima modifica.",
              "The framework instructions save themselves too, a little under a second after the last edit.",
              "Les instructions des frameworks s'enregistrent elles aussi toutes seules, un peu moins d'une seconde après la dernière modification.",
              "Auch die Anweisungen der Frameworks speichern sich von selbst, knapp eine Sekunde nach der letzten Änderung.",
            ),
            tr(
              "Come nell'editor, un'impostazione cambiata rispetto al valore di base ha una barra colorata sul bordo sinistro.",
              "As in the editor, a setting changed from its default value has a colored bar on its left edge.",
              "Comme dans l'éditeur, un paramètre modifié par rapport à sa valeur par défaut a une barre colorée sur le bord gauche.",
              "Wie im Editor hat eine Einstellung, die vom Standardwert abweicht, einen farbigen Balken am linken Rand.",
            ),
            tr(
              "Se l'editor è aperto e cambi lingua, rete o temporanei dei programmi, compare «Riavvia l'editor perché i suoi terminali usino le nuove impostazioni.»: le altre impostazioni non lo richiedono. Il tema arriva subito anche all'editor e ai suoi terminali.",
              "If the editor is open and you change the language, the network or the programs' temporary files, “Restart the editor so that its terminals use the new settings.” appears: the other settings do not need it. The theme reaches the editor and its terminals at once.",
              "Si l'éditeur est ouvert et que vous changez la langue, le réseau ou les fichiers temporaires des programmes, « Redémarrez l'éditeur pour que ses terminaux utilisent les nouveaux paramètres. » apparaît : les autres paramètres ne le demandent pas. Le thème atteint tout de suite l'éditeur et ses terminaux.",
              "Wenn der Editor geöffnet ist und du die Sprache, das Netzwerk oder die temporären Dateien der Programme änderst, erscheint „Starte den Editor neu, damit seine Terminals die neuen Einstellungen verwenden.“: Die anderen Einstellungen brauchen das nicht. Das Design erreicht den Editor und seine Terminals sofort.",
            ),
          ]}
        />
      </Section>

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
                "La lingua di Atollo: quella del sistema («Lingua del sistema»), italiano, inglese, francese o tedesco. Cambia subito.",
                "The language of Atollo: the system's (“System language”), Italian, English, French or German. It changes at once.",
                "La langue d'Atollo : celle du système (« Langue du système »), italien, anglais, français ou allemand. Elle change tout de suite.",
                "Die Sprache von Atollo: die des Systems („Systemsprache“), Italienisch, Englisch, Französisch oder Deutsch. Sie ändert sich sofort.",
              ),
            ],
            [
              tr("Tema", "Theme", "Thème", "Design"),
              tr(
                "Chiaro o scuro: «Come il sistema» (di base), «Chiaro» o «Scuro». Vale subito per tutte le finestre: Atollo, l'editor integrato e i suoi terminali, senza riavviare l'editor.",
                "Light or dark: “Same as the system” (the default), “Light” or “Dark”. It applies at once to every window: Atollo, the integrated editor and its terminals, without restarting the editor.",
                "Clair ou sombre : « Comme le système » (par défaut), « Clair » ou « Sombre ». Il s'applique tout de suite à toutes les fenêtres : Atollo, l'éditeur intégré et ses terminaux, sans redémarrer l'éditeur.",
                "Hell oder dunkel: „Wie das System“ (Standard), „Hell“ oder „Dunkel“. Es gilt sofort für alle Fenster: Atollo, den integrierten Editor und seine Terminals, ohne den Editor neu zu starten.",
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
                "Wo die Programme des Kits ihre temporären Dateien ablegen. „Automatisch“ verwendet den temporären Ordner des Systems, wenn das Kit auf einem USB-Stick, einem Netzlaufwerk oder einem FAT32/exFAT-Laufwerk liegt, sonst data\\tmp. „Im Kit“ immer data\\tmp; „Im System“ immer den Ordner des Systems (die Dateien bleiben auf dem PC, bis du die App schließt).",
              ),
            ],
            [
              tr("Aggiornamento automatico", "Automatic update", "Mise à jour automatique", "Automatisches Update"),
              tr(
                "«Scarica da solo le nuove versioni di Atollo (sostituiscono l'exe alla chiusura)»: la nuova versione sostituisce quella attuale quando chiudi l'app. Attivo di base. Vale subito: attivarlo avvia un controllo all'istante; spegnerlo prima del controllo dei 20 secondi dopo l'avvio ferma quel controllo. Permette anche il controllo giornaliero delle versioni dei componenti installati: al massimo una volta al giorno, dopo l'avvio, Atollo chiede a ogni fonte l'ultima versione e Strumenti mostra il riquadro «Aggiornamenti» con quelli indietro. Spento, non si chiede niente da solo; «Controlla ora» funziona comunque.",
                "“Download new versions of Atollo automatically (they replace the exe on close)”: the new version replaces the current one when you close the app. On by default. It applies at once: turning it on starts a check right away; turning it off before the check 20 seconds after start stops that check. It also allows the daily check of the installed components' versions: at most once a day, after the start, Atollo asks each source for its latest version and Tools shows the ones behind in the “Updates” pane. Off, nothing is asked by itself; “Check now” still works.",
                "« Télécharger automatiquement les nouvelles versions d'Atollo (elles remplacent l'exe à la fermeture) » : la nouvelle version remplace l'actuelle quand vous fermez l'application. Activé par défaut. Il s'applique tout de suite : l'activer lance aussitôt une vérification ; le désactiver avant la vérification des 20 secondes après le démarrage annule cette vérification. Il permet aussi la vérification quotidienne des versions des composants installés : au plus une fois par jour, après le démarrage, Atollo demande à chaque source sa dernière version et Outils montre ceux qui sont en retard dans le volet « Mises à jour ». Désactivé, rien n'est demandé de lui-même ; « Vérifier maintenant » fonctionne toujours.",
                "„Neue Versionen von Atollo automatisch herunterladen (sie ersetzen die EXE beim Schließen)“: Die neue Version ersetzt die aktuelle, wenn du die App schließt. Standardmäßig aktiv. Es gilt sofort: Einschalten startet sofort eine Prüfung; Ausschalten vor der Prüfung 20 Sekunden nach dem Start stoppt diese Prüfung. Es erlaubt auch die tägliche Prüfung der Versionen der installierten Komponenten: höchstens einmal am Tag, nach dem Start, fragt Atollo jede Quelle nach ihrer neuesten Version, und Werkzeuge zeigt die veralteten im Bereich „Aktualisierungen“. Ausgeschaltet wird nichts von selbst abgefragt; „Jetzt prüfen“ funktioniert weiterhin.",
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
            "Programme schreiben und löschen ständig temporäre Dateien: Auf einem USB-Stick verlangsamt das alles und nutzt ihn ab. Auf der Festplatte des PCs geht das viel schneller, und das Kit löscht sie, wenn du die App schließt.",
          )}
        </Callout>
        <Callout
          tone="info"
          title={tr(
            "Versioni e compilatore di Rust",
            "Versions and Rust compiler",
            "Versions et compilateur Rust",
            "Versionen und Rust-Compiler",
          )}
        >
          {rich(
            tr(
              "Le Impostazioni non scelgono più le versioni di Python e PHP: una nuova installazione prende la linea più recente, le altre linee si scaricano dal menu delle versioni di ogni strumento in Strumenti e ogni progetto sceglie la sua ({0}). I valori pythonVersion e phpVersion di un vecchio settings.json vengono ignorati. Il «Compilatore di Rust» è nella pagina di Rust in Strumenti, sezione «Dettagli».",
              "Settings no longer chooses the Python and PHP versions: a new installation takes the newest line, the other lines are downloaded from each tool's versions menu in Tools, and each project chooses its own ({0}). The pythonVersion and phpVersion values of an older settings.json are ignored. The “Rust compiler” is on Rust's page in Tools, “Details” section.",
              "Les Paramètres ne choisissent plus les versions de Python et de PHP : une nouvelle installation prend la ligne la plus récente, les autres lignes se téléchargent depuis le menu des versions de chaque outil dans Outils, et chaque projet choisit la sienne ({0}). Les valeurs pythonVersion et phpVersion d'un ancien settings.json sont ignorées. Le « Compilateur Rust » se trouve dans la page de Rust dans Outils, section « Détails ».",
              "Die Einstellungen wählen die Versionen von Python und PHP nicht mehr: Eine neue Installation nimmt die neueste Linie, die anderen Linien lädst du über das Versionsmenü jedes Werkzeugs unter Werkzeuge herunter, und jedes Projekt wählt seine eigene ({0}). Die Werte pythonVersion und phpVersion einer älteren settings.json werden ignoriert. Der „Rust-Compiler“ steht auf der Seite von Rust unter Werkzeuge, Abschnitt „Details“.",
            ),
            <See key="v" id="tools-versions" />,
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
            "«Chiave API di Gemini (Antigravity senza account Google)»: incolla la chiave e passa ad altro (un clic fuori dal campo o Tab). La chiave si salva quando il campo perde il fuoco, poi il campo si svuota e mostra «salvata: scrivi per sostituirla»: la chiave non viene più mostrata. «Togli la chiave» la cancella subito. Antigravity mostrerà «accesso con chiave API». La chiave è salvata a parte, in data\\app\\secrets\\gemini-api-key, leggibile solo dal tuo utente dove il disco ha i permessi, e passata come GEMINI_API_KEY solo ad Antigravity avviato da Atollo: non alle azioni dei progetti, agli script, ai terminali né all'editor.",
            "“Gemini API key (Antigravity without a Google account)”: paste the key and move on (a click outside the field, or Tab). The key is saved when the field loses focus, then the field empties and shows “saved: type to replace it”: the key is no longer shown. “Remove the key” deletes it at once. Antigravity will show “signed in with an API key”. The key is saved on its own, in data\\app\\secrets\\gemini-api-key, readable by your user only where the disk has permissions, and passed as GEMINI_API_KEY only to the Antigravity that Atollo launches: not to project actions, scripts, terminals or the editor.",
            "« Clé API Gemini (Antigravity sans compte Google) » : collez la clé et passez à autre chose (un clic hors du champ, ou Tab). La clé est enregistrée quand le champ perd le focus, puis le champ se vide et affiche « enregistrée : saisissez-en une autre pour la remplacer » : la clé n'est plus affichée. « Supprimer la clé » l'efface tout de suite. Antigravity affichera « connecté avec une clé API ». La clé est enregistrée à part, dans data\\app\\secrets\\gemini-api-key, lisible par votre seul utilisateur là où le disque gère les permissions, et transmise comme GEMINI_API_KEY au seul Antigravity lancé par Atollo : ni aux actions des projets, ni aux scripts, ni aux terminaux, ni à l'éditeur.",
            "„Gemini-API-Schlüssel (Antigravity ohne Google-Konto)“: Füge den Schlüssel ein und mach woanders weiter (ein Klick außerhalb des Felds oder Tab). Der Schlüssel wird gespeichert, wenn das Feld den Fokus verliert; dann leert sich das Feld und zeigt „gespeichert: zum Ersetzen neu eingeben“: Der Schlüssel wird nicht mehr angezeigt. „Schlüssel entfernen“ löscht ihn sofort. Antigravity zeigt dann „mit API-Schlüssel angemeldet“. Der Schlüssel wird getrennt in data\\app\\secrets\\gemini-api-key gespeichert, nur für deinen Benutzer lesbar, wo das Laufwerk Berechtigungen kennt, und als GEMINI_API_KEY nur dem von Atollo gestarteten Antigravity übergeben: nicht den Aktionen der Projekte, Skripten, Terminals oder dem Editor.",
          )}
        </P>
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
        <P>
          {tr(
            "Sotto l'account, «Chi firma i commit» ha i campi «Nome» ed «E-mail» dei commit fatti con il Git del kit; ognuno si salva quando il campo perde il focus.",
            "Below the account, “Who signs the commits” has the “Name” and “Email” fields of the commits made with the kit's Git; each one is saved when the field loses focus.",
            "Sous le compte, « Qui signe les commits » a les champs « Nom » et « E-mail » des commits faits avec le Git du kit ; chacun s'enregistre quand le champ perd le focus.",
            "Unter dem Konto hat „Wer die Commits signiert“ die Felder „Name“ und „E-Mail“ der Commits, die mit dem Git des Kits gemacht werden; jedes wird gespeichert, wenn das Feld den Fokus verliert.",
          )}
        </P>
      </Section>

      <Section title={tr("Privacy", "Privacy", "Confidentialité", "Datenschutz")}>
        <P>
          {tr(
            "«Cosa fa Atollo con i tuoi dati», in breve: nessuna telemetria e nessun account; impostazioni, chiavi e accessi restano nella cartella del kit. Atollo contatta GitHub per i suoi aggiornamenti (se l'aggiornamento automatico è attivo), le fonti ufficiali di linguaggi e strumenti quando li installi o aggiorni e, una volta al giorno, per leggere le versioni disponibili (anche questo si spegne con ATOLLO_NO_UPDATE), e Open VSX per le estensioni dell'editor. Con la pagina Agenti aperta, ogni 2 minuti legge i limiti del piano di ogni agente collegato, mandandone il token solo al suo fornitore; con l'account GitHub collegato ne usa il token con api.github.com. Agenti, GitHub e registri dei pacchetti parlano direttamente con i loro servizi, con le loro regole.",
            "“What Atollo does with your data”, in short: no telemetry and no account; settings, keys and sign-ins stay in the kit's folder. Atollo contacts GitHub for its own updates (if automatic updates are on), the official sources of languages and tools when you install or update them and, once a day, to read the available versions (ATOLLO_NO_UPDATE turns this off too), and Open VSX for the editor's extensions. While the Agents page is open, every 2 minutes it reads the plan limits of each signed-in agent, sending that agent's token only to its provider; with the GitHub account signed in it uses that account's token with api.github.com. Agents, GitHub and package registries talk directly to their own services, under their own rules.",
            "« Ce qu'Atollo fait de vos données », en bref : aucune télémétrie et aucun compte ; paramètres, clés et connexions restent dans le dossier du kit. Atollo contacte GitHub pour ses propres mises à jour (si la mise à jour automatique est activée), les sources officielles des langages et des outils quand vous les installez ou les mettez à jour et, une fois par jour, pour lire les versions disponibles (ATOLLO_NO_UPDATE désactive cela aussi), et Open VSX pour les extensions de l'éditeur. Quand la page Agents est ouverte, toutes les 2 minutes il lit les limites du forfait de chaque agent connecté, en n'envoyant son jeton qu'à son fournisseur ; avec le compte GitHub connecté, il en utilise le jeton avec api.github.com. Agents, GitHub et registres de paquets parlent directement à leurs propres services, selon leurs propres règles.",
            "„Was Atollo mit deinen Daten macht“, kurz gesagt: keine Telemetrie und kein Konto; Einstellungen, Schlüssel und Anmeldungen bleiben im Ordner des Kits. Atollo kontaktiert GitHub für die eigenen Updates (wenn das automatische Update aktiv ist), die offiziellen Quellen der Sprachen und Werkzeuge, wenn du sie installierst oder aktualisierst, und einmal am Tag, um die verfügbaren Versionen zu lesen (auch das schaltet ATOLLO_NO_UPDATE ab), sowie Open VSX für die Erweiterungen des Editors. Solange die Seite Agenten offen ist, liest es alle 2 Minuten die Planlimits jedes angemeldeten Agenten und schickt dessen Token nur an seinen Anbieter; mit angemeldetem GitHub-Konto verwendet es dessen Token mit api.github.com. Agenten, GitHub und Paket-Registries sprechen direkt mit ihren eigenen Diensten, nach ihren eigenen Regeln.",
          )}
        </P>
        <P>
          {tr(
            "«Leggi l'informativa sulla privacy» apre nel browser l'informativa completa (docs/PRIVACY.md su GitHub), nella lingua dell'app.",
            "“Read the privacy policy” opens the full policy in the browser (docs/PRIVACY.md on GitHub), in the app's language.",
            "« Lire la politique de confidentialité » ouvre la politique complète dans le navigateur (docs/PRIVACY.md sur GitHub), dans la langue de l'application.",
            "„Datenschutzerklärung lesen“ öffnet die vollständige Erklärung im Browser (docs/PRIVACY.md auf GitHub), in der Sprache der App.",
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
        <P>
          {rich(
            tr(
              "«Salute del kit» controlla ciò che del PC e del disco può fermare il kit: il disco (FAT32, exFAT, noexec), lo spazio libero, i caratteri a rischio nel percorso del kit, i file resi privati da un altro account e, su Linux, le librerie di sistema. Ogni riga è verde, gialla o rossa, con sotto che cosa fare; «Controlla di nuovo» rifà i controlli ({0}).",
              "“Kit health” checks what of the PC and the drive may stop the kit: the drive (FAT32, exFAT, noexec), the free space, risky characters in the kit's path, files made private by another account and, on Linux, the system's libraries. Each line is green, yellow or red, with what to do under it; “Check again” runs the checks again ({0}).",
              "« Santé du kit » contrôle ce qui, du PC et du disque, peut arrêter le kit : le disque (FAT32, exFAT, noexec), l'espace libre, les caractères à risque dans le chemin du kit, les fichiers rendus privés par un autre compte et, sous Linux, les bibliothèques du système. Chaque ligne est verte, jaune ou rouge, avec en dessous ce qu'il faut faire ; « Vérifier à nouveau » refait les contrôles ({0}).",
              "„Zustand des Kits“ prüft, was am PC und am Laufwerk das Kit aufhalten kann: das Laufwerk (FAT32, exFAT, noexec), den freien Platz, riskante Zeichen im Pfad des Kits, von einem anderen Konto privat gemachte Dateien und unter Linux die Systembibliotheken. Jede Zeile ist grün, gelb oder rot, darunter steht, was zu tun ist; „Erneut prüfen“ wiederholt die Prüfungen ({0}).",
            ),
            <See key="h" id="troubleshooting" />,
          )}
        </P>
        <P>
          {tr(
            "Sotto, «Segnalare un problema» ha due pulsanti. «Esporta diagnostica» crea in data\\app\\diagnostics uno zip con versione, sistema, disco, ambiente, verifica dei programmi, componenti, porte e la parte finale dei log, con chiavi, token e password del kit sostituiti da ***, la cartella del tuo profilo come ~ e i nomi tuo e del PC tolti; «Apri la cartella» lo mostra. Lo zip resta sul PC: guardalo e allegalo tu. «Segnala un problema» apre nel browser una segnalazione nuova su GitHub con versione e sistema già scritti; nulla parte finché non la crei.",
            "Below, “Reporting a problem” has two buttons. “Export diagnostics” makes a zip in data\\app\\diagnostics with version, system, disk, environment, the programs' check, components, ports and the end of the logs, with the kit's keys, tokens and passwords replaced by ***, your profile folder as ~ and your and the PC's names removed; “Open folder” shows it. The zip stays on the PC: look at it and attach it yourself. “Report a problem” opens a new issue on GitHub in the browser with version and system already filled in; nothing leaves until you create it.",
            "En dessous, « Signaler un problème » a deux boutons. « Exporter le diagnostic » crée dans data\\app\\diagnostics un zip avec la version, le système, le disque, l'environnement, la vérification des programmes, les composants, les ports et la fin des journaux, avec les clés, jetons et mots de passe du kit remplacés par ***, le dossier de votre profil sous la forme ~ et votre nom et celui du PC retirés ; « Ouvrir le dossier » l'affiche. Le zip reste sur le PC : regardez-le et joignez-le vous-même. « Signaler un problème » ouvre dans le navigateur une nouvelle issue sur GitHub avec la version et le système déjà remplis ; rien ne part tant que vous ne la créez pas.",
            "Darunter hat „Ein Problem melden“ zwei Schaltflächen. „Diagnose exportieren“ erstellt in data\\app\\diagnostics ein Zip mit Version, System, Laufwerk, Umgebung, Prüfung der Programme, Komponenten, Ports und dem Ende der Logs, wobei Schlüssel, Tokens und Passwörter des Kits durch *** ersetzt sind, dein Profilordner als ~ erscheint und dein Name und der des PCs entfernt sind; „Ordner öffnen“ zeigt es. Das Zip bleibt auf dem PC: Sieh es dir an und hänge es selbst an. „Problem melden“ öffnet im Browser ein neues Issue auf GitHub mit bereits eingetragener Version und System; nichts geht weg, bevor du es erstellst.",
          )}
        </P>
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
              "Le impostazioni sono in data\\app\\settings.json. Ogni salvataggio rilegge il file, controlla i valori, lo scrive in modo sicuro (prima una copia, poi la sostituzione) e ricostruisce l'ambiente del kit. I salvataggi passano uno alla volta, sempre con i valori più recenti.",
              "The settings are in data\\app\\settings.json. Every save reads the file again, checks the values, writes it safely (first a copy, then the replacement) and rebuilds the kit's environment. Saves go through one at a time, always with the latest values.",
              "Les paramètres sont dans data\\app\\settings.json. Chaque enregistrement relit le fichier, contrôle les valeurs, l'écrit de façon sûre (d'abord une copie, puis le remplacement) et reconstruit l'environnement du kit. Les enregistrements passent un par un, toujours avec les valeurs les plus récentes.",
              "Die Einstellungen liegen in data\\app\\settings.json. Jedes Speichern liest die Datei neu, prüft die Werte, schreibt sie sicher (zuerst eine Kopie, dann das Ersetzen) und baut die Umgebung des Kits neu auf. Gespeichert wird immer nur einmal gleichzeitig, stets mit den neuesten Werten.",
            ),
            tr(
              "Il tema è la voce theme di settings.json. Atollo lo passa alle finestre (Tauri set_theme): le loro pagine lo vedono come prefers-color-scheme. L'editor integrato lo segue grazie a window.autoDetectColorScheme, e i terminali con lui.",
              "The theme is the theme entry of settings.json. Atollo passes it to the windows (Tauri set_theme): their pages see it as prefers-color-scheme. The integrated editor follows it thanks to window.autoDetectColorScheme, and the terminals with it.",
              "Le thème est l'entrée theme de settings.json. Atollo le transmet aux fenêtres (Tauri set_theme) : leurs pages le voient comme prefers-color-scheme. L'éditeur intégré le suit grâce à window.autoDetectColorScheme, et les terminaux avec lui.",
              "Das Design ist der Eintrag theme in settings.json. Atollo gibt es an die Fenster weiter (Tauri set_theme): Ihre Seiten sehen es als prefers-color-scheme. Der integrierte Editor folgt ihm dank window.autoDetectColorScheme, und die Terminals mit ihm.",
            ),
            tr(
              "Se il file non si può leggere, l'app usa i valori predefiniti e lo riscrive alla prima modifica: «Le impostazioni in … non sono leggibili: uso quelle predefinite. Il file verrà riscritto alla prima modifica.» Le voci scritte da una versione più recente di Atollo, che questa non conosce, restano nel file.",
              "If the file cannot be read, the app uses the default values and rewrites it at the first change: “The settings in … cannot be read: using the defaults. The file will be rewritten at the first change.” Entries written by a newer version of Atollo, which this one does not know, stay in the file.",
              "Si le fichier ne peut pas être lu, l'application utilise les valeurs par défaut et le réécrit à la première modification : « Les paramètres de … sont illisibles : les valeurs par défaut sont utilisées. Le fichier sera réécrit à la première modification. » Les entrées écrites par une version plus récente d'Atollo, que celle-ci ne connaît pas, restent dans le fichier.",
              "Wenn die Datei nicht gelesen werden kann, verwendet die App die Standardwerte und schreibt sie bei der ersten Änderung neu: „Die Einstellungen in … sind nicht lesbar: Es gelten die Standardwerte. Die Datei wird bei der ersten Änderung neu geschrieben.“ Einträge, die eine neuere Version von Atollo geschrieben hat und die diese nicht kennt, bleiben in der Datei.",
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

      <Related ids={["network", "updates", "frameworks", "troubleshooting"]} />
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
          "Herausfinden, ob du sie brauchst",
        )}
      >
        <P>
          {tr(
            "Se a casa tutto si scarica e in ufficio i download falliscono con errori di connessione o di certificato, probabilmente la rete aziendale usa un proxy, oppure ispeziona il traffico HTTPS con un suo certificato. I valori giusti te li dà l'assistenza informatica; spesso sono gli stessi impostati nel browser aziendale.",
            "If everything downloads at home but at the office downloads fail with connection or certificate errors, the company network probably uses a proxy, or inspects HTTPS traffic with its own certificate. Your IT support can give you the right values; often they are the same as those set in the company browser.",
            "Si tout se télécharge à la maison mais qu'au bureau les téléchargements échouent avec des erreurs de connexion ou de certificat, le réseau de l'entreprise utilise probablement un proxy, ou inspecte le trafic HTTPS avec son propre certificat. Le support informatique vous donnera les bonnes valeurs ; ce sont souvent les mêmes que celles du navigateur de l'entreprise.",
            "Wenn zu Hause alles heruntergeladen wird, im Büro die Downloads aber mit Verbindungs- oder Zertifikatsfehlern scheitern, verwendet das Firmennetzwerk wahrscheinlich einen Proxy oder untersucht den HTTPS-Verkehr mit einem eigenen Zertifikat. Die richtigen Werte erhältst du von der IT-Abteilung; oft sind es dieselben wie im Browser der Firma.",
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
              "Apri Impostazioni › «Rete» (al primo avvio, il gruppo «Rete» della pagina «Benvenuto in Atollo»: niente si scarica prima di «Installa») e compila i campi che ti servono.",
              "Open Settings › “Network” (at the first start, the “Network” group of the “Welcome to Atollo” page: nothing downloads before “Install”) and fill in the fields you need.",
              "Ouvrez Paramètres › « Réseau » (au premier démarrage, le groupe « Réseau » de la page « Bienvenue dans Atollo » : rien ne se télécharge avant « Installer ») et remplissez les champs dont vous avez besoin.",
              "Öffne Einstellungen › „Netzwerk“ (beim ersten Start die Gruppe „Netzwerk“ der Seite „Willkommen bei Atollo“: Vor „Installieren“ wird nichts heruntergeladen) und fülle die Felder aus, die du brauchst.",
            ),
            tr(
              "Non serve premere nulla: i valori si salvano da soli mentre scrivi. Se l'editor è aperto, premi «Riavvia l'editor» nell'avviso che compare.",
              "There is nothing to press: the values save themselves as you type. If the editor is open, press “Restart the editor” in the notice that appears.",
              "Il n'y a rien à appuyer : les valeurs s'enregistrent toutes seules pendant que vous écrivez. Si l'éditeur est ouvert, appuyez sur « Redémarrer l'éditeur » dans l'avis qui apparaît.",
              "Du musst nichts drücken: Die Werte speichern sich beim Tippen von selbst. Wenn der Editor geöffnet ist, klicke im erscheinenden Hinweis auf „Editor neu starten“.",
            ),
            tr(
              "Torna in Strumenti e riprova il download che non riusciva.",
              "Go back to Tools and retry the download that was failing.",
              "Revenez dans Outils et relancez le téléchargement qui échouait.",
              "Kehre zu Werkzeuge zurück und versuche den fehlgeschlagenen Download erneut.",
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
              "Il file PEM deve esistere: il kit controlla il percorso a ogni salvataggio. Finché il valore non va bene compare il suo errore e il campo non viene salvato; il kit riprova alla modifica successiva.",
              "The PEM file must exist: the kit checks the path at every save. As long as the value is not valid its error appears and the field is not saved; the kit tries again at the next change.",
              "Le fichier PEM doit exister : le kit contrôle le chemin à chaque enregistrement. Tant que la valeur n'est pas valable, son erreur apparaît et le champ n'est pas enregistré ; le kit réessaie à la modification suivante.",
              "Die PEM-Datei muss existieren: Das Kit prüft den Pfad bei jedem Speichern. Solange der Wert nicht gültig ist, erscheint sein Fehler und das Feld wird nicht gespeichert; das Kit versucht es bei der nächsten Änderung erneut.",
            ),
          ]}
        />
      </Section>

      <Section title={tr("Rete locale", "Local network", "Réseau local", "Lokales Netzwerk")}>
        <P>
          {tr(
            "La casella «Permetti ai server dei progetti di farsi raggiungere dagli altri PC della rete» apre i siti dei progetti alla rete: utile per provare un sito dal telefono. Dagli altri dispositivi si usa l'indirizzo IP di questo PC con la porta del sito (per esempio http://192.168.1.20:8000), che non serve ricopiare: nel dettaglio dell'azione in corso, nella pagina del progetto, la riga «Dal telefono» mostra l'indirizzo con «Copia» e «Mostra il QR», da inquadrare con la fotocamera del telefono. Database e phpMyAdmin restano raggiungibili solo da questo PC.",
            "The “Let other PCs on the network reach the project servers” checkbox opens the projects' sites to the network: useful to try a site from your phone. From the other devices you use this PC's IP address with the site's port (for example http://192.168.1.20:8000), with no need to type it: in the details of the running action, on the project page, the “From the phone” row shows the address with “Copy” and “Show the QR code”, to scan with the phone's camera. Databases and phpMyAdmin stay reachable only from this PC.",
            "La case « Permettre aux autres PC du réseau d'accéder aux serveurs des projets » ouvre les sites des projets au réseau : pratique pour essayer un site depuis le téléphone. Depuis les autres appareils, on utilise l'adresse IP de ce PC avec le port du site (par exemple http://192.168.1.20:8000), sans avoir à la recopier : dans le détail de l'action en cours, sur la page du projet, la ligne « Depuis le téléphone » montre l'adresse avec « Copier » et « Afficher le QR code », à scanner avec l'appareil photo du téléphone. Les bases de données et phpMyAdmin restent accessibles uniquement depuis ce PC.",
            "Das Kästchen „Anderen PCs im Netzwerk den Zugriff auf die Projektserver erlauben“ öffnet die Websites der Projekte für das Netzwerk: praktisch, um eine Website auf dem Telefon auszuprobieren. Von den anderen Geräten aus verwendest du die IP-Adresse dieses PCs mit dem Port der Website (zum Beispiel http://192.168.1.20:8000), ohne sie abzutippen: In den Details der laufenden Aktion auf der Projektseite zeigt die Zeile „Vom Telefon“ die Adresse mit „Kopieren“ und „QR-Code anzeigen“, zum Scannen mit der Kamera des Telefons. Datenbanken und phpMyAdmin bleiben nur von diesem PC aus erreichbar.",
          )}
        </P>
        <Bullets
          items={[
            tr(
              "Il server PHP integrato (php -S) e il server di file dei siti statici rispondono a chi li chiama con l'indirizzo IP del PC; con qualsiasi altro nome la richiesta resta rifiutata.",
              "PHP's built-in server (php -S) and the file server of static sites answer whoever calls them by the PC's IP address; with any other name the request stays refused.",
              "Le serveur intégré de PHP (php -S) et le serveur de fichiers des sites statiques répondent à qui les appelle par l'adresse IP du PC ; avec tout autre nom, la requête reste refusée.",
              "Der eingebaute Server von PHP (php -S) und der Dateiserver der statischen Websites antworten jedem, der sie über die IP-Adresse des PCs aufruft; mit jedem anderen Namen bleibt die Anfrage abgelehnt.",
            ),
            tr(
              "Su questo PC un sito statico risponde anche a http://<progetto>.localhost:<porta>, oltre che a localhost e 127.0.0.1, con la casella accesa o spenta.",
              "On this PC a static site also answers at http://<project>.localhost:<port>, besides localhost and 127.0.0.1, with the checkbox on or off.",
              "Sur ce PC, un site statique répond aussi à http://<projet>.localhost:<port>, en plus de localhost et 127.0.0.1, la case cochée ou non.",
              "Auf diesem PC antwortet eine statische Website auch unter http://<Projekt>.localhost:<Port>, neben localhost und 127.0.0.1, ob das Kästchen an ist oder nicht.",
            ),
            tr(
              "La riga «Dal telefono» compare solo per un indirizzo localhost o 127.0.0.1 e solo se questo PC ha un indirizzo privato sulla rete (192.168.x.x, 10.x.x.x, 172.16-31.x.x). Il sito deve essere stato avviato dopo aver acceso «Rete locale»: se era già in corso, fermalo e riavvialo. Con una VPN attiva l'indirizzo mostrato può essere quello della VPN.",
              "The “From the phone” row appears only for a localhost or 127.0.0.1 address and only if this PC has a private address on the network (192.168.x.x, 10.x.x.x, 172.16-31.x.x). The site must have been started after turning “Local network” on: if it was already running, stop it and start it again. With a VPN on, the address shown may be the VPN's.",
              "La ligne « Depuis le téléphone » n'apparaît que pour une adresse localhost ou 127.0.0.1 et seulement si ce PC a une adresse privée sur le réseau (192.168.x.x, 10.x.x.x, 172.16-31.x.x). Le site doit avoir été démarré après l'activation de « Réseau local » : s'il tournait déjà, arrêtez-le et redémarrez-le. Avec un VPN actif, l'adresse affichée peut être celle du VPN.",
              "Die Zeile „Vom Telefon“ erscheint nur bei einer Adresse localhost oder 127.0.0.1 und nur, wenn dieser PC eine private Adresse im Netzwerk hat (192.168.x.x, 10.x.x.x, 172.16-31.x.x). Die Website muss nach dem Einschalten von „Lokales Netzwerk“ gestartet worden sein: Lief sie schon, beende sie und starte sie neu. Mit aktivem VPN kann die angezeigte Adresse die des VPN sein.",
            ),
          ]}
        />
        <Callout tone="warn">
          {tr(
            "Chiunque sia sulla stessa rete potrà vedere i tuoi siti di prova. Attivala su reti di cui ti fidi e spegnila quando hai finito. Windows può chiedere il permesso del firewall la prima volta.",
            "Anyone on the same network will be able to see your test sites. Turn it on on networks you trust and turn it off when you are done. Windows may ask for the firewall permission the first time.",
            "Toute personne sur le même réseau pourra voir vos sites de test. Activez-la sur des réseaux de confiance et désactivez-la quand vous avez fini. Windows peut demander l'autorisation du pare-feu la première fois.",
            "Jeder im selben Netzwerk kann deine Testseiten sehen. Aktiviere die Option in Netzwerken, denen du vertraust, und schalte sie aus, wenn du fertig bist. Windows kann beim ersten Mal nach der Erlaubnis der Firewall fragen.",
          )}
        </Callout>
      </Section>

      <Under>
        <Bullets
          items={[
            tr(
              "I valori finiscono nelle variabili HTTPS_PROXY, HTTP_PROXY, NO_PROXY e NODE_EXTRA_CA_CERTS di tutti i programmi del kit. Con un file PEM, data\\app\\ca-bundle.pem riunisce i certificati del sistema e i suoi, per gli strumenti che leggono un solo file (SSL_CERT_FILE, REQUESTS_CA_BUNDLE, PIP_CERT, CURL_CA_BUNDLE, GIT_SSL_CAINFO, CARGO_HTTP_CAINFO). Un file PEM dentro il kit è salvato relativo al kit: lo segue se cambia disco o cartella.",
              "The values end up in the HTTPS_PROXY, HTTP_PROXY, NO_PROXY and NODE_EXTRA_CA_CERTS variables of all the kit's programs. With a PEM file, data\\app\\ca-bundle.pem joins the system's certificates and its own, for the tools that read a single file (SSL_CERT_FILE, REQUESTS_CA_BUNDLE, PIP_CERT, CURL_CA_BUNDLE, GIT_SSL_CAINFO, CARGO_HTTP_CAINFO). A PEM file inside the kit is saved relative to the kit: it follows it to another drive or folder.",
              "Les valeurs vont dans les variables HTTPS_PROXY, HTTP_PROXY, NO_PROXY et NODE_EXTRA_CA_CERTS de tous les programmes du kit. Avec un fichier PEM, data\\app\\ca-bundle.pem réunit les certificats du système et les siens, pour les outils qui lisent un seul fichier (SSL_CERT_FILE, REQUESTS_CA_BUNDLE, PIP_CERT, CURL_CA_BUNDLE, GIT_SSL_CAINFO, CARGO_HTTP_CAINFO). Un fichier PEM dans le kit est enregistré relativement au kit : il le suit s'il change de disque ou de dossier.",
              "Die Werte landen in den Variablen HTTPS_PROXY, HTTP_PROXY, NO_PROXY und NODE_EXTRA_CA_CERTS aller Programme des Kits. Mit einer PEM-Datei fasst data\\app\\ca-bundle.pem die Zertifikate des Systems und ihre eigenen zusammen, für die Werkzeuge, die nur eine Datei lesen (SSL_CERT_FILE, REQUESTS_CA_BUNDLE, PIP_CERT, CURL_CA_BUNDLE, GIT_SSL_CAINFO, CARGO_HTTP_CAINFO). Eine PEM-Datei im Kit wird relativ zum Kit gespeichert: Sie folgt ihm auf ein anderes Laufwerk oder in einen anderen Ordner.",
            ),
            tr(
              "Anche i download di Atollo usano queste impostazioni: il «Proxy HTTPS» (se vuoto il «Proxy HTTP», altrimenti le variabili del PC o il proxy di sistema), gli host di «Senza proxy per» raggiunti direttamente, e i certificati del sistema più quelli del file PEM. Se il certificato aziendale è già installato nel PC, il file PEM non serve. Un file senza certificati PEM viene rifiutato.",
              "Atollo's own downloads use these settings too: the “HTTPS proxy” (if empty the “HTTP proxy”, otherwise the PC's variables or the system proxy), the “No proxy for” hosts reached directly, and the system's certificates plus those of the PEM file. If the company certificate is already installed on the PC, the PEM file is not needed. A file with no PEM certificate is refused.",
              "Les téléchargements d'Atollo utilisent aussi ces paramètres : le « Proxy HTTPS » (s'il est vide le « Proxy HTTP », sinon les variables du PC ou le proxy du système), les hôtes de « Pas de proxy pour » joints directement, et les certificats du système plus ceux du fichier PEM. Si le certificat de l'entreprise est déjà installé sur le PC, le fichier PEM n'est pas nécessaire. Un fichier sans certificat PEM est refusé.",
              "Auch die Downloads von Atollo verwenden diese Einstellungen: den „HTTPS-Proxy“ (wenn leer den „HTTP-Proxy“, sonst die Variablen des PCs oder den System-Proxy), die Hosts von „Kein Proxy für“ direkt, und die Zertifikate des Systems plus die der PEM-Datei. Ist das Firmenzertifikat schon auf dem PC installiert, braucht es die PEM-Datei nicht. Eine Datei ohne PEM-Zertifikat wird abgelehnt.",
            ),
            tr(
              "I file PAC (configurazione automatica del proxy) non sono supportati: chiedi l'indirizzo del proxy.",
              "PAC files (automatic proxy configuration) are not supported: ask for the proxy address.",
              "Les fichiers PAC (configuration automatique du proxy) ne sont pas pris en charge : demandez l'adresse du proxy.",
              "PAC-Dateien (automatische Proxy-Konfiguration) werden nicht unterstützt: Frag nach der Adresse des Proxys.",
            ),
            tr(
              "«Rete locale» imposta ATOLLO_ALLOW_LAN=1: i comandi dei progetti ascoltano su tutte le interfacce (127.0.0.1 nel comando diventa 0.0.0.0), e così il server di file e i pool di Caddy e Apache. Il router di php -S e il server di file accettano allora come Host anche un indirizzo IP; a ogni altro nome che non sia localhost, *.localhost, 127.0.0.1 o [::1] rispondono 421.",
              "“Local network” sets ATOLLO_ALLOW_LAN=1: the projects' commands listen on all interfaces (127.0.0.1 in the command becomes 0.0.0.0), and so do the file server and the Caddy and Apache pools. The php -S router and the file server then also accept an IP address as Host; to any other name that is not localhost, *.localhost, 127.0.0.1 or [::1] they answer 421.",
              "« Réseau local » définit ATOLLO_ALLOW_LAN=1 : les commandes des projets écoutent sur toutes les interfaces (127.0.0.1 dans la commande devient 0.0.0.0), tout comme le serveur de fichiers et les pools de Caddy et Apache. Le routeur de php -S et le serveur de fichiers acceptent alors aussi une adresse IP comme Host ; à tout autre nom que localhost, *.localhost, 127.0.0.1 ou [::1], ils répondent 421.",
              "„Lokales Netzwerk“ setzt ATOLLO_ALLOW_LAN=1: Die Befehle der Projekte lauschen auf allen Schnittstellen (127.0.0.1 im Befehl wird zu 0.0.0.0), ebenso der Dateiserver und die Pools von Caddy und Apache. Der Router von php -S und der Dateiserver akzeptieren dann als Host auch eine IP-Adresse; auf jeden anderen Namen als localhost, *.localhost, 127.0.0.1 oder [::1] antworten sie mit 421.",
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
            "Die Basen und Ergänzungen von „Neues Projekt“ sind Rezepte: Listen von Befehlen, die du lesen und ändern kannst.",
          ),
          tr(
            "Le trovi in Impostazioni, nel riquadro «Framework».",
            "You find them in Settings, in the “Frameworks” panel.",
            "Vous les trouvez dans Paramètres, dans le panneau « Frameworks ».",
            "Du findest sie unter Einstellungen, im Bereich „Frameworks“.",
          ),
          tr(
            "Puoi cambiare una ricetta ufficiale (e ripristinarla) o crearne di tue.",
            "You can change an official recipe (and restore it) or create your own.",
            "Vous pouvez modifier une recette officielle (et la rétablir) ou créer les vôtres.",
            "Du kannst ein offizielles Rezept ändern (und wiederherstellen) oder eigene erstellen.",
          ),
          tr(
            "«Prova una combinazione» mostra che cosa farebbe una base con delle aggiunte, prima di usarla.",
            "“Try a combination” shows what a base with some additions would do, before you use it.",
            "« Essayer une combinaison » montre ce que ferait une base avec des ajouts, avant de l'utiliser.",
            "„Kombination ausprobieren“ zeigt, was eine Basis mit Ergänzungen tun würde, bevor du sie verwendest.",
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
                "Nessuno, MySQL o MongoDB. La password che la procedura genera per il database vince su una vuota della base (per esempio DB_PASSWORD= in .env.example).",
                "None, MySQL or MongoDB. The password the wizard generates for the database wins over an empty one from the base (for example DB_PASSWORD= in .env.example).",
                "Aucune, MySQL ou MongoDB. Le mot de passe que l'assistant génère pour la base de données l'emporte sur un mot de passe vide de la base (par exemple DB_PASSWORD= dans .env.example).",
                "Keine, MySQL oder MongoDB. Das Passwort, das der Assistent für die Datenbank erzeugt, hat Vorrang vor einem leeren aus der Basis (zum Beispiel DB_PASSWORD= in .env.example).",
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
              "Wähle im Baum eine Basis oder eine Ergänzung und bearbeite die Felder.",
            ),
            tr(
              "Le istruzioni si salvano da sole poco meno di un secondo dopo l'ultima modifica: la barra delle schede mostra «Salvataggio…», poi «Salvato.». Una ricetta ufficiale cambiata mostra «Modificata».",
              "The instructions save themselves a little under a second after the last edit: the tab bar shows “Saving…”, then “Saved.”. A changed official recipe shows “Modified”.",
              "Les instructions s'enregistrent toutes seules un peu moins d'une seconde après la dernière modification : la barre des onglets affiche « Enregistrement… », puis « Enregistré. ». Une recette officielle modifiée affiche « Modifiée ».",
              "Die Anweisungen speichern sich knapp eine Sekunde nach der letzten Änderung von selbst: Die Registerkartenleiste zeigt „Wird gespeichert…“, dann „Gespeichert.“. Ein geändertes offizielles Rezept zeigt „Geändert“.",
            ),
            tr(
              "Per tornare alla versione originale usa «Ripristina le istruzioni ufficiali»; una ricetta tua si toglie con «Togli».",
              "To go back to the original version use “Restore the official instructions”; a recipe of your own is removed with “Remove”.",
              "Pour revenir à la version d'origine, utilisez « Rétablir les instructions officielles » ; une recette à vous se retire avec « Retirer ».",
              "Um zur ursprünglichen Version zurückzukehren, verwende „Offizielle Anweisungen wiederherstellen“; ein eigenes Rezept entfernst du mit „Entfernen“.",
            ),
          ]}
        />
      </Section>

      <Section
        title={tr("Prova una combinazione", "Try a combination", "Essayer une combinaison", "Kombination ausprobieren")}
      >
        <P>
          {tr(
            "Scegli una base e delle aggiunte: la scheda mostra il piano completo che la procedura eseguirebbe (strumenti, comandi, database, .env, azioni), con le istruzioni come sono adesso, anche se l'ultima modifica non è ancora salvata. Non esegue niente.",
            "Choose a base and some additions: the tab shows the complete plan the wizard would run (tools, commands, database, .env, actions), with the instructions as they are now, even if the last edit is not saved yet. It runs nothing.",
            "Choisissez une base et des ajouts : l'onglet montre le plan complet que l'assistant exécuterait (outils, commandes, base de données, .env, actions), avec les instructions telles qu'elles sont maintenant, même si la dernière modification n'est pas encore enregistrée. Il n'exécute rien.",
            "Wähle eine Basis und Ergänzungen: Die Registerkarte zeigt den vollständigen Plan, den der Assistent ausführen würde (Werkzeuge, Befehle, Datenbank, .env, Aktionen), mit den Anweisungen, wie sie jetzt sind, auch wenn die letzte Änderung noch nicht gespeichert ist. Sie führt nichts aus.",
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
              "Deine Änderungen liegen in data\\app\\frameworks.json, die nur die Unterschiede zu den offiziellen Rezepten enthält.",
            ),
            tr(
              "Le basi salvate da un progetto con «Salva come base» sono nel gruppo «Le mie basi» (category «mine»): al posto dei comandi di «Crea il progetto» hanno il campo template, la cartella di data\\frameworks\\mine da copiare. Una base deve avere i comandi di creazione oppure un template valido. Togliendone una con «Togli», al salvataggio se ne va anche la sua cartella.",
              "The bases saved from a project with “Save as a base” are in the “My bases” group (category “mine”): instead of the “Create the project” commands they have the template field, the folder of data\\frameworks\\mine to copy. A base must have the creation commands or a valid template. Removing one with “Remove”, its folder goes too at the save.",
              "Les bases enregistrées depuis un projet avec « Enregistrer comme base » sont dans le groupe « Mes bases » (category « mine ») : à la place des commandes de « Créer le projet », elles ont le champ template, le dossier de data\\frameworks\\mine à copier. Une base doit avoir les commandes de création ou un template valide. Quand on en retire une avec « Retirer », son dossier s'en va aussi à l'enregistrement.",
              "Die mit „Als Basis speichern“ aus einem Projekt gespeicherten Basen stehen in der Gruppe „Meine Basen“ (category „mine“): Statt der Befehle von „Projekt erstellen“ haben sie das Feld template, den Ordner von data\\frameworks\\mine, der kopiert wird. Eine Basis braucht die Erstellungsbefehle oder ein gültiges template. Entfernst du eine mit „Entfernen“, verschwindet beim Speichern auch ihr Ordner.",
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
