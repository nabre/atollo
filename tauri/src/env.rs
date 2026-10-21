//! The kit environment given to every program the app starts.
//!
//! Port of nabre/atollo `setup/windows/_env.cmd` (commit a61a354), adapted to
//! the layout `Atollo.exe` + `data\` + `projects\` (DECISIONS.md D-00, D-01):
//! profile, AppData, temp, caches, agents and language tools all point inside
//! `data\`. Differences from `_env.cmd`, on purpose:
//! - the environment starts from an allow-list of system variables, not from
//!   the whole user environment, and PATH ends with the three system folders
//!   only (`_env.cmd:228` appends the PC's whole PATH);
//! - network settings come from `data\app\settings.json`, not `proxy.cmd`;
//! - PowerShell 7 of the kit (`pwsh`) is on PATH, with telemetry and update
//!   checks off (DECISIONS.md D-23), and `PSModulePath` without the PC's
//!   modules.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use crate::fsx::join_rel;
use crate::kit::{Kit, Os, Target};
use crate::os::volume::{DriveKind, VolumeInfo};
use crate::settings::{ProgramTemp, Settings};

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

    /// Case-insensitive lookup, as Windows does.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.vars
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
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

/// The newest version folder in `ndk` (`30.0.16248370`), if any.
fn newest_ndk(ndk: &Path) -> Option<PathBuf> {
    let key = |p: &PathBuf| -> Vec<u64> {
        p.file_name()
            .map(|n| {
                n.to_string_lossy()
                    .split('.')
                    .filter_map(|x| x.parse().ok())
                    .collect()
            })
            .unwrap_or_default()
    };
    std::fs::read_dir(ndk)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.join("source.properties").is_file())
        .max_by_key(key)
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

    /// The PATH separator used by this environment (`;` on Windows, `:` on Unix).
    pub fn path_sep(&self) -> char {
        self.path_sep
            .unwrap_or(if cfg!(windows) { ';' } else { ':' })
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

impl KitEnv {
    /// Builds the environment of `kit`. Reads only which programs exist in
    /// `data\apps` (like the `if exist` lines of `_env.cmd`); writes nothing.
    pub fn build(kit: &Kit, settings: &Settings, lang: &str, host: &Host) -> KitEnv {
        let is_win = kit.os().is_windows();
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

        let root = kit.root();
        let data = kit.data();
        let apps = kit.apps();
        let home = kit.home();
        let arch = kit.arch().as_str();
        let platform = kit.target().as_str().to_owned();
        let fwd = |p: &Path| p.to_string_lossy().replace('\\', "/");
        // In the folder of a versioned component: that of its default
        // version (versions/mod.rs).
        let app = |rel: &str| crate::setup::catalog::kit_path(kit, rel);
        let has = |rel: &str| app(rel).is_file();

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
        if let Some(real) = host
            .get("CP_HOST_TEMP")
            .or(host.get("TEMP"))
            .or(host.get("TMP"))
            .or(host.get("TMPDIR"))
        {
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
        env.set_path("TEMP", &program_tmp);
        env.set_path("TMP", &program_tmp);
        if !is_win {
            env.set_path("TMPDIR", &program_tmp);
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

        // Python, pip, Node, npm, uv, gh, PHP, Composer (_env.cmd:101-139)
        let pyuser = data.join("python-user");
        env.set_path("PYTHONUSERBASE", &pyuser);
        env.set_path("PIP_CACHE_DIR", join_rel(&data, "cache/pip"));
        env.set("PIP_DISABLE_PIP_VERSION_CHECK", "1");
        env.set("PYTHONIOENCODING", "utf-8");
        let npm_prefix = data.join("npm-global");
        env.set_path("NPM_CONFIG_PREFIX", &npm_prefix);
        env.set_path("NPM_CONFIG_CACHE", join_rel(&data, "cache/npm"));
        env.set_path("NPM_CONFIG_USERCONFIG", home.join(".npmrc"));
        // yarn through corepack (the pnpm component): its downloads in the
        // kit, no question before one, never a change to package.json.
        env.set_path("COREPACK_HOME", join_rel(&data, "cache/corepack"));
        env.set("COREPACK_ENABLE_DOWNLOAD_PROMPT", "0");
        env.set("COREPACK_ENABLE_AUTO_PIN", "0");
        env.set("NODE_OPTIONS", node_options(kit));
        env.set_path("UV_CACHE_DIR", join_rel(&data, "cache/uv"));
        env.set_path("UV_PYTHON_INSTALL_DIR", join_rel(&data, "uv/python"));
        // uv's own Pythons only (downloaded into the kit): never one of the
        // PC's, found through the registry, and none written into it.
        env.set("UV_PYTHON_PREFERENCE", "only-managed");
        env.set("UV_PYTHON_INSTALL_REGISTRY", "0");
        env.set_path("UV_TOOL_DIR", join_rel(&data, "uv/tools"));
        let uv_bin = join_rel(&data, "uv/bin");
        env.set_path("UV_TOOL_BIN_DIR", &uv_bin);
        env.set_path("GH_CONFIG_DIR", data.join("gh"));
        // The PHP in use: php.ini finds its extensions through it, so a
        // saved version of PHP loads its own (versions/mod.rs); each line
        // has its php.ini (PHPRC). Apache's, with `/`, follow them when a
        // version is chosen for the kit or a project (versions::apply).
        env.set_php(&data, &app("php"));
        env.set_path("CP_APACHE", app("apache"));
        env.set("CP_APACHE_FWD", fwd(&app("apache")));
        // Caddy (D-111): its folder, as Apache's.
        env.set_path("CP_CADDY", app("caddy"));
        env.set("CP_CADDY_FWD", fwd(&app("caddy")));
        let composer = data.join("composer");
        env.set_path("COMPOSER_HOME", &composer);
        env.set_path("COMPOSER_CACHE_DIR", join_rel(&data, "cache/composer"));

        // Rust and the other languages (_env.cmd:153-184)
        let rustup = app("rust/rustup");
        let cargo = app("rust/cargo");
        env.set_path("RUSTUP_HOME", &rustup);
        env.set_path("CARGO_HOME", &cargo);
        let gobin = join_rel(&data, "go/bin").join(&platform);
        env.set_path("GOPATH", data.join("go"));
        env.set_path("GOBIN", &gobin);
        env.set_path("GOCACHE", join_rel(&data, "cache/go-build"));
        env.set_path("GOMODCACHE", join_rel(&data, "go/pkg/mod"));
        env.set_path("GOENV", join_rel(&data, "go/env"));
        env.set("GOTELEMETRY", "off");
        if has(r"java\bin\java.exe") {
            env.set_path("JAVA_HOME", app("java"));
        }
        env.set_path("GRADLE_USER_HOME", data.join("gradle"));
        // Gradle 9 runs on Java 17 to 27: the kit's gradle (data\kit\bin)
        // starts it on the newest JDK, whatever Java a project uses (its
        // toolchain compiles). With `/`: the versions' rewrites leave it.
        if let Some(java) = crate::setup::catalog::Catalog::get().find("java") {
            let newest = crate::versions::newest_line(kit, java);
            let fits = newest
                .as_deref()
                .and_then(|l| l.split('.').next())
                .and_then(|m| m.parse::<u32>().ok())
                .is_some_and(|m| m >= 17);
            if let (true, Some(dir)) =
                (fits, crate::versions::dir_for(kit, java, newest.as_deref()))
            {
                env.set("ATOLLO_GRADLE_JAVA", fwd(&dir));
            }
        }
        let dotnet_home = data.join("dotnet").join(&platform);
        env.set_path("DOTNET_ROOT", app("dotnet"));
        env.set_path("DOTNET_CLI_HOME", &dotnet_home);
        env.set_path("NUGET_PACKAGES", join_rel(&data, "cache/nuget"));
        env.set("DOTNET_CLI_TELEMETRY_OPTOUT", "1");
        env.set("DOTNET_NOLOGO", "1");
        env.set("DOTNET_SKIP_FIRST_TIME_EXPERIENCE", "1");
        env.set("DOTNET_MULTILEVEL_LOOKUP", "0");
        let pub_cache = join_rel(&data, "cache/pub");
        env.set_path("PUB_CACHE", &pub_cache);
        env.set_path("ZIG_GLOBAL_CACHE_DIR", join_rel(&data, "cache/zig"));
        env.set_path("JULIA_DEPOT_PATH", data.join("julia").join(&platform));
        let bun = data.join("bun").join(&platform);
        env.set_path("BUN_INSTALL", &bun);
        env.set_path("DENO_DIR", join_rel(&data, "cache/deno"));
        let deno_root = data.join("deno").join(&platform);
        env.set_path("DENO_INSTALL_ROOT", &deno_root);
        env.set_path("TYPST_PACKAGE_PATH", join_rel(&data, "typst/packages"));
        env.set_path("TYPST_PACKAGE_CACHE_PATH", join_rel(&data, "cache/typst"));
        env.set_path("TEXMFHOME", data.join("texmf"));
        let texlive = data.join("texlive").join(&platform);
        env.set_path("TEXMFVAR", texlive.join("texmf-var"));
        env.set_path("TEXMFCONFIG", texlive.join("texmf-config"));
        let pnpm = data.join("pnpm").join(&platform);
        env.set_path("PNPM_HOME", &pnpm);
        env.set_path("pnpm_config_store_dir", join_rel(&data, "cache/pnpm-store"));
        env.set_path("YARN_CACHE_FOLDER", join_rel(&data, "cache/yarn"));
        // One library per minor version of R (`%v` = 4.6): packages
        // compiled for one do not load in another (D-81).
        env.set(
            "R_LIBS_USER",
            format!("{}", data.join("r").join(&platform).join("%v").display()),
        );

        // Variables keyed on installed components, per system (D-112):
        // each hook reads only what is on disk.
        // C and C++ on macOS and Linux (D-112): the shims over Zig in
        // `cc/bin`, for Makefiles, CMake, cc-rs, cgo and native gems, only
        // with the kit's Zig ([`unix_cc_env`]). On Windows the PATH finds
        // llvm-mingw's gcc and clang, as before.
        let unix_cc = if is_win {
            UnixCc::default()
        } else {
            unix_cc_env(&app(r"cc\bin"), has(r"zig\zig.exe"), |p| p.is_file())
        };
        for (name, value) in &unix_cc.vars {
            env.set_path(name, value);
        }
        // rustc links with `cc` from the PATH, the shim then: on a Mac with
        // the Command Line Tools, Apple's linker instead, which knows the
        // flags and frameworks rustc passes.
        if unix_cc.on_path {
            if let Some((name, linker)) = macos_rust_linker(kit.target(), xcode_clt_present()) {
                env.set(&name, linker);
            }
        }
        // Rust on Windows ARM64 (D-112): the `gnullvm` host links with
        // llvm-mingw's clang for aarch64 (the `cc` component).
        if kit.target() == Target::WIN32_ARM64 {
            if let Some(linker) = arm64_rust_linker(&app(r"cc\bin"), |p| p.is_file()) {
                env.set_path("CARGO_TARGET_AARCH64_PC_WINDOWS_GNULLVM_LINKER", linker);
            }
        }
        // Git of dugite-native on macOS and Linux (D-112).
        if !is_win {
            for (name, value) in git_unix_env(kit.os(), &app("git"), |p| p.exists()) {
                env.set_path(name, value);
            }
        }
        // Ruby on macOS and Linux (D-112): gems in the kit, one folder per
        // system and line (native gems are built for both), for the line
        // the kit uses: the newest on disk, as `apply_kit_choice` below
        // moves the PATH to it. On Windows they stay in the folder of
        // their Ruby.
        let gem_home = if is_win || !has(r"ruby\bin\ruby.exe") {
            None
        } else {
            crate::setup::catalog::Catalog::get()
                .find("ruby")
                .and_then(|ruby| {
                    let line = crate::versions::newest_line(kit, ruby)?;
                    let dir = crate::versions::dir_for(kit, ruby, Some(&line))?;
                    gem_home_for_dir(&data, kit.target(), &dir)
                })
        };
        if let Some(gems) = &gem_home {
            env.set_path("GEM_HOME", gems);
            // Gem commands start the Ruby on the PATH (`#!/usr/bin/env
            // ruby`), not the absolute path of the Ruby that installed
            // them: they survive a move of the kit and follow the line.
            env.set_path("GEMRC", gemrc_path(&data));
        }
        // .NET on Linux without libicu (D-112): invariant globalization,
        // or every `dotnet` command stops at start. Whether .NET is on disk
        // or not: an environment built before its installation (the final
        // check of `prepare`) needs it too, and nothing else reads it.
        if kit.os() == Os::Linux && !libicu_present() {
            env.set("DOTNET_SYSTEM_GLOBALIZATION_INVARIANT", "1");
        }

        // PowerShell 7 of the kit (new, D-23)
        env.set("POWERSHELL_TELEMETRY_OPTOUT", "1");
        env.set("POWERSHELL_CLI_TELEMETRY_OPTOUT", "1");
        env.set("POWERSHELL_UPDATECHECK", "Off");
        // .NET would save its startup profile in the real %LOCALAPPDATA%
        // (it asks Windows for the folder, ignoring LOCALAPPDATA).
        env.set("DOTNET_MultiCoreJitNoProfileGather", "1");

        // PATH: kit tools in the order of _env.cmd:187-228, then the system
        // The kit's own commands first (agy.cmd carries Antigravity's login).
        let mut path: Vec<PathBuf> = vec![kit.kit_files().join("bin"), app("claude"), app("tools")];
        if has(r"gh\bin\gh.exe") {
            path.push(app(r"gh\bin"));
        }
        if has(r"uv\uv.exe") {
            path.push(app("uv"));
        }
        if has(r"pwsh\pwsh.exe") {
            path.push(app("pwsh"));
        }
        if has(r"codex\bin\codex.exe") {
            path.push(app(r"codex\bin"));
            path.push(app(r"codex\codex-path"));
        }
        // Antigravity is not on the PATH itself: `agy` is data\kit\bin\agy.cmd,
        // which carries its login in and out of Windows (agents/keyring/).
        if has(r"python\python.exe") {
            if is_win {
                path.push(app("python"));
                path.push(app(r"python\Scripts"));
            } else {
                path.push(app("python/bin"));
                let shims = kit.kit_files().join("shims");
                path.push(shims.clone());
                env.shims = Some(shims);
            }
        }
        if is_win {
            path.extend(subdirs_starting_with(&pyuser, "Python3").map(|d| d.join("Scripts")));
        } else if pyuser.join("bin").is_dir() {
            path.push(pyuser.join("bin"));
        }
        if has(r"node\node.exe") {
            if is_win {
                path.push(app("node"));
                path.push(npm_prefix.clone());
            } else {
                path.push(app("node/bin"));
                path.push(npm_prefix.join("bin"));
            }
        }
        if has(r"php\php.exe") {
            if is_win {
                path.push(app("php"));
            } else {
                path.push(app("php/bin"));
            }
            path.push(join_rel(&composer, "vendor/bin"));
        }
        if has(r"apache\bin\httpd.exe") {
            path.push(app(r"apache\bin"));
        }
        if has(r"caddy\caddy.exe") {
            path.push(app("caddy"));
        }
        let cargo_exe = if is_win { "bin/cargo.exe" } else { "bin/cargo" };
        let rustup_exe = if is_win {
            "bin/rustup.exe"
        } else {
            "bin/rustup"
        };
        if !join_rel(&cargo, cargo_exe).is_file() {
            path.extend(
                subdirs_starting_with(&rustup.join("toolchains"), "stable-").map(|d| d.join("bin")),
            );
        }
        if join_rel(&cargo, rustup_exe).is_file() {
            path.push(cargo.join("bin"));
        }
        path.push(uv_bin);
        let when = |path: &mut Vec<PathBuf>, rel: &str, dirs: &[PathBuf]| {
            if has(rel) {
                path.extend(dirs.iter().cloned());
            }
        };
        when(&mut path, r"go\bin\go.exe", &[app(r"go\bin"), gobin]);
        when(&mut path, r"java\bin\java.exe", &[app(r"java\bin")]);
        when(&mut path, r"maven\bin\mvn.cmd", &[app(r"maven\bin")]);
        when(&mut path, r"gradle\bin\gradle.bat", &[app(r"gradle\bin")]);
        when(&mut path, r"kotlin\bin\kotlinc.bat", &[app(r"kotlin\bin")]);
        // The Android SDK (D-84): Gradle finds it through ANDROID_HOME.
        if has(r"android\cmdline-tools\latest\bin\sdkmanager.bat") {
            env.set_path("ANDROID_HOME", app("android"));
            env.set_path("ANDROID_SDK_ROOT", app("android"));
            path.push(app(r"android\cmdline-tools\latest\bin"));
            path.push(app(r"android\platform-tools"));
            // The newest NDK installed by sdkmanager (Tauri mobile).
            if let Some(ndk) = newest_ndk(&app(r"android\ndk")) {
                env.set_path("NDK_HOME", &ndk);
                env.set_path("ANDROID_NDK_HOME", &ndk);
            }
        }
        when(
            &mut path,
            r"dotnet\dotnet.exe",
            &[app("dotnet"), join_rel(&dotnet_home, ".dotnet/tools")],
        );
        when(
            &mut path,
            r"dart\bin\dart.exe",
            &[app(r"dart\bin"), pub_cache.join("bin")],
        );
        when(&mut path, r"zig\zig.exe", &[app("zig")]);
        when(&mut path, r"julia\bin\julia.exe", &[app(r"julia\bin")]);
        when(&mut path, r"bun\bun.exe", &[app("bun"), bun.join("bin")]);
        when(
            &mut path,
            r"deno\deno.exe",
            &[app("deno"), deno_root.join("bin")],
        );
        when(&mut path, r"typst\typst.exe", &[app("typst")]);
        let tl_bin = format!("tinytex/bin/{}", kit.target().texlive_bin());
        when(&mut path, r"tinytex\bin\windows\tlmgr.bat", &[app(&tl_bin)]);
        let pnpm_installed = if is_win {
            npm_prefix.join("pnpm.cmd").is_file()
        } else {
            join_rel(&npm_prefix, "bin/pnpm").is_file()
        };
        if pnpm_installed {
            path.push(pnpm);
        }
        when(&mut path, r"cmake\bin\cmake.exe", &[app(r"cmake\bin")]);
        if has(r"cmake\bin\cmake.exe") {
            // The kit's Ninja, never a Visual Studio or NMake of the PC;
            // CMake 4 configures projects written for CMake < 3.5 too.
            if has(r"tools\ninja.exe") {
                env.set("CMAKE_GENERATOR", "Ninja");
            }
            env.set("CMAKE_POLICY_VERSION_MINIMUM", "3.5");
        }
        if is_win {
            when(&mut path, r"cc\bin\clang.exe", &[app(r"cc\bin")]);
        } else if unix_cc.on_path {
            path.push(app(r"cc\bin"));
        }
        // A conda environment (Ruby and R off Windows, MongoDB on Linux)
        // has the programs of its dependencies in `bin` too (`openssl`,
        // `tput`, `kinit`...): its `kit-bin` instead, the launchers of the
        // component's own programs (D-125); `bin` while it has none.
        let own_bin = |rel: &str| {
            let bin = app(rel);
            let launchers = bin.with_file_name(crate::install::conda::LAUNCHERS);
            if launchers.is_dir() {
                launchers
            } else {
                bin
            }
        };
        when(
            &mut path,
            r"ruby\bin\ruby.exe",
            // Gems and their commands live in the folder of their Ruby
            // (Windows), or in GEM_HOME (macOS and Linux).
            &[own_bin(r"ruby\bin")],
        );
        if let Some(gems) = &gem_home {
            path.push(gems.join("bin"));
        }
        when(&mut path, r"r\bin\R.cmd", &[own_bin(r"r\bin")]);
        // Databases (D-73): servers and clients; MySQL's `mysql`,
        // `mysqldump`, `mysqladmin` reached through the launchers of
        // data\kit\bin, ahead of its folder (D-110).
        when(&mut path, r"mysql\bin\mysql.exe", &[app(r"mysql\bin")]);
        when(
            &mut path,
            r"mongodb\bin\mongod.exe",
            &[own_bin(r"mongodb\bin")],
        );
        when(
            &mut path,
            r"mongosh\bin\mongosh.exe",
            &[app(r"mongosh\bin")],
        );
        when(
            &mut path,
            r"mongodb-tools\bin\mongodump.exe",
            &[app(r"mongodb-tools\bin")],
        );
        when(&mut path, r"sqlite\sqlite3.exe", &[app("sqlite")]);
        if is_win {
            if has(r"git\bin\bash.exe") {
                env.set_path("CLAUDE_CODE_GIT_BASH_PATH", app(r"git\bin\bash.exe"));
                path.push(app(r"git\cmd"));
            }
            let system32 = PathBuf::from(host.system_root()).join("System32");
            path.push(system32.clone());
            path.push(system32.join("Wbem"));
            path.push(system32.join(r"WindowsPowerShell\v1.0"));
            env.set(
                "PSModulePath",
                format!(
                    "{};{}",
                    join_rel(&data, "pwsh/Modules").display(),
                    system32.join(r"WindowsPowerShell\v1.0\Modules").display()
                ),
            );
        } else {
            if has("git/bin/git") {
                path.push(app("git/bin"));
            }
            for sys in system_path(kit.os()) {
                path.push(PathBuf::from(sys));
            }
            env.set_path("PSModulePath", join_rel(&data, "pwsh/Modules"));
        }
        let path: Vec<String> = path
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        let sep_str = if is_win { ";" } else { ":" };
        env.set("PATH", path.join(sep_str));

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
                crate::log::warn(&format!("{}: owner-only access: {e:#}", tmp.display()));
            }
        }
        Ok(())
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

/// The system folders that end the PATH off Windows: Homebrew's on macOS
/// (`/opt/homebrew/bin` on Apple Silicon, `/usr/local/bin` on Intel) after
/// the kit's programs, so a tool the kit lacks can still come from it.
fn system_path(os: crate::kit::Os) -> &'static [&'static str] {
    if os.is_macos() {
        &[
            "/opt/homebrew/bin",
            "/usr/local/bin",
            "/usr/bin",
            "/bin",
            "/usr/sbin",
            "/sbin",
        ]
    } else {
        &["/usr/local/bin", "/usr/bin", "/bin", "/usr/sbin", "/sbin"]
    }
}

/// The variables of the C and C++ toolchain on macOS and Linux and their
/// program in `cc/bin` (install/lang/cc.rs writes them).
pub(crate) const CC_VARS: [(&str, &str); 4] = [
    ("CC", "cc"),
    ("CXX", "c++"),
    ("AR", "ar"),
    ("RANLIB", "ranlib"),
];

/// What C and C++ of the kit give the environment on macOS and Linux:
/// `cc/bin` on the PATH, and the [`CC_VARS`].
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct UnixCc {
    pub on_path: bool,
    pub vars: Vec<(&'static str, PathBuf)>,
}

/// [`UnixCc`] for the `cc` component in `cc_bin` (`clang` is its key),
/// `zig` when the kit's Zig is installed. The shims of D-112 (`make` is
/// one of them) only with Zig: without it they would hide the system's
/// compiler and fail. Each variable only for a program on disk. The LLVM
/// of an older kit (no `make`, no `cc`) stays on the PATH as before, with
/// no variables.
pub(crate) fn unix_cc_env(cc_bin: &Path, zig: bool, is_file: impl Fn(&Path) -> bool) -> UnixCc {
    if !is_file(&cc_bin.join("clang")) {
        return UnixCc::default();
    }
    if !is_file(&cc_bin.join("make")) {
        return UnixCc {
            on_path: true,
            vars: Vec::new(),
        };
    }
    if !zig {
        return UnixCc::default();
    }
    UnixCc {
        on_path: true,
        vars: CC_VARS
            .iter()
            .map(|(name, tool)| (*name, cc_bin.join(tool)))
            .filter(|(_, p)| is_file(p))
            .collect(),
    }
}

/// The linker of Rust on a Mac whose PATH has the kit's `cc`, when the
/// Command Line Tools are installed (`clt`): Apple's `/usr/bin/cc`, as
/// `CARGO_TARGET_<TRIPLE>_LINKER`. Without them `/usr/bin/cc` only asks to
/// install them: the shim (`zig cc`) stays. `None` off macOS.
pub(crate) fn macos_rust_linker(target: Target, clt: bool) -> Option<(String, &'static str)> {
    if !target.os.is_macos() || !clt {
        return None;
    }
    let triple = target.rust_triple().to_ascii_uppercase().replace('-', "_");
    Some((format!("CARGO_TARGET_{triple}_LINKER"), "/usr/bin/cc"))
}

/// The folders `xcode-select -p` prints when Xcode Command Line Tools, or
/// Xcode itself, are installed, each with the `clang` of its own.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub(crate) const XCODE_DEVELOPER_DIRS: [&str; 2] = [
    "/Library/Developer/CommandLineTools",
    "/Applications/Xcode.app/Contents/Developer",
];

/// Xcode Command Line Tools (or Xcode) on this Mac; looked up once. False
/// elsewhere: a macOS kit prepared on another system cannot know.
fn xcode_clt_present() -> bool {
    #[cfg(target_os = "macos")]
    {
        static PRESENT: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *PRESENT.get_or_init(|| {
            XCODE_DEVELOPER_DIRS
                .iter()
                .any(|d| join_rel(Path::new(d), "usr/bin/clang").is_file())
        })
    }
    #[cfg(not(target_os = "macos"))]
    false
}

/// The linker of Rust's `aarch64-pc-windows-gnullvm` host in llvm-mingw's
/// `cc_bin`: its aarch64 wrapper (the linker rustc looks for by default),
/// else `clang.exe`; `None` without the `cc` component.
pub(crate) fn arm64_rust_linker(cc_bin: &Path, is_file: impl Fn(&Path) -> bool) -> Option<PathBuf> {
    ["aarch64-w64-mingw32-clang.exe", "clang.exe"]
        .iter()
        .map(|name| cc_bin.join(name))
        .find(|p| is_file(p))
}

/// The environment of a Git laid out as dugite-native (`git/libexec/git-core`)
/// on `os`, as dugite sets it (lib/git-environment.ts): its helpers,
/// templates and system gitconfig in its folder, and on Linux its CA bundle
/// when it has one. Empty on Windows and for a Git without `libexec`.
/// `PREFIX` (dugite, Linux only) is left out: in the whole kit environment
/// it would send every `make install` into Git's folder.
pub(crate) fn git_unix_env(
    os: Os,
    git: &Path,
    exists: impl Fn(&Path) -> bool,
) -> Vec<(&'static str, PathBuf)> {
    let exec = join_rel(git, "libexec/git-core");
    if os.is_windows() || !exists(&exec) {
        return Vec::new();
    }
    let mut out = vec![
        ("GIT_EXEC_PATH", exec),
        (
            "GIT_TEMPLATE_DIR",
            join_rel(git, "share/git-core/templates"),
        ),
        ("GIT_CONFIG_SYSTEM", join_rel(git, "etc/gitconfig")),
    ];
    let ca = join_rel(git, "ssl/cacert.pem");
    if os == Os::Linux && exists(&ca) {
        out.push(("GIT_SSL_CAINFO", ca));
    }
    out
}

/// `GEM_HOME` of Ruby `line` on macOS and Linux: `<data>/ruby-gems/<target>/<line>`.
pub(crate) fn gem_home(data: &Path, target: Target, line: &str) -> PathBuf {
    data.join("ruby-gems").join(target.as_str()).join(line)
}

/// `GEM_HOME` of the Ruby in `ruby_dir` (`data/apps/<target>/ruby/<line>`,
/// named after its line), for the swap of a Ruby line (versions/choice.rs):
/// `None` on Windows or for a folder not named after a line.
pub(crate) fn gem_home_for_dir(data: &Path, target: Target, ruby_dir: &Path) -> Option<PathBuf> {
    let line = ruby_dir.file_name()?.to_str()?;
    (!target.is_windows() && crate::versions::valid_line(line))
        .then(|| gem_home(data, target, line))
}

/// The kit's RubyGems configuration on macOS and Linux (`GEMRC`, merged
/// over `~/.gemrc`): `<data>/ruby-gems/gemrc`.
pub(crate) fn gemrc_path(data: &Path) -> PathBuf {
    data.join("ruby-gems").join("gemrc")
}

/// [`gemrc_path`]: gem commands written with `#!/usr/bin/env ruby`
/// (`--env-shebang`), so they run with the Ruby on the PATH after the kit
/// moves (Ruby is installed again elsewhere, D-92) or another line is
/// chosen. Bundler writes them so already.
pub(crate) const GEMRC: &str = "# Atollo: gem commands start the Ruby on the PATH, so they keep\n\
# working when the kit moves (written by Atollo, kept as it is).\n\
gem: --env-shebang\n";

/// Writes [`GEMRC`] at `rc` when missing (afterwards it is the user's).
fn write_gemrc(rc: &Path) -> std::io::Result<()> {
    if rc.exists() {
        return Ok(());
    }
    if let Some(dir) = rc.parent() {
        fs::create_dir_all(dir)?;
    }
    crate::fsx::write_atomic(rc, GEMRC.as_bytes())
}

/// The folders where Linux keeps shared libraries, for `arch` (`x86_64`,
/// `aarch64`): the multiarch ones of Debian and Ubuntu, then the others.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub(crate) fn linux_lib_dirs(arch: &str) -> Vec<PathBuf> {
    let multiarch = format!("{arch}-linux-gnu");
    let mut out = Vec::new();
    for base in ["/usr/lib", "/lib"] {
        out.push(Path::new(base).join(&multiarch));
    }
    for dir in ["/usr/lib64", "/lib64", "/usr/lib", "/lib", "/usr/local/lib"] {
        out.push(PathBuf::from(dir));
    }
    out
}

/// True when one of `dirs` has ICU's common library (`libicuuc.so.<n>`),
/// which .NET loads by its versioned name.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub(crate) fn has_libicu(dirs: &[PathBuf]) -> bool {
    dirs.iter().any(|dir| {
        fs::read_dir(dir).into_iter().flatten().flatten().any(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|n| n.starts_with("libicuuc.so."))
        })
    })
}

