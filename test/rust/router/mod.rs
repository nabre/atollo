use super::*;
use crate::kit::{tests::TempDir, Arch, Kit, Os, Target};
use crate::net::verify::Expect;
use crate::setup::catalog::Catalog;
use std::fs;

#[test]
fn target_execution_matrix_across_os_and_arch() {
    for t in Target::ALL {
        assert!(target::can_execute_between(t, t), "{t} on itself");
    }
    // Windows ARM64 and macOS Apple Silicon run x64 binaries via emulation.
    assert!(target::can_execute_between(
        Target::WIN32_ARM64,
        Target::WIN32_X64
    ));
    assert!(target::can_execute_between(
        Target::DARWIN_ARM64,
        Target::DARWIN_X64
    ));
    // Linux ARM64 does not assume user-mode x64 emulation, and x64 never runs ARM64.
    assert!(!target::can_execute_between(
        Target::LINUX_ARM64,
        Target::LINUX_X64
    ));
    assert!(!target::can_execute_between(
        Target::WIN32_X64,
        Target::WIN32_ARM64
    ));
    assert!(!target::can_execute_between(
        Target::DARWIN_X64,
        Target::DARWIN_ARM64
    ));
    // Cross-OS execution is never allowed.
    assert!(!target::can_execute_between(
        Target::WIN32_X64,
        Target::LINUX_X64
    ));
    assert!(!target::can_execute_between(
        Target::LINUX_X64,
        Target::DARWIN_X64
    ));
}

#[test]
fn bins_resolve_per_target_os() {
    assert_eq!(bins::exe_name(Os::Win32, "git"), "git.exe");
    assert_eq!(bins::exe_name(Os::Linux, "git"), "git");
    assert_eq!(bins::exe_name(Os::Darwin, "git"), "git");

    assert_eq!(bins::cmd_name(Os::Win32, "npm"), "npm.cmd");
    assert_eq!(bins::cmd_name(Os::Linux, "npm"), "npm");

    assert_eq!(bins::bat_name(Os::Win32, "mvn"), "mvn.bat");
    assert_eq!(bins::bat_name(Os::Darwin, "mvn"), "mvn");

    let tmp = TempDir::new("router-bins");
    let win_kit = Kit::at_target(tmp.0.clone(), Target::WIN32_X64);
    let lin_kit = win_kit.for_target(Target::LINUX_X64);

    assert!(bins::editor_node(&win_kit, &win_kit.apps()).ends_with("node.exe"));
    assert!(bins::editor_node(&lin_kit, &lin_kit.apps()).ends_with("node"));
    assert!(bins::agent_launcher(&win_kit, "claude")
        .unwrap()
        .ends_with("claude.exe"));
    assert!(bins::agent_launcher(&lin_kit, "claude")
        .unwrap()
        .ends_with("claude"));
    assert!(bins::agent_launcher(&win_kit, "antigravity")
        .unwrap()
        .ends_with("agy.cmd"));
    assert!(bins::agent_launcher(&lin_kit, "antigravity")
        .unwrap()
        .ends_with("agy"));
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

    let router = Router::new(kit.clone());
    let all = router.probe_all_targets();
    assert_eq!(all.len(), 6);

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
    assert!(bootstrap::take(&foreign_kit, "rustup-init", &expect, &dest).unwrap());
    assert_eq!(fs::read(&dest).unwrap(), payload);

    // On the host target itself, consuming the bootstrap removes the staged file.
    let host_kit = Kit::at_target(tmp.0.join("host"), Target::detect());
    host_kit.prepare().unwrap();
    let host_boot = host_kit.apps().join(".bootstrap").join("rustup-init");
    fs::create_dir_all(host_boot.parent().unwrap()).unwrap();
    fs::write(&host_boot, payload).unwrap();
    let host_dest = tmp.0.join("work-host").join("rustup-init");
    assert!(bootstrap::take(&host_kit, "rustup-init", &expect, &host_dest).unwrap());
    assert!(!host_boot.exists());
}
