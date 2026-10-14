# Dev-Kit

Ambiente di sviluppo portabile per Windows in un solo file: `Dev-Kit.exe`. Editor (VSCodium) integrato, agenti Claude Code, Codex e Antigravity, linguaggi e strumenti scaricati su richiesta. Niente installazione, niente diritti di amministratore, nessun file fuori dalla cartella del kit.

Al primo avvio l'app scarica da sola i componenti di sistema: PowerShell 7, GitHub CLI, l'editor e quello che serve a loro (Git per GitHub CLI). Il resto si sceglie nella pagina «Linguaggi e strumenti», oppure arriva quando serve: Node.js con un'azione `npm`, uv con un server MCP `uvx`, e così via.

> In sviluppo. Stato e piano: [docs/app/PLAN.md](docs/app/PLAN.md), novità: [docs/app/CHANGELOG.md](docs/app/CHANGELOG.md).
> Prende spunto da [nabre/dev-kit](https://github.com/nabre/dev-kit), di cui non è una nuova versione: è un prodotto separato.

## Struttura

```
Dev-Kit\
  Dev-Kit.exe     tutta la logica
  projects\       i tuoi progetti
  data\           programmi scaricati, impostazioni, accessi, cache, temporanei
```

Per un backup basta copiare `projects\` e `data\` (meno `data\apps\` e `data\cache\`, che si riscaricano).

## Riga di comando

```
Dev-Kit.exe                          apre la finestra
Dev-Kit.exe env                      mostra l'ambiente che ricevono i programmi del kit
Dev-Kit.exe components               elenca linguaggi e strumenti, con quelli installati
Dev-Kit.exe check                    verifica che i programmi installati rispondano
Dev-Kit.exe prepare [opzioni]        installa, aggiorna e rimuove i componenti scelti
    --only a,b                       solo questi (e ciò che serve loro)
    --skip a,b                       non questi
    --update-all                     aggiorna anche quelli già installati
    --force                          scarica di nuovo
Dev-Kit.exe sync [--project <cartella>] [--verbose]
                                     allinea istruzioni, skill e MCP dei tre agenti
Dev-Kit.exe keyring restore|save     accesso di Antigravity (lo usa agy.cmd del kit)
Dev-Kit.exe serve <cartella> <porta> server di file per un sito statico
Dev-Kit.exe version                  mostra la versione
Dev-Kit.exe help                     mostra l'aiuto
```

## Compilare

Prerequisiti: Rust stabile (almeno 1.93) con il toolchain MSVC (Visual Studio Build Tools, carico di lavoro C++), Node.js LTS.

```
npm ci
npm run build
```

Le dipendenze JavaScript stanno solo nella radice (`node_modules` e `package-lock.json`): `ui` è un workspace npm.

L'exe è in `target\release\Dev-Kit.exe`. Nessun installer viene creato (`--no-bundle`).

Controlli prima di una modifica:

```
npm run lint
npm test
npm -w ui run build
```

`npm run lint` esegue `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, ESLint e Prettier; `npm test` esegue `cargo test` e i test dell'interfaccia; `npm -w ui run build` controlla i tipi e compila l'interfaccia. Dettagli in [docs/app/TESTING.md](docs/app/TESTING.md).

Sviluppo con ricarica automatica: `npm run dev`.

## Se Windows blocca l'exe

L'exe non è ancora firmato. Se SmartScreen mostra "Windows ha protetto il PC", scegli **Ulteriori informazioni** › **Esegui comunque**. Se l'exe è stato scaricato da Internet: tasto destro › **Proprietà** › **Sblocca**. Su PC aziendali con AppLocker serve il permesso del reparto IT.

## Licenza

MIT.
