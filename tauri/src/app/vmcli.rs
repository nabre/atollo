//! `atollo vm <project>` on the PC (LEVELS.md §4.4, §8.5): the report of
//! the project's VM, «Verifica strumenti», and the actions on a sandbox
//! (`--restart`, `--recreate`, `--refresh`), each through `decide`.
//!
//! The actions run only while Atollo is closed: the open app holds the
//! sandboxes' locks and sessions, and a second process would race it
//! (its panel «VM del progetto» does the same there). The command holds
//! the kit's instance while it runs, and a daemon of the kit's `sbx` it
//! started is stopped at its end: no VM runs without Atollo.

use std::io::{BufRead, Write};

use crate::common::i18n::t;
use crate::common::settings::Settings;
use crate::env::{Host, KitEnv};
use crate::isolation::{self, vm};
use crate::kit::Kit;
use crate::levels::report::{
    self, SandboxReport, SandboxRun, Verdict, VmAction, VmEffects, VmOnOff, VmReport,
};
use crate::levels::{decide, Facts, Placement, Request};

use super::cli::VmArgs;

/// The command: its exit code (0 all good; 1 the VM off, the sandbox
/// missing, a tool not `ok`, an action refused or failed; 2 is a usage
/// error, given by the parser).
pub fn run(
    kit: &Kit,
    kit_env: &KitEnv,
    settings: &Settings,
    host: &Host,
    lang: &str,
    args: &VmArgs,
) -> i32 {
    let mut out = super::cli::output();
    let stdin = std::io::stdin();
    let mut input = stdin.lock();
    run_with(
        kit, kit_env, settings, host, lang, args, &mut *out, &mut input,
    )
}

#[allow(clippy::too_many_arguments)]
fn run_with(
    kit: &Kit,
    kit_env: &KitEnv,
    settings: &Settings,
    host: &Host,
    lang: &str,
    args: &VmArgs,
    out: &mut dyn Write,
    input: &mut dyn BufRead,
) -> i32 {
    let dir = kit.projects().join(&args.project);
    if crate::kit::validate_project_name(&args.project).is_err()
        || !crate::platform::safe_path::is_dir(&kit.projects(), &dir)
    {
        let _ = writeln!(
            out,
            "  x {}",
            t(lang, "Progetto non trovato: {0}", &[&args.project])
        );
        return 1;
    }
    // The app's instance: held while the command runs when Atollo is
    // closed; with Atollo open, only the report and the check.
    let guard = match crate::platform::instance::acquire(kit) {
        Ok(crate::platform::instance::Instance::Primary(guard)) => Some(guard),
        _ => None,
    };
    let daemon_before = daemon_running(kit);
    let code = act_and_report(
        kit,
        kit_env,
        settings,
        host,
        lang,
        args,
        guard.is_some(),
        out,
        input,
    );
    if guard.is_some() && !daemon_before && daemon_running(kit) {
        isolation::stop_daemon(kit, host, kit_env);
    }
    drop(guard);
    code
}

/// True while the kit's `sbx` daemon runs.
fn daemon_running(kit: &Kit) -> bool {
    isolation::kit_sbx(kit).is_some_and(|exe| {
        matches!(
            isolation::daemon::current(kit, &exe),
            isolation::daemon::Daemon::Kit
        )
    })
}

#[allow(clippy::too_many_arguments)]
fn act_and_report(
    kit: &Kit,
    kit_env: &KitEnv,
    settings: &Settings,
    host: &Host,
    lang: &str,
    args: &VmArgs,
    closed: bool,
    out: &mut dyn Write,
    input: &mut dyn BufRead,
) -> i32 {
    let dir = kit.projects().join(&args.project);
    let req = report::sandbox_request(kit, &dir);
    let mut failed = false;
    if let Some(action) = args.action {
        if !closed {
            let _ = writeln!(
                out,
                "  x {}",
                t(
                    lang,
                    "Atollo è aperto: usa il pannello «VM del progetto» nella pagina del progetto, o chiudi Atollo e riprova.",
                    &[],
                )
            );
            return 1;
        }
        let name = vm::name(&args.project);
        if action == VmAction::Recreate && !confirm_recreate(&name, args.yes, lang, out, input) {
            let _ = writeln!(out, "  · {}", t(lang, "Nulla è cambiato.", &[]));
            return 1;
        }
        let facts = crate::levels::facts::facts(kit, host, kit_env, &req);
        let fx = CliVm {
            kit,
            kit_env,
            settings,
            host,
            lang,
        };
        match report::act(&fx, &req, &facts, action) {
            Ok(sessions) => {
                for s in &sessions {
                    let _ = writeln!(out, "  · {}", t(lang, "Sessione fermata: {0}", &[s]));
                }
                let _ = writeln!(out, "  √ {}", done_text(action, &name, lang));
            }
            Err(e) => {
                let _ = writeln!(out, "  x {}", report::act_error_text(&e, lang));
                failed = true;
            }
        }
    }
    let mut report = report_of(kit, kit_env, host, lang, &args.project);
    if args.check && report.vm == VmOnOff::On {
        for s in &mut report.sandboxes {
            let facts = crate::levels::facts::facts(kit, host, kit_env, &req);
            if let Err(e) = report::check(kit, host, kit_env, &req, &facts, &args.project, s, lang)
            {
                s.note = Some(report::act_error_text(&e, lang));
                failed = true;
            }
        }
    }
    if args.json {
        match serde_json::to_string_pretty(&report) {
            Ok(text) => {
                let _ = writeln!(out, "{text}");
            }
            Err(e) => {
                let _ = writeln!(out, "  x {e}");
                return 1;
            }
        }
    } else {
        let _ = print_report(&report, lang, out);
    }
    i32::from(failed || !all_good(&report))
}

