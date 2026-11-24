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
    uv run --quiet python scripts/check_cache.py manifest "$manifest" || use_cache=0
fi
cache_hit() {
    [ "$use_cache" -eq 1 ] || return 1
    uv run --quiet python scripts/check_cache.py check "$cache_dir" "$1" "$manifest"
}
cache_save() {
    [ "$use_cache" -eq 1 ] || return 0
    uv run --quiet python scripts/check_cache.py save "$cache_dir" "$1" "$manifest"
}

markers=$(git grep -nE '^(<{7}|>{7}|\|{7})( |$)' -- ':!*.png' ':!*.gz' || [ $? = 1 ])
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
    [ -z "$added" ] || fail "new files in a problem's tests/ folder; list the case in the problem's tests.toml and let gen.py make it (problem-packages/R9):
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
        lines="$(git diff "$range" --diff-filter=ACMR -U0 --no-color | grep -E '^\+[^+]' || [ $? = 1 ])"
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
            | grep -vxE -f <(grep -v '^#' "$top/scripts/public-hosts.txt") || [ $? = 1 ])"
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

# The tiers' test filters (slow_filter, wall_filter) and CPL_GATE_PART's parse
# (gate_kind, conform_suites, conform_shard, part) live in scripts/ci.py, tested.
gate_part="${CPL_GATE_PART:-}"
ci_plan="$(uv run --quiet python scripts/ci.py plan --mode "$mode" --gate-part "$gate_part")" || exit 1
eval "$ci_plan"
# Tests that mix a timing budget into a correctness check skip the budget.
[ "$mode" = full ] || [ "$mode" = timing ] || export CPL_NO_WALL_CLOCK=1
if [ "$mode" = timing ]; then
    step "wall-clock budgets (reported, never blocking)"
    cargo nextest run --workspace --no-fail-fast \
        -E "$wall_filter or test(=every_corpus_program_and_conformance_case_is_served) or test(=every_port_compiles_with_every_judges_flags)"
    exit
fi

# CPL_GATE_PART's parts are listed in scripts/ci.py.
# Every tracked script test; unrun_script_tests fails on one outside these globs.
mapfile -t script_tests < <(git ls-files 'scripts/test_*.py' 'bench/test_*.py' 'tools/*/test_*.py')
case "$gate_kind" in
    checks) ;;
    conform)
        case " $conform_suites " in
            *" all "* | *" conformance "*)
                step "script unit tests"
                bare_home uv run --quiet python -m unittest "${script_tests[@]}" \
                    || fail "script unit test failure; rerun: uv run python -m unittest scripts/test_*.py"
                ;;
        esac
        step "cpl-conform ratchet ($conform_suites --check against research/scoreboard.json)"
        cargo build -q -p cpl-cli -p cpl-conform
        # Seeded problems (problem-packages/R9) read their cases from target/corpus-tests/;
        # only the corpus suite reads them.
        case " $conform_suites " in
            *" all "* | *" corpus "*)
                uv run --quiet python scripts/corpus.py make >/dev/null \
                    || fail "scripts/corpus.py make failed; rerun it to see which package"
                ;;
        esac
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
    doc)
        step "cargo test --workspace --doc"
        cargo test -q --workspace --doc
        step_end
        printf '\ncheck: doctests passed in %ss\n' "$(( $(date +%s) - gate_t0 ))"
        exit 0
        ;;
    tests)
        # CI compiles the test binaries once (cargo nextest archive) and every
        # partition runs them from CPL_NEXTEST_ARCHIVE; unset, nextest builds.
        from=(--workspace)
        [ -z "${CPL_NEXTEST_ARCHIVE:-}" ] \
            || from=(--archive-file "$CPL_NEXTEST_ARCHIVE" --workspace-remap . --extract-to . --extract-overwrite)
        step "cargo nextest run --partition $part (without the --full tests)"
        # No retries: a flaky test fails on its first run (ci/R35).
        cargo nextest run "${from[@]}" --profile gate --no-fail-fast --partition "$part" -E "not ($slow_filter)" \
            || tests_status=$?
        [ "${tests_status:-0}" = 0 ] || fail "a test failed; nextest's summary above names it, rerun: cargo nextest run -E 'test(<name>)'"
        step_end
        printf '\ncheck: test partition %s passed in %ss\n' "$part" "$(( $(date +%s) - gate_t0 ))"
        exit 0
        ;;
    fmt)
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
esac

