#!/usr/bin/env bash
# The checks to run before a commit or a push. There is no hosted CI on
# this repository: this script, run on your machine, is the gate.
#
#   scripts/check.sh           same as --fast
#   scripts/check.sh --commit  what the pre-commit hook runs: the checks the
#                              staged paths can reach (scripts/affected.py)
#   scripts/check.sh --land    --fast, but only the tests the paths changed
#                              since $CHECK_BASE (default origin/main) reach
#   scripts/check.sh --fast    what CI runs on every tip
#   scripts/check.sh --full    --fast, then the slow suites (15 minutes or more)
#
# --fast  1. nothing from dist/ and no secret is staged or in an unpushed commit
#         2. cargo clippy --workspace --all-targets -- -D warnings
#            and cargo check of cpl-wasm for wasm32 (the playground build)
#         3. the workspace tests (cargo nextest when installed, else cargo
#            test), without the tests listed under 8 and 9
#            (--fast compiles C++ through scripts/cxx-cache.py, which reuses
#            a binary built from the same source, flags and g++;
#            CHECK_NO_CXX_CACHE=1 turns it off)
#         4. the std/prelude.cp name count (decision 281: no budget)
#            and every port in corpus/problems passes cpl check,
#            cpl fmt --check and cpl lint with 0 warnings (under a second)
#            and cpl build writes at most 65,536 bytes of C++ for each
#         6. the count of open known_traps (crates/cpl-cli/tests/known_traps.cp)
#         7. the site's snippet gate (node web/scripts/gate.mjs): every
#            program on the site is at least at its declared status,
#            and the trace viewer's node tests (tools/trace/*.test.cjs)
# --full  (the steps are scripts/ci.py STEPS["full"])
#         8. the prover's products fuzzer, and 100 random programs
#            (scripts/gen-programs.py) against an independent model
#         9. the budget tests and the front-end latency test, each alone
#        10. the WebAssembly build and crates/cpl-wasm/tests/smoke.mjs
#            and scripts/site-check.sh: the site build (with the
#            gzipped cap on cpl.wasm in web/scripts/wasm-size.mjs) and its browser smoke
#        11. the VS Code extension tests (editors/vscode: npm test, and its
#            extension-host suite under a display or xvfb-run), then
#            editors/smoke.sh: Neovim, Emacs and Helix when installed
#
# --commit  1, rustfmt, clippy of the touched crates (and the wasm32 check
#           when cpl-wasm builds one of them), and the nextest tests
#           of every crate that can reach them except the end-to-end suites
#           (profile `commit` in .config/nextest.toml), plus the unit tests
#           of touched scripts and the snippet gate for web/ and docs/. A
#           path scripts/affected.py does not know runs --fast instead.
#
# Needs cargo with clippy, python3 and node; --full also needs npm, pnpm, a
# Playwright Chromium, g++ and the wasm target (rustup target add wasm32-unknown-unknown).
set -euo pipefail

gate_t0=$(date +%s)
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
. scripts/gate-env.sh
rm -f "$target/gate-fast-time.json"

mode=fast
use_cache=1
[ "${CHECK_NO_CACHE:-0}" = "1" ] && use_cache=0
for arg in "$@"; do
    case "$arg" in
        --fast) mode=fast ;;
        --commit) mode=commit ;;
        --land) mode=land ;;
        --full) mode=full ;;
        --timing) mode=timing ;;
        --no-cache) use_cache=0 ;;
        -h | --help) sed -n '2,25s/^# \{0,1\}//p' "$0"; exit 0 ;;
        *) echo "check: unknown argument \`$arg\`" >&2; exit 2 ;;
    esac
done
[ "$mode" = full ] && use_cache=0
cache_dir="$target/check-cache"

# The fast gate compiles the same emitted C++ on every run (profile, judges,
# trap parity), so `g++` goes through scripts/cxx-cache.py, which reuses a
# binary built from the same bytes, flags, compiler and system headers. The
# cache is per user, not per checkout, so a fresh worktree starts warm.
# --full and CHECK_NO_CXX_CACHE=1 compile everything.
if { [ "$mode" = fast ] || [ "$mode" = land ]; } && [ "${CHECK_NO_CXX_CACHE:-0}" != "1" ] && command -v g++ >/dev/null; then
    export CPL_CXX_SHIM_DIR="$target/cxx-shim"
    export CPL_CXX_CACHE_DIR="${CPL_CXX_CACHE_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/cpl/cxx}"
    mkdir -p "$CPL_CXX_SHIM_DIR" "$CPL_CXX_CACHE_DIR"
    ln -sf "$root/scripts/cxx-cache.py" "$CPL_CXX_SHIM_DIR/g++"
    export PATH="$CPL_CXX_SHIM_DIR:$PATH"
    # Fingerprint the system headers and libraries once per gate, not per compile.
    CPL_CXX_SYSTEM_KEY=$(g++ --cpl-system-key 2>/dev/null) || CPL_CXX_SYSTEM_KEY=""
    export CPL_CXX_SYSTEM_KEY
    # Which judge flags this compiler accepts, asked once per gate instead
    # of twice per native build.
    export CPL_JUDGE_PROBE_DIR="$target/judge-probes"
    rm -rf "$CPL_JUDGE_PROBE_DIR"
    # Drop binaries no run has used for a week.
    [ ! -d "$CPL_CXX_CACHE_DIR" ] || find "$CPL_CXX_CACHE_DIR" -type f -mtime +7 -delete
fi

fail() { echo "check: $*" >&2; exit 1; }
# Each cached stage's key hashes the whole tree minus that stage's excludes
# (scripts/check_cache.py), so a file nobody thought to list still reruns it.
manifest="$cache_dir/manifest.$$"
trap 'rm -f "$manifest" "$manifest.reads"' EXIT
if [ "$use_cache" -eq 1 ]; then
    mkdir -p "$cache_dir"
    uv run --quiet python scripts/check_cache.py manifest "$manifest" || use_cache=0
fi

# Conflict markers, untraced programs, committed corpus tests, secrets and
# private links in the change, private npm mirrors (scripts/ci.py guard).
uv run --quiet python scripts/ci.py guard --mode "$mode" || exit 1
need() { command -v "$1" >/dev/null || fail "\`$1\` is not installed; $2"; }
need cargo "see https://rustup.rs"
stage_manifest=""
[ "$use_cache" -eq 0 ] || stage_manifest="$manifest"
# Every tier from here: the touched-path selection, the CPL_GATE_PART parts, the
# build, the tests and the checking scripts, and the step times (scripts/ci.py gate).
uv run --quiet python scripts/ci.py gate --mode "$mode" --gate-part "${CPL_GATE_PART:-}" \
    --cpl "$target/debug/cpl" --cache-dir "$cache_dir" --manifest "$stage_manifest" --since "$gate_t0"
