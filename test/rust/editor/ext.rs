use super::*;
use crate::kit::{tests::TempDir, Arch};
use crate::test_support::prepared_kit;

#[test]
fn user_list_format() {
    let e = parse("# c\nMs-Python.python @python\nfoo.bar  # x\nnot-an-id\na.b.c\nvadimcn.vscode-lldb @rust,zig,cc\nfoo.bar @go\n");
    assert_eq!(e.len(), 3);
    assert_eq!(
        e[0],
        Entry {
            id: "ms-python.python".into(),
            programs: vec!["python".into()]
        }
    );
    assert_eq!(e[1].programs, vec!["rust", "zig", "cc"]);
    assert_eq!(
        e[2],
        Entry {
            id: "foo.bar".into(),
            programs: vec!["go".into()]
        },
        "later line wins"
    );
}

#[test]
fn matrix_is_valid() {
    let rows = matrix();
    assert!(rows.len() >= 60, "embedded matrix loaded");
    let languages: Vec<&str> = crate::detect::LANGUAGES.iter().map(|(n, _)| *n).collect();
    let catalog = Catalog::get();
    let mut seen = BTreeSet::new();
    for e in rows {
        assert!(valid_id(&e.id), "{}", e.id);
        assert_eq!(e.id, e.id.to_ascii_lowercase(), "{}: lower case", e.id);
        assert!(seen.insert(&e.id), "{}: twice", e.id);
        assert!(!e.name.is_empty(), "{}: no name", e.id);
        for l in &e.languages {
            assert!(
                languages.contains(&l.as_str()),
                "{}: unknown language {l}",
                e.id
            );
        }
        for p in e.tools.iter().chain(&e.programs) {
            assert!(catalog.find(p).is_some(), "{}: unknown component {p}", e.id);
        }
        assert!(
            e.programs.is_empty() || e.general(),
            "{}: programs only for general ones",
            e.id
        );
    }
    assert!(rows.iter().any(|e| e.id == CODELLDB));
    // Every language a project can have gets at least one extension.
    for l in &languages {
        assert!(
            rows.iter().any(|e| e.languages.iter().any(|x| x == l)),
            "no extension for {l}"
        );
    }
    let info = matrix_info(&Kit::at(std::env::temp_dir(), Arch::X64));
    assert_eq!(info.languages.len(), languages.len());
    assert_eq!(
        info.tools
            .iter()
            .map(|(id, _)| id.as_str())
            .collect::<Vec<_>>(),
        ["deno", "bun", "cmake", "mysql", "mongodb", "sqlite"]
    );
}

#[test]
fn ids() {
    for ok in ["a.b", "Ms-Python.python", "x_1.y-2"] {
        assert!(valid_id(ok), "{ok}");
    }
    for bad in ["", "a", ".b", "a.", "a.b.c", "a b.c", "a/b.c", r"..\x.y"] {
        assert!(!valid_id(bad), "{bad}");
    }
    let c = clean_choice(
        &[" Foo.Bar ".into(), "bad".into(), "a.b".into()],
        &["foo.bar".into(), "c.d".into(), "c.d".into()],
    );
    assert_eq!(c.added, ["a.b", "foo.bar"]);
    assert_eq!(c.removed, ["c.d"], "an added one is never removed");
}

#[test]
fn a_project_gets_the_extensions_of_its_languages() {
    let langs = |l: &[&str]| l.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    let have: BTreeSet<String> = ["ms-python.python".to_owned()].into();
    let none = Extensions::default();
    let list = of_project(&langs(&["Python"]), &[], &none, &have);
    let ids: Vec<&str> = list.iter().map(|e| e.id.as_str()).collect();
    assert!(ids.contains(&"ms-python.python") && ids.contains(&"charliermarsh.ruff"));
    assert!(!ids.contains(&"vue.volar") && !ids.contains(&"yzhang.markdown-all-in-one"));
    assert!(list.iter().all(|e| e.by_default && e.on));
    assert!(list[0].installed && !list[1].installed);
    assert!(
        !of_project(&langs(&["Dart"]), &[], &none, &have)
            .iter()
            .any(|e| e.id == "dart-code.flutter"),
        "optional: only by hand"
    );
    let deno = of_project(&langs(&["TypeScript"]), &langs(&["deno"]), &none, &have);
    assert!(deno.iter().any(|e| e.id == "denoland.vscode-deno"));
    let choice = clean_choice(
        &["dart-code.flutter".into(), "x.unknown".into()],
        &["charliermarsh.ruff".into()],
    );
    let list = of_project(&langs(&["Python"]), &[], &choice, &have);
    let ruff = list.iter().find(|e| e.id == "charliermarsh.ruff").unwrap();
    assert!(ruff.by_default && !ruff.on, "removed");
    let flutter = list.iter().find(|e| e.id == "dart-code.flutter").unwrap();
    assert!(!flutter.by_default && flutter.on && flutter.name == "Flutter");
    let unknown = list.iter().find(|e| e.id == "x.unknown").unwrap();
    assert_eq!(unknown.name, "x.unknown", "not in the matrix: its id");
}

