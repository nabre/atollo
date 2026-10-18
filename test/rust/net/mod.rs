use super::*;

#[test]
fn checksum_files_of_old_releases() {
    let a = |name: &str| GhAsset {
        name: name.into(),
        browser_download_url: format!("https://x/{name}"),
        digest: None,
    };
    let rel = |assets: Vec<GhAsset>| GhRelease {
        tag_name: "v1.0.0".into(),
        assets,
        prerelease: false,
    };
    let kotlin = rel(vec![
        a("kotlin-compiler-2.1.21.zip"),
        a("kotlin-compiler-2.1.21.zip.sha256"),
    ]);
    let found = checksum_asset(&kotlin, &kotlin.assets[0]).unwrap();
    assert_eq!(found.name, "kotlin-compiler-2.1.21.zip.sha256");
    let deno = rel(vec![
        a("deno-x86_64-pc-windows-msvc.zip"),
        a("deno-x86_64-pc-windows-msvc.sha256sum"),
    ]);
    assert!(verifiable(&deno, &deno.assets[0]));
    let bun = rel(vec![a("bun-windows-x64.zip"), a("SHASUMS256.txt")]);
    assert!(verifiable(&bun, &bun.assets[0]));
    let typst = rel(vec![a("typst-x86_64-pc-windows-msvc.zip")]);
    assert!(!verifiable(&typst, &typst.assets[0]));
}

#[test]
fn checksum_lists() {
    let list = "aa  other.zip\n\
            0123456789abcdef0123456789abcdef0123456789abcdef0123456789ABCDEF *node-v24.1.0-win-x64.zip\n";
    assert_eq!(
        hash_in_list(list, "node-v24.1.0-win-x64.zip").as_deref(),
        Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
    );
    assert_eq!(hash_in_list(list, "missing.zip"), None);
    let single = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef  file.tar.gz";
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

/// D-105: the proxy of System Settings on macOS, from `scutil --proxy`.
#[test]
fn macos_proxy_from_scutil() {
    let both = "<dictionary> {
  ExceptionsList : <array> {
    0 : *.local
    1 : 169.254/16
  }
  FTPPassive : 1
  HTTPEnable : 1
  HTTPPort : 8080
  HTTPProxy : web.corp.example
  HTTPSEnable : 1
  HTTPSPort : 8443
  HTTPSProxy : secure.corp.example
}
";
    assert_eq!(
        scutil_proxy(both).as_deref(),
        Some("http://secure.corp.example:8443")
    );
    let http_only = both.replace("HTTPSEnable : 1", "HTTPSEnable : 0");
    assert_eq!(
        scutil_proxy(&http_only).as_deref(),
        Some("http://web.corp.example:8080")
    );
    let none = "<dictionary> {\n  ExceptionsList : <array> {\n    0 : HTTPEnable : 1\n  }\n  FTPPassive : 1\n}\n";
    assert_eq!(scutil_proxy(none), None);
    assert_eq!(scutil_proxy(""), None);
    assert_eq!(scutil_exceptions(both), ["*.local", "169.254/16"]);
    assert!(scutil_exceptions("<dictionary> {\n  HTTPEnable : 1\n}\n").is_empty());
}

/// D-105: the system's exceptions reach the proxy (a review finding).
#[test]
fn system_proxy_exceptions() {
    assert_eq!(
        gsettings_list("['localhost', '127.0.0.0/8', '*.corp.example']\n"),
        ["localhost", "127.0.0.0/8", "*.corp.example"]
    );
    assert!(gsettings_list("@as []\n").is_empty());
    assert!(gsettings_list("").is_empty());

    let proxy = Proxy::new("http://web.corp.example:8080").unwrap();
    let hosts = ["localhost".to_owned(), "*.corp.example".to_owned()];
    let proxy = with_exceptions(proxy, &hosts);
    assert_eq!((proxy.host(), proxy.port()), ("web.corp.example", 8080));
    let uri = |u: &str| u.parse::<ureq::http::Uri>().unwrap();
    assert!(proxy.is_no_proxy(&uri("http://localhost/x")));
    assert!(proxy.is_no_proxy(&uri("https://git.corp.example/x")));
    assert!(!proxy.is_no_proxy(&uri("https://github.com/x")));
    let plain = with_exceptions(Proxy::new("http://p.example:3128").unwrap(), &[]);
    assert!(!plain.is_no_proxy(&uri("http://localhost/x")));
}

/// D-105: GNOME's proxy, only when set by hand.
#[test]
fn gnome_proxy_from_gsettings() {
    let https = ("'secure.corp.example'\n", "8443\n");
    let http = ("'web.corp.example'\n", "8080\n");
    assert_eq!(
        gnome_proxy("'manual'\n", https, http).as_deref(),
        Some("http://secure.corp.example:8443")
    );
    assert_eq!(
        gnome_proxy("'manual'", ("''", "0"), http).as_deref(),
        Some("http://web.corp.example:8080")
    );
    assert_eq!(gnome_proxy("'none'", https, http), None);
    assert_eq!(gnome_proxy("'auto'", https, http), None);
    assert_eq!(gnome_proxy("'manual'", ("''", "0"), ("''", "0")), None);
    assert_eq!(
        proxy_url("http://p.example/", None).as_deref(),
        Some("http://p.example")
    );
}
