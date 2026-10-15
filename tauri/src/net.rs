//! Downloads for the installer (port of nabre/atollo `_common.ps1`
//! Initialize-Network / Get-Download and `_prepare-helpers.ps1` Get-LatestTag).
//!
//! TLS is Windows' own (SChannel through native-tls) with the Windows
//! certificate store, so corporate SSL inspection works as it does for the
//! browser. Proxy: the one in Settings, else `HTTPS_PROXY`/`HTTP_PROXY`, else
//! the Windows proxy setting (read-only). Every download goes to `<file>.part`
//! and is renamed only when complete; 3 attempts, waiting 5 s then 10 s.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use ureq::tls::{RootCerts, TlsConfig, TlsProvider};
use ureq::{Agent, Proxy};

use crate::settings::Settings;

pub struct Net {
    agent: Agent,
    github_token: Option<String>,
    /// Answers of the GitHub API kept with their ETag (`data\cache\github`).
    github_cache: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GhAsset {
    pub name: String,
    pub browser_download_url: String,
    /// `sha256:<hex>`, computed by GitHub.
    #[serde(default)]
    pub digest: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GhRelease {
    pub tag_name: String,
    #[serde(default)]
    pub assets: Vec<GhAsset>,
    /// Betas and release candidates.
    #[serde(default)]
    pub prerelease: bool,
}

impl GhRelease {
    pub fn asset(&self, pattern: &regex::Regex) -> Option<&GhAsset> {
        self.assets.iter().find(|a| pattern.is_match(&a.name))
    }
}

/// A failed `api_json`: the HTTP status when the server answered
/// (`None`: no answer, `Some(200)`: an answer that is not JSON).
#[derive(Debug, Clone)]
pub struct ApiError {
    pub status: Option<u16>,
    pub retry_after: Option<String>,
    pub detail: String,
}

/// A successful `github_call`.
#[derive(Debug, Clone)]
pub struct GhAnswer {
    pub json: serde_json::Value,
    /// The `Link` header: where the next page of a list is.
    pub link: String,
    /// `X-OAuth-Scopes`: what the token may do (absent for fine-grained tokens).
    pub scopes: Option<String>,
}

/// Cancellation requested by the user: stops a download at once.
#[derive(Debug, thiserror::Error)]
#[error("cancelled")]
pub struct Cancelled;

/// A download that failed and can be done in the browser instead: the
/// interface shows `url`, and the file added by hand (`setup/downloads.rs`) is
/// used at the next attempt when its checksum is the expected one.
#[derive(Debug)]
pub struct ByHand {
    pub url: String,
    pub file_name: String,
    pub cause: anyhow::Error,
}

impl std::fmt::Display for ByHand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#}", self.cause)
    }
}

impl std::error::Error for ByHand {}

/// GitHub's limit of API calls (60 an hour without a token) reached, with no
/// answer kept for that address.
#[derive(Debug, thiserror::Error)]
#[error("GitHub API limit reached: sign in to GitHub in the kit, or try again in an hour")]
pub struct GithubLimit;

impl Net {
    pub fn new(settings: &Settings, github_token: Option<String>) -> Result<Net> {
        let proxy = match settings
            .network
            .https_proxy
            .as_deref()
            .filter(|p| !p.is_empty())
        {
            Some(url) => Some(Proxy::new(url).with_context(|| format!("proxy {url}"))?),
            None => Proxy::try_from_env(),
        };
        let tls = TlsConfig::builder()
            .provider(TlsProvider::NativeTls)
            .root_certs(RootCerts::PlatformVerifier)
            .build();
        let config = Agent::config_builder()
            .tls_config(tls)
            .proxy(proxy)
            .user_agent(format!(
                "Atollo/{} (+https://github.com/nabre/atollo)",
                env!("CARGO_PKG_VERSION")
            ))
            .timeout_connect(Some(Duration::from_secs(30)))
            .timeout_recv_response(Some(Duration::from_secs(60)))
            .build();
        Ok(Net {
            agent: config.new_agent(),
            github_token: github_token.filter(|t| !t.is_empty()),
            github_cache: None,
        })
    }

