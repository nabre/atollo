# ATOLLO — Manuale Operativo di Marketing, Brand e Posizionamento (Guida per Agenti AI)

> **Scopo di questo documento:** Questo file è la fonte di verità unica (*Single Source of Truth*) per qualsiasi agente AI o collaboratore incaricato di creare, aggiornare o verificare testi promozionali, landing page, documentazione pubblica (`README`), materiale di lancio, asset grafici, temi UI e micro-copy del prodotto **Atollo**.

---

## 1. Direttive Operative per l'Agente

Quando ti viene chiesto di lavorare su qualsiasi aspetto di comunicazione, marketing, design o rebranding del progetto, **attieniti rigorosamente a queste regole**:

1. **Non alterare mai la definizione di categoria:** Il posizionamento ufficiale è **"Universal Portable Dev & AI Workspace"**. Non ridurlo mai a "utility per Windows", "alternativa a XAMPP" o "semplice launcher di agenti".
2. **Coerenza cromatica assoluta con `vscode-defaults`:** Ogni pagina web, presentazione, grafico SVG, documento HTML o componente UI deve utilizzare esclusivamente la tavolozza definita nella **Sezione 5**, derivata da [`docs/brand/vscode-defaults/dark.json`](vscode-defaults/dark.json), [`docs/brand/vscode-defaults/light.json`](vscode-defaults/light.json), [`docs/brand/themes.py`](themes.py) e [`docs/brand/make.py`](make.py).
3. **Tono di voce (*Voice & Tone*):**
   * **Sobrio, ingegneristico, concreto, mai gonfiato:** Evita superlativi vuoti (*"rivoluzionario"*, *"magico"*, *"il migliore al mondo"*). Gli sviluppatori si convincono con i fatti tecnici (*zero installazione, zero diritti di amministratore, 6 architetture native, isolamento del PATH, sincronizzazione di 3 agenti AI*).
   * **Evocativo ma funzionale:** La metafora marina dell'**Atollo** (laguna protetta, isola autosufficiente in qualsiasi oceano/OS) serve a rendere immediato il concetto di isolamento e portabilità, senza mai appesantire l'interfaccia tecnica con gergo nautico forzato.
4. **Parità multilingua obbligatoria (`IT`, `EN`, `FR`, `DE`):** Ogni testo rivolto all'utente finale va mantenuto allineato nelle quattro lingue supportate dal progetto.
5. **Trasparenza tecnica:** Dichiara sempre con onestà cosa gira nativo (`✓`), cosa in emulazione (`≈`) e quali sono i limiti fisici dei sistemi operativi ospiti (come documentato in [`docs/app/CATALOG.md`](../app/CATALOG.md), «Disponibilità per target» e «Fonti per sistema»).

---

## 2. Identità del Brand e Posizionamento Ufficiale

### 2.1 Scheda Sintetica

| Attributo | Valore Ufficiale |
| :--- | :--- |
| **Nome del prodotto** | **Atollo** |
| **Comando CLI / Binario** | `atollo` (Windows: `Atollo.exe`, macOS: `Atollo.app`, Linux: `Atollo.AppImage`) |
| **Categoria di mercato** | **Universal Portable Dev & AI Workspace** |
| **Target Piattaforme (`6/6`)** | `win32-x64`, `win32-arm64`, `darwin-x64`, `darwin-arm64`, `linux-x64`, `linux-arm64` |
| **Licenza** | MIT |

### 2.2 Definizione Canonica (Da usare come base ovunque)

* **Italiano:**
  > **Atollo — Universal Portable Dev & AI Workspace.**
  > Un ambiente di sviluppo autonomo, multi-OS e multi-architettura (*Windows, macOS, Linux × x64/ARM64*) che unisce **Editor**, **Toolchain/Runtime Manager**, **Server/Database locali** e **Orchestratore di Agenti AI** in una singola cartella portabile e senza installazione.
