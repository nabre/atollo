# 02 — UX, UI, Interazione e Facilitazione (`v5`)

> **Riferimento Codice**: `HEAD a8aad6c` · [`ui/src/`](../../../ui/src) (128 file `.ts`/`.tsx`, 36.183 righe; 3 fogli CSS, 2.158 righe) · **i18n**: [`ui/src/locales/en.json`](../../../ui/src/locales/en.json) 914 chiavi (765 nel `v4`), `fr`, `de`; italiano nel codice · **Test Frontend**: [`test/ui/`](../../../test/ui) (30 file, 168 `it`/`test`; `npx vitest run` il 20/10/2026: 29 file, **178 superati**).

---

## 1. Struttura del Frontend `ui/src/` (`a8aad6c`)

Conteggio per cartella con `find … -name '*.ts' -o -name '*.tsx'` e `wc -l`. I 18 file della radice (`App.tsx`, `titlebar.tsx`, `activity.tsx`, `terminals.tsx`, `servers.tsx`, `logos.tsx`, `i18n.ts`…) sommano 4.074 righe.

| Area | Cartella | File | Righe | Novità dal `v4` |
|---|---|:---:|:---:|---|
| **Guida in 9 capitoli (`D-136`)** | [`guide/`](../../../ui/src/guide) | 15 | 13.970 | Riscritta: 9 capitoli, 36 argomenti, schermate per lingua. |
| **Progetti** | [`projects/`](../../../ui/src/projects) | 19 | 4.147 | Pannello rapido come prima scheda fissa (D-130). |
| **Bridge IPC** | [`api/`](../../../ui/src/api) | 18 | 2.786 | `rustToolchainSet`, identità Git, `pty`. |
| **Strumenti (componenti)** | [`components/`](../../../ui/src/components) | 13 | 2.568 | Tipi del sito (D-139), matrice per sistema (D-121), pagina di Rust con il compilatore. |
| **Workbench** | [`workbench/`](../../../ui/src/workbench) | 8 | 2.148 | `Watermark` con copertina del marchio, `SettingItem` con barra «modificato». |
| **Wizard** | [`frameworks/`](../../../ui/src/frameworks) | 7 | 1.392 | 63 basi in 10 gruppi (D-131). |
| **Impostazioni** | [`settings/`](../../../ui/src/settings) | 7 | 1.080 | **Nuova cartella**: `autosave.ts`, `page`, `sections`, `firstrun`, `about`, `conda` (D-137). |
| **Database** | [`databases/`](../../../ui/src/databases) | 4 | 1.069 | phpMyAdmin per linea (D-107), stati vuoti con copertina. |
| **Versioni** | [`versions/`](../../../ui/src/versions) | 5 | 954 | |
| **GitHub** | [`github/`](../../../ui/src/github) | 5 | 807 | `identity.tsx`: chi firma i commit. |
| **Agenti** | [`agents/`](../../../ui/src/agents) | 4 | 729 | |
| **Terminale nell'app (`D-109`)** | [`terminal/`](../../../ui/src/terminal) | 5 | 459 | **Nuovo**: xterm.js su macOS e Linux. |

---

## 2. Miglioramenti UX Completati dopo il `v4`

### 2.1 Impostazioni senza «Salva» (`D-137`)

