# Atollo

Ambiente di sviluppo portabile per Windows in un solo file: `Atollo.exe`. Editor (VSCodium) integrato, agenti Claude Code, Codex e Antigravity, linguaggi e strumenti scaricati su richiesta. Niente installazione, niente diritti di amministratore, nessun file fuori dalla cartella del kit.

> In sviluppo. Stato e piano: [docs/app/PLAN.md](docs/app/PLAN.md), novità: [docs/app/CHANGELOG.md](docs/app/CHANGELOG.md).
> Prende spunto da [nabre/atollo](https://github.com/nabre/atollo), di cui non è una nuova versione: è un prodotto separato.

## Struttura

```
Atollo\
  Atollo.exe     tutta la logica
  projects\       i tuoi progetti
  data\           programmi scaricati, impostazioni, accessi, cache, temporanei
```

Per un backup basta copiare `projects\` e `data\` (meno `data\apps\` e `data\cache\`, che si riscaricano).

## Riga di comando

```
Atollo.exe            apre la finestra
Atollo.exe env        mostra l'ambiente che ricevono i programmi del kit
Atollo.exe version    mostra la versione
```

## Compilare

Prerequisiti: Rust stabile con il toolchain MSVC (Visual Studio Build Tools, carico di lavoro C++), Node.js LTS.

```
npm ci
npm run build
```

Le dipendenze JavaScript stanno solo nella radice (`node_modules` e `package-lock.json`): `ui` è un workspace npm.

L'exe è in `target\release\Atollo.exe`. Nessun installer viene creato (`--no-bundle`).

Controlli prima di una modifica:

```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
npm -w ui run lint && npm -w ui run format:check && npm -w ui test
```

Sviluppo con ricarica automatica: `npm run dev`.

## Se Windows blocca l'exe

L'exe non è ancora firmato. Se SmartScreen mostra "Windows ha protetto il PC", scegli **Ulteriori informazioni** › **Esegui comunque**. Se l'exe è stato scaricato da Internet: tasto destro › **Proprietà** › **Sblocca**. Su PC aziendali con AppLocker serve il permesso del reparto IT.

## Licenza

MIT.
