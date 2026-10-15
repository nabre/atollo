//! Languages installed from archives (port of nabre/dev-kit
//! `_languages.ps1:77-340`; sources and checks in docs/app/CATALOG.md §2).

use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;

use super::{arch_of, github_single, pattern, triple_arch, Ctx, Dest, Installer, Part, Release};
use crate::archive::{Inner, Kind};
use crate::catalog::Component;
use crate::net::{first_sha256, github_digest, GhRelease};
use crate::verify::Expect;

/// Numeric parts of a version, for "newest" comparisons (`1.10.2` > `1.9`).
pub fn version_key(v: &str) -> Vec<u64> {
    v.split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect()
}

/// A version made only of numbers and dots (no betas, no `master`).
pub fn is_release(v: &str) -> bool {
    !v.is_empty()
        && v.split('.')
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

fn s<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing {key}"))
}

fn file_name_of(url: &str) -> String {
    url.rsplit('/')
        .next()
        .unwrap_or("download")
        .split('?')
        .next()
        .unwrap_or("download")
        .to_owned()
}

pub struct Go;

/// The Windows archive for `arch` of a release of go.dev's list.
fn go_file<'a>(rel: &'a Value, arch: &str) -> Option<&'a Value> {
    rel.get("files")?.as_array()?.iter().find(|f| {
        f.get("os").and_then(Value::as_str) == Some("windows")
            && f.get("arch").and_then(Value::as_str) == Some(arch)
            && f.get("kind").and_then(Value::as_str) == Some("archive")
    })
}

fn go_arch(ctx: &Ctx, component: &Component) -> &'static str {
    if arch_of(ctx, component) == "arm64" {
        "arm64"
    } else {
        "amd64"
    }
}

/// The stable lines of go.dev's full list (`include=all`), newest first.
pub fn go_lines(list: &[Value], arch: &str) -> Vec<String> {
    let mut out: Vec<String> = list
        .iter()
        .filter(|r| r.get("stable") == Some(&Value::Bool(true)) && go_file(r, arch).is_some())
        .filter_map(|r| crate::versions::line_of(r.get("version")?.as_str()?))
        .collect();
    out.sort_by_key(|l| std::cmp::Reverse(version_key(l)));
    out.dedup();
    out
}

impl Go {
    fn release(ctx: &Ctx, component: &Component, line: Option<&str>) -> Result<Release> {
        let arch = go_arch(ctx, component);
        let url = if line.is_some() {
            "https://go.dev/dl/?mode=json&include=all"
        } else {
            "https://go.dev/dl/?mode=json"
        };
        let list: Vec<Value> = ctx.net.json(url)?;
        // Newest first in the list; the full list is checked anyway.
        let rel = list
            .iter()
            .filter(|r| r.get("stable") == Some(&Value::Bool(true)) && go_file(r, arch).is_some())
            .filter(|r| {
                line.is_none_or(|l| {
                    r.get("version")
                        .and_then(Value::as_str)
                        .and_then(crate::versions::line_of)
                        .as_deref()
                        == Some(l)
                })
            })
            .max_by_key(|r| version_key(r.get("version").and_then(Value::as_str).unwrap_or("")))
            .ok_or_else(|| anyhow!("no stable Go {}", line.unwrap_or("")))?;
        let f = go_file(rel, arch).ok_or_else(|| anyhow!("no Go archive for windows/{arch}"))?;
        let name = s(f, "filename")?;
        Ok(Release::single(
            s(rel, "version")?.trim_start_matches("go").to_owned(),
            format!("https://go.dev/dl/{name}"),
            name.to_owned(),
            Expect::Sha256(s(f, "sha256")?.to_owned()),
            Kind::Zip,
            Inner::Named("go".into()),
        ))
    }
}

impl Installer for Go {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        Go::release(ctx, component, None)
    }

    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, component: &Component) -> Result<Vec<String>> {
        let list: Vec<Value> = ctx.net.json("https://go.dev/dl/?mode=json&include=all")?;
        Ok(go_lines(&list, go_arch(ctx, component)))
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        Go::release(ctx, component, Some(line))
    }
}

/// Java: Temurin JDK (the newest with a build for the kit), with Maven and
/// Gradle in their folders.
pub struct Java;

