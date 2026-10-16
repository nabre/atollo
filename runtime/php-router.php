<?php
// Router of PHP's built-in server (`php -S`) for the projects' action
// «php -S» (detect\php.rs). Written by the app: a hand edit is replaced at
// the next start. A project with a router of its own (`router.php`) keeps
// it, and this one is not used.
//
// - 403 for a hidden file or folder (`.env`, `.git/config`, `.htaccess`),
//   except `.well-known`, and for `.inc` files: the built-in server would
//   send them as text;
// - anything else goes on to the server as without a router (`return false`):
//   the file, the PHP script, or the `index.php` of the folder.

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
        || strtolower(substr($name, -4)) === '.inc') {
        http_response_code(403);
        exit;
    }
}
return false;
