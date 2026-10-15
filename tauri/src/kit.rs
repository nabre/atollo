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

/// Processor platform of the downloaded programs (`data\apps\win32-<arch>`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Arch {
    X64,
    Arm64,
}

impl Arch {
    /// Same rule as `_env.cmd`: ARM64 when the process or its WOW64 host
    /// reports ARM64, otherwise x64.
    pub fn detect() -> Arch {
        Self::from_env_values(
            std::env::var("PROCESSOR_ARCHITECTURE").ok().as_deref(),
            std::env::var("PROCESSOR_ARCHITEW6432").ok().as_deref(),
        )
    }

    fn from_env_values(arch: Option<&str>, wow64: Option<&str>) -> Arch {
        let is_arm = |v: Option<&str>| v.is_some_and(|v| v.eq_ignore_ascii_case("ARM64"));
        if is_arm(arch) || is_arm(wow64) {
            Arch::Arm64
        } else {
            Arch::X64
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Arch::X64 => "x64",
            Arch::Arm64 => "arm64",
        }
    }

    /// `win32-x64` or `win32-arm64`, the folder name used under `data\apps`.
    pub fn platform(self) -> String {
        format!("win32-{}", self.as_str())
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
    arch: Arch,
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
        Ok(Kit::at(root.to_path_buf(), Arch::detect()))
    }

    pub fn at(root: PathBuf, arch: Arch) -> Kit {
        Kit {
            root: strip_verbatim(&root),
            arch,
            system_temp: None,
            volume: None,
        }
    }

