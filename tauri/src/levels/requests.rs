//! The request files (LEVELS.md §4.3): how the processes outside the app,
//! the editor's profiles and the forwarders of the agents' commands, ask
//! the levels where to run. They cannot call a Tauri command; they write a
//! request in `data/app/levels/requests/` and run what the reply says.
//!
//! - `<id>.json` (`<id>`: 32 hex digits, random per request), written by
//!   the client as `<id>.tmp` then renamed: `{ "v": 1, "origin": {…},
//!   "cwd": "…", "args": […] }`. Only the origins of §4.3 are asked this way
//!   (an editor's task, an editor's agent profile, a forwarded command, an
//!   editor extension); level 1 never asks (R3/R4).
//! - [`take`] renames it to `<id>.taken` before reading it: a request is
//!   taken once, and a client that gives up (no app, §4.3) removes its
//!   `<id>.json` only while nobody took it.
//! - The app answers each through `decide` → `environment` → `prepare`
//!   ([`reply_to`]) and writes the reply twice, atomically: `<id>.reply.json`
//!   for `launch.ps1` and `<id>.reply.sh` (the same, as POSIX sh that sets
//!   its variables and `set --` the arguments) for `launch.sh`, which has no
//!   JSON reader. The client deletes both.
//! - `<project>.pi.<pid>`: a client runs Pi for the project (its reply's
//!   `marker`), `<pid>` its own process's; written before, removed when the
//!   session ends ([`pi_markers`]). The port of the local models stays open
//!   only while a marker's pid runs a client (`isolation::editor_pis_alive`).
//!
//! The folder is the PC's only, never mounted in a VM (D-168): a project's
//! code in a VM cannot write a request or a marker. A request can only ask
//! for what `decide` allows the user from the app. The app clears the folder
//! at its start ([`clear_all`]): what the last run left holds nothing.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::env::{Host, KitEnv};
use crate::kit::Kit;

use super::env::{environment, Environment};
use super::prepare::{prepare, Effects, Ready};
use super::{labels, Agent, Facts, Fix, Level, Origin, Placement, Refusal, Request, VmProgram};

/// The version of the request and the reply.
pub const VERSION: u32 = 1;
/// How often the app looks for requests.
pub const POLL: Duration = Duration::from_millis(250);
/// A reply nobody read, or a file a client left half-written, after this
/// goes.
pub const STALE: Duration = Duration::from_secs(15 * 60);
/// The reply for `launch.ps1`.
pub const REPLY_JSON: &str = ".reply.json";
/// The reply for `launch.sh`.
pub const REPLY_SH: &str = ".reply.sh";
/// What follows `<project>` in the name of a marker of a client running
/// Pi: `<project>.pi.<pid>`.
pub const PI_MARKER: &str = ".pi.";

/// `data/app/levels/requests`.
pub fn dir(kit: &Kit) -> PathBuf {
    kit.app_dir().join("levels").join("requests")
}

/// True for a request's id: 32 lowercase hex digits.
pub fn is_id(id: &str) -> bool {
    id.len() == 32
        && id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// A request as the client writes it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wire {
    pub v: u32,
    pub origin: Origin,
    pub cwd: PathBuf,
    #[serde(default)]
    pub args: Vec<String>,
}

/// A request taken from the folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asked {
    pub id: String,
    pub origin: Origin,
    pub cwd: PathBuf,
    /// What the user typed after the agent's command (forwarders).
    pub args: Vec<String>,
}

/// What [`take`] found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Taken {
    Asked(Asked),
    /// A request with an id that cannot be read, of another version, or of
    /// an origin that never asks: answered with an error, nothing started.
    Malformed {
        id: String,
        why: String,
    },
}

/// The origins that ask through a request (§4.3): the editor's tasks and
/// agent profiles, the forwarders, the editor's extensions.
pub fn asks_by_request(origin: &Origin) -> bool {
    matches!(
        origin,
        Origin::EditorTask { .. }
            | Origin::EditorAgentProfile { .. }
            | Origin::Forwarded { .. }
            | Origin::EditorExtension { .. }
    )
}

