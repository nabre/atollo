use super::*;
use crate::kit::tests::TempDir;
use crate::kit::Target;

/// A conda environment in `prefix` whose `conda-meta` has `records`
/// (package name, version, files).
fn env_with(prefix: &Path, records: &[(&str, &str, &[&str])]) {
    let meta = prefix.join("conda-meta");
    fs::create_dir_all(&meta).unwrap();
    for (name, version, files) in records {
        let record = serde_json::json!({ "name": name, "version": version, "files": files });
        fs::write(
            meta.join(format!("{name}-{version}-h0_0.json")),
            record.to_string(),
        )
        .unwrap();
    }
}

/// D-125: the launchers are the programs of the component's package only,
/// from its record (Ruby 4.0.7 of conda-forge: 16 of the 30 of `bin`).
#[test]
fn launchers_only_for_the_programs_of_the_package() {
    let tmp = TempDir::new("conda-launchers");
    let prefix = tmp.0.join("ruby").join("4.0");
    env_with(
        &prefix,
        &[
            (
                "ruby",
                "4.0.7",
                &["bin/ruby", "bin/gem", "bin/irb", "lib/libruby.so", "bin/"],
            ),
            // Same start of the name, another package.
            ("ruby-dev", "1.0", &["bin/not-ruby"]),
            ("openssl", "3.5.0", &["bin/openssl", "bin/c_rehash"]),
            ("ncurses", "6.5", &["bin/tput", "bin/clear"]),
        ],
    );
    assert_eq!(
        package_programs(&prefix, "ruby").unwrap(),
        ["gem", "irb", "ruby"]
    );
    // An old launcher of a program the package no longer has goes.
    let dir = prefix.join(LAUNCHERS);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("old"), "").unwrap();
    write_launchers(&prefix, "ruby").unwrap();
    let mut names: Vec<String> = fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, ["gem", "irb", "ruby"]);
    let text = fs::read_to_string(dir.join("ruby")).unwrap();
    assert_eq!(text, launcher_text("ruby"));
    assert!(text.starts_with("#!/bin/sh\n") && text.contains(r#"exec "$d/bin/ruby" "$@""#));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(dir.join("ruby")).unwrap().permissions().mode();
        assert_eq!(mode & 0o111, 0o111, "executable");
    }
    // No record of the package: an error.
    assert!(package_programs(&prefix, "mongodb").is_err());
    assert!(write_launchers(&tmp.0.join("empty"), "r-base").is_err());
}

/// The launcher runs the program of `bin` with that `bin` first in its
/// PATH, from any folder, also through a path with a space.
#[cfg(unix)]
#[test]
fn a_launcher_runs_the_program_with_its_bin_first() {
    let tmp = TempDir::new("conda-launcher-run");
    let prefix = tmp.0.join("k it").join("r");
    env_with(&prefix, &[("r-base", "4.6.1", &["bin/Rscript"])]);
    let bin = prefix.join("bin");
    crate::platform::files::write_executable(
        &bin.join("Rscript"),
        b"#!/bin/sh\necho \"args=$*\"\necho \"path=${PATH%%:*}\"\n",
    )
    .unwrap();
    write_launchers(&prefix, "r-base").unwrap();
    let out = std::process::Command::new(prefix.join(LAUNCHERS).join("Rscript"))
        .args(["-e", "1"])
        .current_dir(&tmp.0)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("args=-e 1"), "{text}");
    assert!(text.contains(&format!("path={}", bin.display())), "{text}");
}

/// At start: `kit-bin` for the lines installed before the kit wrote it,
/// on the targets where the component is a conda environment; a line that
/// has it already is left alone.
#[test]
fn launchers_for_older_installations() {
    let tmp = TempDir::new("conda-launchers-old");
    let ruby = Catalog::get().find("ruby").unwrap();
    let kit = Kit::at_target(tmp.0.join("Kit"), Target::LINUX_X64);
    for (line, version) in [("4.0", "4.0.7"), ("3.4", "3.4.9")] {
        let dir = crate::common::fsx::join_rel(kit.apps(), &format!("ruby/{line}"));
        env_with(&dir, &[("ruby", version, &["bin/ruby", "bin/gem"])]);
        fs::create_dir_all(dir.join("bin")).unwrap();
        fs::write(dir.join("bin").join("ruby"), "").unwrap();
        crate::versions::write_version(&dir, version).unwrap();
    }
    ruby.save_version(&kit, "4.0.7").unwrap();
    let kept = crate::common::fsx::join_rel(kit.apps(), "ruby/3.4").join(LAUNCHERS);
    fs::create_dir_all(&kept).unwrap();
    launchers_of_older_installations(&kit);
    let new = crate::common::fsx::join_rel(kit.apps(), "ruby/4.0").join(LAUNCHERS);
    assert!(new.join("ruby").is_file() && new.join("gem").is_file());
    assert!(
        !kept.join("ruby").exists(),
        "a line with them is left alone"
    );

    // Windows: Ruby is not a conda environment, nothing written.
    let win = Kit::at_target(tmp.0.join("Win"), Target::WIN32_X64);
    let dir = crate::common::fsx::join_rel(win.apps(), "ruby/4.0");
    env_with(&dir, &[("ruby", "4.0.7", &["bin/ruby"])]);
    fs::create_dir_all(dir.join("bin")).unwrap();
    fs::write(dir.join("bin").join("ruby.exe"), "").unwrap();
    ruby.save_version(&win, "4.0.7").unwrap();
    launchers_of_older_installations(&win);
    assert!(!dir.join(LAUNCHERS).exists());
}
