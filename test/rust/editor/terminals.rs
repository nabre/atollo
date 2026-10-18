use super::*;
use crate::fsx::join_rel;
use crate::kit::{tests::TempDir, Arch};
use crate::setup::catalog::translate_rel;
use crate::versions::{install_dir, saved_dir, write_version};
use std::fs;
use std::path::Path;

/// The key file of `c` in `dir`, a copy of its folder, in the layout of the
/// kit's system (`node/node.exe` on Windows, `node/bin/node` elsewhere).
fn install_fake(kit: &Kit, c: &Component, dir: &Path) {
    let key = translate_rel(kit.target(), &c.key);
    let rel = key.strip_prefix(c.folder.as_deref().unwrap()).unwrap();
    let key = join_rel(dir, rel);
    fs::create_dir_all(key.parent().unwrap()).unwrap();
    fs::write(key, b"").unwrap();
}

fn kit(name: &str) -> (TempDir, Kit) {
    let tmp = TempDir::new(name);
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    (tmp, kit)
}

/// `entries` as a PATH of this system.
fn path_of(entries: &[String]) -> String {
    entries.join(if cfg!(windows) { ";" } else { ":" })
}

/// A folder of the system, outside the kit.
const SYSTEM: &str = if cfg!(windows) {
    r"C:\Windows\System32"
} else {
    "/usr/bin"
};

#[test]
fn strip_drops_entries_and_variables_inside_the_folders() {
    let apps = if cfg!(windows) {
        PathBuf::from(r"C:\K\apps")
    } else {
        PathBuf::from("/K/apps")
    };
    let s = |p: PathBuf| p.display().to_string();
    let tools = s(apps.join("mongodb-tools").join("bin"));
    let node = s(apps.join("node").join("22.11"));
    let mut env = KitEnv::default();
    env.set(
        "PATH",
        path_of(&[
            s(apps.join("mongodb").join("8.0").join("bin")),
            tools.clone(),
            node.clone(),
            SYSTEM.to_owned(),
        ]),
    );
    // Windows: another case and `/` are the same folder.
    let home = if cfg!(windows) {
        "C:/K/apps/MongoDB/8.0/".to_owned()
    } else {
        format!("{}/", s(apps.join("mongodb").join("8.0")))
    };
    env.set("MONGO_HOME", home);
    env.set("NODE_HOME", node.clone());
    env.set("OTHER", s(apps.join("mongodbx")));
    // Linux: another case is another folder.
    let other_case = s(apps.join("MongoDB"));
    env.set("UPPER", other_case.clone());
    strip(&mut env, &[apps.join("mongodb")]);
    assert_eq!(
        env.get("PATH"),
        Some(path_of(&[tools, node.clone(), SYSTEM.to_owned()]).as_str())
    );
    assert_eq!(env.get("MONGO_HOME"), None);
    assert_eq!(env.get("NODE_HOME"), Some(node.as_str()));
    assert_eq!(env.get("OTHER"), Some(s(apps.join("mongodbx")).as_str()));
    let kept = cfg!(target_os = "linux").then_some(other_case.as_str());
    assert_eq!(env.get("UPPER"), kept);
}

#[test]
fn saved_terminals_are_cleaned_and_get_unique_ids() {
    let (_tmp, kit) = kit("terminals-save");
    let view = list(&kit, "it");
    assert_eq!(view.terminals.len(), 1);
    assert!(view.terminals[0].builtin);
    assert_eq!(view.terminals[0].terminal.id, DEFAULT_ID);

    let input = Terminal {
        id: String::new(),
        name: "  Web PHP  ".into(),
        dir: Some(String::new()),
        tools: BTreeMap::from([
            ("php".to_owned(), Some("8.3".to_owned())),
            ("node".to_owned(), Some("not a line".to_owned())),
            ("git".to_owned(), Some("2.47".to_owned())),
            ("pwsh".to_owned(), None),
            ("nope".to_owned(), None),
        ]),
    };
    let id = save(&kit, input.clone(), "it").unwrap();
    assert_eq!(id, "web-php");
    let saved = find(&kit, &id, "it").unwrap();
    assert_eq!(saved.name, "Web PHP");
    assert_eq!(saved.dir, None);
    assert_eq!(
        saved.tools,
        BTreeMap::from([
            ("git".to_owned(), None),
            ("node".to_owned(), None),
            ("php".to_owned(), Some("8.3".to_owned())),
        ])
    );
    // Same name, another id; the default's id is never taken.
    assert_eq!(save(&kit, input, "it").unwrap(), "web-php-2");
    assert_eq!(new_id("Default", &[]), "default-2");
    assert_eq!(slug("Ünïcode !"), "n-code");
    assert_eq!(slug("***"), "terminal");

    // Nothing on disk: the tools it includes are missing.
    let view = list(&kit, "it");
    assert_eq!(view.terminals.len(), 3);
    assert_eq!(view.terminals[1].missing, ["git", "node", "php"]);

    // Changes keep the id; the default one and unknown ids are refused.
    let mut changed = saved.clone();
    changed.name = "PHP".into();
    changed.dir = Some(kit.projects().display().to_string());
    assert_eq!(save(&kit, changed, "it").unwrap(), "web-php");
    assert_eq!(find(&kit, "web-php", "it").unwrap().name, "PHP");
    let mut other = saved.clone();
    other.id = DEFAULT_ID.into();
    assert!(save(&kit, other.clone(), "it").is_err());
    other.id = "gone".into();
    assert!(save(&kit, other.clone(), "it").is_err());
    other.id = String::new();
    other.name = " ".into();
    assert!(save(&kit, other.clone(), "it").is_err());
    other.name = "x".repeat(NAME_MAX + 1);
    assert!(save(&kit, other.clone(), "it").is_err());
    other.name = "Dir".into();
    other.dir = Some(r"C:\no\such\folder".into());
    assert!(save(&kit, other, "it").is_err());

    remove(&kit, "web-php").unwrap();
    remove(&kit, "web-php").unwrap();
    let ids: Vec<String> = list(&kit, "it")
        .terminals
        .into_iter()
        .map(|t| t.terminal.id)
        .collect();
    assert_eq!(ids, [DEFAULT_ID, "web-php-2"]);
}

