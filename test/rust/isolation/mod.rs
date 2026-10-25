use super::*;

#[test]
fn no_blocker_is_the_vm() {
    assert_eq!(Status::of(Vec::new(), None).level, Level::Vm);
    assert_eq!(
        Status::of(vec![Blocker::SbxMissing], None).level,
        Level::Env
    );
}

#[test]
fn sbx_signed_out_is_told_apart_from_other_errors() {
    assert_eq!(answer(true, ""), Ok(()));
    assert_eq!(
        answer(
            false,
            "Error: you are not logged in. Run `sbx login` first.\n"
        ),
        Err(Blocker::SbxSignedOut)
    );
    assert_eq!(
        answer(false, "starting\nError: daemon not running\n\n"),
        Err(Blocker::SbxFailed {
            message: "Error: daemon not running".into()
        })
    );
    assert_eq!(
        answer(false, ""),
        Err(Blocker::SbxFailed {
            message: "sbx".into()
        })
    );
}

#[test]
fn sbx_is_looked_for_in_the_path_then_where_installers_put_it() {
    let host = Host::from_pairs([
        ("Path".into(), r#"C:\Tools;"C:\Program Files\X";;"#.into()),
        ("LOCALAPPDATA".into(), r"C:\Users\u\AppData\Local".into()),
    ]);
    let dirs = sbx_dirs(Os::Win32, &host);
    assert_eq!(dirs[0], PathBuf::from(r"C:\Tools"));
    assert_eq!(dirs[1], PathBuf::from(r"C:\Program Files\X"));
    let winget = PathBuf::from(r"C:\Users\u\AppData\Local")
        .join("Microsoft")
        .join("WinGet")
        .join("Links");
    assert!(dirs.contains(&winget));
    let unix = Host::from_pairs([("PATH".into(), "/usr/bin:/home/u/bin".into())]);
    let dirs = sbx_dirs(Os::Linux, &unix);
    assert_eq!(
        dirs[..2],
        [PathBuf::from("/usr/bin"), PathBuf::from("/home/u/bin")]
    );
    assert_eq!(
        dirs.iter().filter(|d| **d == Path::new("/usr/bin")).count(),
        1
    );
    assert!(dirs.contains(&PathBuf::from("/opt/homebrew/bin")));
}

#[test]
fn an_unsupported_system_is_the_only_reason_given() {
    assert!(Blocker::Processor.unsupported());
    assert!(!Blocker::HypervisorOff.unsupported());
    assert!(!Blocker::SbxMissing.unsupported());
}

/// The blockers reach the interface by `code`, with their details.
#[test]
fn blockers_serialize_with_their_code() {
    let json = serde_json::to_value(Status::of(
        vec![Blocker::OldWindows { build: 19045 }, Blocker::SbxSignedOut],
        Some("sbx".into()),
    ))
    .unwrap();
    assert_eq!(json["level"], "env");
    assert_eq!(json["blockers"][0]["code"], "old_windows");
    assert_eq!(json["blockers"][0]["build"], 19045);
    assert_eq!(json["blockers"][1]["code"], "sbx_signed_out");
    assert_eq!(json["sbx"], "sbx");
}
