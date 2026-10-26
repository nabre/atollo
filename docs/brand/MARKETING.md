# ATOLLO — Manuale Operativo di Marketing, Brand e Posizionamento (Guida per Agenti AI)

> **Scopo di questo documento:** Questo file è la fonte di verità unica (*Single Source of Truth*) per qualsiasi agente AI o collaboratore incaricato di creare, aggiornare o verificare testi promozionali, landing page, documentazione pubblica (`README`), materiale di lancio, asset grafici, temi UI e micro-copy del prodotto **Atollo**.

---

## 1. Direttive Operative per l'Agente

Quando ti viene chiesto di lavorare su qualsiasi aspetto di comunicazione, marketing, design o rebranding del progetto, **attieniti rigorosamente a queste regole**:

1. **Non alterare mai la definizione di categoria:** Il posizionamento ufficiale è **"Universal Portable Dev & AI Workspace"**. Non ridurlo mai a "utility per Windows", "alternativa a" un altro prodotto o "semplice launcher di agenti".
2. **Nessun confronto con altri prodotti:** Non nominare né paragonare altri strumenti, servizi o sistemi (niente tabelle "noi contro loro", niente "meglio di…", niente "l'isolamento di X senza Y"). Atollo si presenta con le sue funzionalità e il suo modo di lavorare: cosa fa, come lo fa, cosa garantisce. Alla domanda «in cosa è diverso?» si risponde con gli aspetti della **Sezione 4**.
3. **Coerenza cromatica assoluta con `vscode-defaults`:** Ogni pagina web, presentazione, grafico SVG, documento HTML o componente UI deve utilizzare esclusivamente la tavolozza definita nella **Sezione 5**, derivata da [`docs/brand/vscode-defaults/dark.json`](vscode-defaults/dark.json), [`docs/brand/vscode-defaults/light.json`](vscode-defaults/light.json), [`docs/brand/themes.py`](themes.py) e [`docs/brand/make.py`](make.py).
4. **Tono di voce (*Voice & Tone*):**
   * **Sobrio, ingegneristico, concreto, mai gonfiato:** Evita superlativi vuoti (*"rivoluzionario"*, *"magico"*, *"il migliore al mondo"*). Gli sviluppatori si convincono con i fatti tecnici (*zero installazione, zero diritti di amministratore, una microVM di Docker Sandboxes per progetto configurata dal kit, 4 sistemi nativi (quelli di Docker Sandboxes), isolamento del PATH, sincronizzazione di 4 agenti AI: Claude Code, Codex, Antigravity, Pi*).
   * **Evocativo ma funzionale:** La metafora marina dell'**Atollo** (laguna protetta, isola autosufficiente in qualsiasi oceano/OS) serve a rendere immediato il concetto di isolamento e portabilità, senza mai appesantire l'interfaccia tecnica con gergo nautico forzato.
5. **Parità multilingua obbligatoria (`IT`, `EN`, `FR`, `DE`):** Ogni testo rivolto all'utente finale va mantenuto allineato nelle quattro lingue supportate dal progetto.
6. **Trasparenza tecnica:** Dichiara sempre con onestà cosa gira nativo (`✓`), cosa manca (`✕`) e quali sono i limiti fisici dei sistemi operativi ospiti (come documentato in [`docs/app/CATALOG.md`](../app/CATALOG.md), “Availability per target” e “Sources per system”; in italiano [`CATALOG.it.md`](../app/CATALOG.it.md), «Disponibilità per target» e «Fonti per sistema»). Per le sandbox dichiara anche dove la microVM c'è (Windows 11 x64 con Windows Hypervisor Platform, Mac Apple silicon con macOS 14+, Ubuntu 24.04+ con KVM), che serve un account Docker (`sbx login`), che l'hypervisor lo attiva l'utente una volta (mai Atollo) e quali tracce restano fuori dalla cartella (DECISIONS.md D-175). Dove la VM non c'è, i progetti girano sul PC e l'app lo dice («Senza VM»): mai promettere un isolamento che quel PC non dà.

---

## 2. Identità del Brand e Posizionamento Ufficiale

### 2.1 Scheda Sintetica

| Attributo | Valore Ufficiale |
| :--- | :--- |
| **Nome del prodotto** | **Atollo** |
| **Comando CLI / Binario** | `atollo` (Windows: `Atollo.exe`, macOS: `Atollo.app`, Linux: `Atollo.AppImage`) |
| **Categoria di mercato** | **Universal Portable Dev & AI Workspace** |
| **Target Piattaforme (`4/4`)** | `win32-x64`, `darwin-arm64`, `linux-x64`, `linux-arm64`: i sistemi su cui gira Docker Sandboxes, il cuore di Atollo (D-176) |
| **Licenza** | MIT |

### 2.2 Definizione Canonica (Da usare come base ovunque)

* **Italiano:**
  > **Atollo — Universal Portable Dev & AI Workspace.**
  > Un ambiente di sviluppo autonomo, multi-OS e multi-architettura (*Windows x64, macOS Apple Silicon, Linux x64/ARM64*) che unisce **Editor**, **Toolchain/Runtime Manager**, **Server/Database locali** e **Orchestratore di Agenti AI** in una singola cartella portabile e senza installazione. Ogni progetto gira nella sua **microVM di Docker Sandboxes**, configurata da Atollo e tenuta nella cartella.
* **English:**
  > **Atollo — Universal Portable Dev & AI Workspace.**
  > A self-contained, multi-OS and multi-architecture development environment (*Windows x64, macOS Apple Silicon, Linux x64/ARM64*) that unifies an **Editor**, **Toolchain/Runtime Manager**, **Local Servers/Databases**, and an **AI Agent Orchestrator** into a single portable, zero-install folder. Each project runs in its own **Docker Sandboxes microVM**, configured by Atollo and kept in the folder.
