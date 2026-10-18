use super::*;
use crate::kit::Target;

/// The FastCGI server Caddy runs on each of the six targets: php-cgi on
/// Windows (as Nginx's), static-php-cli's `php-fpm` elsewhere (no php-cgi).
#[test]
fn fastcgi_program_per_target() {
    let dir = Path::new("kit/php/8.4");
    for t in Target::ALL {
        let want = if t.is_windows() {
            dir.join("php-cgi.exe")
        } else {
            dir.join("sbin").join("php-fpm")
        };
        assert_eq!(fastcgi_exe(t.os, dir), want, "{t}");
    }
}

/// `fpm.conf` (T3.3.4): in the foreground, its files in the server's
/// folder, 127.0.0.1 only, workers on demand, the kit's environment kept,
/// the files PHP runs for Apache and Nginx.
#[test]
fn fpm_conf_expected() {
    let conf = fpm_conf(Path::new("/kit/data/servers/caddy-2.11_php-8.4"), 9123);
    let want = "; Dev-Kit: php-fpm of this web server (written at every start)\n\
[global]\n\
pid = \"/kit/data/servers/caddy-2.11_php-8.4/php-fpm.pid\"\n\
error_log = \"/kit/data/servers/caddy-2.11_php-8.4/php-fpm.log\"\n\
daemonize = no\n\
\n\
[www]\n\
listen = 127.0.0.1:9123\n\
listen.allowed_clients = 127.0.0.1\n\
pm = ondemand\n\
pm.max_children = 16\n\
pm.process_idle_timeout = 30s\n\
pm.max_requests = 500\n\
clear_env = no\n\
catch_workers_output = yes\n\
decorate_workers_output = no\n\
security.limit_extensions = .php .phtml .phar\n";
    assert_eq!(conf, want);
}

/// php-fpm gets the guard and its key as php-cgi does (D-99 «Resta»), in
/// the foreground, with the line's `php.ini` folder.
#[test]
fn fpm_arguments() {
    let args = fpm_args(
        Path::new("/kit/data/servers/p/fpm.conf"),
        Path::new("/kit/data/php/8.4"),
        Path::new("/kit/data/kit/fcgi-guard.php"),
        "k3y",
    );
    assert_eq!(
        args,
        [
            "--nodaemonize",
            "--fpm-config",
            "/kit/data/servers/p/fpm.conf",
            "-c",
            "/kit/data/php/8.4",
            "-d",
            "devkit.fcgi_key=k3y",
            "-d",
            "auto_prepend_file=\"/kit/data/kit/fcgi-guard.php\""
        ]
    );
    let guard = include_str!("../../../runtime/fcgi-guard.php");
    assert!(guard.contains("Caddy") && guard.contains("php-fpm"));
}
