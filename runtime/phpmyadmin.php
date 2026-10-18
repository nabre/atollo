<?php
// Router of PHP's server for phpMyAdmin on the kit's MySQL (DECISIONS.md
// D-107), started by Atollo from the Database page. Written by the app: a
// hand edit is replaced at the next start.
//
// - only whoever has this start's token gets in: the app opens
//   `/?atollo=<token>`, which leaves a cookie (HttpOnly, SameSite=Strict)
//   and goes back to `/` without it; any other request without the cookie
//   gets 403;
// - root's password stays in this process's environment: never in the
//   address, the session or the browser (`config.inc.php` reads it);
// - only requests addressed to this server's own loopback address and port
//   (no DNS rebinding); the app picks an address other than 127.0.0.1,
//   where the projects' sites run, because browsers send a host's cookies to
//   all its ports: on 127.0.0.1 the cookie below and phpMyAdmin's session
//   would reach the projects' PHP;
// - PHP runs only through phpMyAdmin's entry points; the other files are
//   served only from its public folders (js, themes, doc), never its code.

$token = (string) getenv('ATOLLO_PMA_TOKEN');
$self = (string) getenv('ATOLLO_PMA_HOST') . ':' . (string) ($_SERVER['SERVER_PORT'] ?? '');
$host = strtolower($_SERVER['HTTP_HOST'] ?? '');
if ($token === '' || getenv('ATOLLO_PMA_HOST') === false || !hash_equals($self, $host)) {
    http_response_code(403);
    exit;
}
$given = $_GET['atollo'] ?? null;
if (is_string($given) && hash_equals($token, $given)) {
    setcookie('atollo_pma', $token, ['path' => '/', 'httponly' => true, 'samesite' => 'Strict']);
    // The token leaves the address.
    header('Location: /index.php', true, 302);
    exit;
}
if (!hash_equals($token, (string) ($_COOKIE['atollo_pma'] ?? ''))) {
    http_response_code(403);
    header('Content-Type: text/plain; charset=utf-8');
    echo "phpMyAdmin: open it from Atollo, Database page.\n";
    exit;
}

$path = rawurldecode((string) parse_url($_SERVER['REQUEST_URI'] ?? '/', PHP_URL_PATH));
$entry = ['/' => 'index.php', '/index.php' => 'index.php', '/url.php' => 'url.php', '/js/messages.php' => 'js/messages.php'];
if (isset($entry[$path])) {
    $root = rtrim((string) getenv('ATOLLO_PMA_ROOT'), '\\/');
    chdir($root);
    require $root . DIRECTORY_SEPARATOR . $entry[$path];
    return true;
}
if (!str_contains($path, '..') && preg_match('~^/(js|themes|doc)/[^\0]+$|^/favicon\.ico$~', $path)
    && !preg_match('~\.(php|phtml|phar)$~i', $path)) {
    // PHP's server sends the file from phpMyAdmin's folder (`-t`).
    return false;
}
http_response_code(404);
exit;
