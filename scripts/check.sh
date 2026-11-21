#!/usr/bin/env bash
# The checks to run before a commit or a push. There is no hosted CI on
# this repository: this script, run on your machine, is the gate.
#
#   scripts/check.sh           same as --fast
#   scripts/check.sh --fast    what the pre-commit hook runs (about 10 s warm)
#   scripts/check.sh --full    --fast, then the slow suites (15 minutes or more)
#
# --fast  1. nothing from dist/ and no secret is staged or in an unpushed commit
#         2. cargo clippy --workspace --all-targets -- -D warnings
#         3. the workspace tests (cargo nextest when installed, else cargo
#            test), without the tests listed under 8 and 9
#         4. cpl test stdlib examples/local-lib, run beside 3
#         5. cpl fmt --check stdlib (the standard library stays formatted)
#            and stdlib/prelude.cp holds at most 80 names
#            and every port in proto/ports/ecnerwala passes cpl check,
#            cpl fmt --check and cpl lint with 0 warnings (under a second)
#            and cpl build writes at most 65,536 bytes of C++ for each
#         6. the count of open known_traps (crates/cpl-cli/tests/known_traps.cp)
#         7. the offline site bundle (python3 site/bundle.py) stays under
#            2,600,000 bytes
# --full  8. proto_fuzz and proto_tools (the retired prototype), the shrinker
#            fuzzer and the prover's products fuzzer
#         9. the budget tests and the front-end latency test, each alone
#        10. the WebAssembly build and crates/cpl-wasm/tests/smoke.mjs
#        11. the VS Code extension tests (editors/vscode: npm test), then
#            editors/smoke.sh: Neovim, Emacs and Helix when installed
#        12. proto/run_all.sh, in a throwaway clone because it stamps its sources
#
# Needs cargo with clippy, python3 with mistune (python3 -m pip install -r
# site/requirements.txt) and the wasm target (rustup target add
# wasm32-unknown-unknown, for the site bundle); --full also needs node, npm
# and g++.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

mode=fast
for arg in "$@"; do
    case "$arg" in
        --fast) mode=fast ;;
        --full) mode=full ;;
        -h | --help) sed -n '2,25s/^# \{0,1\}//p' "$0"; exit 0 ;;
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
        local top hosts deny
        top="$(git rev-parse --show-toplevel)"
        hosts="$(grep -oE 'https?://[A-Za-z0-9.-]+' <<<"$lines" | sed -E 's#^https?://##' | sort -u \
            | grep -vxE -f <(grep -v '^#' "$top/scripts/public-hosts.txt") || true)"
        if [ -n "$hosts" ]; then
            echo "$hosts"
            fail "added lines link to the hosts above, which are not in scripts/public-hosts.txt; the history will be public, so remove the links or add a public host to that list"
        fi
        if grep -nE '/(home|Users)/[A-Za-z0-9_.-]+/' <<<"$lines" | head -3 | grep .; then
            fail "added lines hold personal home paths; use paths relative to the repository"
        fi
        # Words that must never be published can't be listed in the repository
        # itself, so each machine keeps its own list outside the history.
        deny="$(git rev-parse --git-common-dir)/info/sensitive-words"
        if [ -s "$deny" ] && grep -inF -f "$deny" <<<"$lines" | head -3 | cut -c1-60 | grep .; then
            fail "added lines match the local list in .git/info/sensitive-words"
        fi
    done
}

step "staged files: no dist/ output, no secrets"
staged_guard

# npm writes the registry it installed from into the lockfile; only the
# public registry's URLs belong in the repository.
step "lockfiles: packages resolve from the public npm registry"
if git grep -h -E '"resolved": "' -- '*package-lock.json' | grep -v '"resolved": "https://registry.npmjs.org/' | head -3 | grep .; then
    fail "a lockfile resolves packages from a private mirror; rewrite its URLs to https://registry.npmjs.org/"
fi

need cargo "see https://rustup.rs"
step "cargo clippy --workspace --all-targets -- -D warnings"
cargo clippy --workspace --all-targets -- -D warnings

# The library tests only need the cpl binary, so they run beside the
# workspace tests, one process per library directory, and their output is
# printed after them. A .cp file directly in examples/local-lib/ would not be
# in any subdirectory, so then the whole library runs as one process.
cargo build -q -p cpl-cli
lib_logs="$(mktemp -d)"
lib_dirs=(stdlib)
if compgen -G "examples/local-lib/*.cp" >/dev/null; then
    lib_dirs+=(examples/local-lib)