/// The newest Temurin JDK zip of `feature` (21, 25...) for this kit:
/// its version and the part that becomes the `java` folder.
fn temurin(ctx: &Ctx, component: &Component, feature: u64) -> Result<(String, Part)> {
    let arch = if arch_of(ctx, component) == "arm64" {
        "aarch64"
    } else {
        "x64"
    };
    let assets: Vec<Value> = ctx.net.json(&format!(
        "https://api.adoptium.net/v3/assets/latest/{feature}/hotspot?architecture={arch}&image_type=jdk&os=windows&vendor=eclipse"
    ))?;
    let jdk = assets
        .iter()
        .find(|a| {
            a.pointer("/binary/package/name")
                .and_then(Value::as_str)
                .is_some_and(|n| n.ends_with(".zip"))
        })
        .ok_or_else(|| anyhow!("no Temurin {feature} zip for {arch}"))?;
    let pkg = jdk
        .pointer("/binary/package")
        .ok_or_else(|| anyhow!("Adoptium: no package"))?;
    let version = jdk
        .pointer("/version/semver")
        .and_then(Value::as_str)
        .unwrap_or("?")
        .to_owned();
    Ok((
        version,
        Part {
            url: s(pkg, "link")?.to_owned(),
            file_name: s(pkg, "name")?.to_owned(),
            expect: Expect::Sha256(s(pkg, "checksum")?.to_owned()),
            kind: Kind::Zip,
            inner: Inner::Single,
            dest: Dest::Own,
            sign: None,
        },
    ))
}

/// The feature versions of Adoptium's `available_releases`: LTS ones and the
/// newest, as lines (`25.0`), newest first.
pub fn java_lines(info: &Value) -> Vec<String> {
    let list = |key: &str| -> Vec<u64> {
        info.get(key)
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_u64).collect())
            .unwrap_or_default()
    };
    let mut features = list("available_lts_releases");
    features.extend(
        info.get("most_recent_feature_release")
            .and_then(Value::as_u64),
    );
    features.sort_unstable_by(|a, b| b.cmp(a));
    features.dedup();
    features.iter().map(|f| format!("{f}.0")).collect()
}

impl Installer for Java {
    fn has_lines(&self) -> bool {
        true
    }

    /// On ARM64 only the versions with a Windows aarch64 build (Temurin has
    /// few: 21 today).
    fn lines(&self, ctx: &Ctx, component: &Component) -> Result<Vec<String>> {
        let info: Value = ctx
            .net
            .json("https://api.adoptium.net/v3/info/available_releases")?;
        let lines = java_lines(&info);
        if arch_of(ctx, component) != "arm64" {
            return Ok(lines);
        }
        Ok(lines
            .into_iter()
            .filter(|l| {
                l.split('.')
                    .next()
                    .and_then(|f| f.parse::<u64>().ok())
                    .is_some_and(|f| temurin(ctx, component, f).is_ok())
            })
            .collect())
    }

