use super::*;
use crate::kit::{tests::TempDir, Arch};
use crate::versions::{install_dir, saved_dir, write_version};
use std::fs;

/// The key file of `c` in `dir`, a copy of its folder.
fn install_fake(c: &Component, dir: &Path) {
    let rel = c.key.strip_prefix(c.folder.as_deref().unwrap()).unwrap();
    let key = dir.join(rel.trim_start_matches('/').replace('/', "\\"));
    fs::create_dir_all(key.parent().unwrap()).unwrap();
    fs::write(key, b"").unwrap();
}

fn kit(name: &str) -> (TempDir, Kit) {
    let tmp = TempDir::new(name);
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    (tmp, kit)
}

#[test]
fn strip_drops_entries_and_variables_inside_the_folders() {
    let mut env = KitEnv::default();
    env.set(
        "PATH",
        r"C:\K\apps\mongodb\8.0\bin;C:\K\apps\mongodb-tools\bin;C:\K\apps\node\22.11;C:\Windows\System32",
    );
    env.set("MONGO_HOME", "C:/K/apps/MongoDB/8.0/");
    env.set("NODE_HOME", r"C:\K\apps\node\22.11");
    env.set("OTHER", r"C:\K\apps\mongodbx");
    strip(&mut env, &[PathBuf::from(r"C:\K\apps\mongodb")]);
    assert_eq!(
        env.get("PATH"),
        Some(r"C:\K\apps\mongodb-tools\bin;C:\K\apps\node\22.11;C:\Windows\System32")
    );
    assert_eq!(env.get("MONGO_HOME"), None);
    assert_eq!(env.get("NODE_HOME"), Some(r"C:\K\apps\node\22.11"));
    assert_eq!(env.get("OTHER"), Some(r"C:\K\apps\mongodbx"));
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
    install_fake(node, &node_main);
    node.save_version(&kit, "v22.11.0").unwrap();
    let node_old = saved_dir(&kit, node, "20.18");
    install_fake(node, &node_old);
    write_version(&node_old, "v20.18.1").unwrap();
    let py = install_dir(&kit, python, "3.13.1");
    install_fake(python, &py);
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
        format!(
            r"{};{};{};{}\Scripts;C:\Windows\System32",
            node_main.display(),
            npm.display(),
            py.display(),
            py.display()
        ),
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
        format!(
            r"{};{};C:\Windows\System32",
            node_old.display(),
            npm.display()
        )
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
        format!(
            r"{};{}\Scripts;C:\Windows\System32",
            py.display(),
            py.display()
        )
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
