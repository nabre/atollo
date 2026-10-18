//! Ruby, from RubyInstaller2's releases on Windows and conda-forge on macOS
//! and Linux.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;

use super::{s, version_key};
use crate::fsx::{join_rel, write_atomic};
use crate::install::native::{conda_info, conda_versions, micromamba_create, micromamba_release};
use crate::install::{arch_of, target_of, Ctx, Installer, Progress, Release};
use crate::kit::Target;
use crate::net::archive::{Inner, Kind};
use crate::net::verify::Expect;
use crate::setup::catalog::Component;

/// Ruby: RubyInstaller2 7z on Windows (without the MSYS2 atollo), or
/// conda-forge `ruby` by micromamba on macOS and Linux (D-112), in place
/// of the ruby/ruby-builder archives, built for the folder of GitHub's
/// runners (`/opt/hostedtoolcache`) and for Ubuntu only.
pub struct Ruby;

/// The best `rubyinstaller-X.Y.Z-N-<arch>.7z` among release assets, of
/// `line` (`3.4`) or of any line: (version `X.Y.Z-N`, asset object).
pub fn pick_ruby_line<'a>(
    releases: &'a [Value],
    arch: &str,
    line: Option<&str>,
) -> Option<(String, &'a Value)> {
    ruby_assets(releases, arch)
        .into_iter()
        .filter(|(v, _, _)| line.is_none_or(|l| crate::versions::line_of(v).as_deref() == Some(l)))
        .max_by(|a, b| {
            version_key(&a.0)
                .cmp(&version_key(&b.0))
                .then(a.1.cmp(&b.1))
        })
        .map(|(v, n, a)| (format!("{v}-{n}"), a))
}

/// `(version, build, asset)` of every `rubyinstaller-X.Y.Z-N-<arch>.7z`
/// among the releases that are not previews, with GitHub's SHA-256 (the
/// older ones have none: they could not be checked).
fn ruby_assets<'a>(releases: &'a [Value], arch: &str) -> Vec<(String, u64, &'a Value)> {
    let Ok(re) = regex::Regex::new(&format!(
        r"^rubyinstaller-(\d+\.\d+\.\d+)-(\d+)-{arch}\.7z$"
    )) else {
        return Vec::new();
    };
    releases
        .iter()
        .filter(|r| {
            r.get("prerelease") != Some(&Value::Bool(true))
                && r.get("draft") != Some(&Value::Bool(true))
        })
        .flat_map(|r| {
            r.get("assets")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|a| {
            let name = a.get("name")?.as_str()?;
            a.get("digest")?.as_str()?.strip_prefix("sha256:")?;
            let c = re.captures(name)?;
            Some((c[1].to_owned(), c[2].parse::<u64>().ok()?, a))
        })
        .collect()
}

/// The oldest Ruby line offered from conda-forge (`major`, `minor`): the
/// older ones there (2.3 to 3.1) are past their end of life, and
/// RubyInstaller on Windows no longer lists them either.
pub const RUBY_CONDA_OLDEST: (u32, u32) = (3, 2);

/// The minor lines of conda-forge `ruby` versions (newest first, as
/// `conda_versions` gives them), each once, from `RUBY_CONDA_OLDEST` on.
pub fn ruby_conda_lines(versions: &[String]) -> Vec<String> {
    let supported = |line: &str| {
        let mut it = line.split('.').map(str::parse::<u32>);
        match (it.next(), it.next()) {
            (Some(Ok(major)), Some(Ok(minor))) => (major, minor) >= RUBY_CONDA_OLDEST,
            _ => false,
        }
    };
    let mut out: Vec<String> = Vec::new();
    for line in versions.iter().filter_map(|v| crate::versions::line_of(v)) {
        if supported(&line) && !out.contains(&line) {
            out.push(line);
        }
    }
    out
}

/// The newest conda-forge `ruby` version of `line` (or of any line) among
/// `versions` (newest first).
pub fn ruby_conda_pick<'a>(versions: &'a [String], line: Option<&str>) -> Option<&'a String> {
    versions
        .iter()
        .find(|v| line.is_none_or(|l| crate::versions::line_of(v).as_deref() == Some(l)))
}

/// The `micromamba create` package of a conda-forge Ruby release.
pub fn ruby_conda_spec(version: &str) -> String {
    format!("ruby={version}")
}

/// conda-forge's `etc/gemrc` without its `gemhome:` line, which sends
/// `gem install` into the environment and wins over `GEM_HOME`: the
/// kit's gems belong in `GEM_HOME` (env.rs, `data/ruby-gems`), outside
/// the environment that is installed again when the kit moves. The bundled
/// gems stay visible (Ruby's default dir). `None` when there is none.
pub fn gemrc_without_gemhome(text: &str) -> Option<String> {
    let mut changed = false;
    let mut out = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        if line.trim_start().starts_with("gemhome:") {
            changed = true;
        } else {
            out.push_str(line);
        }
    }
    changed.then_some(out)
}

