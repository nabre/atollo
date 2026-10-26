# I tre livelli: dove gira ogni programma di Atollo

[English](LEVELS.md) · Italiano

Documento normativo della decisione D-179 (DECISIONS.md, 26/10/2026). È l'unica fonte di verità su: quale programma gira dove, con quale `PATH`, con quali agenti, con quale rete e quali montaggi; il codice che lo decide; l'IPC che lo trasporta; e come l'utente lo verifica. Codice, prove, Guida e ARCHITECTURE.md seguono questo file; dove differiscono, vale questo file e la differenza è un errore.

Stato: **progetto, fase 1**; fatto il passo 1 di 12.2 (tipi, `place_of`, `facts`, `decide`, etichette), non ancora chiamato dall'app; fatto il passo 2 (`levels::env`, l'unico costruttore dell'ambiente), usato dai chiamanti di oggi; fatto il passo 3 (i punti di avvio del PC E1, E3, E5–E9, E20 attraverso `decide` ed `environment`, `PtyInfo.level/label/sandbox`, `CmdError.refusal/fix`, i pool sigillati), fatto il passo 4 (`levels::prepare` con `Effects`, `levels::start`, i punti di avvio nella VM E3, E4, E9, E10 attraverso `decide` → `prepare` → `start`, le sessioni dell'editor E11/E12 preparate allo stesso modo fino al passo 5, la prova di regola `no_process_outside_levels`). Nient'altro di quanto scritto è già fatto, salvo dove il testo dice «oggi». Ogni affermazione non provata su un sistema vero è segnata **[non verificato]**; ogni scelta di questo documento non ancora confermata dall'utente è segnata **[ipotesi]**. I nomi nel codice (tipi, funzioni, file, comandi) sono quelli della versione inglese, [LEVELS.md](LEVELS.md).

Indice:

