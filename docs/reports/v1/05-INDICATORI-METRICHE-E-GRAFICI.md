# 05 — Artefatto Complementare: Criteri, Indicatori (KPI), Valutazioni e Rappresentazioni Grafiche

> [!IMPORTANT]
> Questo documento costituisce l'**artefatto complementare** richiesto per promuovere **grafiche, criteri di valutazione, indicatori quantitativi (KPI) e valutazioni analitiche** sui singoli aspetti del progetto Atollo (Atollo). È accompagnato dalla dashboard interattiva [`dashboard-valutazione.html`](file:///c:/Users/nabre/Projects/atollo/reports/dashboard-valutazione.html).

---

## 1. Metodologia di Verifica e Criteri Indicatori (Rubrica di Valutazione)

Per garantire oggettività scientifica e ripetibilità nella verifica del software, ciascuna delle **9 dimensioni** è stata scomposta in **4 sub-indicatori tecnici (KPI)** pesati al 25%, valutati su scala `0–100`:

| Fascia di Punteggio | Grado | Criterio di Accettazione del Verificatore |
| :---: | :---: | :--- |
| **90 – 100** | **A+ / Eccellente** | Architettura/UX allo stato dell'arte, zero debiti strutturali, copertura test automatica, nessun difetto funzionale rilevante. |
| **80 – 89** | **A / Molto Buono** | Implementazione solida e matura; presenti solo anomalie minori o edge-case di concorrenza facilmente risolvibili. |
| **70 – 79** | **B / Buono-Discreto** | Funzionalità core operative, ma presenza di feature parzialmente collegate o bug di media/alta priorità da correggere prima del rilascio 1.0. |
| **50 – 69** | **C / Parziale** | Base architetturale presente e ben progettata, ma copertura incompleta (es. supporto limitato a un solo OS). |
| **< 50** | **D / Critico** | Ristrutturazione profonda necessaria. *(Nessuna dimensione di Atollo ricade in questa fascia).* |

---

## 2. Rappresentazione Grafica 1 — Radar Globale sulle 9 Dimensioni

![Radar 9 Dimensioni](assets/radar-9-dimensioni.svg)

### Grafico Comparativo (Stato Attuale vs Target Release 1.0)

```mermaid
xychart-beta
    title "Punteggio per Dimensione di Verifica (Attuale v0.5.3 vs Target v1.0)"
    x-axis ["1. Arch.", "2. Logica", "3. UX", "4. UI", "5. Uso", "6. Compl.", "7. No-Bug", "8. Multi-OS", "9. Facilit."]
    y-axis "Punteggio (0-100)" 0 --> 100
    bar [92, 84, 86, 90, 88, 78, 76, 57, 91]
    line [95, 92, 92, 94, 92, 90, 92, 85, 95]
```

---

## 3. Matrice Analitica dei 36 Sotto-Indicatori (4 KPI × 9 Dimensioni)

| Dimensione | KPI 1 (25%) | KPI 2 (25%) | KPI 3 (25%) | KPI 4 (25%) | Media Dimensione |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **1. Architettura** | Separazione Layer<br>**96/100** | Sicurezza IPC (`D-06`)<br>**95/100** | Gestione Dati/Temp (`D-89/90`)<br>**92/100** | Modularità Frontend/Bundle<br>**85/100** | **92 / 100** |
| **2. Logica** | Costruzione Env & Rilocazione<br>**94/100** | SemVer & Matrice Relazioni<br>**92/100** | Concorrenza & Atomicità I/O<br>**74/100** | Orchestrazione Server Pool<br>**76/100** | **84 / 100** |
| **3. UX** | Onboarding & Wizard<br>**94/100** | Trasparenza Azioni (`PlanPreview`)<br>**92/100** | Gestione Stati d'Errore/Vuoti<br>**78/100** | Continuità Progetto ↔ Server<br>**80/100** | **86 / 100** |
| **4. UI** | Fedeltà Workbench VSCodium<br>**95/100** | Contrasto & 4 Temi WCAG<br>**96/100** | Iconografia & Densità Visiva<br>**88/100** | Tipografia & Casing i18n<br>**81/100** | **90 / 100** |
| **5. Esperienza d'Uso** | Reattività & Zero Console Flash<br>**92/100** | Igiene Host (Zero Residui/DPAPI)<br>**95/100** | Resilienza Offline / `byhand`<br>**86/100** | Fluidità Terminale & Editor<br>**79/100** | **88 / 100** |
| **6. Completezza** | Copertura 26 Componenti & 18 FW<br>**92/100** | Sync & Statistiche Agenti IA<br>**88/100** | Integrazione Pool Server (`WEB`)<br>**64/100** | DB Multi-Linea & `ProjectMcpBox`<br>**68/100** | **78 / 100** |
| **7. Robustezza (Bug)** | Assenza Panic (`deny(unwrap)`)<br>**96/100** | Sicurezza Path (`safe_path`)<br>**78/100** | Stabilità Processi Server/CGI<br>**62/100** | Assenza Race Asincrone UI<br>**68/100** | **76 / 100** |
| **8. Multipiattaforma** | Portabilità UI & IPC Tauri<br>**94/100** | Portabilità Logica & Archivi<br>**86/100** | Astrazione OS (`os/`, `keyring`)<br>**18/100** | Catalogo Binari macOS/Linux<br>**30/100** | **57 / 100** |
| **9. Facilitazione** | Riduzione *Time-to-Code* (-95%)<br>**96/100** | Facilitazione Neofiti/Studenti<br>**90/100** | Produttività Sviluppatori Senior<br>**88/100** | Valore Pedagogico & Guida Live<br>**90/100** | **91 / 100** |

