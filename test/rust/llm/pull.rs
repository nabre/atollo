use super::*;

use crate::kit::tests::TempDir;
use crate::kit::Arch;

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

/// The same files under another id resume the model already there; an id
/// taken by other files is refused.
#[test]
fn an_entry_is_reused_not_doubled() {
    let tmp = TempDir::new("llm-entry");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let first = entry_for(&kit, &plan(&[1])).unwrap();
    let mut again = plan(&[1]);
    again.id = "other".into();
    assert_eq!(entry_for(&kit, &again).unwrap().id, first.id);
    let mut clash = plan(&[1, 2]);
    clash.repo = "o/s".into();
    assert!(entry_for(&kit, &clash).is_err());
}
