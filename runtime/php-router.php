<?php
// Router of PHP's built-in server (`php -S`) for the projects' action
// «php -S» (detect\php.rs). Written by the app: a hand edit is replaced at
// the next start.
//
// - 421 for a `Host` that is not local (`localhost`, `*.localhost`,
//   `127.0.0.1`, `[::1]`): a page of another site whose name was re-pointed
//   to 127.0.0.1 (DNS rebinding) must not read the project (SEC-V3-03);
// - 403 for a hidden file or folder (`.env`, `.git/config`, `.htaccess`),
//   except `.well-known`, and for the files Apache and Caddy never serve or
//   run as PHP (`.inc`, `.php3`..`.php8`, `.phtml`, `.phar`): the built-in
//   server runs only `.php` and would send them as text (SEC-V3-01);
// - anything else goes to the project's own router when it has one
//   (`router.php`, given as `-d devkit.router=<path>`: `php -S` takes one
//   router only), with its return value; else on to the server as without a
//   router (`return false`): the file, the PHP script, or the `index.php` of
//   the folder.
//
// The checks run in a function, so that no variable of theirs reaches the
// project's router, which is run in the global scope as `php -S` would.

(static function () {
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

    // The path as the built-in server reads it: everything before `?`, decoded
    // once. Not `parse_url`: it reads `//.env` as a host named `.env` and gives
    // up (false) on some paths, and either would let the request through.
    $uri = (string) ($_SERVER['REQUEST_URI'] ?? '/');
    $cut = strcspn($uri, '?#');
    $path = rawurldecode(substr($uri, 0, $cut));
    if ($path === '' || $path[0] !== '/' || strpos($path, "\0") !== false) {
        http_response_code(403);
        exit;
    }
    // On Windows `\` separates folders too, a name with dots or spaces at the
    // end or with an alternate data stream (`::$DATA`) is the same file, and a
    // short 8.3 name (`ENV~1` for `.env`) reaches a file under another name.
    foreach (preg_split('~[/\\\\]+~', $path) as $part) {
        $name = rtrim($part, '. ');
        if ($part === '' || $part === '.' || $part === '..') {
            continue;
        }
        if (strpos($part, ':') !== false
            || preg_match('/~\d/', $part)
            || ($part[0] === '.' && strtolower($name) !== '.well-known')
            || preg_match('~\.(?:inc|php[3-8]|phtml|phar)$~i', $name)
            // The built-in server runs only `.php` in lower case: `index.PHP`
            // and `index.php.` are `index.php` on Windows, but would be sent as
            // static files, with their source.
            || (preg_match('~\.php$~i', $name) && $name !== $part)
            || (preg_match('~\.php$~i', $name) && substr($name, -4) !== '.php')) {
            http_response_code(403);
            exit;
        }
    }
})();

if ((string) get_cfg_var('devkit.router') !== '') {
    // As if `php -S` had been given the project's router (its path relative
    // to the document root, never to the working folder, which a script may
    // change): in SCRIPT_FILENAME where the server put this router's.
    if (realpath((string) ($_SERVER['SCRIPT_FILENAME'] ?? '')) === __FILE__) {
        $_SERVER['SCRIPT_FILENAME'] = $_SERVER['DOCUMENT_ROOT'] . DIRECTORY_SEPARATOR
            . get_cfg_var('devkit.router');
    }
    return require $_SERVER['DOCUMENT_ROOT'] . DIRECTORY_SEPARATOR . get_cfg_var('devkit.router');
}
return false;
