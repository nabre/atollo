<p align="center">
  <a href="https://nabre.github.io/atollo/it/">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="brand/wordmark-dark.svg">
      <img src="brand/wordmark-light.svg" alt="Atollo" width="280">
    </picture>
  </a>
</p>

<h1 align="center">Atollo — Universal Portable Dev & AI Workspace</h1>

<p align="center">
  Un ambiente di sviluppo portabile e senza installazione per Windows, macOS e Linux, su x64 e ARM64.<br>
  Editor, più di 20 linguaggi e runtime, database locali, modelli AI locali e gli agenti AI Claude Code, Codex, Antigravity e Pi in una cartella da portare su una chiavetta USB.
</p>

<p align="center">
  <a href="https://github.com/nabre/atollo/releases"><img alt="Latest release" src="https://img.shields.io/github/v/release/nabre/atollo?include_prereleases&label=release"></a>
  <a href="https://github.com/nabre/atollo/releases"><img alt="Downloads" src="https://img.shields.io/github/downloads/nabre/atollo/total?label=downloads"></a>
  <a href="https://github.com/nabre/atollo/actions/workflows/app.yml"><img alt="Build" src="https://img.shields.io/github/actions/workflow/status/nabre/atollo/app.yml?branch=main&label=build"></a>
  <a href="../LICENSE"><img alt="License: MIT" src="https://img.shields.io/github/license/nabre/atollo?label=license"></a>
  <img alt="Platforms: Windows, macOS, Linux" src="https://img.shields.io/badge/platforms-Windows%20%7C%20macOS%20%7C%20Linux-informational">
  <img alt="Architectures: x64, ARM64" src="https://img.shields.io/badge/arch-x64%20%7C%20ARM64-informational">
  <img alt="Built with Rust and Tauri 2" src="https://img.shields.io/badge/built%20with-Rust%20%2B%20Tauri%202-informational">
</p>

<p align="center">
  <a href="https://nabre.github.io/atollo/it/"><b>Sito</b></a> ·
  <a href="https://github.com/nabre/atollo/releases/latest"><b>Download</b></a> ·
  <a href="https://nabre.github.io/atollo/it/guide/"><b>Guida</b></a> ·
  <a href="https://github.com/nabre/atollo/issues"><b>Segnalazioni</b></a> ·
  <a href="PRIVACY.it.md"><b>Privacy</b></a>
</p>

<p align="center"><a href="../README.md">English</a> · Italiano · <a href="README.fr.md">Français</a> · <a href="README.de.md">Deutsch</a></p>

**Atollo** è un *Universal Portable Dev & AI Workspace* autonomo (`Atollo.exe`) che unisce **Editor** (VSCodium integrato), **Toolchain/Runtime Manager** (più di 20 linguaggi e strumenti scaricati su richiesta), **Server/Database locali** (MySQL, MongoDB, SQLite, Caddy/Apache) e **Orchestratore di Agenti AI** (Claude Code, Codex, Antigravity e Pi con istruzioni, skill e server MCP sincronizzati, più i modelli linguistici locali con llama.cpp) in una singola cartella portabile. Niente installazione, niente diritti di amministratore, nessun servizio di sistema fuori dalla cartella del workspace.

Al primo avvio l'app scarica da sola i componenti di sistema: PowerShell 7, GitHub CLI e l'editor. Il resto si sceglie nella pagina «Strumenti», oppure arriva quando serve: Node.js con un'azione `npm`, uv con un server MCP `uvx`, e così via.

<picture>
  <source media="(prefers-color-scheme: light)" srcset="../site/shots/it/shot-projects-light.png">
  <img src="../site/shots/it/shot-projects-dark.png" alt="La pagina Progetti di Atollo: pannello rapido e attività" width="100%">
</picture>

## Funzionalità

