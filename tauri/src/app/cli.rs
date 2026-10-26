//! Command line of the same exe (DECISIONS.md D-21): `env`, `components`,
//! `versions`, `check`, `prepare`, `sync`, `keyring`, `mcp`, `serve`, `models`,
//! `llm`, `diagnostics`, `version`, `help`. The exe is a GUI program, so output goes
//! to the redirected handle when there is one, otherwise to the parent
//! console. Step lines use the markers `── N. Name ──`, `√`, `x`, `!` and
//! `·`.

use std::ffi::OsString;
use std::io::{self, Write};
use std::path::Path;

use crate::common::i18n::t;
use crate::common::settings::Settings;
use crate::common::{i18n, log};
use crate::env::{Host, KitEnv};
use crate::kit::Kit;
use crate::setup::runner::{Event, Options, Outcome, Sink};
use crate::setup::{Catalog, Choice};
use crate::{install, kit, llm, net, platform, setup, versions};

#[derive(Debug, PartialEq, Eq)]
pub enum Cli {
    /// No command: open the window.
    Gui,
    /// The kit environment; with `--project <name>`, the one of its programs.
    Env {
        project: Option<String>,
    },
    Prepare(PrepareArgs),
    /// The catalog; with `--target <id>`, for that system (what is not
    /// available there says why).
    Components(Option<crate::kit::Target>),
    /// The saved versions; with an id, the lines that can be downloaded.
    Versions(Option<String>),
    /// Every installed program answers its version command; with `--kit`,
    /// the health of the kit (diagnostics/health.rs).
    Check {
        kit: bool,
    },
    /// `check --files`: the installed programs' files against the manifest
    /// of their installation (setup/integrity.rs).
    CheckFiles,
    /// Aligns the agents (`--project <folder>`, `--verbose`).
    Sync {
        project: Option<std::path::PathBuf>,
        verbose: bool,
    },
    /// Antigravity's login: `restore` before agy, `save` after.
    Keyring(String),
    /// The kit's GitHub MCP server on stdio (`mcp github`, started by the
    /// agents with the app's absolute path): the token stays here.
    McpGithub,
    /// `secret github <nonce>`: the kit's GitHub token on standard output,
    /// for the command Docker Sandboxes runs on the PC when its proxy needs
    /// it (D-172): the token is never copied, never in a VM; the nonce is
    /// the one of a registration Atollo noted.
    SecretGithub(String),
    /// File server of a static site: `serve <folder> <port>` (projects/run/).
    Serve {
        dir: std::path::PathBuf,
        port: u16,
    },
    /// The local models (D-151).
    Models(Models),
    /// Their server: `start`, `stop`, `status`.
    Llm(String),
    /// The zip for a report of a problem (diagnostics/mod.rs).
    Diagnostics,
    Version,
    Help,
    Unknown(String),
}