/// The request in `bytes`, of request `id`.
pub fn parse(id: &str, bytes: &[u8]) -> Taken {
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    let wire: Wire = match serde_json::from_slice(bytes) {
        Ok(wire) => wire,
        Err(e) => {
            return Taken::Malformed {
                id: id.to_owned(),
                why: format!("not a request: {e}"),
            }
        }
    };
    if wire.v != VERSION {
        return Taken::Malformed {
            id: id.to_owned(),
            why: format!("request version {}, not {VERSION}", wire.v),
        };
    }
    if !asks_by_request(&wire.origin) {
        return Taken::Malformed {
            id: id.to_owned(),
            why: "this origin does not ask through a request".to_owned(),
        };
    }
    Taken::Asked(Asked {
        id: id.to_owned(),
        origin: wire.origin,
        cwd: wire.cwd,
        args: wire.args,
    })
}

/// The requests waiting in the folder, each taken once (renamed to
/// `<id>.taken`, read, removed). A file named like no request is removed
/// and ignored; replies nobody read and files left half-written go after
/// [`STALE`]. Markers stay.
pub fn take(kit: &Kit) -> Vec<Taken> {
    let root = dir(kit);
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&root).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();
        if name.ends_with(REPLY_JSON) || name.ends_with(REPLY_SH) || name.contains(".tmp") {
            let old = entry
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.elapsed().ok())
                .is_some_and(|age| age > STALE);
            if old {
                let _ = std::fs::remove_file(&path);
            }
            continue;
        }
        let Some(id) = name.strip_suffix(".json") else {
            continue;
        };
        if !is_id(id) {
            let _ = std::fs::remove_file(&path);
            continue;
        }
        let taken = root.join(format!("{id}.taken"));
        // Gone meanwhile: its client gave up.
        if std::fs::rename(&path, &taken).is_err() {
            continue;
        }
        let bytes = std::fs::read(&taken).unwrap_or_default();
        let _ = std::fs::remove_file(&taken);
        out.push(parse(id, &bytes));
    }
    out
}

/// What the client applies to its environment before it runs the program.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct EnvChange {
    pub set: BTreeMap<String, String>,
    pub unset: Vec<String>,
}

impl EnvChange {
    /// What turns `from` into `to`: the variables `to` has with another
    /// value, those only `from` has. Names compared as the system does
    /// (`KitEnv::get`).
    pub fn between(from: &KitEnv, to: &KitEnv) -> EnvChange {
        let mut change = EnvChange::default();
        for (k, v) in to.iter() {
            if from.get(k) != Some(v) {
                change.set.insert(k.to_owned(), v.to_owned());
            }
        }
        for (k, _) in from.iter() {
            if to.get(k).is_none() {
                change.unset.push(k.to_owned());
            }
        }
        change
    }
}

/// Where the reply says to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Where {
    Vm,
    Pc,
    Refused,
    /// The request could not be answered (malformed, or the VM could not be
    /// made ready): `text` says why; nothing runs.
    Error,
}

/// The reply to a request (§4.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reply {
    pub v: u32,
    pub placement: Where,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<Level>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The program to run, with `argv`, in the client's own terminal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,
    pub argv: Vec<String>,
    pub env: EnvChange,
    /// Printed on stderr before the program.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<String>,
    /// Pi's session: the marker's name without its pid
    /// (`<project>.pi`, [`PI_MARKER`]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<Refusal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<Fix>,
    /// Why nothing runs (a refusal with what to do, or an error).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

impl Reply {
    fn empty(placement: Where) -> Reply {
        Reply {
            v: VERSION,
            placement,
            level: None,
            label: None,
            program: None,
            argv: Vec::new(),
            env: EnvChange::default(),
            banner: None,
            marker: None,
            refusal: None,
            fix: None,
            text: None,
        }
    }

    /// Nothing runs: `text` says why.
    pub fn error(text: impl Into<String>) -> Reply {
        Reply {
            text: Some(text.into()),
            ..Reply::empty(Where::Error)
        }
    }

    /// The refusal of `placement` (a `Placement::Refused`): its reason, its
    /// fix and their text in `lang` (I7).
    pub fn refused(placement: &Placement, lang: &str) -> Reply {
        let Placement::Refused { level, reason, fix } = placement else {
            return Reply::error("levels: not a refusal");
        };
        Reply {
            level: Some(*level),
            refusal: Some(reason.clone()),
            fix: Some(*fix),
            text: Some(labels::refusal_text(reason, lang)),
            ..Reply::empty(Where::Refused)
        }
    }

