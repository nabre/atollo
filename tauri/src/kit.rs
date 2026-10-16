//! The kit on disk: its root (the folder that holds `Dev-Kit.exe`), the
//! processor platform and every path the app uses.
//!
//! Layout (docs/app/PLAN.md §2): `Dev-Kit.exe`, `projects\` and `data\` side by
//! side; everything the kit downloads or writes lives in `data\`. Nothing is
//! ever created outside the root. The platform rule and the "kit was moved"
//! record follow nabre/dev-kit `setup/windows/_env.cmd` (lines 13-16, 238-267).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::fsx::write_atomic;
use crate::os::volume::VolumeInfo;

/// Operating system family of a kit target (`win32`, `darwin`, `linux`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Os {
    Win32,
    Darwin,
    Linux,
}

#[allow(dead_code)]
impl Os {
    /// Operating system of the running binary.
    pub const fn current() -> Os {
        #[cfg(target_os = "macos")]
        {
            Os::Darwin
        }
        #[cfg(target_os = "linux")]
        {
            Os::Linux
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            Os::Win32
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Os::Win32 => "win32",
            Os::Darwin => "darwin",
            Os::Linux => "linux",
        }
    }

    pub const fn is_windows(self) -> bool {
        matches!(self, Os::Win32)
    }

    pub const fn is_macos(self) -> bool {
        matches!(self, Os::Darwin)
    }

    pub const fn is_linux(self) -> bool {
        matches!(self, Os::Linux)
    }

    pub const fn is_unix(self) -> bool {
        matches!(self, Os::Darwin | Os::Linux)
    }

    pub const fn exe_suffix(self) -> &'static str {
        match self {
            Os::Win32 => ".exe",
            Os::Darwin | Os::Linux => "",
        }
    }

    pub const fn cmd_suffix(self) -> &'static str {
        match self {
            Os::Win32 => ".cmd",
            Os::Darwin | Os::Linux => "",
        }
    }

    pub const fn bat_suffix(self) -> &'static str {
        match self {
            Os::Win32 => ".bat",
            Os::Darwin | Os::Linux => "",
        }
    }

    pub const fn path_sep(self) -> char {
        match self {
            Os::Win32 => ';',
            Os::Darwin | Os::Linux => ':',
        }
    }
}

/// Processor platform of the downloaded programs (`x64` or `arm64`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Arch {
    X64,
    Arm64,
}

impl Arch {
    /// Same rule as `_env.cmd` on Windows (ARM64 when the process or its WOW64
    /// host reports ARM64, otherwise x64), and compile-time `target_arch` on
    /// macOS and Linux.
    pub fn detect() -> Arch {
        #[cfg(windows)]
        {
            Self::from_env_values(
                std::env::var("PROCESSOR_ARCHITECTURE").ok().as_deref(),
                std::env::var("PROCESSOR_ARCHITEW6432").ok().as_deref(),
            )
        }
        #[cfg(not(windows))]
        {
            if cfg!(target_arch = "aarch64") {
                Arch::Arm64
            } else {
                Arch::X64
            }
        }
    }

    fn from_env_values(arch: Option<&str>, wow64: Option<&str>) -> Arch {
        let is_arm = |v: Option<&str>| v.is_some_and(|v| v.eq_ignore_ascii_case("ARM64"));
        if is_arm(arch) || is_arm(wow64) {
            Arch::Arm64
        } else {
            Arch::X64
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Arch::X64 => "x64",
            Arch::Arm64 => "arm64",
        }
    }

    /// Folder name used under `data/apps` on the current OS (`win32-x64`, etc.).
    pub fn platform(self) -> String {
        Target::new(Os::current(), self).as_str().to_owned()
    }
}

/// Combined OS + CPU target (`win32-x64`, `win32-arm64`, `darwin-arm64`,
/// `darwin-x64`, `linux-x64`, `linux-arm64`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Target {
    pub os: Os,
    pub arch: Arch,
}

#[allow(dead_code)]
impl Target {
    pub const WIN32_X64: Target = Target {
        os: Os::Win32,
        arch: Arch::X64,
    };
    pub const WIN32_ARM64: Target = Target {
        os: Os::Win32,
        arch: Arch::Arm64,
    };
    pub const DARWIN_ARM64: Target = Target {
        os: Os::Darwin,
        arch: Arch::Arm64,
    };
    pub const DARWIN_X64: Target = Target {
        os: Os::Darwin,
        arch: Arch::X64,
    };
    pub const LINUX_X64: Target = Target {
        os: Os::Linux,
        arch: Arch::X64,
    };
    pub const LINUX_ARM64: Target = Target {
        os: Os::Linux,
        arch: Arch::Arm64,
    };

