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
# --full  8. the prover's products fuzzer, and 100 random programs
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
    find "$CPL_CXX_CACHE_DIR" -type f -mtime +7 -delete 2>/dev/null || true
fi

# Each step's wall time, so a slow gate shows where its time goes.
step_name="setup"
step_t0=$gate_t0
step_times=()
step_end() {
    step_times+=("$(( $(date +%s) - step_t0 ))"$'\t'"$step_name")
}
# Script tests run with an empty HOME and no system git config, like a CI
# runner: no git identity, no user config. The toolchains keep
# their real homes.
bare_home() {
    local home status=0
    home=$(mktemp -d)
    # A commit hook exports GIT_DIR and GIT_INDEX_FILE; a test inheriting them
    # writes its scratch repo into the real index.
    env -u GIT_DIR -u GIT_INDEX_FILE -u GIT_WORK_TREE -u GIT_OBJECT_DIRECTORY -u GIT_PREFIX \
        HOME="$home" XDG_CONFIG_HOME="$home/.config" GIT_CONFIG_NOSYSTEM=1 \
        CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}" RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}" \
        MISE_DATA_DIR="${MISE_DATA_DIR:-$HOME/.local/share/mise}" \
        MISE_CONFIG_DIR="${MISE_CONFIG_DIR:-$HOME/.config/mise}" \
        UV_CACHE_DIR="${UV_CACHE_DIR:-$HOME/.cache/uv}" "$@" || status=$?
    rm -rf "$home"
    return $status
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
# Every test program has its saved trace: trace_goldens fails without one,
# and the commit tier can stop before it runs. A commit answers only for
# the programs it adds or changes, so one already on main blocks no lane.
programs() {
    if [ "$mode" = commit ]; then
        git diff --cached --name-only --diff-filter=AM -- 'crates/cpl-cli/tests/programs/*.cp'
    else
        git ls-files 'crates/cpl-cli/tests/programs/*.cp'
    fi
}
untraced=$(programs |
    while read -r f; do [ -e "${f%.cp}.trace" ] || echo "$f"; done)
[ -z "$untraced" ] || fail "test programs with no saved trace; record them with
CPL_BLESS=1 cargo nextest run -p cpl-cli --test trace_goldens, read the .trace, commit it:
$untraced"
# Corpus tests are generated by each problem's gen.py and tests.toml, never
# committed (problem-packages/R9). debt-ratchet's corpus_test_files counts
# the files still in the tree.
if [ "$mode" = commit ]; then
    added=$(git diff --cached --name-only --diff-filter=A -- 'corpus/problems/*/tests/*')
    [ -z "$added" ] || fail "new files under corpus/problems/*/tests/; list the case in the problem's tests.toml and let gen.py make it (problem-packages/R9):
$added"
fi
need() { command -v "$1" >/dev/null || fail "\`$1\` is not installed; $2"; }
# The playground build. CI's site job builds cpl-wasm itself, and a lane
# box without the wasm32 target warns and skips.
wasm_check() {
    step "cargo check -p cpl-wasm --target wasm32-unknown-unknown (the playground build)"
    if [ -n "${CI:-}" ]; then
        echo "skipped in CI: the site build job builds cpl-wasm"
        return 0
    fi
    local dir
    dir=$(rustc --print target-libdir --target wasm32-unknown-unknown 2>/dev/null)
    if [ -z "$dir" ] || [ ! -d "$dir" ]; then
        echo "WARNING: skipped; the wasm32 target is not installed (run: rustup target add wasm32-unknown-unknown)" >&2
        return 0
    fi
    RUSTFLAGS="${RUSTFLAGS:-} -D warnings" cargo check -q -p cpl-wasm --target wasm32-unknown-unknown
}

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
            fail "added lines match the local list in .git/info/sensitive-words; remove those words"
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

# Which --fast leaves to --full (the reasons are above the nextest step).
slow_tests=(
    products_and_variable_divisors_are_never_decided_wrongly
    front_end_check_is_under_50_ms
    recovery_scoreboard
    fix_it_scoreboard
    committed_scoreboard_ratchet
)
slow_filter='binary(budget)'
for t in "${slow_tests[@]}"; do slow_filter+=" or test(/(^|::)$t\$/)"; done

