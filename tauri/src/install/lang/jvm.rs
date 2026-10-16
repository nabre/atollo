//! The JVM languages: Java (Temurin JDK, with Maven and Gradle) and Kotlin.

use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;

use super::{github_line, github_stable, is_release, release_lines, s, version_key};
use crate::fsx::join_rel;
use crate::install::{
    arch_of, github_single, pattern, target_of, Ctx, Dest, Installer, Part, Release,
};
use crate::kit::{Os, Target};
use crate::net::archive::{Inner, Kind};
use crate::net::first_sha256;
use crate::net::verify::Expect;
use crate::setup::catalog::Component;

/// Java: Temurin JDK (the newest with a build for the kit), with Maven and
/// Gradle in their folders.
pub struct Java;

pub(crate) fn temurin_spec(
    target: Target,
) -> (&'static str, &'static str, &'static str, Kind, Inner) {
    let os = target.adoptium_os();
    let arch = target.adoptium_arch();
    match target.os {
        Os::Win32 => (os, arch, ".zip", Kind::Zip, Inner::Single),
        Os::Darwin => (
            os,
            arch,
            ".tar.gz",
            Kind::TarGz,
            Inner::SingleThen("Contents/Home"),
        ),
        Os::Linux => (os, arch, ".tar.gz", Kind::TarGz, Inner::Single),
    }
}

/// The newest Temurin JDK archive of `feature` (21, 25...) for this kit:
/// its version and the part that becomes the `java` folder.
fn temurin(ctx: &Ctx, component: &Component, feature: u64) -> Result<(String, Part)> {
    let target = target_of(ctx, component);
    let (os, arch, ext, kind, inner) = temurin_spec(target);
    let assets: Vec<Value> = ctx.net.json(&format!(
        "https://api.adoptium.net/v3/assets/latest/{feature}/hotspot?architecture={arch}&image_type=jdk&os={os}&vendor=eclipse"
    ))?;
    let jdk = assets
        .iter()
        .find(|a| {
            a.pointer("/binary/package/name")
                .and_then(Value::as_str)
                .is_some_and(|n| n.ends_with(ext))
        })
        .ok_or_else(|| anyhow!("no Temurin {feature} {ext} for {os}/{arch}"))?;
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
            kind,
            inner,
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

    /// On ARM64 only the versions with an aarch64 build (Temurin on Windows
    /// ARM64 has few: 21 today).
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
        let settings = join_rel(ctx.kit.apps(), "maven/conf/settings.xml");
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

#[cfg(test)]
#[path = "../../../../test/rust/install/lang/jvm.rs"]
mod tests;
