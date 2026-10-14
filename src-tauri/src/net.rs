//! Downloads for the installer (port of nabre/dev-kit `_common.ps1`
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

/// Cancellation requested by the user: stops a download at once.
#[derive(Debug, thiserror::Error)]
#[error("cancelled")]
pub struct Cancelled;

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
                "Dev-Kit/{} (+https://github.com/nabre/dev-kit2)",
                env!("CARGO_PKG_VERSION")
            ))
            .timeout_connect(Some(Duration::from_secs(30)))
            .timeout_recv_response(Some(Duration::from_secs(60)))
            .build();
        Ok(Net {
            agent: config.new_agent(),
            github_token: github_token.filter(|t| !t.is_empty()),
        })
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

    pub fn github_latest(&self, repo: &str) -> Result<GhRelease> {
        self.json(&format!(
            "https://api.github.com/repos/{repo}/releases/latest"
        ))
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
        let json = serde_json::from_str::<serde_json::Value>(text.trim_start_matches('\u{feff}'));
        match json {
            Ok(value) if (200..300).contains(&status) => Ok(value),
            _ => Err(ApiError {
                status: Some(status),
                retry_after,
                detail: format!("HTTP {status}"),
            }),
        }
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
