//! The `php.ini` of each line of PHP in `data\php\<line>`: written once from
//! the line's template (or the kit's own for static-php-cli), then the
//! user's; an older kit's single `php.ini` moved to the default line.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::common::fsx::write_atomic;
use crate::env::php_target;
use crate::install::pattern;

pub use crate::env::php_ini_dir;

/// The `php.ini` of the PHP in `php_dir`: written from that PHP's own
/// `php.ini-development` when missing (then it is the user's); an existing
/// one only gets the paths of an older kit fixed.
pub fn ensure_php_ini(data: &Path, php_dir: &Path) -> Result<()> {
    let ini = php_ini_dir(data, php_dir).join("php.ini");
    let dev_ini = php_dir.join("php.ini-development");
    if !ini.exists() && dev_ini.is_file() {
        let template = fs::read_to_string(&dev_ini).context("php.ini-development")?;
        fs::create_dir_all(ini.parent().unwrap_or(php_dir))?;
        write_atomic(&ini, php_ini(&template)?.as_bytes())?;
    } else if !ini.exists() && php_dir.join("bin").join("php").is_file() {
        // static-php-cli (macOS, Linux) ships no template: the kit's own.
        fs::create_dir_all(ini.parent().unwrap_or(php_dir))?;
        write_atomic(&ini, php_ini_unix().as_bytes())?;
    } else if let Ok(text) = fs::read_to_string(&ini) {
        // Written before the saved versions (D-80): the paths follow the
        // PHP in use, not the default folder.
        let new = php_ini_paths(&text);
        if new != text {
            write_atomic(&ini, new.as_bytes())?;
        }
    }
    Ok(())
}

/// An older kit had one `data\php\php.ini` for every PHP: it becomes the
/// `php.ini` of the default line (the user's changes stay with the PHP the
/// kit used), and every line on disk without one gets its own from its
/// template. At start, with `versions::migrate`, and after PHP is installed.
pub fn migrate_php_ini(kit: &crate::kit::Kit) -> Result<()> {
    let Some(php) = crate::setup::Catalog::get().find("php") else {
        return Ok(());
    };
    let data = &kit.data();
    let default = crate::versions::default_dir(kit, php);
    let old = data.join("php").join("php.ini");
    let new = php_ini_dir(data, &default).join("php.ini");
    // That single php.ini was always Windows': never the one of a Unix PHP.
    let windows = php_target(data, &default).is_none_or(|t| t.os == crate::kit::Os::Win32);
    if windows
        && old.is_file()
        && new != old
        && !new.exists()
        && crate::versions::has_key(php, &default)
    {
        fs::create_dir_all(new.parent().unwrap_or(data))?;
        fs::rename(&old, &new).with_context(|| new.display().to_string())?;
        crate::common::log::info(&format!("php.ini: moved to {}", new.display()));
    }
    let dirs = std::iter::once(default).chain(
        crate::versions::saved(kit, php)
            .into_iter()
            .map(|l| crate::versions::saved_dir(kit, php, &l.line)),
    );
    for dir in dirs.filter(|d| crate::versions::has_key(php, d)) {
        ensure_php_ini(data, &dir)?;
    }
    Ok(())
}

