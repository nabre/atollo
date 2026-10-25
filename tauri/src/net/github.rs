//! GitHub: releases (the API with ETags, or the release pages), checksums, the account's calls.

use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};

use super::{retry, ApiError, GhAnswer, GhAsset, GhRelease, GithubLimit, Net};

impl Net {
    pub(super) fn github_cached(&self, url: &str, dir: &std::path::Path) -> Result<String> {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        url.hash(&mut h);
        let body_path = dir.join(format!("{:016x}.json", h.finish()));
        let etag_path = body_path.with_extension("etag");
        let cached = std::fs::read_to_string(&body_path).ok();
        let etag = std::fs::read_to_string(&etag_path)
            .ok()
            .filter(|_| cached.is_some());
        let mut request = self
            .agent
            .get(url)
            .header("Accept", "application/vnd.github+json");
        if let Some(token) = &self.github_token {
            request = request.header("Authorization", format!("Bearer {token}"));
        }
        if let Some(tag) = &etag {
            request = request.header("If-None-Match", tag.trim());
        }
        let response = request
            .config()
            .http_status_as_error(false)
            .build()
            .call()
            .with_context(|| format!("GET {url}"))?;
        let status = response.status().as_u16();
        let new_tag = response
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        match (status, cached) {
            (304, Some(body)) => Ok(body),
            (200..=299, _) => {
                let body = response
                    .into_body()
                    .with_config()
                    .limit(32 * 1024 * 1024)
                    .read_to_string()
                    .with_context(|| format!("reading {url}"))?;
                if std::fs::create_dir_all(dir).is_ok() {
                    let _ = crate::common::fsx::write_atomic(&body_path, body.as_bytes());
                    match new_tag {
                        Some(tag) => {
                            let _ = crate::common::fsx::write_atomic(&etag_path, tag.as_bytes());
                        }
                        None => {
                            let _ = std::fs::remove_file(&etag_path);
                        }
                    }
                }
                Ok(body)
            }
            (403 | 429, Some(body)) => {
                crate::common::log::warn(&format!(
                    "GitHub API limit reached: the last answer kept for {url}"
                ));
                Ok(body)
            }
            (403 | 429, None) => Err(anyhow::Error::new(ureq::Error::StatusCode(status))
                .context(GithubLimit)
                .context(format!("GET {url}"))),
            _ => {
                Err(anyhow::Error::new(ureq::Error::StatusCode(status))
                    .context(format!("GET {url}")))
            }
        }
    }

    /// The SHA-256 of a release asset: GitHub's digest, or the release's
    /// checksum file for the older assets (`checksum_asset`).
    pub fn release_sha256(&self, release: &GhRelease, asset: &GhAsset) -> Result<String> {
        if let Ok(hash) = github_digest(asset) {
            return Ok(hash);
        }
        let file = checksum_asset(release, asset)
            .ok_or_else(|| anyhow!("GitHub gives no SHA-256 for {}", asset.name))?;
        let text = self.text(&file.browser_download_url)?;
        hash_in_list(&text, &asset.name)
            .or_else(|| {
                // A file for this asset only: its hash, maybe without a name.
                (!file.name.eq_ignore_ascii_case("SHASUMS256.txt")
                    && !file.name.eq_ignore_ascii_case("SHA256SUMS")
                    && !file.name.eq_ignore_ascii_case("sha256sums.txt"))
                .then(|| first_sha256(&text))
                .flatten()
            })
            .ok_or_else(|| anyhow!("{}: no SHA-256 for {}", file.name, asset.name))
    }

    /// The latest release of `repo`. When the API's limit is reached and no
    /// answer is kept, the same from github.com's pages (`github_latest_page`).
    pub fn github_latest(&self, repo: &str) -> Result<GhRelease> {
        let api = self.json(&format!(
            "https://api.github.com/repos/{repo}/releases/latest"
        ));
        match api {
            Err(e) if is_github_limit(&e) => self.github_latest_page(repo).map_err(|page| {
                crate::common::log::warn(&format!("{repo}: release page: {page:#}"));
                e
            }),
            other => other,
        }
    }

