//! Links of an extraction or a copy: kept inside the folder, copied where the disk has none.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use super::copy::copy_tree;
use super::{no_link_on_the_way, prepare_dir};

/// The file system refuses symbolic links: exFAT and FAT answer `EPERM`
/// (Linux) or `ENOTSUP`/`EOPNOTSUPP` (macOS). Any other error is a real one.
pub(crate) fn links_unsupported(e: &io::Error) -> bool {
    #[cfg(unix)]
    if let Some(code) = e.raw_os_error() {
        return code == libc::EPERM || code == libc::ENOTSUP || code == libc::EOPNOTSUPP;
    }
    e.kind() == io::ErrorKind::Unsupported
}

/// The hard link `path` of a tar entry to `target`, a regular file already
/// checked to be inside the folder with no link on the way. Where the file
/// system has no hard links ([`links_unsupported`]: exFAT, FAT) the file is
/// copied; any other error stops the extraction. `make` creates the link
/// (`fs::hard_link`; the tests inject a refusal).
pub(super) fn hard_link_or_copy(
    target: &Path,
    path: &Path,
    make: impl FnOnce(&Path, &Path) -> io::Result<()>,
) -> Result<()> {
    match make(target, path) {
        Ok(()) => Ok(()),
        Err(e) if links_unsupported(&e) => {
            fs::copy(target, path).with_context(|| path.display().to_string())?;
            Ok(())
        }
        Err(e) => Err(e).with_context(|| path.display().to_string()),
    }
}

/// `link_target`, from the folder `rel_parent` (relative to the extraction
/// folder), can only stay inside it by its text alone: not empty, not
/// absolute, no `\` or drive, never climbing above the top. What lies on the
/// way is checked later ([`resolve_inside`]); a target failing this never
/// resolves, whatever is extracted.
fn lexically_inside(rel_parent: &Path, link_target: &str) -> bool {
    if link_target.is_empty() || link_target.starts_with('/') || link_target.contains(['\\', ':']) {
        return false;
    }
    let mut depth = rel_parent.components().count();
    for step in link_target.split('/') {
        match step {
            "" | "." => {}
            ".." if depth == 0 => return false,
            ".." => depth -= 1,
            _ => depth += 1,
        }
    }
    true
}

/// Where the link `link_path -> link_target` lands, relative to `dest`, if
/// it resolves inside `dest` the way the kernel resolves it: every name
/// crossed on the way (all but the last) must be a real folder, not a link,
/// so a link made earlier (`t -> .`, then `s -> t/..`) cannot be used to
/// climb out. `None` for an absolute target or one outside `dest`.
fn resolve_inside(dest: &Path, link_path: &Path, link_target: &str) -> Option<PathBuf> {
    if link_target.is_empty() || link_target.starts_with('/') || link_target.contains(['\\', ':']) {
        return None;
    }
    let rel_parent = link_path.parent()?.strip_prefix(dest).ok()?;
    let mut steps: Vec<&std::ffi::OsStr> = rel_parent.iter().collect();
    steps.extend(
        link_target
            .split('/')
            .filter(|s| !s.is_empty() && *s != ".")
            .map(std::ffi::OsStr::new),
    );
    let mut resolved = PathBuf::new();
    for (i, step) in steps.iter().enumerate() {
        if *step == ".." {
            if !resolved.pop() {
                return None; // Above `dest`.
            }
            continue;
        }
        resolved.push(step);
        if i + 1 < steps.len() {
            match fs::symlink_metadata(dest.join(&resolved)) {
                Ok(meta) if meta.is_dir() => {}
                _ => return None, // A link, a file or nothing: not followed.
            }
        }
    }
    Some(resolved)
}

