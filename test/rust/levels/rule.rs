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
    (
        "agents/claude_token.rs",
        "claude setup-token (E16): removed by step 6",
    ),
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
    (
        "os/",
        "the system's layer (and Antigravity's keyring helpers, §5.2)",
    ),
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
    ("agents/mod.rs", "codex logout (E17): removed by step 6"),
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
