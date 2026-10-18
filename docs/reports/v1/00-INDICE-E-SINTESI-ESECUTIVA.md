# Rapporto di Verifica Tecnica Globale — Atollo v0.5.3

> [!IMPORTANT]
> **Ruolo del Verificatore**: Audit indipendente end-to-end (Architettura, Logica, UX, UI, Esperienza d'Uso, Completezza Funzionale, Identificazione Bug & Sicurezza, Readiness Multipiattaforma, Facilitazione e Qualità dell'Interazione) condotto sull'intero repository [`atollo`](../../..) (`570+` file tra backend Rust/Tauri 2, frontend React 19/TypeScript, runtime shell e suite di test).

![Radar Valutazione 9 Dimensioni](assets/radar-9-dimensioni.svg)

---

## 1. Struttura del Dossier di Verifica (`reports/`)

Per garantire massima leggibilità, tracciabilità puntuale (`file:linea`) e azionabilità ingegneristica, l'audit è organizzato in **6 documenti specialistici** più una **Dashboard Visuale Interattiva**:

| Documento | Oggetto e Dimensioni Coperte | Link Diretto |
| :--- | :--- | :--- |
| **00 — Indice e Sintesi Esecutiva** | Panoramica generale, Scorecard globale sulle 9 dimensioni, verdetto sintetico e roadmap prioritaria. | [00-INDICE-E-SINTESI-ESECUTIVA.md](../../../reports/00-INDICE-E-SINTESI-ESECUTIVA.md) |
| **01 — Architettura e Logica** | Verifica approfondita dell'architettura stratificata Rust/Tauri + React, sicurezza IPC, gestione processi/Job Objects, layout dati `D-89`/`D-90` e logica di dominio. | [01-ARCHITETTURA-E-LOGICA.md](../../../reports/01-ARCHITETTURA-E-LOGICA.md) |
| **02 — UX, UI, Esperienza d'Uso e Facilitazione** | Analisi critica del Design System VSCodium/Atollo, ergonomia cognitiva, accessibilità, internazionalizzazione (IT/EN/FR/DE) e indice di facilitazione utente (*Time-to-Code*). | [02-UX-UI-INTERAZIONE-FACILITAZIONE.md](../../../reports/02-UX-UI-INTERAZIONE-FACILITAZIONE.md) |
| **03 — Bug, Sicurezza e Funzionalità Mancanti** | Catalogo completo di **21 bug/anomalie** (con severità, root cause e fix), vulnerabilità edge-case e funzionalità incompiute rispetto a specifiche e aspettative. | [03-BUG-SICUREZZA-E-FUNZIONALITA-MANCANTI.md](../../../reports/03-BUG-SICUREZZA-E-FUNZIONALITA-MANCANTI.md) |
| **04 — Promozione Multipiattaforma** | Audit puntuale della portabilità verso **macOS** e **Linux**: cosa è già pronto (~57%), cosa va adattato (~21%), cosa è vincolato a Win32 (~22%) e piano di refactoring in 4 fasi. | [04-PROMOZIONE-MULTIPIATTAFORMA.md](../../../reports/04-PROMOZIONE-MULTIPIATTAFORMA.md) |
| **05 — Indicatori, Metriche e Grafici (Complementare)** | Artefatto complementare dedicato ai criteri di valutazione, KPI quantitativi, rubriche di punteggio, grafici Mermaid (`xychart-beta`, `flowchart`, `stateDiagram-v2`) e infografiche SVG. | [05-INDICATORI-METRICHE-E-GRAFICI.md](../../../reports/05-INDICATORI-METRICHE-E-GRAFICI.md) |
| **Dashboard Interattiva HTML** | Esploratore visuale interattivo con grafici radar, filtri per severità bug, barre di readiness multipiattaforma e calcolatore KPI. | [dashboard-valutazione.html](../../../reports/dashboard-valutazione.html) |

---

## 2. Scorecard Sintetica sulle 9 Dimensioni di Verifica

