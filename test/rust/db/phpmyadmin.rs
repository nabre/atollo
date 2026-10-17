use super::*;

#[test]
fn address_carries_the_token() {
    let host = Ipv4Addr::new(127, 9, 8, 7);
    assert_eq!(
        address(host, 7400, "ab12"),
        "http://127.9.8.7:7400/?atollo=ab12"
    );
}

/// Never 127.0.0.1, where the projects' sites share cookies with any port.
#[test]
#[cfg(not(target_os = "macos"))]
fn loopback_is_not_the_sites_one() {
    for random in [[0, 0, 0], [253, 255, 253], [255, 255, 255], [0, 0, 254]] {
        let ip = private_loopback(random);
        assert!(ip.is_loopback(), "{ip}");
        assert_ne!(ip, Ipv4Addr::LOCALHOST);
        assert_ne!(ip.octets()[1], 0, "{ip}");
        assert!(!matches!(ip.octets()[3], 0 | 255), "{ip}");
    }
}

/// The router answers only for its own address and port, and the cookie it
/// sets is bound to that host.
#[test]
fn router_checks_its_own_host() {
    let router = include_str!("../../../runtime/phpmyadmin.php");
    assert!(router.contains("getenv('ATOLLO_PMA_HOST')"));
    assert!(router.contains("hash_equals($self, $host)"));
    assert!(!router.contains("localhost"), "no shared host accepted");
}

/// The config written into phpMyAdmin's folder reads everything from the
/// environment: no password, port or key in the file.
#[test]
fn config_comes_from_the_environment() {
    for name in [
        "ATOLLO_PMA_TOKEN",
        "ATOLLO_DB_PORT",
        "ATOLLO_DB_PASSWORD",
        "ATOLLO_PMA_SECRET",
        "ATOLLO_PMA_TMP",
    ] {
        assert!(CONFIG.contains(&format!("getenv('{name}')")), "{name}");
    }
    assert!(CONFIG.contains("'auth_type'] = 'config'"));
    assert!(CONFIG.contains("$cfg['VersionCheck'] = false"));
}

/// phpMyAdmin of the kit given by ATOLLO_TEST_KIT on its MariaDB already
/// running at ATOLLO_TEST_MARIADB_PORT (an open Atollo, whose data the
/// real test of `db/services/` must not start a second server on).
#[test]
fn real_phpmyadmin_on_running_mariadb() {
    let (Ok(root), Ok(db_port)) = (
        std::env::var("ATOLLO_TEST_KIT"),
        std::env::var("ATOLLO_TEST_MARIADB_PORT"),
    ) else {
        eprintln!("ATOLLO_TEST_KIT or ATOLLO_TEST_MARIADB_PORT not set: skipped");
        return;
    };
    let kit = Kit::at(PathBuf::from(root), crate::kit::Arch::detect());
    let env = KitEnv::build(
        &kit,
        &crate::settings::Settings::default(),
        "en",
        &crate::env::Host::capture(),
    );
    let pma = PhpMyAdmin::default();
    let url = pma
        .open(&kit, &env, db_port.parse().unwrap(), "en")
        .unwrap();
    assert_logs_in(&url);
    pma.stop();
    assert!(ureq::get(&url).call().is_err(), "stopped");
}

/// The checks of a real phpMyAdmin at `url` (the real test of
/// `db/services/`): 403 without the token, then logged in as root with it;
/// its public files are served, its code is not.
pub fn assert_logs_in(url: &str) {
    let authority = url
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap()
        .to_owned();
    let base = format!("http://{authority}");
    let (_, port) = authority.rsplit_once(':').unwrap();
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .proxy(None)
        .http_status_as_error(false)
        .max_redirects(0)
        .build()
        .into();
    let status = |path: &str, cookie: &str| {
        let mut req = agent.get(&format!("{base}{path}"));
        if !cookie.is_empty() {
            req = req.header("Cookie", cookie);
        }
        req.call().unwrap().status().as_u16()
    };
    assert_eq!(status("/", ""), 403, "no token, no phpMyAdmin");
    // The same port reached through 127.0.0.1 or localhost (where the
    // projects' sites and their cookies live) is refused, token or not.
    for other in ["127.0.0.1", "localhost"] {
        let mut answer = agent
            .get(url)
            .header("Host", &format!("{other}:{port}"))
            .call()
            .unwrap();
        assert_eq!(answer.status().as_u16(), 403, "Host {other}");
        let _ = answer.body_mut().read_to_string();
    }
    assert_eq!(status("/?atollo=0", ""), 403, "wrong token");
    assert_eq!(
        status("/themes/pmahomme/css/theme.css", ""),
        403,
        "no token, no files"
    );
    let entry = agent.get(url).call().unwrap();
    assert_eq!(entry.status().as_u16(), 302);
    let cookie: Vec<String> = entry
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .filter_map(|c| c.split(';').next().map(str::to_owned))
        .collect();
    let cookie = cookie.join("; ");
    let location = entry
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok())
        .unwrap()
        .to_owned();
    assert!(!location.contains("atollo="), "token left in {location}");
    let mut page = agent
        .get(&format!("{base}/index.php?route=/server/databases"))
        .header("Cookie", &cookie)
        .call()
        .unwrap();
    let code = page.status().as_u16();
    let body = page.body_mut().read_to_string().unwrap();
    assert_eq!(code, 200, "{body}");
    assert!(!body.contains("input_password"), "no login form: {body}");
    assert!(
        body.contains("information_schema"),
        "databases listed: {body}"
    );
    assert_eq!(status("/themes/pmahomme/css/theme.css", &cookie), 200);
    assert_eq!(status("/doc/html/index.html", &cookie), 200, "local help");
    for hidden in [
        "/config.inc.php",
        "/libraries/config.default.php",
        "/vendor/autoload.php",
        "/setup/index.php",
        "/show_config_errors.php",
        "/js/../config.inc.php",
        "/js/%2e%2e/config.inc.php",
    ] {
        assert_eq!(status(hidden, &cookie), 404, "{hidden}");
    }
}
