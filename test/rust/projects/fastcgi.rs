use super::*;
use crate::kit::Target;

/// The FastCGI server Caddy runs on each of the four targets: php-cgi on
/// Windows, static-php-cli's `php-fpm` elsewhere (no php-cgi).
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

/// `fpm.conf` (D-111): in the foreground, its files in the server's
/// folder, on an owner-only Unix socket, workers on demand, the kit's
/// environment kept, the files PHP runs for Apache and Caddy.
#[test]
fn fpm_conf_expected() {
    let conf = fpm_conf(
        Path::new("/kit/data/servers/caddy-2.11_php-8.4"),
        Path::new("/run/user/1000/atollo-caddy-1000/c-0123456789ab/fpm.sock"),
    );
    let want = "; Atollo: php-fpm of this web server (written at every start)\n\
[global]\n\
pid = \"/kit/data/servers/caddy-2.11_php-8.4/php-fpm.pid\"\n\
error_log = \"/kit/data/servers/caddy-2.11_php-8.4/php-fpm.log\"\n\
daemonize = no\n\
\n\
[www]\n\
listen = \"/run/user/1000/atollo-caddy-1000/c-0123456789ab/fpm.sock\"\n\
listen.mode = 0600\n\
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

/// php-fpm never listens on TCP (review of D-111): it takes `PHP_VALUE`
/// and `PHP_ADMIN_VALUE` from whoever connects, and on a port of 127.0.0.1
/// any local program, or an SSRF of a local site, could set
/// `auto_prepend_file=none` and run scripts without the key.
#[test]
fn fpm_conf_has_no_tcp_listen() {
    let conf = fpm_conf(
        Path::new("/k/run"),
        Path::new("/tmp/atollo-caddy-1/c-x/fpm.sock"),
    );
    let listen: Vec<&str> = conf
        .lines()
        .filter(|l| l.trim_start().starts_with("listen"))
        .collect();
    assert_eq!(
        listen,
        [
            "listen = \"/tmp/atollo-caddy-1/c-x/fpm.sock\"",
            "listen.mode = 0600"
        ]
    );
    assert!(!conf.contains("127.0.0.1") && !conf.contains("allowed_clients"));
}

/// The address a web server is given: php-cgi on a TCP port of 127.0.0.1,
/// php-fpm on its socket (`unix/` for Caddy).
#[test]
fn fastcgi_addresses() {
    let tcp = FcgiAddr::Tcp(9123);
    assert_eq!(tcp.caddy_upstream(), "127.0.0.1:9123");
    let tmp = crate::kit::tests::TempDir::new("fastcgi-addr");
    let sock = PrivateSocket::in_leaf(&tmp.0, "c-1", "fpm.sock").unwrap();
    let path = fwd(&sock.path);
    let unix = FcgiAddr::Unix(Arc::new(sock));
    assert_eq!(unix.caddy_upstream(), format!("\"unix/{path}\""));
    assert!(unix.describe().starts_with("socket "));
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
            "atollo.fcgi_key=k3y",
            "-d",
            "auto_prepend_file=\"/kit/data/kit/fcgi-guard.php\""
        ]
    );
    let guard = include_str!("../../../runtime/fcgi-guard.php");
    assert!(guard.contains("Caddy") && guard.contains("php-fpm"));
}
