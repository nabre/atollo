use super::*;

#[test]
fn secrets_are_masked() {
    assert_eq!(
        mask_secrets("GEMINI_API_KEY=abc123 next"),
        "GEMINI_API_KEY=*** next"
    );
    assert_eq!(
        mask_secrets("url ?tkn=deadbeef&folder=x"),
        "url ?tkn=***&folder=x"
    );
    assert_eq!(mask_secrets("GH_TOKEN=\"x y\" ok"), "GH_TOKEN=\"***\" ok");
    assert_eq!(mask_secrets("SECRET='a b"), "SECRET='***'");
    assert_eq!(
        mask_secrets("PATH=C:\\x MY_PASSWORD=p"),
        "PATH=C:\\x MY_PASSWORD=***"
    );
    assert_eq!(mask_secrets("no secrets here"), "no secrets here");
}

/// What `Atollo env` and `KitEnv`'s `Debug` show: every `*_TOKEN`,
/// `*_KEY`, secret and password masked, a proxy's password too; the rest
/// as it is.
#[test]
fn secret_values_are_masked_by_name() {
    for name in [
        "CLAUDE_CODE_OAUTH_TOKEN",
        "GH_TOKEN",
        "github_token",
        "GEMINI_API_KEY",
        "ATOLLO_FCGI_KEY",
        "CLIENT_SECRET",
        "DB_PASSWORD",
        "MYSQL_PASSWD",
        "tkn",
    ] {
        assert!(is_secret(name), "{name}");
        assert_eq!(masked(name, "sk-ant-oat01-x"), "***", "{name}");
    }
    for name in [
        "PATH",
        "HOME",
        "ATOLLO_LANG",
        "GCM_CREDENTIAL_STORE",
        "TERM",
    ] {
        assert!(!is_secret(name), "{name}");
    }
    assert_eq!(masked("PATH", "/a:/b"), "/a:/b");
    assert_eq!(
        masked("HTTPS_PROXY", "http://me:p%40ss@proxy:8080/x"),
        "http://me:***@proxy:8080/x"
    );
    assert_eq!(
        masked("HTTP_PROXY", "http://token@proxy:8080"),
        "http://token:***@proxy:8080"
    );
    assert_eq!(
        masked("HTTP_PROXY", "http://proxy:8080/a@b"),
        "http://proxy:8080/a@b"
    );
    assert_eq!(
        masked("HTTP_PROXY", "http://proxy:8080"),
        "http://proxy:8080"
    );
}
