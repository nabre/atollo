# Atollo: informativa sulla privacy e condizioni d'uso

[English](PRIVACY.md) · Italiano · [Français](PRIVACY.fr.md) · [Deutsch](PRIVACY.de.md)

In vigore dal: 22 ottobre 2026

Questa pagina spiega che cosa fanno con i tuoi dati l'app desktop Atollo (`Atollo.exe`, la `.app` di macOS e l'AppImage di Linux) e il suo sito. Atollo è open source: tutto quello che è scritto qui si può verificare nel codice di questo repository.

## 1. In breve

- Atollo **non ha telemetria, statistiche d'uso né report dei crash**. Non ha account né server propri.
- Tutto quello che Atollo salva resta **nella cartella di Atollo** sul tuo dispositivo (`data/` e `projects/`), con le poche eccezioni elencate nella sezione 4.
- Atollo si collega alla rete solo per **cercare i propri aggiornamenti**, per **scaricare gli strumenti che usi dalle loro fonti ufficiali** e per parlare con **GitHub e i fornitori degli agenti AI quando usi queste funzioni**. La tabella della sezione 3 elenca ogni connessione e come disattivarla.
- Gli strumenti e gli agenti AI che avvii dentro Atollo (Claude Code, Codex, Antigravity, Pi, Git, npm…) sono programmi di terzi con le proprie informative sulla privacy.
- I **modelli AI locali** girano sul tuo computer: con un modello locale i tuoi prompt e i tuoi file non lo lasciano mai.

## 2. Che cosa Atollo non raccoglie

Atollo non raccoglie, non invia e non vende:

- dati personali, nomi, indirizzi e-mail o identificativi del dispositivo;
- statistiche d'uso, clic, pagine viste o funzioni usate;
- report dei crash o log (lo zip della diagnostica della sezione 4 si crea sul tuo dispositivo e ne esce solo se lo alleghi tu);
- il contenuto dei tuoi progetti, file, database o conversazioni con gli agenti AI.

Nessuno di chi gestisce il progetto Atollo riceve dati dall'app. Il progetto non ha server: aggiornamenti e release sono ospitati su GitHub.

