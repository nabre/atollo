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
    let file = dir.join(r"sub\a.json");
    write_json(&file, &BTreeMap::from([("a", 2)])).unwrap();
    let v: BTreeMap<String, u32> = read_json(&file).unwrap();
    assert_eq!(v["a"], 2);
    let names: Vec<_> = fs::read_dir(dir.join("sub")).unwrap().flatten().collect();
    assert_eq!(names.len(), 1);
    let _ = fs::remove_dir_all(&dir);
}
