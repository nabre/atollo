use super::*;

/// phpMyAdmin's version.json (2025-10): the newest release, every language.
#[test]
fn phpmyadmin_zip_from_version_json() {
    let versions: Value = serde_json::from_str(
        r#"{"version":"5.2.3","date":"2025-10-08","releases":[
            {"version":"5.2.3","date":"2025-10-08","php_versions":">=7.2,<8.4","mysql_versions":">=5.5"},
            {"version":"4.9.11","date":"2023-02-08","php_versions":">=5.5,<8.0","mysql_versions":">=5.5"}]}"#,
    )
    .unwrap();
    let (v, url) = phpmyadmin_zip(&versions).unwrap();
    assert_eq!(v, "5.2.3");
    assert_eq!(
        url,
        "https://files.phpmyadmin.net/phpMyAdmin/5.2.3/phpMyAdmin-5.2.3-all-languages.zip"
    );
    // Only digits and dots reach the address.
    for bad in [r#"{}"#, r#"{"version":""}"#, r#"{"version":"5.2/../x"}"#] {
        assert!(
            phpmyadmin_zip(&serde_json::from_str(bad).unwrap()).is_err(),
            "{bad}"
        );
    }
}
