//! I1 (LEVELS.md §2, §5): no code path starts a process of levels 1–3
//! without going through `levels::decide`. A scan of `tauri/src`: the
//! terminals (a pty session), the actions, the creations' lines and the
//! `sbx exec`/`sbx run` of a session are built or opened only in `levels/`
//! (`levels::start`), where they come from a `Ready`; a process is spawned
//! only by the kit's own programs (§5.2), each named, or by what
//! `levels::start` calls. A new caller fails here until it goes through
//! the levels or is named, with its reason, below.

use std::path::{Path, PathBuf};

/// Who opens a terminal, starts an action, builds a creation's line or the
/// `sbx` arguments of a session.
const GATES: &[&str] = &[
    "open_launch(",
    "open_launch_then(",
    "pty::shell(",
    "pty::command(",
    "pty::program(",
    ".start_in(",
    "line_command(",
    "scaffold_args(",
    "kit_terminal_args(",
    "pi_session_args(",
    "agy_session_args(",
    "agent_session_args(",
    "exec_args(",
    "web_line(",
];

/// Where [`GATES`] may be called besides `levels/`: their definitions, and
/// the exceptions named with their reason.
const GATES_ALLOWED: &[(&str, &str)] = &[
    ("editor/terminal.rs", "defines open_launch*"),
    ("editor/pty.rs", "defines the pty launches"),
    ("isolation/vm.rs", "the pure sbx argument builders"),
    (
        "isolation/mod.rs",
        "Vm's lifecycle (launch, agent_session_args); fix_launch is the kit's sbx (§5.2)",
    ),
    ("projects/run/start.rs", "defines start_in and its sbx exec"),
    ("projects/frameworks/runner.rs", "defines line_command"),
    ("ipc/isolation.rs", "isolation_run: the kit's sbx (§5.2)"),
];

/// What spawns a process.
const SPAWNS: &[&str] = &[
    "Command::new(",
    "kit_command(",
    "spawn_in(",
    "shell_command(",
];

/// Where [`SPAWNS`] may be called besides `levels/`: the system's layer,
/// the kit's own programs (§5.2), and the programs of level 2 that only
/// `levels::start` or a decided entry point starts.
const SPAWNS_ALLOWED: &[(&str, &str)] = &[
    ("platform/", "the system's layer"),
    ("os/", "the system's layer"),
    (
        "db/",
        "database servers, clients, phpMyAdmin, mongo-express (§5.2)",
    ),
    ("llm/service.rs", "llama-server (§5.2)"),
    ("github/", "Git and gh (§5.2)"),
    ("projects/mod.rs", "Git (§5.2)"),
    ("projects/frameworks/mine.rs", "Git (§5.2)"),
    ("install/", "installers (§5.2)"),
    ("setup/checkup.rs", "installers (§5.2)"),
    ("editor/mod.rs", "the editor server (§5.2)"),
    ("editor/ext.rs", "extension installs (§5.2)"),
    ("editor/theme.rs", "the editor's theme (§5.2)"),
    ("isolation/", "sbx itself (§5.2)"),
    (
        "projects/run/start.rs",
        "an action: Runner::start_in, from levels::start",
    ),
    (
        "projects/apache.rs",
        "a site's Apache, from Runner::start_in",
    ),
    (
        "projects/caddy/command.rs",
        "a site's Caddy, from Runner::start_in",
    ),
    (
        "projects/fastcgi.rs",
        "a site's FastCGI, from Runner::start_in",
    ),
    (
        "projects/frameworks/runner.rs",
        "a creation's line, from levels::start::scaffold_command",
    ),
    ("servers/process.rs", "a pool, decided (R2/R6) since step 3"),
];

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Every call of `patterns` in `tauri/src` outside `levels/` and `allowed`:
/// `file:line: text`. Comments are not calls.
fn outside(patterns: &[&str], allowed: &[(&str, &str)]) -> (Vec<String>, usize) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    sources(&root, &mut files);
    let mut found = Vec::new();
    let mut seen = 0;
    for file in files {
        let rel = file
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let text = std::fs::read_to_string(&file).unwrap();
        for (n, line) in text.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") || !patterns.iter().any(|p| code.contains(p)) {
                continue;
            }
            seen += 1;
            let ok = rel.starts_with("levels/") || allowed.iter().any(|(a, _)| rel.starts_with(a));
            if !ok {
                found.push(format!("{rel}:{}: {}", n + 1, code));
            }
        }
    }
    (found, seen)
}