Atollo spegne inoltre, nel proprio ambiente, la telemetria e i controlli di aggiornamento di diversi programmi che gestisce: l'editor integrato (VSCodium) parte con la telemetria spenta e senza aggiornamento automatico delle estensioni; l'ambiente imposta `GOTELEMETRY=off`, `DOTNET_CLI_TELEMETRY_OPTOUT=1`, `POWERSHELL_TELEMETRY_OPTOUT=1`, `POWERSHELL_UPDATECHECK=Off`, `PIP_DISABLE_PIP_VERSION_CHECK=1`, `GH_NO_UPDATE_NOTIFIER=1` (per le chiamate dell'app a GitHub CLI), `DISABLE_UPDATES=1` (Claude Code), `AGY_CLI_DISABLE_AUTO_UPDATE=true` (Antigravity), `PI_SKIP_VERSION_CHECK=1` e `PI_TELEMETRY=0` (Pi). Gli altri programmi mantengono le loro impostazioni predefinite.

## 3. Le connessioni di rete dell'app

Ogni richiesta che l'app fa da sé porta uno `User-Agent` della forma `Atollo/<versione> (+https://github.com/nabre/atollo)` e, come ogni connessione a Internet, fa conoscere il tuo indirizzo IP al server contattato. Le richieste usano il proxy impostato nelle Impostazioni, altrimenti `HTTPS_PROXY`/`HTTP_PROXY`, altrimenti il proxy del sistema.

| Quando | Verso chi | Che cosa viene inviato | Come disattivarla |
|---|---|---|---|
| Circa 20 secondi dopo l'avvio, e quando premi "Cerca aggiornamenti" | GitHub (`api.github.com`, poi il download della release da `github.com`) | La richiesta dell'elenco delle release di `nabre/atollo`; se c'è una versione più recente, il suo download. Se sul computer è impostata una variabile `GITHUB_TOKEN` o `GH_TOKEN`, viene inviata solo ad `api.github.com` (per evitare il limite di richieste di GitHub). | Impostazioni › Generale › **Aggiornamento automatico** spento, oppure la variabile `ATOLLO_NO_UPDATE` impostata sul computer. Il pulsante funziona comunque a richiesta. |
| Circa 10 secondi dopo l'avvio, al massimo una volta al giorno per strumento | La fonte ufficiale di ogni strumento che offre più versioni (per esempio `nodejs.org`, `go.dev`, `api.adoptium.net`, `api.github.com`) | La richiesta dell'elenco delle versioni disponibili. Solo verso `api.github.com`: il token sopra, oppure quello del tuo account GitHub se hai fatto l'accesso (sezione 4). | La variabile `ATOLLO_NO_UPDATE`. |
| Subito dopo la riga precedente, al massimo una volta al giorno; e quando premi "Controlla ora" nel riquadro "Aggiornamenti" della pagina Strumenti | La fonte ufficiale di ogni strumento installato, la stessa che interroga un suo aggiornamento (per esempio `nodejs.org`, `go.dev`, `api.adoptium.net`, `api.github.com`) | La richiesta dell'ultima versione dello strumento, una per strumento installato; le risposte di GitHub passano dalla cache della sezione 4. Solo verso `api.github.com`: lo stesso token della riga sopra. | Impostazioni › Generale › **Aggiornamento automatico** spento, oppure la variabile `ATOLLO_NO_UPDATE`. "Controlla ora" funziona comunque a richiesta. |
| Al primo avvio, quando confermi la pagina del primo avvio (dopo aver scelto lingua e rete); in seguito, da solo, quando manca un componente necessario o un aggiornamento dell'app ne richiede di più recenti | Fonti ufficiali di PowerShell 7, GitHub CLI e dell'editor (release su GitHub) | Richieste di download. | Nessun interruttore: questi componenti sono necessari. Nient'altro viene installato senza una tua azione. |
| Quando installi o aggiorni uno strumento (pagina Strumenti) o un'azione ne richiede uno (per esempio Node.js per `npm`) | La fonte ufficiale dello strumento, per esempio `nodejs.org`, `registry.npmjs.org`, `go.dev`, `static.rust-lang.org`, `api.adoptium.net`, `repo.maven.apache.org`, `services.gradle.org`, `builds.dotnet.microsoft.com`, `windows.php.net`, `getcomposer.org`, `cdn.mysql.com`, `downloads.mongodb.org`, `sqlite.org`, `files.phpmyadmin.net`, `conda.anaconda.org`, `api.anaconda.org`, `dl.google.com`, `storage.googleapis.com`, `raw.githubusercontent.com`, `www.apachelounge.com`, `dl.static-php.dev`, `curl.se`, `aka.ms`, `api.nuget.org`, `julialang-s3.julialang.org`, `ziglang.org`, il server di aggiornamento di Antigravity di Google, le release su GitHub (tra cui `ggml-org/llama.cpp` per il server dei modelli locali e `earendil-works/pi` per Pi) | Richieste di download; i checksum vengono verificati in locale. | Non installare lo strumento. |
| Solo quando scarichi un modello locale (pagina Modelli locali, o `models pull`) o cerchi un repository di Hugging Face | Hugging Face: `huggingface.co` (l'API, per l'ultimo commit di un repository e i suoi file GGUF), poi la sua CDN per i file del modello | Richieste per l'elenco del repository e per i file del modello a un commit fissato; un download sospeso o interrotto riprende con una richiesta `Range`. Il tuo token di Hugging Face, solo se ne hai salvato uno per i modelli ad accesso controllato, e solo a `huggingface.co` (non passa alla CDN). Lo SHA-256 di ogni file viene verificato in locale. | Non scaricare modelli. |
| In background dopo l'avvio dell'editor, e quando si apre un progetto | La galleria delle estensioni dell'editor (Open VSX); CodeLLDB dalla sua release su GitHub | Le richieste delle estensioni usate dal kit e dai tuoi progetti. | Non aprire l'editor, oppure rimuovere le estensioni dall'editor. |
| Solo quando fai l'accesso a GitHub (Impostazioni › Account GitHub) e usi i tuoi repository (Progetti › «Dai tuoi repository GitHub») | GitHub (`github.com`, `api.github.com`), tramite GitHub CLI | L'accesso (un codice monouso da digitare su `github.com/login/device`), poi il token del tuo account con le richieste per elencare, clonare, rinominare o eliminare i tuoi repository. | Non fare l'accesso, oppure esci in Impostazioni › Account GitHub. |
| Una volta, all'accesso o a un avvio con l'accesso fatto, solo se nome ed e-mail dei commit sono vuoti | GitHub (`api.github.com/user`), tramite GitHub CLI | Il token del tuo account, per leggere nome, login e numero dell'account: diventano il nome dei tuoi commit e l'indirizzo privato `noreply` di GitHub. | Compila Impostazioni › Account GitHub › «Chi firma i commit», oppure non fare l'accesso. |
| Solo mentre è aperta la pagina Agenti (all'apertura, poi ogni 2 minuti), per ogni agente in cui hai fatto l'accesso | Claude Code: `api.anthropic.com`; Codex: `chatgpt.com`; Antigravity: `cloudcode-pa.googleapis.com` | Il token di accesso dell'agente, inviato solo al fornitore di quell'agente, per leggere i limiti d'uso del tuo piano. Il token viene solo letto, mai rinnovato, e mai scritto nel log. | Non aprire la pagina Agenti, oppure non fare l'accesso all'agente. |

**Non** sono connessioni fatte da Atollo:

- i programmi che avvii nell'editor, nelle console o con le azioni (`git`, `npm`, `pip`, `cargo`, gli agenti AI…) si collegano dove sono progettati per farlo;
- i link che clicchi si aprono nel tuo browser, e così «Segnala un problema» (sezione 4).

Server locali: l'editor e i server dei progetti ascoltano solo su questo computer (`127.0.0.1`/`localhost`), a meno che tu non attivi **Rete locale** nelle Impostazioni per i server dei progetti. Con la rete locale accesa, il QR dell'indirizzo di un sito per il telefono lo crea Atollo su questo computer: nessun servizio su internet vede l'indirizzo, e trovare l'IP di questo PC sulla rete locale non manda alcun pacchetto. Il server dei modelli locali (llama.cpp) ascolta sempre e solo su `127.0.0.1`, su una porta sua (9931 e seguenti), e risponde solo alle richieste con la sua chiave casuale; `llm status` non manda quella chiave a chi ascolta sulla porta. I modelli girano su questo computer: con un modello locale niente va in rete.

## 4. I dati salvati sul tuo dispositivo

Atollo è portabile: tiene i suoi dati accanto all'eseguibile, nella cartella `data/`, e i tuoi progetti in `projects/`. Non viene caricato nulla. Eliminare la cartella di Atollo elimina tutto.

| Che cosa | Dove (dentro la cartella di Atollo) |
|---|---|
| Impostazioni: lingua, tema, proxy, certificati, aggiornamento automatico, compilatore di Rust | `data/app/settings.json` |
| Le impostazioni di Git, con il nome e l'e-mail che firmano i tuoi commit | `data/home/.gitconfig` |
| La **chiave API di Gemini**, se la inserisci (leggibile solo dal tuo utente, vedi sotto) | `data/app/secrets/gemini-api-key` |
| I certificati radice del sistema più il tuo file PEM aggiuntivo, solo se ne imposti uno | `data/app/ca-bundle.pem` |
| Log dell'app (segreti mascherati), ruotati oltre 1 MB | `data/app/logs/` |
| Zip della diagnostica, solo quando ne crei uno (segreti mascherati, vedi sotto) | `data/app/diagnostics/` |
| Strumenti installati e le loro versioni | `data/apps/`, `data/app/versions/` |
| L'ultima versione di ogni strumento installato, con l'ora dell'ultimo controllo | `data/app/updates.json` |
| Risposte dell'API di GitHub in cache | `data/cache/github/` |
| Memoria dell'interfaccia (disposizione della finestra) e cache della web view | `data/webview2/` (Windows e Linux) |
| Account GitHub (token di GitHub CLI, salvato come file perché viaggi con il kit) | `data/gh/` |
| Credenziali Git per altri host (Git Credential Manager, cifrate con DPAPI di Windows per il tuo utente) | `data/home/.gcm/` (Windows) |
| Accessi e sessioni degli agenti AI, scritti dagli agenti stessi | `data/home/.claude/`, `data/home/.codex/`, `data/home/.gemini/` |
| L'accesso Google di Antigravity, tolto dall'archivio credenziali del sistema dopo ogni esecuzione | `data/antigravity/keyring.json` |
| Il token di lunga durata di Claude Code, se usi "Collega con un token" (macOS) | `data/agents/claude/oauth-token` |
| Password dei database (casuali, generate da Atollo) | `data/app/db/secrets.json`, `data/app/db/mysql.cnf`, `data/projects/<progetto>/private/db.json` |
| Modelli locali (file GGUF) e cosa ne ha scaricato Atollo | `data/models/gguf/<proprietario>--<repository>/` (con `atollo.json`) |
| Impostazioni dei modelli locali (porta, avvio automatico, il modello di Pi) e i preset del server | `data/app/llm.json`, `data/app/llm/presets.ini` |
| La **chiave del server dei modelli locali** (casuale, generata da Atollo, leggibile solo dal tuo utente) | `data/app/secrets/llama-key` |
| Il **token di Hugging Face**, se ne inserisci uno per i modelli ad accesso controllato (leggibile solo dal tuo utente) | `data/app/secrets/hf-token` |
| Impostazioni, accessi (`auth.json`) e sessioni di Pi, e il `models.json` del kit con indirizzo e chiave del server locale (leggibile solo dal tuo utente) | `data/home/.pi/agent/` |
| Dati del server dell'editor e il suo token di accesso per sessione | `data/vscodium-server/` |
| I tuoi progetti | `projects/` |

Le password dei database, la chiave API di Gemini, il file del token di Claude, la chiave dei modelli locali e il token di Hugging Face vengono creati leggibili solo dal tuo utente, dove il disco supporta i permessi (NTFS, APFS, ext4…). Gli altri file, `settings.json` compreso, sono file normali. Sulle chiavette formattate FAT32 o exFAT non esistono permessi: **chiunque abbia la chiavetta può leggere tutti i file sopra**. Tieni la chiavetta al sicuro, oppure usa un disco con i permessi.

Il token di Claude e la chiave API di Gemini vengono passati (come `CLAUDE_CODE_OAUTH_TOKEN` e `GEMINI_API_KEY`) solo agli agenti avviati da Atollo, non alle azioni dei progetti, agli script, ai terminali né all'editor. La chiave dei modelli locali va a Pi quando lo avvia Atollo (`LLAMA_API_KEY`) e nel `models.json` di Pi, così `pi` funziona anche dai terminali del kit. Il token di Hugging Face va solo a `huggingface.co`.

Eccezioni, fuori dalla cartella di Atollo:

- **File temporanei**: i file in transito vanno in `<temp del sistema>/Atollo/`; secondo Impostazioni › Generale › **Temporanei dei programmi**, anche i programmi possono usare la cartella temporanea del sistema (per impostazione predefinita quando il kit è su una chiavetta, su un disco di rete o su FAT32/exFAT).
- **Accesso di Antigravity**: mentre Antigravity è in esecuzione, il suo accesso sta nell'archivio credenziali del sistema (su Windows solo per la sessione di accesso), poi torna nel kit e viene rimosso. Le credenziali già presenti sul computer vengono rimesse com'erano; una nota sigillata (`keyring-before.json`, leggibile solo su quel computer) resta nel kit fino al salvataggio successivo.
- **Claude Code su macOS** tiene il proprio accesso nel Portachiavi, a meno che tu non usi "Collega con un token".
- **Web view di macOS**: su macOS la web view del sistema tiene i suoi dati dove decide macOS (nella Libreria del tuo utente).
- **rustup** scrive una voce nel registro di Windows a ogni installazione; Atollo la rimuove.

### Segnalare un problema

Impostazioni › Il kit › «Segnalare un problema» ha due pulsanti; niente parte da solo.

- **«Esporta diagnostica»** (o `Atollo diagnostics` dalla riga di comando) crea uno zip **sul tuo dispositivo**, in `data/app/diagnostics/`: versione di Atollo, sistema del kit e del computer (build di Windows, RAM), disco del kit (file system, tipo, spazio libero), cartella del kit, ambiente del kit, verifica dei programmi installati, componenti, porte e gli ultimi 256 KB di ogni log di `data/app/logs/`. Prima di scrivere lo zip, ogni segreto che Atollo conserva (i file di `data/app/secrets/`, il token di Claude Code, le password dei database, gli accessi degli agenti nel kit, il token di GitHub) diventa `***` ovunque compaia, oltre alla mascheratura delle variabili dal nome segreto; la cartella del tuo profilo diventa `~`, e i nomi del tuo utente e del computer (`USERNAME`, `COMPUTERNAME`, `HOSTNAME`…) diventano `***`. **L'app non manda mai lo zip**: aprilo, leggilo e allegalo tu, se vuoi. L'output di un progetto nei log può contenere comunque cose che Atollo non sa essere private.
- **«Segnala un problema»** apre nel tuo browser `github.com/nabre/atollo/issues/new`, con il modulo dei problemi e due campi compilati attraverso l'indirizzo: la versione di Atollo e il sistema (sistema del kit, build di Windows, file system e tipo del disco del kit). Nient'altro del tuo dispositivo è nell'indirizzo, e nulla arriva a GitHub finché non accedi e crei la segnalazione. Da lì la segnalazione, con ciò che alleghi, è pubblica e vale l'[informativa sulla privacy di GitHub](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement).

## 5. Strumenti e agenti AI di terzi

Atollo installa e avvia programmi fatti da altri. Quando li usi, parlano direttamente con i propri servizi e **valgono le loro informative sulla privacy e le loro condizioni**. Atollo non fa da proxy a quel traffico, non lo legge e non lo salva, a parte le sessioni e gli accessi che quei programmi scrivono da sé nella cartella del kit (sezione 4).

- **Claude Code** (Anthropic): [anthropic.com/legal/privacy](https://www.anthropic.com/legal/privacy)
- **Codex** (OpenAI): [openai.com/policies/privacy-policy](https://openai.com/policies/privacy-policy/)
- **Antigravity** e l'API di Gemini (Google): [policies.google.com/privacy](https://policies.google.com/privacy)
- **Pi** (Earendil): [pi.dev](https://pi.dev); con un provider in cloud (accesso con `/login`) vale l'informativa di quel provider
- **Hugging Face**, da cui si scaricano i modelli locali: [huggingface.co/privacy](https://huggingface.co/privacy)
- **llama.cpp** (ggml-org), il server dei modelli locali: gira sul tuo computer e non manda nulla; [github.com/ggml-org/llama.cpp](https://github.com/ggml-org/llama.cpp)
- **GitHub** e GitHub CLI: [GitHub Privacy Statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement)
- **VSCodium** e Open VSX: [open-vsx.org](https://open-vsx.org/)
- I registri dei pacchetti (npm, PyPI, crates.io, Maven Central, NuGet…) e i siti di download di ogni strumento, secondo le loro regole.

Gli agenti AI inviano ai loro fornitori i tuoi prompt e i file che leggono: è così che funzionano. Scegli tu che cosa aprire con loro. Fa eccezione Pi quando usa un modello locale: allora prompt e file restano sul tuo computer; con un provider in cloud scelto in Pi, vanno a quel provider.

## 6. Il sito

Il sito [nabre.github.io/atollo](https://nabre.github.io/atollo/it/) è una pagina statica ospitata su GitHub Pages.

- Niente statistiche, tracciamento, cookie o pubblicità. Font e immagini sono serviti dal sito stesso.
- Il sito ricorda il tema e la lingua scelti nel `localStorage` del tuo browser (`atollo-theme`, `atollo-lang`); non escono dal browser.
- Per far puntare il pulsante di download al file giusto, il tuo browser chiede ad `api.github.com` l'elenco delle release di Atollo.
- GitHub, come host, può registrare dati tecnici come gli indirizzi IP: vedi il [GitHub Privacy Statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement).
- Il sito è verificato in Google Search Console con un file di verifica; questo non aggiunge alcun tracciamento per chi visita.

## 7. Minori

Atollo è uno strumento per sviluppatori e non è rivolto ai minori. Non raccoglie dati da nessuno, minori compresi.

## 8. Condizioni d'uso

Atollo è software libero rilasciato con la [licenza MIT](../LICENSE). È fornito **"così com'è", senza garanzie di alcun tipo**, esplicite o implicite; gli autori non rispondono di reclami, danni o altre responsabilità derivanti dal suo uso. Sei responsabile di come lo usi e del rispetto delle licenze e delle condizioni degli strumenti e dei servizi di terzi che installa o a cui si collega (sezione 5).

## 9. Modifiche

Quando questa informativa cambia, la nuova versione viene pubblicata in questo file con una nuova data di entrata in vigore; le versioni precedenti restano nella cronologia del repository.

## 10. Contatti

Domande o dubbi: apri una segnalazione su [github.com/nabre/atollo/issues](https://github.com/nabre/atollo/issues).
