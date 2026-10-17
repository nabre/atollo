use super::*;
use crate::kit::{tests::TempDir, Arch, Target};

#[test]
fn spellings_and_boundaries() {
    let r = Replacer::new(Os::Win32, r"E:\kit", r"F:\Atollo");
    let text = br#"a E:\kit\x "E:\\kit\\y" e:/kit/z file:///e%3A/kit/w E:\kit2\no E:\kitty"#;
    let out = String::from_utf8(r.replace(text).unwrap()).unwrap();
    assert!(out.contains(r"F:\Atollo\x"), "{out}");
    assert!(out.contains(r#""F:\\Atollo\\y""#), "{out}");
    assert!(out.contains("f:/Atollo/z"), "drive case kept: {out}");
    assert!(out.contains("file:///f%3A/Dev%20Kit/w"), "{out}");
    assert!(
        out.contains(r"E:\kit2\no") && out.contains(r"E:\kitty"),
        "whole names only: {out}"
    );
    assert!(r.replace(b"nothing here").is_none());
}

#[test]
fn windows_ignores_case_unix_does_not() {
    let win = Replacer::new(Os::Win32, r"E:\Kit", r"F:\kit");
    let out = String::from_utf8(win.replace(br"e:\KIT\a").unwrap()).unwrap();
    assert_eq!(out, r"f:\kit\a", "same folder on Windows");

    for os in [Os::Linux, Os::Darwin] {
        let r = Replacer::new(os, "/media/u/Kit", "/media/u/kit");
        let out = String::from_utf8(r.replace(b"/media/u/Kit/a /media/u/KIT/b").unwrap()).unwrap();
        assert_eq!(
            out, "/media/u/kit/a /media/u/KIT/b",
            "{os:?}: only the exact spelling"
        );
        assert!(r.replace(b"/media/u/kit/a").is_none(), "{os:?}");
    }
}

#[test]
fn unix_paths_keep_their_first_letter() {
    // PORT-V4-02: the first letter after `/` is a folder name, not a drive.
    for (from, to) in [
        ("/home/u/kit", "/Home/u/kit"),
        ("/home/u/kit", "/Users/u/kit"),
        ("/Volumes/Kit", "/media/u/Kit"),
        // Looks like a drive letter, but on Unix the match is exact: the
        // new spelling is written as given.
        ("/a:x/kit", "/B:y/kit"),
    ] {
        let r = Replacer::new(Os::Linux, from, to);
        let text = format!("{from}/a \"{from}/b\" file://{}/c", uri_path(from));
        let out = String::from_utf8(r.replace(text.as_bytes()).unwrap()).unwrap();
        let want = format!("{to}/a \"{to}/b\" file://{}/c", uri_path(to));
        assert_eq!(out, want);
    }
}

#[test]
fn drive_letter_found_only_with_a_colon() {
    assert!(has_drive_letter(br"E:\kit", 0));
    assert!(has_drive_letter(b"/e%3A/kit", 1));
    assert!(has_drive_letter(b"/e%3a/kit", 1));
    assert!(!has_drive_letter(b"/home/u", 1));
    assert!(!has_drive_letter(br"\\server\share", 0));
    assert!(!has_drive_letter(b"E", 0));
}

#[test]
fn toml_tables_deduped_by_the_rules_of_the_os() {
    let text = "[projects.'/a/Kit']\nx = 1\n[projects.'/a/kit']\nx = 2\n";
    assert_eq!(dedupe_toml(text, false), text, "two folders on Unix");
    assert_eq!(
        dedupe_toml(text, true),
        "[projects.'/a/Kit']\nx = 1\n",
        "one folder on Windows"
    );
}

#[test]
fn relocates_agents_and_claude_projects() {
    let tmp = TempDir::new("relocate");
    let kit = Kit::at(tmp.0.join("new kit"), Arch::X64);
    kit.prepare().unwrap();
    let p = Paths::of(&kit);
    let old = r"E:\old kit";
    fs::write(kit.old_root_file(), old).unwrap();
    fs::create_dir_all(&p.claude).unwrap();
    let old_fwd = old.replace('\\', "/");
    fs::write(
        p.claude_json(),
        serde_json::json!({ "projects": { format!("{old_fwd}/projects/a"): {"history": [1]}, format!("{}/projects/a", kit.root().display().to_string().replace('\\', "/")): {"hasTrustDialogAccepted": true} } }).to_string(),
    )
    .unwrap();
    let old_name = claude_folder_name(&format!(r"{old}\projects\a"));
    fs::create_dir_all(p.claude.join("projects").join(&old_name)).unwrap();
    fs::write(
        p.claude.join("projects").join(&old_name).join("s.jsonl"),
        "x",
    )
    .unwrap();
    fs::create_dir_all(&p.codex).unwrap();
    // Codex writes the folder in its own case: on Windows that is the
    // same folder, elsewhere the spelling must match.
    let codex_old = if cfg!(windows) {
        old.to_lowercase()
    } else {
        old.to_owned()
    };
    fs::write(
        p.codex.join("config.toml"),
        format!("[projects.'{codex_old}\\projects\\a']\ntrust_level = \"trusted\"\n"),
    )
    .unwrap();

    let mut report = Report::default();
    relocate(&kit, &mut report).unwrap();
    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    let cfg: Value = serde_json::from_str(&fs::read_to_string(p.claude_json()).unwrap()).unwrap();
    let new_key = format!(
        "{}/projects/a",
        kit.root().display().to_string().replace('\\', "/")
    );
    assert_eq!(cfg["projects"][&new_key]["history"][0], 1, "merged");
    assert_eq!(
        cfg["projects"][&new_key]["hasTrustDialogAccepted"], true,
        "new place wins"
    );
    assert!(p
        .claude
        .join("projects")
        .join(claude_folder_name(&format!(
            r"{}\projects\a",
            kit.root().display()
        )))
        .join("s.jsonl")
        .is_file());
    let toml = fs::read_to_string(p.codex.join("config.toml")).unwrap();
    assert!(toml
        .to_lowercase()
        .contains(&kit.root().display().to_string().to_lowercase()));
    assert!(!kit.old_root_file().exists(), "done");
}

/// A Linux kit renamed only in case (`/media/u/Kit` -> `/media/u/kit`) has
/// moved: its files are updated. Run on every host with a Linux target,
/// since the decision only compares strings.
#[test]
fn case_only_rename_is_a_move_on_unix() {
    let tmp = TempDir::new("relocate-case");
    let kit = Kit::at_target(tmp.0.join("kit"), Target::new(Os::Linux, Arch::X64));
    let p = Paths::of(&kit);
    fs::create_dir_all(kit.app_dir()).unwrap();
    fs::create_dir_all(&p.claude).unwrap();
    let new = kit.root().display().to_string();
    let old = tmp.0.join("Kit").display().to_string();
    fs::write(kit.old_root_file(), &old).unwrap();
    let fwd = |s: &str| s.replace('\\', "/");
    fs::write(
        p.claude.join("settings.json"),
        format!("{{\"a\": \"{}/x\"}}", fwd(&old)),
    )
    .unwrap();

    let mut report = Report::default();
    relocate(&kit, &mut report).unwrap();
    let text = fs::read_to_string(p.claude.join("settings.json")).unwrap();
    assert!(text.contains(&format!("{}/x", fwd(&new))), "{text}");
    assert!(!kit.old_root_file().exists(), "done");

    // The same rename on Windows is no move: nothing is touched.
    let wkit = Kit::at_target(tmp.0.join("wkit"), Target::new(Os::Win32, Arch::X64));
    let wp = Paths::of(&wkit);
    fs::create_dir_all(wkit.app_dir()).unwrap();
    fs::create_dir_all(&wp.claude).unwrap();
    let wold = tmp.0.join("WKIT").display().to_string();
    fs::write(wkit.old_root_file(), &wold).unwrap();
    let before = format!("{{\"a\": \"{}/x\"}}", fwd(&wold));
    fs::write(wp.claude.join("settings.json"), &before).unwrap();
    relocate(&wkit, &mut Report::default()).unwrap();
    assert_eq!(
        fs::read_to_string(wp.claude.join("settings.json")).unwrap(),
        before
    );
}

/// Claude's per-folder data renamed only in case. On a file system that
/// ignores case (macOS, exFAT) the new name already "exists": it is the
/// same folder, renamed in place, never merged into itself.
#[test]
#[cfg_attr(
    windows,
    ignore = "a case-only move is never one on Windows (kit::same_root)"
)]
fn claude_folder_renamed_only_in_case() {
    let tmp = TempDir::new("relocate-claude-case");
    let kit = Kit::at(tmp.0.join("kit"), Arch::X64);
    let p = Paths::of(&kit);
    let from = tmp.0.join("Kit").display().to_string();
    let to = kit.root().display().to_string();
    let old = p
        .claude
        .join("projects")
        .join(claude_folder_name(&format!("{from}/a")));
    fs::create_dir_all(&old).unwrap();
    fs::write(old.join("s.jsonl"), "x").unwrap();
    let r = Replacer::new(kit.os(), &from, &to);
    let mut m = Moved::default();
    relocate_claude_projects(&p, &r, &from, &to, &mut m);
    assert!(m.failed.is_empty(), "{:?}", m.failed);
    let names: Vec<String> = fs::read_dir(p.claude.join("projects"))
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, [claude_folder_name(&format!("{to}/a"))]);
    let new = p.claude.join("projects").join(&names[0]);
    assert_eq!(fs::read_to_string(new.join("s.jsonl")).unwrap(), "x");
}

#[test]
fn programs_with_other_length_are_reported() {
    let tmp = TempDir::new("relocate-bin");
    let f = tmp.0.join("tool.exe");
    fs::write(&f, b"MZ\0\0E:\\kit\\python.exe\0").unwrap();
    let mut m = Moved::default();
    relocate_file(&f, &Replacer::new(Os::Win32, r"E:\kit", r"F:\kit2"), &mut m);
    assert_eq!(m.programs, vec![f.clone()]);
    relocate_file(&f, &Replacer::new(Os::Win32, r"E:\kit", r"F:\kit"), &mut m);
    assert!(
        fs::read(&f).unwrap().windows(6).any(|w| w == b"F:\\kit"),
        "same length: changed"
    );
}

#[test]
fn length_change_found_by_the_rules_of_the_os() {
    let bin = b"\0/opt/KIT/bin\0";
    assert!(Replacer::new(Os::Win32, "/opt/kit", "/opt/kit2").has_length_change(bin));
    assert!(!Replacer::new(Os::Linux, "/opt/kit", "/opt/kit2").has_length_change(bin));
    assert!(Replacer::new(Os::Linux, "/opt/KIT", "/opt/kit2").has_length_change(bin));
}
