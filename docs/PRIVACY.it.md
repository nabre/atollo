# Atollo: informativa sulla privacy e condizioni d'uso

[English](PRIVACY.md) · Italiano · [Français](PRIVACY.fr.md) · [Deutsch](PRIVACY.de.md)

In vigore dal: 20 ottobre 2026

Questa pagina spiega che cosa fanno con i tuoi dati l'app desktop Atollo (`Atollo.exe`, la `.app` di macOS e l'AppImage di Linux) e il suo sito. Atollo è open source: tutto quello che è scritto qui si può verificare nel codice di questo repository.

## 1. In breve

- Atollo **non ha telemetria, statistiche d'uso né report dei crash**. Non ha account né server propri.
- Tutto quello che Atollo salva resta **nella cartella di Atollo** sul tuo dispositivo (`data/` e `projects/`), con le poche eccezioni elencate nella sezione 4.
- Atollo si collega alla rete solo per **cercare i propri aggiornamenti**, per **scaricare gli strumenti che usi dalle loro fonti ufficiali** e per parlare con **GitHub e i fornitori degli agenti AI quando usi queste funzioni**. La tabella della sezione 3 elenca ogni connessione e come disattivarla.
- Gli strumenti e gli agenti AI che avvii dentro Atollo (Claude Code, Codex, Antigravity, Git, npm…) sono programmi di terzi con le proprie informative sulla privacy.

## 2. Che cosa Atollo non raccoglie

Atollo non raccoglie, non invia e non vende:

- dati personali, nomi, indirizzi e-mail o identificativi del dispositivo;
- statistiche d'uso, clic, pagine viste o funzioni usate;
- report dei crash o log;
- il contenuto dei tuoi progetti, file, database o conversazioni con gli agenti AI.

Nessuno di chi gestisce il progetto Atollo riceve dati dall'app. Il progetto non ha server: aggiornamenti e release sono ospitati su GitHub.