/// ICU is installed on this Linux PC; looked up once. Elsewhere true:
/// a Linux kit built on another system cannot know.
#[cfg(target_os = "linux")]
fn libicu_present() -> bool {
    static PRESENT: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *PRESENT.get_or_init(|| has_libicu(&linux_lib_dirs(std::env::consts::ARCH)))
}

#[cfg(not(target_os = "linux"))]
fn libicu_present() -> bool {
    true
}

/// Writes `dir/python` ([`PYTHON_SHIM`]) when it is missing or different,
/// executable (`LF`, 755). An unchanged script that lost its exec bits (a
/// kit copied by a tool that drops modes) gets them back.
fn write_python_shim(dir: &Path) -> std::io::Result<()> {
    let file = dir.join("python");
    if !fs::read(&file).is_ok_and(|b| b == PYTHON_SHIM.as_bytes()) {
        fs::create_dir_all(dir)?;
        crate::fsx::write_atomic(&file, PYTHON_SHIM.as_bytes())?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if fs::metadata(&file)?.permissions().mode() & 0o111 != 0o111 {
            fs::set_permissions(&file, fs::Permissions::from_mode(0o755))?;
        }
    }
    Ok(())
}

fn subdirs_starting_with(dir: &Path, prefix: &str) -> impl Iterator<Item = PathBuf> {
    let mut dirs: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .filter(|e| e.file_name().to_string_lossy().starts_with(prefix))
        .map(|e| e.path())
        .collect();
    dirs.sort();
    dirs.into_iter()
}

