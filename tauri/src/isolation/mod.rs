//! How shut in a project's programs are (D-167), worked out by itself:
//! nothing to turn on in the settings. The VM (a Docker Sandboxes microVM
//! per project, `sbx`) when the PC has what it needs; otherwise the kit's
//! environment only, with the reasons the VM is off and, for each one, what
//! the user does about it (the interface's notice in the status bar).
//!
//! `sbx` is the kit's (the required `sandboxes` component, in `data`),
//! never one of the PC (D-175): it runs with the kit's environment, so its
//! sandboxes, settings and state travel with the kit. Its sign-in to
//! Docker stays in the system's keyring, a declared trace.
//!
//! On every level a project's programs get a sealed environment
//! (`levels::env::seal`): less of what reaches them by itself. It is no wall (they run
//! as the user and can read what the user can): the VM is.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::env::{Host, KitEnv};
use crate::kit::Kit;
use crate::platform::processes::{path_key, Proc};
use crate::platform::virt::{self, Gap};

pub mod daemon;
pub mod kit;
pub mod secrets;
pub mod vm;

/// How long a status stays good: `sbx` is asked at most once a minute.
const FRESH: Duration = Duration::from_secs(60);
/// How long `sbx` may take to answer.
const ASK_LIMIT: Duration = Duration::from_secs(15);

// The sealed environment (`SEALED`, `seal`) and the stripping of the
// agents (`agent_dirs`, `strip_agents`) are in `levels::env` (D-179).

/// What shuts a project's programs in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    /// A Docker Sandboxes microVM per project.
    Vm,
    /// The kit's environment only: the programs run as the user, on the PC.
    Env,
}

/// Why the VM is off: the interface says it and what to do, by `code`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum Blocker {
    OldWindows {
        build: u32,
    },
    OldMacos {
        version: String,
    },
    Distribution {
        name: String,
    },
    HypervisorOff,
    KvmMissing,
    KvmDenied,
    /// AppArmor keeps the kit's microVM shim (its path) from starting.
    AppArmor {
        shim: String,
    },
    /// The kit's Docker Sandboxes is not installed (one of the PC is
    /// never used, D-175); `failed`: why its last installation failed.
    SbxMissing {
        #[serde(skip_serializing_if = "Option::is_none")]
        failed: Option<String>,
    },
    /// `sbx` is there but not signed in to Docker.
    SbxSignedOut,
    /// `sbx` answered with an error (its text).
    SbxFailed {
        message: String,
    },
    /// The daemon running is another `sbx`'s (the PC's, another kit's):
    /// the kit's VMs would go into its state, out of the kit
    /// ([`daemon::Daemon::Other`]).
    ForeignDaemon,
    /// The kit's disk has no sparse files nor permissions (FAT32, exFAT):
    /// each VM's disks (20 + 10 GB, sparse on NTFS) would not fit or would
    /// take their whole size.
    KitDisk {
        fs: String,
    },
}

impl From<Gap> for Blocker {
    fn from(gap: Gap) -> Blocker {
        match gap {
            Gap::OldWindows(build) => Blocker::OldWindows { build },
            Gap::OldMacos(version) => Blocker::OldMacos { version },
            Gap::Distribution(name) => Blocker::Distribution { name },
            Gap::HypervisorOff => Blocker::HypervisorOff,
            Gap::KvmMissing => Blocker::KvmMissing,
            Gap::KvmDenied => Blocker::KvmDenied,
            Gap::AppArmor(shim) => Blocker::AppArmor {
                shim: shim.display().to_string(),
            },
        }
    }
}

/// The commands of the PC that ask for an administrator or a new sign-in:
/// shown to be copied, never run by Atollo (zero admin, D-178).
pub const HYPERVISOR_COMMAND: &str =
    "Enable-WindowsOptionalFeature -Online -FeatureName HypervisorPlatform -All";
pub const KVM_COMMAND: &str = "sudo usermod -aG kvm $USER";

impl Blocker {
    /// A system Docker does not support: nothing the user installs helps.
    pub fn unsupported(&self) -> bool {
        matches!(
            self,
            Blocker::OldWindows { .. } | Blocker::OldMacos { .. } | Blocker::Distribution { .. }
        )
    }

    /// Why the VM is off and what to do, in a line of `lang`, for the
    /// places outside the notice (the editor's terminals, an agent
    /// refused): an administrator's command is only named, to be run by
    /// the user; the notice of the status bar has the others.
    pub fn explain(&self, lang: &str) -> String {
        use crate::common::i18n::t;
        match self {
            Blocker::OldWindows { build } => t(
                lang,
                "Windows build {0} non è Windows 11: Docker Sandboxes funziona solo su Windows 11.",
                &[&build.to_string()],
            ),
            Blocker::OldMacos { version } => t(
                lang,
                "macOS {0}: Docker Sandboxes richiede macOS 14 Sonoma o successivo.",
                &[version],
            ),
            Blocker::Distribution { name } => t(
                lang,
                "{0}: Docker Sandboxes supporta solo Ubuntu 24.04 o successivo (non le derivate).",
                &[name],
            ),
            Blocker::HypervisorOff => t(
                lang,
                "Windows Hypervisor Platform non è attivo: in PowerShell come amministratore «{0}», poi riavvia il PC.",
                &[HYPERVISOR_COMMAND],
            ),
            Blocker::KvmMissing => t(
                lang,
                "/dev/kvm non c'è: attiva la virtualizzazione (Intel VT-x o AMD-V) nel BIOS/UEFI e riavvia il PC.",
                &[],
            ),
            Blocker::KvmDenied => t(
                lang,
                "Il tuo utente non è nel gruppo kvm: in un terminale del PC «{0}», poi esci e rientra nella sessione.",
                &[KVM_COMMAND],
            ),
            Blocker::AppArmor { .. } => t(
                lang,
                "AppArmor non lascia partire la microVM del kit: il comando del profilo, da copiare, è nell'avviso «Senza VM» di Atollo.",
                &[],
            ),
            Blocker::SbxMissing { .. } => t(
                lang,
                "Docker Sandboxes (sbx) non è nel kit: installalo con «Installa nel kit» nell'avviso «Senza VM» di Atollo.",
                &[],
            ),
            Blocker::SbxSignedOut => t(
                lang,
                "sbx non ha accesso a Docker: «Lancia» su «sbx login» nell'avviso «Senza VM» di Atollo.",
                &[],
            ),
            Blocker::SbxFailed { message } => {
                t(lang, "sbx ha risposto con un errore: {0}", &[message])
            }
            Blocker::ForeignDaemon => t(
                lang,
                "Sul PC è attivo un altro Docker Sandboxes: fermalo («sbx daemon stop»), poi «Ricontrolla» nell'avviso «Senza VM» di Atollo.",
                &[],
            ),
            Blocker::KitDisk { fs } => t(
                lang,
                "Il kit è su un disco {0}: spostalo su un disco NTFS (o APFS, ext4 su macOS e Linux).",
                &[fs],
            ),
        }
    }
}

/// Why the VM is off, one reason per line (`- …`), in `lang`; then that
/// Atollo checks again by itself.
pub fn why_no_vm(status: &Status, lang: &str) -> String {
    use crate::common::i18n::t;
    let mut out = t(lang, "Perché la VM non è attiva:", &[]);
    for b in &status.blockers {
        out.push_str("\n- ");
        out.push_str(&b.explain(lang));
    }
    out.push('\n');
    out.push_str(&t(
        lang,
        "Atollo ricontrolla da solo ogni minuto: quando la VM è pronta, riprova.",
        &[],
    ));
    out
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub level: Level,
    /// Empty when the level is the VM.
    pub blockers: Vec<Blocker>,
    /// The `sbx` found, for the notice.
    pub sbx: Option<String>,
}

impl Status {
    /// The level the blockers leave.
    pub fn of(blockers: Vec<Blocker>, sbx: Option<String>) -> Status {
        let level = if blockers.is_empty() {
            Level::Vm
        } else {
            Level::Env
        };
        Status {
            level,
            blockers,
            sbx,
        }
    }
}

/// The last status and when it was worked out.
static LAST: Mutex<Option<(Instant, Status)>> = Mutex::new(None);
/// When the VM was last found ready ([`status`]'s grace).
static GOOD: Mutex<Option<Instant>> = Mutex::new(None);
/// One [`compute`] at a time.
static COMPUTE: Mutex<()> = Mutex::new(());

/// How long a VM found ready outlives an `sbx` that does not answer for a
/// moment (another VM pulling its image, starting).
const GRACE: Duration = Duration::from_secs(180);

fn last() -> std::sync::MutexGuard<'static, Option<(Instant, Status)>> {
    LAST.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The last status worked out, however old, without asking `sbx`; `None`
/// before the first (`levels::agents::known_vm`).
pub fn last_status() -> Option<Status> {
    last().as_ref().map(|(_, st)| st.clone())
}

/// The status, asked again when older than a minute or when `force` (the
/// notice's "Ricontrolla", an installation of Docker Sandboxes).
///
/// One check at a time, and nobody waits behind it while an older answer
/// exists: a check can take minutes (a daemon started, restarted). An
/// `sbx` that fails for a moment ([`Blocker::SbxFailed`] alone) within
/// [`GRACE`] of a VM found ready keeps the VM, asked again at the next
/// call: the programs stay in their VM, whose start then says what fails,
/// never on the PC without a word.
pub fn status(kit: &Kit, host: &Host, env: &KitEnv, force: bool) -> Status {
    let fresh = |force: bool| {
        last()
            .as_ref()
            .filter(|(at, _)| !force && at.elapsed() < FRESH)
            .map(|(_, st)| st.clone())
    };
    if let Some(st) = fresh(force) {
        return st;
    }
    let _one = match COMPUTE.try_lock() {
        Ok(guard) => guard,
        Err(std::sync::TryLockError::Poisoned(p)) => p.into_inner(),
        Err(std::sync::TryLockError::WouldBlock) => {
            if let Some((_, st)) = last().as_ref().filter(|_| !force) {
                return st.clone();
            }
            COMPUTE
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        }
    };
    // Worked out meanwhile by the check this one waited for.
    if let Some(st) = fresh(force) {
        return st;
    }
    let st = compute(kit, host, env);
    let mut good = GOOD
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let transient = matches!(st.blockers.as_slice(), [Blocker::SbxFailed { .. }]);
    if st.level == Level::Vm {
        *good = Some(Instant::now());
    } else if transient && good.is_some_and(|at| at.elapsed() < GRACE) {
        crate::common::log::warn(&format!("sbx: {:?}: the VM kept", st.blockers));
        let kept = Status::of(Vec::new(), st.sbx.clone());
        // Not noted as fresh: the next call asks again.
        return kept;
    }
    *last() = Some((Instant::now(), st.clone()));
    st
}

