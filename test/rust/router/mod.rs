use super::*;
use crate::kit::{tests::TempDir, Arch, Kit, Os, Target};
use crate::net::verify::Expect;
use crate::setup::Catalog;
use std::fs;

#[test]
fn target_execution_matrix_across_os_and_arch() {
    use target::can_execute_between as can;
    for host in Target::ALL {
        for t in Target::ALL {
            // Each of the four runs its own programs only: no translation
            // layer, x64 never runs ARM64 nor ARM64 x64, and cross-OS
            // execution is never allowed.
            assert_eq!(can(host, t), host == t, "{host} -> {t}");
        }
    }
}

/// Installable is available: no host-side condition is left.
#[test]
fn installable_is_available_in_the_catalog() {
    for c in Catalog::get().all() {
        for t in Target::ALL {
            assert_eq!(
                target::installable_on(c, t),
                c.is_available_on(t),
                "{} on {t}",
                c.id
            );
        }
    }
}

/// micromamba has a native build on every target.
#[test]
fn micromamba_asset_per_target() {
    let assets: Vec<&str> = Target::ALL.iter().map(|t| t.micromamba_asset()).collect();
    assert_eq!(
        assets,
        [
            "micromamba-win-64",
            "micromamba-osx-arm64",
            "micromamba-linux-64",
            "micromamba-linux-aarch64",
        ]
    );
}

#[test]
fn bins_resolve_per_target_os() {
    assert_eq!(bins::exe_name(Os::Win32, "git"), "git.exe");
    assert_eq!(bins::exe_name(Os::Linux, "git"), "git");
    assert_eq!(bins::exe_name(Os::Darwin, "git"), "git");

    let tmp = TempDir::new("router-bins");
    let win_kit = Kit::at_target(tmp.0.clone(), Target::WIN32_X64);
    let lin_kit = win_kit.for_target(Target::LINUX_X64);

    assert!(bins::editor_node(&win_kit, &win_kit.apps()).ends_with("node.exe"));
    assert!(bins::editor_node(&lin_kit, &lin_kit.apps()).ends_with("node"));
}

#[test]
fn probe_all_targets_and_static_health_work_without_spawning() {
    let tmp = TempDir::new("router-probe");
    let kit = Kit::at_target(tmp.0.clone(), Target::WIN32_X64);
    kit.prepare().unwrap();

    // Fake an installed component on linux-arm64 and a bootstrap-staged component on darwin-arm64.
    let linux_kit = kit.for_target(Target::LINUX_ARM64);
    linux_kit.prepare().unwrap();
    let catalog = Catalog::get();
    let uv = catalog.find("uv").unwrap();
    for p in uv.key_paths(&linux_kit) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, b"#!/bin/sh\n").unwrap();
    }
    uv.save_version(&linux_kit, "0.9.5").unwrap();

    let mac_kit = kit.for_target(Target::DARWIN_ARM64);
    mac_kit.prepare().unwrap();
    let rust = catalog.find("rust").unwrap();
    fs::create_dir_all(mac_kit.apps().join("rust")).unwrap();
    rust.save_version(&mac_kit, "1.91.0").unwrap();
    rust.mark_reinstall(&mac_kit, "bootstrap:darwin-arm64")
        .unwrap();

    let all = probe::probe_all_targets(&kit, "en");
    assert_eq!(all.len(), 4);

    let linux_summary = all.iter().find(|s| s.target == "linux-arm64").unwrap();
    assert_eq!(linux_summary.os, Os::Linux);
    assert_eq!(linux_summary.arch, Arch::Arm64);
    let uv_status = linux_summary.tools.iter().find(|t| t.id == "uv").unwrap();
    assert!(uv_status.installed);
    assert_eq!(uv_status.version.as_deref(), Some("0.9.5"));

    let check = probe::check_static(&linux_kit, uv).unwrap();
    assert!(check.ok);
    assert_eq!(check.answer, "0.9.5 (linux-arm64)");

    // Bootstrap-staged component on darwin-arm64 is recognized by bootstrap::is_staged,
    // check_static, and setup::task::outdated_app.
    assert!(bootstrap::is_staged(&mac_kit, rust));
    let rust_check = probe::check_static(&mac_kit, rust).unwrap();
    assert!(rust_check.ok);
    assert_eq!(rust_check.answer, "1.91.0 (bootstrap darwin-arm64)");
    assert!(crate::setup::task::outdated_app(&mac_kit).contains(&"rust".to_owned()));
}

