//! The kit environment given to every program the app starts.
//!
//! It follows `_env.cmd`, the environment script of the earlier kits, adapted
//! to the layout `Atollo.exe` + `data\` + `projects\` (DECISIONS.md D-00,
//! D-01): profile, AppData, temp, caches, agents and language tools all point
//! inside `data\`. Differences from `_env.cmd`, on purpose:
//! - the environment starts from an allow-list of system variables, not from
//!   the whole user environment, and PATH ends with the three system folders
//!   only (`_env.cmd:228` appends the PC's whole PATH);
//! - network settings come from `data\app\settings.json`, not `proxy.cmd`;
//! - PowerShell 7 of the kit (`pwsh`) is on PATH, with telemetry and update
//!   checks off (DECISIONS.md D-23), and `PSModulePath` without the PC's
//!   modules.
//!
//! [`KitEnv::build`] reads in parts: the kit's own variables, the agents,
//! the portable profile and the settings here; the languages (`langs`),
//! the variables of each system (`unix`) and the PATH (`path`); `secrets`
//! masks what must never be shown.

mod langs;
mod path;
mod secrets;
mod unix;

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use crate::common::fsx::join_rel;
use crate::common::settings::{ProgramTemp, Settings};
use crate::kit::Kit;
use crate::platform::volume::{DriveKind, VolumeInfo};
use langs::language_vars;
use path::path_var;
pub use secrets::{mask_secrets, masked};
#[cfg(test)]
pub(crate) use unix::gem_home;
pub(crate) use unix::{gem_home_for_dir, has_libicu, linux_lib_dirs};
use unix::{os_vars, write_gemrc};

/// System variables kept from the real environment: what Windows and Unix
/// programs need to run, nothing that points into the user's profile.
const SYSTEM_VARS: &[&str] = &[
    "ALLUSERSPROFILE",
    "CommonProgramFiles",
    "CommonProgramFiles(x86)",
    "CommonProgramW6432",
    "COMPUTERNAME",
    "ComSpec",
    "DBUS_SESSION_BUS_ADDRESS",
    "DISPLAY",
    "DriverData",
    "LANG",
    "LC_ALL",
    "LOGNAME",
    "LOGONSERVER",
    "NUMBER_OF_PROCESSORS",
    "OS",
    "PATHEXT",
    "PROCESSOR_ARCHITECTURE",
    "PROCESSOR_ARCHITEW6432",
    "PROCESSOR_IDENTIFIER",
    "PROCESSOR_LEVEL",
    "PROCESSOR_REVISION",
    "ProgramData",
    "ProgramFiles",
    "ProgramFiles(x86)",
    "ProgramW6432",
    "PUBLIC",
    "SESSIONNAME",
    "SHELL",
    "SSH_AUTH_SOCK",
    "SystemDrive",
    "SystemRoot",
    "TERM",
    "USER",
    "USERDOMAIN",
    "USERDOMAIN_ROAMINGPROFILE",
    "USERNAME",
    "WAYLAND_DISPLAY",
    "windir",
    // The X11 cookie and the desktop session (Linux): windows of the kit's
    // programs under X11/XWayland, and `xdg-open` picking the handler.
    "XAUTHORITY",
    "XDG_CURRENT_DESKTOP",
    "XDG_RUNTIME_DIR",
    "XDG_SESSION_TYPE",
];

/// Proxy variables of the PC: kept (as `_env.cmd` does), overridden by the
/// settings when those are filled in.
const PROXY_VARS: &[&str] = &[
    "HTTPS_PROXY",
    "HTTP_PROXY",
    "NO_PROXY",
    "NODE_EXTRA_CA_CERTS",
];

/// The variables of the real process that the kit environment reads.
#[derive(Debug, Clone, Default)]
pub struct Host {
    vars: BTreeMap<String, String>,
}

/// The names of the variables a kit environment took from the settings
/// (`HTTPS_PROXY;ATOLLO_LANG`…).
pub const SETTINGS_VARS: &str = "ATOLLO_SETTINGS_VARS";

impl Host {
    /// Snapshot of the current process environment. Taken once at start,
    /// before `apply_to_current_process` redirects the profile folders.
    /// Started from a kit console, the variables that console got from the
    /// settings (a proxy, the language, the certificates) are not the PC's:
    /// they leave the snapshot and this process, so a setting emptied since
    /// does not come back from them (D-137).
    pub fn capture() -> Host {
        let mut host = Host::from_pairs(std::env::vars_os());
        for name in host.forget_kit_settings() {
            std::env::remove_var(name);
        }
        host
    }