/// Creates the symbolic links `(path, target)` collected while extracting or
/// copying into `dest`, after every file and folder exists. A link whose
/// target is outside `dest` (or crosses another link) is skipped. Where the
/// file system has no links ([`links_unsupported`]: exFAT, FAT) the target
/// is copied in place of the link, in a second pass, so a target that is
/// itself such a copy is complete first; a link whose way crossed such a
/// copy is then resolved again. `make` creates one link
/// (`symlink(target, path)`; the tests inject a refusal). Returns the links
/// skipped, for a warning.
fn finish_links(
    dest: &Path,
    links: Vec<(PathBuf, String)>,
    make: impl FnMut(&str, &Path) -> io::Result<()>,
) -> Result<Vec<String>> {
    finish_links_within(dest, links, make, COPY_EXTRA)
}

/// Bytes the copies in place of links may add beyond [`COPY_FACTOR`] times
/// the tree.
const COPY_EXTRA: u64 = 256 << 20;
/// Links to folders nested in each other (`d1/a -> ../d0`, `d2/a -> ../d1`,
/// ...) double a small archive at every step once each is a copy. The
/// copies may add this many times the tree as it was before them (plus
/// [`COPY_EXTRA`]), far above a real package: a macOS framework's
/// `Versions/Current` adds it once.
const COPY_FACTOR: u64 = 4;

/// Bytes of the files under `path`, links not followed.
fn tree_size(path: &Path) -> u64 {
    let Ok(meta) = fs::symlink_metadata(path) else {
        return 0;
    };
    if !meta.is_dir() {
        return meta.len();
    }
    fs::read_dir(path)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| tree_size(&e.path()))
        .sum()
}

/// [`finish_links`], with the copies in place of links limited to
/// [`COPY_FACTOR`] times the tree plus `extra` bytes.
fn finish_links_within(
    dest: &Path,
    links: Vec<(PathBuf, String)>,
    mut make: impl FnMut(&str, &Path) -> io::Result<()>,
    extra: u64,
) -> Result<Vec<String>> {
    let mut skipped = Vec::new();
    let mut copies: Vec<(PathBuf, PathBuf)> = Vec::new();
    // Links whose way crosses a name that is not a real folder (yet).
    let mut unresolved: Vec<(PathBuf, String)> = Vec::new();
    let mut refused = false;
    for (path, target) in links {
        let Some(parent) = path.parent() else {
            continue;
        };
        // Checked before any folder is made for it: a target that can never
        // stay inside, or a parent reached through a link, is skipped as
        // the extraction always did, never an error.
        let inside = parent
            .strip_prefix(dest)
            .is_ok_and(|rel| lexically_inside(rel, &target));
        if !inside || no_link_on_the_way(dest, parent).is_err() {
            skipped.push(format!("{} -> {target}", path.display()));
            continue;
        }
        prepare_dir(dest, parent)?;
        let Some(resolved) = resolve_inside(dest, &path, &target) else {
            unresolved.push((path, target));
            continue;
        };
        match fs::symlink_metadata(&path) {
            Ok(meta) if meta.is_dir() => continue, // A real folder stays.
            Ok(_) => fs::remove_file(&path)?,
            Err(_) => {}
        }
        match make(&target, &path) {
            Ok(()) => {}
            Err(e) if links_unsupported(&e) => {
                refused = true;
                copies.push((path, dest.join(resolved)));
            }
            Err(e) => return Err(e).with_context(|| path.display().to_string()),
        }
    }
    let mut budget = if refused {
        tree_size(dest)
            .saturating_mul(COPY_FACTOR)
            .saturating_add(extra)
    } else {
        0
    };
    loop {
        // A source holding a link not resolved yet waits for it, so the
        // copy of a folder (`d -> a`) also gets `a/x`, made only below.
        let waiting: Vec<PathBuf> = unresolved.iter().map(|(p, _)| p.clone()).collect();
        copy_in_place_of_links(&mut copies, &waiting, &mut budget, &mut skipped)?;
        // Without links, a way through a link (`x -> lib64/foo`, with
        // `lib64 -> lib`) crosses a real folder once that link is a copy:
        // resolved again, by the same rules, until nothing more resolves.
        let before = unresolved.len();
        if refused {
            unresolved.retain(|(path, target)| match resolve_inside(dest, path, target) {
                Some(resolved) if fs::symlink_metadata(path).is_err() => {
                    copies.push((path.clone(), dest.join(resolved)));
                    false
                }
                _ => true,
            });
        }
        if unresolved.len() != before {
            continue;
        }
        // Nothing ready and nothing more resolves: links that point at
        // each other, or a source holding a link that never resolves. One
        // copy goes first, as it is, and the rest is tried again.
        let there = |p: &Path| fs::symlink_metadata(p).is_ok();
        let first = copies
            .iter()
            .position(|(path, s)| {
                there(s) && !copies.iter().any(|(p, _)| p != path && p.starts_with(s))
            })
            .or_else(|| copies.iter().position(|(_, s)| there(s)));
        match first {
            Some(i) => {
                let (path, source) = copies.remove(i);
                copy_one(&path, &source, &mut budget, &mut skipped)?;
            }
            None => break,
        }
    }
    skipped.extend(
        copies
            .drain(..)
            .map(|(path, source)| format!("{} -> {}", path.display(), source.display())),
    );
    skipped.extend(
        unresolved
            .into_iter()
            .map(|(path, target)| format!("{} -> {target}", path.display())),
    );
    Ok(skipped)
}