1. [Scopo e modello delle minacce](#1-scopo-e-modello-delle-minacce)
2. [La tabella dei vincoli](#2-la-tabella-dei-vincoli)
3. [Il modello del codice: `tauri/src/levels/`](#3-il-modello-del-codice-taurisrclevels)
4. [Contratto IPC](#4-contratto-ipc)
5. [Inventario dei punti di avvio](#5-inventario-dei-punti-di-avvio)
6. [Flussi](#6-flussi)
7. [Agenti disattivati senza VM](#7-agenti-disattivati-senza-vm)
8. [Funzioni di fiducia](#8-funzioni-di-fiducia)
9. [Antigravity](#9-antigravity)
10. [Estensioni dell'editor che eseguono un agente](#10-estensioni-delleditor-che-eseguono-un-agente)
11. [Segreti: nessuno del kit entra in una VM](#11-segreti-nessuno-del-kit-entra-in-una-vm)
12. [Migrazione](#12-migrazione)
13. [Codice di oggi in contrasto con questo documento](#13-codice-di-oggi-in-contrasto-con-questo-documento)
14. [Domande aperte e punti non verificati](#14-domande-aperte-e-punti-non-verificati)

---

## 1. Scopo e modello delle minacce

Atollo avvia tre tipi di programmi, che meritano tre livelli diversi di fiducia:

- **Livello 1, Manuale.** I comandi che l'utente scrive: i terminali del kit e il terminale integrato dell'editor. L'utente è l'autore di ogni comando; il rischio è suo, come in qualsiasi terminale del PC. Girano sul PC, sempre, con o senza VM.
- **Livello 2, Codice del progetto.** Il codice che arriva con un progetto e gira senza che l'utente lo legga riga per riga: `npm run dev`, uno script `postinstall`, la creazione di un framework (`npm create`, `composer create-project`), il PHP di un sito, un task dell'editor. Codice di terzi: può essere ostile (un repository clonato, un pacchetto compromesso). Con la VM gira nella microVM del progetto; senza VM gira sul PC con l'ambiente sigillato (D-167) e la barra di stato dice «Senza VM».
- **Livello 3, Agente.** Un programma autonomo che decide da sé quali comandi eseguire e quali file scrivere: Claude Code, Codex, Pi, Antigravity (`agy`) e le estensioni dell'editor che eseguono un agente. È il livello più pericoloso («pericoloso è l'agente», l'utente, 26/10/2026): agisce da solo, legge ciò che trova e può essere guidato da testo dentro il progetto (prompt injection). **Gira solo in una VM.** Senza VM nessun agente gira da nessuna parte, e tutto ciò che ne avvierebbe uno è disattivato con il motivo.

Cosa proteggono i livelli, e cosa no:

| Minaccia | Livello 1 (PC) | Livello 2 con VM | Livello 2 senza VM | Livello 3 (solo VM) |
|---|---|---|---|---|
| Leggere i file dell'utente fuori dal progetto | possibile (è l'utente) | no: sono montati solo la cartella del progetto (rw) e gli strumenti del kit per Linux (ro) | possibile: stesso utente; l'ambiente sigillato toglie solo le strade comode (agente SSH, login root di MySQL, percorso del profilo vero) | no, come il livello 2 con VM |
| Usare le chiavi SSH, il portachiavi, i segreti del kit | chiavi SSH: sì, l'agente SSH dell'utente resta (`SSH_AUTH_SOCK`, Git su SSH; **deciso dall'utente, 26/10/2026**); il resto non dato (`seal`: niente percorso del profilo vero, bus di sessione, root di MySQL), ma raggiungibile come utente | no | ridotto (`seal`, anche l'agente SSH), non impedito | no |
| Raggiungere i servizi di rete del PC | sì | solo le porte che Atollo apre per quella sandbox (i suoi database, D-171; i modelli locali per Pi, D-178) | sì | come il livello 2 con VM |
| Cambiare i programmi del kit | possibile | no (strumenti montati in sola lettura; il kit e il profilo non sono mai uno spazio di lavoro, D-168) | possibile | no |
| Agire su GitHub a nome dell'utente | possibile | solo se il progetto ha acceso «GitHub nella VM» (D-172), attraverso il proxy di `sbx` | possibile | come il livello 2 con VM |

Il confine è la VM (D-167). Senza VM Atollo non finge un confine: il livello 2 sul PC è «sigillato», non «isolato», e l'interfaccia lo dice.

Una quarta categoria **non** è un livello: i **programmi del kit** (server di database, phpMyAdmin, mongo-express, llama.cpp, Git per clonare, il processo del server dell'editor, `sbx`, l'app). Per costruzione non eseguono mai codice di un progetto e restano sul PC (D-175, «Kit sul PC»). La sezione 5 li elenca, perché l'inventario sia completo.

## 2. La tabella dei vincoli

La stessa tabella vale per l'app e per l'editor. «VM attiva» significa `isolation::status(...).level == Vm` al momento della richiesta (vedi 3.3).

| Chi (origine) | Livello | Dove, VM attiva | Dove, VM non attiva | `PATH` | Comandi degli agenti (`claude`, `codex`, `pi`, `agy`, `antigravity`) | Rete e porte | Montaggi |
|---|---|---|---|---|---|---|---|
| Terminale del kit («Tutti gli strumenti», terminali dell'utente), qualsiasi cartella | 1 | PC | PC | strumenti del kit; nella cartella di un progetto alle linee del progetto (3.5) | **inoltri** in testa al `PATH`: nella cartella di un progetto inoltrano all'agente nella VM del progetto; altrove, o senza VM, dicono perché ed escono con 1 | quella del PC | nessuno (PC) |
| Terminale integrato dell'editor (ogni profilo di Atollo tranne gli agenti) | 1 | PC | PC | come il terminale del kit; il progetto è quello della cartella in cui si apre | inoltri, come sopra | quella del PC | nessuno (PC) |
| Terminale del progetto (pagina del progetto) | 2 | VM del progetto `atollo-p-…` | PC, sigillato | VM: strumenti del kit per Linux alle linee del progetto, poi quelli dell'immagine; PC: strumenti del kit alle linee del progetto | VM: nessuno (le cartelle Linux degli agenti tolte, gli agenti di `sbx` non sono nell'immagine `shell` **[non verificato]**); PC: inoltri, che rifiutano (VM non attiva) | VM: politica `balanced` di `sbx`, le porte dei suoi database, GitHub se scelto; PC: quella del PC | VM: progetto rw, strumenti del kit per Linux ro |
| Azione (`npm run dev`…) | 2 | VM del progetto | PC, sigillato | come il terminale del progetto | nessuno / inoltri | VM: la sua porta web pubblicata su `127.0.0.1` (o `0.0.0.0` con «Rete locale») | come il terminale del progetto |
| Sito di un'azione (statico, Apache, Caddy) | 2 | un container del Docker della VM | PC: il suo server o un pool | — | nessuno | come l'azione | come l'azione, più gli alias del progetto stesso |
| Pool di Server Web | 2 | **rifiutato** (ogni sito gira nella VM del suo progetto) | PC | strumenti del kit, niente agenti | inoltri (rifiutano) | quella del PC | nessuno |
| Task dell'editor (`tasks.json`) | 2 | VM del progetto | PC, sigillato | come il terminale del progetto | come il terminale del progetto | come il terminale del progetto | come il terminale del progetto |
| Creazione di un progetto (framework, «Le mie basi») | 2 | VM del progetto (la sandbox del progetto nuovo) **[nuovo]** | PC, sigillato | come il terminale del progetto | nessuno / inoltri | VM: `balanced` (registri dei pacchetti) | VM: la cartella del progetto nuovo rw, strumenti del kit per Linux ro |
| Claude Code, Codex (pagina del progetto, palette, profilo dell'editor, inoltro, involucro di un'estensione) | 3 | la sua sandbox di `sbx` `atollo-a-<agente>-<progetto>-<hash>` | **rifiutato** | strumenti del kit per Linux alle linee del progetto, poi quelli dell'immagine | solo l'agente stesso | come la VM del progetto (stesse porte dei database, stessa regola per GitHub) | progetto rw, strumenti del kit per Linux ro |
| Pi, Antigravity (`agy`) | 3 | VM del progetto `atollo-p-…`, la loro build per Linux dal kit | **rifiutato** | come il terminale del progetto | solo l'agente stesso | come la VM del progetto; Pi: più la porta dei modelli locali quando li usa | come il terminale del progetto |
| Estensioni dell'editor che sono agenti (sezione 10) | 3 | attraverso l'inoltro quando l'estensione lo permette; altrimenti disattivate | disattivate | — | — | — | — |

Invarianti che seguono dalla tabella (normative; ognuna ha la sua prova, sezione 12.3):

- **I1** Nessuna strada del codice avvia un processo dei livelli 1–3 senza passare da `levels::decide()`. I programmi del kit (sezione 5, tabella «Kit») sono l'unica eccezione e sono elencati per nome.
- **I2** Il livello è una funzione dell'origine (`Origin::level()`), mai un parametro scelto da chi chiama.
- **I3** Un collocamento `Agent` non è mai `Pc`.
- **I4** Un collocamento `Pc` non ha mai nel `PATH` una cartella di un programma di un agente (`levels::env::agent_dirs` del target del PC), e ha in testa la cartella degli inoltri.
- **I5** Un collocamento `Vm` non porta mai un segreto del kit: né negli argomenti, né in `-e`, né in `--env-file` (la sezione 11 definisce «segreto del kit»).
- **I6** Senza VM nulla del livello 3 si installa, si aggiorna, si avvia o si configura nell'editor; i dati degli agenti nel kit restano sul disco, intatti.
- **I7** Un rifiuto porta sempre un motivo e un rimedio (`Refusal`, `Fix`); nulla ripiega in silenzio dalla VM al PC.
- **I8** `decide()` è pura e totale: stessi ingressi, stessa uscita; ogni combinazione di ingressi ha una riga di 3.4, e la prima riga che corrisponde dà il suo unico collocamento.

## 3. Il modello del codice: `tauri/src/levels/`

### 3.1 File

| File | Contenuto | Viene da |
|---|---|---|
| `levels/mod.rs` | `Level`, `Agent`, `AgentRunner`, `Origin`, `Place`, `Request`, `VmState`, `Facts`, `Placement`, `Sandbox`, `Mounts`, `VmProgram`, `HostAccess`, `Refusal`, `Fix`, `Rule`; `decide()`, `decided()` | nuovo |
| `levels/place.rs` | `place_of(kit, dir) -> Place` (a quale progetto appartiene una cartella) | la logica di `runtime/vm.ps1` (`$projects`, `$below`) e `ipc::project_dir` |
| `levels/facts.rs` | `facts(kit, host, env, request) -> Facts` (l'unica lettura con effetti prima di `decide`) | `isolation::status`, `vm::guest_path`, `isolation::restart_pending`, disponibilità nel catalogo |
| `levels/env.rs` | `Environment`, `environment()`, `pc_environment()` (la sua parte `Pc`, ogni passo una funzione: `project_lines`, `seal`, `strip_agents`, `forward_first`, `mark`), `SEALED`, `SSH_AGENT`, `agent_dirs()`, `forward_dir()`, `FORWARD_DIR`, `LEVEL_VAR`, `PROJECT_VAR`, `TERMINAL_VAR` | `isolation::{SEALED, seal, agent_dirs, without_agents, agents_notice, with_agents_notice}`, `versions::{project_env, choice_env}` (la parte che sigilla), `editor::terminals::env` (la parte degli agenti) |
| `levels/prepare.rs` | `Effects`, `Ready`, `prepare()`: strumenti per il Linux della VM, database e loro porte (`db.env`), modelli locali per Pi (`pi.env`), la sandbox costruita e avviata, `info.json` (§8.4) | `ipc::terminals::{project_vm_ready, project_vm_tools, project_vm_dbs}`, `ipc::agents::{agent_vm_ready, pi_local_models}`, `isolation::vm_for` |
| `levels/start.rs` | `vm_session_args()`, `start_terminal()`, `open_terminal()`, `start_action()`, `scaffold_command()`, `scaffold_end()`: da `Ready` a una sessione della pty, a un'azione o al `Command` di una creazione | le code di `terminal_launch`, `terminal_open`, `agent_launch`, `run_start`, `frameworks_create` |
| `levels/requests.rs` | il protocollo a file per i processi fuori dall'app (editor, inoltri) | sostituisce `isolation/session.rs` |
| `levels/report.rs` | `VmReport`, `ToolCheck`, `report()`, `check_tools()` | nuovo |
| `levels/labels.rs` | `LevelLabel`, `label_of(placement)`, i testi di `label(placement, lang)`, `banner_line(placement, lang)`, `refusal_text(reason, lang)` | nuovo; i testi di rifiuto di `ipc::agents::no_vm_for_agents` e `ipc::servers::refused_with_vm` |

`isolation/` tiene ciò che riguarda `sbx` in sé: `status`, `Blocker`, `Sbx`, `Vm` (vita della sandbox, montaggi, porte, pubblicazione, arresto), `vm.rs` (costruttori di argomenti), `daemon.rs`, `kit.rs`, `secrets.rs`. Non decide più chi va dove.

`isolation::Level { Vm, Env }` diventa `isolation::Availability { Vm, NoVm }` (serializzato ancora `"vm"`/`"env"` per l'interfaccia fino al passo 10 di 12.2), così `levels::Level` è l'unico «livello» del codice.

### 3.2 Tipi

I tipi sono quelli di [LEVELS.md, 3.2](LEVELS.md#32-types), che fa fede per i nomi e le firme; qui con i commenti in italiano.

```rust
// levels/mod.rs

/// I tre livelli (D-179). Il numero è quello mostrato all'utente.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Manual = 1,
    Project = 2,
    Agent = 3,
}

/// Gli agenti che Atollo conosce (gruppo `agents` del catalogo).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Agent {
    Claude,
    Codex,
    Pi,
    Antigravity,
}

impl Agent {
    pub const ALL: [Agent; 4] = [Agent::Claude, Agent::Codex, Agent::Pi, Agent::Antigravity];
    /// L'id del catalogo: "claude", "codex", "pi", "antigravity".
    pub fn id(self) -> &'static str;
    /// Il nome che legge l'utente: "Claude Code", "Codex", "Pi", "Antigravity".
    pub fn name(self) -> &'static str;
    /// I comandi che l'utente scrive: ["claude"], ["codex"], ["pi"], ["agy", "antigravity"].
    pub fn commands(self) -> &'static [&'static str];
    /// Dove gira nella VM.
    pub fn runner(self) -> AgentRunner;
    pub fn from_id(id: &str) -> Option<Agent>;
    pub fn from_command(cmd: &str) -> Option<Agent>;
}

/// Dove gira un agente con la VM attiva.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRunner {
    /// Un agente di `sbx` in una sandbox tutta sua (`atollo-a-`): Claude, Codex.
    SbxAgent,
    /// La sua build per Linux dal kit, nella sandbox del progetto (`atollo-p-`): Pi, Antigravity.
    ProjectVm,
}

/// Chi chiede. Il livello ne segue (I2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Origin {
    // Livello 1
    KitTerminal { terminal: String },
    EditorTerminal,
    // Livello 2
    ProjectTerminal,
    EditorTask { command: String },
    /// Una riga di comando del progetto; `web`: ascolta su una porta (`Kind::Web`).
    Action { action: String, web: bool },
    /// Un'azione servita da un server web del kit (statico, Apache, Caddy).
    Site { action: String },
    Pool { pool: String },
    Scaffold { base: String },
    // Livello 3
    AgentButton { agent: Agent },
    EditorAgentProfile { agent: Agent },
    Forwarded { agent: Agent, from: Level },
    EditorExtension { extension: String, agent: Agent },
}

impl Origin {
    pub const fn level(&self) -> Level; // tabella in 3.4, colonna «Livello»
    pub fn agent(&self) -> Option<Agent>; // Some solo per le quattro origini del livello 3
}

/// Dove sta la richiesta, ricavato da una cartella con `place_of` (mai da un
/// nome scritto da chi chiama: un progetto è una cartella vera sotto
/// `projects/`, non un link, D-77).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Place {
    /// `dir` è `projects/<name>`; `below` la parte sotto ("web/src"), con `/`.
    Project { name: String, dir: PathBuf, below: Option<String> },
    /// Qualsiasi altra cartella del PC.
    Folder { dir: PathBuf },
}

impl Place {
    /// La cartella ricavata: quella del progetto con `below`, o la cartella.
    pub fn dir(&self) -> PathBuf;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub origin: Origin,
    pub place: Place,
}

/// La VM come l'ha vista l'ultimo stato.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmState {
    On,
    Off { blockers: Vec<crate::isolation::Blocker> },
}

/// Tutto ciò che serve a `decide` oltre alla richiesta, letto una volta da `levels::facts`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    pub vm: VmState,
    /// La cartella del luogo come la vede la VM (`vm::guest_path`): per un
    /// progetto la sua cartella, senza `below`; `None` per un percorso che la
    /// VM non monta (una cartella di rete).
    pub guest_dir: Option<String>,
    /// `isolation::restart_pending`: il daemon del kit deve ripartire prima di aprire una VM.
    pub restart_pending: bool,
    /// Per gli agenti `AgentRunner::ProjectVm`: il catalogo ha una build per il
    /// target Linux della VM (`Component::is_available_on(guest)`), installata o installabile.
    /// `false` per ogni altra origine (allora non si legge).
    pub linux_build: bool,
    /// Gli strumenti del kit per il Linux della VM (`kit.for_target(guest).apps()`),
    /// montati in sola lettura; vuoto dove il PC non ha VM. Sta in `Facts`
    /// perché `decide` non ha un `Kit`.
    pub guest_tools: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "where", rename_all = "camelCase")]
pub enum Placement {
    Pc {
        level: Level,                // Manual o Project, mai Agent (I3)
        project: Option<String>,     // il progetto del luogo, se c'è
        dir: PathBuf,                // dove parte il processo
    },
    Vm {
        level: Level,                // Project o Agent
        project: String,
        sandbox: Sandbox,
        /// La cartella di lavoro nella VM (cartella del progetto + `below`).
        workdir: String,
        /// Cartelle montate: il progetto rw, gli strumenti del kit per Linux ro.
        mounts: Mounts,
        /// Quali porte del PC può raggiungere la sandbox (i numeri li trova `prepare`).
        host: HostAccess,
        /// Se una porta della VM si pubblica sul PC (azioni web).
        publish: bool,
        program: VmProgram,
    },
    Refused {
        level: Level,
        reason: Refusal,
        fix: Fix,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Sandbox {
    /// `atollo-p-<progetto>-<hash>` (`vm::name`).
    Project { name: String },
    /// `atollo-a-<agente>-<progetto>-<hash>` (`vm::agent_name`).
    Agent { name: String, agent: Agent },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Mounts {
    pub workspace_rw: PathBuf,       // la cartella del progetto
    pub tools_ro: Vec<PathBuf>,      // data/apps/linux-x64 o linux-arm64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct HostAccess {
    pub databases: bool,             // i database del progetto sui server del kit (D-171)
    pub local_models: bool,          // llama-server, solo Pi, quando usa un modello locale (D-178)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum VmProgram {
    Shell,                                   // `bash -l` interattivo
    Line { command: String },                // task dell'editor: `sh -lc`
    Scaffold { base: String },               // le righe della base, ognuna con il PowerShell del kit per Linux via `sbx exec` (`start::scaffold_command`)
    Action { action: String },               // `sh -lc`, annotato (vm::tracked)
    Site { action: String },                 // un container del Docker della VM (vm::web_line)
    SbxAgent { agent: Agent },               // `sbx run --name <atollo-a-…>`
    AgentBinary { agent: Agent },            // pi / agy, build per Linux, `sbx exec -it`
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum Refusal {
    /// Livello 3 senza VM: gli agenti girano solo nella VM.
    AgentsNeedVm { blockers: Vec<crate::isolation::Blocker> },
    /// Livello 3 fuori dalla cartella di un progetto.
    AgentOutsideProject,
    /// Livello 2 fuori dalla cartella di un progetto con la VM attiva (un task dell'editor in `projects/`).
    NotAProject,
    /// La cartella non ha un percorso nella VM (cartella di rete).
    NoPathInVm { dir: String },
    /// Un pool di Server Web con la VM attiva.
    PoolsOnlyWithoutVm,
    /// Pi o Antigravity non ha una build per il Linux della VM.
    NoLinuxBuild { agent: Agent },
    /// Il daemon del kit deve ripartire per un'impostazione di sicurezza (isolation::restart_pending).
    DaemonRestartPending,
}

impl Refusal {
    /// L'unico rimedio di ogni motivo (I7): AgentsNeedVm → OpenVmNotice;
    /// AgentOutsideProject, NotAProject → OpenFromProject; NoPathInVm →
    /// UseLocalDisk; PoolsOnlyWithoutVm → StartFromProject; NoLinuxBuild →
    /// None; DaemonRestartPending → RestartDaemon.
    pub fn fix(&self) -> Fix;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Fix {
    /// Apri l'avviso «Senza VM» (i motivi e i rimedi).
    OpenVmNotice,
    /// Apri l'agente dalla cartella di un progetto o dalla pagina del progetto.
    OpenFromProject,
    /// Sposta il progetto su un disco del PC.
    UseLocalDisk,
    /// Avvia il sito dalla pagina del progetto (gira nella VM del progetto).
    StartFromProject,
    /// Chiudi le sessioni nella VM, poi «Ricontrolla» nella finestra della VM.
    RestartDaemon,
    /// Nulla che l'utente possa fare in Atollo (nessuna build per Linux pubblicata).
    None,
}
```

### 3.3 Ingressi di `decide` e da dove vengono

```rust
// levels/place.rs
/// Il progetto di `dir`: la prima cartella sotto `kit.projects()`, letta senza
/// seguire i link (`safe_path::is_dir`, e nessun link da `projects/` fino a
/// `dir`: una cartella raggiunta attraverso un link non è di nessun
/// progetto); altrimenti `Folder`. `dir` si legge come testo (`\\?\` tolto,
/// `.`/`..` risolti); i nomi si confrontano come fa il file system del PC,
/// senza badare alle maiuscole su Windows e macOS, e il nome del progetto è
/// quello sul disco.
pub fn place_of(kit: &Kit, dir: &Path) -> Place;

// levels/facts.rs
/// Legge i fatti per `req`: `isolation::status(kit, host, env, false)` (tenuto
/// al più 60 s, `FRESH`), `vm::guest_path`, `isolation::restart_pending` e, per
/// un agente ProjectVm, la disponibilità nel catalogo su `vm::guest_target`.
pub fn facts(kit: &Kit, host: &Host, env: &KitEnv, req: &Request) -> Facts;
```

Chi avvia una creazione crea prima la cartella (vuota) del progetto nuovo, poi chiama `place_of`, così la creazione ha un `Place::Project` e la sua sandbox (R15).

Lo stato usato è quello del momento della richiesta: nessun livello resta più annotato in file (il `.level` di oggi di `isolation::session`, scritto all'apertura dell'editor, sparisce).

### 3.4 `decide`: la tabella delle decisioni

```rust
/// L'unica decisione (D-179): pura, totale, senza I/O.
pub fn decide(req: &Request, facts: &Facts) -> Placement;
/// `decide` con la riga (`Rule::R1` … `Rule::R17`) che ha dato il collocamento:
/// per le prove e il log.
pub fn decided(req: &Request, facts: &Facts) -> (Rule, Placement);
```

`Origin::level()`:

| Origine | Livello |
|---|---|
| `KitTerminal`, `EditorTerminal` | 1 Manuale |
| `ProjectTerminal`, `EditorTask`, `Action`, `Site`, `Pool`, `Scaffold` | 2 Progetto |
| `AgentButton`, `EditorAgentProfile`, `Forwarded`, `EditorExtension` | 3 Agente |

Regole, valutate dall'alto in basso; la prima che corrisponde dà il collocamento. «Project» nella colonna Luogo è `Place::Project`, «Folder» è `Place::Folder`, «qualsiasi» l'uno o l'altro.

| # | Livello | Origine | Luogo | `vm` | Altri fatti | Collocamento |
|---|---|---|---|---|---|---|
| R1 | 3 | qualsiasi | qualsiasi | Off | — | `Refused { AgentsNeedVm { blockers }, OpenVmNotice }` |
| R2 | 2 | `Pool` | qualsiasi | On | — | `Refused { PoolsOnlyWithoutVm, StartFromProject }` |
| R3 | 1 | qualsiasi | Project | qualsiasi | — | `Pc { Manual, Some(name), dir+below }` |
| R4 | 1 | qualsiasi | Folder | qualsiasi | — | `Pc { Manual, None, dir }` |
| R5 | 2 | qualsiasi | Project | Off | — | `Pc { Project, Some(name), dir+below }` |
| R6 | 2 | qualsiasi | Folder | Off | — | `Pc { Project, None, dir }` (sigillato senza progetto: un pool, un task fuori da un progetto) |
| R7 | 2 | qualsiasi | Folder | On | — | `Refused { NotAProject, OpenFromProject }` |
| R8 | 3 | qualsiasi | Folder | On | — | `Refused { AgentOutsideProject, OpenFromProject }` |
| R9 | 2, 3 | qualsiasi | Project | On | `restart_pending` | `Refused { DaemonRestartPending, RestartDaemon }` |
| R10 | 2, 3 | qualsiasi | Project | On | `guest_dir == None` | `Refused { NoPathInVm { dir }, UseLocalDisk }` |
| R11 | 3 | agente con `runner() == ProjectVm` | Project | On | `!linux_build` | `Refused { NoLinuxBuild { agent }, None }` |
| R12 | 3 | agente con `runner() == SbxAgent` | Project | On | — | `Vm { Agent, sandbox: Agent{agent_name(project, id)}, program: SbxAgent, host: {databases: true, local_models: false}, publish: false }` |
| R13 | 3 | agente con `runner() == ProjectVm` | Project | On | — | `Vm { Agent, sandbox: Project{name(project)}, program: AgentBinary, host: {databases: true, local_models: agent == Pi}, publish: false }` |
| R14 | 2 | `ProjectTerminal` | Project | On | — | `Vm { Project, sandbox del progetto, Shell, publish: false }` |
| R15 | 2 | `EditorTask { command }`, `Scaffold { base }` | Project | On | — | `Vm { Project, sandbox del progetto, Line { command }` (task) o `Scaffold { base }` (creazione)`, publish: false }` |
| R16 | 2 | `Action { action, web }` | Project | On | — | `Vm { Project, sandbox del progetto, Action, publish: web }` |
| R17 | 2 | `Site { action }` | Project | On | — | `Vm { Project, sandbox del progetto, Site, publish: true }` |

Totalità: il livello 1 è coperto da R3/R4 per ogni luogo e stato della VM; il livello 2 da R2, R5–R7, R9, R10, R14–R17 (ogni origine del livello 2 compare in R14–R17 salvo `Pool`, rifiutato da R2 con la VM attiva e collocato da R5/R6 senza); il livello 3 da R1, R8–R13. In ogni collocamento `Vm`, `mounts` è `{ workspace_rw: cartella del progetto, tools_ro: facts.guest_tools }` (`[kit.for_target(guest).apps()]`), `workdir` è `guest_dir` unito a `below`, e il livello 2 (R14–R17) ha `host: { databases: true, local_models: false }`. Il `dir` di un collocamento `Pc` è la cartella del luogo (`Place::dir`: quella del progetto unita a `below`, o la cartella). Il `fix` di un rifiuto è sempre `reason.fix()`. Le righe possono sovrapporsi (R2 e R7 per un pool fuori da un progetto, R9 e R10): vince la prima. La prova `every_request_has_exactly_one_placement` enumera tutte le origini × {progetto, sottocartella del progetto, cartella} × ogni combinazione di fatti (VM attiva, spenta con e senza motivi, `restart_pending`, `guest_dir`, `linux_build`, `guest_tools`) e verifica che una riga corrisponda, che la prima riga che corrisponde (la tabella scritta riga per riga nella prova) sia quella che riporta `decided`, e le invarianti (12.3).

Su `Forwarded { from }`: `from` serve solo al log e all'etichetta; non cambia mai la riga. Un inoltro esiste solo sul PC (livello 1, o livello 2 senza VM); le sessioni di livello 2 nella VM non hanno inoltri, quindi un agente non si avvia dall'interno della VM di un progetto (sezione 6.8).

### 3.5 `environment`: l'unico costruttore del `PATH` e dell'ambiente

```rust
// levels/env.rs

/// La variabile che ogni processo dei livelli 1 e 2 sul PC riceve: "manual" o
/// "project"; nella VM: "project" o "agent". La leggono profile.ps1 e bashrc (intestazione).
pub const LEVEL_VAR: &str = "ATOLLO_LEVEL";
/// Il progetto di un processo sul PC, quando ne ha uno (profile.ps1 e bashrc non lo indovinano più dalla cartella).
pub const PROJECT_VAR: &str = "ATOLLO_PROJECT";
/// Il nome di un terminale del kit (`Origin::KitTerminal`): la sua intestazione,
/// e il profilo del kit non riapplica le linee del progetto.
pub const TERMINAL_VAR: &str = "ATOLLO_TERMINAL";
/// `data/kit/forward`: gli inoltri, in testa a ogni `PATH` del PC (sostituisce `data/kit/no-agents`).
pub const FORWARD_DIR: &str = "forward";

/// Con cosa gira un collocamento.
pub enum Environment {
    /// L'ambiente del processo sul PC.
    Pc(KitEnv),
    /// L'ambiente di `sbx` sul PC (`isolation::sbx_session_env`, il corpo di
    /// `Sbx::session_env`: quello del kit, il temp del PC, `daemon::ENV`) e
    /// ciò che entra nella VM.
    Vm { sbx_env: KitEnv, guest: GuestTools },
}

/// L'ambiente di `placement` a partire da `base` del kit (costruito per il
/// target del PC). `terminal`: la scelta degli strumenti del terminale del kit,
/// solo per `Origin::KitTerminal`. `host`: le variabili del PC, per l'ambiente
/// del kit per il Linux della VM e la cartella temporanea di `sbx`.
/// `None` esattamente per `Placement::Refused`.
pub fn environment(
    kit: &Kit,
    base: &KitEnv,
    placement: &Placement,
    terminal: Option<&crate::editor::terminals::Terminal>,
    lang: &str,
    host: &Host,
) -> Option<Environment>;

/// La parte `Pc` (i passi 1–7 sotto), per i chiamanti che non hanno ancora
/// un `Placement` (passi 2–4 di 12.2).
pub fn pc_environment(
    kit: &Kit,
    base: KitEnv,
    level: Level,
    project: Option<&str>,
    terminal: Option<&crate::editor::terminals::Terminal>,
) -> KitEnv;
```

`Pc { level, project, .. }`, in quest'ordine (ogni passo è una funzione con un nome, provata da sola); applicato di nuovo al proprio risultato non cambia nulla (`the_environment_is_idempotent`):

1. `base` copiato.
2. Se `terminal` è un terminale dell'utente: la sua scelta di strumenti e gli strumenti esclusi tolti (`editor::terminals::apply_tools`: `versions::swap_lines` dalle linee del kit, poi `editor::terminals::strip`), come oggi. Restituisce le linee su cui è l'ambiente (quelle del kit, `versions::kit_choice`, con sopra quelle del terminale); il terminale predefinito non cambia nulla.
3. Se `project` è `Some`: `project_lines`, le linee del progetto (`versions::project_choice`) scambiate **a partire dalle linee del passo 2**, così uno strumento scelto dal progetto è alla linea del progetto in qualsiasi terminale aperto nella sua cartella; gli strumenti che il progetto non ha scelto tengono le linee del terminale (Q1, decisa). Uno strumento escluso dal terminale resta escluso: il progetto sceglie linee, non gli strumenti del terminale.
4. `seal(kit, level, project, env)`: fuori le variabili di `SEALED`; dentro il login MySQL del progetto quando `project` è `Some` (D-167). Il livello 2 non ne tiene nessuna. Il livello 1 tiene solo l'agente SSH dell'utente (`SSH_AGENT` = `SSH_AUTH_SOCK`), così Git su SSH in un terminale in cui scrive l'utente usa le chiavi che l'utente ha caricato (**deciso dall'utente, 26/10/2026**); a un client SSH non serve altro (`SSH_AGENT_PID` serve solo a `ssh-agent -k` e l'ambiente del kit non lo prende mai dal PC; su Windows l'agente di OpenSSH è una named pipe trovata senza variabile). Il percorso del profilo vero, il bus di sessione e il root di MySQL del kit restano fuori anche dal livello 1: il terminale predefinito del kit, che prima di D-179 li teneva, li perde; i terminali dell'utente, che prima di D-179 perdevano l'agente SSH, lo riavranno.
5. `strip_agents(kit, env)`: ogni voce di `agent_dirs(kit)` esce dal `PATH`, con le variabili che vi puntano (`CP_ANTIGRAVITY`); `agent_dirs` = `data/apps/<target-del-PC>/{claude,codex,antigravity,pi}` e `data/kit/agents`. Esce anche `forward_dir(kit)`, perché il passo 6 la metta una volta sola.
6. `forward_first`: `forward_dir(kit)` in testa al `PATH`: `data/kit/forward` dal passo 5 di 12.2; fino ad allora i sostituti di `data/kit/no-agents`.
7. `mark`: `LEVEL_VAR` = `manual` o `project`; `PROJECT_VAR` = il progetto quando `Some`, tolta altrimenti; `TERMINAL_VAR` = il nome del terminale del kit quando `terminal` è `Some`, tolta altrimenti.

`Vm { project, sandbox, program, .. }`:

I passi 1–3 sono `levels::env::guest_tools(kit, level, project, program, lang, host)`, che `prepare` richiama dopo aver installato ciò che mancava (uno strumento installato allora è nel `PATH`):

1. `guest = kit.for_target(vm::guest_target(kit.target()))`; `base_guest = KitEnv::build(&guest, settings, lang, host)`, con `settings` letto da `Settings::load(kit)`.
2. `isolation::project_guest_tools(kit, &guest, project, base_guest)`: `pc_environment` di `guest` al livello 2 (le linee del progetto, sigillato, agenti tolti: il passo 5 applicato a `guest`), poi `guest_view`: solo le voci del `PATH` e le variabili dentro `data/apps/linux-…`, come le vede la VM. Per `AgentBinary { agent }` la cartella Linux dell'agente stesso torna, in fondo, in `guest.path` (è il programma che si esegue, non un comando offerto ad altri).
3. `-e ATOLLO_LEVEL=project|agent` aggiunto a `guest.vars`.
4. `sbx_env = isolation::sbx_session_env(base, host)`.
5. `guest.env_file` lo imposta `prepare` (le variabili del database, D-171), mai `environment`.

`Refused` → `None`.

Le funzioni di oggi (passo 2 di 12.2): `versions::project_env(kit, p, env)` era un involucro sottile di `pc_environment(kit, env, Project, Some(p), None)` per i suoi molti chiamanti, tolto con gli ultimi chiamanti dal passo 4; `versions::choice_env` ed `editor::terminals::env` non ci sono più: `terminal_launch` chiama `pc_environment(kit, base, Manual, il progetto di place_of(dir), Some(terminal))`.

### 3.6 `prepare` e `start`

Firme in [LEVELS.md, 3.6](LEVELS.md#36-prepare-and-start): il tratto `Effects` (`ensure_guest_tools`, `start_databases`, `start_local_models`, `start_sandbox`, `watch_local_models`: ciò che `prepare` chiede all'app, così si prova senza Tauri e senza `sbx`; quelli dell'app sono `ipc::levels::AppEffects`), `LocalModels { port, key }`, `Ready { placement, env, vm }`, `prepare(fx, kit, host, placement, env, needs, lang) -> anyhow::Result<Ready>`, `vm_session_args`, `start_terminal`, `open_terminal`, `start_action`, `scaffold_command`, `scaffold_end`.

- `prepare` per `Vm`, in quest'ordine: gli strumenti per il Linux della VM (D-170: la scelta del kit, `needs`, il PowerShell del kit per una creazione, la build dell'agente stesso per `AgentBinary`, alle linee del progetto) con `fx.ensure_guest_tools`, poi `guest_tools` riletto; i database (`fx.start_databases`) e `db.env` (D-171); l'`isolation::Vm` costruita dal collocamento (`isolation::project_vm`, poi `Vm::for_agent` per la sandbox di un agente; senza lo `sbx` del kit è un errore, mai il PC); i modelli locali per Pi (`fx.start_local_models`, `pi.env`, la porta, D-178); `info.json` (§8.4); `fx.start_sandbox`; `fx.watch_local_models` per una porta che non è di un database. Per `Pc`: nulla (l'`ensure` del chiamante è già avvenuto prima di `environment`, che si costruisce da ciò che restituisce). `Refused`: un errore.
- `start_terminal`: la sessione pty di un collocamento da terminale (Shell, SbxAgent, AgentBinary, o una shell del PC), con il suo ambiente e la sua cartella; il suo `PtyInfo` porta `level`, `label` e `sandbox` (8.3). `open_terminal` la apre nel terminale dell'app; la sessione di Pi tiene aperta la porta dei modelli mentre gira.
- `start_action`: un'azione o un sito come `ready` li ha collocati: `Runner::start_in` con l'ambiente `Pc`, o nella VM con l'ambiente di `sbx` e la `Vm` avviata (`vm::exec_args`/`vm::web_line` con `Vm::command`, la porta pubblicata con `Vm::publish`; la sandbox non si avvia più lì).
- `scaffold_command`: una riga della creazione in `at` con `vars`; PC: `frameworks::line_command` (il PowerShell del kit, l'ambiente `Pc`); VM: `vm::scaffold_args`, il PowerShell del kit per il Linux della VM con `sbx exec -w <at nella VM>`, `vars` con le variabili dei database in `scaffold.env` (leggibile solo dal proprietario, mai sulla riga di comando); la esegue `frameworks::run_command`. `scaffold_end` toglie `scaffold.env`.

Una creazione nella VM (R15): le righe del catalogo sono di PowerShell (`frameworks.json`: «one command run with the kit's PowerShell»), quindi ognuna gira con il PowerShell del kit per il Linux della VM, non con `sh -lc`. Le righe `create` fanno `{name}` nella cartella corrente, che sul PC è `projects/`; la VM raggiunge solo la cartella del progetto, quindi girano in `<progetto>/.atollo-new` (`ipc::frameworks::VM_STAGING`), e ciò che hanno fatto si sposta nel progetto (i file già presenti restano e si elencano, come in D-90), la cartella di appoggio si toglie. La cartella del progetto, area di lavoro della sandbox, non si toglie e rifà mai come sul PC. Un database del piano si crea sul server del kit, poi `prepare` gira di nuovo (la sua porta aperta per la sandbox, `db.env` riscritto) e i passi ricevono `ATOLLO_DB_*` con la strada della VM verso il PC.

Cosa resta dove:

| Oggi | Destino |
|---|---|
| `isolation::vm_for` | fatto nel passo 4: `isolation::project_vm` (`pub(crate)`, senza chiedere lo stato: l'ha già fatto la decisione), chiamata solo da `levels::prepare` (una `Vm` nasce solo da un `Placement::Vm`) |
| `isolation::Vm::for_agent`, `agent_session_args`, `launch`, `start`, `publish`, `stop_run` | restano in `isolation` (vita della sandbox); chiamate solo da `levels::prepare`/`start` e da `projects::run` (arresto) |
| costruttori `isolation::vm::*` | restano (costruttori puri degli argomenti di `sbx`, regola di D-175) |
| `isolation::seal`, `SEALED`, `agent_dirs`, `without_agents` | vanno in `levels::env` (`without_agents` diventa `strip_agents` e non mette più i sostituti: lo fa il passo 6) |
| `isolation::agents_notice`, `with_agents_notice`, `AGENTS_NOTICE_VAR` | tolte; gli inoltri ricevono il motivo dalla risposta (4.3) |
| `isolation::session` (tutto il file) | sostituito da `levels::requests` |
| `ipc::terminals::{project_vm_ready, project_vm_tools, project_vm_dbs, ensure_guest_tools}` | fatto nel passo 4: le prime tre dentro `levels::prepare`; `ensure_guest_tools` resta in `ipc::terminals` come `Effects::ensure_guest_tools` dell'app (e per «Installa i mancanti» con la VM) |
| `ipc::agents::{agent_vm_ready, refused_on_project, no_vm_for_agents, pi_local_models, watch_agent_requests}` | passo 4: `agent_vm_ready`, `pi_local_models` → `levels::prepare` (la sorveglianza della porta dei modelli resta: `pi_port_touch` in `prepare`, `watch_pi_port` dietro `Effects`, `pi_session` in `start::open_terminal`); `no_vm_for_agents` → il testo di R1 in `levels::labels`; `refused_on_project` resta per Antigravity fino al passo 7; `watch_agent_requests` passa da `decide(EditorAgentProfile)` → `prepare` → `start::vm_session_args` finché il passo 5 non la sostituisce con `levels::requests::serve` |
| `ipc::editing::{editor_vm_begin, editor_vm_session}` | tolte nel passo 5 (nessuna sessione della VM per l'editor: ogni richiesta di task o agente si decide quando arriva); nel passo 4 passano da `decide(ProjectTerminal)` → `prepare` |
| `Runner::start` (l'involucro sul PC di `start_in`), `frameworks::run_line` | tolte nel passo 4: `levels::start::start_action` chiama `Runner::start_in`; `frameworks::run_line` divisa in `line_command` (costruita da `levels::start::scaffold_command`) e `run_command` |
| `editor::terminal::open_console` | tolta nel passo 4: `levels::start::start_terminal` (`pty::shell`) e `open_terminal` (`open_launch_then`) |
| `ipc::servers::refused_with_vm` | tolta (R2) |
| `vm::starts_in_vm`, `vm::sbx_agent` | sostituite da `Agent::runner()` |
| `setup::kit_files::{NO_AGENTS_DIR, AGENT_COMMANDS, no_agent_cmd, no_agent_sh, AGENTS_NOTICE_FALLBACK}` | sostituite da `FORWARD_DIR` e dai file degli inoltri (6.8) |
| `runtime/vm.ps1` | sostituito da `runtime/launch.ps1` e `runtime/launch.sh` (solo client delle richieste, 4.3) |

## 4. Contratto IPC

Tre canali, ciascuno con il suo contratto: comandi ed eventi di Tauri (le finestre dell'app), i file delle richieste (i processi fuori dall'app: i profili dell'editor e gli inoltri) e la riga di comando.

### 4.1 Comandi di Tauri

I tipi TypeScript (`ui/src/ipc/levels.ts`, nuovo) sono in [LEVELS.md, 4.1](LEVELS.md#41-tauri-commands): `Level`, `AgentId`, `LevelLabel` (`"pc" | "projectVm" | "agentVm"`), `LevelsState { vm, reason, agentsEnabled }`, `Refusal` (per `code`), `Fix`, `RefusedError { message, refusal, fix }`.

| Comando | Richiesta | Risposta | Stato |
|---|---|---|---|
| `levels_state` | `{}` | `LevelsState` | **nuovo** |
| `project_vm_report` | `{ project: string; check: boolean }` | `VmReport` (8.1) | **nuovo** |
| `terminal_launch` | `{ id: string; dir?: string }` | `void` | resta; passa da `decide(KitTerminal)` |
| `terminal_open` | `{ project: string }` | `void` | resta; `decide(ProjectTerminal)` |
| `run_start` | `{ project: string; action: string }` | `RunInfo` | resta; `decide(Action|Site)`, i pool dopo R5 |
| `server_start`, `server_reload` | `{ id: string }` | `void` | restano; `decide(Pool)` (R2 sostituisce `refused_with_vm`) |
| `frameworks_create` | come oggi | `string` | resta; `decide(Scaffold)` (nella VM se attiva: nuovo) |
| `agent_launch` | `{ agent: AgentId; project: string }` | `void` | resta; `decide(AgentButton)` |
| `agents_status` | `{}` | `AgentInfo[]` con i campi nuovi `available: boolean`, `unavailableReason: string \| null`, `runsIn: "sbxSandbox" \| "projectVm"` | cambia |
| `editor_open` | `{ project: string }` | `void` | resta; non prepara più una VM né scrive `.level` |
| `pty_sessions`, evento `pty-opened` | — | `PtyInfo` con i campi nuovi `level: Level`, `label: LevelLabel`, `sandbox: string \| null` | cambia |
| `isolation_status` | `{ force: boolean }` | `IsolationStatus` | resta (il campo `level` diventa `availability` al passo 10) |
| `agent_token_link` | — | — | **tolto** (esegue `claude setup-token` sul PC: livello 3 sul PC, I3) |
| `agent_token_save` | `{ token }` | `void` | resta solo come dato del kit; il token non va mai a una VM (I5); nascosto nell'interfaccia **[ipotesi]** |
| `agent_sign_out` | `{ agent }` | `void` | resta solo per i dati del kit; `codex logout` non si esegue più (avvia il programma dell'agente sul PC); `auth.json` si toglie direttamente |
| `components_apply`, `components_install`, `versions_install`, `updates_list`, `setup_start` | come oggi | come oggi | agenti esclusi quando `agentsEnabled == false` (sezione 7) |
| `vms_list`, `vm_remove`, `isolation_run` | come oggi | come oggi | invariati |

Ogni comando rifiutato risponde con `CmdError` con i nuovi campi facoltativi `refusal` e `fix` (`ipc/error.rs`), così l'interfaccia mostra il pulsante giusto senza leggere il testo. Sul filo (fatto nel passo 3): un `CmdError` senza rifiuto resta la stringa semplice di sempre, così ogni altro errore arriva all'interfaccia come prima; un rifiuto è l'oggetto `RefusedError` (`{ message, refusal, fix }`). L'interfaccia legge entrambi con `errorText(e)`/`fixOf(e)` (`ui/src/ipc/levels.ts`); `useAction` tiene il `fix` accanto all'errore ed `ErrorLine` mostra l'azione dei rimedi che l'app sa fare: «Apri l'avviso VM» per `open_vm_notice` e `restart_daemon` (nell'avviso c'è «Ricontrolla»); gli altri rimedi li dice il testo del rifiuto. Lato Rust: `ipc/levels.rs` (`admitted`, `pc_env`, `placed`, `place_now`), con una funzione che fa la richiesta per ogni punto di avvio (`kit_terminal_request`, `project_terminal_request`, `action_request`, `pool_request`, `scaffold_request`). Un posto `Vm` senza `sbx` nel kit è un errore («Docker Sandboxes (sbx) non è nel kit.»), mai un avvio sul PC (I7).

### 4.2 Eventi

| Evento | Contenuto | Quando |
|---|---|---|
| `levels-changed` | `LevelsState` | **nuovo**: la VM si è accesa o spenta (`levels::watch`, un filo che chiede `isolation::status` ogni 60 s e a ogni controllo forzato, ed emette solo al cambiamento) |
| `pty-opened`, `pty-exit`, `pty-moved` | `PtyInfo` (con `level`, `label`, `sandbox`) | contenuto cambiato |
| `job` | `Job` (§8.6) | **nuovo**: un'installazione, un aggiornamento o la preparazione di una VM è cambiata (in coda, un passo, i suoi byte, la fine); con i comandi `jobs_list` e `job_cancel` |

### 4.3 File delle richieste (editor e inoltri)

I processi fuori dall'app (i profili di VSCodium, gli inoltri scritti in un terminale) non possono chiamare i comandi di Tauri. Usano file in `data/app/levels/requests/` (sostituisce `data/app/sandboxes/sessions/`), scritti e letti dallo stesso utente.

La richiesta, scritta dal client come `<id>.json` (`<id>`: 32 cifre esadecimali, a caso per ogni richiesta), in modo atomico (scrive `<id>.tmp`, poi rinomina); il formato è quello di [LEVELS.md, 4.3](LEVELS.md#43-request-files-editor-and-forwarders): `v`, `origin` (`editorTask` con `command`, `editorAgentProfile` con `agent`, `forwarded` con `agent` e `from`, `editorExtension` con `extension` e `agent`), `cwd`, `args` (gli argomenti scritti dopo il comando dell'agente). Le origini del livello 1 non mandano mai una richiesta: girano sul PC senza chiedere (R3/R4).

L'app (`levels::requests::serve`, che guarda ogni 250 ms, un filo per richiesta) toglie la richiesta, ricava `place_of(cwd)`, legge i fatti, chiama `decide`, `environment`, `prepare` e scrive `<id>.reply.json`:

- `placement: "vm"`: `level`, `label`, `program` (l'`sbx` del kit), `argv`, `env` (`set`, `unset`: le variabili per `sbx`), `banner`;
- `placement: "pc"` (un `editorTask` senza VM): `program` = la shell del kit, `env` = la differenza fra l'ambiente del server dell'editor e `environment(Pc{Project})` (gli scambi del progetto e il suo login MySQL; l'ambiente del server dell'editor è già di livello 1, 6.2);
- `placement: "refused"`: `refusal`, `fix`, `text` (motivo e cosa fare, nella lingua dell'app).

Il client (`runtime/launch.ps1`, PowerShell 7, il componente obbligatorio; `runtime/launch.sh`, sh POSIX, per bash senza PowerShell) scrive la richiesta, aspetta la risposta al più 10 minuti (il primo avvio di una VM scarica la sua immagine), scrive `banner` sullo standard error, applica `env` ed esegue `program argv…` nello stesso terminale con gli stessi ingressi e uscite, ed esce con il suo codice. Per un rifiuto scrive `text` ed esce con 1. Senza risposta entro 15 s e senza un'app in esecuzione (il blocco `data/app/instance` libero) scrive «Atollo non è aperto: apri il terminale da Atollo.» ed esce con 1. Il client non decide nulla: la sua logica è scrivere, aspettare, eseguire.

Chi può scrivere richieste: qualsiasi processo dell'utente, come oggi (`<progetto>.<agente>.request`). Una richiesta può causare solo ciò che l'utente farebbe dall'app: una sessione che `decide` permette. Un processo dentro una VM non può scriverne: `data/app` non è mai montata (D-168).

### 4.4 Riga di comando

| Comando | Uscita | Codice |
|---|---|---|
| `atollo vm <progetto> [--check] [--json]` | il rapporto della VM (8.2) | 0 tutto bene; 1 VM non attiva, sandbox assente o un controllo non `ok`; 2 uso sbagliato |
| `atollo env --project <p>` | l'uscita di oggi, ora `environment(Pc{Project, p})`, più `ATOLLO_LEVEL=project` | invariato |

Gli inoltri non passano dalla riga di comando: `Atollo.exe` è un programma del sottosistema grafico di Windows (`main.rs`, `windows_subsystem = "windows"`), e far girare come suo figlio un `sbx exec -it` interattivo da una console è qualcosa che il codice oggi non fa **[non verificato]**; i client PowerShell e sh eseguono `sbx` da sé, come fa oggi `vm.ps1` (provato, D-175).

## 5. Inventario dei punti di avvio

Ogni punto del codice di oggi che avvia un terminale, un task, un'azione, un sito, un pool, una creazione o un agente (ricerca di `open_console`, `open_launch`, `pty::shell|command|program`, `kit_command`, `spawn_in`, `Command::new`, `sbx exec|run`, `vm_for`, `seal`, `without_agents`, `project_env`, `choice_env` e dei profili dell'editor), il 26/10/2026, ramo `feature/levels` = `main` a `c9aa2561`.

### 5.1 Livelli 1–3

| # | Punto di avvio (file: funzione) | Avvia | Livello | Oggi | Con `levels` |
|---|---|---|---|---|---|
| E1 | `ipc/terminals.rs: terminal_launch` → `editor/terminal.rs: open_console` | terminale del kit (pty) | 1 | PC; `terminals::env` (scelta, `without_agents`, sostituti); `with_agents_notice`; **niente versioni del progetto** (`ATOLLO_TERMINAL` fa saltare gli scambi a profile.ps1 e bashrc) | `decide(KitTerminal)` → R3/R4 → `environment(Pc, terminal)` → `start_terminal` |
| E2 | `editor/cfg.rs: kit_profiles` («Atollo», «Atollo - Git Bash», «Atollo - cmd», «Atollo - Bash», «Atollo - zsh») → `runtime/vm.ps1 -Shell` | terminale dell'editor | 1 | solo nella VM (D-178), chiuso per difetto; sul PC nulla | shell del kit diretta (`pwsh_args`/`bash_args`), nessuna richiesta; `decide(EditorTerminal)` è R3/R4 per costruzione; l'ambiente del server dell'editor è già di livello 1 (E13) |
| E3 | `ipc/editing.rs: terminal_open` | terminale del progetto | 2 | `vm_for` → `project_vm_ready` → `kit_terminal_args`; altrimenti `open_console` con `project_env` | `decide(ProjectTerminal)` → R14 o R5 |
| E4 | `ipc/runs.rs: run_start` (ramo VM) → `projects/run/start.rs: Runner::start_in` → `in_vm` (`exec_args`, `web_line`, `publish`) | azione / sito nella VM | 2 | `vm_for` → `project_vm_ready` | `decide(Action|Site)` → R16/R17 → `prepare` → `start_action` (passo 4) |
| E5 | `ipc/runs.rs: run_start` (ramo PC) → `Runner::start_in` (`Atollo.exe serve`, `projects/apache.rs: command`, `projects/caddy/command.rs`, `projects/fastcgi.rs: spawn_fastcgi_if_present`, comando semplice) | azione / sito sul PC | 2 | `project_env` | R5 → `environment(Pc{Project})` |
| E6 | `ipc/runs.rs: run_start` (ramo pool) → `servers.attach` → `servers/process.rs` (avvio del pool, `without_agents`) | sito in un pool | 2 | PC; con la VM vince prima il ramo VM | prima `decide(Site)`; solo con R5 il pool |
| E7 | `ipc/servers.rs: server_start`, `server_reload` (`refused_with_vm`) → `servers/process.rs` | pool | 2 | rifiutato con la VM | `decide(Pool)` → R2 o R6 |
| E8 | `app/mod.rs`, avvio: `servers.autostart` (saltato con la VM) | pool all'avvio dell'app | 2 | stesso controllo in linea | `decide(Pool)` per ogni pool |
| E9 | `ipc/frameworks.rs: frameworks_create` → `projects/frameworks/runner.rs: run_line` (pwsh) | creazione | 2 | **sempre sul PC**, `seal(None)` | `decide(Scaffold)` → R15 (sandbox del progetto nuovo) o R5 |
| E10 | `ipc/agents.rs: agent_launch` → `agent_vm_ready` → `Vm::launch` → `open_launch` | agente dall'app | 3 | solo VM; Antigravity rifiutato | `decide(AgentButton)` → R1/R8–R13 |
| E11 | `ipc/agents.rs: watch_agent_requests` ← `vm.ps1 -Agent` (`<progetto>.<agente>.request`) ← `editor/cfg.rs: kit_profiles` (Claude Code, Codex, Antigravity, Pi) | agente dall'editor | 3 | solo VM, serve una sessione dell'editor su quel progetto | `levels::requests::serve` → `decide(EditorAgentProfile)` |
| E12 | `editor/cfg.rs: merge` (`automationProfile`) → `vm.ps1 -Command` | task dell'editor | 2 | solo VM; senza VM nulla | richiesta `editorTask` → R15 o R5/R6 |
| E13 | `editor/mod.rs: Editor::start_on` (`kit_command` con `self.env()` = l'ambiente completo del kit) | server dell'editor e il suo extension host | kit, ma ospita le estensioni | **agenti nel `PATH`** (`env/path.rs` aggiunge `data/kit/agents`, `claude`, `codex/bin`, `codex/codex-path`, `pi`) | ambiente del server = `environment(Pc{Manual, None})` (niente agenti, inoltri in testa) |
| E14 | `tauri/catalog/extensions.json` (`anthropic.claude-code`, generale) → installazione in `editor/ext.rs` | estensione Claude Code, la sua CLI incorporata, sul PC | 3 | installata quando `claude` è nel kit; gira sul PC | sezione 10 |
| E15 | `setup/kit_files.rs: no_agent_cmd/no_agent_sh` (`data/kit/no-agents`) | sostituti | 3 (solo rifiuto) | scrivono un testo fisso, escono con 1 | inoltri (6.8) |
| E16 | `agents/claude_token.rs: link` (`pty::command` di `claude setup-token`, macOS) da `ipc/agents.rs: agent_token_link` | programma dell'agente sul PC | 3 | PC | tolto (I3) |
| E17 | `agents/mod.rs: codex_logout` (`kit_command(codex, …) logout`) da `agent_sign_out` | programma dell'agente sul PC | 3 | PC | tolto; `auth.json` cancellato direttamente |
| E18 | `editor/cfg.rs: kit_profiles` e `merge` quando manca la PowerShell del kit | i profili degli agenti eseguono `claude.exe`/`codex.exe`/`agy`/`pi.exe` direttamente; i task `/bin/bash` | 3 / 2 | **sul PC** | nessun profilo di agente e nessun profilo dei task senza la PowerShell del kit o `launch.sh` (I3) |
| E19 | gli adattatori del debugger dell'editor e le estensioni che eseguono gli interpreti del progetto (`ms-python.python`, `xdebug.php-debug`…) | codice del progetto sul PC | 2 | PC (limite dichiarato in D-175) | limite dichiarato mantenuto, domanda Q14 |
| E20 | `main.rs: Cli::Env { project }` | scrive un ambiente (nessun processo) | 2 | `project_env` | `environment(Pc{Project})` |

Conteggio: livello 1: 2 (E1, E2); livello 2: 12 (E3–E9, E12, E19, E20, più il ricaricamento e l'avvio automatico dei pool contati in E7/E8 come punti a sé: `server_start`, `server_reload`, `autostart`); livello 3: 6 (E10, E11, E14, E15, E16, E17) più il ripiego sul PC di E18 (livelli 2 e 3); E13 è il processo del kit che ospita E14 ed E19.

### 5.2 I programmi del kit (non sono livelli; esenti da I1 per nome)

| Programma | Dove parte |
|---|---|
| server e client dei database | `db/services/{process,access,data,tools}.rs` |
| phpMyAdmin, mongo-express | `db/phpmyadmin.rs`, `db/mongo_express.rs` |
| llama-server | `llm/service.rs` |
| Git (clone, `ls-files`, identità), gh | `github/mod.rs`, `github/repos.rs`, `projects/mod.rs`, `projects/frameworks/mine.rs` (con `-c core.fsmonitor=false`, D-167) |
| installatori | `install/**`, `setup/checkup.rs` |
| server dell'editor, installazione delle estensioni, tema | `editor/mod.rs`, `editor/ext.rs`, `editor/theme.rs` |
| `sbx` (stato, login, ls, daemon) | `isolation/**`, `ipc/isolation.rs: isolation_run` |
| aiuti del portachiavi di Antigravity | `os/macos/agy_keyring.rs`, `agents/keyring` (nessun agente avviato) |

La prova di regola (12.3, `no_process_outside_levels`, accesa dal passo 4) cerca in `tauri/src` (commenti esclusi) due volte. I varchi: `open_launch`, `open_launch_then`, `pty::shell`, `pty::command`, `pty::program`, `.start_in`, `line_command`, `scaffold_args`, `kit_terminal_args`, `pi_session_args`, `agent_session_args`, `exec_args`, `web_line`, ammessi solo in `levels/`, nelle loro definizioni (`editor/terminal.rs`, `editor/pty.rs`, `isolation/vm.rs`, `isolation/mod.rs`, `projects/run/start.rs`, `projects/frameworks/runner.rs`) e nelle eccezioni nominate: `ipc/isolation.rs` (`isolation_run`, lo `sbx` del kit), `isolation/session.rs` (E11/E12, fino al passo 5), `agents/claude_token.rs` (E16, fino al passo 6). Gli avvii di processi: `Command::new`, `kit_command`, `spawn_in`, `shell_command`, ammessi solo in `levels/`, nello strato del sistema (`platform/`, `os/`), nei file della tabella sopra, e nei programmi di livello 2 che avvia solo un punto di avvio deciso (`projects/run/start.rs`, `projects/apache.rs`, `projects/caddy/command.rs`, `projects/fastcgi.rs`, `projects/frameworks/runner.rs`, `servers/process.rs`), più `agents/mod.rs` (E17, fino al passo 6). Ogni eccezione è scritta nella prova con il suo motivo.

## 6. Flussi

I diagrammi dei flussi sono in [LEVELS.md, 6](LEVELS.md#6-flows), uguali per le due lingue (nomi del codice). In breve:

- **6.1 Terminale del kit (livello 1)**: `terminal_launch` → `place_of` → `decide(KitTerminal)` = `Pc{Manual}` (R3/R4) → `environment` (versioni del terminale, poi del progetto; sigillato; niente agenti; `forward/` in testa) → `start_terminal` (`PtyInfo{level: manual, label: «PC»}`) → intestazione di profile.ps1/bashrc «PC · <terminale> · <progetto>».
- **6.2 Terminale dell'editor (livello 1)**: il server dell'editor ha l'ambiente `environment(Pc{Manual, None})`; il profilo «Atollo» apre la PowerShell del kit con `profile.ps1`, senza richiesta; `profile.ps1` ricava il progetto dalla cartella (stessa regola di `place_of`) e applica le sue versioni; `claude` scritto lì → 6.8.
- **6.3 Task dell'editor (livello 2)**: `automationProfile` = `launch.ps1 -Origin editorTask -Command "<riga>"` → richiesta → `decide`: VM attiva e progetto → `Vm{Line}` (R15), risposta con `exec_args`; VM non attiva → `Pc{Project}` (R5), risposta con la shell del kit e la differenza d'ambiente; VM attiva fuori da un progetto → rifiuto `NotAProject`. Il client esegue nello stesso terminale.
- **6.4 Azione (livello 2)**: `run_start` → origine `Site{a}` per un'azione con `static_dir`/`apache_dir`/`caddy_dir`, altrimenti `Action{a, web}` → `Vm` (R16/R17: `prepare`, `Runner::start_in` con `exec_args`/`web_line`, porta pubblicata) o `Pc` (R5: pool o server suo, oppure comando) o rifiuto (R9/R10).
- **6.5 Sito in un pool (livello 2)**: `server_start` → `decide(Pool)`: VM attiva → `Refused{PoolsOnlyWithoutVm, StartFromProject}` (R2); VM non attiva → `Pc{Project, None}` (R6).
- **6.6 Agente dalla pagina del progetto (livello 3)**: `agent_launch` → VM non attiva: `Refused{AgentsNeedVm}` (R1), l'interfaccia mostra motivo e «Apri l'avviso VM»; `claude`/`codex` → `Vm{Agent, Sandbox::Agent, SbxAgent}` (R12: `Vm::for_agent`, strumenti, `db.env`, porte, avvio, `agent_session_args`); `pi`/`antigravity` → `Vm{Agent, Sandbox::Project, AgentBinary}` (R13: build per Linux, D-173, strumenti, `db.env` e per Pi i modelli locali, avvio, `pi_session_args`/`agy_session_args`); sessione nel terminale dell'app con `PtyInfo{level: agent, label: «Agente · VM», sandbox}`.
- **6.7 Agente dal profilo dell'editor (livello 3)**: il profilo «Claude Code» = `launch.ps1 -Origin editorAgentProfile -Agent claude` (presente solo con `agentsEnabled`, sezione 7) → come 6.6 → risposta con `sbx` e `argv`. Diversamente da oggi non serve una sessione dell'editor su quel progetto: decide `cwd`.
- **6.8 `claude` scritto in un terminale sul PC (inoltro)**: in `data/kit/forward/` (scritti da `setup::kit_files`, al posto di `data/kit/no-agents`), per ogni comando di `Agent::commands()`: `<cmd>.ps1` (`& "$env:CP_DATA/kit/launch.ps1" -Origin forwarded -Agent <id> -From $env:ATOLLO_LEVEL -- @args`; PowerShell trova `.ps1` nel `PATH` prima di `.cmd` **[non verificato con pwsh 7 quando ci sono tutti e due; da provare]**), `<cmd>.cmd` (`pwsh … -File "%CP_DATA%\kit\launch.ps1" … -- %*`; argomenti con `"`, `%` o `^` possono cambiare: limite dichiarato, Q16), `<cmd>` per sh (`exec "$CP_DATA/kit/launch.sh" forwarded <id> "$ATOLLO_LEVEL" "$@"`). Nella cartella di un progetto con la VM: R12/R13, stesso terminale, ingressi e uscite passati; senza VM: R1, «Gli agenti partono solo nella VM e la VM non è attiva: <motivo>. <cosa fare>», uscita 1; fuori da un progetto: R8, uscita 1. Da una sessione di livello 2 nella VM non c'è inoltro (`data/kit` non è montata e le cartelle Linux degli agenti sono tolte): lì `claude` dà «command not found» della VM **[non verificato: se l'immagine `shell` di `sbx` porta la CLI di un agente]**.
- **6.9 Estensione dell'editor (livello 3)**: Claude Code con `claudeCode.claudeProcessWrapper` = `data/kit/forward/claude-extension(.cmd|sh)`; l'estensione esegue `<involucro> <percorso-del-binario-incorporato> <suoi argomenti…>`; l'involucro toglie il primo argomento e poi fa come 6.8 con origine `editorExtension` → R12. Fattibilità **non verificata** (Q2): percorsi del PC negli argomenti, la connessione locale dell'estensione, `sbx run` senza TTY. Finché una prova non lo dimostra, l'estensione è **disattivata** nell'editor di Atollo (sezione 10).

### 6.10 Un agente scritto nel terminale «VM progetto» (deciso dall'utente, 26/10/2026)

Una VM non può chiedere al PC di aprire la sandbox di un agente, quindi nella VM non c'è inoltro. Il PATH della VM del progetto ha dei sostituti `claude`, `codex`, `pi`, `agy` (nella cartella in sola lettura del §8.4) che stampano «Gli agenti si avviano dalla pagina del progetto, dall'editor, o scrivendo il loro nome in un terminale del PC nella cartella del progetto» ed escono con 1. Non ci sono quando l'agente stesso gira lì (Pi e Antigravity con la loro build). Prova: `an_agent_typed_in_the_vm_says_where_to_start_it`.

## 7. Agenti disattivati senza VM

`agentsEnabled = (disponibilità di isolation == Vm)`. Lo decide una sola funzione, `levels::agents_enabled(&VmState) -> bool`; tutto ciò che segue la legge.

| Dove | Con la VM | Senza VM |
|---|---|---|
| Pagina Strumenti, gruppo «Agenti» (`ui/src/components/table.tsx`) | Installa/Aggiorna offerti: Pi e Antigravity per il Linux della VM (D-173 `Scope::Tools`); Claude Code e Codex segnati «fornito da Docker Sandboxes» (le immagini degli agenti di `sbx` li portano) **[ipotesi, Q11]** | Installa, Aggiorna, scelta della linea disattivati; suggerimento: «Gli agenti girano solo nella VM, che non è attiva: <motivo>» con «Apri l'avviso VM» |
| Piani del runner (`setup::runner`, `components_apply`, `components_install`, `versions_install`, `setup_start`, installazioni orarie e al primo avvio) | agenti previsti solo per il target della VM | `Group::Agents` escluso da ogni piano (un solo filtro, `levels::installable(component, &VmState)`) |
| Aggiornamenti (`updates_list`, riepilogo D-161) | come sopra | agenti non elencati |
| Pagina Agenti (`ui/src/agents/page.tsx`) | come oggi | una fascia con il motivo; avvio, accesso e token disattivati; MCP e istruzioni restano modificabili (dati del kit) |
| `AgentButtons` (pagina del progetto, riga della barra laterale, scheda della Home) e voci della palette (`ui/src/shell/commands.tsx`) | mostrati | mostrati disattivati con il motivo come titolo (non nascosti: l'utente vede perché) |
| `startsOnProject` (`ui/src/ipc/agents.ts`) | tolta: Antigravity parte su un progetto (sezione 9); la disponibilità viene da `AgentInfo.available` | — |
| IPC `agent_launch` | R12/R13 | R1 `Refused{AgentsNeedVm}` |
| Impostazioni dell'editor (`editor/cfg.rs: kit_profiles`) | profili degli agenti scritti | profili degli agenti **assenti** (`kit_profiles(kit, system_root, agents_enabled)`); un `defaultProfile` che ne indica uno torna ad «Atollo» |
| Estensioni con `"level": "agent"` in `extensions.json` (sezione 10) | come da sezione 10 | non installate; quelle installate disattivate |
| Inoltri | inoltrano (6.8) | scrivono il motivo (R1) ed escono con 1 |
| Dati del kit (`data/home/.claude`, `.codex`, la cartella di Pi, il portachiavi di Antigravity, `data/apps/*/claude…`) | tenuti | tenuti, intatti |

Riattivazione: `levels::watch` emette `levels-changed`; l'app riscrive allora le impostazioni dell'editor (`cfg::write`), allinea le estensioni, e l'interfaccia ricarica `agents_status` e la pagina Strumenti. VSCodium applica un `settings.json` di Machine cambiato senza riavvio **[non verificato]**; se non lo fa, l'avviso nella barra di stato dell'editor dice «Riavvia l'editor per gli agenti».

## 8. Funzioni di fiducia

### 8.1 Il rapporto della VM (modello dei dati)

Il tipo `VmReport` è in [LEVELS.md, 8.1](LEVELS.md#81-the-vm-report-data-model): `project`, `vm` (`on`/`off`), `reason`, `sandbox` (del progetto: nome, stato secondo `sbx ls`, o `absent`, ultimo uso), `agents` (le sandbox `atollo-a-…`), `mounts` (da `kit::mounts`, `.mounts`, e `sbx ls --json`: percorso del PC, percorso nella VM, `rw`/`ro`), `host_ports` (da `kit::host_ports`, `.ports`: quali porte del PC raggiunge la VM e per cosa, «mysql 8.4», «llama-server»), `published` (le azioni del runner nella VM con una porta, indirizzo `127.0.0.1` o `0.0.0.0`), `github` (`github_in_vm`, D-172), `tools` (solo dopo «Verifica strumenti»: per ogni strumento `chosen`, `command`, `path`, `found`, `verdict` fra `ok`, `otherLine`, `missing`, `notFromKit`, `notInVm`, e `output`, le prime tre righe).

La pagina del progetto riceve un pannello «VM del progetto» (`ui/src/projects/vm.tsx`, nuovo): stato, nome della sandbox, montaggi con rw/ro, porte del PC raggiunte e porte pubblicate, GitHub nella VM, sandbox degli agenti; pulsante «Verifica strumenti» (`project_vm_report{check:true}`); senza VM mostra il motivo e «Apri l'avviso VM».

### 8.2 «Verifica strumenti»: cosa gira davvero

Solo con la sandbox accesa (altrimenti `verdict: missing` per tutti e il testo «Avvia il terminale o un'azione del progetto»; il controllo non avvia mai una sandbox da sé **[ipotesi]**). Un `sbx exec` per sandbox, con gli strumenti della sessione (`tools_options`: `ATOLLO_PATH`, variabili), così il controllo vede ciò che vede una sessione:

```
sbx exec -e ATOLLO_PATH=<…> [-e K=V…] <vm> sh -c '
  PATH="$ATOLLO_PATH${ATOLLO_PATH:+:}$PATH"
  per ogni strumento:  printf "@@%s\n" <id>; command -v <prog> || true; <prog> <args> 2>&1 | head -n 3
'
```

`<prog> <args>` vengono dal `check` del catalogo (`tauri/catalog/components.json`), con il percorso tradotto per Linux (`setup::catalog::translate_rel`) e ridotto al nome del file (così si verifica proprio la risoluzione del `PATH`); `notFromKit` quando `command -v` non sta sotto `data/apps/linux-…` montata. La tabella completa, strumento per strumento (comando del catalogo → comando nella VM, o «non nella VM» con il motivo), è in [LEVELS.md, 8.2](LEVELS.md#82-verifica-strumenti-what-really-runs). In sintesi: `git --version`, `python3 --version`, `node --version`, `uv --version`, `php --version`, `go version`, `java -version` (lo standard error letto), `kotlinc -version`, `dotnet --version`, `dart --version`, `zig version`, `julia --version`, `bun --version`, `deno --version`, `typst --version`, `pdftex --version`, `cmake --version`, `clang --version`, `ruby --version`, `mysql --version`, `mongod --version` **[ipotesi, Q17]**, `sqlite3 --version`, `pi --version`, `agy --version`; Claude Code e Codex nelle loro sandbox (`sbx exec <sandbox> claude --version`, `codex --version`); non nella VM: Apache e Caddy (i siti usano le immagini `httpd:2.4` e `php:<linea>-apache`, D-175), Rust, Android, pnpm e R (si completano sul loro sistema, `router::bootstrap`, D-168), llama (gira sul PC, D-178); non controllati: gh, pwsh, sandboxes, vscodium-server, conpty, phpmyadmin, mongo-express (kit sul PC).

Confronto: `chosen` = `versions::project_choice(progetto)[id]`, altrimenti `versions::kit_choice(kit)[id]` (la linea che il `PATH` della VM risolve quando il progetto non ha scelto). `found` = la prima corrispondenza di `[0-9]+(\.[0-9]+)+` nell'uscita (`go version go1.23.2` → `1.23.2`, `openjdk version "21.0.5"` → `21.0.5`). Esito: `ok` quando `found == chosen` o `found` comincia con `chosen + "."`, e il percorso sta sotto gli strumenti montati; `otherLine` quando trovato ma di un'altra linea; `notFromKit` quando il percorso è fuori dal montaggio; `missing` quando `command -v` non trova nulla; gli strumenti senza linee sono `ok` quando trovati dal kit.

`atollo vm <progetto>` scrive lo stesso rapporto (testo, o JSON con `--json`); `--check` esegue la verifica.

### 8.3 Etichette e intestazioni

| Livello / collocamento | Etichetta (it) | Etichetta (en) | Dove |
|---|---|---|---|
| 1, `Pc` | «PC» | "PC" | scheda del terminale e titolo della finestra staccata (`PtyInfo.label`, `ui/src/terminal/panel.tsx`; il titolo della finestra lo mette `editor/terminal.rs: detach`); intestazione di profile.ps1/bashrc |
| 2, `Pc` | «PC · senza VM» | "PC · no VM" | come sopra, più «Senza VM» nella barra di stato (invariato) |
| 2, `Vm` | «VM progetto» | "Project VM" | scheda; intestazione di `launch`; prima riga del log di un'azione |
| 3, `Vm` | «Agente · VM» | "Agent · VM" | scheda; intestazione di `launch` |

Sulla scheda (passo 3): l'etichetta segue il titolo della sessione nel colore delle descrizioni ed è la descrizione accessibile della scheda (`aria-describedby`), il nome resta il titolo; il nome della sandbox è il suggerimento della scheda («Gira in <sandbox>»). Le sessioni dei programmi del kit (`sbx` di `isolation_run`, `claude setup-token` su macOS fino al passo 6) hanno l'etichetta «PC», livello 1. L'intestazione delle shell qui sotto arriva col passo 8: profile.ps1/bashrc la compongono da `ATOLLO_LEVEL`/`ATOLLO_PROJECT`/`ATOLLO_TERMINAL`, che `mark` già mette; nessuna variabile porta una riga pronta (`labels::banner_line` è `None` sul PC).

Testo dell'intestazione (standard error, una riga, prima del programma):

- profile.ps1 / bashrc, da `ATOLLO_LEVEL`: `PC · <terminale o "editor"> · <progetto o "nessun progetto"> · agenti: inoltrati alla VM del progetto se attiva`. La shell non conosce lo stato della VM in ogni momento, quindi l'intestazione non dice se è attiva **[ipotesi]**.
- sessioni nella VM (`start_terminal`, `banner` della risposta di `launch`): `[Atollo] VM progetto: <sandbox> · <cartella nella VM>` e `[Atollo] Agente · VM: <agente> in <sandbox>`. La scrive il lato del PC prima che parta `sbx`, quindi non dipende dalla shell della VM; un prefisso nel prompt della VM non si imposta (il `/etc/profile` dell'immagine può cambiare `PS1`) **[non verificato]**.

### 8.4 `atollo` nella VM: solo lettura (deciso dall'utente, 26/10/2026)

Dentro una sandbox `atollo` dice dove sei e cosa hai; non comanda mai il kit (la riga di comando del kit installa, toglie, cambia versioni, sposta gli accessi e stampa segreti come `atollo secret github`: nulla di questo è mai raggiungibile da una VM).

- **Cosa mostra:** livello e sandbox, progetto, strumenti con le versioni (le stesse righe del rapporto della VM, §8.1), i database del progetto con indirizzo nella VM (`host.docker.internal:<porta>`) e utente, mai la password, e le porte del PC aperte per questa sandbox.
- **Come:** Atollo scrive `info.json` nella cartella del kit della sandbox (`data/app/sandboxes/<sandbox>/`, mai nella cartella del progetto) a ogni `prepare` (fatto nel passo 4: `levels::prepare::SandboxInfo`: `v`, `level`, `sandbox`, `project`, `agent`, `workdir`, `tools` (id → linea scelta), `databases` (`engine`, `database`, `user`, `address`), `hostPorts`); quella cartella è montata nella VM in sola lettura (passo 8). Un piccolo script POSIX, `atollo`, primo nel PATH della VM, la stampa (`atollo`, `atollo --json`); ogni altro argomento dice che il kit si gestisce dal PC ed esce con 1.
- **Mai:** una chiamata al PC, una scrittura, un segreto. Prove: `the_vm_info_holds_no_secret`, `the_vm_atollo_is_read_only`, `the_vm_info_follows_the_report`.

### 8.5 La VM del progetto: riavvia, ricrea, aggiorna (deciso dall'utente, 26/10/2026)

**Ogni sandbox, una per agente (deciso dall'utente, 26/10/2026).** I §8.1–§8.5 e il §6.10 valgono per ogni sandbox di un progetto, non solo per `atollo-p-`: Claude Code e Codex hanno ciascuno la propria (`atollo-a-<agente>-<progetto>-<hash>`), e ognuna ha il suo `info.json` e il suo `atollo` in sola lettura (livello «Agente · VM», nome dell'agente, la sua sandbox), la sua «Verifica strumenti» e le quattro azioni. Il riquadro elenca la sandbox del progetto, poi quella di ogni agente con il suo stato; la riga di comando accetta `--agent <id>` (`atollo vm <progetto> --agent claude --restart`), senza vale la sandbox del progetto. I sostituti degli agenti (§6.10) sono in ogni sandbox tranne dove gira quell'agente. Prove: `every_sandbox_of_a_project_has_its_info`, `the_actions_take_the_agents_sandbox`.

Il riquadro «VM del progetto» (§8.1) ha quattro azioni, e la riga di comando sul PC le stesse:

| Azione | Cosa fa | Riga di comando |
|---|---|---|
| **Riavvia** | ferma e riavvia la sandbox del progetto | `atollo vm <progetto> --restart` |
| **Ricrea** | toglie la sandbox e la rifà con gli strumenti attuali del kit (`Vm::make_again`), dopo una conferma che dice cosa si perde (la home della VM, le immagini scaricate) e cosa resta (progetto, database, strumenti, accessi tenuti dal kit) | `atollo vm <progetto> --recreate` (chiede, salvo `--yes`) |
| **Aggiorna strumenti** | rifà `prepare`: strumenti per la VM, versioni del progetto, database, porte, `info.json`; nessun riavvio se il kit della sandbox non è cambiato | `atollo vm <progetto> --refresh` |
| **Verifica strumenti** | §8.2 | `atollo vm <progetto> --check` |

Ogni azione passa da `decide` con l'origine del terminale del progetto (un rifiuto dice perché e come rimediare) e prende il blocco della sandbox. Le sessioni in corso nella sandbox (terminali, azioni, agenti) sono elencate prima di «Riavvia» e «Ricrea», che le fermano.

### 8.6 L'avanzamento in vista e la pill della VM (deciso dall'utente, 26/10/2026)

L'utente (26/10/2026): «tutto il processo di download e preparazione della VM e strumenti deve essere evidente che si sta svolgendo»; «per il pulsante della VM, metti un'icona dedicata con un colore di fondo o pill che fa capire lo stato»; «per aggiornamento e download degli strumenti sia la pagina sia il resto deve essere più fluido e meno macchinoso».

**Un modello per il lavoro lungo**: `common::jobs::Job` (`tauri/src/common/jobs.rs`, `ui/src/ipc/jobs.ts`): `{ id, kind: install | update | vm-prepare, title, project, items, step: { index, total, label, state, error }, steps, bytes: { done, total }, note, state: queued | running | done | error | cancelled, error, created, automatic }`. Ogni cambiamento è l'evento `job`; `jobs_list` dà quelli tenuti (in corso, in coda e gli ultimi 20 finiti), `job_cancel(id)` ne annulla uno. Il registro è del compito di preparazione (`SetupTask::jobs`); l'app gli dà chi lo ascolta all'avvio.

- **Installazioni e aggiornamenti**: ogni `SetupTask::start` è un lavoro, col titolo di `setup::task::describe` («Installo Node.js», «Aggiorno 3 strumenti», «Scarico Node.js 22.11», «… · VM» per il Linux della VM). Chiesto mentre un'altra gira, **aspetta in coda** (`Snapshot.pending`, stato `queued`) invece di essere rifiutato («C'è già un'installazione in corso» non c'è più per installare, aggiornare e «Applica»); la coda gira nel thread della preparazione, una dopo l'altra, `running` finché è vuota (sempre una preparazione per kit: il blocco del runner non cambia). `job_cancel` toglie una preparazione dalla coda o ferma quella in corso (il segnale dei download, come prima); le altre proseguono; `setup_cancel` le annulla tutte. Gli eventi del runner diventano quelli del lavoro: `Step` → passo i/n col nome del componente, `Bytes` → byte, `Note` → nota, `Done` → stato ed errore del passo. Verifica, ripristino ed esiti non cambiano. `ensure_guest_tools` usa `start_and_wait` (il suo lavoro, atteso per id).
- **La preparazione di una VM**: `levels::prepare` dice `Effects::progress(PrepareProgress)` (metodo nuovo del trait, così il finto delle prove lo registra senza `sbx`): `Begin { project, sandbox, agent, total }`, poi `Step { index, total, label }` **prima** di ogni passo («Strumenti per il Linux della VM»; «Database del progetto» se li raggiunge; «Modelli locali» per Pi; «Informazioni della VM»; «Avvio la VM»), poi `Done` o `Failed { error }`; niente per il PC. L'`AppEffects` dell'app ne fa il lavoro «Preparazione VM · <progetto>[ · <agente>]» (`ipc::levels::VmJob`): il download degli strumenti per la VM si segue nel suo passo (lo strumento come nota, i suoi byte, `VmJob::follow`); `sbx` non stampa avanzamento mentre scarica l'immagine la prima volta, quindi `Vm::start(env, lang, making)` chiama `making` appena prima di `sbx create` e il passo diventa «Scarico l'immagine della VM…» (indeterminato). Una preparazione non si annulla (il suo passo finisce).
- **Barra di stato** (`ui/src/shell/statusbar.tsx`, `JobsItem`): mentre un lavoro gira o aspetta, il suo titolo · passo i/n ed etichetta · una barra piccola con la percentuale (quella del download se se ne sa la dimensione, altrimenti quella dei passi; un tratto che scorre se non si sa); con più lavori, «3 lavori in corso:» e prima quello che gira. Un clic apre Attività su quel lavoro (`JobDetails`: ogni passo col suo errore, «Annulla» per un'installazione, «Riprova» per una non riuscita). Il pannello Attività elenca i lavori in corso e gli ultimi cinque finiti.
- **La sessione che aspetta la sua VM** (un terminale, un'azione, un agente): la pagina del progetto mostra «Preparazione della VM…» col passo e la barra del lavoro sotto i suoi pulsanti (`VmPreparing`); nelle altre pagine la stessa scheda sta sopra la barra di stato, con «Dettagli».
- **La pill della VM** (`vmPill`, `VmPillBadge` in `ui/src/shell/isolation.tsx`, `VmItem`): la codicon `vm` e il testo dello stato su una pill: verde «VM» pronta; blu, pulsante, «VM… n%» mentre si prepara una VM; arancione «Senza VM» quando l'utente può rimediare (`hypervisor_off`, `kvm_denied`, `apparmor`, `sbx_missing`, `sbx_signed_out`, `foreign_daemon`); grigia «Senza VM» quando questo PC non può averla (`old_windows`, `old_macos`, `distribution`, `kvm_missing`: firmware, `kit_disk`); rossa «VM ⚠» quando l'ultima preparazione non è riuscita (e nessuna è finita bene dopo), rossa «Senza VM» con `sbx_failed`. I colori sono i token `--vsc-pill-*` (alias dei colori di successo, informazione, arancione, descrizione ed errore del workbench, chiaro e scuro), il testo nello sfondo dell'editor; nome e suggerimento del pulsante dicono lo stato e cosa fare, mai il solo colore. Un clic apre la finestra della VM come prima.
- **Strumenti fluidi** (pagina Strumenti, pannello rapido, riquadro Aggiornamenti, pagina dello strumento, primo avvio): ogni riga o riquadro mostra la propria installazione («in coda», in corso con la barra, non riuscita con «Riprova»), «Annulla» per elemento (`itemJob` → `job_cancel`); aspetta solo l'elemento in lavorazione (`itemBusy`), gli altri si possono chiedere intanto e vanno in coda; un passo che finisce rilegge solo quella riga (`components_list { ids }`) e la unisce, tenendo la scelta non ancora applicata, senza rifare nulla; la matrice dei quattro sistemi si rilegge quando la coda finisce; «Applica» chiede prima solo se toglie qualcosa.

Prove: `test/rust/common/jobs.rs` (passi, fine, notifica, tenuti), `test/rust/setup/task.rs` (`runs_asked_meanwhile_wait_in_the_queue`, `cancelling_the_running_job_stops_its_run`, `the_runs_events_are_its_jobs_progress`, `a_runs_title_says_what_it_does`), `test/rust/levels/prepare.rs` (`prepare_tells_its_steps_through_the_effects`, `prepare_tells_the_agent_and_the_error`, `nothing_is_told_for_the_pc`), `test/rust/ipc/levels.rs` (`a_vms_preparation_is_a_job`); `test/ui/shell/jobs.test.tsx` (voce della barra di stato, pill per stato, dettagli, scheda d'attesa, eventi), `test/ui/projects/hometools.test.tsx` e `test/ui/components/setup.test.tsx` (avanzamento in riga, coda, annulla, nessun rimontaggio).

## 9. Antigravity

Cosa cambia rispetto a D-178: `agy` di Antigravity è una CLI da terminale come `claude` e `codex` (l'utente, 26/10/2026: «la cli antigravity non ha il medesimo utilizzo della cli codex oppure claude code?»), non un editor per il desktop. Diventa un agente di livello 3 con `AgentRunner::ProjectVm`, come Pi.

- **Build per Linux.** `install/agents.rs: Antigravity::latest` legge `…/manifests/{go_os}_{go_arch}.json`; per `linux_amd64`/`linux_arm64` l'`url` è un `.tar.gz` con un solo programma `antigravity`, installato come `data/apps/linux-x64/antigravity/agy` (`antigravity_download`, coperto dalle prove di oggi in `test/rust/install/agents.rs`). Installato solo per il Linux della VM (D-173 `Scope::Tools`), mai più per il PC (I6, Q11).
- **Sessione.** `vm::agy_session_args(vm, dir, agy, tools)` = `pi_session_args` con `agy` (`sbx exec -it -w <cartella> <strumenti> <vm> sh -c '<TOOLS_FIRST>; exec "$0" "$@"' <agy> <argomenti>`). Variabili: `AGY_CLI_DISABLE_AUTO_UPDATE=true` con `-e`; niente `CP_ANTIGRAVITY`, niente portachiavi (`Atollo.exe keyring restore/save` sono solo del PC e non si eseguono).
- **Accesso.** Due strade, nessuna mette un segreto del kit nella VM:
  1. **Chiave attraverso il proxy di `sbx` (predefinita) [ipotesi]**: il servizio `google` di `sbx secret` copre `GEMINI_API_KEY`/`GOOGLE_API_KEY` per `generativelanguage.googleapis.com`, `oauth2.googleapis.com`, `aiplatform.googleapis.com`, `vertexai.googleapis.com` (pagina di Docker, sezione 11). Atollo lo registra per sandbox con un comando, come D-172 per GitHub: `sbx secret set google --sandbox <atollo-p-…> --command '"<Atollo.exe>" secret gemini <nonce>'` (nuovo comando interno `secret gemini`, stesse protezioni: `asked_by_sbx`, nonce in `secrets.json`). La VM vede un segnaposto. Se `agy` manda la chiave in un'intestazione che il proxy sostituisce, e verso quale host, è **[non verificato]** (Q6).
  2. **Account Google dentro la VM**: l'accesso di `agy` eseguito nella VM; `browser.enabled=false` (D-175) gli fa scrivere il link; il ritorno di OAuth verso una porta locale dentro la VM può non raggiungere il browser del PC **[non verificato]** (Q6). L'accesso resta nel disco della sandbox (la home della VM) e si perde se la sandbox si rifà.
- **I dati di Antigravity nel kit** (il portachiavi, le impostazioni in `data/home`, la chiave Gemini delle impostazioni) restano sul disco; non vengono mai montati né passati.
- **Tolti:** il rifiuto di Antigravity in `vm.ps1` e in `ipc::agents::refused_on_project`; `ui/src/ipc/agents.ts: startsOnProject`; la regola «nessun pulsante per Antigravity» della pagina Agenti; il profilo «Antigravity» dell'editor diventa un profilo di livello 3 come gli altri.

## 10. Estensioni dell'editor che eseguono un agente

Regola: un'estensione che esegue un agente (un modello che decide comandi e modifica file) è di livello 3. Nell'editor di Atollo è ammessa solo se può essere indirizzata all'inoltro, così che l'agente giri nella VM; altrimenti è disattivata con un messaggio. Le estensioni girano nell'extension host del server dell'editor, sul PC (E13).

Cosa è emerso (26/10/2026):

| Estensione (id di Open VSX) | Esegue | Impostazione per indirizzarla altrove | Fonte | Esito |
|---|---|---|---|---|
| Claude Code (`anthropic.claude-code`) — oggi nel catalogo di Atollo (`extensions.json`, generale, `programs: ["claude"]`) | la sua copia **incorporata** della CLI («The extension bundles its own copy of the CLI … for the chat panel») | `claudeCode.claudeProcessWrapper`: «Executable used to launch the Claude process. The bundled binary path is passed as an argument when present.» Inoltre `claudeCode.environmentVariables`, `claudeCode.useTerminal` | https://code.claude.com/docs/en/vs-code (letta il 26/10/2026) | l'involucro esiste → candidata all'inoltro (6.9). **[non verificato]**: che il pannello funzioni quando il processo è `sbx run …` in un altro file system (percorsi del PC negli argomenti, la sua connessione locale all'IDE su `127.0.0.1` del PC, nessun TTY). Finché una prova non passa: **disattivata** |
| Codex (`openai.chatgpt`) | la sua CLI di Codex incorporata | `chatgpt.cliExecutable`: «Development only: Set the path to the Codex CLI executable.» La pagina avverte: «manually overriding the bundled executable can prevent parts of the extension from working.» Inoltre `chatgpt.runCodexInWindowsSubsystemForLinux` (solo WSL) | https://learn.chatgpt.com/docs/developer-settings?surface=ide (rinvio di https://developers.openai.com/codex/ide/settings, letta il 26/10/2026) | documentata come solo per lo sviluppo → **disattivata** |
| Gemini Code Assist (`google.geminicodeassist`) | modalità agente nell'extension host **[non verificato]** | nessuna trovata | non verificato | **disattivata** **[ipotesi]** |
| Gemini CLI companion (`google.gemini-cli-vscode-ide-companion`) | nulla da sé: si collega a una CLI `gemini` in un terminale | — | non verificato | ammessa (il kit non ha la CLI `gemini`; innocua) **[ipotesi]** |
| Cline (`saoudrizwan.claude-dev`), Roo Code (`rooveterinaryinc.roo-cline`), Continue (`continue.continue`), Kilo Code (`kilocode.kilo-code`) | il ciclo dell'agente dentro l'extension host sul PC, verso API del cloud; eseguono comandi nel terminale dell'editor (livello 1, PC) | nessuna (sono loro l'agente) | non verificato | **disattivate** |

Meccanismo:

- `tauri/catalog/extensions.json` riceve `"level": "agent"` per le estensioni note che sono agenti (sopra), e la voce di Claude Code passa da «generale, `programs: ["claude"]`» a `"level": "agent"`. `editor/ext.rs` non installa mai un'estensione `level: agent` e a ogni avvio dell'editor disattiva quelle che trova installate. La disattivazione usa l'opzione di avvio `--disable-extension <id>` del server **[non verificato per il server web di VSCodium; Q10]**; in mancanza, all'avvio si disinstallano (`--uninstall-extension`), e la scelta dell'utente non si tiene.
- L'editor mostra una volta per estensione un messaggio: «<nome> esegue un agente: in Atollo gli agenti girano solo nella VM. Usa il profilo «<agente>» del terminale o la pagina del progetto.»
- Un'estensione fuori dall'elenco non si riconosce in generale: limite dichiarato (Q15).

## 11. Segreti: nessuno del kit entra in una VM

Definizione: un **segreto del kit** è una credenziale che il kit tiene per l'utente o per sé: il token OAuth di Claude (D-115), gli accessi di Codex e di Pi nella home del kit, la chiave API di Gemini delle impostazioni, il portachiavi di Antigravity, il token GitHub del `gh` del kit, i login di root dei server di database del kit, la chiave di llama-server, il token di connessione dell'editor. **Non** sono segreti del kit: l'utente e la password del database del progetto (D-171: sono del progetto e stanno nel suo `.env`).

Come accede ogni agente, con la VM:

| Agente | Accesso | Chiave attraverso il proxy di `sbx` | Fonte |
|---|---|---|---|
| Claude Code | dentro la sua sandbox di `sbx` (`/login`); «the flow runs on the host, so the token is never exposed inside the sandbox» | `anthropic` (`ANTHROPIC_API_KEY`; `api.anthropic.com`, `console.anthropic.com`, `claude.ai`, `mcp-proxy.anthropic.com`) | https://docs.docker.com/ai/sandboxes/security/credentials/ |
| Codex | `sbx run codex` chiede sul PC; `sbx secret set openai --oauth` in anticipo | `openai` (`OPENAI_API_KEY`; `api.openai.com`, `openai.com`, `chatgpt.com`) | idem |
| Antigravity | sezione 9 | `google` (`GEMINI_API_KEY`, `GOOGLE_API_KEY`; `generativelanguage.googleapis.com`, `oauth2.googleapis.com`, `aiplatform.googleapis.com`, `vertexai.googleapis.com`) | idem |
| Pi, fornitori del cloud | il suo `/login` nella VM (il suo file di accesso nella home della VM), o una chiave attraverso il proxy | `anthropic`, `openai`, `google`, `openrouter`, `groq`, `mistral`, `xai` coprono i fornitori comuni di Pi **[ipotesi: Pi legge queste variabili; Q7]** | idem |
| Pi, modelli locali (D-178) | — | nessuno: la chiave di llama-server non è un servizio di `sbx`. Oggi va in `pi.env` nella VM (**contrasto**, 13). Proposta: una chiave solo per quella sandbox (llama-server che accetta più chiavi con `--api-key-file` **[non verificato]**), ritirata alla fine dell'ultima sessione di Pi; oppure `sbx secret set-custom --sandbox` con un segnaposto sostituito nell'intestazione `Authorization` («Replacement applies only to HTTP request headers») verso `host.docker.internal` **[non verificato che i segreti personalizzati valgano per quell'host]** | idem; Q7 |

Dalla pagina di Docker (letta il 26/10/2026): i segreti dei servizi sono globali di serie e «Sandbox-scoped secrets take precedence over global secrets»; «Adding, updating, or removing a service secret takes effect in existing local sandboxes without a restart»; i segreti personalizzati sono sperimentali. Atollo registra solo segreti per sandbox (regola di D-172). Una chiave registrata per la sandbox del progetto è usabile (non leggibile) anche dal codice del progetto in quella VM, come GitHub in D-172 e i modelli locali in D-178: dichiarato.

## 12. Migrazione

### 12.1 Cosa si toglie o cambia nome

L'elenco completo, simbolo per simbolo, è in [LEVELS.md, 12.1](LEVELS.md#121-removed-or-renamed). In sintesi: `isolation::Level` → `Availability`; `seal`, `SEALED`, `agent_dirs`, `without_agents` → `levels::env` (fatto nel passo 2, con `versions::choice_env` ed `editor::terminals::env` tolte e `versions::project_env` involucro sottile: il passo 3 ne ha tolto i chiamanti in E1, E3, E5, E20, il passo 4 gli ultimi e l'involucro); `isolation::vm_for` → `isolation::project_vm`, chiamata solo da `levels::prepare` (passo 4); tolti `agents_notice`, `with_agents_notice` e i sostituti di `data/kit/no-agents` (aggiunta a `OBSOLETE`), al loro posto `data/kit/forward`; `isolation/session.rs` → `levels/requests.rs`; `vm::starts_in_vm`, `vm::sbx_agent` → `Agent::runner()`; le funzioni di preparazione di `ipc::terminals` e `ipc::agents` → `levels::prepare`; tolte `editor_vm_begin`, `editor_vm_session`, `refused_with_vm`, `agent_token_link`, `claude_token::link`, `codex_logout`, `agents::launch_env`; `runtime/vm.ps1` → `runtime/launch.ps1` e `runtime/launch.sh`; tolti `runtime/agents/{claude, agy, agy.cmd}` (danno segreti del kit agli agenti sul PC) e le voci degli agenti in `env/path.rs`; in `editor/cfg.rs` niente più profili dell'utente né `left_by_the_kit`, e un file con commenti o JSON non valido spostato in `settings.json.bak-<data>` e riscritto **[ipotesi]**; in profile.ps1 e bashrc tolto il salto delle versioni del progetto con `ATOLLO_TERMINAL`; in `ui/src/ipc/agents.ts` tolta `startsOnProject`.

### 12.2 Ordine (ogni passo compila e passa `cargo test` e `npm test`)

1. **Tipi e decide.** `levels/mod.rs`, `place.rs`, `facts.rs` (solo letture), `labels.rs`; prove R1–R17 e totalità. Nessun chiamante cambia.
2. **Ambiente.** `levels/env.rs`: spostati `seal`, `SEALED`, `agent_dirs`, `strip_agents`; `environment()` per `Pc` e `Vm`; `versions::{project_env, choice_env}` ed `editor::terminals::env` la chiamano (comportamento invariato salvo: le versioni del progetto nei terminali del kit (Q1), ogni terminale del kit sigillato al modo del livello 1, che tiene solo l'agente SSH dell'utente, `ATOLLO_LEVEL`/`ATOLLO_PROJECT`). La cartella in testa al `PATH` resta quella dei sostituti, `data/kit/no-agents` (`forward_dir`), finché il passo 5 non fa gli inoltri. I pool hanno tenuto il loro ambiente non sigillato e senza agenti fino al passo 3.
3. **Punti di avvio sul PC.** E1, E3 (ramo PC), E5, E6, E7, E8, E9 (ramo PC), E20 attraverso `decide` + `environment`; `PtyInfo.level/label/sandbox`; `CmdError.refusal/fix`. Fatto così: ogni punto di avvio fa la sua `Request` (il posto da `place_of`), legge i `facts` e risponde `decide`; `Refused` è l'errore del comando con motivo e rimedio; `Pc` parte con `environment`; `Vm` tiene la strada nella VM di oggi (E3, E4), con i rifiuti R9/R10 già applicati. E6 decide prima il sito (`Site`), poi il pool in cui entra è deciso come pool (`Pool`, R6). E8 decide pool per pool (`WebServers::autostart` riceve l'ambiente di ogni pool). Ogni avvio di un pool (pagina, avvio dell'app, sito di un progetto, ricarica) gira con `pc_environment(Project, None)`: sigillato, senza agenti, la cartella degli inoltri in testa (`servers/process.rs`, di nuovo sull'ambiente che passa il comando: allora non cambia nulla). E9 fa la cartella (vuota) del nuovo progetto prima di `place_of` e la toglie di nuovo prima della creazione (come per un progetto vuoto); con la VM attiva gira ancora sul PC, sigillato per nessun progetto, fino al passo 4. E20 non avvia nulla, quindi non chiede `decide`: `environment(Pc{Project, p})` qualunque sia la VM.
4. **Preparazione e punti di avvio nella VM.** `levels/prepare.rs` con `Effects`; E3, E4, E10 attraverso di essa; E9 nella VM (nuovo: creazione nella sandbox del progetto nuovo); prova di regola `no_process_outside_levels` accesa. Fatto così: `ipc::levels::ready` è `admitted` → `environment` → `prepare` con gli `AppEffects` dell'app; E3 ed E10 aprono la sessione con `levels::start::open_terminal` (anche E1, con il suo `Ready` sul PC), E4/E5 partono con `start::start_action`, E9 costruisce ogni riga con `start::scaffold_command` (3.6). La sandbox la avvia solo `prepare` (non più `Runner::start_in`). Le sessioni dell'editor (E11 `watch_agent_requests`, `editor_vm_begin`/`editor_vm_session` per E12) passano dallo stesso `decide` → `prepare` finché il passo 5 non le toglie; Antigravity resta rifiutato prima di `decide` (`refused_on_project`) fino al passo 7. `sbx` gira con lo `sbx_env` di `environment` (l'ambiente del kit con le variabili del demone, 3.5), non più con quello sigillato del progetto. Tolte: `isolation::vm_for`, `ipc::terminals::{project_vm_ready, project_vm_tools, project_vm_dbs}`, `ipc::agents::{agent_vm_ready, pi_local_models, no_vm_for_agents}`, `ipc::editing::no_sbx`, `versions::project_env`, `Runner::start`, `frameworks::run_line`, `editor::terminal::open_console`, `ipc::levels::Placed::Vm`.
5. **Richieste.** `levels/requests.rs`, `runtime/launch.ps1`, `runtime/launch.sh`; profili dell'editor: terminali diretti (E2 livello 1), task e agenti attraverso `launch` (E11, E12); i file degli inoltri in `data/kit/forward` (`FORWARD_DIR`, al posto di `data/kit/no-agents` in `forward_dir`), che inoltrano (6.8); ambiente del server dell'editor = `environment(Pc{Manual})` (insieme ai suoi profili: fino ad allora i suoi terminali passano ancora da `vm.ps1` con l'ambiente dell'editor); tolti `session.rs`, `vm.ps1`, `editor_vm_*`; impostazioni dell'editor: solo i profili di Atollo.
6. **Agenti disattivati senza VM.** `levels::agents_enabled`, `levels::installable`, filtro del runner, `AgentInfo.available`, `levels-changed` e `levels::watch`, profili dell'editor condizionati, interfaccia (Strumenti, pagina Agenti, pulsanti, palette). Tolti `agent_token_link`, `codex_logout`, `runtime/agents/*`, le voci degli agenti in `env/path.rs` (E16, E17, E18).
7. **Antigravity nella VM.** `vm::agy_session_args`, `Agent::Antigravity.runner() = ProjectVm`, `secret gemini`, interfaccia.
8. **Fiducia.** `levels/report.rs`, `project_vm_report`, `atollo vm`, il pannello del progetto, etichette e intestazioni.
9. **Estensioni.** `level: agent` in `extensions.json`, disattivazione all'avvio, messaggio; prova di fattibilità di `claudeProcessWrapper` (Q2) e, se passa, `forward/claude-extension`.
10. **Documentazione.** ARCHITECTURE.md (la sezione Isolamento rimanda qui), la Guida, le «Prove» di D-179, il nuovo nome serializzato di `isolation::Level` nell'interfaccia.

### 12.3 Piano delle prove

I nomi delle prove sono in [LEVELS.md, 12.3](LEVELS.md#123-test-plan) e valgono così come sono (le prove si chiamano in inglese). Riassunto:

- **Unitarie** (`test/rust/levels/`): `decide.rs` (livello dall'origine, totalità esaustiva, una prova per ciascuna regola R1–R17, `an_agent_is_never_on_the_pc`, `a_refusal_always_has_a_reason_and_a_fix`, la serializzazione per l'IPC); `facts.rs` (le parti pure: `guest_dir`, `guest_tools`, `linux_build`); `labels.rs` (etichette, intestazioni, testi dei rifiuti); `env.rs` sui quattro target `win32-x64`, `darwin-arm64`, `linux-x64`, `linux-arm64` (nessuna cartella di agente sul PC, inoltri in testa, versioni del progetto, variabili sigillate (il livello 1 tiene solo l'agente SSH, il livello 2 nulla), solo strumenti del kit per Linux nella VM, `no_kit_secret_reaches_a_vm_environment`, idempotenza, valori di riferimento dei chiamanti di oggi); `place.rs`; `requests.rs`; `report.rs` (il comando di ogni strumento del catalogo, confronto per linea, `notFromKit`, `java -version` sullo standard error); `prepare.rs` con `Effects` finto (`a_project_vm_gets_tools_databases_and_ports`, `pi_gets_the_local_models_port`, `an_agent_sandbox_mirrors_the_projects`, `the_vm_info_holds_no_secret`, `a_scaffold_gets_the_kits_powershell_and_the_plans_tools`, `no_sandbox_without_the_kits_sbx`, `nothing_is_prepared_for_the_pc`); `rule.rs` (`no_process_outside_levels`, accesa dal passo 4; `no_agent_binary_is_started_by_the_app_on_the_pc`, con il passo 6).
- **Prove di oggi che cambiano**: `test/rust/isolation/{mod,session,vm}.rs`, `test/rust/ipc/agents.rs`, `test/rust/setup/kit_files.rs`, `test/rust/editor/{cfg,terminals,ext}.rs`, `test/rust/setup/runner.rs`.
- **Integrazione per punto di avvio** (Rust con finti dove si può; il resto a mano su Windows 11 con sbx e su un Linux, annotato nelle «Prove» di D-179): E1–E12, l'inoltro di 6.8 dentro e fuori un progetto e senza VM, E14, `atollo vm shop --check`.
- **Interfaccia** (`test/ui/`): `terminal/panel.test.tsx`, `agents/page.test.tsx`, `projects/vm.test.tsx`, `components/table.test.tsx`, `shell/commands.test.tsx`, `servers/page.test.tsx`.

## 13. Codice di oggi in contrasto con questo documento

1. ~~**I terminali del kit ignorano le versioni del progetto** nella sua cartella: `editor/terminals.rs: env` imposta `ATOLLO_TERMINAL` per ogni terminale del kit, e `profile.ps1`/`bashrc` saltano gli scambi del progetto quando c'è.~~ Risolto dal passo 2: `terminal_launch` dà a `pc_environment` il progetto della sua cartella (`place_of`); `ATOLLO_TERMINAL` ora evita solo che il profilo le applichi due volte.
2. **I terminali dell'editor sono solo nella VM** (D-178, `vm.ps1` chiuso per difetto); D-179 li fa di livello 1 sul PC.
3. ~~**La creazione dei progetti gira sempre sul PC**: `ipc/frameworks.rs: frameworks_create` usa `frameworks::run_line` anche con la VM attiva.~~ Risolto dal passo 4: è decisa (`Scaffold`, R5/R9/R10/R15) e con la VM attiva gira nella sandbox del progetto nuovo (3.6).
4. **Il server dell'editor, e quindi ogni estensione, ha gli agenti nel `PATH`**: `env/path.rs` mette `data/kit/agents`, `claude`, `codex/bin`, `codex/codex-path`, `pi` nel `PATH` del kit, ed `editor/mod.rs: Editor::start_on` avvia il server con l'ambiente completo del kit.
5. **Il kit installa l'estensione Claude Code** (`extensions.json`, generale, `programs: ["claude"]`), che esegue la sua CLI incorporata sul PC (livello 3 sul PC).
6. **Programmi degli agenti avviati sul PC dall'app**: `agents/claude_token.rs: link` (`claude setup-token`, macOS) e `agents/mod.rs: codex_logout`.
7. **Senza la PowerShell del kit** i profili degli agenti dell'editor eseguono `claude.exe`/`codex.exe`/`agy`/`pi.exe` direttamente sul PC e il profilo dei task è `/bin/bash` sul PC (`editor/cfg.rs: kit_profiles`, `merge`).
8. **Un segreto del kit entra in una VM**: la chiave dei modelli locali di Pi va in `pi.env` (`ipc/agents.rs: pi_local_models`), passato con `--env-file` (D-178).
9. **Antigravity è rifiutato sui progetti** (`vm.ps1`, `ipc/agents.rs: refused_on_project`, `ui/src/ipc/agents.ts: startsOnProject`).
10. **I profili di terminale dell'utente restano** e un file di impostazioni illeggibile resta com'è (`editor/cfg.rs: merge`, `write`): D-179 vuole solo i profili di Atollo, riscritti a ogni avvio.
11. **Gli agenti si installano per il PC** (e Claude Code e Codex anche per il Linux della VM: `terminals::chosen_tools` comprende il gruppo `agents`) anche se nessun agente gira sul PC e le sandbox degli agenti di `sbx` portano i loro Claude Code e Codex.
12. **La pagina Agenti mostra l'accesso del PC** (`agents::status` legge la home del kit), non quello che usano gli agenti nella VM.
13. **Le richieste degli agenti dell'editor vogliono una sessione dell'editor su quel progetto** (`session::take_requests` controlla la cartella della sessione): un inoltro non potrebbe usarle così come sono.
14. **Il livello dell'editor si annota una volta, all'apertura dell'editor** (`.level`): una VM che si accende o si spegne dopo non è vista dai terminali dell'editor.
15. **`agents::launch_env` non ha chiamanti** e gli involucri `runtime/agents/{claude, agy, agy.cmd}` danno segreti del kit (token di Claude, chiave Gemini) agli agenti sul PC.
16. **Il debugger e le estensioni dei linguaggi eseguono codice del progetto sul PC** con la VM attiva (limite dichiarato di D-175; contro lo spirito di I1).

## 14. Domande aperte e punti non verificati

| # | Domanda | Scelta in questo documento |
|---|---|---|
| Q1 | Un terminale dell'utente con linee esplicite aperto nella cartella di un progetto: linee del progetto o del terminale? | del progetto per gli strumenti che il progetto ha scelto, del terminale per gli altri — **deciso dall'utente, 26/10/2026** |
| Q2 | L'estensione Claude Code può girare con `claudeCode.claudeProcessWrapper` → inoltro → `sbx run` (percorsi del PC negli argomenti, la sua connessione locale all'editor, nessun TTY)? | disattivata finché una prova non lo dimostra — **deciso dall'utente, 26/10/2026**: disattivata ora; una prova a parte tenta l'involucro e, se regge, la riattiva attraverso la VM |
| Q3 | Estensione Codex: `chatgpt.cliExecutable` è «Development only» | disattivata |
| Q4 | `sbx run --name <sandbox>` passa argomenti in più all'agente (`-- --resume`)? | [non verificato]; se no, gli argomenti inoltrati sono rifiutati per Claude e Codex con un messaggio |
| Q5 | `sbx run` funziona senza TTY (il canale di un'estensione)? | [non verificato] |
| Q6 | Antigravity nella VM: `agy` accetta `GEMINI_API_KEY` attraverso il servizio `google` del proxy (intestazione o parametro, host)? Il suo accesso Google si completa dentro la VM con il browser disattivato? | chiave attraverso il proxy [ipotesi]; accesso nella VM come ripiego |
| Q7 | Pi: quali variabili legge per i fornitori del cloud; una chiave di llama-server per sandbox (`--api-key-file`) o un segreto personalizzato verso `host.docker.internal` | chiave per sandbox, ritirata all'ultima sessione [proposta] |
| Q8 | L'immagine `shell` di `sbx` contiene la CLI di un agente? | [non verificato]; se sì, `environment(Vm{Project})` deve nasconderla (un `PATH` senza la cartella dell'agente dell'immagine) |
| Q9 | VSCodium: `automationProfile` vale anche per i terminali del debug; un workspace fidato può aggiungere profili di terminale; il `settings.json` di Machine si ricarica da solo? | [non verificato] |
| Q10 | `--disable-extension` del server web di VSCodium | [non verificato]; in mancanza, disinstallazione |
| Q11 | Smettere del tutto di installare gli agenti per il PC, e Claude Code e Codex per il Linux della VM (le immagini di `sbx` li hanno)? | sì: gli agenti si installano solo per la VM (build Linux di Pi e Antigravity; Claude Code e Codex dalle immagini di `sbx`) — **deciso dall'utente, 26/10/2026** |
| Q12 | Un'etichetta nel prompt del `bash -l` della VM | solo l'intestazione [non verificato] |
| Q13 | I server MCP del kit per gli agenti nella VM («Da fare» di D-175) | invariato, fuori da D-179 |
| Q14 | Adattatori del debugger ed estensioni dei linguaggi che eseguono codice del progetto sul PC con la VM attiva | limite dichiarato mantenuto; candidato a una decisione successiva |
| Q15 | Estensioni che sono agenti ma non sono nell'elenco | non si riconoscono in generale: limite dichiarato |
| Q16 | Inoltro `.cmd` su Windows: argomenti con `"`, `%`, `^` attraverso `%*` | limite dichiarato; gli inoltri `.ps1` e sh li passano intatti |
| Q17 | MongoDB nella VM: si controlla `mongod` o `mongosh`? | `mongod --version` [ipotesi] |
| Q18 | PowerShell può catturare l'uscita di `Atollo.exe` (sottosistema grafico)? Conta solo se un client chiamasse la riga di comando | non usato: i client usano i file |
| Q19 | Una creazione nella VM: le righe PowerShell del catalogo scritte per Windows (`vendor\bin\wp`, `Expand-Archive`, `Invoke-WebRequest`) girano con PowerShell per Linux? «Annulla» (che chiude `sbx exec` sul PC) ferma la riga dentro la VM? | [non verificato]; una riga che fallisce lo dice nella procedura guidata, come sul PC; una riga annullata può continuare nella VM fino alla fine (come un'azione prima di `vm::tracked`) |
