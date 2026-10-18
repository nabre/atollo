#!/usr/bin/env bash
# Builds the kit's relocatable Apache httpd for one macOS or Linux target
# (DECISIONS.md D-116): bash build.sh <target> <build> <out-dir>
#   target  linux-x64 | linux-arm64 | darwin-arm64 | darwin-x64 (the host's own)
#   build   the kit's build number of this httpd version (1, 2, ...)
# Writes <out-dir>/httpd-<version>-<build>-<target>.tar.gz and its .sha256.
#
# Every source comes from sources.txt and is refused unless its SHA-256
# matches. OpenSSL, PCRE2, nghttp2, Expat and zlib are linked statically;
# APR and APR-util are httpd's own shared libraries in lib/, found through
# $ORIGIN (Linux) or @loader_path (macOS), so the tree runs from any folder
# with `httpd -d <tree> -f <conf>`. Linux builds belong in an ubuntu:22.04
# container (glibc 2.35, the kit's floor); macOS builds target macOS 12.
set -euo pipefail

target=${1:?target}
build=${2:?build number}
out=${3:?out dir}
here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$out"
out=$(cd "$out" && pwd)

case "$target" in
  linux-x64) os=linux; ossl=linux-x86_64; want_arch=x86_64 ;;
  linux-arm64) os=linux; ossl=linux-aarch64; want_arch=aarch64 ;;
  darwin-arm64) os=darwin; ossl=darwin64-arm64-cc; want_arch=arm64 ;;
  darwin-x64) os=darwin; ossl=darwin64-x86_64-cc; want_arch=x86_64 ;;
  *) echo "unknown target $target" >&2; exit 2 ;;
esac
[ "$(uname -m)" = "$want_arch" ] || { echo "$target is built on a $want_arch host, not $(uname -m)" >&2; exit 2; }
case "$build" in ''|*[!0-9]*) echo "build must be a number" >&2; exit 2 ;; esac

# Compiled-in paths only: the kit always passes -d and -f.
prefix=/opt/atollo-httpd
work=${WORK:-$(mktemp -d)}
deps="$work/deps"
mkdir -p "$work/dl" "$work/src" "$deps"
jobs=$( (nproc || sysctl -n hw.ncpu) 2>/dev/null || echo 2)

sha256() { if command -v sha256sum >/dev/null; then sha256sum "$1" | cut -d' ' -f1; else shasum -a 256 "$1" | cut -d' ' -f1; fi; }

# Field 2 (version), 3 (sha256) or 4 (url) of a source; no associative
# arrays, for the bash 3.2 of macOS.
src() { awk -v n="$1" -v c="$2" '$1 == n { print $c; found = 1 } END { if (!found) exit 1 }' "$here/sources.txt"; }
ver() { src "$1" 2; }

# Download, check, unpack into $work/src/<name>.
fetch() {
  local name=$1 file link want got
  link=$(src "$name" 4)
  want=$(src "$name" 3)
  file="$work/dl/$(basename "$link")"
  [ -f "$file" ] || curl -fsSL --retry 3 -o "$file" "$link"
  got=$(sha256 "$file")
  [ "$got" = "$want" ] || { echo "$name: SHA-256 $got, want $want" >&2; exit 1; }
  rm -rf "$work/src/$name"
  mkdir -p "$work/src/$name"
  tar -xf "$file" -C "$work/src/$name" --strip-components 1
}

if [ "$os" = darwin ]; then
  export MACOSX_DEPLOYMENT_TARGET=12.0
  # Room for the @rpath rewrite below.
  export LDFLAGS="-headerpad_max_install_names"
else
  export LDFLAGS=""
fi
export CFLAGS="-O2 -fPIC"
export CPPFLAGS="-I$deps/include"
export LDFLAGS="$LDFLAGS -L$deps/lib"
export PKG_CONFIG_PATH="$deps/lib/pkgconfig"
# Only our own .pc files: nothing from the build machine (Homebrew on macOS).
export PKG_CONFIG_LIBDIR="$deps/lib/pkgconfig"