    /// Removes the variables listed in `ATOLLO_SETTINGS_VARS`, and that
    /// variable; returns their names.
    pub fn forget_kit_settings(&mut self) -> Vec<String> {
        let Some(list) = self.get(SETTINGS_VARS).map(str::to_owned) else {
            return Vec::new();
        };
        let names: Vec<String> = list
            .split(';')
            .map(str::trim)
            .filter(|n| !n.is_empty())
            .chain([SETTINGS_VARS])
            .map(str::to_owned)
            .collect();
        self.vars
            .retain(|k, _| !names.iter().any(|n| n.eq_ignore_ascii_case(k)));
        names
    }

    pub fn from_pairs(pairs: impl IntoIterator<Item = (OsString, OsString)>) -> Host {
        let vars = pairs
            .into_iter()
            .filter_map(|(k, v)| Some((k.into_string().ok()?, v.into_string().ok()?)))
            .collect();
        Host { vars }
    }

    /// Every variable, to run a program of the PC with the PC's
    /// environment (`sbx`, D-167).
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.vars.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Case-insensitive lookup, as Windows does.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.vars
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    /// The host's temporary folder (D-90): `CP_HOST_TEMP` when started from
    /// a kit console, whose TEMP is the kit's; then `TEMP`, `TMP` and, the
    /// only one macOS and Linux desktops set, `TMPDIR`. One lookup for the
    /// app's own `Kit::with_host` and for the consoles it passes it to, so
    /// an Atollo started from Finder and one started from a kit console
    /// agree (BUG-V7-17).
    pub fn temp(&self) -> Option<&str> {
        self.get("CP_HOST_TEMP")
            .or(self.get("TEMP"))
            .or(self.get("TMP"))
            .or(self.get("TMPDIR"))
    }

    fn system_root(&self) -> String {
        self.get("SystemRoot").unwrap_or(r"C:\Windows").to_owned()
    }
}

/// `NODE_OPTIONS` of the kit: every Node.js loads `loopback.js`, so a server
/// on all the network cards listens on this PC only (no Firewall prompt).
pub fn node_options(kit: &Kit) -> String {
    let loopback = kit.kit_files().join("loopback.js");
    format!(
        "--require \"{}\"",
        loopback.to_string_lossy().replace('\\', "/")
    )
}

/// The computed environment. Keys are unique ignoring case. Its `Debug`
/// shows secrets masked ([`masked`]).
#[derive(Clone, Default)]
pub struct KitEnv {
    vars: BTreeMap<String, String>,
    /// `data\tmp` when the programs' temp is in the kit on a disk with
    /// permissions: `create_dirs` gives it an owner-only DACL.
    private_tmp: Option<PathBuf>,
    path_sep: Option<char>,
    /// `data/kit/shims` on the PATH (macOS, Linux): `create_dirs` writes
    /// its `python` there ([`PYTHON_SHIM`]).
    shims: Option<PathBuf>,
}

/// Never a token or a key in a log or a panic message.
impl std::fmt::Debug for KitEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vars: BTreeMap<&str, String> = self.iter().map(|(k, v)| (k, masked(k, v))).collect();
        f.debug_struct("KitEnv")
            .field("vars", &vars)
            .field("private_tmp", &self.private_tmp)
            .field("path_sep", &self.path_sep)
            .field("shims", &self.shims)
            .finish()
    }
}

/// `python` on macOS and Linux, where the kit's Python has only `python3`
/// and the commands the kit proposes start with `python`. A script, not a
/// link (exFAT and FAT keep none), in a folder only the Unix PATH has: in
/// Git Bash on Windows `python` stays the kit's `python.exe`. The folder
/// comes right after the kit's Python, so `python3` is the kit's (or the
/// version a project chose); a virtual environment has its own `python`.
pub(crate) const PYTHON_SHIM: &str =
    "#!/bin/sh\n# The kit's Python on macOS and Linux has only python3.\nexec python3 \"$@\"\n";

