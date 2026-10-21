//! After an installation from the network: the archives of the new
//! environment copied out of micromamba's cache into the kit, with the
//! explicit list and the record of the line.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;

use super::list::{explicit_text, is_sha256, parse_explicit, safe_file, Archive, Package, Record};
use super::{lock_name, locks_dir, pkgs_dir};
use crate::fsx::write_atomic;
use crate::install::native::run_output;
use crate::install::{installer, Ctx, Progress};
use crate::net::verify::{self, Expect};
use crate::settings::Settings;
use crate::setup::catalog::Component;

/// The packages of `component` are kept: it cannot follow the kit on this
/// target (D-92) and the setting is on (the default).
pub(in crate::install) fn wanted(ctx: &Ctx, component: &Component) -> bool {
    let target = ctx.kit.target();
    installer(&component.id).is_some_and(|i| !i.relocatable(target))
        && Settings::load(ctx.kit).settings.keep_conda_packages
}

/// After `micromamba create` from the network: the archives of the
/// environment at `prefix`, found in micromamba's package cache `cache`,
/// copied into the kit with the lists of the line of `version`. A failure
/// leaves the environment as it is, without the copy: a warning only.
#[allow(clippy::too_many_arguments)]
pub(in crate::install) fn keep(
    ctx: &Ctx,
    component: &Component,
    version: &str,
    mm: &Path,
    envs: &[(&str, &Path)],
    prefix: &Path,
    cache: &Path,
    progress: &mut dyn Progress,
) {
    match keep_lists(ctx, component, version, mm, envs, prefix, cache) {
        Ok(bytes) => progress.note(&format!(
            "packages kept to reinstall without the network: {} MB",
            bytes.div_ceil(1 << 20)
        )),
        Err(e) => {
            let text = format!(
                "{}: packages not kept, a move of the kit will need the network ({e:#})",
                component.id
            );
            crate::log::warn(&text);
            progress.note(&text);
        }
    }
}

/// What `conda-meta` says of each package of an environment, by URL:
/// file name, SHA-256 and size, from conda-forge's metadata.
fn conda_meta(prefix: &Path) -> Result<BTreeMap<String, Package>> {
    let dir = prefix.join("conda-meta");
    let mut out = BTreeMap::new();
    for entry in fs::read_dir(&dir).with_context(|| dir.display().to_string())? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(meta) = serde_json::from_slice::<Value>(&fs::read(&path)?) else {
            continue;
        };
        let text = |k: &str| meta.get(k).and_then(Value::as_str).map(str::to_owned);
        let (Some(url), Some(file), Some(sha256)) = (text("url"), text("fn"), text("sha256"))
        else {
            continue;
        };
        let size = meta.get("size").and_then(Value::as_u64).unwrap_or(0);
        out.insert(url, Package { file, sha256, size });
    }
    Ok(out)
}

/// `src` into `dest` of `pkgs/`, unless it is there already with `sha256`;
/// the copy is checked before it takes the name.
fn store(src: &Path, dest: &Path, sha256: &str) -> Result<()> {
    let expect = Expect::Sha256(sha256.to_owned());
    if dest.is_file() && verify::matches(dest, &expect)? {
        return Ok(());
    }
    let part = dest.with_extension("part");
    fs::copy(src, &part).with_context(|| part.display().to_string())?;
    verify::check(&part, &expect)?;
    fs::rename(&part, dest).with_context(|| dest.display().to_string())?;
    Ok(())
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn keep_lists(
    ctx: &Ctx,
    component: &Component,
    version: &str,
    mm: &Path,
    envs: &[(&str, &Path)],
    prefix: &Path,
    cache: &Path,
) -> Result<u64> {
    let prefix_arg = prefix.display().to_string();
    // micromamba's order, the one it installs an explicit list in.
    let exported = run_output(
        ctx,
        mm,
        &["env", "export", "--no-rc", "--explicit", "-p", &prefix_arg],
        envs,
    )?;
    let urls: Vec<&str> = exported
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("https://"))
        .map(|l| l.split('#').next().unwrap_or(l))
        .collect();
    if urls.is_empty() {
        bail!("micromamba listed no packages");
    }
    let meta = conda_meta(prefix)?;
    let pkgs = pkgs_dir(ctx.kit);
    fs::create_dir_all(&pkgs).with_context(|| pkgs.display().to_string())?;
    let mut archives = Vec::with_capacity(urls.len());
    let mut packages = Vec::with_capacity(urls.len());
    let mut total = 0;
    for url in urls {
        let m = meta
            .get(url)
            .ok_or_else(|| anyhow!("{url}: not in conda-meta"))?;
        if !safe_file(&m.file) || !url.ends_with(&format!("/{}", m.file)) || !is_sha256(&m.sha256) {
            bail!("{url}: unexpected file or hash in conda-meta");
        }
        let found = crate::install::find_file(cache, &m.file)
            .ok_or_else(|| anyhow!("{}: not in micromamba's cache", m.file))?;
        let dest = pkgs.join(&m.file);
        store(&found, &dest, &m.sha256)?;
        let size = fs::metadata(&dest)?.len();
        if m.size != 0 && m.size != size {
            bail!("{}: {size} bytes, conda-meta says {}", m.file, m.size);
        }
        total += size;
        archives.push(Archive {
            url: url.to_owned(),
            file: m.file.clone(),
            sha256: m.sha256.to_ascii_lowercase(),
        });
        packages.push(Package {
            file: m.file.clone(),
            sha256: m.sha256.to_ascii_lowercase(),
            size,
        });
    }
    let target = ctx.kit.target();
    let text = explicit_text(&archives, target.conda_subdir());
    // The same rules as the reading: a list the kit would refuse is not kept.
    parse_explicit(&text)?;
    let record = Record {
        version: version.to_owned(),
        target: target.as_str().to_owned(),
        created: now(),
        micromamba: run_output(ctx, mm, &["--version"], envs)
            .ok()
            .map(|v| v.trim().to_owned()),
        packages,
    };
    let locks = locks_dir(ctx.kit);
    fs::create_dir_all(&locks).with_context(|| locks.display().to_string())?;
    let name = lock_name(&component.id, &crate::versions::dir_name(version));
    write_atomic(
        &locks.join(format!("{name}.json")),
        serde_json::to_string_pretty(&record)?.as_bytes(),
    )?;
    write_atomic(&locks.join(format!("{name}.txt")), text.as_bytes())?;
    Ok(total)
}
