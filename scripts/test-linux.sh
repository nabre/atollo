#!/usr/bin/env bash
# Runs the Rust tests of the app on Linux, from a Windows checkout through
# WSL2 (PLAN-PIATTAFORME.md §2.1, WP-0.6) or from a Linux checkout.
#
#   From Windows, in the repository root:  wsl -d Ubuntu -- bash scripts/test-linux.sh
#   Inside WSL or on Linux:                bash scripts/test-linux.sh
#
# The repository is copied with rsync into the Linux file system
# ($ATOLLO_LINUX_DIR, default ~/atollo): on /mnt/<drive> permissions and
# symbolic links are not Linux's and the build is slow. target/ stays there
# between runs, so the next build is incremental.
#
# Options:
#   --clippy   also cargo clippy --all-targets -- -D warnings, before the tests
#   --         what follows goes to cargo test (a filter, -- --nocapture, ...)
#
# Exit codes: 0 tests green, 1 tests failed or did not build, 2 something
# to install first (the command to run is printed).

set -euo pipefail

src="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
dest="${ATOLLO_LINUX_DIR:-$HOME/atollo}"
clippy=0
cargo_args=()
while [ $# -gt 0 ]; do
    case "$1" in
        --clippy) clippy=1 ;;
        --) shift; cargo_args=("$@"); break ;;
        -h | --help) sed -n '2,19p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "Unknown option: $1 (see --help)" >&2; exit 2 ;;
    esac
    shift
done

if [ "$(uname -s)" != "Linux" ]; then
    echo "This script runs on Linux (WSL2 from Windows: wsl -d Ubuntu -- bash scripts/test-linux.sh)." >&2
    exit 2
fi

# A non-login shell (wsl -- bash ...) does not read ~/.profile: rustup's
# cargo is found through its env file.
if ! command -v cargo >/dev/null 2>&1 && [ -f "$HOME/.cargo/env" ]; then
    # shellcheck disable=SC1091
    . "$HOME/.cargo/env"
fi

# --- What must be installed ------------------------------------------------

# Build tools and the system libraries of Tauri 2 (the same list as the
# Linux jobs of .github/workflows/app.yml).
packages=(
    build-essential pkg-config rsync
    libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev libayatana-appindicator3-dev
    libssl-dev libdbus-1-dev libxdo-dev patchelf
)
missing=()
for p in "${packages[@]}"; do
    dpkg -s "$p" >/dev/null 2>&1 || missing+=("$p")
done

# The frontend in dist/ (tauri::generate_context! embeds it): the one built
# on Windows is copied; without it, Node.js builds it here.
need_node=0
if [ ! -f "$src/dist/index.html" ]; then
    need_node=1
fi

problems=0
if [ ${#missing[@]} -gt 0 ]; then
    echo "Missing system packages: ${missing[*]}"
    echo "Install them with:"
    echo "  sudo apt-get update && sudo apt-get install -y ${missing[*]}"
    echo
    problems=1
fi
if ! command -v cargo >/dev/null 2>&1; then
    echo "Rust (cargo) is missing. Install it for this user, without sudo:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --component clippy"
    echo "then open a new shell (or: . \"\$HOME/.cargo/env\")."
    echo
    problems=1
else
    # rust-version of tauri/Cargo.toml.
    want="$(sed -n 's/^rust-version *= *"\([0-9.]*\)".*/\1/p' "$src/tauri/Cargo.toml" | head -n 1)"
    have="$(rustc --version | sed -n 's/^rustc \([0-9][0-9.]*\).*/\1/p')"
    if [ -n "$want" ] && [ -n "$have" ] &&
        [ "$(printf '%s\n%s\n' "$want" "$have" | sort -V | head -n 1)" != "$want" ]; then
        echo "Rust $have is older than $want (rust-version of tauri/Cargo.toml). Update it with:"
        echo "  rustup update stable"
        echo
        problems=1
    fi
    if [ $clippy -eq 1 ] && ! cargo clippy --version >/dev/null 2>&1; then
        echo "clippy is missing. Add it with:"
        echo "  rustup component add clippy"
        echo
        problems=1
    fi
fi
if [ $need_node -eq 1 ] && ! command -v npm >/dev/null 2>&1; then
    echo "dist/ is missing and Node.js is not installed here. Either build the"
    echo "frontend on Windows first (npm -w ui run build, in the repository root),"
    echo "or install Node.js in Linux (https://nodejs.org, or: sudo apt-get install -y nodejs npm)."
    echo
    problems=1
fi
if [ $problems -ne 0 ]; then
    exit 2
fi

# --- Copy into the Linux file system ----------------------------------------

mkdir -p "$dest"
dest="$(cd "$dest" && pwd)"
if [ "$src" = "$dest" ]; then
    echo "Already in $dest: nothing to copy"
else
    echo "Copying $src -> $dest"
    # Anchored with /: tauri/src/projects and the data folders inside the
    # sources are copied. node_modules holds programs for Windows; the kit
    # folders (data, projects) and the agents' worktrees are not sources.
    # Excluded paths are never deleted from the copy: its target/ stays.
    rsync -a --delete \
        --exclude='/target' --exclude='/target-*' --exclude='/target_*' \
        --exclude='node_modules/' \
        --exclude='/.claude/worktrees/' \
        --exclude='/data/' --exclude='/projects/' \
        --exclude='/.git' \
        "$src/" "$dest/"
fi

if [ $need_node -eq 1 ]; then
    echo "Building the frontend (dist/)"
    (cd "$dest" && npm ci --no-fund --no-audit && npm -w ui run build)
fi

# --- Tests --------------------------------------------------------------------

log="$dest/target/test-linux.log"
mkdir -p "$dest/target"
status=0
if [ $clippy -eq 1 ]; then
    echo "cargo clippy --all-targets -- -D warnings"
    cargo clippy --manifest-path "$dest/Cargo.toml" --all-targets -- -D warnings || status=1
fi
echo "cargo test ${cargo_args[*]:-}"
cargo test --manifest-path "$dest/Cargo.toml" "${cargo_args[@]}" 2>&1 | tee "$log" || status=1

echo
echo "=== Summary (linux, $(uname -m)) ==="
if grep -q '^test result:' "$log"; then
    grep '^test result:' "$log" | awk '{ p += $4; f += $6; i += $8 }
        END { printf "passed %d, failed %d, ignored %d\n", p, f, i }'
    if grep -qE '^test .* \.\.\. FAILED$' "$log"; then
        echo "Failed:"
        grep -E '^test .* \.\.\. FAILED$' "$log" | sed 's/^test \(.*\) \.\.\. FAILED$/  \1/'
    fi
else
    echo "No test result: the tests did not build (see $log)."
    status=1
fi
echo "Log: $log"
exit $status
