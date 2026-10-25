//! The parts of a release: each downloaded, checked and placed in its
//! folder, a file or an archive.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::net::archive::{self, Kind};
use crate::platform::signature::Verdict;
use crate::setup::Component;

use super::flow::fetch;
use super::keys::check_key_for;
use super::{Ctx, Dest, Installer, Part, Progress};

/// Every part of a release downloaded, checked and extracted, each in its
/// own `work\<i>`, before the first one is put in place: a download that
/// fails (the FPM archive of PHP, Composer) leaves the installed folder as
/// it was, never a new `bin/php` without its `sbin/php-fpm` or
/// `composer.phar` (issue #4).
pub(crate) fn install_parts(
    ctx: &Ctx,
    inst: &dyn Installer,
    component: &Component,
    parts: &[&Part],
    own: Option<&Path>,
    work: &Path,
    progress: &mut dyn Progress,
) -> Result<()> {
    let mut trees = Vec::with_capacity(parts.len());
    for (i, part) in parts.iter().enumerate() {
        trees.push(fetch_part(ctx, part, &work.join(i.to_string()), progress)?);
    }
    for (part, tree) in parts.iter().zip(&trees) {
        place_part(ctx, inst, component, part, tree, own)?;
    }
    Ok(())
}

/// One part downloaded and put in place at once (the bootstrap's tools,
/// the parts of MongoDB after its environment).
#[allow(clippy::too_many_arguments)]
pub(crate) fn install_part(
    ctx: &Ctx,
    inst: &dyn Installer,
    component: &Component,
    part: &Part,
    own: Option<&Path>,
    work: &Path,
    progress: &mut dyn Progress,
) -> Result<()> {
    let tree = fetch_part(ctx, part, work, progress)?;
    place_part(ctx, inst, component, part, &tree, own)
}

/// Downloads and verifies `part` into `work` and extracts it: the tree it
/// brings (its signature checked on Windows), nothing in place yet.
pub(super) fn fetch_part(
    ctx: &Ctx,
    part: &Part,
    work: &Path,
    progress: &mut dyn Progress,
) -> Result<PathBuf> {
    fs::create_dir_all(work)?;
    let file = fetch(
        ctx,
        &part.url,
        &part.file_name,
        &part.expect,
        work,
        progress,
    )?;
    let extracted = work.join("x");
    archive::extract(&file, part.kind, &extracted)?;
    if part.kind != Kind::File {
        let _ = fs::remove_file(&file);
    }
    let tree = archive::pick_inner(&extracted, &part.inner)?;
    if ctx.kit.os().is_windows() {
        if let Some(sign) = &part.sign {
            let file = crate::common::fsx::join_rel(&tree, &sign.file);
            match crate::platform::signature::check(&file, sign.publisher, sign.required)? {
                Some(Verdict::Valid { subject }) => {
                    progress.note(&format!("signature ok: {subject}"))
                }
                Some(Verdict::RevocationUnknown { subject }) => {
                    progress.note(&format!(
                        "signature of {subject}: revocation not verifiable (offline?)"
                    ));
                }
                None => {}
            }
        }
    }
    Ok(tree)
}

/// Puts the tree [`fetch_part`] extracted where `part` goes.
pub(super) fn place_part(
    ctx: &Ctx,
    inst: &dyn Installer,
    component: &Component,
    part: &Part,
    tree: &Path,
    own: Option<&Path>,
) -> Result<()> {
    let tree = tree.to_path_buf();
    let apps = ctx.kit.apps();
    match &part.dest {
        Dest::Own => {
            let Some(folder) = own else {
                bail!("{} has no folder of its own", component.id);
            };
            check_key_for(ctx.kit.target(), component, inst.inside(), &tree)?;
            inst.before_swap(ctx, component, &tree, folder)?;
            crate::platform::files::ensure_tree_executables(ctx.kit.target(), &tree);
            if let Some(parent) = folder.parent() {
                archive::clean_leftovers(parent);
            }
            // From the PC's temporary folder: checked, copied next to the
            // folder, then swapped (on the same disk, renamed as before).
            archive::stage_into(&tree, folder)
        }
        Dest::Folder(name) => {
            let folder = apps.join(name);
            crate::platform::files::ensure_tree_executables(ctx.kit.target(), &tree);
            archive::clean_leftovers(&apps);
            archive::stage_into(&tree, &folder)
        }
        Dest::Tools(names) => {
            let tools = apps.join("tools");
            fs::create_dir_all(&tools)?;
            for name in names {
                let found = find_file(&tree, name)
                    .with_context(|| format!("{name} not found in {}", part.file_name))?;
                let dest = tools.join(name);
                put_file(&found, &dest)?;
                let _ = crate::platform::files::set_executable(&dest);
            }
            Ok(())
        }
        Dest::File { folder, name, from } => {
            let src = crate::common::fsx::join_rel(&tree, from);
            if !src.is_file() {
                bail!("{from} not found in {}", part.file_name);
            }
            // The component's own folder: that of its line (Claude Code).
            let dir = match own {
                Some(o) if component.folder.as_deref() == Some(*folder) => o.to_path_buf(),
                _ => apps.join(folder),
            };
            fs::create_dir_all(&dir)?;
            let dest = dir.join(name);
            put_file(&src, &dest)?;
            let _ = crate::platform::files::set_executable(&dest);
            Ok(())
        }
        Dest::Inside(name) => {
            let Some(folder) = own else {
                bail!("{} has no folder of its own", component.id);
            };
            let found = find_file(&tree, name)
                .with_context(|| format!("{name} not found in {}", part.file_name))?;
            let dest = folder.join(name);
            put_file(&found, &dest)?;
            let _ = crate::platform::files::set_executable(&dest);
            Ok(())
        }
    }
}

/// One file of the new tree in place of `dest`, always copied: checked
/// against the disk of `dest` first, so the installed one stays when it
/// cannot fit.
pub(crate) fn put_file(new: &Path, dest: &Path) -> Result<()> {
    archive::check_fits(new, dest)?;
    archive::check_space(new, dest)?;
    archive::replace_file(new, dest)
}

/// First file called `name` (case-insensitive) under `dir`.
pub fn find_file(dir: &Path, name: &str) -> Option<PathBuf> {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in fs::read_dir(&d).ok()?.flatten() {
            let path = entry.path();
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                stack.push(path);
            } else if entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case(name)
            {
                return Some(path);
            }
        }
    }
    None
}