    /// `releases/latest` without the API, which github.com's pages do not
    /// count: the address redirects to the tag, and the list of assets
    /// (`expanded_assets`) has each asset's SHA-256 as the API gives it.
    fn github_latest_page(&self, repo: &str) -> Result<GhRelease> {
        let url = format!("https://github.com/{repo}/releases/latest");
        let location = retry(|| {
            let response = self
                .agent
                .get(&url)
                .config()
                .max_redirects(0)
                .http_status_as_error(false)
                .build()
                .call()
                .with_context(|| format!("GET {url}"))?;
            let status = response.status().as_u16();
            response
                .headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned)
                .filter(|_| response.status().is_redirection())
                .ok_or_else(|| {
                    anyhow::Error::new(ureq::Error::StatusCode(status))
                        .context(format!("GET {url}: no release"))
                })
        })?;
        // The repository as GitHub names it now (renamed, other case).
        let (canonical, tag) = location
            .strip_prefix("https://github.com/")
            .and_then(|path| path.split_once("/releases/tag/"))
            .filter(|(r, t)| r.split('/').count() == 2 && !t.is_empty() && !t.contains('/'))
            .ok_or_else(|| anyhow!("GET {url}: unexpected redirect to {location}"))?;
        let html = self.text(&format!(
            "https://github.com/{canonical}/releases/expanded_assets/{tag}"
        ))?;
        let assets = expanded_assets(&html, canonical, tag);
        if assets.is_empty() {
            bail!("{canonical} {tag}: no assets on the release page");
        }
        Ok(GhRelease {
            tag_name: percent_decode(tag).unwrap_or_else(|| tag.to_owned()),
            assets,
            prerelease: false,
        })
    }

    /// One call to the GitHub API with the kit's GitHub account
    /// (`github/`): `method` is GET, PATCH or DELETE; the token goes to
    /// `https://api.github.com/` only. A single attempt: renaming or deleting
    /// twice is not the same as once.
    pub fn github_call(
        &self,
        method: &str,
        url: &str,
        token: &str,
        body: Option<&serde_json::Value>,
    ) -> std::result::Result<GhAnswer, ApiError> {
        let failed = |status: Option<u16>, detail: String| ApiError {
            status,
            retry_after: None,
            detail,
        };
        if !url.starts_with("https://api.github.com/") {
            return Err(failed(None, format!("not the GitHub API: {url}")));
        }
        let auth = format!("Bearer {token}");
        let headers = [
            ("Authorization", auth.as_str()),
            ("Accept", "application/vnd.github+json"),
            ("X-GitHub-Api-Version", "2022-11-28"),
        ];
        let timeout = Some(Duration::from_secs(30));
        let response = match (method, body) {
            ("PATCH", Some(body)) => {
                let mut request = self.agent.patch(url);
                for (name, value) in headers {
                    request = request.header(name, value);
                }
                request
                    .config()
                    .http_status_as_error(false)
                    .timeout_global(timeout)
                    .build()
                    .send_json(body)
            }
            ("GET" | "DELETE", None) => {
                let mut request = if method == "GET" {
                    self.agent.get(url)
                } else {
                    self.agent.delete(url)
                };
                for (name, value) in headers {
                    request = request.header(name, value);
                }
                request
                    .config()
                    .http_status_as_error(false)
                    .timeout_global(timeout)
                    .build()
                    .call()
            }
            _ => return Err(failed(None, format!("unsupported call {method} {url}"))),
        }
        .map_err(|e| failed(None, e.to_string()))?;
        let status = response.status().as_u16();
        let header = |name: &str| {
            response
                .headers()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned)
        };
        let link = header("link").unwrap_or_default();
        let scopes = header("x-oauth-scopes");
        let text = response
            .into_body()
            .with_config()
            .limit(32 * 1024 * 1024)
            .read_to_string()
            .map_err(|e| failed(Some(status), e.to_string()))?;
        let json = serde_json::from_str::<serde_json::Value>(&text).unwrap_or_default();
        if (200..300).contains(&status) {
            return Ok(GhAnswer { json, link, scopes });
        }
        // GitHub explains a refusal in `message`.
        let detail = json
            .get("message")
            .and_then(|m| m.as_str())
            .map_or_else(|| format!("HTTP {status}"), str::to_owned);
        Err(failed(Some(status), detail))
    }
}

