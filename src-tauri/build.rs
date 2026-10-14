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