/// The cells of the grid of the Tools page: the needs of one target are
/// chosen there (Zig next to C and C++ on macOS and Linux), every
/// unavailable cell says why, and what this host cannot run completes on
/// the first start.
#[test]
fn grid_cells_say_chosen_reason_and_first_start() {
    use crate::setup::Choice;
    let tmp = TempDir::new("router-grid");
    let kit = Kit::at_target(tmp.0.clone(), Target::detect());
    kit.prepare().unwrap();
    let catalog = Catalog::get();
    let mut choice = Choice::defaults(catalog);
    choice.set(catalog, "cc", true);
    choice.set(catalog, "zig", false);
    choice.write(&kit, catalog).unwrap();

    let all = probe::probe_all_targets(&kit, "en");
    assert_eq!(
        all.iter().map(|s| s.target.as_str()).collect::<Vec<_>>(),
        target::all_profiles().map(|p| p.id)
    );
    let host = Target::detect();
    for s in &all {
        let t = Target::parse(&s.target).unwrap();
        let tool = |id: &str| s.tools.iter().find(|x| x.id == id).unwrap();
        if tool("cc").available && tool("zig").available {
            assert_eq!(tool("zig").chosen, t.is_unix(), "{t}");
        }
        for x in &s.tools {
            // Not available: either why, or not needed (D-177), never both.
            assert_eq!(
                !x.available,
                x.unavailable_reason.is_some() || x.not_needed.is_some(),
                "{} {t}",
                x.id
            );
            assert!(x.unavailable_reason.is_none() || x.not_needed.is_none());
            assert!(!x.first_start || (x.available && !x.installed && !x.bootstrap_staged));
            // The host's own target runs its installers now.
            if t == host {
                assert!(!x.first_start, "{} {t}", x.id);
            }
        }
        // Rust's installer runs rustup-init: from another system it waits
        // for the first start there.
        let rust = tool("rust");
        if rust.available && !target::can_execute_on_host(t) {
            assert!(rust.first_start, "{t}");
        }
        if t == Target::LINUX_ARM64 {
            assert_eq!(
                tool("android").unavailable_reason.as_deref(),
                Some("Google does not publish platform-tools and build-tools for Linux ARM64")
            );
        }
    }
}

/// D-177: ConPTY is available on Windows only and not needed elsewhere,
/// with the catalog's reason: never unavailable, chosen or installed there.
#[test]
fn the_grid_says_conpty_is_not_needed_off_windows() {
    let tmp = TempDir::new("router-conpty");
    let kit = Kit::at_target(tmp.0.clone(), Target::WIN32_X64);
    kit.prepare().unwrap();
    for t in Target::ALL {
        let s = probe::probe_target(&kit, t, "en");
        let c = s.tools.iter().find(|x| x.id == "conpty").unwrap();
        if t.is_windows() {
            assert!(c.available, "{t}");
            assert_eq!(c.not_needed, None, "{t}");
        } else {
            assert!(!c.available && !c.chosen && !c.installed, "{t}");
            assert_eq!(c.unavailable_reason, None, "{t}");
            assert_eq!(
                c.not_needed.as_deref(),
                Some("the app's terminal uses the system's PTY"),
                "{t}"
            );
        }
    }
}

#[test]
fn bootstrap_take_verifies_and_consumes_on_host_target() {
    let tmp = TempDir::new("router-boot");
    let foreign_kit = Kit::at_target(tmp.0.clone(), Target::LINUX_ARM64);
    foreign_kit.prepare().unwrap();

    let boot_dir = foreign_kit.apps().join(".bootstrap");
    fs::create_dir_all(&boot_dir).unwrap();
    let payload = b"rustup-init-binary";
    let boot_file = boot_dir.join("rustup-init");
    fs::write(&boot_file, payload).unwrap();

    let sha = crate::net::verify::sha256_hex(&boot_file).unwrap();
    let expect = Expect::Sha256(sha);

    let dest = tmp.0.join("work").join("rustup-init");
    let staged = bootstrap::take(&foreign_kit, "rustup-init", &expect, &dest)
        .unwrap()
        .unwrap();
    assert_eq!(fs::read(&dest).unwrap(), payload);
    if !target::can_execute_on_host(foreign_kit.target()) {
        bootstrap::used(&foreign_kit, &staged, false);
        assert!(boot_file.exists(), "kept for the first start there");
    }

    // On the host target itself, the staged file is copied, and consumed
    // only once the installation succeeds (BUG-V6-14).
    let host_kit = Kit::at_target(tmp.0.join("host"), Target::detect());
    host_kit.prepare().unwrap();
    let host_boot = host_kit.apps().join(".bootstrap").join("rustup-init");
    fs::create_dir_all(host_boot.parent().unwrap()).unwrap();
    fs::write(&host_boot, payload).unwrap();
    let host_dest = tmp.0.join("work-host").join("rustup-init");
    let staged = bootstrap::take(&host_kit, "rustup-init", &expect, &host_dest)
        .unwrap()
        .unwrap();
    assert!(host_boot.exists(), "copied, not moved");
    bootstrap::used(&host_kit, &staged, true);
    assert!(!host_boot.exists());
}

