use super::super::tests::{kept, sha};
use super::*;
use crate::kit::tests::TempDir;

const SHA_A: &str = "f027973cd4f9396ce907f95a7853a33bc9a38127ff473f8bf29eb129a14ce841";

#[test]
fn explicit_list_of_conda_forge_with_sha256() {
    let text = format!(
        "# platform: linux-64\n@EXPLICIT\n\n{CHANNEL}linux-64/sqlite-3.54.1-h6e79ffa_100.conda#{SHA_A}\n{CHANNEL}noarch/tzdata-2026a-h0c530f3_0.conda#sha256:{}\n",
        SHA_A.to_uppercase()
    );
    let list = parse_explicit(&text).unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].file, "sqlite-3.54.1-h6e79ffa_100.conda");
    assert_eq!(list[0].subdir(), Some("linux-64"));
    assert_eq!(list[1].subdir(), Some("noarch"));
    assert_eq!(list[1].sha256, SHA_A, "hashes are kept lowercase");
    // What the kit writes, it reads back the same.
    assert_eq!(
        parse_explicit(&explicit_text(&list, "linux-64")).unwrap(),
        list
    );
}

#[test]
fn explicit_list_refuses_md5_other_channels_and_paths() {
    let ok = format!("{CHANNEL}linux-64/a-1-0.conda#{SHA_A}");
    for bad in [
        // micromamba env export's MD5: not what the archives are checked against.
        format!("@EXPLICIT\n{CHANNEL}linux-64/a-1-0.conda#d904c7ddd95e026254b4b80808ee9e12"),
        format!("@EXPLICIT\n{CHANNEL}linux-64/a-1-0.conda"),
        ok.clone(),
        format!("@EXPLICIT\nhttps://example.org/conda-forge/linux-64/a-1-0.conda#{SHA_A}"),
        format!("@EXPLICIT\nhttp://conda.anaconda.org/conda-forge/linux-64/a-1-0.conda#{SHA_A}"),
        format!("@EXPLICIT\nfile:///tmp/a-1-0.conda#{SHA_A}"),
        format!("@EXPLICIT\n{CHANNEL}linux-64/../../a-1-0.conda#{SHA_A}"),
        format!("@EXPLICIT\n{CHANNEL}linux-64/sub/a-1-0.conda#{SHA_A}"),
        format!("@EXPLICIT\n{CHANNEL}linux-64/.hidden.conda#{SHA_A}"),
        format!("@EXPLICIT\n{CHANNEL}linux-64/a-1-0.zip#{SHA_A}"),
        "@EXPLICIT\n".to_owned(),
        String::new(),
    ] {
        assert!(parse_explicit(&bad).is_err(), "accepted: {bad}");
    }
    assert!(parse_explicit(&format!("@EXPLICIT\n{ok}")).is_ok());
}

#[test]
fn offline_only_with_every_archive_and_its_hash() {
    let tmp = TempDir::new("conda-check");
    let pkgs = tmp.0.join("pkgs");
    let (list, record) = kept(
        &pkgs,
        "linux-64",
        &[("a-1-0.conda", b"aaa"), ("b-2-0.tar.bz2", b"bbbb")],
    );
    let ok = check(Some(&list), Some(&record), "linux-64", &pkgs).unwrap();
    assert_eq!(ok.len(), 2);

    // No list (installed before D-118, freed): the network.
    assert_eq!(
        check(None, Some(&record), "linux-64", &pkgs),
        Err(Fallback::NoList)
    );
    assert_eq!(
        check(Some(&list), None, "linux-64", &pkgs),
        Err(Fallback::NoList)
    );
    assert!(matches!(
        check(Some("garbage"), Some(&record), "linux-64", &pkgs),
        Err(Fallback::Unreadable(_))
    ));
    assert!(matches!(
        check(Some(&list), Some("{"), "linux-64", &pkgs),
        Err(Fallback::Unreadable(_))
    ));

    // Same size, other bytes: never used.
    fs::write(pkgs.join("b-2-0.tar.bz2"), b"BBBB").unwrap();
    assert_eq!(
        check(Some(&list), Some(&record), "linux-64", &pkgs),
        Err(Fallback::Changed("b-2-0.tar.bz2".into()))
    );
    // A partial copy, then none.
    fs::write(pkgs.join("b-2-0.tar.bz2"), b"bb").unwrap();
    assert_eq!(
        check(Some(&list), Some(&record), "linux-64", &pkgs),
        Err(Fallback::Missing("b-2-0.tar.bz2".into()))
    );
    fs::remove_file(pkgs.join("b-2-0.tar.bz2")).unwrap();
    assert_eq!(
        check(Some(&list), Some(&record), "linux-64", &pkgs),
        Err(Fallback::Missing("b-2-0.tar.bz2".into()))
    );
}

#[test]
fn a_record_changed_alone_lets_nothing_through() {
    let tmp = TempDir::new("conda-record");
    let pkgs = tmp.0.join("pkgs");
    let (list, _) = kept(&pkgs, "linux-64", &[("a-1-0.conda", b"aaa")]);
    // The archive replaced and the record written for it: the list, from
    // conda-forge's metadata, still says the old hash.
    fs::write(pkgs.join("a-1-0.conda"), b"evil").unwrap();
    let forged = Record {
        version: "1".into(),
        target: "linux-x64".into(),
        created: 1,
        micromamba: None,
        packages: vec![Package {
            file: "a-1-0.conda".into(),
            sha256: sha(b"evil"),
            size: 4,
        }],
    };
    let forged = serde_json::to_string(&forged).unwrap();
    assert_eq!(
        check(Some(&list), Some(&forged), "linux-64", &pkgs),
        Err(Fallback::Disagree("a-1-0.conda".into()))
    );
    // One package fewer in the record.
    let (list2, _) = kept(
        &pkgs,
        "linux-64",
        &[("a-1-0.conda", b"aaa"), ("c-1-0.conda", b"c")],
    );
    let (_, one) = kept(&tmp.0.join("other"), "linux-64", &[("a-1-0.conda", b"aaa")]);
    assert!(matches!(
        check(Some(&list2), Some(&one), "linux-64", &pkgs),
        Err(Fallback::Disagree(_))
    ));
}

#[test]
fn never_an_archive_of_another_target() {
    let tmp = TempDir::new("conda-target");
    let pkgs = tmp.0.join("pkgs");
    let (list, record) = kept(&pkgs, "osx-arm64", &[("a-1-0.conda", b"aaa")]);
    assert_eq!(
        check(Some(&list), Some(&record), "linux-64", &pkgs),
        Err(Fallback::OtherTarget("a-1-0.conda".into()))
    );
    assert!(check(Some(&list), Some(&record), "osx-arm64", &pkgs).is_ok());
    let (noarch, record) = kept(&pkgs, "noarch", &[("t-1-0.conda", b"t")]);
    assert!(check(Some(&noarch), Some(&record), "linux-aarch64", &pkgs).is_ok());
}

#[test]
fn file_urls_of_a_kit_folder_with_spaces() {
    assert_eq!(
        file_url(Path::new("/media/u/My Kit (1)/tmp/kept/a-1-0.conda")),
        "file:///media/u/My%20Kit%20%281%29/tmp/kept/a-1-0.conda"
    );
    assert_eq!(
        file_url(Path::new(r"C:\Temp\Atollo\a b\x+1.conda")),
        "file:///C:/Temp/Atollo/a%20b/x%2B1.conda"
    );
}
