use super::*;
use crate::kit::tests::TempDir;
use crate::test_support::prepared_kit;
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
    assert!(bad(|s| s.rust_toolchain = Some("x".into())));
    assert!(bad(|s| s.gemini_api_key = Some("a b".into())));
}

#[test]
fn round_trip_and_partial_file() {
    let (_tmp, kit) = prepared_kit("settings-rt");
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

#[test]
fn keys_of_a_newer_version_are_kept() {
    let (_tmp, kit) = prepared_kit("settings-other");
    fs::write(
        kit.settings_file(),
        r#"{"lang":"it","theme":"dark","network":{"allowLan":true,"pac":"http://wpad/"}}"#,
    )
    .unwrap();
    let saved = Settings::update(&kit, |s, problem| {
        assert!(problem.is_none());
        s.auto_update = false;
        Ok::<(), anyhow::Error>(())
    })
    .unwrap();
    assert!(!saved.auto_update);
    let text = fs::read_to_string(kit.settings_file()).unwrap();
    assert!(text.contains("\"theme\": \"dark\""), "{text}");
    assert!(text.contains("\"pac\": \"http://wpad/\""), "{text}");
    assert!(text.contains("\"allowLan\": true"), "{text}");
}

#[test]
fn update_refused_leaves_the_file() {
    let (_tmp, kit) = prepared_kit("settings-refused");
    fs::write(kit.settings_file(), "{\"lang\":\"de\"}").unwrap();
    let refused = Settings::update(&kit, |s, _| {
        s.lang = Some("es".into());
        anyhow::bail!("no")
    });
    assert!(refused.is_err());
    assert_eq!(
        fs::read_to_string(kit.settings_file()).unwrap(),
        "{\"lang\":\"de\"}"
    );
}

#[test]
fn gemini_key_lives_in_its_own_file() {
    let (_tmp, kit) = prepared_kit("settings-gemini");
    let s = Settings {
        gemini_api_key: Some("AIza-secret".into()),
        ..Settings::default()
    };
    s.save(&kit).unwrap();
    let json = fs::read_to_string(kit.settings_file()).unwrap();
    assert!(!json.contains("AIza-secret"), "{json}");
    assert_eq!(
        fs::read_to_string(gemini_file(&kit)).unwrap(),
        "AIza-secret"
    );
    assert_eq!(
        Settings::load(&kit).settings.gemini_api_key.as_deref(),
        Some("AIza-secret")
    );
    Settings::default().save(&kit).unwrap();
    assert!(!gemini_file(&kit).exists(), "no key, no file");
}

/// PORT-V6-01: a key file that is there but cannot be read is an error
/// naming it, never "no key"; an older key does not take its place.
#[test]
fn unreadable_gemini_key_is_not_absent() {
    let tmp = TempDir::new("settings-gemini-unreadable");
    let kit = Kit::at(tmp.0.join("O’Brien {kit}"), crate::test_support::kit_arch());
    kit.prepare().unwrap();
    let file = gemini_file(&kit);
    assert!(read_secret(&file).unwrap().is_none(), "absent");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, " \n").unwrap();
    assert!(read_secret(&file).unwrap().is_none(), "empty");
    fs::remove_file(&file).unwrap();
    // Something there that cannot be read as the key.
    fs::create_dir_all(&file).unwrap();
    let e = format!("{:#}", read_secret(&file).unwrap_err());
    assert!(e.contains("gemini-api-key"), "{e}");
    fs::write(kit.settings_file(), "{\"geminiApiKey\":\"AIza-old\"}").unwrap();
    move_secrets(&kit);
    assert!(file.is_dir(), "left as it is");
    assert!(fs::read_to_string(kit.settings_file())
        .unwrap()
        .contains("AIza-old"));
    // Another setting saved meanwhile neither fails nor removes the key.
    fs::remove_file(kit.settings_file()).unwrap();
    let saved = Settings::update(&kit, |s, _| {
        s.auto_update = false;
        Ok::<(), anyhow::Error>(())
    })
    .unwrap();
    assert!(saved.gemini_unreadable && file.is_dir(), "kept");
}

#[test]
fn gemini_key_of_an_older_version_moves() {
    let (_tmp, kit) = prepared_kit("settings-gemini-old");
    fs::write(
        kit.settings_file(),
        "{\"lang\":\"fr\",\"geminiApiKey\":\"AIza-old\"}",
    )
    .unwrap();
    move_secrets(&kit);
    let json = fs::read_to_string(kit.settings_file()).unwrap();
    assert!(!json.contains("AIza-old"), "{json}");
    assert!(json.contains("\"lang\": \"fr\""), "{json}");
    assert_eq!(
        Settings::load(&kit).settings.gemini_api_key.as_deref(),
        Some("AIza-old")
    );
}

#[test]
fn pem_file_in_the_kit_is_saved_relative() {
    let (_tmp, kit) = prepared_kit("settings-pem");
    let pem = kit.data().join("certs").join("company.pem");
    let mut s = Settings::default();
    s.network.extra_ca_certs = Some(pem.display().to_string());
    s.save(&kit).unwrap();
    let json = fs::read_to_string(kit.settings_file()).unwrap();
    assert!(
        json.contains("\"extraCaCerts\": \"data/certs/company.pem\""),
        "{json}"
    );
    assert_eq!(
        Settings::load(&kit)
            .settings
            .network
            .extra_ca_certs
            .map(std::path::PathBuf::from),
        Some(pem)
    );
}

#[test]
fn theme_default_and_unknown() {
    assert_eq!(Settings::default().theme, Theme::System);
    assert_eq!(Theme::System.window(), None);
    assert_eq!(Theme::Dark.window(), Some(tauri::Theme::Dark));
    let read = |json: &str| {
        let tmp = TempDir::new("settings-theme");
        let path = tmp.0.join("settings.json");
        fs::write(&path, json).unwrap();
        Settings::load_from(&path).settings.theme
    };
    assert_eq!(read("{\"theme\":\"light\"}"), Theme::Light);
    assert_eq!(read("{\"theme\":\"Dark\"}"), Theme::Dark);
    assert_eq!(read("{\"theme\":\"sepia\"}"), Theme::System);
    assert_eq!(serde_json::to_string(&Theme::Light).unwrap(), "\"light\"");
}
