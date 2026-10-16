use super::*;
use crate::kit::{tests::TempDir, Arch};
use std::fs;

#[test]
fn missing_file_gives_defaults() {
    let tmp = TempDir::new("settings-missing");
    let loaded = Settings::load_from(&tmp.0.join("settings.json"));
    assert_eq!(loaded.settings, Settings::default());
    assert!(loaded.problem.is_none());
    assert!(loaded.settings.auto_update);
}

#[test]
fn program_temp_default_and_unknown() {
    assert_eq!(Settings::default().program_temp, ProgramTemp::Auto);
    let read = |json: &str| {
        let tmp = TempDir::new("settings-ptemp");
        let path = tmp.0.join("settings.json");
        fs::write(&path, json).unwrap();
        Settings::load_from(&path).settings.program_temp
    };
    assert_eq!(read("{\"lang\":\"it\"}"), ProgramTemp::Auto);
    assert_eq!(read("{\"programTemp\":\"kit\"}"), ProgramTemp::Kit);
    assert_eq!(read("{\"programTemp\":\"system\"}"), ProgramTemp::System);
    assert_eq!(read("{\"programTemp\":\"banana\"}"), ProgramTemp::Auto);
    assert_eq!(
        serde_json::to_string(&ProgramTemp::System).unwrap(),
        "\"system\""
    );
}

#[test]
fn validation() {
    let ok = Settings {
        lang: Some("de".into()),
        python_version: Some("3.13".into()),
        php_version: Some("8.4".into()),
        rust_toolchain: Some("msvc".into()),
        ..Settings::default()
    };
    assert!(ok.validate().is_ok());
    let bad = |f: fn(&mut Settings)| {
        let mut s = Settings::default();
        f(&mut s);
        s.validate().is_err()
    };
    assert!(bad(|s| s.lang = Some("es".into())));
    assert!(bad(|s| s.network.https_proxy = Some("proxy:8080".into())));
    assert!(bad(
        |s| s.network.extra_ca_certs = Some(r"Z:\none.pem".into())
    ));
    assert!(bad(|s| s.python_version = Some("3.13 & x".into())));
    assert!(bad(|s| s.php_version = Some("7.4".into())));
    assert!(bad(|s| s.rust_toolchain = Some("x".into())));
    assert!(bad(|s| s.gemini_api_key = Some("a b".into())));
}

#[test]
fn round_trip_and_partial_file() {
    let tmp = TempDir::new("settings-rt");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let s = Settings {
        lang: Some("de".into()),
        network: Network {
            allow_lan: true,
            ..Network::default()
        },
        ..Settings::default()
    };
    s.save(&kit).unwrap();
    assert_eq!(Settings::load(&kit).settings, s);

    fs::write(kit.settings_file(), "\u{feff}{\"lang\":\"fr\"}").unwrap();
    let loaded = Settings::load(&kit).settings;
    assert_eq!(loaded.lang.as_deref(), Some("fr"));
    assert!(loaded.auto_update, "missing keys take the default");
}

#[test]
fn broken_file_is_reported_not_replaced() {
    let tmp = TempDir::new("settings-bad");
    let path = tmp.0.join("settings.json");
    fs::write(&path, "{ \"lang\": ").unwrap();
    let loaded = Settings::load_from(&path);
    assert!(loaded.problem.is_some());
    assert_eq!(fs::read_to_string(&path).unwrap(), "{ \"lang\": ");
}
