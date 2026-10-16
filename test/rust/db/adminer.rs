use super::*;

#[test]
fn address_carries_the_token() {
    assert_eq!(address(7400, "ab12"), "http://127.0.0.1:7400/?atollo=ab12");
}

/// Adminer of the kit given by ATOLLO_TEST_KIT on its MariaDB already
/// running at ATOLLO_TEST_MARIADB_PORT (an open Atollo, whose data the
/// real test of `db/services/` must not start a second server on).
#[test]
fn real_adminer_on_running_mariadb() {
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
    let adminer = Adminer::default();
    let url = adminer.open(&kit, &env, db_port.parse().unwrap()).unwrap();
    assert_logs_in(&url);
    adminer.stop();
    assert!(ureq::get(&url).call().is_err(), "stopped");
}

/// The checks of a real Adminer at `url` (the real test of `db/services/`):
/// 403 without the token, then logged in as root with it.
pub fn assert_logs_in(url: &str) {
    let port = url
        .trim_start_matches("http://127.0.0.1:")
        .split('/')
        .next()
        .unwrap()
        .to_owned();
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .proxy(None)
        .http_status_as_error(false)
        .max_redirects(0)
        .build()
        .into();
    let bare = agent
        .get(&format!("http://127.0.0.1:{port}/"))
        .call()
        .unwrap();
    assert_eq!(bare.status().as_u16(), 403, "no token, no Adminer");
    let wrong = agent
        .get(&format!("http://127.0.0.1:{port}/?atollo=0"))
        .call()
        .unwrap();
    assert_eq!(wrong.status().as_u16(), 403, "wrong token");
    let entry = agent.get(url).call().unwrap();
    assert_eq!(entry.status().as_u16(), 302);
    let cookies: Vec<String> = entry
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .filter_map(|c| c.split(';').next().map(str::to_owned))
        .collect();
    let location = entry
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok())
        .unwrap()
        .to_owned();
    assert!(!location.contains("atollo="), "token left in {location}");
    let mut page = agent
        .get(&format!("http://127.0.0.1:{port}/{location}"))
        .header("Cookie", cookies.join("; "))
        .call()
        .unwrap();
    let status = page.status().as_u16();
    let body = page.body_mut().read_to_string().unwrap();
    assert_eq!(status, 200, "{body}");
    assert!(body.contains("db=mysql"), "databases listed: {body}");
}
