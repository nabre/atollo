// Build script: embeds the Windows manifest (asInvoker, no elevation) and
// restricts the app's commands to the permissions listed in capabilities/.
// Without `AppManifest::commands`, Tauri allows every registered command
// from every window, including the editor windows (see DECISIONS.md D-14).

/// Every Tauri command of the app. Keep in sync with `command_handler!` in
/// `src/commands/mod.rs`.
const COMMANDS: &[&str] = &[
    "app_info",
    "editor_status",
    "editor_open",
    "editor_restart",
    "projects_list",
    "terminal_open",
    "terminals_list",
    "terminal_save",
    "terminal_remove",
    "terminal_launch",
    "terminal_dir_pick",
    "components_list",
    "components_sync",
    "components_check",
    "target_profiles",
    "tools_target_set",
    "setup_state",
    "setup_start",
    "setup_cancel",
    "downloads_add",
    "downloads_pick",
    "setup_by_hand_open",
    "components_apply",
    "components_install",
    "conda_kept",
    "conda_kept_free",
    "agents_status",
    "agents_stats",
    "agent_launch",
    "agent_sign_out",
    "agent_token_link",
    "agent_token_save",
    "folder_open",
    "project_create",
    "project_delete",
    "project_delete_plan",
    "frameworks_get",
    "frameworks_set",
    "frameworks_plan",
    "frameworks_create",
    "frameworks_cancel",
    "frameworks_targets",
    "project_clone",
    "project_site",
    "project_install_missing",
    "project_extensions",
    "versions_list",
    "versions_available",
    "versions_add",
    "versions_install",
    "versions_remove",
    "project_versions",
    "project_versions_set",
    "relations_matrix",
    "extensions_matrix",
    "github_status",
    "github_sign_in",
    "github_sign_in_cancel",
    "github_sign_out",
    "github_repos",
    "github_clone",
    "github_rename",
    "github_delete",
    "github_open",
    "project_backups",
    "project_db_export",
    "project_db_restore",
    "project_mcp",
    "project_mcp_set",
    "project_editor_settings",
    "project_editor_settings_set",
    "project_action_save",
    "project_action_remove",
    "project_action_default",
    "project_status",
    "project_databases",
    "project_db_create",
    "project_db_line",
    "project_env_preview",
    "project_env_apply",
    "services_list",
    "service_start",
    "service_stop",
    "service_set",
    "service_line",
    "service_line_port",
    "servers_list",
    "server_save",
    "server_remove",
    "server_start",
    "server_stop",
    "server_reload",
    "server_assign_project",
    "projects_orphans",
    "project_orphan_adopt",
    "db_browse_tree",
    "db_browse_rows",
    "db_admin_open",
    "run_list",
    "run_start",
    "run_stop",
    "run_log",
    "run_clear",
    "run_ports",
    "run_set_port",
    "url_open",
    "settings_get",
    "settings_set",
    "update_status",
    "update_check",
    "eject_list",
    "eject_run",
    "app_quit",
];

fn main() {
    println!("cargo:rerun-if-changed=windows-app.manifest");
    println!("cargo:rerun-if-changed=i18n/catalog.json");
    println!("cargo:rerun-if-changed=catalog/components.json");
    println!("cargo:rerun-if-changed=catalog/relations.json");
    let windows =
        tauri_build::WindowsAttributes::new().app_manifest(include_str!("windows-app.manifest"));
    let attributes = tauri_build::Attributes::new()
        .windows_attributes(windows)
        .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS));
    if let Err(error) = tauri_build::try_build(attributes) {
        panic!("tauri build failed: {error:#}");
    }
    if std::env::var_os("CARGO_CFG_TARGET_OS").is_some_and(|os| os == "windows") {
        patch_windows_version_info();
    }
}

/// Ensures the Windows PE `VERSIONINFO` resource carries complete, accurate
/// SignPath-compliant metadata (`ProductName`, `FileDescription`, `CompanyName`,
/// `OriginalFilename`, `InternalName`). `tauri-build` copies `productName` into
/// `FileDescription` and omits `OriginalFilename`, so we patch the generated
/// `resource.rc` and recompile `resource.lib` in place.
fn patch_windows_version_info() {
    use std::path::{Path, PathBuf};
    use std::process::Command;

    let Some(out_dir) = std::env::var_os("OUT_DIR").map(PathBuf::from) else {
        return;
    };
    let rc_path = out_dir.join("resource.rc");
    let Ok(rc) = std::fs::read_to_string(&rc_path) else {
        return;
    };

    let patched = rc
        .replace(
            "VALUE \"CompanyName\", \"github\"",
            "VALUE \"CompanyName\", \"nabre\"",
        )
        .replace(
            "VALUE \"FileDescription\", \"Atollo\"",
            "VALUE \"FileDescription\", \"Atollo — Universal Portable Dev & AI Workspace\"\nVALUE \"InternalName\", \"Atollo.exe\"\nVALUE \"OriginalFilename\", \"Atollo.exe\"",
        );

    if patched == rc {
        return;
    }
    std::fs::write(&rc_path, patched).expect("failed to update resource.rc");

    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_env == "msvc" {
        let lib_path = out_dir.join("resource.lib");
        let rc_exe = find_rc_exe().unwrap_or_else(|| PathBuf::from("rc.exe"));
        let status = Command::new(&rc_exe)
            .arg("/fo")
            .arg(&lib_path)
            .arg("/I")
            .arg(&out_dir)
            .arg(&rc_path)
            .status()
            .unwrap_or_else(|e| panic!("failed to run {}: {e}", rc_exe.display()));
        assert!(status.success(), "rc.exe failed on patched resource.rc");
    }

    fn find_rc_exe() -> Option<PathBuf> {
        if let Some(rc) = std::env::var_os("RC") {
            return Some(PathBuf::from(rc));
        }
        let host = std::env::var("HOST").unwrap_or_default();
        let arch = if host.starts_with("aarch64") {
            "arm64"
        } else if host.starts_with("x86_64") {
            "x64"
        } else {
            "x86"
        };
        let pf86 = std::env::var_os("ProgramFiles(x86)")?;
        let bin_root = Path::new(&pf86).join("Windows Kits").join("10").join("bin");
        let mut candidates: Vec<PathBuf> = std::fs::read_dir(bin_root)
            .ok()?
            .filter_map(|e| e.ok())
            .map(|e| e.path().join(arch).join("rc.exe"))
            .filter(|p| p.is_file())
            .collect();
        candidates.sort();
        candidates.pop()
    }
}