# Wall-clock assertions (decision 225): a timing budget can fail on a loaded
# machine with no code change, so no blocking tier runs one. CI reports them
# in a job that never fails the workflow (--timing); --full runs them too.
wall_clock_tests=(
    lsp_requests_are_under_100_ms
    keystrokes_on_the_largest_ports_and_a_broken_file_are_within_budget_at_p95
    largest_corpus_file_diagnostics_hover_and_completion_p95_under_100ms
    the_checks_of_a_typical_function_fit_the_budget
    a_segment_tree_monoid_is_checked_within_the_budget
    ten_million_steps_fit_the_debugging_budgets
    every_command_starts_cold_in_under_100_ms
)
wall_filter='binary(budget) or test(/(^|::)front_end_check_is_under_50_ms$/)'
for t in "${wall_clock_tests[@]}"; do
    wall_filter+=" or test(/(^|::)$t\$/)"
    slow_filter+=" or test(/(^|::)$t\$/)"
done
# Tests that mix a timing budget into a correctness check skip the budget.
[ "$mode" = full ] || [ "$mode" = timing ] || export CPL_NO_WALL_CLOCK=1
if [ "$mode" = timing ]; then
    step "wall-clock budgets (reported, never blocking)"
    cargo nextest run --workspace --no-fail-fast \
        -E "$wall_filter or test(=every_corpus_program_and_conformance_case_is_served)"
    exit
fi

