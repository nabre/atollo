# 02 — Verifica di UX, UI, Esperienza di Utilizzo e Facilitazione Utente (v2 — Confronto con v1)

> [!NOTE]
> Questo documento aggiorna la valutazione delle **Dimensioni 3 (UX: `86/100`, `=`)**, **4 (UI: `90/100`, `=`)**, **5 (Esperienza di utilizzo: da `88` a `89/100`, `+1 pt`)** e **9 (Qualità dell'esperienza e facilitazione: `91/100`, `=`)** confrontando la baseline `v1` con lo stato attuale `v2`.

---

## 1. User Interface (UI) e Design System (v1: `90/100` → v2: `90/100` | `=`)

### 1.1 Stato del Frontend e del Workbench (`ui/src/**`)

Poiché il commit `fa8a34d` si è concentrato interamente sul backend Rust (`tauri/src/**`), sul catalogo (`tauri/catalog/components.json`) e sugli script runtime (`runtime/bashrc`), il codice frontend in `ui/src/**` è rimasto invariato rispetto alla `v1`:

- **Punti di Forza Confermati**:
  - Layout Workbench VSCodium-native a 5 aree ([`ui/src/workbench/layout.tsx`](file:///c:/Users/nabre/Projects/dev-kit2/ui/src/workbench/layout.tsx));
  - 4 temi completi (**Scuro**, **Chiaro**, **Contrasto elevato**, **Chiaro a contrasto elevato**) verificati matematicamente dai test WCAG in [`test/ui/theme.test.ts`](file:///c:/Users/nabre/Projects/dev-kit2/test/ui/theme.test.ts) e sincronizzati con VSCodium e Windows Terminal;
  - Navigazione da tastiera sull'`ActivityBar` e scorciatoie native (`Ctrl+B`, `Ctrl+J`).

- **Difetti UI Ancora Aperti da `v1` (Quick Wins per il prossimo ciclo frontend)**:
  | Aspetto | File & Righe | Stato in `v2` | Correzione Consigliata |
  | :--- | :--- | :---: | :--- |
  | **`capitalize` CSS su lingue romanze (`BUG-21`)** | [`ui/src/workbench/layout.tsx:165,218,535`](file:///c:/Users/nabre/Projects/dev-kit2/ui/src/workbench/layout.tsx#L165) | Aperto | Rimuovere la classe Tailwind `capitalize` da `SideBarTitle`, `ViewPane` e `Panel`. |
  | **Micro-tipografia `<12px`** | [`ui/src/App.tsx:310`](file:///c:/Users/nabre/Projects/dev-kit2/ui/src/App.tsx#L310) (`text-[9px]`), [`ui/src/guide/widgets.tsx:321`](file:///c:/Users/nabre/Projects/dev-kit2/ui/src/guide/widgets.tsx#L321) (`text-[10px]`) | Aperto | Portare i badge compatti ad almeno `11px–12px`. |
  | **Badge `"latest"` / `"locked"` non localizzati** | [`ui/src/versions/kit.tsx:69,184,275`](file:///c:/Users/nabre/Projects/dev-kit2/ui/src/versions/kit.tsx#L69) | Aperto | Passare le stringhe per `t()` nei cataloghi `locales/*.json`. |

---

## 2. User Experience (UX) ed Esperienza di Utilizzo (UX: `86/100` `=` | Utilizzo: `88 → 89/100` `+1`)

### 2.1 Novità Esperienziali Introdotte dal Backend `v2`

Sebbene i file `.tsx` non siano cambiati, le modifiche al backend in [`tauri/src/commands/setup.rs:82-103`](file:///c:/Users/nabre/Projects/dev-kit2/tauri/src/commands/setup.rs#L82-L103), [`tauri/src/setup/runner.rs:194-281`](file:///c:/Users/nabre/Projects/dev-kit2/tauri/src/setup/runner.rs#L194-L281) e [`tauri/src/kit.rs:518-541`](file:///c:/Users/nabre/Projects/dev-kit2/tauri/src/kit.rs#L518-L541) migliorano direttamente l'esperienza d'uso multi-dispositivo:

1. **Coabitazione Trasparente su Unità Esterna tra Windows, macOS e Linux (`+1 pt` su Esperienza d'Uso)**:
   - In `v1`, se uno sviluppatore o docente usava NEXKORA su una chiavetta USB/SSD esterno passando da un PC Windows (`E:\DevKit`) a un Mac (`/Volumes/USB/DevKit`), `check_moved()` sovrascriveva l'unico `data/app/root.txt` e leggeva i file `data/app/versions/<id>.txt` di Windows credendo che i tool fossero già installati anche per macOS.
   - In `v2`, `check_moved()` mantiene tracciati separati per sistema operativo (`root.txt` / `old-root.txt` per Windows, `root-darwin.txt` per macOS, `root-linux.txt` per Linux) e isola le versioni in `data/app/versions/<target>/<id>.txt` mentre preserva un'unica scelta utente condivisa in `data/app/components.txt`. L'utente ritrova così i propri componenti scelti già selezionati sul nuovo OS, pronti per essere scaricati con 1 clic nella cartella `data/apps/<target>/` senza intaccare l'installazione Windows!
2. **Disponibilità Intelligente dei Componenti su Architetture ARM64 (`darwin-arm64`, `linux-arm64` vs `win32-arm64`)**:
   - In `v1`, i flag `no_arm64` (usato da `R`) e `x64_only` (usato da `PHP`, `Apache`, `Julia`, `TinyTeX`, `MariaDB`, `MongoDB`, `VSCodium`) venivano valutati contro `kit.arch() == Arch::Arm64` in modo indiscriminato. Su un Mac Apple Silicon (`darwin-arm64`) o Linux ARM64 (`linux-arm64`), `R` sarebbe risultato bloccato ("non disponibile su ARM64") nonostante `conda-forge` fornisca build native `osx-arm64` e `linux-aarch64`!
   - In `v2`, [`Component::is_available_on(target)`](file:///c:/Users/nabre/Projects/dev-kit2/tauri/src/setup/catalog.rs#L250-L258) e [`Component::is_emulated_on(target)`](file:///c:/Users/nabre/Projects/dev-kit2/tauri/src/setup/catalog.rs#L262-L264) circoscrivono `no_arm64` e `x64_only` esclusivamente a `Target::WIN32_ARM64`, e filtrano `Windows Terminal` tramite `targets: ["win32-x64", "win32-arm64"]`.

### 2.2 Punti di Attrito UX Ancora Aperti da `v1`

1. **Scollegamento tra "Avvia" nella Pagina Progetto e la Pagina "Server Web"**: Premere *Avvia* in [`ui/src/projects/run.tsx`](file:///c:/Users/nabre/Projects/dev-kit2/ui/src/projects/run.tsx) avvia ancora un processo per-progetto anziché il Pool Server Condiviso.
2. **Stati Vuoti Silenziosi (`BUG-18`) ed Errori "Sticky" (`BUG-19`)**: In [`ui/src/editorsettings.tsx:21-43`](file:///c:/Users/nabre/Projects/dev-kit2/ui/src/editorsettings.tsx#L21-L43) e [`ui/src/extensions.tsx:36`](file:///c:/Users/nabre/Projects/dev-kit2/ui/src/extensions.tsx#L36) un errore di lettura rende ancora `null` senza messaggio né pulsante *Riprova*.
3. **Focus Trap nei Dialoghi Modali (`BUG-20`)**: [`ConfirmDialog`](file:///c:/Users/nabre/Projects/dev-kit2/ui/src/workbench/dialog.tsx#L37-L60) non intrappola ancora il tasto `Tab`.
4. **Messaggio di Errore `commands/mod.rs:141` Specifico per ARM64**: In [`tauri/src/commands/mod.rs:140-142`](file:///c:/Users/nabre/Projects/dev-kit2/tauri/src/commands/mod.rs#L140-L142), quando `!c.is_available_on(target)` scatta (ad es. se su macOS/Linux venisse richiesto `terminal`), il messaggio dice ancora `"{0} non è disponibile sui PC ARM64."` anziché indicare la piattaforma corrente (`runner.rs:242-246` è invece già stato aggiornato correttamente in `v2`).

---

## 3. Qualità dell'Esperienza e Facilitazione Utente (v1: `91/100` → v2: `91/100` | Eccellente)

Il verdetto sulla facilitazione d'uso si conferma **eccellente (`91 / 100`)**:

| Profilo Utente | Impatto `v1` | Valore Aggiunto in `v2` (`fa8a34d`) |
| :--- | :--- | :--- |
| **1. Studente / Sviluppatore Junior** | Setup completo in ~4 min (-95% tempo rispetto a ~95 min manuali). | Invariato su Windows; su Mac M1/M2/M3/M4 (`darwin-arm64`) e Linux ottiene ora anche R, Python, Node, Java (con estrazione automatica di `Contents/Home`), CMake e TinyTeX pronti all'uso. |
| **2. Full-Stack Multi-Progetto** | Cambio versione istantaneo per progetto (`dev-settings.json`). | [`runtime/bashrc`](file:///c:/Users/nabre/Projects/dev-kit2/runtime/bashrc#L16-L55) applica ora lo switch automatico delle versioni di progetto (`versions.txt`) anche nei terminali Bash nativi di macOS e Linux senza richiedere `cygpath`. |
| **3. Sviluppatore AI-First** | Sync 1-click tra Claude, Codex, Gemini e Antigravity. | [`agents/relocate.rs:369-380`](file:///c:/Users/nabre/Projects/dev-kit2/tauri/src/agents/relocate.rs#L369-L380) riloca ora automaticamente anche i virtualenv Python con struttura POSIX (`bin/`, `lib/python*/site-packages`). |
| **4. Docente / Sviluppatore Nomade (USB/SSD)** | Zero UAC, Temp su host, Rimozione sicura USB. | **Coabitazione Multi-OS reale**: la stessa chiavetta può essere usata in un laboratorio Windows la mattina e su un MacBook il pomeriggio mantenendo `projects/` e `data/home/` condivisi e i binari `data/apps/<target>/` perfettamente isolati. |