* **Français:**
  > **Atollo — Universal Portable Dev & AI Workspace.**
  > Un environnement de développement autonome, multi-OS et multi-architecture (*Windows x64, macOS Apple Silicon, Linux x64/ARM64*) qui réunit **Éditeur**, **Gestionnaire de langages et d'outils**, **Serveurs/Bases de données locales** et **Orchestrateur d'agents IA** dans un seul dossier portable sans installation. Chaque projet s'exécute dans sa propre **microVM Docker Sandboxes**, configurée par Atollo et conservée dans le dossier.
* **Deutsch:**
  > **Atollo — Universal Portable Dev & AI Workspace.**
  > Eine eigenständige Multi-OS- und Multi-Architektur-Entwicklungsumgebung (*Windows x64, macOS Apple Silicon, Linux x64/ARM64*), die **Editor**, **Toolchain-/Runtime-Manager**, **lokale Server/Datenbanken** und **KI-Agenten-Orchestrierung** in einem einzigen portablen Ordner ohne Installation vereint. Jedes Projekt läuft in seiner eigenen **Docker-Sandboxes-microVM**, von Atollo konfiguriert und im Ordner aufbewahrt.

---

## 3. Narrativa e Architettura del Valore (I 4 Pilastri + 1 Fondamento)

### 3.1 La Metafora del Brand ("Perché Atollo")
In geografia, un **atollo** è un anello corallino che si innalza in mezzo all'oceano aperto e racchiude al suo interno una **laguna calma, fertile e completamente protetta** dalle correnti esterne.
Nel mondo del software:
* **L'oceano esterno** è il computer ospite: può essere un PC Windows scolastico o aziendale blindato senza permessi di amministratore, un Mac Apple Silicon a casa, una workstation Linux, o un sistema pieno di versioni globali in conflitto.
* **La laguna interna è Atollo:** un perimetro isolato e autosufficiente (`projects/` + `data/`) dove il tuo editor, i tuoi 20+ linguaggi, i tuoi database, i tuoi server web, il tuo account GitHub e i tuoi 4 agenti AI sono già configurati, dialogano tra loro e non lasciano alcuna traccia sul sistema ospite quando smonti l'unità (salvo le poche tracce di Docker Sandboxes che il kit dichiara).
* **Ogni progetto è un'isola della laguna:** la sua microVM, che Atollo configura e tiene nella cartella. Il codice del progetto lavora lì dentro; il resto del PC e gli altri progetti restano fuori.

### 3.2 I 4 Pilastri Funzionali + Il Fondamento Multi-OS

Ogni presentazione completa di Atollo deve articolarsi su questi **5 punti**:

1. **Progetti in Sandbox, Editor e Terminali (Workstation in-a-Box)** — il nucleo
   * **Configuratore portatile di Docker Sandboxes:** ogni progetto ha la sua microVM (kernel suo, rete negata salvo le regole). Terminale, azioni (`npm run dev`…), i loro siti, i terminali e i task dell'editor e gli agenti (Claude Code e Codex nelle loro sandbox, Pi nella VM del progetto) girano lì, con la cartella del progetto in scrittura e gli strumenti del kit per Linux, alle versioni scelte dal progetto, in sola lettura: scaricati una volta nel kit e verificati, portabili, senza rete, immutabili per la VM. I terminali del kit restano sul PC, senza i CLI degli agenti; Antigravity, un editor desktop, non parte mai su un progetto. `sbx` stesso è nel kit: sandbox, dischi, impostazioni e registri viaggiano con la cartella.
   * **Confini chiari:** i server di database restano sul PC e la VM ne raggiunge solo la porta, come utente del progetto; le chiavi non entrano mai nella VM (GitHub solo se il progetto lo sceglie, aggiunto dal proxy di `sbx`).
   * **Detto, mai finto:** dove la microVM non c'è (sistema non supportato, hypervisor spento, nessun accesso a Docker) terminale, azioni e siti del progetto girano sul PC con l'ambiente sigillato del kit, i terminali dell'editor e gli agenti non partono, e la barra di stato dice «Senza VM», con il motivo e cosa fare (i passi da amministratore con il comando da copiare, mai lanciati da Atollo).
   * Editor VSCodium integrato in finestre dedicate con tema nativo Atollo Dark/Light.
   * Terminali dedicati e profili personalizzabili che vedono esclusivamente il `PATH` e le variabili d'ambiente del kit: i programmi installati sul PC ospite non interferiscono e il PC ospite non viene mai "sporcato".
2. **Visual Toolchain & Runtime Manager (Poliglotta e On-Demand)**
   * Più di 20 linguaggi, compilatori e strumenti (Node.js, Python, uv, PHP, Rust, Go, JDK/Temurin, .NET, Dart, Zig, Julia, Ruby, C/C++, Bun, Deno, Typst, TinyTeX, R, Android SDK…) scaricati solo quando servono, verificati tramite checksum SHA-256 o firma digitale.
   * Gestione di versioni *side-by-side* per singolo progetto o terminale, con verifica automatica della matrice delle relazioni e compatibilità.
3. **Local Server & Database Stack (Zero Servizi di Sistema)**
   * Motori database locali (MySQL, MongoDB, SQLite) avviabili e arrestabili con un clic, con creazione automatica di database, utenti e credenziali `.env` isolati per ciascun progetto, backup integrati e phpMyAdmin pronto all'uso.
   * Pool di server web condivisi o dedicati per progetto e scaffolding guidato dei principali framework.
