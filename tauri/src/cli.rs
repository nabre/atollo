//! Command line of the same exe (DECISIONS.md D-21): `env`, `components`,
//! `versions`, `check`, `prepare`, `sync`, `keyring`, `serve`, `version`, `help`. The exe is a GUI program, so output goes
//! to the redirected handle when there is one, otherwise to the parent
//! console. Step lines use the markers of nabre/dev-kit's setup output
//! (`── N. Name ──`, `√`, `x`, `!`, `·`), so they read the same.

use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{self, Write};

use windows::Win32::System::Console::{
    AttachConsole, GetStdHandle, ATTACH_PARENT_PROCESS, STD_OUTPUT_HANDLE,
};

use crate::env::KitEnv;
use crate::i18n::t;
use crate::kit::Kit;
use crate::setup::catalog::{Catalog, Choice};
use crate::setup::runner::{Event, Options, Outcome, Sink};

#[derive(Debug, PartialEq, Eq)]
pub enum Cli {
    /// No command: open the window.
    Gui,
    /// The kit environment; with `--project <name>`, the one of its programs.
    Env {
        project: Option<String>,
    },
    Prepare(PrepareArgs),
    Components,
    /// The saved versions; with an id, the lines that can be downloaded.
    Versions(Option<String>),
    /// Every installed program answers its version command.
    Check,
    /// Aligns the agents (`--project <folder>`, `--verbose`).
    Sync {
        project: Option<std::path::PathBuf>,
        verbose: bool,
    },
    /// Antigravity's login: `restore` before agy, `save` after.
    Keyring(String),
    /// File server of a static site: `serve <folder> <port>` (projects/run.rs).
    Serve {
        dir: std::path::PathBuf,
        port: u16,
    },
    Version,
    Help,
    Unknown(String),
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct PrepareArgs {
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
            only: self.only.clone(),
            skip: self.skip.clone(),
            update_all: self.update_all,
            force: self.force,
            lines: self.lines.clone(),
            at: Vec::new(),
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
        "components" => Cli::Components,
        "versions" if args.len() <= 2 => {
            Cli::Versions(args.get(1).map(|a| a.to_string_lossy().to_lowercase()))
        }
        "check" => Cli::Check,
        "sync" => {
            let mut project = None;
            let mut verbose = false;
            let mut rest = args[1..].iter();
            while let Some(arg) = rest.next() {
                match arg.to_string_lossy().as_ref() {
                    "--project" => project = rest.next().map(std::path::PathBuf::from),
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
        "version" | "--version" | "-V" => Cli::Version,
        "help" | "--help" | "-h" | "/?" => Cli::Help,
        other => Cli::Unknown(other.to_owned()),
    }
}

/// Where command output goes.
pub fn output() -> Box<dyn Write> {
    // SAFETY: plain handle query.
    let redirected =
        unsafe { GetStdHandle(STD_OUTPUT_HANDLE) }.is_ok_and(|h| !h.is_invalid() && !h.0.is_null());
    if redirected {
        return Box::new(io::stdout());
    }
    // SAFETY: attaches to the console of the parent (cmd, PowerShell) if any.
    if unsafe { AttachConsole(ATTACH_PARENT_PROCESS) }.is_ok() {
        if let Ok(console) = OpenOptions::new().write(true).open("CONOUT$") {
            return Box::new(console);
        }
    }
    Box::new(io::sink())
}

/// `KEY=VALUE` lines, sorted, as `set` prints them.
pub fn print_env(env: &KitEnv, out: &mut dyn Write) -> io::Result<()> {
    for (name, value) in env.iter() {
        writeln!(out, "{name}={value}")?;
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
    for c in catalog.all() {
        let mark = if choice.is_chosen(&c.id) {
            "[x]"
        } else {
            "[ ]"
        };
        let state = if c.is_installed(kit) {
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
    names: Vec<(String, String)>,
}

impl<'a> Printer<'a> {
    pub fn new(out: &'a mut dyn Write, lang: &'a str) -> Printer<'a> {
        Printer {
            out,
            lang,
            names: Vec::new(),
        }
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
                            "{0}: non ancora disponibile in questa versione di NEXKORA",
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
                            line.push_str(&format!(
                                "
    {}",
                                t(
                                    lang,
                                    "Scaricalo dal browser ({0}), mettilo nella cartella data\\downloads del kit e riprova.",
                                    &[&b.url]
                                )
                            ));
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

#[cfg(test)]
#[path = "../../test/rust/cli.rs"]
mod tests;
