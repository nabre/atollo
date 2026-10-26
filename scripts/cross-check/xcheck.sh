#!/usr/bin/env bash
# Cross "cargo check" / "cargo clippy" of the crate for a Unix target from
# Windows (docs/app/TESTING.md): fake cc, ar and pkg-config (node
# scripts) and an empty OpenSSL let the -sys build scripts pass. Nothing is
# linked and no test runs: it only proves that the cfg(unix) code compiles
# and passes clippy. Tests on Unix: scripts/test-linux.sh (WSL2) and the CI.
#
# Usage, from Git Bash in the repository or a worktree:
#   bash scripts/cross-check/xcheck.sh <target> <cargo subcommand> [args...]
#   bash scripts/cross-check/xcheck.sh x86_64-unknown-linux-gnu clippy --all-targets -- -D warnings
# Targets: x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu,
#          aarch64-apple-darwin (the systems of Docker Sandboxes, D-176)
#          (rustup target add <target> once).
# The build goes to target/xcheck-<target> so it never mixes with the
# Windows build.

set -euo pipefail
if [ $# -lt 2 ]; then
    sed -n '2,/^$/p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
    exit 2
fi
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -W)"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
t="$1"
shift
u="${t//-/_}"
export OPENSSL_DIR="$here" PKG_CONFIG_ALLOW_CROSS=1 PKG_CONFIG="$here/fakepc.cmd"
export "CC_$u=$here/fakecc.cmd" "AR_$u=$here/fakear.cmd" "CXX_$u=$here/fakecc.cmd"
export CARGO_TARGET_DIR="$root/target/xcheck-$t"
cd "$root"
sub="$1"
shift
exec cargo "$sub" --target "$t" "$@"
