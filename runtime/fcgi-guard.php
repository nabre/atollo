<?php
// Run by PHP's FastCGI server before every script it runs for Nginx or
// Caddy (`-d auto_prepend_file`, projects\fastcgi.rs): php-cgi on Windows,
// php-fpm on macOS and Linux. Written by the app: a hand edit is replaced
// at the next start.
//
// php-cgi has no authentication of its own, and Nginx for Windows speaks
// FastCGI over TCP only: on its port of 127.0.0.1 any local program, or a
// request forged through a local site (SSRF, `gopher://`), could ask it to
// run any PHP file of the disk. So the web server sends a random key, new at
// every start, in the FastCGI parameter DEVKIT_FCGI_KEY (Caddy: `env
// DEVKIT_FCGI_KEY` of `php_fastcgi`), and PHP got the same key as an ini
// value of its command line (`-d devkit.fcgi_key=`), which no request and no
// `.user.ini` can change. Without the key: 403, nothing runs.
// (A `.user.ini` that sets its own `auto_prepend_file` replaces this file for
// the scripts below it: such a folder is not guarded.)
//
// In a function: no variable of its own reaches the script.

(static function () {
    $want = (string) get_cfg_var('devkit.fcgi_key');
    $got = (string) ($_SERVER['DEVKIT_FCGI_KEY'] ?? '');
    unset($_SERVER['DEVKIT_FCGI_KEY']);
    if ($want === '' || !hash_equals($want, $got)) {
        http_response_code(403);
        exit;
    }
})();
