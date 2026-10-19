#!/usr/bin/env bash
# Checks a tarball of the kit's PHP (issue #4): bash test.sh <tarball>
# Unpacked in a folder of its own, from another working directory: the
# extensions the kit relies on are there (pdo_sqlite and pdo_pgsql above
# all, the reason of this build), SQLite works through PDO, php-fpm starts.
set -euo pipefail

tarball=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")
dir=$(mktemp -d)
tar -xzf "$tarball" -C "$dir"
php="$dir/bin/php"
fpm="$dir/sbin/php-fpm"
[ -x "$php" ] && [ -x "$fpm" ] || { echo "bin/php or sbin/php-fpm missing" >&2; exit 1; }
cd /

"$php" -v
mods=$("$php" -n -m | tr 'A-Z' 'a-z')
missing=""
want="pdo_sqlite pdo_pgsql pdo_mysql sqlite3 mysqli intl sodium mbstring openssl curl zip gd opcache"
# Swoole on Linux only (build.sh: it does not build on macOS today).
[ "$(uname -s)" = Linux ] && want="$want swoole"
for m in $want; do
  name=$m
  [ "$m" = opcache ] && name="zend opcache"
  printf '%s\n' "$mods" | grep -qx "$name" || missing="$missing $m"
done
[ -z "$missing" ] || { echo "missing extensions:$missing" >&2; exit 1; }

# What composer.json's ext-pdo_sqlite and Laravel on SQLite need.
"$php" -n -r '
  if (!extension_loaded("pdo_sqlite")) { fwrite(STDERR, "pdo_sqlite not loaded\n"); exit(1); }
  $db = new PDO("sqlite::memory:");
  $db->exec("create table t (x integer)");
  $db->exec("insert into t values (41)");
  $v = (int) $db->query("select x + 1 from t")->fetchColumn();
  if ($v !== 42) { fwrite(STDERR, "sqlite gave $v\n"); exit(1); }
  if (!in_array("pgsql", PDO::getAvailableDrivers(), true)) { fwrite(STDERR, "no pdo pgsql driver\n"); exit(1); }
  echo "PDO SQLite ok, drivers: ", implode(", ", PDO::getAvailableDrivers()), "\n";
'

"$fpm" -v
conf="$dir/fpm.conf"
printf '[global]\nerror_log = /dev/stderr\n[www]\nlisten = 127.0.0.1:9123\npm = static\npm.max_children = 1\n' > "$conf"
"$fpm" -n -t -y "$conf"
echo "ok: $(basename "$tarball")"
