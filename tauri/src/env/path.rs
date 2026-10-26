//! The PATH of the kit environment.

use std::fs;
use std::path::{Path, PathBuf};

use crate::common::fsx::join_rel;

use super::langs::LangDirs;
use super::unix::OsTools;
use super::{Cx, KitEnv};

/// The PATH: the kit's tools in the order of `_env.cmd:187-228`, then the
/// system's folders.
pub(super) fn path_var(env: &mut KitEnv, cx: &Cx, dirs: LangDirs, tools: OsTools) {
    let LangDirs {
        pyuser,
        npm_prefix,
        uv_bin,
        composer,
        rustup,
        cargo,
        gobin,
        dotnet_home,
        pub_cache,
        bun,
        deno_root,
        pnpm,
    } = dirs;
    let OsTools { unix_cc, gem_home } = tools;
    let Cx {
        kit, host, is_win, ..
    } = *cx;
    let data = cx.data.clone();
    let app = |rel: &str| cx.app(rel);
    let has = |rel: &str| cx.has(rel);
    // PATH: kit tools in the order of _env.cmd:187-228, then the system
    // The kit's own commands first, then the agents' launchers (agy.cmd
    // carries Antigravity's login), in a folder of their own: the shells of
    // the PC never have it (D-178, `levels::env::strip_agents`).
    let mut path: Vec<PathBuf> = vec![
        kit.kit_files().join("bin"),
        kit.kit_files().join(crate::setup::kit_files::AGENTS_DIR),
        app("claude"),
        app("tools"),
    ];
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
    if has(r"pi\pi.exe") {
        path.push(app("pi"));
    }
    // Antigravity is not on the PATH itself: `agy` is data\kit\agents\agy.cmd,
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
    // Docker Sandboxes (D-167): `sbx` in the kit's terminals, for
    // `sbx login` and a look at the projects' microVMs.
    when(
        &mut path,
        r"sandboxes\bin\sbx.exe",
        &[app(r"sandboxes\bin")],
    );
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

/// The system folders that end the PATH off Windows: Homebrew's on macOS
/// (`/opt/homebrew/bin` on Apple Silicon, `/usr/local/bin` on Intel) after
/// the kit's programs, so a tool the kit lacks can still come from it.
pub(super) fn system_path(os: crate::kit::Os) -> &'static [&'static str] {
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

#[cfg(test)]
#[path = "../../../test/rust/env/path.rs"]
mod tests;