# CI splits --fast across parallel jobs (.github/workflows/ci.yml):
# CPL_GATE_PART=checks runs everything but the workspace tests, the
# conformance ratchet and fmt_check (CPL_GATE_PART=fmt), CPL_GATE_PART=conform runs the ratchet and the script
# unit tests (CPL_GATE_PART=conform:<suite+...> runs only those suites, the
# script tests with the conformance suite; a trailing @N/M passes --shard N/M), and CPL_GATE_PART=tests:<nextest partition> (e.g. tests:hash:1/12)
# runs only that share of the tests, CPL_GATE_PART=tests:doc the doctests. Unset runs
# all of --fast.
script_tests=(scripts/test_dream_gaps.py scripts/test_review_check.py scripts/test_check_cache.py scripts/test_cxx_cache.py scripts/test_release.py scripts/test_debt_ratchet.py scripts/test_next_decision.py scripts/test_fmt_check.py scripts/test_skill_gen.py scripts/test_affected.py scripts/test_land_stats.py scripts/test_devex_stats.py scripts/test_new_lint.py scripts/test_conform_staged.py scripts/test_domain_of.py scripts/test_culprit.py scripts/test_commit_status.py scripts/test_check_handbook.py scripts/test_expectation_edits.py scripts/test_conformance_gaps.py scripts/test_mutation_score.py scripts/test_conformance_bugcatch.py scripts/test_status.py scripts/test_coverage_matrix.py scripts/test_workflows.py scripts/test_wall_clock.py scripts/test_flaky.py scripts/test_justfile.py scripts/test_port_check.py scripts/test_check_usecases.py scripts/test_corpus.py scripts/test_corpus_tests.py scripts/test_differential.py scripts/test_generators.py scripts/test_stdlib_measure.py scripts/test_tools_measure.py scripts/test_conformance_coverage.py scripts/test_check_doc_links.py scripts/test_writable_from_docs.py scripts/test_spec_gate.py scripts/test_std_api_lint.py bench/test_sizes.py bench/test_v03_perf_bench.py)
gate_part="${CPL_GATE_PART:-}"
case "$gate_part" in
    "" | checks) ;;
    conform | conform:*)
        [ "$mode" = fast ] || fail "CPL_GATE_PART=$gate_part needs --fast"
        conform_suites=all
        [ "$gate_part" = conform ] || conform_suites="${gate_part#conform:}"
        conform_shard=
        case "$conform_suites" in
            *@*) conform_shard="--shard ${conform_suites#*@}"; conform_suites="${conform_suites%@*}" ;;
        esac
        conform_suites="${conform_suites//[+,]/ }"
        case " $conform_suites " in
            *" all "* | *" conformance "*)
                step "script unit tests"
                bare_home uv run --quiet python -m unittest "${script_tests[@]}" \
                    || fail "script unit test failure; rerun: uv run python -m unittest scripts/test_*.py"
                ;;
        esac
        step "cpl-conform ratchet ($conform_suites --check against research/scoreboard.json)"
        cargo build -q -p cpl-cli -p cpl-conform
        # Seeded problems (problem-packages/R9) read their cases from target/corpus-tests/.
        uv run --quiet python scripts/corpus-tests.py make --seeded >/dev/null \
            || fail "corpus-tests.py make --seeded failed; rerun it to see which problem"
        # shellcheck disable=SC2086
        # CI keeps main's last green run in $CPL_CONFORM_PREV, so a case newer than
        # research/scoreboard.json still can't regress unseen (ci.yml).
        "$target/debug/cpl-conform" $conform_suites $conform_shard --check \
            ${CPL_CONFORM_PREV:+--prev "$CPL_CONFORM_PREV" --save "$CPL_CONFORM_PREV.new"} \
            || fail "cpl-conform ratchet failure; rerun: cargo run -p cpl-conform -- $conform_suites $conform_shard --check"
        step_end
        printf '\ncheck: conformance part passed in %ss\n' "$(( $(date +%s) - gate_t0 ))"
        exit 0
        ;;
    tests:doc)
        [ "$mode" = fast ] || fail "CPL_GATE_PART=$gate_part needs --fast"
        step "cargo test --workspace --doc"
        cargo test -q --workspace --doc
        step_end
        printf '\ncheck: doctests passed in %ss\n' "$(( $(date +%s) - gate_t0 ))"
        exit 0
        ;;
    tests:*)
        [ "$mode" = fast ] || fail "CPL_GATE_PART=$gate_part needs --fast"
        part="${gate_part#tests:}"
        # CI compiles the test binaries once (cargo nextest archive) and every
        # partition runs them from CPL_NEXTEST_ARCHIVE; unset, nextest builds.
        from=(--workspace)
        [ -z "${CPL_NEXTEST_ARCHIVE:-}" ] \
            || from=(--archive-file "$CPL_NEXTEST_ARCHIVE" --workspace-remap . --extract-to . --extract-overwrite)
        step "cargo nextest run --partition $part (without the --full tests)"
        # A test that passes only on its retry fails the run as flaky, and
        # flaky.py names it (ci/R35): flakes surface instead of hiding.
        nextest_log="$(mktemp)"
        cargo nextest run "${from[@]}" --profile gate --no-fail-fast --retries 1 --flaky-result fail --partition "$part" -E "not ($slow_filter)" \
            2>&1 | tee "$nextest_log" || tests_status=$?
        uv run --quiet python scripts/flaky.py "$nextest_log"
        [ "${tests_status:-0}" = 0 ] || fail "a test failed or was flaky; nextest's summary above names it, rerun: cargo nextest run -E 'test(<name>)'"
        step_end
        printf '\ncheck: test partition %s passed in %ss\n' "$part" "$(( $(date +%s) - gate_t0 ))"
        exit 0
        ;;
    fmt)
        [ "$mode" = fast ] || fail "CPL_GATE_PART=$gate_part needs --fast"
        step "cpl fmt is idempotent and keeps meaning on every .cp file"
        if cache_hit fmt_corpus; then
            echo "cached (tree unchanged outside its excludes)"
        else
            cargo build -q -p cpl-cli
            scripts/fmt_check.py --cpl "$cpl" \
                || fail "cpl fmt is unstable or changes meaning; rerun: scripts/fmt_check.py"
            cache_save fmt_corpus
        fi
        step_end
        printf '\ncheck: fmt part passed in %ss\n' "$(( $(date +%s) - gate_t0 ))"
        exit 0
        ;;
    *) fail "unknown CPL_GATE_PART \`$gate_part\` (checks, fmt, conform or tests:<partition>)" ;;
esac

if [ "$mode" = land ]; then
    land_base="${CHECK_BASE:-$(git rev-parse -q --verify origin/main || true)}"
    sel_all=1
    sel_reason="no base to diff the train against"
    if [ -n "$land_base" ]; then
        while IFS='=' read -r key value; do
            printf -v "sel_$key" '%s' "$value"
        done < <(uv run --quiet python scripts/affected.py --base "$land_base")
    fi
    if [ "$sel_all" = 1 ]; then
        printf '\ncheck: %s is not mapped by scripts/affected.py; running --fast\n' "$sel_reason"
        mode=fast
    fi
