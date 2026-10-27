//! Keeps Claude Code, Codex, Antigravity and Pi aligned: the same global
//! instructions, skills and MCP servers for all of them. The shared copy
//! lives in `data\agents`:
//!
//! ```text
//! data\agents\AGENTS.md   = data\home\.claude\CLAUDE.md   (Claude)
//!                         = data\home\.codex\AGENTS.md    (Codex)
//!                         = data\home\.gemini\AGENTS.md   (Antigravity)
//!                         = data\home\.pi\agent\AGENTS.md (Pi)
//! data\agents\skills\     <- junctions in each agent's skills folder
//! data\agents\mcp.json    = "mcpServers" in .claude\.claude.json
//!                         -> [mcp_servers] in .codex\config.toml
//!                         -> .gemini\config\mcp_config.json
//!                         -> .pi\agent\mcp.json
//! ```
//!
//! A change made on either side is carried to the others, deletions too,
//! thanks to the state of the last sync in `data\agents\.sync`; when both
//! sides changed the newer wins and the other is saved in
//! `.sync\conflicts`. Also: the kit rule for the agents, the project skills
//! (`.agents\skills` <-> `.claude\skills`) with `project`, and Pi's own
//! files (the local models, its settings: `pi`). No project is trusted
//! from Atollo (SEC-V6-02, D-178).
//!
//! Run at start, before and after every agent, and when an agent ends.
//! Never blocks the agents: each step fails alone, as a warning.

pub(crate) mod codex;
mod links;
pub(crate) mod mcp;
pub(crate) mod pi;
mod project;
mod tree;
mod trust;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{bail, Result};
use serde_json::Value;

use crate::common::fsx::{join_rel, write_atomic};
use crate::kit::Kit;

/// Folders of the kit and of the agents.
#[derive(Debug, Clone)]
pub struct Paths {
    pub root: PathBuf,

    pub agents: PathBuf,
    pub state: PathBuf,
    pub claude: PathBuf,
    pub codex: PathBuf,
    pub gemini: PathBuf,
    /// Pi's folder, `.pi\agent` (`PI_CODING_AGENT_DIR`).
    pub pi: PathBuf,
    pub projects: PathBuf,
}

impl Paths {
    pub fn of(kit: &Kit) -> Paths {
        let data = kit.data();
        let home = kit.home();
        Paths {
            root: kit.root().to_path_buf(),
            agents: data.join("agents"),
            state: join_rel(&data, "agents/.sync"),
            claude: home.join(".claude"),
            codex: home.join(".codex"),
            gemini: home.join(".gemini"),
            pi: pi::dir(&home),
            projects: kit.projects(),
        }
    }

    pub fn mcp_file(&self) -> PathBuf {
        self.agents.join("mcp.json")
    }

    pub fn claude_json(&self) -> PathBuf {
        self.claude.join(".claude.json")
    }
}

/// What a run reports: notes for `--verbose`, warnings for the user.
#[derive(Debug, Default)]
pub struct Report {
    pub notes: Vec<String>,
    pub warnings: Vec<String>,
}

impl Report {
    pub fn note(&mut self, s: impl Into<String>) {
        self.notes.push(s.into());
    }
    pub fn warn(&mut self, s: impl Into<String>) {
        self.warnings.push(s.into());
    }
}

/// Missing or empty file: `None`. Damaged: an error that says where.
/// `fsx::try_read_json` on a [`Value`], the agents' files of other
/// programs, so the callers name no type.
pub fn read_json(file: &Path) -> Result<Option<Value>> {
    crate::common::fsx::try_read_json(file)
}

/// State of the last sync: rebuilt (empty) if damaged.
pub fn read_state(file: &Path) -> Value {
    read_json(file)
        .ok()
        .flatten()
        .unwrap_or(Value::Object(Default::default()))
}

pub use crate::common::fsx::write_json;