for name in httpd apr apr-util openssl pcre2 nghttp2 expat zlib; do fetch "$name"; done

echo "== zlib $(ver zlib)"
(cd "$work/src/zlib" && ./configure --static --prefix="$deps" && make -j"$jobs" && make install)

echo "== expat $(ver expat)"
(cd "$work/src/expat" && ./configure --prefix="$deps" --disable-shared --enable-static --with-pic \
  --without-docbook --without-examples --without-tests --without-xmlwf && make -j"$jobs" && make install)

echo "== pcre2 $(ver pcre2)"
# No JIT: it needs writable and executable memory, refused under some
# macOS signing modes, and httpd's regexes are small.
(cd "$work/src/pcre2" && ./configure --prefix="$deps" --disable-shared --enable-static --with-pic \
  --disable-jit && make -j"$jobs" && make install)

echo "== nghttp2 $(ver nghttp2)"
(cd "$work/src/nghttp2" && ./configure --prefix="$deps" --enable-lib-only --disable-shared --enable-static \
  --with-pic && make -j"$jobs" && make install)

echo "== openssl $(ver openssl)"
(cd "$work/src/openssl" && ./Configure "$ossl" no-shared no-tests no-docs no-module -fPIC \
  --prefix="$deps" --libdir=lib --openssldir=/etc/ssl && make -j"$jobs" && make install_sw)

echo "== httpd $(ver httpd) (apr $(ver apr), apr-util $(ver apr-util))"
h="$work/src/httpd"
rm -rf "$h/srclib/apr" "$h/srclib/apr-util"
cp -R "$work/src/apr" "$h/srclib/apr"
cp -R "$work/src/apr-util" "$h/srclib/apr-util"
# Modules that need libraries the kit does not carry are left out, as are
# suexec (setuid) and systemd; the rest of "reallyall" is built shared.
# APR-util gets no database or LDAP drivers: they would link libraries of
# the build machine.
(cd "$h" && ./configure --prefix="$prefix" \
  --with-included-apr \
  --with-expat="$deps" \
  --without-berkeley-db --without-gdbm --without-ndbm --without-sqlite2 --without-sqlite3 \
  --without-pgsql --without-mysql --without-odbc --without-oracle --without-freetds \
  --without-ldap --without-crypto \
  --with-pcre="$deps/bin/pcre2-config" \
  --enable-so \
  --enable-mpms-shared=all --with-mpm=event \
  --enable-mods-shared=reallyall \
  --enable-ssl --with-ssl="$deps" \
  --enable-http2 --with-nghttp2="$deps" \
  --enable-deflate --with-z="$deps" \
  --disable-lua --disable-xml2enc --disable-proxy-html --disable-brotli --disable-md \
  --disable-ldap --disable-authnz-ldap --disable-socache-dc --disable-session-crypto \
  --disable-suexec --disable-systemd --disable-privileges \
  && make -j"$jobs" && make install DESTDIR="$work/inst")

tree="$work/inst$prefix"

# Only what the kit runs: no headers, manual, build files, sample CGI
# scripts, or tools with compiled-in paths.
rm -rf "$tree/build" "$tree/include" "$tree/man" "$tree/manual" "$tree/conf/original"
rm -f "$tree/cgi-bin/"* "$tree/bin/apxs" "$tree/bin/apachectl" "$tree/bin/envvars" "$tree/bin/envvars-std" "$tree/bin/dbmmanage"
rm -f "$tree/lib/"*.la "$tree/lib/"*.a "$tree/lib/"*.exp
rm -rf "$tree/lib/pkgconfig"
find "$tree/modules" -name '*.la' -delete 2>/dev/null || true