    pub const ALL: [Target; 6] = [
        Self::WIN32_X64,
        Self::WIN32_ARM64,
        Self::DARWIN_ARM64,
        Self::DARWIN_X64,
        Self::LINUX_X64,
        Self::LINUX_ARM64,
    ];

    pub const fn new(os: Os, arch: Arch) -> Target {
        Target { os, arch }
    }

    pub fn detect() -> Target {
        Target::new(Os::current(), Arch::detect())
    }

    pub const fn with_arch(self, arch: Arch) -> Target {
        Target { os: self.os, arch }
    }

    /// Canonical folder and matrix identifier (`win32-x64`, `darwin-arm64`, ...).
    pub const fn as_str(self) -> &'static str {
        match (self.os, self.arch) {
            (Os::Win32, Arch::X64) => "win32-x64",
            (Os::Win32, Arch::Arm64) => "win32-arm64",
            (Os::Darwin, Arch::Arm64) => "darwin-arm64",
            (Os::Darwin, Arch::X64) => "darwin-x64",
            (Os::Linux, Arch::X64) => "linux-x64",
            (Os::Linux, Arch::Arm64) => "linux-arm64",
        }
    }

    pub fn parse(s: &str) -> Option<Target> {
        match s.trim() {
            "win32-x64" => Some(Self::WIN32_X64),
            "win32-arm64" => Some(Self::WIN32_ARM64),
            "darwin-arm64" => Some(Self::DARWIN_ARM64),
            "darwin-x64" => Some(Self::DARWIN_X64),
            "linux-x64" => Some(Self::LINUX_X64),
            "linux-arm64" => Some(Self::LINUX_ARM64),
            _ => None,
        }
    }

    pub const fn is_windows(self) -> bool {
        self.os.is_windows()
    }

    pub const fn is_macos(self) -> bool {
        self.os.is_macos()
    }

    pub const fn is_linux(self) -> bool {
        self.os.is_linux()
    }

    pub const fn is_unix(self) -> bool {
        self.os.is_unix()
    }

    pub const fn exe_suffix(self) -> &'static str {
        self.os.exe_suffix()
    }

    /// Rust / LLVM target triple (`x86_64-pc-windows-msvc`, `aarch64-apple-darwin`, ...).
    pub const fn rust_triple(self) -> &'static str {
        match (self.os, self.arch) {
            (Os::Win32, Arch::X64) => "x86_64-pc-windows-msvc",
            (Os::Win32, Arch::Arm64) => "aarch64-pc-windows-msvc",
            (Os::Darwin, Arch::Arm64) => "aarch64-apple-darwin",
            (Os::Darwin, Arch::X64) => "x86_64-apple-darwin",
            (Os::Linux, Arch::X64) => "x86_64-unknown-linux-gnu",
            (Os::Linux, Arch::Arm64) => "aarch64-unknown-linux-gnu",
        }
    }

    /// GNU / LLVM CPU name (`x86_64` or `aarch64`).
    pub const fn gnu_arch(self) -> &'static str {
        match self.arch {
            Arch::X64 => "x86_64",
            Arch::Arm64 => "aarch64",
        }
    }

    /// Go / GitHub CLI / Docker OS name (`windows`, `darwin`, `linux`).
    pub const fn go_os(self) -> &'static str {
        match self.os {
            Os::Win32 => "windows",
            Os::Darwin => "darwin",
            Os::Linux => "linux",
        }
    }

    /// Go / GitHub CLI / Docker CPU name (`amd64` or `arm64`).
    pub const fn go_arch(self) -> &'static str {
        match self.arch {
            Arch::X64 => "amd64",
            Arch::Arm64 => "arm64",
        }
    }

    /// Node.js / VSCodium / Dart OS name (`win`, `darwin`, `linux`).
    pub const fn node_os(self) -> &'static str {
        match self.os {
            Os::Win32 => "win",
            Os::Darwin => "darwin",
            Os::Linux => "linux",
        }
    }

    /// Eclipse Adoptium OS name (`windows`, `mac`, `linux`).
    pub const fn adoptium_os(self) -> &'static str {
        match self.os {
            Os::Win32 => "windows",
            Os::Darwin => "mac",
            Os::Linux => "linux",
        }
    }

    /// Eclipse Adoptium CPU name (`x64` or `aarch64`).
    pub const fn adoptium_arch(self) -> &'static str {
        match self.arch {
            Arch::X64 => "x64",
            Arch::Arm64 => "aarch64",
        }
    }

    /// Conda-forge subdirectory (`win-64`, `osx-arm64`, `linux-64`, ...).
    pub const fn conda_subdir(self) -> &'static str {
        match (self.os, self.arch) {
            (Os::Win32, Arch::X64) => "win-64",
            (Os::Win32, Arch::Arm64) => "win-arm64",
            (Os::Darwin, Arch::Arm64) => "osx-arm64",
            (Os::Darwin, Arch::X64) => "osx-64",
            (Os::Linux, Arch::X64) => "linux-64",
            (Os::Linux, Arch::Arm64) => "linux-aarch64",
        }
    }

    /// Micromamba standalone binary asset name on GitHub releases.
    pub const fn micromamba_asset(self) -> &'static str {
        match (self.os, self.arch) {
            (Os::Win32, _) => "micromamba-win-64",
            (Os::Darwin, Arch::Arm64) => "micromamba-osx-arm64",
            (Os::Darwin, Arch::X64) => "micromamba-osx-64",
            (Os::Linux, Arch::X64) => "micromamba-linux-64",
            (Os::Linux, Arch::Arm64) => "micromamba-linux-aarch64",
        }
    }

    /// TinyTeX / TeX Live binary subdirectory under `tinytex/bin/`.
    pub const fn texlive_bin(self) -> &'static str {
        match (self.os, self.arch) {
            (Os::Win32, _) => "windows",
            (Os::Darwin, _) => "universal-darwin",
            (Os::Linux, Arch::X64) => "x86_64-linux",
            (Os::Linux, Arch::Arm64) => "aarch64-linux",
        }
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Error)]
pub enum KitError {
    #[error("the program path has no parent folder: {0}")]
    NoRoot(PathBuf),
    #[error("cannot write in {path}: {source}")]
    NotWritable { path: PathBuf, source: io::Error },
}

