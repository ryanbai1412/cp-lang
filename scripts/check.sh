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
#            (--fast compiles C++ through scripts/cxx-cache.py, which reuses
#            a binary built from the same source, flags and g++;
#            CHECK_NO_CXX_CACHE=1 turns it off)
#         4. cpl test stdlib examples/local-lib, run beside 3
#         5. cpl fmt --check stdlib (the standard library stays formatted)
#            and stdlib/prelude.cp holds at most 80 names
#            and every port in corpus/problems passes cpl check,
#            cpl fmt --check and cpl lint with 0 warnings (under a second)
#            and cpl build writes at most 65,536 bytes of C++ for each
#         6. the count of open known_traps (crates/cpl-cli/tests/known_traps.cp)
#         7. the offline site bundle (python3 site/bundle.py) builds; parts
#            over their caps (compiler 2,100,000 bytes, docs 500,000,
#            gallery 150,000) warn
#            and site/test_site.py passes (links, reader copy, documented
#            programs compiled natively; about two minutes)
# --full  8. the prover's products fuzzer
#         9. the budget tests and the front-end latency test, each alone
#        10. the WebAssembly build and crates/cpl-wasm/tests/smoke.mjs
#        11. the VS Code extension tests (editors/vscode: npm test, and its
#            extension-host suite under a display or xvfb-run), then
#            editors/smoke.sh: Neovim, Emacs and Helix when installed
#
# Needs cargo with clippy, python3 with mistune (python3 -m pip install -r
# site/requirements.txt) and the wasm target (rustup target add
# wasm32-unknown-unknown, for the site bundle); --full also needs node, npm
# and g++.
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
        --full) mode=full ;;
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
if [ "$mode" = fast ] && [ "${CHECK_NO_CXX_CACHE:-0}" != "1" ] && command -v g++ >/dev/null; then
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
    find "$CPL_CXX_CACHE_DIR" -type f -mtime +7 -delete 2>/dev/null || true
fi

# Each step's wall time, so a slow gate shows where its time goes.
step_name="setup"
step_t0=$gate_t0
step_times=()
step_end() {
    step_times+=("$(( $(date +%s) - step_t0 ))"$'\t'"$step_name")
}
step() {
    step_end
    step_name="$*"
    step_t0=$(date +%s)
    printf '\n==> %s\n' "$*"
}
fail() { echo "check: $*" >&2; exit 1; }
# Each cached stage's key hashes the whole tree minus that stage's excludes
# (scripts/check_cache.py), so a file nobody thought to list still reruns it.
manifest="$cache_dir/manifest.$$"
if [ "$use_cache" -eq 1 ]; then
    mkdir -p "$cache_dir"
    python3 scripts/check_cache.py manifest "$manifest" || use_cache=0
fi
cache_hit() {
    [ "$use_cache" -eq 1 ] || return 1
    python3 scripts/check_cache.py check "$cache_dir" "$1" "$manifest"
}
cache_save() {
    [ "$use_cache" -eq 1 ] || return 0
    python3 scripts/check_cache.py save "$cache_dir" "$1" "$manifest"
}

markers=$(git grep -nE '^(<{7}|>{7}|\|{7})( |$)' -- ':!*.png' ':!*.gz' || true)
[ -z "$markers" ] || fail "merge conflict markers left in:
$markers"
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
cpl="$target/debug/cpl"

step "cargo clippy --workspace --all-targets -- -D warnings"
if cache_hit clippy; then
    echo "cached (tree unchanged outside its excludes)"
else
    cargo clippy --workspace --all-targets -- -D warnings
    cache_save clippy
fi

# The library tests only need the cpl binary, so they run beside the
# workspace tests, one process per library directory, and their output is
# printed after them. A .cp file directly in examples/local-lib/ would not be
# in any subdirectory, so then the whole library runs as one process.
# Never cached: another tree may have rebuilt the shared binary since.
cargo build -q -p cpl-cli -p cpl-conform
lib_logs="$(mktemp -d)"
lib_dirs=(stdlib)
if compgen -G "examples/local-lib/*.cp" >/dev/null; then
    lib_dirs+=(examples/local-lib)
