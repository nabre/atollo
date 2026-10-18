#!/usr/bin/env bash
# Checks a tarball of build.sh as the kit will use it: bash test.sh <tar.gz>
#   - unpacked into a folder with a space and a parenthesis, run with -d/-f;
#   - the modules the kit loads (event MPM, rewrite, proxy_fcgi, ssl, http2);
#   - a real start in the foreground on a loopback port, one request;
#   - moved to another folder, it still starts (no compiled-in path used);
#   - Linux: no glibc symbol above 2.35, no missing or foreign libraries;
#     macOS: macOS 12 minimum, system libraries only, valid signatures.
set -euo pipefail

tarball=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")
base=$(mktemp -d)
root="$base/kit (1)/apache"
mkdir -p "$root"
tar -xzf "$tarball" -C "$root"
os=$(uname -s)

fail() { echo "FAIL: $*" >&2; exit 1; }

# A configuration in the shape the kit writes: stock httpd.conf with
# SRVROOT replaced, Listen on loopback, run-time files outside the tree.
write_conf() {
  local tree=$1 run=$2 port=$3
  mkdir -p "$run"
  sed -e "s|^Define SRVROOT .*|Define SRVROOT \"$tree\"|" -e 's|^Listen 80$|#Listen 80|' \
    -e 's|^#LoadModule rewrite_module|LoadModule rewrite_module|' \
    "$tree/conf/httpd.conf" > "$run/httpd.conf"
  cat >> "$run/httpd.conf" <<EOF
Listen 127.0.0.1:$port
ServerName localhost:$port
PidFile "$run/httpd.pid"
DefaultRuntimeDir "$run"
ErrorLog "$run/error.log"
<IfModule !proxy_module>
LoadModule proxy_module modules/mod_proxy.so
</IfModule>
LoadModule proxy_fcgi_module modules/mod_proxy_fcgi.so
LoadModule ssl_module modules/mod_ssl.so
LoadModule http2_module modules/mod_http2.so
LoadModule socache_shmcb_module modules/mod_socache_shmcb.so
EOF
}

run_once() {
  local tree=$1 run=$2 port=$3
  write_conf "$tree" "$run" "$port"
  "$tree/bin/httpd" -d "$tree" -f "$run/httpd.conf" -t || fail "httpd -t in $tree"
  "$tree/bin/httpd" -d "$tree" -f "$run/httpd.conf" -V | grep -E '^Server (version|MPM)|loaded'
  local mods
  mods=$("$tree/bin/httpd" -d "$tree" -f "$run/httpd.conf" -M)
  for m in mpm_event_module rewrite_module proxy_module proxy_fcgi_module ssl_module http2_module dir_module mime_module alias_module headers_module setenvif_module authz_core_module log_config_module unixd_module; do
    grep -q " $m " <<<"$mods" || fail "module $m not loaded"
  done
  "$tree/bin/httpd" -d "$tree" -f "$run/httpd.conf" -DFOREGROUND &
  local pid=$! body="" i
  for i in $(seq 1 50); do
    body=$(curl -fsS "http://127.0.0.1:$port/" 2>/dev/null) && break
    sleep 0.2
  done
  kill -TERM "$pid" 2>/dev/null || true
  wait "$pid" 2>/dev/null || true
  grep -q 'It works!' <<<"$body" || { cat "$run/error.log" >&2 || true; fail "no answer from $tree"; }
  echo "ok: $tree answered on $port"
}

run_once "$root" "$base/run (1)" 18080

# Moved: the same files from another place.
moved="$base/other place/httpd"
mkdir -p "$(dirname "$moved")"
mv "$root" "$moved"
run_once "$moved" "$base/run (2)" 18081
"$moved/bin/ab" -V >/dev/null || fail "ab does not start"
"$moved/bin/htpasswd" -nbB user pass | grep -q '^user:\$2y\$' || fail "htpasswd does not start"

if [ "$os" = Linux ]; then
  floor=$(find "$moved" -type f \( -name '*.so*' -o -perm -u+x \) -exec sh -c 'file "$1" | grep -q ELF && objdump -T "$1"' _ {} \; \
    | grep -o 'GLIBC_[0-9.]*' | sort -uV | tail -n 1)
  echo "glibc floor: $floor"
  [ "$(printf '%s\n' "$floor" GLIBC_2.35 | sort -V | tail -n 1)" = GLIBC_2.35 ] || fail "$floor is above GLIBC_2.35"
  for f in "$moved"/bin/* "$moved"/modules/*.so "$moved"/lib/*.so*; do
    file -L "$f" | grep -q ELF || continue
    deps=$(ldd "$f")
    grep -q 'not found' <<<"$deps" && fail "$f: $(grep 'not found' <<<"$deps")"
    # Inside the tree, or the C library and its friends.
    # "name => /path with spaces (0x...)": the path between "=> " and " (0x".
    bad=$(sed -n 's|^.* => \(/.*\) (0x[0-9a-f]*)$|\1|p' <<<"$deps" | while IFS= read -r p; do readlink -f "$p"; done \
      | grep -vF "$(readlink -f "$moved")/lib/" \
      | grep -Ev '/(libc|libm|libdl|libpthread|librt|libcrypt|libresolv|libgcc_s)\.so' || true)
    [ -z "$bad" ] || fail "$f links $bad"
  done
else
  for f in "$moved"/bin/* "$moved"/modules/*.so "$moved"/lib/*.dylib; do
    [ -L "$f" ] && continue
    file "$f" | grep -q Mach-O || continue
    codesign --verify --strict "$f" || fail "$f: signature"
    minos=$(otool -l "$f" | awk '/LC_BUILD_VERSION/ { b = 1 } b && $1 == "minos" { print $2; exit }')
    [ "$minos" = "12.0" ] || fail "$f targets macOS $minos"
    bad=$(otool -L "$f" | awk 'NR > 1 { print $1 }' | grep -Ev '^(/usr/lib/|/System/|@rpath/libapr)' || true)
    [ -z "$bad" ] || fail "$f links $bad"
  done
fi
echo "all checks passed: $(basename "$tarball")"