    /// Keeps the answers of the GitHub API in `dir` (D-81): asked again with
    /// `If-None-Match`, an unchanged one costs nothing (a 304 does not count
    /// in GitHub's limit of 60 calls an hour without a token), and when the
    /// limit is reached (403, 429) the last answer is used.
    pub fn with_github_cache(mut self, dir: std::path::PathBuf) -> Net {
        self.github_cache = Some(dir);
        self
    }

    fn github_cached(&self, url: &str, dir: &std::path::Path) -> Result<String> {
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
                    let _ = crate::fsx::write_atomic(&body_path, body.as_bytes());
                    match new_tag {
                        Some(tag) => {
                            let _ = crate::fsx::write_atomic(&etag_path, tag.as_bytes());
                        }
                        None => {
                            let _ = std::fs::remove_file(&etag_path);
                        }
                    }
                }
                Ok(body)
            }
            (403 | 429, Some(body)) => {
                crate::log::warn(&format!(
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

    fn get(&self, url: &str) -> Result<ureq::http::Response<ureq::Body>> {
        let mut request = self.agent.get(url);
        // The token (CI only) goes to the GitHub API and nowhere else.
        if let Some(token) = self
            .github_token
            .as_deref()
            .filter(|_| url.starts_with("https://api.github.com/"))
        {
            request = request.header("Authorization", format!("Bearer {token}"));
        }
        if url.starts_with("https://api.github.com/") {
            request = request.header("Accept", "application/vnd.github+json");
        }
        request.call().with_context(|| format!("GET {url}"))
    }

    pub fn text(&self, url: &str) -> Result<String> {
        if let Some(dir) = self
            .github_cache
            .as_deref()
            .filter(|_| url.starts_with("https://api.github.com/"))
        {
            return retry(|| self.github_cached(url, dir));
        }
        retry(|| {
            self.get(url)?
                .into_body()
                .with_config()
                .limit(32 * 1024 * 1024)
                .read_to_string()
                .with_context(|| format!("reading {url}"))
        })
    }

    pub fn json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let text = self.text(url)?;
        serde_json::from_str(&text).with_context(|| format!("unexpected answer from {url}"))
    }

    /// The latest release of `repo`. When the API's limit is reached and no
    /// answer is kept, the same from github.com's pages (`github_latest_page`).
    pub fn github_latest(&self, repo: &str) -> Result<GhRelease> {
        let api = self.json(&format!(
            "https://api.github.com/repos/{repo}/releases/latest"
        ));
        match api {
            Err(e) if is_github_limit(&e) => self.github_latest_page(repo).map_err(|page| {
                crate::log::warn(&format!("{repo}: release page: {page:#}"));
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

    /// One JSON request to an agent's server (`stats.rs`): a single attempt,
    /// the HTTP status and `Retry-After` kept for the caller. The headers go
    /// to `url` only; a `User-Agent` among them replaces the app's.
    pub fn api_json(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        body: Option<&serde_json::Value>,
        timeout: Duration,
    ) -> std::result::Result<serde_json::Value, ApiError> {
        let failed = |detail: String| ApiError {
            status: None,
            retry_after: None,
            detail,
        };
        let response = match body {
            Some(body) => {
                let mut request = self.agent.post(url);
                for (name, value) in headers {
                    request = request.header(*name, *value);
                }
                request
                    .config()
                    .http_status_as_error(false)
                    .timeout_global(Some(timeout))
                    .build()
                    .send_json(body)
            }
            None => {
                let mut request = self.agent.get(url);
                for (name, value) in headers {
                    request = request.header(*name, *value);
                }
                request
                    .config()
                    .http_status_as_error(false)
                    .timeout_global(Some(timeout))
                    .build()
                    .call()
            }
        }
        .map_err(|e| failed(e.to_string()))?;
        let status = response.status().as_u16();
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        let text = response
            .into_body()
            .with_config()
            .limit(4 * 1024 * 1024)
            .read_to_string()
            .map_err(|e| failed(e.to_string()))?;
        let json = serde_json::from_str::<serde_json::Value>(crate::fsx::strip_bom(&text));
        match json {
            Ok(value) if (200..300).contains(&status) => Ok(value),
            _ => Err(ApiError {
                status: Some(status),
                retry_after,
                detail: format!("HTTP {status}"),
            }),
        }
    }

    /// One call to the GitHub API with the kit's GitHub account
    /// (`github.rs`): `method` is GET, PATCH or DELETE; the token goes to
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

    /// Downloads `url` to `dest` through `dest.part`. `progress(done, total)`
    /// is called while bytes arrive.
    pub fn download(
        &self,
        url: &str,
        dest: &Path,
        cancel: &AtomicBool,
        progress: &mut dyn FnMut(u64, Option<u64>),
    ) -> Result<()> {
        let part = dest.with_extension(match dest.extension() {
            Some(ext) => format!("{}.part", ext.to_string_lossy()),
            None => "part".into(),
        });
        if let Some(dir) = dest.parent() {
            fs::create_dir_all(dir)?;
        }
        let result = retry(|| {
            if cancel.load(Ordering::Relaxed) {
                return Err(Cancelled.into());
            }
            let response = self.get(url)?;
            let total = response
                .headers()
                .get("content-length")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok());
            let mut body = response.into_body().into_reader();
            let mut file = File::create(&part).with_context(|| part.display().to_string())?;
            let mut buf = vec![0u8; 256 * 1024];
            let mut done = 0u64;
            loop {
                if cancel.load(Ordering::Relaxed) {
                    return Err(Cancelled.into());
                }
                let n = body
                    .read(&mut buf)
                    .with_context(|| format!("downloading {url}"))?;
                if n == 0 {
                    break;
                }
                file.write_all(&buf[..n])?;
                done += n as u64;
                progress(done, total);
            }
            file.flush()?;
            if let Some(total) = total {
                if done != total {
                    bail!("download of {url} stopped at {done} of {total} bytes");
                }
            }
            Ok(())
        });
        match result {
            Ok(()) => fs::rename(&part, dest).with_context(|| dest.display().to_string()),
            Err(e) => {
                let _ = fs::remove_file(&part);
                Err(e)
            }
        }
    }
}

/// Three attempts, 5 s and 10 s apart; cancellation is never retried.
fn retry<T>(mut f: impl FnMut() -> Result<T>) -> Result<T> {
    let waits = [5, 10];
    let mut attempt = 0;
    loop {
        match f() {
            Ok(v) => return Ok(v),
            Err(e) if e.is::<Cancelled>() => return Err(e),
            Err(e) if is_permanent(&e) || attempt == waits.len() => return Err(e),
            Err(_) => {
                std::thread::sleep(Duration::from_secs(waits[attempt]));
                attempt += 1;
            }
        }
    }
}

/// 404 and similar answers do not change by asking again.
fn is_permanent(e: &anyhow::Error) -> bool {
    e.chain().any(|cause| {
        matches!(
            cause.downcast_ref::<ureq::Error>(),
            Some(ureq::Error::StatusCode(400..=404 | 410))
        )
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_lists() {
        let list = "aa  other.zip\n\
            0123456789abcdef0123456789abcdef0123456789abcdef0123456789ABCDEF *node-v24.1.0-win-x64.zip\n";
        assert_eq!(
            hash_in_list(list, "node-v24.1.0-win-x64.zip").as_deref(),
            Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
        );
        assert_eq!(hash_in_list(list, "missing.zip"), None);
        let single =
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef  file.tar.gz";
        assert_eq!(first_sha256(single).map(|h| h.len()), Some(64));
        assert_eq!(first_sha256("no hash"), None);
    }

    #[test]
    fn github_digest_format() {
        let asset = |d: Option<&str>| GhAsset {
            name: "x.zip".into(),
            browser_download_url: String::new(),
            digest: d.map(Into::into),
        };
        let hex = "ab".repeat(32);
        assert_eq!(
            github_digest(&asset(Some(&format!("sha256:{hex}")))).unwrap(),
            hex
        );
        assert!(github_digest(&asset(None)).is_err());
        assert!(github_digest(&asset(Some("sha512:00"))).is_err());
    }

    /// A server answering, in order: 200 with an ETag, 304 when asked with
    /// it, then 403 (the limit of GitHub's API).
    #[test]
    fn github_answers_kept_with_their_etag() {
        use std::io::{BufRead, BufReader, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/repos/x/y", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let mut asked_with_tag = Vec::new();
            for answer in 0..3 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut tagged = false;
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line
                        .to_ascii_lowercase()
                        .starts_with("if-none-match: \"v1\"")
                    {
                        tagged = true;
                    }
                    if line == "\r\n" || line.is_empty() {
                        break;
                    }
                }
                asked_with_tag.push(tagged);
                let reply = match answer {
                    0 => "HTTP/1.1 200 OK\r\nETag: \"v1\"\r\nContent-Length: 7\r\nConnection: close\r\n\r\n{\"a\":1}",
                    1 => "HTTP/1.1 304 Not Modified\r\nConnection: close\r\n\r\n",
                    _ => "HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                };
                stream.write_all(reply.as_bytes()).unwrap();
            }
            asked_with_tag
        });
        let tmp = crate::kit::tests::TempDir::new("github-cache");
        let net = Net::new(&Settings::default(), None).unwrap();
        for _ in 0..3 {
            assert_eq!(net.github_cached(&url, &tmp.0).unwrap(), "{\"a\":1}");
        }
        assert_eq!(server.join().unwrap(), [false, true, true]);
    }

    /// The shape of github.com's `expanded_assets` (October 2026): two
    /// links per asset, then its digest; an asset without a digest.
    #[test]
    fn release_page_assets() {
        let hex = "AE".repeat(32);
        let html = format!(
            r#"<li><a href="/cli/cli/releases/download/v2.1/gh_2.1_windows_amd64.zip" rel="nofollow">
            <span>GitHub CLI</span></a>
            <a href="/cli/cli/releases/download/v2.1/gh_2.1_windows_amd64.zip" rel="nofollow"></a>
            <span class="Truncate-text">sha256:{hex}</span>
            <clipboard-copy value="sha256:{hex}"></clipboard-copy></li>
            <li><a href="/cli/cli/releases/download/v2.1/notes%20v2.txt" rel="nofollow"></a></li>
            <li><a href="/cli/cli/archive/refs/tags/v2.1.zip" rel="nofollow"></a>
            <span>sha256:{}</span></li>"#,
            "00".repeat(32)
        );
        let assets = expanded_assets(&html, "cli/cli", "v2.1");
        assert_eq!(assets.len(), 2);
        assert_eq!(assets[0].name, "gh_2.1_windows_amd64.zip");
        assert_eq!(
            assets[0].browser_download_url,
            "https://github.com/cli/cli/releases/download/v2.1/gh_2.1_windows_amd64.zip"
        );
        assert_eq!(github_digest(&assets[0]).unwrap(), hex.to_ascii_lowercase());
        assert_eq!(assets[1].name, "notes v2.txt");
        assert_eq!(assets[1].digest, None);
    }

    #[test]
    fn limit_errors_are_recognised() {
        let limit = anyhow::Error::new(ureq::Error::StatusCode(403))
            .context(GithubLimit)
            .context("GET x");
        assert!(is_github_limit(&limit));
        assert!(is_permanent(&limit));
        let missing = anyhow::Error::new(ureq::Error::StatusCode(404)).context("GET x");
        assert!(!is_github_limit(&missing));
    }

    /// The real github.com (`cargo test -- --ignored`).
    #[test]
    #[ignore]
    fn release_page_live() {
        let net = Net::new(&Settings::default(), None).unwrap();
        let rel = net.github_latest_page("PowerShell/PowerShell").unwrap();
        let zip = rel
            .asset(&regex::Regex::new(r"^PowerShell-.+-win-x64\.zip$").unwrap())
            .unwrap();
        assert_eq!(github_digest(zip).unwrap().len(), 64);
    }

    #[test]
    fn retry_stops_on_cancel() {
        let mut calls = 0;
        let result: Result<()> = retry(|| {
            calls += 1;
            Err(Cancelled.into())
        });
        assert!(result.is_err());
        assert_eq!(calls, 1);
    }
}