#[test]
fn a_project_hides_what_it_does_not_use() {
    let langs = |l: &[&str]| l.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    let none = BTreeSet::new();
    let choice = clean_choice(&["eamodio.gitlens".into()], &["charliermarsh.ruff".into()]);
    let project = of_project(&langs(&["Python"]), &[], &choice, &none);
    let others: BTreeSet<String> = ["x.other".to_owned(), "usernamehw.errorlens".to_owned()].into();
    let user = parse("vue.volar\n");
    let h = hidden(&project, &others, &user);
    let has = |id: &str| h.iter().any(|x| x == id);
    assert!(!has("ms-python.python"), "its language");
    assert!(has("charliermarsh.ruff"), "removed from the project");
    assert!(!has("eamodio.gitlens"), "added by hand");
    assert!(
        has("rust-lang.rust-analyzer") && has("dart-code.flutter"),
        "other languages, optional"
    );
    assert!(has("x.other"), "added to another project");
    assert!(!has("usernamehw.errorlens"), "general: always on");
    assert!(!has("vue.volar"), "the user's list");
    assert!(!has("y.installed-by-hand"));
}

#[test]
fn pending_follows_projects_programs_offered_and_installed() {
    let (_tmp, kit) = prepared_kit("ext");
    let none = BTreeSet::new();
    let has = |p: &[String], id: &str| p.iter().any(|x| x == id);
    let p = pending(&kit, &none);
    assert!(has(&p, "usernamehw.errorlens"), "general: always");
    assert!(
        !has(&p, "anthropic.claude-code"),
        "Claude Code not in the kit"
    );
    assert!(!has(&p, "ms-python.python"), "no Python project");
    assert!(!has(&p, "eamodio.gitlens"), "optional");

    let claude = Catalog::get().find("claude").unwrap().key_path(&kit);
    fs::create_dir_all(claude.parent().unwrap()).unwrap();
    fs::write(&claude, b"").unwrap();
    let app = kit.projects().join("app");
    fs::create_dir_all(&app).unwrap();
    fs::write(app.join("main.py"), b"").unwrap();
    let p = pending(&kit, &none);
    assert!(has(&p, "anthropic.claude-code") && has(&p, "ms-python.python"));
    assert!(!has(&p, "vue.volar"), "not a language of a project");

    crate::projects::settings::update(&kit, "app", |s| {
        s.extensions = clean_choice(&["eamodio.gitlens".into()], &["charliermarsh.ruff".into()]);
    });
    let p = pending(&kit, &none);
    assert!(has(&p, "eamodio.gitlens"), "added by hand");
    assert!(!has(&p, "charliermarsh.ruff"), "removed from the project");

    mark_offered(&kit, &["ms-python.python".into(), "eamodio.gitlens".into()]).unwrap();
    fs::create_dir_all(extensions_dir(&kit).join("usernamehw.errorlens-3.26.0-universal")).unwrap();
    let p = pending(&kit, &none);
    assert!(!has(&p, "ms-python.python"), "offered once");
    assert!(
        has(&p, "eamodio.gitlens"),
        "added by hand: back if uninstalled"
    );
    assert!(!has(&p, "usernamehw.errorlens"), "already installed");

    let skip: BTreeSet<String> = ["eamodio.gitlens".to_owned()].into();
    assert!(
        !has(&pending(&kit, &skip), "eamodio.gitlens"),
        "failed in this run"
    );
}

#[test]
fn codelldb_follows_the_server() {
    use crate::kit::Target;
    let tmp = TempDir::new("codelldb");
    let on = |target| codelldb_asset(&Kit::at_target(tmp.0.clone(), target));
    // Native servers everywhere (no `rosetta` on the editor's server).
    assert!(!Catalog::get().find("vscodium-server").unwrap().rosetta);
    assert_eq!(on(Target::WIN32_X64), "codelldb-win32-x64.vsix");
    assert_eq!(on(Target::LINUX_X64), "codelldb-linux-x64.vsix");
    assert_eq!(on(Target::LINUX_ARM64), "codelldb-linux-arm64.vsix");
    assert_eq!(on(Target::DARWIN_ARM64), "codelldb-darwin-arm64.vsix");
}

#[test]
fn output_parsing() {
    let text = "Installing extension 'a.b'...\nExtension 'mikestead.dotenv' v1.0.1 was successfully installed.\nExtension 'X.Y' v2 is already installed.\nFailed Installing Extensions: c.d";
    let got = installed_from_output(text);
    assert!(got.contains("mikestead.dotenv") && got.contains("x.y"));
    assert!(!got.contains("a.b") && !got.contains("c.d"));
}

/// The matrix of a target chosen on the Tools page (D-121): no tool column
/// nor extension that only serves what the target cannot have; the
/// general extensions stay.
#[test]
fn matrix_follows_the_target() {
    let catalog = Catalog::get();
    let on = |id: &str, target| catalog.find(id).is_none_or(|c| c.is_available_on(target));
    let general = matrix().iter().filter(|e| e.general()).count();
    for target in crate::kit::Target::ALL {
        let kit = Kit::at_target(std::env::temp_dir(), target);
        let info = matrix_info(&kit);
        for (id, _) in &info.tools {
            assert!(on(id, target), "{id} on {target}");
        }
        for r in &info.extensions {
            let e = &r.extension;
            assert!(e.tools.is_empty() || e.tools.iter().any(|t| on(t, target)));
            assert!(e.programs.is_empty() || e.programs.iter().any(|p| on(p, target)));
        }
        let shown = info
            .extensions
            .iter()
            .filter(|r| r.extension.general())
            .count();
        assert_eq!(shown, general, "{target}");
    }
}
