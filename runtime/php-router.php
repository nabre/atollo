<?php
// Router of PHP's built-in server (`php -S`) for the projects' action
// «php -S» (detect\php.rs). Written by the app: a hand edit is replaced at
// the next start. A project with a router of its own (`router.php`) keeps
// it, and this one is not used.
//
// - 421 for a `Host` that is not local (`localhost`, `*.localhost`,
//   `127.0.0.1`, `[::1]`): a page of another site whose name was re-pointed
//   to 127.0.0.1 (DNS rebinding) must not read the project (SEC-V3-03);
// - 403 for a hidden file or folder (`.env`, `.git/config`, `.htaccess`),
//   except `.well-known`, and for the files Apache and Nginx never serve or
//   run as PHP (`.inc`, `.php3`..`.php8`, `.phtml`, `.phar`): the built-in
//   server runs only `.php` and would send them as text (SEC-V3-01);
// - anything else goes on to the server as without a router (`return false`):
//   the file, the PHP script, or the `index.php` of the folder.

$host = strtolower((string) ($_SERVER['HTTP_HOST'] ?? ''));
if ($host !== '') {
    // Without the port: `[::1]:8000` keeps its brackets, `name:8000` not.
    $host = preg_replace('~^(\[[^\]]*\]|[^:]*)(:\d*)?$~', '$1', $host);
    $host = rtrim($host, '.');
    if ($host !== 'localhost' && $host !== '127.0.0.1' && $host !== '[::1]'
        && substr($host, -10) !== '.localhost') {
        http_response_code(421);
        exit;
    }
}

$path = rawurldecode((string) parse_url($_SERVER['REQUEST_URI'] ?? '/', PHP_URL_PATH));
// On Windows `\` separates folders too, and a name with dots or spaces at
// the end, or with an alternate data stream (`::$DATA`), is the same file.
foreach (preg_split('~[/\\\\]+~', $path) as $part) {
    $name = rtrim($part, '. ');
    if ($part === '' || $part === '.' || $part === '..') {
        continue;
    }
    if (strpos($part, ':') !== false
        || ($part[0] === '.' && strtolower($name) !== '.well-known')
        || preg_match('~\.(?:inc|php[3-8]|phtml|phar)$~i', $name)
        // `index.php.` is `index.php` on Windows, but the built-in server
        // would send it as a static file, with its source.
        || ($name !== $part && preg_match('~\.php$~i', $name))) {
        http_response_code(403);
        exit;
    }
}
return false;