/// Where the programs' temp (`TEMP`, `TMP`) or Claude's (`CLAUDE_CODE_TMPDIR`,
/// `claude = true`) goes, D-90. `data` is the kit's data folder.
/// `Auto` is the host's temp when the kit is on a removable or remote disk or
/// on a file system without permissions; with the disk unknown it is the kit.
pub(crate) fn program_temp_dir(
    choice: ProgramTemp,
    volume: Option<VolumeInfo>,
    system_temp: Option<&Path>,
    data: &Path,
    claude: bool,
) -> PathBuf {
    let in_kit = if claude {
        data.join("tmp")
    } else {
        data.join("tmp").join("system")
    };
    let Some(host) = system_temp else {
        return in_kit;
    };
    let system = match choice {
        ProgramTemp::Kit => false,
        ProgramTemp::System => true,
        ProgramTemp::Auto => volume.is_some_and(|v| {
            matches!(v.drive, DriveKind::Removable | DriveKind::Remote) || !v.fs.has_permissions()
        }),
    };
    if system {
        host.join(if claude { "claude" } else { "system" })
    } else {
        in_kit
    }
}

impl KitEnv {
    pub fn get(&self, name: &str) -> Option<&str> {
        self.vars
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.vars.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    pub(crate) fn set(&mut self, name: &str, value: impl Into<String>) {
        self.vars.retain(|k, _| !k.eq_ignore_ascii_case(name));
        self.vars.insert(name.to_owned(), value.into());
    }

    pub(crate) fn remove(&mut self, name: &str) {
        self.vars.retain(|k, _| !k.eq_ignore_ascii_case(name));
    }

    fn set_path(&mut self, name: &str, value: impl AsRef<Path>) {
        self.set(name, value.as_ref().to_string_lossy().into_owned());
    }

    /// The PHP in `php_dir` and its own `php.ini` (`data\php\<line>`,
    /// install/web/php_ini.rs): `CP_PHP`, `PHPRC` and, with `/` for Apache,
    /// `CP_PHP_FWD` and `CP_PHPRC_FWD`.
    pub fn set_php(&mut self, data: &Path, php_dir: &Path) {
        let ini = crate::install::php_ini_dir(data, php_dir);
        self.set_path("CP_PHP", php_dir);
        self.set("CP_PHP_FWD", php_dir.to_string_lossy().replace('\\', "/"));
        self.set_path("PHPRC", &ini);
        self.set("CP_PHPRC_FWD", ini.to_string_lossy().replace('\\', "/"));
    }

    /// The PATH separator used by this environment (`;` on Windows, `:` on
    /// Unix): its own, else this system's.
    pub fn path_sep(&self) -> char {
        self.path_sep.unwrap_or(crate::platform::PATH_SEPARATOR)
    }

    /// The PATH entries, in order.
    pub fn path_entries(&self) -> Vec<&str> {
        let sep = self.path_sep();
        self.get("PATH")
            .map(|p| split_path(p, sep))
            .unwrap_or_default()
    }
}

/// Splits a `sep`-separated path list without breaking Windows drive prefixes
/// (`C:\...`) when testing a Unix (`:`) target on a Windows host.
pub(crate) fn split_path(value: &str, sep: char) -> Vec<&str> {
    if sep != ':' {
        return value.split(sep).collect();
    }
    let bytes = value.as_bytes();
    let mut out = Vec::new();
    let mut start = 0;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b':' {
            let is_drive = i == start + 1
                && bytes[start].is_ascii_alphabetic()
                && matches!(bytes.get(i + 1), Some(b'\\' | b'/'));
            if !is_drive {
                out.push(&value[start..i]);
                start = i + 1;
            }
        }
    }
    out.push(&value[start..]);
    out
}

/// What every part of [`KitEnv::build`] reads: the kit, the settings, the
/// host and the kit's folders.
pub(super) struct Cx<'a> {
    pub(super) kit: &'a Kit,
    pub(super) settings: &'a Settings,
    pub(super) lang: &'a str,
    pub(super) host: &'a Host,
    pub(super) is_win: bool,
    pub(super) data: PathBuf,
    pub(super) apps: PathBuf,
    pub(super) home: PathBuf,
    pub(super) platform: String,
}

impl<'a> Cx<'a> {
    fn new(kit: &'a Kit, settings: &'a Settings, lang: &'a str, host: &'a Host) -> Cx<'a> {
        Cx {
            kit,
            settings,
            lang,
            host,
            is_win: kit.os().is_windows(),
            data: kit.data(),
            apps: kit.apps(),
            home: kit.home(),
            platform: kit.target().as_str().to_owned(),
        }
    }

    /// In the folder of a versioned component: that of its default
    /// version (versions/mod.rs).
    pub(super) fn app(&self, rel: &str) -> PathBuf {
        crate::setup::kit_path(self.kit, rel)
    }

    pub(super) fn has(&self, rel: &str) -> bool {
        self.app(rel).is_file()
    }
}