/// The report of `project` from the PC, without the app: no sessions and
/// no published ports (they live in the open app).
fn report_of(kit: &Kit, kit_env: &KitEnv, host: &Host, lang: &str, project: &str) -> VmReport {
    let status = isolation::status(kit, host, kit_env, false);
    let on = status.level == isolation::Level::Vm;
    let ls = on
        .then(|| {
            isolation::sandboxes_json(kit, host, &isolation::sbx_session_env(kit_env, host)).ok()
        })
        .flatten();
    let seen = report::Seen {
        on,
        reason: (!on).then(|| isolation::why_no_vm(&status, lang)),
        ls,
        github: crate::projects::settings::load(kit, project).github_in_vm,
    };
    report::report(kit, project, seen, &|_| Vec::new())
}

/// «Ricrea» asks first (§8.5): what is lost and what stays, then `s`/`y`
/// to go on; `--yes` does not ask. Nothing read (no console): no.
pub fn confirm_recreate(
    name: &str,
    yes: bool,
    lang: &str,
    out: &mut dyn Write,
    input: &mut dyn BufRead,
) -> bool {
    let (lost, kept) = report::recreate_warning(lang);
    let _ = writeln!(
        out,
        "  ! {}",
        t(lang, "La sandbox {0} si rimuove e si rifà.", &[name])
    );
    let _ = writeln!(out, "    {lost}");
    let _ = writeln!(out, "    {kept}");
    if yes {
        return true;
    }
    let _ = write!(out, "  {} ", t(lang, "Procedo? [s/N]", &[]));
    let _ = out.flush();
    let mut line = String::new();
    if input.read_line(&mut line).is_err() {
        return false;
    }
    matches!(
        line.trim().to_lowercase().as_str(),
        "s" | "si" | "sì" | "y" | "yes" | "o" | "oui" | "j" | "ja"
    )
}

fn done_text(action: VmAction, name: &str, lang: &str) -> String {
    match action {
        VmAction::Restart => t(
            lang,
            "{0}: fermata; riparte con la prossima sessione in Atollo.",
            &[name],
        ),
        VmAction::Recreate => t(
            lang,
            "{0}: rimossa; si rifà con gli strumenti di adesso alla prossima sessione in Atollo.",
            &[name],
        ),
        VmAction::Refresh => t(
            lang,
            "{0}: strumenti per il Linux della VM aggiornati; il resto alla prossima sessione in Atollo.",
            &[name],
        ),
    }
}

/// True when the VM is on, the project's sandbox is there, and every tool
/// checked is `ok` (or not meant for the VM).
pub fn all_good(report: &VmReport) -> bool {
    report.vm == VmOnOff::On
        && report
            .sandboxes
            .iter()
            .all(|s| s.state != SandboxRun::Missing)
        && report.sandboxes.iter().all(|s| {
            s.tools.as_ref().is_none_or(|tools| {
                tools
                    .iter()
                    .all(|c| matches!(c.verdict, Verdict::Ok | Verdict::NotInVm))
            })
        })
}