* **English:**
  > **Atollo — Universal Portable Dev & AI Workspace.**
  > A self-contained, multi-OS and multi-architecture development environment (*Windows, macOS, Linux × x64/ARM64*) that unifies an **Editor**, **Toolchain/Runtime Manager**, **Local Servers/Databases**, and an **AI Agent Orchestrator** into a single portable, zero-install folder.
* **Français:**
  > **Atollo — Universal Portable Dev & AI Workspace.**
  > Un environnement de développement autonome, multi-OS et multi-architecture (*Windows, macOS, Linux × x64/ARM64*) qui réunit **Éditeur**, **Gestionnaire de langages et d'outils**, **Serveurs/Bases de données locales** et **Orchestrateur d'agents IA** dans un seul dossier portable sans installation.
* **Deutsch:**
  > **Atollo — Universal Portable Dev & AI Workspace.**
  > Eine eigenständige Multi-OS- und Multi-Architektur-Entwicklungsumgebung (*Windows, macOS, Linux × x64/ARM64*), die **Editor**, **Toolchain-/Runtime-Manager**, **lokale Server/Datenbanken** und **KI-Agenten-Orchestrierung** in einem einzigen portablen Ordner ohne Installation vereint.

---

## 3. Narrativa e Architettura del Valore (I 4 Pilastri + 1 Fondamento)

### 3.1 La Metafora del Brand ("Perché Atollo")
In geografia, un **atollo** è un anello corallino che si innalza in mezzo all'oceano aperto e racchiude al suo interno una **laguna calma, fertile e completamente protetta** dalle correnti esterne.
Nel mondo del software:
* **L'oceano esterno** è il computer ospite: può essere un PC Windows scolastico o aziendale blindato senza permessi di amministratore, un Mac Apple Silicon a casa, una workstation Linux, o un sistema pieno di versioni globali in conflitto.
* **La laguna interna è Atollo:** un perimetro isolato e autosufficiente (`projects/` + `data/`) dove il tuo editor, i tuoi 20+ linguaggi, i tuoi database, i tuoi server web, il tuo account GitHub e i tuoi 3 agenti AI sono già configurati, dialogano tra loro e non lasciano alcuna traccia sul sistema ospite quando smonti l'unità.

### 3.2 I 4 Pilastri Funzionali + Il Fondamento Multi-OS

Ogni presentazione completa di Atollo deve articolarsi su questi **5 punti**:

1. **Editor & Terminali Isolati (Workstation in-a-Box)**
   * Editor VSCodium integrato in finestre dedicate con tema nativo Atollo Dark/Light.
   * Terminali dedicati e profili personalizzabili che vedono esclusivamente il `PATH` e le variabili d'ambiente del kit: i programmi installati sul PC ospite non interferiscono e il PC ospite non viene mai "sporcato".
2. **Visual Toolchain & Runtime Manager (Poliglotta e On-Demand)**
   * Più di 20 linguaggi, compilatori e strumenti (Node.js, Python, uv, PHP, Rust, Go, JDK/Temurin, .NET, Dart, Zig, Julia, Ruby, C/C++, Bun, Deno, Typst, TinyTeX, R, Android SDK…) scaricati solo quando servono, verificati tramite checksum SHA-256 o firma digitale.
   * Gestione di versioni *side-by-side* per singolo progetto o terminale, con verifica automatica della matrice delle relazioni e compatibilità.
3. **Local Server & Database Stack (Zero Servizi di Sistema)**
   * Motori database locali (MariaDB/MySQL, MongoDB, SQLite) avviabili e arrestabili con un clic, con creazione automatica di database, utenti e credenziali `.env` isolati per ciascun progetto, backup integrati e phpMyAdmin/Adminer pronti all'uso.
   * Pool di server web condivisi o dedicati per progetto e scaffolding guidato dei principali framework.
4. **AI Agent Hub & Orchestrator (3 Agenti, 1 Sola Mente)**
   * Installazione, accesso e avvio diretto di **Claude Code**, **Codex** e **Antigravity**.
   * **Sincronizzazione unificata (`sync`):** scrivi una sola volta istruzioni, skill, regole e server MCP in `data/agents` (o per singolo progetto) e Atollo li mantiene allineati automaticamente per tutti e tre gli agenti.
   * **Controllo dei consumi:** monitoraggio in tempo reale dei limiti del piano e dei consumi giornalieri e degli ultimi 7 giorni.
