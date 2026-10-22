// Chapter 4 of the Guide, «Agenti IA»: the four agents, what they share,
// their limits and usage, and the local models Pi runs on.
import { rich } from "../../i18n";
import { tr } from "../model";
import {
  Bullets,
  Callout,
  Code,
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

const MCP_EXAMPLE = `{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "."]
    },
    "docs": {
      "type": "http",
      "url": "https://example.com/mcp"
    }
  }
}`;

export const AGENTS: TopicTexts = {
  agents: () => (
    <>
      <InBrief
        points={[
          tr(
            "Un agente IA è un assistente che lavora nella cartella del progetto: legge il codice, lo modifica, esegue comandi.",
            "An AI agent is an assistant that works in the project folder: it reads the code, changes it, runs commands.",
            "Un agent IA est un assistant qui travaille dans le dossier du projet : il lit le code, le modifie, exécute des commandes.",
            "Ein KI-Agent ist ein Assistent, der im Projektordner arbeitet: Er liest den Code, ändert ihn und führt Befehle aus.",
          ),
          tr(
            "Il kit ne offre quattro: Claude Code (Anthropic), Codex (OpenAI), Antigravity (Google) e Pi (Earendil). Si installano da Strumenti.",
            "The kit offers four: Claude Code (Anthropic), Codex (OpenAI), Antigravity (Google) and Pi (Earendil). You install them from Tools.",
            "Le kit en propose quatre : Claude Code (Anthropic), Codex (OpenAI), Antigravity (Google) et Pi (Earendil). Ils s'installent depuis Outils.",
            "Das Kit bietet vier: Claude Code (Anthropic), Codex (OpenAI), Antigravity (Google) und Pi (Earendil). Sie werden über Werkzeuge installiert.",
          ),
          tr(
            "Pi funziona anche senza account, sui modelli locali del kit, che girano sul tuo PC.",
            "Pi also works without an account, on the kit's local models, which run on your PC.",
            "Pi fonctionne aussi sans compte, sur les modèles locaux du kit, qui tournent sur votre PC.",
            "Pi funktioniert auch ohne Konto, mit den lokalen Modellen des Kits, die auf deinem PC laufen.",
          ),
          tr(
            "Si aprono sempre dentro un progetto, dalla pagina Progetti, con il pulsante che porta il logo dell'agente.",
            "They always open inside a project, from the Projects page, with the button that shows the agent's logo.",
            "Ils s'ouvrent toujours dans un projet, depuis la page Projets, avec le bouton qui porte le logo de l'agent.",
            "Sie öffnen sich immer in einem Projekt, auf der Seite Projekte, mit der Schaltfläche, die das Logo des Agenten trägt.",
          ),
          tr(
            "L'accesso al tuo account si fa al primo avvio dell'agente e resta nel kit, anche su un altro PC.",
            "You sign in to your account the first time the agent starts, and the sign-in stays in the kit, even on another PC.",
            "La connexion à votre compte se fait au premier lancement de l'agent et reste dans le kit, même sur un autre PC.",
            "Die Anmeldung bei deinem Konto erfolgt beim ersten Start des Agenten und bleibt im Kit, auch auf einem anderen PC.",
          ),
        ]}
      />

      <GuideShot
        src="agents.webp"
        title={tr("La pagina Agenti", "The Agents page", "La page Agents", "Die Seite Agenten")}
        caption={tr(
          "Gli agenti installati con accesso e limite più alto; a destra limiti e consumi, accesso e cartella condivisa.",
          "The installed agents with sign-in and highest limit; on the right limits and usage, sign-in and shared folder.",
          "Les agents installés avec connexion et limite la plus haute ; à droite limites et consommation, connexion et dossier partagé.",
          "Die installierten Agenten mit Anmeldung und höchstem Limit; rechts Limits und Verbrauch, Anmeldung und gemeinsamer Ordner.",
        )}
      />

      <Section
        title={tr(
          "Che cosa sono e che cosa serve",
          "What they are and what you need",
          "Ce qu'ils sont et ce qu'il faut",
          "Was sie sind und was du brauchst",
        )}
      >
        <P>
          {tr(
            "Un agente è un programma a riga di comando che dialoga con un modello di intelligenza artificiale del fornitore. Gli chiedi in italiano che cosa fare («aggiungi una pagina di contatto», «perché questo test fallisce?») e lui legge i file, propone modifiche, esegue comandi e ti mostra i risultati. Per usarlo serve un account del fornitore con un piano che lo comprenda, oppure una chiave API; Pi può usare invece un modello locale, senza account.",
            "An agent is a command-line program that talks with an artificial intelligence model from its provider. You tell it in plain English what to do (“add a contact page”, “why does this test fail?”) and it reads the files, proposes changes, runs commands and shows you the results. To use it you need an account with the provider and a plan that includes it, or an API key; Pi can use a local model instead, with no account.",
            "Un agent est un programme en ligne de commande qui dialogue avec un modèle d'intelligence artificielle du fournisseur. Vous lui demandez en français ce qu'il doit faire (« ajoute une page de contact », « pourquoi ce test échoue-t-il ? ») et il lit les fichiers, propose des modifications, exécute des commandes et vous montre les résultats. Pour l'utiliser, il faut un compte chez le fournisseur avec un abonnement qui l'inclut, ou une clé API ; Pi peut utiliser à la place un modèle local, sans compte.",
            "Ein Agent ist ein Kommandozeilenprogramm, das mit einem KI-Modell des Anbieters kommuniziert. Du sagst ihm auf Deutsch, was es tun soll („füge eine Kontaktseite hinzu“, „warum schlägt dieser Test fehl?“), und es liest die Dateien, schlägt Änderungen vor, führt Befehle aus und zeigt dir die Ergebnisse. Dafür brauchst du ein Konto beim Anbieter mit einem Tarif, der ihn enthält, oder einen API-Schlüssel; Pi kann stattdessen ein lokales Modell verwenden, ohne Konto.",
          )}
        </P>
        <Table
          columns={[
            tr("Agente", "Agent", "Agent", "Agent"),
            tr("Fornitore", "Provider", "Fournisseur", "Anbieter"),
            tr("Accesso", "Sign-in", "Connexion", "Anmeldung"),
          ]}
          rows={[
            [
              tr("Claude Code", "Claude Code", "Claude Code", "Claude Code"),
              tr("Anthropic", "Anthropic", "Anthropic", "Anthropic"),
              tr(
                "Account Claude (abbonamento); su macOS anche un token di claude setup-token.",
                "Claude account (subscription); on macOS also a token from claude setup-token.",
                "Compte Claude (abonnement) ; sur macOS aussi un jeton de claude setup-token.",
                "Claude-Konto (Abonnement); unter macOS auch ein Token von claude setup-token.",
              ),
            ],
            [
              tr("Codex", "Codex", "Codex", "Codex"),
              tr("OpenAI", "OpenAI", "OpenAI", "OpenAI"),
              tr(
                "Account ChatGPT oppure chiave API salvata da Codex con codex login --with-api-key.",
                "ChatGPT account or an API key saved by Codex with codex login --with-api-key.",
                "Compte ChatGPT ou clé API enregistrée par Codex avec codex login --with-api-key.",
                "ChatGPT-Konto oder ein API-Schlüssel, den Codex mit codex login --with-api-key speichert.",
              ),
            ],
            [
              tr("Antigravity", "Antigravity", "Antigravity", "Antigravity"),
              tr("Google", "Google", "Google", "Google"),
              tr(
                "Account Google oppure chiave API di Gemini, da scrivere in Impostazioni › Agenti.",
                "Google account or a Gemini API key, to enter in Settings › Agents.",
                "Compte Google ou clé API de Gemini, à saisir dans Paramètres › Agents.",
                "Google-Konto oder API-Schlüssel von Gemini, einzutragen unter Einstellungen › Agenten.",
              ),
            ],
            [
              tr("Pi", "Pi", "Pi", "Pi"),
              tr(
                "Earendil (Mario Zechner)",
                "Earendil (Mario Zechner)",
                "Earendil (Mario Zechner)",
                "Earendil (Mario Zechner)",
              ),
              tr(
                "Nessuno con i modelli locali del kit; per un provider in cloud, /login dentro Pi.",
                "None with the kit's local models; for a cloud provider, /login inside Pi.",
                "Aucune avec les modèles locaux du kit ; pour un fournisseur dans le cloud, /login dans Pi.",
                "Keine mit den lokalen Modellen des Kits; für einen Cloud-Anbieter /login in Pi.",
              ),
            ],
          ]}
        />
        <P>
          {tr(
            "Gli agenti si installano dalla pagina Strumenti, tipo «Agenti AI» (il primo avvio propone già Claude Code, Codex e Antigravity; Pi non è tra quelli proposti, si spunta a mano). Seguono sempre l'ultima versione e portano con sé ciò che serve; Git, che usano tutti, è già tra i componenti sempre installati.",
            "Agents are installed from the Tools page, “AI agents” kind (the first start already suggests Claude Code, Codex and Antigravity; Pi is not among the suggested ones, you tick it by hand). They always follow the latest version and bring along what they need; Git, which they all use, is already among the components that are always installed.",
            "Les agents s'installent depuis la page Outils, type « Agents IA » (le premier démarrage propose déjà Claude Code, Codex et Antigravity ; Pi ne fait pas partie des agents proposés, il se coche à la main). Ils suivent toujours la dernière version et apportent ce dont ils ont besoin ; Git, qu'ils utilisent tous, fait déjà partie des composants toujours installés.",
            "Die Agenten werden auf der Seite Werkzeuge in der Art „KI-Agenten“ installiert (der erste Start schlägt Claude Code, Codex und Antigravity bereits vor; Pi gehört nicht dazu, du kreuzt ihn von Hand an). Sie folgen immer der neuesten Version und bringen mit, was sie brauchen; Git, das sie alle verwenden, gehört schon zu den immer installierten Komponenten.",
          )}
        </P>
        <Callout tone="info" title={tr("Pi", "Pi", "Pi", "Pi")}>
          {rich(
            tr(
              "Pi è un agente da terminale che lavora con molti fornitori. Nel kit parte da solo su un modello locale: quello scelto nella pagina «Modelli locali», altrimenti il primo del kit che sa usare gli strumenti, e prima di aprirlo il kit avvia il server dei modelli. Se in Pi scegli un provider in cloud con /login, resta su quello; i modelli locali restano nel suo elenco (/model). Su Windows Pi usa il Git Bash del kit come shell. Vedi {0}.",
              "Pi is a terminal agent that works with many providers. In the kit it starts by itself on a local model: the one chosen on the “Local models” page, otherwise the kit's first one that can use tools, and before opening it the kit starts the models server. If you choose a cloud provider in Pi with /login, it stays on that one; the local models remain in its list (/model). On Windows Pi uses the kit's Git Bash as its shell. See {0}.",
              "Pi est un agent de terminal qui fonctionne avec de nombreux fournisseurs. Dans le kit, il démarre tout seul sur un modèle local : celui choisi dans la page « Modèles locaux », sinon le premier du kit qui sait utiliser les outils, et avant de l'ouvrir le kit démarre le serveur des modèles. Si vous choisissez dans Pi un fournisseur dans le cloud avec /login, il reste sur celui-ci ; les modèles locaux restent dans sa liste (/model). Sur Windows, Pi utilise le Git Bash du kit comme shell. Voir {0}.",
              "Pi ist ein Terminal-Agent, der mit vielen Anbietern arbeitet. Im Kit startet er von selbst mit einem lokalen Modell: dem auf der Seite „Lokale Modelle“ gewählten, sonst dem ersten des Kits, das Werkzeuge nutzen kann, und bevor er geöffnet wird, startet das Kit den Server der Modelle. Wählst du in Pi mit /login einen Cloud-Anbieter, bleibt er bei diesem; die lokalen Modelle bleiben in seiner Liste (/model). Unter Windows verwendet Pi die Git Bash des Kits als Shell. Siehe {0}.",
            ),
            <See key="m" id="models" />,
          )}
        </Callout>
      </Section>

      <Section
        title={tr(
          "Aprire un agente in un progetto",
          "Opening an agent in a project",
          "Ouvrir un agent dans un projet",
          "Einen Agenten in einem Projekt öffnen",
        )}
      >
        <P>
          {tr(
            "La pagina Agenti non avvia gli agenti: un agente lavora sempre nella cartella di un progetto, quindi si apre dalla pagina Progetti.",
            "The Agents page does not start agents: an agent always works in a project's folder, so you open it from the Projects page.",
            "La page Agents ne lance pas les agents : un agent travaille toujours dans le dossier d'un projet, il s'ouvre donc depuis la page Projets.",
            "Die Seite Agenten startet keine Agenten: Ein Agent arbeitet immer im Ordner eines Projekts, daher wird er auf der Seite Projekte geöffnet.",
          )}
        </P>
        <Steps
          steps={[
            tr("Apri il progetto.", "Open the project.", "Ouvrez le projet.", "Öffne das Projekt."),
            tr(
              "In alto, accanto a «Editor» e «Terminale», c'è un pulsante per ogni agente installato, con il suo logo («Apri Claude Code», «Apri Codex», «Apri Antigravity», «Apri Pi»). Fai clic su quello dell'agente che vuoi usare.",
              "At the top, next to “Editor” and “Terminal”, there is a button for each installed agent, with its logo (“Open Claude Code”, “Open Codex”, “Open Antigravity”, “Open Pi”). Click the one of the agent you want to use.",
              "En haut, à côté de « Éditeur » et « Terminal », il y a un bouton pour chaque agent installé, avec son logo (« Ouvrir Claude Code », « Ouvrir Codex », « Ouvrir Antigravity », « Ouvrir Pi »). Cliquez sur celui de l'agent que vous voulez utiliser.",
              "Oben neben „Editor“ und „Terminal“ gibt es für jeden installierten Agenten eine Schaltfläche mit seinem Logo („Claude Code öffnen“, „Codex öffnen“, „Antigravity öffnen“, „Pi öffnen“). Klicke auf die des Agenten, den du verwenden möchtest.",
            ),
            tr(
              "L'agente parte subito, nella cartella del progetto, in una scheda del pannello dei terminali dentro Atollo (su ogni sistema).",
              "The agent starts right away, in the project folder, in a tab of the terminal panel inside Atollo (on every system).",
              "L'agent démarre aussitôt, dans le dossier du projet, dans un onglet du panneau des terminaux dans Atollo (sur tous les systèmes).",
              "Der Agent startet sofort, im Projektordner, in einer Registerkarte des Terminalbereichs in Atollo (auf jedem System).",
            ),
          ]}
        />
        <P>
          {tr(
            "Gli stessi pulsanti sono nella riga del progetto (barra laterale o «Pannello rapido»): un clic sul logo apre quell'agente nel progetto. Un agente non installato non ha il suo pulsante; se non ne è installato nessuno, non ci sono pulsanti degli agenti.",
            "The same buttons are on the project's row (sidebar or “Quick panel”): a click on a logo opens that agent in the project. An agent that is not installed has no button; if none is installed, there are no agent buttons.",
            "Les mêmes boutons se trouvent sur la ligne du projet (barre latérale ou « Panneau rapide ») : un clic sur un logo ouvre cet agent dans le projet. Un agent non installé n'a pas de bouton ; si aucun n'est installé, il n'y a pas de boutons d'agents.",
            "Dieselben Schaltflächen gibt es in der Zeile des Projekts (Seitenleiste oder „Schnellübersicht“): Ein Klick auf ein Logo öffnet diesen Agenten im Projekt. Ein nicht installierter Agent hat keine Schaltfläche; ist keiner installiert, gibt es keine Agenten-Schaltflächen.",
          )}
        </P>
        <Under>
          <Bullets
            items={[
              tr(
                "L'ultimo agente aperto viene annotato nel file dev-settings.json del progetto.",
                "The last agent opened is noted in the project's dev-settings.json file.",
                "Le dernier agent ouvert est noté dans le fichier dev-settings.json du projet.",
                "Der zuletzt geöffnete Agent wird in der Datei dev-settings.json des Projekts vermerkt.",
              ),
              tr(
                "Prima dell'avvio il kit installa ciò che manca (l'agente, le sue dipendenze, i programmi dei server MCP come Node.js o uv), applica le versioni del progetto e allinea istruzioni, skill e server MCP. Quando l'agente termina, li allinea di nuovo.",
                "Before the start, the kit installs what is missing (the agent, its dependencies, the programs of the MCP servers such as Node.js or uv), applies the project's versions and syncs instructions, skills and MCP servers. When the agent ends, it syncs them again.",
                "Avant le lancement, le kit installe ce qui manque (l'agent, ses dépendances, les programmes des serveurs MCP comme Node.js ou uv), applique les versions du projet et synchronise instructions, skills et serveurs MCP. Quand l'agent se termine, il les synchronise à nouveau.",
                "Vor dem Start installiert das Kit, was fehlt (den Agenten, seine Abhängigkeiten, die Programme der MCP-Server wie Node.js oder uv), wendet die Versionen des Projekts an und gleicht Anweisungen, Skills und MCP-Server ab. Wenn der Agent endet, gleicht es sie erneut ab.",
              ),
              tr(
                "Antigravity parte attraverso un piccolo programma del kit (agy) che rimette il tuo accesso nel sistema prima dell'avvio e lo riporta nel kit alla fine.",
                "Antigravity starts through a small kit program (agy) that puts your sign-in back into the system before the start and brings it back into the kit at the end.",
                "Antigravity démarre via un petit programme du kit (agy) qui remet votre connexion dans le système avant le lancement et la rapporte dans le kit à la fin.",
                "Antigravity startet über ein kleines Programm des Kits (agy), das deine Anmeldung vor dem Start ins System zurücklegt und sie am Ende wieder ins Kit holt.",
              ),
            ]}
          />
        </Under>
      </Section>

      <Section title={tr("La pagina Agenti", "The Agents page", "La page Agents", "Die Seite Agenten")}>
        <P>
          {tr(
            "A sinistra il riquadro «Installati» elenca gli agenti con la versione, lo stato dell'accesso, «in esecuzione» quando sono aperti e il limite più alto raggiunto (per esempio «5 ore 42%»). A destra, per l'agente scelto, tre parti:",
            "On the left, the “Installed” box lists the agents with their version, sign-in status, “running” when they are open and the highest limit reached (for example “5 hours 42%”). On the right, for the chosen agent, three parts:",
            "À gauche, le panneau « Installés » liste les agents avec la version, l'état de la connexion, « en cours d'exécution » quand ils sont ouverts et la limite la plus haute atteinte (par exemple « 5 heures 42% »). À droite, pour l'agent choisi, trois parties :",
            "Links listet der Bereich „Installiert“ die Agenten mit Version, Anmeldestatus, „läuft“, wenn sie geöffnet sind, und dem höchsten erreichten Limit auf (zum Beispiel „5 Stunden 42%“). Rechts, für den gewählten Agenten, drei Teile:",
          )}
        </P>
        <Table
          columns={[
            tr("Parte", "Part", "Partie", "Teil"),
            tr("Che cosa contiene", "What it contains", "Ce qu'elle contient", "Was er enthält"),
          ]}
          rows={[
            [
              tr("Limiti e consumi", "Limits and usage", "Limites et consommation", "Limits und Verbrauch"),
              <See key="u" id="agents-usage" />,
            ],
            [
              tr("Accesso", "Sign-in", "Connexion", "Anmeldung"),
              tr(
                "Lo stato dell'accesso, «Esci» e, su macOS per Claude Code, «Collega con un token».",
                "The sign-in status, “Sign out” and, on macOS for Claude Code, “Connect with a token”.",
                "L'état de la connexion, « Se déconnecter » et, sur macOS pour Claude Code, « Connecter avec un jeton ».",
                "Der Anmeldestatus, „Abmelden“ und unter macOS für Claude Code „Mit einem Token verbinden“.",
              ),
            ],
            [
              tr(
                "Condiviso fra gli agenti",
                "Shared by the agents",
                "Partagé entre les agents",
                "Von den Agenten geteilt",
              ),
              <See key="s" id="agents-shared" />,
            ],
          ]}
        />
        <P>
          {tr(
            "In cima alla barra laterale: «Aggiorna i consumi», «Apri la cartella condivisa» (data\\agents) e il libro della guida. Se non c'è nessun agente: «Nessun agente installato: aggiungili da Strumenti.»",
            "At the top of the sidebar: “Refresh usage”, “Open shared folder” (data\\agents) and the guide's book. If there is no agent: “No agent installed: add them from Tools.”",
            "En haut de la barre latérale : « Actualiser la consommation », « Ouvrir le dossier partagé » (data\\agents) et le livre du guide. S'il n'y a aucun agent : « Aucun agent installé : ajoutez-les depuis Outils. »",
            "Oben in der Seitenleiste: „Verbrauch aktualisieren“, „Gemeinsamen Ordner öffnen“ (data\\agents) und das Buch der Anleitung. Wenn kein Agent vorhanden ist: „Kein Agent installiert: über Werkzeuge hinzufügen.“",
          )}
        </P>
        <Go page="agents">
          {tr("Apri la pagina Agenti", "Open the Agents page", "Ouvrir la page Agents", "Seite Agenten öffnen")}
        </Go>
      </Section>

      <Section
        title={tr(
          "L'accesso al tuo account",
          "Signing in to your account",
          "La connexion à votre compte",
          "Die Anmeldung bei deinem Konto",
        )}
      >
        <Table
          columns={[tr("Testo", "Text", "Texte", "Text"), tr("Significato", "Meaning", "Signification", "Bedeutung")]}
          rows={[
            [
              tr(
                "accesso non fatto: si fa al primo avvio dell'agente",
                "not signed in: you sign in the first time the agent starts",
                "non connecté : la connexion se fait au premier lancement de l'agent",
                "nicht angemeldet: Die Anmeldung erfolgt beim ersten Start des Agenten",
              ),
              tr(
                "Apri l'agente in un progetto e segui le sue istruzioni (di solito una pagina nel browser).",
                "Open the agent in a project and follow its instructions (usually a page in the browser).",
                "Ouvrez l'agent dans un projet et suivez ses instructions (en général une page dans le navigateur).",
                "Öffne den Agenten in einem Projekt und folge seinen Anweisungen (meist eine Seite im Browser).",
              ),
            ],
            [
              tr("accesso fatto", "signed in", "connecté", "angemeldet"),
              tr(
                "L'agente ha un accesso con il tuo account, salvato nel kit.",
                "The agent is signed in with your account, saved in the kit.",
                "L'agent est connecté avec votre compte, enregistré dans le kit.",
                "Der Agent ist mit deinem Konto angemeldet, gespeichert im Kit.",
              ),
            ],
            [
              tr(
                "accesso con chiave API",
                "signed in with an API key",
                "connecté avec une clé API",
                "mit API-Schlüssel angemeldet",
              ),
              tr(
                "Codex con la chiave salvata da codex login --with-api-key, Antigravity con la chiave di Gemini delle Impostazioni, o Pi con una chiave API salvata nel suo auth.json: niente limiti di piano, paghi a consumo.",
                "Codex with the key saved by codex login --with-api-key, Antigravity with the Gemini key from Settings, or Pi with an API key saved in its auth.json: no plan limits, you pay for what you use.",
                "Codex avec la clé enregistrée par codex login --with-api-key, Antigravity avec la clé Gemini des Paramètres, ou Pi avec une clé API enregistrée dans son auth.json : pas de limites d'abonnement, vous payez à la consommation.",
                "Codex mit dem von codex login --with-api-key gespeicherten Schlüssel, Antigravity mit dem Gemini-Schlüssel aus den Einstellungen oder Pi mit einem in seiner auth.json gespeicherten API-Schlüssel: keine Tariflimits, du zahlst nach Verbrauch.",
              ),
            ],
            [
              tr(
                "accesso con un token",
                "signed in with a token",
                "connecté avec un jeton",
                "mit einem Token angemeldet",
              ),
              tr(
                "Solo Claude Code su macOS, collegato con un token (vedi sotto).",
                "Only Claude Code on macOS, connected with a token (see below).",
                "Uniquement Claude Code sur macOS, connecté avec un jeton (voir plus bas).",
                "Nur Claude Code unter macOS, mit einem Token verbunden (siehe unten).",
              ),
            ],
            [
              tr(
                "modelli locali del kit, senza accesso",
                "the kit's local models, no sign-in",
                "modèles locaux du kit, sans connexion",
                "lokale Modelle des Kits, ohne Anmeldung",
              ),
              tr(
                "Solo Pi: nessun provider in cloud, usa un modello scaricato nella pagina «Modelli locali». Non c'è niente da cui uscire.",
                "Only Pi: no cloud provider, it uses a model downloaded on the “Local models” page. There is nothing to sign out of.",
                "Uniquement Pi : aucun fournisseur dans le cloud, il utilise un modèle téléchargé dans la page « Modèles locaux ». Il n'y a rien dont se déconnecter.",
                "Nur Pi: kein Cloud-Anbieter, er verwendet ein auf der Seite „Lokale Modelle“ heruntergeladenes Modell. Es gibt nichts, wovon man sich abmelden müsste.",
              ),
            ],
          ]}
        />
        <P>
          {tr(
            "L'accesso resta nella cartella del kit: portando il kit su un altro PC non devi rifarlo. «Esci» toglie l'accesso salvato nel kit dopo una domanda («Togliere l'accesso salvato nel kit?», Esc annulla), con l'agente chiuso: mentre è aperto il pulsante dice «Chiudi prima l'agente». Con una chiave API «Esci» non c'è.",
            "The sign-in stays in the kit folder: when you take the kit to another PC you do not have to sign in again. “Sign out” removes the sign-in saved in the kit after a question (“Remove the sign-in saved in the kit?”, Esc cancels), with the agent closed: while it is open the button says “Close the agent first”. With an API key there is no “Sign out”.",
            "La connexion reste dans le dossier du kit : en emportant le kit sur un autre PC, vous n'avez pas à la refaire. « Se déconnecter » retire la connexion enregistrée dans le kit après une question (« Supprimer la connexion enregistrée dans le kit ? », Échap annule), avec l'agent fermé : tant qu'il est ouvert, le bouton indique « Fermez d'abord l'agent ». Avec une clé API, il n'y a pas de « Se déconnecter ».",
            "Die Anmeldung bleibt im Ordner des Kits: Wenn du das Kit auf einen anderen PC mitnimmst, musst du dich nicht erneut anmelden. „Abmelden“ entfernt die im Kit gespeicherte Anmeldung nach einer Frage („Die im Kit gespeicherte Anmeldung entfernen?“, Esc bricht ab), bei geschlossenem Agenten: Solange er geöffnet ist, zeigt die Schaltfläche „Schließe zuerst den Agenten“. Mit einem API-Schlüssel gibt es kein „Abmelden“.",
          )}
        </P>
        <Callout
          tone="warn"
          title={tr(
            "Se perdi la chiavetta",
            "If you lose the USB stick",
            "Si vous perdez la clé USB",
            "Wenn du den USB-Stick verlierst",
          )}
        >
          {tr(
            "Gli accessi degli agenti e di GitHub sono salvati nel kit, in chiaro. «Esci» non revoca le sessioni sul sito del fornitore: se perdi la chiavetta, revocale dalle impostazioni del tuo account di Anthropic, OpenAI, Google e GitHub (e del provider che usi con Pi).",
            "The sign-ins of the agents and of GitHub are saved in the kit, unencrypted. “Sign out” does not revoke the sessions on the provider's site: if you lose the USB stick, revoke them from your account settings at Anthropic, OpenAI, Google and GitHub (and at the provider you use with Pi).",
            "Les connexions des agents et de GitHub sont enregistrées dans le kit, en clair. « Se déconnecter » ne révoque pas les sessions sur le site du fournisseur : si vous perdez la clé USB, révoquez-les depuis les paramètres de votre compte chez Anthropic, OpenAI, Google et GitHub (et chez le fournisseur que vous utilisez avec Pi).",
            "Die Anmeldungen der Agenten und von GitHub sind im Kit gespeichert, unverschlüsselt. „Abmelden“ widerruft die Sitzungen auf der Website des Anbieters nicht: Wenn du den USB-Stick verlierst, widerrufe sie in den Einstellungen deines Kontos bei Anthropic, OpenAI, Google und GitHub (und bei dem Anbieter, den du mit Pi verwendest).",
          )}
        </Callout>
        <Callout
          tone="info"
          title={tr("Claude Code su macOS", "Claude Code on macOS", "Claude Code sur macOS", "Claude Code unter macOS")}
        >
          {tr(
            "Su macOS Claude Code salverebbe l'accesso nel Portachiavi, che resta su quel Mac. «Collega con un token» apre claude setup-token nel terminale dell'app: completi l'accesso nel browser e il token (valido un anno) viene salvato nel kit, così funziona su ogni Mac. Se non viene riconosciuto, incollalo nel campo «Token di Claude Code» e premi «Salva il token». Il token arriva, come CLAUDE_CODE_OAUTH_TOKEN, solo a Claude Code avviato da Atollo: non alle azioni dei progetti, agli script, ai terminali né all'editor.",
            "On macOS Claude Code would save the sign-in in the Keychain, which stays on that Mac. “Connect with a token” opens claude setup-token in the app's terminal: you complete the sign-in in the browser and the token (valid for one year) is saved in the kit, so it works on every Mac. If it is not recognized, paste it into the “Claude Code token” field and press “Save the token”. The token reaches, as CLAUDE_CODE_OAUTH_TOKEN, only the Claude Code that Atollo launches: not project actions, scripts, terminals or the editor.",
            "Sur macOS, Claude Code enregistrerait la connexion dans le Trousseau, qui reste sur ce Mac. « Connecter avec un jeton » ouvre claude setup-token dans le terminal de l'application : vous terminez la connexion dans le navigateur et le jeton (valable un an) est enregistré dans le kit, il fonctionne ainsi sur chaque Mac. S'il n'est pas reconnu, collez-le dans le champ « Jeton de Claude Code » et appuyez sur « Enregistrer le jeton ». Le jeton n'arrive, comme CLAUDE_CODE_OAUTH_TOKEN, qu'au Claude Code lancé par Atollo : ni aux actions des projets, ni aux scripts, ni aux terminaux, ni à l'éditeur.",
            "Unter macOS würde Claude Code die Anmeldung im Schlüsselbund speichern, der auf diesem Mac bleibt. „Mit einem Token verbinden“ öffnet claude setup-token im Terminal der App: Du schließt die Anmeldung im Browser ab, und das Token (ein Jahr gültig) wird im Kit gespeichert, sodass es auf jedem Mac funktioniert. Wird es nicht erkannt, füge es in das Feld „Token von Claude Code“ ein und drück „Token speichern“. Das Token erhält, als CLAUDE_CODE_OAUTH_TOKEN, nur das von Atollo gestartete Claude Code: nicht die Aktionen der Projekte, Skripte, Terminals oder der Editor.",
          )}
        </Callout>
        <Under>
          <Table
            columns={[
              tr("Agente", "Agent", "Agent", "Agent"),
              tr(
                "Dove sta l'accesso",
                "Where the sign-in is stored",
                "Où se trouve la connexion",
                "Wo die Anmeldung liegt",
              ),
            ]}
            rows={[
              [
                tr("Claude Code", "Claude Code", "Claude Code", "Claude Code"),
                <Code key="c">data\home\.claude\.credentials.json</Code>,
              ],
              [tr("Codex", "Codex", "Codex", "Codex"), <Code key="x">data\home\.codex\auth.json</Code>],
              [
                tr("Antigravity", "Antigravity", "Antigravity", "Antigravity"),
                <Code key="a">data\antigravity\keyring.json</Code>,
              ],
              [
                tr(
                  "Pi (provider in cloud; «Esci» lo toglie)",
                  "Pi (cloud provider; “Sign out” removes it)",
                  "Pi (fournisseur dans le cloud ; « Se déconnecter » le retire)",
                  "Pi (Cloud-Anbieter; „Abmelden“ entfernt ihn)",
                ),
                <Code key="p">data\home\.pi\agent\auth.json</Code>,
              ],
              [
                tr(
                  "Claude Code, token su macOS",
                  "Claude Code, token on macOS",
                  "Claude Code, jeton sur macOS",
                  "Claude Code, Token unter macOS",
                ),
                <Code key="t">data\agents\claude\oauth-token</Code>,
              ],
            ]}
          />
          <P>
            {tr(
              "Antigravity salva il suo accesso solo nel portachiavi del sistema (Gestione credenziali su Windows, Portachiavi su macOS, Secret Service su Linux). Il kit ce lo mette prima dell'avvio e lo riporta in keyring.json alla fine, rimettendo a posto le eventuali credenziali di Antigravity del PC (conservate, cifrate, in keyring-before.json). Su Windows l'accesso nel sistema vive solo per la sessione di Windows.",
              "Antigravity saves its sign-in only in the system keychain (Credential Manager on Windows, Keychain on macOS, Secret Service on Linux). The kit puts it there before the start and brings it back into keyring.json at the end, restoring any Antigravity credentials of the PC (kept, encrypted, in keyring-before.json). On Windows the sign-in in the system lasts only for the Windows session.",
              "Antigravity enregistre sa connexion uniquement dans le trousseau du système (Gestionnaire d'identification sur Windows, Trousseau sur macOS, Secret Service sur Linux). Le kit l'y place avant le lancement et la rapporte dans keyring.json à la fin, en remettant en place les éventuels identifiants Antigravity du PC (conservés, chiffrés, dans keyring-before.json). Sur Windows, la connexion dans le système ne dure que le temps de la session Windows.",
              "Antigravity speichert seine Anmeldung nur im Schlüsselbund des Systems (Anmeldeinformationsverwaltung unter Windows, Schlüsselbund unter macOS, Secret Service unter Linux). Das Kit legt sie vor dem Start dort ab und holt sie am Ende zurück in keyring.json, wobei es eventuelle Antigravity-Anmeldedaten des PCs wiederherstellt (verschlüsselt aufbewahrt in keyring-before.json). Unter Windows gilt die Anmeldung im System nur für die Windows-Sitzung.",
            )}
          </P>
        </Under>
      </Section>

      <Related ids={["agents-shared", "agents-usage", "models", "settings"]} />
    </>
  ),

  "agents-shared": () => (
    <>
      <InBrief
        points={[
          tr(
            "Istruzioni, skill e server MCP si scrivono una volta sola, nella cartella data\\agents, e valgono per tutti gli agenti.",
            "Instructions, skills and MCP servers are written only once, in the data\\agents folder, and apply to all the agents.",
            "Instructions, skills et serveurs MCP s'écrivent une seule fois, dans le dossier data\\agents, et valent pour tous les agents.",
            "Anweisungen, Skills und MCP-Server werden nur einmal geschrieben, im Ordner data\\agents, und gelten für alle Agenten.",
          ),
          tr(
            "Il kit li allinea all'avvio dell'app, prima e dopo ogni agente e quando un agente termina.",
            "The kit syncs them when the app starts, before and after each agent and when an agent ends.",
            "Le kit les synchronise au démarrage de l'application, avant et après chaque agent et quand un agent se termine.",
            "Das Kit gleicht sie beim Start der App ab, vor und nach jedem Agenten und wenn ein Agent endet.",
          ),
          tr(
            "Puoi modificare anche i file di un agente: la modifica passa agli altri.",
            "You can also edit an agent's files: the change passes to the others.",
            "Vous pouvez aussi modifier les fichiers d'un agent : la modification passe aux autres.",
            "Du kannst auch die Dateien eines Agenten ändern: Die Änderung geht an die anderen über.",
          ),
          tr(
            "I server MCP del kit si aggiungono anche dalla pagina Agenti, riquadro «Server MCP del kit», con i server pronti GitHub, Playwright e Fetch a un clic.",
            "The kit's MCP servers can also be added from the Agents page, “The kit's MCP servers” box, with the ready servers GitHub, Playwright and Fetch one click away.",
            "Les serveurs MCP du kit s'ajoutent aussi depuis la page Agents, panneau « Serveurs MCP du kit », avec les serveurs prêts GitHub, Playwright et Fetch en un clic.",
            "Die MCP-Server des Kits lassen sich auch auf der Seite Agenten im Bereich „MCP-Server des Kits“ hinzufügen, mit den fertigen Servern GitHub, Playwright und Fetch per Klick.",
          ),
          tr(
            "I server MCP di un solo progetto si aggiungono nella sezione «Editor» del progetto.",
            "MCP servers for a single project are added in the project's “Editor” section.",
            "Les serveurs MCP d'un seul projet s'ajoutent dans la section « Éditeur » du projet.",
            "MCP-Server für ein einzelnes Projekt werden im Abschnitt „Editor“ des Projekts hinzugefügt.",
          ),
        ]}
      />

      <Section title={tr("Che cosa si condivide", "What is shared", "Ce qui est partagé", "Was geteilt wird")}>
        <Table
          columns={[
            tr("Che cosa", "What", "Quoi", "Was"),
            tr("A che cosa serve", "What it is for", "À quoi ça sert", "Wozu es dient"),
            tr("Il file da modificare", "The file to edit", "Le fichier à modifier", "Die zu bearbeitende Datei"),
          ]}
          rows={[
            [
              tr("Istruzioni globali", "Global instructions", "Instructions globales", "Globale Anweisungen"),
              tr(
                "Regole e preferenze che valgono in ogni progetto («rispondi in italiano», «usa pnpm», il tuo stile di codice).",
                "Rules and preferences that apply in every project (“reply in English”, “use pnpm”, your coding style).",
                "Règles et préférences valables dans chaque projet (« réponds en français », « utilise pnpm », votre style de code).",
                "Regeln und Vorlieben, die in jedem Projekt gelten („antworte auf Deutsch“, „verwende pnpm“, dein Codestil).",
              ),
              <Code key="i">data\agents\AGENTS.md</Code>,
            ],
            [
              tr("Skill", "Skills", "Skills", "Skills"),
              tr(
                "Procedure riutilizzabili che l'agente carica quando servono, una cartella per skill con un file SKILL.md.",
                "Reusable procedures that the agent loads when needed, one folder per skill with a SKILL.md file.",
                "Procédures réutilisables que l'agent charge quand il en a besoin, un dossier par skill avec un fichier SKILL.md.",
                "Wiederverwendbare Abläufe, die der Agent bei Bedarf lädt, ein Ordner pro Skill mit einer Datei SKILL.md.",
              ),
              <Code key="s">
                {"data\\agents\\skills\\" + tr("<nome>", "<name>", "<nom>", "<Name>") + "\\SKILL.md"}
              </Code>,
            ],
            [
              tr("Server MCP", "MCP servers", "Serveurs MCP", "MCP-Server"),
              tr(
                "Programmi o servizi che danno all'agente nuovi strumenti (leggere una documentazione, interrogare un servizio, usare il browser).",
                "Programs or services that give the agent new tools (reading documentation, querying a service, using the browser).",
                "Programmes ou services qui donnent à l'agent de nouveaux outils (lire une documentation, interroger un service, utiliser le navigateur).",
                "Programme oder Dienste, die dem Agenten neue Werkzeuge geben (eine Dokumentation lesen, einen Dienst abfragen, den Browser verwenden).",
              ),
              <Code key="m">data\agents\mcp.json</Code>,
            ],
          ]}
        />
        <Steps
          steps={[
            tr(
              "Nella pagina Agenti premi «Apri la cartella condivisa».",
              "On the Agents page, press “Open shared folder”.",
              "Dans la page Agents, appuyez sur « Ouvrir le dossier partagé ».",
              "Drück auf der Seite Agenten „Gemeinsamen Ordner öffnen“.",
            ),
            tr(
              "Modifica AGENTS.md con un editor di testo, aggiungi cartelle in skills o server in mcp.json.",
              "Edit AGENTS.md with a text editor, add folders in skills or servers in mcp.json.",
              "Modifiez AGENTS.md avec un éditeur de texte, ajoutez des dossiers dans skills ou des serveurs dans mcp.json.",
              "Bearbeite AGENTS.md mit einem Texteditor, füge Ordner in skills oder Server in mcp.json hinzu.",
            ),
            tr(
              "La modifica arriva agli agenti al loro prossimo avvio (o subito, se riavvii l'app).",
              "The change reaches the agents at their next start (or right away, if you restart the app).",
              "La modification arrive aux agents à leur prochain lancement (ou tout de suite, si vous redémarrez l'application).",
              "Die Änderung erreicht die Agenten bei ihrem nächsten Start (oder sofort, wenn du die App neu startest).",
            ),
          ]}
        />
        <GuideCode
          title={tr("Esempio di mcp.json", "Example mcp.json", "Exemple de mcp.json", "Beispiel für mcp.json")}
          path={"data\\agents\\mcp.json"}
          code={MCP_EXAMPLE}
        />
        <Callout tone="tip">
          {tr(
            "I programmi dei server MCP si installano da soli all'avvio di un agente: npx e node portano Node.js, uvx e uv portano uv, python porta Python.",
            "The MCP servers' programs install themselves when an agent starts: npx and node bring Node.js, uvx and uv bring uv, python brings Python.",
            "Les programmes des serveurs MCP s'installent tout seuls au lancement d'un agent : npx et node apportent Node.js, uvx et uv apportent uv, python apporte Python.",
            "Die Programme der MCP-Server installieren sich beim Start eines Agenten von selbst: npx und node bringen Node.js, uvx und uv bringen uv, python bringt Python.",
          )}
        </Callout>
      </Section>

      <Section
        title={tr(
          "Server MCP del kit e server pronti",
          "The kit's MCP servers and the ready servers",
          "Serveurs MCP du kit et serveurs prêts",
          "MCP-Server des Kits und fertige Server",
        )}
      >
        <Steps
          steps={[
            tr(
              "Nella pagina Agenti, sotto la cartella condivisa, c'è il riquadro «Server MCP del kit»: l'elenco di data\\agents\\mcp.json.",
              "On the Agents page, below the shared folder, there is the “The kit's MCP servers” box: the list of data\\agents\\mcp.json.",
              "Dans la page Agents, sous le dossier partagé, se trouve le panneau « Serveurs MCP du kit » : la liste de data\\agents\\mcp.json.",
              "Auf der Seite Agenten gibt es unter dem gemeinsamen Ordner den Bereich „MCP-Server des Kits“: die Liste von data\\agents\\mcp.json.",
            ),
            tr(
              "In «Server pronti» premi «Aggiungi» accanto al server che ti serve; per un altro server premi «Aggiungi un server» e compila nome, tipo e programma o indirizzo.",
              "Under “Ready servers” press “Add” beside the server you need; for another server press “Add a server” and fill in name, type and program or address.",
              "Dans « Serveurs prêts », appuyez sur « Ajouter » à côté du serveur qu'il vous faut ; pour un autre serveur, appuyez sur « Ajouter un serveur » et remplissez nom, type et programme ou adresse.",
              "Drück unter „Fertige Server“ neben dem gewünschten Server auf „Hinzufügen“; für einen anderen Server drück „Server hinzufügen“ und gib Name, Typ und Programm oder Adresse ein.",
            ),
            tr(
              "Atollo scrive mcp.json e allinea subito gli agenti: Claude Code, Codex, Antigravity e Pi hanno il server senza aspettare il prossimo avvio. «Togli» su una riga lo rimuove.",
              "Atollo writes mcp.json and syncs the agents at once: Claude Code, Codex, Antigravity and Pi have the server without waiting for their next start. “Remove” on a row removes it.",
              "Atollo écrit mcp.json et synchronise aussitôt les agents : Claude Code, Codex, Antigravity et Pi ont le serveur sans attendre leur prochain lancement. « Retirer » sur une ligne le supprime.",
              "Atollo schreibt mcp.json und gleicht die Agenten sofort ab: Claude Code, Codex, Antigravity und Pi haben den Server, ohne auf ihren nächsten Start zu warten. „Entfernen“ in einer Zeile entfernt ihn.",
            ),
          ]}
        />
        <Table
          columns={[
            tr("Server pronto", "Ready server", "Serveur prêt", "Fertiger Server"),
            tr("Che cosa fa", "What it does", "Ce qu'il fait", "Was er tut"),
            tr("Dove", "Where", "Où", "Wo"),
          ]}
          rows={[
            [
              <Code key="g">github</Code>,
              tr(
                "Issue, pull request e repository su GitHub, con l'account GitHub del kit. È il server di GitHub stesso, in rete (api.githubcopilot.com/mcp): niente da scaricare.",
                "Issues, pull requests and repositories on GitHub, with the kit's GitHub account. It is GitHub's own server, online (api.githubcopilot.com/mcp): nothing to download.",
                "Issues, pull requests et dépôts sur GitHub, avec le compte GitHub du kit. C'est le serveur de GitHub lui-même, en ligne (api.githubcopilot.com/mcp) : rien à télécharger.",
                "Issues, Pull Requests und Repositorys auf GitHub, mit dem GitHub-Konto des Kits. Es ist der Server von GitHub selbst, online (api.githubcopilot.com/mcp): nichts herunterzuladen.",
              ),
              tr("il kit", "the kit", "le kit", "das Kit"),
            ],
            [
              <Code key="p">playwright</Code>,
              tr(
                "Un browser che l'agente apre e usa: pagine, clic, moduli, schermate.",
                "A browser the agent opens and uses: pages, clicks, forms, screenshots.",
                "Un navigateur que l'agent ouvre et utilise : pages, clics, formulaires, captures d'écran.",
                "Ein Browser, den der Agent öffnet und benutzt: Seiten, Klicks, Formulare, Bildschirmfotos.",
              ),
              tr("il kit", "the kit", "le kit", "das Kit"),
            ],
            [
              <Code key="f">fetch</Code>,
              tr(
                "Legge una pagina web e la dà all'agente come testo.",
                "Reads a web page and gives it to the agent as text.",
                "Lit une page web et la donne à l'agent sous forme de texte.",
                "Liest eine Webseite und gibt sie dem Agenten als Text.",
              ),
              tr("il kit", "the kit", "le kit", "das Kit"),
            ],
            [
              <Code key="s">filesystem</Code>,
              tr(
                "Legge e scrive i file, solo nella cartella del progetto.",
                "Reads and writes files, only in the project's folder.",
                "Lit et écrit les fichiers, seulement dans le dossier du projet.",
                "Liest und schreibt Dateien, nur im Ordner des Projekts.",
              ),
              tr("il progetto", "the project", "le projet", "das Projekt"),
            ],
            [
              <Code key="m">mysql</Code>,
              tr(
                "Interroga il database MySQL del progetto (DBHub), quando è stato creato.",
                "Queries the project's MySQL database (DBHub), once it has been created.",
                "Interroge la base de données MySQL du projet (DBHub), une fois qu'elle a été créée.",
                "Fragt die MySQL-Datenbank des Projekts ab (DBHub), sobald sie angelegt ist.",
              ),
              tr("il progetto", "the project", "le projet", "das Projekt"),
            ],
            [
              <Code key="d">mongodb</Code>,
              tr(
                "Interroga il database MongoDB del progetto, quando è stato creato.",
                "Queries the project's MongoDB database, once it has been created.",
                "Interroge la base de données MongoDB du projet, une fois qu'elle a été créée.",
                "Fragt die MongoDB-Datenbank des Projekts ab, sobald sie angelegt ist.",
              ),
              tr("il progetto", "the project", "le projet", "das Projekt"),
            ],
          ]}
        />
        <Callout
          tone="info"
          title={tr(
            "Nessun segreto nei file",
            "No secret in the files",
            "Aucun secret dans les fichiers",
            "Kein Geheimnis in den Dateien",
          )}
        >
          {tr(
            "Il token di GitHub e gli indirizzi dei database non sono mai scritti in mcp.json o in .mcp.json: il server legge una variabile (${GITHUB_PERSONAL_ACCESS_TOKEN}, ${DSN}, ${MDB_MCP_CONNECTION_STRING}) che Atollo dà all'agente quando lo avvia. Il token di GitHub va solo ai server del kit, mai a quelli che il .mcp.json di un progetto chiede: un repository clonato potrebbe mandarlo ovunque. Per GitHub serve l'accesso a GitHub del kit. Un agente avviato da un terminale dell'editor, invece che dall'app, non ha queste variabili.",
            "The GitHub token and the databases' addresses are never written in mcp.json or in .mcp.json: the server reads a variable (${GITHUB_PERSONAL_ACCESS_TOKEN}, ${DSN}, ${MDB_MCP_CONNECTION_STRING}) that Atollo gives the agent when it starts it. The GitHub token goes only to the kit's servers, never to those a project's .mcp.json asks for: a cloned repository could send it anywhere. GitHub needs the kit to be signed in to GitHub. An agent started from an editor terminal, rather than from the app, does not have these variables.",
            "Le jeton GitHub et les adresses des bases de données ne sont jamais écrits dans mcp.json ni dans .mcp.json : le serveur lit une variable (${GITHUB_PERSONAL_ACCESS_TOKEN}, ${DSN}, ${MDB_MCP_CONNECTION_STRING}) qu'Atollo donne à l'agent quand il le lance. Le jeton GitHub ne va qu'aux serveurs du kit, jamais à ceux que demande le .mcp.json d'un projet : un dépôt cloné pourrait l'envoyer n'importe où. GitHub demande que le kit soit connecté à GitHub. Un agent lancé depuis un terminal de l'éditeur, plutôt que depuis l'application, n'a pas ces variables.",
            "Das GitHub-Token und die Adressen der Datenbanken werden nie in mcp.json oder .mcp.json geschrieben: Der Server liest eine Variable (${GITHUB_PERSONAL_ACCESS_TOKEN}, ${DSN}, ${MDB_MCP_CONNECTION_STRING}), die Atollo dem Agenten beim Start gibt. Das GitHub-Token geht nur an die Server des Kits, nie an die, die die .mcp.json eines Projekts verlangt: Ein geklontes Repository könnte es überallhin schicken. GitHub setzt voraus, dass das Kit bei GitHub angemeldet ist. Ein Agent, der aus einem Terminal des Editors statt aus der App gestartet wird, hat diese Variablen nicht.",
          )}
        </Callout>
      </Section>

      <Section
        title={tr(
          "Come avviene l'allineamento",
          "How the sync works",
          "Comment se fait la synchronisation",
          "Wie der Abgleich funktioniert",
        )}
      >
        <Bullets
          items={[
            tr(
              "Istruzioni: AGENTS.md viene copiato come CLAUDE.md per Claude Code e come AGENTS.md per Codex, Antigravity e Pi. Se modifichi uno di quei file, la modifica torna in AGENTS.md e passa agli altri. Se cambiano entrambe le parti, vince la più recente e l'altra viene conservata nella cartella dei conflitti.",
              "Instructions: AGENTS.md is copied as CLAUDE.md for Claude Code and as AGENTS.md for Codex, Antigravity and Pi. If you edit one of those files, the change goes back into AGENTS.md and on to the others. If both sides change, the most recent wins and the other is kept in the conflicts folder.",
              "Instructions : AGENTS.md est copié en CLAUDE.md pour Claude Code et en AGENTS.md pour Codex, Antigravity et Pi. Si vous modifiez l'un de ces fichiers, la modification revient dans AGENTS.md et passe aux autres. Si les deux côtés changent, le plus récent l'emporte et l'autre est conservé dans le dossier des conflits.",
              "Anweisungen: AGENTS.md wird als CLAUDE.md für Claude Code und als AGENTS.md für Codex, Antigravity und Pi kopiert. Wenn du eine dieser Dateien änderst, geht die Änderung zurück in AGENTS.md und an die anderen. Ändern sich beide Seiten, gewinnt die neuere, und die andere wird im Konfliktordner aufbewahrt.",
            ),
            tr(
              "Skill: ogni agente vede la cartella skills attraverso un collegamento (copie sui dischi FAT32 ed exFAT). Una skill creata dentro un agente viene spostata in data\\agents\\skills e diventa di tutti; una skill cancellata in un agente viene messa da parte e si può recuperare.",
              "Skills: each agent sees the skills folder through a link (copies on FAT32 and exFAT drives). A skill created inside an agent is moved to data\\agents\\skills and becomes everyone's; a skill deleted in an agent is set aside and can be recovered.",
              "Skills : chaque agent voit le dossier skills à travers un lien (des copies sur les disques FAT32 et exFAT). Une skill créée dans un agent est déplacée dans data\\agents\\skills et devient commune à tous ; une skill supprimée dans un agent est mise de côté et peut être récupérée.",
              "Skills: Jeder Agent sieht den Ordner skills über eine Verknüpfung (Kopien auf FAT32- und exFAT-Laufwerken). Ein in einem Agenten erstellter Skill wird nach data\\agents\\skills verschoben und gehört dann allen; ein in einem Agenten gelöschter Skill wird beiseitegelegt und lässt sich wiederherstellen.",
            ),
            tr(
              "Server MCP: mcp.json viene tradotto nel formato di ciascun agente. Con Claude Code vale in entrambe le direzioni: un server aggiunto con claude mcp add -s user torna in mcp.json. Codex, Antigravity e Pi ricevono i server del kit e lasciano stare quelli che hai aggiunto a mano nelle loro configurazioni. Pi non accetta i server SSE: il kit li lascia fuori dal suo mcp.json, con un avviso.",
              "MCP servers: mcp.json is translated into each agent's format. With Claude Code it works in both directions: a server added with claude mcp add -s user goes back into mcp.json. Codex, Antigravity and Pi receive the kit's servers and leave alone the ones you added by hand in their configurations. Pi does not accept SSE servers: the kit leaves them out of its mcp.json, with a warning.",
              "Serveurs MCP : mcp.json est traduit dans le format de chaque agent. Avec Claude Code, cela vaut dans les deux sens : un serveur ajouté avec claude mcp add -s user revient dans mcp.json. Codex, Antigravity et Pi reçoivent les serveurs du kit et ne touchent pas à ceux que vous avez ajoutés à la main dans leurs configurations. Pi n'accepte pas les serveurs SSE : le kit les laisse hors de son mcp.json, avec un avertissement.",
              "MCP-Server: mcp.json wird in das Format jedes Agenten übersetzt. Bei Claude Code gilt das in beide Richtungen: Ein mit claude mcp add -s user hinzugefügter Server kommt zurück in mcp.json. Codex, Antigravity und Pi erhalten die Server des Kits und lassen die, die du von Hand in ihren Konfigurationen hinzugefügt hast, unangetastet. Pi akzeptiert keine SSE-Server: Das Kit lässt sie mit einer Warnung aus seiner mcp.json weg.",
            ),
          ]}
        />
        <Callout tone="info" title={tr("La regola del kit", "The kit's rule", "La règle du kit", "Die Regel des Kits")}>
          {tr(
            "A ogni allineamento il kit dà agli agenti una regola comune: nei progetti i file CLAUDE.md, AGENTS.md e GEMINI.md sono equivalenti, e le skill di un progetto stanno nella sua cartella .agents/skills. Per Pi la regola sta in un blocco marcato di APPEND_SYSTEM.md, riscritto a ogni allineamento: il testo che ci scrivi tu, fuori dal blocco, resta.",
            "At every sync the kit gives the agents a common rule: in projects the files CLAUDE.md, AGENTS.md and GEMINI.md are equivalent, and a project's skills are in its .agents/skills folder. For Pi the rule is in a marked block of APPEND_SYSTEM.md, rewritten at every sync: the text you write there, outside the block, stays.",
            "À chaque synchronisation, le kit donne aux agents une règle commune : dans les projets, les fichiers CLAUDE.md, AGENTS.md et GEMINI.md sont équivalents, et les skills d'un projet se trouvent dans son dossier .agents/skills. Pour Pi, la règle se trouve dans un bloc balisé d'APPEND_SYSTEM.md, réécrit à chaque synchronisation : le texte que vous y écrivez, hors du bloc, reste.",
            "Bei jedem Abgleich gibt das Kit den Agenten eine gemeinsame Regel: In Projekten sind die Dateien CLAUDE.md, AGENTS.md und GEMINI.md gleichwertig, und die Skills eines Projekts liegen in seinem Ordner .agents/skills. Für Pi steht die Regel in einem markierten Block von APPEND_SYSTEM.md, der bei jedem Abgleich neu geschrieben wird: Der Text, den du dort außerhalb des Blocks schreibst, bleibt.",
          )}
        </Callout>
      </Section>

      <Section
        title={tr(
          "Server MCP di un solo progetto",
          "MCP servers for a single project",
          "Serveurs MCP d'un seul projet",
          "MCP-Server für ein einzelnes Projekt",
        )}
      >
        <Steps
          steps={[
            tr(
              "Apri il progetto, sezione «Editor», riquadro «Server MCP del progetto».",
              "Open the project, “Editor” section, “Project MCP servers” box.",
              "Ouvrez le projet, section « Éditeur », panneau « Serveurs MCP du projet ».",
              "Öffne das Projekt, Abschnitt „Editor“, Bereich „MCP-Server des Projekts“.",
            ),
            tr(
              "Per un server pronto premi «Aggiungi» accanto a lui in «Server pronti»: filesystem (i file, solo nella cartella del progetto) e, quando il database del progetto è stato creato con «Crea database e utente», mysql o mongodb. Per un altro server continua dal passo seguente.",
              "For a ready server press “Add” beside it under “Ready servers”: filesystem (the files, only in the project's folder) and, once the project's database has been created with “Create database and user”, mysql or mongodb. For another server go on with the next step.",
              "Pour un serveur prêt, appuyez sur « Ajouter » à côté de lui dans « Serveurs prêts » : filesystem (les fichiers, seulement dans le dossier du projet) et, une fois la base de données du projet créée avec « Créer la base et l'utilisateur », mysql ou mongodb. Pour un autre serveur, continuez à l'étape suivante.",
              "Für einen fertigen Server drück unter „Fertige Server“ daneben auf „Hinzufügen“: filesystem (die Dateien, nur im Ordner des Projekts) und, sobald die Datenbank des Projekts mit „Datenbank und Benutzer erstellen“ angelegt ist, mysql oder mongodb. Für einen anderen Server mach mit dem nächsten Schritt weiter.",
            ),
            tr(
              "Premi «Aggiungi un server».",
              "Press “Add a server”.",
              "Appuyez sur « Ajouter un serveur ».",
              "Drück „Server hinzufügen“.",
            ),
            tr(
              "Scrivi il «Nome» (lettere, cifre, - e _) e scegli il «Tipo»: «programma» (con «Programma», per esempio npx, e gli «Argomenti, uno per riga») oppure «indirizzo (http)» (con l'«Indirizzo»).",
              "Write the “Name” (letters, digits, - and _) and choose the “Type”: “program” (with “Program”, for example npx, and the “Arguments, one per line”) or “address (http)” (with the “Address”).",
              "Écrivez le « Nom » (lettres, chiffres, - et _) et choisissez le « Type » : « programme » (avec « Programme », par exemple npx, et les « Arguments, un par ligne ») ou « adresse (http) » (avec l'« Adresse »).",
              "Gib den „Name“ ein (Buchstaben, Ziffern, - und _) und wähle den „Typ“: „Programm“ (mit „Programm“, zum Beispiel npx, und den „Argumente, eines pro Zeile“) oder „Adresse (http)“ (mit der „Adresse“).",
            ),
            tr(
              "Premi «Aggiungi». «Togli» su una riga lo rimuove.",
              "Press “Add”. “Remove” on a row removes it.",
              "Appuyez sur « Ajouter ». « Retirer » sur une ligne le supprime.",
              "Drück „Hinzufügen“. „Entfernen“ in einer Zeile entfernt ihn.",
            ),
          ]}
        />
        <P>
          {tr(
            "Questi server valgono solo in quel progetto, oltre a quelli del kit. Claude Code li legge dal file .mcp.json del progetto (e chiede conferma al primo uso); Codex da .codex\\config.toml, che Atollo tiene allineato; Antigravity e Pi usano solo i server del kit. I programmi che servono (Node per npx, uv per uvx) si installano all'avvio dell'agente; il database del progetto arriva al server mysql o mongodb come variabile, mai scritto in .mcp.json, che può stare nel repository.",
            "These servers apply only in that project, in addition to the kit's. Claude Code reads them from the project's .mcp.json file (and asks for confirmation on first use); Codex from .codex\\config.toml, which Atollo keeps in sync; Antigravity and Pi use only the kit's servers. The programs they need (Node for npx, uv for uvx) are installed when the agent starts; the project's database reaches the mysql or mongodb server as a variable, never written in .mcp.json, which may be in the repository.",
            "Ces serveurs ne valent que dans ce projet, en plus de ceux du kit. Claude Code les lit dans le fichier .mcp.json du projet (et demande confirmation à la première utilisation) ; Codex dans .codex\\config.toml, qu'Atollo garde synchronisé ; Antigravity et Pi n'utilisent que les serveurs du kit. Les programmes nécessaires (Node pour npx, uv pour uvx) s'installent au lancement de l'agent ; la base de données du projet arrive au serveur mysql ou mongodb sous forme de variable, jamais écrite dans .mcp.json, qui peut se trouver dans le dépôt.",
            "Diese Server gelten nur in diesem Projekt, zusätzlich zu denen des Kits. Claude Code liest sie aus der Datei .mcp.json des Projekts (und fragt bei der ersten Verwendung nach einer Bestätigung); Codex aus .codex\\config.toml, das Atollo abgeglichen hält; Antigravity und Pi verwenden nur die Server des Kits. Die nötigen Programme (Node für npx, uv für uvx) werden beim Start des Agenten installiert; die Datenbank des Projekts erreicht den Server mysql oder mongodb als Variable, nie in .mcp.json geschrieben, die im Repository liegen kann.",
          )}
        </P>
      </Section>

      <Under>
        <Table
          columns={[
            tr("Originale", "Original", "Original", "Original"),
            tr("Copie per gli agenti", "Copies for the agents", "Copies pour les agents", "Kopien für die Agenten"),
          ]}
          rows={[
            [
              <Code key="a">data\agents\AGENTS.md</Code>,
              <Code key="b">
                data\home\.claude\CLAUDE.md · data\home\.codex\AGENTS.md · data\home\.gemini\AGENTS.md ·
                data\home\.pi\agent\AGENTS.md
              </Code>,
            ],
            [
              <Code key="c">data\agents\skills</Code>,
              <Code key="d">
                data\home\.claude\skills · data\home\.codex\skills · data\home\.gemini\antigravity-cli\skills ·
                data\home\.pi\agent\skills
              </Code>,
            ],
            [
              <Code key="e">data\agents\mcp.json</Code>,
              <Code key="f">
                data\home\.claude\.claude.json · data\home\.codex\config.toml · data\home\.gemini\config\mcp_config.json
                · data\home\.pi\agent\mcp.json
              </Code>,
            ],
          ]}
        />
        <Bullets
          items={[
            tr(
              "Lo stato dell'allineamento, i conflitti e le skill cancellate sono in data\\agents\\.sync (conflicts, deleted).",
              "The sync state, the conflicts and the deleted skills are in data\\agents\\.sync (conflicts, deleted).",
              "L'état de la synchronisation, les conflits et les skills supprimées sont dans data\\agents\\.sync (conflicts, deleted).",
              "Der Abgleichstatus, die Konflikte und die gelöschten Skills liegen in data\\agents\\.sync (conflicts, deleted).",
            ),
            tr(
              "Gli avvisi dell'allineamento finiscono nel log dell'app, data\\app\\logs\\app.log.",
              "Sync warnings end up in the app's log, data\\app\\logs\\app.log.",
              "Les avertissements de la synchronisation finissent dans le journal de l'application, data\\app\\logs\\app.log.",
              "Warnungen des Abgleichs landen im Protokoll der App, data\\app\\logs\\app.log.",
            ),
            rich(
              tr(
                "L'allineamento si può lanciare anche dalla riga di comando: {0}.",
                "You can also run the sync from the command line: {0}.",
                "La synchronisation peut aussi se lancer depuis la ligne de commande : {0}.",
                "Der Abgleich lässt sich auch über die Kommandozeile starten: {0}.",
              ),
              <Code key="s">{`Atollo.exe sync [--project ${tr("<cartella>", "<folder>", "<dossier>", "<Ordner>")}] [--verbose]`}</Code>,
            ),
            tr(
              "Claude Code, Codex e Pi considerano attendibile la cartella di un progetto solo dopo che vi hai aperto un agente da Atollo: da lì possono eseguire comandi senza chiedere la fiducia ogni volta. Le altre cartelle di projects non sono attendibili in anticipo.",
              "Claude Code, Codex and Pi trust a project's folder only after you open an agent in that project from Atollo: from then on they can run commands there without asking for trust every time. The other folders in projects are not trusted in advance.",
              "Claude Code, Codex et Pi ne considèrent le dossier d'un projet comme fiable qu'après que vous y avez ouvert un agent depuis Atollo : dès lors ils peuvent y exécuter des commandes sans demander la confiance à chaque fois. Les autres dossiers de projects ne sont pas fiables d'avance.",
              "Claude Code, Codex und Pi vertrauen dem Ordner eines Projekts erst, nachdem du darin einen Agenten aus Atollo geöffnet hast: Ab dann können sie dort Befehle ausführen, ohne jedes Mal nach Vertrauen zu fragen. Die anderen Ordner in projects gelten nicht im Voraus als vertrauenswürdig.",
            ),
            tr(
              "In Codex le variabili ${VAR} non vengono espanse: il kit le traduce dove Codex ha una chiave apposita, altrimenti le copia così come sono con un commento.",
              "In Codex ${VAR} variables are not expanded: the kit translates them where Codex has a dedicated key, otherwise it copies them as they are with a comment.",
              "Dans Codex, les variables ${VAR} ne sont pas développées : le kit les traduit là où Codex a une clé dédiée, sinon il les copie telles quelles avec un commentaire.",
              "In Codex werden ${VAR}-Variablen nicht ersetzt: Das Kit übersetzt sie dort, wo Codex einen eigenen Schlüssel hat, und kopiert sie sonst unverändert mit einem Kommentar.",
            ),
            tr(
              "Pi tiene tutto in data\\home\\.pi\\agent (la variabile PI_CODING_AGENT_DIR). Oltre a istruzioni, skill e server MCP, il kit vi scrive models.json (il provider «atollo»: i modelli locali su disco, con la porta e la chiave del server, solo per il proprietario) e settings.json (su Windows il Git Bash del kit come shell, la fiducia nei progetti «ask» se non l'hai impostata, il modello locale come predefinito finché non scegli un altro provider). I tuoi provider e le tue impostazioni restano.",
              "Pi keeps everything in data\\home\\.pi\\agent (the PI_CODING_AGENT_DIR variable). Besides instructions, skills and MCP servers, the kit writes there models.json (the “atollo” provider: the local models on disk, with the server's port and key, owner-only) and settings.json (on Windows the kit's Git Bash as the shell, project trust “ask” if you have not set it, the local model as the default until you choose another provider). Your own providers and settings stay.",
              "Pi garde tout dans data\\home\\.pi\\agent (la variable PI_CODING_AGENT_DIR). En plus des instructions, skills et serveurs MCP, le kit y écrit models.json (le fournisseur « atollo » : les modèles locaux sur le disque, avec le port et la clé du serveur, réservé au propriétaire) et settings.json (sur Windows le Git Bash du kit comme shell, la confiance dans les projets « ask » si vous ne l'avez pas réglée, le modèle local par défaut tant que vous ne choisissez pas un autre fournisseur). Vos propres fournisseurs et paramètres restent.",
              "Pi bewahrt alles in data\\home\\.pi\\agent auf (die Variable PI_CODING_AGENT_DIR). Neben Anweisungen, Skills und MCP-Servern schreibt das Kit dort models.json (den Anbieter „atollo“: die lokalen Modelle auf dem Datenträger, mit Port und Schlüssel des Servers, nur für den Eigentümer lesbar) und settings.json (unter Windows die Git Bash des Kits als Shell, das Projektvertrauen „ask“, wenn du es nicht eingestellt hast, das lokale Modell als Standard, bis du einen anderen Anbieter wählst). Deine eigenen Anbieter und Einstellungen bleiben.",
            ),
          ]}
        />
      </Under>

      <Related ids={["agents", "project-editor", "cli"]} />
    </>
  ),

  "agents-usage": () => (
    <>
      <InBrief
        points={[
          tr(
            "«Limiti del piano» mostra quanta parte del tuo piano hai usato in ogni finestra e quando si azzera.",
            "“Plan limits” shows how much of your plan you have used in each window and when it resets.",
            "« Limites de l'abonnement » montre quelle part de votre abonnement vous avez utilisée dans chaque fenêtre et quand elle se réinitialise.",
            "„Tariflimits“ zeigt, wie viel deines Tarifs du in jedem Zeitfenster verbraucht hast und wann es zurückgesetzt wird.",
          ),
          tr(
            "«Consumi» mostra token e sessioni di oggi e degli ultimi sette giorni, con un grafico per giorno.",
            "“Usage” shows tokens and sessions for today and the last seven days, with a chart per day.",
            "« Consommation » montre les jetons et les sessions d'aujourd'hui et des sept derniers jours, avec un graphique par jour.",
            "„Verbrauch“ zeigt Tokens und Sitzungen von heute und den letzten sieben Tagen, mit einem Diagramm pro Tag.",
          ),
          tr(
            "I limiti si leggono dai server del fornitore ogni due minuti; «Aggiorna i consumi» li rilegge subito.",
            "The limits are read from the provider's servers every two minutes; “Refresh usage” reads them again right away.",
            "Les limites sont lues sur les serveurs du fournisseur toutes les deux minutes ; « Actualiser la consommation » les relit tout de suite.",
            "Die Limits werden alle zwei Minuten von den Servern des Anbieters abgefragt; „Verbrauch aktualisieren“ fragt sie sofort neu ab.",
          ),
        ]}
      />

      <Section title={tr("Limiti del piano", "Plan limits", "Limites de l'abonnement", "Tariflimits")}>
        <P>
          {tr(
            "Per ogni finestra di limite c'è una casella con la percentuale usata, una barra e «si azzera …». Le finestre sono «5 ore» e «Settimana» per Claude Code, «… ore» e «Settimana» per Codex, e quote per modello per Antigravity. Sopra, il piano, la fonte dei dati e quando sono stati letti. Pi non ha finestre di limite: i modelli locali non ne hanno, e il kit non interroga i provider in cloud che Pi può usare.",
            "For each limit window there is a box with the percentage used, a bar and “resets …”. The windows are “5 hours” and “Week” for Claude Code, “… hours” and “Week” for Codex, and per-model quotas for Antigravity. Above: the plan, the data source and when the data was read. Pi has no limit windows: the local models have none, and the kit does not query the cloud providers Pi may use.",
            "Pour chaque fenêtre de limite, il y a une case avec le pourcentage utilisé, une barre et « se réinitialise … ». Les fenêtres sont « 5 heures » et « Semaine » pour Claude Code, « … heures » et « Semaine » pour Codex, et des quotas par modèle pour Antigravity. Au-dessus : l'abonnement, la source des données et quand elles ont été lues. Pi n'a pas de fenêtres de limite : les modèles locaux n'en ont pas, et le kit n'interroge pas les fournisseurs dans le cloud que Pi peut utiliser.",
            "Für jedes Limit-Zeitfenster gibt es ein Feld mit dem verbrauchten Prozentsatz, einem Balken und „wird … zurückgesetzt“. Die Zeitfenster sind „5 Stunden“ und „Woche“ für Claude Code, „… Stunden“ und „Woche“ für Codex und Kontingente pro Modell für Antigravity. Darüber: der Tarif, die Datenquelle und wann die Daten abgefragt wurden. Pi hat keine Limit-Zeitfenster: Die lokalen Modelle haben keine, und das Kit fragt die Cloud-Anbieter, die Pi verwenden kann, nicht ab.",
          )}
        </P>
        <Table
          columns={[tr("Colore", "Color", "Couleur", "Farbe"), tr("Quando", "When", "Quand", "Wann")]}
          rows={[
            [tr("Verde", "Green", "Vert", "Grün"), tr("sotto il 70%", "below 70%", "sous 70%", "unter 70%")],
            [tr("Arancione", "Orange", "Orange", "Orange"), tr("dal 70%", "from 70%", "à partir de 70%", "ab 70%")],
            [tr("Rosso", "Red", "Rouge", "Rot"), tr("dal 90%", "from 90%", "à partir de 90%", "ab 90%")],
          ]}
        />
        <P>
          {tr(
            "Il valore più alto compare anche nella riga dell'agente nella barra laterale. Per Claude Code i crediti extra a pagamento sono mostrati a parte («Crediti extra: … di …»).",
            "The highest value also appears on the agent's row in the sidebar. For Claude Code, paid extra credits are shown separately (“Extra credits: … of …”).",
            "La valeur la plus haute apparaît aussi sur la ligne de l'agent dans la barre latérale. Pour Claude Code, les crédits supplémentaires payants sont affichés à part (« Crédits supplémentaires : … sur … »).",
            "Der höchste Wert erscheint auch in der Zeile des Agenten in der Seitenleiste. Für Claude Code wird das bezahlte Zusatzguthaben separat angezeigt („Zusatzguthaben: … von …“).",
          )}
        </P>
        <Table
          columns={[
            tr("Messaggio", "Message", "Message", "Meldung"),
            tr("Che cosa fare", "What to do", "Que faire", "Was tun"),
          ]}
          rows={[
            [
              tr(
                "accesso non fatto: apri … per vedere limiti e consumi",
                "not signed in: open … to see limits and usage",
                "non connecté : ouvrez … pour voir les limites et la consommation",
                "nicht angemeldet: … öffnen, um Limits und Verbrauch zu sehen",
              ),
              tr(
                "Apri l'agente in un progetto e fai l'accesso.",
                "Open the agent in a project and sign in.",
                "Ouvrez l'agent dans un projet et connectez-vous.",
                "Öffne den Agenten in einem Projekt und melde dich an.",
              ),
            ],
            [
              tr(
                "accesso scaduto: apri Claude Code per rinnovarlo",
                "sign-in expired: open Claude Code to renew it",
                "connexion expirée : ouvrez Claude Code pour la renouveler",
                "Anmeldung abgelaufen: Claude Code öffnen, um sie zu erneuern",
              ),
              tr(
                "Apri l'agente: rinnova l'accesso da solo.",
                "Open the agent: it renews the sign-in by itself.",
                "Ouvrez l'agent : il renouvelle la connexion tout seul.",
                "Öffne den Agenten: Er erneuert die Anmeldung von selbst.",
              ),
            ],
            [
              tr(
                "chiave API: nessun limite di piano",
                "API key: no plan limits",
                "clé API : aucune limite d'abonnement",
                "API-Schlüssel: keine Tariflimits",
              ),
              tr(
                "Con una chiave API non ci sono finestre di limite.",
                "With an API key there are no limit windows.",
                "Avec une clé API, il n'y a pas de fenêtres de limite.",
                "Mit einem API-Schlüssel gibt es keine Limit-Zeitfenster.",
              ),
            ],
            [
              tr(
                "Modelli locali: nessun limite di piano",
                "Local models: no plan limits",
                "Modèles locaux : aucune limite de forfait",
                "Lokale Modelle: keine Tariflimits",
              ),
              tr(
                "Pi sui modelli locali del kit: girano sul tuo PC, senza piano. I consumi qui sotto ci sono comunque.",
                "Pi on the kit's local models: they run on your PC, with no plan. The usage below is still there.",
                "Pi sur les modèles locaux du kit : ils tournent sur votre PC, sans forfait. La consommation ci-dessous reste affichée.",
                "Pi mit den lokalen Modellen des Kits: Sie laufen auf deinem PC, ohne Tarif. Der Verbrauch unten wird trotzdem angezeigt.",
              ),
            ],
            [
              tr(
                "I limiti dei provider di Pi non sono letti dal kit",
                "The limits of Pi's providers are not read by the kit",
                "Les limites des fournisseurs de Pi ne sont pas lues par le kit",
                "Die Limits der Anbieter von Pi liest das Kit nicht",
              ),
              tr(
                "Pi con un provider in cloud: i limiti si vedono nel sito del provider o in Pi.",
                "Pi with a cloud provider: you see the limits on the provider's site or in Pi.",
                "Pi avec un fournisseur dans le cloud : les limites se voient sur le site du fournisseur ou dans Pi.",
                "Pi mit einem Cloud-Anbieter: Die Limits siehst du auf der Website des Anbieters oder in Pi.",
              ),
            ],
            [
              tr(
                "nessun dato ancora: i limiti arrivano con la prossima risposta di Codex",
                "no data yet: the limits arrive with Codex's next reply",
                "pas encore de données : les limites arrivent avec la prochaine réponse de Codex",
                "noch keine Daten: die Limits kommen mit der nächsten Antwort von Codex",
              ),
              tr(
                "Usa Codex una volta: i limiti arrivano con la risposta.",
                "Use Codex once: the limits arrive with the response.",
                "Utilisez Codex une fois : les limites arrivent avec la réponse.",
                "Verwende Codex einmal: Die Limits kommen mit der Antwort.",
              ),
            ],
            [
              tr(
                "Il server chiede di aspettare: nuova lettura …",
                "The server asks to wait: next reading …",
                "Le serveur demande d'attendre : nouvelle lecture …",
                "Der Server bittet zu warten: nächste Abfrage …",
              ),
              tr(
                "Il fornitore ha limitato le letture: il kit riprova da solo più tardi.",
                "The provider has limited the readings: the kit tries again by itself later.",
                "Le fournisseur a limité les lectures : le kit réessaie tout seul plus tard.",
                "Der Anbieter hat die Abfragen begrenzt: Das Kit versucht es später von selbst erneut.",
              ),
            ],
          ]}
        />
      </Section>

      <Section title={tr("Consumi", "Usage", "Consommation", "Verbrauch")}>
        <Bullets
          items={[
            tr(
              "«Token · Oggi» con i token di oggi e degli ultimi 7 giorni; passandoci sopra, quanti sono nuovi, quanti presi dalla cache e quanti in uscita.",
              "“Tokens · Today” with the tokens of today and the last 7 days; hovering over it shows how many are new, how many come from the cache and how many are output.",
              "« Jetons · Aujourd'hui » avec les jetons d'aujourd'hui et des 7 derniers jours ; en passant dessus, combien sont nouveaux, combien viennent du cache et combien sont en sortie.",
              "„Tokens · Heute“ mit den Tokens von heute und den letzten 7 Tagen; beim Darüberfahren siehst du, wie viele neu sind, wie viele aus dem Cache kommen und wie viele ausgegeben wurden.",
            ),
            tr(
              "«Sessioni · Oggi» con le sessioni di oggi e degli ultimi 7 giorni.",
              "“Sessions · Today” with the sessions of today and the last 7 days.",
              "« Sessions · Aujourd'hui » avec les sessions d'aujourd'hui et des 7 derniers jours.",
              "„Sitzungen · Heute“ mit den Sitzungen von heute und den letzten 7 Tagen.",
            ),
            tr(
              "Un grafico a barre per i 7 giorni, l'ultima attività e i tre modelli più usati.",
              "A bar chart for the 7 days, the last activity and the three most used models.",
              "Un graphique à barres pour les 7 jours, la dernière activité et les trois modèles les plus utilisés.",
              "Ein Balkendiagramm für die 7 Tage, die letzte Aktivität und die drei meistgenutzten Modelle.",
            ),
            tr(
              "Antigravity non salva i token: per lui si contano solo le sessioni.",
              "Antigravity does not save tokens: for it only sessions are counted.",
              "Antigravity n'enregistre pas les jetons : pour lui, seules les sessions sont comptées.",
              "Antigravity speichert keine Tokens: Bei ihm werden nur die Sitzungen gezählt.",
            ),
            tr(
              "Per Pi i token si leggono dalle sue sessioni, anche quelle sui modelli locali.",
              "For Pi the tokens are read from its sessions, including those on the local models.",
              "Pour Pi, les jetons se lisent dans ses sessions, y compris celles sur les modèles locaux.",
              "Bei Pi werden die Tokens aus seinen Sitzungen gelesen, auch aus denen mit lokalen Modellen.",
            ),
          ]}
        />
        <Callout tone="info">
          {tr(
            "I consumi si calcolano dai file delle sessioni salvati nel kit, quindi comprendono le sessioni aperte da questo kit su qualsiasi PC. Non ci sono costi in denaro: solo token e sessioni.",
            "Usage is calculated from the session files saved in the kit, so it includes the sessions opened from this kit on any PC. There are no money costs: only tokens and sessions.",
            "La consommation se calcule à partir des fichiers de sessions enregistrés dans le kit, elle comprend donc les sessions ouvertes depuis ce kit sur n'importe quel PC. Il n'y a pas de coûts en argent : seulement des jetons et des sessions.",
            "Der Verbrauch wird aus den im Kit gespeicherten Sitzungsdateien berechnet und umfasst daher die Sitzungen, die von diesem Kit auf beliebigen PCs geöffnet wurden. Es gibt keine Geldbeträge: nur Tokens und Sitzungen.",
          )}
        </Callout>
      </Section>

      <Under>
        <Bullets
          items={[
            tr(
              "Fonti dei limiti: per Claude Code il server di Anthropic, per Codex quello di OpenAI (o, in mancanza, l'ultima risposta di Codex), per Antigravity quello di Google. Il kit usa l'accesso salvato, solo per leggere: non lo rinnova e non lo manda altrove.",
              "Sources of the limits: for Claude Code the Anthropic server, for Codex the OpenAI one (or, failing that, Codex's last response), for Antigravity the Google one. The kit uses the saved sign-in only to read: it does not renew it and does not send it anywhere else.",
              "Sources des limites : pour Claude Code le serveur d'Anthropic, pour Codex celui d'OpenAI (ou, à défaut, la dernière réponse de Codex), pour Antigravity celui de Google. Le kit utilise la connexion enregistrée uniquement pour lire : il ne la renouvelle pas et ne l'envoie nulle part ailleurs.",
              "Quellen der Limits: für Claude Code der Server von Anthropic, für Codex der von OpenAI (oder, falls nicht verfügbar, die letzte Antwort von Codex), für Antigravity der von Google. Das Kit verwendet die gespeicherte Anmeldung nur zum Lesen: Es erneuert sie nicht und sendet sie nirgendwohin.",
            ),
            tr(
              "Letture al massimo ogni 2 minuti, ogni 20 secondi con «Aggiorna i consumi»; dopo una risposta «troppe richieste» aspetta quanto chiesto dal server (da 2 a 30 minuti).",
              "Readings at most every 2 minutes, every 20 seconds with “Refresh usage”; after a “too many requests” response it waits as long as the server asks (from 2 to 30 minutes).",
              "Lectures au plus toutes les 2 minutes, toutes les 20 secondes avec « Actualiser la consommation » ; après une réponse « trop de requêtes », il attend le temps demandé par le serveur (de 2 à 30 minutes).",
              "Abfragen höchstens alle 2 Minuten, alle 20 Sekunden mit „Verbrauch aktualisieren“; nach einer Antwort „zu viele Anfragen“ wartet es so lange, wie der Server verlangt (2 bis 30 Minuten).",
            ),
            tr(
              "File delle sessioni: data\\home\\.claude\\projects, data\\home\\.codex\\sessions, data\\home\\.gemini (conversazioni di Antigravity), data\\home\\.pi\\agent\\sessions (Pi).",
              "Session files: data\\home\\.claude\\projects, data\\home\\.codex\\sessions, data\\home\\.gemini (Antigravity conversations), data\\home\\.pi\\agent\\sessions (Pi).",
              "Fichiers des sessions : data\\home\\.claude\\projects, data\\home\\.codex\\sessions, data\\home\\.gemini (conversations d'Antigravity), data\\home\\.pi\\agent\\sessions (Pi).",
              "Sitzungsdateien: data\\home\\.claude\\projects, data\\home\\.codex\\sessions, data\\home\\.gemini (Unterhaltungen von Antigravity), data\\home\\.pi\\agent\\sessions (Pi).",
            ),
          ]}
        />
      </Under>

      <Related ids={["agents", "agents-shared", "models"]} />
    </>
  ),

  models: () => (
    <>
      <InBrief
        points={[
          tr(
            "Un modello locale è un modello linguistico che gira sul tuo PC: niente rete, niente account, niente piano. Va sulla CPU o sulla scheda video (Vulkan su Windows e Linux, Metal su macOS).",
            "A local model is a language model that runs on your PC: no network, no account, no plan. It runs on the CPU or on the graphics card (Vulkan on Windows and Linux, Metal on macOS).",
            "Un modèle local est un modèle de langage qui tourne sur votre PC : pas de réseau, pas de compte, pas de forfait. Il tourne sur le processeur ou sur la carte graphique (Vulkan sur Windows et Linux, Metal sur macOS).",
            "Ein lokales Modell ist ein Sprachmodell, das auf deinem PC läuft: kein Netzwerk, kein Konto, kein Tarif. Es läuft auf der CPU oder auf der Grafikkarte (Vulkan unter Windows und Linux, Metal unter macOS).",
          ),
          tr(
            "Il server è llama.cpp, i modelli sono file GGUF: si scaricano dalla pagina «Modelli locali», dal catalogo di Atollo o da Hugging Face.",
            "The server is llama.cpp, the models are GGUF files: you download them from the “Local models” page, from Atollo's catalog or from Hugging Face.",
            "Le serveur est llama.cpp, les modèles sont des fichiers GGUF : ils se téléchargent depuis la page « Modèles locaux », depuis le catalogue d'Atollo ou depuis Hugging Face.",
            "Der Server ist llama.cpp, die Modelle sind GGUF-Dateien: Du lädst sie auf der Seite „Lokale Modelle“ herunter, aus dem Katalog von Atollo oder von Hugging Face.",
          ),
          tr(
            "Li usano Pi e una chat nel browser. Il server ascolta solo su questo PC, con una chiave.",
            "Pi and a chat in the browser use them. The server listens only on this PC, with a key.",
            "Pi et un chat dans le navigateur les utilisent. Le serveur n'écoute que sur ce PC, avec une clé.",
            "Pi und ein Chat im Browser verwenden sie. Der Server lauscht nur auf diesem PC, mit einem Schlüssel.",
          ),
          tr(
            "Un download sospeso o interrotto riprende da dove si era fermato, anche dopo aver chiuso Atollo.",
            "A paused or interrupted download resumes where it stopped, even after closing Atollo.",
            "Un téléchargement suspendu ou interrompu reprend là où il s'était arrêté, même après la fermeture d'Atollo.",
            "Ein angehaltener oder unterbrochener Download setzt dort fort, wo er aufgehört hat, auch nachdem du Atollo geschlossen hast.",
          ),
        ]}
      />

      <Section
        title={tr(
          "Che cosa sono e che cosa serve",
          "What they are and what you need",
          "Ce qu'ils sont et ce qu'il faut",
          "Was sie sind und was du brauchst",
        )}
      >
        <P>
          {tr(
            "Un modello GGUF è un unico file (a volte diviso in parti) con i pesi del modello già compressi: la «quantizzazione» (Q4_K_M, Q8_0, MXFP4…) dice quanto, e più è bassa più il file è piccolo e meno è preciso. Il server llama.cpp carica il modello nella memoria della scheda video, se ce n'è abbastanza, altrimenti lo divide fra scheda video e RAM o usa solo la CPU. La scelta è automatica: llama.cpp si installa la prima volta che avvii il server, una sola versione per sistema, verificata con il suo SHA-256.",
            "A GGUF model is a single file (sometimes split into parts) with the model's weights already compressed: the “quantization” (Q4_K_M, Q8_0, MXFP4…) says how much, and the lower it is, the smaller the file and the less precise it is. The llama.cpp server loads the model into the graphics card's memory if there is enough, otherwise it splits it between the graphics card and the RAM or uses the CPU only. The choice is automatic: llama.cpp installs itself the first time you start the server, a single version per system, verified with its SHA-256.",
            "Un modèle GGUF est un seul fichier (parfois divisé en parties) avec les poids du modèle déjà compressés : la « quantification » (Q4_K_M, Q8_0, MXFP4…) dit à quel point, et plus elle est basse, plus le fichier est petit et moins il est précis. Le serveur llama.cpp charge le modèle dans la mémoire de la carte graphique s'il y en a assez, sinon il le partage entre la carte graphique et la RAM ou utilise seulement le processeur. Le choix est automatique : llama.cpp s'installe la première fois que vous démarrez le serveur, une seule version par système, vérifiée avec son SHA-256.",
            "Ein GGUF-Modell ist eine einzige Datei (manchmal in Teile aufgeteilt) mit den bereits komprimierten Gewichten des Modells: Die „Quantisierung“ (Q4_K_M, Q8_0, MXFP4…) gibt an, wie stark, und je niedriger sie ist, desto kleiner die Datei und desto ungenauer das Modell. Der llama.cpp-Server lädt das Modell in den Speicher der Grafikkarte, wenn er reicht, sonst verteilt er es auf Grafikkarte und RAM oder nutzt nur die CPU. Die Wahl erfolgt automatisch: llama.cpp installiert sich beim ersten Start des Servers, eine einzige Version pro System, geprüft mit ihrer SHA-256.",
          )}
        </P>
        <P>
          {tr(
            "I modelli occupano molti GB e chiedono memoria: un modello da 7-9 miliardi di parametri sta in circa 8 GB, uno da 30 miliardi ne vuole più di 20. Sono meno capaci dei modelli in cloud di Claude Code o Codex, ma restano sul tuo PC.",
            "Models take up many GB and need memory: a model with 7-9 billion parameters fits in about 8 GB, one with 30 billion needs more than 20. They are less capable than the cloud models of Claude Code or Codex, but they stay on your PC.",
            "Les modèles occupent de nombreux Go et demandent de la mémoire : un modèle de 7 à 9 milliards de paramètres tient dans environ 8 Go, un de 30 milliards en demande plus de 20. Ils sont moins capables que les modèles dans le cloud de Claude Code ou Codex, mais ils restent sur votre PC.",
            "Modelle belegen viele GB und brauchen Speicher: Ein Modell mit 7 bis 9 Milliarden Parametern passt in etwa 8 GB, eines mit 30 Milliarden braucht mehr als 20. Sie sind weniger leistungsfähig als die Cloud-Modelle von Claude Code oder Codex, bleiben aber auf deinem PC.",
          )}
        </P>
      </Section>

      <Section
        title={tr(
          "La pagina «Modelli locali»",
          "The “Local models” page",
          "La page « Modèles locaux »",
          "Die Seite „Lokale Modelle“",
        )}
      >
        <P>
          {tr(
            "Si apre dalla barra delle attività (l'icona del chip). A sinistra, «Nel kit»: il server llama.cpp con il suo stato e la porta, e i modelli scaricati («modello di Pi», «in memoria», «scaricamento sospeso»). In alto, «Avvia» o «Ferma», «Apri la chat» e «Copia la chiave». Sotto, le sezioni:",
            "It opens from the activity bar (the chip icon). On the left, “In the kit”: the llama.cpp server with its status and port, and the downloaded models (“Pi's model”, “in memory”, “download paused”). At the top, “Start” or “Stop”, “Open the chat” and “Copy the key”. Below, the sections:",
            "Elle s'ouvre depuis la barre d'activité (l'icône de la puce). À gauche, « Dans le kit » : le serveur llama.cpp avec son état et son port, et les modèles téléchargés (« modèle de Pi », « en mémoire », « téléchargement suspendu »). En haut, « Démarrer » ou « Arrêter », « Ouvrir le chat » et « Copier la clé ». En dessous, les sections :",
            "Sie öffnet sich über die Aktivitätsleiste (das Chip-Symbol). Links „Im Kit“: der llama.cpp-Server mit Status und Port und die heruntergeladenen Modelle („Modell von Pi“, „im Speicher“, „Download angehalten“). Oben „Starten“ oder „Beenden“, „Chat öffnen“ und „Schlüssel kopieren“. Darunter die Abschnitte:",
          )}
        </P>
        <Table
          columns={[
            tr("Sezione", "Section", "Section", "Abschnitt"),
            tr("Che cosa contiene", "What it contains", "Ce qu'elle contient", "Was er enthält"),
          ]}
          rows={[
            [
              tr("Server", "Server", "Serveur", "Server"),
              tr(
                "L'indirizzo (http://127.0.0.1:<porta>), i modelli in memoria e «Avvia con Atollo», per accendere il server all'apertura dell'app. «Avvia» resta spento finché non c'è almeno un modello scaricato.",
                "The address (http://127.0.0.1:<port>), the models in memory and “Start with Atollo”, to turn the server on when the app opens. “Start” stays disabled until at least one model is downloaded.",
                "L'adresse (http://127.0.0.1:<port>), les modèles en mémoire et « Démarrer avec Atollo », pour allumer le serveur à l'ouverture de l'application. « Démarrer » reste désactivé tant qu'aucun modèle n'est téléchargé.",
                "Die Adresse (http://127.0.0.1:<Port>), die Modelle im Speicher und „Mit Atollo starten“, um den Server beim Öffnen der App einzuschalten. „Starten“ bleibt deaktiviert, solange kein Modell heruntergeladen ist.",
              ),
            ],
            [
              tr("Modello di Pi", "Pi's model", "Modèle de Pi", "Modell von Pi"),
              tr(
                "«Pi parte con»: un modello scaricato, oppure «il primo modello del kit». Con un provider in cloud scelto in Pi (/login), Pi resta su quello; i modelli locali restano nel suo elenco (/model).",
                "“Pi starts with”: a downloaded model, or “the kit's first model”. With a cloud provider chosen in Pi (/login), Pi stays on that one; the local models remain in its list (/model).",
                "« Pi démarre avec » : un modèle téléchargé, ou « le premier modèle du kit ». Avec un fournisseur dans le cloud choisi dans Pi (/login), Pi reste sur celui-ci ; les modèles locaux restent dans sa liste (/model).",
                "„Pi startet mit“: ein heruntergeladenes Modell oder „dem ersten Modell des Kits“. Mit einem in Pi gewählten Cloud-Anbieter (/login) bleibt Pi bei diesem; die lokalen Modelle bleiben in seiner Liste (/model).",
              ),
            ],
            [
              tr("Catalogo di Atollo", "Atollo's catalog", "Catalogue d'Atollo", "Katalog von Atollo"),
              tr(
                "I modelli scelti per il kit, ognuno con quantizzazione, dimensione, licenza, la memoria che chiede, se entra in questo PC e se usa gli strumenti. «Scarica», poi «Sospendi» o «Riprendi»; il cestino lo cancella.",
                "The models chosen for the kit, each with quantization, size, license, the memory it needs, whether it fits this PC and whether it uses tools. “Download”, then “Pause” or “Resume”; the bin deletes it.",
                "Les modèles choisis pour le kit, chacun avec quantification, taille, licence, la mémoire qu'il demande, s'il tient sur ce PC et s'il utilise les outils. « Télécharger », puis « Suspendre » ou « Reprendre » ; la corbeille le supprime.",
                "Die für das Kit ausgewählten Modelle, jedes mit Quantisierung, Größe, Lizenz, dem benötigten Speicher, ob es auf diesen PC passt und ob es Werkzeuge nutzt. „Herunterladen“, dann „Anhalten“ oder „Fortsetzen“; der Papierkorb löscht es.",
              ),
            ],
            [
              tr(
                "Altri modelli nel kit",
                "Other models in the kit",
                "Autres modèles dans le kit",
                "Weitere Modelle im Kit",
              ),
              tr(
                "I modelli presi da Hugging Face e i file .gguf messi a mano nella cartella dei modelli.",
                "The models taken from Hugging Face and the .gguf files put by hand in the models folder.",
                "Les modèles pris sur Hugging Face et les fichiers .gguf placés à la main dans le dossier des modèles.",
                "Die von Hugging Face geholten Modelle und die von Hand in den Ordner der Modelle gelegten .gguf-Dateien.",
              ),
            ],
            [
              tr("Da Hugging Face", "From Hugging Face", "Depuis Hugging Face", "Von Hugging Face"),
              tr(
                "Un repository qualsiasi (proprietario/nome), la scelta della quantizzazione e il token per i modelli ad accesso controllato.",
                "Any repository (owner/name), the choice of quantization and the token for gated models.",
                "N'importe quel dépôt (propriétaire/nom), le choix de la quantification et le jeton pour les modèles à accès contrôlé.",
                "Ein beliebiges Repository (Eigentümer/Name), die Wahl der Quantisierung und das Token für zugangsbeschränkte Modelle.",
              ),
            ],
            [
              tr("Questo PC", "This PC", "Ce PC", "Dieser PC"),
              tr(
                "La RAM, le schede video con la loro memoria (si vedono dopo l'installazione di llama.cpp), il file system del disco del kit con lo spazio libero, e la cartella dei modelli.",
                "The RAM, the graphics cards with their memory (visible after llama.cpp is installed), the file system of the kit's drive with the free space, and the models folder.",
                "La RAM, les cartes graphiques avec leur mémoire (visibles après l'installation de llama.cpp), le système de fichiers du disque du kit avec l'espace libre, et le dossier des modèles.",
                "Der RAM, die Grafikkarten mit ihrem Speicher (sichtbar nach der Installation von llama.cpp), das Dateisystem des Kit-Laufwerks mit dem freien Platz und der Ordner der Modelle.",
              ),
            ],
          ]}
        />
        <Go page="models">
          {tr(
            "Apri la pagina Modelli locali",
            "Open the Local models page",
            "Ouvrir la page Modèles locaux",
            "Seite Lokale Modelle öffnen",
          )}
        </Go>
      </Section>

      <Section
        title={tr(
          "Scegliere e scaricare un modello",
          "Choosing and downloading a model",
          "Choisir et télécharger un modèle",
          "Ein Modell auswählen und herunterladen",
        )}
      >
        <P>
          {tr(
            "Accanto a ogni modello la pagina dice se entra nel PC, confrontando la memoria che chiede con la più grande fra la RAM e la memoria della scheda video:",
            "Next to each model the page says whether it fits the PC, comparing the memory it needs with the larger of the RAM and the graphics card's memory:",
            "À côté de chaque modèle, la page dit s'il tient sur le PC, en comparant la mémoire qu'il demande avec la plus grande entre la RAM et la mémoire de la carte graphique :",
            "Neben jedem Modell sagt die Seite, ob es auf den PC passt, indem sie den benötigten Speicher mit dem größeren von RAM und Grafikspeicher vergleicht:",
          )}
        </P>
        <Table
          columns={[tr("Testo", "Text", "Texte", "Text"), tr("Significato", "Meaning", "Signification", "Bedeutung")]}
          rows={[
            [
              tr("entra", "fits", "tient", "passt"),
              tr(
                "Chiede al massimo l'80% di quella memoria: c'è margine.",
                "It needs at most 80% of that memory: there is room to spare.",
                "Il demande au plus 80 % de cette mémoire : il reste de la marge.",
                "Es braucht höchstens 80 % dieses Speichers: Es bleibt Spielraum.",
              ),
            ],
            [
              tr("al limite", "just fits", "à la limite", "knapp"),
              tr(
                "Ci sta, ma lascia poco al resto del sistema: può essere lento.",
                "It fits, but leaves little for the rest of the system: it may be slow.",
                "Il tient, mais laisse peu au reste du système : il peut être lent.",
                "Es passt, lässt aber dem restlichen System wenig übrig: Es kann langsam sein.",
              ),
            ],
            [
              tr(
                "troppo grande per questo PC",
                "too large for this PC",
                "trop grand pour ce PC",
                "zu groß für diesen PC",
              ),
              tr(
                "Chiede più memoria di quella che il PC ha.",
                "It needs more memory than the PC has.",
                "Il demande plus de mémoire que le PC n'en a.",
                "Es braucht mehr Speicher, als der PC hat.",
              ),
            ],
          ]}
        />
        <P>
          {tr(
            "«usa gli strumenti (adatto agli agenti)» vuol dire che il modello sa chiamare i comandi e leggere i file quando lavora con Pi; «solo chat» vuol dire che con gli agenti non li usa bene (scrive la chiamata come testo), ma va bene per chiacchierare nel browser.",
            "“uses tools (suited to the agents)” means the model can call commands and read files when it works with Pi; “chat only” means it does not use them well with the agents (it writes the call as text), but it is fine for chatting in the browser.",
            "« utilise les outils (adapté aux agents) » signifie que le modèle sait appeler les commandes et lire les fichiers quand il travaille avec Pi ; « chat seulement » signifie qu'il les utilise mal avec les agents (il écrit l'appel comme du texte), mais il convient pour discuter dans le navigateur.",
            "„nutzt Werkzeuge (für Agenten geeignet)“ bedeutet, dass das Modell Befehle aufrufen und Dateien lesen kann, wenn es mit Pi arbeitet; „nur Chat“ bedeutet, dass es sie mit den Agenten nicht gut nutzt (es schreibt den Aufruf als Text), aber zum Chatten im Browser taugt.",
          )}
        </P>
        <Table
          columns={[
            tr("Modello del catalogo", "Catalog model", "Modèle du catalogue", "Modell des Katalogs"),
            tr("Memoria", "Memory", "Mémoire", "Speicher"),
            tr("Uso", "Use", "Usage", "Verwendung"),
          ]}
          rows={[
            [
              tr("Qwen2.5-Coder 7B", "Qwen2.5-Coder 7B", "Qwen2.5-Coder 7B", "Qwen2.5-Coder 7B"),
              tr("circa 7 GB", "about 7 GB", "environ 7 Go", "etwa 7 GB"),
              tr(
                "solo chat; in due parti, entra anche su FAT32",
                "chat only; in two parts, it fits on FAT32 too",
                "chat seulement ; en deux parties, il tient aussi sur FAT32",
                "nur Chat; in zwei Teilen, passt auch auf FAT32",
              ),
            ],
            [
              tr("Qwen3.5 9B", "Qwen3.5 9B", "Qwen3.5 9B", "Qwen3.5 9B"),
              tr("circa 8 GB", "about 8 GB", "environ 8 Go", "etwa 8 GB"),
              tr("usa gli strumenti", "uses tools", "utilise les outils", "nutzt Werkzeuge"),
            ],
            [
              tr("gpt-oss 20B", "gpt-oss 20B", "gpt-oss 20B", "gpt-oss 20B"),
              tr("circa 15 GB", "about 15 GB", "environ 15 Go", "etwa 15 GB"),
              tr("usa gli strumenti", "uses tools", "utilise les outils", "nutzt Werkzeuge"),
            ],
            [
              tr("Devstral Small 2 24B", "Devstral Small 2 24B", "Devstral Small 2 24B", "Devstral Small 2 24B"),
              tr("circa 18 GB", "about 18 GB", "environ 18 Go", "etwa 18 GB"),
              tr("usa gli strumenti", "uses tools", "utilise les outils", "nutzt Werkzeuge"),
            ],
            [
              tr("Qwen3-Coder 30B-A3B", "Qwen3-Coder 30B-A3B", "Qwen3-Coder 30B-A3B", "Qwen3-Coder 30B-A3B"),
              tr("circa 22 GB", "about 22 GB", "environ 22 Go", "etwa 22 GB"),
              tr("usa gli strumenti", "uses tools", "utilise les outils", "nutzt Werkzeuge"),
            ],
          ]}
        />
        <Steps
          steps={[
            tr(
              "Nel «Catalogo di Atollo» scegli un modello che «entra» e «usa gli strumenti», se lo vuoi per Pi.",
              "In “Atollo's catalog” choose a model that “fits” and “uses tools”, if you want it for Pi.",
              "Dans le « Catalogue d'Atollo », choisissez un modèle qui « tient » et « utilise les outils », si vous le voulez pour Pi.",
              "Wähle im „Katalog von Atollo“ ein Modell, das „passt“ und „Werkzeuge nutzt“, wenn du es für Pi willst.",
            ),
            tr(
              "Premi «Scarica». La barra mostra percentuale e GB; il download continua anche se cambi pagina.",
              "Press “Download”. The bar shows the percentage and the GB; the download goes on even if you change page.",
              "Appuyez sur « Télécharger ». La barre montre le pourcentage et les Go ; le téléchargement continue même si vous changez de page.",
              "Drück „Herunterladen“. Der Balken zeigt Prozent und GB; der Download läuft weiter, auch wenn du die Seite wechselst.",
            ),
            tr(
              "«Sospendi» lo ferma e tiene la parte già scaricata; «Riprendi» riparte da lì, anche dopo aver chiuso Atollo o dopo un errore di rete.",
              "“Pause” stops it and keeps the part already downloaded; “Resume” starts again from there, even after closing Atollo or after a network error.",
              "« Suspendre » l'arrête et garde la partie déjà téléchargée ; « Reprendre » repart de là, même après la fermeture d'Atollo ou après une erreur réseau.",
              "„Anhalten“ stoppt ihn und behält den bereits heruntergeladenen Teil; „Fortsetzen“ setzt dort wieder an, auch nachdem du Atollo geschlossen hast oder nach einem Netzwerkfehler.",
            ),
            tr(
              "Alla fine il modello compare in «Nel kit» con «nel kit»; se il server è acceso, si riavvia per vederlo.",
              "At the end the model appears in “In the kit” marked “in the kit”; if the server is on, it restarts to see it.",
              "À la fin, le modèle apparaît dans « Dans le kit » marqué « dans le kit » ; si le serveur est allumé, il redémarre pour le voir.",
              "Am Ende erscheint das Modell unter „Im Kit“ mit „im Kit“; läuft der Server, startet er neu, um es zu sehen.",
            ),
          ]}
        />
        <Callout tone="info" title={tr("Controlli", "Checks", "Contrôles", "Prüfungen")}>
          {tr(
            "Ogni file del catalogo ha il suo SHA-256: il kit lo calcola mentre i byte arrivano e cancella un file che non corrisponde. Prima di scrivere un byte, rifiuta un modello più grande dello spazio libero e, su un disco FAT32, un file oltre i 4 GB, dicendo che cosa fare.",
            "Every file in the catalog has its SHA-256: the kit computes it while the bytes arrive and deletes a file that does not match. Before writing a byte, it refuses a model larger than the free space and, on a FAT32 drive, a file over 4 GB, telling you what to do.",
            "Chaque fichier du catalogue a son SHA-256 : le kit le calcule pendant que les octets arrivent et supprime un fichier qui ne correspond pas. Avant d'écrire un octet, il refuse un modèle plus grand que l'espace libre et, sur un disque FAT32, un fichier de plus de 4 Go, en vous disant quoi faire.",
            "Jede Datei des Katalogs hat ihre SHA-256: Das Kit berechnet sie, während die Bytes ankommen, und löscht eine Datei, die nicht übereinstimmt. Bevor es ein Byte schreibt, lehnt es ein Modell ab, das größer als der freie Platz ist, und auf einem FAT32-Laufwerk eine Datei über 4 GB, und sagt dir, was zu tun ist.",
          )}
        </Callout>
        <Callout tone="warn" title={tr("Dischi FAT32", "FAT32 drives", "Disques FAT32", "FAT32-Laufwerke")}>
          {tr(
            "FAT32 non tiene file oltre i 4 GB: la pagina lo avvisa e scrive «non entra su FAT32» accanto ai modelli troppo grandi. Qwen2.5-Coder 7B, in due parti più piccole, entra comunque. Per tutti gli altri formatta il disco in exFAT o NTFS.",
            "FAT32 cannot hold files over 4 GB: the page warns about it and writes “does not fit on FAT32” next to the models that are too large. Qwen2.5-Coder 7B, in two smaller parts, fits anyway. For all the others, format the drive as exFAT or NTFS.",
            "FAT32 ne peut pas contenir de fichiers de plus de 4 Go : la page le signale et écrit « ne tient pas sur FAT32 » à côté des modèles trop grands. Qwen2.5-Coder 7B, en deux parties plus petites, tient quand même. Pour tous les autres, formatez le disque en exFAT ou NTFS.",
            "FAT32 kann keine Dateien über 4 GB aufnehmen: Die Seite weist darauf hin und schreibt „passt nicht auf FAT32“ neben die zu großen Modelle. Qwen2.5-Coder 7B passt in zwei kleineren Teilen trotzdem. Für alle anderen formatiere das Laufwerk mit exFAT oder NTFS.",
          )}
        </Callout>
      </Section>

      <Section title={tr("Da Hugging Face", "From Hugging Face", "Depuis Hugging Face", "Von Hugging Face")}>
        <Steps
          steps={[
            tr(
              "In «Repository di Hugging Face» scrivi proprietario/nome del repository, per esempio unsloth/Qwen3-8B-GGUF.",
              "In “Hugging Face repository” write the repository's owner/name, for example unsloth/Qwen3-8B-GGUF.",
              "Dans « Dépôt Hugging Face », écrivez propriétaire/nom du dépôt, par exemple unsloth/Qwen3-8B-GGUF.",
              "Gib unter „Hugging-Face-Repository“ Eigentümer/Name des Repositorys ein, zum Beispiel unsloth/Qwen3-8B-GGUF.",
            ),
            tr(
              "Premi «Cerca i file GGUF»: il kit legge l'ultimo commit e raggruppa i file per quantizzazione, con dimensione, numero di parti, se entra nel PC e la licenza.",
              "Press “Find the GGUF files”: the kit reads the latest commit and groups the files by quantization, with size, number of parts, whether it fits the PC and the license.",
              "Appuyez sur « Chercher les fichiers GGUF » : le kit lit le dernier commit et regroupe les fichiers par quantification, avec la taille, le nombre de parties, s'il tient sur le PC et la licence.",
              "Drück „GGUF-Dateien suchen“: Das Kit liest den neuesten Commit und gruppiert die Dateien nach Quantisierung, mit Größe, Anzahl der Teile, ob es auf den PC passt, und der Lizenz.",
            ),
            tr(
              "Premi «Scarica» sulla quantizzazione che vuoi: il kit scarica tutte le sue parti da quel commit, o nessuna. Q4_K_M è di solito un buon compromesso.",
              "Press “Download” on the quantization you want: the kit downloads all its parts from that commit, or none. Q4_K_M is usually a good compromise.",
              "Appuyez sur « Télécharger » sur la quantification voulue : le kit télécharge toutes ses parties depuis ce commit, ou aucune. Q4_K_M est en général un bon compromis.",
              "Drück „Herunterladen“ bei der gewünschten Quantisierung: Das Kit lädt alle ihre Teile aus diesem Commit herunter oder keinen. Q4_K_M ist meist ein guter Kompromiss.",
            ),
          ]}
        />
        <P>
          {tr(
            "Alcuni modelli sono ad accesso controllato: si accettano le condizioni sulla pagina del modello e serve un token di Hugging Face. Scrivilo in «Token di Hugging Face» e premi «Salva il token»; «Togli il token» lo cancella. Il token è salvato nel kit, leggibile solo dal tuo utente, e va solo a huggingface.co: non segue il reindirizzamento verso la CDN da cui arrivano i file.",
            "Some models are gated: you accept the conditions on the model's page and you need a Hugging Face token. Write it in “Hugging Face token” and press “Save the token”; “Remove the token” deletes it. The token is saved in the kit, readable only by your user, and goes only to huggingface.co: it does not follow the redirect to the CDN the files come from.",
            "Certains modèles sont à accès contrôlé : on accepte les conditions sur la page du modèle et il faut un jeton Hugging Face. Écrivez-le dans « Jeton Hugging Face » et appuyez sur « Enregistrer le jeton » ; « Retirer le jeton » le supprime. Le jeton est enregistré dans le kit, lisible seulement par votre utilisateur, et ne va qu'à huggingface.co : il ne suit pas la redirection vers le CDN d'où viennent les fichiers.",
            "Manche Modelle sind zugangsbeschränkt: Du akzeptierst die Bedingungen auf der Seite des Modells und brauchst ein Hugging-Face-Token. Gib es unter „Hugging-Face-Token“ ein und drück „Token speichern“; „Token entfernen“ löscht es. Das Token wird im Kit gespeichert, nur für deinen Benutzer lesbar, und geht nur an huggingface.co: Es folgt nicht der Weiterleitung an das CDN, von dem die Dateien kommen.",
          )}
        </P>
        <Callout tone="tip">
          {tr(
            "Un file .gguf copiato a mano in una cartella di data\\models\\gguf è un modello anche lui: compare in «Altri modelli nel kit» con il nome del file.",
            "A .gguf file copied by hand into a folder of data\\models\\gguf is a model too: it appears in “Other models in the kit” with the file's name.",
            "Un fichier .gguf copié à la main dans un dossier de data\\models\\gguf est aussi un modèle : il apparaît dans « Autres modèles dans le kit » avec le nom du fichier.",
            "Eine von Hand in einen Ordner von data\\models\\gguf kopierte .gguf-Datei ist ebenfalls ein Modell: Sie erscheint unter „Weitere Modelle im Kit“ mit dem Namen der Datei.",
          )}
        </Callout>
      </Section>

      <Section
        title={tr(
          "La chat nel browser",
          "The chat in the browser",
          "Le chat dans le navigateur",
          "Der Chat im Browser",
        )}
      >
        <Steps
          steps={[
            tr(
              "Premi «Avvia» (o lascia fare a «Avvia con Atollo»). La prima volta il kit installa llama.cpp.",
              "Press “Start” (or let “Start with Atollo” do it). The first time, the kit installs llama.cpp.",
              "Appuyez sur « Démarrer » (ou laissez faire « Démarrer avec Atollo »). La première fois, le kit installe llama.cpp.",
              "Drück „Starten“ (oder überlass es „Mit Atollo starten“). Beim ersten Mal installiert das Kit llama.cpp.",
            ),
            tr(
              "Premi «Copia la chiave»: la chiave del server va negli appunti.",
              "Press “Copy the key”: the server's key goes to the clipboard.",
              "Appuyez sur « Copier la clé » : la clé du serveur va dans le presse-papiers.",
              "Drück „Schlüssel kopieren“: Der Schlüssel des Servers kommt in die Zwischenablage.",
            ),
            tr(
              "Premi «Apri la chat»: si apre la chat di llama.cpp nel browser. Nelle sue impostazioni incolla la chiave, poi scegli il modello e scrivi.",
              "Press “Open the chat”: llama.cpp's chat opens in the browser. Paste the key in its settings, then choose the model and write.",
              "Appuyez sur « Ouvrir le chat » : le chat de llama.cpp s'ouvre dans le navigateur. Collez la clé dans ses paramètres, puis choisissez le modèle et écrivez.",
              "Drück „Chat öffnen“: Der Chat von llama.cpp öffnet sich im Browser. Füge den Schlüssel in seinen Einstellungen ein, wähle dann das Modell und schreib.",
            ),
          ]}
        />
        <P>
          {tr(
            "Pi non ha bisogno di questi passi: il kit gli scrive porta e chiave nella sua configurazione e accende il server prima di aprirlo, quando usa un modello locale.",
            "Pi does not need these steps: the kit writes the port and the key into its configuration and turns the server on before opening it, when it uses a local model.",
            "Pi n'a pas besoin de ces étapes : le kit écrit le port et la clé dans sa configuration et allume le serveur avant de l'ouvrir, quand il utilise un modèle local.",
            "Pi braucht diese Schritte nicht: Das Kit schreibt Port und Schlüssel in seine Konfiguration und schaltet den Server ein, bevor es ihn öffnet, wenn er ein lokales Modell verwendet.",
          )}
        </P>
      </Section>

      <Section
        title={tr(
          "Sicurezza e risorse",
          "Security and resources",
          "Sécurité et ressources",
          "Sicherheit und Ressourcen",
        )}
      >
        <Bullets
          items={[
            tr(
              "Il server ascolta solo su 127.0.0.1: nessun altro PC della rete lo raggiunge, anche con la rete locale attiva nelle Impostazioni.",
              "The server listens only on 127.0.0.1: no other PC on the network reaches it, even with the local network enabled in Settings.",
              "Le serveur n'écoute que sur 127.0.0.1 : aucun autre PC du réseau ne l'atteint, même avec le réseau local activé dans les Paramètres.",
              "Der Server lauscht nur auf 127.0.0.1: Kein anderer PC im Netzwerk erreicht ihn, auch nicht mit aktiviertem lokalem Netzwerk in den Einstellungen.",
            ),
            tr(
              "Ogni richiesta vuole la chiave del server, creata al primo avvio e data al server come file, mai sulla riga di comando (dove altri programmi del PC la vedrebbero). Senza chiave il server risponde 401.",
              "Every request needs the server's key, created at the first start and given to the server as a file, never on the command line (where other programs on the PC would see it). Without the key the server answers 401.",
              "Chaque requête demande la clé du serveur, créée au premier démarrage et donnée au serveur sous forme de fichier, jamais sur la ligne de commande (où d'autres programmes du PC la verraient). Sans clé, le serveur répond 401.",
              "Jede Anfrage braucht den Schlüssel des Servers, der beim ersten Start erzeugt und dem Server als Datei übergeben wird, nie in der Befehlszeile (wo andere Programme des PCs ihn sehen würden). Ohne Schlüssel antwortet der Server mit 401.",
            ),
            tr(
              "Un modello si carica quando Pi o la chat lo chiede, uno alla volta, e si libera dopo dieci minuti senza richieste: la memoria torna al PC.",
              "A model loads when Pi or the chat asks for it, one at a time, and is unloaded after ten minutes without requests: the memory goes back to the PC.",
              "Un modèle se charge quand Pi ou le chat le demande, un à la fois, et se libère après dix minutes sans requêtes : la mémoire revient au PC.",
              "Ein Modell wird geladen, wenn Pi oder der Chat es anfordert, eines nach dem anderen, und nach zehn Minuten ohne Anfragen freigegeben: Der Speicher geht an den PC zurück.",
            ),
            tr(
              "Il server compare nel pannello «Attività» (con «Ferma» e i modelli in memoria) e si ferma con «Ferma tutto», con «Rimuovi l'unità» e alla chiusura dell'app.",
              "The server appears in the “Activity” panel (with “Stop” and the models in memory) and stops with “Stop everything”, with “Remove drive” and when the app closes.",
              "Le serveur apparaît dans le panneau « Activité » (avec « Arrêter » et les modèles en mémoire) et s'arrête avec « Tout arrêter », avec « Retirer le lecteur » et à la fermeture de l'application.",
              "Der Server erscheint im Bereich „Aktivität“ (mit „Beenden“ und den Modellen im Speicher) und stoppt mit „Alles stoppen“, mit „Laufwerk entfernen“ und beim Schließen der App.",
            ),
            tr(
              "Mentre cancelli un modello il server si ferma: su Windows un file aperto non si può cancellare.",
              "While you delete a model the server stops: on Windows an open file cannot be deleted.",
              "Pendant que vous supprimez un modèle, le serveur s'arrête : sur Windows, un fichier ouvert ne peut pas être supprimé.",
              "Während du ein Modell löschst, stoppt der Server: Unter Windows lässt sich eine geöffnete Datei nicht löschen.",
            ),
          ]}
        />
      </Section>

      <Under>
        <Table
          columns={[
            tr("File o cartella", "File or folder", "Fichier ou dossier", "Datei oder Ordner"),
            tr("Che cosa contiene", "What it contains", "Ce qu'il contient", "Was sie enthält"),
          ]}
          rows={[
            [
              <Code key="a">
                {"data\\models\\gguf\\" +
                  tr("<proprietario>--<nome>", "<owner>--<name>", "<propriétaire>--<nom>", "<Eigentümer>--<Name>") +
                  "\\"}
              </Code>,
              tr(
                "I modelli, una cartella per repository, una sola copia per Windows, macOS e Linux. atollo.json dice che cosa ha scaricato il kit (file, commit, se è completo); un download a metà lascia un file .part.",
                "The models, one folder per repository, a single copy for Windows, macOS and Linux. atollo.json says what the kit downloaded (files, commit, whether it is complete); a half-done download leaves a .part file.",
                "Les modèles, un dossier par dépôt, une seule copie pour Windows, macOS et Linux. atollo.json dit ce que le kit a téléchargé (fichiers, commit, s'il est complet) ; un téléchargement à moitié laisse un fichier .part.",
                "Die Modelle, ein Ordner pro Repository, eine einzige Kopie für Windows, macOS und Linux. atollo.json sagt, was das Kit heruntergeladen hat (Dateien, Commit, ob vollständig); ein halber Download hinterlässt eine .part-Datei.",
              ),
            ],
            [
              <Code key="b">data\app\secrets\llama-key</Code>,
              tr(
                "La chiave del server, solo per il proprietario.",
                "The server's key, owner-only.",
                "La clé du serveur, réservée au propriétaire.",
                "Der Schlüssel des Servers, nur für den Eigentümer.",
              ),
            ],
            [
              <Code key="c">data\app\secrets\hf-token</Code>,
              tr(
                "Il token di Hugging Face, solo per il proprietario.",
                "The Hugging Face token, owner-only.",
                "Le jeton Hugging Face, réservé au propriétaire.",
                "Das Hugging-Face-Token, nur für den Eigentümer.",
              ),
            ],
            [
              <Code key="d">data\app\llm.json</Code>,
              tr(
                "La porta del server (9931 o la prima libera dopo), «Avvia con Atollo» e il modello di Pi.",
                "The server's port (9931 or the first free one after it), “Start with Atollo” and Pi's model.",
                "Le port du serveur (9931 ou le premier libre après), « Démarrer avec Atollo » et le modèle de Pi.",
                "Der Port des Servers (9931 oder der erste freie danach), „Mit Atollo starten“ und das Modell von Pi.",
              ),
            ],
            [
              <Code key="e">data\app\llm\presets.ini</Code>,
              tr(
                "I modelli che il server offre, con i percorsi assoluti: si riscrive a ogni avvio, perché il kit può essersi spostato.",
                "The models the server offers, with absolute paths: it is rewritten at every start, because the kit may have moved.",
                "Les modèles que le serveur propose, avec les chemins absolus : il est réécrit à chaque démarrage, car le kit a pu être déplacé.",
                "Die Modelle, die der Server anbietet, mit absoluten Pfaden: Die Datei wird bei jedem Start neu geschrieben, weil das Kit verschoben worden sein kann.",
              ),
            ],
            [
              <Code key="f">data\app\logs\llm.log</Code>,
              tr(
                "L'uscita del server, da leggere se non parte.",
                "The server's output, to read if it does not start.",
                "La sortie du serveur, à lire s'il ne démarre pas.",
                "Die Ausgabe des Servers, zum Nachlesen, wenn er nicht startet.",
              ),
            ],
          ]}
        />
        <Bullets
          items={[
            tr(
              "llama.cpp è fissato a una build (b11669) con lo SHA-256 dell'archivio di ogni sistema: la build Vulkan su Windows e Linux (che contiene anche i motori per la CPU), quella Metal su macOS.",
              "llama.cpp is pinned to one build (b11669) with the SHA-256 of each system's archive: the Vulkan build on Windows and Linux (which also contains the CPU engines), the Metal one on macOS.",
              "llama.cpp est figé sur une build (b11669) avec le SHA-256 de l'archive de chaque système : la build Vulkan sur Windows et Linux (qui contient aussi les moteurs pour le processeur), celle Metal sur macOS.",
              "llama.cpp ist auf einen Build (b11669) festgelegt, mit der SHA-256 des Archivs jedes Systems: der Vulkan-Build unter Windows und Linux (der auch die CPU-Engines enthält), der Metal-Build unter macOS.",
            ),
            tr(
              "I modelli del catalogo sono fissati a un commit di Hugging Face, ogni file con SHA-256 e dimensione; quelli presi da Hugging Face si scaricano dall'ultimo commit al momento della ricerca.",
              "The catalog's models are pinned to a Hugging Face commit, each file with SHA-256 and size; those taken from Hugging Face are downloaded from the latest commit at the time of the search.",
              "Les modèles du catalogue sont figés sur un commit de Hugging Face, chaque fichier avec SHA-256 et taille ; ceux pris sur Hugging Face se téléchargent depuis le dernier commit au moment de la recherche.",
              "Die Modelle des Katalogs sind auf einen Hugging-Face-Commit festgelegt, jede Datei mit SHA-256 und Größe; die von Hugging Face geholten werden vom neuesten Commit zum Zeitpunkt der Suche heruntergeladen.",
            ),
            rich(
              tr(
                "Dalla riga di comando: {0} elenca i modelli, {1} ne scarica uno (un id del catalogo o proprietario/nome:quantizzazione, riprende se interrotto), {2} lo cancella, {3} comanda il server.",
                "From the command line: {0} lists the models, {1} downloads one (a catalog id or owner/name:quantization, resumes if interrupted), {2} deletes it, {3} controls the server.",
                "Depuis la ligne de commande : {0} liste les modèles, {1} en télécharge un (un id du catalogue ou propriétaire/nom:quantification, reprend si interrompu), {2} le supprime, {3} pilote le serveur.",
                "Über die Kommandozeile: {0} listet die Modelle auf, {1} lädt eines herunter (eine Katalog-ID oder Eigentümer/Name:Quantisierung, setzt nach einer Unterbrechung fort), {2} löscht es, {3} steuert den Server.",
              ),
              <Code key="m">models</Code>,
              <Code key="p">{`models pull ${tr("<modello>", "<model>", "<modèle>", "<Modell>")}`}</Code>,
              <Code key="r">models rm &lt;id&gt;</Code>,
              <Code key="l">llm start|stop|status</Code>,
            ),
          ]}
        />
      </Under>

      <Related ids={["agents", "tools", "cli", "eject"]} />
    </>
  ),
};