/// Paths of one kit. Cheap to clone; computed once at start.
#[derive(Debug, Clone)]
pub struct Kit {
    root: PathBuf,
    target: Target,
    /// `<host TEMP>\Dev-Kit\<id>`, for files in transit (PLAN-DATA.md §4.2).
    system_temp: Option<PathBuf>,
    /// The disk of the root, read once at start.
    volume: Option<VolumeInfo>,
}

impl Kit {
    /// The kit whose root is the folder of `exe`.
    pub fn from_exe(exe: &Path) -> Result<Kit, KitError> {
        let exe = strip_verbatim(exe);
        let root = exe
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .ok_or_else(|| KitError::NoRoot(exe.clone()))?;
        Ok(Kit::at_target(root.to_path_buf(), Target::detect()))
    }

    #[cfg(test)]
    pub fn at(root: PathBuf, arch: Arch) -> Kit {
        Self::at_target(root, Target::new(Os::current(), arch))
    }

    pub fn at_target(root: PathBuf, target: Target) -> Kit {
        Kit {
            root: strip_verbatim(&root),
            target,
            system_temp: None,
            volume: None,
        }
    }

    /// The kit with what the host gives at start: its temporary folder
    /// (`TEMP`/`TMP` of the real process, read before D-04 moves them into
    /// `data\`) and the disk of the root. Without them the kit works as before:
    /// every temporary file in `data\tmp`.
    ///
    /// A `TEMP` that is already the kit's (a Dev-Kit.exe started from a kit
    /// console without `CP_HOST_TEMP`) is not the host's: inside the root it
    /// is ignored, inside `Dev-Kit\<id>` it is taken back to its base.
    pub fn with_host(mut self, host_temp: Option<&Path>) -> Kit {
        let id = self.id();
        let mine = Path::new("Dev-Kit").join(&id);
        self.system_temp = host_temp
            .filter(|t| t.is_absolute())
            .map(strip_verbatim)
            .map(|t| match t.ancestors().find(|a| a.ends_with(&mine)) {
                Some(a) => a.to_path_buf(),
                None => t.join(&mine),
            })
            .filter(|t| !t.starts_with(&self.root));
        self.volume = crate::os::volume::info(&self.root).ok();
        self
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn target(&self) -> Target {
        self.target
    }
    pub fn os(&self) -> Os {
        self.target.os
    }
    pub fn arch(&self) -> Arch {
        self.target.arch
    }
    pub fn data(&self) -> PathBuf {
        self.root.join("data")
    }
    pub fn projects(&self) -> PathBuf {
        self.root.join("projects")
    }
    /// The kit's files of each project, one folder per project (D-89).
    pub fn project_data(&self) -> PathBuf {
        self.data().join("projects")
    }
    /// `<host TEMP>\Dev-Kit\<id>`: files in transit only (D-90); `None` when
    /// the host gave no temporary folder.
    pub fn system_temp(&self) -> Option<&Path> {
        self.system_temp.as_deref()
    }
    /// The disk of the root, when it could be read.
    pub fn volume(&self) -> Option<VolumeInfo> {
        self.volume
    }
    /// Downloaded programs for this target: `data/apps/<os>-<arch>`.
    pub fn apps(&self) -> PathBuf {
        self.data().join("apps").join(self.target.as_str())
    }
    /// Installed component version files for this target: `data/app/versions/<os>-<arch>`.
    pub fn versions_dir(&self) -> PathBuf {
        self.app_dir().join("versions").join(self.target.as_str())
    }
    /// The app's own state: settings, component choice, versions, logs.
    pub fn app_dir(&self) -> PathBuf {
        self.data().join("app")
    }
    pub fn logs(&self) -> PathBuf {
        self.app_dir().join("logs")
    }
    /// Files the programs need on disk, written from the exe (D-25).
    pub fn kit_files(&self) -> PathBuf {
        self.data().join("kit")
    }
    /// Portable profile: `USERPROFILE`, `HOME` and the agents' folders.
    pub fn home(&self) -> PathBuf {
        self.data().join("home")
    }
    pub fn webview_data(&self) -> PathBuf {
        self.data().join("webview2")
    }
    pub fn settings_file(&self) -> PathBuf {
        self.app_dir().join("settings.json")
    }
    /// Before D-89: `data\app\projects`. Read only by the migration (WP-6).
    pub fn project_states(&self) -> PathBuf {
        self.app_dir().join("projects")
    }

    /// First 16 hex chars of SHA-256 of the lower-case root: the same scheme
    /// nabre/dev-kit uses for its per-kit mutex names.
    pub fn id(&self) -> String {
        let lower = self.root.to_string_lossy().to_lowercase();
        let digest = Sha256::digest(lower.as_bytes());
        digest.iter().take(8).map(|b| format!("{b:02x}")).collect()
    }

    /// Creates `data\`, `projects\` and the app folders; checks that the root
    /// can be written. Returns the previous root when the kit has moved since
    /// the last start (another drive letter or folder). The components that
    /// follow the kit set the folders written in their files to the current
    /// ones (`install::relocate`: R's Makeconf on Windows).
    pub fn prepare(&self) -> Result<Option<PathBuf>, KitError> {
        for dir in [
            self.projects(),
            self.app_dir(),
            self.logs(),
            self.kit_files(),
        ] {
            fs::create_dir_all(&dir).map_err(|source| KitError::NotWritable {
                path: dir.clone(),
                source,
            })?;
        }
        let moved = self.check_moved().map_err(|source| KitError::NotWritable {
            path: self.app_dir(),
            source,
        })?;
        crate::install::relocate(self);
        Ok(moved)
    }

    /// `data/app/root.txt` (or `root-<os>.txt` on macOS/Linux) keeps the last
    /// root for this operating system. When it differs, the old one is also
    /// saved in `data/app/old-root.txt` (or `old-root-<os>.txt`), and the
    /// components that cannot move on this target (conda environments on
    /// macOS/Linux: Git, R) are marked to be installed again (D-92), before
    /// the new root is recorded, so a failed start notices the move again.
    fn check_moved(&self) -> io::Result<Option<PathBuf>> {
        let (record_name, pending_name) = match self.target.os {
            Os::Win32 => ("root.txt", "old-root.txt"),
            Os::Darwin => ("root-darwin.txt", "old-root-darwin.txt"),
            Os::Linux => ("root-linux.txt", "old-root-linux.txt"),
        };
        let record = self.app_dir().join(record_name);
        let now = self.root.to_string_lossy().into_owned();
        let last = fs::read_to_string(&record).ok();
        let moved = match last.as_deref().map(str::trim) {
            Some(last) if !last.is_empty() && !same_root(self.target.os, last, &now) => {
                let pending = self.app_dir().join(pending_name);
                if !pending.exists() {
                    write_atomic(&pending, last.as_bytes())?;
                }
                crate::install::flag_not_relocatable(self, last)?;
                Some(PathBuf::from(last))
            }
            _ => None,
        };
        if last.as_deref().map(str::trim) != Some(now.as_str()) {
            write_atomic(&record, now.as_bytes())?;
        }
        Ok(moved)
    }
}

/// The recorded root and the current one are the same folder. On Windows
/// paths ignore case (`E:\Kit` is `e:\kit`). Elsewhere they are compared
/// exactly: on Linux `/media/u/Kit` and `/media/u/kit` are two folders, so
/// a kit renamed only in case has moved (D-92: its conda environments point
/// at the old folder). macOS is compared exactly too: its usual file system
/// ignores case, but a volume under `/Volumes` may not, and a move noticed
/// once too often costs a reinstallation, one missed leaves Git or R broken.
pub(crate) fn same_root(os: Os, recorded: &str, now: &str) -> bool {
    if os.is_windows() {
        recorded.eq_ignore_ascii_case(now)
    } else {
        recorded == now
    }
}

/// `\\?\C:\x` -> `C:\x` (keeps `\\?\UNC\` and other forms unchanged).
fn strip_verbatim(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    match s.strip_prefix(r"\\?\") {
        Some(rest) if rest.as_bytes().get(1) == Some(&b':') => PathBuf::from(rest),
        _ => path.to_path_buf(),
    }
}

/// The first folder of `path` whose name breaks VSCodium's file watcher, if
/// any. VSCodium passes absolute Windows paths (`<kit>\data\vscodium-server\
/// extensions/**`) to `@parcel/watcher` as glob excludes, which become
/// regular expressions: after a backslash, a digit is a back reference and
/// `x`/`u` start hex escapes, so `D:\2026\kit` fails with `regex_error` and
/// changes made outside the editor are no longer noticed (verified with
/// VSCodium 1.135.06055, @parcel/watcher 2.5.6, DECISIONS.md D-33). Parentheses, spaces and `&` are fine.
pub fn watcher_unsafe_folder(path: &Path) -> Option<String> {
    path.components().find_map(|c| match c {
        std::path::Component::Normal(name) => {
            let name = name.to_string_lossy();
            let first = name.chars().next()?;
            (first.is_ascii_digit() || first == 'x' || first == 'u').then(|| name.into_owned())
        }
        _ => None,
    })
}

/// Why a project name is not accepted (rules of nabre/dev-kit
/// `core/projects.js:37-44`).
#[derive(Debug, Error, PartialEq, Eq)]
pub enum NameError {
    #[error("empty name")]
    Empty,
    #[error("forbidden character")]
    ForbiddenChar,
    #[error("starts with a dot or ends with a dot or a space")]
    BadEdge,
    #[error("reserved Windows name")]
    Reserved,
    #[error("longer than 100 characters")]
    TooLong,
}

pub fn validate_project_name(name: &str) -> Result<(), NameError> {
    if name.is_empty() {
        return Err(NameError::Empty);
    }
    if name.chars().count() > 100 {
        return Err(NameError::TooLong);
    }
    if name.chars().any(|c| {
        c.is_control() || matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
    }) {
        return Err(NameError::ForbiddenChar);
    }
    if name.starts_with('.') || name.ends_with('.') || name.ends_with(' ') {
        return Err(NameError::BadEdge);
    }
    let stem = name.split('.').next().unwrap_or(name).to_ascii_lowercase();
    let reserved = matches!(stem.as_str(), "con" | "prn" | "aux" | "nul")
        || ((stem.starts_with("com") || stem.starts_with("lpt"))
            && stem.len() == 4
            && stem.as_bytes()[3].is_ascii_digit());
    if reserved {
        return Err(NameError::Reserved);
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../test/rust/kit.rs"]
pub(crate) mod tests;
