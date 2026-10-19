#!/usr/bin/env bash
# Builds the kit's PHP for one macOS or Linux target (issue #4):
#   bash build.sh <target> <php-version> <build> <out-dir>
#   target       linux-x64 | linux-arm64 | darwin-arm64 | darwin-x64 (the host's own)
#   php-version  one of the php lines of sources.txt (8.4.26)
#   build        the kit's build number of this PHP version (1, 2, ...)
# Writes <out-dir>/php-<version>-<build>-<target>.tar.gz and its .sha256:
# bin/php, sbin/php-fpm and BUILDINFO.txt, the tree the installer expects.
#
# static-php-cli (spc) builds one static binary per SAPI with the
# extensions of extensions.txt built in. spc and the PHP source are refused
# unless their SHA-256 matches sources.txt. On Linux spc links against musl
# (its doctor installs the toolchain), so the binaries run on any Linux;
# macOS builds target macOS 12.
set -euo pipefail

target=${1:?target}
version=${2:?php version}
build=${3:?build number}
out=${4:?out dir}
here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$out"
out=$(cd "$out" && pwd)

case "$target" in
  linux-x64) want_arch=x86_64 ;;
  linux-arm64) want_arch=aarch64 ;;
  darwin-arm64) want_arch=arm64 ;;
  darwin-x64) want_arch=x86_64 ;;
  *) echo "unknown target $target" >&2; exit 2 ;;
esac
[ "$(uname -m)" = "$want_arch" ] || { echo "$target is built on a $want_arch host, not $(uname -m)" >&2; exit 2; }
case "$build" in ''|*[!0-9]*) echo "build must be a number" >&2; exit 2 ;; esac

sha256() { if command -v sha256sum >/dev/null; then sha256sum "$1" | cut -d' ' -f1; else shasum -a 256 "$1" | cut -d' ' -f1; fi; }

# Field 3 (sha256) or 4 (url) of the line whose name is $1 and version $2.
src() { awk -v n="$1" -v v="$2" -v c="$3" '$1 == n && (v == "" || $2 == v) { print $c; found = 1 } END { if (!found) exit 1 }' "$here/sources.txt"; }

php_sha=$(src php "$version" 3) || { echo "PHP $version is not in sources.txt" >&2; exit 2; }
spc_version=$(src "spc-$target" "" 2)
exts=$(grep -v '^#' "$here/extensions.txt" | tr -d ' \n')
# Swoole 6.2.3 does not build on macOS (src/core/misc.cc: undeclared
# sw_usleep): the macOS builds leave it and its MySQL hook out, until a
# release of Swoole fixes it.
case "$target" in
  darwin-*) exts=$(printf '%s' "$exts" | tr ',' '\n' | grep -v '^swoole' | paste -sd, -) ;;
esac

work=${WORK:-$(mktemp -d)}
mkdir -p "$work"
cd "$work"

# spc, checked.
spc_tar="$work/spc.tar.gz"
curl -fsSL --retry 3 -o "$spc_tar" "$(src "spc-$target" "" 4)"
got=$(sha256 "$spc_tar")
want=$(src "spc-$target" "" 3)
[ "$got" = "$want" ] || { echo "spc: SHA-256 $got, want $want" >&2; exit 1; }
tar -xzf "$spc_tar"
chmod +x spc
./spc --version

if [ "$(uname -s)" = Darwin ]; then
  export MACOSX_DEPLOYMENT_TARGET=12.0
fi

./spc doctor --auto-fix
./spc download --with-php="$version" --for-extensions="$exts" --retry=3

# The PHP source spc took must be php.net's own.
php_src=$(find downloads -maxdepth 1 -type f -name "php-$version.tar.*" | head -n 1)
[ -n "$php_src" ] || { echo "spc did not download php-$version" >&2; ls downloads >&2; exit 1; }
got=$(sha256 "$php_src")
if [ "$got" != "$php_sha" ]; then
  # spc may fetch the .tar.gz: then check it against php.net's own list.
  echo "$(basename "$php_src"): SHA-256 $got, not the .tar.xz of sources.txt; checking php.net" >&2
  curl -fsSL "https://www.php.net/releases/?json&version=$version" | grep -q "\"$got\"" \
    || { echo "php-$version: SHA-256 $got is not one php.net publishes" >&2; exit 1; }
fi

./spc build "$exts" --build-cli --build-fpm

name="php-$version-$build-$target"
tree="$work/$name"
rm -rf "$tree"
mkdir -p "$tree/bin" "$tree/sbin"
cp buildroot/bin/php "$tree/bin/php"
cp buildroot/bin/php-fpm "$tree/sbin/php-fpm"
chmod 755 "$tree/bin/php" "$tree/sbin/php-fpm"
{
  echo "PHP $version, build $build of the kit, for $target"
  echo "static-php-cli $spc_version (spc), php source SHA-256 $got"
  echo "extensions: $exts"
  echo
  "$tree/bin/php" -v
} > "$tree/BUILDINFO.txt"

(cd "$tree" && tar -czf "$out/$name.tar.gz" bin sbin BUILDINFO.txt)
(cd "$out" && sha256 "$name.tar.gz" | awk -v f="$name.tar.gz" '{ print $1 "  " f }' > "$name.tar.gz.sha256")
echo "== $out/$name.tar.gz"
cat "$out/$name.tar.gz.sha256"