fn compute(kit: &Kit, host: &Host, env: &KitEnv) -> Status {
    let sbx = Sbx::find(kit, host);
    let shim = sbx.as_ref().and_then(Sbx::shim);
    let mut blockers: Vec<Blocker> = virt::gaps(shim.as_deref())
        .into_iter()
        .map(Blocker::from)
        .collect();
    if blockers.iter().any(Blocker::unsupported) {
        return Status::of(blockers, None);
    }
    match &sbx {
        None => blockers.push(Blocker::SbxMissing {
            failed: install_failure(kit),
        }),
        Some(sbx) => {
            if let Err(b) = kit_daemon(kit, sbx, host, env)
                .and_then(|()| ask(sbx, host, env))
                .and_then(|()| policy(sbx, host, env))
            {
                blockers.push(b);
            } else {
                daemon_settings(kit, sbx, host, env);
                legacy_global_github(kit, sbx, host, env);
                legacy_terminals(kit, sbx, host, env);
            }
        }
    }
    Status::of(blockers, sbx.map(|s| s.path().display().to_string()))
}

/// Where the kit's `sbx` keeps its VMs (D-169): in the kit, so on a disk
/// with sparse files, and with a daemon of its own. The PC's `sbx` keeps
/// its own where the user installed it: nothing to check.
fn kit_daemon(kit: &Kit, sbx: &Sbx, host: &Host, env: &KitEnv) -> Result<(), Blocker> {
    let exe = sbx.path();
    if let Some(volume) = kit.volume().filter(|v| !v.fs.has_permissions()) {
        return Err(Blocker::KitDisk {
            fs: volume.fs.as_str().to_owned(),
        });
    }
    match daemon::current(kit, exe) {
        daemon::Daemon::Other => Err(Blocker::ForeignDaemon),
        daemon::Daemon::Kit => Ok(()),
        daemon::Daemon::None => {
            // Started here, its output to nothing: a daemon started by a
            // command whose output is read keeps that pipe open, and the
            // reader waits for it until its limit.
            // A daemon started now reads the settings as they are: a
            // restart owed to the one before is not owed any more.
            if let Some(temp) = host.temp() {
                daemon::unlink_foreign(Path::new(temp), &kit.home());
            }
            if daemon_call(sbx, host, env, &["daemon", "start"]) {
                let _ = std::fs::remove_file(kit::root(kit).join(".restart-pending"));
            }
            Ok(())
        }
    }
}

/// `sbx <args>` of the daemon (`daemon start`, `daemon restart`), its
/// output to nothing, waited for at most a minute.
fn daemon_call(sbx: &Sbx, host: &Host, env: &KitEnv, args: &[&str]) -> bool {
    let mut cmd = sbx.command(env, host);
    cmd.args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    let child = match crate::platform::jobs::spawn(&mut cmd) {
        Ok(child) => child,
        Err(e) => {
            crate::common::log::warn(&format!("sbx {}: {e}", args.join(" ")));
            return false;
        }
    };
    let (tx, rx) = mpsc::channel();
    let mut child = child;
    std::thread::spawn(move || {
        let _ = tx.send(child.wait());
    });
    match rx.recv_timeout(Duration::from_secs(60)) {
        Ok(Ok(status)) if status.success() => true,
        Ok(Ok(status)) => {
            crate::common::log::warn(&format!("sbx {}: {status}", args.join(" ")));
            false
        }
        Ok(Err(e)) => {
            crate::common::log::warn(&format!("sbx {}: {e}", args.join(" ")));
            false
        }
        Err(_) => {
            crate::common::log::warn(&format!("sbx {}: no end in 60 s", args.join(" ")));
            false
        }
    }
}