- **Editor e terminali isolati.** VSCodium integrato con i temi Atollo scuro e chiaro, in una finestra misurata sul workbench di VSCodium 1.135: le stesse dimensioni, gli stessi colori e le icone codicon. I terminali vedono solo il `PATH` e le variabili del kit: i programmi del PC ospite non interferiscono, e il PC non si sporca.
- **Linguaggi e strumenti su richiesta.** Più di 20 linguaggi e strumenti (Node.js, Python, uv, PHP, Rust, Go, Java, .NET, Dart, Julia, Ruby, C/C++, Bun, Deno, Typst, TinyTeX, R, Android SDK…), scaricati quando servono e verificati con checksum SHA-256 o firma. Più versioni affiancate, una per progetto.
- **Database e server web locali.** MySQL con phpMyAdmin, MongoDB con mongo-express e SQLite, avviati e fermati con un clic, con database, utente e password per ogni progetto; più versioni affiancate, ognuna con la sua porta. Caddy e Apache per i siti su `<progetto>.localhost`, su ogni sistema, da soli o in un server condiviso da più progetti. Una porta già occupata viene rifiutata, dicendo chi la tiene. Nessun servizio di sistema.
- **Quattro agenti AI, una sola configurazione.** Claude Code, Codex, Antigravity e Pi: installazione, accesso e avvio. Istruzioni, skill e server MCP si scrivono una volta in `data/agents` e `atollo sync` li allinea per tutti e quattro. I server MCP pronti sono a un clic: GitHub, Playwright e Fetch per tutto il kit, i file del progetto e il suo database MySQL o MongoDB per un progetto, senza token né password scritti nei file. Limiti del piano e consumi di oggi e degli ultimi 7 giorni.
- **Modelli AI locali.** Modelli linguistici che girano sul PC, senza rete e senza account: modelli GGUF dal catalogo di Atollo o da qualunque repository di Hugging Face, download che riprendono da dove si sono fermati (anche dopo aver chiuso Atollo) e controllati con lo SHA-256. Il server di llama.cpp (CPU, o GPU con Vulkan o Metal) ascolta solo su 127.0.0.1, con una chiave; Pi lo usa senza altre impostazioni, e la sua chat si apre nel browser. Una copia sola in `data/models`, condivisa da Windows, macOS e Linux.
- **Una cartella per sei sistemi.** La stessa cartella, o lo stesso SSD esterno, passa da Windows a macOS a Linux con progetti, database e configurazioni degli agenti intatti: su ogni macchina girano i programmi nativi di quel sistema. «Rimuovi l'unità» chiude tutto ciò che Atollo ha aperto e lascia espellere il disco.

## Documentazione