/// conda-forge's `rbconfig.rb` with the build tools under their plain
/// names (`x86_64-conda-linux-gnu-cc` → `cc`,
/// `arm64-apple-darwin20.0.0-clang` → `clang`): conda's compilers are not
/// in the environment, so a native gem would stop at "cc: not found". The
/// plain names are the kit's `cc` (D-112) or the system's. The
/// preprocessor is `$(CC) -E`, as Ruby's own default. On macOS conda's
/// `NM` is `llvm-nm --no-llvm-bc`, which the Xcode Command Line Tools do
/// not ship: it becomes their `nm`. `None` when nothing changes.
pub fn rbconfig_with_plain_tools(text: &str) -> Option<String> {
    let re = regex::Regex::new(
        r#"^(\s*CONFIG\["(CC|CXX|CPP|LD|AR|AS|NM|RANLIB|STRIP|OBJDUMP|OBJCOPY|PKG_CONFIG)"\] = ")[\w.]+-(?:conda-linux-gnu|apple-darwin[\d.]*)-([^"]*)(".*)$"#,
    )
    .ok()?;
    let llvm_nm = regex::Regex::new(r#"^(\s*CONFIG\["NM"\] = ")llvm-nm(?: [^"]*)?(".*)$"#).ok()?;
    let mut changed = false;
    let mut out = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        let body = line.trim_end_matches(['\r', '\n']);
        let ending = &line[body.len()..];
        if let Some(c) = re.captures(body) {
            let tool = if &c[2] == "CPP" { "$(CC) -E" } else { &c[3] };
            out.push_str(&format!("{}{tool}{}{ending}", &c[1], &c[4]));
            changed = true;
        } else if let Some(c) = llvm_nm.captures(body) {
            out.push_str(&format!("{}nm{}{ending}", &c[1], &c[2]));
            changed = true;
        } else {
            out.push_str(line);
        }
    }
    changed.then_some(out)
}

/// `gemrc_without_gemhome` and `rbconfig_with_plain_tools` applied to the
/// environment at `prefix` (every `rbconfig.rb` under `lib/ruby`).
fn fit_conda_ruby(prefix: &Path) -> Result<()> {
    let gemrc = join_rel(prefix, "etc/gemrc");
    if let Some(new) = fs::read_to_string(&gemrc)
        .ok()
        .and_then(|t| gemrc_without_gemhome(&t))
    {
        write_atomic(&gemrc, new.as_bytes()).with_context(|| gemrc.display().to_string())?;
    }
    let mut stack = vec![join_rel(prefix, "lib/ruby")];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                stack.push(path);
            } else if entry.file_name() == "rbconfig.rb" {
                if let Some(new) = fs::read_to_string(&path)
                    .ok()
                    .and_then(|t| rbconfig_with_plain_tools(&t))
                {
                    write_atomic(&path, new.as_bytes())
                        .with_context(|| path.display().to_string())?;
                }
            }
        }
    }
    Ok(())
}

/// The minor lines RubyInstaller has for `arch`, newest first (3.3 has no
/// ARM64 build).
pub fn ruby_lines(releases: &[Value], arch: &str) -> Vec<String> {
    let mut out: Vec<String> = ruby_assets(releases, arch)
        .into_iter()
        .filter_map(|(v, _, _)| crate::versions::line_of(&v))
        .collect();
    out.sort_by_key(|l| std::cmp::Reverse(version_key(l)));
    out.dedup();
    out
}

fn ruby_arch(ctx: &Ctx, component: &Component) -> &'static str {
    if arch_of(ctx, component) == "arm64" {
        "arm"
    } else {
        "x64"
    }
}

fn ruby_releases(ctx: &Ctx) -> Result<Vec<Value>> {
    ctx.net
        .json("https://api.github.com/repos/oneclick/rubyinstaller2/releases?per_page=30")
}

/// The conda-forge versions of Ruby for `target`'s subdir, newest first.
fn ruby_conda_versions(ctx: &Ctx, target: Target) -> Result<Vec<String>> {
    Ok(conda_versions(
        &conda_info(ctx, "ruby")?,
        target.conda_subdir(),
    ))
}