4. **AI Agent Hub & Orchestrator (4 Agenti, 1 Sola Mente)**
   * Installazione, accesso e avvio diretto di **Claude Code**, **Codex**, **Antigravity** e **Pi**.
   * **Sincronizzazione unificata (`sync`):** scrivi una sola volta istruzioni, skill, regole e server MCP in `data/agents` (o per singolo progetto) e Atollo li mantiene allineati automaticamente per tutti e quattro gli agenti.
   * **Modelli AI locali:** modelli GGUF (dal catalogo di Atollo o da Hugging Face) che girano sul PC con llama.cpp, senza rete e senza account; download che riprendono, verificati con lo SHA-256; il server ascolta solo su `127.0.0.1` con una chiave e Pi lo usa senza altre impostazioni. Una sola copia in `data/models` per Windows, macOS e Linux.
   * **Controllo dei consumi:** monitoraggio in tempo reale dei limiti del piano e dei consumi giornalieri e degli ultimi 7 giorni.
5. **Il Fondamento: Continuità Fisica Multi-OS (`6/6`) e Rimozione Sicura**
   * Grazie al Router multipiattaforma (`win32-x64`, `darwin-arm64`, `linux-x64`, `linux-arm64`, i sistemi di Docker Sandboxes), la stessa cartella o lo stesso SSD esterno può passare da Windows a macOS a Linux mantenendo intatti progetti, database, credenziali e configurazioni AI, eseguendo su ogni macchina i binari nativi del sistema ospite (la matrice in [`docs/app/TOOLS-MATRIX.html`](../app/TOOLS-MATRIX.html) dice cosa c'è dove).
   * La funzione **Rimuovi l'unità** chiude tutti i processi figli, mette in sicurezza le sessioni e le credenziali nel kit e permette di espellere il disco senza rischi.

---

## 4. Aspetti Distintivi del Programma ("Come funziona")

Alla domanda «in cosa è diverso?» non si risponde con un confronto (Direttiva 2) ma con questi quattro aspetti di Atollo, che descrivono **come lavora** il programma. Sono anche la sezione «Come funziona» del sito (`site/strings/*.json`, chiavi `aspectsTitle` e `a1`…`a4`). **Da allineare:** `a1Title`/`a1Text` del sito dicono ancora «Isolated, with native programs: no containers and no virtual machines», superato da D-167…D-175; vanno riscritti nelle quattro lingue come la prima riga qui sotto.

| Aspetto | Cosa dire | Fatti da citare |
| :--- | :--- | :--- |
| **Ogni progetto nella sua sandbox** | Il codice di un progetto gira nella sua microVM, che Atollo configura e tiene nella cartella; il resto del PC e gli altri progetti restano fuori. | Docker Sandboxes (`sbx`) nel kit, una microVM per progetto (terminale, azioni, siti, terminali dell'editor e agenti), gli strumenti del kit per Linux in sola lettura, il database sul PC raggiunto solo sulla sua porta, le chiavi mai nella VM; nessun diritto di amministratore per Atollo, nessun servizio di sistema, registro e `PATH` del sistema mai toccati; dove la VM non c'è, «Senza VM» nella barra di stato con il motivo; avvio anche da chiavetta USB o SSD esterno. |
| **Tutto da una finestra** | Ogni area del lavoro ha la sua pagina, e ciò che succede si vede in un posto solo. | Pagine **Progetti**, **AI** (agenti e modelli locali), **Database**, **Strumenti**, **Impostazioni**; pannello **Attività** con output ed errori; riga di comando per gli script (`atollo check`, `atollo prepare`, `atollo sync`). |
| **La cartella viaggia con te** | Cambiano unità, cartella o sistema: il lavoro resta lo stesso. | Percorsi riallineati al cambio di lettera o cartella; su ognuno dei 4 target i programmi di quel sistema; progetti, database, accessi, configurazioni degli agenti e il kit di ogni sandbox intatti (l'accesso a Docker si rifà su ogni PC con `sbx login`). |
| **Ogni componente verificato** | Nulla si usa senza essere controllato, e le versioni non si pestano i piedi. | Checksum SHA-256 o firma su ogni download; versioni *side-by-side* per progetto; matrice delle relazioni che avvisa delle incompatibilità. |

---

## 5. Sistema Colore e Tavolozza Ufficiale (Basata su `vscode-defaults`)

La tavolozza di **Atollo** non è arbitraria: nasce dall'unione rigorosa dei valori standard di **VS Code / VSCodium ("Dark Modern" e "Light Modern")** archiviati in [`docs/brand/vscode-defaults/dark.json`](vscode-defaults/dark.json) e [`docs/brand/vscode-defaults/light.json`](vscode-defaults/light.json) con gli accenti di brand definiti in [`docs/brand/themes.py`](themes.py) e [`docs/brand/make.py`](make.py).

I colori del brand riflettono naturalmente gli elementi fisici di un **Atollo oceanico**: il verde abissale della laguna profonda (*Forest*), lo smeraldo vivo della barriera (*Emerald*), la menta delle acque basse (*Mint*) e il corallo/ambra vivo (*Coral Orange & Sun Amber*).

### 5.1 Colori Primari del Brand (Accenti e Identità)

| Nome Semantico Atollo | Costante (`make.py` / `themes.py`) | Codice HEX | Ruolo nel Brand e nella UI |
| :--- | :--- | :--- | :--- |
| **Deep Lagoon (Forest)** | `FOREST` | `#0F3D2E` | Colore primario profondo, wordmark su sfondo chiaro, badge *remote* della statusbar (`statusBarItem.remoteBackground`), testo su superfici Mint. |
| **Abyssal Tile Gradient** | `APP_TILE_DEFS` | `#14503C` → `#082419` | Gradiente diagonale dello sfondo dell'icona app (`symbol.svg`) e hover del badge remoto (`#14503C`). |
| **Deep Trench Ink** | — (`themes.py`) | `#04140D` | Testo ad alto contrasto (WCAG > 7:1) sopra i pulsanti e la statusbar `#10B981` in tema scuro. |
| **Atoll Emerald (Dark Accent)** | `EMERALD` | `#10B981` | **Accento primario nel Tema Scuro**: `button.background`, `statusBar.background`, `focusBorder`, `progressBar.background`, `tab.activeBorderTop`, `activityBarBadge.background`; al 20% (`#10B98133`) la voce attiva della barra delle attività, `modernActivityBar.activeBackground` (la chiave che legge VSCodium 1.135, D-142; `activityBar.activeBorder` resta solo per la barra classica dell'editor). |
| **Atoll Emerald Hover (Dark)** | — (`themes.py`) | `#0EA572` | Hover dei pulsanti primari nel Tema Scuro (`button.hoverBackground`). |
| **Reef Emerald (Light Accent)** | — (`themes.py`) | `#047857` | **Accento primario nel Tema Chiaro** (più scuro per garantire contrasto WCAG ≥ 4.5:1 con testo bianco `#FFFFFF`): pulsanti, statusbar, focus, link e bordi attivi. |
| **Reef Emerald Hover (Light)** | — (`themes.py`) | `#065F46` | Hover dei pulsanti primari nel Tema Chiaro (`button.hoverBackground`). |
| **Surf Glow (Highlight)** | — (`make.py` / `themes.py`) | `#34D399` | Link e testo evidenziato su tema scuro (`textLink.foreground`, `list.highlightForeground`), bordo tab selezionato su tema chiaro. |
| **Crest Mint** | — (`themes.py`) | `#6EE7B7` | Bordo superiore del tab selezionato in tema scuro (`tab.selectedBorderTop`). |
| **Shallow Mint (Soft Surface)** | `MINT` | `#D1F0E0` | Superficie morbida di accento, `inputOption.activeBackground` e `modernActivityBar.activeBackground` (la voce attiva della barra delle attività) in tema chiaro, testo su `FOREST`. |
| **Coral Orange (Dark Action)** | `ORANGE` | `#F97316` | **Accento secondario / Attenzione (Tema Scuro):** `charts.orange`, `statusBar.debuggingBackground`, isolotto a sud-est del simbolo e della `O` finale del wordmark, vertice caldo del simbolo. |
| **Deep Coral (Light Action)** | — (`themes.py`) | `#C2410C` | **Accento secondario / Attenzione (Tema Chiaro):** `charts.orange`, `statusBar.debuggingBackground` (contrasto pieno con `#FFFFFF`). |
| **Sun Amber** | `AMBER` | `#FBBF24` | Transizione centrale del gradiente caldo nel simbolo e negli sfondi hero (`#10B981` → `#FBBF24` → `#F97316`). |

### 5.2 Superfici, Testi e Bordi Neutri (Da `docs/brand/vscode-defaults`)

Per garantire che il sito web, la documentazione HTML, i componenti UI React (`--vsc-*`) e l'editor VSCodium integrato siano **visivamente indistinguibili**, usa sempre i neutri di `vscode-defaults`:

| Token App (`--vsc-*`) | Variabile `vscode-defaults` | **Tema Scuro (`dark.json`)** | **Tema Chiaro (`light.json`)** | Utilizzo |
| :--- | :--- | :--- | :--- | :--- |
| `--vsc-bg` | `--vscode-editor-background` | `#1F1F1F` | `#FFFFFF` | Sfondo principale dell'area di lavoro / pagina |
| `--vsc-sidebar` / `--vsc-panel` | `--vscode-sideBar-background` | `#181818` | `#F8F8F8` | Barra laterale, Activity Bar, pannello inferiore Attività, header/footer |
| `--vsc-widget` / `--vsc-quickinput` | `--vscode-editorWidget-background` | `#202020` | `#F8F8F8` | Card flottanti, modali, menu a tendina |
| `--vsc-input` | `--vscode-input-background` | `#313131` | `#FFFFFF` | Campi di input, caselle di ricerca, pulsanti secondari (dark) |
| `--vsc-button2` | `--vscode-button-secondaryBackground` | `#313131` (hover `#3C3C3C`) | `#E5E5E5` (hover `#CCCCCC`) | Pulsanti secondari |
| `--vsc-border` | `--vscode-sideBar-border` | `#2B2B2B` | `#E5E5E5` | Bordi strutturali di pannelli, tabelle e card |
| `--vsc-input-border` | `--vscode-input-border` | `#3C3C3C` | `#CECECE` | Bordi di campi input, checkbox e controlli |
| `--vsc-fg` | `--vscode-foreground` | `#CCCCCC` | `#3B3B3B` | Testo principale del corpo |
| `--vsc-titlebar-fg` | `--vscode-titleBar-activeForeground` | `#CCCCCC` (icone `#D7D7D7`) | `#1E1E1E` / `#1F1F1F` | Titoli principali ad alto contrasto |
| `--vsc-desc` | `--vscode-descriptionForeground` | `#9D9D9D` | `#717171` (inactive `#616161`) | Sottotitoli, descrizioni secondarie, metadati |
| `--vsc-list-hover` | `--vscode-list-hoverBackground` | `#2A2D2E` | `#F2F2F2` / `#E8E8E8` | Hover sulle righe di tabella o liste |
| `--vsc-list-active` | `--vscode-list-activeSelectionBackground` | `#0B3D2C` | `#0060C0` / `#D1F0E0` | Riga selezionata attiva |
| `--vsc-selection` | `--vscode-editor-selectionBackground` | `#10B9814D` (`rgba(16,185,129,0.3)`) | `#A7E3C6` | Selezione del testo |
| `--vsc-code-block` | `--vscode-textCodeBlock-background` | `#0A0A0A` / `#181818` | `#F8F8F8` | Blocchi di codice e terminale |

### 5.3 Colori Semantici di Stato e Grafici (Da `docs/brand/vscode-defaults` + `themes.py`)

| Stato / Grafico | Variabile `vscode-defaults` | **Tema Scuro (`dark.json`)** | **Tema Chiaro (`light.json`)** | Significato in Atollo |
| :--- | :--- | :--- | :--- | :--- |
| **Success / Nativo (`✓`)** | `--vscode-gitDecoration-untrackedResourceForeground` / `charts-green` | `#73C991` / `#89D185` | `#007100` / `#388A34` | Componente installato, test superato, target nativo pronto |
| **Warning / Da sistemare (`!`)** | `--vscode-list-warningForeground` / `charts-yellow` | `#CCA700` | `#855F00` / `#BF8803` | Avviso non bloccante, rilocazione richiesta, attenzione |
| **Action / Scelta richiesta** | `--vscode-charts-orange` | `#F97316` | `#C2410C` | Azione dell'utente richiesta, debug attivo, consumo elevato |
| **Error / Blocco (`✕`)** | `--vscode-list-errorForeground` / `charts-red` | `#F88070` / `#F14C4C` | `#B01011` / `#E51400` | Errore critico, target incompatibile, arresto anomalo |
| **Info / Catalogo (`○`)** | `--vscode-editorInfo-foreground` / `charts-blue` | `#3794FF` | `#1A85FF` | Informazione, build disponibile upstream da collegare |
| **Substitute / Special (`⇄`)** | `--vscode-charts-purple` | `#B180D7` | `#652D90` | Sostituto consigliato (es. Caddy, MySQL), funzionalità avanzata AI |

### 5.4 Aspetto dell'Interfaccia (VSCodium 1.135)

L'app ha l'aspetto del workbench dell'editor che il kit installa, **VSCodium 1.135** con la UI moderna a pannelli flottanti (DECISIONS.md D-142): chi apre Atollo e poi l'editor non vede un cambio di stile.

* **Misure dal workbench, mai inventate:** barra del titolo di 30 px, barra delle attività di 36 px, schede dell'editor di 24 px in una barra di 32, righe delle liste rientrate di 4 px con raggio 4, schede del pannello a pillola, dialoghi con raggio 12. Vengono da [`vscodium-1.135/workbench-ref.css`](vscodium-1.135/workbench-ref.css) (estratto da `scripts/vscodium-css/extract.mjs`) e diventano i token di `ui/src/theme/theme-sizes.css`.
* **Icone:** solo i **codicon** di VSCodium, dal loro font (`@vscode/codicons`); i loghi di marchi e linguaggi (Simple Icons) restano solo dove si parla di quel prodotto.
* **Colori delle voci della barra delle attività:** le chiavi `modernActivityBar.*` di VSCodium 1.135, nell'app come nel tema dell'editor.
* **Nei materiali promozionali** l'interfaccia si mostra sempre con schermate vere dell'app (Sezione 9), mai con mockup ridisegnati: un mockup che non segue queste misure smentisce la promessa «stesso aspetto dell'editor».

---

## 6. Tipografia e Geometria Visiva

1. **Font UI (Interfaccia e Titoli): Geist** (variabile, pesi 100–900, licenza OFL-1.1).
   * Stack: `"Geist", system-ui, -apple-system, "Segoe UI", sans-serif` (nell'app il nome del pacchetto viene prima: `"Geist Variable", "Geist", system-ui, "Segoe UI", sans-serif`).
   * Dimensione base UI: `13px` (nell'app desktop, come da `vscode-defaults`) · `14px–16px` (nei documenti web e landing page).
   * Pesi: `400` testo, `600` titoletti e voci attive, `700` titoli grandi (mai `800`: Geist è più largo e più scuro di Segoe UI). Titoli grandi con spaziatura stretta: da `-0.035em` (H2) a `-0.045em` (H1 della hero).
2. **Font Monospace (Codice, CLI, Percorsi, Versioni, Badge, Terminale, Editor): Geist Mono** (variabile, pesi 100–900, OFL-1.1).
   * Stack: `"Geist Mono", ui-monospace, "SF Mono", Consolas, monospace` (nell'app: `"Geist Mono Variable", "Geist Mono", ui-monospace, Consolas, monospace`; nell'editor integrato: `'Geist Mono', ui-monospace, Consolas, 'Courier New', monospace`).
   * Dimensione base Editor/Code: `14px` (`11px–12px` per badge tecnici, SHA, identificatori di target come `win32-x64` o `darwin-arm64`). Legature spente.
   * Etichette in maiuscolo (kicker, eyebrow): peso `500`, spaziatura `0.06em–0.1em`.
3. **Sempre in locale, mai da CDN:** l'app è portabile e offline (CSP `'self'`), il sito non carica Google Fonts né altri servizi.
   * App: pacchetti npm `@fontsource-variable/geist` e `@fontsource-variable/geist-mono` (`ui/src/main.tsx`).
   * Sito: `site/assets/fonts/` (latin, latin-ext e, per Geist Mono, i caratteri di riquadro degli alberi di cartelle), `@font-face` con `font-display: swap`, preload del solo Geist latin.
   * Editor integrato (VSCodium): gli stessi file del sito, inclusi nell'eseguibile e registrati dallo script della finestra con l'API `FontFace` (`tauri/src/editor/window.rs`); `editor.fontFamily` e `terminal.integrated.fontFamily` in `Machine\settings.json` (`tauri/src/editor/cfg.rs`), solo se l'utente non ne ha già scelto uno.
4. **Logo nelle intestazioni:**
   * In header, barre di navigazione e firme si usa il wordmark (`wordmark-dark.svg` / `wordmark-light.svg`). Non esiste un lockup separato: la `O` finale di ATOLLO **è** il simbolo intero (l'atollo visto dall'alto: barriera ad anello, tre isolotti, laguna turchese e le due onde, che scendono sotto la linea di base), quindi il wordmark non va mai affiancato al simbolo. La `A` resta una lettera piena, senza accenti.
   * Su sfondo chiaro barriera e onde della `O` sono smeraldo, su sfondo scuro menta. Il file è 514 × 129: le maiuscole occupano 100, le O sporgono di 1,5 sopra e sotto come ogni lettera tonda, le onde scendono sotto la linea di base per i restanti 27,5. La O finale è disegnata nelle unità delle lettere, non rimpicciolita dall'icona: isolotti spessi quanto un'asta, barriera a un terzo d'asta perché la O si legga chiusa.
   * Il simbolo da solo resta per favicon, icona dell'app e immagini hero.
5. **Uso dei Tag Tecnici:**
   * Tutti i nomi di target (`win32-x64`, `darwin-arm64`, `linux-x64`…), i percorsi (`projects/`, `data/apps/`), i comandi CLI (`atollo check`, `atollo sync`) e i formati di archivio (`.tar.gz`, `.zip`, `.AppImage`) vanno sempre resi in **monospace** con bordo sottile `1px solid var(--vsc-border)`.

---

## 7. Architettura dei Messaggi per Profilo Utente (Persona Matrix)

Quando generi copy per una campagna, una sezione del sito o un post social, scegli l'angolo corrispondente al pubblico:

### Persona 1: Lo Sviluppatore "AI-First"
* **Dolore:** Usa Claude Code, Codex, Antigravity e Pi, ma deve mantenere 4 cartelle di configurazione diverse, 4 file di istruzioni, 4 elenchi di server MCP e non sa mai quanto budget/token sta consumando su ciascun fornitore; per il codice riservato vorrebbe un modello che non esca dal PC.
* **Hook:** *"Quattro agenti AI. Una sola mente. Sul tuo disco."*
* **Funzionalità da evidenziare:** Pagina **AI**, riquadro **Agenti**, comando `atollo sync`, allineamento automatico di istruzioni/skill/MCP in `data/agents`, grafico dei consumi giornalieri e settimanali, salvataggio delle credenziali nel kit; Claude Code e Codex nelle sandbox di agente di Docker Sandboxes e Pi nella VM del progetto, lontani dal resto del PC; nella stessa pagina il riquadro **Modelli locali** (Pi su un modello che gira sul PC, senza rete).

### Persona 2: Lo Sviluppatore Full-Stack / Poliglotta
* **Dolore:** Ha il PC lento e il `PATH` inquinato da 4 versioni di Node, 3 di Python, Java, PHP, servizi di MySQL e MongoDB che partono all'avvio di Windows/macOS, e ogni `npm install` o `composer install` di un repository clonato gira con il suo utente, accanto ai suoi file e alle sue chiavi.
* **Hook:** *"Ogni progetto nella sua sandbox. Zero servizi nel sistema."*
* **Funzionalità da evidenziare:** Pagine **Progetti**, **Database** e **Strumenti**, la microVM di ogni progetto con gli strumenti del kit alle versioni del progetto, il database sul PC raggiunto dalla VM solo sulla sua porta, versioni *side-by-side* per progetto, matrice delle relazioni (con la colonna della VM), avvio e arresto istantaneo di DB e web server solo quando servono.

### Persona 3: Lo Studente, Docente o Consulente Nomade
* **Dolore:** Lavora su PC scolastici, universitari, di laboratorio o aziendali dove non ha i permessi di amministratore, oppure si sposta continuamente tra un fisso Windows, un MacBook e una macchina Linux.
* **Hook:** *"Il tuo intero studio di sviluppo in tasca. Su Windows, macOS e Linux, senza permessi di amministratore."*
* **Funzionalità da evidenziare:** Esecuzione da chiavetta USB o SSD esterno, rilocazione automatica dei percorsi al cambio di lettera/cartella, account GitHub unico per tutto il kit, pulsante **Rimuovi l'unità**.

---

## 8. Copy Ufficiale Pronto all'Uso (Multilingua)

### 8.1 Slogan e Tagline Brevi

| Formato | Italiano (`IT`) | English (`EN`) | Français (`FR`) | Deutsch (`DE`) |
| :--- | :--- | :--- | :--- | :--- |
| **Definizione di Categoria** | Universal Portable Dev & AI Workspace | Universal Portable Dev & AI Workspace | Universal Portable Dev & AI Workspace | Universal Portable Dev & AI Workspace |
| **Headline Hero (Primaria)** | Sviluppo, tutto in una cartella, su qualsiasi computer. | Dev & AI, all in one folder, on any machine. | Développement, tout dans un dossier, sur n'importe quel ordinateur. | Dev & KI, alles in einem Ordner, auf jedem Computer. |
| **Headline Tecnica** | Editor, Runtime, Database e Agenti AI. In una sola cartella, su 4 sistemi. | Editor, Runtimes, Databases, and AI Agents. One portable folder across 4 systems. | Éditeur, Runtimes, Bases de données et Agents IA. Un seul dossier sur 4 systèmes. | Editor, Runtimes, Datenbanken und KI-Agenten. Ein portabler Ordner für 4 Systeme. |
| **Sandbox** | Ogni progetto nella sua microVM. La configurazione nella tua cartella. | Every project in its own microVM. The setup in your folder. | Chaque projet dans sa propre microVM. La configuration dans votre dossier. | Jedes Projekt in seiner eigenen microVM. Die Konfiguration in deinem Ordner. |
| **Sottotitolo "Zero-Friction"** | Niente installazione. Niente diritti di amministratore. Nessuna traccia fuori dal tuo Atollo. | Zero install. Zero admin rights. Zero footprint outside your Atollo. | Aucune installation. Aucun droit d'administrateur. Aucune trace hors de votre Atollo. | Keine Installation. Keine Admin-Rechte. Keine Spuren außerhalb deines Atollo. |

### 8.2 Elevator Pitch Completo (Per GitHub README / Pagina "Informazioni")

**Italiano:**
> **Atollo** è un *Universal Portable Dev & AI Workspace* per Windows x64, macOS su Apple Silicon e Linux (`x64` e `ARM64`), i sistemi su cui gira Docker Sandboxes. Racchiude in un'unica cartella portabile un editor integrato (VSCodium), più di 20 linguaggi e strumenti scaricati su richiesta, database e server web locali (MySQL, MongoDB, SQLite, Caddy/Apache), modelli AI locali (llama.cpp) e gli agenti **Claude Code, Codex, Antigravity e Pi** con istruzioni, skill e server MCP sincronizzati. Ogni progetto gira nella sua microVM di Docker Sandboxes, che Atollo configura e tiene nella cartella (dove il PC la permette; altrimenti l'app lo dice). Non richiede installazione né diritti di amministratore, tiene il proprio ambiente separato dal sistema ospite e ti segue su qualsiasi disco o computer.

**English:**
> **Atollo** is a *Universal Portable Dev & AI Workspace* for Windows x64, macOS on Apple Silicon, and Linux (`x64` and `ARM64`), the systems Docker Sandboxes runs on. It packs into a single portable folder a built-in editor (VSCodium), 20+ on-demand languages and toolchains, local databases and web servers (MySQL, MongoDB, SQLite, Caddy/Apache), local AI models (llama.cpp), and **Claude Code, Codex, Antigravity, and Pi** agents with synchronized instructions, skills, and MCP servers. Each project runs in its own Docker Sandboxes microVM, which Atollo configures and keeps in the folder (where the PC supports it; otherwise the app says so). It requires no installation and no administrator rights, keeps its environment apart from the host OS, and travels with you on any drive or machine.

---

## 9. Strategia di Lancio e Promozione ("Show, Don't Tell")

Quando un agente prepara materiali per il lancio pubblico, deve dare priorità a formati **dimostrativi e verificabili**:

### 9.1 Il Video Hero da 45 Secondi: *"The Unplug Test"*
1. **0:00 – 0:15 (In mezzo all'oceano Windows):** Apertura di `Atollo` da un'unità esterna su Windows. Avvio con un clic di un progetto full-stack + database + agente AI attivo nell'editor.
2. **0:15 – 0:25 (Chiusura senza tracce):** Clic su **Rimuovi l'unità**. Tutti i processi del kit si chiudono istantaneamente, le credenziali rientrano nel kit, l'unità è pronta per l'espulsione.
3. **0:25 – 0:40 (Approdo su macOS / Linux):** Lo stesso disco viene collegato a un Mac Apple Silicon o a una macchina Linux. Si apre `Atollo`: il Router rileva il nuovo target (`darwin-arm64` / `linux-x64`), e lo stesso identico progetto, con lo stesso database e gli stessi 4 agenti AI sincronizzati, riparte con i binari nativi del nuovo sistema operativo.
4. **0:40 – 0:45 (Payoff finale):** Logo **Atollo** su sfondo `#1F1F1F` con accento `#10B981` — *"Universal Portable Dev & AI Workspace. Zero install. Four systems."*

### 9.2 Canali Prioritari e Angoli Editoriali
* **Hacker News (`Show HN`):**
  * *Titolo consigliato:* `Show HN: Atollo – Portable, zero-install dev environment and AI agent hub (Windows x64, Apple Silicon, Linux x64/ARM64)`
  * *Primo commento tecnico:* Spiegare l'architettura in Rust + Tauri 2, come funziona il `Router` sui 4 target (quelli di Docker Sandboxes), Docker Sandboxes configurato dal kit (una microVM per progetto, `sbx` e il kit di ogni sandbox nella cartella, i tre «box»: VM, kit sul PC, Senza VM), l'ambiente del kit senza permessi di root, la gestione dei processi tramite Job Object / gruppi di processi e la sincronizzazione unificata di Claude Code, Codex, Antigravity e Pi, il server llama.cpp dei modelli locali su `127.0.0.1` con chiave.
* **Scuole, Università, Bootcamp e Formazione IT:**
  * Proporre **Atollo** come soluzione definitiva al *"Laboratorio bloccato"*: gli studenti tengono `Atollo` sulla propria chiavetta USB o cartella profilo, lavorano in aula senza chiedere la password di amministratore al tecnico IT e continuano a casa sullo stesso identico ambiente.
* **Community AI & Developer (`r/ClaudeAI`, `r/LocalLLaMA`, `r/rust`, `r/webdev`, X/Twitter):**
  * Mostrare screenshot reali della pagina **AI** (con i consumi di oggi/7 giorni e la sincronizzazione MCP/skill tra i 4 agenti) e della matrice **Strumenti** multipiattaforma, nella lingua del pubblico: `site/shots/<lingua>/` ha le stesse sei schermate in `en`, `it`, `fr`, `de` (D-143). Per un link condiviso basta la social card `docs/brand/social.png`, in inglese.

---

## 10. Mappa Tecnica del Repository per il Rebranding (Riferimento per Agenti)

Se all'agente viene chiesto di applicare o aggiornare il brand **Atollo** nel codice sorgente, questi sono i punti esatti di intervento:

1. **Generatori del Brand e dei Temi (Fonte primaria):**
   * [`docs/brand/make.py`](make.py): genera i file SVG (`symbol*.svg`, `wordmark-*.svg`, `hero-*.svg`), i PNG (`logo-32.png`, `logo-256.png`, `symbol-512.png`, `ui/public/favicon.png`) e tutte le icone di `tauri/icons/` (`icon.ico`, `icon.icns`, `icon.png`, `32x32.png`, `128x128*.png`). Fino a 48 px si usa `symbol-small.svg`: isolotti più grossi, niente barriera sommersa, un'onda sola e spessa, perché a quelle dimensioni i dettagli si impastano. Gli sfondi `hero-*.svg` riprendono il simbolo: l'atollo in alto a destra e file di onde che sfumano verso sinistra, dove sta il contenuto.
   * [`docs/brand/themes.py`](themes.py): legge [`docs/brand/vscode-defaults/dark.json`](vscode-defaults/dark.json) e [`docs/brand/vscode-defaults/light.json`](vscode-defaults/light.json) insieme a `vscodium-themes/` (con le chiavi rinominate come le legge VSCodium 1.135, `RENAMED`) e alle misure di [`vscodium-1.135/workbench-ref.css`](vscodium-1.135/workbench-ref.css), applica la mappa colori `BRAND` e genera:
     * [`tauri/editor-theme/package.json`](../../tauri/editor-theme/package.json) e `themes/{dark,light}.json` (il tema installato in VSCodium);
     * [`ui/src/theme/theme-vscode.css`](../../ui/src/theme/theme-vscode.css), [`ui/src/theme/theme-colors.css`](../../ui/src/theme/theme-colors.css) e [`ui/src/theme/theme-sizes.css`](../../ui/src/theme/theme-sizes.css) (le misure, Sezione 5.4).
   * [`scripts/vscodium-css/extract.mjs`](../../scripts/vscodium-css/extract.mjs): estrae dal VSCodium fissato nel catalogo le regole che l'app imita, in `vscodium-1.135/workbench-ref.css` (licenza MIT di Code-OSS); si rilancia quando il catalogo fissa un altro editor, poi `themes.py`.
2. **Interfaccia Utente e Configurazione Tauri:**
   * [`ui/src/shared/brand.tsx`](../../ui/src/shared/brand.tsx) e [`ui/index.html`](../../ui/index.html)
   * [`tauri/tauri.conf.json`](../../tauri/tauri.conf.json) (`productName`, `mainBinaryName`, `identifier`, titolo finestra)
   * [`tauri/src/editor/theme.rs`](../../tauri/src/editor/theme.rs) e [`tauri/src/editor/cfg.rs`](../../tauri/src/editor/cfg.rs) (ID e nome del tema dell'editor)
3. **Localizzazione e Documentazione:**
   * [`tauri/i18n/catalog.json`](../../tauri/i18n/catalog.json) (stringhe del motore Rust in `it`, `en`, `fr`, `de`)
   * [`ui/src/locales/en.json`](../../ui/src/locales/en.json), [`fr.json`](../../ui/src/locales/fr.json), [`de.json`](../../ui/src/locales/de.json) e testi italiani nei componenti `ui/src/**`
   * [`README.md`](../../README.md) (inglese, la pagina principale), `docs/README.{it,fr,de}.md`, il testo della Guida (pubblicata su [https://nabre.github.io/atollo/guide/](https://nabre.github.io/atollo/guide/)), [`docs/app/ARCHITECTURE.md`](../app/ARCHITECTURE.md) e [`ARCHITECTURE.it.md`](../app/ARCHITECTURE.it.md), [`docs/app/CATALOG.md`](../app/CATALOG.md) e [`docs/app/TESTING.md`](../app/TESTING.md) con le loro copie `.it.md`, la matrice [`docs/app/TOOLS-MATRIX.html`](../app/TOOLS-MATRIX.html) (generata da `scripts/matrix/gen_matrix.mjs`, solo in inglese)
4. **Sito (`https://nabre.github.io/atollo/`):**
   * [`site/strings/`](../../site/strings/) (`it`, `en`, `fr`, `de`, stesse chiavi: la build si ferma se una manca), [`site/landing.html`](../../site/landing.html) con [`site/partials/`](../../site/partials/) (intestazione e piè di pagina comuni alla Guida), [`site/assets/site.css`](../../site/assets/site.css) (i colori della Sezione 5); l'hero e la chiusura usano `hero-dark.svg` / `hero-light.svg` secondo il tema; le schermate di `site/shots/<lingua>/` (fatte da `site/shots/make.mjs` nelle quattro lingue, D-143; ogni pagina e ogni README usa quelle della sua lingua, anche nel JSON-LD) si mostrano ritagliate (`.crop`: via la barra del titolo e quella di stato, 60 e 56 px a 2×) dentro una cornice del sito; la social card `docs/brand/social.png` (1280×640, da `site/shots/social.mjs`, in inglese per tutte le lingue) è l'`og:image` del sito e l'anteprima del repository su GitHub; le schermate della Guida (`ui/public/guide/<lingua>/`) vengono da `scripts/guide-shots/make.mjs`; la tabella «Strumenti per sistema» si genera da `tauri/catalog/components.json` in [`site/matrix.mjs`](../../site/matrix.mjs) (generi in `KIND_OF`), mai a mano; compilato da [`site/build.mjs`](../../site/build.mjs) e pubblicato da `.github/workflows/site.yml` (DECISIONS.md D-126).