#[test]
fn no_process_outside_levels() {
    let (gates, seen) = outside(GATES, GATES_ALLOWED);
    assert!(seen > 0, "the scan sees the gates");
    assert!(
        gates.is_empty(),
        "a terminal, an action, a creation or a session in a VM started outside levels::start \
         (LEVELS.md I1):\n{}",
        gates.join("\n")
    );
    let (spawns, seen) = outside(SPAWNS, SPAWNS_ALLOWED);
    assert!(seen > 0, "the scan sees the spawns");
    assert!(
        spawns.is_empty(),
        "a process spawned outside levels and the kit's own programs (LEVELS.md I1, §5.2):\n{}",
        spawns.join("\n")
    );
    // The entry points of step 4 do reach the gates through `levels::start`.
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let start = std::fs::read_to_string(src.join("levels/start.rs")).unwrap();
    for gate in [
        "open_launch_then(",
        ".start_in(",
        "line_command(",
        "scaffold_args(",
        "exec_args(",
    ] {
        assert!(start.contains(gate), "{gate}");
    }
    // Step 5: the editor's sessions and the forwarders are answered by
    // `levels::requests` (decide → prepare → start), and nothing is left of
    // the editor's own way to the VM.
    let requests = std::fs::read_to_string(src.join("levels/requests.rs")).unwrap();
    for step in ["decide(", "environment(", "prepare(", "vm_session_args("] {
        assert!(requests.contains(step), "{step}");
    }
    assert!(!src.join("isolation/session.rs").exists());
}

/// Step 6 (LEVELS.md I3, E16, E17): the app starts no agent's program on
/// the PC. No file of the app names an agent's binary (`agents::exe_of`,
/// `claude.exe`, `codex.exe`, `agy.exe`, `pi.exe`, `setup-token`) beside
/// a spawn or a pty session (`codex logout` is checked by name below:
/// `gh auth logout` is the kit's own); the only reader of
/// `exe_of` is the VM's program of an agent (`isolation::agent_in_vm`,
/// the build for the VM's Linux, run by `sbx exec`).
#[test]
fn no_agent_binary_is_started_by_the_app_on_the_pc() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    sources(&root, &mut files);
    let binaries = ["claude.exe", "codex.exe", "agy.exe", "pi.exe"];
    let mut exe_of = Vec::new();
    let mut found = Vec::new();
    for file in files {
        let rel = file
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let text = std::fs::read_to_string(&file).unwrap();
        let code: Vec<&str> = text
            .lines()
            .map(str::trim_start)
            .filter(|l| !l.starts_with("//"))
            .collect();
        if code.iter().any(|l| l.contains("exe_of(")) && rel != "agents/mod.rs" {
            exe_of.push(rel.clone());
        }
        let spawns = code
            .iter()
            .any(|l| SPAWNS.iter().chain(GATES).any(|p| l.contains(p)));
        let names = code
            .iter()
            .any(|l| binaries.iter().any(|b| l.contains(b)) || l.contains("setup-token"));
        if spawns && names {
            found.push(rel);
        }
    }
    assert_eq!(exe_of, ["isolation/mod.rs"], "who reads an agent's binary");
    assert!(
        found.is_empty(),
        "an agent's binary near a spawn on the PC: {found:?}"
    );
    // Gone with it: the token's terminal session, `codex logout`, the
    // launchers and the agents' environment of the PC.
    let agents = std::fs::read_to_string(root.join("agents/mod.rs")).unwrap();
    for gone in ["fn launch_env", "fn codex_logout", "fn running"] {
        assert!(!agents.contains(gone), "{gone}");
    }
    let token = std::fs::read_to_string(root.join("agents/claude_token.rs")).unwrap();
    assert!(!token.contains("fn link"));
    assert!(!Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../runtime/agents")
        .exists());
}