- **Autosave** ([`ui/src/settings/autosave.ts`](../../../ui/src/settings/autosave.ts)): ogni modifica attende 500 ms (`DELAY`, riga 10) il tasto successivo, poi va a `settings_set`, un salvataggio alla volta e sempre con gli ultimi valori. Un valore rifiutato resta a schermo con il suo errore; la modifica seguente riprova. La chiave Gemini si salva quando il campo perde il focus ([`sections.tsx:309`](../../../ui/src/settings/sections.tsx#L309)).
- **Riavvio dell'editor segnalato**: `envChanged` confronta lingua, temporanei e rete; se cambiano, la pagina avvisa che VSCodium già aperto tiene i valori vecchi fino al riavvio (`needsRestart`, [`autosave.ts:15-29`](../../../ui/src/settings/autosave.ts#L15-L29)).
- **Lato backend**: la pagina non sovrascrive «Piattaforma» di Strumenti né le chiavi di una versione più nuova (`Settings::update`, vedi file 01 §2.7).
- **Sezioni** ([`settings/page.tsx:70-82`](../../../ui/src/settings/page.tsx#L70-L82)): Generale, Rete, Agenti, Aggiornamenti, Account GitHub, **Privacy**, Il kit. La sezione «Versioni» non esiste più: le linee di Python e PHP si scelgono in Strumenti e per progetto, il compilatore Rust di Windows (`gnu`/`msvc`) è sulla pagina di Rust in Strumenti ([`ui/src/components/rust.tsx`](../../../ui/src/components/rust.tsx), `rust_toolchain_set`).

### 2.2 Scelte del primo avvio prima di ogni download

[`ui/src/settings/firstrun.tsx`](../../../ui/src/settings/firstrun.tsx) mostra «Prima di iniziare» con lingua, tema, aggiornamento automatico e, solo dove la rete lo richiede, proxy e certificati di un firewall che ispeziona HTTPS. Usa lo stesso `useSettingsAutosave` della pagina Impostazioni; i componenti richiesti aspettano «Installa», quindi i download usano già queste scelte. Il testo ricorda che si ritrovano e si cambiano in Impostazioni.

### 2.3 Privacy, tema e identità Git

| Funzione | Dove | Comportamento |
|---|---|---|
| **Privacy** | [`settings/about.tsx:82-102`](../../../ui/src/settings/about.tsx#L82-L102); informativa [`docs/PRIVACY.md`](../../../docs/PRIVACY.md) + `.it/.fr/.de` | Atollo non ha telemetria né account; elenca le uniche uscite di rete (aggiornamenti dell'app su GitHub se attivi, download dalle fonti ufficiali, versioni disponibili una volta al giorno, estensioni da Open VSX). |
| **Tema** | [`settings/sections.tsx:97-106`](../../../ui/src/settings/sections.tsx#L97-L106) | Sistema / Chiaro / Scuro; applicato a tutte le finestre, editor compreso, con `set_theme` di Tauri, senza riavvio. |
| **Identità Git** | [`ui/src/github/identity.tsx`](../../../ui/src/github/identity.tsx) | Nome ed e-mail dei commit; riempiti una volta dall'account GitHub con l'indirizzo `noreply`. |

### 2.4 Pagina Strumenti per tipi, con blocchi titolati (`D-139`)

- [`ui/src/components/table.tsx`](../../../ui/src/components/table.tsx): un `<tbody>` per tipo (sistema, agenti, linguaggi, strumenti, web, database) con `aria-label` del titolo, il titolo cliccabile che piega il blocco, una casella «Scegli tutti: …» e la descrizione del tipo sotto il titolo ([`table.tsx:446-480`](../../../ui/src/components/table.tsx#L446-L480)). Filtro «Tutti i tipi» (riga 339), colonne ordinabili dentro ogni tipo.
- Gli stessi tipi del sito ([`ui/src/components/model.ts`](../../../ui/src/components/model.ts) ↔ [`site/matrix.mjs`](../../../site/matrix.mjs), test di uguaglianza) per tabella, albero, matrice e primo avvio.

### 2.5 Stati vuoti con la copertina del marchio

`Watermark` ([`workbench/layout.tsx:529-537`](../../../ui/src/workbench/layout.tsx#L529-L537)) mette i passi successivi su `hero-{dark,light}.svg` (classe `bg-hero`), come l'intestazione di benvenuto. Usato in Impostazioni senza scheda ([`settings/page.tsx:366`](../../../ui/src/settings/page.tsx#L366)), tabella di database non scelta ([`databases/browser.tsx:285`](../../../ui/src/databases/browser.tsx#L285)) e server di database fermo ([`databases/page.tsx:331`](../../../ui/src/databases/page.tsx#L331)) (commit `e7186dd`).

### 2.6 Logo di MySQL e test dei loghi

MySQL non aveva logo (`SiMysql`, `#4479A1`, [`ui/src/logos.tsx:180`](../../../ui/src/logos.tsx#L180)). Il nuovo test [`test/ui/logos.test.ts`](../../../test/ui/logos.test.ts) richiede un logo per ogni componente di `components.json` (merge `fd78913`).

### 2.7 Quattro lingue e registri

- Interfaccia: italiano nel codice, `en`/`fr`/`de` in `ui/src/locales` (914 chiavi `en`); backend: [`tauri/i18n/catalog.json`](../../../tauri/i18n/catalog.json) 390 voci (277 nel `v4`). Privacy e README in 4 lingue, `docs/app` in inglese + italiano (D-135).
- Registri: «tu» in italiano, «vous» in francese (D-56, [`DECISIONS.md:366`](../../../docs/app/DECISIONS.md#L366)). Il tedesco, che D-56 voleva con il «Sie», passa al «du» in README e sito (D-135, [`DECISIONS.md:1262`](../../../docs/app/DECISIONS.md#L1262)) e poi nella Guida e nei cataloghi dell'app (D-136, [`DECISIONS.md:1286`](../../../docs/app/DECISIONS.md#L1286)).

---

## 3. Stato dei Rilievi UX del `v4`

| ID | Stato | Evidenza `a8aad6c` |
|---|---|---|
| `UX-V4-01` Barra di stato e Attività senza pool e linee MySQL secondarie | **Aperto** → `UX-V5-01` | `live` conta solo le esecuzioni ([`App.tsx:104`](../../../ui/src/App.tsx#L104), 505, 532); `liveServices` guarda solo lo stato della linea principale ([`activity.tsx:49`](../../../ui/src/activity.tsx#L49)), anche se `ServiceInfo.others` porta lo stato delle altre ([`db/services/mod.rs:445-455`](../../../tauri/src/db/services/mod.rs#L445-L455)); nessun pool nel pannello; nessuna azione «Ferma tutto». |
| `UX-V4-02` Siti sganciati, nome azione | **Risolto** | `AttachedSite.action`, `server_name`, `detached` ([`servers/mod.rs:78-95`](../../../tauri/src/servers/mod.rs#L78-L95)); [`servers.tsx:160-193`](../../../ui/src/servers.tsx#L160-L193) (commit `b58fdd7`). |
| `UX-V4-03` «Apache dedicato», dominio statico | **Risolto** | Nome dal pool (`serverName`) con ripiego `<label>.localhost` ([`projects/site.tsx:16`](../../../ui/src/projects/site.tsx#L16), 61). |
| `UX-V4-04` `DbVersionSelect` nascosto | **Risolto** | `versions.lines.length < 2 && missing === null` ([`projects/databases.tsx:133`](../../../ui/src/projects/databases.tsx#L133); commit `04dfb88`). |
| `UX-V4-05` `Project.lines` non esposto | **Risolto** | `lines?: Record<string, string>` ([`api/projects.ts:50`](../../../ui/src/api/projects.ts#L50); commit `09e489a`). |
| `UX-V4-06` Conferme e `onCancel` | **Parzialmente** → `UX-V5-02` | I DB si cancellano ora con il piano di eliminazione del progetto ([`api/projects.ts:217-253`](../../../ui/src/api/projects.ts#L217-L253)); restano `InlineConfirm` senza `onCancel` ([`terminals.tsx:258`](../../../ui/src/terminals.tsx#L258)) e «Elimina profilo» del pool senza conferma ([`servers.tsx:137`](../../../ui/src/servers.tsx#L137), 524-528). |

Bilancio UX: 4 risolti, 1 parziale, 1 aperto su 6.

## 4. Nuovi Rilievi UX (`UX-V5-xx`)

### `UX-V5-01` (Media, residuo di `UX-V4-01`) — Pool web e linee MySQL secondarie invisibili nell'attività
- **Dove**: [`ui/src/App.tsx:104`](../../../ui/src/App.tsx#L104); [`ui/src/activity.tsx:49`](../../../ui/src/activity.tsx#L49).
- **Problema**: pool web attivi e linee MySQL secondarie in esecuzione non entrano né nel conteggio della barra di stato né nel pannello Attività; con la linea principale ferma, una secondaria attiva non si vede da nessuna parte.
- **Rimedio**: includere `s.others.filter(o => o.state !== "stopped")` e i pool attivi (`serversList`) in `liveServices` e nel conteggio.

### `UX-V5-02` (Bassa, residuo di `UX-V4-06`) — Escape e conferma distruttiva
- **Dove**: [`ui/src/terminals.tsx:258`](../../../ui/src/terminals.tsx#L258); [`ui/src/servers.tsx:137`](../../../ui/src/servers.tsx#L137), [`524-528`](../../../ui/src/servers.tsx#L524-L528).
- **Problema**: Escape non chiude la conferma di eliminazione del terminale; un profilo di pool si elimina al primo clic.
- **Rimedio**: `onCancel={() => setConfirmRemove(false)}`; `InlineConfirm` prima di `serverRemove`.

### Rilievi collegati in altri file
- `BUG-V5-01` (file 03): la casella «Rete locale» di Impostazioni ([`sections.tsx:247`](../../../ui/src/settings/sections.tsx#L247)) promette i server dei progetti in LAN, ma le azioni `php -S` rispondono 421 ai dispositivi della rete. Per l'utente è una funzione che non funziona senza alcun messaggio.

---

## 5. Allineamento al workbench di VSCodium

L'interfaccia di Atollo imita il workbench «modern UI» di VSCodium e prende i colori dai temi Dark Modern / Light Modern (`theme-colors.css` e `theme-vscode.css` generati da [`docs/brand/themes.py`](../../../docs/brand/themes.py)). Il confronto con il CSS del workbench di **VSCodium 1.135** (sorgente `src-1.135.06055`, estratto in 21 parti `00..20-*.css`) e con `ui/src/theme/*.css` e `ui/src/workbench/*` ha trovato **15 differenze**. Le fasi 0-5 qui sotto sono state **approvate dall'utente**: sono lavoro pianificato, non rilievi aperti, e non entrano nel conteggio dei rilievi né nel punteggio.

### 5.1 Le 15 differenze

Selettori e misure di VSCodium citati come `parti/NN-*.css:riga` dell'estratto; file di Atollo su `a8aad6c`.

| # | Parte | VSCodium 1.135 (selettore / misura) | Atollo (file:riga) | Differenza |
|:-:|---|---|---|---|
| 1 | Schede del pannello | `.part.panel>.title … .action-item.checked .active-item-indicator:before` con `border-top-color: var(--vscode-panelTitle-activeBorder)` (08-panel.css:21, 57-59): indicatore a tutta larghezza della scheda | `Panel`: `after:inset-x-2 after:bottom-1.5 after:h-px` ([`workbench/layout.tsx:576-578`](../../../ui/src/workbench/layout.tsx#L576-L578)) | L'analisi indicava una pillola. Alla verifica su `a8aad6c` il pannello ha già una sottolineatura, ma rientrata di 8 px e alzata di 6 px rispetto all'indicatore di VSCodium; la pillola resta nelle schede dell'editor (#10). |
| 2 | Barra delle attività | `--activity-bar-width` 48 px (03-activitybar.css:6-7); colori `--vscode-modernActivityBar-{active,hover}{Background,Foreground}` (03-activitybar.css:160-162) | Pulsante `size-9` con riquadro `size-8 rounded` e segno sinistro di 3 px `rounded-full` ([`App.tsx:345-358`](../../../ui/src/App.tsx#L345-L358)); chiavi `modernActivityBarItem-*` e `modernActivityBar-border` ([`theme-colors.css:54-56`](../../../ui/src/theme/theme-colors.css#L54-L56), [`theme-vscode.css:465-469`](../../../ui/src/theme/theme-vscode.css#L465-L469)) | Forma, bordo e spaziatura diversi; le chiavi colore usate da Atollo non sono quelle che VSCodium 1.135 legge. |
| 3 | Righe di liste e alberi | `.monaco-list-row { width:100% }`, righe di 22 px (05-list-tree.css:6, 102-103, 199) | `h-[22px] … rounded pr-1` ([`workbench/list.tsx:88`](../../../ui/src/workbench/list.tsx#L88)), righe da 62 px `rounded pr-3 pl-4` (riga 158) | In Atollo arrotondate e rientrate, in VSCodium a tutta larghezza e squadrate. |
| 4 | Intestazioni dei pannelli laterali | `.pane>.pane-header { height: var(--pane-header-size); font-size:11px; font-weight:700 }` (04-sidebar-pane.css:65); hover modern `background-color: var(--vscode-list-hoverBackground)` (04-sidebar-pane.css:521) | `ViewPane`: intestazione 28 px `h-7 rounded … hover:bg-vsc-list-hover` ([`workbench/layout.tsx:211-242`](../../../ui/src/workbench/layout.tsx#L211-L242)) | Hover e raggio diversi. |
| 5 | Barre di scorrimento | `.monaco-scrollable-element>.scrollbar>.slider` (13-scrollbar.css:12) | Pseudo-elementi `::-webkit-scrollbar*` ([`theme/theme.css:85-96`](../../../ui/src/theme/theme.css#L85-L96)); i token hanno solo i colori ([`theme-colors.css:117-119`](../../../ui/src/theme/theme-colors.css#L117-L119)) | Barre del browser colorate, non le barre personalizzate del workbench. |
| 6 | Pulsanti | `.monaco-text-button { padding:4px 8px; border-radius:4px; line-height:16px }`, `.small { line-height:14px; font-size:11px; padding:3px 6px }` (12-controls.css, `key-rules2.txt`) | `<vscode-button>`; variante piccola `h-[18px] px-[5px] py-0 text-[11px] leading-4` ([`workbench/controls.tsx:160-164`](../../../ui/src/workbench/controls.tsx#L160-L164)) | Dimensioni diverse. |
| 7 | Intestazione delle estensioni | `.extension-editor>.header { padding:20px 0 12px 20px }`, icona 128 × 128 px (17-extension-editor.css:6, 8-9), barra `.navbar` (riga 71) | Icona `size-16` (64 px) ([`workbench/layout.tsx:408`](../../../ui/src/workbench/layout.tsx#L408)) | 64 px contro 128 px, senza la barra di navigazione. |
| 8 | Editor delle impostazioni | `.settings-toc-container` (10-settings-editor.css:52-54); `.setting-item-modified-indicator` 2 px, `left:5px; top:15px; bottom:18px` (10-settings-editor.css:73-74) | `SettingsEditor` con `toc` ([`workbench/settings.tsx:21-69`](../../../ui/src/workbench/settings.tsx#L21-L69)), `SettingItem modified` ([`settings.tsx:136-150`](../../../ui/src/workbench/settings.tsx#L136-L150)) | Indice, titoli e barra del valore modificato non coincidono. |
| 9 | Barra del titolo | Pulsanti finestra `width:46px` (02-titlebar.css:96); icona dell'app `line-height:30px` (02-titlebar.css:83); area di sovrapposizione `env(titlebar-area-height, 35px)` (riga 87-88) | Barra `h-[35px]` ([`titlebar.tsx:63`](../../../ui/src/titlebar.tsx#L63)); pulsanti finestra `w-11` = 44 px ([`titlebar.tsx:59`](../../../ui/src/titlebar.tsx#L59), [`106`](../../../ui/src/titlebar.tsx#L106)) | 44 px contro 46 px dei pulsanti finestra; sovrapposizione 35 px contro 30 px. |
| 10 | Schede dell'editor | `.tab { background-color: var(--vscode-tab-inactiveBackground) }` (06-editor-tabs.css:71); `.tab-actions-right` (riga 84) | `TAB`: pillola di 24 px rientrata di 2 px, chiusura al passaggio e sulla scheda attiva ([`workbench/layout.tsx:288-297`](../../../ui/src/workbench/layout.tsx#L288-L297)) | Colore della scheda inattiva e posizione della chiusura. |
| 11 | Finestre di dialogo | `.monaco-dialog-box { min-width:480px; padding:8px; border-radius: var(--vscode-cornerRadius-xLarge); box-shadow: var(--vscode-shadow-xl) }` (16-dialog.css:3) | `rounded … shadow-[0_0_8px_2px_var(--vsc-shadow)]`, larghezza `min(32rem,90vw)` ([`workbench/dialog.tsx:121`](../../../ui/src/workbench/dialog.tsx#L121)) | Raggio e ombra diversi. |
| 12 | Barra di stato | `.statusbar-item-label { margin:0 3px; padding:0 5px; height:100% }`, `line-height:22px` (09-statusbar.css:14, 20) | `flex h-6 … rounded px-1.5` ([`App.tsx:457`](../../../ui/src/App.tsx#L457)) | Padding e raggio degli elementi. |
| 13 | Campi e menu a tendina | `.monaco-select-box { border-radius: var(--vscode-cornerRadius-small) }`, in barra `padding:2px 23px 2px 8px` (12-controls.css:1, 4) | `TextField`, `Select` ([`workbench/controls.tsx:304`](../../../ui/src/workbench/controls.tsx#L304), [`493`](../../../ui/src/workbench/controls.tsx#L493)) | Misure di input e select diverse. |
| 14 | Icone | Font `codicon` (`@font-face … codicon.ttf`, 18-codicon.css:21) | `react-icons/vsc` importato in 64 file di `ui/src` | Stesse forme, ma SVG di React invece del font: dimensioni e allineamento ottici diversi. |
| 15 | Token di dimensione | Variabili di misura (`--vscode-spacing-size*`, `--vscode-cornerRadius-*`, `--pane-header-size`, `--activity-bar-width`) | [`theme-colors.css`](../../../ui/src/theme/theme-colors.css) ha solo token di colore | Mancano i token di altezza di righe, schede e intestazioni: ogni componente usa misure scritte a mano. |

### 5.2 Piano approvato (fasi 0-5)

| Fase | Contenuto | Differenze chiuse | File toccati (previsti) |
|:-:|---|:-:|---|
| **P0 — Fondamenta** | Token di misura (altezze, spaziature, raggi) accanto ai token di colore; chiavi colore `modernActivityBar-{active,hover}{Background,Foreground}` come in VSCodium 1.135; script di *vendoring* che estrae dal sorgente di VSCodium le regole usate e ne conserva la licenza MIT. | #15, #2 (colori) | `ui/src/theme/*`, `docs/brand/themes.py`, nuovo script in `scripts/` |
| **P1 — Cornice** | Barra del titolo, barra delle attività, barra di stato, schede del pannello. | #9, #2, #12, #1 | `titlebar.tsx`, `App.tsx`, `workbench/layout.tsx` |
| **P2 — Barra laterale** | Barra laterale, pannelli (pane header), righe di liste e alberi. | #4, #3 | `workbench/layout.tsx`, `workbench/list.tsx` |
| **P3 — Editor** | Schede dell'editor, intestazione delle estensioni. | #10, #7 | `workbench/layout.tsx` |
| **P4 — Controlli** | Pulsanti, campi e menu a tendina, barre di scorrimento, codicons. | #6, #13, #5, #14 | `workbench/controls.tsx`, `theme/theme.css`, 64 file con `react-icons/vsc` |
| **P5 — Superfici** | Editor delle impostazioni, finestre di dialogo, notifiche, quick input. | #8, #11 | `workbench/settings.tsx`, `workbench/dialog.tsx`; riferimenti 15-notifications.css, 11-quick-input.css |

L'ordine segue le dipendenze: P0 dà i token che le fasi successive usano, P1-P2 cambiano ciò che si vede in ogni pagina, P3-P5 parti più circoscritte. Le schermate della Guida (`ui/public/guide/{it,en,fr,de}`, rigenerate da [`scripts/guide-shots/make.mjs`](../../../scripts/guide-shots/make.mjs)) andranno rifatte alla fine delle fasi che cambiano la cornice.