    /// The run of `program` with `argv` as `placement` placed it.
    fn run(
        placement: &Placement,
        lang: &str,
        program: String,
        argv: Vec<String>,
        env: EnvChange,
    ) -> Reply {
        let label = labels::label(placement, lang);
        let banner = labels::banner_line(placement, lang)
            .or_else(|| label.as_ref().map(|l| format!("[Atollo] {l}")));
        Reply {
            level: Some(placement.level()),
            label,
            program: Some(program),
            argv,
            env,
            banner,
            ..Reply::empty(match placement {
                Placement::Vm { .. } => Where::Vm,
                _ => Where::Pc,
            })
        }
    }
}

/// The words for the agent of a request of `origin`: as asked, but for an
/// editor extension's wrapper (LEVELS.md §6.9, Q2), which the extension
/// runs as `<wrapper> <its bundled binary> <its flags…>` ("The bundled
/// binary path is passed as an argument when present"): its first word,
/// when it is not a flag, is that PC path and is dropped; the agent in the
/// VM is the sandbox's own `claude`. The flags stay as they are.
pub fn agent_args(origin: &Origin, args: &[String]) -> Vec<String> {
    match (origin, args.split_first()) {
        (Origin::EditorExtension { .. }, Some((first, rest))) if !first.starts_with('-') => {
            rest.to_vec()
        }
        _ => args.to_vec(),
    }
}

/// The reply to `req` (with the user's `args`) decided with `facts`
/// (LEVELS.md §4.3): a refusal with its reason, fix and text; on the PC
/// (an editor's task without the VM, R5/R6) the kit's shell running the
/// task's line, with what changes from the editor server's environment
/// (`editor_env`) to `environment(Pc{Project})`; in a VM, the sandbox made
/// ready by `prepare` (with `fx`) and the `sbx` arguments of the session,
/// with `sbx`'s variables. `base`: the kit's environment for the PC.
#[allow(clippy::too_many_arguments)]
pub fn reply_to(
    fx: &dyn Effects,
    kit: &Kit,
    base: &KitEnv,
    editor_env: &KitEnv,
    host: &Host,
    req: &Request,
    args: &[String],
    facts: &Facts,
    lang: &str,
) -> Reply {
    let args = &agent_args(&req.origin, args);
    let placement = super::decide(req, facts);
    if let Placement::Refused { .. } = placement {
        return Reply::refused(&placement, lang);
    }
    let Some(env) = environment(kit, base, &placement, None, lang, host) else {
        return Reply::error("levels: refused");
    };
    match &placement {
        Placement::Pc { .. } => {
            let Origin::EditorTask { command } = &req.origin else {
                return Reply::error("levels: only an editor's task runs on the PC");
            };
            let Environment::Pc(to) = &env else {
                return Reply::error("levels: not a placement on the PC");
            };
            match task_on_pc(kit, command) {
                Some((program, argv)) => Reply::run(
                    &placement,
                    lang,
                    program,
                    argv,
                    EnvChange::between(editor_env, to),
                ),
                None => Reply::error(crate::common::i18n::t(
                    lang,
                    "Senza il PowerShell del kit i task dell'editor non partono: installalo da Strumenti.",
                    &[],
                )),
            }
        }
        _ => match prepare(fx, kit, host, placement.clone(), env, &[], lang) {
            Ok(ready) => {
                vm_reply(kit, &ready, args, lang).unwrap_or_else(|e| Reply::error(format!("{e:#}")))
            }
            Err(e) => Reply::error(format!("{e:#}")),
        },
    }
}