    /// The kit with what the host gives at start: its temporary folder
    /// (`TEMP`/`TMP` of the real process, read before D-04 moves them into
    /// `data\`) and the disk of the root. Without them the kit works as before:
    /// every temporary file in `data\tmp`.
    pub fn with_host(mut self, host_temp: Option<&Path>) -> Kit {
        self.system_temp = host_temp
            .filter(|t| t.is_absolute())
            .map(|t| strip_verbatim(t).join("Dev-Kit").join(self.id()));
        self.volume = crate::os::volume::info(&self.root).ok();
        self
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn arch(&self) -> Arch {
        self.arch
    }
    pub fn data(&self) -> PathBuf {
        self.root.join("data")
    }
    pub fn projects(&self) -> PathBuf {
        self.root.join("projects")
    }
    /// The kit's files of each project, one folder per project (D-89).
    #[allow(dead_code)] // PLAN-DATA.md WP-0 contract, used from wave 1; removed in WP-14
    pub fn project_data(&self) -> PathBuf {
        self.data().join("projects")
    }
    /// `<host TEMP>\Dev-Kit\<id>`: files in transit only (D-90); `None` when
    /// the host gave no temporary folder.
    #[allow(dead_code)] // PLAN-DATA.md WP-0 contract, used from wave 1; removed in WP-14
    pub fn system_temp(&self) -> Option<&Path> {
        self.system_temp.as_deref()
    }
    /// The disk of the root, when it could be read.
    #[allow(dead_code)] // PLAN-DATA.md WP-0 contract, used from wave 1; removed in WP-14
    pub fn volume(&self) -> Option<VolumeInfo> {
        self.volume
    }
    /// Downloaded programs for this processor: `data\apps\win32-<arch>`.
    pub fn apps(&self) -> PathBuf {
        self.data().join("apps").join(self.arch.platform())
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
    /// the last start (another drive letter or folder).
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
        self.check_moved().map_err(|source| KitError::NotWritable {
            path: self.app_dir(),
            source,
        })
    }

    /// `data\app\root.txt` keeps the last root. When it differs, the old one is
    /// also saved in `data\app\old-root.txt` (kept until the paths in the
    /// agents' files are rewritten, phase 5), like `old-root-win32.txt`.
    fn check_moved(&self) -> io::Result<Option<PathBuf>> {
        let record = self.app_dir().join("root.txt");
        let now = self.root.to_string_lossy().into_owned();
        let last = fs::read_to_string(&record).ok();
        let moved = match last.as_deref().map(str::trim) {
            Some(last) if !last.is_empty() && !last.eq_ignore_ascii_case(&now) => {
                let pending = self.app_dir().join("old-root.txt");
                if !pending.exists() {
                    write_atomic(&pending, last.as_bytes())?;
                }
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
pub(crate) mod tests {
    use super::*;

    /// A fresh folder under the system temp, removed on drop.
    pub struct TempDir(pub PathBuf);
    impl TempDir {
        pub fn new(name: &str) -> TempDir {
            let dir = std::env::temp_dir().join(format!(
                "devkit-test-{}-{}-{name}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir_all(&dir).unwrap();
            TempDir(dir)
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn arch_follows_env_rule() {
        assert_eq!(Arch::from_env_values(Some("AMD64"), None), Arch::X64);
        assert_eq!(Arch::from_env_values(Some("ARM64"), None), Arch::Arm64);
        assert_eq!(
            Arch::from_env_values(Some("x86"), Some("arm64")),
            Arch::Arm64
        );
        assert_eq!(Arch::from_env_values(None, None), Arch::X64);
    }

    #[test]
    fn root_is_exe_folder_with_odd_names() {
        for name in [
            "Dev Kit",
            "kit (2)",
            "a&b",
            "percent%20",
            "già è",
            r"x\very\deep\path\to\the\kit",
        ] {
            let exe = PathBuf::from(r"E:\").join(name).join("Dev-Kit.exe");
            let kit = Kit::from_exe(&exe).unwrap();
            assert_eq!(kit.root(), exe.parent().unwrap());
            assert_eq!(kit.data(), exe.parent().unwrap().join("data"));
        }
    }

    #[test]
    fn verbatim_prefix_is_removed() {
        let kit = Kit::from_exe(Path::new(r"\\?\D:\Kit\Dev-Kit.exe")).unwrap();
        assert_eq!(kit.root(), Path::new(r"D:\Kit"));
    }

    #[test]
    fn apps_folder_per_platform() {
        let kit = Kit::at(PathBuf::from(r"F:\K"), Arch::Arm64);
        assert_eq!(kit.apps(), PathBuf::from(r"F:\K\data\apps\win32-arm64"));
    }

    #[test]
    fn id_ignores_case() {
        let a = Kit::at(PathBuf::from(r"E:\Kit"), Arch::X64);
        let b = Kit::at(PathBuf::from(r"e:\kit"), Arch::X64);
        assert_eq!(a.id(), b.id());
        assert_eq!(a.id().len(), 16);
    }

    #[test]
    fn host_temp_and_disk() {
        let tmp = TempDir::new("with-host");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        assert_eq!(kit.system_temp(), None);
        assert_eq!(kit.project_data(), tmp.0.join(r"data\projects"));
        let host = kit
            .clone()
            .with_host(Some(Path::new(r"C:\Users\u\AppData\Local\Temp")));
        assert_eq!(
            host.system_temp().unwrap(),
            Path::new(r"C:\Users\u\AppData\Local\Temp\Dev-Kit").join(kit.id())
        );
        assert!(host.volume().is_some_and(|v| v.free > 0));
        // a relative TEMP is not used: everything stays in data\tmp
        assert_eq!(kit.with_host(Some(Path::new("Temp"))).system_temp(), None);
    }

    #[test]
    fn prepare_creates_only_inside_root_and_detects_move() {
        let tmp = TempDir::new("prepare");
        let first = Kit::at(tmp.0.join("one (1) & co"), Arch::X64);
        assert_eq!(first.prepare().unwrap(), None);
        assert!(first.projects().is_dir() && first.logs().is_dir());
        assert_eq!(first.prepare().unwrap(), None, "same root: not moved");

        // Same data folder seen from another root (drive letter change).
        let moved_root = tmp.0.join("two");
        fs::rename(first.root(), &moved_root).unwrap();
        let moved = Kit::at(moved_root, Arch::X64);
        assert_eq!(moved.prepare().unwrap(), Some(first.root().to_path_buf()));
        let pending = fs::read_to_string(moved.app_dir().join("old-root.txt")).unwrap();
        assert_eq!(Path::new(&pending), first.root());
        assert_eq!(moved.prepare().unwrap(), None, "recorded: not moved again");

        let entries: Vec<_> = fs::read_dir(&tmp.0).unwrap().collect();
        assert_eq!(entries.len(), 1, "nothing created next to the kit");
    }

    #[test]
    fn folders_that_break_the_editor_watcher() {
        assert_eq!(watcher_unsafe_folder(Path::new(r"E:\Dev-Kit")), None);
        assert_eq!(
            watcher_unsafe_folder(Path::new(r"E:\Dev Kit (1) & co")),
            None
        );
        assert_eq!(
            watcher_unsafe_folder(Path::new(r"C:\Users\Mario\code\kit")),
            None
        );
        assert_eq!(
            watcher_unsafe_folder(Path::new(r"D:\2026\kit")).as_deref(),
            Some("2026")
        );
        assert_eq!(
            watcher_unsafe_folder(Path::new(r"D:\x-kit")).as_deref(),
            Some("x-kit")
        );
        assert_eq!(
            watcher_unsafe_folder(Path::new(r"C:\users\a")).as_deref(),
            Some("users")
        );
    }

    #[test]
    fn project_names() {
        for ok in [
            "app",
            "my app",
            "Progetto è",
            "a.b",
            "con1",
            "comx",
            "x".repeat(100).as_str(),
        ] {
            assert_eq!(validate_project_name(ok), Ok(()), "{ok}");
        }
        assert_eq!(validate_project_name(""), Err(NameError::Empty));
        assert_eq!(
            validate_project_name(&"x".repeat(101)),
            Err(NameError::TooLong)
        );
        for bad in [
            "a/b", r"a\b", "a:b", "a*", "a?", "a\"b", "<a>", "a|b", "a\u{7}",
        ] {
            assert_eq!(
                validate_project_name(bad),
                Err(NameError::ForbiddenChar),
                "{bad}"
            );
        }
        for bad in [".git", "a.", "a "] {
            assert_eq!(validate_project_name(bad), Err(NameError::BadEdge), "{bad}");
        }
        for bad in ["CON", "nul", "Aux.txt", "COM1", "lpt9.log"] {
            assert_eq!(
                validate_project_name(bad),
                Err(NameError::Reserved),
                "{bad}"
            );
        }
    }
}
