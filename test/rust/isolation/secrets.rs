use super::*;
use crate::platform::processes::{ancestors, Proc};
use std::path::PathBuf;

/// The command `sbx` runs: the app's path quoted (a path with spaces
/// unquoted fails: sbx 0.49.2), then `secret github`.
#[test]
fn the_command_quotes_the_apps_path() {
    assert_eq!(
        github_command(Path::new(r"D:\My Kit\Atollo.exe"), true, "ab12"),
        r#""D:\My Kit\Atollo.exe" secret github ab12"#
    );
    assert_eq!(
        github_prefix(Path::new("/media/u/My Kit/Atollo.AppImage"), false),
        "'/media/u/My Kit/Atollo.AppImage' secret github"
    );
}

/// `secret github` answers only a nonce of a command Atollo noted, in a
/// file readable by its owner only; a file of a first version (an entry
/// without a sandbox) or a damaged entry never drops the rest.
#[test]
fn only_a_noted_nonce_opens_the_token() {
    let (_tmp, kit) = crate::test_support::prepared_kit("isolation-nonce");
    let nonce = new_nonce().unwrap();
    assert_eq!(nonce.len(), 64);
    assert!(!nonce_known(&kit, &nonce), "nothing noted yet");
    let command = github_command(Path::new(r"D:\kit\Atollo.exe"), true, &nonce);
    note(&kit, "github", "PC1", "atollo-p-shop", Some(&command)).unwrap();
    assert!(nonce_known(&kit, &nonce));
    assert!(!nonce_known(&kit, &new_nonce().unwrap()), "another nonce");
    assert!(!nonce_known(&kit, ""), "no nonce");
    assert!(!nonce_known(&kit, &nonce[..10]), "a part of it");
    note(&kit, "github", "PC1", "atollo-p-shop", None).unwrap();
    assert!(!nonce_known(&kit, &nonce), "taken back");

    let path = crate::isolation::kit::root(&kit).join("secrets.json");
    std::fs::write(
        &path,
        r#"{"github":[{"computer":"PC1","command":"old"},{"bad":1},{"computer":"PC2","sandbox":"atollo-p-x","command":"y"}]}"#,
    )
    .unwrap();
    let all = noted(&kit);
    assert_eq!(all["github"].len(), 2, "the damaged entry only left out");
    assert_eq!(
        mine(&kit, "github", "PC1", "").map(|n| n.command),
        Some("old".into()),
        "a global note of a first version, read to be taken back"
    );
}

/// Registered when the sandbox has none; again over Atollo's own when the
/// kit moved; the user's own (none noted for this PC) never touched.
#[test]
fn only_atollos_own_secret_is_registered_or_replaced() {
    let prefix = "\"D:\\kit\\Atollo.exe\" secret github";
    let mine = Noted {
        computer: "PC1".into(),
        sandbox: "atollo-p-shop".into(),
        command: format!("{prefix} 0123abcd"),
    };
    assert_eq!(step(false, None, prefix), Step::Register);
    assert_eq!(step(true, Some(&mine), prefix), Step::Keep, "any nonce");
    assert_eq!(
        step(true, Some(&mine), "\"E:\\kit\\Atollo.exe\" secret github"),
        Step::Replace
    );
    assert_eq!(step(true, None, prefix), Step::Keep, "the user's own");
}

/// `sbx secret ls` of sbx 0.49.2, read for one scope: a sandbox's own
/// secret is not another's, nor a global one.
#[test]
fn a_secret_is_read_for_its_scope() {
    let ls = "SCOPE           TYPE      NAME     SECRET\n(global)        service   openai   (stored)\natollo-p-shop   service   github   (stored)\n";
    assert!(listed(ls, "github", "atollo-p-shop"));
    assert!(!listed(ls, "github", "atollo-p-blog"));
    assert!(!listed(ls, "github", "(global)"));
    assert!(listed(ls, "openai", "(global)"));
    assert!(!listed("No secrets found\n", "github", "atollo-p-shop"));
}

/// Noted per PC and sandbox, one entry each; taken back with `None`.
#[test]
fn the_commands_are_noted_per_pc_and_sandbox() {
    let (_tmp, kit) = crate::test_support::prepared_kit("isolation-secrets");
    assert!(noted(&kit).is_empty());
    note(&kit, "github", "PC1", "atollo-p-a", Some("x")).unwrap();
    note(&kit, "github", "PC1", "atollo-p-b", Some("y")).unwrap();
    note(&kit, "github", "PC2", "atollo-p-a", Some("z")).unwrap();
    note(&kit, "github", "PC1", "atollo-p-a", Some("x2")).unwrap();
    assert_eq!(noted(&kit)["github"].len(), 3);
    assert_eq!(
        mine(&kit, "github", "PC1", "atollo-p-a").map(|n| n.command),
        Some("x2".into())
    );
    note(&kit, "github", "PC1", "atollo-p-a", None).unwrap();
    assert_eq!(mine(&kit, "github", "PC1", "atollo-p-a"), None);
    assert!(mine(&kit, "github", "PC2", "atollo-p-a").is_some());
}

fn proc(pid: u32, parent: u32, path: &str) -> Proc {
    Proc {
        pid,
        parent,
        name: Path::new(path)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
        path: PathBuf::from(path),
    }
}

/// `secret github` answers only when the kit's `sbx` started it, even
/// through a shell; not an agent or a terminal of the PC.
#[test]
fn the_token_is_given_only_to_the_kits_sbx() {
    let sbx = Path::new(r"D:\kit\data\apps\win32-x64\sandboxes\bin\sbx.exe");
    let all = [
        proc(1, 0, r"C:\Windows\explorer.exe"),
        proc(10, 1, r"D:\kit\data\apps\win32-x64\sandboxes\bin\sbx.exe"),
        proc(11, 10, r"C:\Windows\System32\cmd.exe"),
        proc(12, 11, r"D:\kit\Atollo.exe"),
        proc(20, 1, r"D:\kit\data\apps\win32-x64\claude\claude.exe"),
        proc(21, 20, r"D:\kit\Atollo.exe"),
    ];
    assert!(
        asked_by_sbx(&ancestors(&all, 12, 3), sbx),
        "sbx through cmd"
    );
    assert!(!asked_by_sbx(&ancestors(&all, 21, 3), sbx), "an agent");
    let other_sbx = Path::new(r"C:\Users\me\AppData\Local\DockerSandboxes\bin\sbx.exe");
    assert!(!asked_by_sbx(&ancestors(&all, 12, 3), other_sbx));
    // A loop of parents ends; the depth is kept.
    let looped = [proc(5, 6, "/a"), proc(6, 5, "/b")];
    assert_eq!(ancestors(&looped, 5, 10).len(), 2);
    assert_eq!(ancestors(&all, 12, 1).len(), 1);
}