fi
if [ "$mode" = commit ]; then
    while IFS='=' read -r key value; do
        printf -v "sel_$key" '%s' "$value"
    done < <(uv run --quiet python scripts/affected.py)
    if [ "$sel_all" = 1 ]; then
        printf '\ncheck: %s is not mapped by scripts/affected.py; running --fast\n' "$sel_reason"
        mode=fast
    fi
fi
if [ "$mode" = commit ]; then
    step "tech-debt ratchet (scripts/debt-ratchet.py --check)"
    uv run --quiet python scripts/debt-ratchet.py --check \
        || fail "tech-debt ratchet failed; see scripts/debt-ratchet.json"
    step "use cases: unique IDs, every field, evidence that exists (scripts/check-usecases.py)"
    uv run --quiet python scripts/check-usecases.py --check >/dev/null || fail "use-case catalogue check failed; see above"
    step "generated files current: skill pages, COVERAGE.md, gap lists (scripts/gen.py --check)"
    # CI's regen job regenerates stale files on main, so a commit isn't
    # blocked by staleness it may not have caused.
    uv run --quiet python scripts/gen.py --check \
        || echo "WARNING: generated files are stale; run: just gen (CI's regen job also fixes them)" >&2
    if [ -n "${sel_slices:-}" ]; then
        step "touched-path slices: $sel_slices (scripts/affected.py, each under 20 s)"
        uv run --quiet python scripts/affected.py --run-slices $sel_slices \
            || fail "a touched-path slice failed or took over 20 s; the lines above name it, rerun: uv run scripts/affected.py --run-slices $sel_slices"
    fi
    if [ -n "$sel_packages" ]; then
        read -ra pkgs <<<"$sel_packages"
        step "cargo fmt --check of the touched crates"
        cargo fmt --check $(printf -- '-p %s ' "${pkgs[@]}") \
            || fail "Rust files are not formatted; run: cargo fmt $(printf -- '-p %s ' "${pkgs[@]}")"
        step "cargo clippy of the touched crates and nextest of the crates they reach"
        cargo clippy -q $(printf -- '-p %s ' "${pkgs[@]}") --all-targets -- -D warnings
        wasm_deps=" $(cargo tree -q -p cpl-wasm --target wasm32-unknown-unknown -e normal --prefix none \
            | cut -d' ' -f1 | sort -u | tr '\n' ' ')"
        for p in "${pkgs[@]}"; do
            if [[ "$wasm_deps" == *" $p "* ]]; then
                wasm_check
                break
            fi
        done
        cargo nextest run --workspace --profile commit --no-fail-fast \
            -E "($sel_filter) & not ($slow_filter)${sel_slice_filter:+ & not ($sel_slice_filter)}"
    fi
    if [ -n "${sel_data_filter:-}" ]; then
        step "tests that read the touched data files: $sel_data_filter"
        cargo nextest run --workspace --no-fail-fast -E "($sel_data_filter) & not ($slow_filter)"
    fi
    if [ -n "${sel_check_files:-}" ]; then
        step "cpl check of the touched corpus programs"
        cargo build -q -p cpl-cli
        read -ra files <<<"$sel_check_files"
        for f in "${files[@]}"; do
            [ -f "$f" ] || continue
            "$cpl" check "$f" || fail "$f has check errors; cpl check's message above says how to fix it"
        done
    fi
    if [ "${sel_audit:-0}" = 1 ]; then
        step "corpus package audit, no compiles (corpus/audit.py --check --no-run)"
        uv run --quiet python corpus/audit.py --check --no-run \
            || fail "a corpus package breaks handbook/guidelines/problem-packages-checklist.md; the finding above names the rule"
        step "staged ports: generators and interpreter vs C++ (corpus/generators.py, corpus/differential.py)"
        cargo build -q -p cpl-cli
        uv run --quiet python corpus/generators.py --check --changed staged --cpl "$cpl" \
            || fail "a staged port's generator or answer disagrees with its accepted C++; the finding above names the port and test"
        uv run --quiet python corpus/differential.py --check --changed staged --cpl "$cpl" \
            || fail "a staged port's cpl run disagrees with its C++; the finding above names the port and test"
        mapfile -t dreams < <(git diff --cached --name-only --diff-filter=AMR -- 'corpus/problems/*/dream.cp' \
            | sed 's|^corpus/problems/||; s|/dream\.cp$||')
        if [ "${#dreams[@]}" -gt 0 ]; then
            step "staged dreams agree with the accepted C++ (scripts/corpus.py verify --check)"
            uv run --quiet python scripts/corpus.py verify --check --cpl "$cpl" "${dreams[@]}" \
                || fail "a staged dream.cp disagrees with its accepted C++ or meta.json lacks its record; run scripts/corpus.py verify <id> and stage meta.json"
        fi
    fi
    if [ "${sel_idioms_catalog:-0}" = 1 ]; then
        step "lint catalog names only real APIs (scripts/idioms-catalog.py --check)"
        uv run --quiet python scripts/idioms-catalog.py --check \
            || fail "a spec/idioms/lints.md row names an API std doesn't define; the finding above names the lint and the name"
    fi
    if [ -n "$sel_script_tests" ]; then
        step "script unit tests: $sel_script_tests"
        read -ra tests <<<"$sel_script_tests"
        bare_home uv run --quiet python -m unittest "${tests[@]}"
    fi
    if [ -n "${sel_conform_cases:-}" ]; then
        step "staged conformance cases (scripts/conform_staged.py)"
        cargo build -q -p cpl-cli -p cpl-conform
        read -ra cases <<<"$sel_conform_cases"
        uv run --quiet python scripts/conform_staged.py "$target/debug/cpl-conform" "${cases[@]}" \
            || fail "a staged conformance case fails; cpl-conform's message above says how to fix it"
    fi
    if [ "$sel_site" = 1 ]; then
        step "site snippet gate (node web/scripts/gate.mjs)"
        node web/scripts/gate.mjs
    fi
    step_end
    printf '\nstep times (s):\n'
    printf '%s\n' "${step_times[@]}" | sort -rn | awk -F'\t' '$1 > 0 { printf "%5d  %s\n", $1, $2 }'
    printf '\ncheck: commit-tier checks passed in %ss; CI runs --fast on every main tip\n' \
        "$(( $(date +%s) - gate_t0 ))"
    exit 0