/// `models`, `models pull <id|owner/repo[:quant]>`, `models rm <id>`.
#[derive(Debug, PartialEq, Eq)]
pub enum Models {
    List,
    Pull(String),
    Remove(String),
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct PrepareArgs {
    pub target: Option<crate::kit::Target>,
    pub only: Vec<String>,
    pub skip: Vec<String>,
    pub update_all: bool,
    pub force: bool,
    /// `--line node=22.21`: only these lines, saved next to the default.
    pub lines: Vec<(String, String)>,
}

impl PrepareArgs {
    pub fn options(&self) -> Options {
        Options {
            target: self.target,
            only: self.only.clone(),
            skip: self.skip.clone(),
            update_all: self.update_all,
            force: self.force,
            lines: self.lines.clone(),
            ..Options::default()
        }
    }
}

fn ids(value: Option<&OsString>) -> Vec<String> {
    value
        .map(|v| v.to_string_lossy().to_lowercase())
        .unwrap_or_default()
        .split([',', ';', ' '])
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

pub fn parse(args: &[OsString]) -> Cli {
    let Some(first) = args.first() else {
        return Cli::Gui;
    };
    match first.to_string_lossy().as_ref() {
        "env" => match args
            .get(1)
            .map(|a| a.to_string_lossy().into_owned())
            .as_deref()
        {
            None => Cli::Env { project: None },
            Some("--project") if args.len() == 3 => Cli::Env {
                project: Some(args[2].to_string_lossy().into_owned()),
            },
            Some(other) => Cli::Unknown(format!("env {other}")),
        },
        "components" => match args
            .get(1)
            .map(|a| a.to_string_lossy().into_owned())
            .as_deref()
        {
            None => Cli::Components(None),
            Some("--target") if args.len() == 3 => {
                match crate::kit::Target::parse(&args[2].to_string_lossy().to_lowercase()) {
                    Some(t) => Cli::Components(Some(t)),
                    None => Cli::Unknown("components --target".into()),
                }
            }
            Some(other) => Cli::Unknown(format!("components {other}")),
        },
        "versions" if args.len() <= 2 => {
            Cli::Versions(args.get(1).map(|a| a.to_string_lossy().to_lowercase()))
        }
        "check" => match args
            .get(1)
            .map(|a| a.to_string_lossy().into_owned())
            .as_deref()
        {
            None => Cli::Check { kit: false },
            Some("--kit") if args.len() == 2 => Cli::Check { kit: true },
            Some("--files") if args.len() == 2 => Cli::CheckFiles,
            Some(other) => Cli::Unknown(format!("check {other}")),
        },
        "sync" => {
            let mut project = None;
            let mut verbose = false;
            let mut rest = args[1..].iter();
            while let Some(arg) = rest.next() {
                match arg.to_string_lossy().as_ref() {
                    // Without its value a usage error, never the whole kit
                    // (BUG-V6-33).
                    "--project" => match rest.next() {
                        Some(p) => project = Some(std::path::PathBuf::from(p)),
                        None => return Cli::Unknown("sync --project".into()),
                    },
                    "--verbose" => verbose = true,
                    other => return Cli::Unknown(other.to_owned()),
                }
            }
            Cli::Sync { project, verbose }
        }
        "keyring" => match args
            .get(1)
            .map(|a| a.to_string_lossy().into_owned())
            .as_deref()
        {
            Some(a @ ("restore" | "save")) => Cli::Keyring(a.to_owned()),
            other => Cli::Unknown(format!("keyring {}", other.unwrap_or_default())),
        },
        "secret" => match args
            .get(1)
            .map(|a| a.to_string_lossy().into_owned())
            .as_deref()
        {
            Some("github") if args.len() == 3 => {
                Cli::SecretGithub(args[2].to_string_lossy().into_owned())
            }
            other => Cli::Unknown(format!("secret {}", other.unwrap_or_default())),
        },
        "mcp" => match args
            .get(1)
            .map(|a| a.to_string_lossy().into_owned())
            .as_deref()
        {
            Some("github") if args.len() == 2 => Cli::McpGithub,
            other => Cli::Unknown(format!("mcp {}", other.unwrap_or_default())),
        },
        "serve" => match (
            args.get(1),
            args.get(2).and_then(|p| p.to_str()?.parse().ok()),
        ) {
            (Some(dir), Some(port)) if args.len() == 3 => Cli::Serve {
                dir: std::path::PathBuf::from(dir),
                port,
            },
            _ => Cli::Unknown("serve".into()),
        },
        "prepare" => {
            let mut p = PrepareArgs::default();
            let mut rest = args[1..].iter();
            while let Some(arg) = rest.next() {
                match arg.to_string_lossy().as_ref() {
                    "--target" => {
                        let value = rest.next().map(|v| v.to_string_lossy().to_lowercase());
                        match value.as_deref().and_then(crate::kit::Target::parse) {
                            Some(t) => p.target = Some(t),
                            None => return Cli::Unknown("prepare --target".into()),
                        }
                    }
                    "--only" => p.only.extend(ids(rest.next())),
                    "--skip" => p.skip.extend(ids(rest.next())),
                    "--update-all" => p.update_all = true,
                    "--force" => p.force = true,
                    "--line" => {
                        let value = rest.next().map(|v| v.to_string_lossy().to_lowercase());
                        match value.as_deref().and_then(|v| v.split_once('=')) {
                            Some((id, line)) => p.lines.push((id.to_owned(), line.to_owned())),
                            None => return Cli::Unknown("prepare --line".into()),
                        }
                    }
                    other => return Cli::Unknown(format!("prepare {other}")),
                }
            }
            Cli::Prepare(p)
        }
        "models" => {
            let arg = |i: usize| args.get(i).map(|a| a.to_string_lossy().into_owned());
            match (arg(1).as_deref(), arg(2), args.len()) {
                (None, _, _) => Cli::Models(Models::List),
                (Some("pull"), Some(spec), 3) => Cli::Models(Models::Pull(spec)),
                (Some("rm"), Some(id), 3) => Cli::Models(Models::Remove(id)),
                (Some(other), _, _) => Cli::Unknown(format!("models {other}")),
            }
        }
        "llm" => match args
            .get(1)
            .map(|a| a.to_string_lossy().into_owned())
            .as_deref()
        {
            Some(a @ ("start" | "stop" | "status")) if args.len() == 2 => Cli::Llm(a.to_owned()),
            other => Cli::Unknown(format!("llm {}", other.unwrap_or_default())),
        },
        "diagnostics" if args.len() == 1 => Cli::Diagnostics,
        "version" | "--version" | "-V" => Cli::Version,
        "help" | "--help" | "-h" | "/?" => Cli::Help,
        other => Cli::Unknown(other.to_owned()),
    }
}

/// Where command output goes (`platform::console`).
pub fn output() -> Box<dyn Write> {
    crate::platform::console::output()
}

/// `KEY=VALUE` lines, sorted, as `set` prints them. Secrets masked
/// (`CLAUDE_CODE_OAUTH_TOKEN=***`, a proxy's password; `env::masked`): the
/// output ends up in terminals, logs and bug reports.
pub fn print_env(env: &KitEnv, out: &mut dyn Write) -> io::Result<()> {
    for (name, value) in env.iter() {
        writeln!(out, "{name}={}", crate::env::masked(name, value))?;
    }
    out.flush()
}

/// The catalog with the choice and what is installed.
/// The saved versions of the languages: `* ` marks the one the kit uses.
pub fn print_versions(kit: &Kit, lang: &str, out: &mut dyn Write) -> io::Result<()> {
    for tool in crate::versions::tools(kit).iter().filter(|x| x.installed) {
        let used = tool
            .preferred
            .clone()
            .or_else(|| tool.default.as_ref().map(|d| d.line.clone()));
        let row = |line: &str, version: Option<&String>, note: String| {
            let mark = if used.as_deref() == Some(line) {
                "*"
            } else {
                " "
            };
            format!(
                "  {mark} {:<16} {:<8} {}{note}",
                tool.id,
                line,
                version.map_or("", String::as_str)
            )
        };
        if let Some(d) = &tool.default {
            let note = format!("  ({})", t(lang, "predefinita", &[]));
            writeln!(out, "{}", row(&d.line, d.version.as_ref(), note))?;
        }
        for s in &tool.saved {
            writeln!(out, "{}", row(&s.line, s.version.as_ref(), String::new()))?;
        }
    }
    out.flush()
}

/// The health of the kit, one finding per line (`√`, `!`, `x`) with what
/// to do under it.
pub fn print_health(
    findings: &[crate::diagnostics::health::Finding],
    out: &mut dyn Write,
) -> io::Result<()> {
    use crate::diagnostics::health::Level;
    for f in findings {
        let mark = match f.level {
            Level::Ok => "√",
            Level::Warn => "!",
            Level::Err => "x",
        };
        writeln!(out, "  {mark} {}", f.text.replace('\n', "\n    "))?;
        if let Some(fix) = &f.fix {
            writeln!(out, "    → {fix}")?;
        }
    }
    out.flush()
}

/// `check --files`: one line per installed folder, the damaged files under
/// it. Returns whether something is damaged.
pub fn print_integrity(
    report: &[crate::setup::integrity::Integrity],
    lang: &str,
    out: &mut dyn Write,
) -> io::Result<bool> {
    let mut damaged = false;
    for i in report {
        let name = match &i.line {
            Some(line) => format!("{} {line}", i.name),
            None => i.name.clone(),
        };
        match (&i.damage, &i.error) {
            (_, Some(e)) => {
                damaged = true;
                writeln!(out, "  x {name}: {e}")?;
            }
            (None, None) => writeln!(
                out,
                "  - {name}: {}",
                t(
                    lang,
                    "installato prima del manifesto, si verifica dalla prossima installazione",
                    &[]
                )
            )?,
            (Some(d), None) if !d.is_damaged() => writeln!(
                out,
                "  √ {name}: {}",
                t(lang, "{0} file a posto", &[&d.checked.to_string()])
            )?,
            (Some(d), None) => {
                damaged = true;
                writeln!(
                    out,
                    "  x {name}: {}",
                    t(
                        lang,
                        "{0} file cambiati, {1} mancanti",
                        &[&d.changed.len().to_string(), &d.missing.len().to_string()]
                    )
                )?;
                for p in d.changed.iter().chain(&d.missing) {
                    writeln!(out, "      {p}")?;
                }
            }
        }
    }
    if damaged {
        writeln!(
            out,
            "\n  → {}",
            t(
                lang,
                "Reinstalla i componenti danneggiati da Strumenti › «Verifica i file».",
                &[]
            )
        )?;
    }
    out.flush()?;
    Ok(damaged)
}

/// The final check, one line per component: `√ Name   answer`.
pub fn print_checks(
    checks: &[crate::setup::checkup::Check],
    out: &mut dyn Write,
) -> io::Result<()> {
    let width = checks
        .iter()
        .map(|c| c.name.chars().count())
        .max()
        .unwrap_or(0);
    for c in checks {
        let mark = if c.ok { "√" } else { "x" };
        writeln!(out, "  {mark} {:<width$}  {}", c.name, c.answer)?;
    }
    out.flush()
}

pub fn print_components(kit: &Kit, lang: &str, out: &mut dyn Write) -> io::Result<()> {
    let catalog = Catalog::get();
    let choice = Choice::read(kit, catalog).unwrap_or_else(|| Choice::defaults(catalog));
    // Chosen also when a chosen one needs it on this target only (Zig for
    // C and C++ on macOS and Linux), as the Tools page shows it: the run
    // installs it, the saved choice does not have it.
    let here = catalog.needs_only_on(&choice.chosen_ids(catalog), kit.target());
    for c in catalog.all() {
        let mark = if choice.is_chosen(&c.id) || here.contains(c.id.as_str()) {
            "[x]"
        } else {
            "[ ]"
        };
        let unavailable = crate::router::availability::unavailable_message(c, kit.target(), lang);
        let state = if let Some(why) = unavailable {
            why
        } else if c.is_installed(kit) {
            c.installed_version(kit)
                .unwrap_or_else(|| t(lang, "installato", &[]))
        } else {
            "-".into()
        };
        writeln!(
            out,
            "{mark} {:<16} {:<18} {:<10} {state}",
            c.id,
            c.name,
            format!("{:?}", c.group).to_lowercase()
        )?;
    }
    out.flush()
}

/// Prints the events of a preparation.
pub struct Printer<'a> {
    pub out: &'a mut dyn Write,
    pub lang: &'a str,
    pub downloads_dir: Option<std::path::PathBuf>,
    names: Vec<(String, String)>,
}

impl<'a> Printer<'a> {
    pub fn new(out: &'a mut dyn Write, lang: &'a str) -> Printer<'a> {
        Printer {
            out,
            lang,
            downloads_dir: None,
            names: Vec::new(),
        }
    }

