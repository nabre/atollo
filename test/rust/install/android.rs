use super::*;

#[test]
fn tools_from_the_download_page() {
    let page = r#"<tr><td><nobr>Windows</nobr></td>
      <td>
        <button class="devsite-dialog-button button-white button-regular"
            data-modal-dialog-id="sdk_win_download"
            >commandlinetools-win-15859902_latest.zip</button>
      </td>
      <td>155.7 MB</td>
      <td>90ae805d20434428bffcb699c290860f19bb5f66a67e6b330067e3de801fb04a</td>
    </tr>"#;
    let (name, build, sha) = tools_on_page(page).unwrap();
    assert_eq!(name, "commandlinetools-win-15859902_latest.zip");
    assert_eq!(build, "15859902");
    assert!(sha.starts_with("90ae805d"));
    assert!(tools_on_page("<td>commandlinetools-mac-1_latest.zip</td>").is_none());
}

#[test]
fn newest_stable_packages() {
    let list = "Installed packages:\n  Path | Version | Description | Location\n\
            Available Packages:\n  Path                 | Version | Description\n  -------              | ------- | -------\n\
            build-tools;35.0.1   | 35.0.1  | Android SDK Build-Tools 35.0.1\n\
            build-tools;36.0.0   | 36.0.0  | Android SDK Build-Tools 36\n\
            build-tools;37.0.0-rc1 | 37.0.0 rc1 | Android SDK Build-Tools 37-rc1\n\
            platforms;android-35 | 2       | Android SDK Platform 35\n\
            platforms;android-36 | 2       | Android SDK Platform 36\n\
            platforms;android-36-ext18 | 1 | Android SDK Platform 36-ext18\n\
            platforms;android-CinnamonBun | 1 | Android SDK Platform CinnamonBun\n";
    let (platform, build) = newest_packages(list);
    assert_eq!(platform.as_deref(), Some("platforms;android-36"));
    assert_eq!(build.as_deref(), Some("build-tools;36.0.0"));
    assert_eq!(newest_packages(""), (None, None));
}

/// The real download page (`cargo test -- --ignored`).
#[test]
#[ignore]
fn real_download_page() {
    let net = crate::net::Net::new(&crate::settings::Settings::default(), None).unwrap();
    assert!(tools_on_page(&net.text(STUDIO).unwrap()).is_some());
}
