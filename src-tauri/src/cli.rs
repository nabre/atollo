//! Command line of the same exe (DECISIONS.md D-21): `env`, `prepare`,
//! `components`, `version`, `help`. The exe is a GUI program, so output goes
//! to the redirected handle when there is one, otherwise to the parent
//! console. Step lines use the markers of nabre/atollo's setup output
//! (`── N. Name ──`, `√`, `x`, `!`, `·`), so they read the same.

use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{self, Write};

use windows::Win32::System::Console::{
    AttachConsole, GetStdHandle, ATTACH_PARENT_PROCESS, STD_OUTPUT_HANDLE,
};

use crate::catalog::{Catalog, Choice};
use crate::env::KitEnv;
use crate::i18n::t;
use crate::kit::Kit;
use crate::setup_runner::{Event, Options, Outcome, Sink};

#[derive(Debug, PartialEq, Eq)]
pub enum Cli {
    /// No command: open the window.
    Gui,
    Env,
    Prepare(PrepareArgs),
    Components,
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
}

impl PrepareArgs {
    pub fn options(&self) -> Options {
        Options {
            only: self.only.clone(),
            skip: self.skip.clone(),
            update_all: self.update_all,
            force: self.force,
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
        "env" => Cli::Env,
        "components" => Cli::Components,
        "prepare" => {
            let mut p = PrepareArgs::default();
            let mut rest = args[1..].iter();
            while let Some(arg) = rest.next() {
                match arg.to_string_lossy().as_ref() {
                    "--only" => p.only.extend(ids(rest.next())),
                    "--skip" => p.skip.extend(ids(rest.next())),
                    "--update-all" => p.update_all = true,
                    "--force" => p.force = true,
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
                    Outcome::Failed { message } => format!(
                        "  x {}",
                        t(lang, "{0}: non riuscito: {1}", &[&name, &message])
                    ),
                }
            }
        };
        let _ = writeln!(self.out, "{line}");
        let _ = self.out.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<OsString> {
        list.iter().map(OsString::from).collect()
    }

    #[test]
    fn commands() {
        assert_eq!(parse(&[]), Cli::Gui);
        assert_eq!(parse(&args(&["env"])), Cli::Env);
        assert_eq!(parse(&args(&["--version"])), Cli::Version);
        assert_eq!(parse(&args(&["/?"])), Cli::Help);
        assert_eq!(parse(&args(&["components"])), Cli::Components);
        assert_eq!(parse(&args(&["x"])), Cli::Unknown("x".into()));
    }

    #[test]
    fn prepare_options() {
        let parsed = parse(&args(&[
            "prepare",
            "--only",
            "pwsh,Terminal",
            "--skip",
            "codex;rust",
            "--update-all",
            "--force",
        ]));
        assert_eq!(
            parsed,
            Cli::Prepare(PrepareArgs {
                only: vec!["pwsh".into(), "terminal".into()],
                skip: vec!["codex".into(), "rust".into()],
                update_all: true,
                force: true,
            })
        );
        assert_eq!(
            parse(&args(&["prepare", "--bogus"])),
            Cli::Unknown("prepare --bogus".into())
        );
    }

    #[test]
    fn printer_uses_kit_markers() {
        let mut buf = Vec::new();
        {
            let mut p = Printer::new(&mut buf, "it");
            p.emit(Event::Step {
                index: 1,
                total: 2,
                id: "pwsh".into(),
                name: "PowerShell 7".into(),
            });
            p.emit(Event::Done {
                id: "pwsh".into(),
                outcome: Outcome::Installed {
                    version: "7.6.7".into(),
                },
            });
            p.emit(Event::Done {
                id: "pwsh".into(),
                outcome: Outcome::Failed {
                    message: "boom".into(),
                },
            });
        }
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("── 1. PowerShell 7 ──"));
        assert!(text.contains("  √ PowerShell 7: installato 7.6.7"));
        assert!(text.contains("  x PowerShell 7: non riuscito: boom"));
    }
}