else
    lib_dirs+=(examples/local-lib/*/)
fi
lib_pids=()
for d in "${lib_dirs[@]}"; do
    "${CARGO_TARGET_DIR:-target}/debug/cpl" test "$d" >"$lib_logs/$(tr / _ <<<"$d").log" 2>&1 &
    lib_pids+=($!)
done
cleanup() {
    kill "${lib_pids[@]}" 2>/dev/null || true
    rm -rf "$lib_logs" ${tmp:+"$tmp"}
}
trap cleanup EXIT

# These run in --full instead:
# - proto_fuzz and proto_tools check the retired Python prototype (over a
#   minute, and 3 to 7 s per proto_tools test; with plain cargo test only
#   proto_fuzz is left out);
# - the shrinker and prover fuzzers take 21 s and 7 s, longer than the rest
#   of the suite together;
# - budget and the 50 ms front-end latency test measure wall-clock time,
#   which other tests running beside them skew;
# - the recovery scoreboard checks 3,400 broken ports on every core, which
#   skews the LSP latency test beside it, so it runs after the rest.
slow_tests=(
    shrinks_a_failing_program_to_a_few_lines
    products_and_variable_divisors_are_never_decided_wrongly
    front_end_check_is_under_50_ms
    recovery_scoreboard
)
if cargo nextest --version >/dev/null 2>&1; then
    step "cargo nextest run --workspace (without the --full tests)"
    filter='binary(proto_fuzz) or binary(proto_tools) or binary(budget)'
    for t in "${slow_tests[@]}"; do filter+=" or test(/(^|::)$t\$/)"; done
    cargo nextest run --workspace --no-fail-fast -E "not ($filter)"
    step "cargo test --workspace --doc"
    cargo test -q --workspace --doc
    step "cargo test: the recovery scoreboard, on its own"
    cargo test -q -p cpl-cli --test recovery
else
    step "cargo test --workspace (without the --full tests)"
    skips=()
    for t in "${slow_tests[@]}" prototype_agrees_with_the_reference_evaluator \
        run_answers_within_200_ms build_takes_under_a_second_and_fits_64_kb; do
        skips+=(--skip "$t")
    done
    cargo test --workspace -- "${skips[@]}"
    step "cargo test: the recovery scoreboard, on its own"
    cargo test -q -p cpl-cli --test recovery
fi

step "cpl test stdlib examples/local-lib"
lib_status=0
for pid in "${lib_pids[@]}"; do
    wait "$pid" || lib_status=1
done
cat "$lib_logs"/*.log | awk '/^[0-9]+ passed, [0-9]+ failed$/ { p += $1; f += $3 }
    END { printf "%d passed, %d failed\n", p, f }'
[ "$lib_status" -eq 0 ] || { cat "$lib_logs"/*.log; fail "library tests failed; rerun: cargo run -p cpl-cli -- test stdlib examples/local-lib"; }

step "stdlib/prelude.cp: at most 80 names"
prelude=$(sed 's://.*::' stdlib/prelude.cp | tr -d '\n' | grep -o '{[^}]*}' | tr ',' '\n' | grep -c '[A-Za-z_]')
echo "prelude: $prelude names"
[ "$prelude" -le 80 ] || { echo "check: stdlib/prelude.cp has $prelude names, over the cap of 80" >&2; exit 1; }

step "cpl fmt --check stdlib"
cargo run -q -p cpl-cli -- fmt --check stdlib || fail "stdlib/ is not formatted; run: cargo run -p cpl-cli -- fmt stdlib"

# Fast validation scoreboard: every active port under proto/ports/ passes
# cpl check, cpl fmt --check, cpl lint (0 warnings), and cpl build (<= 64 KB),
# plus contestant tokens, friction_lint (0 wa_trap), friction-report summary,
# and regression check against research/scoreboard.json.
step "validation scoreboard (--fast --check against research/scoreboard.json)"
cpl="${CARGO_TARGET_DIR:-target}/debug/cpl"
python3 -m unittest scripts/test_scoreboard.py >/dev/null \
    || fail "scripts/test_scoreboard.py failed; rerun: python3 -m unittest scripts/test_scoreboard.py -v"
python3 scripts/scoreboard.py --fast --check --quiet --cpl "$cpl" \
    || fail "scoreboard regression detected; rerun: python3 scripts/scoreboard.py --fast --check"

# Taste reviews: every port newer than research/reviews/baseline.json has a
# dream and a review, and the count of those scoring under 4 may not grow.
step "taste reviews (scripts/review-check.py --check)"
python3 -m unittest scripts/test_review_check.py >/dev/null 2>&1 \
    || fail "scripts/test_review_check.py failed; rerun: python3 -m unittest scripts/test_review_check.py -v"
python3 scripts/review-check.py --check \
    || fail "taste review gate failed; see research/reviews/README.md"

# The offline edition is one file people download; site/test_site.py holds
# the same cap. The WASM build is a cargo no-op when nothing changed.
step "site bundle: dist/cp-lang.html under 2,600,000 bytes"
rustc --print target-libdir --target wasm32-unknown-unknown 2>/dev/null | xargs test -d \
    || fail "the site bundle needs the wasm target; run: rustup target add wasm32-unknown-unknown"
need python3 "site/bundle.py runs on it"
python3 -c 'import mistune' 2>/dev/null \
    || fail "the site bundle needs the Python package \`mistune\`; run: python3 -m pip install -r site/requirements.txt"
python3 site/bundle.py >/dev/null 2>&1 || fail "site/bundle.py failed; rerun: python3 site/bundle.py"
bundle_bytes=$(wc -c <dist/cp-lang.html)
echo "dist/cp-lang.html: $bundle_bytes bytes"
[ "$bundle_bytes" -lt 2600000 ] || fail "dist/cp-lang.html is $bundle_bytes bytes, over the cap of 2,600,000"

step "docs examples, course references, cheat-sheet budget and links"
python3 site/check_docs.py || fail "docs gate failed; rerun: python3 site/check_docs.py"

if [ "$mode" = full ]; then
    need node "the wasm smoke test and the VS Code tests run on it"
    need npm "the VS Code tests run on it"
    need python3 "proto/run_all.sh runs on it"
    need g++ "proto/run_all.sh compiles the ports with it"

    tmp="$(mktemp -d)"

    step "cargo test: proto_fuzz, proto_tools and the slow fuzzers"
    cargo test -p cpl-verify --test proto_fuzz
    cargo test -p cpl-verify --test proto_tools
    cargo test -p cpl-verify --lib -- --exact fuzz::shrink::tests::shrinks_a_failing_program_to_a_few_lines
    cargo test -p cpl-prover --test fuzz -- --exact products_and_variable_divisors_are_never_decided_wrongly

    step "cargo test: every port compiled with every judge profile's flags"
    cargo test -p cpl-cli --test judges -- --ignored --exact every_port_compiles_with_every_judges_flags

    step "cargo test: budget, then the front-end latency, each on its own"
    cargo test -p cpl-cli --test budget -- --test-threads=1
    cargo test -p cpl-lsp --test latency -- --exact front_end_check_is_under_50_ms

    step "wasm build and crates/cpl-wasm/tests/smoke.mjs"
    scripts/build-wasm.sh --smoke --out "$tmp/wasm"

    step "editors/vscode: npm test"
    cargo build -q -p cpl-cli    # the LSP test starts target/debug/cpl
    (
        cd editors/vscode
        [ -d node_modules ] || npm ci --no-audit --no-fund
        npm test
    )

    step "editors/smoke.sh: the other editor setups against cpl lsp"
    editors/smoke.sh

    step "proto/run_all.sh (in a throwaway clone)"
    git clone -q --no-hardlinks "$root" "$tmp/repo"
    git -C "$tmp/repo" checkout -q --detach "$(git rev-parse HEAD)"
    # carry over uncommitted changes to tracked files, so the tree under test is the one on disk
    if ! git diff --quiet HEAD; then
        git diff --binary HEAD | git -C "$tmp/repo" apply
    fi
    bash "$tmp/repo/proto/run_all.sh"
fi

# The Trap Parity Gate's open register: wrong programs that `cpl run`, the
# judge build, or both still let through. trap_parity.rs fails when one closes.
known_traps=$(grep -c '^if which == ' crates/cpl-cli/tests/known_traps.cp)
printf '\nknown_traps: %s open (crates/cpl-cli/tests/known_traps.cp)\n' "$known_traps"

printf '\ncheck: all %s checks passed\n' "$mode"
