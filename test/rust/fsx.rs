use super::*;
use std::collections::BTreeMap;

fn temp(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("atollo-fsx-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

#[test]
fn missing_and_empty_are_nothing_saved() {
    let dir = temp("empty");
    let file = dir.join("a.json");
    assert!(try_read_json::<BTreeMap<String, u32>>(&file)
        .unwrap()
        .is_none());
    fs::create_dir_all(&dir).unwrap();
    fs::write(&file, " \r\n").unwrap();
    assert!(try_read_json::<BTreeMap<String, u32>>(&file)
        .unwrap()
        .is_none());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn bom_and_lossy_text_are_read() {
    let dir = temp("bom");
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("a.json");
    fs::write(&file, b"\xef\xbb\xbf{\"a\":1}").unwrap();
    let v: BTreeMap<String, u32> = read_json(&file).unwrap();
    assert_eq!(v["a"], 1);
    fs::write(&file, b"\xef\xbb\xbfx\xffy").unwrap();
    assert_eq!(read_text(&file).unwrap(), "x\u{fffd}y");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn damaged_file_is_a_problem_with_its_path() {
    let dir = temp("bad");
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("a.json");
    fs::write(&file, "{nope").unwrap();
    let (v, problem): (BTreeMap<String, u32>, _) = load_json(&file);
    assert!(v.is_empty());
    assert!(problem.unwrap().contains("a.json"));
    assert!(read_json::<BTreeMap<String, u32>>(&file).is_none());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn write_json_creates_the_folder_and_leaves_no_temporary() {
    let dir = temp("write");
    let file = dir.join("sub").join("a.json");
    write_json(&file, &BTreeMap::from([("a", 2)])).unwrap();
    let v: BTreeMap<String, u32> = read_json(&file).unwrap();
    assert_eq!(v["a"], 2);
    let names: Vec<_> = fs::read_dir(dir.join("sub")).unwrap().flatten().collect();
    assert_eq!(names.len(), 1);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn write_atomic_concurrent_threads_do_not_collide_on_tmp() {
    let dir = temp("concurrent");
    fs::create_dir_all(&dir).unwrap();
    let handles: Vec<_> = (0..8u32)
        .map(|i| {
            let dir = dir.clone();
            std::thread::spawn(move || {
                // Threads share the same stem (`cfg.json`, `cfg.toml`, etc.) and target files.
                let ext = if i % 2 == 0 { "json" } else { "toml" };
                let file = dir.join(format!("cfg-{}.{ext}", i % 4));
                for step in 0..10u32 {
                    let payload = format!("{{\"thread\":{i},\"step\":{step}}}");
                    write_atomic(&file, payload.as_bytes()).unwrap();
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    let names: Vec<String> = fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        names.iter().all(|n| !n.contains(".tmp-")),
        "leftover tmp files: {names:?}"
    );
    assert_eq!(names.len(), 4);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn join_rel_normalizes_both_slashes_and_skips_dot() {
    let base = Path::new("root");
    let expected = base.join("a").join("b").join("c.txt");
    assert_eq!(join_rel(base, "a/b/c.txt"), expected);
    assert_eq!(join_rel(base, r"a\b\c.txt"), expected);
    assert_eq!(join_rel(base, "./a//b/./c.txt"), expected);
}

/// Many threads saving the same file at once: on Windows two renames over
/// one target answer "access denied" to one of them (seen 7 times in 4800
/// writes before `rename_over`); every write must succeed.
#[test]
fn write_atomic_same_file_from_many_threads_never_fails() {
    let dir = temp("same-file");
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("one.json");
    let handles: Vec<_> = (0..16u32)
        .map(|i| {
            let file = file.clone();
            std::thread::spawn(move || {
                let mut errs = Vec::new();
                for step in 0..150u32 {
                    if let Err(e) = write_atomic(&file, format!("{i}-{step}").as_bytes()) {
                        errs.push(format!("{:?} {:?}", e.kind(), e.raw_os_error()));
                    }
                }
                errs
            })
        })
        .collect();
    let errs: Vec<String> = handles
        .into_iter()
        .flat_map(|h| h.join().unwrap())
        .collect();
    let names: Vec<_> = fs::read_dir(&dir).unwrap().flatten().collect();
    let _ = fs::remove_dir_all(&dir);
    assert!(
        errs.is_empty(),
        "{} errors: {:?}",
        errs.len(),
        &errs[..errs.len().min(5)]
    );
    assert_eq!(names.len(), 1, "no temporary left");
}
