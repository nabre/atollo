# Atollo

Italiano · [English](docs/README.en.md) · [Français](docs/README.fr.md) · [Deutsch](docs/README.de.md)

Ambiente di sviluppo portabile per Windows in un solo file: `Atollo.exe`. Editor (VSCodium) integrato, agenti Claude Code, Codex e Antigravity, linguaggi e strumenti scaricati su richiesta. Niente installazione, niente diritti di amministratore, nessun file fuori dalla cartella del kit.

Al primo avvio l'app scarica da sola i componenti di sistema: PowerShell 7, GitHub CLI, l'editor e quello che serve a loro (Git per GitHub CLI). Il resto si sceglie nella pagina «Strumenti», oppure arriva quando serve: Node.js con un'azione `npm`, uv con un server MCP `uvx`, e così via.

> In sviluppo. Stato e piano: [docs/app/PLAN.md](docs/app/PLAN.md), novità: [docs/app/CHANGELOG.md](docs/app/CHANGELOG.md).
> Prende spunto da [nabre/atollo](https://github.com/nabre/atollo), di cui non è una nuova versione: è un prodotto separato.

## Come si usa

1. Copia `Atollo.exe` in una cartella vuota: una chiavetta, un disco esterno o una cartella del tuo profilo (anche con spazi, parentesi o `&` nel nome).
2. Doppio clic: l'app crea `projects\` e `data\`. Al primo avvio scegli agenti, linguaggi e strumenti: l'app li scarica e li verifica (checksum o firma).
3. Crea o clona un progetto nella pagina **Progetti**. Editor, terminale e agenti si aprono sempre nella cartella di un progetto, mai altrove.

## Pagine

| Pagina | Cosa c'è |
| --- | --- |
| **Progetti** | i progetti con Avvia, Editor, Terminale e agente; **Nuovo progetto**: vuoto, dai tuoi repository GitHub (con clone, rinomina ed eliminazione su GitHub) o da un indirizzo Git; avvisi sul kit. Ogni progetto ha la sua pagina: **Riepilogo** (l'azione principale con il suo output, linguaggi, database, agente, Git), **Risorse** (componenti, database del progetto con utente e password, estensioni dell'editor), **Avanzate** (azioni, porte, sito statico, Git, istruzioni degli agenti) |
| **Agenti** | Claude Code, Codex e Antigravity installati: versione, accesso, limiti del piano e consumi di oggi e degli ultimi 7 giorni; istruzioni, skill e server MCP condivisi in `data\agents` |
| **Database** | i server MariaDB (compatibile con MySQL) e MongoDB del kit: avvio e arresto, password dell'amministratore, porta, dati in `data\db`, solo su questo PC; SQLite senza server |
| **Strumenti** | linguaggi e strumenti con il loro logo, le dipendenze, lo spazio su disco; installa, aggiorna e rimuove; la matrice delle estensioni dell'editor |
| **Impostazioni** | lingua, terminale, rete (proxy, certificati, rete locale), chiave di Gemini, versioni di Python e PHP, compilatore di Rust (GNU o MSVC), aggiornamenti dell'app; l'**account GitHub** del kit, uno per tutto (app, Git, editor, console, agenti, GitHub CLI); dove si trova il kit |
| **Rimuovi l'unità** | chiude tutto ciò che il kit ha aperto, riporta nel kit l'accesso di Antigravity e lascia espellere la chiavetta |

In basso, su ogni pagina, il pannello **Attività**: cosa è avviato o in installazione (progetti, database, componenti), con l'output e gli errori in un solo posto.

La console del kit è la console classica di Windows con PowerShell 7 del kit e il solo ambiente del kit: i programmi installati sul PC non si vedono.

## Cosa resta del Atollo

Il menu in console di [nabre/atollo](https://github.com/nabre/atollo) non c'è: ogni sua voce ha una pagina dell'app («Proxy e impostazioni» è la pagina Impostazioni) e la logica degli script è in Rust dentro l'exe. Restano gli stessi componenti (più PowerShell 7 e l'editor integrato, meno VSCodium desktop), lo stesso allineamento di istruzioni, skill e MCP fra i tre agenti, la stessa rilocazione quando il kit cambia cartella o lettera di unità. I kit del Atollo non vengono importati.

## Limiti noti

- Solo Windows 10 e 11, x64 e ARM64 (su ARM64 l'editor, PHP, Apache e TinyTeX girano in emulazione x64; R non c'è).
- Fuori dal kit restano solo le tracce di Windows, l'accesso di GitHub CLI nella Gestione credenziali se lo fai in una console con `gh auth login` (quello fatto nelle Impostazioni resta nel kit, in `data\gh`) e quello di Antigravity mentre `agy` è aperto (solo per la sessione).
- Le cartelle in `projects\` sono attendibili per Claude Code e Codex: apri solo progetti di cui ti fidi. L'editor invece chiede alla prima apertura di ogni progetto e apre in «Restricted Mode» quelli di cui non ti fidi.
- I limiti d'uso degli agenti vengono da interfacce non documentate dei fornitori e possono smettere di funzionare dopo un loro aggiornamento.
- «Rimuovi l'unità» non vede i programmi del PC che tengono aperto solo un file del kit.
- L'exe non è firmato (vedi sotto).

## Struttura

```
Atollo\
  Atollo.exe     tutta la logica
  projects\       i tuoi progetti
  data\           programmi scaricati, impostazioni, accessi, cache, temporanei
                  (le impostazioni del kit di ogni progetto: data\projects\<progetto>\dev-settings.json)
```

Per un backup basta copiare `projects\` e `data\` (meno `data\apps\` e `data\cache\`, che si riscaricano).

## Riga di comando

```
Atollo.exe                          apre la finestra
Atollo.exe env                      mostra l'ambiente che ricevono i programmi del kit
Atollo.exe components               elenca linguaggi e strumenti, con quelli installati
Atollo.exe check                    verifica che i programmi installati rispondano
Atollo.exe prepare [opzioni]        installa, aggiorna e rimuove i componenti scelti
    --only a,b                       solo questi (e ciò che serve loro)
    --skip a,b                       non questi
    --update-all                     aggiorna anche quelli già installati
    --force                          scarica di nuovo
Atollo.exe sync [--project <cartella>] [--verbose]
                                     allinea istruzioni, skill e MCP dei tre agenti
Atollo.exe keyring restore|save     accesso di Antigravity (lo usa agy.cmd del kit)
Atollo.exe serve <cartella> <porta> server di file per un sito statico
Atollo.exe version                  mostra la versione
Atollo.exe help                     mostra l'aiuto
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

L'exe non è ancora firmato. Se SmartScreen mostra "Windows ha protetto il PC", scegli **Ulteriori informazioni** › **Esegui comunque**. Se l'exe è stato scaricato da Internet: tasto destro › **Proprietà** › **Sblocca**. Su PC aziendali con AppLocker serve il permesso del reparto IT.

## Licenza

MIT.
