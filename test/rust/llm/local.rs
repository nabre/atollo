use super::*;

use crate::kit::tests::TempDir;
use crate::kit::Arch;

fn kit() -> (TempDir, Kit) {
    let tmp = TempDir::new("llm-local");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    (tmp, kit)
}

fn file(name: &str, bytes: u64) -> ModelFile {
    ModelFile {
        name: name.into(),
        sha256: "0".repeat(64),
        bytes,
    }
}

#[test]
fn parts_and_stems() {
    assert_eq!(part_of("m-q4_k_m-00001-of-00002"), Some((1, 2)));
    assert_eq!(part_of("m-q4_k_m"), None);
    assert_eq!(part_of("m-1-of-2"), None);
    assert_eq!(model_stem("m-Q4_K_M-00002-of-00003.gguf"), "m-Q4_K_M");
    assert_eq!(model_stem("gpt-oss-20b-MXFP4.gguf"), "gpt-oss-20b-MXFP4");
    assert!(is_mmproj("mmproj-F16.gguf"));
    assert_eq!(folder_of("unsloth/Qwen3-GGUF"), "unsloth--Qwen3-GGUF");
}

#[test]
fn repositories_of_hugging_face_only() {
    assert!(valid_repo("ggml-org/gpt-oss-20b-GGUF"));
    assert!(!valid_repo("gpt-oss"));
    assert!(!valid_repo("a/b/c"));
    assert!(!valid_repo("../x"));
    assert!(!valid_repo("a/.."));
    assert!(!valid_repo("a/b c"));
}

/// The models of a folder: those of its manifest (complete only with all
/// their files), then each file put there by hand, its parts together.
#[test]
fn a_folder_lists_its_models() {
    let (_tmp, kit) = kit();
    let dir = crate::llm::models_dir(&kit).join("Qwen--Coder-GGUF");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("coder-q4-00001-of-00002.gguf"), b"aa").unwrap();
    fs::write(dir.join("coder-q4-00002-of-00002.gguf"), b"bbb").unwrap();
    fs::write(dir.join("other-q8.gguf"), b"c").unwrap();
    fs::write(dir.join("mmproj-F16.gguf"), b"p").unwrap();
    fs::write(dir.join("half-q2.gguf.part"), b"pp").unwrap();
    write_manifest(
        &dir,
        &Manifest {
            models: vec![
                Entry {
                    id: "coder".into(),
                    name: "Coder".into(),
                    repo: "Qwen/Coder-GGUF".into(),
                    commit: "c".repeat(40),
                    files: vec![
                        file("coder-q4-00001-of-00002.gguf", 2),
                        file("coder-q4-00002-of-00002.gguf", 3),
                    ],
                    catalog: Some("coder".into()),
                    ctx: Some(8192),
                    complete: true,
                },
                Entry {
                    id: "half".into(),
                    name: "Half".into(),
                    repo: "Qwen/Coder-GGUF".into(),
                    commit: "c".repeat(40),
                    files: vec![file("half-q2.gguf", 10)],
                    catalog: None,
                    ctx: None,
                    complete: false,
                },
            ],
        },
    )
    .unwrap();
    let models = list(&kit);
    let ids: Vec<&str> = models.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(ids, ["coder", "half", "other-q8"]);
    let coder = &models[0];
    assert!(coder.complete && !coder.by_hand);
    assert_eq!(coder.bytes, 5);
    assert_eq!(coder.mmproj.as_deref(), Some("mmproj-F16.gguf"));
    let half = &models[1];
    assert!(!half.complete);
    assert_eq!((half.downloaded, half.bytes), (2, 10));
    let other = &models[2];
    assert!(other.by_hand && other.complete);
    assert_eq!(other.repo, "Qwen/Coder-GGUF");
}

/// A file put by hand next to a model of the same name gets the folder in
/// its id; the parts of a model are complete only all together.
#[test]
fn files_by_hand_get_unique_ids() {
    let (_tmp, kit) = kit();
    let root = crate::llm::models_dir(&kit);
    for d in ["a--x", "b--y"] {
        fs::create_dir_all(root.join(d)).unwrap();
        fs::write(root.join(d).join("same-q4.gguf"), b"x").unwrap();
    }
    fs::write(root.join("b--y").join("big-00001-of-00002.gguf"), b"x").unwrap();
    let models = list(&kit);
    let ids: Vec<&str> = models.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(ids, ["same-q4", "big", "same-q4-b--y"]);
    assert!(!models[1].complete, "one part of two");
}

/// Removing a model deletes its files and its part, its entry, and the
/// folder once nothing else is there.
#[test]
fn remove_cleans_the_folder() {
    let (_tmp, kit) = kit();
    let dir = crate::llm::models_dir(&kit).join("o--r");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("m.gguf"), b"x").unwrap();
    fs::write(dir.join("n.gguf.part"), b"x").unwrap();
    let entry = |id: &str, f: &str| Entry {
        id: id.into(),
        name: id.into(),
        repo: "o/r".into(),
        commit: "c".repeat(40),
        files: vec![file(f, 1)],
        catalog: None,
        ctx: None,
        complete: true,
    };
    write_manifest(
        &dir,
        &Manifest {
            models: vec![entry("m", "m.gguf"), entry("n", "n.gguf")],
        },
    )
    .unwrap();
    remove(&kit, "n").unwrap();
    assert!(!dir.join("n.gguf.part").exists());
    assert_eq!(read_manifest(&dir).unwrap().models.len(), 1);
    remove(&kit, "m").unwrap();
    assert!(!dir.exists());
    assert!(remove(&kit, "m").is_err());
}