/// The kit daemon's settings ([`daemon::SETTINGS`]): set when they differ,
/// then the daemon restarted for them when no sandbox runs (a running one
/// would stop; the next status tries again). Failures only go to the log:
/// the VMs work without them.
fn daemon_settings(kit: &Kit, sbx: &Sbx, host: &Host, env: &KitEnv) {
    // A restart still owed (a setting changed while a sandbox ran) stays
    // owed across checks and starts of the app: `settings get` already
    // answers the new value, the daemon running does not have it yet.
    let pending = kit::root(kit).join(".restart-pending");
    for (key, value) in daemon::SETTINGS {
        let now = run(sbx, host, env, &["settings", "get", key], ASK_LIMIT)
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned());
        if now.as_deref() == Ok(value) {
            continue;
        }
        match run(sbx, host, env, &["settings", "set", key, value], ASK_LIMIT) {
            Ok(out) if out.status.success() => {
                let _ = std::fs::create_dir_all(kit::root(kit));
                if let Err(e) = std::fs::write(&pending, key) {
                    crate::common::log::warn(&format!("{}: {e}", pending.display()));
                }
            }
            Ok(out) => crate::common::log::warn(&format!(
                "sbx settings set {key}: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )),
            Err(e) => crate::common::log::warn(&format!("sbx settings set {key}: {e}")),
        }
    }
    if !pending.exists() {
        return;
    }
    let running = run(sbx, host, env, &["ls", "--json"], ASK_LIMIT)
        .map(|o| sandboxes_running(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or(true);
    if running {
        crate::common::log::info("sbx settings changed: restart when no sandbox runs");
        return;
    }
    if daemon_call(sbx, host, env, &["daemon", "restart"]) {
        let _ = std::fs::remove_file(&pending);
    }
}

/// True while the daemon still has to restart for a setting of
/// [`daemon::SETTINGS`] (a sandbox ran when it changed): no sandbox starts
/// until then, so none runs without it.
pub fn restart_pending(kit: &Kit) -> bool {
    kit::root(kit).join(".restart-pending").exists()
}

/// The PC's name, to note per PC what Atollo registered in `sbx`:
/// `HOSTNAME` is seldom exported on macOS and Linux, so the system's own
/// name next; "pc" only when nothing answers (then every such PC shares
/// the notes).
fn computer(host: &Host) -> String {
    host.get("COMPUTERNAME")
        .or(host.get("HOSTNAME"))
        .map(str::to_owned)
        .or_else(|| Some(crate::platform::host::host_name()).filter(|n| !n.is_empty()))
        .unwrap_or_else(|| LEGACY_PC.to_owned())
}

/// The name [`computer`] gave on macOS and Linux before it read the
/// system's own.
const LEGACY_PC: &str = "pc";

/// What Atollo noted for `service` on `pc` for `sandbox`, a note under
/// [`LEGACY_PC`] moved to `pc` first: left there it would never be found,
/// and its registration never taken back. Moved only when `listed_here`
/// (this PC's `sbx` has that secret): a kit carried between PCs without
/// a name shared that key, and another PC's note stays its own.
fn mine_noted(
    kit: &Kit,
    service: &str,
    pc: &str,
    sandbox: &str,
    listed_here: impl FnOnce() -> bool,
) -> Option<secrets::Noted> {
    if let Some(n) = secrets::mine(kit, service, pc, sandbox) {
        return Some(n);
    }
    if pc == LEGACY_PC {
        return None;
    }
    let old = secrets::mine(kit, service, LEGACY_PC, sandbox)?;
    if !listed_here() {
        return None;
    }
    let moved = secrets::note(kit, service, pc, sandbox, Some(&old.command))
        .and_then(|()| secrets::note(kit, service, LEGACY_PC, sandbox, None));
    if let Err(e) = moved {
        crate::common::log::warn(&format!("secrets.json: {e:#}"));
    }
    Some(old)
}

/// The kit's GitHub access for the sandbox `sandbox` (D-172): given when
/// `wanted` (the project allowed it), taken back when not
/// ([`secret_for_sandbox`]).
pub(crate) fn github_for_sandbox(
    kit: &Kit,
    sbx: &Sbx,
    host: &Host,
    env: &KitEnv,
    sandbox: &str,
    wanted: bool,
) {
    secret_for_sandbox(
        kit,
        sbx,
        host,
        env,
        sandbox,
        secrets::Secret::Github,
        wanted,
    );
}

/// True when the kit holds the value of `secret` now: signed in to GitHub
/// with the kit's `gh`, a Gemini key in the settings.
fn secret_held(kit: &Kit, env: &KitEnv, secret: secrets::Secret) -> bool {
    match secret {
        secrets::Secret::Github => crate::github::token(kit, env).is_some(),
        secrets::Secret::Gemini => crate::agents::gemini_key(kit).is_some(),
    }
}

/// The kit's `secret` for the sandbox `sandbox` ([`secrets`], D-172), that
/// one only (`--sandbox`), never global: given when `wanted` and the kit
/// holds its value ([`secret_held`]); taken back when not wanted any more
/// and Atollo had given it. A secret Atollo did not register (the user's
/// own) is never touched. Failures only go to the log: the VM works
/// without it.
pub(crate) fn secret_for_sandbox(
    kit: &Kit,
    sbx: &Sbx,
    host: &Host,
    env: &KitEnv,
    sandbox: &str,
    secret: secrets::Secret,
    wanted: bool,
) {
    let Some(exe) = crate::kit::app_exe() else {
        return;
    };
    let service = secret.service();
    let prefix = secrets::prefix(&exe, kit.target().is_windows(), secret);
    let pc = computer(host);
    let listed_here = || {
        run(sbx, host, env, &["secret", "ls"], ASK_LIMIT).is_ok_and(|o| {
            o.status.success()
                && secrets::listed(&String::from_utf8_lossy(&o.stdout), service, sandbox)
        })
    };
    let mine = mine_noted(kit, service, &pc, sandbox, listed_here);
    let warn = |what: &str, out: Result<std::process::Output, String>| match out {
        Ok(o) if o.status.success() => true,
        Ok(o) => {
            crate::common::log::warn(&format!(
                "sbx secret {what} {service} {sandbox}: {}",
                String::from_utf8_lossy(&o.stderr).trim()
            ));
            false
        }
        Err(e) => {
            crate::common::log::warn(&format!("sbx secret {what} {service} {sandbox}: {e}"));
            false
        }
    };
    let remove = || {
        let args = secrets::rm_args(secret, sandbox);
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        warn("rm", run(sbx, host, env, &args, ASK_LIMIT))
    };
    if !wanted {
        // A note `sbx` no longer lists (removed by hand, the sandbox gone)
        // goes too: its nonce must not open `secret <word>` any more.
        let gone = || {
            run(sbx, host, env, &["secret", "ls"], ASK_LIMIT).is_ok_and(|o| {
                o.status.success()
                    && !secrets::listed(&String::from_utf8_lossy(&o.stdout), service, sandbox)
            })
        };
        if mine.is_some() && (gone() || remove()) {
            if let Err(e) = secrets::note(kit, service, &pc, sandbox, None) {
                crate::common::log::warn(&format!("secrets.json: {e:#}"));
            }
        }
        return;
    }
    let Ok(ls) = run(sbx, host, env, &["secret", "ls"], ASK_LIMIT) else {
        return;
    };
    let listed = secrets::listed(&String::from_utf8_lossy(&ls.stdout), service, sandbox);
    let step = secrets::step(listed, mine.as_ref(), &prefix);
    if step == secrets::Step::Keep || !secret_held(kit, env, secret) {
        return;
    }
    if step == secrets::Step::Replace && !remove() {
        return;
    }
    // A nonce of its own: `secret <word>` answers only a command that
    // carries one Atollo noted.
    let Ok(nonce) = secrets::new_nonce() else {
        return;
    };
    let command = secrets::command(&exe, kit.target().is_windows(), secret, &nonce);
    // Noted first: `sbx` runs the command already to verify it.
    if let Err(e) = secrets::note(kit, service, &pc, sandbox, Some(&command)) {
        crate::common::log::warn(&format!("secrets.json: {e:#}"));
        return;
    }
    let args = secrets::set_args(secret, sandbox, &command);
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    if !warn("set", run(sbx, host, env, &args, ASK_LIMIT)) {
        // Not registered: its nonce opens nothing.
        if let Err(e) = secrets::note(kit, service, &pc, sandbox, None) {
            crate::common::log::warn(&format!("secrets.json: {e:#}"));
        }
    }
}

/// A global GitHub secret a first version of Atollo may have registered on
/// this PC (a note without a sandbox) is taken back: GitHub goes only to
/// the projects that chose it (D-172).
fn legacy_global_github(kit: &Kit, sbx: &Sbx, host: &Host, env: &KitEnv) {
    let pc = computer(host);
    let listed_here = || {
        run(sbx, host, env, &["secret", "ls"], ASK_LIMIT).is_ok_and(|o| {
            o.status.success()
                && secrets::listed(&String::from_utf8_lossy(&o.stdout), "github", "(global)")
        })
    };
    if mine_noted(kit, "github", &pc, "", listed_here).is_none() {
        return;
    }
    let listed = run(sbx, host, env, &["secret", "ls"], ASK_LIMIT)
        .map(|o| secrets::listed(&String::from_utf8_lossy(&o.stdout), "github", "(global)"))
        .unwrap_or(false);
    if listed {
        let args = ["secret", "rm", "github", "--force"];
        match run(sbx, host, env, &args, ASK_LIMIT) {
            Ok(o) if o.status.success() => {}
            _ => {
                crate::common::log::warn(
                    "sbx secret rm github (global): failed, tried again later",
                );
                return;
            }
        }
    }
    if let Err(e) = secrets::note(kit, "github", &pc, "", None) {
        crate::common::log::warn(&format!("secrets.json: {e:#}"));
    }
}

/// The sandboxes of the kit's terminals `sbx ls --json` lists
/// (`atollo-t-…`): made by an earlier Atollo, when the kit's terminals ran
/// in a VM (D-168, ended by D-178).
pub(crate) fn terminal_vms_in(json: &str) -> Vec<String> {
    let list: Vec<serde_json::Value> = serde_json::from_str(json).unwrap_or_default();
    list.iter()
        .filter_map(|s| s["name"].as_str())
        .filter(|n| n.starts_with(vm::TERMINAL_PREFIX) && is_atollos(n))
        .map(str::to_owned)
        .collect()
}

/// The kit's terminals run on the PC (D-178): the sandboxes an earlier
/// Atollo made for them, those `sbx` lists and those with a kit of a
/// terminal, are removed with their disks, once per run of the app, off
/// the status's thread; a failure only goes to the log (best effort, like
/// [`remove_owned`]).
fn legacy_terminals(kit: &Kit, sbx: &Sbx, host: &Host, env: &KitEnv) {
    static DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if DONE.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    let mut names: std::collections::BTreeSet<String> = kit::all(kit)
        .into_iter()
        .filter(|(_, owner)| matches!(owner, kit::Owner::Terminal { .. }))
        .map(|(name, _)| name)
        .collect();
    if let Ok(out) = run(sbx, host, env, &["ls", "--json"], ASK_LIMIT) {
        if out.status.success() {
            names.extend(terminal_vms_in(&String::from_utf8_lossy(&out.stdout)));
        }
    }
    if names.is_empty() {
        return;
    }
    crate::common::log::info(&format!(
        "kit terminals run on the PC: removing their sandboxes {names:?}"
    ));
    let names: Vec<String> = names.into_iter().collect();
    let (kit, host, env) = (kit.clone(), host.clone(), env.clone());
    std::thread::spawn(move || remove_owned(&kit, &host, &env, &names));
}

/// True when `sbx ls --json` lists a running sandbox.
pub(crate) fn sandboxes_running(json: &str) -> bool {
    serde_json::from_str::<Vec<serde_json::Value>>(json)
        .map(|list| list.iter().any(|s| s["status"] == "running"))
        .unwrap_or(true)
}

/// The `sbx` the projects' microVMs run with: the kit's own (the
/// `sandboxes` component), never one of the PC (D-175: nothing of a
/// project's box leaves the kit). It runs with the kit's environment, so
/// its sandboxes, settings and state are in the kit and travel with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sbx(PathBuf);

impl Sbx {
    /// The kit's, when installed.
    pub fn find(kit: &Kit, _host: &Host) -> Option<Sbx> {
        kit_sbx(kit).map(Sbx)
    }

    /// The kit's `sbx` at `path` (the tests).
    #[cfg(test)]
    pub(crate) fn at(path: PathBuf) -> Sbx {
        Sbx(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    /// The microVM's shim beside the kit's `sbx` (`libexec`), which
    /// AppArmor must let create user namespaces (Linux).
    fn shim(&self) -> Option<PathBuf> {
        Some(
            self.0
                .parent()?
                .parent()?
                .join("libexec")
                .join("containerd-shim-nerdbox-v1"),
        )
    }

    /// A command of this `sbx`, with its environment ([`Sbx::session_env`]).
    pub fn command(&self, env: &KitEnv, host: &Host) -> std::process::Command {
        crate::platform::jobs::kit_command(self.path(), &self.session_env(env, host))
    }

    /// The environment every run of this `sbx` gets, commands and sessions
    /// of the app's terminal alike, so its sign-in is the one the status
    /// asks: the PC's for the PC's; the kit's for the kit's, but with the
    /// PC's temporary folder. The daemon puts there the junction that
    /// shortens its sockets' paths (D-167), and the kit's (`data\tmp`, deep
    /// in a kit folder) would leave them over the limit of 107 bytes.
    pub fn session_env(&self, env: &KitEnv, host: &Host) -> KitEnv {
        sbx_session_env(env, host)
    }
}

/// The environment of a run of the kit's `sbx` ([`Sbx::session_env`]),
/// which needs no `sbx` to be worked out: `env` with the PC's temporary
/// folder and the daemon's variables (`levels::env::environment`).
pub fn sbx_session_env(env: &KitEnv, host: &Host) -> KitEnv {
    let mut out = env.clone();
    if let Some(temp) = host.temp() {
        for name in ["TEMP", "TMP", "TMPDIR"] {
            out.set(name, temp);
        }
    }
    for (name, value) in daemon::ENV {
        out.set(name, value);
    }
    out
}

/// What the notice's «Lancia» runs in the app's terminal: one of these,
/// never a text of the interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Fix {
    /// `sbx login`: the browser opens for the Docker account.
    Login,
    /// `sbx ls`: shows the error `sbx` answers with.
    Ls,
}

impl Fix {
    pub fn args(self) -> Vec<String> {
        let args: &[&str] = match self {
            Fix::Login => &["login"],
            Fix::Ls => &["ls"],
        };
        args.iter().map(|a| (*a).to_owned()).collect()
    }
}

/// Runs `fix` with the `sbx` found in a new session of the app's terminal,
/// in `dir`; `None` without one.
pub fn fix_launch(
    kit: &Kit,
    host: &Host,
    env: &KitEnv,
    fix: Fix,
    dir: &Path,
) -> Option<(crate::editor::pty::Launch, KitEnv)> {
    let sbx = Sbx::find(kit, host)?;
    let launch = crate::editor::pty::program(kit, sbx.path(), fix.args(), dir, "sbx");
    let mut env = sbx.session_env(env, host);
    if fix == Fix::Login {
        // The sign-in page is not opened by `sbx`, whose environment is the
        // kit's (its browser would make its profile in the kit): its link
        // is in the terminal, and a click opens it with the PC's (D-169).
        env.set("SBX_NO_BROWSER_OPEN", "1");
    }
    Some((launch, env))
}

/// The component of the kit's `sbx`: required, but the editor and the
/// consoles never wait for it (the notice says it is missing).
pub const COMPONENT: &str = "sandboxes";

fn install_failure_file(kit: &Kit) -> PathBuf {
    kit::root(kit).join("install-failed.txt")
}

/// Notes the outcome of an installation of [`COMPONENT`] (`setup::runner`):
/// a failure's message, for the notice to say why the VM is still missing;
/// taken away once it is installed.
pub fn note_install(kit: &Kit, outcome: &crate::setup::runner::Outcome) {
    use crate::setup::runner::Outcome;
    let file = install_failure_file(kit);
    let done = match outcome {
        Outcome::Failed { message, .. } => {
            std::fs::create_dir_all(kit::root(kit)).and_then(|()| std::fs::write(&file, message))
        }
        Outcome::Installed { .. } | Outcome::UpToDate { .. } => match std::fs::remove_file(&file) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        },
        _ => Ok(()),
    };
    if let Err(e) = done {
        crate::common::log::warn(&format!("{}: {e}", file.display()));
    }
}

/// Why the last installation of [`COMPONENT`] failed, if it did.
pub fn install_failure(kit: &Kit) -> Option<String> {
    std::fs::read_to_string(install_failure_file(kit))
        .ok()
        .map(|t| t.trim().to_owned())
        .filter(|t| !t.is_empty())
}

/// The kit's `sbx` (`data/apps/<target>/sandboxes/bin/sbx`), when the
/// component is installed.
pub fn kit_sbx(kit: &Kit) -> Option<PathBuf> {
    let c = crate::setup::Catalog::get().find(COMPONENT)?;
    c.is_installed(kit)
        .then(|| c.key_paths(kit).into_iter().next())
        .flatten()
}

/// Runs `sbx` with `args`, its output once it ends; after `limit` it is
/// ended, never left half way (an `sbx run -d` still making a sandbox).
fn run(
    sbx: &Sbx,
    host: &Host,
    env: &KitEnv,
    args: &[&str],
    limit: Duration,
) -> Result<std::process::Output, String> {
    use std::io::Read;
    let mut cmd = sbx.command(env, host);
    cmd.args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = crate::platform::jobs::spawn(&mut cmd).map_err(|e| e.to_string())?;
    // Both pipes read while it runs: a full pipe would stop it.
    let read = |pipe: Option<Box<dyn Read + Send>>| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(mut p) = pipe {
                let _ = p.read_to_end(&mut buf);
            }
            buf
        })
    };
    let stdout = read(
        child
            .stdout
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
    );
    let stderr = read(
        child
            .stderr
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
    );
    let until = Instant::now() + limit;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < until => std::thread::sleep(Duration::from_millis(25)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("sbx: no answer in {} s", limit.as_secs()));
            }
            Err(e) => return Err(e.to_string()),
        }
    };
    Ok(std::process::Output {
        status,
        stdout: stdout.join().unwrap_or_default(),
        stderr: stderr.join().unwrap_or_default(),
    })
}

