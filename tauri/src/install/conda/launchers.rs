//! `kit-bin` of a conda environment (D-125): a launcher for each program of
//! the component's own package, on the PATH in place of the whole `bin`.

use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde_json::Value;

use crate::install::installer;
use crate::kit::Kit;
use crate::setup::Catalog;

/// The folder of a conda environment with the launchers of the
/// component's own programs (D-125): on the PATH in place of `bin`, which
/// also holds those of every dependency (`openssl`, `tput`, `kinit`,
/// `envsubst`... 30 programs for Ruby's 16, 56 for MongoDB's 2, 156 for
/// R's 2).
pub const LAUNCHERS: &str = "kit-bin";

/// The programs of `bin/` that `package` put in the environment `prefix`,
/// from its record in `conda-meta` (`files`), sorted.
pub fn package_programs(prefix: &Path, package: &str) -> Result<Vec<String>> {
    let dir = prefix.join("conda-meta");
    for entry in fs::read_dir(&dir).with_context(|| dir.display().to_string())? {
        let path = entry?.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        // `<name>-<version>-<build>.json`: only the records that can be it.
        let Some(rest) = name.strip_prefix(package).and_then(|r| r.strip_prefix('-')) else {
            continue;
        };
        if !rest.starts_with(|c: char| c.is_ascii_digit()) || !name.ends_with(".json") {
            continue;
        }
        let meta: Value = serde_json::from_slice(&fs::read(&path)?)
            .with_context(|| path.display().to_string())?;
        if meta.get("name").and_then(Value::as_str) != Some(package) {
            continue;
        }
        let mut programs: Vec<String> = meta
            .get("files")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .filter_map(|f| f.strip_prefix("bin/"))
            .filter(|p| !p.is_empty() && !p.contains('/'))
            .map(str::to_owned)
            .collect();
        programs.sort();
        return Ok(programs);
    }
    bail!("{}: no record of {package}", dir.display())
}

/// The launcher of `program` in `kit-bin`: the program of `bin` next to
/// it, with that `bin` first in the PATH of the program only (its own
/// tools, as when the whole `bin` was on the kit's PATH).
pub fn launcher_text(program: &str) -> String {
    format!(
        "#!/bin/sh\n# Atollo: {program} of the conda-forge environment, with its bin in its PATH only\nd=$(cd \"$(dirname \"$0\")/..\" && pwd)\nPATH=\"$d/bin:$PATH\" exec \"$d/bin/{program}\" \"$@\"\n"
    )
}

/// `kit-bin` of `prefix` made again with a launcher for each program of
/// `package`; nothing else in it.
pub fn write_launchers(prefix: &Path, package: &str) -> Result<()> {
    let programs = package_programs(prefix, package)?;
    if programs.is_empty() {
        bail!("{package}: no program in bin");
    }
    let dir = prefix.join(LAUNCHERS);
    if dir.exists() {
        fs::remove_dir_all(&dir).with_context(|| dir.display().to_string())?;
    }
    for program in &programs {
        let path = dir.join(program);
        crate::platform::files::write_executable(&path, launcher_text(program).as_bytes())
            .with_context(|| path.display().to_string())?;
    }
    Ok(())
}

/// The launchers of the environments installed before the kit wrote them
/// (at start, `versions::side_by_side::refresh`): every line of every
/// component with [`Installer::conda_programs`], where `kit-bin` is missing.
///
/// [`Installer::conda_programs`]: crate::install::Installer::conda_programs
pub fn launchers_of_older_installations(kit: &Kit) {
    let target = kit.target();
    for c in Catalog::get().all() {
        let Some(inst) = installer(&c.id) else {
            continue;
        };
        let Some(package) = inst.conda_programs(target) else {
            continue;
        };
        let lines = crate::versions::default_line(kit, c)
            .into_iter()
            .chain(crate::versions::saved(kit, c).into_iter().map(|l| l.line));
        for line in lines {
            let Some(prefix) = crate::versions::dir_for(kit, c, Some(&line))
                .and_then(|dir| inst.conda_prefix(target, &dir))
            else {
                continue;
            };
            if prefix.join(LAUNCHERS).is_dir() || !prefix.join("conda-meta").is_dir() {
                continue;
            }
            if let Err(e) = write_launchers(&prefix, package) {
                crate::common::log::warn(&format!("{}@{line}: launchers: {e:#}", c.id));
            }
        }
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/install/conda/launchers.rs"]
mod tests;
