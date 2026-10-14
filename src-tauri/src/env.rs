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
//!   checks off (DECISIONS.md D-23).

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use crate::kit::Kit;
use crate::settings::Settings;

/// System variables kept from the real environment: what Windows programs
/// need to run, nothing that points into the user's profile.
const SYSTEM_VARS: &[&str] = &[
    "ALLUSERSPROFILE",
    "CommonProgramFiles",
    "CommonProgramFiles(x86)",
    "CommonProgramW6432",
    "COMPUTERNAME",
    "ComSpec",
    "DriverData",
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
    "SystemDrive",
    "SystemRoot",
    "USERDOMAIN",
    "USERDOMAIN_ROAMINGPROFILE",
    "USERNAME",
    "windir",
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

impl Host {
    /// Snapshot of the current process environment. Taken once at start,
    /// before `apply_to_current_process` redirects the profile folders.
    pub fn capture() -> Host {
        Host::from_pairs(std::env::vars_os())
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

/// The computed environment. Keys are unique ignoring case.
#[derive(Debug, Clone, Default)]
pub struct KitEnv {
    vars: BTreeMap<String, String>,
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

    fn set(&mut self, name: &str, value: impl Into<String>) {
        self.vars.retain(|k, _| !k.eq_ignore_ascii_case(name));
        self.vars.insert(name.to_owned(), value.into());
    }

    fn set_path(&mut self, name: &str, value: impl AsRef<Path>) {
        self.set(name, value.as_ref().to_string_lossy().into_owned());
    }

    /// The PATH entries, in order.
    pub fn path_entries(&self) -> Vec<&str> {
        self.get("PATH")
            .map(|p| p.split(';').collect())
            .unwrap_or_default()
    }

    /// Builds the environment of `kit`. Reads only which programs exist in
    /// `data\apps` (like the `if exist` lines of `_env.cmd`); writes nothing.
    pub fn build(kit: &Kit, settings: &Settings, lang: &str, host: &Host) -> KitEnv {
        let mut env = KitEnv::default();
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
        let platform = kit.arch().platform();
        let fwd = |p: &Path| p.to_string_lossy().replace('\\', "/");
        let has = |rel: &str| apps.join(rel).is_file();

        // Kit folders (_env.cmd:7-16, 22)
        env.set_path("ATOLLO", root);
        env.set_path("CP_ROOT", root);
        env.set_path("CP_DATA", &data);
        env.set("CP_ARCH", arch);
        env.set_path("CP_APPS", &apps);
        // The app itself, for the kit's wrappers (datakitinagy.cmd).
        if let Ok(exe) = std::env::current_exe() {
            env.set_path("ATOLLO_EXE", exe);
        }
        env.set("CP_LANG", lang);
        env.set("CP_ROOT_FWD", fwd(root));
        env.set("CP_APPS_FWD", fwd(&apps));

        // Claude Code, Codex, Antigravity (_env.cmd:55-73)
        env.set_path("CLAUDE_CONFIG_DIR", home.join(".claude"));
        env.set_path("CLAUDE_CODE_TMPDIR", data.join("tmp"));
        env.set("DISABLE_UPDATES", "1");
        env.set_path("CODEX_HOME", home.join(".codex"));
        env.set("AGY_CLI_DISABLE_AUTO_UPDATE", "true");

        // Portable profile (_env.cmd:76-98)
        if let Some(real) = host.get("CP_USER_PROFILE").or(host.get("USERPROFILE")) {
            env.set("CP_USER_PROFILE", real);
        }
        env.set_path("HOME", &home);
        env.set_path("USERPROFILE", &home);
        let home_s = home.to_string_lossy().into_owned();
        if let Some((drive, rest)) = home_s.split_once(':') {
            env.set("HOMEDRIVE", format!("{drive}:"));
            env.set("HOMEPATH", rest);
        }
        let appdata = home.join("AppData").join("Roaming");
        let localappdata = home.join("AppData").join("Local");
        env.set_path("APPDATA", &appdata);
        env.set_path("LOCALAPPDATA", &localappdata);
        env.set_path("TEMP", data.join("tmp").join("system"));
        env.set_path("TMP", data.join("tmp").join("system"));
        env.set_path("NODE_REPL_HISTORY", home.join(".node_repl_history"));
        env.set_path("PYTHON_HISTORY", home.join(".python_history"));
        env.set_path(
            "PSModuleAnalysisCachePath",
            localappdata.join(r"Microsoft\Windows\PowerShell\ModuleAnalysisCache"),
        );
        env.set("GCM_CREDENTIAL_STORE", "dpapi");
        env.set_path(
            "GCM_DPAPI_STORE_PATH",
            home.join(".gcm").join("dpapi_store"),
        );

        // Python, pip, Node, npm, uv, gh, PHP, Composer (_env.cmd:101-139)
        let pyuser = data.join("python-user");
        env.set_path("PYTHONUSERBASE", &pyuser);
        env.set_path("PIP_CACHE_DIR", data.join(r"cache\pip"));
        env.set("PIP_DISABLE_PIP_VERSION_CHECK", "1");
        env.set("PYTHONIOENCODING", "utf-8");
        let npm_prefix = data.join("npm-global");
        env.set_path("NPM_CONFIG_PREFIX", &npm_prefix);
        env.set_path("NPM_CONFIG_CACHE", data.join(r"cache\npm"));
        env.set_path("NPM_CONFIG_USERCONFIG", home.join(".npmrc"));
        env.set(
            "NODE_OPTIONS",
            format!(
                "--require \"{}\"",
                fwd(&kit.kit_files().join("loopback.js"))
            ),
        );
        env.set_path("UV_CACHE_DIR", data.join(r"cache\uv"));
        env.set_path("UV_PYTHON_INSTALL_DIR", data.join(r"uv\python"));
        env.set_path("UV_TOOL_DIR", data.join(r"uv\tools"));
        let uv_bin = data.join(r"uv\bin");
        env.set_path("UV_TOOL_BIN_DIR", &uv_bin);
        env.set_path("GH_CONFIG_DIR", data.join("gh"));
        env.set_path("PHPRC", data.join("php"));
        let composer = data.join("composer");
        env.set_path("COMPOSER_HOME", &composer);
        env.set_path("COMPOSER_CACHE_DIR", data.join(r"cache\composer"));

        // Rust and the other languages (_env.cmd:153-184)
        let rustup = apps.join(r"rust\rustup");
        let cargo = apps.join(r"rust\cargo");
        env.set_path("RUSTUP_HOME", &rustup);
        env.set_path("CARGO_HOME", &cargo);
        let gobin = data.join(r"go\bin").join(&platform);
        env.set_path("GOPATH", data.join("go"));
        env.set_path("GOBIN", &gobin);
        env.set_path("GOCACHE", data.join(r"cache\go-build"));
        env.set_path("GOMODCACHE", data.join(r"go\pkg\mod"));
        env.set_path("GOENV", data.join(r"go\env"));
        env.set("GOTELEMETRY", "off");
        if has(r"java\bin\java.exe") {
            env.set_path("JAVA_HOME", apps.join("java"));
        }
        env.set_path("GRADLE_USER_HOME", data.join("gradle"));
        let dotnet_home = data.join("dotnet").join(&platform);
        env.set_path("DOTNET_ROOT", apps.join("dotnet"));
        env.set_path("DOTNET_CLI_HOME", &dotnet_home);
        env.set_path("NUGET_PACKAGES", data.join(r"cache\nuget"));
        env.set("DOTNET_CLI_TELEMETRY_OPTOUT", "1");
        env.set("DOTNET_NOLOGO", "1");
        env.set("DOTNET_SKIP_FIRST_TIME_EXPERIENCE", "1");
        env.set("DOTNET_MULTILEVEL_LOOKUP", "0");
        let pub_cache = data.join(r"cache\pub");
        env.set_path("PUB_CACHE", &pub_cache);
        env.set_path("ZIG_GLOBAL_CACHE_DIR", data.join(r"cache\zig"));
        env.set_path("JULIA_DEPOT_PATH", data.join("julia").join(&platform));
        let bun = data.join("bun").join(&platform);
        env.set_path("BUN_INSTALL", &bun);
        env.set_path("DENO_DIR", data.join(r"cache\deno"));
        let deno_root = data.join("deno").join(&platform);
        env.set_path("DENO_INSTALL_ROOT", &deno_root);
        env.set_path("TYPST_PACKAGE_PATH", data.join(r"typst\packages"));
        env.set_path("TYPST_PACKAGE_CACHE_PATH", data.join(r"cache\typst"));
        env.set_path("TEXMFHOME", data.join("texmf"));
        let texlive = data.join("texlive").join(&platform);
        env.set_path("TEXMFVAR", texlive.join("texmf-var"));
        env.set_path("TEXMFCONFIG", texlive.join("texmf-config"));
        let pnpm = data.join("pnpm").join(&platform);
        env.set_path("PNPM_HOME", &pnpm);
        env.set_path("pnpm_config_store_dir", data.join(r"cache\pnpm-store"));
        env.set_path("YARN_CACHE_FOLDER", data.join(r"cache\yarn"));
        let gem_home = data.join("ruby").join(&platform);
        env.set_path("GEM_HOME", &gem_home);
        env.set_path("R_LIBS_USER", data.join("r").join(&platform));

        // PowerShell 7 of the kit (new, D-23)
        env.set("POWERSHELL_TELEMETRY_OPTOUT", "1");
        env.set("POWERSHELL_CLI_TELEMETRY_OPTOUT", "1");
        env.set("POWERSHELL_UPDATECHECK", "Off");
        // .NET would save its startup profile in the real %LOCALAPPDATA%
        // (it asks Windows for the folder, ignoring LOCALAPPDATA).
        env.set("DOTNET_MultiCoreJitNoProfileGather", "1");

        // PATH: kit tools in the order of _env.cmd:187-228, then the system
        // The kit's own commands first (agy.cmd carries Antigravity's login).
        let mut path: Vec<PathBuf> = vec![
            kit.kit_files().join("bin"),
            apps.join("claude"),
            apps.join("tools"),
        ];
        if has(r"pwsh\pwsh.exe") {
            path.push(apps.join("pwsh"));
        }
        if has(r"codex\bin\codex.exe") {
            path.push(apps.join(r"codex\bin"));
            path.push(apps.join(r"codex\codex-path"));
        }
        // Antigravity is not on the PATH itself: `agy` is data\kit\bin\agy.cmd,
        // which carries its login in and out of Windows (keyring.rs).
        if has(r"python\python.exe") {
            path.push(apps.join("python"));
            path.push(apps.join(r"python\Scripts"));
        }
        path.extend(subdirs_starting_with(&pyuser, "Python3").map(|d| d.join("Scripts")));
        if has(r"node\node.exe") {
            path.push(apps.join("node"));
            path.push(npm_prefix.clone());
        }
        if has(r"php\php.exe") {
            path.push(apps.join("php"));
            path.push(composer.join(r"vendor\bin"));
        }
        if !cargo.join(r"bin\cargo.exe").is_file() {
            path.extend(
                subdirs_starting_with(&rustup.join("toolchains"), "stable-").map(|d| d.join("bin")),
            );
        }
        if cargo.join(r"bin\rustup.exe").is_file() {
            path.push(cargo.join("bin"));
        }
        path.push(uv_bin);
        let when = |path: &mut Vec<PathBuf>, rel: &str, dirs: &[PathBuf]| {
            if has(rel) {
                path.extend(dirs.iter().cloned());
            }
        };
        when(&mut path, r"go\bin\go.exe", &[apps.join(r"go\bin"), gobin]);
        when(&mut path, r"java\bin\java.exe", &[apps.join(r"java\bin")]);
        when(&mut path, r"maven\bin\mvn.cmd", &[apps.join(r"maven\bin")]);
        when(
            &mut path,
            r"gradle\bin\gradle.bat",
            &[apps.join(r"gradle\bin")],
        );
        when(
            &mut path,
            r"kotlin\bin\kotlinc.bat",
            &[apps.join(r"kotlin\bin")],
        );
        when(
            &mut path,
            r"dotnet\dotnet.exe",
            &[apps.join("dotnet"), dotnet_home.join(r".dotnet\tools")],
        );
        when(
            &mut path,
            r"dart\bin\dart.exe",
            &[apps.join(r"dart\bin"), pub_cache.join("bin")],
        );
        when(&mut path, r"zig\zig.exe", &[apps.join("zig")]);
        when(
            &mut path,
            r"julia\bin\julia.exe",
            &[apps.join(r"julia\bin")],
        );
        when(
            &mut path,
            r"bun\bun.exe",
            &[apps.join("bun"), bun.join("bin")],
        );
        when(
            &mut path,
            r"deno\deno.exe",
            &[apps.join("deno"), deno_root.join("bin")],
        );
        when(&mut path, r"typst\typst.exe", &[apps.join("typst")]);
        when(
            &mut path,
            r"tinytex\bin\windows\tlmgr.bat",
            &[apps.join(r"tinytex\bin\windows")],
        );
        if npm_prefix.join("pnpm.cmd").is_file() {
            path.push(pnpm);
        }
        when(
            &mut path,
            r"cmake\bin\cmake.exe",
            &[apps.join(r"cmake\bin")],
        );
        when(&mut path, r"cc\bin\clang.exe", &[apps.join(r"cc\bin")]);
        when(
            &mut path,
            r"ruby\bin\ruby.exe",
            &[apps.join(r"ruby\bin"), gem_home.join("bin")],
        );
        when(&mut path, r"r\bin\R.cmd", &[apps.join(r"r\bin")]);
        if has(r"git\bin\bash.exe") {
            env.set_path("CLAUDE_CODE_GIT_BASH_PATH", apps.join(r"git\bin\bash.exe"));
            path.push(apps.join(r"git\cmd"));
        }
        let system32 = PathBuf::from(host.system_root()).join("System32");
        path.push(system32.clone());
        path.push(system32.join("Wbem"));
        path.push(system32.join(r"WindowsPowerShell\v1.0"));
        let path: Vec<String> = path
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        env.set("PATH", path.join(";"));

        // Settings (replace data\proxy.cmd, _env.cmd:270-273)
        let net = &settings.network;
        for (name, value) in [
            ("HTTPS_PROXY", &net.https_proxy),
            ("HTTP_PROXY", &net.http_proxy),
            ("NO_PROXY", &net.no_proxy),
            ("NODE_EXTRA_CA_CERTS", &net.extra_ca_certs),
            ("GEMINI_API_KEY", &settings.gemini_api_key),
            ("ATOLLO_LANG", &settings.lang),
        ] {
            if let Some(value) = value.as_deref().filter(|v| !v.is_empty()) {
                env.set(name, value);
            }
        }
        if net.allow_lan {
            env.set("ATOLLO_ALLOW_LAN", "1");
        }
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
            "NPM_CONFIG_PREFIX",
            "UV_TOOL_BIN_DIR",
        ] {
            if let Some(dir) = self.get(name) {
                fs::create_dir_all(dir)?;
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
        "CP_USER_PROFILE",
    ] {
        if let Some(value) = env.get(name) {
            std::env::set_var(name, value);
        }
    }
    std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", kit.webview_data());
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

/// Replaces the values of variables whose name looks secret, for logs.
pub fn mask_secrets(text: &str) -> String {
    const SECRET: [&str; 4] = ["KEY", "TOKEN", "SECRET", "PASSWORD"];
    let is_secret = |name: &str| {
        let upper = name.to_ascii_uppercase();
        SECRET.iter().any(|s| upper.contains(s)) || upper == "TKN"
    };
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
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

    fn host() -> Host {
        Host::from_pairs(
            [
                ("SystemRoot", r"C:\Windows"),
                (
                    "Path",
                    r"C:\Users\me\bin;C:\Windows\System32;C:\Program Files\Thing",
                ),
                ("USERPROFILE", r"C:\Users\me"),
                ("APPDATA", r"C:\Users\me\AppData\Roaming"),
                ("OneDrive", r"C:\Users\me\OneDrive"),
                ("PATHEXT", ".COM;.EXE;.BAT;.CMD"),
                ("HTTPS_PROXY", "http://proxy:8080"),
                ("CARGO_HOME", r"C:\Users\me\.cargo"),
            ]
            .into_iter()
            .map(|(k, v)| (k.into(), v.into())),
        )
    }

    fn fake_kit(name: &str, installed: &[&str]) -> (TempDir, Kit) {
        let tmp = TempDir::new(name);
        let kit = Kit::at(tmp.0.join("My Kit (1) & co"), Arch::X64);
        for rel in installed {
            let file = kit.apps().join(rel);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, b"").unwrap();
        }
        (tmp, kit)
    }

    #[test]
    fn profile_and_temp_inside_kit() {
        let (_tmp, kit) = fake_kit("env-profile", &[]);
        let env = KitEnv::build(&kit, &Settings::default(), "it", &host());
        let data = kit.data();
        for name in [
            "USERPROFILE",
            "HOME",
            "APPDATA",
            "LOCALAPPDATA",
            "TEMP",
            "TMP",
            "CLAUDE_CONFIG_DIR",
            "CODEX_HOME",
            "NPM_CONFIG_CACHE",
            "PIP_CACHE_DIR",
            "CARGO_HOME",
            "GOPATH",
        ] {
            let value = env.get(name).unwrap_or_else(|| panic!("{name} missing"));
            assert!(Path::new(value).starts_with(&data), "{name} = {value}");
        }
        assert_eq!(env.get("CP_USER_PROFILE"), Some(r"C:\Users\me"));
        assert_eq!(env.get("ATOLLO"), Some(kit.root().to_str().unwrap()));
        assert_eq!(env.get("CP_LANG"), Some("it"));
        assert_eq!(env.get("DISABLE_UPDATES"), Some("1"));
        assert_eq!(
            env.get("HOMEPATH").map(|p| p.ends_with(r"\data\home")),
            Some(true)
        );
    }

    #[test]
    fn nothing_from_the_user_profile_leaks() {
        let (_tmp, kit) = fake_kit("env-leak", &["git/bin/bash.exe", "node/node.exe"]);
        let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
        for (name, value) in env.iter() {
            if name == "CP_USER_PROFILE" {
                continue;
            }
            assert!(
                !value.contains(r"C:\Users\me"),
                "{name} leaks the real profile: {value}"
            );
        }
        assert!(env.get("OneDrive").is_none());
        assert_eq!(env.get("PATHEXT"), Some(".COM;.EXE;.BAT;.CMD"));
        assert_eq!(env.get("HTTPS_PROXY"), Some("http://proxy:8080"));
    }

    #[test]
    fn path_is_kit_then_system_only() {
        let (_tmp, kit) = fake_kit(
            "env-path",
            &[
                "git/bin/bash.exe",
                "node/node.exe",
                "python/python.exe",
                "pwsh/pwsh.exe",
                "go/bin/go.exe",
            ],
        );
        let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
        let entries = env.path_entries();
        let n = entries.len();
        assert_eq!(
            &entries[n - 3..],
            [
                r"C:\Windows\System32",
                r"C:\Windows\System32\Wbem",
                r"C:\Windows\System32\WindowsPowerShell\v1.0"
            ]
        );
        for entry in &entries[..n - 3] {
            assert!(Path::new(entry).starts_with(kit.data()), "{entry}");
        }
        let apps = kit.apps();
        let pos = |p: PathBuf| {
            entries
                .iter()
                .position(|e| Path::new(e) == p)
                .unwrap_or_else(|| panic!("{} missing", p.display()))
        };
        assert!(pos(apps.join("claude")) < pos(apps.join("python")));
        assert!(pos(apps.join("node")) < pos(apps.join(r"go\bin")));
        assert!(pos(apps.join(r"go\bin")) < pos(apps.join(r"git\cmd")));
        assert!(
            entries
                .iter()
                .all(|e| !Path::new(e).starts_with(apps.join("java"))),
            "java not installed"
        );
        assert_eq!(
            env.get("CLAUDE_CODE_GIT_BASH_PATH")
                .map(|p| p.ends_with(r"git\bin\bash.exe")),
            Some(true)
        );
        assert!(env.get("JAVA_HOME").is_none());
    }

    #[test]
    fn node_options_uses_forward_slashes() {
        let (_tmp, kit) = fake_kit("env-node", &[]);
        let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
        let opts = env.get("NODE_OPTIONS").unwrap();
        assert!(opts.starts_with("--require \"") && opts.ends_with("/data/kit/loopback.js\""));
        assert!(!opts.contains('\\'));
    }

    #[test]
    fn settings_override_and_add() {
        let (_tmp, kit) = fake_kit("env-settings", &[]);
        let s = Settings {
            lang: Some("fr".into()),
            network: crate::settings::Network {
                https_proxy: Some("http://corp:3128".into()),
                allow_lan: true,
                ..Default::default()
            },
            gemini_api_key: Some("abc".into()),
            ..Settings::default()
        };
        let env = KitEnv::build(&kit, &s, "fr", &host());
        assert_eq!(env.get("HTTPS_PROXY"), Some("http://corp:3128"));
        assert_eq!(env.get("ATOLLO_ALLOW_LAN"), Some("1"));
        assert_eq!(env.get("GEMINI_API_KEY"), Some("abc"));
        assert_eq!(env.get("ATOLLO_LANG"), Some("fr"));
    }

    #[test]
    fn keys_unique_ignoring_case() {
        let (_tmp, kit) = fake_kit("env-case", &[]);
        let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
        let mut seen = std::collections::HashSet::new();
        for (k, _) in env.iter() {
            assert!(seen.insert(k.to_ascii_uppercase()), "duplicate {k}");
        }
    }

    #[test]
    fn secrets_are_masked() {
        assert_eq!(
            mask_secrets("GEMINI_API_KEY=abc123 next"),
            "GEMINI_API_KEY=*** next"
        );
        assert_eq!(
            mask_secrets("url ?tkn=deadbeef&folder=x"),
            "url ?tkn=***&folder=x"
        );
        assert_eq!(mask_secrets("GH_TOKEN=\"x y\" ok"), "GH_TOKEN=\"***\" ok");
        assert_eq!(mask_secrets("SECRET='a b"), "SECRET='***'");
        assert_eq!(
            mask_secrets("PATH=C:\\x MY_PASSWORD=p"),
            "PATH=C:\\x MY_PASSWORD=***"
        );
        assert_eq!(mask_secrets("no secrets here"), "no secrets here");
    }
}