impl KitEnv {
    /// Builds the environment of `kit`. Reads only which programs exist in
    /// `data\apps` (like the `if exist` lines of `_env.cmd`); writes nothing.
    pub fn build(kit: &Kit, settings: &Settings, lang: &str, host: &Host) -> KitEnv {
        let sep = kit.os().path_sep();
        let mut env = KitEnv {
            path_sep: Some(sep),
            ..KitEnv::default()
        };
        for name in SYSTEM_VARS.iter().chain(PROXY_VARS) {
            if let Some(value) = host.get(name) {
                env.set(name, value);
            }
        }
        let cx = Cx::new(kit, settings, lang, host);
        kit_vars(&mut env, &cx);
        let program_tmp = agent_vars(&mut env, &cx);
        profile_vars(&mut env, &cx, &program_tmp);
        let dirs = language_vars(&mut env, &cx);
        let tools = os_vars(&mut env, &cx);
        powershell_vars(&mut env);
        path_var(&mut env, &cx, dirs, tools);
        settings_vars(&mut env, &cx);
        // Claude Code's token saved in the kit (D-115) is not here either:
        // projects' actions, scripts, terminals and the editor never see
        // it, only the agents do (`agents::launch_env`, the kit's `claude`
        // and `agy` wrappers).
        // The versions preferred for the kit instead of the default folders.
        crate::versions::apply_kit_choice(kit, &mut env);
        env
    }

    /// Folders the environment points at that programs expect to exist
    /// (`_env.cmd:186`).
    pub fn create_dirs(&self) -> std::io::Result<()> {
        for name in [
            "CLAUDE_CONFIG_DIR",
            "CLAUDE_CODE_TMPDIR",
            "CODEX_HOME",
            "PI_CODING_AGENT_DIR",
            "HOME",
            "APPDATA",
            "LOCALAPPDATA",
            "TEMP",
            "XDG_CONFIG_HOME",
            "XDG_CACHE_HOME",
            "XDG_DATA_HOME",
            "XDG_STATE_HOME",
            "NPM_CONFIG_PREFIX",
            "UV_TOOL_BIN_DIR",
        ] {
            // %APPDATA% and %LOCALAPPDATA% are Windows folders: off Windows the
            // variables stay set (a few tools read them) but nothing creates
            // data/home/AppData.
            if self.path_sep() != ';' && matches!(name, "APPDATA" | "LOCALAPPDATA") {
                continue;
            }
            if let Some(dir) = self.get(name) {
                fs::create_dir_all(dir)?;
            }
        }
        if let Some(dir) = &self.shims {
            write_python_shim(dir)?;
        }
        if let Some(rc) = self.get("GEMRC") {
            write_gemrc(Path::new(rc))?;
        }
        // Kit mode on NTFS/ReFS: the programs' leftovers are for the owner only.
        if let Some(tmp) = &self.private_tmp {
            if let Err(e) = crate::editor::restrict_dir_to_owner(tmp) {
                crate::common::log::warn(&format!("{}: owner-only access: {e:#}", tmp.display()));
            }
        }
        Ok(())
    }
}

fn kit_vars(env: &mut KitEnv, cx: &Cx) {
    let Cx { kit, lang, .. } = *cx;
    let root = kit.root();
    let data = cx.data.clone();
    let apps = cx.apps.clone();
    let arch = kit.arch().as_str();
    let fwd = |p: &Path| p.to_string_lossy().replace('\\', "/");
    let app = |rel: &str| cx.app(rel);
    // Kit folders (_env.cmd:7-16, 22)
    env.set_path("ATOLLO", root);
    env.set_path("CP_ROOT", root);
    env.set_path("CP_DATA", &data);
    // The `[client]` login of the kit's MySQL (db/auth/, D-74, D-110),
    // on the port of the version the server runs: the launchers of
    // data\kit\bin (`mysql`, `mysqldump`, `mysqladmin`) run the programs
    // of CP_MYSQL with `--defaults-extra-file` of this login; a project's
    // programs get the login of another version when its database is
    // there (`db::services::project_client_env`).
    env.set_path(
        "ATOLLO_MYSQL_CNF",
        data.join("app").join("db").join("mysql.cnf"),
    );
    env.set_path("CP_MYSQL", app("mysql"));
    env.set("CP_ARCH", arch);
    env.set_path("CP_APPS", &apps);
    // The app itself, for the kit's wrappers (data\kit\bin\agy.cmd):
    // on Linux the AppImage, not its program inside a temporary mount.
    if let Some(exe) = crate::kit::app_exe() {
        env.set_path("ATOLLO_EXE", exe);
    }
    env.set("CP_LANG", lang);
    env.set("CP_ROOT_FWD", fwd(root));
    env.set("CP_APPS_FWD", fwd(&apps));
    env.set("CP_PROJECTS_FWD", fwd(&kit.projects()));
}