fn ruby_release(ctx: &Ctx, component: &Component, line: Option<&str>) -> Result<Release> {
    let target = target_of(ctx, component);
    if !target.is_windows() {
        let versions = ruby_conda_versions(ctx, target)?;
        let version = ruby_conda_pick(&versions, line).ok_or_else(|| {
            anyhow!(
                "no conda-forge Ruby {} for {}",
                line.unwrap_or(""),
                target.conda_subdir()
            )
        })?;
        return micromamba_release(ctx, version);
    }
    let releases = ruby_releases(ctx)?;
    let arch = ruby_arch(ctx, component);
    let (version, asset) = pick_ruby_line(&releases, arch, line)
        .ok_or_else(|| anyhow!("no RubyInstaller {} for {arch}", line.unwrap_or("")))?;
    let digest = asset
        .get("digest")
        .and_then(Value::as_str)
        .and_then(|d| d.strip_prefix("sha256:"))
        .ok_or_else(|| anyhow!("RubyInstaller: no SHA-256"))?;
    Ok(Release::single(
        version,
        s(asset, "browser_download_url")?.to_owned(),
        s(asset, "name")?.to_owned(),
        Expect::Sha256(digest.to_owned()),
        Kind::SevenZip,
        Inner::Single,
    ))
}

/// Windows: RubyInstaller2, gems in the folder of their Ruby
/// (`lib\ruby\gems\3.4.0`, commands in `bin\`), one set per minor version
/// (D-81). macOS and Linux (D-112): conda-forge `ruby` in an environment
/// created by micromamba in the folder of its line, like R; it has its
/// folder written inside, so it is installed again when the kit moves
/// (D-92), and gems belong outside it (`GEM_HOME`, env.rs).
impl Installer for Ruby {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        ruby_release(ctx, component, None)
    }

    /// micromamba on macOS and Linux, the shared path on Windows.
    fn custom(&self, target: Target) -> bool {
        !target.is_windows()
    }

    /// RubyInstaller moves with the kit; a conda environment does not.
    fn relocatable(&self, target: Target) -> bool {
        target.is_windows()
    }

    fn install_custom(
        &self,
        ctx: &Ctx,
        component: &Component,
        release: &Release,
        progress: &mut dyn Progress,
    ) -> Result<()> {
        let dir = crate::versions::install_dir(ctx.kit, component, &release.version);
        self.install_custom_at(ctx, component, release, &dir, progress)
    }

    /// The environment is created where it stays (conda writes its folder
    /// into some files).
    fn install_custom_at(
        &self,
        ctx: &Ctx,
        component: &Component,
        release: &Release,
        target: &Path,
        progress: &mut dyn Progress,
    ) -> Result<()> {
        let spec = ruby_conda_spec(&release.version);
        micromamba_create(ctx, component, release, target, &[&spec], progress)?;
        self.after_conda(ctx, component, target, progress)
    }

    fn conda_prefix(&self, target: Target, dir: &Path) -> Option<PathBuf> {
        (!target.is_windows()).then(|| dir.to_path_buf())
    }

    fn conda_programs(&self, target: Target) -> Option<&'static str> {
        (!target.is_windows()).then_some("ruby")
    }

    /// `gemrc` and `rbconfig.rb` fitted to the kit (`fit_conda_ruby`) and
    /// the launchers of Ruby's programs (D-121), the same from the network
    /// and from the kept packages (D-118). The gems are outside, in
    /// `data/ruby-gems` (D-112).
    fn after_conda(
        &self,
        ctx: &Ctx,
        component: &Component,
        dir: &Path,
        _progress: &mut dyn Progress,
    ) -> Result<()> {
        if !crate::versions::has_key_for(target_of(ctx, component), component, dir) {
            bail!("bin/ruby not found after the installation");
        }
        fit_conda_ruby(dir)?;
        crate::install::conda::write_launchers(dir, "ruby")
    }

    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, component: &Component) -> Result<Vec<String>> {
        let target = target_of(ctx, component);
        if target.is_windows() {
            Ok(ruby_lines(&ruby_releases(ctx)?, ruby_arch(ctx, component)))
        } else {
            Ok(ruby_conda_lines(&ruby_conda_versions(ctx, target)?))
        }
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        ruby_release(ctx, component, Some(line))
    }

    /// A new patch of the same line replaces the folder: the gems installed
    /// in the old one (and their commands) move to the new tree.
    fn before_swap(
        &self,
        _ctx: &Ctx,
        _component: &Component,
        tree: &Path,
        old: &Path,
    ) -> Result<()> {
        for sub in ["lib/ruby/gems", "bin"] {
            copy_missing(&join_rel(old, sub), &join_rel(tree, sub))?;
        }
        Ok(())
    }
}

/// Files and folders of `from` that `to` does not have, copied.
fn copy_missing(from: &Path, to: &Path) -> Result<()> {
    let Ok(entries) = fs::read_dir(from) else {
        return Ok(());
    };
    fs::create_dir_all(to)?;
    for entry in entries.flatten() {
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_missing(&entry.path(), &target)?;
        } else if !target.exists() {
            fs::copy(entry.path(), &target).with_context(|| target.display().to_string())?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../../test/rust/install/lang/ruby.rs"]
mod tests;