    /// The JDK only: Maven and Gradle stay those of the default.
    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        let feature = line
            .split('.')
            .next()
            .and_then(|f| f.parse::<u64>().ok())
            .ok_or_else(|| anyhow!("Java {line}"))?;
        let (version, part) = temurin(ctx, component, feature)?;
        Ok(Release {
            version,
            parts: vec![part],
        })
    }

    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let info: Value = ctx
            .net
            .json("https://api.adoptium.net/v3/info/available_releases")?;
        // The newest version with a build for this kit (the line marked
        // "latest"; no Windows aarch64 build of 25 or 27: an ARM64 kit
        // gets 21).
        let (java_version, jdk) = java_lines(&info)
            .iter()
            .filter_map(|l| l.split('.').next().and_then(|f| f.parse::<u64>().ok()))
            .find_map(|f| temurin(ctx, component, f).ok())
            .ok_or_else(|| anyhow!("Adoptium: no JDK for this PC"))?;

        let meta = ctx.net.text(
            "https://repo.maven.apache.org/maven2/org/apache/maven/apache-maven/maven-metadata.xml",
        )?;
        // Maven 3: Maven 4 wants Java 17 and the kit keeps 8 and 11 too.
        let maven = pattern(r"<version>([^<]+)</version>")?
            .captures_iter(&meta)
            .map(|c| c[1].to_owned())
            .filter(|v| is_release(v) && v.starts_with("3."))
            .max_by_key(|v| version_key(v))
            .ok_or_else(|| anyhow!("no Maven 3 version"))?;
        let maven_zip = format!("apache-maven-{maven}-bin.zip");
        let maven_url = format!("https://repo.maven.apache.org/maven2/org/apache/maven/apache-maven/{maven}/{maven_zip}");
        let maven_sha = ctx.net.text(&format!("{maven_url}.sha512"))?;
        let maven_sha = maven_sha
            .split(|c: char| !c.is_ascii_hexdigit())
            .find(|w| w.len() == 128)
            .ok_or_else(|| anyhow!("Maven: no SHA-512"))?
            .to_owned();

        let gradle: Value = ctx
            .net
            .json("https://services.gradle.org/versions/current")?;
        let gradle_version = s(&gradle, "version")?.to_owned();
        let gradle_sha = first_sha256(&ctx.net.text(s(&gradle, "checksumUrl")?)?)
            .ok_or_else(|| anyhow!("Gradle: no SHA-256"))?;

        Ok(Release {
            version: format!("{java_version} (Maven {maven}, Gradle {gradle_version})"),
            parts: vec![
                jdk,
                Part {
                    url: maven_url,
                    file_name: maven_zip,
                    expect: Expect::Sha512(maven_sha),
                    kind: Kind::Zip,
                    inner: Inner::Named(format!("apache-maven-{maven}")),
                    dest: Dest::Folder("maven"),
                    sign: None,
                },
                Part {
                    url: s(&gradle, "downloadUrl")?.to_owned(),
                    file_name: format!("gradle-{gradle_version}-bin.zip"),
                    expect: Expect::Sha256(gradle_sha),
                    kind: Kind::Zip,
                    inner: Inner::Named(format!("gradle-{gradle_version}")),
                    dest: Dest::Folder("gradle"),
                    sign: None,
                },
            ],
        })
    }

    /// Maven keeps its repository in the kit (`data\maven\repository`).
    fn finish(&self, ctx: &Ctx, _component: &Component, _dest: &Path) -> Result<()> {
        let settings = ctx.kit.apps().join(r"maven\conf\settings.xml");
        let text = fs::read_to_string(&settings).with_context(|| settings.display().to_string())?;
        if !text.contains("${env.CP_DATA}/maven/repository") {
            let re = pattern(r"(<settings[^>]*>)")?;
            let new = re
                .replacen(
                    &text,
                    1,
                    "$1\n  <localRepository>${env.CP_DATA}/maven/repository</localRepository>",
                )
                .into_owned();
            if new == text {
                bail!("Maven settings.xml: <settings> not found");
            }
            fs::write(&settings, new)?;
        }
        Ok(())
    }
}

pub struct Kotlin;

const KOTLIN_ASSET: &str = r"^kotlin-compiler-[\d.]+\.zip$";

/// The stable releases of `repo` with an asset matching `asset`, newest
/// first (its last 100 releases: betas and RCs left out). For the versions
/// of tools published on GitHub (Kotlin, Typst).
fn github_stable(ctx: &Ctx, repo: &str, asset: &str) -> Result<Vec<GhRelease>> {
    let all: Vec<GhRelease> = ctx.net.json(&format!(
        "https://api.github.com/repos/{repo}/releases?per_page=100"
    ))?;
    let re = pattern(asset)?;
    let mut out: Vec<GhRelease> = all
        .into_iter()
        .filter(|r| !r.prerelease && is_release(r.tag_name.trim_start_matches('v')))
        .filter(|r| r.asset(&re).is_some())
        .collect();
    out.sort_by_key(|r| std::cmp::Reverse(version_key(r.tag_name.trim_start_matches('v'))));
    Ok(out)
}

/// The minor lines of `releases` (newest first, each once).
pub fn release_lines(releases: &[GhRelease]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for r in releases {
        if let Some(line) = crate::versions::line_of(&r.tag_name) {
            if !out.contains(&line) {
                out.push(line);
            }
        }
    }
    out
}