    pub fn with_downloads_dir(mut self, dir: std::path::PathBuf) -> Printer<'a> {
        self.downloads_dir = Some(dir);
        self
    }

    fn name(&self, id: &str) -> String {
        self.names
            .iter()
            .find(|(i, _)| i == id)
            .map(|(_, n)| n.clone())
            .unwrap_or_else(|| id.to_owned())
    }
}

impl Sink for Printer<'_> {
    fn emit(&mut self, event: Event) {
        let lang = self.lang;
        let line = match event {
            // The steps say themselves one by one.
            Event::Queue { .. } => return,
            Event::Step {
                index, id, name, ..
            } => {
                self.names.push((id, name.clone()));
                let title = format!("── {index}. {name} ");
                let pad = 60usize.saturating_sub(title.chars().count());
                format!("\n{title}{}", "─".repeat(pad))
            }
            Event::Note { text, .. } => format!("  · {text}"),
            Event::Bytes { done, total, .. } => match total {
                Some(total) if total > 0 && (done * 100 / total) % 25 == 0 => {
                    format!("  · {}% ({} MB)", done * 100 / total, total / (1024 * 1024))
                }
                _ => return,
            },
            Event::Done { id, outcome } => {
                let name = self.name(&id);
                match outcome {
                    Outcome::Installed { version } => {
                        format!("  √ {}", t(lang, "{0}: installato {1}", &[&name, &version]))
                    }
                    Outcome::UpToDate { version } => format!(
                        "  √ {}",
                        t(
                            lang,
                            "{0}: già aggiornato ({1})",
                            &[&name, version.as_deref().unwrap_or("?")]
                        )
                    ),
                    Outcome::Removed => format!("  √ {}", t(lang, "{0}: rimosso", &[&name])),
                    Outcome::InUse => format!(
                        "  x {}",
                        t(
                            lang,
                            "{0}: in uso, chiudi i programmi del kit e riprova",
                            &[&name]
                        )
                    ),
                    Outcome::Unsupported => format!(
                        "  ! {}",
                        t(
                            lang,
                            "{0}: non ancora disponibile in questa versione di Atollo",
                            &[&name]
                        )
                    ),
                    Outcome::Cancelled => format!("  ! {}", t(lang, "{0}: annullato", &[&name])),
                    Outcome::Failed { message, by_hand } => {
                        let mut line = format!(
                            "  x {}",
                            t(lang, "{0}: non riuscito: {1}", &[&name, &message])
                        );
                        if let Some(b) = by_hand {
                            // The folder where fetch looks (D-90: in the
                            // system's TEMP); data\downloads works too.
                            let dir = self.downloads_dir.as_ref().map_or_else(
                                || "data\\downloads".to_owned(),
                                |d| d.display().to_string(),
                            );
                            let hint = t(
                                lang,
                                "Scaricalo dal browser ({0}), mettilo nella cartella {1} e riprova.",
                                &[&b.url, &dir],
                            );
                            line.push_str(&format!("\n    {hint}"));
                        }
                        line
                    }
                }
            }
        };
        let _ = writeln!(self.out, "{line}");
        let _ = self.out.flush();
    }
}

