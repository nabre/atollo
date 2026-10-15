//! Keeps Claude Code, Codex and Antigravity aligned: the same global
//! instructions, skills and MCP servers for the three (port of nabre/atollo
//! `setup/agents/sync.js` and its modules). The shared copy lives in
//! `data\agents`:
//!
//! ```text
//! data\agents\AGENTS.md   = data\home\.claude\CLAUDE.md   (Claude)
//!                         = data\home\.codex\AGENTS.md    (Codex)
//!                         = data\home\.gemini\AGENTS.md   (Antigravity)
//! data\agents\skills\     <- junctions in each agent's skills folder
//! data\agents\mcp.json    = "mcpServers" in .claude\.claude.json
//!                         -> [mcp_servers] in .codex\config.toml
//!                         -> .gemini\config\mcp_config.json
//! ```
//!
//! A change made on either side is carried to the others, deletions too,
//! thanks to the state of the last sync in `data\agents\.sync`; when both
//! sides changed the newer wins and the other is saved in
//! `.sync\conflicts`. Also: the kit rule for the three agents, the trusted
//! folders (`projects\` and its subfolders) for Claude and Codex, and the
//! project skills (`.agents\skills` <-> `.claude\skills`) with `project`.
//!
//! Run at start, before and after every agent, and when an agent ends.
//! Never blocks the agents: each step fails alone, as a warning.

pub(crate) mod codex;
mod links;
pub(crate) mod mcp;
mod project;
mod tree;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{bail, Context, Result};
use serde_json::Value;

use crate::fsx::write_atomic;
use crate::kit::Kit;

/// Folders of the kit and of the three agents.
#[derive(Debug, Clone)]
pub struct Paths {
    pub root: PathBuf,

    pub agents: PathBuf,
    pub state: PathBuf,
    pub claude: PathBuf,
    pub codex: PathBuf,
    pub gemini: PathBuf,
    pub projects: PathBuf,
}