/// The newest release of `line` among the stable ones of `repo`, as one
/// archive (GitHub's SHA-256 digest).
fn github_line(ctx: &Ctx, repo: &str, asset: &str, line: &str, inner: Inner) -> Result<Release> {
    let releases = github_stable(ctx, repo, asset)?;
    let rel = releases
        .iter()
        .find(|r| crate::versions::line_of(&r.tag_name).as_deref() == Some(line))
        .ok_or_else(|| anyhow!("{repo} {line} not available"))?;
    let file = rel
        .asset(&pattern(asset)?)
        .ok_or_else(|| anyhow!("{repo} {}: no asset", rel.tag_name))?;
    Ok(Release::single(
        rel.tag_name.trim_start_matches('v').to_owned(),
        file.browser_download_url.clone(),
        file.name.clone(),
        Expect::Sha256(github_digest(file)?),
        Kind::Zip,
        inner,
    ))
}

impl Installer for Kotlin {
    fn latest(&self, ctx: &Ctx, _component: &Component) -> Result<Release> {
        github_single(
            ctx,
            "JetBrains/kotlin",
            KOTLIN_ASSET,
            Kind::Zip,
            Inner::Named("kotlinc".into()),
        )
    }

    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, _component: &Component) -> Result<Vec<String>> {
        Ok(release_lines(&github_stable(
            ctx,
            "JetBrains/kotlin",
            KOTLIN_ASSET,
        )?))
    }

    fn release_line(&self, ctx: &Ctx, _component: &Component, line: &str) -> Result<Release> {
        github_line(
            ctx,
            "JetBrains/kotlin",
            KOTLIN_ASSET,
            line,
            Inner::Named("kotlinc".into()),
        )
    }
}

/// .NET SDK of the active LTS channel.
pub struct Dotnet;

const DOTNET_INDEX: &str =
    "https://builds.dotnet.microsoft.com/dotnet/release-metadata/releases-index.json";

fn dotnet_channels(index: &Value) -> impl Iterator<Item = &Value> {
    index
        .get("releases-index")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}

/// The channels of .NET that are not previews (`10.0`, `9.0`...), newest first.
pub fn dotnet_lines(index: &Value) -> Vec<String> {
    let mut out: Vec<String> = dotnet_channels(index)
        .filter(|c| {
            !matches!(
                c.get("support-phase").and_then(Value::as_str),
                Some("preview" | "go-live")
            )
        })
        .filter_map(|c| c.get("channel-version")?.as_str().map(str::to_owned))
        .filter(|v| crate::versions::valid_line(v))
        .collect();
    out.sort_by_key(|l| std::cmp::Reverse(version_key(l)));
    out
}

impl Installer for Dotnet {
    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, _component: &Component) -> Result<Vec<String>> {
        Ok(dotnet_lines(&ctx.net.json(DOTNET_INDEX)?))
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        let index: Value = ctx.net.json(DOTNET_INDEX)?;
        let channel = dotnet_channels(&index)
            .find(|c| c.get("channel-version").and_then(Value::as_str) == Some(line))
            .ok_or_else(|| anyhow!(".NET {line}: no channel"))?;
        dotnet_release(ctx, component, channel)
    }

    /// The newest channel that is not a preview (the line marked "latest").
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let index: Value = ctx.net.json(DOTNET_INDEX)?;
        let newest = dotnet_lines(&index)
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!(".NET: no channel"))?;
        let channel = dotnet_channels(&index)
            .find(|c| c.get("channel-version").and_then(Value::as_str) == Some(newest.as_str()))
            .ok_or_else(|| anyhow!(".NET {newest}: no channel"))?;
        dotnet_release(ctx, component, channel)
    }
}

