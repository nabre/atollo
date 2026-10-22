//! Downloads and what is checked on them: the network (this file), the
//! checksums (`verify`), the Authenticode signatures (`signature`) and the
//! archives with the folder swaps of the installer (`archive`).
//!
//! Downloads for the installer: network setup, the download itself and the
//! latest release tag of a project.
//!
//! TLS is Windows' own (SChannel through native-tls) with the Windows
//! certificate store, so corporate SSL inspection works as it does for the
//! browser, plus the extra PEM file of the Settings when there is one
//! (`ca`). Proxy: the HTTPS one in Settings (else its HTTP one, with the
//! «Senza proxy per» hosts reached directly), else `HTTPS_PROXY`/`HTTP_PROXY`, else
//! the system's setting, read only: Windows', macOS `scutil --proxy`, or
//! GNOME's on Linux, with their exceptions (read once per run, no PAC).
//! Every download goes to `<file>.part`
//! and is renamed only when complete; 3 attempts, waiting 5 s then 10 s.
//!
//! Here `Net`, its downloads and texts; GitHub's releases, checksums and
//! calls in `github`, the system's proxy and its exceptions in `proxy`, the
//! large downloads resumed with `Range` (the local models) in `resume`.

pub mod archive;
pub mod ca;
mod github;
pub mod lan;
mod proxy;
pub mod resume;
/// Authenticode (WinVerifyTrust): Windows only, like the files it checks.
#[cfg(windows)]
pub mod signature;
pub mod verify;

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use ureq::tls::{RootCerts, TlsConfig, TlsProvider};
use ureq::{Agent, Proxy};

use crate::settings::Settings;
pub use github::{first_sha256, github_digest, hash_in_list, verifiable};
#[cfg(unix)]
use proxy::system_proxy;
use proxy::{no_proxy_hosts, with_exceptions};

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
    /// The checksum the file must have: also how the run knows, later, which
    /// component a file added by hand is for (`downloads::want`).
    pub expect: verify::Expect,
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
        let n = &settings.network;
        let set = |v: &Option<String>| {
            v.as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_owned)
        };
        let proxy = match set(&n.https_proxy).or_else(|| set(&n.http_proxy)) {
            Some(url) => {
                let proxy = Proxy::new(&url).with_context(|| format!("proxy {url}"))?;
                Some(with_exceptions(
                    proxy,
                    &no_proxy_hosts(n.no_proxy.as_deref()),
                ))
            }
            #[cfg(windows)]
            None => Proxy::try_from_env(),
            #[cfg(unix)]
            None => Proxy::try_from_env().or_else(system_proxy),
        };
        let roots = match set(&n.extra_ca_certs) {
            Some(file) => RootCerts::Specific(Arc::new(
                ca::roots_with(Path::new(&file)).with_context(|| format!("certificates {file}"))?,
            )),
            None => RootCerts::PlatformVerifier,
        };
        let tls = TlsConfig::builder()
            .provider(TlsProvider::NativeTls)
            .root_certs(roots)
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
            // Texts and API answers (at most 32 MB): a body that stalls after
            // the headers ends here instead of at the system's TCP timeout.
            // Downloads have no total limit, only `STALL` (`copy_body`).
            .timeout_recv_body(Some(TEXT_BODY))
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

    fn get(&self, url: &str) -> Result<ureq::http::Response<ureq::Body>> {
        self.request(url)
            .call()
            .with_context(|| format!("GET {url}"))
    }

    fn request(&self, url: &str) -> ureq::RequestBuilder<ureq::typestate::WithoutBody> {
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
        request
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
            let response = self
                .request(url)
                .config()
                .timeout_recv_body(None)
                .build()
                .call()
                .with_context(|| format!("GET {url}"))?;
            let total = response
                .headers()
                .get("content-length")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok());
            let body = response.into_body().into_reader();
            let mut file = File::create(&part).with_context(|| part.display().to_string())?;
            let done = copy_body(body, &mut file, cancel, STALL, &mut |done| {
                progress(done, total)
            })
            .with_context(|| format!("downloading {url}"))?;
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

/// The longest wait for the body of a text or an API answer.
const TEXT_BODY: Duration = Duration::from_secs(300);

/// A download that receives nothing for this long has stalled (a proxy
/// that stops sending after the headers) and is given up.
const STALL: Duration = Duration::from_secs(60);

/// Copies `body` to `file` and gives the bytes copied; `progress(done)`
/// after each read. The reads happen in a thread of their own, so a read
/// that never returns does not hold the download: Cancel stops it at once
/// and `stall` without a byte is an error, instead of waiting for the
/// system's TCP timeout. The thread ends at its next read, or when the
/// connection closes.
pub(crate) fn copy_body(
    mut body: impl Read + Send + 'static,
    file: &mut impl Write,
    cancel: &AtomicBool,
    stall: Duration,
    progress: &mut dyn FnMut(u64),
) -> Result<u64> {
    use std::sync::mpsc::{sync_channel, RecvTimeoutError};
    // A few reads ahead of the disk at most.
    let (tx, rx) = sync_channel::<std::io::Result<Vec<u8>>>(4);
    std::thread::spawn(move || loop {
        let mut buf = vec![0u8; 256 * 1024];
        let read = body.read(&mut buf).map(|n| {
            buf.truncate(n);
            buf
        });
        let end = !matches!(&read, Ok(b) if !b.is_empty());
        if tx.send(read).is_err() || end {
            break;
        }
    });
    let mut done = 0u64;
    let mut last = std::time::Instant::now();
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err(Cancelled.into());
        }
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(Ok(chunk)) if chunk.is_empty() => return Ok(done),
            Ok(Ok(chunk)) => {
                file.write_all(&chunk)?;
                done += chunk.len() as u64;
                last = std::time::Instant::now();
                progress(done);
            }
            Ok(Err(e)) => return Err(e.into()),
            Err(RecvTimeoutError::Timeout) if last.elapsed() >= stall => {
                bail!("no data for {stall:?} after {done} bytes")
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => bail!("the download stopped"),
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

#[cfg(test)]
#[path = "../../../test/rust/net/mod.rs"]
mod tests;