/// `Atollo.exe prepare`: 0 all good, 1 some component failed, 3 another
/// preparation is running.
pub(crate) fn prepare(
    kit: &Kit,
    kit_env: &KitEnv,
    settings: &Settings,
    host: &Host,
    lang: &str,
    args: &PrepareArgs,
) -> i32 {
    let kit_scoped;
    let kit = match args.target {
        Some(t) => {
            kit_scoped = kit.for_target(t);
            &kit_scoped
        }
        None => kit,
    };
    let mut out = output();
    if let Err(e) = platform::jobs::init() {
        log::warn(&format!("child processes: {e:#}"));
    }
    let net = match net::Net::new(settings, crate::github_token(host)) {
        Ok(net) => net.with_github_cache(kit.data().join("cache").join("github")),
        Err(e) => {
            let _ = writeln!(out, "  x {e:#}");
            return 1;
        }
    };
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let ctx = install::Ctx {
        kit,
        net: &net,
        env: kit_env,
        cancel: &cancel,
    };
    log::info(&format!("prepare {args:?}"));
    let mut printer = Printer::new(&mut *out, lang).with_downloads_dir(setup::downloads::dir(kit));
    let results = match setup::runner::run(&ctx, &args.options(), &mut printer) {
        Ok(results) => results,
        Err(e) if e.is::<setup::runner::Busy>() => {
            let _ = writeln!(
                out,
                "  x {}",
                i18n::t(
                    lang,
                    "Un'altra preparazione è già in corso in questo kit.",
                    &[]
                )
            );
            return 3;
        }
        Err(e) => {
            let _ = writeln!(out, "  x {e:#}");
            return 1;
        }
    };
    let failed = results.iter().filter(|(_, o)| o.is_failure()).count();
    // The final check of what this run installed (`Test-Installation`).
    let touched: Vec<&str> = results
        .iter()
        .filter(|(_, o)| matches!(o, setup::runner::Outcome::Installed { .. }))
        .map(|(id, _)| id.as_str())
        .collect();
    let checks: Vec<_> = setup::catalog::Catalog::get()
        .all()
        .iter()
        .filter(|c| touched.contains(&c.id.as_str()))
        .filter_map(|c| setup::checkup::check(kit, kit_env, c))
        .collect();
    if !checks.is_empty() {
        let _ = writeln!(out, "\n── {} ──", i18n::t(lang, "Verifica", &[]));
        let _ = print_checks(&checks, &mut *out);
    }
    for (id, outcome) in &results {
        log::info(&format!("prepare {id}: {outcome:?}"));
    }
    i32::from(failed > 0)
}