Atollo spegne inoltre, nel proprio ambiente, la telemetria e i controlli di aggiornamento di diversi programmi che gestisce: l'editor integrato (VSCodium) parte con la telemetria spenta e senza aggiornamento automatico delle estensioni; l'ambiente imposta `GOTELEMETRY=off`, `DOTNET_CLI_TELEMETRY_OPTOUT=1`, `POWERSHELL_TELEMETRY_OPTOUT=1`, `POWERSHELL_UPDATECHECK=Off`, `PIP_DISABLE_PIP_VERSION_CHECK=1`, `GH_NO_UPDATE_NOTIFIER=1` (per le chiamate dell'app a GitHub CLI), `DISABLE_UPDATES=1` (Claude Code) e `AGY_CLI_DISABLE_AUTO_UPDATE=true` (Antigravity). Gli altri programmi mantengono le loro impostazioni predefinite.

## 3. Le connessioni di rete dell'app

Ogni richiesta che l'app fa da sé porta uno `User-Agent` della forma `Atollo/<versione> (+https://github.com/nabre/atollo)` e, come ogni connessione a Internet, fa conoscere il tuo indirizzo IP al server contattato. Le richieste usano il proxy impostato nelle Impostazioni, altrimenti `HTTPS_PROXY`/`HTTP_PROXY`, altrimenti il proxy del sistema.

| Quando | Verso chi | Che cosa viene inviato | Come disattivarla |
|---|---|---|---|
| Circa 20 secondi dopo l'avvio, e quando premi "Cerca aggiornamenti" | GitHub (`api.github.com`, poi il download della release da `github.com`) | La richiesta dell'elenco delle release di `nabre/atollo`; se c'è una versione più recente, il suo download. Se sul computer è impostata una variabile `GITHUB_TOKEN` o `GH_TOKEN`, viene inviata solo ad `api.github.com` (per evitare il limite di richieste di GitHub). | Impostazioni › Generale › **Aggiornamento automatico** spento, oppure la variabile `ATOLLO_NO_UPDATE` impostata sul computer. Il pulsante funziona comunque a richiesta. |
| Circa 10 secondi dopo l'avvio, al massimo una volta al giorno per strumento | La fonte ufficiale di ogni strumento che offre più versioni (per esempio `nodejs.org`, `go.dev`, `api.adoptium.net`, `api.github.com`) | La richiesta dell'elenco delle versioni disponibili. Solo verso `api.github.com`: il token sopra, oppure quello del tuo account GitHub se hai fatto l'accesso (sezione 4). | La variabile `ATOLLO_NO_UPDATE`. |
| Al primo avvio, quando confermi la pagina del primo avvio (dopo aver scelto lingua e rete); in seguito, da solo, quando manca un componente necessario o un aggiornamento dell'app ne richiede di più recenti | Fonti ufficiali di PowerShell 7, GitHub CLI e dell'editor (release su GitHub) | Richieste di download. | Nessun interruttore: questi componenti sono necessari. Nient'altro viene installato senza una tua azione. |
| Quando installi o aggiorni uno strumento (pagina Strumenti) o un'azione ne richiede uno (per esempio Node.js per `npm`) | La fonte ufficiale dello strumento, per esempio `nodejs.org`, `registry.npmjs.org`, `go.dev`, `static.rust-lang.org`, `api.adoptium.net`, `repo.maven.apache.org`, `services.gradle.org`, `builds.dotnet.microsoft.com`, `windows.php.net`, `getcomposer.org`, `cdn.mysql.com`, `downloads.mongodb.org`, `sqlite.org`, `files.phpmyadmin.net`, `conda.anaconda.org`, `dl.google.com`, il server di aggiornamento di Antigravity di Google, le release su GitHub | Richieste di download; i checksum vengono verificati in locale. | Non installare lo strumento. |
| In background dopo l'avvio dell'editor, e quando si apre un progetto | La galleria delle estensioni dell'editor (Open VSX); CodeLLDB dalla sua release su GitHub | Le richieste delle estensioni usate dal kit e dai tuoi progetti. | Non aprire l'editor, oppure rimuovere le estensioni dall'editor. |
| Solo quando usi la pagina GitHub | GitHub (`github.com`, `api.github.com`), tramite GitHub CLI | L'accesso (un codice monouso da digitare su `github.com/login/device`), poi il token del tuo account con le richieste per elencare, clonare, rinominare o eliminare i tuoi repository. | Non fare l'accesso, oppure esci dalla pagina GitHub. |
| Una volta, all'accesso o a un avvio con l'accesso fatto, solo se nome ed e-mail dei commit sono vuoti | GitHub (`api.github.com/user`), tramite GitHub CLI | Il token del tuo account, per leggere nome, login e numero dell'account: diventano il nome dei tuoi commit e l'indirizzo privato `noreply` di GitHub. | Compila Impostazioni › Account GitHub › «Chi firma i commit», oppure non fare l'accesso. |
| Solo mentre è aperta la pagina Agenti (all'apertura, poi ogni 2 minuti), per ogni agente in cui hai fatto l'accesso | Claude Code: `api.anthropic.com`; Codex: `chatgpt.com`; Antigravity: `cloudcode-pa.googleapis.com` | Il token di accesso dell'agente, inviato solo al fornitore di quell'agente, per leggere i limiti d'uso del tuo piano. Il token viene solo letto, mai rinnovato, e mai scritto nel log. | Non aprire la pagina Agenti, oppure non fare l'accesso all'agente. |

**Non** sono connessioni fatte da Atollo:

- i programmi che avvii nell'editor, nelle console o con le azioni (`git`, `npm`, `pip`, `cargo`, gli agenti AI…) si collegano dove sono progettati per farlo;
- i link che clicchi si aprono nel tuo browser.

Server locali: l'editor e i server dei progetti ascoltano solo su questo computer (`127.0.0.1`/`localhost`), a meno che tu non attivi **Rete locale** nelle Impostazioni per i server dei progetti.

## 4. I dati salvati sul tuo dispositivo

Atollo è portabile: tiene i suoi dati accanto all'eseguibile, nella cartella `data/`, e i tuoi progetti in `projects/`. Non viene caricato nulla. Eliminare la cartella di Atollo elimina tutto.

| Che cosa | Dove (dentro la cartella di Atollo) |
|---|---|
| Impostazioni: lingua, proxy, certificati, aggiornamento automatico, versioni | `data/app/settings.json` |
| La **chiave API di Gemini**, se la inserisci (leggibile solo dal tuo utente, vedi sotto) | `data/app/secrets/gemini-api-key` |
| I certificati radice del sistema più il tuo file PEM aggiuntivo, solo se ne imposti uno | `data/app/ca-bundle.pem` |
| Log dell'app (segreti mascherati), ruotati oltre 1 MB | `data/app/logs/` |
| Strumenti installati e le loro versioni | `data/apps/`, `data/app/versions/` |
| Risposte dell'API di GitHub in cache | `data/cache/github/` |
| Memoria dell'interfaccia (disposizione della finestra) e cache della web view | `data/webview2/` (Windows e Linux) |
| Account GitHub (token di GitHub CLI, salvato come file perché viaggi con il kit) | `data/gh/` |
| Credenziali Git per altri host (Git Credential Manager, cifrate con DPAPI di Windows per il tuo utente) | `data/home/.gcm/` (Windows) |
| Accessi e sessioni degli agenti AI, scritti dagli agenti stessi | `data/home/.claude/`, `data/home/.codex/`, `data/home/.gemini/` |
| L'accesso Google di Antigravity, tolto dall'archivio credenziali del sistema dopo ogni esecuzione | `data/antigravity/keyring.json` |
| Il token di lunga durata di Claude Code, se usi "Collega con un token" (macOS) | `data/agents/claude/oauth-token` |
| Password dei database (casuali, generate da Atollo) | `data/app/db/secrets.json`, `data/app/db/mysql.cnf`, `data/projects/<progetto>/private/db.json` |
| Dati del server dell'editor e il suo token di accesso per sessione | `data/vscodium-server/` |
| I tuoi progetti | `projects/` |

Le password dei database, la chiave API di Gemini e il file del token di Claude vengono creati leggibili solo dal tuo utente, dove il disco supporta i permessi (NTFS, APFS, ext4…). Gli altri file, `settings.json` compreso, sono file normali. Sulle chiavette formattate FAT32 o exFAT non esistono permessi: **chiunque abbia la chiavetta può leggere tutti i file sopra**. Tieni la chiavetta al sicuro, oppure usa un disco con i permessi.

Eccezioni, fuori dalla cartella di Atollo:

- **File temporanei**: i file in transito vanno in `<temp del sistema>/Atollo/`; secondo Impostazioni › Generale › **Cartella temporanea dei programmi**, anche i programmi possono usare la cartella temporanea del sistema (per impostazione predefinita quando il kit è su una chiavetta, su un disco di rete o su FAT32/exFAT).
- **Accesso di Antigravity**: mentre Antigravity è in esecuzione, il suo accesso sta nell'archivio credenziali del sistema (su Windows solo per la sessione di accesso), poi torna nel kit e viene rimosso. Le credenziali già presenti sul computer vengono rimesse com'erano; una nota sigillata (`keyring-before.json`, leggibile solo su quel computer) resta nel kit fino al salvataggio successivo.
- **Claude Code su macOS** tiene il proprio accesso nel Portachiavi, a meno che tu non usi "Collega con un token".
- **Web view di macOS**: su macOS la web view del sistema tiene i suoi dati dove decide macOS (nella Libreria del tuo utente).
- **rustup** scrive una voce nel registro di Windows a ogni installazione; Atollo la rimuove.

## 5. Strumenti e agenti AI di terzi

Atollo installa e avvia programmi fatti da altri. Quando li usi, parlano direttamente con i propri servizi e **valgono le loro informative sulla privacy e le loro condizioni**. Atollo non fa da proxy a quel traffico, non lo legge e non lo salva, a parte le sessioni e gli accessi che quei programmi scrivono da sé nella cartella del kit (sezione 4).

- **Claude Code** (Anthropic): [anthropic.com/legal/privacy](https://www.anthropic.com/legal/privacy)
- **Codex** (OpenAI): [openai.com/policies/privacy-policy](https://openai.com/policies/privacy-policy/)
- **Antigravity** e l'API di Gemini (Google): [policies.google.com/privacy](https://policies.google.com/privacy)
- **GitHub** e GitHub CLI: [GitHub Privacy Statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement)
- **VSCodium** e Open VSX: [open-vsx.org](https://open-vsx.org/)
- I registri dei pacchetti (npm, PyPI, crates.io, Maven Central, NuGet…) e i siti di download di ogni strumento, secondo le loro regole.

Gli agenti AI inviano ai loro fornitori i tuoi prompt e i file che leggono: è così che funzionano. Scegli tu che cosa aprire con loro.

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
