use super::*;
use serde_json::json;

#[test]
fn php_branch_and_build() {
    let releases = json!({
        "8.3": {"version": "8.3.20", "ts-vs16-x64": {"zip": {"path": "php-8.3.20-Win32-vs16-x64.zip", "sha256": "aa"}}},
        "8.4": {"version": "8.4.9", "nts-vs17-x64": {"zip": {"path": "nts.zip", "sha256": "bb"}}, "ts-vs17-x64": {"zip": {"path": "php-8.4.9-Win32-vs17-x64.zip", "sha256": "cc"}}},
    });
    assert_eq!(
        pick_php(&releases, None).unwrap(),
        (
            "8.4.9".into(),
            "php-8.4.9-Win32-vs17-x64.zip".into(),
            "cc".into()
        )
    );
    assert_eq!(pick_php(&releases, Some("8.3")).unwrap().0, "8.3.20");
    assert!(pick_php(&releases, Some("7.4")).is_err());
    assert_eq!(php_lines(&releases), ["8.4", "8.3"]);

    // PHP 7 and 5.6 builds are supported alongside PHP 8.
    let mut with_7 = releases.clone();
    with_7["7.4"] = json!({"version": "7.4.33", "ts-vc15-x64": {"zip": {"path": "php-7.4.33.zip", "sha256": "dd"}}});
    assert_eq!(php_lines(&with_7), ["8.4", "8.3", "7.4"]);
}

#[test]
fn composer_for_each_php() {
    let versions = json!({
        "stable": [
            {"path": "/download/2.10.3/composer.phar", "version": "2.10.3", "min-php": 70205}
        ],
        "preview": [{"path": "/download/2.11.0-RC1/composer.phar", "version": "2.11.0-RC1", "min-php": 70205}],
        "2.2": [{"path": "/download/2.2.30/composer.phar", "version": "2.2.30", "min-php": 50300}]
    });
    let stable = (
        "2.10.3".to_owned(),
        "https://getcomposer.org/download/2.10.3/composer.phar".to_owned(),
    );
    assert_eq!(pick_composer(&versions, "8.4.13").unwrap(), stable);
    assert_eq!(pick_composer(&versions, "7.2.5").unwrap(), stable);
    assert_eq!(
        pick_composer(&versions, "7.1.33").unwrap().0,
        "2.2.30",
        "older PHP: the LTS"
    );
    assert!(pick_composer(&json!({"stable": []}), "8.4.1").is_err());
    assert!(pick_composer(
        &json!({"stable": [{"path": "https://x/composer.phar", "version": "1"}]}),
        "8.4.1"
    )
    .is_err());

    let tmp = crate::kit::tests::TempDir::new("composer-launchers");
    composer_launchers(&tmp.0).unwrap();
    let cmd = fs::read_to_string(tmp.0.join("composer.cmd")).unwrap();
    assert!(
        cmd.contains(r#""%~dp0php.exe" "%~dp0composer.phar""#),
        "{cmd}"
    );
    // PowerShell's launcher: the `^` of a version constraint reaches
    // Composer, cmd.exe would drop it.
    let ps1 = fs::read_to_string(tmp.0.join("composer.ps1")).unwrap();
    assert!(
        ps1.contains("& $php (Join-Path $PSScriptRoot 'composer.phar') @args\r\n"),
        "{ps1}"
    );
    assert!(ps1.ends_with("exit $LASTEXITCODE\r\n"), "{ps1}");
}