---

## 4. Rappresentazione Grafica 2 — Readiness Multipiattaforma per Sottosistema

![Readiness Multipiattaforma](assets/multipiattaforma-readiness.svg)

```mermaid
xychart-beta
    title "Percentuale di Codice Già Pronto per macOS/Linux per Sottosistema"
    x-axis ["UI React", "Runtime sh", "Detect/Ver", "IPC Tauri", "Net/Zip", "Agents", "DB/Srv", "Editor", "Install", "Keyring", "WTerm", "OS Win32"]
    y-axis "% Pronto Senza Riscrittura" 0 --> 100
    bar [96, 90, 92, 88, 78, 65, 50, 48, 30, 20, 15, 12]
```

---

## 5. Rappresentazione Grafica 3 — Matrice Bug & Facilitazione Utente (*Time-to-Code*)

![Matrice Bug e Facilitazione](assets/bug-e-gap-matrix.svg)

### Confronto Tempi di Configurazione Iniziale (Minuti — Scala Reale)

```mermaid
xychart-beta
    title "Tempo Richiesto per Configurare un Ambiente Completo (Minuti — Meno è Meglio)"
    x-axis ["Stack PHP/Laravel + SSL + DB", "Cambio Versione Multi-Progetto", "Sync 4 Agenti IA + MCP", "Migrazione su Nuovo PC / USB"]
    y-axis "Minuti" 0 --> 100
    bar [95, 25, 40, 60]
    line [4, 1, 1, 2]
```
> *Legenda*: **Barre** = Configurazione manuale tradizionale senza Atollo; **Linea** = Configurazione assistita con Atollo.

---

## 6. Diagramma di Stato del Ciclo di Vita di un Pool Server (Stato Attuale vs Stato Corretto Target)

```mermaid
stateDiagram-v2
    [*] --> Stopped: Pool Configurato (servers.json)

    state "Flusso Attuale (Con BUG-03, BUG-04, BUG-05)" as Current {
        Stopped --> Spawning: start() (Senza lock per-pool)
        Spawning --> WaitPort: spawn httpd/nginx + php-cgi (max 500 req!)
        WaitPort --> Running_FalsePositive: Timeout 6s scaduto o crash immediato (Restituisce comunque Ok!)
        WaitPort --> Running_OK: Porta TCP risponde entro 6s
        Running_OK --> BadGateway_502: php-cgi termina dopo 500 richieste (Nessun supervisore)
        Running_OK --> Reloading_Race: reload() Apache (stop + sleep 120ms + start -> Porta ancora occupata!)
    }

    state "Flusso Target Corretto (Raccomandato)" as Target {
        Stopped --> LockedStart: Acquisizione Mutex per-pool
        LockedStart --> SupervisedSpawn: Avvio httpd/nginx + supervisore php-cgi (MAX_REQUESTS=0)
        SupervisedSpawn --> HealthCheck: Loop 6s con controllo child.try_wait() + TCP connect
        HealthCheck --> ErrorWithLog: Crash o Timeout -> Legge error.log e ritorna Err()
        HealthCheck --> ActiveHealthy: Porta attiva -> Notifica UI & Badge ActivityBar
        ActiveHealthy --> GracefulReload: httpd -k graceful / nginx -s reload (o wait porta libera)
        GracefulReload --> ActiveHealthy
    }
```

---

## 7. Indicatori di Qualità del Codice e Testabilità (Metriche Statiche Repository)

| Indicatore Tecnico | Valore Rilevato nel Progetto | Benchmark Industria | Valutazione |
| :--- | :---: | :---: | :---: |
| **Occorrenze `unwrap()` / `expect()` in codice di produzione Rust** | **0** (`deny` in [`main.rs:7`](file:///c:/Users/nabre/Projects/atollo/tauri/src/main.rs#L7)) | `< 5` | **100 / 100** |
| **Comandi IPC protetti da `ensure_main(&webview)`** | **113 / 113 (100%)** | `100%` | **100 / 100** |
| **Moduli di test Rust (`test/rust/**`) e UI (`test/ui/**`)** | **80+ file di test** (unitari, integrazione, contrasto temi, i18n) | Alta copertura | **92 / 100** |
| **Lingue supportate nel catalogo UI/Backend** | **4** (`it`, `en`, `fr`, `de` — Guida parziale in FR/DE) | `>= 2` | **85 / 100** |
| **Componenti gestiti nel Catalogo (`components.json`)** | **26 strumenti** (con verifica SHA-256/512/Authenticode) | `10–15` | **96 / 100** |
| **Framework supportati dal Wizard di creazione** | **18 framework** (con anteprima albero e comandi) | `5–10` | **95 / 100** |
