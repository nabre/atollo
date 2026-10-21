#!/usr/bin/env bash
# Builds mongo-express for the kit (DECISIONS.md D-147):
#   bash build.sh <version> <build> <out-dir>
#   version  the kit-version of a line of sources.txt (1.1.0-rc.4)
#   build    the kit's build number of that version (1, 2, ...)
# Writes <out-dir>/mongo-express-<version>-<build>.zip and its .sha256: the
# runtime tree at the root of the archive (app.js, lib/, build/, public/,
# node_modules/ with the production dependencies only, BUILDINFO.txt). Pure
# JavaScript: one archive for every system, run by the kit's Node.js.
#
# Why a build: the npm package has no build/ and no build-assets.json (the
# webpack output), so `node app.js` ends at once (ENOENT build-assets.json).
# The build follows upstream's Dockerfile: the source at the pinned commit,
# `yarn install --immutable` from upstream's yarn.lock with upstream's own
# Yarn (.yarn/releases, named by .yarnrc.yml), `yarn build` (webpack), then
# the runtime files in a new folder and `yarn workspaces focus --production`
# there: the same lock file, only the dependencies the server needs.
# No package runs an install script (YARN_ENABLE_SCRIPTS=0): webpack needs
# none, and the ones of the development tools (Cypress, the MongoDB of
# mongodb-memory-server) would download large binaries for nothing.
set -euo pipefail

version=${1:?version}
build=${2:?build number}
out=${3:?out dir}
here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$out"
out=$(cd "$out" && pwd)
case "$build" in ''|*[!0-9]*) echo "build must be a number" >&2; exit 2 ;; esac

sha256() { if command -v sha256sum >/dev/null; then sha256sum "$1" | cut -d' ' -f1; else shasum -a 256 "$1" | cut -d' ' -f1; fi; }

line=$(awk -v v="$version" '$1 == "mongo-express" && $2 == v' "$here/sources.txt")
[ -n "$line" ] || { echo "mongo-express $version is not in sources.txt" >&2; exit 2; }
read -r _ _ upstream tag commit repo <<< "$line"

work=${WORK:-$(mktemp -d)}
mkdir -p "$work"
src="$work/src"
rm -rf "$src"

# The source at the pinned commit: the tag is fetched, then its commit
# must be the one in sources.txt.
git init -q "$src"
git -C "$src" fetch -q --depth 1 "$repo" "refs/tags/$tag"
git -C "$src" -c advice.detachedHead=false checkout -q FETCH_HEAD
got=$(git -C "$src" rev-parse HEAD)
[ "$got" = "$commit" ] || { echo "$tag is at $got, want $commit" >&2; exit 1; }
pkg_version=$(node -p 'require(process.argv[1]).version' "$src/package.json")
[ "$pkg_version" = "$upstream" ] || { echo "package.json says $pkg_version, want $upstream" >&2; exit 1; }

# Upstream's own Yarn, from the commit itself.
yarn_rel=$(awk -F': *' '$1 == "yarnPath" { print $2 }' "$src/.yarnrc.yml")
[ -n "$yarn_rel" ] && [ -f "$src/$yarn_rel" ] || { echo "no yarnPath in .yarnrc.yml" >&2; exit 1; }
yarn_js="$src/$yarn_rel"
export YARN_ENABLE_SCRIPTS=0
export YARN_ENABLE_TELEMETRY=0
export YARN_ENABLE_GLOBAL_CACHE=0
export YARN_CACHE_FOLDER="$work/yarn-cache"
export CYPRESS_INSTALL_BINARY=0
export MONGOMS_DISABLE_POSTINSTALL=1
export HUSKY=0
yarn_version=$(cd "$src" && node "$yarn_js" --version)

(cd "$src" && node "$yarn_js" install --immutable)
(cd "$src" && node "$yarn_js" build)
[ -f "$src/build-assets.json" ] || { echo "the build wrote no build-assets.json" >&2; exit 1; }

# The runtime tree: what upstream's Dockerfile copies, without the client
# sources of lib/scripts (webpack's input).
tree="$work/tree"
rm -rf "$tree"
mkdir -p "$tree/lib" "$tree/.yarn"
for f in app.js config.default.js package.json build-assets.json yarn.lock .yarnrc.yml LICENSE.md README.md; do
  cp "$src/$f" "$tree/"
done
cp -R "$src/build" "$src/public" "$tree/"
cp "$src"/lib/*.js "$tree/lib/"
cp -R "$src/lib/routes" "$src/lib/views" "$tree/lib/"
cp -R "$src/.yarn/releases" "$src/.yarn/plugins" "$tree/.yarn/"
(cd "$tree" && node "$yarn_js" workspaces focus --production)
# Yarn and its state go; the lock file stays, as the record of what is in
# node_modules. No links (node_modules/.bin): a zip cannot carry them to
# Windows.
rm -rf "$tree/.yarn" "$tree/.yarnrc.yml" "$tree/node_modules/.yarn-state.yml"
find "$tree/node_modules" -type d -name .bin -prune -exec rm -rf {} +
links=$(find "$tree" -type l)
[ -z "$links" ] || { echo "symbolic links left: $links" >&2; exit 1; }
# Pure JavaScript, or it is not one archive for every system.
native=$(find "$tree" -name '*.node' -o -name binding.gyp)
[ -z "$native" ] || { echo "native modules: $native" >&2; exit 1; }
for dep in express mongodb csurf; do
  [ -f "$tree/node_modules/$dep/package.json" ] || { echo "$dep missing from node_modules" >&2; exit 1; }
done
for dev in webpack cypress mongodb-memory-server eslint; do
  [ ! -e "$tree/node_modules/$dev" ] || { echo "development package $dev in node_modules" >&2; exit 1; }
done

name="mongo-express-$version-$build"
{
  echo "mongo-express $upstream for Atollo, build $build"
  echo "source: $repo tag $tag commit $commit"
  echo "built: $(date -u +%Y-%m-%dT%H:%M:%SZ) with Node.js $(node --version) and Yarn $yarn_version"
  echo "steps: yarn install --immutable, yarn build (webpack), yarn workspaces focus --production; no install scripts"
} > "$tree/BUILDINFO.txt"

rm -f "$out/$name.zip"
# Sorted names and no extra attributes (-X): the same tree gives the same
# list of entries.
(cd "$tree" && find . -type f | LC_ALL=C sort | sed 's|^\./||' | zip -q -X -@ "$out/$name.zip")
(cd "$out" && sha256 "$name.zip" | awk -v f="$name.zip" '{ print $1 "  " f }' > "$name.zip.sha256")
echo "== $out/$name.zip"
cat "$out/$name.zip.sha256"
