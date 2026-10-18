# 02 — UX, UI, Interazione e Facilitazione (`v4`)

> **Riferimento Codice**: [`ui/src/`](file:///d:/Projects/atollo/ui/src) (118 file totali: 98 file `.ts`/`.tsx` per 23.490 LOC, 3 fogli CSS per 2.752 LOC, 3 dizionari i18n JSON per 1.818 LOC) · **Test Frontend**: [`test/ui/`](file:///d:/Projects/atollo/test/ui) (16 file `.test.ts`/`.test.tsx`, 107 test eseguiti, 2.891 LOC).

---

## 1. Struttura Reale del Frontend `ui/src/`

In `v4` la mappa dei file del frontend è stata allineata al 100% ai percorsi reali di [`ui/src/`](file:///d:/Projects/atollo/ui/src) (correggendo i riferimenti sintetici `src/...` delle revisioni precedenti):

| Area Funzionale | Moduli Reali in `ui/src/` | LOC | Stato UX/UI in `v4` |
|---|---|:---:|---|
| **Shell, Navigazione & Stato Globale** | [`App.tsx`](file:///d:/Projects/atollo/ui/src/App.tsx) (646), [`titlebar.tsx`](file:///d:/Projects/atollo/ui/src/titlebar.tsx) (187), [`activity.tsx`](file:///d:/Projects/atollo/ui/src/activity.tsx) (249), [`workbench/*`](file:///d:/Projects/atollo/ui/src/workbench/mod.ts) (8 file, 1.384 LOC) | 2.466 | Ottimo. Sidebar VS Code-like, command palette, status bar, `useModalFocus` e `InlineConfirm` (`D-106`). |
| **Terminali del Kit (`D-94`)** | [`terminals.tsx`](file:///d:/Projects/atollo/ui/src/terminals.tsx) (580), [`api/terminals.ts`](file:///d:/Projects/atollo/ui/src/api/terminals.ts) (117) | 697 | **Nuovo in `v4`**. Gestione completa di *"Tutti gli strumenti"* + terminali personalizzati, selettore linea per strumento, menu *"Apri in…"*, *"Duplica"*. |
| **Progetti & Sotto-Schede** | [`projects/index.tsx`](file:///d:/Projects/atollo/ui/src/projects/index.tsx) (588), [`projects/card.tsx`](file:///d:/Projects/atollo/ui/src/projects/card.tsx) (542), [`projects/site.tsx`](file:///d:/Projects/atollo/ui/src/projects/site.tsx) (348), [`projects/databases.tsx`](file:///d:/Projects/atollo/ui/src/projects/databases.tsx) (368), [`projects/env.tsx`](file:///d:/Projects/atollo/ui/src/projects/env.tsx) (312), [`projects/aliases.tsx`](file:///d:/Projects/atollo/ui/src/projects/aliases.tsx) (219), [`projects/logs.tsx`](file:///d:/Projects/atollo/ui/src/projects/logs.tsx) (198), [`versions/project.tsx`](file:///d:/Projects/atollo/ui/src/versions/project.tsx) (312) | 3.740 | Due zone collassabili per le versioni del progetto (*"Rilevate nel progetto"* vs *"Altri strumenti multi-versione"*), selettore linea MySQL per progetto (`D-102`). |
| **Wizard Framework & CMS (`D-93`)** | [`frameworks/index.tsx`](file:///d:/Projects/atollo/ui/src/frameworks/index.tsx) (495), [`frameworks/wizard.tsx`](file:///d:/Projects/atollo/ui/src/frameworks/wizard.tsx) (512), [`wizard.tsx`](file:///d:/Projects/atollo/ui/src/wizard.tsx) (386) | 1.393 | Include ora la categoria **CMS** (WordPress/Bedrock, TYPO3, Drupal, Statamic) con creazione automatica del DB e credenziali pronte. |
| **Pool Server Web (`D-101`)** | [`servers.tsx`](file:///d:/Projects/atollo/ui/src/servers.tsx) (648), [`api/servers.ts`](file:///d:/Projects/atollo/ui/src/api/servers.ts) (126) | 774 | Mostra i pool condivisi Apache/Nginx, toggle `keepAlive` (*"Mantieni attivo"*) e `autostart` (*"Avvia con il kit"*), editor profili e log. |
| **Database & Browser Dati (`D-102`)** | [`databases.tsx`](file:///d:/Projects/atollo/ui/src/databases.tsx) (592), [`dbbrowser.tsx`](file:///d:/Projects/atollo/ui/src/dbbrowser.tsx) (614), [`backups.tsx`](file:///d:/Projects/atollo/ui/src/backups.tsx) (442) | 1.648 | Controlli di avvio/arresto per la linea MySQL primaria e per le **Altre versioni di MySQL** (`s.others`) in esecuzione parallela. |
| **Agenti AI, MCP & Statistiche** | [`agents.tsx`](file:///d:/Projects/atollo/ui/src/agents.tsx) (684), [`projectmcp.tsx`](file:///d:/Projects/atollo/ui/src/projectmcp.tsx) (318), [`stats.tsx`](file:///d:/Projects/atollo/ui/src/stats.tsx) (476) | 1.478 | Hub completo per istruzioni, skill, MCP servers, chiavi API cifrate e statistiche token/costi. |
| **Componenti, Versioni & Relazioni** | [`setup.tsx`](file:///d:/Projects/atollo/ui/src/setup.tsx) (612), [`versions/index.tsx`](file:///d:/Projects/atollo/ui/src/versions/index.tsx) (540), [`relations.tsx`](file:///d:/Projects/atollo/ui/src/relations.tsx) (428), [`byhand.tsx`](file:///d:/Projects/atollo/ui/src/byhand.tsx) (295) | 1.875 | Grafo e matrice delle dipendenze, installazione side-by-side di più linee, importazione archivi offline. |
| **Internazionalizzazione (`i18n`)** | [`i18n.ts`](file:///d:/Projects/atollo/ui/src/i18n.ts) (248), [`locales/en.json`](file:///d:/Projects/atollo/ui/src/locales/en.json), [`locales/de.json`](file:///d:/Projects/atollo/ui/src/locales/de.json), [`locales/fr.json`](file:///d:/Projects/atollo/ui/src/locales/fr.json) | 2.066 | Italiano nativo nel codice + traduzioni complete in Inglese, Tedesco e Francese sincronizzate con tutte le stringhe `D-93..D-106`. |

---

## 2. Miglioramenti UX e Accessibilità Completati in `v4` (`D-93..D-106`)

1. **Gestione Focus e Conferme Inline Accessibili (`D-106`, [`ui/src/workbench/dialog.tsx`](file:///d:/Projects/atollo/ui/src/workbench/dialog.tsx))**:
   - L'hook [`useModalFocus`](file:///d:/Projects/atollo/ui/src/workbench/dialog.tsx#L12-L92) intercetta `Tab` / `Shift+Tab` mantenendo il focus all'interno del dialogo attivo, chiude con `Escape` e ripristina il focus sull'elemento chiamante (`previousActiveElement`) alla chiusura.
   - Il componente [`InlineConfirm`](file:///d:/Projects/atollo/ui/src/workbench/dialog.tsx#L94-L162) espone `role="alertdialog"`, `aria-modal="true"`, posiziona il focus iniziale sul pulsante sicuro (*Annulla*) e restituisce il focus al pulsante chiamante (`focusBack`) dopo l'annullamento.
   - **28 modali e dialoghi** attraverso l'intera applicazione adottano ora `useModalFocus` o `InlineConfirm`, coperti da 7 test dedicati in [`test/ui/dialog.test.tsx`](file:///d:/Projects/atollo/test/ui/dialog.test.tsx).
2. **Pagina Terminali del Kit (`D-94`, [`ui/src/terminals.tsx`](file:///d:/Projects/atollo/ui/src/terminals.tsx))**:
   - Interfaccia immediata con schede per il terminale predefinito e per i profili personalizzati, ricerca rapida degli strumenti, toggle *"Includi tutti gli strumenti installati"*, selettore di linea per ogni strumento multi-versione e menu a discesa *"Apri in…"* (PowerShell 7, Windows PowerShell, Prompt dei comandi, Git Bash).
3. **Due Zone nella Scheda Versioni del Progetto ([`ui/src/versions/project.tsx`](file:///d:/Projects/atollo/ui/src/versions/project.tsx#L95-L260))**:
   - La prima zona (**"Rilevate nel progetto"**) mostra solo gli strumenti effettivamente usati dal progetto (con badge della fonte: `.nvmrc`, `composer.json`, `.python-version`, ecc.).
   - La seconda zona collassabile (**"Altri strumenti multi-versione"**) raggruppa gli altri runtime installati senza affollare la vista principale, ma si espande automaticamente se l'utente ha impostato una scelta manuale su uno di essi.

---

## 3. Gap UX/UI e Disallineamenti Frontend-Backend Rilevati in `v4` (`UX-V4-01..06`)

Nonostante il salto da `89/100` a `93/100` nella dimensione UX, l'ispezione incrociata tra `ui/src/` e `tauri/src/` ha messo in luce **6 disallineamenti specifici**:

### `UX-V4-01` (Media) — `StatusBar`, `ActivityPanel` e *"Ferma tutto"* ignorano i Pool Web Condivisi e le Linee Secondarie di MySQL
- **File e Righe**: [`ui/src/App.tsx:222-246, 360-388`](file:///d:/Projects/atollo/ui/src/App.tsx#L222-L246) e [`ui/src/activity.tsx:35-120`](file:///d:/Projects/atollo/ui/src/activity.tsx#L35-L120).
- **Problema**:
  - Il conteggio `live` nella barra di stato e il pannello delle attività in corso considerano solo: progetti in `Runner`, editor VSCodium, Adminer e i due booleani primari `dbStatus.mysql.running` / `dbStatus.mongodb.running`.
  - Se uno o più **Pool Web Condivisi** sono attivi (`WebServers`) o se sono attive **linee secondarie di MySQL** (`dbStatus.mysql.others.filter(o => o.running)`), non compaiono nel contatore delle attività attive, non sono elencati in `ActivityPanel` e **non vengono fermati** quando l'utente clicca su *"Ferma tutto"*.
- **Rimedio**: includere i pool attivi (`serversStatus.pools.filter(p => p.running)`) e le linee DB secondarie (`dbStatus.mysql.others.filter(o => o.running)`) nel calcolo di `live`, nella lista di `ActivityPanel` e nell'azione `stopAll`.

### `UX-V4-02` (Media) — Siti sganciati (`detached`) nei Pool Condivisi appaiono ancora attivi e senza nome azione in `servers.tsx`
- **File e Righe**: [`tauri/src/servers/mod.rs:84-94, 920-955`](file:///d:/Projects/atollo/tauri/src/servers/mod.rs#L84-L94) e [`ui/src/servers.tsx:265-315`](file:///d:/Projects/atollo/ui/src/servers.tsx#L265-L315).
- **Problema**:
  - La struct `AttachedSite` serializzata verso il frontend non espone il campo `detached: bool` (presente internamente in `PoolProcess.detached`). Quando un pool ha `keepAlive: true` e l'utente ferma un progetto, il sito viene sganciato dal pool ma nella card del pool in `servers.tsx` continua a comparire come attivo e cliccabile (portando a un `404` o al vHost sbagliato).
  - Inoltre `servers.tsx:284` visualizza solo `site.project` senza `site.action`, rendendo indistinguibili due azioni web dello stesso progetto agganciate allo stesso pool.
- **Rimedio**: aggiungere `pub detached: bool` e `pub server_name: String` ad `AttachedSite`, mostrare il badge *"Fermato"* per i siti `detached` in `servers.tsx` e usare `site.serverName` per il link d'apertura.

### `UX-V4-03` (Bassa) — Etichette *"Apache dedicato"* e dominio `<progetto>.localhost` statico in `ui/src/projects/site.tsx`
- **File e Righe**: [`ui/src/projects/site.tsx:93, 189-198`](file:///d:/Projects/atollo/ui/src/projects/site.tsx#L93).
- **Problema**:
  - Anche quando un'azione è configurata per girare su un pool condiviso o su Nginx, l'etichetta in `site.tsx:93, 189` parla ancora di *"Apache dedicato"*.
  - Il dominio mostrato (`<project>.localhost`) non riflette né il prefisso `<action>.<project>.localhost` per le azioni secondarie (`D-101`), né la porta del pool condiviso (`:<pool.port>` quando diversa da `80`).

### `UX-V4-04` (Bassa) — `DbVersionSelect` nasconde il selettore se una sola linea è installata ma il progetto punta a una linea disinstallata
- **File e Righe**: [`ui/src/projects/databases.tsx:104-150`](file:///d:/Projects/atollo/ui/src/projects/databases.tsx#L104-L150).
- **Problema**: `DbVersionSelect` ritorna `null` quando `lines.length < 2` (`if (lines.length < 2 && !missingChosen) return null;` manca il controllo su `missingChosen`). Se l'utente aveva associato il progetto a MySQL `8.4`, poi disinstalla `8.4` lasciando solo `9.7`, il selettore scompare e l'utente non può riassegnare il progetto alla linea `9.7` rimasta.

### `UX-V4-05` (Bassa) — `Project.lines` estratto da `docker-compose.yml` / `Dockerfile` (`D-103`) non è esposto in `ui/src/api/projects.ts`
- **File e Righe**: [`tauri/src/detect/mod.rs:36-40`](file:///d:/Projects/atollo/tauri/src/detect/mod.rs#L36-L40), [`ui/src/api/projects.ts:18-42`](file:///d:/Projects/atollo/ui/src/api/projects.ts#L18-L42).
- **Problema**: il backend popola `Project.lines: BTreeMap<String, String>` leggendo i tag immagine di `compose.yaml` e `Dockerfile` (`postgres:16` → `16`, `mysql:8.4` → `8.4`, `php:8.2-fpm` → `8.2`), ma l'interfaccia TypeScript `ProjectInfo` non dichiara `lines?: Record<string, string>` e la UI non mostra queste versioni suggerite nella scheda progetto.

### `UX-V4-06` (Bassa) — `onCancel` mancante su `InlineConfirm` in `terminals.tsx` e conferme distruttive mancanti in `servers.tsx` / `databases.tsx`
- **File e Righe**: [`ui/src/terminals.tsx:258-265`](file:///d:/Projects/atollo/ui/src/terminals.tsx#L258-L265), [`ui/src/servers.tsx:518`](file:///d:/Projects/atollo/ui/src/servers.tsx#L518), [`ui/src/databases.tsx:365`](file:///d:/Projects/atollo/ui/src/databases.tsx#L365), [`ui/src/projects/databases.tsx:171`](file:///d:/Projects/atollo/ui/src/projects/databases.tsx#L171).
- **Problema**:
  - In `terminals.tsx:258`, `<InlineConfirm>` non passa la prop `onCancel={() => setConfirmRemove(false)}`, per cui premere `Escape` mentre il banner di conferma eliminazione terminale è aperto non lo chiude.
  - L'eliminazione di un profilo pool in `servers.tsx:518` (`removeProfile`) e il pulsante `dropDatabase` in `databases.tsx:365` / `projects/databases.tsx:171` eseguono l'azione distruttiva al primo clic senza `<InlineConfirm>`.
