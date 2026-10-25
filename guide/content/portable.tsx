// Chapter 7 of the Guide, «Portare il kit con sé»: removing the drive,
// moving the kit, updates and backups.
import { rich, tr } from "../core/lang";
import {
  Bullets,
  Callout,
  Code,
  Flow,
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
} from "../core/widgets";
import type { TopicTexts } from "../core/types";

export const PORTABLE: TopicTexts = {
  eject: () => (
    <>
      <InBrief
        points={[
          tr(
            "Prima di staccare la chiavetta o il disco del kit, usa la pagina «Rimuovi l'unità».",
            "Before unplugging the kit's USB stick or drive, use the “Remove drive” page.",
            "Avant de débrancher la clé USB ou le disque du kit, utilisez la page « Retirer le lecteur ».",
            "Bevor du den USB-Stick oder das Laufwerk des Kits abziehst, verwende die Seite „Laufwerk entfernen“.",
          ),
          tr(
            "Chiude con ordine tutto ciò che il kit ha aperto: progetti, server, editor, console, database.",
            "It neatly closes everything the kit has opened: projects, servers, editor, consoles, databases.",
            "Elle ferme proprement tout ce que le kit a ouvert : projets, serveurs, éditeur, consoles, bases de données.",
            "Sie schließt geordnet alles, was das Kit geöffnet hat: Projekte, Server, Editor, Konsolen, Datenbanken.",
          ),
          tr(
            "Poi chiudi Atollo e rimuovi l'unità con la «Rimozione sicura dell'hardware» di Windows.",
            "Then close Atollo and remove the drive with Windows' “Safely Remove Hardware”.",
            "Fermez ensuite Atollo et retirez le lecteur avec le « Retirer le périphérique en toute sécurité » de Windows.",
            "Schließe dann Atollo und entferne das Laufwerk mit „Hardware sicher entfernen“ von Windows.",
          ),
        ]}
      />

      <GuideShot
        src="eject.webp"
        title={tr(
          "La pagina «Rimuovi l'unità»",
          "The “Remove drive” page",
          "La page « Retirer le lecteur »",
          "Die Seite „Laufwerk entfernen“",
        )}
        caption={tr(
          "Che cosa è aperto adesso e il pulsante che chiude tutto prima di staccare l'unità.",
          "What is open right now and the button that closes everything before you unplug the drive.",
          "Ce qui est ouvert maintenant et le bouton qui ferme tout avant de débrancher le lecteur.",
          "Was gerade geöffnet ist und die Schaltfläche, die alles schließt, bevor du das Laufwerk abziehst.",
        )}
      />

      <Section title={tr("Perché serve", "Why it matters", "Pourquoi c'est utile", "Warum das nötig ist")}>
        <P>
          {tr(
            "Un database spento di colpo può perdere gli ultimi dati scritti, e un programma che ha un file aperto sulla chiavetta impedisce a Windows di rimuoverla. La pagina chiude tutto nel modo giusto: i database con il loro comando di arresto, i server e i programmi uno per uno. E salva nel kit l'accesso di Antigravity, così sul PC non resta niente.",
            "A database switched off abruptly can lose the last data written, and a program with a file open on the USB stick prevents Windows from removing it. The page closes everything the right way: the databases with their shutdown command, the servers and the programs one by one. And it saves the Antigravity sign-in in the kit, so nothing is left on the PC.",
            "Une base de données arrêtée brutalement peut perdre les dernières données écrites, et un programme qui a un fichier ouvert sur la clé USB empêche Windows de la retirer. La page ferme tout comme il faut : les bases de données avec leur commande d'arrêt, les serveurs et les programmes un par un. Et elle enregistre dans le kit la connexion d'Antigravity, ainsi rien ne reste sur le PC.",
            "Eine plötzlich abgeschaltete Datenbank kann die zuletzt geschriebenen Daten verlieren, und ein Programm mit einer geöffneten Datei auf dem USB-Stick hindert Windows daran, ihn zu entfernen. Die Seite schließt alles auf die richtige Weise: die Datenbanken mit ihrem Befehl zum Herunterfahren, die Server und die Programme einzeln. Und sie speichert die Antigravity-Anmeldung im Kit, damit auf dem PC nichts zurückbleibt.",
          )}
        </P>
      </Section>

      <Section title={tr("Passo per passo", "Step by step", "Pas à pas", "Schritt für Schritt")}>
        <Steps
          steps={[
            tr(
              "Salva il lavoro aperto nell'editor e negli altri programmi.",
              "Save the open work in the editor and the other programs.",
              "Enregistrez le travail ouvert dans l'éditeur et les autres programmes.",
              "Speichere die offene Arbeit im Editor und in den anderen Programmen.",
            ),
            tr(
              "Apri «Rimuovi l'unità» (l'ultima icona in basso nella barra delle attività). La parte «Aperti adesso» elenca che cosa è aperto: installazione in corso, editor, console, progetti avviati, database avviati, il server dei modelli locali e i programmi del kit avviati fuori dall'app, con nome e PID.",
              "Open “Remove drive” (the last icon at the bottom of the activity bar). The “Open now” part lists what is open: installation in progress, editor, consoles, running projects, running databases, the local models server and the kit programs started outside the app, with name and PID.",
              "Ouvrez « Retirer le lecteur » (la dernière icône en bas de la barre d'activité). La partie « Ouverts actuellement » liste ce qui est ouvert : installation en cours, éditeur, consoles, projets lancés, bases de données démarrées, le serveur des modèles locaux et les programmes du kit lancés hors de l'application, avec nom et PID.",
              "Öffne „Laufwerk entfernen“ (das letzte Symbol unten in der Aktivitätsleiste). Der Teil „Derzeit geöffnet“ listet auf, was geöffnet ist: laufende Installation, Editor, Konsolen, gestartete Projekte, gestartete Datenbanken, der Server der lokalen Modelle und die außerhalb der App gestarteten Programme des Kits, mit Name und PID.",
            ),
            tr(
              "Premi «Chiudi tutto e prepara la rimozione».",
              "Press “Close everything and prepare for removal”.",
              "Appuyez sur « Tout fermer et préparer le retrait ».",
              "Klicke auf „Alles schließen und Entfernen vorbereiten“.",
            ),
            tr(
              "Quando compare «Programmi del kit chiusi e accesso di Antigravity salvato nel kit.», premi «Chiudi Atollo». Se qualcosa è rimasto aperto, è elencato sotto «Ancora aperti (chiudili a mano)».",
              "When “Kit programs closed and Antigravity sign-in saved in the kit.” appears, press “Close Atollo”. If something is still open, it is listed under “Still open (close them manually)”.",
              "Quand « Programmes du kit fermés et connexion d'Antigravity enregistrée dans le kit. » apparaît, appuyez sur « Fermer Atollo ». Si quelque chose est resté ouvert, c'est listé sous « Encore ouverts (fermez-les manuellement) ».",
              "Wenn „Programme des Kits geschlossen und Antigravity-Anmeldung im Kit gespeichert.“ erscheint, klicke auf „Atollo schließen“. Wenn etwas geöffnet geblieben ist, steht es unter „Noch geöffnet (schließe sie manuell)“.",
            ),
            tr(
              "Rimuovi l'unità dal sistema: su Windows con l'icona «Rimozione sicura dell'hardware»; su macOS espellendola dal Finder.",
              "Remove the drive from the system: on Windows with the “Safely Remove Hardware” icon; on macOS by ejecting it from the Finder.",
              "Retirez le lecteur du système : sur Windows avec l'icône « Retirer le périphérique en toute sécurité » ; sur macOS en l'éjectant depuis le Finder.",
              "Entferne das Laufwerk über das System: unter Windows mit dem Symbol „Hardware sicher entfernen“; unter macOS, indem du es im Finder auswirfst.",
            ),
          ]}
        />
        <Callout tone="warn">
          {tr(
            "I file non salvati nei programmi elencati andranno persi. La pagina non espelle l'unità da sola: l'ultimo passo lo fai tu dal sistema.",
            "Unsaved files in the listed programs will be lost. The page does not eject the drive by itself: you do the last step from the system.",
            "Les fichiers non enregistrés dans les programmes listés seront perdus. La page n'éjecte pas le lecteur toute seule : c'est vous qui faites la dernière étape depuis le système.",
            "Nicht gespeicherte Dateien in den aufgelisteten Programmen gehen verloren. Die Seite wirft das Laufwerk nicht selbst aus: Den letzten Schritt machst du über das System.",
          )}
        </Callout>
        <Go page="eject">
          {tr(
            "Apri «Rimuovi l'unità»",
            "Open “Remove drive”",
            "Ouvrir « Retirer le lecteur »",
            "„Laufwerk entfernen“ öffnen",
          )}
        </Go>
      </Section>

      <Section
        title={tr(
          "E se chiudo semplicemente la finestra?",
          "And if I just close the window?",
          "Et si je ferme simplement la fenêtre ?",
          "Und wenn ich einfach das Fenster schließe?",
        )}
      >
        <P>
          {tr(
            "Se c'è qualcosa di aperto, Atollo chiede «Chiudere Atollo?» ed elenca che cosa si chiuderà. «Chiudi tutto» ferma azioni e server, spegne i database con ordine ed esce. È sicuro anche questo; la pagina «Rimuovi l'unità» in più chiude i programmi del kit avviati fuori dall'app (per esempio da una console) e ti dice se resta qualcosa.",
            "If something is open, Atollo asks “Close Atollo?” and lists what will be closed. “Close everything” stops actions and servers, shuts down the databases neatly and exits. This is safe too; the “Remove drive” page also closes the kit programs started outside the app (for example from a console) and tells you if anything is left.",
            "Si quelque chose est ouvert, Atollo demande « Fermer Atollo ? » et liste ce qui va être fermé. « Tout fermer » arrête les actions et les serveurs, éteint proprement les bases de données et quitte. C'est sûr aussi ; la page « Retirer le lecteur » ferme en plus les programmes du kit lancés hors de l'application (par exemple depuis une console) et vous dit s'il reste quelque chose.",
            "Wenn etwas geöffnet ist, fragt Atollo „Atollo schließen?“ und listet auf, was geschlossen wird. „Alles schließen“ stoppt Aktionen und Server, fährt die Datenbanken geordnet herunter und beendet sich. Auch das ist sicher; die Seite „Laufwerk entfernen“ schließt zusätzlich die außerhalb der App gestarteten Programme des Kits (zum Beispiel aus einer Konsole) und sagt dir, ob etwas übrig bleibt.",
          )}
        </P>
        <P>
          {tr(
            "Lo stesso vale per «Chiudi Atollo» nel menu dell'icona di Atollo, nell'area di notifica di Windows, nella barra dei menu di macOS o nell'area di stato di Linux. Quel menu ha anche «Mostra Atollo» e «Progetti», con i progetti più recenti: un clic apre la pagina del progetto.",
            "The same goes for “Close Atollo” in the menu of Atollo's icon, in the Windows notification area, the macOS menu bar or the Linux status area. That menu also has “Show Atollo” and “Projects”, with the most recent projects: a click opens the project's page.",
            "Il en va de même pour « Fermer Atollo » dans le menu de l'icône d'Atollo, dans la zone de notification de Windows, la barre des menus de macOS ou la zone d'état de Linux. Ce menu propose aussi « Afficher Atollo » et « Projets », avec les projets les plus récents : un clic ouvre la page du projet.",
            "Dasselbe gilt für „Atollo schließen“ im Menü des Atollo-Symbols, im Infobereich von Windows, in der Menüleiste von macOS oder im Statusbereich von Linux. Dieses Menü hat auch „Atollo anzeigen“ und „Projekte“ mit den zuletzt verwendeten Projekten: Ein Klick öffnet die Seite des Projekts.",
          )}
        </P>
      </Section>

      <Under>
        <P>
          {tr(
            "L'ordine con cui la pagina chiude le cose:",
            "The order in which the page closes things:",
            "L'ordre dans lequel la page ferme les choses :",
            "Die Reihenfolge, in der die Seite die Dinge schließt:",
          )}
        </P>
        <Flow
          steps={[
            {
              title: tr("Installazione", "Installation", "Installation", "Installation"),
              text: tr(
                "Si ferma alla fine del passo in corso.",
                "It stops at the end of the current step.",
                "Elle s'arrête à la fin de l'étape en cours.",
                "Sie hält am Ende des laufenden Schritts an.",
              ),
            },
            {
              title: tr("Progetti", "Projects", "Projets", "Projekte"),
              text: tr(
                "Le azioni avviate.",
                "The running actions.",
                "Les actions lancées.",
                "Die gestarteten Aktionen.",
              ),
            },
            {
              title: tr("Server web", "Web servers", "Serveurs web", "Webserver"),
              text: tr(
                "I pool di Caddy e Apache.",
                "The Caddy and Apache pools.",
                "Les pools de Caddy et Apache.",
                "Die Pools von Caddy und Apache.",
              ),
            },
            {
              title: tr("Editor", "Editor", "Éditeur", "Editor"),
              text: tr(
                "Le finestre; il server dell'editor si ferma subito dopo i database.",
                "The windows; the editor server stops right after the databases.",
                "Les fenêtres ; le serveur de l'éditeur s'arrête juste après les bases de données.",
                "Die Fenster; der Server des Editors stoppt direkt nach den Datenbanken.",
              ),
            },
            {
              title: tr("Database", "Databases", "Bases de données", "Datenbanken"),
              text: tr(
                "Arresto ordinato di MySQL e MongoDB, ogni versione avviata.",
                "Orderly shutdown of MySQL and MongoDB, every running version.",
                "Arrêt propre de MySQL et MongoDB, chaque version démarrée.",
                "Geordnetes Herunterfahren von MySQL und MongoDB, jeder gestarteten Version.",
              ),
            },
            {
              title: tr("VM", "VM", "VM", "VM"),
              text: tr(
                "Docker Sandboxes del kit si ferma, e con lui le microVM, i cui dischi sono nel kit.",
                "The kit's Docker Sandboxes stops, and with it the microVMs, whose disks are in the kit.",
                "Le Docker Sandboxes du kit s'arrête, et avec lui les microVM, dont les disques sont dans le kit.",
                "Das Docker Sandboxes des Kits wird beendet, und mit ihm die microVMs, deren Festplatten im Kit liegen.",
              ),
            },
            {
              title: tr("Il resto", "The rest", "Le reste", "Der Rest"),
              text: tr(
                "Console e ogni processo con il programma dentro il kit.",
                "Consoles and every process whose program is inside the kit.",
                "Les consoles et chaque processus dont le programme est dans le kit.",
                "Konsolen und jeder Prozess, dessen Programm im Kit liegt.",
              ),
            },
            {
              title: tr("Antigravity", "Antigravity", "Antigravity", "Antigravity"),
              text: tr(
                "L'accesso torna nel kit, il sistema torna com'era.",
                "The sign-in goes back into the kit, the system goes back to how it was.",
                "La connexion revient dans le kit, le système redevient comme avant.",
                "Die Anmeldung kehrt ins Kit zurück, das System wird wieder wie vorher.",
              ),
            },
          ]}
        />
        <P>
          {tr(
            "Il kit non trova i programmi del PC che hanno aperto un file della chiavetta (per esempio un documento aperto in un'app esterna): chiudili tu.",
            "The kit cannot find the PC's programs that have opened a file on the USB stick (for example a document open in an outside app): close them yourself.",
            "Le kit ne trouve pas les programmes du PC qui ont ouvert un fichier de la clé USB (par exemple un document ouvert dans une application externe) : fermez-les vous-même.",
            "Das Kit findet keine Programme des PCs, die eine Datei des USB-Sticks geöffnet haben (zum Beispiel ein Dokument in einer externen App): Schließe sie selbst.",
          )}
        </P>
      </Under>

      <Related ids={["move", "lifecycle", "updates"]} />
    </>
  ),

  move: () => (
    <>
      <InBrief
        points={[
          tr(
            "Puoi spostare o copiare la cartella del kit dove vuoi: un'altra lettera, un'altra cartella, un altro PC.",
            "You can move or copy the kit folder wherever you like: another drive letter, another folder, another PC.",
            "Vous pouvez déplacer ou copier le dossier du kit où vous voulez : une autre lettre, un autre dossier, un autre PC.",
            "Du kannst den Kit-Ordner verschieben oder kopieren, wohin du willst: ein anderer Laufwerksbuchstabe, ein anderer Ordner, ein anderer PC.",
          ),
          tr(
            "Al primo avvio nel nuovo posto il kit se ne accorge e sistema da solo i percorsi.",
            "At the first start in the new place the kit notices and fixes the paths by itself.",
            "Au premier démarrage dans le nouvel emplacement, le kit s'en aperçoit et corrige tout seul les chemins.",
            "Beim ersten Start am neuen Ort bemerkt das Kit das und passt die Pfade selbst an.",
          ),
          tr(
            "La stessa chiavetta si apre su Windows, macOS e Linux: progetti e dati sono in comune, i programmi sono separati per sistema.",
            "The same USB stick opens on Windows, macOS and Linux: projects and data are shared, the programs are kept separate for each system.",
            "La même clé USB s'ouvre sur Windows, macOS et Linux : projets et données sont communs, les programmes sont séparés par système.",
            "Derselbe USB-Stick öffnet sich unter Windows, macOS und Linux: Projekte und Daten sind gemeinsam, die Programme sind nach System getrennt.",
          ),
        ]}
      />

      <Section
        title={tr(
          "Cambiare lettera, cartella o PC",
          "Changing letter, folder or PC",
          "Changer de lettre, de dossier ou de PC",
          "Buchstabe, Ordner oder PC wechseln",
        )}
      >
        <P>
          {tr(
            "Su un altro PC la chiavetta potrebbe diventare E: invece di D:. Non devi fare niente: all'avvio il kit confronta la cartella attuale con quella dell'ultima volta e, se è cambiata, mostra «Il kit è stato spostato: prima era in …, ora in …» e aggiorna i percorsi.",
            "On another PC the USB stick might become E: instead of D:. You do not have to do anything: at start the kit compares the current folder with the one from last time and, if it changed, shows “The kit has been moved: it was in …, it is now in …” and updates the paths.",
            "Sur un autre PC, la clé USB pourrait devenir E: au lieu de D:. Vous n'avez rien à faire : au démarrage, le kit compare le dossier actuel avec celui de la dernière fois et, s'il a changé, affiche « Le kit a été déplacé : il était dans …, il est maintenant dans … » et met à jour les chemins.",
            "Auf einem anderen PC könnte der USB-Stick zu E: statt D: werden. Du musst nichts tun: Beim Start vergleicht das Kit den aktuellen Ordner mit dem vom letzten Mal und zeigt, wenn er sich geändert hat, „Das Kit wurde verschoben: Es lag in …, jetzt liegt es in …“ und aktualisiert die Pfade.",
          )}
        </P>
        <Bullets
          items={[
            tr(
              "L'ambiente dei programmi (PATH e variabili) si ricostruisce a ogni avvio dalla cartella attuale.",
              "The programs' environment (PATH and variables) is rebuilt at every start from the current folder.",
              "L'environnement des programmes (PATH et variables) est reconstruit à chaque démarrage à partir du dossier actuel.",
              "Die Umgebung der Programme (PATH und Variablen) wird bei jedem Start aus dem aktuellen Ordner neu aufgebaut.",
            ),
            tr(
              "I percorsi scritti nei file degli agenti e dell'editor vengono riscritti con il nuovo percorso.",
              "The paths written in the agents' and the editor's files are rewritten with the new path.",
              "Les chemins écrits dans les fichiers des agents et de l'éditeur sont réécrits avec le nouveau chemin.",
              "Die in den Dateien der Agenten und des Editors geschriebenen Pfade werden mit dem neuen Pfad neu geschrieben.",
            ),
            tr(
              "Le configurazioni dei server web si rigenerano a ogni avvio e usano percorsi relativi.",
              "The web servers' configurations are regenerated at every start and use relative paths.",
              "Les configurations des serveurs web sont régénérées à chaque démarrage et utilisent des chemins relatifs.",
              "Die Konfigurationen der Webserver werden bei jedem Start neu erzeugt und verwenden relative Pfade.",
            ),
            tr(
              "Nel .gitconfig del kit, safe.directory (le cartelle di cui Git si fida) viene riscritto a ogni avvio con la cartella attuale del kit, per esempio D:/Atollo/*. Il kit toglie solo il valore che aveva scritto prima (lo ricorda in data\\app\\safe-directory.txt): le cartelle che hai aggiunto tu restano.",
              "In the kit's .gitconfig, safe.directory (the folders Git trusts) is rewritten at every start with the kit's current folder, for example D:/Atollo/*. The kit removes only the value it wrote before (it remembers it in data\\app\\safe-directory.txt): the folders you added yourself stay.",
              "Dans le .gitconfig du kit, safe.directory (les dossiers auxquels Git fait confiance) est réécrit à chaque démarrage avec le dossier actuel du kit, par exemple D:/Atollo/*. Le kit ne retire que la valeur qu'il avait écrite avant (il la garde dans data\\app\\safe-directory.txt) : les dossiers que vous avez ajoutés vous-même restent.",
              "In der .gitconfig des Kits wird safe.directory (die Ordner, denen Git vertraut) bei jedem Start mit dem aktuellen Ordner des Kits neu geschrieben, zum Beispiel D:/Atollo/*. Das Kit entfernt nur den Wert, den es vorher geschrieben hat (es merkt ihn sich in data\\app\\safe-directory.txt): Ordner, die du selbst hinzugefügt hast, bleiben.",
            ),
            tr(
              "Il file PEM dei certificati (Impostazioni › Rete) e la cartella di un terminale, se sono dentro il kit, sono salvati relativi al kit: seguono lo spostamento.",
              "The certificates' PEM file (Settings › Network) and a terminal's folder, when they are inside the kit, are saved relative to the kit: they follow the move.",
              "Le fichier PEM des certificats (Paramètres › Réseau) et le dossier d'un terminal, s'ils sont dans le kit, sont enregistrés relativement au kit : ils suivent le déplacement.",
              "Die PEM-Datei der Zertifikate (Einstellungen › Netzwerk) und der Ordner eines Terminals werden, wenn sie im Kit liegen, relativ zum Kit gespeichert: Sie folgen dem Verschieben.",
            ),
            tr(
              "Accessi, impostazioni, progetti e database sono nella cartella: viaggiano con lei.",
              "Sign-ins, settings, projects and databases are in the folder: they travel with it.",
              "Connexions, paramètres, projets et bases de données sont dans le dossier : ils voyagent avec lui.",
              "Anmeldungen, Einstellungen, Projekte und Datenbanken liegen im Ordner: Sie reisen mit ihm.",
            ),
            tr(
              "Anche le sandbox delle VM sono nel kit: quella di un progetto si rifà da sola nella cartella nuova, con una riga nel log. File, database, strumenti e accessi restano; si perdono solo la home della VM e le immagini scaricate.",
              "The VM sandboxes are in the kit too: a project's one is made again by itself in the new folder, with a line in the log. Files, databases, tools and sign-ins stay; only the VM's home and the downloaded images are lost.",
              "Les sandbox des VM sont aussi dans le kit : celle d'un projet est refaite toute seule dans le nouveau dossier, avec une ligne dans le journal. Fichiers, bases de données, outils et connexions restent ; seuls le dossier personnel de la VM et les images téléchargées sont perdus.",
              "Auch die Sandboxes der VMs liegen im Kit: Die eines Projekts wird im neuen Ordner von selbst neu angelegt, mit einer Zeile im Log. Dateien, Datenbanken, Werkzeuge und Anmeldungen bleiben; verloren gehen nur das Home der VM und die heruntergeladenen Images.",
            ),
            tr(
              "Su un altro PC la VM chiede di nuovo ciò che non viaggia con il kit: l'accesso a Docker (sbx login), l'hypervisor attivo e, su Ubuntu, il profilo AppArmor, da rifare anche quando il kit cambia cartella. L'avviso «Senza VM» dice che cosa manca.",
              "On another PC the VM asks again for what does not travel with the kit: the Docker sign-in (sbx login), the hypervisor on and, on Ubuntu, the AppArmor profile, to be redone also when the kit changes folder. The “No VM” notice says what is missing.",
              "Sur un autre PC, la VM redemande ce qui ne voyage pas avec le kit : la connexion à Docker (sbx login), l'hyperviseur activé et, sur Ubuntu, le profil AppArmor, à refaire aussi quand le kit change de dossier. L'avis « Sans VM » dit ce qui manque.",
              "Auf einem anderen PC verlangt die VM erneut, was nicht mit dem Kit reist: die Anmeldung bei Docker (sbx login), den eingeschalteten Hypervisor und unter Ubuntu das AppArmor-Profil, das auch neu nötig ist, wenn das Kit den Ordner wechselt. Der Hinweis „Ohne VM“ sagt, was fehlt.",
            ),
          ]}
        />
        <Callout tone="info">
          {tr(
            "Si può aprire un kit alla volta per cartella: avviare di nuovo Atollo dalla stessa cartella porta in primo piano la finestra già aperta. Due kit diversi (due chiavette) possono invece girare insieme.",
            "Only one kit at a time can be open per folder: starting Atollo again from the same folder brings the window already open to the front. Two different kits (two USB sticks), on the other hand, can run together.",
            "On ne peut ouvrir qu'un kit à la fois par dossier : relancer Atollo depuis le même dossier met au premier plan la fenêtre déjà ouverte. Deux kits différents (deux clés USB) peuvent en revanche tourner ensemble.",
            "Pro Ordner kann jeweils nur ein Kit geöffnet sein: Wenn du Atollo aus demselben Ordner erneut startest, wird das bereits geöffnete Fenster in den Vordergrund geholt. Zwei verschiedene Kits (zwei USB-Sticks) können dagegen gleichzeitig laufen.",
          )}
        </Callout>
        <P>
          {rich(
            tr(
              "Per portare solo il tuo lavoro in un kit che c'è già, senza i programmi, usa «Copia del kit» in Impostazioni › Il kit: esporta in uno zip da qui, importa di là ({0}).",
              "To bring only your work into a kit that already exists, without the programs, use “Copy of the kit” in Settings › The kit: export to a zip here, import there ({0}).",
              "Pour apporter seulement votre travail dans un kit qui existe déjà, sans les programmes, utilisez « Copie du kit » dans Paramètres › Le kit : exportez dans un zip ici, importez là-bas ({0}).",
              "Um nur deine Arbeit in ein bereits vorhandenes Kit zu bringen, ohne die Programme, verwende „Kopie des Kits“ unter Einstellungen › Das Kit: hier in ein Zip exportieren, dort importieren ({0}).",
            ),
            <See key="b" id="updates" />,
          )}
        </P>
      </Section>

      <Section
        title={tr(
          "Lo stesso kit su Windows, macOS e Linux",
          "The same kit on Windows, macOS and Linux",
          "Le même kit sur Windows, macOS et Linux",
          "Dasselbe Kit unter Windows, macOS und Linux",
        )}
      >
        <P>
          {tr(
            "I programmi sono diversi per ogni sistema, e il kit li tiene in cartelle separate (data\\apps\\win32-x64, data\\apps\\darwin-arm64, data\\apps\\linux-x64…). Tutto il resto è in comune: progetti, impostazioni, accessi, istruzioni degli agenti, database.",
            "The programs are different for each system, and the kit keeps them in separate folders (data\\apps\\win32-x64, data\\apps\\darwin-arm64, data\\apps\\linux-x64…). Everything else is shared: projects, settings, sign-ins, agent instructions, databases.",
            "Les programmes sont différents pour chaque système, et le kit les garde dans des dossiers séparés (data\\apps\\win32-x64, data\\apps\\darwin-arm64, data\\apps\\linux-x64…). Tout le reste est commun : projets, paramètres, connexions, instructions des agents, bases de données.",
            "Die Programme sind für jedes System verschieden, und das Kit hält sie in getrennten Ordnern (data\\apps\\win32-x64, data\\apps\\darwin-arm64, data\\apps\\linux-x64…). Alles andere ist gemeinsam: Projekte, Einstellungen, Anmeldungen, Anweisungen der Agenten, Datenbanken.",
          )}
        </P>
        <Steps
          steps={[
            tr(
              "Formatta la chiavetta in exFAT, che tutti e tre i sistemi leggono e scrivono. Su exFAT però la VM dei progetti non parte («Senza VM»): per usarla tieni il kit su NTFS, APFS o ext4.",
              "Format the USB stick as exFAT, which all three systems can read and write. On exFAT, though, the projects' VM does not start (“No VM”): to use it keep the kit on NTFS, APFS or ext4.",
              "Formatez la clé USB en exFAT, que les trois systèmes savent lire et écrire. Sur exFAT, cependant, la VM des projets ne démarre pas (« Sans VM ») : pour l'utiliser, gardez le kit sur NTFS, APFS ou ext4.",
              "Formatiere den USB-Stick mit exFAT, das alle drei Systeme lesen und schreiben können. Auf exFAT startet die VM der Projekte allerdings nicht („Ohne VM“): Um sie zu nutzen, lass das Kit auf NTFS, APFS oder ext4.",
            ),
            tr(
              "Metti nella cartella del kit l'eseguibile di ciascun sistema (Atollo.exe, Atollo.app, il file AppImage).",
              "Put the executable of each system in the kit folder (Atollo.exe, Atollo.app, the AppImage file).",
              "Placez dans le dossier du kit l'exécutable de chaque système (Atollo.exe, Atollo.app, le fichier AppImage).",
              "Leg die ausführbare Datei jedes Systems in den Kit-Ordner (Atollo.exe, Atollo.app, die AppImage-Datei).",
            ),
            tr(
              "Su ogni sistema, apri Atollo: alla prima apertura installa i componenti scelti per quel sistema. In alternativa, dalla pagina Strumenti scegli quel sistema in «Piattaforma» e prepara i programmi in anticipo da un altro PC.",
              "On each system, open Atollo: the first time it installs the components chosen for that system. Alternatively, on the Tools page choose that system in “Platform” and prepare the programs in advance from another PC.",
              "Sur chaque système, ouvrez Atollo : à la première ouverture, il installe les composants choisis pour ce système. Sinon, dans la page Outils, choisissez ce système dans « Plateforme » et préparez les programmes à l'avance depuis un autre PC.",
              "Öffne Atollo auf jedem System: Beim ersten Öffnen installiert es die für dieses System gewählten Komponenten. Alternativ wählst du auf der Seite Werkzeuge dieses System unter „Plattform“ und bereitest die Programme vorab auf einem anderen PC vor.",
            ),
          ]}
        />
        <Callout tone="warn">
          {tr(
            "Un database usato da più sistemi va sempre spento prima di cambiare PC (usa «Rimuovi l'unità»). Su macOS e Linux alcuni componenti (R, Ruby, MongoDB su Linux, SQLite su Linux ARM64, make) vanno reinstallati quando il kit cambia cartella: compaiono come «da aggiornare» e il kit li reinstalla da solo, senza rete se hai attivato «Pacchetti per reinstallare senza rete».",
            "A database used from several systems must always be shut down before changing PC (use “Remove drive”). On macOS and Linux some components (R, Ruby, MongoDB on Linux, SQLite on Linux ARM64, make) must be reinstalled when the kit changes folder: they show as “to update” and the kit reinstalls them by itself, without the network if you turned on “Packages to reinstall without the network”.",
            "Une base de données utilisée par plusieurs systèmes doit toujours être arrêtée avant de changer de PC (utilisez « Retirer le lecteur »). Sur macOS et Linux, certains composants (R, Ruby, MongoDB sur Linux, SQLite sur Linux ARM64, make) doivent être réinstallés quand le kit change de dossier : ils apparaissent comme « à mettre à jour » et le kit les réinstalle tout seul, sans réseau si vous avez activé « Paquets pour réinstaller sans réseau ».",
            "Eine Datenbank, die von mehreren Systemen genutzt wird, muss vor dem PC-Wechsel immer heruntergefahren werden (verwende „Laufwerk entfernen“). Unter macOS und Linux müssen einige Komponenten (R, Ruby, MongoDB unter Linux, SQLite unter Linux ARM64, make) neu installiert werden, wenn das Kit den Ordner wechselt: Sie erscheinen als „zu aktualisieren“ und das Kit installiert sie selbst neu, ohne Netzwerk, wenn du „Pakete zum Neuinstallieren ohne Netzwerk“ aktiviert hast.",
          )}
        </Callout>
        <Under>
          <Bullets
            items={[
              tr(
                "L'ultima cartella è in data\\app\\root.txt (root-darwin.txt e root-linux.txt su macOS e Linux); la precedente in old-root.txt finché i file degli agenti non sono stati aggiornati.",
                "The last folder is in data\\app\\root.txt (root-darwin.txt and root-linux.txt on macOS and Linux); the previous one in old-root.txt until the agents' files have been updated.",
                "Le dernier dossier est dans data\\app\\root.txt (root-darwin.txt et root-linux.txt sur macOS et Linux) ; le précédent dans old-root.txt tant que les fichiers des agents n'ont pas été mis à jour.",
                "Der letzte Ordner steht in data\\app\\root.txt (root-darwin.txt und root-linux.txt unter macOS und Linux); der vorherige in old-root.txt, bis die Dateien der Agenten aktualisiert sind.",
              ),
              tr(
                "Su Windows il confronto delle cartelle ignora maiuscole e minuscole; su macOS e Linux no.",
                "On Windows the folder comparison ignores upper and lower case; on macOS and Linux it does not.",
                "Sur Windows, la comparaison des dossiers ignore les majuscules et minuscules ; sur macOS et Linux, non.",
                "Unter Windows ignoriert der Ordnervergleich Groß- und Kleinschreibung; unter macOS und Linux nicht.",
              ),
              tr(
                "Le versioni installate di ogni sistema sono in data\\app\\versions\\<sistema>.",
                "The installed versions of each system are in data\\app\\versions\\<system>.",
                "Les versions installées de chaque système sont dans data\\app\\versions\\<système>.",
                "Die installierten Versionen jedes Systems liegen in data\\app\\versions\\<System>.",
              ),
              tr(
                "macOS: l'app non è autenticata da Apple. Se macOS la apre da una posizione temporanea («App Translocation»), Atollo si ferma e mostra il comando xattr da eseguire per togliere la quarantena. Linux: servono glibc 2.35 o più recente e WebKitGTK 4.1; se mancano, Atollo mostra il comando per installarli.",
                "macOS: the app is not notarized by Apple. If macOS opens it from a temporary location (“App Translocation”), Atollo stops and shows the xattr command to run to remove the quarantine. Linux: glibc 2.35 or newer and WebKitGTK 4.1 are needed; if they are missing, Atollo shows the command to install them.",
                "macOS : l'application n'est pas notarisée par Apple. Si macOS l'ouvre depuis un emplacement temporaire (« App Translocation »), Atollo s'arrête et affiche la commande xattr à exécuter pour retirer la quarantaine. Linux : il faut glibc 2.35 ou plus récent et WebKitGTK 4.1 ; s'ils manquent, Atollo affiche la commande pour les installer.",
                "macOS: Die App ist nicht von Apple beglaubigt. Wenn macOS sie von einem temporären Ort öffnet („App Translocation“), hält Atollo an und zeigt den xattr-Befehl, mit dem du die Quarantäne aufhebst. Linux: Benötigt werden glibc 2.35 oder neuer und WebKitGTK 4.1; fehlen sie, zeigt Atollo den Befehl zum Installieren.",
              ),
            ]}
          />
        </Under>
      </Section>

      <Related ids={["eject", "vm", "isolation", "tools-overview"]} />
    </>
  ),

  updates: () => (
    <>
      <InBrief
        points={[
          tr(
            "Atollo si aggiorna da solo: scarica la nuova versione e la mette al posto della vecchia quando chiudi l'app.",
            "Atollo updates itself: it downloads the new version and puts it in place of the old one when you close the app.",
            "Atollo se met à jour tout seul : il télécharge la nouvelle version et la met à la place de l'ancienne quand vous fermez l'application.",
            "Atollo aktualisiert sich selbst: Es lädt die neue Version herunter und setzt sie an die Stelle der alten, wenn du die App schließt.",
          ),
          tr(
            "Gli strumenti si aggiornano dalla pagina Strumenti, con «Aggiorna <nome>» su una riga o «Aggiorna tutto».",
            "The tools are updated from the Tools page, with “Update <name>” on a row or “Update all”.",
            "Les outils se mettent à jour depuis la page Outils, avec « Mettre à jour <nom> » sur une ligne ou « Tout mettre à jour ».",
            "Die Werkzeuge werden auf der Seite Werkzeuge aktualisiert, mit „<Name> aktualisieren“ in einer Zeile oder „Alle aktualisieren“.",
          ),
          tr(
            "Per una copia di sicurezza completa c'è «Copia del kit» in Impostazioni › Il kit: progetti e dati in una cartella o in uno zip, senza i programmi, che si riscaricano, e l'importazione in un altro kit. Non copia nemmeno immagini e dischi delle VM: le sandbox si rifanno da sole.",
            "For a complete backup there is “Copy of the kit” in Settings › The kit: projects and data into a folder or a zip, without the programs, which are downloaded again, and the import into another kit. It does not copy the VMs' images and disks either: the sandboxes are made again by themselves.",
            "Pour une sauvegarde complète, il y a « Copie du kit » dans Paramètres › Le kit : projets et données dans un dossier ou un zip, sans les programmes, qui se retéléchargent, et l'importation dans un autre kit. Elle ne copie pas non plus les images et les disques des VM : les sandbox sont refaites toutes seules.",
            "Für eine vollständige Sicherung gibt es „Kopie des Kits“ unter Einstellungen › Das Kit: Projekte und Daten in einen Ordner oder ein Zip, ohne die Programme, die sich erneut herunterladen lassen, und den Import in ein anderes Kit. Auch Images und Festplatten der VMs werden nicht kopiert: Die Sandboxes werden von selbst neu angelegt.",
          ),
        ]}
      />

      <Section title={tr("Aggiornare Atollo", "Updating Atollo", "Mettre à jour Atollo", "Atollo aktualisieren")}>
        <P>
          {tr(
            "Con «Aggiornamento automatico» attivo (Impostazioni › Generale), venti secondi dopo l'avvio il kit controlla se c'è una nuova versione, la scarica, ne verifica l'impronta e la prepara. Quando chiudi l'app, la nuova versione prende il posto della vecchia; al successivo avvio la vecchia copia viene cancellata. L'interruttore vale subito: attivarlo avvia un controllo all'istante, spegnerlo prima dei venti secondi ferma quel controllo.",
            "With “Automatic update” on (Settings › General), twenty seconds after start the kit checks whether there is a new version, downloads it, verifies its hash and prepares it. When you close the app, the new version takes the place of the old one; at the next start the old copy is deleted. The switch applies at once: turning it on starts a check right away, turning it off before the twenty seconds stops that check.",
            "Avec « Mise à jour automatique » activée (Paramètres › Général), vingt secondes après le démarrage le kit vérifie s'il y a une nouvelle version, la télécharge, en vérifie l'empreinte et la prépare. Quand vous fermez l'application, la nouvelle version prend la place de l'ancienne ; au démarrage suivant, l'ancienne copie est supprimée. L'interrupteur s'applique tout de suite : l'activer lance aussitôt une vérification, le désactiver avant les vingt secondes annule cette vérification.",
            "Mit aktivem „Automatisches Update“ (Einstellungen › Allgemein) prüft das Kit zwanzig Sekunden nach dem Start, ob es eine neue Version gibt, lädt sie herunter, prüft ihren Hash und bereitet sie vor. Wenn du die App schließt, tritt die neue Version an die Stelle der alten; beim nächsten Start wird die alte Kopie gelöscht. Der Schalter gilt sofort: Einschalten startet sofort eine Prüfung, Ausschalten vor den zwanzig Sekunden stoppt diese Prüfung.",
          )}
        </P>
        <P>
          {tr(
            "In Impostazioni › «Aggiornamenti» vedi lo stato («Atollo … è la versione più recente.», «La versione … è pronta: sostituisce questa alla chiusura di Atollo.») e il pulsante «Cerca aggiornamenti», che controlla e scarica subito anche con l'aggiornamento automatico spento. Lo stesso fa un clic sulla versione, l'ultima voce della barra di stato: accanto compare l'esito, e passandoci sopra lo leggi per intero.",
            "In Settings › “Updates” you see the state (“Atollo … is the latest version.”, “Version … is ready: it replaces this one when Atollo closes.”) and the “Check for updates” button, which checks and downloads right away even with automatic update off. A click on the version, the last item of the status bar, does the same: the result appears beside it, and hovering over it shows it in full.",
            "Dans Paramètres › « Mises à jour », vous voyez l'état (« Atollo … est la version la plus récente. », « La version … est prête : elle remplacera celle-ci à la fermeture d'Atollo. ») et le bouton « Rechercher des mises à jour », qui vérifie et télécharge tout de suite, même avec la mise à jour automatique désactivée. Un clic sur la version, le dernier élément de la barre d'état, fait de même : le résultat s'affiche à côté, et en la survolant vous le lisez en entier.",
            "Unter Einstellungen › „Updates“ siehst du den Stand („Atollo … ist die neueste Version.“, „Version … ist bereit: Sie ersetzt diese beim Schließen von Atollo.“) und die Schaltfläche „Nach Updates suchen“, die sofort prüft und herunterlädt, auch wenn das automatische Update ausgeschaltet ist. Dasselbe macht ein Klick auf die Version, das letzte Element der Statusleiste: Daneben erscheint das Ergebnis, und mit der Maus darüber liest du es vollständig.",
          )}
        </P>
        <Callout tone="info">
          {tr(
            "Su macOS e Linux, se usi il programma semplice (dall'archivio tar.gz) invece di Atollo.app o dell'AppImage, il kit non può sostituirlo: ti dice che la nuova versione è disponibile e la scarichi dalla pagina della release.",
            "On macOS and Linux, if you use the plain program (from the tar.gz archive) instead of Atollo.app or the AppImage, the kit cannot replace it: it tells you the new version is available and you download it from the release page.",
            "Sur macOS et Linux, si vous utilisez le programme simple (de l'archive tar.gz) au lieu d'Atollo.app ou de l'AppImage, le kit ne peut pas le remplacer : il vous dit que la nouvelle version est disponible et vous la téléchargez depuis la page de la release.",
            "Wenn du unter macOS und Linux das einfache Programm (aus dem tar.gz-Archiv) statt Atollo.app oder des AppImage verwendest, kann das Kit es nicht ersetzen: Es teilt dir mit, dass die neue Version verfügbar ist, und du lädst sie von der Seite des Releases herunter.",
          )}
        </Callout>
        <Go page="settings">
          {tr("Apri le Impostazioni", "Open Settings", "Ouvrir les Paramètres", "Einstellungen öffnen")}
        </Go>
      </Section>

      <Section
        title={tr(
          "Aggiornare gli strumenti",
          "Updating the tools",
          "Mettre à jour les outils",
          "Die Werkzeuge aktualisieren",
        )}
      >
        <Bullets
          items={[
            tr(
              "«Aggiorna <nome>» (l'icona con le due frecce) sulla riga di un componente scarica subito la sua versione nuova, se c'è.",
              "“Update <name>” (the icon with the two arrows) on a component's row downloads its new version right away, if there is one.",
              "« Mettre à jour <nom> » (l'icône aux deux flèches) sur la ligne d'un composant télécharge tout de suite sa nouvelle version, s'il y en a une.",
              "„<Name> aktualisieren“ (das Symbol mit den zwei Pfeilen) in der Zeile einer Komponente lädt sofort deren neue Version herunter, falls es eine gibt.",
            ),
            tr(
              "«Aggiorna tutto», in fondo alla pagina Strumenti, aggiorna ogni componente installato e porta ogni versione all'ultima correzione (per esempio PHP 8.4.1 → 8.4.3), senza aggiungere né togliere versioni.",
              "“Update all”, at the bottom of the Tools page, updates every installed component and brings every version to its latest fix (for example PHP 8.4.1 → 8.4.3), without adding or removing versions.",
              "« Tout mettre à jour », en bas de la page Outils, met à jour chaque composant installé et porte chaque version à son dernier correctif (par exemple PHP 8.4.1 → 8.4.3), sans ajouter ni retirer de versions.",
              "„Alle aktualisieren“, unten auf der Seite Werkzeuge, aktualisiert jede installierte Komponente und bringt jede Version auf die letzte Korrektur (zum Beispiel PHP 8.4.1 → 8.4.3), ohne Versionen hinzuzufügen oder zu entfernen.",
            ),
            tr(
              "Gli agenti seguono sempre l'ultima versione e non si aggiornano da soli mentre lavori: lo fa il kit con «Aggiorna <nome>» o «Aggiorna tutto».",
              "The agents always follow the latest version and do not update themselves while you work: the kit does it with “Update <name>” or “Update all”.",
              "Les agents suivent toujours la dernière version et ne se mettent pas à jour tout seuls pendant que vous travaillez : c'est le kit qui le fait avec « Mettre à jour <nom> » ou « Tout mettre à jour ».",
              "Die Agenten folgen immer der neuesten Version und aktualisieren sich nicht selbst, während du arbeitest: Das macht das Kit mit „<Name> aktualisieren“ oder „Alle aktualisieren“.",
            ),
          ]}
        />
        <Callout tone="tip">
          {tr(
            "Aggiorna quando non hai progetti avviati: i programmi in uso non si possono sostituire, e la riga mostrerebbe «in uso: chiudi i programmi del kit e riprova».",
            "Update when you have no projects running: programs in use cannot be replaced, and the row would show “in use: close the kit programs and try again”.",
            "Mettez à jour quand aucun projet n'est lancé : les programmes en cours d'utilisation ne peuvent pas être remplacés, et la ligne afficherait « en cours d'utilisation : fermez les programmes du kit et réessayez ».",
            "Aktualisiere, wenn keine Projekte laufen: Programme in Verwendung lassen sich nicht ersetzen, und die Zeile würde „in Verwendung: Schließe die Programme des Kits und versuche es erneut“ zeigen.",
          )}
        </Callout>
      </Section>

      <Section title={tr("Copie di sicurezza", "Backups", "Sauvegardes", "Sicherungen")}>
        <Steps
          steps={[
            tr(
              "Apri Impostazioni › Il kit › «Copia del kit» e premi «Esporta in uno zip…» (o «Esporta in una cartella…»), poi scegli dove metterla: un disco esterno, una cartella sincronizzata. Non dentro il kit: lì è rifiutata.",
              "Open Settings › The kit › “Copy of the kit” and press “Export to a zip…” (or “Export to a folder…”), then choose where to put it: an external disk, a synced folder. Not inside the kit: there it is refused.",
              "Ouvrez Paramètres › Le kit › « Copie du kit » et appuyez sur « Exporter dans un zip… » (ou « Exporter dans un dossier… »), puis choisissez où la mettre : un disque externe, un dossier synchronisé. Pas dans le kit : elle y est refusée.",
              "Öffne Einstellungen › Das Kit › „Kopie des Kits“ und drücke „In ein Zip exportieren…“ (oder „In einen Ordner exportieren…“), dann wähle, wohin sie kommt: eine externe Festplatte, ein synchronisierter Ordner. Nicht in das Kit: Dort wird sie abgelehnt.",
            ),
            tr(
              "Il kit ferma i server dei database, copia progetti e dati in Atollo-<data-ora> e riavvia quelli con «Avvia con Atollo». Alla fine dice quanti file e quanto spazio: «Copia fatta: … file, …».",
              "The kit stops the database servers, copies projects and data into Atollo-<date-time> and starts again those with “Start with Atollo”. At the end it says how many files and how much space: “Copy made: … files, …”.",
              "Le kit arrête les serveurs de bases de données, copie projets et données dans Atollo-<date-heure> et redémarre ceux avec « Démarrer avec Atollo ». À la fin, il indique combien de fichiers et quelle taille : « Copie faite : … fichiers, … ».",
              "Das Kit hält die Datenbankserver an, kopiert Projekte und Daten nach Atollo-<Datum-Uhrzeit> und startet die mit „Mit Atollo starten“ wieder. Am Ende sagt es, wie viele Dateien und wie viel Platz: „Kopie erstellt: … Dateien, …“.",
            ),
            tr(
              "Per riportarla, in questo kit o in uno nuovo su un altro PC: «Importa da uno zip…» (o «Importa da una cartella…»), poi «Chiudi Atollo per importarla» e riapri Atollo. All'avvio la copia prende il posto di progetti e dati; quelli di prima restano in .atollo-before-import-<data-ora>, nella cartella del kit, finché non li cancelli tu. «Annulla l'importazione» ci ripensa.",
              "To bring it back, into this kit or a new one on another PC: “Import from a zip…” (or “Import from a folder…”), then “Close Atollo to import it” and open Atollo again. At the start the copy takes the place of projects and data; the previous ones stay in .atollo-before-import-<date-time>, in the kit's folder, until you delete them. “Cancel the import” changes your mind.",
              "Pour la remettre, dans ce kit ou dans un nouveau sur un autre PC : « Importer depuis un zip… » (ou « Importer depuis un dossier… »), puis « Fermer Atollo pour l'importer » et rouvrez Atollo. Au démarrage, la copie remplace projets et données ; les précédents restent dans .atollo-before-import-<date-heure>, dans le dossier du kit, jusqu'à ce que vous les supprimiez. « Annuler l'importation » revient en arrière.",
              "Um sie zurückzuholen, in dieses Kit oder ein neues auf einem anderen PC: „Aus einem Zip importieren…“ (oder „Aus einem Ordner importieren…“), dann „Atollo schließen, um sie zu importieren“ und Atollo wieder öffnen. Beim Start ersetzt die Kopie Projekte und Daten; die bisherigen bleiben in .atollo-before-import-<Datum-Uhrzeit>, im Ordner des Kits, bis du sie löschst. „Import abbrechen“ macht es rückgängig.",
            ),
          ]}
        />
        <P>
          {tr(
            "Su un altro PC o in un'altra cartella il kit si sistema come quando lo sposti: i programmi già scaricati restano, quelli che mancano si riscaricano da soli. L'accesso di Antigravity va rifatto su un altro PC o account, e i dati dei database vogliono le stesse versioni dei server, che si riscaricano per versione.",
            "On another PC or in another folder the kit adjusts as when you move it: the programs already downloaded stay, the missing ones are downloaded again on their own. Antigravity's sign-in has to be done again on another PC or account, and the database data needs the same server versions, which are downloaded again by version.",
            "Sur un autre PC ou dans un autre dossier, le kit s'adapte comme quand vous le déplacez : les programmes déjà téléchargés restent, ceux qui manquent se retéléchargent tout seuls. La connexion d'Antigravity est à refaire sur un autre PC ou compte, et les données des bases de données demandent les mêmes versions des serveurs, qui se retéléchargent par version.",
            "Auf einem anderen PC oder in einem anderen Ordner passt sich das Kit an wie beim Verschieben: Die bereits heruntergeladenen Programme bleiben, die fehlenden werden von selbst erneut heruntergeladen. Die Anmeldung bei Antigravity musst du auf einem anderen PC oder Konto wiederholen, und die Daten der Datenbanken brauchen dieselben Serverversionen, die nach Version erneut heruntergeladen werden.",
          )}
        </P>
        <P>
          {tr(
            "A mano, con Atollo chiuso, la copia è questa:",
            "By hand, with Atollo closed, the copy is this:",
            "À la main, avec Atollo fermé, la copie est celle-ci :",
            "Von Hand, bei geschlossenem Atollo, ist die Kopie diese:",
          )}
        </P>
        <Table
          columns={[tr("Da copiare", "To copy", "À copier", "Zu kopieren"), tr("Perché", "Why", "Pourquoi", "Warum")]}
          rows={[
            [
              <Code key="p">projects\</Code>,
              tr("I tuoi progetti.", "Your projects.", "Vos projets.", "Deine Projekte."),
            ],
            [
              <Code key="d">data\</Code>,
              tr(
                "Impostazioni, accessi, istruzioni degli agenti, dati dei database, copie dei database, dati dei progetti.",
                "Settings, sign-ins, agent instructions, database data, database backups, project data.",
                "Paramètres, connexions, instructions des agents, données des bases de données, copies des bases de données, données des projets.",
                "Einstellungen, Anmeldungen, Anweisungen der Agenten, Daten der Datenbanken, Sicherungen der Datenbanken, Daten der Projekte.",
              ),
            ],
          ]}
        />
        <P>
          {tr(
            "Dentro data puoi lasciare fuori data\\apps (i programmi) e data\\cache (file scaricati che si possono riscaricare): sono la parte più grande e il kit li ricostruisce. Anche data\\models (i modelli locali, molti GB) si può riscaricare dalla pagina AI, riquadro Modelli locali, tranne i file .gguf messi a mano. Per copiare i dati dei database in modo coerente, spegni prima i server (o usa «Rimuovi l'unità»).",
            "Inside data you can leave out data\\apps (the programs) and data\\cache (downloaded files that can be downloaded again): they are the biggest part and the kit rebuilds them. data\\models (the local models, many GB) can also be downloaded again from the AI page, Local models pane, except the .gguf files put there by hand. To copy the database data consistently, shut down the servers first (or use “Remove drive”).",
            "Dans data, vous pouvez laisser de côté data\\apps (les programmes) et data\\cache (fichiers téléchargés qui peuvent se retélécharger) : c'est la plus grosse partie et le kit les reconstruit. data\\models (les modèles locaux, de nombreux Go) peut aussi se retélécharger depuis la page IA, volet Modèles locaux, sauf les fichiers .gguf placés à la main. Pour copier les données des bases de données de façon cohérente, arrêtez d'abord les serveurs (ou utilisez « Retirer le lecteur »).",
            "Innerhalb von data kannst du data\\apps (die Programme) und data\\cache (heruntergeladene Dateien, die sich erneut herunterladen lassen) weglassen: Sie sind der größte Teil, und das Kit baut sie neu auf. Auch data\\models (die lokalen Modelle, viele GB) lässt sich auf der Seite KI im Bereich Lokale Modelle erneut herunterladen, außer den von Hand abgelegten .gguf-Dateien. Um die Daten der Datenbanken konsistent zu kopieren, fahr zuerst die Server herunter (oder verwende „Laufwerk entfernen“).",
          )}
        </P>
        <Callout tone="warn">
          {tr(
            "La cartella data contiene accessi e password. Tieni le copie in un posto sicuro, e cifra la chiavetta se la porti in giro (per esempio con BitLocker To Go su Windows).",
            "The data folder contains sign-ins and passwords. Keep the backups in a safe place, and encrypt the USB stick if you carry it around (for example with BitLocker To Go on Windows).",
            "Le dossier data contient des connexions et des mots de passe. Gardez les copies en lieu sûr, et chiffrez la clé USB si vous la transportez (par exemple avec BitLocker To Go sur Windows).",
            "Der Ordner data enthält Anmeldungen und Passwörter. Bewahre die Sicherungen an einem sicheren Ort auf, und verschlüssele den USB-Stick, wenn du ihn mitnimmst (zum Beispiel mit BitLocker To Go unter Windows).",
          )}
        </Callout>
        <P>
          {rich(
            tr(
              "Per un singolo database c'è la copia dalla pagina del progetto: {0}.",
              "For a single database there is the backup from the project page: {0}.",
              "Pour une seule base de données, il y a la copie depuis la page du projet : {0}.",
              "Für eine einzelne Datenbank gibt es die Sicherung auf der Seite des Projekts: {0}.",
            ),
            <See key="d" id="project-db" />,
          )}
        </P>
      </Section>

      <Under>
        <Bullets
          items={[
            tr(
              "Le nuove versioni arrivano dalle release app-v… del repository GitHub nabre/atollo; l'impronta SHA-256 viene da GitHub o dal file .sha256 accanto al file.",
              "New versions come from the app-v… releases of the GitHub repository nabre/atollo; the SHA-256 hash comes from GitHub or from the .sha256 file next to the file.",
              "Les nouvelles versions viennent des releases app-v… du dépôt GitHub nabre/atollo ; l'empreinte SHA-256 vient de GitHub ou du fichier .sha256 à côté du fichier.",
              "Neue Versionen kommen aus den Releases app-v… des GitHub-Repositorys nabre/atollo; der SHA-256-Hash kommt von GitHub oder aus der .sha256-Datei neben der Datei.",
            ),
            tr(
              "Su Windows la nuova versione attende in data\\app\\update; la copia sostituita resta come Atollo.old.exe fino al prossimo avvio.",
              "On Windows the new version waits in data\\app\\update; the replaced copy stays as Atollo.old.exe until the next start.",
              "Sur Windows, la nouvelle version attend dans data\\app\\update ; la copie remplacée reste sous le nom Atollo.old.exe jusqu'au prochain démarrage.",
              "Unter Windows wartet die neue Version in data\\app\\update; die ersetzte Kopie bleibt bis zum nächsten Start als Atollo.old.exe erhalten.",
            ),
            tr(
              "La variabile ATOLLO_NO_UPDATE, impostata nel PC, spegne il controllo automatico e anche la lettura giornaliera delle versioni offerte dalle fonti.",
              "The ATOLLO_NO_UPDATE variable, set on the PC, turns off the automatic check and also the daily reading of the versions the sources offer.",
              "La variable ATOLLO_NO_UPDATE, définie sur le PC, désactive la vérification automatique et aussi la lecture quotidienne des versions proposées par les sources.",
              "Die Variable ATOLLO_NO_UPDATE, auf dem PC gesetzt, schaltet die automatische Prüfung aus und auch das tägliche Lesen der Versionen, die die Quellen anbieten.",
            ),
          ]}
        />
      </Under>

      <Related ids={["tools", "eject", "disk-layout"]} />
    </>
  ),
};