/// The agents' variables; the programs' temp, for the profile.
fn agent_vars(env: &mut KitEnv, cx: &Cx) -> PathBuf {
    let Cx { kit, settings, .. } = *cx;
    let data = cx.data.clone();
    let home = cx.home.clone();
    let app = |rel: &str| cx.app(rel);
    // Claude Code, Codex, Antigravity (_env.cmd:55-73)
    env.set_path("CLAUDE_CONFIG_DIR", home.join(".claude"));
    let temp_for = |claude| {
        program_temp_dir(
            settings.program_temp,
            kit.volume(),
            kit.system_temp(),
            &data,
            claude,
        )
    };
    let (program_tmp, claude_tmp) = (temp_for(false), temp_for(true));
    if claude_tmp == data.join("tmp") && kit.volume().is_some_and(|v| v.fs.has_permissions()) {
        env.private_tmp = Some(claude_tmp.clone());
    }
    env.set_path("CLAUDE_CODE_TMPDIR", claude_tmp);
    env.set("DISABLE_UPDATES", "1");
    env.set_path("CODEX_HOME", home.join(".codex"));
    env.set("AGY_CLI_DISABLE_AUTO_UPDATE", "true");
    // The folder of its line, for the kit's agy.cmd.
    env.set_path("CP_ANTIGRAVITY", app("antigravity"));
    // Pi (D-152): its folder in the kit's home, no update check, no
    // telemetry. The address and key of the local models are not here:
    // only Pi's launch has them (`agents::launch_env`).
    env.set_path("PI_CODING_AGENT_DIR", crate::agents::sync::pi::dir(&home));
    env.set("PI_SKIP_VERSION_CHECK", "1");
    env.set("PI_TELEMETRY", "0");
    program_tmp
}

fn profile_vars(env: &mut KitEnv, cx: &Cx, program_tmp: &Path) {
    let Cx { host, is_win, .. } = *cx;
    let data = cx.data.clone();
    let home = cx.home.clone();
    // Portable profile (_env.cmd:76-98)
    if let Some(real) = host
        .get("CP_USER_PROFILE")
        .or(host.get("USERPROFILE"))
        .or(host.get("HOME"))
    {
        env.set("CP_USER_PROFILE", real);
    }
    // The host's temporary folder for files in transit (D-90), kept for a
    // Atollo.exe started from the kit's own consoles, whose TEMP is the kit's.
    if let Some(real) = host.temp() {
        env.set("CP_HOST_TEMP", real);
    }
    env.set_path("HOME", &home);
    env.set_path("USERPROFILE", &home);
    let home_s = home.to_string_lossy().into_owned();
    // A drive letter exists only on Windows; a `:` in a Unix folder
    // name is not one.
    if let Some((drive, rest)) = home_s.split_once(':').filter(|_| is_win) {
        env.set("HOMEDRIVE", format!("{drive}:"));
        env.set("HOMEPATH", rest);
    }
    let appdata = home.join("AppData").join("Roaming");
    let localappdata = home.join("AppData").join("Local");
    env.set_path("APPDATA", &appdata);
    env.set_path("LOCALAPPDATA", &localappdata);
    env.set_path("TEMP", program_tmp);
    env.set_path("TMP", program_tmp);
    if !is_win {
        env.set_path("TMPDIR", program_tmp);
        env.set_path("XDG_CONFIG_HOME", home.join(".config"));
        env.set_path("XDG_CACHE_HOME", join_rel(&data, "cache/xdg"));
        env.set_path("XDG_DATA_HOME", join_rel(&home, ".local/share"));
        env.set_path("XDG_STATE_HOME", join_rel(&home, ".local/state"));
    }
    env.set_path("NODE_REPL_HISTORY", home.join(".node_repl_history"));
    env.set_path("PYTHON_HISTORY", home.join(".python_history"));
    env.set_path(
        "PSModuleAnalysisCachePath",
        join_rel(
            &localappdata,
            "Microsoft/Windows/PowerShell/ModuleAnalysisCache",
        ),
    );
    // Git Credential Manager's DPAPI store exists only on Windows.
    if is_win {
        env.set("GCM_CREDENTIAL_STORE", "dpapi");
        env.set_path(
            "GCM_DPAPI_STORE_PATH",
            home.join(".gcm").join("dpapi_store"),
        );
    }
}

