use super::*;

#[test]
fn address_carries_the_token() {
    assert_eq!(address(7400, "ab12"), "http://127.0.0.1:7400/?devkit=ab12");
}

/// The config written into phpMyAdmin's folder reads everything from the
/// environment: no password, port or key in the file.
#[test]
fn config_comes_from_the_environment() {
    for name in [
        "DEVKIT_PMA_TOKEN",
        "DEVKIT_DB_PORT",
        "DEVKIT_DB_PASSWORD",
        "DEVKIT_PMA_SECRET",
        "DEVKIT_PMA_TMP",
    ] {
        assert!(CONFIG.contains(&format!("getenv('{name}')")), "{name}");
    }
    assert!(CONFIG.contains("'auth_type'] = 'config'"));
    assert!(CONFIG.contains("$cfg['VersionCheck'] = false"));
}

/// phpMyAdmin of the kit given by DEVKIT_TEST_KIT on its MariaDB already
/// running at DEVKIT_TEST_MARIADB_PORT (an open Dev-Kit, whose data the
/// real test of `db/services/` must not start a second server on).
#[test]
fn real_phpmyadmin_on_running_mariadb() {
    let (Ok(root), Ok(db_port)) = (
        std::env::var("DEVKIT_TEST_KIT"),
        std::env::var("DEVKIT_TEST_MARIADB_PORT"),
    ) else {
        eprintln!("DEVKIT_TEST_KIT or DEVKIT_TEST_MARIADB_PORT not set: skipped");
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
    let port = url
        .trim_start_matches("http://127.0.0.1:")
        .split('/')
        .next()
        .unwrap()
        .to_owned();
    let base = format!("http://127.0.0.1:{port}");
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
    assert_eq!(status("/?devkit=0", ""), 403, "wrong token");
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
    assert!(!location.contains("devkit="), "token left in {location}");
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