# Stock httpd.conf in the shape of Apache Lounge's (Define SRVROOT, then
# ServerRoot "${SRVROOT}"), the one the installer rewrites on every system.
conf="$tree/conf/httpd.conf"
perl -0pi -e 's{^ServerRoot "\Q'"$prefix"'\E"}{Define SRVROOT "'"$prefix"'"\nServerRoot "\${SRVROOT}"}m or die "no ServerRoot\n"' "$conf"
perl -pi -e 's{"\Q'"$prefix"'\E}{"\${SRVROOT}}g unless /^Define SRVROOT/' "$conf"
# Not root: httpd never changes user, so the lines only warn.
perl -pi -e 's{^(User|Group) }{#$1 }' "$conf"
for f in "$tree/conf/extra/"*.conf; do perl -pi -e 's{"\Q'"$prefix"'\E}{"\${SRVROOT}}g; s{\Q'"$prefix"'\E}{\${SRVROOT}}g' "$f"; done

# Relocation.
if [ "$os" = linux ]; then
  for f in "$tree"/bin/* "$tree"/modules/*.so; do
    file "$f" | grep -q ELF || continue
    patchelf --set-rpath '$ORIGIN/../lib' "$f"
  done
  for f in "$tree"/lib/*.so*; do
    [ -L "$f" ] && continue
    patchelf --set-rpath '$ORIGIN' "$f"
  done
else
  macho() { file "$1" | grep -q 'Mach-O'; }
  fix() {
    local f=$1 rpath=$2 dep
    for dep in $(otool -L "$f" | awk 'NR > 1 { print $1 }' | grep "^$prefix/lib/" || true); do
      install_name_tool -change "$dep" "@rpath/$(basename "$dep")" "$f"
    done
    install_name_tool -add_rpath "$rpath" "$f" 2>/dev/null || true
  }
  for f in "$tree"/lib/*.dylib; do
    [ -L "$f" ] && continue
    install_name_tool -id "@rpath/$(basename "$f")" "$f"
    fix "$f" @loader_path
  done
  for f in "$tree"/bin/* "$tree"/modules/*.so; do macho "$f" && fix "$f" @loader_path/../lib; done
  # Ad hoc signatures again after the rewrite (arm64 refuses unsigned code).
  for f in "$tree"/lib/*.dylib "$tree"/bin/* "$tree"/modules/*.so; do
    [ -L "$f" ] && continue
    macho "$f" && codesign --force --sign - "$f"
  done
fi

# Licenses of everything inside.
mkdir -p "$tree/licenses"
cp "$h/LICENSE" "$tree/LICENSE"
cp "$h/NOTICE" "$tree/NOTICE"
cp "$work/src/apr/LICENSE" "$tree/licenses/apr-LICENSE"
cp "$work/src/apr-util/LICENSE" "$tree/licenses/apr-util-LICENSE"
cp "$work/src/openssl/LICENSE.txt" "$tree/licenses/openssl-LICENSE.txt"
cp "$work/src/pcre2/LICENCE.md" "$tree/licenses/pcre2-LICENCE.md"
cp "$work/src/nghttp2/COPYING" "$tree/licenses/nghttp2-COPYING"
cp "$work/src/expat/COPYING" "$tree/licenses/expat-COPYING"
cp "$work/src/zlib/LICENSE" "$tree/licenses/zlib-LICENSE"
{
  echo "Apache httpd $(ver httpd) for $target, kit build $build"
  echo "Built by scripts/build-httpd/build.sh of github.com/nabre/dev-kit2"
  echo
  while read -r name ver sum link; do
    case "$name" in ''|'#'*) continue ;; esac
    echo "$name $ver sha256:$sum"
  done < "$here/sources.txt"
} > "$tree/BUILDINFO.txt"

name="httpd-$(ver httpd)-$build-$target.tar.gz"
if tar --version 2>/dev/null | grep -q GNU; then
  tar --owner=0 --group=0 --numeric-owner -czf "$out/$name" -C "$tree" .
else
  tar --uid 0 --gid 0 -czf "$out/$name" -C "$tree" .
fi
(cd "$out" && echo "$(sha256 "$name")  $name" > "$name.sha256" && cat "$name.sha256")
