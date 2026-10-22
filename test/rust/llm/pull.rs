use super::*;

use crate::kit::tests::TempDir;
use crate::kit::Arch;
use crate::test_support::prepared_kit;

fn plan(sizes: &[u64]) -> Plan {
    Plan {
        id: "m".into(),
        name: "M".into(),
        repo: "o/r".into(),
        commit: "c".repeat(40),
        files: sizes
            .iter()
            .enumerate()
            .map(|(i, b)| ModelFile {
                name: format!("Q/m-{i}.gguf"),
                sha256: "0".repeat(64),
                bytes: *b,
            })
            .collect(),
        catalog: None,
        ctx: None,
    }
}

/// FAT32 refuses a file over 4 GiB before anything is downloaded; the
/// parts of a model under it pass. The free space counts what is already
/// downloaded.
#[test]
fn the_drive_is_checked_first() {
    let fat = Some(u64::from(u32::MAX));
    let big = plan(&[5_000_000_000]);
    assert_eq!(
        room_for(&big, "FAT32", fat, u64::MAX, 0),
        Some(Room::TooLargeForFs {
            file: "m-0.gguf".into(),
            bytes: 5_000_000_000,
            fs: "FAT32".into()
        })
    );
    assert_eq!(room_for(&big, "exFAT", None, u64::MAX, 0), None);
    let parts = plan(&[3_000_000_000, 2_000_000_000]);
    assert_eq!(room_for(&parts, "FAT32", fat, u64::MAX, 0), None);
    assert!(matches!(
        room_for(&parts, "NTFS", None, 1_000_000_000, 0),
        Some(Room::NoSpace { .. })
    ));
    assert_eq!(
        room_for(&parts, "NTFS", None, 1_000_000_000, 4_600_000_000),
        None
    );
}

#[test]
fn messages_say_what_to_do() {
    let m = Room::TooLargeForFs {
        file: "m.gguf".into(),
        bytes: 12_100_000_000,
        fs: "FAT32".into(),
    }
    .message("it");
    assert!(m.contains("12.1 GB") && m.contains("exFAT"), "{m}");
    let m = Room::NoSpace {
        need: 2_000_000_000,
        free: 1_000_000_000,
    }
    .message("en");
    assert!(m.contains("2.0") && m.contains("1.0"), "{m}");
}

/// A plan names its files on disk by their last part, in the folder of
/// its repository; a catalog model keeps its id and context.
#[test]
fn plans_of_the_catalog_and_of_a_repository() {
    let p = plan(&[1]);
    assert_eq!(p.local_files()[0].name, "m-0.gguf");
    let tmp = TempDir::new("llm-plan");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    assert!(p.dir(&kit).ends_with("o--r"));
    let m = crate::llm::catalog::find("gpt-oss-20b").unwrap();
    let p = Plan::from_catalog(m);
    assert_eq!(
        (p.id.as_str(), p.catalog.as_deref(), p.ctx),
        ("gpt-oss-20b", Some("gpt-oss-20b"), Some(m.ctx))
    );
    let info = RepoInfo {
        repo: "unsloth/Qwen3-8B-GGUF".into(),
        commit: "c".repeat(40),
        license: None,
        choices: Vec::new(),
    };
    let choice = Choice {
        id: "qwen3-8b-q4_k_m".into(),
        label: "Qwen3-8B-Q4_K_M".into(),
        quant: Some("Q4_K_M".into()),
        files: Vec::new(),
        bytes: 0,
    };
    assert_eq!(Plan::from_choice(&info, &choice).name, "Qwen3-8B Q4_K_M");
}

/// The same files under another id resume the model already there; the
/// same id from another publisher gets the publisher's name; an id taken
/// by other files of the same repository is refused.
#[test]
fn an_entry_is_reused_not_doubled() {
    let (_tmp, kit) = prepared_kit("llm-entry");
    let first = entry_for(&kit, &plan(&[1]), "en").unwrap();
    let mut again = plan(&[1]);
    again.id = "other".into();
    assert_eq!(entry_for(&kit, &again, "en").unwrap().id, first.id);
    let mut elsewhere = plan(&[1]);
    elsewhere.repo = "bartowski/r".into();
    assert_eq!(entry_for(&kit, &elsewhere, "en").unwrap().id, "m-bartowski");
    let mut clash = plan(&[1, 2]);
    clash.files[0].name = "Q/x.gguf".into();
    let refused = entry_for(&kit, &clash, "de").unwrap_err().to_string();
    assert!(refused.contains("bereits im Kit"), "{refused}");
}