fi

step "cargo fmt --all --check"
cargo fmt --all --check || fail "Rust files are not formatted; run: cargo fmt --all"

step "cargo clippy --workspace --all-targets -- -D warnings"
if cache_hit clippy; then
    echo "cached (tree unchanged outside its excludes)"
else
    cargo clippy --workspace --all-targets -- -D warnings
    cache_save clippy
fi

wasm_check

step "cargo build -p cpl-cli -p cpl-conform (the binaries the checks below run)"
cargo build -q -p cpl-cli -p cpl-conform
# Logs of the steps that run beside the workspace tests.
lib_logs="$(mktemp -d)"
site_gate_pid=""
# CI's checks part runs no tests, so the snippet gate overlaps fmt_check
# instead of running after it.
if [ "$gate_part" = checks ] && ! cache_hit site_gate; then
    need node "the site's snippet gate runs on it"
    CPL="$cpl" node web/scripts/gate.mjs >"$lib_logs/site_gate.log" 2>&1 &
    site_gate_pid=$!
fi
script_test_pid=""
recovery_pid=""
scoreboard_pid=""
fmt_pid=""
# fmt(fmt(x)) == fmt(x) on every tracked .cp file, and formatting changes
# no file's check result or interpreter output.
fmt_cached=0
if [ "$gate_part" = checks ]; then
    fmt_cached=2
elif cache_hit fmt_corpus; then
    fmt_cached=1
else
    scripts/fmt_check.py --cpl "$cpl" >"$lib_logs/fmt_corpus.log" 2>&1 &
    fmt_pid=$!
fi
scoreboard_cached=0
if [ "$gate_part" = checks ]; then
    scoreboard_cached=2
elif cache_hit scoreboard_fast; then
    scoreboard_cached=1
else
    # cpl-conform is already built above; overlap it and script unit tests with nextest.
    bare_home python3 -m unittest "${script_tests[@]}" >"$lib_logs/script_tests.log" 2>&1 &
    script_test_pid=$!
    "$target/debug/cpl-conform" all --check >"$lib_logs/scoreboard.log" 2>&1 &
    scoreboard_pid=$!