/// The report as text: the project's sandbox with its state, mounts, ports
/// and tools.
pub fn print_report(report: &VmReport, lang: &str, out: &mut dyn Write) -> std::io::Result<()> {
    writeln!(
        out,
        "── {} ──",
        t(lang, "VM del progetto {0}", &[&report.project])
    )?;
    if report.vm == VmOnOff::Off {
        writeln!(out, "  x {}", t(lang, "La VM non è attiva.", &[]))?;
        if let Some(reason) = &report.reason {
            writeln!(out, "    {}", reason.replace('\n', "\n    "))?;
        }
    }
    for s in &report.sandboxes {
        print_sandbox(s, lang, out)?;
    }
    writeln!(
        out,
        "  · {}",
        if report.github {
            t(lang, "GitHub nella VM: sì", &[])
        } else {
            t(lang, "GitHub nella VM: no", &[])
        }
    )
}

fn state_text(state: SandboxRun, lang: &str) -> String {
    match state {
        SandboxRun::Running => t(lang, "accesa", &[]),
        SandboxRun::Stopped => t(lang, "spenta", &[]),
        SandboxRun::Missing => t(lang, "non c'è", &[]),
    }
}

fn print_sandbox(s: &SandboxReport, lang: &str, out: &mut dyn Write) -> std::io::Result<()> {
    let whose = t(lang, "Progetto", &[]);
    writeln!(out)?;
    writeln!(out, "  {whose}: {} ({})", s.name, state_text(s.state, lang))?;
    for m in &s.mounts {
        let mode = match m.mode {
            report::Mode::Rw => "rw",
            report::Mode::Ro => "ro",
        };
        writeln!(out, "    {mode}  {} → {}", m.host, m.guest)?;
    }
    for p in &s.host_ports {
        writeln!(
            out,
            "    {}",
            t(
                lang,
                "Porta del PC raggiunta: {0} ({1})",
                &[&p.port.to_string(), &p.what]
            )
        )?;
    }
    if let Some(note) = &s.note {
        writeln!(out, "    ! {note}")?;
    }
    for c in s.tools.iter().flatten() {
        let mark = match c.verdict {
            Verdict::Ok => "√",
            Verdict::NotInVm => "·",
            _ => "x",
        };
        let detail = match c.verdict {
            Verdict::Ok => c.found.clone().unwrap_or_default(),
            Verdict::OtherLine => t(
                lang,
                "atteso {0}, trovato {1}",
                &[
                    c.chosen.as_deref().unwrap_or("?"),
                    c.found.as_deref().unwrap_or("?"),
                ],
            ),
            Verdict::Missing => t(lang, "manca", &[]),
            Verdict::NotFromKit => t(
                lang,
                "non è del kit: {0}",
                &[c.path.as_deref().unwrap_or("?")],
            ),
            Verdict::NotInVm => t(lang, "non gira nella VM", &[]),
        };
        writeln!(out, "    {mark} {} {detail}", c.name)?;
    }
    Ok(())
}

/// The command line's [`VmEffects`] (Atollo closed): no session of the
/// app to stop; `sbx stop` and `sbx rm -f` of the kit's `sbx`; and of the
/// preparation only what needs no app: the tools for the VM's Linux, the
/// rest (databases, ports, `info.json`, the start) at the next session in
/// Atollo.
struct CliVm<'a> {
    kit: &'a Kit,
    kit_env: &'a KitEnv,
    settings: &'a Settings,
    host: &'a Host,
    lang: &'a str,
}

impl VmEffects for CliVm<'_> {
    fn sessions(&self, _sandbox: &str) -> Vec<String> {
        Vec::new()
    }

    fn end_sessions(&self, _sandbox: &str) {}

    fn stop_sandbox(&self, sandbox: &str) -> anyhow::Result<()> {
        let env = isolation::sbx_session_env(self.kit_env, self.host);
        isolation::stop_sandbox(self.kit, self.host, &env, sandbox)
    }

    fn remove_sandbox(&self, sandbox: &str) -> anyhow::Result<()> {
        let env = isolation::sbx_session_env(self.kit_env, self.host);
        isolation::remove_to_make_again(self.kit, self.host, &env, sandbox)
    }

    fn prepare(&self, req: &Request, facts: &Facts) -> anyhow::Result<()> {
        let Placement::Vm {
            project, program, ..
        } = decide(req, facts)
        else {
            anyhow::bail!("levels: not a placement in a VM");
        };
        let Some(guest) = vm::guest_target(self.kit.target()).map(|t| self.kit.for_target(t))
        else {
            return Ok(());
        };
        let wanted =
            crate::levels::prepare::wanted_tools(self.kit, project.as_deref(), Some(&program), &[]);
        for opts in crate::levels::prepare::guest_plans(&guest, &wanted) {
            let code = super::cli::run_plan(
                self.kit,
                self.kit_env,
                self.settings,
                self.host,
                self.lang,
                &opts,
            );
            anyhow::ensure!(code == 0, "prepare: {code}");
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../test/rust/app/vmcli.rs"]
mod tests;