/// A download paused at one commit and resumed after the repository got a
/// new one does not join the old bytes to the new file: the part of a file
/// that changed is dropped, and the entry takes the new commit. A part of
/// the same commit is kept.
#[test]
fn a_part_of_an_older_commit_starts_again() {
    let (_tmp, kit) = prepared_kit("llm-commit");
    let old = plan(&[10]);
    let dir = old.dir(&kit);
    entry_for(&kit, &old, "en").unwrap();
    let part = crate::net::resume::part_of(&dir.join("m-0.gguf"));
    std::fs::write(&part, b"12345").unwrap();
    entry_for(&kit, &old, "en").unwrap();
    assert!(part.exists());
    let mut new = plan(&[10]);
    new.commit = "d".repeat(40);
    new.files[0].sha256 = "1".repeat(64);
    let entry = entry_for(&kit, &new, "en").unwrap();
    assert!(!part.exists());
    assert_eq!(entry.commit, new.commit);
    assert_eq!(entry.files[0].sha256, new.files[0].sha256);
    let manifest = local::read_manifest(&dir).unwrap();
    assert_eq!(manifest.models.len(), 1);
    assert_eq!(manifest.models[0].commit, new.commit);
}

/// One download per repository folder, also seen from another process
/// (its lock file with a live pid); Cancel of every download; a removal
/// refused meanwhile.
#[test]
fn one_download_per_folder() {
    let (_tmp, kit) = prepared_kit("llm-busy");
    let dir = crate::llm::models_dir(&kit).join("o--busy");
    let flag = Arc::new(AtomicBool::new(false));
    let hold = Pulling::take(&dir, "o--busy", "m", Arc::clone(&flag), "en").unwrap();
    assert!(busy(&kit, "o--busy"));
    assert!(pulling().contains(&"o--busy/m".to_owned()));
    assert!(Pulling::take(&dir, "o--busy", "n", Arc::new(AtomicBool::new(false)), "en").is_err());
    // The refusal reaches the page: in the interface's language.
    let refused = Pulling::take(&dir, "o--busy", "n", Arc::new(AtomicBool::new(false)), "fr")
        .err()
        .unwrap();
    assert!(refused.to_string().contains("déjà"), "{refused}");
    cancel("o--busy/m");
    assert!(flag.load(std::sync::atomic::Ordering::Relaxed));
    drop(hold);
    assert!(!busy(&kit, "o--busy"));
    assert!(!dir.join(LOCK).exists());
    // A lock left by a process that is gone does not hold the folder.
    std::fs::write(dir.join(LOCK), "4294967").unwrap();
    assert!(!busy(&kit, "o--busy"));
}

/// Two spellings of one folder are one folder on Windows and macOS: the
/// second download is refused, and Cancel reaches the first.
#[cfg(any(windows, target_os = "macos"))]
#[test]
fn a_folder_is_one_whatever_its_case() {
    let (_tmp, kit) = prepared_kit("llm-case");
    let dir = crate::llm::models_dir(&kit).join("O--Case");
    let flag = Arc::new(AtomicBool::new(false));
    let hold = Pulling::take(&dir, "O--Case", "m", Arc::clone(&flag), "en").unwrap();
    let other = crate::llm::models_dir(&kit).join("o--case");
    assert!(Pulling::take(
        &other,
        "o--case",
        "m",
        Arc::new(AtomicBool::new(false)),
        "en"
    )
    .is_err());
    assert!(busy(&kit, "o--case"));
    assert!(pulling().contains(&"O--Case/m".to_owned()));
    cancel("o--case/m");
    assert!(flag.load(std::sync::atomic::Ordering::Relaxed));
    drop(hold);
    assert!(!busy(&kit, "o--case"));
}

