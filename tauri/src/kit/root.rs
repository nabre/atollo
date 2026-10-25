//! Where the kit's root is: the folder of the program, of its macOS
//! bundle or of its Linux AppImage (D-113), and what can stop the app
//! from using it.

use std::path::{Path, PathBuf};

use super::target::Os;

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

/// The kit root for the program at `exe` on `os` (D-113):
/// on Linux the folder of the AppImage when the app runs from one
/// (`image`, its `$APPIMAGE`; the program itself is in a temporary mount),
/// on macOS the folder that holds `Atollo.app` when `exe` is its
/// `Contents/MacOS/` program (nothing is ever written inside the bundle),
/// otherwise the folder of `exe`. `None`: no folder to use.
pub fn root_from(os: Os, exe: &Path, image: Option<&Path>) -> Option<PathBuf> {
    let non_empty = |p: &Path| (!p.as_os_str().is_empty()).then(|| p.to_path_buf());
    if let Some(image) = image.filter(|i| os.is_linux() && i.has_root()) {
        return image.parent().and_then(non_empty);
    }
    if os.is_macos() {
        if let Some(bundle) = app_bundle(exe) {
            return bundle.parent().and_then(non_empty);
        }
    }
    exe.parent().and_then(non_empty)
}

/// The `*.app` bundle whose program is `exe` (`X.app/Contents/MacOS/x`).
pub(super) fn app_bundle(exe: &Path) -> Option<&Path> {
    let macos = exe.parent()?;
    let contents = macos.parent()?;
    let bundle = contents.parent()?;
    let named = |p: &Path, n: &str| p.file_name().is_some_and(|f| f == n);
    let is_app = bundle
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("app"));
    (named(macos, "MacOS") && named(contents, "Contents") && is_app).then_some(bundle)
}

/// The AppImage the app runs from: `$APPIMAGE`, when `exe` is inside the
/// image's mount (`$APPDIR`). A plain binary started from a program that
/// itself runs from an AppImage inherits both variables, but is not inside
/// that mount.
#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
pub(super) fn appimage_of(
    exe: &Path,
    image: Option<&std::ffi::OsStr>,
    appdir: Option<&std::ffi::OsStr>,
) -> Option<PathBuf> {
    let (image, appdir) = (image?, appdir?);
    (!image.is_empty() && !appdir.is_empty() && exe.starts_with(appdir))
        .then(|| PathBuf::from(image))
}

#[cfg(target_os = "linux")]
pub(super) fn running_appimage(exe: &Path) -> Option<PathBuf> {
    appimage_of(
        exe,
        std::env::var_os("APPIMAGE").as_deref(),
        std::env::var_os("APPDIR").as_deref(),
    )
}

#[cfg(not(target_os = "linux"))]
pub(super) fn running_appimage(_exe: &Path) -> Option<PathBuf> {
    None
}

/// The program the kit's wrappers start to run the app again
/// (`ATOLLO_EXE`): the AppImage when the app runs from one, whose inner
/// program goes away with its mount; otherwise this program.
pub fn app_exe() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    Some(running_appimage(&exe).unwrap_or(exe))
}

/// The app bundle macOS runs from a random read-only copy (App
/// Translocation, a downloaded app not yet moved or cleared of its
/// quarantine): its name (`Atollo.app`), or `None` for a normal start.
pub fn translocated_bundle(exe: &Path) -> Option<String> {
    let moved = exe
        .components()
        .any(|c| c.as_os_str() == "AppTranslocation");
    moved.then(|| {
        app_bundle(exe).and_then(Path::file_name).map_or_else(
            || "Atollo.app".to_owned(),
            |n| n.to_string_lossy().into_owned(),
        )
    })
}

/// What stops the app at start before the kit is touched (D-113): on
/// macOS a translocated bundle, whose folder is a temporary read-only copy
/// where a kit would be created and lost. The text says how to fix it
/// (D-113: no Developer ID signature, so the quarantine flag is cleared).
pub fn launch_problem(os: Os, exe: &Path, lang: &str) -> Option<String> {
    if !os.is_macos() {
        return None;
    }
    let bundle = translocated_bundle(exe)?;
    let quoted = crate::platform::sh_quote(&bundle);
    Some(format!(
        "{}\n\n{}",
        crate::common::i18n::t(lang, "macOS ha aperto Atollo da una copia temporanea in sola lettura (App Translocation): lì il kit non può tenere i suoi dati.", &[]),
        crate::common::i18n::t(lang, "Esci da Atollo, apri il Terminale nella cartella del kit ed esegui: xattr -dr com.apple.quarantine {0}. Poi riapri l'app dalla cartella del kit.", &[&quoted]),
    ))
}

/// `\\?\C:\x` -> `C:\x` (keeps `\\?\UNC\` and other forms unchanged).
pub(crate) fn strip_verbatim(path: &Path) -> PathBuf {
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
    watcher_unsafe_folder_on(Os::current(), path)
}

/// [`watcher_unsafe_folder`] on `os`. Only Windows paths have backslashes:
/// on macOS and Linux the globs hold `/` and every folder name is fine
/// (`/home/user/2026` would otherwise be reported for `user` and `2026`).
pub fn watcher_unsafe_folder_on(os: Os, path: &Path) -> Option<String> {
    if !os.is_windows() {
        return None;
    }
    path.components().find_map(|c| match c {
        std::path::Component::Normal(name) => {
            let name = name.to_string_lossy();
            let first = name.chars().next()?;
            (first.is_ascii_digit() || first == 'x' || first == 'u').then(|| name.into_owned())
        }
        _ => None,
    })
}