/// The newest SDK of a channel of `releases-index.json`.
fn dotnet_release(ctx: &Ctx, component: &Component, channel: &Value) -> Result<Release> {
    let rid = if arch_of(ctx, component) == "arm64" {
        "win-arm64"
    } else {
        "win-x64"
    };
    let sdk = s(channel, "latest-sdk")?.to_owned();
    let releases: Value = ctx.net.json(s(channel, "releases.json")?)?;
    let file = releases
        .get("releases")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|r| {
            r.get("sdks")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .chain(r.get("sdk"))
        })
        .filter(|x| x.get("version").and_then(Value::as_str) == Some(sdk.as_str()))
        .flat_map(|x| {
            x.get("files")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .find(|f| {
            f.get("rid").and_then(Value::as_str) == Some(rid)
                && f.get("name")
                    .and_then(Value::as_str)
                    .is_some_and(|n| n.ends_with(".zip"))
        })
        .ok_or_else(|| anyhow!(".NET SDK {sdk}: no zip for {rid}"))?;
    Ok(Release::single(
        sdk.clone(),
        s(file, "url")?.to_owned(),
        format!("dotnet-sdk-{sdk}-{rid}.zip"),
        Expect::Sha512(s(file, "hash")?.to_owned()),
        Kind::Zip,
        Inner::Root,
    ))
}

pub struct Dart;

impl Installer for Dart {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let arch = arch_of(ctx, component);
        let info: Value = ctx.net.json(
            "https://storage.googleapis.com/dart-archive/channels/stable/release/latest/VERSION",
        )?;
        let v = s(&info, "version")?.to_owned();
        let name = format!("dartsdk-windows-{arch}-release.zip");
        let url = format!(
            "https://storage.googleapis.com/dart-archive/channels/stable/release/{v}/sdk/{name}"
        );
        let sha = first_sha256(&ctx.net.text(&format!("{url}.sha256sum"))?)
            .ok_or_else(|| anyhow!("Dart: no SHA-256"))?;
        Ok(Release::single(
            v,
            url,
            name,
            Expect::Sha256(sha),
            Kind::Zip,
            Inner::Named("dart-sdk".into()),
        ))
    }
}

pub struct Zig;

/// The release lines of Zig's index with a build for `key`, newest first.
pub fn zig_lines(index: &Value, key: &str) -> Vec<String> {
    let mut out: Vec<String> = index
        .as_object()
        .into_iter()
        .flatten()
        .filter(|(k, v)| is_release(k) && v.get(key).is_some())
        .filter_map(|(k, _)| crate::versions::line_of(k))
        .collect();
    out.sort_by_key(|l| std::cmp::Reverse(version_key(l)));
    out.dedup();
    out
}

impl Installer for Zig {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        Zig::release(ctx, component, None)
    }

    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, component: &Component) -> Result<Vec<String>> {
        let key = format!("{}-windows", triple_arch(ctx, component));
        let index: Value = ctx.net.json("https://ziglang.org/download/index.json")?;
        Ok(zig_lines(&index, &key))
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        Zig::release(ctx, component, Some(line))
    }
}

impl Zig {
    fn release(ctx: &Ctx, component: &Component, line: Option<&str>) -> Result<Release> {
        let key = format!("{}-windows", triple_arch(ctx, component));
        let index: Value = ctx.net.json("https://ziglang.org/download/index.json")?;
        let obj = index.as_object().ok_or_else(|| anyhow!("Zig index"))?;
        let v = obj
            .keys()
            .filter(|k| is_release(k))
            .filter(|k| line.is_none_or(|l| crate::versions::line_of(k).as_deref() == Some(l)))
            .max_by_key(|k| version_key(k))
            .ok_or_else(|| anyhow!("no Zig release {}", line.unwrap_or("")))?;
        let f = obj[v]
            .get(&key)
            .ok_or_else(|| anyhow!("Zig {v}: no {key}"))?;
        let url = s(f, "tarball")?.to_owned();
        Ok(Release::single(
            v.clone(),
            url.clone(),
            file_name_of(&url),
            Expect::Sha256(s(f, "shasum")?.to_owned()),
            Kind::Zip,
            Inner::Single,
        ))
    }
}

pub struct Julia;

impl Installer for Julia {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let arch = triple_arch(ctx, component);
        let versions: Value = ctx
            .net
            .json("https://julialang-s3.julialang.org/bin/versions.json")?;
        let obj = versions
            .as_object()
            .ok_or_else(|| anyhow!("Julia versions"))?;
        let v = obj
            .iter()
            .filter(|(k, v)| is_release(k) && v.get("stable") == Some(&Value::Bool(true)))
            .map(|(k, _)| k)
            .max_by_key(|k| version_key(k))
            .ok_or_else(|| anyhow!("no stable Julia"))?;
        let f = obj[v]
            .get("files")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find(|f| {
                f.get("os").and_then(Value::as_str) == Some("winnt")
                    && f.get("arch").and_then(Value::as_str) == Some(arch)
                    && f.get("kind").and_then(Value::as_str) == Some("archive")
                    && f.get("extension").and_then(Value::as_str) == Some("zip")
            })
            .ok_or_else(|| anyhow!("Julia {v}: no zip for {arch}"))?;
        let url = s(f, "url")?.to_owned();
        Ok(Release::single(
            v.clone(),
            url.clone(),
            file_name_of(&url),
            Expect::Sha256(s(f, "sha256")?.to_owned()),
            Kind::Zip,
            Inner::Single,
        ))
    }
}