/// `${CP_APPS}\php\…` of an older kit's `php.ini` as `${CP_PHP}\…`.
pub fn php_ini_paths(text: &str) -> String {
    text.replace(r"${CP_APPS}\php\", r"${CP_PHP}\")
}

/// The kit's `php.ini` from `php.ini-development`: paths through the kit
/// variables (valid on any drive letter), common extensions on,
/// Europe/Rome, CA bundle for curl and OpenSSL.
pub fn php_ini(template: &str) -> Result<String> {
    let mut t = template.to_owned();
    let swaps: [(&str, &str); 5] = [
        (
            r#"(?m)^;\s*extension_dir\s*=\s*"ext"[ \t]*(\r?)$"#,
            r#"extension_dir = "$${CP_PHP}\ext"$1"#,
        ),
        (
            // `curl` since 7.2, `php_curl.dll` before; `gd2` until 7.4.
            r"(?m)^;(extension=(?:php_)?(?:curl|fileinfo|gd2?|intl|mbstring|mysqli|openssl|pdo_mysql|pdo_sqlite|sodium|sqlite3|zip)(?:\.dll)?)[ \t]*(\r?)$",
            "$1$2",
        ),
        (
            r"(?m)^;date\.timezone\s*=[^\r\n]*",
            "date.timezone = Europe/Rome",
        ),
        (
            r"(?m)^;curl\.cainfo\s*=[^\r\n]*",
            r#"curl.cainfo = "$${CP_PHP}\extras\ssl\cacert.pem""#,
        ),
        (
            r"(?m)^;openssl\.cafile\s*=[^\r\n]*",
            r#"openssl.cafile = "$${CP_PHP}\extras\ssl\cacert.pem""#,
        ),
    ];
    for (re, with) in swaps {
        t = pattern(re)?.replace_all(&t, with).into_owned();
    }
    Ok(t)
}

/// The kit's `php.ini` for static-php-cli's PHP (macOS and Linux), which
/// ships no `php.ini-development`: that file's development values, no
/// `extension=` line (the extensions are built in, there is no `ext`
/// folder), Europe/Rome, the CA bundle for curl and OpenSSL through
/// `${CP_PHP}` with `/`.
pub fn php_ini_unix() -> String {
    [
        "; Atollo php.ini for the PHP of static-php-cli (macOS, Linux): yours,",
        "; updates never touch it. Its extensions are built in: no extension= line.",
        "; The kit's own build (BUILDINFO.txt next to bin/) has pdo_sqlite and",
        "; pdo_pgsql. static-php-cli's bulk build, used for a line the kit has no",
        "; build of, has SQLite through PDO by swoole-hook-sqlite only: php -m does",
        "; not list pdo_sqlite, so a composer.json that requires ext-pdo_sqlite and",
        "; extension_loaded('pdo_sqlite') do not see it.",
        "[PHP]",
        "engine = On",
        "short_open_tag = Off",
        "precision = 14",
        "output_buffering = 4096",
        "zlib.output_compression = Off",
        "implicit_flush = Off",
        "serialize_precision = -1",
        "zend.enable_gc = On",
        "zend.exception_ignore_args = Off",
        "zend.exception_string_param_max_len = 15",
        "expose_php = On",
        "max_execution_time = 30",
        "max_input_time = 60",
        "memory_limit = 128M",
        "error_reporting = E_ALL",
        "display_errors = On",
        "display_startup_errors = On",
        "log_errors = On",
        "ignore_repeated_errors = Off",
        "ignore_repeated_source = Off",
        "report_memleaks = On",
        "variables_order = \"GPCS\"",
        "request_order = \"GP\"",
        "register_argc_argv = Off",
        "auto_globals_jit = On",
        "post_max_size = 8M",
        "default_mimetype = \"text/html\"",
        "default_charset = \"UTF-8\"",
        "enable_dl = Off",
        "file_uploads = On",
        "upload_max_filesize = 2M",
        "max_file_uploads = 20",
        "allow_url_fopen = On",
        "allow_url_include = Off",
        "default_socket_timeout = 60",
        "",
        "[Date]",
        "date.timezone = Europe/Rome",
        "",
        "[Session]",
        "session.save_handler = files",
        "session.use_strict_mode = 0",
        "session.use_cookies = 1",
        "session.use_only_cookies = 1",
        "session.name = PHPSESSID",
        "session.cookie_httponly =",
        "session.cookie_samesite =",
        "session.gc_probability = 1",
        "session.gc_divisor = 1000",
        "session.gc_maxlifetime = 1440",
        "",
        "[Assertion]",
        "zend.assertions = 1",
        "",
        "[curl]",
        "curl.cainfo = \"${CP_PHP}/extras/ssl/cacert.pem\"",
        "",
        "[openssl]",
        "openssl.cafile = \"${CP_PHP}/extras/ssl/cacert.pem\"",
        "",
    ]
    .join("\n")
}

#[cfg(test)]
#[path = "../../../../test/rust/install/web/php_ini.rs"]
mod tests;