#[test]
fn a_terminal_keeps_its_tools_at_their_lines() {
    let (_tmp, kit) = kit("terminals-env");
    let catalog = Catalog::get();
    let node = catalog.find("node").unwrap();
    let python = catalog.find("python").unwrap();
    let node_main = install_dir(&kit, node, "v22.11.0");
    install_fake(&kit, node, &node_main);
    node.save_version(&kit, "v22.11.0").unwrap();
    let node_old = saved_dir(&kit, node, "20.18");
    install_fake(&kit, node, &node_old);
    write_version(&node_old, "v20.18.1").unwrap();
    let py = install_dir(&kit, python, "3.13.1");
    install_fake(&kit, python, &py);
    python.save_version(&kit, "3.13.1").unwrap();

    let options = tools(&kit);
    let ids: Vec<&str> = options.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, ["python", "node"]);
    let node_opt = &options[1];
    assert_eq!(node_opt.lines, ["22.11", "20.18"]);
    assert_eq!(
        node_opt.versions.get("20.18").map(String::as_str),
        Some("v20.18.1")
    );

    let npm = kit.data().join("npm-global");
    let mut base = KitEnv::default();
    base.set(
        "PATH",
        path_of(&[
            node_main.display().to_string(),
            npm.display().to_string(),
            py.display().to_string(),
            py.join("Scripts").display().to_string(),
            SYSTEM.to_owned(),
        ]),
    );
    base.set("NPM_CONFIG_PREFIX", npm.display().to_string());

    // The default terminal: the kit's environment, with its name.
    let all = find(&kit, DEFAULT_ID, "it").unwrap();
    assert_eq!(all.tools.len(), 2);
    let env_all = env(&kit, &all, base.clone());
    assert_eq!(env_all.get("PATH"), base.get("PATH"));
    assert_eq!(env_all.get("DEVKIT_TERMINAL"), Some(all.name.as_str()));

    // Node 20.18 only: Python and its folders leave.
    let only_node = Terminal {
        id: "n".into(),
        name: "Node 20".into(),
        dir: None,
        tools: BTreeMap::from([("node".to_owned(), Some("20.18".to_owned()))]),
    };
    let e = env(&kit, &only_node, base.clone());
    assert_eq!(
        e.get("PATH").unwrap(),
        path_of(&[
            node_old.display().to_string(),
            npm.display().to_string(),
            SYSTEM.to_owned(),
        ])
    );
    assert_eq!(e.get("DEVKIT_TERMINAL"), Some("Node 20"));

    // Python at its newest line, no Node: npm's global packages leave too.
    let only_py = Terminal {
        id: "p".into(),
        name: "Py".into(),
        dir: None,
        tools: BTreeMap::from([("python".to_owned(), None)]),
    };
    let e = env(&kit, &only_py, base);
    assert_eq!(
        e.get("PATH").unwrap(),
        path_of(&[
            py.display().to_string(),
            py.join("Scripts").display().to_string(),
            SYSTEM.to_owned(),
        ])
    );
    assert_eq!(e.get("NPM_CONFIG_PREFIX"), None);

    assert_eq!(
        start_dir(&kit, &only_py, None, "it").unwrap(),
        kit.projects()
    );
    let elsewhere = kit.data().display().to_string();
    assert_eq!(
        start_dir(&kit, &only_py, Some(&elsewhere), "it").unwrap(),
        kit.data()
    );
    assert!(start_dir(&kit, &only_py, Some("relative"), "it").is_err());
}
