<?php
// Router of PHP's server for Adminer on the kit's MariaDB (DECISIONS.md D-85),
// started by NEXKORA from the Database page. Written by the app: a hand edit
// is replaced at the next start.
//
// - only whoever has this start's token gets in: the app opens
//   `/?devkit=<token>`, which leaves a cookie (HttpOnly, SameSite=Strict)
//   and logs in as root; any other request without the cookie gets 403;
// - root's password stays in this process's environment: never in the
//   address, the session or the browser;
// - only requests addressed to 127.0.0.1 or localhost (no DNS rebinding).

$token = (string) getenv('DEVKIT_ADMINER_TOKEN');
$host = strtolower($_SERVER['HTTP_HOST'] ?? '');
if ($token === '' || !preg_match('~^(127\.0\.0\.1|localhost):\d+$~', $host)) {
    http_response_code(403);
    exit;
}
$given = $_GET['devkit'] ?? null;
if (is_string($given) && hash_equals($token, $given)) {
    setcookie('devkit_adminer', $token, ['path' => '/', 'httponly' => true, 'samesite' => 'Strict']);
    // The token leaves the address Adminer redirects to.
    unset($_GET['devkit']);
    $_SERVER['REQUEST_URI'] = rtrim(preg_replace('~(?<=[?&])devkit=[^&]*(&|$)~', '', $_SERVER['REQUEST_URI']), '?&');
    define('DEVKIT_ENTRY', true);
    // Adminer's own login form, sent for the user: the password comes from
    // credentials() below.
    $_POST = ['auth' => ['driver' => 'server', 'server' => '', 'username' => 'root', 'password' => '', 'db' => '']];
} elseif (!hash_equals($token, (string) ($_COOKIE['devkit_adminer'] ?? ''))) {
    http_response_code(403);
    header('Content-Type: text/plain; charset=utf-8');
    echo "Adminer: open it from NEXKORA, Database page.\n";
    exit;
}

function adminer_object() {
    class DevKitAdminer extends \Adminer\Adminer {
        function credentials() {
            return ['127.0.0.1:' . getenv('DEVKIT_DB_PORT'), 'root', (string) getenv('DEVKIT_DB_PASSWORD')];
        }

        function login($login, $password) {
            return true;
        }

        // The login sent above has no form token; every other form keeps it.
        function verifyLoginToken() {
            return !defined('DEVKIT_ENTRY');
        }
    }
    return new DevKitAdminer;
}

include getenv('DEVKIT_ADMINER');