5. **Il Fondamento: Continuità Fisica Multi-OS (`6/6`) e Rimozione Sicura**
   * Grazie al Router multipiattaforma (`win32-x64`, `win32-arm64`, `darwin-x64`, `darwin-arm64`, `linux-x64`, `linux-arm64`), la stessa cartella o lo stesso SSD esterno può passare da Windows a macOS a Linux mantenendo intatti progetti, database, credenziali e configurazioni AI, eseguendo su ogni macchina i binari nativi del sistema ospite.
   * La funzione **Rimuovi l'unità** chiude tutti i processi figli, mette in sicurezza le sessioni e le credenziali nel kit e permette di espellere il disco senza rischi.

---

## 4. Posizionamento Competitivo (Come rispondere a "In cosa è diverso da…?")

| Strumento esistente | Problema che lascia aperto | La risposta di **Atollo** |
| :--- | :--- | :--- |
| **Docker / Devcontainers** | Richiede diritti di amministratore, Docker Desktop o WSL2; lento sul filesystem; impossibile da avviare su PC ospiti da una chiavetta USB. | **Isolamento senza virtualizzazione:** binari nativi sul metallo, zero permessi di root/admin, avvio istantaneo anche da disco esterno. |
| **Nix / Devbox / mise / Scoop** | Solo riga di comando, curva di apprendimento ripida, su Windows spesso dipende da WSL2; non include editor, DB manager né orchestrazione AI. | **Potenza visuale e completa:** interfaccia grafica immediata, nativa su Windows/macOS/Linux, con Editor, DB e Agenti AI già integrati. |
| **Laragon / Laravel Herd / XAMPP** | Legati a 1 o 2 sistemi operativi, focalizzati quasi solo su PHP/Web, non portabili tra OS diversi sullo stesso disco, privi di gestione multi-agente AI. | **Poliglotta e Cross-OS:** 20+ ecosistemi di linguaggi, rilocazione automatica tra 6 target hardware/OS e gestione nativa degli agenti AI. |
| **Cursor / VS Code / CLI AI sparse** | Si installano nel sistema ospite, dipendono dai runtime globali del PC e costringono a configurare tre volte MCP, skill e regole per ogni agente. | **Sovranità e allineamento:** l'editor e i 3 agenti AI vivono dentro il kit, vedono gli stessi runtime del progetto e condividono un'unica configurazione. |

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
| **Atoll Emerald (Dark Accent)** | `EMERALD` | `#10B981` | **Accento primario nel Tema Scuro**: `button.background`, `statusBar.background`, `focusBorder`, `activityBar.activeBorder`, `progressBar.background`, `tab.activeBorderTop`. |
| **Atoll Emerald Hover (Dark)** | — (`themes.py`) | `#0EA572` | Hover dei pulsanti primari nel Tema Scuro (`button.hoverBackground`). |
| **Reef Emerald (Light Accent)** | — (`themes.py`) | `#047857` | **Accento primario nel Tema Chiaro** (più scuro per garantire contrasto WCAG ≥ 4.5:1 con testo bianco `#FFFFFF`): pulsanti, statusbar, focus, link e bordi attivi. |
| **Reef Emerald Hover (Light)** | — (`themes.py`) | `#065F46` | Hover dei pulsanti primari nel Tema Chiaro (`button.hoverBackground`). |
| **Surf Glow (Highlight)** | — (`make.py` / `themes.py`) | `#34D399` | Link e testo evidenziato su tema scuro (`textLink.foreground`, `list.highlightForeground`), bordo tab selezionato su tema chiaro. |
| **Crest Mint** | — (`themes.py`) | `#6EE7B7` | Bordo superiore del tab selezionato in tema scuro (`tab.selectedBorderTop`). |
| **Shallow Mint (Soft Surface)** | `MINT` | `#D1F0E0` | Superficie morbida di accento, `inputOption.activeBackground` e `modernActivityBarItem.activeBackground` in tema chiaro, testo su `FOREST`. |
| **Coral Orange (Dark Action)** | `ORANGE` | `#F97316` | **Accento secondario / Attenzione (Tema Scuro):** `charts.orange`, `statusBar.debuggingBackground`, triangolo interno della lettera `A` nel wordmark e vertice caldo del simbolo. |
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

