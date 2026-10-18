//! Containers as a hint of the servers a project uses: the `image:` of its
//! Compose file and the `FROM` of its `Dockerfile`. The image's name gives
//! the kit's component (exact name, not a substring), its tag the line
//! (`php:8.3-apache` → PHP 8.3 and Apache).

use std::collections::BTreeMap;
use std::path::Path;

use super::read;

/// The names of a Compose file, in Docker's order of preference.
const COMPOSE: [&str; 4] = [
    "compose.yaml",
    "compose.yml",
    "docker-compose.yaml",
    "docker-compose.yml",
];

/// Images named by the project's Compose files and Dockerfiles.
fn images(dir: &Path) -> Vec<String> {
    let mut found = Vec::new();
    for name in COMPOSE {
        for line in read(dir, name).lines() {
            let line = line.trim_start().trim_start_matches("- ").trim_start();
            if let Some(value) = line.strip_prefix("image:") {
                found.extend(first_word(value));
            }
        }
    }
    for name in ["Dockerfile", "Containerfile"] {
        for line in read(dir, name).lines() {
            let mut words = line.split_whitespace();
            if !words.next().is_some_and(|w| w.eq_ignore_ascii_case("FROM")) {
                continue;
            }
            // `FROM [--platform=…] image [AS name]`
            found.extend(words.find(|w| !w.starts_with("--")).map(str::to_owned));
        }
    }
    found
}

/// The value of a YAML scalar: no quotes, no comment.
fn first_word(value: &str) -> Option<String> {
    let value = value.trim();
    let quoted = value
        .strip_prefix('"')
        .and_then(|v| v.split('"').next())
        .or_else(|| value.strip_prefix('\'').and_then(|v| v.split('\'').next()));
    let word = quoted.or_else(|| value.split_whitespace().next())?;
    (!word.is_empty()).then(|| word.to_owned())
}

/// `[registry/][namespace/]name[:tag][@digest]` → (name, tag), lowercase.
fn split_image(image: &str) -> (String, String) {
    let image = image.split('@').next().unwrap_or_default();
    let last = image.rsplit('/').next().unwrap_or_default();
    let (name, tag) = last.split_once(':').unwrap_or((last, ""));
    (name.to_ascii_lowercase(), tag.to_ascii_lowercase())
}

/// The line of a tag that starts with it: `8.3-apache` → `8.3`; `latest`,
/// `alpine3.20` or `8` give none.
fn line_of_tag(tag: &str) -> Option<String> {
    tag.starts_with(|c: char| c.is_ascii_digit())
        .then(|| crate::versions::line_of(tag))
        .flatten()
}

/// The kit's components (with their line, when the tag gives it) of one
/// image.
fn components(image: &str) -> Vec<(&'static str, Option<String>)> {
    let (name, tag) = split_image(image);
    let line = line_of_tag(&tag);
    match name.as_str() {
        "httpd" | "apache" => vec![("apache", line)],
        "nginx" => vec![("nginx", line)],
        "php" if tag.contains("apache") => vec![("php", line), ("apache", None)],
        "php" => vec![("php", line)],
        // The kit's SQL server is MySQL (D-117): MariaDB's lines are not
        // MySQL's, so `mariadb:11.4` gives none.
        "mysql" => vec![("mysql", line)],
        "mariadb" => vec![("mysql", None)],
        "mongo" => vec![("mongodb", line)],
        _ => Vec::new(),
    }
}

/// The components the containers name, each with its line if one is given
/// (the first image that gives one wins).
pub(super) fn servers(dir: &Path) -> BTreeMap<&'static str, Option<String>> {
    let mut found: BTreeMap<&'static str, Option<String>> = BTreeMap::new();
    for image in images(dir) {
        for (id, line) in components(&image) {
            let slot = found.entry(id).or_default();
            if slot.is_none() {
                *slot = line;
            }
        }
    }
    found
}

#[cfg(test)]
#[path = "../../../test/rust/detect/container.rs"]
mod tests;