/// Help text of the command line (Italian source; translated by the
/// catalog). `{0}` is the program's name ([`program_name`]), `{1}` the
/// kit's Antigravity command ([`help_text`]); the target ids
/// are those of `Target::ALL`.
pub(crate) const HELP: &str = "Uso: {0} [comando]\n\nComandi:\n  env                     mostra l'ambiente del kit\n      --project <nome>    quello dei programmi di un progetto\n  components              elenca linguaggi e strumenti, con quelli installati\n      --target <id>       per un altro sistema: win32-x64, darwin-arm64,\n                          linux-x64, linux-arm64\n  versions [id]           le versioni salvate; con un id, quelle da scaricare\n  check                   verifica che i programmi installati rispondano\n      --kit               la salute del kit: disco, spazio, percorso, permessi, librerie\n      --files             i file dei programmi: mancanti o cambiati dall'installazione\n  prepare [opzioni]       installa e aggiorna i componenti scelti,\n                          rimuove quelli non scelti\n      --only a,b          solo questi e ciò che serve loro, senza rimuovere\n      --skip a,b          non questi\n      --update-all        aggiorna anche quelli già installati\n      --force             scarica di nuovo\n      --line id=x.y       salva questa versione accanto alla predefinita\n      --target <id>       prepara per un altro sistema (id come per components)\n  sync [opzioni]          allinea istruzioni, skill e server MCP degli agenti\n      --project <cartella> anche le istruzioni di questo progetto\n      --verbose           mostra ogni modifica\n  keyring restore|save    accesso di Antigravity (lo usa {1} del kit)\n  serve <cartella> <porta> server di file per un sito statico\n  models                  i modelli locali: del catalogo e scaricati\n  models pull <modello>   scarica un id del catalogo o un modello di Hugging Face\n                          (proprietario/nome:quantizzazione), riprende se interrotto\n  models rm <id>          cancella un modello scaricato\n  llm start|stop|status   il server dei modelli locali (chat e agenti)\n  diagnostics             crea lo zip da allegare a una segnalazione (segreti mascherati)\n  version                 mostra la versione\n  help                    mostra questo aiuto\n\nSenza comando apre la finestra di Atollo.";