else
    lib_dirs+=(examples/local-lib/*/)
fi
lib_pids=()
bundle_pid=""
script_test_pid=""
recovery_pid=""
scoreboard_pid=""
site_test_pid=""
stdlib_cached=0
if cache_hit stdlib_test; then
    stdlib_cached=1
else
    for d in "${lib_dirs[@]}"; do
        "$cpl" test "$d" >"$lib_logs/$(tr / _ <<<"$d").log" 2>&1 &
        lib_pids+=($!)
    done
fi
bundle_cached=0
if [ -f dist/cp-lang.html ] && cache_hit site_bundle; then
    bundle_cached=1
else
    rustc --print target-libdir --target wasm32-unknown-unknown 2>/dev/null | xargs test -d \
        || fail "the site bundle needs the wasm target; run: rustup target add wasm32-unknown-unknown"
    need python3 "site/bundle.py runs on it"
    python3 -c 'import mistune' 2>/dev/null \
        || fail "the site bundle needs the Python package \`mistune\`; run: python3 -m pip install -r site/requirements.txt"
    # Separate CARGO_TARGET_DIR so the wasm build runs alongside nextest without Cargo lock contention.
    CARGO_TARGET_DIR="$target/wasm-cargo" python3 site/bundle.py >"$lib_logs/site_bundle.log" 2>&1 &
    bundle_pid=$!
fi
scoreboard_cached=0
if cache_hit scoreboard_fast; then
    scoreboard_cached=1
fi
cleanup() {
    kill "${lib_pids[@]}" ${bundle_pid:+"$bundle_pid"} ${script_test_pid:+"$script_test_pid"} ${recovery_pid:+"$recovery_pid"} ${scoreboard_pid:+"$scoreboard_pid"} ${site_test_pid:+"$site_test_pid"} 2>/dev/null || true
    rm -rf "$lib_logs" "$manifest" ${tmp:+"$tmp"}
}
trap cleanup EXIT

# These run in --full instead:
# - the prover fuzzer takes 7 s, longer than most of the suite together;
# - budget and the 50 ms front-end latency test measure wall-clock time,
#   which other tests running beside them skew;
# - the recovery and fix-it scoreboards check thousands of broken ports on
#   every core, which skews the LSP latency test beside them, so they run
#   after the rest;
# - committed_scoreboard_ratchet runs the same conformance+corpus+docs ratchet
#   as `cpl-conform all --check` below (which also checks `diff`), so running
#   both in --fast would execute ~3,400 cases twice.
slow_tests=(
    products_and_variable_divisors_are_never_decided_wrongly
    front_end_check_is_under_50_ms
    recovery_scoreboard
    fix_it_scoreboard
    committed_scoreboard_ratchet
)
if cache_hit nextest_doc; then
    step "cargo nextest + doctests"
    echo "cached (tree unchanged outside its excludes)"
else
    if cargo nextest --version >/dev/null 2>&1; then
        step "cargo nextest run --workspace (without the --full tests)"
        filter='binary(budget)'
        for t in "${slow_tests[@]}"; do filter+=" or test(/(^|::)$t\$/)"; done
        cargo nextest run --workspace --no-fail-fast -E "not ($filter)"
        step "cargo test --workspace --doc"
        cargo test -q --workspace --doc
    else
        step "cargo test --workspace (without the --full tests)"
        skips=()
        for t in "${slow_tests[@]}" \
            run_answers_within_200_ms build_takes_under_a_second_and_fits_64_kb; do
            skips+=(--skip "$t")
        done
        cargo test --workspace -- "${skips[@]}"
    fi
    cache_save nextest_doc
fi

# Run the recovery scoreboard in parallel with the remaining fast checks
# (nextest, doctests, and LSP latency tests have finished, so no Cargo lock contention).
recovery_cached=0
if cache_hit recovery; then
    recovery_cached=1
else
    cargo test -q -p cpl-cli --test recovery >"$lib_logs/recovery.log" 2>&1 &
    recovery_pid=$!
fi

step "cpl test stdlib examples/local-lib"
if [ "$stdlib_cached" -eq 1 ]; then
    echo "cached (tree unchanged outside its excludes)"
else
    lib_status=0
    for pid in "${lib_pids[@]}"; do
        wait "$pid" || lib_status=1
    done
    cat "$lib_logs"/*.log | awk '/^[0-9]+ passed, [0-9]+ failed$/ { p += $1; f += $3 }
        END { printf "%d passed, %d failed\n", p, f }'
    [ "$lib_status" -eq 0 ] || { cat "$lib_logs"/*.log; fail "library tests failed; rerun: cargo run -p cpl-cli -- test stdlib examples/local-lib"; }
    cache_save stdlib_test
fi

step "stdlib/prelude.cp: at most 80 names"
prelude=$(sed 's://.*::' stdlib/prelude.cp | tr -d '\n' | grep -o '{[^}]*}' | tr ',' '\n' | grep -c '[A-Za-z_]')
echo "prelude: $prelude names"
[ "$prelude" -le 80 ] || echo "check: WARNING: stdlib/prelude.cp has $prelude names, over the target of 80" >&2

step "cpl fmt --check stdlib"
"$cpl" fmt --check stdlib || fail "stdlib/ is not formatted; run: cargo run -p cpl-cli -- fmt stdlib"

# Fast validation scoreboard + script unit tests run in parallel with docs/site tests.
if [ "$scoreboard_cached" -eq 0 ]; then
    python3 -m unittest scripts/test_dream_gaps.py scripts/test_review_check.py scripts/test_check_cache.py scripts/test_cxx_cache.py >"$lib_logs/script_tests.log" 2>&1 &
    script_test_pid=$!
    "$target/debug/cpl-conform" all --check >"$lib_logs/scoreboard.log" 2>&1 &
    scoreboard_pid=$!
fi

# Taste reviews: always run (fast, checks working-tree/staged ports).
step "taste reviews (scripts/review-check.py --check)"
python3 scripts/review-check.py --check \
    || fail "taste review gate failed; see research/reviews/README.md"

# The offline edition is one file people download; site/test_site.py holds
# the same cap. The WASM build is a cargo no-op when nothing changed.
check_bundle_parts() {
    # Size caps are targets, not correctness: an overrun warns and never blocks a landing.
    python3 site/bundle.py --check-caps ||
        printf 'check: WARNING: a site bundle part is over its cap (caps in site/bundle.py)\n' >&2
}
step "site bundle: compiler under 2,100,000 bytes, docs under 500,000, gallery under 150,000"
if [ "$bundle_cached" -eq 1 ]; then
    bundle_bytes=$(wc -c <dist/cp-lang.html)
    echo "dist/cp-lang.html: $bundle_bytes bytes (cached)"
    check_bundle_parts
else
    bundle_status=0
    wait "$bundle_pid" || bundle_status=1
    bundle_pid=""
    [ "$bundle_status" -eq 0 ] || { cat "$lib_logs/site_bundle.log"; fail "site/bundle.py failed; rerun: python3 site/bundle.py"; }
    bundle_bytes=$(wc -c <dist/cp-lang.html)
    echo "dist/cp-lang.html: $bundle_bytes bytes"
    check_bundle_parts
    cache_save site_bundle
fi

step "docs gate, then site tests"
if cache_hit docs_site; then
    echo "cached (tree unchanged outside its excludes)"
else
    # test_site rebuilds site/_build (bundle.bundle()), which check_docs reads, so it starts after.
    python3 site/check_docs.py || fail "docs gate failed; rerun: python3 site/check_docs.py"
    CPL_SITE_PREBUILT=1 python3 site/test_site.py >"$lib_logs/site_tests.log" 2>&1 &
    site_test_pid=$!
    site_test_status=0
    wait "$site_test_pid" || site_test_status=1
    site_test_pid=""
    [ "$site_test_status" -eq 0 ] || { cat "$lib_logs/site_tests.log"; fail "site tests failed; rerun: python3 site/test_site.py"; }
    cache_save docs_site
fi

step "cpl-conform ratchet (all --check against research/scoreboard.json)"
if [ "$scoreboard_cached" -eq 1 ]; then
    echo "cached (tree unchanged outside its excludes)"
elif [ -n "${scoreboard_pid:-}" ]; then
    script_status=0
    if [ -n "${script_test_pid:-}" ]; then
        wait "$script_test_pid" || script_status=1
        script_test_pid=""
    fi
    sb_status=0
    wait "$scoreboard_pid" || sb_status=1
    cat "$lib_logs/script_tests.log" "$lib_logs/scoreboard.log"
    scoreboard_pid=""
    [ "$script_status" -eq 0 ] || fail "script unit test failure; rerun: python3 -m unittest scripts/test_*.py"
    [ "$sb_status" -eq 0 ] || fail "cpl-conform ratchet failure; rerun: cargo run -p cpl-conform -- all --check"
    cache_save scoreboard_fast
fi

if [ "$recovery_cached" -eq 1 ]; then
    step "cargo test: the recovery scoreboard"
    echo "cached (tree unchanged outside its excludes)"
elif [ -n "${recovery_pid:-}" ]; then
    step "cargo test: the recovery scoreboard (backgrounded)"
    recovery_status=0
    wait "$recovery_pid" || recovery_status=1
    cat "$lib_logs/recovery.log"
    [ "$recovery_status" -eq 0 ] || fail "recovery scoreboard failed; rerun: cargo test -p cpl-cli --test recovery"
    recovery_pid=""
    cache_save recovery
fi
if [ "$mode" = full ]; then
    need node "the wasm smoke test and the VS Code tests run on it"
    need npm "the VS Code tests run on it"
    need g++ "the judge-profile test compiles the ports with it"

    tmp="$(mktemp -d)"

    step "cargo test: the prover fuzzer"
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
        if [ -n "${DISPLAY:-}" ]; then
            npm run test:integration
        elif command -v xvfb-run >/dev/null; then
            xvfb-run -a npm run test:integration
        else
            echo "editors/vscode: SKIPPED extension-host tests (no display and no xvfb-run)"
        fi
    )

    step "editors/smoke.sh: the other editor setups against cpl lsp"
    editors/smoke.sh
fi

# The Trap Parity Gate's open register: wrong programs that `cpl run`, the
# judge build, or both still let through. trap_parity.rs fails when one closes.
known_traps=$(grep -c '^if which == ' crates/cpl-cli/tests/known_traps.cp)
printf '\nknown_traps: %s open (crates/cpl-cli/tests/known_traps.cp)\n' "$known_traps"

step_end
printf '\nstep times (s):\n'
printf '%s\n' "${step_times[@]}" | sort -rn | awk -F'\t' '$1 > 0 { printf "%5d  %s\n", $1, $2 }'
gate_elapsed=$(( $(date +%s) - gate_t0 ))
if [ "$mode" = fast ]; then
    gate_time_file="$target/gate-fast-time.json"
    mkdir -p "$(dirname "$gate_time_file")"
    printf '{"mode":"fast","elapsed_s":%d,"budget_s":180}\n' "$gate_elapsed" >"$gate_time_file"
    printf '\ncheck: all fast checks passed in %ss (budget <= 180s)\n' "$gate_elapsed"
    # .githooks/pre-push lands only trees recorded here, so a rebased tip must
    # pass again before it reaches main. Unstaged edits mean the checked tree
    # is not the index tree, so nothing is recorded.
    if git diff --quiet 2>/dev/null; then
        stamp="$(git rev-parse --git-common-dir)/cpl-gate-passed"
        { tail -n 199 "$stamp" 2>/dev/null || true; git write-tree; } >"$stamp.tmp" && mv "$stamp.tmp" "$stamp"
    fi
    # Wall time depends on the machine and on how warm the build is, so an
    # overrun warns and stays recorded in the scoreboard instead of failing.
    [ "$gate_elapsed" -le 180 ] ||
        printf 'check: WARNING: fast gate took %ss, over the 180s (3 min) budget\n' "$gate_elapsed"
else
    printf '\ncheck: all %s checks passed in %ss\n' "$mode" "$gate_elapsed"
fi