/// `sbx ls --json`: answers only when the daemon runs and the user is
/// signed in.
fn ask(sbx: &Sbx, host: &Host, env: &KitEnv) -> Result<(), Blocker> {
    match run(sbx, host, env, &["ls", "--json"], ASK_LIMIT) {
        Ok(out) => answer(out.status.success(), &String::from_utf8_lossy(&out.stderr)),
        Err(message) => Err(Blocker::SbxFailed { message }),
    }
}

/// The global network policy no VM starts without: the kit's `sbx` gets
/// `balanced` (already set is fine).
fn policy(sbx: &Sbx, host: &Host, env: &KitEnv) -> Result<(), Blocker> {
    match run(sbx, host, env, &vm::POLICY_ARGS, ASK_LIMIT) {
        Ok(out) => policy_answer(
            out.status.success(),
            &format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            ),
        ),
        Err(message) => Err(Blocker::SbxFailed { message }),
    }
}

/// What `sbx policy init` says: done, or set already, is fine.
pub(crate) fn policy_answer(success: bool, text: &str) -> Result<(), Blocker> {
    if success || text.to_lowercase().contains("already initialized") {
        return Ok(());
    }
    answer(false, text)
}

/// How long the start of a sandbox may take: the first one pulls its
/// image.
const START_LIMIT: Duration = Duration::from_secs(600);

/// A project's microVM, when the level is the VM (`project_vm`, built by
/// `levels::prepare`).
#[derive(Debug, Clone)]
pub struct Vm {
    sbx: Sbx,
    host: Host,
    /// The sandbox's name (`vm::name`).
    pub name: String,
    /// The project's folder, its workspace.
    pub dir: PathBuf,
    /// Folders of the PC mounted where the VM cannot change them (the
    /// kit's tools for Linux, for the kit's terminals).
    pub read_only: Vec<PathBuf>,
    /// The kit the sandbox's own kit is written in (`data\app\sandboxes`).
    kit: Kit,
    /// Whose sandbox it is, written in its kit.
    owner: kit::Owner,
    /// The kit's tools a session gets ([`Vm::with_tools`]).
    tools: GuestTools,
    /// The PC's ports it may reach ([`Vm::with_host_ports`]).
    host_ports: Vec<u16>,
    /// What `sbx` runs in it: `shell`, or an agent of `sbx`
    /// ([`Vm::for_agent`]).
    agent: String,
    /// The kit's Gemini key for this sandbox through the proxy of `sbx`
    /// ([`Vm::with_gemini`]): Antigravity runs in it.
    gemini: bool,
}

/// A sandbox of Atollo, for the list of the VMs (D-169).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VmInfo {
    pub name: String,
    /// Whose it is, from its kit; `None` for an agent's sandbox (`sbx`'s
    /// own kit) or one made before the kits.
    pub owner: Option<kit::Owner>,
    /// `running`, `stopped`… as `sbx` says it.
    pub status: String,
    pub last_used: Option<String>,
}

/// The sandboxes of Atollo and the room their state takes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VmList {
    pub vms: Vec<VmInfo>,
    /// Bytes the kit daemon's state takes on the disk (images, the VMs'
    /// disks, sparse): `None` before the kit's daemon ever ran.
    pub disk: Option<u64>,
}

/// Atollo's sandboxes in `sbx ls --json` (`atollo-…`), with their owner
/// from their kits; never a kit terminal's (`atollo-t-…`, D-178: removed
/// by themselves, [`legacy_terminals`]).
pub(crate) fn vms_of(kit: &Kit, json: &str) -> Vec<VmInfo> {
    let list: Vec<serde_json::Value> = serde_json::from_str(json).unwrap_or_default();
    list.iter()
        .filter_map(|s| {
            let name = s["name"].as_str()?;
            (name.starts_with(vm::PREFIX) && !name.starts_with(vm::TERMINAL_PREFIX)).then(|| {
                VmInfo {
                    name: name.to_owned(),
                    owner: kit::owner(kit, name),
                    status: s["status"].as_str().unwrap_or_default().to_owned(),
                    last_used: s["last_used_at"].as_str().map(str::to_owned),
                }
            })
        })
        .collect()
}

/// The sandboxes of Atollo `sbx` has, and the room the kit's take.
pub fn vm_list(kit: &Kit, host: &Host, env: &KitEnv) -> anyhow::Result<VmList> {
    let Some(sbx) = Sbx::find(kit, host) else {
        return Ok(VmList {
            vms: Vec::new(),
            disk: None,
        });
    };
    let out = run(&sbx, host, env, &["ls", "--json"], ASK_LIMIT).map_err(anyhow::Error::msg)?;
    anyhow::ensure!(
        out.status.success(),
        "sbx ls: {}",
        String::from_utf8_lossy(&out.stderr).trim()
    );
    let disk = daemon::state_dir(kit).map(|d| crate::platform::files::allocated_size(&d));
    Ok(VmList {
        vms: vms_of(kit, &String::from_utf8_lossy(&out.stdout)),
        disk,
    })
}

/// The workspace of sandbox `name` in `sbx ls --json`: the first of its
/// `workspaces`.
pub(crate) fn workspace_of(json: &str, name: &str) -> Option<String> {
    let list: Vec<serde_json::Value> = serde_json::from_str(json).ok()?;
    list.iter()
        .find(|s| s["name"] == name)?
        .get("workspaces")?
        .get(0)?
        .as_str()
        .map(str::to_owned)
}

/// A name of one of Atollo's sandboxes: `atollo-` and `[a-z0-9.-]`.
pub fn is_atollos(name: &str) -> bool {
    name.strip_prefix(vm::PREFIX).is_some_and(|rest| {
        !rest.is_empty()
            && rest
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-')
    })
}