pub struct Bun;

impl Installer for Bun {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let arch = if arch_of(ctx, component) == "arm64" {
            "aarch64"
        } else {
            "x64"
        };
        github_single(
            ctx,
            "oven-sh/bun",
            &format!(r"^bun-windows-{arch}\.zip$"),
            Kind::Zip,
            Inner::Single,
        )
    }
}

pub struct Deno;

impl Installer for Deno {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let arch = triple_arch(ctx, component);
        github_single(
            ctx,
            "denoland/deno",
            &format!(r"^deno-{arch}-pc-windows-msvc\.zip$"),
            Kind::Zip,
            Inner::Root,
        )
    }
}

pub struct Typst;

/// The Typst zip for this kit's platform.
fn typst_asset(ctx: &Ctx, component: &Component) -> String {
    let arch = triple_arch(ctx, component);
    format!(r"^typst-{arch}-pc-windows-msvc\.zip$")
}

/// Typst's 0.x versions change the syntax: old documents need their own
/// (versioned, D-81; `typst.toml` declares the compiler).
impl Installer for Typst {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        github_single(
            ctx,
            "typst/typst",
            &typst_asset(ctx, component),
            Kind::Zip,
            Inner::Single,
        )
    }

    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, component: &Component) -> Result<Vec<String>> {
        Ok(release_lines(&github_stable(
            ctx,
            "typst/typst",
            &typst_asset(ctx, component),
        )?))
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        github_line(
            ctx,
            "typst/typst",
            &typst_asset(ctx, component),
            line,
            Inner::Single,
        )
    }
}

/// TinyTeX-1 (x64 build, also on ARM64): a self-extracting 7z, read directly.
pub struct TinyTex;

/// The packages a TinyTeX has (`tlmgr info --list --only-installed`).
fn tlmgr_packages(ctx: &Ctx, root: &Path) -> Option<Vec<String>> {
    let tlmgr = root.join(r"bin\windows\tlmgr.bat");
    if !tlmgr.is_file() {
        return None;
    }
    let out = crate::jobs::kit_command(&tlmgr, ctx.env)
        .args(["info", "--list", "--only-installed", "--data", "name"])
        .stdin(std::process::Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())?;
    Some(
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(str::trim)
            // `xetex.windows`: the binaries of a package, which comes with it.
            .filter(|l| !l.is_empty() && !l.contains('.'))
            .map(str::to_owned)
            .collect(),
    )
}

/// The packages of the TinyTeX being replaced: TeX Live accepts updates only
/// from its own year, so the yearly release is a new TinyTeX, and the
/// packages added with tlmgr (or by R Markdown, latexmk…) come back.
fn tinytex_list(ctx: &Ctx) -> std::path::PathBuf {
    ctx.kit.data().join(r"texlive\packages.txt")
}

impl Installer for TinyTex {
    fn latest(&self, ctx: &Ctx, _component: &Component) -> Result<Release> {
        github_single(
            ctx,
            "rstudio/tinytex-releases",
            r"^TinyTeX-1-windows-v[\d.]+\.exe$",
            Kind::SevenZip,
            Inner::Named("TinyTeX".into()),
        )
    }

    fn before_swap(
        &self,
        ctx: &Ctx,
        _component: &Component,
        _tree: &Path,
        old: &Path,
    ) -> Result<()> {
        if let Some(list) = tlmgr_packages(ctx, old) {
            let file = tinytex_list(ctx);
            fs::create_dir_all(file.parent().unwrap_or(&file))?;
            crate::fsx::write_atomic(&file, list.join("\n").as_bytes())?;
        }
        Ok(())
    }

