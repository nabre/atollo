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
    let net = crate::net::Net::new(&crate::common::settings::Settings::default(), None).unwrap();
    assert!(tools_on_page(&net.text(STUDIO).unwrap()).is_some());
}

/// Since build 15859902 the page has one Mac zip per processor (checked on
/// developer.android.com/studio on 19/10/2026); a page with the older single
/// Mac zip still works.
#[test]
fn mac_tools_by_processor() {
    let row = |name: &str| {
        format!(
            r#"<tr><td>Mac</td><td><button class="devsite-dialog-button"
            >{name}</button></td><td>150 MB</td>
            <td>{}</td></tr>"#,
            "b".repeat(64)
        )
    };
    let page = row("commandlinetools-mac_arm64-15859902_latest.zip")
        + &row("commandlinetools-mac_x86_64-15859902_latest.zip")
        + &row("commandlinetools-linux-15859902_latest.zip");
    let pick = |target: Target, page: &str| {
        os_tags_of(target)
            .iter()
            .find_map(|tag| tools_on_page_for(page, tag))
            .map(|(name, _, _)| name)
    };
    assert_eq!(
        pick(Target::DARWIN_ARM64, &page).as_deref(),
        Some("commandlinetools-mac_arm64-15859902_latest.zip")
    );
    assert_eq!(
        pick(Target::LINUX_ARM64, &page).as_deref(),
        Some("commandlinetools-linux-15859902_latest.zip")
    );
    let old = row("commandlinetools-mac-13114758_latest.zip");
    assert_eq!(
        pick(Target::DARWIN_ARM64, &old).as_deref(),
        Some("commandlinetools-mac-13114758_latest.zip")
    );
}

/// The sdkmanager needs the kit's own Java, built for its target: from a
/// PC that cannot run that target it completes on the first start there.
#[test]
fn android_waits_for_a_system_whose_java_cannot_run_here() {
    let android = Catalog::get().find("android").unwrap();
    for target in Target::ALL {
        let staged = crate::router::bootstrap::stages_for(&Android, android, target);
        if !crate::router::target::can_execute_on_host(target) {
            assert!(staged, "{target}");
        }
        if target == Target::detect() {
            assert!(!staged, "{target}: this PC");
        }
    }
}

/// The real download page has the zip of every system (`cargo test -- --ignored`).
#[test]
#[ignore]
fn real_download_page_every_system() {
    let net = crate::net::Net::new(&crate::common::settings::Settings::default(), None).unwrap();
    let page = net.text(STUDIO).unwrap();
    for target in Target::ALL {
        assert!(
            os_tags_of(target)
                .iter()
                .any(|tag| tools_on_page_for(&page, tag).is_some()),
            "{target}"
        );
    }
}