/// A variable whose name looks secret (`*_TOKEN`, `*_KEY`, `*SECRET*`,
/// `*PASSWORD*`, `*PASSWD*`, `tkn`), ignoring case.
pub fn is_secret(name: &str) -> bool {
    const SECRET: [&str; 5] = ["KEY", "TOKEN", "SECRET", "PASSWORD", "PASSWD"];
    let upper = name.to_ascii_uppercase();
    SECRET.iter().any(|s| upper.contains(s)) || upper == "TKN"
}

/// `value` of the variable `name` as it may be shown (`Atollo env`,
/// logs): `***` for a secret name ([`is_secret`]), the password of a URL
/// (`http://user:pass@proxy:8080`) masked in any other.
pub fn masked(name: &str, value: &str) -> String {
    if is_secret(name) {
        return "***".into();
    }
    let Some(scheme) = value.find("://").map(|i| i + 3) else {
        return value.to_owned();
    };
    let rest = &value[scheme..];
    let host_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    match rest[..host_end].rfind('@') {
        Some(at) => {
            let user = rest[..at].split(':').next().unwrap_or_default();
            format!("{}{user}:***{}", &value[..scheme], &rest[at..])
        }
        None => value.to_owned(),
    }
}

/// Replaces the values of variables whose name looks secret, for logs.
pub fn mask_secrets(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(eq) = rest.find('=') {
        let (before, after) = rest.split_at(eq);
        let name_start = before
            .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .map(|i| i + 1)
            .unwrap_or(0);
        out.push_str(before);
        out.push('=');
        let value = &after[1..];
        if is_secret(&before[name_start..]) {
            let quote = value.chars().next().filter(|c| *c == '"' || *c == '\'');
            let end = match quote {
                Some(q) => value[1..].find(q).map(|i| i + 2).unwrap_or(value.len()),
                None => value
                    .find(|c: char| c.is_whitespace() || c == '&' || c == '"' || c == '\'')
                    .unwrap_or(value.len()),
            };
            match quote {
                Some(q) => out.extend([q, '*', '*', '*', q]),
                None => out.push_str("***"),
            }
            rest = &value[end..];
        } else {
            rest = value;
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
#[path = "../../test/rust/env.rs"]
mod tests;