---

## 6. Tipografia e Geometria Visiva

1. **Font UI (Interfaccia e Titoli):**
   * Stack: `"Segoe WPC", "Segoe UI", "Inter", system-ui, -apple-system, sans-serif`
   * Dimensione base UI: `13px` (nell'app desktop, come da `vscode-defaults`) · `14px–16px` (nei documenti web e landing page).
2. **Font Monospace (Codice, CLI, Percorsi, Versioni, Badge):**
   * Stack: `Consolas, "JetBrains Mono", "SF Mono", "Courier New", ui-monospace, monospace`
   * Dimensione base Editor/Code: `14px` (`11px–12px` per badge tecnici, SHA, identificatori di target come `win32-x64` o `darwin-arm64`).
3. **Uso dei Tag Tecnici:**
   * Tutti i nomi di target (`win32-x64`, `darwin-arm64`, `linux-x64`…), i percorsi (`projects/`, `data/apps/`), i comandi CLI (`atollo check`, `atollo sync`) e i formati di archivio (`.tar.gz`, `.zip`, `.AppImage`) vanno sempre resi in **monospace** con bordo sottile `1px solid var(--vsc-border)`.

---

## 7. Architettura dei Messaggi per Profilo Utente (Persona Matrix)

Quando generi copy per una campagna, una sezione del sito o un post social, scegli l'angolo corrispondente al pubblico:

### Persona 1: Lo Sviluppatore "AI-First"
* **Dolore:** Usa Claude Code, Codex e Antigravity, ma deve mantenere 3 cartelle di configurazione diverse, 3 file di istruzioni, 3 elenchi di server MCP e non sa mai quanto budget/token sta consumando su ciascun fornitore.
* **Hook:** *"Tre agenti AI. Una sola mente. Sul tuo disco."*
* **Funzionalità da evidenziare:** Pagina **Agenti**, comando `atollo sync`, allineamento automatico di istruzioni/skill/MCP in `data/agents`, grafico dei consumi giornalieri e settimanali, salvataggio delle credenziali nel kit.

### Persona 2: Lo Sviluppatore Full-Stack / Poliglotta
* **Dolore:** Ha il PC lento e il `PATH` inquinato da 4 versioni di Node, 3 di Python, Java, PHP, servizi di MariaDB e MongoDB che partono all'avvio di Windows/macOS, oppure Docker Desktop che consuma 8 GB di RAM solo per far girare un sito locale.
* **Hook:** *"L'isolamento di un Devcontainer. La velocità del metallo nativo. Zero servizi nel sistema."*
* **Funzionalità da evidenziare:** Pagine **Progetti**, **Database** e **Strumenti**, versioni *side-by-side* per progetto, matrice delle relazioni, avvio e arresto istantaneo di DB e web server solo quando servono.

### Persona 3: Lo Studente, Docente o Consulente Nomade
* **Dolore:** Lavora su PC scolastici, universitari, di laboratorio o aziendali dove non ha i permessi di amministratore, oppure si sposta continuamente tra un fisso Windows, un MacBook e una macchina Linux.
* **Hook:** *"Il tuo intero studio di sviluppo in tasca. Su 6 architetture, senza permessi di amministratore."*
* **Funzionalità da evidenziare:** Esecuzione da chiavetta USB o SSD esterno, rilocazione automatica dei percorsi al cambio di lettera/cartella, account GitHub unico per tutto il kit, pulsante **Rimuovi l'unità**.

---

## 8. Copy Ufficiale Pronto all'Uso (Multilingua)

### 8.1 Slogan e Tagline Brevi

| Formato | Italiano (`IT`) | English (`EN`) | Français (`FR`) | Deutsch (`DE`) |
| :--- | :--- | :--- | :--- | :--- |
| **Definizione di Categoria** | Universal Portable Dev & AI Workspace | Universal Portable Dev & AI Workspace | Universal Portable Dev & AI Workspace | Universal Portable Dev & AI Workspace |
| **Headline Hero (Primaria)** | Il tuo spazio di sviluppo sovrano, su qualsiasi computer. | Your sovereign dev & AI workspace, on any machine. | Votre espace de développement souverain, sur n'importe quel ordinateur. | Dein souveräner Dev- & KI-Workspace auf jedem Computer. |
| **Headline Tecnica** | Editor, Runtime, Database e Agenti AI. In una sola cartella, su 6 architetture. | Editor, Runtimes, Databases, and AI Agents. One portable folder across 6 targets. | Éditeur, Runtimes, Bases de données et Agents IA. Un seul dossier sur 6 architectures. | Editor, Runtimes, Datenbanken und KI-Agenten. Ein portabler Ordner für 6 Architekturen. |
| **Sottotitolo "Zero-Friction"** | Niente installazione. Niente diritti di amministratore. Nessuna traccia fuori dal tuo Atollo. | Zero install. Zero admin rights. Zero footprint outside your Atollo. | Aucune installation. Aucun droit d'administrateur. Aucune trace hors de votre Atollo. | Keine Installation. Keine Admin-Rechte. Keine Spuren außerhalb deines Atollo. |

### 8.2 Elevator Pitch Completo (Per GitHub README / Pagina "Informazioni")

**Italiano:**
> **Atollo** è un *Universal Portable Dev & AI Workspace* per Windows, macOS e Linux (`x64` e `ARM64`). Racchiude in un'unica cartella portabile un editor integrato (VSCodium), più di 20 linguaggi e strumenti scaricati su richiesta, database e server web locali (MariaDB/MySQL, MongoDB, SQLite, Apache/Nginx/Caddy) e gli agenti **Claude Code, Codex e Antigravity** con istruzioni, skill e server MCP sincronizzati. Non richiede installazione né diritti di amministratore, isola completamente il proprio ambiente dal sistema ospite e ti segue su qualsiasi disco o computer.

**English:**
> **Atollo** is a *Universal Portable Dev & AI Workspace* for Windows, macOS, and Linux (`x64` and `ARM64`). It packs into a single portable folder a built-in editor (VSCodium), 20+ on-demand languages and toolchains, local databases and web servers (MariaDB/MySQL, MongoDB, SQLite, Apache/Nginx/Caddy), and **Claude Code, Codex, and Antigravity** agents with synchronized instructions, skills, and MCP servers. It requires no installation and no administrator rights, completely isolates its environment from the host OS, and travels with you on any drive or machine.

---

## 9. Strategia di Lancio e Promozione ("Show, Don't Tell")

Quando un agente prepara materiali per il lancio pubblico, deve dare priorità a formati **dimostrativi e verificabili**:

### 9.1 Il Video Hero da 45 Secondi: *"The Unplug Test"*
1. **0:00 – 0:15 (In mezzo all'oceano Windows):** Apertura di `Atollo` da un'unità esterna su Windows. Avvio con un clic di un progetto full-stack + database + agente AI attivo nell'editor.
2. **0:15 – 0:25 (Chiusura senza tracce):** Clic su **Rimuovi l'unità**. Tutti i processi del kit si chiudono istantaneamente, le credenziali rientrano nel kit, l'unità è pronta per l'espulsione.
3. **0:25 – 0:40 (Approdo su macOS / Linux):** Lo stesso disco viene collegato a un Mac Apple Silicon o a una macchina Linux. Si apre `Atollo`: il Router rileva il nuovo target (`darwin-arm64` / `linux-x64`), e lo stesso identico progetto, con lo stesso database e gli stessi 3 agenti AI sincronizzati, riparte con i binari nativi del nuovo sistema operativo.
4. **0:40 – 0:45 (Payoff finale):** Logo **Atollo** su sfondo `#1F1F1F` con accento `#10B981` — *"Universal Portable Dev & AI Workspace. Zero install. Six architectures."*

### 9.2 Canali Prioritari e Angoli Editoriali
* **Hacker News (`Show HN`):**
  * *Titolo consigliato:* `Show HN: Atollo – Portable, zero-install dev environment and AI agent hub (Win/Mac/Linux, x64/ARM64)`
  * *Primo commento tecnico:* Spiegare l'architettura in Rust + Tauri 2, come funziona il `Router` sui 6 target, l'isolamento del `PATH` senza container né permessi di root, la gestione dei processi tramite Job Object / gruppi di processi e la sincronizzazione unificata di Claude Code, Codex e Antigravity.
* **Scuole, Università, Bootcamp e Formazione IT:**
  * Proporre **Atollo** come soluzione definitiva al *"Laboratorio bloccato"*: gli studenti tengono `Atollo` sulla propria chiavetta USB o cartella profilo, lavorano in aula senza chiedere la password di amministratore al tecnico IT e continuano a casa sullo stesso identico ambiente.
* **Community AI & Developer (`r/ClaudeAI`, `r/LocalLLaMA`, `r/rust`, `r/webdev`, X/Twitter):**
  * Mostrare screenshot reali della pagina **Agenti** (con i consumi di oggi/7 giorni e la sincronizzazione MCP/skill tra i 3 agenti) e della matrice **Strumenti** multipiattaforma.

---

## 10. Mappa Tecnica del Repository per il Rebranding (Riferimento per Agenti)

Se all'agente viene chiesto di applicare o aggiornare il brand **Atollo** nel codice sorgente, questi sono i punti esatti di intervento:

1. **Generatori del Brand e dei Temi (Fonte primaria):**
   * [`docs/brand/make.py`](make.py): genera i file SVG (`symbol*.svg`, `wordmark-*.svg`, `lockup-*.svg`, `hero-*.svg`), i PNG (`logo-32.png`, `logo-256.png`, `symbol-512.png`) e l'icona `tauri/icons/icon.ico`.
   * [`docs/brand/themes.py`](themes.py): legge [`docs/brand/vscode-defaults/dark.json`](vscode-defaults/dark.json) e [`docs/brand/vscode-defaults/light.json`](vscode-defaults/light.json) insieme a `vscodium-themes/`, applica la mappa colori `BRAND` e genera:
     * [`tauri/editor-theme/package.json`](../../tauri/editor-theme/package.json) e `themes/{dark,light}.json` (il tema installato in VSCodium);
     * [`ui/src/theme/theme-vscode.css`](../../ui/src/theme/theme-vscode.css) e [`ui/src/theme/theme-colors.css`](../../ui/src/theme/theme-colors.css).
2. **Interfaccia Utente e Configurazione Tauri:**
   * [`ui/src/brand.tsx`](../../ui/src/brand.tsx) e [`ui/index.html`](../../ui/index.html)
   * [`tauri/tauri.conf.json`](../../tauri/tauri.conf.json) (`productName`, `mainBinaryName`, `identifier`, titolo finestra)
   * [`tauri/src/editor/theme.rs`](../../tauri/src/editor/theme.rs) e [`tauri/src/editor/cfg.rs`](../../tauri/src/editor/cfg.rs) (ID e nome del tema dell'editor)
3. **Localizzazione e Documentazione:**
   * [`tauri/i18n/catalog.json`](../../tauri/i18n/catalog.json) (stringhe del motore Rust in `it`, `en`, `fr`, `de`)
   * [`ui/src/locales/en.json`](../../ui/src/locales/en.json), [`fr.json`](../../ui/src/locales/fr.json), [`de.json`](../../ui/src/locales/de.json) e testi italiani nei componenti `ui/src/**`
   * [`README.md`](../../README.md), `docs/README.{en,fr,de}.md`, [`docs/app/GUIDA.md`](../app/GUIDA.md), [`docs/app/ARCHITECTURE.md`](../app/ARCHITECTURE.md)