/// The copies of [`finish_links`] in place of the links `(path, source)`
/// that are ready: the source exists and holds no copy still to make (a
/// link to a link, a folder with links) and no link still `waiting` to be
/// resolved. The others stay in `copies`. `budget`: see [`copy_one`].
fn copy_in_place_of_links(
    copies: &mut Vec<(PathBuf, PathBuf)>,
    waiting: &[PathBuf],
    budget: &mut u64,
    skipped: &mut Vec<String>,
) -> Result<()> {
    loop {
        let ready = copies.iter().position(|(path, source)| {
            fs::symlink_metadata(source).is_ok()
                && !copies
                    .iter()
                    .any(|(p, _)| p != path && p.starts_with(source))
                && !waiting.iter().any(|w| w.starts_with(source))
        });
        let Some(i) = ready else {
            return Ok(());
        };
        let (path, source) = copies.remove(i);
        copy_one(&path, &source, budget, skipped)?;
    }
}

/// One copy of [`copy_in_place_of_links`]: the file or folder `source` at
/// `path`, in place of a link. Its bytes come out of `budget` first; a copy
/// beyond it stops the extraction, nothing of it written.
fn copy_one(path: &Path, source: &Path, budget: &mut u64, skipped: &mut Vec<String>) -> Result<()> {
    let meta = fs::symlink_metadata(source)?;
    if meta.file_type().is_symlink() || path.starts_with(source) {
        // A link not followed, or a folder that would hold itself.
        skipped.push(format!("{} -> {}", path.display(), source.display()));
        return Ok(());
    }
    let size = tree_size(source);
    if size > *budget {
        bail!(
            "the file system has no links, and copying them in their place would take \
             too much space: {} -> {}",
            path.display(),
            source.display()
        );
    }
    *budget -= size;
    if meta.is_dir() {
        copy_tree(source, path).with_context(|| path.display().to_string())?;
    } else {
        fs::copy(source, path).with_context(|| path.display().to_string())?;
    }
    Ok(())
}

/// [`finish_links`] with real links on Unix, the skipped ones in the log.
/// On Windows, where a link needs a privilege, each one becomes a copy of
/// its target, as on a disk without links: the archive of another system
/// unpacked from this PC (`python/bin/python3 -> python3.13`).
pub(super) fn finish_links_here(dest: &Path, links: Vec<(PathBuf, String)>) -> Result<()> {
    let skipped = finish_links(dest, links, |target, path| {
        #[cfg(unix)]
        return std::os::unix::fs::symlink(target, path);
        #[cfg(not(unix))]
        {
            let _ = (target, path);
            Err(io::Error::from(io::ErrorKind::Unsupported))
        }
    })?;
    for link in skipped {
        crate::common::log::warn(&format!(
            "link not made (outside the folder, or through another link): {link}"
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../../test/rust/net/archive/links.rs"]
mod tests;