/// GitHub's API refused for its limit (403 and 429 without a token).
fn is_github_limit(e: &anyhow::Error) -> bool {
    e.downcast_ref::<GithubLimit>().is_some()
        || e.chain().any(|cause| {
            matches!(
                cause.downcast_ref::<ureq::Error>(),
                Some(ureq::Error::StatusCode(403 | 429))
            )
        })
}

/// The assets of a release page's `expanded_assets`: each download link,
/// with the first `sha256:` before the next one in its row. The source archives are
/// not assets (they are under `/archive/`).
fn expanded_assets(html: &str, repo: &str, tag: &str) -> Vec<GhAsset> {
    let prefix = format!("href=\"/{repo}/releases/download/{tag}/");
    let mut assets: Vec<GhAsset> = Vec::new();
    for part in html.split(prefix.as_str()).skip(1) {
        let Some((raw, rest)) = part.split_once('"') else {
            continue;
        };
        let Some(name) = percent_decode(raw).filter(|n| !n.is_empty() && !n.contains('/')) else {
            continue;
        };
        let row = rest.split("</li>").next().unwrap_or(rest);
        let digest = row.split("sha256:").nth(1).and_then(|d| {
            let hex = d.get(..64)?;
            hex.chars()
                .all(|c| c.is_ascii_hexdigit())
                .then(|| format!("sha256:{}", hex.to_ascii_lowercase()))
        });
        match assets.iter_mut().find(|a| a.name == name) {
            Some(known) => {
                if known.digest.is_none() {
                    known.digest = digest;
                }
            }
            None => assets.push(GhAsset {
                browser_download_url: format!(
                    "https://github.com/{repo}/releases/download/{tag}/{raw}"
                ),
                name,
                digest,
            }),
        }
    }
    assets
}

fn percent_decode(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            out.push(u8::from_str_radix(s.get(i + 1..i + 3)?, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// First run of 64 hex characters in a checksum file (`hash  name` or `hash`).
pub fn first_sha256(text: &str) -> Option<String> {
    text.split(|c: char| !c.is_ascii_hexdigit())
        .find(|w| w.len() == 64)
        .map(str::to_ascii_lowercase)
}

/// The hash of `file_name` in a list of `hash  name` lines (Find-HashInList).
pub fn hash_in_list(text: &str, file_name: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let hash = parts.next()?;
        let name = parts.next()?.trim_start_matches('*');
        (name == file_name && hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit()))
            .then(|| hash.to_ascii_lowercase())
    })
}

/// `sha256:<hex>` from the GitHub API.
pub fn github_digest(asset: &GhAsset) -> Result<String> {
    asset
        .digest
        .as_deref()
        .and_then(|d| d.strip_prefix("sha256:"))
        .filter(|h| h.len() == 64)
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| anyhow!("GitHub gives no SHA-256 for {}", asset.name))
}

/// The checksum file a release publishes for `asset`, for the assets older
/// than GitHub's digests: `<name>.sha256` (Kotlin), `<stem>.sha256sum`
/// (Deno), or a list for every asset (`SHASUMS256.txt` of Bun).
pub fn checksum_asset<'a>(release: &'a GhRelease, asset: &GhAsset) -> Option<&'a GhAsset> {
    let stem = asset
        .name
        .rsplit_once('.')
        .map_or(asset.name.as_str(), |(s, _)| s);
    let names = [
        format!("{}.sha256", asset.name),
        format!("{}.sha256sum", asset.name),
        format!("{stem}.sha256sum"),
        "SHASUMS256.txt".to_owned(),
        "SHA256SUMS".to_owned(),
        "sha256sums.txt".to_owned(),
    ];
    names.iter().find_map(|n| {
        release
            .assets
            .iter()
            .find(|a| a.name.eq_ignore_ascii_case(n))
    })
}

/// `asset` can be checked: GitHub's digest or a checksum file of its release.
pub fn verifiable(release: &GhRelease, asset: &GhAsset) -> bool {
    github_digest(asset).is_ok() || checksum_asset(release, asset).is_some()
}

#[cfg(test)]
#[path = "../../../test/rust/net/github.rs"]
mod tests;