/// Removes the sandbox `name` of Atollo (`sbx rm -f`: its disk with it),
/// then its kit.
pub fn remove_vm(kit: &Kit, host: &Host, env: &KitEnv, name: &str) -> anyhow::Result<()> {
    anyhow::ensure!(is_atollos(name), "not a sandbox of Atollo: {name}");
    if let Some(sbx) = Sbx::find(kit, host) {
        // GitHub and the Gemini key taken back first, if Atollo had given
        // them (D-172, D-179).
        github_for_sandbox(kit, &sbx, host, env, name, false);
        secret_for_sandbox(kit, &sbx, host, env, name, secrets::Secret::Gemini, false);
        let out =
            run(&sbx, host, env, &["rm", "-f", name], START_LIMIT).map_err(anyhow::Error::msg)?;
        let text = String::from_utf8_lossy(&out.stderr).to_lowercase();
        anyhow::ensure!(
            out.status.success() || text.contains("not found") || text.contains("no such"),
            "sbx rm: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    kit::remove(kit, name)
}

/// The sandboxes of `project`: its own and its agents'.
pub fn project_sandboxes(project: &str) -> Vec<String> {
    let mut names = vec![vm::name(project)];
    names.extend(["claude", "codex"].map(|a| vm::agent_name(project, a)));
    names
}

/// The sandboxes of the kit terminal `terminal`, one per folder, from
/// their kits.
pub fn terminal_sandboxes(kit: &Kit, terminal: &str) -> Vec<String> {
    kit::all(kit)
        .into_iter()
        .filter(
            |(_, owner)| matches!(owner, kit::Owner::Terminal { terminal: t, .. } if t == terminal),
        )
        .map(|(name, _)| name)
        .collect()
}

/// Removes `names` with their owner gone (a project deleted, a terminal
/// removed): a failure only goes to the log, the list of the VMs still
/// shows what is left.
pub fn remove_owned(kit: &Kit, host: &Host, env: &KitEnv, names: &[String]) {
    for name in names {
        if let Err(e) = remove_vm(kit, host, env, name) {
            crate::common::log::warn(&format!("sandbox {name}: {e:#}"));
        }
    }
}

/// The microVM of `project` (folder `dir`, its workspace) with the
/// `read_only` folders (the kit's tools for the VM's Linux, D-170); `None`
/// without the kit's `sbx`. Built only by `levels::prepare`, from a
/// `Placement::Vm` that `levels::decide` gave (LEVELS.md §3.6): whether
/// the VM is on is the decision's business, not asked again here.
pub(crate) fn project_vm(
    kit: &Kit,
    host: &Host,
    project: &str,
    dir: &Path,
    read_only: Vec<PathBuf>,
) -> Option<Vm> {
    Some(Vm {
        sbx: Sbx::find(kit, host)?,
        host: host.clone(),
        name: vm::name(project),
        dir: dir.to_path_buf(),
        read_only,
        kit: kit.clone(),
        owner: kit::Owner::Project {
            project: project.to_owned(),
        },
        tools: GuestTools::default(),
        host_ports: Vec::new(),
        agent: "shell".into(),
        gemini: false,
    })
}

/// True when the VM is on (the level of the last status).
pub fn vm_on(kit: &Kit, host: &Host, env: &KitEnv) -> bool {
    status(kit, host, env, false).level == Level::Vm
}

/// What a session or an action in a VM gets: the kit's tools' `PATH`
/// entries (first, before the VM's own) and variables (`JAVA_HOME`…), as
/// the VM sees them, and a file of variables on the PC with what must not
/// be on a command line (the project's database login, D-171).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GuestTools {
    pub path: Vec<String>,
    pub vars: Vec<(String, String)>,
    pub env_file: Option<PathBuf>,
}

/// `db` (a project's database on the kit's server, at `port`) as a
/// program in the VM reaches it: through [`vm::HOST`].
pub fn db_in_vm(db: &crate::db::auth::ProjectDb, port: u16) -> crate::db::auth::ProjectDb {
    let engine = crate::db::DbEngine::from_id(&db.engine).unwrap_or(crate::db::DbEngine::MySql);
    let url = engine
        .project_url(&db.database, &db.password, port)
        .replacen("@127.0.0.1:", &format!("@{}:", vm::HOST), 1);
    crate::db::auth::ProjectDb {
        host: vm::HOST.to_owned(),
        port,
        url,
        ..db.clone()
    }
}

/// The program of the agent `id` (Pi, Antigravity: their own builds) for
/// the VM's Linux (`guest`, the kit scoped to its target), as the VM sees
/// it, when that build is installed (D-178, D-179).
pub fn agent_in_vm(kit: &Kit, guest: &Kit, id: &str) -> Option<String> {
    let agent = crate::setup::Catalog::get().find(id)?;
    if !agent.is_installed(guest) {
        return None;
    }
    let exe = crate::setup::kit_path(guest, crate::agents::exe_of(id)?);
    vm::guest_path(kit.target().is_windows(), &exe)
}

/// What Pi in a VM gets to reach the kit's local models (D-178): the
/// server stays on the PC's `127.0.0.1:<port>` and the VM reaches it
/// through [`vm::HOST`], like a database (D-171); its key, which goes in a
/// file of variables, never on a command line. Pi's `/llama` reads them.
pub fn pi_local_models_env(port: u16, key: &str) -> Vec<(String, String)> {
    vec![
        (
            "LLAMA_BASE_URL".to_owned(),
            crate::llm::service::url_of(port).replacen("127.0.0.1", vm::HOST, 1),
        ),
        ("LLAMA_API_KEY".to_owned(), key.to_owned()),
    ]
}

/// `ports` without `port`, the others in their order.
pub fn ports_without(ports: &[u16], port: u16) -> Vec<u16> {
    ports.iter().copied().filter(|p| *p != port).collect()
}

/// Whether the port of the local models (`port`) is closed once Pi ended:
/// only while it is open for the sandbox (`noted`, [`kit::host_ports`]: a
/// start without Pi closed it already) and never when one of its
/// databases needs it too (`db_ports`).
pub fn pi_port_closes(port: u16, noted: &[u16], db_ports: &[u16]) -> bool {
    noted.contains(&port) && !db_ports.contains(&port)
}

/// The ports of the local models to close once Pi ended in a sandbox:
/// every one opened for Pi there while watched (`opened`: the settings may
/// have moved the server meanwhile) that [`pi_port_closes`], once each.
pub fn pi_ports_to_close(opened: &[u16], noted: &[u16], db_ports: &[u16]) -> Vec<u16> {
    let mut out: Vec<u16> = Vec::new();
    for port in opened {
        if pi_port_closes(*port, noted, db_ports) && !out.contains(port) {
            out.push(*port);
        }
    }
    out
}

/// How long after Pi was made ready in a sandbox its port stays open with
/// no session of Pi seen: Pi is being started.
pub const PI_GRACE: Duration = Duration::from_secs(60);
/// How often the watcher looks at the PC's sessions of Pi.
const PI_POLL: Duration = Duration::from_secs(20);
/// The most the port stays open after the last start of Pi from the PC
/// (the app, or the editor), whatever its sessions look like: a new start
/// renews it.
pub const PI_MAX: Duration = Duration::from_secs(12 * 60 * 60);

/// A sandbox whose port of the local models a watcher closes once Pi ends
/// ([`watch_pi_port`]).
struct PiWatch {
    /// The last start of Pi from the PC: the app's, or a new session of the
    /// editor's ([`pi_port_touch`]).
    touched: Instant,
    /// The port Pi gets now, kept by every other start ([`pi_port_of`]).
    port: u16,
    /// Every port of the local models opened for Pi in the sandbox while
    /// watched: those still open close at the end ([`pi_ports_to_close`]).
    opened: Vec<u16>,
    /// The ports of the project's databases: never closed.
    db_ports: Vec<u16>,
    /// The app's terminal sessions of Pi that still run ([`PiSession`]).
    sessions: BTreeSet<u64>,
    /// The PowerShell pids of the editor's sessions of Pi seen running: a
    /// new one is a new start.
    seen: Vec<u32>,
}

/// The sandboxes whose port of the local models a watcher closes once Pi
/// ends.
fn pi_watched() -> std::sync::MutexGuard<'static, BTreeMap<String, PiWatch>> {
    static WATCHED: Mutex<BTreeMap<String, PiWatch>> = Mutex::new(BTreeMap::new());
    WATCHED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Pi is being made ready from the PC in the sandbox `name` on the local
/// models' `port` (D-178): a watcher of the sandbox waits [`PI_GRACE`]
/// again, its [`PI_MAX`] starts again, and `port` is the one every start
/// keeps open from now on (another one of before closes at that start, or
/// at the end).
pub fn pi_port_touch(name: &str, port: u16) {
    if let Some(w) = pi_watched().get_mut(name) {
        w.touched = Instant::now();
        w.port = port;
        if !w.opened.contains(&port) {
            w.opened.push(port);
        }
    }
}

/// The port of the local models a watcher keeps open for the sandbox
/// `name` while Pi runs there ([`watch_pi_port`]), if any.
pub fn pi_port_of(name: &str) -> Option<u16> {
    pi_watched().get(name).map(|w| w.port)
}

/// A session of the app's terminal running Pi in a sandbox (D-178): its
/// watcher keeps the port open while it is held. Dropped when the session
/// ends, or when it never opened.
pub struct PiSession {
    name: String,
    id: u64,
}

/// A [`PiSession`] in the sandbox `name`, counted only while a watcher is
/// there (no watcher: no port to keep).
pub fn pi_session(name: &str) -> PiSession {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if let Some(w) = pi_watched().get_mut(name) {
        w.sessions.insert(id);
    }
    PiSession {
        name: name.to_owned(),
        id,
    }
}

impl Drop for PiSession {
    fn drop(&mut self) {
        if let Some(w) = pi_watched().get_mut(&self.name) {
            w.sessions.remove(&self.id);
        }
    }
}

/// How the processes of the PC name the kit's PowerShell (`pwsh`): its
/// path as given and as the file system resolves it (a link; Windows'
/// `\\?\` dropped), each a [`path_key`].
pub fn pwsh_keys(pwsh: &Path) -> Vec<String> {
    let mut keys = vec![path_key(&pwsh.display().to_string())];
    if let Ok(real) = std::fs::canonicalize(pwsh) {
        let real = real.display().to_string();
        let key = path_key(real.strip_prefix(r"\\?\").unwrap_or(&real));
        if !keys.contains(&key) {
            keys.push(key);
        }
    }
    keys
}

/// How the processes of the PC name the programs a request's client runs
/// in ([`pwsh_keys`] of each): the kit's PowerShell (`launch.ps1`), and
/// the shells `launch.sh` may run in: Git's `sh` and `bash` of the kit on
/// Windows, the system's `/bin/sh` (and what it resolves to) and the
/// shells macOS's `/bin/sh` hands over to elsewhere (LEVELS.md §4.3).
pub fn client_keys(kit: &Kit) -> Vec<String> {
    let mut programs = vec![crate::setup::kit_path(kit, r"pwsh\pwsh.exe")];
    if kit.target().is_windows() {
        for rel in [
            r"git\usr\bin\sh.exe",
            r"git\usr\bin\bash.exe",
            r"git\bin\bash.exe",
            r"git\bin\sh.exe",
        ] {
            programs.push(crate::setup::kit_path(kit, rel));
        }
    } else {
        for shell in ["/bin/sh", "/bin/bash", "/bin/dash", "/bin/zsh"] {
            programs.push(PathBuf::from(shell));
        }
    }
    let mut keys = Vec::new();
    for program in programs {
        for key in pwsh_keys(&program) {
            if !keys.contains(&key) {
                keys.push(key);
            }
        }
    }
    keys
}

/// The clients' sessions of Pi that still run (D-178), from the markers'
/// pids (`markers`, [`crate::levels::requests::pi_markers`]): only a pid
/// that runs on the PC (`procs`) as a client's program (`pwsh`,
/// [`client_keys`]). A marker left behind, or its pid now another
/// program's, is none.
pub fn editor_pis_alive(markers: &[u32], procs: &[Proc], pwsh: &[String]) -> Vec<u32> {
    markers
        .iter()
        .copied()
        .filter(|pid| {
            procs
                .iter()
                .any(|p| p.pid == *pid && pwsh.contains(&path_key(&p.path.display().to_string())))
        })
        .collect()
}

/// Whether the port of the local models stays open (D-178), from what the
/// PC knows only, never from the VM (the project's code runs there and
/// could fake a Pi): `since` the last start of Pi from the PC within
/// [`PI_GRACE`] (Pi is being started), or a session of Pi that runs, the
/// app's (`app`) or the editor's (`editor`, [`editor_pis_alive`]); never
/// past [`PI_MAX`] since that start.
pub fn pi_port_kept(since: Duration, app: usize, editor: usize) -> bool {
    since < PI_MAX && (since < PI_GRACE || app > 0 || editor > 0)
}

/// What a watcher finds in its sandbox ([`pi_now`]).
enum PiNow {
    /// Not watched any more.
    Gone,
    /// The port stays open ([`pi_port_kept`]).
    Kept,
    /// It closes: the sandbox no longer watched when asked to `end`, with
    /// what it had.
    Over(Option<PiWatch>),
}

/// [`pi_port_kept`] for the watched sandbox `name` of `project` now; a
/// session of the editor not seen before is a new start of Pi (as
/// [`pi_port_touch`]). With `end`, a port no longer kept leaves the
/// watched ones in the same step: nothing touches it in between.
fn pi_now(kit: &Kit, name: &str, project: &str, pwsh: &[String], end: bool) -> PiNow {
    let markers = crate::levels::requests::pi_markers(kit, project);
    let editor = if markers.is_empty() {
        Vec::new()
    } else {
        editor_pis_alive(&markers, &crate::platform::processes::all(), pwsh)
    };
    let mut all = pi_watched();
    let Some(w) = all.get_mut(name) else {
        return PiNow::Gone;
    };
    if editor.iter().any(|pid| !w.seen.contains(pid)) {
        w.touched = Instant::now();
    }
    w.seen = editor;
    if pi_port_kept(w.touched.elapsed(), w.sessions.len(), w.seen.len()) {
        PiNow::Kept
    } else {
        PiNow::Over(end.then(|| all.remove(name)).flatten())
    }
}

/// The PC's ports a start of a sandbox opens: `ports` (its databases, or
/// whatever its session asked), plus the port of the local models while
/// Pi's watcher keeps it open (`pi_port`, D-178), once. Another start of
/// the same sandbox (the project's terminal, an action, the editor) never
/// closes it under a running Pi.
pub fn ports_with(ports: &[u16], pi_port: Option<u16>) -> Vec<u16> {
    let mut all = ports.to_vec();
    if let Some(port) = pi_port.filter(|p| !all.contains(p)) {
        all.push(port);
    }
    all
}

/// Closes the port of the local models (`port`) of `vm`, the VM of
/// `project`, once no Pi started from the PC runs there any more (D-178):
/// the project's own code runs in the same VM, and would use the models
/// too while the port is open. Whether Pi runs is never asked to the VM,
/// where that code could fake it: the app's terminal sessions of Pi
/// ([`pi_session`]) and the requests' clients' (`launch.ps1`'s and
/// `launch.sh`'s markers with their pid, [`editor_pis_alive`]) tell it, on
/// the PC. One
/// watcher per sandbox, a thread: it waits [`PI_GRACE`], then looks every
/// [`PI_POLL`]; when no session runs, or [`PI_MAX`] passed since the last
/// start, it closes every port it opened for Pi (never one of the
/// databases', `db_ports`), deletes `pi.env` and ends. Called again while
/// it watches, it takes the new `port` and waits [`PI_GRACE`] again.
pub fn watch_pi_port(vm: Vm, env: KitEnv, project: String, port: u16, db_ports: Vec<u16>) {
    {
        let mut all = pi_watched();
        if let Some(w) = all.get_mut(&vm.name) {
            w.touched = Instant::now();
            w.port = port;
            if !w.opened.contains(&port) {
                w.opened.push(port);
            }
            w.db_ports = db_ports;
            return;
        }
        all.insert(
            vm.name.clone(),
            PiWatch {
                touched: Instant::now(),
                port,
                opened: vec![port],
                db_ports,
                sessions: BTreeSet::new(),
                seen: Vec::new(),
            },
        );
    }
    let pwsh = client_keys(&vm.kit);
    std::thread::spawn(move || loop {
        let Some(since) = pi_watched().get(&vm.name).map(|w| w.touched.elapsed()) else {
            return;
        };
        if since < PI_GRACE {
            std::thread::sleep(PI_GRACE - since);
            continue;
        }
        match pi_now(&vm.kit, &vm.name, &project, &pwsh, false) {
            PiNow::Gone => return,
            PiNow::Kept => {
                std::thread::sleep(PI_POLL);
                continue;
            }
            PiNow::Over(_) => {}
        }
        // Under the sandbox's lock: a start of Pi waits, then opens the
        // port again. Looked at again: started meanwhile, it stays.
        let lock = sandbox_lock(&vm.name);
        let _one = lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let watch = match pi_now(&vm.kit, &vm.name, &project, &pwsh, true) {
            PiNow::Gone => return,
            PiNow::Kept | PiNow::Over(None) => continue,
            PiNow::Over(Some(watch)) => watch,
        };
        if let Err(e) = vm.close_pi_ports(&env, &watch.opened, &watch.db_ports) {
            crate::common::log::warn(&format!("{}: local models: {e:#}", vm.name));
        }
        return;
    });
}

/// The kit's tools for the VM's Linux in `project`, at the lines it chose
/// (D-170), sealed and without the agents, as on the PC
/// (`levels::env::pc_environment` on `guest`), then [`guest_view`].
pub fn project_guest_tools(kit: &Kit, guest: &Kit, project: &str, base: KitEnv) -> GuestTools {
    let env = crate::levels::env::pc_environment(
        guest,
        base,
        crate::levels::Level::Project,
        Some(project),
        None,
    );
    guest_view(kit, guest, &env)
}

/// The `PATH` entries and the variables of `env` (an environment of
/// `guest`, the kit scoped to the VM's target, [`vm::guest_target`]) that
/// lie in its `data/apps`, as the VM sees them. The rest of the kit's
/// environment (its home, temp, agents) is the PC's business: the VM has
/// its own.
pub fn guest_view(kit: &Kit, guest: &Kit, env: &KitEnv) -> GuestTools {
    let windows = kit.target().is_windows();
    let apps = guest.apps();
    let in_vm = |value: &str| -> Option<String> {
        let value = if windows {
            value.replace('/', "\\")
        } else {
            value.to_owned()
        };
        let path = PathBuf::from(value);
        path.starts_with(&apps)
            .then(|| vm::guest_path(windows, &path))
            .flatten()
    };
    let path = env.path_entries().into_iter().filter_map(in_vm).collect();
    let vars = env
        .iter()
        .filter(|(name, _)| !name.eq_ignore_ascii_case("PATH"))
        .filter_map(|(name, value)| Some((name.to_owned(), in_vm(value)?)))
        .collect();
    GuestTools {
        path,
        vars,
        env_file: None,
    }
}

impl Vm {
    /// The VM of `project` in `dir` with an `sbx` named `sbx` (the tests).
    #[cfg(test)]
    pub(crate) fn for_tests(kit: &Kit, project: &str, dir: &Path) -> Vm {
        Vm {
            sbx: Sbx(PathBuf::from("sbx")),
            host: Host::default(),
            name: vm::name(project),
            dir: dir.to_path_buf(),
            read_only: Vec::new(),
            kit: kit.clone(),
            owner: kit::Owner::Project {
                project: project.to_owned(),
            },
            tools: GuestTools::default(),
            host_ports: Vec::new(),
            agent: "shell".into(),
            gemini: false,
        }
    }

    /// A command of `sbx` with `args`, with its environment.
    pub fn command(&self, env: &KitEnv, args: &[String]) -> std::process::Command {
        let mut cmd = self.sbx.command(env, &self.host);
        cmd.args(args);
        cmd
    }

    /// Runs `sbx` with `args` to its end; its error text when it fails.
    fn call(&self, env: &KitEnv, args: &[String], limit: Duration) -> anyhow::Result<()> {
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        let out = run(&self.sbx, &self.host, env, &args, limit).map_err(anyhow::Error::msg)?;
        let stderr = String::from_utf8_lossy(&out.stderr);
        anyhow::ensure!(
            out.status.success(),
            "sbx {}: {}",
            args.iter().take(2).copied().collect::<Vec<_>>().join(" "),
            stderr.trim()
        );
        Ok(())
    }

    /// Creates the sandbox the first time, starts it again when stopped.
    /// Writes the sandbox's kit (D-169), then: the first time the sandbox
    /// is made with it (`--kit`); after, a changed kit goes in with
    /// `sbx kit add` (its disk kept); then it starts again when stopped.
    ///
    /// A read-only folder an existing sandbox lacks (one made before the
    /// kit's tools were mounted, D-170) is mounted with `sbx mount` once it
    /// runs (a stopped one is refused), its disk kept ([`Vm::mount_missing`]).
    ///
    /// `making` is called just before the sandbox is made (the first time,
    /// or again after the kit moved): `sbx` then pulls its image, without a
    /// progress of its own (LEVELS.md §8.6).
    pub fn start(&self, env: &KitEnv, lang: &str, making: &dyn Fn()) -> anyhow::Result<()> {
        self.ready(lang)?;
        let (kit_dir, print) = kit::write(&self.kit, &self.name, &self.owner)?;
        let mounts = self.mounts();
        // One start per sandbox at a time: a terminal and an action of the
        // same project would both find it missing and both make it.
        let lock = sandbox_lock(&self.name);
        let _one = lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut exists = self.exists(env)?;
        if exists && self.moved(env)? {
            self.make_again(env)?;
            exists = false;
        }
        if exists {
            if kit::applied(&self.kit, &self.name).as_deref() != Some(print.as_str()) {
                let args = vec![
                    "kit".to_owned(),
                    "add".to_owned(),
                    self.name.clone(),
                    kit_dir.display().to_string(),
                ];
                self.call(env, &args, START_LIMIT)?;
                kit::mark_applied(&self.kit, &self.name, &print)?;
            }
            self.call(env, &vm::restart_args(&self.name), START_LIMIT)?;
            // After the start: `sbx mount` refuses a stopped sandbox
            // ("409 Conflict: sandbox … is not running").
            self.mount_missing(env, lang)?;
            self.sync_host_ports(env, false)?;
            self.sync_github(env);
            self.sync_gemini(env);
            return Ok(());
        }
        making();
        self.call(
            env,
            &vm::start_args(
                &self.name,
                &self.dir,
                &self.read_only,
                Some(&kit_dir),
                &self.agent,
            ),
            START_LIMIT,
        )?;
        kit::mark_mounts(&self.kit, &self.name, &mounts)?;
        kit::mark_applied(&self.kit, &self.name, &print)?;
        self.sync_host_ports(env, true)?;
        self.sync_github(env);
        self.sync_gemini(env);
        Ok(())
    }

    /// True when the sandbox of a project works in another folder than the
    /// project's: the kit moved (another drive letter, a folder renamed).
    /// `sbx` never changes a sandbox's workspace and refuses to start one
    /// whose folder is gone (tried with sbx 0.49.2). A kit terminal's
    /// sandbox is named after its folder: never moved.
    fn moved(&self, env: &KitEnv) -> anyhow::Result<bool> {
        if !matches!(self.owner, kit::Owner::Project { .. }) {
            return Ok(false);
        }
        let noted =
            kit::mounts(&self.kit, &self.name).and_then(|m| m.lines().next().map(str::to_owned));
        let workspace = match noted {
            Some(w) => w,
            None => match self.workspace(env)? {
                Some(w) => w,
                None => return Ok(false),
            },
        };
        Ok(crate::platform::processes::path_key(&workspace)
            != crate::platform::processes::path_key(&self.dir.display().to_string()))
    }

    /// Removes the sandbox of a project whose kit moved, to make it again
    /// in the project's folder (D-175): its disk holds nothing that is not
    /// in the kit (the project's files, its database, the tools and the
    /// sign-ins are all there), only the VM's home and the images it
    /// pulled. GitHub is taken back first (D-172), the PC's ports go with
    /// the sandbox (D-171).
    fn make_again(&self, env: &KitEnv) -> anyhow::Result<()> {
        crate::common::log::info(&format!(
            "{}: the kit moved, the sandbox is made again in {}",
            self.name,
            self.dir.display()
        ));
        github_for_sandbox(&self.kit, &self.sbx, &self.host, env, &self.name, false);
        self.take_back_gemini(env);
        self.call(
            env,
            &["rm".to_owned(), "-f".to_owned(), self.name.clone()],
            START_LIMIT,
        )?;
        kit::forget(&self.kit, &self.name)
    }

    /// GitHub for this sandbox as its project chose (`github_in_vm`, off
    /// unless turned on; never for a kit terminal), D-172.
    fn sync_github(&self, env: &KitEnv) {
        let wanted = match &self.owner {
            kit::Owner::Project { project } => {
                crate::projects::settings::load(&self.kit, project).github_in_vm
            }
            kit::Owner::Terminal { .. } => false,
        };
        github_for_sandbox(&self.kit, &self.sbx, &self.host, env, &self.name, wanted);
    }

    /// The kit's Gemini key for this sandbox through the proxy of `sbx`
    /// (D-179, LEVELS.md §9), when Antigravity runs in it
    /// ([`Vm::with_gemini`]): registered while the settings hold a key,
    /// taken back once they do not. A start without Antigravity leaves it
    /// as it is: an `agy` of the same sandbox may be running.
    fn sync_gemini(&self, env: &KitEnv) {
        if !self.gemini {
            return;
        }
        let held = crate::agents::gemini_key(&self.kit).is_some();
        secret_for_sandbox(
            &self.kit,
            &self.sbx,
            &self.host,
            env,
            &self.name,
            secrets::Secret::Gemini,
            held,
        );
    }

    /// The Gemini key taken back from this sandbox, if Atollo had given it.
    fn take_back_gemini(&self, env: &KitEnv) {
        secret_for_sandbox(
            &self.kit,
            &self.sbx,
            &self.host,
            env,
            &self.name,
            secrets::Secret::Gemini,
            false,
        );
    }

    /// This sandbox gets the kit's Gemini key through the proxy of `sbx`
    /// at its start ([`Vm::start`]): Antigravity runs in it (D-179).
    pub fn with_gemini(mut self, gemini: bool) -> Vm {
        self.gemini = gemini;
        self
    }

    /// True when its start gives it the kit's Gemini key ([`Vm::with_gemini`]).
    #[cfg(test)]
    pub fn gemini(&self) -> bool {
        self.gemini
    }

    /// Mounts the read-only folders an existing sandbox lacks, after those
    /// noted at its making (`.mounts`; none noted: a sandbox made before,
    /// every one tried, one mounted already is fine), then notes them. The
    /// workspace never changes (the sandbox's name comes from it): another
    /// one noted is an error, never a sandbox made again, which would delete
    /// its disk.
    fn mount_missing(&self, env: &KitEnv, lang: &str) -> anyhow::Result<()> {
        let noted = kit::mounts(&self.kit, &self.name).unwrap_or_default();
        let lines: Vec<&str> = noted.lines().collect();
        // Nothing noted is nothing known: the workspace `sbx` has, checked,
        // never taken for this one.
        let workspace = match lines.first() {
            Some(w) => (*w).to_owned(),
            None => self.workspace(env)?.unwrap_or_default(),
        };
        let same = crate::platform::processes::path_key(&workspace)
            == crate::platform::processes::path_key(&self.dir.display().to_string());
        if !same {
            anyhow::bail!(crate::common::i18n::t(
                lang,
                "La sandbox {0} lavora in un'altra cartella: rimuovila in «Sandbox delle VM» (si cancella il suo disco, non i file del progetto) e riapri.",
                &[&self.name],
            ));
        }
        let windows = self.kit.target().is_windows();
        let mut changed = noted.is_empty();
        for folder in &self.read_only {
            let shown = folder.display().to_string();
            if lines.iter().skip(1).any(|l| *l == shown) {
                continue;
            }
            let guest = vm::guest_path(windows, folder)
                .ok_or_else(|| anyhow::anyhow!("{shown}: no path in the VM"))?;
            let args = vm::mount_args(&self.name, folder, &guest);
            let args: Vec<&str> = args.iter().map(String::as_str).collect();
            let out =
                run(&self.sbx, &self.host, env, &args, START_LIMIT).map_err(anyhow::Error::msg)?;
            let text = String::from_utf8_lossy(&out.stderr);
            anyhow::ensure!(
                out.status.success() || text.to_lowercase().contains("already"),
                "sbx mount: {}",
                text.trim()
            );
            changed = true;
        }
        if changed {
            kit::mark_mounts(&self.kit, &self.name, &self.mounts())?;
        }
        Ok(())
    }

    /// The workspace `sbx` has for this sandbox (`sbx ls --json`, the first
    /// of its `workspaces`), if any.
    fn workspace(&self, env: &KitEnv) -> anyhow::Result<Option<String>> {
        let out = run(&self.sbx, &self.host, env, &["ls", "--json"], ASK_LIMIT)
            .map_err(anyhow::Error::msg)?;
        anyhow::ensure!(
            out.status.success(),
            "sbx ls: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
        Ok(workspace_of(
            &String::from_utf8_lossy(&out.stdout),
            &self.name,
        ))
    }

    /// The folders this sandbox mounts, one per line: the workspace, then
    /// the read-only ones.
    fn mounts(&self) -> String {
        std::iter::once(&self.dir)
            .chain(&self.read_only)
            .map(|p| format!("{}\n", p.display()))
            .collect()
    }

    /// The sandbox of `agent` (one of `sbx`'s, [`vm::sbx_agent`]) for this
    /// project's VM: its own (`atollo-a-`), the same folders, kit, tools,
    /// databases and ports as the project's (D-175); `None` for an agent
    /// `sbx` has none for.
    pub fn for_agent(mut self, project: &str, agent: &str) -> Option<Vm> {
        let id = vm::sbx_agent(agent)?;
        self.name = vm::agent_name(project, id);
        self.agent = id.to_owned();
        Some(self)
    }

    /// The session of the agent in this sandbox, started ([`Vm::start`]):
    /// `sbx run --name` with the database's variables and the kit's tools
    /// first on the `PATH` before the image's own, read in the sandbox.
    pub fn agent_session_args(&self, env: &KitEnv) -> anyhow::Result<Vec<String>> {
        let args = [
            "exec".to_owned(),
            self.name.clone(),
            "sh".to_owned(),
            "-c".to_owned(),
            r#"printf %s "$PATH""#.to_owned(),
        ];
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        let out = run(&self.sbx, &self.host, env, &args, ASK_LIMIT).map_err(anyhow::Error::msg)?;
        anyhow::ensure!(
            out.status.success(),
            "sbx exec {}: {}",
            self.name,
            String::from_utf8_lossy(&out.stderr).trim()
        );
        let image_path = String::from_utf8_lossy(&out.stdout).trim().to_owned();
        Ok(vm::agent_session_args(&self.name, &image_path, &self.tools))
    }

    /// The kit's tools its sessions and actions get (D-170).
    pub fn with_tools(mut self, tools: GuestTools) -> Vm {
        self.tools = tools;
        self
    }

    pub fn tools(&self) -> &GuestTools {
        &self.tools
    }

    /// True when `sbx` has this sandbox.
    fn exists(&self, env: &KitEnv) -> anyhow::Result<bool> {
        let out = run(&self.sbx, &self.host, env, &["ls", "-q"], ASK_LIMIT)
            .map_err(anyhow::Error::msg)?;
        anyhow::ensure!(
            out.status.success(),
            "sbx ls: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
        Ok(String::from_utf8_lossy(&out.stdout)
            .lines()
            .any(|l| l.trim() == self.name))
    }

    /// Publishes `port` of the VM on the PC: on `127.0.0.1`, or on the
    /// local network when `lan` (Settings). The other address, left by a
    /// start with the other setting, is taken back first, and failing that
    /// nothing starts: with the local network turned off, the site must not
    /// stay on it (`--unpublish` of nothing published succeeds, so a
    /// failure is a real one). A port published already on the same
    /// address (an action that ended by itself) is fine: `sbx` answers
    /// `409 … port <address>:<port>/tcp4 is already published` (tried with
    /// sbx 0.49.2).
    pub fn publish(&self, env: &KitEnv, port: u16, lan: bool) -> anyhow::Result<()> {
        self.call(env, &vm::unpublish_args(&self.name, port, !lan), ASK_LIMIT)?;
        let args = vm::publish_args(&self.name, port, lan);
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        let out = run(&self.sbx, &self.host, env, &args, ASK_LIMIT).map_err(anyhow::Error::msg)?;
        let text = String::from_utf8_lossy(&out.stderr);
        anyhow::ensure!(
            out.status.success() || vm::published_already(&text, port, lan),
            "sbx ports: {}",
            text.trim()
        );
        Ok(())
    }

    /// Stops what the action noted as `tag` runs in the VM ([`vm::stop_args`]:
    /// ending its `sbx exec` on the PC does not), its web `container` if
    /// any, and takes its `port` back from the PC. Failures only go to the
    /// log: the daemon's stop at the app's exit ends the VM anyway.
    pub fn stop_run(
        &self,
        env: &KitEnv,
        tag: &str,
        container: Option<&str>,
        port: Option<u16>,
        lan: bool,
    ) {
        let mut steps = vec![vm::stop_args(&self.name, tag, container)];
        if let Some(p) = port {
            steps.push(vm::unpublish_args(&self.name, p, lan));
        }
        for args in steps {
            if let Err(e) = self.call(env, &args, ASK_LIMIT) {
                crate::common::log::warn(&format!("{}: {e:#}", self.name));
            }
        }
    }

    /// The session of the app's terminal for `args` (`vm::kit_terminal_args`,
    /// `vm::agent_args`), titled `what`; refused while [`Vm::ready`] is not
    /// (an agent's `sbx run` makes its sandbox without [`Vm::start`]).
    pub fn launch(
        &self,
        kit: &Kit,
        args: Vec<String>,
        what: &str,
        lang: &str,
    ) -> anyhow::Result<crate::editor::pty::Launch> {
        self.ready(lang)?;
        Ok(crate::editor::pty::program(
            kit,
            self.sbx.path(),
            args,
            &self.dir,
            what,
        ))
    }

    /// The one gate of every way into a sandbox (start, sessions, agents):
    /// a daemon that has not restarted with the kit's settings yet (no SSH
    /// agent of the PC in the VMs) starts and opens nothing, and never
    /// falls back to the PC.
    fn ready(&self, lang: &str) -> anyhow::Result<()> {
        if restart_pending(&self.kit) {
            anyhow::bail!(crate::common::i18n::t(
                lang,
                "Docker Sandboxes deve ripartire per un'impostazione di sicurezza: chiudi le sessioni nella VM, poi «Ricontrolla» nella finestra della VM e riprova.",
                &[],
            ));
        }
        Ok(())
    }

    /// The kit's `sbx` that runs this sandbox's sessions (a request's
    /// reply, `levels::requests`).
    pub fn sbx_path(&self) -> &Path {
        self.sbx.path()
    }

    /// The environment of that session ([`Sbx::session_env`]).
    pub fn session_env(&self, env: &KitEnv) -> KitEnv {
        self.sbx.session_env(env, &self.host)
    }

    /// The PC's ports this sandbox may reach (its databases on the kit's
    /// servers, D-171), opened by [`Vm::start`].
    pub fn with_host_ports(mut self, ports: Vec<u16>) -> Vm {
        self.host_ports = ports;
        self
    }

    /// One more port of the PC this sandbox may reach (the kit's local
    /// models for Pi, D-178), opened by [`Vm::start`] like the databases'.
    pub fn with_host_port(mut self, port: u16) -> Vm {
        if !self.host_ports.contains(&port) {
            self.host_ports.push(port);
        }
        self
    }

    /// The PC's ports it may reach, as given ([`Vm::with_host_ports`]).
    pub fn host_ports(&self) -> &[u16] {
        &self.host_ports
    }

    /// Closes the PC's `port` for this sandbox and takes it out of the
    /// noted ones, only once its rule is gone (the order of
    /// [`Vm::sync_host_ports`]: a rule never open without its note).
    pub fn close_host_port(&self, env: &KitEnv, port: u16) -> anyhow::Result<()> {
        self.call(env, &vm::host_port_close(&self.name, port), ASK_LIMIT)?;
        let noted = ports_without(&kit::host_ports(&self.kit, &self.name), port);
        kit::mark_host_ports(&self.kit, &self.name, &noted)
    }

    /// Pi ended in this sandbox (D-178): its `pi.env` (the models' key)
    /// deleted and the ports of the local models opened for it (`opened`)
    /// closed ([`pi_ports_to_close`]), each tried; the first failure told.
    fn close_pi_ports(&self, env: &KitEnv, opened: &[u16], db_ports: &[u16]) -> anyhow::Result<()> {
        let file = kit::dir(&self.kit, &self.name).join("pi.env");
        match std::fs::remove_file(&file) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                crate::common::log::warn(&format!("{}: {e}", file.display()));
            }
            _ => {}
        }
        let noted = kit::host_ports(&self.kit, &self.name);
        let mut first = None;
        for port in pi_ports_to_close(opened, &noted, db_ports) {
            if let Err(e) = self.close_host_port(env, port) {
                first.get_or_insert(e.context(format!("port {port}")));
            }
        }
        first.map_or(Ok(()), Err)
    }

    /// Opens the PC's ports of this sandbox, closes those it had and has
    /// no more (a database moved to another version's server), and notes
    /// them; `made`: the sandbox was just made, nothing open before. The
    /// port of the local models stays among them while Pi's watcher keeps
    /// it open ([`ports_with`]); the caller holds the sandbox's lock, which
    /// the watcher takes to close it.
    fn sync_host_ports(&self, env: &KitEnv, made: bool) -> anyhow::Result<()> {
        let before = if made {
            Vec::new()
        } else {
            kit::host_ports(&self.kit, &self.name)
        };
        let wanted = ports_with(&self.host_ports, pi_port_of(&self.name));
        // Noted before any rule is opened: a rule opened and then lost by
        // a failure or the app's end would never be closed.
        let mut noted = wanted.clone();
        noted.extend(before.iter().filter(|p| !wanted.contains(p)));
        kit::mark_host_ports(&self.kit, &self.name, &noted)?;
        for port in &wanted {
            self.call(env, &vm::host_port_allow(&self.name, *port), ASK_LIMIT)?;
        }
        // A port leaves the list only once its rule is gone: one that
        // could not be closed stays noted, and the next start tries again.
        let mut noted = wanted.clone();
        for port in before.iter().filter(|p| !wanted.contains(p)) {
            if let Err(e) = self.call(env, &vm::host_port_close(&self.name, *port), ASK_LIMIT) {
                crate::common::log::warn(&format!("{}: port {port}: {e:#}", self.name));
                noted.push(*port);
            }
        }
        kit::mark_host_ports(&self.kit, &self.name, &noted)
    }
}

/// The lock of the sandbox `name`, the same for every caller.
fn sandbox_lock(name: &str) -> std::sync::Arc<Mutex<()>> {
    static LOCKS: Mutex<BTreeMap<String, std::sync::Arc<Mutex<()>>>> = Mutex::new(BTreeMap::new());
    let mut all = LOCKS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    std::sync::Arc::clone(all.entry(name.to_owned()).or_default())
}

/// The PHP line of `project` in its VM: its own choice, the kit's, or
/// `8.4` (the official images have every line the kit offers).
pub fn php_line(kit: &Kit, project: &str) -> String {
    crate::versions::project_choice(kit, project)
        .get("php")
        .cloned()
        .or_else(|| crate::versions::kit_choice(kit).get("php").cloned())
        .unwrap_or_else(|| "8.4".to_owned())
}

/// Stops the daemon of the kit's `sbx` and the microVMs it runs, at the
/// app's exit and before the drive is removed: the PC's own daemon is the
/// user's, never stopped.
pub fn stop_daemon(kit: &Kit, host: &Host, env: &KitEnv) {
    let Some(exe) = kit_sbx(kit) else {
        return;
    };
    // The daemon is one per user: with another `sbx`'s running, the kit's
    // `daemon stop` would stop that one and its VMs.
    if !matches!(daemon::current(kit, &exe), daemon::Daemon::Kit) {
        return;
    }
    let sbx = Sbx(exe);
    match run(&sbx, host, env, &["daemon", "stop"], ASK_LIMIT) {
        Ok(out) if out.status.success() => crate::common::log::info("sbx: daemon stopped"),
        Ok(out) => crate::common::log::warn(&format!(
            "sbx daemon stop: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )),
        Err(e) => crate::common::log::warn(&format!("sbx daemon stop: {e}")),
    }
}

/// What `sbx`'s exit and error text say: signed out, another error (its
/// last line), or ready.
pub(crate) fn answer(success: bool, stderr: &str) -> Result<(), Blocker> {
    if success {
        return Ok(());
    }
    let text = stderr.to_lowercase();
    if [
        "sbx login",
        "not logged in",
        "log in",
        "sign in",
        "not authenticated",
        "unauthorized",
    ]
    .iter()
    .any(|w| text.contains(w))
    {
        return Err(Blocker::SbxSignedOut);
    }
    let message = stderr
        .lines()
        .map(str::trim)
        .rfind(|l| !l.is_empty())
        .unwrap_or("sbx")
        .chars()
        .take(300)
        .collect();
    Err(Blocker::SbxFailed { message })
}

#[cfg(test)]
#[path = "../../../test/rust/isolation/mod.rs"]
mod tests;