fn powershell_vars(env: &mut KitEnv) {
    // PowerShell 7 of the kit (new, D-23)
    env.set("POWERSHELL_TELEMETRY_OPTOUT", "1");
    env.set("POWERSHELL_CLI_TELEMETRY_OPTOUT", "1");
    env.set("POWERSHELL_UPDATECHECK", "Off");
    // .NET would save its startup profile in the real %LOCALAPPDATA%
    // (it asks Windows for the folder, ignoring LOCALAPPDATA).
    env.set("DOTNET_MultiCoreJitNoProfileGather", "1");
}

fn settings_vars(env: &mut KitEnv, cx: &Cx) {
    let Cx { kit, settings, .. } = *cx;
    // Settings (replace data\proxy.cmd, _env.cmd:270-273). The Gemini
    // key is not here: only the agents get it (`agents::launch_env`).
    let net = &settings.network;
    let mut from_settings: Vec<&str> = Vec::new();
    for (name, value) in [
        ("HTTPS_PROXY", &net.https_proxy),
        ("HTTP_PROXY", &net.http_proxy),
        ("NO_PROXY", &net.no_proxy),
        ("NODE_EXTRA_CA_CERTS", &net.extra_ca_certs),
        ("ATOLLO_LANG", &settings.lang),
    ] {
        if let Some(value) = value.as_deref().filter(|v| !v.is_empty()) {
            env.set(name, value);
            from_settings.push(name);
        }
    }
    // The system's roots plus the extra PEM file, for the tools that
    // read one bundle (net/ca.rs); written before the environment.
    let bundle = crate::net::ca::bundle_file(kit);
    if net.extra_ca_certs.as_deref().is_some_and(|v| !v.is_empty()) && bundle.is_file() {
        for name in crate::net::ca::BUNDLE_VARS {
            env.set_path(name, bundle.clone());
            from_settings.push(name);
        }
    }
    if net.allow_lan {
        env.set("ATOLLO_ALLOW_LAN", "1");
        from_settings.push("ATOLLO_ALLOW_LAN");
    }
    // Which of them come from the settings: Atollo started from a kit
    // console must not take them for the PC's own (Host::capture).
    if !from_settings.is_empty() {
        env.set(SETTINGS_VARS, from_settings.join(";"));
    }
}

/// Points the app's own process at the kit folders (DECISIONS.md D-04), so
/// that nothing the app or a library writes lands in the real profile.
/// Must run before any other thread starts: `set_var` is not thread-safe.
pub fn apply_to_current_process(env: &KitEnv, kit: &Kit) {
    for name in [
        "USERPROFILE",
        "HOME",
        "APPDATA",
        "LOCALAPPDATA",
        "TEMP",
        "TMP",
        "TMPDIR",
        "XDG_CONFIG_HOME",
        "XDG_CACHE_HOME",
        "XDG_DATA_HOME",
        "XDG_STATE_HOME",
        "CP_USER_PROFILE",
    ] {
        if let Some(value) = env.get(name) {
            std::env::set_var(name, value);
        }
    }
    std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", kit.webview_data());
}

/// Writes `dir/python` ([`PYTHON_SHIM`]) when it is missing or different,
/// executable (`LF`, 755). An unchanged script that lost its exec bits (a
/// kit copied by a tool that drops modes) gets them back.
fn write_python_shim(dir: &Path) -> std::io::Result<()> {
    let file = dir.join("python");
    if !fs::read(&file).is_ok_and(|b| b == PYTHON_SHIM.as_bytes()) {
        fs::create_dir_all(dir)?;
        crate::common::fsx::write_atomic(&file, PYTHON_SHIM.as_bytes())?;
    }
    if !crate::platform::files::is_executable(&file)? {
        crate::platform::files::set_executable(&file)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../test/rust/env/mod.rs"]
mod tests;