/// JSON with sorted keys: two servers are equal when their settings are.
pub fn canonical(v: &Value) -> String {
    match v {
        Value::Array(a) => format!(
            "[{}]",
            a.iter().map(canonical).collect::<Vec<_>>().join(",")
        ),
        Value::Object(o) => {
            let mut keys: Vec<&String> = o.keys().collect();
            keys.sort();
            format!(
                "{{{}}}",
                keys.iter()
                    .map(|k| format!("{}:{}", Value::String((*k).clone()), canonical(&o[*k])))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
        other => other.to_string(),
    }
}

/// `yyyymmdd-hhmmss` (UTC), for kept copies.
pub fn stamp() -> String {
    let secs = crate::common::time::now_secs();
    let days = secs / 86_400;
    let (y, m, d) = civil_from_days(days as i64);
    let rem = secs % 86_400;
    format!(
        "{y:04}{m:02}{d:02}-{:02}{:02}{:02}",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

/// Days since 1970-01-01 to (year, month, day) (Howard Hinnant's algorithm).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// The kit rule: each agent loads its own project file; the rule makes all
/// of them read all of them (`rules.js`).
pub const KIT_RULE_TEXT: &str = "In this kit (Atollo) Claude Code, Codex, Antigravity and Pi share the same instructions and skills.
- Project instructions: CLAUDE.md, AGENTS.md and GEMINI.md are equivalent. At the start of a task, read each of these files in the workspace root that is not already in your context, and follow all of them together.
- Project skills: the shared folder is .agents/skills/<name>/SKILL.md in the project root (read by Codex, Antigravity and Pi). The kit copies it to and from .claude/skills/ for Claude Code at every start. Create new project skills in .agents/skills/.
- Global instructions and skills are shared through the kit as well: a change made in one agent reaches the others at their next start.";

fn sync_rules(p: &Paths) -> Result<()> {
    let rule = format!("<!-- Atollo: regola del kit, riscritta a ogni avvio (non modificare) -->\n{KIT_RULE_TEXT}\n");
    for file in [
        join_rel(&p.claude, "rules/atollo.md"),
        join_rel(&p.gemini, "config/rules/atollo.md"),
    ] {
        if fs::read_to_string(&file).ok().as_deref() != Some(rule.as_str()) {
            if let Some(dir) = file.parent() {
                fs::create_dir_all(dir)?;
            }
            write_atomic(&file, rule.as_bytes())?;
        }
    }
    Ok(())
}

/// How often the holder of the lock shows it is still at work.
#[cfg(not(test))]
const BEAT: Duration = Duration::from_secs(15);
#[cfg(test)]
const BEAT: Duration = Duration::from_millis(50);

/// One sync at a time per kit: a lock folder with the pid inside (created
/// atomically); a stale one (holder gone, or silent for 2 minutes) is
/// taken. While the lock is held a thread rewrites `beat` in it every
/// [`BEAT`], and the silence is measured on that file: a long sync (a slow
/// USB drive) is never taken for a dead one (BUG-V6-18).
struct Lock {
    dir: PathBuf,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    beat: Option<std::thread::JoinHandle<()>>,
}

impl Lock {
    /// Time since the holder last showed it is alive: its last beat, or
    /// the lock's own time before the first one.
    fn silence(lock: &Path) -> Duration {
        [lock.join("beat"), lock.join("pid"), lock.to_path_buf()]
            .iter()
            .find_map(|f| fs::metadata(f).and_then(|m| m.modified()).ok())
            .and_then(|t| t.elapsed().ok())
            .unwrap_or(Duration::MAX)
    }

    fn take(state: &Path) -> Result<Lock> {
        fs::create_dir_all(state)?;
        let lock = state.join("lock");
        let pid_file = lock.join("pid");
        let me = std::process::id();
        for _ in 0..100 {
            match fs::create_dir(&lock) {
                Ok(()) => {
                    fs::write(&pid_file, me.to_string())?;
                    return Ok(Lock::beating(lock));
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    let age = Self::silence(&lock);
                    let holder: u32 = fs::read_to_string(&pid_file)
                        .ok()
                        .and_then(|s| s.trim().parse().ok())
                        .unwrap_or(0);
                    let stale = if holder > 0 {
                        !alive(holder) || age > Duration::from_secs(120)
                    } else {
                        age > Duration::from_secs(30)
                    };
                    if stale {
                        let _ = fs::remove_dir_all(&lock);
                    } else {
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }
        bail!("another sync is still running: skipped")
    }

    /// The lock just taken, with its heartbeat started.
    fn beating(dir: PathBuf) -> Lock {
        use std::sync::atomic::Ordering;
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let (flag, at) = (std::sync::Arc::clone(&stop), dir.clone());
        Self::beat_in(&at);
        let beat = std::thread::Builder::new()
            .name("agents-sync-lock".into())
            .spawn(move || {
                let mut last = std::time::Instant::now();
                while !flag.load(Ordering::Relaxed) {
                    std::thread::sleep(Duration::from_millis(10).min(BEAT));
                    if last.elapsed() >= BEAT {
                        Self::beat_in(&at);
                        last = std::time::Instant::now();
                    }
                }
            })
            .ok();
        Lock { dir, stop, beat }
    }

    /// A new time on `beat`, only while the lock is still this process's
    /// (never on one another process took after judging this one dead).
    fn beat_in(dir: &Path) {
        let me = std::process::id().to_string();
        if fs::read_to_string(dir.join("pid")).is_ok_and(|s| s.trim() == me) {
            let _ = fs::write(dir.join("beat"), std::process::id().to_string());
        }
    }

    fn touch(&self) {
        Self::beat_in(&self.dir);
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(t) = self.beat.take() {
            let _ = t.join();
        }
        let mine = fs::read_to_string(self.dir.join("pid"))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            == Some(std::process::id());
        if mine {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }
}

/// True if a process with this id runs (`platform::processes::exists`).
pub fn alive(pid: u32) -> bool {
    crate::platform::processes::exists(pid)
}

/// One agent's places.
pub struct Tool {
    pub id: &'static str,
    pub name: &'static str,
    pub instructions: PathBuf,
    pub skills: PathBuf,
    pub skills_key: String,
}

fn tools(p: &Paths) -> Vec<Tool> {
    vec![
        Tool {
            id: "claude",
            name: "Claude",
            instructions: p.claude.join("CLAUDE.md"),
            skills: p.claude.join("skills"),
            skills_key: "skills-claude".into(),
        },
        Tool {
            id: "codex",
            name: "Codex",
            instructions: p.codex.join("AGENTS.md"),
            skills: p.codex.join("skills"),
            skills_key: "skills-codex".into(),
        },
        Tool {
            id: "antigravity",
            name: "Antigravity",
            instructions: p.gemini.join("AGENTS.md"),
            skills: join_rel(&p.gemini, "antigravity-cli/skills"),
            skills_key: "skills-antigravity-cli".into(),
        },
        // Not `~/.agents/skills` as well, which Pi also reads: it would see
        // every skill twice.
        Tool {
            id: "pi",
            name: "Pi",
            instructions: p.pi.join("AGENTS.md"),
            skills: p.pi.join("skills"),
            skills_key: "skills-pi".into(),
        },
    ]
}

/// `work` with the sync's lock held: a change of the shared files (the kit's
/// MCP servers from the page) never meets a sync halfway.
pub fn locked<T>(kit: &Kit, work: impl FnOnce(&Paths) -> Result<T>) -> Result<T> {
    let p = Paths::of(kit);
    let _lock = Lock::take(&p.state)?;
    work(&p)
}

/// The whole sync. `project`: also the project skills of that folder.
pub fn run(kit: &Kit, project: Option<&Path>) -> Report {
    let p = Paths::of(kit);
    let mut report = Report::default();
    for dir in [&p.claude, &p.codex, &p.gemini.join("config"), &p.pi] {
        let _ = fs::create_dir_all(dir);
    }
    let lock = match Lock::take(&p.state) {
        Ok(l) => l,
        Err(e) => {
            report.warn(format!("{e:#}"));
            return report;
        }
    };
    let step = |who: &str, report: &mut Report, f: &mut dyn FnMut(&mut Report) -> Result<()>| {
        lock.touch();
        if let Err(e) = f(report) {
            report.warn(format!("{who}: sync failed: {e:#}"));
        }
    };

    step("kit moved", &mut report, &mut |r| {
        crate::agents::relocate::relocate(kit, r)
    });
    let tools = tools(&p);
    // A second pass on all but the last: they get what came from the later
    // ones.
    for t in tools.iter().chain(tools.iter().take(tools.len() - 1)) {
        let master = p.agents.join("AGENTS.md");
        step(t.name, &mut report, &mut |r| {
            tree::sync_tree(
                &p,
                &master,
                &t.instructions,
                &format!("instructions-{}", t.id),
                false,
                "AGENTS.md",
                r,
            )
        });
    }
    // Skills: deletions of every agent first, so none links again a skill
    // another deleted; on a disk without links, copies both ways.
    for t in &tools {
        step(t.name, &mut report, &mut |r| {
            links::remove_deleted(&p, t, r)
        });
    }
    // Two passes: a skill adopted from a later agent (moved into data\agents)
    // reaches the earlier ones in the same sync.
    let mut copied = Vec::new();
    for pass in 0..2 {
        for t in &tools {
            if copied.iter().any(|c: &&Tool| c.id == t.id) {
                continue;
            }
            let mut linked = true;
            step(t.name, &mut report, &mut |r| {
                linked = links::link_skills(&p, t, r)?;
                Ok(())
            });
            if !linked && pass == 0 {
                copied.push(t);
            }
        }
    }
    let n = copied.len();
    for t in copied.iter().chain(copied.iter().take(n.saturating_sub(1))) {
        let shared = p.agents.join("skills");
        step(t.name, &mut report, &mut |r| {
            tree::sync_tree(&p, &shared, &t.skills, &t.skills_key, true, "skills", r)
        });
    }
    // MCP: without the list Codex and Antigravity are left as they are (an
    // empty list would remove the kit's servers from them).
    let mut servers = None;
    step("MCP", &mut report, &mut |r| {
        let s = mcp::sync_claude(&p, mcp::load(&p)?, r)?;
        if !p.mcp_file().exists() {
            mcp::save(&p, &s)?;
        }
        servers = Some(s);
        Ok(())
    });
    if let Some(servers) = servers {
        step("Codex", &mut report, &mut |r| {
            codex::sync_config(&p, &servers, r)
        });
        step("Antigravity", &mut report, &mut |r| {
            mcp::sync_antigravity(&p, &servers, r)
        });
        step("Pi", &mut report, &mut |r| pi::sync_mcp(&p, &servers, r));
    }
    step("kit rule", &mut report, &mut |_| sync_rules(&p));
    step("Pi", &mut report, &mut |_| pi::sync_rule(&p));
    step("Pi", &mut report, &mut |_| pi::sync_models(kit, &p));
    step("Pi", &mut report, &mut |_| pi::sync_settings(kit, &p));
    step("trusted folders", &mut report, &mut |_| {
        trust::forget_pretrust(&p)
    });
    if let Some(dir) = project {
        step("project skills", &mut report, &mut |r| {
            project::sync(&p, dir, r)
        });
    }
    drop(lock);
    for w in &report.warnings {
        crate::common::log::warn(&format!("agents sync: {w}"));
    }
    report
}

#[cfg(test)]
#[path = "../../../../test/rust/agents/sync/mod.rs"]
mod tests;
