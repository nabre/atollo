use super::*;

/// The command `sbx` runs: the app's path quoted (a path with spaces
/// unquoted fails: sbx 0.49.2), then `secret github`.
#[test]
fn the_command_quotes_the_apps_path() {
    assert_eq!(
        github_command(Path::new(r"D:\My Kit\Atollo.exe"), true),
        r#""D:\My Kit\Atollo.exe" secret github"#
    );
    assert_eq!(
        github_command(Path::new("/media/u/My Kit/Atollo.AppImage"), false),
        "'/media/u/My Kit/Atollo.AppImage' secret github"
    );
}

/// Registered when `sbx` has none; again over Atollo's own when the kit
/// moved; the user's own (none noted for this PC) never touched.
#[test]
fn only_atollos_own_secret_is_registered_or_replaced() {
    let mine = Noted {
        computer: "PC1".into(),
        command: "\"D:\\kit\\Atollo.exe\" secret github".into(),
    };
    assert_eq!(step(false, None, &mine.command), Step::Register);
    assert_eq!(step(true, Some(&mine), &mine.command), Step::Keep);
    assert_eq!(
        step(true, Some(&mine), "\"E:\\kit\\Atollo.exe\" secret github"),
        Step::Replace
    );
    assert_eq!(
        step(true, None, &mine.command),
        Step::Keep,
        "the user's own"
    );
}

/// `sbx secret ls` of sbx 0.49.2, read for a global secret.
#[test]
fn a_global_secret_is_read_from_the_list() {
    let ls = "SCOPE      TYPE      NAME   SECRET\n(global)   service   github   (stored)\nshop       service   openai   (stored)\n";
    assert!(listed(ls, "github"));
    assert!(!listed(ls, "openai"), "scoped to one sandbox, not global");
    assert!(!listed("No secrets found\n", "github"));
}

/// Noted per PC, one entry each, kept across reads.
#[test]
fn the_commands_are_noted_per_pc() {
    let (_tmp, kit) = crate::test_support::prepared_kit("isolation-secrets");
    assert!(noted(&kit).is_empty());
    note(&kit, "github", "PC1", "a").unwrap();
    note(&kit, "github", "PC2", "b").unwrap();
    note(&kit, "github", "PC1", "c").unwrap();
    let all = noted(&kit);
    let list = &all["github"];
    assert_eq!(list.len(), 2);
    assert!(list.contains(&Noted {
        computer: "PC1".into(),
        command: "c".into()
    }));
}
