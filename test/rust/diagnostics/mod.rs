use super::*;

use std::io::Read;

use crate::kit::tests::TempDir;
use crate::test_support::prepared_kit;

fn kit() -> (TempDir, Kit) {
    prepared_kit("diagnostics")
}

/// The text of every entry of the zip at `path`.
fn entries(path: &Path) -> Vec<(String, String)> {
    let mut zip = zip::ZipArchive::new(fs::File::open(path).unwrap()).unwrap();
    (0..zip.len())
        .map(|i| {
            let mut f = zip.by_index(i).unwrap();
            let mut text = String::new();
            f.read_to_string(&mut text).unwrap();
            (f.name().to_owned(), text)
        })
        .collect()
}

#[test]
fn secrets_and_the_profile_are_cleaned() {
    let secrets = vec!["s3cr3t-token-value".to_owned()];
    let text =
        r"key=s3cr3t-token-value at C:\Users\Mario\x and C:/Users/Mario/y GEMINI_API_KEY=abc";
    let out = clean(text, &secrets, Some(r"C:\Users\Mario"));
    assert!(!out.contains("s3cr3t"), "{out}");
    assert!(!out.contains("Mario"), "{out}");
    assert!(out.contains(r"~\x") && out.contains("~/y"), "{out}");
    assert!(out.contains("GEMINI_API_KEY=***"), "{out}");
    // Without the drive, as HOMEPATH has it.
    let out = clean(r"HOMEPATH=\Users\Mario\kit", &[], Some(r"C:\Users\Mario"));
    assert_eq!(out, r"HOMEPATH=~\kit");
}

#[test]
fn the_user_and_the_pc_are_not_named() {
    let text = "PATH=x\nUSERNAME=Mario\nCOMPUTERNAME=MARIO-PC\nuser=m\n";
    assert_eq!(
        without_identity(text),
        "PATH=x\nUSERNAME=***\nCOMPUTERNAME=***\nuser=***\n"
    );
}

/// No secret the kit keeps reaches the zip, wherever it was written: a
/// log, the environment, a project's database password, an agent's login.
#[test]
fn no_secret_of_the_kit_in_the_zip() {
    let (_tmp, kit) = kit();
    let secret_dir = kit.app_dir().join("secrets");
    fs::create_dir_all(&secret_dir).unwrap();
    fs::write(
        secret_dir.join("llama-key"),
        "a47598339e6ff07292b8ab99854422",
    )
    .unwrap();
    fs::write(secret_dir.join("hf-token"), "hf_ABCDEFGHIJKLMNOPQRSTUV").unwrap();
    let db = crate::db::auth::dir(&kit);
    fs::create_dir_all(&db).unwrap();
    fs::write(
        db.join("secrets.json"),
        r#"{"mysql":{"root":"9f8e7d6c5b4a39281706"}}"#,
    )
    .unwrap();
    let project = kit.project_data().join("shop").join("private");
    fs::create_dir_all(&project).unwrap();
    fs::write(
        project.join("db.json"),
        r#"{"mysql":{"user":"shop","password":"p4ssw0rd-of-the-shop"}}"#,
    )
    .unwrap();
    let pi = crate::agents::sync::pi::dir(&kit.home());
    fs::create_dir_all(&pi).unwrap();
    fs::write(
        pi.join("auth.json"),
        r#"{"anthropic":{"type":"api_key","key":"sk-ant-api-0123456789"}}"#,
    )
    .unwrap();
    fs::create_dir_all(kit.logs()).unwrap();
    fs::write(
        kit.logs().join("app.log"),
        "1 INFO started with a47598339e6ff07292b8ab99854422 and hf_ABCDEFGHIJKLMNOPQRSTUV\n2 INFO p4ssw0rd-of-the-shop 9f8e7d6c5b4a39281706 sk-ant-api-0123456789\n",
    )
    .unwrap();
    let mut env = KitEnv::default();
    env.set("SOME_VAR", "uses sk-ant-api-0123456789 inside");
    let zip = export(&kit, &env, "en").unwrap();
    assert!(zip.starts_with(dir(&kit)));
    let all = entries(&zip);
    let names: Vec<&str> = all.iter().map(|(n, _)| n.as_str()).collect();
    for want in [
        "summary.txt",
        "env.txt",
        "check.txt",
        "components.txt",
        "ports.txt",
        "logs/app.log",
    ] {
        assert!(names.contains(&want), "{want}: {names:?}");
    }
    for (name, text) in &all {
        for secret in [
            "a47598339e6ff07292b8ab99854422",
            "hf_ABCDEFGHIJKLMNOPQRSTUV",
            "9f8e7d6c5b4a39281706",
            "p4ssw0rd-of-the-shop",
            "sk-ant-api-0123456789",
        ] {
            assert!(!text.contains(secret), "{secret} in {name}");
        }
    }
    let summary = &all.iter().find(|(n, _)| n == "summary.txt").unwrap().1;
    assert!(summary.contains(env!("CARGO_PKG_VERSION")));
}

/// A long log goes in from its last lines, cut at a line.
#[test]
fn the_end_of_a_log() {
    let (_tmp, kit) = kit();
    let log = kit.app_dir().join("big.log");
    let line = "x".repeat(99) + "\n";
    fs::write(&log, line.repeat(5000)).unwrap();
    let t = tail(&log, 1000).unwrap();
    assert!(t.len() <= 1000);
    assert!(t.lines().all(|l| l.len() == 99));
}

/// The new issue's address: the bug form, the version, the system, nothing
/// that is not encoded.
#[test]
fn the_issue_has_version_and_system() {
    let (_tmp, kit) = kit();
    let path = issue_path(&kit);
    assert!(path.starts_with("nabre/atollo/issues/new?template=bug.yml&version="));
    assert!(path.contains(&format!("version={}", env!("CARGO_PKG_VERSION"))));
    assert!(path.contains(&format!("&system={}", kit.target().as_str())));
    assert!(!path.contains(' '), "{path}");
}
