//! The kit on disk: its root (the folder that holds `Atollo.exe`), the
//! processor platform and every path the app uses.
//!
//! Layout (DECISIONS.md D-00): `Atollo.exe`, `projects\` and `data\` side by
//! side; everything the kit downloads or writes lives in `data\`. Nothing is
//! ever created outside the root. On macOS the root holds `Atollo.app`, on
//! Linux the AppImage (`root_from`). The platform rule and the "kit was moved"
//! record are the ones earlier kits used.

//!
//! The parts: `target` (the systems and processors a kit is for: Os,
//! Arch, Target), `root` (where the root is, from the program's path, a
//! macOS bundle or a Linux AppImage), `names` (the valid project names).

pub mod backup;
mod names;
mod root;
mod target;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::common::fsx::{read_text, write_atomic};
use crate::platform::volume::VolumeInfo;

pub use names::validate_project_name;
pub use root::{app_exe, launch_problem, root_from, watcher_unsafe_folder};
pub(crate) use root::{same_root, strip_verbatim};
pub use target::{unsupported_host, Arch, Os, Target};

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
    /// `<host TEMP>\Atollo\<id>`, for files in transit (D-90).
    system_temp: Option<PathBuf>,
    /// The disk of the root, read once at start.
    volume: Option<VolumeInfo>,
}

impl Kit {
    /// The kit of the program at `exe`: the folder of `exe`, or the folder
    /// next to `Atollo.app` or the AppImage ([`root_from`]).
    pub fn from_exe(exe: &Path) -> Result<Kit, KitError> {
        let exe = strip_verbatim(exe);
        let image = crate::platform::host::running_appimage(&exe);
        let root = root_from(Os::current(), &exe, image.as_deref())
            .ok_or_else(|| KitError::NoRoot(exe.clone()))?;
        Ok(Kit::at_target(root, Target::detect()))
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
    /// A `TEMP` that is already the kit's (an Atollo.exe started from a kit
    /// console without `CP_HOST_TEMP`) is not the host's: inside the root it
    /// is ignored, inside `Atollo\<id>` it is taken back to its base.
    pub fn with_host(mut self, host_temp: Option<&Path>) -> Kit {
        let id = self.id();
        let mine = Path::new("Atollo").join(&id);
        self.system_temp = host_temp
            .filter(|t| t.is_absolute())
            .map(strip_verbatim)
            .map(|t| match t.ancestors().find(|a| a.ends_with(&mine)) {
                Some(a) => a.to_path_buf(),
                None => t.join(&mine),
            })
            .filter(|t| !t.starts_with(&self.root));
        self.volume = crate::platform::volume::info(&self.root).ok();
        self
    }

    /// The same kit root scoped to `target` (`data/apps/<target>`,
    /// `data/app/versions/<target>`), preserving host temp and volume info.
    pub fn for_target(&self, target: Target) -> Kit {
        Kit {
            root: self.root.clone(),
            target,
            system_temp: self.system_temp.clone(),
            volume: self.volume,
        }
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
    /// `<host TEMP>\Atollo\<id>`: files in transit only (D-90); `None` when
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
        self.versions_root().join(self.target.as_str())
    }
    /// `data/app/versions`: the folders of the targets, the app's own
    /// `app.txt` and the component files from before the targets.
    pub fn versions_root(&self) -> PathBuf {
        self.app_dir().join("versions")
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
    /// Before D-89: `data\app\projects`. Read only by the migration (D-89).
    pub fn project_states(&self) -> PathBuf {
        self.app_dir().join("projects")
    }

    /// First 16 hex chars of SHA-256 of the root as this PC compares it
    /// (lower-case on Windows and macOS: the same scheme earlier kits used
    /// for their per-kit mutex names). Exact on Linux, where `/media/u/Kit`
    /// and `/media/u/kit` are two kits and must not share the instance
    /// socket or `<TEMP>/Atollo/<id>` (BUG-V7-18). The keyring seal keeps
    /// the id it was made with (`SEAL_ID_FILE`).
    pub fn id(&self) -> String {
        let key = crate::platform::processes::path_key(&self.root.to_string_lossy());
        let digest = Sha256::digest(key.as_bytes());
        crate::common::hex(&digest[..8])
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
        let (record, pending) = self.root_files();
        let now = self.root.to_string_lossy().into_owned();
        let last = read_text(&record).ok();
        let moved = match last.as_deref().map(str::trim) {
            Some(last) if !last.is_empty() && !same_root(self.target.os, last, &now) => {
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

    /// The last root of this operating system and the old root waiting for
    /// `agents::relocate` after a move (see [`Self::check_moved`]).
    fn root_files(&self) -> (PathBuf, PathBuf) {
        let (record, pending) = match self.target.os {
            Os::Win32 => ("root.txt", "old-root.txt"),
            Os::Darwin => ("root-darwin.txt", "old-root-darwin.txt"),
            Os::Linux => ("root-linux.txt", "old-root-linux.txt"),
        };
        (self.app_dir().join(record), self.app_dir().join(pending))
    }

    /// The old root saved by a move, until the agents' files follow it.
    pub fn old_root_file(&self) -> PathBuf {
        self.root_files().1
    }
}

/// The message for a kit whose folders cannot be created (D-113). On
/// macOS and Linux a read-only disk (a locked card, a read-only mount, an
/// image) is named as such; Windows keeps its message.
pub fn prepare_error_message(os: Os, error: &KitError, lang: &str) -> String {
    let path = match error {
        KitError::NotWritable { path, .. } | KitError::NoRoot(path) => path.display().to_string(),
    };
    let read_only = matches!(error, KitError::NotWritable { source, .. }
        if source.kind() == io::ErrorKind::ReadOnlyFilesystem);
    let cannot = crate::common::i18n::t(lang, "Impossibile scrivere nella cartella {0}.", &[&path]);
    if os.is_windows() {
        return format!(
            "{cannot}\n\n{}",
            crate::common::i18n::t(lang, "Sposta Atollo.exe in una cartella dove puoi scrivere (per esempio una chiavetta o una cartella del tuo profilo) e riavvialo.", &[])
        );
    }
    if read_only {
        return format!(
            "{}\n\n{}",
            crate::common::i18n::t(lang, "Il disco del kit è in sola lettura: impossibile scrivere nella cartella {0}.", &[&path]),
            crate::common::i18n::t(lang, "Copia la cartella del kit su un disco dove puoi scrivere, o rimonta il disco in lettura e scrittura, e riavvia Atollo.", &[])
        );
    }
    format!(
        "{cannot}\n\n{}",
        crate::common::i18n::t(lang, "Sposta la cartella del kit dove puoi scrivere (per esempio una chiavetta o una cartella del tuo profilo) e riavvia Atollo.", &[])
    )
}

#[cfg(test)]
#[path = "../../../test/rust/kit/mod.rs"]
pub(crate) mod tests;