/// Each component's pre-downloaded files are in its own folder of
/// `.bootstrap`: found by checksum wherever they are, and removed with
/// their component only. Micromamba of Git stays while R goes; the files
/// an older version left at the top go with the last component staged.
#[test]
fn bootstrap_files_go_with_their_component_only() {
    let tmp = TempDir::new("router-boot-remove");
    let foreign = Target::ALL
        .into_iter()
        .find(|t| !target::can_execute_on_host(*t))
        .unwrap();
    let kit = Kit::at_target(tmp.0.clone(), foreign);
    kit.prepare().unwrap();
    let catalog = Catalog::get();
    let (git, r) = (catalog.find("git").unwrap(), catalog.find("r").unwrap());
    let payload = b"micromamba-binary";
    for id in ["git", "r"] {
        let dir = bootstrap::staged_dir(&kit, id, None);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("micromamba"), payload).unwrap();
    }
    let r_line = bootstrap::staged_dir(&kit, "r", Some("4.4"));
    fs::create_dir_all(&r_line).unwrap();
    fs::write(r_line.join("micromamba"), b"older").unwrap();
    let top = kit.apps().join(".bootstrap");
    fs::write(top.join("legacy.tar.gz"), b"old layout").unwrap();
    assert!(bootstrap::has_staged(&kit, "git") && bootstrap::has_staged(&kit, "r"));
    assert!(!bootstrap::has_staged(&kit, "rust"));

    let expect =
        Expect::Sha256(crate::net::verify::sha256_hex(&r_line.join("micromamba")).unwrap());
    let dest = tmp.0.join("work").join("micromamba");
    assert!(bootstrap::take(&kit, "micromamba", &expect, &dest)
        .unwrap()
        .is_some());
    assert_eq!(
        fs::read(&dest).unwrap(),
        b"older",
        "found in a line's folder"
    );
    let other = Expect::Sha256("0".repeat(64));
    assert!(bootstrap::take(&kit, "micromamba", &other, &dest)
        .unwrap()
        .is_none());
    assert!(
        r_line.join("micromamba").is_file(),
        "another version of the same name is skipped, not deleted"
    );
    // The same at the top, in the older layout: it may be another
    // component's (BUG-V6-22).
    fs::write(top.join("micromamba"), b"legacy").unwrap();
    assert!(bootstrap::take(&kit, "micromamba", &expect, &dest)
        .unwrap()
        .is_some());
    assert_eq!(fs::read(top.join("micromamba")).unwrap(), b"legacy");
    fs::remove_file(top.join("micromamba")).unwrap();

    bootstrap::remove(&kit, r).unwrap();
    assert!(!bootstrap::staged_dir(&kit, "r", None).exists());
    assert!(!r_line.exists(), "its saved lines too");
    assert_eq!(
        fs::read(bootstrap::staged_dir(&kit, "git", None).join("micromamba")).unwrap(),
        payload,
        "Git keeps its own copy"
    );
    assert!(
        top.join("legacy.tar.gz").is_file(),
        "Git may still need what the older layout staged"
    );

    bootstrap::remove(&kit, git).unwrap();
    assert!(!top.exists(), "the emptied .bootstrap goes");

    // A line removed from the Tools page takes its own folder only.
    for line in [None, Some("4.4")] {
        let dir = bootstrap::staged_dir(&kit, "r", line);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("micromamba"), payload).unwrap();
    }
    bootstrap::remove_line(&kit, "r", Some("4.4")).unwrap();
    assert!(!bootstrap::staged_dir(&kit, "r", Some("4.4")).exists());
    assert!(bootstrap::has_staged(&kit, "r"));
}
