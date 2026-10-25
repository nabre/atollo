use super::*;

use crate::kit::tests::TempDir;
use crate::kit::Arch;
use crate::test_support::prepared_kit;

fn project(dir: &Path) {
    for (rel, text) in [
        ("composer.json", "{}"),
        ("src/App.php", "<?php"),
        ("public/index.php", "<?php"),
        (".env", "DB_PASSWORD=secret"),
        (".env.local", "X=1"),
        (".env.example", "DB_PASSWORD="),
        ("vendor/autoload.php", "<?php"),
        ("node_modules/x/index.js", "x"),
        ("README.md", "# shop"),
    ] {
        let p = dir.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
    }
}

/// In a repository, with the Git of the kit given by ATOLLO_TEST_KIT:
/// `.gitignore` decides, new files not ignored are taken.
#[test]
fn real_git_files() {
    let Ok(root) = std::env::var("ATOLLO_TEST_KIT") else {
        eprintln!("ATOLLO_TEST_KIT not set: skipped");
        return;
    };
    let kit = Kit::at(PathBuf::from(root), Arch::detect());
    let env = KitEnv::build(
        &kit,
        &crate::common::settings::Settings::default(),
        "en",
        &crate::env::Host::capture(),
    );
    let tmp = TempDir::new("mine-git");
    let dir = tmp.0.join("shop");
    project(&dir);
    fs::write(dir.join(".gitignore"), "public/\n").unwrap();
    let git = crate::setup::kit_path(&kit, r"git\cmd\git.exe");
    let ok = crate::platform::jobs::kit_command(&git, &env)
        .args(["init", "-q"])
        .current_dir(&dir)
        .status()
        .unwrap()
        .success();
    assert!(ok);
    let files = files_of(&kit, &env, &dir);
    assert!(files.contains(&"src/App.php".to_owned()), "{files:?}");
    assert!(files.contains(&".gitignore".to_owned()));
    assert!(!files.iter().any(|f| f.starts_with("public/")), "ignored");
    // Not in .gitignore, so Git would commit them; .env never.
    assert!(files.contains(&"vendor/autoload.php".to_owned()));
    assert!(!files.iter().any(|f| f == ".env" || f == ".env.local"));
}

#[test]
fn ids_of_names() {
    assert_eq!(
        id_of("Laravel di sempre").as_deref(),
        Some("mine-laravel-di-sempre")
    );
    assert_eq!(id_of("  Più  API!! ").as_deref(), Some("mine-pi-api"));
    assert_eq!(id_of("!!!"), None);
    assert!(id_of(&"a".repeat(80)).unwrap().len() <= 40);
}

/// Without Git: every file but the folders without sources and the
/// secrets.
#[test]
fn files_of_a_folder() {
    let tmp = TempDir::new("mine-files");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    let dir = tmp.0.join("projects").join("shop");
    project(&dir);
    let files = files_of(&kit, &KitEnv::default(), &dir);
    assert_eq!(
        files,
        [
            ".env.example",
            "README.md",
            "composer.json",
            "public/index.php",
            "src/App.php"
        ]
    );
    let top = preview(&kit, &KitEnv::default(), &dir);
    let src = top.iter().find(|e| e.name == "src").unwrap();
    assert!(src.dir && src.files == 1 && src.bytes == 5);
}

/// Saved: the folder copied, the base in the user's bases, created from
/// a template; the same name replaces it; removed, its folder goes.
#[test]
fn a_project_saved_as_a_base() {
    let (tmp, kit) = prepared_kit("mine-save");
    let dir = tmp.0.join("projects").join("shop");
    project(&dir);
    let env = KitEnv::default();
    let id = save(
        &kit,
        &env,
        &dir,
        "Laravel di sempre",
        "con i pacchetti",
        &["public".into()],
    )
    .unwrap();
    assert_eq!(id, "mine-laravel-di-sempre");
    let base = mine_dir(&kit).join(&id);
    assert!(base.join("src").join("App.php").is_file());
    assert!(!base.join("public").exists(), "left out");
    assert!(!base.join(".env").exists(), "never the passwords");
    let piece = super::super::load(&kit)
        .pieces
        .into_iter()
        .find(|p| p.id == id)
        .unwrap();
    assert_eq!(piece.category.as_deref(), Some(CATEGORY));
    assert_eq!(piece.template.as_deref(), Some(id.as_str()));
    assert!(piece.create.is_empty());

    // Again with the same name: replaced, with the folder.
    fs::remove_file(dir.join("README.md")).unwrap();
    save(&kit, &env, &dir, "Laravel di sempre", "", &[]).unwrap();
    assert!(!base.join("README.md").exists());
    assert!(base.join("public").join("index.php").is_file());
    assert_eq!(
        super::super::load(&kit)
            .pieces
            .iter()
            .filter(|p| p.id == id)
            .count(),
        1
    );

    // Removed from the bases: its folder goes at the save.
    let rest: Vec<Piece> = super::super::load(&kit)
        .pieces
        .into_iter()
        .filter(|p| p.id != id)
        .collect();
    super::super::save(&kit, rest).unwrap();
    assert!(!base.exists());

    assert!(save(&kit, &env, &dir, "", "", &[]).is_err());
    assert!(template_dir(&kit, "../x").is_none());
}
