// Chapter 4 of the Guide, «Agenti IA»: the three agents, what they share,
// their limits and usage.
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
            "Il kit ne offre tre: Claude Code (Anthropic), Codex (OpenAI) e Antigravity (Google). Si installano da Strumenti.",
            "The kit offers three: Claude Code (Anthropic), Codex (OpenAI) and Antigravity (Google). You install them from Tools.",
            "Le kit en propose trois : Claude Code (Anthropic), Codex (OpenAI) et Antigravity (Google). Ils s'installent depuis Outils.",
            "Das Kit bietet drei: Claude Code (Anthropic), Codex (OpenAI) und Antigravity (Google). Sie werden über Werkzeuge installiert.",
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
            "Un agente è un programma a riga di comando che dialoga con un modello di intelligenza artificiale del fornitore. Gli chiedi in italiano che cosa fare («aggiungi una pagina di contatto», «perché questo test fallisce?») e lui legge i file, propone modifiche, esegue comandi e ti mostra i risultati. Per usarlo serve un account del fornitore con un piano che lo comprenda, oppure una chiave API.",
            "An agent is a command-line program that talks with an artificial intelligence model from its provider. You tell it in plain English what to do (“add a contact page”, “why does this test fail?”) and it reads the files, proposes changes, runs commands and shows you the results. To use it you need an account with the provider and a plan that includes it, or an API key.",
            "Un agent est un programme en ligne de commande qui dialogue avec un modèle d'intelligence artificielle du fournisseur. Vous lui demandez en français ce qu'il doit faire (« ajoute une page de contact », « pourquoi ce test échoue-t-il ? ») et il lit les fichiers, propose des modifications, exécute des commandes et vous montre les résultats. Pour l'utiliser, il faut un compte chez le fournisseur avec un abonnement qui l'inclut, ou une clé API.",
            "Ein Agent ist ein Kommandozeilenprogramm, das mit einem KI-Modell des Anbieters kommuniziert. Du sagst ihm auf Deutsch, was es tun soll („füge eine Kontaktseite hinzu“, „warum schlägt dieser Test fehl?“), und es liest die Dateien, schlägt Änderungen vor, führt Befehle aus und zeigt dir die Ergebnisse. Dafür brauchst du ein Konto beim Anbieter mit einem Tarif, der ihn enthält, oder einen API-Schlüssel.",
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
          ]}
        />
        <P>
          {tr(
            "Gli agenti si installano dalla pagina Strumenti, tipo «Agenti AI» (il primo avvio li propone già). Seguono sempre l'ultima versione e portano con sé ciò che serve; Git, che usano tutti, è già tra i componenti sempre installati.",
            "Agents are installed from the Tools page, “AI agents” kind (the first start already suggests them). They always follow the latest version and bring along what they need; Git, which they all use, is already among the components that are always installed.",
            "Les agents s'installent depuis la page Outils, type « Agents IA » (le premier démarrage les propose déjà). Ils suivent toujours la dernière version et apportent ce dont ils ont besoin ; Git, qu'ils utilisent tous, fait déjà partie des composants toujours installés.",
            "Die Agenten werden auf der Seite Werkzeuge in der Art „KI-Agenten“ installiert (der erste Start schlägt sie bereits vor). Sie folgen immer der neuesten Version und bringen mit, was sie brauchen; Git, das sie alle verwenden, gehört schon zu den immer installierten Komponenten.",
          )}
        </P>
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
              "In alto, accanto a «Editor» e «Terminale», c'è un pulsante per ogni agente installato, con il suo logo («Apri Claude Code», «Apri Codex», «Apri Antigravity»). Fai clic su quello dell'agente che vuoi usare.",
              "At the top, next to “Editor” and “Terminal”, there is a button for each installed agent, with its logo (“Open Claude Code”, “Open Codex”, “Open Antigravity”). Click the one of the agent you want to use.",
              "En haut, à côté de « Éditeur » et « Terminal », il y a un bouton pour chaque agent installé, avec son logo (« Ouvrir Claude Code », « Ouvrir Codex », « Ouvrir Antigravity »). Cliquez sur celui de l'agent que vous voulez utiliser.",
              "Oben neben „Editor“ und „Terminal“ gibt es für jeden installierten Agenten eine Schaltfläche mit seinem Logo („Claude Code öffnen“, „Codex öffnen“, „Antigravity öffnen“). Klicke auf die des Agenten, den du verwenden möchtest.",
            ),
            tr(
              "L'agente parte subito. Su Windows si apre una console del kit nella cartella del progetto con l'agente avviato; su macOS e Linux una scheda del pannello dei terminali.",
              "The agent starts right away. On Windows a kit console opens in the project folder with the agent started; on macOS and Linux, a tab in the terminal panel.",
              "L'agent démarre aussitôt. Sur Windows, une console du kit s'ouvre dans le dossier du projet avec l'agent lancé ; sur macOS et Linux, un onglet du panneau des terminaux.",
              "Der Agent startet sofort. Unter Windows öffnet sich eine Konsole des Kits im Projektordner mit dem gestarteten Agenten; unter macOS und Linux eine Registerkarte im Terminalbereich.",
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
                "Codex con la chiave salvata da codex login --with-api-key, o Antigravity con la chiave di Gemini delle Impostazioni: niente limiti di piano, paghi a consumo.",
                "Codex with the key saved by codex login --with-api-key, or Antigravity with the Gemini key from Settings: no plan limits, you pay for what you use.",
                "Codex avec la clé enregistrée par codex login --with-api-key, ou Antigravity avec la clé Gemini des Paramètres : pas de limites d'abonnement, vous payez à la consommation.",
                "Codex mit dem von codex login --with-api-key gespeicherten Schlüssel oder Antigravity mit dem Gemini-Schlüssel aus den Einstellungen: keine Tariflimits, du zahlst nach Verbrauch.",
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
            "Gli accessi degli agenti e di GitHub sono salvati nel kit, in chiaro. «Esci» non revoca le sessioni sul sito del fornitore: se perdi la chiavetta, revocale dalle impostazioni del tuo account di Anthropic, OpenAI, Google e GitHub.",
            "The sign-ins of the agents and of GitHub are saved in the kit, unencrypted. “Sign out” does not revoke the sessions on the provider's site: if you lose the USB stick, revoke them from your account settings at Anthropic, OpenAI, Google and GitHub.",
            "Les connexions des agents et de GitHub sont enregistrées dans le kit, en clair. « Se déconnecter » ne révoque pas les sessions sur le site du fournisseur : si vous perdez la clé USB, révoquez-les depuis les paramètres de votre compte chez Anthropic, OpenAI, Google et GitHub.",
            "Die Anmeldungen der Agenten und von GitHub sind im Kit gespeichert, unverschlüsselt. „Abmelden“ widerruft die Sitzungen auf der Website des Anbieters nicht: Wenn du den USB-Stick verlierst, widerrufe sie in den Einstellungen deines Kontos bei Anthropic, OpenAI, Google und GitHub.",
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

      <Related ids={["agents-shared", "agents-usage", "settings"]} />
    </>
  ),

  "agents-shared": () => (
    <>
      <InBrief
        points={[
          tr(
            "Istruzioni, skill e server MCP si scrivono una volta sola, nella cartella data\\agents, e valgono per tutti e tre gli agenti.",
            "Instructions, skills and MCP servers are written only once, in the data\\agents folder, and apply to all three agents.",
            "Instructions, skills et serveurs MCP s'écrivent une seule fois, dans le dossier data\\agents, et valent pour les trois agents.",
            "Anweisungen, Skills und MCP-Server werden nur einmal geschrieben, im Ordner data\\agents, und gelten für alle drei Agenten.",
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
          "Come avviene l'allineamento",
          "How the sync works",
          "Comment se fait la synchronisation",
          "Wie der Abgleich funktioniert",
        )}
      >
        <Bullets
          items={[
            tr(
              "Istruzioni: AGENTS.md viene copiato come CLAUDE.md per Claude Code, AGENTS.md per Codex e AGENTS.md per Antigravity. Se modifichi uno di quei file, la modifica torna in AGENTS.md e passa agli altri. Se cambiano entrambe le parti, vince la più recente e l'altra viene conservata nella cartella dei conflitti.",
              "Instructions: AGENTS.md is copied as CLAUDE.md for Claude Code, AGENTS.md for Codex and AGENTS.md for Antigravity. If you edit one of those files, the change goes back into AGENTS.md and on to the others. If both sides change, the most recent wins and the other is kept in the conflicts folder.",
              "Instructions : AGENTS.md est copié en CLAUDE.md pour Claude Code, AGENTS.md pour Codex et AGENTS.md pour Antigravity. Si vous modifiez l'un de ces fichiers, la modification revient dans AGENTS.md et passe aux autres. Si les deux côtés changent, le plus récent l'emporte et l'autre est conservé dans le dossier des conflits.",
              "Anweisungen: AGENTS.md wird als CLAUDE.md für Claude Code, AGENTS.md für Codex und AGENTS.md für Antigravity kopiert. Wenn du eine dieser Dateien änderst, geht die Änderung zurück in AGENTS.md und an die anderen. Ändern sich beide Seiten, gewinnt die neuere, und die andere wird im Konfliktordner aufbewahrt.",
            ),
            tr(
              "Skill: ogni agente vede la cartella skills attraverso un collegamento (copie sui dischi FAT32 ed exFAT). Una skill creata dentro un agente viene spostata in data\\agents\\skills e diventa di tutti; una skill cancellata in un agente viene messa da parte e si può recuperare.",
              "Skills: each agent sees the skills folder through a link (copies on FAT32 and exFAT drives). A skill created inside an agent is moved to data\\agents\\skills and becomes everyone's; a skill deleted in an agent is set aside and can be recovered.",
              "Skills : chaque agent voit le dossier skills à travers un lien (des copies sur les disques FAT32 et exFAT). Une skill créée dans un agent est déplacée dans data\\agents\\skills et devient commune à tous ; une skill supprimée dans un agent est mise de côté et peut être récupérée.",
              "Skills: Jeder Agent sieht den Ordner skills über eine Verknüpfung (Kopien auf FAT32- und exFAT-Laufwerken). Ein in einem Agenten erstellter Skill wird nach data\\agents\\skills verschoben und gehört dann allen; ein in einem Agenten gelöschter Skill wird beiseitegelegt und lässt sich wiederherstellen.",
            ),
            tr(
              "Server MCP: mcp.json viene tradotto nel formato di ciascun agente. Con Claude Code vale in entrambe le direzioni: un server aggiunto con claude mcp add -s user torna in mcp.json. Codex e Antigravity ricevono i server del kit e lasciano stare quelli che hai aggiunto a mano nelle loro configurazioni.",
              "MCP servers: mcp.json is translated into each agent's format. With Claude Code it works in both directions: a server added with claude mcp add -s user goes back into mcp.json. Codex and Antigravity receive the kit's servers and leave alone the ones you added by hand in their configurations.",
              "Serveurs MCP : mcp.json est traduit dans le format de chaque agent. Avec Claude Code, cela vaut dans les deux sens : un serveur ajouté avec claude mcp add -s user revient dans mcp.json. Codex et Antigravity reçoivent les serveurs du kit et ne touchent pas à ceux que vous avez ajoutés à la main dans leurs configurations.",
              "MCP-Server: mcp.json wird in das Format jedes Agenten übersetzt. Bei Claude Code gilt das in beide Richtungen: Ein mit claude mcp add -s user hinzugefügter Server kommt zurück in mcp.json. Codex und Antigravity erhalten die Server des Kits und lassen die, die du von Hand in ihren Konfigurationen hinzugefügt hast, unangetastet.",
            ),
          ]}
        />
        <Callout tone="info" title={tr("La regola del kit", "The kit's rule", "La règle du kit", "Die Regel des Kits")}>
          {tr(
            "A ogni allineamento il kit dà agli agenti una regola comune: nei progetti i file CLAUDE.md, AGENTS.md e GEMINI.md sono equivalenti, e le skill di un progetto stanno nella sua cartella .agents/skills.",
            "At every sync the kit gives the agents a common rule: in projects the files CLAUDE.md, AGENTS.md and GEMINI.md are equivalent, and a project's skills are in its .agents/skills folder.",
            "À chaque synchronisation, le kit donne aux agents une règle commune : dans les projets, les fichiers CLAUDE.md, AGENTS.md et GEMINI.md sont équivalents, et les skills d'un projet se trouvent dans son dossier .agents/skills.",
            "Bei jedem Abgleich gibt das Kit den Agenten eine gemeinsame Regel: In Projekten sind die Dateien CLAUDE.md, AGENTS.md und GEMINI.md gleichwertig, und die Skills eines Projekts liegen in seinem Ordner .agents/skills.",
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
            "Questi server valgono solo in quel progetto, oltre a quelli del kit. Claude Code li legge dal file .mcp.json del progetto (e chiede conferma al primo uso); Codex da .codex\\config.toml, che Atollo tiene allineato; Antigravity usa solo i server del kit.",
            "These servers apply only in that project, in addition to the kit's. Claude Code reads them from the project's .mcp.json file (and asks for confirmation on first use); Codex from .codex\\config.toml, which Atollo keeps in sync; Antigravity uses only the kit's servers.",
            "Ces serveurs ne valent que dans ce projet, en plus de ceux du kit. Claude Code les lit dans le fichier .mcp.json du projet (et demande confirmation à la première utilisation) ; Codex dans .codex\\config.toml, qu'Atollo garde synchronisé ; Antigravity n'utilise que les serveurs du kit.",
            "Diese Server gelten nur in diesem Projekt, zusätzlich zu denen des Kits. Claude Code liest sie aus der Datei .mcp.json des Projekts (und fragt bei der ersten Verwendung nach einer Bestätigung); Codex aus .codex\\config.toml, das Atollo abgeglichen hält; Antigravity verwendet nur die Server des Kits.",
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
                data\home\.claude\CLAUDE.md · data\home\.codex\AGENTS.md · data\home\.gemini\AGENTS.md
              </Code>,
            ],
            [
              <Code key="c">data\agents\skills</Code>,
              <Code key="d">
                data\home\.claude\skills · data\home\.codex\skills · data\home\.gemini\antigravity-cli\skills
              </Code>,
            ],
            [
              <Code key="e">data\agents\mcp.json</Code>,
              <Code key="f">
                data\home\.claude\.claude.json · data\home\.codex\config.toml · data\home\.gemini\config\mcp_config.json
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
              "Claude Code e Codex considerano attendibile la cartella di un progetto solo dopo che vi hai aperto un agente da Atollo: da lì possono eseguire comandi senza chiedere la fiducia ogni volta. Le altre cartelle di projects non sono attendibili in anticipo.",
              "Claude Code and Codex trust a project's folder only after you open an agent in that project from Atollo: from then on they can run commands there without asking for trust every time. The other folders in projects are not trusted in advance.",
              "Claude Code et Codex ne considèrent le dossier d'un projet comme fiable qu'après que vous y avez ouvert un agent depuis Atollo : dès lors ils peuvent y exécuter des commandes sans demander la confiance à chaque fois. Les autres dossiers de projects ne sont pas fiables d'avance.",
              "Claude Code und Codex vertrauen dem Ordner eines Projekts erst, nachdem du darin einen Agenten aus Atollo geöffnet hast: Ab dann können sie dort Befehle ausführen, ohne jedes Mal nach Vertrauen zu fragen. Die anderen Ordner in projects gelten nicht im Voraus als vertrauenswürdig.",
            ),
            tr(
              "In Codex le variabili ${VAR} non vengono espanse: il kit le traduce dove Codex ha una chiave apposita, altrimenti le copia così come sono con un commento.",
              "In Codex ${VAR} variables are not expanded: the kit translates them where Codex has a dedicated key, otherwise it copies them as they are with a comment.",
              "Dans Codex, les variables ${VAR} ne sont pas développées : le kit les traduit là où Codex a une clé dédiée, sinon il les copie telles quelles avec un commentaire.",
              "In Codex werden ${VAR}-Variablen nicht ersetzt: Das Kit übersetzt sie dort, wo Codex einen eigenen Schlüssel hat, und kopiert sie sonst unverändert mit einem Kommentar.",
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
            "Per ogni finestra di limite c'è una casella con la percentuale usata, una barra e «si azzera …». Le finestre sono «5 ore» e «Settimana» per Claude Code, «… ore» e «Settimana» per Codex, e quote per modello per Antigravity. Sopra, il piano, la fonte dei dati e quando sono stati letti.",
            "For each limit window there is a box with the percentage used, a bar and “resets …”. The windows are “5 hours” and “Week” for Claude Code, “… hours” and “Week” for Codex, and per-model quotas for Antigravity. Above: the plan, the data source and when the data was read.",
            "Pour chaque fenêtre de limite, il y a une case avec le pourcentage utilisé, une barre et « se réinitialise … ». Les fenêtres sont « 5 heures » et « Semaine » pour Claude Code, « … heures » et « Semaine » pour Codex, et des quotas par modèle pour Antigravity. Au-dessus : l'abonnement, la source des données et quand elles ont été lues.",
            "Für jedes Limit-Zeitfenster gibt es ein Feld mit dem verbrauchten Prozentsatz, einem Balken und „wird … zurückgesetzt“. Die Zeitfenster sind „5 Stunden“ und „Woche“ für Claude Code, „… Stunden“ und „Woche“ für Codex und Kontingente pro Modell für Antigravity. Darüber: der Tarif, die Datenquelle und wann die Daten abgefragt wurden.",
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
              "File delle sessioni: data\\home\\.claude\\projects, data\\home\\.codex\\sessions, data\\home\\.gemini (conversazioni di Antigravity).",
              "Session files: data\\home\\.claude\\projects, data\\home\\.codex\\sessions, data\\home\\.gemini (Antigravity conversations).",
              "Fichiers des sessions : data\\home\\.claude\\projects, data\\home\\.codex\\sessions, data\\home\\.gemini (conversations d'Antigravity).",
              "Sitzungsdateien: data\\home\\.claude\\projects, data\\home\\.codex\\sessions, data\\home\\.gemini (Unterhaltungen von Antigravity).",
            ),
          ]}
        />
      </Under>

      <Related ids={["agents", "agents-shared"]} />
    </>
  ),
};