/// The program's name in the help: the file it was started from
/// (`Atollo-0.2.0-win32-x64.exe`, `Atollo.AppImage`, `Atollo`, ...); on
/// Windows only an `.exe` name, else `Atollo.exe`; elsewhere `Atollo`
/// when that is unknown.
pub(crate) fn program_name(os: kit::Os, exe: Option<&Path>) -> String {
    let name = exe
        .and_then(Path::file_name)
        .map(|n| n.to_string_lossy().into_owned())
        .filter(|n| !n.is_empty());
    if os.is_windows() {
        // A Windows path read anywhere: `\` separates there, not on Unix.
        return name
            .and_then(|n| n.rsplit(['\\', '/']).next().map(str::to_owned))
            .filter(|n| n.to_ascii_lowercase().ends_with(".exe"))
            .unwrap_or_else(|| "Atollo.exe".to_owned());
    }
    name.unwrap_or_else(|| "Atollo".to_owned())
}

/// The help for `os`; `{1}` is the kit's Antigravity command (`agy.cmd`
/// on Windows, as always; `agy` elsewhere).
pub(crate) fn help_text(lang: &str, os: kit::Os, exe: Option<&Path>) -> String {
    let agy = if os.is_windows() { "agy.cmd" } else { "agy" };
    i18n::t(lang, HELP, &[&program_name(os, exe), agy])
}