impl Paths {
    pub fn of(kit: &Kit) -> Paths {
        let data = kit.data();
        let home = kit.home();
        Paths {
            root: kit.root().to_path_buf(),
            agents: data.join("agents"),
            state: data.join(r"agents\.sync"),
            claude: home.join(".claude"),
            codex: home.join(".codex"),
            gemini: home.join(".gemini"),
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
pub fn read_json(file: &Path) -> Result<Option<Value>> {
    crate::fsx::try_read_json(file)
}

/// State of the last sync: rebuilt (empty) if damaged.
pub fn read_state(file: &Path) -> Value {
    read_json(file)
        .ok()
        .flatten()
        .unwrap_or(Value::Object(Default::default()))
}

pub fn write_json(file: &Path, v: &Value) -> Result<()> {
    crate::fsx::write_json(file, v)
}

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
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
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
/// three read all of them (`rules.js`).
pub const KIT_RULE_TEXT: &str = "In this kit (Atollo) Claude Code, Codex and Antigravity share the same instructions and skills.
- Project instructions: CLAUDE.md, AGENTS.md and GEMINI.md are equivalent. At the start of a task, read each of these files in the workspace root that is not already in your context, and follow all of them together.
- Project skills: the shared folder is .agents/skills/<name>/SKILL.md in the project root (read by Codex and Antigravity). The kit copies it to and from .claude/skills/ for Claude Code at every start. Create new project skills in .agents/skills/.
- Global instructions and skills are shared through the kit as well: a change made in one agent reaches the others at their next start.";

fn sync_rules(p: &Paths) -> Result<()> {
    let rule = format!("<!-- Atollo: regola del kit, riscritta a ogni avvio (non modificare) -->\n{KIT_RULE_TEXT}\n");
    for file in [
        p.claude.join(r"rules\atollo.md"),
        p.gemini.join(r"config\rules\atollo.md"),
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

/// `projects\` and every non-dot folder in it (`trust.js`).
pub fn trusted_folders(p: &Paths) -> Vec<PathBuf> {
    let mut out = vec![p.projects.clone()];
    let mut subs: Vec<PathBuf> = fs::read_dir(&p.projects)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| {
            e.file_type().is_ok_and(|t| t.is_dir())
                && !e.file_name().to_string_lossy().starts_with('.')
        })
        .map(|e| e.path())
        .collect();
    subs.sort();
    out.extend(subs);
    out
}

/// Claude: `projects["D:/kit/projects/x"].hasTrustDialogAccepted = true`, with
/// the drive letter both upper and lower case (the kit's windows write
/// "D:/...", VSCodium "d:/..."). Re-read just before writing.
fn trust_claude(p: &Paths) -> Result<()> {
    let file = p.claude_json();
    let current = read_json(&file)?;
    let trusted = |cfg: &Option<Value>, k: &str| {
        cfg.as_ref()
            .and_then(|c| {
                c.pointer(&format!(
                    "/projects/{}",
                    k.replace('~', "~0").replace('/', "~1")
                ))
            })
            .and_then(|v| v.get("hasTrustDialogAccepted"))
            == Some(&Value::Bool(true))
    };
    let mut todo = Vec::new();
    for f in trusted_folders(p) {
        let key = f.display().to_string().replace('\\', "/");
        let mut chars = key.chars();
        let first = chars.next().unwrap_or_default();
        let rest: String = chars.collect();
        for k in [
            format!("{}{rest}", first.to_ascii_uppercase()),
            format!("{}{rest}", first.to_ascii_lowercase()),
        ] {
            if !trusted(&current, &k) && !todo.contains(&k) {
                todo.push(k);
            }
        }
    }
    if current.is_some() && todo.is_empty() {
        return Ok(());
    }
    let mut fresh = read_json(&file)?.unwrap_or_else(|| Value::Object(Default::default()));
    let obj = fresh
        .as_object_mut()
        .context(".claude.json is not an object")?;
    let projects = obj
        .entry("projects")
        .or_insert_with(|| Value::Object(Default::default()));
    if !projects.is_object() {
        *projects = Value::Object(Default::default());
    }
    if let Some(map) = projects.as_object_mut() {
        for k in todo {
            let entry = map
                .entry(k)
                .or_insert_with(|| Value::Object(Default::default()));
            if !entry.is_object() {
                *entry = Value::Object(Default::default());
            }
            if let Some(e) = entry.as_object_mut() {
                e.insert("hasTrustDialogAccepted".into(), Value::Bool(true));
            }
        }
    }
    write_json(&file, &fresh)
}

/// One sync at a time per kit: a lock folder with the pid inside (created
/// atomically); a stale one (holder gone, or older than 2 minutes) is taken.
struct Lock(PathBuf);

impl Lock {
    fn take(state: &Path) -> Result<Lock> {
        fs::create_dir_all(state)?;
        let lock = state.join("lock");
        let pid_file = lock.join("pid");
        let me = std::process::id();
        for _ in 0..100 {
            match fs::create_dir(&lock) {
                Ok(()) => {
                    fs::write(&pid_file, me.to_string())?;
                    return Ok(Lock(lock));
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    let age = fs::metadata(&lock)
                        .and_then(|m| m.modified())
                        .ok()
                        .and_then(|t| t.elapsed().ok())
                        .unwrap_or(Duration::MAX);
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

    fn touch(&self) {
        let _ = fs::write(self.0.join("pid"), std::process::id().to_string());
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let mine = fs::read_to_string(self.0.join("pid"))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            == Some(std::process::id());
        if mine {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

/// True if a process with this id runs.
pub fn alive(pid: u32) -> bool {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    // SAFETY: query-only handle, closed right after.
    unsafe {
        let Ok(h) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return false;
        };
        let mut code = 0u32;
        let ok = GetExitCodeProcess(h, &mut code).is_ok() && code == 259; // STILL_ACTIVE
        let _ = CloseHandle(h);
        ok
    }
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
            skills: p.gemini.join(r"antigravity-cli\skills"),
            skills_key: "skills-antigravity-cli".into(),
        },
    ]
}

/// The whole sync. `project`: also the project skills of that folder.
pub fn run(kit: &Kit, project: Option<&Path>) -> Report {
    let p = Paths::of(kit);
    let mut report = Report::default();
    for dir in [&p.claude, &p.codex, &p.gemini.join("config")] {
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
    // Second pass on Claude and Codex: they get what came from the later ones.
    for t in tools.iter().chain(tools.iter().take(2)) {
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
    }
    step("kit rule", &mut report, &mut |_| sync_rules(&p));
    step("trusted folders", &mut report, &mut |_| trust_claude(&p));
    if let Some(dir) = project {
        step("project skills", &mut report, &mut |r| {
            project::sync(&p, dir, r)
        });
    }
    drop(lock);
    for w in &report.warnings {
        crate::log::warn(&format!("agents sync: {w}"));
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

    pub fn kit(name: &str) -> (TempDir, Kit, Paths) {
        let tmp = TempDir::new(name);
        let kit = Kit::at(tmp.0.join("k (1) & co"), Arch::X64);
        kit.prepare().unwrap();
        let p = Paths::of(&kit);
        (tmp, kit, p)
    }

    #[test]
    fn instructions_reach_every_agent_and_back() {
        let (_t, kit, p) = kit("sync-instr");
        fs::write(p.agents.join("AGENTS.md").tap_dir(), "shared v1").unwrap();
        let r = run(&kit, None);
        assert!(r.warnings.is_empty(), "{:?}", r.warnings);
        for f in [
            p.claude.join("CLAUDE.md"),
            p.codex.join("AGENTS.md"),
            p.gemini.join("AGENTS.md"),
        ] {
            assert_eq!(
                fs::read_to_string(&f).unwrap(),
                "shared v1",
                "{}",
                f.display()
            );
        }
        // Codex changes it: everyone gets it.
        std::thread::sleep(Duration::from_millis(20));
        fs::write(p.codex.join("AGENTS.md"), "from codex").unwrap();
        run(&kit, None);
        assert_eq!(
            fs::read_to_string(p.agents.join("AGENTS.md")).unwrap(),
            "from codex"
        );
        assert_eq!(
            fs::read_to_string(p.claude.join("CLAUDE.md")).unwrap(),
            "from codex"
        );
        assert_eq!(
            fs::read_to_string(p.gemini.join("AGENTS.md")).unwrap(),
            "from codex"
        );
        // Claude deletes it: deleted everywhere.
        fs::remove_file(p.claude.join("CLAUDE.md")).unwrap();
        run(&kit, None);
        assert!(!p.agents.join("AGENTS.md").exists());
        assert!(!p.gemini.join("AGENTS.md").exists());
    }

    trait TapDir {
        fn tap_dir(self) -> Self;
    }
    impl TapDir for PathBuf {
        fn tap_dir(self) -> Self {
            fs::create_dir_all(self.parent().unwrap()).unwrap();
            self
        }
    }

    #[test]
    fn rule_and_trust() {
        let (_t, kit, p) = kit("sync-trust");
        fs::create_dir_all(p.projects.join("app")).unwrap();
        fs::create_dir_all(p.projects.join(".hidden")).unwrap();
        fs::create_dir_all(&p.claude).unwrap();
        fs::write(
            p.claude_json(),
            r#"{"projects":{"x":{"history":[1]}},"numStartups":3}"#,
        )
        .unwrap();
        run(&kit, None);
        assert!(fs::read_to_string(p.claude.join(r"rules\atollo.md"))
            .unwrap()
            .contains("Atollo"));
        assert!(p.gemini.join(r"config\rules\atollo.md").is_file());
        let cfg: Value =
            serde_json::from_str(&fs::read_to_string(p.claude_json()).unwrap()).unwrap();
        let app = p
            .projects
            .join("app")
            .display()
            .to_string()
            .replace('\\', "/");
        let lower = format!("{}{}", app[..1].to_ascii_lowercase(), &app[1..]);
        assert_eq!(cfg["projects"][&app]["hasTrustDialogAccepted"], true);
        assert_eq!(cfg["projects"][&lower]["hasTrustDialogAccepted"], true);
        assert_eq!(cfg["numStartups"], 3, "the rest kept");
        assert_eq!(cfg["projects"]["x"]["history"][0], 1);
        let hidden = p
            .projects
            .join(".hidden")
            .display()
            .to_string()
            .replace('\\', "/");
        assert!(cfg["projects"].get(&hidden).is_none());
    }

    #[test]
    fn lock_is_released_and_stale_lock_taken() {
        let (_t, _kit, p) = kit("sync-lock");
        {
            let _l = Lock::take(&p.state).unwrap();
            assert!(p.state.join("lock").exists());
        }
        assert!(!p.state.join("lock").exists());
        fs::create_dir_all(p.state.join("lock")).unwrap();
        fs::write(p.state.join(r"lock\pid"), "4000000000").unwrap();
        assert!(Lock::take(&p.state).is_ok(), "holder gone: stale");
    }

    #[test]
    fn stamp_format() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20_740), (2026, 10, 14));
        assert_eq!(stamp().len(), 15);
    }

    #[test]
    fn canonical_sorts_keys() {
        let a: Value = serde_json::from_str(r#"{"b":1,"a":{"y":2,"x":[1,"s"]}}"#).unwrap();
        let b: Value = serde_json::from_str(r#"{"a":{"x":[1,"s"],"y":2},"b":1}"#).unwrap();
        assert_eq!(canonical(&a), canonical(&b));
    }
}