/// The reply of a `Vm` placement made ready: `sbx` with the session's
/// arguments ([`super::start::vm_session_args`], `args` after the agent's
/// own), `sbx`'s temporary folder and daemon variables, the banner; Pi's
/// marker.
fn vm_reply(kit: &Kit, ready: &Ready, args: &[String], lang: &str) -> Result<Reply> {
    let (
        Some(vm),
        Environment::Vm { sbx_env, .. },
        Placement::Vm {
            project,
            sandbox,
            program,
            ..
        },
    ) = (&ready.vm, &ready.env, &ready.placement)
    else {
        anyhow::bail!("levels: not ready in a VM");
    };
    let argv = super::start::vm_session_args(kit, ready, args)?;
    let mut env = EnvChange::default();
    let wanted = ["TEMP", "TMP", "TMPDIR"]
        .into_iter()
        .chain(crate::isolation::daemon::ENV.iter().map(|(k, _)| *k));
    for name in wanted {
        if let Some(value) = sbx_env.get(name) {
            env.set.insert(name.to_owned(), value.to_owned());
        }
    }
    let mut reply = Reply::run(
        &ready.placement,
        lang,
        vm.sbx_path().display().to_string(),
        argv,
        env,
    );
    if let VmProgram::Agent { agent: Agent::Pi } = program {
        // A folder's sandbox has no project: its sandbox names it (D-187).
        let owner = project.as_deref().unwrap_or(sandbox.name());
        reply.marker = Some(format!("{owner}.pi"));
    }
    Ok(reply)
}

/// The kit's shell running an editor's task `line` on the PC: the kit's
/// PowerShell on Windows (Git Bash without it), the system's bash on
/// macOS and Linux (tasks written for sh and bash, never the user's login
/// shell); `None` on Windows with neither.
pub fn task_on_pc(kit: &Kit, line: &str) -> Option<(String, Vec<String>)> {
    if kit.target().is_windows() {
        let pwsh = crate::setup::kit_path(kit, r"pwsh\pwsh.exe");
        if pwsh.is_file() {
            let args = [
                "-NoLogo",
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                line,
            ];
            return Some((pwsh.display().to_string(), args.map(String::from).into()));
        }
        let bash = crate::setup::kit_path(kit, r"git\bin\bash.exe");
        return bash
            .is_file()
            .then(|| (bash.display().to_string(), vec!["-c".into(), line.into()]));
    }
    Some((
        crate::editor::cfg::BASH.to_owned(),
        vec!["-c".into(), line.into()],
    ))
}

/// `text` as one word of POSIX sh, in single quotes.
pub fn sh_quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', r"'\''"))
}

/// A name sh can set: letters, digits and `_`, not first a digit.
fn sh_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with(|c: char| c.is_ascii_digit())
        && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// The reply as POSIX sh for `launch.sh` (`windows`: the kit's system,
/// where the client is Git Bash's sh): its fields as variables
/// (`placement`, `level`, `label`, `banner`, `text`, `program`, `marker`),
/// the environment's changes in `atollo_env`, the arguments with `set --`.
/// On Windows the program's path has `/` and the session in a VM asks
/// Git Bash to leave its arguments as they are (no path conversion: they
/// are the VM's paths).
pub fn reply_sh(reply: &Reply, windows: bool) -> String {
    let mut out = String::from("# Atollo: the reply to a request (levels::requests).\n");
    let field = |out: &mut String, name: &str, value: Option<&str>| {
        out.push_str(&format!("{name}={}\n", sh_quote(value.unwrap_or_default())));
    };
    let placement = serde_json::to_value(reply.placement)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default();
    field(&mut out, "placement", Some(&placement));
    let level = reply.level.map(super::env::level_value);
    field(&mut out, "level", level);
    field(&mut out, "label", reply.label.as_deref());
    field(&mut out, "banner", reply.banner.as_deref());
    field(&mut out, "text", reply.text.as_deref());
    let program = reply.program.as_deref().map(|p| {
        if windows {
            p.replace('\\', "/")
        } else {
            p.to_owned()
        }
    });
    field(&mut out, "program", program.as_deref());
    field(&mut out, "marker", reply.marker.as_deref());
    out.push_str("atollo_env() {\n");
    for (k, v) in reply.env.set.iter().filter(|(k, _)| sh_name(k)) {
        out.push_str(&format!("    export {}\n", sh_quote(&format!("{k}={v}"))));
    }
    if windows && reply.placement == Where::Vm {
        out.push_str("    export MSYS_NO_PATHCONV=1 MSYS2_ARG_CONV_EXCL='*'\n");
    }
    for k in reply.env.unset.iter().filter(|k| sh_name(k)) {
        out.push_str(&format!("    unset {}\n", sh_quote(k)));
    }
    out.push_str("    :\n}\nset --");
    for arg in &reply.argv {
        out.push(' ');
        out.push_str(&sh_quote(arg));
    }
    out.push('\n');
    out
}

