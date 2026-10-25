//! What each system adds to the kit environment (D-112): C and C++ over Zig,
//! Rust's linkers, Git of dugite-native, Ruby's gems, .NET without libicu.

use std::fs;
use std::path::{Path, PathBuf};

use crate::common::fsx::join_rel;
use crate::kit::{Os, Target};

use super::{Cx, KitEnv};

/// What the PATH needs of [`os_vars`]: the kit's `cc`, and the gems' folder.
pub(super) struct OsTools {
    pub(super) unix_cc: UnixCc,
    pub(super) gem_home: Option<PathBuf>,
}

/// The variables keyed on installed components, per system (D-112); what
/// the PATH needs of them after.
pub(super) fn os_vars(env: &mut KitEnv, cx: &Cx) -> OsTools {
    let Cx { kit, is_win, .. } = *cx;
    let data = cx.data.clone();
    let app = |rel: &str| cx.app(rel);
    let has = |rel: &str| cx.has(rel);
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
        crate::setup::Catalog::get().find("ruby").and_then(|ruby| {
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
    OsTools { unix_cc, gem_home }
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
pub(super) fn write_gemrc(rc: &Path) -> std::io::Result<()> {
    if rc.exists() {
        return Ok(());
    }
    if let Some(dir) = rc.parent() {
        fs::create_dir_all(dir)?;
    }
    crate::common::fsx::write_atomic(rc, GEMRC.as_bytes())
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
pub(super) fn libicu_present() -> bool {
    static PRESENT: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *PRESENT.get_or_init(|| has_libicu(&linux_lib_dirs(std::env::consts::ARCH)))
}

#[cfg(not(target_os = "linux"))]
pub(super) fn libicu_present() -> bool {
    true
}

#[cfg(test)]
#[path = "../../../test/rust/env/unix.rs"]
mod tests;
