use super::*;

use crate::agents::sync::codex::codex_server;
use crate::kit::tests::TempDir;

fn value(p: &Preset) -> serde_json::Value {
    let dir = TempDir::new(&format!("preset-{}", p.id));
    crate::projects::mcp::set(&dir.0, std::slice::from_ref(&p.server)).unwrap();
    let text = std::fs::read_to_string(dir.0.join(".mcp.json")).unwrap();
    let o: serde_json::Value = serde_json::from_str(&text).unwrap();
    o["mcpServers"][p.id].clone()
}

/// Every ready server is a valid one, named as its id, with no secret: only
/// `${VAR}` of the variables the app sets.
#[test]
fn ready_servers_are_valid_and_hold_no_secret() {
    let all = all();
    let ids: Vec<&str> = all.iter().map(|p| p.id).collect();
    assert_eq!(
        ids,
        [
            "github",
            "playwright",
            "fetch",
            "filesystem",
            "mysql",
            "mongodb"
        ]
    );
    for p in &all {
        assert_eq!(p.server.name, p.id);
        assert!(["kit", "project"].contains(&p.scope), "{}", p.id);
        let v = value(p);
        for (k, val) in p.server.env.iter().chain(p.server.headers.iter()) {
            assert!(val.contains("${"), "{}: {k} holds a value", p.id);
        }
        assert!(v.is_object(), "{}", p.id);
    }
    // The servers that read the project's database are the project's.
    for p in all.iter().filter(|p| p.engine.is_some()) {
        assert_eq!(p.scope, "project", "{}", p.id);
    }
}

/// Codex never expands `${VAR}`: each variable reaches it as a variable
/// to pass on, never as text.
#[test]
fn codex_gets_the_variables() {
    let all = all();
    let codex = |id: &str| {
        let p = all.iter().find(|p| p.id == id).unwrap();
        codex_server(id, &value(p)).unwrap()
    };
    // The github server is the app by its path: no variable, no token.
    let gh = codex("github");
    assert!(gh.contains("args = [\"mcp\", \"github\"]"), "{gh}");
    assert!(
        !gh.contains("env_vars") && !gh.contains("bearer_token_env_var") && !gh.contains("${"),
        "{gh}"
    );
    let mysql = codex("mysql");
    assert!(mysql.contains("env_vars = [\"DSN\"]"), "{mysql}");
    assert!(!mysql.contains("${"), "{mysql}");
    let mongo = codex("mongodb");
    assert!(
        mongo.contains(MONGODB_VAR) && !mongo.contains("${"),
        "{mongo}"
    );
}

/// SEC-V7-01: the GitHub token is in no ready server's files or
/// environment; the kit's github server (the app) reads it itself.
#[test]
fn the_token_never_in_the_agents_environment() {
    let github = all().into_iter().find(|p| p.id == "github").unwrap();
    let text = serde_json::to_string(&value(&github)).unwrap();
    assert!(!text.contains("GITHUB_PERSONAL_ACCESS_TOKEN"), "{text}");
    assert_eq!(github.server.kind, "stdio");
    assert!(github.server.url.is_none() && github.server.headers.is_empty());
    assert!(github.server.env.is_empty());
}

/// SEC-V7-01: the github server is this app by its absolute path, never a
/// name the agent looks up: in the project's folder, where the agent runs,
/// Windows looks first, and a cloned repository's `atollo-mcp.cmd` or
/// `cmd.exe` would run as the kit's server. When the kit moves, the path is
/// rewritten with the rest of `mcp.json` (`agents::relocate`).
#[test]
fn github_runs_the_app_by_its_absolute_path() {
    let (old_root, new_root, exe) = if cfg!(windows) {
        (r"E:\kit", r"F:\My Atollo", r"E:\kit\Atollo.exe")
    } else {
        (
            "/media/u/kit",
            "/media/u/My Atollo",
            "/media/u/kit/Atollo.AppImage",
        )
    };
    let github = |exe: Option<&Path>| {
        all_for(exe)
            .into_iter()
            .find(|p| p.id == "github")
            .map(|p| p.server)
    };
    let server = github(Some(Path::new(exe))).unwrap();
    assert_eq!(server.command.as_deref(), Some(exe));
    assert_eq!(server.args, ["mcp", "github"]);
    // The app of this run too.
    let ours = all().into_iter().find(|p| p.id == "github").unwrap().server;
    let command = ours.command.unwrap();
    assert!(Path::new(&command).is_absolute(), "{command}");
    assert_eq!(
        Path::new(&command),
        std::env::current_exe().unwrap().as_path()
    );
    for s in [&server.command.clone().unwrap(), &command] {
        let name = crate::setup::needs::file_name(s).unwrap().to_lowercase();
        assert!(
            !name.starts_with("atollo-mcp") && !name.starts_with("cmd"),
            "{s}"
        );
    }
    // No app, or a relative one: no github server rather than a lookup.
    assert!(github(None).is_none());
    assert!(github(Some(Path::new("Atollo.exe"))).is_none());

    // The kit moved: the path in mcp.json follows the root.
    let tmp = TempDir::new("preset-moved");
    crate::projects::mcp::set(&tmp.0, std::slice::from_ref(&server)).unwrap();
    let text = std::fs::read(tmp.0.join(".mcp.json")).unwrap();
    let moved =
        crate::agents::relocate::Replacer::new(crate::kit::Os::current(), old_root, new_root)
            .replace(&text)
            .unwrap();
    let o: serde_json::Value = serde_json::from_slice(&moved).unwrap();
    let expected = Path::new(new_root).join(Path::new(exe).file_name().unwrap());
    assert_eq!(
        o["mcpServers"]["github"]["command"],
        expected.to_string_lossy().as_ref()
    );
}
