#!/usr/bin/env bash
# The checks to run before a commit or a push. There is no hosted CI on
# this repository: this script, run on your machine, is the gate.
#
#   scripts/check.sh           same as --fast
#   scripts/check.sh --fast    what the pre-commit hook runs (about 2 minutes)
#   scripts/check.sh --full    --fast, then the slow suites (15 minutes or more)
#
# --fast  1. nothing from dist/ and no secret is staged or in an unpushed commit
#         2. cargo clippy --workspace --all-targets -- -D warnings
#         3. cargo test --workspace
#         4. cpl test stdlib examples/local-lib
# --full  5. the WebAssembly build and crates/cpl-wasm/tests/smoke.mjs
#         6. the VS Code extension tests (editors/vscode: npm test)
#         7. proto/run_all.sh, in a throwaway clone because it stamps its sources
#
# Needs cargo with clippy; --full also needs node, npm, python3, g++ and
# the wasm target (rustup target add wasm32-unknown-unknown).
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

mode=fast
for arg in "$@"; do
    case "$arg" in
        --fast) mode=fast ;;
        --full) mode=full ;;
        -h | --help) sed -n '2,19s/^# \{0,1\}//p' "$0"; exit 0 ;;
        *) echo "check: unknown argument \`$arg\`" >&2; exit 2 ;;
    esac
done

step() { printf '\n==> %s\n' "$*"; }
fail() { echo "check: $*" >&2; exit 1; }
need() { command -v "$1" >/dev/null || fail "\`$1\` is not installed; $2"; }

# Staged changes, plus commits that are not on origin/main yet, so a bot
# that has already committed is checked before it pushes.
staged_guard() {
    local ranges=(--cached) range paths lines
    if git rev-parse -q --verify origin/main >/dev/null; then
        ranges+=("origin/main...HEAD")
    fi
    for range in "${ranges[@]}"; do
        paths="$(git diff "$range" --name-only --diff-filter=ACMR)"
        if grep -E '(^|/)dist/' <<<"$paths"; then
            fail "the files above are build output under dist/; never commit them"
        fi
        if grep -E '(^|/)(\.env(\..+)?|id_(rsa|dsa|ecdsa|ed25519)|.+\.(pem|p12|pfx|keystore))$' <<<"$paths"; then
            fail "the files above look like credentials; never commit them"
        fi
        lines="$(git diff "$range" --diff-filter=ACMR -U0 --no-color | grep -E '^\+[^+]' || true)"
        if grep -oE \
            -e 'gh[pousr]_[A-Za-z0-9]{36,}' \
            -e 'github_pat_[A-Za-z0-9_]{40,}' \
            -e 'AKIA[0-9A-Z]{16}' \
            -e 'xox[abprs]-[A-Za-z0-9-]{10,}' \
            -e 'sk-(ant-|proj-)?[A-Za-z0-9_-]{32,}' \
            -e 'AIza[0-9A-Za-z_-]{35}' \
            -e '-----BEGIN [A-Z ]*PRIVATE KEY-----' \
            <<<"$lines" | cut -c1-12 | sed 's/$/.../' | grep .; then
            fail "added lines hold what look like secrets (starting as above); remove them"
        fi
    done
}

step "staged files: no dist/ output, no secrets"
staged_guard

need cargo "see https://rustup.rs"
step "cargo clippy --workspace --all-targets -- -D warnings"
cargo clippy --workspace --all-targets -- -D warnings

step "cargo test --workspace"
cargo test --workspace

step "cpl test stdlib examples/local-lib"
cargo run -q -p cpl-cli -- test stdlib examples/local-lib | tail -n 1
[ "${PIPESTATUS[0]}" -eq 0 ] || fail "library tests failed; rerun: cargo run -p cpl-cli -- test stdlib examples/local-lib"

if [ "$mode" = full ]; then
    need node "the wasm smoke test and the VS Code tests run on it"
    need npm "the VS Code tests run on it"
    need python3 "proto/run_all.sh runs on it"
    need g++ "proto/run_all.sh compiles the ports with it"

    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT

    step "wasm build and crates/cpl-wasm/tests/smoke.mjs"
    scripts/build-wasm.sh --smoke --out "$tmp/wasm"

    step "editors/vscode: npm test"
    cargo build -q -p cpl-cli    # the LSP test starts target/debug/cpl
    (
        cd editors/vscode
        [ -d node_modules ] || npm ci --no-audit --no-fund
        npm test
    )

    step "proto/run_all.sh (in a throwaway clone)"
    git clone -q --no-hardlinks "$root" "$tmp/repo"
    git -C "$tmp/repo" checkout -q --detach "$(git rev-parse HEAD)"
    # carry over uncommitted changes to tracked files, so the tree under test is the one on disk
    if ! git diff --quiet HEAD; then
        git diff --binary HEAD | git -C "$tmp/repo" apply
    fi
    bash "$tmp/repo/proto/run_all.sh"
fi

printf '\ncheck: all %s checks passed\n' "$mode"