/// Writes the reply to request `id`: `<id>.reply.json`, then
/// `<id>.reply.sh`, each atomically. Both clients wait for the one written
/// last, then read theirs and delete both.
pub fn answer(kit: &Kit, id: &str, reply: &Reply) -> Result<()> {
    let root = dir(kit);
    std::fs::create_dir_all(&root).with_context(|| root.display().to_string())?;
    let json = root.join(format!("{id}{REPLY_JSON}"));
    let text = serde_json::to_string(reply)?;
    crate::common::fsx::write_atomic(&json, text.as_bytes())
        .with_context(|| json.display().to_string())?;
    let sh = root.join(format!("{id}{REPLY_SH}"));
    crate::common::fsx::write_atomic(&sh, reply_sh(reply, kit.target().is_windows()).as_bytes())
        .with_context(|| sh.display().to_string())
}

/// Answers every request of the folder while the app runs (LEVELS.md
/// §4.3): it looks every [`POLL`], answers a malformed one with its error,
/// and each request in a thread of its own with `answer_with` (a VM may
/// take minutes to be ready, the others do not wait for it).
pub fn serve<F>(kit: Kit, answer_with: F)
where
    F: Fn(Asked) -> Reply + Send + Sync + 'static,
{
    let answer_with = std::sync::Arc::new(answer_with);
    std::thread::spawn(move || loop {
        std::thread::sleep(POLL);
        for taken in take(&kit) {
            match taken {
                Taken::Malformed { id, why } => {
                    crate::common::log::warn(&format!("levels request {id}: {why}"));
                    if let Err(e) = answer(&kit, &id, &Reply::error(why)) {
                        crate::common::log::warn(&format!("levels request {id}: {e:#}"));
                    }
                }
                Taken::Asked(asked) => {
                    let (kit, answer_with) = (kit.clone(), std::sync::Arc::clone(&answer_with));
                    std::thread::spawn(move || {
                        let id = asked.id.clone();
                        let reply = answer_with(asked);
                        if let Err(e) = answer(&kit, &id, &reply) {
                            crate::common::log::warn(&format!("levels request {id}: {e:#}"));
                        }
                    });
                }
            }
        }
    });
}

/// The pids of the markers of `project` among the file names `names`
/// (`<project>.pi.<pid>`, the pid digits only): another project's
/// (`shop.pi.5.pi.7` is `shop.pi.5`'s), a request or anything else is not
/// one.
pub fn pi_marker_pids<'a>(names: impl IntoIterator<Item = &'a str>, project: &str) -> Vec<u32> {
    let prefix = format!("{project}{PI_MARKER}");
    let mut pids: Vec<u32> = names
        .into_iter()
        .filter_map(|name| name.strip_prefix(&prefix))
        .filter(|pid| !pid.is_empty() && pid.bytes().all(|b| b.is_ascii_digit()))
        .filter_map(|pid| pid.parse().ok())
        .collect();
    pids.sort_unstable();
    pids.dedup();
    pids
}

/// The pids of the clients running Pi for `project` that noted a marker
/// ([`pi_marker_pids`]); a stale marker is told apart by its pid
/// (`isolation::editor_pis_alive`).
pub fn pi_markers(kit: &Kit, project: &str) -> Vec<u32> {
    let names: Vec<String> = std::fs::read_dir(dir(kit))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    pi_marker_pids(names.iter().map(String::as_str), project)
}

/// Everything of the last run gone (the app's start): requests, replies,
/// markers.
pub fn clear_all(kit: &Kit) -> Result<()> {
    let root = dir(kit);
    match std::fs::remove_dir_all(&root) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            Err(e).with_context(|| root.display().to_string())
        }
        _ => Ok(()),
    }
}

/// `place_of` the request's folder: the request the levels decide.
pub fn request_of(kit: &Kit, asked: &Asked) -> Request {
    Request {
        origin: asked.origin.clone(),
        place: super::place::place_of(kit, &asked.cwd),
    }
}

#[cfg(test)]
#[path = "../../../test/rust/levels/requests.rs"]
mod tests;
