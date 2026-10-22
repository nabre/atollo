use super::*;

use crate::kit::tests::TempDir;

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

fn kit(root: &Path, name: &str) {
    for (rel, text) in [
        ("projects/shop/index.php", name),
        ("data/app/settings.json", name),
        ("data/app/db/secrets.json", "{}"),
        ("data/db/mysql/8.4/ibdata1", "rows"),
        ("data/home/.claude/settings.json", "{}"),
        ("data/apps/win32-x64/php/8.4/php.exe", "program"),
        ("data/cache/github/x.json", "{}"),
        ("data/models/gguf/a/m.gguf", "weights"),
        ("data/app/logs/atollo.log", "log"),
        ("data/webview2/Default/x", "profile"),
    ] {
        write(root, rel, text);
    }
}

/// The kit given by ATOLLO_TEST_KIT exported to a zip next to it and put
/// back as the copy waiting: the app applies it at its next start.
#[test]
fn real_export_then_stage() {
    let Ok(root) = std::env::var("ATOLLO_TEST_KIT") else {
        eprintln!("ATOLLO_TEST_KIT not set: skipped");
        return;
    };
    let root = PathBuf::from(root);
    let out = root.parent().unwrap().join("kit-copies");
    fs::create_dir_all(&out).unwrap();
    let made = export(&root, &out, true).unwrap();
    eprintln!(
        "exported {} files, {} bytes: {}",
        made.files, made.bytes, made.path
    );
    let m = stage(&root, Path::new(&made.path)).unwrap();
    assert_eq!(m.files, made.files);
    assert!(pending(&root));
}

#[test]
fn what_a_copy_leaves_out() {
    for rel in [
        "data/apps/win32-x64/php/php.exe",
        "data/cache/x",
        "data/models/gguf/m.gguf",
        "data/tmp/x",
        "data/webview2/x",
        "data/app/logs/a.log",
        "data/app/diagnostics/d.zip",
        "data/app/conda/linux-x64/x",
    ] {
        assert!(left_out(rel), "{rel}");
    }
    for rel in [
        "projects/shop/a",
        "data/app/settings.json",
        "data/db/mysql/8.4/ibdata1",
        "data/home/.claude/x",
        "data/applications/x",
    ] {
        assert!(!left_out(rel), "{rel}");
    }
}

/// Exported to a folder and to a zip, imported into another kit: what it
/// replaces is kept aside, its programs stay, nothing of the copy's is
/// left out.
#[test]
fn export_then_import() {
    let tmp = TempDir::new("backup");
    let (a, b, out) = (tmp.0.join("a"), tmp.0.join("b"), tmp.0.join("out"));
    kit(&a, "A");
    kit(&b, "B");
    write(&b, "projects/old/readme.md", "B");
    fs::create_dir_all(&out).unwrap();

    let folder = export(&a, &out, false).unwrap();
    let zip = export(&a, &out, true).unwrap();
    assert_eq!(folder.files, 5);
    assert_eq!(zip.files, 5);
    let folder = PathBuf::from(folder.path);
    assert!(folder.join(MARKER).is_file());
    assert!(folder.join("data/db/mysql/8.4/ibdata1").is_file());
    assert!(!folder.join("data/apps").exists() && !folder.join("data/models").exists());
    // Never inside the kit itself.
    assert!(export(&a, &a.join("data"), true).is_err());

    // From the zip into kit B.
    let m = stage(&b, Path::new(&zip.path)).unwrap();
    assert_eq!(m.files, 5);
    assert!(pending(&b));
    let aside = apply_pending(&b).unwrap().unwrap();
    assert!(!pending(&b));
    assert_eq!(
        fs::read_to_string(b.join("projects/shop/index.php")).unwrap(),
        "A"
    );
    assert_eq!(
        fs::read_to_string(b.join("data/app/settings.json")).unwrap(),
        "A"
    );
    assert!(!b.join("projects/old").exists());
    // B's own programs and caches stay; what was replaced is aside.
    assert!(b.join("data/apps/win32-x64/php/8.4/php.exe").is_file());
    assert!(b.join("data/models/gguf/a/m.gguf").is_file());
    assert_eq!(
        fs::read_to_string(aside.join("projects/old/readme.md")).unwrap(),
        "B"
    );
    assert_eq!(
        fs::read_to_string(aside.join("data/app/settings.json")).unwrap(),
        "B"
    );
    assert!(apply_pending(&b).unwrap().is_none(), "applied once");

    // From the folder too; a folder without the marker is refused.
    stage(&b, &folder).unwrap();
    assert!(pending(&b));
    cancel(&b).unwrap();
    assert!(!pending(&b));
    assert!(stage(&b, &a).is_err());
    assert!(!pending(&b));
}

/// A move that fails halfway (Windows: a file of `data\db` held open, as by
/// a database server still running) puts back those done: the kit as
/// before, the copy still waiting.
#[cfg(windows)]
#[test]
fn a_failed_swap_puts_everything_back() {
    let tmp = TempDir::new("backup-undo");
    let (a, b, out) = (tmp.0.join("a"), tmp.0.join("b"), tmp.0.join("out"));
    kit(&a, "A");
    kit(&b, "B");
    fs::create_dir_all(&out).unwrap();
    let made = export(&a, &out, false).unwrap();
    stage(&b, Path::new(&made.path)).unwrap();
    let held = fs::File::open(b.join("data/db/mysql/8.4/ibdata1")).unwrap();
    assert!(apply_pending(&b).is_err());
    drop(held);
    // As before: B's files back, the copy still waiting, no folder aside.
    assert_eq!(
        fs::read_to_string(b.join("projects/shop/index.php")).unwrap(),
        "B"
    );
    assert_eq!(
        fs::read_to_string(b.join("data/app/settings.json")).unwrap(),
        "B"
    );
    assert!(pending(&b));
    assert!(!fs::read_dir(&b)
        .unwrap()
        .flatten()
        .any(|e| e.file_name().to_string_lossy().starts_with(ASIDE)));
    // Then it works.
    apply_pending(&b).unwrap().unwrap();
    assert_eq!(
        fs::read_to_string(b.join("data/app/settings.json")).unwrap(),
        "A"
    );
}

/// A zip with paths outside `projects/` and `data/`, or climbing out, is
/// applied without them.
#[test]
fn a_zip_cannot_write_elsewhere() {
    let tmp = TempDir::new("backup-zip");
    let (b, z) = (tmp.0.join("b"), tmp.0.join("evil.zip"));
    fs::create_dir_all(&b).unwrap();
    let mut w = zip::ZipWriter::new(fs::File::create(&z).unwrap());
    let o = zip::write::SimpleFileOptions::default();
    let marker = serde_json::to_string(&Manifest {
        atollo: "0".into(),
        created: 0,
        root: "x".into(),
        files: 1,
        bytes: 1,
    })
    .unwrap();
    for (name, text) in [
        (MARKER, marker.as_str()),
        ("data/../Atollo.exe", "x"),
        ("Atollo.exe", "x"),
        ("data/apps/win32-x64/x.exe", "x"),
        ("projects/ok.txt", "ok"),
    ] {
        w.start_file(name, o).unwrap();
        w.write_all(text.as_bytes()).unwrap();
    }
    w.finish().unwrap();
    stage(&b, &z).unwrap();
    apply_pending(&b).unwrap();
    assert!(b.join("projects/ok.txt").is_file());
    assert!(!b.join("Atollo.exe").exists());
    assert!(!b.join("data/apps").exists());
}