fi
cleanup() {
    kill ${script_test_pid:+"$script_test_pid"} ${recovery_pid:+"$recovery_pid"} ${scoreboard_pid:+"$scoreboard_pid"} ${fmt_pid:+"$fmt_pid"} ${site_gate_pid:+"$site_gate_pid"} 2>/dev/null || true
    rm -rf "$lib_logs" "$manifest" "$manifest.reads" ${tmp:+"$tmp"}
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
if [ "$gate_part" = checks ]; then
    step "cargo nextest + doctests"
    echo "skipped: the CI test partitions run them (CPL_GATE_PART=checks)"
elif cache_hit nextest_doc; then
    step "cargo nextest + doctests"
    echo "cached (tree unchanged outside its excludes)"
elif [ "$mode" = land ]; then
    # Not cached: a partial run must not vouch for the whole tree.
    step "cargo nextest of the crates the train reaches: ${sel_packages:-none}"
    if [ -n "$sel_packages" ]; then
        read -ra pkgs <<<"$sel_packages"
        cargo nextest run --workspace --no-fail-fast -E "($sel_filter) & not ($slow_filter)"
        step "cargo test --doc of the touched crates"
        cargo test -q $(printf -- '-p %s ' "${pkgs[@]}") --doc
    fi
else
    if cargo nextest --version >/dev/null 2>&1; then
        step "cargo nextest run --workspace (without the --full tests)"
        cargo nextest run --workspace --no-fail-fast -E "not ($slow_filter)"
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

step "corpus ports pass cpl check (scripts/port-check.sh, hard gate)"
scripts/port-check.sh "$cpl" || fail "a corpus port fails cpl check; fix it"

step "std/prelude.cp: name count"
prelude=$(sed 's://.*::' std/prelude.cp | tr -d '\n' | grep -o '{[^}]*}' | tr ',' '\n' | grep -c '[A-Za-z_]')
echo "prelude: $prelude names"

step "cpl fmt is idempotent and keeps meaning on every .cp file"
if [ "$fmt_cached" -eq 2 ]; then
    echo "skipped: the CI fmt part runs it (CPL_GATE_PART=checks)"
elif [ "$fmt_cached" -eq 1 ]; then
    echo "cached (tree unchanged outside its excludes)"
else
    fmt_status=0
    wait "$fmt_pid" || fmt_status=1
    cat "$lib_logs/fmt_corpus.log"
    fmt_pid=""
    [ "$fmt_status" -eq 0 ] || fail "cpl fmt is unstable or changes meaning; rerun: scripts/fmt_check.py"
    cache_save fmt_corpus
fi

# Tech-debt ratchet: .rs file-size cap allowlist (shrink-only) + stdlib todo / #[ignore] / pending counts.
step "tech-debt ratchet (scripts/debt-ratchet.py --check)"
python3 scripts/debt-ratchet.py --check \
    || fail "tech-debt ratchet failed; see scripts/debt-ratchet.json"

step "handbook reads for an outsider (scripts/check-handbook.py)"
uv run --quiet python scripts/check-handbook.py || fail "handbook doc leans on shared context; see above"
step "use cases: unique IDs, every field, evidence that exists (scripts/check-usecases.py)"
uv run --quiet python scripts/check-usecases.py --check >/dev/null || fail "use-case catalogue check failed; see above"

# On a main push CI sets CPL_DERIVED_HEALED=1: the regen job in the same run
# commits these files (and goes red if it can't), so staleness only warns.
derived() {
    if [ "${CPL_DERIVED_HEALED:-}" = 1 ]; then
        echo "warning: $1 (the regen job commits the fix)" >&2
    else
        fail "$1"
    fi
}
step "bundled skill (scripts/skill-gen.py --check)"
uv run python scripts/skill-gen.py --check --cpl "$cpl" \
    || derived "bundled skill check failed; rerun: uv run python scripts/skill-gen.py, then with --check"
step "skill prose uses current spellings (scripts/skill-prose.py)"
uv run --quiet python scripts/skill-prose.py --cpl "$cpl" \
    || fail "inline code in the skill's prose uses a removed spelling; the finding above names the span and its current form"

step "emitted C++ fits 32 KB and every bench program builds (bench/sizes.py --check)"
sizes_status=0
uv run --quiet python bench/sizes.py --check --cpl "$cpl" >"$target/sizes.log" 2>&1 || sizes_status=1
grep -v ': warning: ' "$target/sizes.log"
[ "$sizes_status" -eq 0 ] \
    || fail "a program is over the 32 KB bar or a bench program stopped building; see above, then bench/oversize.txt or bench/unbuilt.txt"

step "conformance coverage: COVERAGE.md current, no new rule gaps"
uv run scripts/conformance-coverage.py --check ||
    derived "rule coverage gate failed; see above, then rerun: uv run scripts/conformance-coverage.py"

step "conformance gaps: SUBSET.md generated, no gap past 48 h (scripts/spec-gate.py subset, gap-age)"
uv run --quiet python scripts/spec-gate.py subset || fail "spec/SUBSET.md is stale; run: uv run scripts/spec-gate.py subset --write"
uv run --quiet python scripts/spec-gate.py gap-age --staged || fail "a conformance gap is past 48 h or re-dated; make the case pass, then rerun: uv run scripts/spec-gate.py gap-age --staged"

step "conformance matrix: generated cases current, every cell a case or n/a"
uv run scripts/conformance-matrix.py --check || fail "matrix gate failed; see above"
uv run scripts/conformance-widths.py --check || fail "spec/conformance/widths/ is stale; run scripts/conformance-widths.py"
uv run scripts/conformance-ownership.py --check || fail "spec/conformance/ownership/ is stale; run scripts/conformance-ownership.py"
uv run scripts/spec-xref.py --check || fail "a rule id names no rule in spec/core; see above"
uv run scripts/spec-changelog.py || fail "a spelling the spec removes has no changelog line; see above"
uv run scripts/spec-tool-rules.py --max 0 || fail "a tool feature in spec/tools/ has no rule id; see above"
uv run scripts/spec-std-signatures.py --check || fail "a spec/lib chapter and std disagree on a return type; see above"
uv run scripts/spec-decisions-explained.py --max 0 || fail "a decision cited in spec/ has no handbook explanation; see above"

step "conformance traits: every claimed trait method has a case or a gap line"
uv run scripts/conformance-traits.py --check || fail "trait gate failed; see above"

step "conformance sigs: named args in spec/core and spec/lib exist in std"
uv run scripts/conformance-sigs.py --check || fail "sig gate failed; see above"

step "conformance prelude: table fns of spec/lib chapters are prelude names (R11.3.2)"
uv run scripts/conformance-prelude.py --check || fail "prelude gate failed; see above"

step "std containers print, walk and have is_empty (scripts/std-trait-matrix.py)"
uv run --quiet python scripts/std-trait-matrix.py --max 76 \
    || fail "a std container lacks a common trait; see above"

step "std docs follow the std-docs checklist (scripts/std-docs-lint.py)"
uv run --quiet python scripts/std-docs-lint.py --max 0 \
    || fail "a std doc breaks a std-docs rule; see above"

step "std signatures follow the std-api checklist (scripts/std-api-lint.py)"
uv run --quiet python scripts/std-api-lint.py --max 1 \
    || fail "a std signature breaks a std-api rule; see above"

step "std is lint-clean, shrink-only (scripts/std-lint-ratchet.py)"
uv run --quiet python scripts/std-lint-ratchet.py --cpl "$cpl" \
    || fail "std gained a lint hit; fix it or mark it with lint(...: allow)"

step "prelude exports what its fns return (scripts/prelude-signatures.py)"
uv run --quiet python scripts/prelude-signatures.py \
    || fail "a prelude fn returns a std type the prelude does not export; see above"

step "std trap examples trap (scripts/std-trap-examples.py)"
uv run --quiet python scripts/std-trap-examples.py --cpl "$cpl" \
    || fail "a std \`trap:\` doc example did not trap; see above"

# Taste reviews: always run (fast, checks working-tree/staged ports).
step "taste reviews (scripts/review-check.py --check)"
python3 scripts/review-check.py --check \
    || fail "taste review gate failed; see research/reviews/README.md"

step "site snippet gate (node web/scripts/gate.mjs)"
if [ -n "$site_gate_pid" ]; then
    site_gate_status=0
    wait "$site_gate_pid" || site_gate_status=1
    cat "$lib_logs/site_gate.log"
    site_gate_pid=""
    [ "$site_gate_status" -eq 0 ] || fail "site snippet gate failed; rerun: CPL=$cpl node web/scripts/gate.mjs"
    cache_save site_gate
elif cache_hit site_gate; then
    echo "cached (tree unchanged outside its excludes)"
elif [ "$mode" = land ] && [ "$sel_site" != 1 ] && [ -z "$sel_packages" ]; then
    echo "skipped: the train touches no site, docs or crate path"
else
    need node "the site's snippet gate runs on it"
    CPL="$cpl" node web/scripts/gate.mjs || fail "site snippet gate failed; rerun: CPL=$cpl node web/scripts/gate.mjs"
    cache_save site_gate
fi

step "trace viewer tests (node tools/trace/*.test.cjs)"
need node "the trace viewer's tests run on it"
for t in tools/trace/trace.test.cjs tools/trace/state.test.cjs; do
    node --test-reporter=dot "$t" || fail "trace viewer tests failed; rerun: node $t"
done

step "cpl-conform ratchet (all --check against research/scoreboard.json)"
if [ "$scoreboard_cached" -eq 2 ]; then
    echo "skipped: the CI conform part runs it and the script tests (CPL_GATE_PART=checks)"
elif [ "$scoreboard_cached" -eq 1 ]; then
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

    step "random programs: interpreter, C++ and sanitized C++ match an independent model (conformance/R6)"
    uv run scripts/gen-programs.py -n 100 --seed 1 --jobs "$(nproc)" --out "$tmp/gen" --avoid rev-pieces \
        --cxxflags '-O1 -g -fsanitize=address,undefined -D_GLIBCXX_DEBUG'
    uv run scripts/conformance-metamorphic.py --check || fail "a meaning-preserving edit changed a case's output; see above"
    uv run scripts/grammar-fuzz.py --n 300 --check --max-syntax 231 --show 0 || fail "a program derived from the grammar crashed cpl, or more derived programs are syntax errors; see above"
    uv run scripts/conformance-speed.py || fail "the conformance and spec-block suites went over their CPU budget; see above"
    uv run scripts/explain-copies.py --check --max 1 || fail "cpl explain claims more copies than the C++ makes in more cases; see above"
    uv run scripts/tool-vocab.py || fail "help, explain or hover uses a spelling spec/GLOSSARY.md bans; see above"
    uv run scripts/conformance-flakes.py --runs 2 --check || fail "a conformance case changed verdict between runs; see above"
    step "conformance gaps: every failing case has a // gap: line, every gap case fails (scripts/spec-gate.py gaps)"
    uv run --quiet python scripts/spec-gate.py gaps --conform "$target/debug/cpl-conform" \
        || fail "a failing case has no // gap: line or a gap case passes; see above"

    step "cargo test: every port compiled with every judge profile's flags"
    cargo test -p cpl-cli --test judges -- --ignored --exact every_port_compiles_with_every_judges_flags

    step "cargo test: budget, then the front-end latency, each on its own"
    cargo test -p cpl-cli --test budget -- --test-threads=1
    cargo test -p cpl-lsp --test latency -- --exact front_end_check_is_under_50_ms

    step "wasm build and crates/cpl-wasm/tests/smoke.mjs"
    scripts/build-wasm.sh --smoke --out "$tmp/wasm"

    step "scripts/site-check.sh: site build, cpl.wasm size cap, browser smoke"
    CPL="$cpl" scripts/site-check.sh --no-cache

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
if [ "$mode" = fast ] || [ "$mode" = land ]; then
    gate_time_file="$target/gate-fast-time.json"
    mkdir -p "$(dirname "$gate_time_file")"
    printf '{"mode":"%s","elapsed_s":%d,"budget_s":180}\n' "$mode" "$gate_elapsed" >"$gate_time_file"
    printf '\ncheck: all %s checks passed in %ss (budget <= 180s)\n' "$mode" "$gate_elapsed"
    # Wall time depends on the machine and on how warm the build is, so an
    # overrun warns and stays recorded in the scoreboard instead of failing.
    [ "$gate_elapsed" -le 180 ] ||
        printf 'check: WARNING: fast gate took %ss, over the 180s (3 min) budget\n' "$gate_elapsed"
else
    printf '\ncheck: all %s checks passed in %ss\n' "$mode" "$gate_elapsed"
fi