| # | Dimensione di Verifica | Voto (/100) | Livello | Sintesi del Giudizio Tecnico |
| :-: | :--- | :-: | :---: | :--- |
| **1** | **Architettura** | **92 / 100** | Eccellente | Separazione netta a 4 livelli ([`ARCHITECTURE.md`](../../app/ARCHITECTURE.md)), singolo binario GUI+CLI ([`main.rs`](../../../tauri/src/main.rs)), isolamento IPC rigoroso (`ensure_main` + [`build.rs`](../../../tauri/build.rs)), zero `unwrap`/`expect` in produzione, eccellente riforma dei dati `D-89`/`D-90`. |
| **2** | **Logica** | **84 / 100** | Molto Buono | Logica di rilocazione percorsi, risoluzione semver ([`ranges.rs`](../../../tauri/src/versions/ranges.rs)), matrice relazioni ([`relations/`](../../../tauri/src/versions/relations/mod.rs)) e sync agenti IA solida; penalizzata da race condition nei pool web server ([`servers/mod.rs`](../../../tauri/src/servers/mod.rs)) e scrittura atomica basata solo su PID ([`fsx.rs:67`](../../../tauri/src/fsx.rs#L67)). |
| **3** | **UX (User Experience)** | **86 / 100** | Molto Buono | Flussi di onboarding ([`wizard.tsx`](../../../ui/src/wizard.tsx)), creazione guidata framework ([`projects/new.tsx`](../../../ui/src/projects/new.tsx)) e rimozione sicura USB ([`eject.tsx`](../../../ui/src/eject.tsx)) esemplari; migliorabile la visibilità degli errori di lettura silenziosi e il collegamento tra "Avvia progetto" e i pool server. |
| **4** | **UI (User Interface)** | **90 / 100** | Eccellente | Design System VSCodium-native coerente ([`workbench/`](../../../ui/src/workbench/layout.tsx)), 4 temi ad alto contrasto verificati da test automatici ([`theme.test.ts`](../../../test/ui/theme.test.ts)), icone vettoriali nitide. Lievi difetti: `capitalize` CSS su titoli IT/FR e micro-tipografia `<12px` in alcuni badge. |
| **5** | **Esperienza di Utilizzo** | **88 / 100** | Molto Buono | Avvio rapido, zero sporcizia lasciata sul PC ospite (neanche nel Credential Manager grazie a [`keyring.rs`](../../../tauri/src/agents/keyring.rs)), terminale portabile già pre-configurato con PATH e temi coordinati ([`wterm.rs`](../../../tauri/src/editor/wterm.rs)). |
| **6** | **Funzionalità Mancanti (Completezza)** | **78 / 100** | Buono / Discreto | Manca il montaggio in UI di [`ProjectMcpBox`](../../../ui/src/projectmcp.tsx) (codice già scritto ma orfano!), l'integrazione del pulsante "Avvia" progetto con i Pool Server Condivisi (`WEB-1..8`), il multi-versioning concorrente dei DB (es. MySQL 8.4 + 9.7 insieme) e il rilevamento CMS esteso (Drupal/PrestaShop/Joomla). |
| **7** | **Identificazione dei Bug (Robustezza)** | **76 / 100** | Discreto | Nessun crash catastrofico grazie al divieto di `unwrap()`, ma **21 bug puntuali** individuati: 5 ad alta priorità (tra cui avvio pool Apache/Nginx che riporta successo anche se il processo muore subito, bypass D-77 in `serve.rs`, race su `write_atomic`), 9 medi e 7 bassi. |
| **8** | **Promozione Multipiattaforma** | **57 / 100** | Parziale | Il **57%** del progetto (intera UI React, IPC Tauri, rilevamento progetti, semver, archivi, script `bashrc`/`agy`/`gradle`) è **già multipiattaforma**, ma il **43%** (layer `os/`, `keyring.rs`, `wterm.rs`, `webview.rs`, URL catalogo e binari `.exe` in `install/`) è cablato su Windows/Win32 senza `#[cfg(unix)]`. |
| **9** | **Qualità Esperienza & Facilitazione** | **91 / 100** | Eccellente | **Riduce del 95% il *Time-to-Code*** (da ~95 min a ~4 min per uno stack completo con SSL, DB, VSCodium e Agenti IA). Lo strumento facilita genuinamente sia lo studente/junior sia lo sviluppatore full-stack nomade. |
| — | **MEDIA PONDERATA GLOBALE** | **82.4 / 100** | **Solido (Grado A-)** | Prodotto ingegnerizzato con cura rara nel panorama desktop Tauri/Rust; risolvendo i 5 bug ad alta priorità e completando l'integrazione dei Pool Server raggiunge la piena maturità di produzione su Windows, con una base chiara per lo sbarco su macOS/Linux. |

---

## 3. Verdetto Esecutivo: L'Utente è Facilitato nell'Impiego dello Strumento?

**Sì, in misura netta e misurabile (91/100).** Atollo risolve con successo tre problemi storicamente ostici nello sviluppo software locale:

1. **Zero Configurazione Ambientale ("It Just Works")**: L'utente non deve modificare il `PATH` di Windows, non deve installare servizi di sistema globali che confliggono tra progetti, non deve generare manualmente certificati TLS (`mkcert` è automatizzato in [`tls.rs`](../../../tauri/src/projects/tls.rs)) e non deve configurare manualmente `php.ini` o i VirtualHost di Apache/Nginx.
2. **Isolamento Multi-Versione Trasparente**: Un progetto legacy su PHP 8.1 / Node 20 e uno moderno su PHP 8.4 / Node 24 coesistono nella stessa cartella `projects\`, ciascuno con il proprio [`dev-settings.json`](../../../tauri/src/projects/settings.rs) e con verifica automatica delle incompatibilità nella pagina **Relazioni** ([`relations.tsx`](../../../ui/src/relations.tsx)).
3. **Orchestrazione Unificata degli Agenti IA**: La sincronizzazione unificata di istruzioni (`CLAUDE.md` / `AGENTS.md` / `GEMINI.md`), Skills e server MCP tra Claude Code, OpenAI Codex, Gemini CLI e Antigravity ([`agents/sync/`](../../../tauri/src/agents/sync/mod.rs)), unita alla cattura effimera del login Google nel Credential Manager ([`agents/keyring.rs`](../../../tauri/src/agents/keyring.rs)), offre un'esperienza d'uso che nessun concorrente (Laragon, Herd, XAMPP, Devbox) possiede oggi.

---

## 4. Top 10 Interventi Prioritari Raccomandati (Quick Wins & Strutturali)

1. **[P0 - Bug/Sicurezza]** Sostituire `fs::canonicalize` con [`os::safe_path`](../../../tauri/src/os/safe_path.rs) nel server HTTP statico ([`tauri/src/projects/serve.rs:215`](../../../tauri/src/projects/serve.rs#L215)) e imporre un limite al numero di thread/dimensione header per rispettare la decisione `D-77` (anti-NTLM leak e anti-DoS).
2. **[P0 - Bug Concorrenza]** Rendere [`fsx::write_atomic`](../../../tauri/src/fsx.rs#L67-L73) thread-safe intra-processo aggiungendo un contatore atomico (`AtomicU64`) al suffisso `.tmp-<pid>-<seq>` e preservando l'estensione originale del file.
3. **[P0 - Bug Server Pool]** In [`tauri/src/servers/mod.rs:648-740`](../../../tauri/src/servers/mod.rs#L648-L740), acquisire il lock di avvio per-pool, controllare `child.try_wait()` durante il loop di attesa della porta e restituire `Err(...)` (leggendo il log degli errori) se `httpd.exe` o `nginx.exe` terminano prematuramente o non aprono la porta entro 6 secondi.
4. **[P1 - Feature Pronta ma Nascosta]** Montare il componente [`ProjectMcpBox`](../../../ui/src/projectmcp.tsx#L99-L172) all'interno di [`ui/src/projects/page.tsx`](../../../ui/src/projects/page.tsx) (tab Ambiente o nuova tab MCP), riattivando immediatamente la gestione MCP per-progetto già supportata dal backend.
5. **[P1 - Performance & I/O]** Rimuovere gli effetti collaterali di scrittura su disco (`save_file`) e la scansione pesante `detect::detect` da [`WebServers::list()`](../../../tauri/src/servers/mod.rs#L337-L512), che viene invocata ogni 3 secondi dal polling di [`ui/src/servers.tsx`](../../../ui/src/servers.tsx).
6. **[P1 - Integrazione Pool Server]** Collegare il pulsante **Avvia / Ferma** della pagina Progetto ([`tauri/src/projects/run.rs`](../../../tauri/src/projects/run.rs)) ai Pool Server Condivisi (`WebServers`) per i progetti Apache/Nginx, come previsto da [`PLAN.md §13.1`](../../app/PLAN.md#L211) e [`PLAN-DATA.md §9.1`](../../app/PLAN-DATA.md#L569).
7. **[P1 - Race Frontend]** Aggiungere un contatore di sequenza / `AbortController` a [`useLoad`](../../../ui/src/hooks.ts#L92-L107) e correggere la sovrascrittura dello stato in [`useRuns`](../../../ui/src/projects/runs.ts#L17-L27) e la chiusura stale in [`Sash` / `useStoredSize`](../../../ui/src/workbench/layout.tsx#L57-L104).
8. **[P2 - Allineamento Path UI/Guida e i18n]** Aggiornare i path obsoleti `data\app\projects` in [`ui/src/projects/site.tsx:191`](../../../ui/src/projects/site.tsx#L191) e `data\backup\` in [`ui/src/guide/widgets.tsx:304`](../../../ui/src/guide/widgets.tsx#L304), e completare le traduzioni FR/DE nella Guida ([`ui/src/guide/sections.tsx`](../../../ui/src/guide/sections.tsx)).
9. **[P2 - Accessibilità & Tipografia]** Aggiungere il focus trap in [`ConfirmDialog`](../../../ui/src/workbench/dialog.tsx#L37-L60), rimuovere la classe CSS `capitalize` dai titoli in [`ui/src/workbench/layout.tsx:165,218,535`](../../../ui/src/workbench/layout.tsx#L165) e portare i micro-testi `text-[9px]`/`text-[10px]` ad almeno `11px–12px`.
10. **[P2 - Architettura Multipiattaforma]** Avviare la Fase 1 del piano multipiattaforma ([`04-PROMOZIONE-MULTIPIATTAFORMA.md`](../../../reports/04-PROMOZIONE-MULTIPIATTAFORMA.md)): spostare `windows`, `webview2-com` e `junction` sotto `[target.'cfg(windows)'.dependencies]` in [`tauri/Cargo.toml`](../../../tauri/Cargo.toml) e introdurre il layer `os::unix` per processi, lock, permessi `0700` e keyring.
