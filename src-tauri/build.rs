// Build script: embeds the Windows manifest (asInvoker, no elevation) and
// restricts the app's commands to the permissions listed in capabilities/.
// Without `AppManifest::commands`, Tauri allows every registered command
// from every window, including the editor windows (see DECISIONS.md D-14).

/// Every Tauri command of the app. Keep in sync with `commands::handler()`.
const COMMANDS: &[&str] = &[
    "app_info",
    "editor_status",
    "editor_open",
    "editor_restart",
    "projects_list",
    "terminal_open",
    "components_list",
    "setup_state",
    "setup_start",
    "setup_cancel",
    "components_apply",
    "agents_status",
    "agents_stats",
    "agent_launch",
    "agent_sign_out",
    "folder_open",
    "project_create",
    "project_delete",
    "project_clone",
    "project_site",
    "project_install_missing",
    "project_extensions",
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
    "project_env_preview",
    "project_env_apply",
    "services_list",
    "service_start",
    "service_stop",
    "service_set",
    "db_browse_tree",
    "db_browse_rows",
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
    let windows =
        tauri_build::WindowsAttributes::new().app_manifest(include_str!("windows-app.manifest"));
    let attributes = tauri_build::Attributes::new()
        .windows_attributes(windows)
        .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS));
    if let Err(error) = tauri_build::try_build(attributes) {
        panic!("tauri build failed: {error:#}");
    }
}