/// The lock is created only when it is not there: one being written (no
/// pid yet) holds the folder, a stale one is replaced, and a download lets
/// go only of its own lock.
#[test]
fn the_lock_is_taken_once() {
    let (_tmp, kit) = prepared_kit("llm-lock");
    let dir = crate::llm::models_dir(&kit).join("o--lock");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(LOCK), "").unwrap();
    assert!(busy(&kit, "o--lock"));
    assert!(!claim(&dir).unwrap());
    std::fs::write(dir.join(LOCK), "4294967").unwrap();
    let hold = Pulling::take(&dir, "o--lock", "m", Arc::new(AtomicBool::new(false)), "en").unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.join(LOCK)).unwrap(),
        std::process::id().to_string()
    );
    std::fs::write(dir.join(LOCK), "4294967").unwrap();
    drop(hold);
    assert!(dir.join(LOCK).exists());
}

/// A model whose id got the publisher's name after it is resumed by that
/// id: its choice is found by its files, and the plan keeps the id.
#[test]
fn a_renamed_model_is_resumed() {
    let (_tmp, kit) = prepared_kit("llm-renamed");
    entry_for(&kit, &plan(&[1]), "en").unwrap();
    let mut elsewhere = plan(&[1]);
    elsewhere.repo = "bartowski/r".into();
    assert_eq!(entry_for(&kit, &elsewhere, "en").unwrap().id, "m-bartowski");
    let info = RepoInfo {
        repo: "bartowski/r".into(),
        commit: "c".repeat(40),
        license: None,
        choices: vec![Choice {
            id: "m".into(),
            label: "Q/m".into(),
            quant: None,
            files: elsewhere.files.clone(),
            bytes: 1,
        }],
    };
    assert_eq!(
        Plan::of_choice(&kit, &info, "m-bartowski").unwrap().id,
        "m-bartowski"
    );
    assert_eq!(Plan::of_choice(&kit, &info, "m").unwrap().id, "m");
    assert!(Plan::of_choice(&kit, &info, "x").is_err());
}

/// A file of the right size is taken as downloaded only when the entry
/// checked it with the same checksum, or it is read now and matches; one
/// with another content is deleted, to be downloaded again.
#[test]
fn a_file_of_the_right_size_is_checked() {
    let tmp = TempDir::new("llm-there");
    let dest = tmp.0.join("m-0.gguf");
    std::fs::write(&dest, b"abc").unwrap();
    let sha = crate::net::verify::sha256_hex(&dest).unwrap();
    let f = ModelFile {
        name: "Q/m-0.gguf".into(),
        sha256: sha,
        bytes: 3,
    };
    let mut entry = Entry {
        id: "m".into(),
        name: "M".into(),
        repo: "o/r".into(),
        commit: "c".repeat(40),
        files: vec![ModelFile {
            name: "m-0.gguf".into(),
            ..f.clone()
        }],
        catalog: None,
        ctx: None,
        complete: false,
    };
    assert!(already_there(&dest, &entry, &f).unwrap());
    let other = ModelFile {
        sha256: "0".repeat(64),
        ..f.clone()
    };
    entry.complete = true;
    entry.files[0].sha256 = other.sha256.clone();
    assert!(already_there(&dest, &entry, &other).unwrap());
    entry.complete = false;
    assert!(!already_there(&dest, &entry, &other).unwrap());
    assert!(!dest.exists());
    assert!(!already_there(&dest, &entry, &f).unwrap());
}

/// A file refused to the token is a gated repository the token has no
/// access to; other failures stay as they are.
#[test]
fn a_refused_file_says_the_repository_is_gated() {
    let refused =
        anyhow::Error::new(ureq::Error::StatusCode(403)).context("GET https://huggingface.co/x");
    let e = refused_to_gated(refused, "o/r");
    assert!(matches!(e.downcast_ref::<HfError>(), Some(HfError::Gated(r)) if r == "o/r"));
    let missing = anyhow::Error::new(ureq::Error::StatusCode(404)).context("GET x");
    assert!(refused_to_gated(missing, "o/r")
        .downcast_ref::<HfError>()
        .is_none());
}