    /// The packages of the old TinyTeX it does not have, installed again; a
    /// failure (offline) only goes to the log: the list stays for next time.
    fn finish(&self, ctx: &Ctx, _component: &Component, dest: &Path) -> Result<()> {
        let Ok(text) = fs::read_to_string(tinytex_list(ctx)) else {
            return Ok(());
        };
        let have = tlmgr_packages(ctx, dest).unwrap_or_default();
        let missing: Vec<&str> = text
            .lines()
            .map(str::trim)
            .filter(|p| !p.is_empty() && !have.iter().any(|h| h == p))
            .collect();
        if missing.is_empty() {
            return Ok(());
        }
        let ok = crate::jobs::kit_command(dest.join(r"bin\windows\tlmgr.bat"), ctx.env)
            .arg("install")
            .args(&missing)
            .stdin(std::process::Stdio::null())
            .output()
            .is_ok_and(|o| o.status.success());
        if ok {
            crate::log::info(&format!("TinyTeX: {} packages back", missing.len()));
        } else {
            crate::log::warn(&format!(
                "TinyTeX: packages not installed again ({}): tlmgr install them by hand",
                missing.join(" ")
            ));
        }
        Ok(())
    }
}

/// CMake, with Ninja in `tools\`.
pub struct Cmake;

impl Installer for Cmake {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let arm = arch_of(ctx, component) == "arm64";
        let arch = if arm { "arm64" } else { "x86_64" };
        let mut release = github_single(
            ctx,
            "Kitware/CMake",
            &format!(r"^cmake-[\d.]+-windows-{arch}\.zip$"),
            Kind::Zip,
            Inner::Single,
        )?;
        let ninja = ctx.net.github_latest("ninja-build/ninja")?;
        let name = if arm {
            "ninja-winarm64.zip"
        } else {
            "ninja-win.zip"
        };
        let asset = ninja
            .assets
            .iter()
            .find(|a| a.name == name)
            .ok_or_else(|| anyhow!("no {name}"))?;
        release.version = format!(
            "{} (Ninja {})",
            release.version,
            ninja.tag_name.trim_start_matches('v')
        );
        release.parts.push(Part {
            url: asset.browser_download_url.clone(),
            file_name: asset.name.clone(),
            expect: Expect::Sha256(github_digest(asset)?),
            kind: Kind::Zip,
            inner: Inner::Root,
            dest: Dest::Tools(vec!["ninja.exe"]),
            sign: None,
        });
        Ok(release)
    }
}

/// C and C++: llvm-mingw (UCRT). The cross `*-w64-mingw32-gcc/g++` are
/// removed: on the PATH they would make Rust's `gnu` toolchain use them as
/// linker and fail (`_languages.ps1:302-316`).
pub struct Cc;

impl Installer for Cc {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let arch = triple_arch(ctx, component);
        github_single(
            ctx,
            "mstorsjo/llvm-mingw",
            &format!(r"^llvm-mingw-\d+-ucrt-{arch}\.zip$"),
            Kind::Zip,
            Inner::Single,
        )
    }

    fn finish(&self, _ctx: &Ctx, _component: &Component, dest: &Path) -> Result<()> {
        let re = pattern(r"(?i)^\w+-w64-mingw32-(gcc|g\+\+)\.exe$")?;
        for entry in fs::read_dir(dest.join("bin"))?.flatten() {
            if re.is_match(&entry.file_name().to_string_lossy()) {
                fs::remove_file(entry.path())?;
            }
        }
        make_launchers(&dest.join("bin"))
    }
}

/// `make` is mingw32-make in llvm-mingw (make.exe is only in busybox, not
/// on the PATH): Makefiles, Ruby's native gems and R's source packages
/// call `make`. Also at start, for a C/C++ installed before (D-81).
pub fn make_launchers(bin: &Path) -> Result<()> {
    if !bin.join("mingw32-make.exe").is_file() {
        return Ok(());
    }
    let files: [(&str, &[u8]); 2] = [
        ("make.cmd", b"@\"%~dp0mingw32-make.exe\" %*\r\n"),
        (
            "make",
            b"#!/bin/sh\nexec \"$(dirname \"$0\")/mingw32-make.exe\" \"$@\"\n",
        ),
    ];
    for (name, content) in files {
        let path = bin.join(name);
        if fs::read(&path).ok().as_deref() != Some(content) {
            crate::fsx::write_atomic(&path, content)?;
        }
    }
    Ok(())
}

/// Ruby: RubyInstaller2 7z (without the MSYS2 devkit), highest version and
/// build among the non-prerelease releases.
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
/// among the releases that are not previews.
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
            let c = re.captures(name)?;
            Some((c[1].to_owned(), c[2].parse::<u64>().ok()?, a))
        })
        .collect()
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

