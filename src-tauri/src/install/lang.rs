//! Languages installed from archives (port of nabre/atollo
//! `_languages.ps1:77-340`; sources and checks in docs/app/CATALOG.md §2).

use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;

use super::{arch_of, github_single, pattern, triple_arch, Ctx, Dest, Installer, Part, Release};
use crate::archive::{Inner, Kind};
use crate::catalog::Component;
use crate::net::{first_sha256, github_digest};
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

/// Java: Temurin JDK (latest LTS), with Maven and Gradle in their folders.
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

    fn lines(&self, ctx: &Ctx, _component: &Component) -> Result<Vec<String>> {
        let info: Value = ctx
            .net
            .json("https://api.adoptium.net/v3/info/available_releases")?;
        Ok(java_lines(&info))
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
        let lts = info
            .get("most_recent_lts")
            .and_then(Value::as_u64)
            .ok_or_else(|| anyhow!("Adoptium: no LTS"))?;
        let (java_version, jdk) = temurin(ctx, component, lts)?;

        let meta = ctx.net.text(
            "https://repo.maven.apache.org/maven2/org/apache/maven/apache-maven/maven-metadata.xml",
        )?;
        let maven = pattern(r"<version>([^<]+)</version>")?
            .captures_iter(&meta)
            .map(|c| c[1].to_owned())
            .filter(|v| is_release(v))
            .max_by_key(|v| version_key(v))
            .ok_or_else(|| anyhow!("no Maven version"))?;
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

impl Installer for Kotlin {
    fn latest(&self, ctx: &Ctx, _component: &Component) -> Result<Release> {
        github_single(
            ctx,
            "JetBrains/kotlin",
            r"^kotlin-compiler-[\d.]+\.zip$",
            Kind::Zip,
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

    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let index: Value = ctx.net.json(DOTNET_INDEX)?;
        let channel = dotnet_channels(&index)
            .find(|c| {
                c.get("release-type").and_then(Value::as_str) == Some("lts")
                    && c.get("support-phase").and_then(Value::as_str) == Some("active")
            })
            .ok_or_else(|| anyhow!(".NET: no active LTS channel"))?;
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

impl Installer for Typst {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let arch = triple_arch(ctx, component);
        github_single(
            ctx,
            "typst/typst",
            &format!(r"^typst-{arch}-pc-windows-msvc\.zip$"),
            Kind::Zip,
            Inner::Single,
        )
    }
}

/// TinyTeX-1 (x64 build, also on ARM64): a self-extracting 7z, read directly.
pub struct TinyTex;

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
        Ok(())
    }
}

/// Ruby: RubyInstaller2 7z (without the MSYS2 atollo), highest version and
/// build among the non-prerelease releases.
pub struct Ruby;

/// The best `rubyinstaller-X.Y.Z-N-<arch>.7z` among release assets:
/// (version `X.Y.Z-N`, asset object).
pub fn pick_ruby<'a>(releases: &'a [Value], arch: &str) -> Option<(String, &'a Value)> {
    let re = regex::Regex::new(&format!(
        r"^rubyinstaller-(\d+\.\d+\.\d+)-(\d+)-{arch}\.7z$"
    ))
    .ok()?;
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
        .max_by(|a, b| {
            version_key(&a.0)
                .cmp(&version_key(&b.0))
                .then(a.1.cmp(&b.1))
        })
        .map(|(v, n, a)| (format!("{v}-{n}"), a))
}

impl Installer for Ruby {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let arch = if arch_of(ctx, component) == "arm64" {
            "arm"
        } else {
            "x64"
        };
        let releases: Vec<Value> = ctx
            .net
            .json("https://api.github.com/repos/oneclick/rubyinstaller2/releases?per_page=30")?;
        let (version, asset) =
            pick_ruby(&releases, arch).ok_or_else(|| anyhow!("no RubyInstaller for {arch}"))?;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
            pick_ruby(&releases, "x64").map(|(v, _)| v),
            Some("3.4.5-2".into())
        );
        assert_eq!(
            pick_ruby(&releases, "arm").map(|(v, _)| v),
            Some("3.3.9-1".into())
        );
    }
}