| Documento | Cosa contiene |
| --- | --- |
| [Guida](https://nabre.github.io/atollo/it/guide/) | ogni pagina dell'app, passo per passo, con le schermate, sul sito (anche dentro l'app) |
| [Catalogo dei componenti](app/CATALOG.it.md) | i componenti, da dove si scarica ciascuno, la disponibilità per sistema |
| [Matrice strumenti × sistemi](app/TOOLS-MATRIX.html) | ogni componente sui sei target: nativo, emulato o assente (in inglese) |
| [Architettura](app/ARCHITECTURE.it.md) | com'è organizzato il codice e dove va una modifica |
| [Prove](app/TESTING.it.md) | controlli automatici e checklist manuale per sistema |
| [Decisioni](app/DECISIONS.md) e [novità](app/CHANGELOG.md) | il perché di ogni scelta e cosa è cambiato |

Il lavoro aperto è nelle [segnalazioni](https://github.com/nabre/atollo/issues). La documentazione principale è in inglese: [README](../README.md), [guide](https://nabre.github.io/atollo/guide/).

## Come si usa

1. Copia l'exe di Atollo (per esempio `Atollo-0.6.0-win32-x64.exe`) in una cartella vuota: una chiavetta, un disco esterno o una cartella del tuo profilo (anche con spazi, parentesi o `&` nel nome).
2. Doppio clic: l'app crea `projects\` e `data\`. Al primo avvio scegli agenti, linguaggi e strumenti: l'app li scarica e li verifica (checksum o firma).
3. Crea o clona un progetto nella pagina **Progetti**. Editor, terminale e agenti di un progetto si aprono sempre nella sua cartella.

Dalla [pagina dei rilasci](https://github.com/nabre/atollo/releases): `Atollo-<versione>-win32-x64.exe` o `Atollo-<versione>-win32-arm64.exe` per Windows, `Atollo-<versione>-darwin-arm64.app.zip` (Apple Silicon) o `Atollo-<versione>-darwin-x64.app.zip` (Intel) per macOS, `Atollo-<versione>-linux-x64` o `Atollo-<versione>-linux-arm64` (`.AppImage` e `.tar.gz`) per Linux; ogni file ha il suo `.sha256`. Il nome dice la versione: quando l'app si aggiorna da sola, l'exe e l'AppImage prendono il nome della versione nuova. Negli esempi qui sotto `Atollo.exe` sta per il tuo exe.

**macOS** (12 o successivo). L'app ha solo una firma ad hoc, senza notarizzazione di Apple:
1. Estrai lo zip e sposta `Atollo.app` con il Finder nella cartella del kit (per esempio `/Volumes/KIT/Atollo/`).
2. Nel Terminale togli la quarantena del browser: `xattr -dr com.apple.quarantine /Volumes/KIT/Atollo/Atollo.app`.
3. Apri `Atollo.app` da quella cartella: `projects/` e `data/` nascono accanto a lei, mai dentro.

Senza il passo 2 macOS avvia l'app da una copia temporanea (App Translocation) e Atollo si ferma mostrando lo stesso comando, invece di creare il kit in una cartella che sparisce.

**Linux** (x64 e ARM64, glibc 2.35 o successiva: Ubuntu 22.04, Debian 12 e più recenti). Copia `Atollo-<versione>-linux-<arch>.AppImage` nella cartella del kit, `chmod +x Atollo-<versione>-linux-<arch>.AppImage` e avviala: il kit è la cartella dell'AppImage. Senza FUSE (container, alcune distribuzioni minime): `./Atollo-<versione>-linux-<arch>.AppImage --appimage-extract-and-run`, oppure `Atollo-<versione>-linux-<arch>.tar.gz` con WebKitGTK 4.1 installato (su Ubuntu e Debian `sudo apt install libwebkit2gtk-4.1-0`): estrai `Atollo` nella cartella del kit e avvia `./Atollo`. L'AppImage si aggiorna da sola, il `tar.gz` dice che c'è una versione nuova da scaricare.

## Pagine

| Pagina | Cosa c'è |
| --- | --- |
| **Progetti** | il **pannello rapido** (cosa è in esecuzione, preferiti e recenti con Avvia, Editor, Terminale e agente, cosa c'è da sistemare), la lista dei progetti con ▶ su ogni riga, Ctrl+P per aprirne uno per nome, Ctrl+Shift+P per la palette dei comandi (avviare o fermare un progetto, aprirlo, un agente nel progetto, andare a una pagina, fermare tutto); **Nuovo progetto**, una scheda accanto ai progetti: vuoto, dai tuoi repository GitHub (con clone, rinomina ed eliminazione su GitHub), da un indirizzo Git o da una delle 63 basi in gruppi (Laravel, Next.js, Django, WordPress, Astro, Tauri…); avvisi sull'ambiente. Ogni progetto ha la sua pagina, con Avvia, Editor, Terminale e agente in alto e sette sezioni: **Avvio** (azioni, porte e il loro output), **Sito web**, **Componenti** (con le versioni), **Database** (database del progetto con utente e password, copie, una copia di sicurezza prima di ogni ripristino e cambio di versione), **Estensioni**, **Editor** (impostazioni e server MCP), **Git** |
| **AI** | tutta l'IA del kit in due riquadri. **Agenti**: Claude Code, Codex, Antigravity e Pi installati: versione, accesso (con la chiave di Gemini di Antigravity), limiti del piano e consumi di oggi e degli ultimi 7 giorni; istruzioni, skill e server MCP condivisi in `data\agents`. **Modelli locali**: il server di llama.cpp (avvio e arresto, indirizzo, la chat nel browser e la sua chiave), il modello di Pi, il catalogo di Atollo dei modelli GGUF con «entra / al limite / troppo grande» per la RAM o la memoria video di questo PC, qualunque repository di Hugging Face (con un token facoltativo per i modelli ad accesso controllato), download da sospendere e riprendere; modelli in `data\models`, solo su questo PC |
| **Database** | i server MySQL (con phpMyAdmin) e MongoDB (con mongo-express) di Atollo: avvio e arresto, password dell'amministratore, porta, più versioni affiancate ognuna con la sua porta, dati in `data\db`, solo su questo PC; SQLite senza server |
| **Server Web** | i server Caddy e Apache condivisi dai progetti (pool) e i tuoi profili, con i siti `<progetto>.localhost` che servono; solo su questo PC, o sulla rete locale se le Impostazioni lo permettono |
| **Terminali** | console con gli strumenti del kit in qualunque cartella del PC: «Tutti gli strumenti» e terminali tuoi, con gli strumenti e le versioni che scegli |
| **Strumenti** | linguaggi e strumenti divisi per tipo (sistema ed editor, agenti AI, modelli locali, linguaggi, strumenti di sviluppo, server web, database, visualizzazione database), con il loro logo, le dipendenze, lo spazio su disco; installa, aggiorna e rimuove, più versioni; il compilatore di Rust (GNU o MSVC) nella pagina di Rust; la matrice delle estensioni dell'editor |
| **Guida** | la guida dell'app, in nove capitoli; ogni pagina apre il suo argomento |
| **Impostazioni** | salvate mentre le cambi: lingua, tema, i file temporanei dei programmi, i pacchetti tenuti per reinstallare senza rete, rete (proxy, certificati, rete locale), aggiornamenti dell'app, privacy; l'**account GitHub** di Atollo, uno per tutto (app, Git, editor, console, agenti, GitHub CLI), e chi firma i commit; dove si trova la cartella di Atollo; le istruzioni dei framework; **Il kit**: le sue cartelle, il suo disco, la piattaforma e le versioni, con Salute del kit, Copia del kit, Esporta diagnostica e Segnala un problema |
| **Rimuovi l'unità** | chiude tutto ciò che Atollo ha aperto, riporta in `data\` l'accesso di Antigravity e lascia espellere l'unità |

In basso, su ogni pagina, il pannello **Attività**: cosa è avviato o in installazione (azioni dei progetti, ogni versione dei database, pool dei server web, componenti), con l'output e gli errori in un solo posto; «Ferma tutto» ferma ogni cosa dopo una conferma. La barra di stato conta le stesse cose.

La console di Atollo è una scheda del pannello dei terminali dell'app su ogni sistema, con il PowerShell 7 del kit (su Windows sul ConPTY di Microsoft, la console di Windows Terminal, scaricato con il kit); non apre mai il terminale del PC e ha solo l'ambiente isolato di Atollo: i programmi installati sul PC ospite non interferiscono. Se la versione salvata di un terminale è stata rimossa, dice quale usa al suo posto (per esempio PHP 8.2 → 8.4).

## C'è ancora da fare

Atollo è in alpha (versione 0.6.0). Windows x64 è il sistema provato di più, anche con installazioni reali in CI; macOS, Linux e Windows ARM64 sono costruiti e provati solo in CI. La checklist completa è in [docs/app/TESTING.it.md](app/TESTING.it.md).

- **L'alpha, da provare.** [La 0.6.0 alpha](https://github.com/nabre/atollo/releases/tag/app-v0.6.0) è su GitHub per Windows, macOS e Linux, ognuno su x64 e ARM64. I file pubblicati sono costruiti e provati in CI: ora vanno scaricati e avviati su macchine pulite.
- **Alpha test.** Servono persone che usino Atollo su progetti veri e su PC diversi, e che segnalino cosa non va: installazione dei componenti, progetti, database, agenti. Impostazioni › Il kit › «Segnala un problema» apre una segnalazione su GitHub con versione e sistema già scritti; allegale lo zip di «Esporta diagnostica» (segreti mascherati, creato sul tuo PC, mai inviato dall'app).
- **macOS su Mac veri** ([#5](https://github.com/nabre/atollo/issues/5), [#6](https://github.com/nabre/atollo/issues/6)). Apple Silicon e Intel sono costruiti e avviati in CI, mai su un Mac pulito: lo zip scaricato con il browser, quarantena e Gatekeeper, App Translocation, il kit su una chiavetta exFAT, gli accessi degli agenti portati da un Mac all'altro.
- **Linux su macchine vere** ([#5](https://github.com/nabre/atollo/issues/5)). x64 e ARM64 su Ubuntu 24.04, Fedora 42 e Debian 13: AppImage con e senza FUSE, il `tar.gz` con WebKitGTK 4.1, chiavette exFAT o montate `noexec`, accessi con GNOME, KeePassXC o KWallet.
- **Aggiornamenti e build del kit fuori da Windows.** L'aggiornamento dell'app da una versione all'altra su macOS e Linux non è ancora provato. L'Apache del kit (2.4.69, D-116) è nel catalogo ma provato solo in CI; anche il PHP del kit con `pdo_sqlite` (8.4.26 e 8.5.11, D-133), che l'app usa dalla 0.3.0.
- **Firma del codice.** Su macOS solo una firma ad hoc, senza notarizzazione di Apple: serve `xattr -dr com.apple.quarantine`. Su Windows la 0.6.0 esce senza firma: la firma con SignPath è ancora da attivare, e fino ad allora SmartScreen può avvisare al primo avvio.

Hai un Mac, un PC Linux o un PC con Windows ARM64? Prova Atollo e raccontaci com'è andata: [apri una segnalazione su GitHub](https://github.com/nabre/atollo/issues).

## Limiti noti

- Su Windows 10 e 11 (`x64`), Windows 11 (`ARM64`), macOS (Apple Silicon e Intel) e Linux (`x64` e `ARM64`). Su Windows ARM64 l'editor, PHP, Apache, MySQL, MongoDB, Android SDK, Julia, TinyTeX e R girano in emulazione x64; su Linux ARM64 manca Android. I rilasci per macOS e Linux sono provati in CI ma non ancora su macchine pulite ([#5](https://github.com/nabre/atollo/issues/5)), e gli accessi degli agenti vanno provati su un Mac ([#6](https://github.com/nabre/atollo/issues/6)); componenti e fonti per sistema in [docs/app/CATALOG.it.md](app/CATALOG.it.md), la matrice componente × sistema in [docs/app/TOOLS-MATRIX.html](app/TOOLS-MATRIX.html) (in inglese). Apache su macOS e Linux è una build del kit (D-116), con PHP tramite `php-fpm`.
- Su Linux MySQL vuole due librerie del sistema, `libaio` e `libnuma`: su Ubuntu 24.04 e Debian 13 `sudo apt install libaio1t64 libnuma1` (su versioni precedenti `libaio1`), su Fedora e RHEL `sudo dnf install libaio numactl-libs`. Se mancano, Atollo lo dice con il comando da eseguire.
- Su macOS e Linux una cartella di Atollo su exFAT o FAT (una chiavetta formattata per Windows) funziona, ma quei file system non hanno link: dove un programma ne porta, Atollo copia i file (per esempio i 142 link interni di Git), con più spazio occupato. Su una chiavetta solo per Mac o Linux conviene APFS o ext4.
- Fuori dalla cartella di Atollo restano solo le tracce di Windows, i file temporanei di passaggio nel temp di sistema (rimossi automaticamente alla chiusura; con Impostazioni › Generale › Temporanei dei programmi, anche i programmi possono usarlo mentre girano), l'accesso di GitHub CLI nella Gestione credenziali se lo fai in una console con `gh auth login` (quello fatto nelle Impostazioni resta in `data\gh`), quello di Antigravity mentre `agy` è aperto (solo per la sessione), su macOS l'accesso di Claude Code nel Portachiavi (se non ti colleghi con un token) e i dati della webview nella tua Libreria, e la voce del registro che rustup scrive su Windows, che Atollo rimuove. L'elenco completo è in [PRIVACY §4](PRIVACY.it.md#4-i-dati-salvati-sul-tuo-dispositivo).
- La cartella di un progetto diventa attendibile per Claude Code, Codex e Pi quando vi apri un agente da Atollo (le altre cartelle di `projects\` no): apri un agente solo nei progetti di cui ti fidi. I token degli agenti (`CLAUDE_CODE_OAUTH_TOKEN`, `GEMINI_API_KEY`, `LLAMA_API_KEY` dei modelli locali) vanno solo agli agenti avviati da Atollo, non alle azioni dei progetti, agli script, ai terminali né all'editor; la chiave dei modelli locali è scritta anche nel `models.json` di Pi (`data\home\.pi\agent`, leggibile solo dal tuo utente), perché `pi` funzioni dai terminali del kit. L'editor invece chiede alla prima apertura di ogni progetto e apre in «Restricted Mode» quelli di cui non ti fidi.
- I limiti d'uso degli agenti vengono da interfacce non documentate dei fornitori e possono smettere di funzionare dopo un loro aggiornamento.
- «Rimuovi l'unità» non vede i programmi del PC che tengono aperto solo un file della cartella di Atollo.

## Struttura

```
Atollo\
  Atollo.exe      tutta la logica
  projects\       i tuoi progetti
  data\           programmi scaricati, impostazioni, accessi, cache, temporanei
                  (le impostazioni di ogni progetto: data\projects\<progetto>\dev-settings.json)
```

Per un backup c'è Impostazioni › Il kit › **Copia del kit**: esporta `projects\` e `data\` in una cartella o in uno zip, senza `data\apps\`, `data\cache\` e `data\models\` (si riscaricano), con i database fermi per la copia, e importa una copia così in un kit al suo avvio successivo. A mano, copia le stesse cartelle con Atollo chiuso. La copia contiene le password del kit: tienila al sicuro.

## Disinstallazione

Atollo non usa un installer, non registra servizi di sistema né attività pianificate e non modifica il registro o il `PATH` di Windows:

1. Apri la pagina **Rimuovi l'unità** (oppure chiudi l'applicazione) per arrestare tutti i processi avviati da Atollo e riportare in `data\` l'eventuale sessione di Antigravity.
2. Elimina la cartella che contiene `Atollo.exe`, `data\` e `projects\` (dopo aver salvato i progetti che vuoi conservare).
3. Eventuali cartelle temporanee di passaggio in `%TEMP%` vengono rimosse automaticamente alla chiusura; se avevi eseguito `gh auth login` manualmente da terminale (anziché dalla pagina **Impostazioni**), puoi rimuovere la voce corrispondente dalla **Gestione credenziali** di Windows.

## Riga di comando

```
Atollo.exe                           apre la finestra
Atollo.exe env [--project <nome>]    mostra l'ambiente che ricevono i programmi di Atollo
                                     (con un progetto, con le sue versioni)
Atollo.exe components [--target <sistema>]
                                     elenca linguaggi e strumenti, con quelli installati
Atollo.exe versions [<id>]           le versioni su disco; con un id, quelle da scaricare
Atollo.exe check                     verifica che i programmi installati rispondano
    --kit                            la salute del kit: disco, spazio, percorso, permessi, librerie
    --files                          i file dei programmi: mancanti o cambiati dall'installazione
Atollo.exe prepare [opzioni]         installa e aggiorna i componenti scelti
                                     e rimuove quelli non scelti
    --only a,b                       solo questi (e ciò che serve loro), non rimuove niente
    --skip a,b                       non questi
    --update-all                     aggiorna anche quelli già installati
    --force                          scarica di nuovo
    --line id=x.y                    aggiunge quella versione accanto alle altre (ripetibile)
    --target <sistema>               prepara per un altro sistema: win32-x64, win32-arm64,
                                     darwin-arm64, darwin-x64, linux-x64, linux-arm64
Atollo.exe sync [--project <cartella>] [--verbose]
                                     allinea istruzioni, skill e server MCP degli agenti
Atollo.exe keyring restore|save      accesso di Antigravity (lo usa agy.cmd del kit)
Atollo.exe serve <cartella> <porta>  server di file per un sito statico
Atollo.exe models                    i modelli locali: del catalogo e scaricati
Atollo.exe models pull <modello>     scarica un id del catalogo o un modello di Hugging Face
                                     (proprietario/nome:quantizzazione), riprende se interrotto
Atollo.exe models rm <id>            cancella un modello scaricato
Atollo.exe llm start|stop|status     il server dei modelli locali (chat e agenti)
Atollo.exe diagnostics               crea lo zip da allegare a una segnalazione (segreti mascherati)
Atollo.exe version                   mostra la versione
Atollo.exe help                      mostra l'aiuto
```

Da PowerShell aggiungi `| Out-Host` (per esempio `.\Atollo.exe check | Out-Host`): `Atollo.exe` è un programma a finestra, e senza una pipe PowerShell non lo aspetta e non ne legge l'output né il codice di uscita.

## Compilare

Prerequisiti: Rust stabile (almeno 1.93) con il toolchain MSVC (Visual Studio Build Tools, carico di lavoro C++), Node.js LTS.

```
npm ci
npm run build
```

Le dipendenze JavaScript stanno solo nella radice (`node_modules` e `package-lock.json`): `ui` è un workspace npm.

L'exe è in `target\release\Atollo.exe`. Nessun installer viene creato (`--no-bundle`).

Controlli prima di una modifica:

```
npm run lint
npm test
npm -w ui run build
```

`npm run lint` esegue `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, ESLint e Prettier; `npm test` esegue `cargo test` e i test dell'interfaccia; `npm -w ui run build` controlla i tipi e compila l'interfaccia. Dettagli in [docs/app/TESTING.it.md](app/TESTING.it.md). Com'è organizzato il codice e dove va una modifica: [docs/app/ARCHITECTURE.it.md](app/ARCHITECTURE.it.md).

Sviluppo con ricarica automatica: `npm run dev`.

## Se Windows blocca l'exe

Finché il certificato di firma non è attivo sulla release scaricata, se SmartScreen mostra "Windows ha protetto il PC", scegli **Ulteriori informazioni** › **Esegui comunque**. Se l'exe è stato scaricato da Internet: tasto destro › **Proprietà** › **Sblocca**. Su PC aziendali con AppLocker serve il permesso del reparto IT.

## Code signing policy

Free code signing on Windows provided by [SignPath.io](https://signpath.io/), certificate by [SignPath Foundation](https://signpath.org/).

- **Ruoli del progetto (Committer, Reviewer e Approver):** [nabre](https://github.com/nabre)
- **Provenienza dei binari:** gli eseguibili delle release ufficiali vengono compilati e firmati esclusivamente dalla pipeline automatizzata GitHub Actions ([`.github/workflows/app.yml`](../.github/workflows/app.yml)) a partire dai sorgenti pubblici di questo repository, senza interventi manuali sui binari.

## Privacy policy

Il testo completo, con ogni connessione di rete e ogni file che l'app conserva, è in **[Informativa sulla privacy e condizioni d'uso](PRIVACY.it.md)**. In breve:

- **Nessuna telemetria o raccolta dati:** l'applicazione non raccoglie dati personali, statistiche d'uso, telemetria o report di crash, e non contatta alcun server gestito dall'autore del progetto. Disattiva inoltre la telemetria predefinita dei componenti gestiti (come PowerShell 7 e il server di VSCodium).
- **Archiviazione locale:** impostazioni, progetti, database locali, log e credenziali sono salvati in locale nella cartella `data\` (e `projects\`) di Atollo sul dispositivo dell'utente; le poche eccezioni (file temporanei, l'accesso di Antigravity mentre è in esecuzione) sono elencate nell'informativa.
- **Connessioni di rete:** l'applicazione si collega solo a GitHub per cercare i propri aggiornamenti (Impostazioni › Generale › Aggiornamento automatico, oppure `ATOLLO_NO_UPDATE`), alle fonti ufficiali dei componenti che scarica, al registro Open VSX per le estensioni dell'editor e, quando l'utente fa l'accesso a GitHub (Impostazioni › Account GitHub) o apre la pagina AI, a GitHub e ai fornitori degli agenti (**Claude Code**, **Codex**, **Antigravity** e il provider in cloud scelto in **Pi**) secondo le loro informative sulla privacy. I modelli locali arrivano da Hugging Face (`huggingface.co` e la sua CDN), i programmi llama.cpp e Pi dalle loro release su GitHub (`ggml-org/llama.cpp`, `earendil-works/pi`); un modello locale gira sul PC, e i suoi prompt non lo lasciano mai.

## Licenza

MIT: fornito "così com'è", senza garanzie. Vedi [LICENSE](../LICENSE) e le [condizioni d'uso](PRIVACY.it.md#8-condizioni-duso).