fn ruby_release(ctx: &Ctx, component: &Component, line: Option<&str>) -> Result<Release> {
    let arch = ruby_arch(ctx, component);
    let releases = ruby_releases(ctx)?;
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

/// Gems live in the folder of their Ruby (`lib\ruby\gems\3.4.0`, commands
/// in `bin\`), one set per minor version (D-81): no GEM_HOME shared by all.
impl Installer for Ruby {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        ruby_release(ctx, component, None)
    }

    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, component: &Component) -> Result<Vec<String>> {
        Ok(ruby_lines(&ruby_releases(ctx)?, ruby_arch(ctx, component)))
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
        for sub in [r"lib\ruby\gems", "bin"] {
            copy_missing(&old.join(sub), &tree.join(sub))?;
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
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn kotlin_lines_from_releases() {
        let rel = |tag: &str| GhRelease {
            tag_name: tag.into(),
            assets: Vec::new(),
            prerelease: false,
        };
        let list = [
            rel("v2.2.21"),
            rel("v2.2.20"),
            rel("v2.1.21"),
            rel("v2.0.0"),
        ];
        assert_eq!(release_lines(&list), ["2.2", "2.1", "2.0"]);
    }

    #[test]
    fn lines_of_the_sources() {
        let go = |v: &str, stable: bool, arch: &str| {
            json!({"version": v, "stable": stable, "files": [
                {"os": "windows", "arch": arch, "kind": "archive", "filename": "x.zip", "sha256": "a"}
            ]})
        };
        let list = [
            go("go1.25.3", true, "amd64"),
            go("go1.26rc1", false, "amd64"),
            go("go1.24.9", true, "amd64"),
            go("go1.24.10", true, "amd64"),
            go("go1.23.1", true, "arm64"),
        ];
        assert_eq!(go_lines(&list, "amd64"), ["1.25", "1.24"]);
        assert_eq!(go_lines(&list, "arm64"), ["1.23"]);

        let info = json!({"available_lts_releases": [8, 11, 17, 21, 25], "most_recent_feature_release": 26});
        assert_eq!(
            java_lines(&info),
            ["26.0", "25.0", "21.0", "17.0", "11.0", "8.0"]
        );

        let index = json!({"releases-index": [
            {"channel-version": "11.0", "support-phase": "preview"},
            {"channel-version": "10.0", "support-phase": "active"},
            {"channel-version": "9.0", "support-phase": "active"},
            {"channel-version": "6.0", "support-phase": "eol"}
        ]});
        assert_eq!(dotnet_lines(&index), ["10.0", "9.0", "6.0"]);

        let zig = json!({
            "master": {"x86_64-windows": {}},
            "0.15.2": {"x86_64-windows": {}},
            "0.15.1": {"x86_64-windows": {}},
            "0.14.1": {"x86_64-windows": {}, "aarch64-windows": {}}
        });
        assert_eq!(zig_lines(&zig, "x86_64-windows"), ["0.15", "0.14"]);
        assert_eq!(zig_lines(&zig, "aarch64-windows"), ["0.14"]);
    }

    #[test]
    fn versions() {
        assert!(version_key("1.10.2") > version_key("1.9.9"));
        assert!(
            is_release("0.14.1")
                && !is_release("master")
                && !is_release("4.0.0-beta-1")
                && !is_release("")
        );
    }

    #[test]
    fn ruby_highest_version_and_build() {
        let rel = |pre: bool, names: &[&str]| json!({"prerelease": pre, "assets": names.iter().map(|n| json!({"name": n})).collect::<Vec<_>>()});
        let releases = vec![
            rel(
                false,
                &[
                    "rubyinstaller-3.3.9-1-x64.7z",
                    "rubyinstaller-3.3.9-1-arm.7z",
                ],
            ),
            rel(
                false,
                &[
                    "rubyinstaller-3.4.5-1-x64.7z",
                    "rubyinstaller-3.4.5-2-x64.7z",
                ],
            ),
            rel(true, &["rubyinstaller-3.5.0-1-x64.7z"]),
        ];
        assert_eq!(
            pick_ruby_line(&releases, "x64", None).map(|(v, _)| v),
            Some("3.4.5-2".into())
        );
        assert_eq!(
            pick_ruby_line(&releases, "arm", None).map(|(v, _)| v),
            Some("3.3.9-1".into())
        );
    }
}