/// `models`: the local models, a download, a removal (D-151).
pub(crate) fn models(kit: &Kit, settings: &Settings, host: &Host, lang: &str, cmd: Models) -> i32 {
    let mut out = output();
    let result = match cmd {
        Models::List => llm::cli::list(kit, lang, &mut *out).map_err(Into::into),
        Models::Pull(spec) => net::Net::new(settings, crate::github_token(host))
            .and_then(|net| llm::cli::pull(kit, &net, &spec, lang, &mut *out))
            .map(drop),
        Models::Remove(id) => llm::cli::remove(kit, &id, lang, &mut *out),
    };
    match result {
        Ok(()) => 0,
        Err(e) => {
            let _ = writeln!(out, "  x {e:#}");
            1
        }
    }
}

/// `versions <id>`: the lines of `id` its source offers.
pub(crate) fn version_lines(
    kit: &Kit,
    kit_env: &KitEnv,
    settings: &Settings,
    host: &Host,
    id: &str,
) -> i32 {
    let mut out = output();
    let result = (|| -> anyhow::Result<Vec<String>> {
        versions::check(id, None)?;
        let c = setup::catalog::Catalog::get()
            .find(id)
            .ok_or_else(|| anyhow::anyhow!("{id}"))?;
        let installer = install::installer(id).ok_or_else(|| anyhow::anyhow!("{id}"))?;
        let net = net::Net::new(settings, crate::github_token(host))?
            .with_github_cache(kit.data().join("cache").join("github"));
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let ctx = install::Ctx {
            kit,
            net: &net,
            env: kit_env,
            cancel: &cancel,
        };
        let lines = installer.lines(&ctx, c)?;
        let _ = versions::save_offered(kit, id, &lines);
        Ok(lines)
    })();
    match result {
        Ok(lines) => {
            for l in lines {
                let _ = writeln!(out, "  {l}");
            }
            0
        }
        Err(e) => {
            let _ = writeln!(out, "  x {e:#}");
            1
        }
    }
}

/// The help for this program: [`kit::app_exe`] names the AppImage when the
/// app runs from one, not the inner program of its temporary mount, which
/// the user cannot start.
pub(crate) fn help(lang: &str) -> String {
    help_text(lang, kit::Os::current(), kit::app_exe().as_deref())
}

#[cfg(test)]
#[path = "../../../test/rust/app/cli.rs"]
mod tests;
