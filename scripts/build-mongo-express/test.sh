#!/usr/bin/env bash
# Checks an archive of the kit's mongo-express (DECISIONS.md D-147):
#   bash test.sh <zip>
# Against a running MongoDB with authentication, whose user `admin` of the
# `admin` database has the password in MONGO_PASSWORD, at MONGO_PORT
# (27017). Unpacked in a folder with a space and parentheses, started from
# an empty working folder by the kit's own starter (runtime/mongo-express.js,
# the one the app writes), on TEST_HOST (127.0.0.2: a loopback address other
# than 127.0.0.1, as the app picks; 127.0.0.1 on macOS).
#   - 403 without the token, with a wrong one, for another Host;
#   - the token gives an HttpOnly, SameSite=Strict cookie and goes back to /;
#   - with the cookie: the databases listed (admin), the webpack assets
#     served, a POST without mongo-express's CSRF token refused.
set -euo pipefail

: "${MONGO_PASSWORD:?the password of the admin user of MongoDB}"
port_db=${MONGO_PORT:-27017}
host=${TEST_HOST:-127.0.0.2}
port=${TEST_PORT:-18081}
here=$(cd "$(dirname "$0")" && pwd)
launcher="$here/../../runtime/mongo-express.js"
zip=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")

base=$(mktemp -d)
root="$base/mongo express (test)"
mkdir -p "$root" "$base/cwd"
unzip -q "$zip" -d "$root"
[ -f "$root/app.js" ] && [ -f "$root/build-assets.json" ] && [ -d "$root/node_modules/mongodb" ] \
  || { echo "app.js, build-assets.json or node_modules/mongodb missing" >&2; exit 1; }
cat "$root/BUILDINFO.txt"

token=$(node -e 'process.stdout.write(require("crypto").randomBytes(32).toString("hex"))')
log="$base/mongo-express.log"
(
  cd "$base/cwd"
  ATOLLO_ME_ROOT="$root" ATOLLO_ME_HOST="$host" ATOLLO_ME_PORT="$port" ATOLLO_ME_TOKEN="$token" \
    ATOLLO_DB_PORT="$port_db" ATOLLO_DB_PASSWORD="$MONGO_PASSWORD" \
    exec node "$launcher"
) > "$log" 2>&1 &
pid=$!
trap 'kill "$pid" 2>/dev/null || true' EXIT

url="http://$host:$port"
for _ in $(seq 1 60); do
  grep -q "listening on $host:$port" "$log" && break
  kill -0 "$pid" 2>/dev/null || { cat "$log" >&2; echo "mongo-express ended" >&2; exit 1; }
  sleep 0.5
done
grep -q "listening on $host:$port" "$log" || { cat "$log" >&2; echo "no answer on $url" >&2; exit 1; }

code() { curl -s -o /dev/null -w '%{http_code}' "$@"; }
expect() {
  local want=$1 what=$2; shift 2
  local got
  got=$(code "$@")
  [ "$got" = "$want" ] || { cat "$log" >&2; echo "$what: HTTP $got, want $want" >&2; exit 1; }
  echo "ok $what ($got)"
}

expect 403 "no token" "$url/"
expect 403 "wrong token" "$url/?atollo=0"
expect 403 "no token, no files" "$url/public/css/style.css"
expect 403 "Host localhost" -H "Host: localhost:$port" "$url/?atollo=$token"
expect 403 "Host 127.0.0.1" -H "Host: 127.0.0.1:$port" "$url/?atollo=$token"

headers=$(curl -s -D - -o /dev/null "$url/?atollo=$token")
printf '%s\n' "$headers" | grep -q '^HTTP/1.1 302' || { echo "$headers" >&2; echo "the token does not redirect" >&2; exit 1; }
set_cookie=$(printf '%s\n' "$headers" | tr -d '\r' | grep -i '^set-cookie: atollo_me=')
case "$set_cookie" in
  *HttpOnly*SameSite=Strict*) echo "ok cookie" ;;
  *) echo "cookie: $set_cookie" >&2; exit 1 ;;
esac
location=$(printf '%s\n' "$headers" | tr -d '\r' | awk -F': ' 'tolower($1) == "location" { print $2 }')
[ "$location" = "/" ] || { echo "Location: $location" >&2; exit 1; }
cookie="atollo_me=$token"

jar="$base/jar.txt"
page=$(curl -s -c "$jar" -b "$cookie" "$url/")
printf '%s' "$page" | grep -q 'mongo-express' || { printf '%s\n' "$page" | head -40 >&2; echo "not mongo-express's page" >&2; exit 1; }
printf '%s' "$page" | grep -q '/db/admin/' || { printf '%s\n' "$page" | head -80 >&2; cat "$log" >&2; echo "database admin not listed" >&2; exit 1; }
echo "ok databases listed"
expect 200 "database page" -b "$cookie" "$url/db/admin/"
asset=$(node -e 'const a = require(process.argv[1]); process.stdout.write(a.vendor.js)' "$root/build-assets.json")
expect 200 "webpack asset $asset" -b "$cookie" "$url/$asset"
# mongo-express's CSRF protection (csurf): a form without its token.
expect 403 "POST without CSRF token" -b "$cookie" -X POST -d 'database=atollo_csrf_probe' "$url/"
echo "== mongo-express works"
