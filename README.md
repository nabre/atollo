# Atollo — Universal Portable Dev & AI Workspace

Italiano · [English](docs/README.en.md) · [Français](docs/README.fr.md) · [Deutsch](docs/README.de.md)

**Atollo** è un *Universal Portable Dev & AI Workspace* autonomo (`Atollo.exe`) che unisce **Editor** (VSCodium integrato), **Toolchain/Runtime Manager** (più di 20 linguaggi e strumenti scaricati su richiesta), **Server/Database locali** (MySQL, MongoDB, SQLite, Caddy/Apache) e **Orchestratore di Agenti AI** (Claude Code, Codex e Antigravity con istruzioni, skill e server MCP sincronizzati) in una singola cartella portabile. Niente installazione, niente diritti di amministratore, nessun servizio di sistema fuori dalla cartella del workspace.

Al primo avvio l'app scarica da sola i componenti di sistema: PowerShell 7, GitHub CLI e l'editor. Il resto si sceglie nella pagina «Strumenti», oppure arriva quando serve: Node.js con un'azione `npm`, uv con un server MCP `uvx`, e così via.

> Guida: [docs/app/GUIDA.md](docs/app/GUIDA.md), decisioni: [docs/app/DECISIONS.md](docs/app/DECISIONS.md), novità: [docs/app/CHANGELOG.md](docs/app/CHANGELOG.md), lavoro aperto: [issue](https://github.com/nabre/atollo/issues).

## Come si usa

1. Copia l'exe di Atollo (per esempio `Atollo-0.2.0-win32-x64.exe`) in una cartella vuota: una chiavetta, un disco esterno o una cartella del tuo profilo (anche con spazi, parentesi o `&` nel nome).
2. Doppio clic: l'app crea `projects\` e `data\`. Al primo avvio scegli agenti, linguaggi e strumenti: l'app li scarica e li verifica (checksum o firma).
3. Crea o clona un progetto nella pagina **Progetti**. Editor, terminale e agenti si aprono sempre nella cartella di un progetto, mai altrove.

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
| **Progetti** | i progetti con Avvia, Editor, Terminale e agente; **Nuovo progetto**: vuoto, dai tuoi repository GitHub (con clone, rinomina ed eliminazione su GitHub) o da un indirizzo Git; avvisi sull'ambiente. Ogni progetto ha la sua pagina: **Riepilogo** (l'azione principale con il suo output, linguaggi, database, agente, Git), **Risorse** (componenti, database del progetto con utente e password, estensioni dell'editor), **Avanzate** (azioni, porte, sito statico, Git, istruzioni degli agenti) |
| **Agenti** | Claude Code, Codex e Antigravity installati: versione, accesso, limiti del piano e consumi di oggi e degli ultimi 7 giorni; istruzioni, skill e server MCP condivisi in `data\agents` |
| **Database** | i server MySQL (con phpMyAdmin) e MongoDB di Atollo: avvio e arresto, password dell'amministratore, porta, dati in `data\db`, solo su questo PC; SQLite senza server |
| **Strumenti** | linguaggi e strumenti con il loro logo, le dipendenze, lo spazio su disco; installa, aggiorna e rimuove; la matrice delle estensioni dell'editor |
| **Impostazioni** | lingua, terminale, rete (proxy, certificati, rete locale), chiave di Gemini, versioni di Python e PHP, compilatore di Rust (GNU o MSVC), aggiornamenti dell'app; l'**account GitHub** di Atollo, uno per tutto (app, Git, editor, console, agenti, GitHub CLI); dove si trova la cartella di Atollo |
| **Rimuovi l'unità** | chiude tutto ciò che Atollo ha aperto, riporta in `data\` l'accesso di Antigravity e lascia espellere l'unità |

In basso, su ogni pagina, il pannello **Attività**: cosa è avviato o in installazione (progetti, database, componenti), con l'output e gli errori in un solo posto.

La console di Atollo è la console classica di Windows con PowerShell 7 integrata e il solo ambiente isolato di Atollo: i programmi installati sul PC ospite non interferiscono.

## Limiti noti

- Su Windows 10 e 11 (`x64` e `ARM64`), macOS (Apple Silicon e Intel) e Linux (`x64` e `ARM64`). Su Windows ARM64 l'editor, PHP, Apache, MySQL, MongoDB, Android SDK, Julia, TinyTeX e R girano in emulazione x64; su Linux ARM64 manca Android. I rilasci per macOS e Linux, dal prossimo tag, sono provati in CI ma non ancora su macchine pulite ([#5](https://github.com/nabre/atollo/issues/5)), e gli accessi degli agenti vanno provati su un Mac ([#6](https://github.com/nabre/atollo/issues/6)); componenti e fonti per sistema in [docs/app/CATALOG.md](docs/app/CATALOG.md), la matrice componente × sistema in [docs/app/MATRICE-STRUMENTI.html](docs/app/MATRICE-STRUMENTI.html). Apache su macOS e Linux è una build del kit (D-116), con PHP tramite `php-fpm`.
- Su Linux MySQL vuole due librerie del sistema, `libaio` e `libnuma`: su Ubuntu 24.04 e Debian 13 `sudo apt install libaio1t64 libnuma1` (su versioni precedenti `libaio1`), su Fedora e RHEL `sudo dnf install libaio numactl-libs`. Se mancano, Atollo lo dice con il comando da eseguire.
- Su macOS e Linux una cartella di Atollo su exFAT o FAT (una chiavetta formattata per Windows) funziona, ma quei file system non hanno link: dove un programma ne porta, Atollo copia i file (per esempio i 142 link interni di Git), con più spazio occupato. Su una chiavetta solo per Mac o Linux conviene APFS o ext4.
- Fuori dalla cartella di Atollo restano solo le tracce di Windows, i file temporanei di passaggio nel temp di sistema (rimossi automaticamente alla chiusura), l'accesso di GitHub CLI nella Gestione credenziali se lo fai in una console con `gh auth login` (quello fatto nelle Impostazioni resta in `data\gh`) e quello di Antigravity mentre `agy` è aperto (solo per la sessione).
- Le cartelle in `projects\` sono attendibili per Claude Code e Codex: apri solo progetti di cui ti fidi. L'editor invece chiede alla prima apertura di ogni progetto e apre in «Restricted Mode» quelli di cui non ti fidi.
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

Per un backup basta copiare `projects\` e `data\` (meno `data\apps\` e `data\cache\`, che si riscaricano).

## Disinstallazione

Atollo non usa un installer, non registra servizi di sistema né attività pianificate e non modifica il registro o il `PATH` di Windows:

1. Apri la pagina **Rimuovi l'unità** (oppure chiudi l'applicazione) per arrestare tutti i processi avviati da Atollo e riportare in `data\` l'eventuale sessione di Antigravity.
2. Elimina la cartella che contiene `Atollo.exe`, `data\` e `projects\` (dopo aver salvato i progetti che vuoi conservare).
3. Eventuali cartelle temporanee di passaggio in `%TEMP%` vengono rimosse automaticamente alla chiusura; se avevi eseguito `gh auth login` manualmente da terminale (anziché dalla pagina **Impostazioni**), puoi rimuovere la voce corrispondente dalla **Gestione credenziali** di Windows.

## Riga di comando

```
Atollo.exe                           apre la finestra
Atollo.exe env                       mostra l'ambiente che ricevono i programmi di Atollo
Atollo.exe components                elenca linguaggi e strumenti, con quelli installati
Atollo.exe check                     verifica che i programmi installati rispondano
Atollo.exe prepare [opzioni]         installa, aggiorna e rimuove i componenti scelti
    --only a,b                       solo questi (e ciò che serve loro)
    --skip a,b                       non questi
    --update-all                     aggiorna anche quelli già installati
    --force                          scarica di nuovo
Atollo.exe sync [--project <cartella>] [--verbose]
                                     allinea istruzioni, skill e MCP dei tre agenti
Atollo.exe keyring restore|save      accesso di Antigravity (usato da agy.cmd)
Atollo.exe serve <cartella> <porta>  server di file per un sito statico
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

`npm run lint` esegue `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, ESLint e Prettier; `npm test` esegue `cargo test` e i test dell'interfaccia; `npm -w ui run build` controlla i tipi e compila l'interfaccia. Dettagli in [docs/app/TESTING.md](docs/app/TESTING.md). Com'è organizzato il codice e dove va una modifica: [docs/app/ARCHITECTURE.md](docs/app/ARCHITECTURE.md).

Sviluppo con ricarica automatica: `npm run dev`.

## Se Windows blocca l'exe

Finché il certificato di firma non è attivo sulla release scaricata, se SmartScreen mostra "Windows ha protetto il PC", scegli **Ulteriori informazioni** › **Esegui comunque**. Se l'exe è stato scaricato da Internet: tasto destro › **Proprietà** › **Sblocca**. Su PC aziendali con AppLocker serve il permesso del reparto IT.

## Code signing policy

Free code signing on Windows provided by [SignPath.io](https://signpath.io/), certificate by [SignPath Foundation](https://signpath.org/).

- **Ruoli del progetto (Committer, Reviewer e Approver):** [nabre](https://github.com/nabre)
- **Provenienza dei binari:** gli eseguibili delle release ufficiali vengono compilati e firmati esclusivamente dalla pipeline automatizzata GitHub Actions ([`.github/workflows/app.yml`](.github/workflows/app.yml)) a partire dai sorgenti pubblici di questo repository, senza interventi manuali sui binari.

## Privacy policy

Atollo (`Atollo.exe`) rispetta pienamente la riservatezza dell'utente:

- **Nessuna telemetria o raccolta dati:** l'applicazione non raccoglie dati personali, statistiche d'uso, telemetria o report di crash, e non contatta alcun server gestito dall'autore del progetto. Disattiva inoltre la telemetria predefinita dei componenti gestiti (come PowerShell 7 e il server di VSCodium).
- **Archiviazione locale:** impostazioni, progetti, database locali, log e credenziali sono salvati esclusivamente in locale nella cartella `data\` (e `projects\`) di Atollo sul dispositivo dell'utente.
- **Connessioni di rete:** l'applicazione effettua richieste di rete solo verso le fonti ufficiali upstream per verificare gli aggiornamenti e scaricare i componenti richiesti dall'utente (`api.github.com`, mirror ufficiali dei linguaggi/strumenti, registro Open VSX per le estensioni dell'editor) e, quando l'utente configura e utilizza l'integrazione GitHub o gli agenti AI (**Claude Code**, **Codex**, **Antigravity**) o la chiave API Gemini, direttamente verso le rispettive API dei fornitori secondo le loro informative sulla privacy.

## Licenza

MIT.
