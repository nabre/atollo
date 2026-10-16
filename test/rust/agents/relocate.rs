use super::*;
use crate::kit::{tests::TempDir, Arch};

#[test]
fn spellings_and_boundaries() {
    let r = Replacer::new(r"E:\kit", r"F:\Atollo");
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
fn relocates_agents_and_claude_projects() {
    let tmp = TempDir::new("relocate");
    let kit = Kit::at(tmp.0.join("new kit"), Arch::X64);
    kit.prepare().unwrap();
    let p = Paths::of(&kit);
    let old = r"E:\old kit";
    fs::write(kit.app_dir().join("old-root.txt"), old).unwrap();
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
    fs::write(
        p.codex.join("config.toml"),
        format!(
            "[projects.'{}\\projects\\a']\ntrust_level = \"trusted\"\n",
            old.to_lowercase()
        ),
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
    assert!(!kit.app_dir().join("old-root.txt").exists(), "done");
}

#[test]
fn programs_with_other_length_are_reported() {
    let tmp = TempDir::new("relocate-bin");
    let f = tmp.0.join("tool.exe");
    fs::write(&f, b"MZ\0\0E:\\kit\\python.exe\0").unwrap();
    let mut m = Moved::default();
    relocate_file(&f, &Replacer::new(r"E:\kit", r"F:\kit2"), &mut m);
    assert_eq!(m.programs, vec![f.clone()]);
    relocate_file(&f, &Replacer::new(r"E:\kit", r"F:\kit"), &mut m);
    assert!(
        fs::read(&f).unwrap().windows(6).any(|w| w == b"F:\\kit"),
        "same length: changed"
    );
}