# --land diffs the train against origin/main, --commit the staged files.
if [ "$mode" = land ] || [ "$mode" = commit ]; then
    sel_all=1
    sel_reason="no base to diff the train against"
    affected=(scripts/affected.py)
    if [ "$mode" = land ]; then
        land_base="${CHECK_BASE:-$(git rev-parse -q --verify origin/main || [ $? = 1 ])}"
        [ -z "$land_base" ] || affected+=(--base "$land_base")
    fi
    if [ "$mode" = commit ] || [ -n "$land_base" ]; then
        while IFS='=' read -r key value; do
            printf -v "sel_$key" '%s' "$value"
        done < <(uv run --quiet python "${affected[@]}")
    fi
    if [ "$sel_all" = 1 ]; then
        printf '\ncheck: %s is not mapped by scripts/affected.py; running --fast\n' "$sel_reason"
        mode=fast
    fi
fi
if [ "$mode" = commit ]; then
    step "checking scripts (scripts/ci.py steps commit)"
    uv run --quiet python scripts/ci.py steps commit --cpl "$cpl" || exit 1
    step "generated files current: skill pages, COVERAGE.md, gap lists (scripts/gen.py --check)"
    # CI's regen job regenerates stale files on main, so a commit isn't
    # blocked by staleness it may not have caused.
    # Only the summary line: the per-file lines are CI's regen job's to act on.
    if ! gen_out=$(uv run --quiet python scripts/gen.py --check 2>&1); then
        printf '%s\n' "$gen_out" | tail -n 1
        echo "note: not blocking; CI's regen job rewrites these on main (details: uv run scripts/gen.py --check)"
    fi
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
        step "staged corpus packages pass every mode (scripts/corpus.py verify --changed staged)"
        cargo build -q -p cpl-cli
        uv run --quiet python scripts/corpus.py make --changed staged >/dev/null \
            || fail "scripts/corpus.py make --changed staged failed; rerun it to see which package"
        uv run --quiet python scripts/corpus.py verify --changed staged --skip dream --cpl "$cpl" \
            || fail "a staged corpus package fails scripts/corpus.py verify; the finding above names the mode and the package"
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
    if [ -n "${sel_steps:-}" ]; then
        step "touched checking scripts (scripts/ci.py steps touched $sel_steps)"
        read -ra touched <<<"$sel_steps"
        uv run --quiet python scripts/ci.py steps touched --cpl "$cpl" "${touched[@]}" || exit 1
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
# The workspace tests, the cached stages that overlap them and the checking
# scripts, in order (scripts/ci.py tail, tested in scripts/test_ci.py).
step_end
tail_times="$(mktemp)"
tail_status=0
tail_manifest=""
[ "$use_cache" -eq 0 ] || tail_manifest="$manifest"
uv run --quiet python scripts/ci.py tail --mode "$mode" --gate-part "$gate_part" --cpl "$cpl" \
    --cache-dir "$cache_dir" --manifest "$tail_manifest" --sel-packages "${sel_packages:-}" \
    --sel-filter "${sel_filter:-}" --sel-site "${sel_site:-}" --times "$tail_times" || tail_status=$?
mapfile -t -O "${#step_times[@]}" step_times <"$tail_times"
rm -f "$tail_times" "$manifest" "$manifest.reads"
[ "$tail_status" -eq 0 ] || exit 1
step_name="" step_t0=$(date +%s)

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
