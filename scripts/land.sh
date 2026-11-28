#!/usr/bin/env bash
# Lands HEAD on main: rebase onto the remote's main, push, and on a rejected
# push (someone landed first) rebase and push again. Nothing is gated here:
# the commit hook ran the commit tier, CI runs the full gate on every main
# tip, and the infra sheriff reverts or fixes a red one. Every commit to land
# needs a `Domain: <d>` trailer. Before the first push it runs the checks
# behind most red mains, in a few seconds: conflict markers in the changed
# files, the debt ratchet, and `cargo fmt --check` and clippy on the touched
# crates. Generated files are CI's regen job's: it rewrites and commits them.
# It also says when AGENTS.md, handbook/ or plan/product/ changed since this worktree's
# last land, so product direction reaches every lane.
# Each landing appends one JSON line (domain, seconds, tries) to
# .git/cpl-land.log; scripts/land_stats.py reads it.
#
# Usage: scripts/land.sh [remote]   (default origin; CPL_LAND_TRIES, default 5; CPL_LAND_FULL=1)
set -euo pipefail
# The copy running now: a pull that changes scripts/land.sh lands with the new one.
self_blob=$(git hash-object "${BASH_SOURCE[0]}")
land_args=("$@")
cd "$(git rev-parse --show-toplevel)"
# A rebase merges the generated ratchet and END-STATE.md itself (.gitattributes).
[ ! -x scripts/merge-drivers.sh ] || scripts/merge-drivers.sh
remote="${1:-origin}"
tries="${CPL_LAND_TRIES:-5}"
t0=${CPL_LAND_T0:-$(date +%s)}
# Seconds per phase, summed across pulls and re-execs, for the timing line: where a slow
# land went (rebase, cpl build, ratchet, lock wait), not only its total.
declare -A phase_s=()
for kv in ${CPL_LAND_PHASES:-}; do phase_s[${kv%%=*}]=${kv#*=}; done
phase_t=${CPL_LAND_PHASE_T:-$t0}
phase() { local now; now=$(date +%s); phase_s[$1]=$(( ${phase_s[$1]:-0} + now - phase_t )); phase_t=$now; }
phases() { local k out=""; for k in "${!phase_s[@]}"; do out="$out $k=${phase_s[$k]}"; done; printf %s "${out# }"; }
# By default a land is fast: rebase, conflict markers, fmt, the case-list rule and the
# ratchet (check, then lower what improved), push,
# in under a minute. A lane ran its focused tests and `debt-ratchet --check` before
# it; CI gates every main tip, culprit.yml names the commit that turned it red, and
# the infra lane runs the suites. CPL_LAND_FULL=1 also reruns the gates, the slices,
# the conformance suite, the ratchet (lowering improved counters), wasm and clippy at
# the rebased tip: 116-697 s a land, the most of every lane's process time.
fast=1
[ -z "${CPL_LAND_FULL:-}" ] || fast=
git fetch -q "$remote" main
# A checkout behind main lands with main's land.sh, so a fix to landing reaches
# every lane at once instead of after its next rebase.
if [ -z "${CPL_LAND_MAIN:-}" ] && git cat-file -e FETCH_HEAD:scripts/land.sh 2>/dev/null &&
    ! git show FETCH_HEAD:scripts/land.sh | cmp -s - scripts/land.sh; then
    # A copy per run: bash reads its script as it goes, so a copy shared by the
    # worktrees, rewritten by another land when main moved, broke this one mid-run.
    gd=$(git rev-parse --absolute-git-dir)
    find "$gd" -maxdepth 1 -name 'cpl-land-main.*' -mmin +1440 -exec rm -rf {} +
    dir=$(mktemp -d "$gd/cpl-land-main.XXXXXX")
    for f in land.sh retry.sh; do git show "FETCH_HEAD:scripts/$f" >"$dir/$f"; done
    CPL_LAND_MAIN=1 exec bash "$dir/land.sh" "$@"
fi
# The rows main's plan/signoffs.toml holds of one kind, one per line, formatted by `fmt`: the
# gates' one authority (audit #18 (2)), never a commit message's text.
signoff_rows() {
    local rows
    rows=$(git show "FETCH_HEAD:plan/signoffs.toml" 2>/dev/null) || return 0
    printf '%s\n' "$rows" | uv run --quiet python -c '
import sys, tomllib
for r in tomllib.loads(sys.stdin.read()).get(sys.argv[1], []):
    print(sys.argv[2].format(**r))' "$1" "$2"
}
# A pushed commit with no Lane: takes one only from a [[lane_for]] row.
granted=$(signoff_rows lane_for '{sha}')
lane_for() { for g in $granted; do case "$1" in "$g"*) return 0 ;; esac; done; return 1; }
for c in $(git rev-list "FETCH_HEAD..HEAD"); do
    git log -1 --format='%(trailers:key=Domain,valueonly)' "$c" | grep -q . ||
        { echo "land: $(git log -1 --format='%h %s' "$c") has no Domain: trailer" >&2; exit 1; }
    # The commit-msg hook adds Lane:, but a commit made without the hooks has none, and a
    # counter or verdict traced to it then reaches no owner.
    git log -1 --format='%(trailers:key=Lane,valueonly)' "$c" | grep -q . || lane_for "$c" ||
        { echo "land: $(git log -1 --format='%h %s' "$c") has no Lane: trailer (set CPL_LANE, or git commit --amend --trailer 'Lane: X')" >&2; exit 1; }
done
domain=$(git log -1 --format='%(trailers:key=Domain,valueonly,separator=)' HEAD)
# A conflict only in generated files (a gen.toml entry's, or pnpm-lock.yaml) is
# resolved by taking main's copy; CI's regen job regenerates it on the pushed
# tip. Any other conflict stops here: taking main's side of a hand-written file
# drops this commit's change to it.
ratchet=scripts/debt-ratchet.json index=corpus/index.json
rebasing() { [ -d "$(git rev-parse --git-path rebase-merge)" ] || [ -d "$(git rev-parse --git-path rebase-apply)" ]; }
# A rebase that worked can still leave the autostash's pop conflicted; say so
# rather than land around it.
stash_conflict() {
    rebasing && return 1
    left=$(git diff --name-only --diff-filter=U)
    [ -n "$left" ] || return 1
    echo "land: your uncommitted changes conflict with main in: $(echo $left); they are kept in" \
        "$(git stash list -1 --format=%gd); resolve the files, then \`git stash drop\`" >&2
}
# A rebase across a change to a counter's measuring code restates that count by
# the tip's rule, not the lower of two rules' numbers (scripts/ratchet_remeasure.py).
remeasure() {
    [ -f scripts/ratchet_remeasure.py ] && [ "$(git rev-parse HEAD)" != "$before" ] || return 0
    # The amend goes on this checkout's own commit, so with none there is
    # nothing to restate; a hand edit of the ratchet stays uncommitted.
    [ -n "$(git rev-list FETCH_HEAD..HEAD)" ] || return 0
    if ! git diff --quiet HEAD -- "$ratchet"; then
        echo "land: WARNING: $ratchet has uncommitted edits; not re-measuring it" >&2
        return 0
    fi
    uv run --quiet python scripts/ratchet_remeasure.py "$before" FETCH_HEAD || return 1
    git diff --quiet -- "$ratchet" || git commit -q --amend --no-edit --no-verify -- "$ratchet"
}
# plan/END-STATE.md is generated from inputs (the ratchet) that every lane's landing
# moves, and the ratchet fails a stale one: rewrite it on the rebased tree rather
# than make each lane rerun its land for a file nobody writes by hand.
regen_end_state() {
    [ -f scripts/end-state.py ] && [ -n "$(git rev-list FETCH_HEAD..HEAD)" ] || return 0
    uv run --quiet python scripts/end-state.py >/dev/null || { echo "land: could not regenerate plan/END-STATE.md" >&2; return 1; }
    git diff --quiet -- plan/END-STATE.md && return 0
    git commit -q --amend --no-edit --no-verify -- plan/END-STATE.md
    echo "land: regenerated plan/END-STATE.md on the rebased tree" >&2
}
pull() {
    before=$(git rev-parse HEAD)
    if git pull -q --rebase --autostash "$remote" main; then
        stash_conflict && return 1
        phase rebase
        remeasure && regen_end_state
        phase remeasure
        return
    fi
    while rebasing; do
        conflicts=$(git diff --name-only --diff-filter=U)
        # The corpus index sums every package's meta.json: rebuild it from the rebased tree.
        if printf '%s\n' "$conflicts" | grep -qx "$index"; then
            git checkout -q --ours -- "$index" && uv run --quiet python scripts/corpus.py build >/dev/null &&
                git add -- "$index" && git checkout -q -- corpus ||
                { echo "land: could not rebuild $index" >&2; return 1; }
            echo "land: rebuilt $index on main's packages and this commit's" >&2
            conflicts=$(git diff --name-only --diff-filter=U)
            if [ -z "$conflicts" ]; then
                GIT_EDITOR=true git rebase --continue >/dev/null 2>&1 || rebasing ||
                    { echo "land: git rebase --continue failed" >&2; return 1; }
                continue
            fi
        fi
        # Lanes tighten the ratchet all day; merge both sides' tightening.
        if printf '%s\n' "$conflicts" | grep -qx "$ratchet"; then
            uv run --quiet python scripts/ratchet_merge.py "$ratchet" && git add -- "$ratchet" ||
                { echo "land: could not merge $ratchet" >&2; return 1; }
            echo "land: merged main's and this commit's $ratchet" >&2
            conflicts=$(git diff --name-only --diff-filter=U)
            if [ -z "$conflicts" ]; then
                GIT_EDITOR=true git rebase --continue >/dev/null 2>&1 || rebasing ||
                    { echo "land: git rebase --continue failed" >&2; return 1; }
                continue
            fi
        fi
        hand=$(printf '%s\n' $conflicts | grep -vx 'pnpm-lock\.yaml' || :)
        if [ -z "$conflicts" ] || { [ -n "$hand" ] && ! uv run --quiet python scripts/gen.py --owns $hand >/dev/null; }; then
            echo "land: rebase conflict in $(echo $conflicts); resolve it and run land.sh again" >&2
            return 1
        fi
        git checkout -q --ours -- $conflicts
        git add -- $conflicts
        echo "land: took main's $(echo $conflicts); CI's regen job regenerates them" >&2
        # A failed continue that leaves a rebase going stopped on the next conflict.
        GIT_EDITOR=true git rebase --continue >/dev/null 2>&1 || rebasing ||
            { echo "land: git rebase --continue failed" >&2; return 1; }
    done
    remeasure && regen_end_state
}
# Where this commit's hook ran: main's tip before the pull, as far as HEAD has it.
checked=${CPL_LAND_CHECKED:-$(git merge-base HEAD FETCH_HEAD 2>/dev/null || git rev-parse FETCH_HEAD)}
# A running bash keeps the land.sh it started with (perf's land waited 320 s on the
# old lock after main fixed it), and reads on from a file the rebase rewrote: each
# pull that changed it lands with the new copy from here (Site A's 2807 s land kept
# a fail-open copy past its one re-exec). CPL_LAND_REEXEC=1 opts out.
reexec_if_land_changed() {
    [ -z "${CPL_LAND_REEXEC:-}" ] && [ -f scripts/land.sh ] || return 0
    [ "$(git hash-object scripts/land.sh)" != "$self_blob" ] || return 0
    echo "land: the pull changed scripts/land.sh; landing with the new copy" >&2
    # Set only once push() ran, after drop_lock is defined; the first pull holds no lock.
    [ -z "${lock_held:-}" ] || drop_lock
    [ -z "${queued:-}" ] || dequeue
    CPL_LAND_MAIN=1 CPL_LAND_T0=$t0 CPL_LAND_PHASES="$(phase rebase; phases)" CPL_LAND_PHASE_T=$(date +%s) CPL_LAND_CHECKED=$checked CPL_LAND_TICKET=${ticket:-} \
        CPL_LAND_RECHECKED=${rechecked:-} CPL_LAND_STEPS_RUNS=${steps_runs:-0} \
        exec bash scripts/land.sh ${land_args[@]+"${land_args[@]}"}
}
pull || exit 1
phase rebase
# The main this land rebased onto, pinned: FETCH_HEAD moves with any fetch in this
# worktree meanwhile, and diffing against a newer main reads its std commits as this
# change's (VM-expr's interp-only series ran the std signature callers).
main_base=$(git rev-parse FETCH_HEAD)
reexec_if_land_changed
# The hook ran the commit tier on the old base; a rebase past a change to the
# gates themselves (scripts/, the hooks, gen.toml, the justfile) reruns it on the
# rebased tip, staged against main, since passing the old gates says nothing about
# the new ones (5 fixtures passed on an old base after cafd6ffbf's check landed).
gate_paths="scripts .githooks gen.toml justfile"
# Once per land, across re-execs: 4a53bbd4's land reran the commit tier after each pull
# that changed land.sh, and main changed it every few minutes (20+ min, no lock wait).
rechecked=${CPL_LAND_RECHECKED:-}
recheck() {
    [ -z "$fast" ] || return 0
    moved=$(git diff --name-only "$checked" "$main_base" -- $gate_paths)
    checked=$main_base
    [ -n "$moved" ] && [ -x .githooks/pre-commit ] || return 0
    # Land's own scripts, the script tests and docs judge no staged change: perf B's
    # 42-min land reran the ~9 min tier for land.sh, new_reds.py and test_ci.py.
    live=$(printf '%s\n' $moved | grep -Ev '^scripts/(land\.sh|new_reds\.py|test_[^/]*\.py|[^/]*\.md)$' || :)
    [ -n "$live" ] || { echo "land: main changed only land's own scripts and tests ($(echo $moved)); no rerun" >&2; return 0; }
    # Only step scripts and their data (debt-ratchet.json): rerun just the steps, not the tier.
    only=
    known=
    [ -n "$(printf '%s\n' $live | grep -Ev '^scripts/[^/]*\.(py|json)$')" ] ||
        known=$(uv run --quiet python -c 'import sys; sys.path.insert(0, "scripts"); import ci; print("\n".join(sorted(ci.step_scripts())))' 2>/dev/null) || known=
    if [ -n "$known" ] && [ -z "$(printf '%s\n' $live | sed 's/\.json$/.py/' | grep -vxF "$known")" ]; then
        only=scripts
    fi
    # Once per land: rerunning on every lost push race livelocks landing while
    # main keeps changing scripts/; CI gates the tip that lands.
    [ -z "$rechecked" ] || { echo "land: main changed the gates again; landing on the rerun's result, CI checks this tip" >&2; return 0; }
    rechecked=1
    echo "land: main changed the gates ($(printf '%s\n' $live | head -3 | tr '\n' ' ' | sed 's/ $//')); rerunning ${only:+its steps of }the commit tier on the rebased tip" >&2
    tip=$(git rev-parse HEAD)
    prev=$(trap -p EXIT)
    trap 'git reset -q --soft "$tip"' EXIT
    git reset -q --soft "$main_base"
    # No cpl is handed over: the rebase rewrote main's crate files, and the commit tier
    # rebuilds target/debug/cpl before the corpus audit and the ratchet run it.
    set +e
    if [ -n "$only" ]; then
        uv run --quiet python scripts/ci.py commit --only "$only" >&2
    else
        .githooks/pre-commit >&2
    fi
    rc=$?
    set -e
    git reset -q --soft "$tip"
    eval "${prev:-trap - EXIT}"
    [ "$rc" = 0 ] || { echo "land: the commit tier fails on the rebased tip (above); fix it and land again" >&2; exit 1; }
}
# Only committed work lands (handbook/guidelines/ci-checklist.md ci/R27): a
# commit whose hook failed left nothing new, and pushing the regeneration
# alone would ship generated files without their source.
failed_log="$(git rev-parse --git-common-dir)/cpl-commit-failed.log"
if [ -z "$(git rev-list $main_base..HEAD)" ]; then
    echo "land: nothing to land: HEAD is already on main (did the commit or its hook fail?)" >&2
    [ ! -s "$failed_log" ] || { echo "land: the last failed pre-commit ended:" >&2; tail -15 "$failed_log" >&2; }
    exit 1
fi
rm -f "$failed_log"
recheck
# The slices a hook's 25 s cut skipped (a cold build of their tests) run here,
# on the rebased tip, before anything lands; a slice red on main too doesn't refuse (ci/R40).
# So do the slices the rebased tip's affected.py selects for the change: one main
# added after the commit was made never reached its hook (fc73818ec and
# silent-unknowns).
pending="$(git rev-parse --git-dir)/cpl-pending-slices"
tip_slices=$(uv run --quiet python scripts/affected.py --base FETCH_HEAD 2>/dev/null | sed -n 's/^slices=//p') || tip_slices=
slices=$({ cat "$pending" 2>/dev/null; printf '%s\n' $tip_slices; } | sed '/^$/d' | sort -u | tr '\n' ' ' | sed 's/ $//')
if [ -z "$fast" ] && [ -n "$slices" ]; then
    echo "land: running the touched-path slices at the rebased tip: $slices" >&2
    { uv run --quiet python scripts/affected.py --build-slices $slices >&2 &&
        uv run --quiet python scripts/affected.py --run-slices --base "$main_base" $slices >&2; } ||
        { echo "land: a touched-path slice fails at the rebased tip; fix it and land again: uv run scripts/affected.py --run-slices $slices" >&2; exit 1; }
    rm -f "$pending"
fi
# Pending files are per worktree (git-dir): one lane's land neither runs nor
# deletes another's. The commit-tier steps a hook's cut never reached (b3b3c30dd's came before its
# wasm check), on the rebased tip staged against main as the hook stages them,
# and again on the new tip after a lost push race.
pending_steps="$(git rev-parse --git-dir)/cpl-pending-steps"
steps_runs=${CPL_LAND_STEPS_RUNS:-0}
run_pending_steps() {
    [ -z "$fast" ] && [ -s "$pending_steps" ] || return 0
    steps=$(sort -u "$pending_steps" | tr '\n' ',' | sed 's/,$//')
    # Once, on the first rebased tip: every rerun gives main another window to
    # move, and at its pace one per lost race cost minutes a try; CI checks the tip.
    steps_runs=$((steps_runs + 1))
    [ "$steps_runs" -le 1 ] || { echo "land: main moved again; landing on the skipped steps' rerun, CI checks this tip" >&2; return 0; }
    echo "land: running the commit-tier steps the hook's cut skipped: $steps" >&2
    tip=$(git rev-parse HEAD)
    prev=$(trap -p EXIT)
    trap 'git reset -q --soft "$tip"' EXIT
    git reset -q --soft "$main_base"
    set +e
    uv run --quiet python scripts/ci.py commit --only "$steps" >&2
    rc=$?
    set -e
    git reset -q --soft "$tip"
    eval "${prev:-trap - EXIT}"
    [ "$rc" = 0 ] || { echo "land: a commit-tier step the hook's cut skipped fails at the rebased tip (above); fix it and land again: uv run scripts/ci.py commit --only $steps" >&2; exit 1; }
}
run_pending_steps
# The staged conformance cases a hook's cut skipped, against what lands: 4aaa8b0db
# landed a case without the codegen fix its author had left unstaged.
# Checks too quick to leave to a hook a cut may have skipped (7e3ff18e1 landed
# a std trait with no traits.toml case while its hook was cut). Only a problem
# line main's tree doesn't print refuses the land, as the ratchet judges main's
# own reds; CI on main still fails on those.
std_changed=$(git diff --name-only "$main_base" HEAD -- 'std/*.cp' spec/conformance/traits.toml)
if [ -z "$fast" ] && [ -n "$std_changed" ] && [ -f scripts/spec-gate.py ]; then
    traits_gate() { { uv run --quiet python "$1/scripts/spec-gate.py" traits 2>/dev/null || :; } | sort; }
    tip_traits=$(traits_gate .)
    if ! uv run --quiet python scripts/spec-gate.py traits >/dev/null 2>&1; then
        base_dir=$(mktemp -d)
        base_paths=$(git ls-tree --name-only "$main_base" std scripts/spec-gate.py scripts/specgate \
            spec/conformance/traits.toml spec/conformance/traits)
        [ -z "$base_paths" ] || git archive "$main_base" $base_paths | tar -x -C "$base_dir"
        base_traits=$(traits_gate "$base_dir")
        rm -rf "$base_dir"
        new_traits=$(comm -23 <(printf '%s\n' "$tip_traits") <(printf '%s\n' "$base_traits") | grep . || :)
        if [ -n "$new_traits" ] || [ -z "$tip_traits" ]; then
            printf '%s\n' "${new_traits:-spec-gate traits failed with no problem line}" >&2
            echo "land: a std trait impl has no spec/conformance/traits.toml case; write it, then: scripts/spec-gate.py traits --write" >&2
            exit 1
        fi
        echo "land: spec-gate traits: $(printf '%s\n' "$tip_traits" | wc -l) problems, all already on main" >&2
    fi
fi
pending_cases="$(git rev-parse --git-dir)/cpl-pending-cases"
# A std pub fn signature change runs every case and corpus program that calls
# it (8f1685382 changed splice and three cases went red on main).
sig_callers=
if [ -z "$fast" ] && [ -n "$std_changed" ] && [ -f scripts/affected.py ]; then
    sig_callers=$(uv run --quiet python scripts/affected.py --sig-callers "$main_base")
    printf '%s\n' $sig_callers | grep '^spec/conformance/' >>"$pending_cases" || :
fi
if [ -z "$fast" ] && [ -s "$pending_cases" ]; then
    cases=$(sort -u "$pending_cases" | while read -r c; do [ ! -f "$c" ] || echo "$c"; done | tr '\n' ' ' | sed 's/ $//')
    if [ -n "$cases" ]; then
        git diff --quiet HEAD -- crates std Cargo.toml Cargo.lock ||
            { echo "land: uncommitted changes under crates/ or std/ would be tested in place of what lands; commit or stash them, then land again" >&2; exit 1; }
        echo "land: running the staged conformance cases the hook's cut skipped: $cases" >&2
        uv run --quiet python scripts/conform_staged.py --base "$main_base" --build $cases >&2 ||
            { echo "land: a staged conformance case fails at the rebased tip; fix it and land again: uv run scripts/conform_staged.py --build $cases" >&2; exit 1; }
    fi
    rm -f "$pending_cases"
fi
sig_programs=$(printf '%s\n' $sig_callers | grep -v '^spec/conformance/' | tr '\n' ' ' | sed 's/ $//') || :
if [ -n "$sig_programs" ]; then
    echo "land: cpl check of the corpus programs that call a changed std signature: $sig_programs" >&2
    uv run --quiet python scripts/conform_staged.py --base "$main_base" --check $sig_programs >&2 ||
        { echo "land: a corpus program no longer checks against the changed std signature (above); update it in this change" >&2; exit 1; }
fi
# A compiler crate or prelude std change runs the whole suite before the lock,
# refusing only a case that passes at the base: cea2d50e4 changed cpl-check and
# cpl-ir, and std-graph/bfs-over-named-product-domain (prelude `bfs`, no `use`) went red on main.
if [ -z "$fast" ] && [ -f scripts/conform_staged.py ]; then
    uv run --quiet python scripts/conform_staged.py --base "$main_base" --all >&2 ||
        { echo "land: a conformance case that passes on main fails at the rebased tip (above); fix it and land again: uv run scripts/conform_staged.py --base origin/main --all" >&2; exit 1; }
fi
changed=$(git diff --name-only --diff-filter=ACMR "$main_base" HEAD)
# A fast land changes a case list only by listing the cases it adds: nothing ran the suite
# that would justify any other edit, and every next_* ratchet counts the lists. The
# integrator lane regenerates them (CPL_LAND_INTEGRATOR=1).
lists='crates/cpl-next/tests/passing.txt crates/cpl-next/tests/wrong.tsv crates/cpl-next/tests/masked.txt crates/cpl-next/tests/refused.tsv'
if [ -n "$fast" ] && [ -z "${CPL_LAND_INTEGRATOR:-}" ]; then
    added=$(git diff --name-only --diff-filter=A "$main_base" HEAD -- spec/conformance crates/cpl-next/tests/programs |
        sed -n -e 's|^spec/conformance/||p' -e 's|^crates/cpl-next/tests/programs/|next/|p')
    edits=$(git diff -U0 "$main_base" HEAD -- $lists | grep -E '^[-+]' | grep -vE '^(\+\+\+|---) ' || :)
    foreign=$(printf '%s\n' "$edits" | sed '/^$/d' | while IFS= read -r l; do
        id=${l#[-+]}; id=${id%%$'\t'*}
        case $l in +*) printf '%s\n' "$added" | grep -qxF "$id" || echo "$l" ;; *) echo "$l" ;; esac
    done)
    if [ -n "$foreign" ]; then
        echo "land: a fast land edits the case lists only to list the cases it adds; drop these lines (the integrator regenerates the lists):" >&2
        printf '%s\n' "$foreign" | head -10 >&2
        exit 1
    fi
fi
# A wasm budget raise plan/signoffs.toml holds is exact and outside the transfer rule
# (web/scripts/wasm-crates.mjs signedRaises); no commit message signs one (audit #18 (2)).
CPL_BUDGET_RAISES=$(signoff_rows budget_raise '{crate} {old}->{new}')
export CPL_BUDGET_RAISES
# The touched crates' cpl.wasm budgets, from the named build .githooks/post-commit
# started (scripts/wasm-warm.sh), never waiting on one: 849a0be7d's commit hook
# was cut before its wasm check and grew cpl_lsp 544 B gz past its budget. Held to
# main's sizes, a crate over its budget refuses only if it grew, so main never gets
# worse while lanes cut a crate back under.
wasm_touched=$(printf '%s\n' "$changed" | sed -n 's|^crates/\([^/]*\)/.*|\1|p' | sort -u)
if [ -z "$fast" ] && [ -n "$wasm_touched" ] && [ -x scripts/wasm-warm.sh ]; then
    # Fail closed (wasm-cut): skipping when the warm build was of the pre-rebase tree let
    # 277f37f42 and five more grow an over-budget cpl_check. --check builds main's sizes.
    if ! wasm=$(scripts/wasm-warm.sh --current $wasm_touched 2>/dev/null); then
        echo "land: building the rebased tip's cpl.wasm to hold $(echo $wasm_touched) to their budgets" >&2
        { CPL_WARM_MAIN=$main_base scripts/wasm-warm.sh --wait >&2 && wasm=$(scripts/wasm-warm.sh --current $wasm_touched); } ||
            { echo "land: no cpl.wasm of the rebased tip (above: a missing wasm32 target, a build error or uncommitted crate changes), so its crates can't be held to their budgets; fix that and land again" >&2; exit 1; }
    fi
    scripts/wasm-warm.sh --check "$wasm" "$main_base" $wasm_touched >&2 ||
        { echo "land: a cpl.wasm crate grew past main while over its budget, or crossed its budget (above), touched here or not; cut it at its source and land again" >&2; exit 1; }
    transfer_checked=1
fi
# A raised wasm budget no build measured against moved code (decisions.md 634) refuses.
if [ -z "${transfer_checked:-}" ] && printf '%s\n' "$changed" | grep -qx web/scripts/wasm-crate-budgets.json; then
    was="$(git rev-parse --git-dir)/cpl-parent-budgets.json"
    git show FETCH_HEAD:web/scripts/wasm-crate-budgets.json >"$was" 2>/dev/null || echo '{}' >"$was"
    node web/scripts/wasm-crates.mjs --budgets "$was" >&2 ||
        { echo "land: a wasm budget rose with no measured transfer (above); land it with the code that moved and a current wasm build" >&2; exit 1; }
fi
seen="$(git rev-parse --git-dir)/cpl-last-land"
if [ -s "$seen" ] && git cat-file -e "$(cat "$seen")^{commit}" 2>/dev/null; then
    product=$(git diff --stat "$(cat "$seen")" "$main_base" -- AGENTS.md handbook plan/product)
    [ -z "$product" ] || printf 'land: product direction or the handbook changed since your last land; read it:\n%s\n' "$product"
fi
if [ -n "$changed" ]; then
    marks=$(git grep -nE '^(<{7}|>{7}|\|{7})( |$)' HEAD -- $changed ':!*.png' ':!*.gz' || [ $? = 1 ])
    [ -z "$marks" ] || { echo "land: merge conflict markers:"$'\n'"$marks" >&2; exit 1; }
fi
# Two std changes each green alone can make a std cpl-next rejects for every program
# (e9aad9fb6 and 28a4c9495 each added `impl Seq for Subset`, a minute apart): check the
# rebased std with this worktree's cpl-next, about 1 s, unless main's std fails with it too.
next_bin=target/debug/cpl-next
if [ -n "$(git diff --name-only "$main_base" HEAD -- 'std/*.cp')" ] && [ -x "$next_bin" ] &&
    git cat-file -e "$main_base:std" 2>/dev/null; then
    std_dir=$(mktemp -d)
    git archive "$main_base" std | tar -x -C "$std_dir"
    printf 'print(1)\n' > "$std_dir/one.cp"
    if "$next_bin" --std "$std_dir/std" --check "$std_dir/one.cp" >/dev/null 2>&1 &&
        ! said=$("$next_bin" --std std --check "$std_dir/one.cp" 2>&1); then
        rm -rf "$std_dir"
        printf '%s\n' "$said" | head -5 >&2
        echo "land: cpl-next rejects the rebased std, which main's passes (above); fix std and land again" >&2
        exit 1
    fi
    rm -rf "$std_dir"
fi
# The rebase rewrote main's crate files: corpus_fmt_dirty refuses a stale cpl, so build it.
# A fast land runs the ratchet too (about 30 s): it is what holds main's counters and lists.
[ ! -f scripts/debt-ratchet.py ] || [ ! -f Cargo.toml ] || cargo build -q -p cpl-cli ||
    { echo "land: cargo build -p cpl-cli failed at the rebased tip; the ratchet runs that cpl" >&2; exit 1; }
phase build
# A counter that improved is lowered to its number in the commit that improved it.
tighten=--check
[ -z "$(git rev-list "$main_base"..HEAD)" ] || ! git diff --quiet HEAD -- "$ratchet" || tighten=--tighten
[ ! -f scripts/debt-ratchet.py ] || uv run --quiet python scripts/debt-ratchet.py $tighten >/dev/null ||
    { uv run --quiet python scripts/debt-ratchet.py --check >&2; exit 1; }
[ "$tighten" = --check ] || git diff --quiet -- "$ratchet" ||
    { git commit -q --amend --no-edit --no-verify -- "$ratchet" &&
        echo "land: lowered the ratchet's improved counters in $(git rev-parse --short HEAD)" >&2; }
phase ratchet
# A decision number is taken once it is on main (decision 505): a heading
# main never had, even as a `--reserve` stub, would collide with another lane.
[ ! -f scripts/specgate/decisions.py ] || uv run --quiet python scripts/spec-gate.py decisions --check-landing "$main_base" || exit 1
crates=$(printf '%s\n' "$changed" | sed -n 's|^crates/\([^/]*\)/.*\.rs$|-p \1|p' | sort -u | tr '\n' ' ')
[ -z "$crates" ] || cargo fmt --check $crates >/dev/null 2>&1 ||
    { echo "land: not formatted; run: cargo fmt $crates" >&2; exit 1; }
# The files that rustc rejects (error[E...]) in cargo's output on stdin, in a
# crate this change touches: two changes each green alone can combine into one
# that doesn't compile (5246e1c33 retyped a field another file still pushed),
# and that one must not land.
own_compile_errors() {
    awk '/^(error|warning)/ {e = /^error\[E/} e && /^ *--> / {sub(/^ *--> /, ""); sub(/:.*/, ""); print}' |
        sort -u | while IFS= read -r f; do
            c=${f#crates/}; c=${c%%/*}
            case " $crates " in *" -p $c "*) echo "$f" ;; *) printf '%s\n' "$changed" | grep -Fx -- "$f" || : ;; esac
        done
}
refuse_if_broken() {
    bad=$(printf '%s\n' "$1" | own_compile_errors)
    [ -z "$bad" ] || { printf '%s\n' "$1" | grep -E '^error|-->' | head -20 >&2
        echo "land: $(echo $bad) no longer compiles at the rebased tip; fix it and land again" >&2; exit 1; }
}
# Clippy as CI runs it (-D warnings, every target) on the touched crates and
# the workspace crates that depend on them, at the rebased tip. A compile
# error in a crate this change touches refuses the land; on any other
# failure it warns and lands, leaving it to CI (whose regen job applies
# `clippy --fix`). It runs to the end, so a cold cache is slow, never a pass.
users=""
if [ -z "$fast" ] && [ -n "$crates" ]; then
    users=$(cargo metadata -q --format-version 1 --no-deps | uv run --quiet python -c '
import json, sys
meta = json.load(sys.stdin)
names = {p["name"] for p in meta["packages"]}
users = {n: set() for n in names}
for p in meta["packages"]:
    for d in p["dependencies"]:
        if d["name"] in names:
            users[d["name"]].add(p["name"])
todo, seen = [a for a in sys.argv[1:] if a != "-p"], set()
while todo:
    n = todo.pop()
    if n in names and n not in seen:
        seen.add(n)
        todo += users[n]
print(" ".join("-p " + n for n in sorted(seen)))
' $crates)
    gate_start=$(date +%s)
    set +e
    out=$(cargo clippy -q $users --all-targets -- -D warnings 2>&1)
    rc=$?
    set -e
    echo "land: clippy on $(printf '%s\n' $users | grep -vc -- -p) crate(s) in $(( $(date +%s) - gate_start ))s" >&2
    case "$rc" in
        0) ;;
        *) refuse_if_broken "$out"
           printf '%s\n' "$out" | grep -E '^(error|warning)|-->' | head -20 >&2
           echo "land: WARNING: clippy fails on $users (maybe already on main); landing, CI's regen applies clippy --fix" >&2 ;;
    esac
fi
. "$(dirname "$0")/retry.sh"
# Lanes take turns pushing: whoever holds refs/land/lock on the remote checks that
# main hasn't moved and pushes, so a gated change no longer loses push races to
# unrelated lands (explain's counter lost 10 in a row). The lock is held only for
# that check and push: a lane whose main moved drops it while it rebases and reruns
# what main's new commits change, or every other lane waits out its reruns. A lock
# older than $lock_stale s is a dead holder's and is taken over, compare-and-swap, so
# two waiters can't both take the same one; pushes that skip land.sh still race,
# hence retry. Waiters take turns in arrival order (Core and Idioms starved for
# 20 min while later lands won each 2 s poll): each pushes a ticket, re-stamped
# while it waits, and only the oldest ticket stamped within $lock_stale s may take
# the lock; a dead waiter's ticket expires. A land keeps its ticket's place
# across lost races.
# -v2: copies of land.sh from before c457bd860 never restamp and take every lock
# held over 120 s; on their own ref they contend only with each other.
lock_ref=refs/land/lock-v2
old_lock_ref=refs/land/lock
lock_stale=${CPL_LAND_LOCK_STALE:-120}
# Seconds, not minutes: 30 let consumers' land take 4 live lands' locks in 3 minutes.
case "$lock_stale" in '' | *[!0-9]*) lock_stale=0 ;; esac
if [ "$lock_stale" -lt 120 ]; then
    echo "land: CPL_LAND_LOCK_STALE=${CPL_LAND_LOCK_STALE} is under 120 s and would take live lands' locks; using 120" >&2
    lock_stale=120
fi
# A ticket only orders waiters, so its expiry may be shorter (tests); the lock's may not.
ticket_stale=${CPL_LAND_TICKET_STALE:-$lock_stale}
# Unset, a waiter never gives up (tests bound it to observe a wait).
lock_wait=${CPL_LAND_LOCK_WAIT:-}
# The holder restamps the lock this often, so a lock is stale only when its holder
# stopped beating: a live one is never taken over however long it holds.
lock_beat=${CPL_LAND_LOCK_BEAT:-20}
# Every box is named devin-box: a random token tells holders apart.
lock_who="$(hostname)-$(od -An -N4 -tx4 /dev/urandom | tr -d ' \n')"
lock_held=""
# In the git dir, not /tmp: a cleaned /tmp would lease every beat on nothing.
lock_file=$(mktemp "$(git rev-parse --absolute-git-dir)/cpl-land-lock.XXXXXX")
beat_pid=""
lock_branch=$(git rev-parse --abbrev-ref HEAD 2>/dev/null) || lock_branch=detached
lock_step() { echo "$1" >"$lock_file.step"; }
# Fields 1-5 are what every copy reads (the stale check reads 4); the rest tells a
# waiter whose land it is and what it's doing (Core: a wait explains itself).
lock_note() {
    echo "land-lock $lock_who $$ $(date +%s) $domain $lock_branch@$(git rev-parse --short HEAD) $(cat "$lock_file.step" 2>/dev/null || echo holding)"
}
start_beat() {
    echo "$lock_held" >"$lock_file"
    rm -f "$lock_file.lost"
    (
        stop="" nap="" wait_s=$lock_beat
        # Kills the nap too: a live sleep holds the land's output open for $lock_beat s.
        trap 'stop=1; [ -z "$nap" ] || kill "$nap" 2>/dev/null' TERM
        while [ -z "$stop" ]; do
            sleep "$wait_s" &
            nap=$!
            wait "$nap" 2>/dev/null || :
            [ -z "$stop" ] || break
            new=$(git commit-tree "$(git hash-object -t tree /dev/null)" -m "$(lock_note)") || continue
            wait_s=$lock_beat
            # Bounded: a push hung on the proxy past 120 s let a live holder look stale.
            if err=$(timeout 20 git push -q --no-verify --force-with-lease="$lock_ref:$(cat "$lock_file")" "$remote" "$new:$lock_ref" 2>&1); then
                echo "$new" >"$lock_file"
                continue
            fi
            # A beat whose reply was lost (a 504 after the update) leaves the lease
            # behind, and every later beat fails: compiler took a live holder's lock
            # at 120 s. Adopt the lock while it still names this holder.
            cur=$(git ls-remote "$remote" "$lock_ref" 2>/dev/null | cut -f1) || cur=""
            if [ -n "$cur" ] && [ "$cur" = "$(cat "$lock_file")" ]; then
                # Still ours, unchanged: the proxy's 429s and 504s pass, so try again
                # in seconds, not at the next beat.
                echo "land: a land lock heartbeat failed (${err##*$'\n'}); retrying" >&2
                wait_s=3
            elif [ -n "$cur" ] && git fetch -q --no-write-fetch-head "$remote" "$lock_ref" 2>/dev/null &&
                [ "$(git log -1 --format=%s "$cur" 2>/dev/null | cut -d' ' -f2,3)" = "$lock_who $$" ]; then
                echo "$cur" >"$lock_file"
            elif { [ -n "$cur" ] && [ "$cur" != "$(cat "$lock_file")" ]; } ||
                { [ -z "$cur" ] && [[ "$err" == *"stale info"* ]]; }; then
                # Another lane holds it now: push() must not push main on this hold.
                echo "land: lost the land lock (now ${cur:-free}); taking it again before pushing main" >&2
                : >"$lock_file.lost"
                break
            else
                echo "land: a land lock heartbeat failed (${err##*$'\n'}); retrying" >&2
                wait_s=3
            fi
        done
    ) &
    beat_pid=$!
}
stop_beat() {
    [ -n "$beat_pid" ] || return 0
    kill -TERM "$beat_pid" 2>/dev/null || :
    wait "$beat_pid" 2>/dev/null || :
    beat_pid=""
}
# One name per land, kept across lost races, failed pushes and the re-exec, so a
# land never rejoins at the back. A live waiter never times out: only a ticket
# (or lock) that stops changing is dead.
ticket=${CPL_LAND_TICKET:-refs/locks/queue/$(date +%s%N)-$$-$(printf %s "${domain:-none}" | tr -c 'A-Za-z0-9_-' _)}
queued=""
beat=0
# A waiter this far back looks every $queue_poll s, not 2: at 58 waiters,
# 2 s polls and 10 s heartbeats drew GitHub 504s (about 35 requests/s).
queue_poll=${CPL_LAND_QUEUE_POLL:-15}
heartbeat=${CPL_LAND_HEARTBEAT:-30}
enqueue() {
    beat=$(date +%s)
    git push -q --no-verify -f "$remote" \
        "$(git commit-tree "$(git hash-object -t tree /dev/null)" -m "land ticket $(hostname) $$ $domain $lock_branch@$(git rev-parse --short HEAD)"):$ticket" 2>/dev/null &&
        queued=1
}
dequeue() {
    [ -n "$queued" ] || return 0
    git push -q --no-verify "$remote" ":$ticket" 2>/dev/null || :
    queued=""
}
drop_ticket() {
    echo "land: dropping queue ticket ${1##*/}: $3" >&2
    git push -q --no-verify --force-with-lease="$1:$2" "$remote" ":$1" 2>/dev/null || :
}
# Sets $first to the oldest live ticket, $ahead to the live tickets before ours
# and $lock_now to the lock's sha (empty when free), from one ls-remote. A ticket is live while its sha changes
# (each heartbeat is a new commit), timed by this land's clock from ls-remote
# alone: fetching the tickets' commits failed whole on one bad ref (HTTP 500),
# and that let every waiter skip the line. Fails if the queue can't be listed.
declare -A ticket_sha=() ticket_seen=()
first_in_line() {
    local list sha ref name now mine="" re='^[0-9]{19}-[0-9]+-[A-Za-z0-9_-]+$'
    first="" ahead=0
    lock_now=""
    list=$(git ls-remote "$remote" 'refs/locks/queue/*' "$lock_ref" 2>/dev/null) || return 1
    now=$(date +%s)
    while read -r sha ref; do
        [ -n "$ref" ] || continue
        [ "$ref" != "$lock_ref" ] || { lock_now=$sha; continue; }
        name=${ref#refs/locks/queue/}
        [[ $name =~ $re ]] || { drop_ticket "$ref" "$sha" "malformed name"; continue; }
        if [ "${ticket_sha[$name]:-}" != "$sha" ]; then
            ticket_sha[$name]=$sha
            ticket_seen[$name]=$now
        elif [ "$ref" != "$ticket" ] && [ $((now - ticket_seen[$name])) -gt "$ticket_stale" ]; then
            drop_ticket "$ref" "$sha" "no heartbeat for over ${ticket_stale}s"
            continue
        fi
        first=${first:-$ref}
        [ "$ref" = "$ticket" ] || [ -n "${mine:-}" ] || ahead=$((ahead + 1))
        [ "$ref" != "$ticket" ] || mine=1
    done < <(printf '%s\n' "$list" | sort -k2)
}
# Old copies read only refs/land/lock: parked with a stamp in 2100, it never looks
# 120 s stale to them, so they wait it out instead of racing v2 lands onto main.
park_old_lock() {
    local old subj parked
    old=$(git ls-remote "$remote" "$old_lock_ref" 2>/dev/null | cut -f1) || return 0
    if [ -n "$old" ]; then
        git fetch -q --no-write-fetch-head "$remote" "$old_lock_ref" 2>/dev/null || :
        subj=$(git log -1 --format=%s "$old" 2>/dev/null) || subj="$old, not fetched"
        case "$subj" in "land-lock retired-use-lock-v2 "*) return 0 ;; esac
        echo "land: a land.sh from before c457bd860 holds $old_lock_ref ($subj); its lane should stop it, pull and land again. Parking the old lock" >&2
    fi
    parked=$(git commit-tree "$(git hash-object -t tree /dev/null)" -m "land-lock retired-use-lock-v2 0 4102444800 factory")
    git push -q --no-verify --force-with-lease="$old_lock_ref:$old" "$remote" "$parked:$old_lock_ref" 2>/dev/null || :
}

take_lock() {
    local me holder held_by at waited=0 lined="" blind="" nap nap_said="" stuck_head="" stuck_since=0 poll="" seen_holder="" beat_said="" last_holder="" last_change=0 age why head main_now
    # On unless CPL_LAND_QUEUE= : without it the lock went to whoever polled first, and
    # Tools' passed land waited 20+ min while 8 later lands pushed.
    [ -z "${CPL_LAND_QUEUE-1}" ] || enqueue || :
    park_old_lock
    while :; do
        first=""
        if [ -n "$queued" ] && ! first_in_line; then
            [ -n "$blind" ] || echo "land: can't list the land queue; waiting, not skipping it" >&2
            blind=1 first="?"
        fi
        if [ -n "$first" ] && [ "$first" != "$ticket" ]; then
            [ "$first" = "$lined" ] || [ "$first" = "?" ] || echo "land: waiting in line behind ${first##*/}" >&2
            lined=$first
            # A live head that leaves the lock free isn't landing (Idioms' 30 min
            # behind one ticket): past $ticket_stale s it loses its place.
            if [ "$first" = "?" ] || [ -n "$lock_now" ] || [ "$first" != "$stuck_head" ]; then
                stuck_head=$first stuck_since=$(date +%s)
            elif [ $(($(date +%s) - stuck_since)) -gt "$ticket_stale" ]; then
                drop_ticket "$first" "${ticket_sha[${first#refs/locks/queue/}]}" \
                    "first in line but left the land lock free for over ${ticket_stale}s"
                stuck_head=""
                continue
            fi
            [ -z "$lock_wait" ] || [ "$waited" -lt "$lock_wait" ] || { echo "land: no land lock after ${waited}s; land again" >&2; dequeue; return 1; }
            nap=2
            if [ "${ahead:-0}" -gt 2 ]; then
                [ "$nap_said" = "$queue_poll" ] || echo "land: $ahead ahead in line; looking every ${queue_poll}s" >&2
                nap=$queue_poll nap_said=$queue_poll
            fi
            sleep "$nap"
            waited=$((waited + nap))
            [ $(($(date +%s) - beat)) -lt "$heartbeat" ] || enqueue || :
            continue
        fi
        # Stamped at each try, so a lock is as old as its holder has held it, not
        # as old as the wait for it: otherwise a long wait makes it look stale at once.
        me=$(git commit-tree "$(git hash-object -t tree /dev/null)" -m "$(lock_note)")
        if err=$(git push -q --no-verify --force-with-lease="$lock_ref:" "$remote" "$me:$lock_ref" 2>&1); then
            lock_held=$me
            start_beat
            dequeue
            [ "$waited" = 0 ] || echo "land: took the land lock after ${waited}s" >&2
            return 0
        fi
        holder=$(git ls-remote "$remote" "$lock_ref" | cut -f1)
        if [ -z "$holder" ]; then
            # A lost compare-and-swap is a lane that took the lock and dropped it
            # before this look, not a refusal: counted as one, churn pushed unlocked.
            # The server's own swap losing (a lock made between our look and our
            # write, then dropped before this look) is a race too: the 19:58Z
            # infra land refused on five of them in a row, behind fast holders.
            case "$err" in
            *"stale info"* | *"incorrect old value"* | *"cannot lock ref"* | *"failed to update ref"* | *"already exists"*)
                raced=$((${raced:-0} + 1))
                [ "$raced" -lt 60 ] || { echo "land: lost the land lock race $raced times in a row; land again" >&2; dequeue; return 1; }
                continue ;;
            esac
            # Fails closed: Idioms' land pushed unlocked after two HTTP 500s.
            refused=$((${refused:-0} + 1))
            [ "$refused" -lt 5 ] || { echo "land: $remote refuses $lock_ref (${err##*$'\n'}); land again" >&2; dequeue; return 1; }
            sleep "$refused"
            continue
        fi
        git fetch -q --no-write-fetch-head "$remote" "$lock_ref" 2>/dev/null || :
        held_by=$(git log -1 --format=%s "$holder" 2>/dev/null) || held_by="$holder, not fetched"
        at=$(echo "$held_by" | cut -d' ' -f4)
        # Its own: a drop whose delete failed (idioms' three HTTP 500s on main, then
        # on the lock) left it, and waiting it out cost 120 s with nobody holding it.
        if [ "$(echo "$held_by" | cut -d' ' -f2,3)" = "$lock_who $$" ]; then
            git push -q --no-verify --force-with-lease="$lock_ref:$holder" "$remote" "$me:$lock_ref" 2>/dev/null &&
                { lock_held=$me; start_beat; dequeue; echo "land: took back this land's own lock" >&2; return 0; }
            continue
        fi
        # A holder this lane can't read (the fetch lost a race, or no stamp) is
        # never stale: it may be live, so wait and look again.
        case "$at" in
        '' | *[!0-9]*) at=$(date +%s) beat_said="no readable stamp" ;;
        *) beat_said="beat $(($(date +%s) - at))s ago" ;;
        esac
        # Stale by its own stamp and also unchanged here for two beats: a waiter
        # whose clock runs ahead of the holder's never takes a lock still beating.
        [ "$holder" = "$last_holder" ] || { last_holder=$holder; last_change=$(date +%s); }
        # A holder whose head is on main has pushed and only failed to delete its lock
        # (killed, or the delete failed): it is taken once a beat is missed.
        age=$(( $(date +%s) - ${at:-0} )) why=""
        head=$(echo "$held_by" | cut -d' ' -f6) head=${head##*@}
        if [ "$age" -gt "$lock_beat" ] && [ -n "$head" ] &&
            main_now=$(git ls-remote "$remote" refs/heads/main | cut -f1) && [ -n "$main_now" ] &&
            git fetch -q --no-write-fetch-head "$remote" main 2>/dev/null &&
            git merge-base --is-ancestor "$head" "$main_now" 2>/dev/null; then
            why="$head is on main"
        elif [ "$age" -gt "$lock_stale" ] && [ $(( $(date +%s) - last_change )) -ge $((2 * lock_beat)) ]; then
            why="no beat for over ${lock_stale}s"
        fi
        if [ -n "$why" ]; then
            echo "land: taking over a land lock last beat ${age}s ago (holder $(echo "$held_by" | cut -d' ' -f2,3), $why)" >&2
            git push -q --no-verify --force-with-lease="$lock_ref:$holder" "$remote" "$me:$lock_ref" 2>/dev/null &&
                { lock_held=$me; start_beat; dequeue; return 0; }
            continue
        fi
        # Said again for each new holder, with how long since it last beat.
        if [ "$(echo "$held_by" | cut -d' ' -f2,3)" != "$seen_holder" ]; then
            seen_holder=$(echo "$held_by" | cut -d' ' -f2,3)
            echo "land: waiting for the land lock ($held_by; $beat_said)" >&2
        fi
        [ -z "$lock_wait" ] || [ "$waited" -lt "$lock_wait" ] || { echo "land: no land lock after ${waited}s; land again" >&2; dequeue; return 1; }
        # Backs off 6 s to about 30 s, jittered: 19 waiters polling every 2 s drew the
        # proxy's 429s that failed the holder's beats.
        poll=$(( ${poll:-3} * 2 > 24 ? 24 : ${poll:-3} * 2 ))
        nap=$((poll + RANDOM % 7))
        [ -z "${CPL_LAND_LOCK_POLL:-}" ] || nap=$CPL_LAND_LOCK_POLL
        sleep "$nap"
        waited=$((waited + nap))
        [ -z "$queued" ] || [ $(($(date +%s) - beat)) -lt "$heartbeat" ] || enqueue || :
    done
}
drop_lock() {
    [ -n "$lock_held" ] || return 0
    # Deleted first, while the beat still runs: a holder killed after its push (a
    # tool's timeout) left its lock to go stale, and every waiter paid 120 s. The
    # beat's lease can't bring a deleted lock back.
    [ -e "$lock_file.lost" ] || RETRY_TRIES=5 RETRY_WAIT=1 retry delete_lock ||
        echo "land: could not delete the land lock after 5 tries; a waiter takes it once its beat is ${lock_beat}s old and $(git rev-parse --short HEAD) is on main" >&2
    stop_beat
    rm -f "$lock_file.lost"
    lock_held=""
}
trap 'drop_lock; dequeue; rm -f "$lock_file" "$lock_file.lost" "$lock_file.step"' EXIT
# Rebase and recheck run without the lock: the non-force push already refuses a
# half-gated tip, so the lock covers only the fast-forward check and the push.
# Rechecking under it let one holder keep every lane waiting for minutes.
try=0
push() {
    try=$((try + 1))
    git fetch -q "$remote" main
    # Chasing main without the lock never ends at its pace (lint-fmt's c0493782e
    # went round 4 times on its cargo check): after two pulls, rebase holding the lock.
    chases=0
    while ! git merge-base --is-ancestor FETCH_HEAD HEAD; do
        [ "$chases" -lt 2 ] || { echo "land: main keeps moving; taking the lock to rebase holding it" >&2; break; }
        chases=$((chases + 1))
        pull || exit 1
        main_base=$(git rev-parse FETCH_HEAD)
        reexec_if_land_changed
        recheck
        run_pending_steps
        # What landed meanwhile can break this change's files: check again.
        [ -z "$users" ] || refuse_if_broken "$(cargo check -q $users --all-targets 2>&1)"
        git fetch -q "$remote" main
    done
    phase gates
    lock_step pushing
    take_lock || exit 1
    phase lock
    git fetch -q "$remote" main
    # The holder this land waited on pushed: rebase holding the lock, else every
    # waiter's turn goes to the next push (compiler's land lost 5 of 5 tries so).
    # A move of land.sh, or of the gates before their one rerun, still lets go.
    while ! git merge-base --is-ancestor FETCH_HEAD HEAD; do
        git diff --quiet "$main_base" FETCH_HEAD -- scripts/land.sh &&
            { [ -n "$rechecked" ] || git diff --quiet "$checked" FETCH_HEAD -- $gate_paths; } ||
            { drop_lock; return 1; }
        echo "land: main moved while this land waited for the lock; rebasing holding it" >&2
        lock_step "rebasing under the lock"
        pull || exit 1
        main_base=$(git rev-parse FETCH_HEAD)
        recheck
        run_pending_steps
        [ -z "$users" ] || refuse_if_broken "$(cargo check -q $users --all-targets 2>&1)"
        git fetch -q "$remote" main
    done
    # A failed push (Idioms' "remote rejected ... (failed)") tries again holding
    # the lock; only main moving under it lets the lock and the turn go.
    lock_step pushing
    moved=""
    RETRY_TRIES=3 RETRY_WAIT=1 retry push_held && [ -z "$moved" ] && { drop_lock; return 0; }
    drop_lock
    return 1
}
delete_lock() {
    git push -q --no-verify --force-with-lease="$lock_ref:$(cat "$lock_file")" "$remote" ":$lock_ref" 2>/dev/null
}
# Succeeds once pushed, or once main moved under the lock (set $moved): then
# only a rebase helps, not another try.
push_held() {
    # A lost lock let another lane push meanwhile: the try starts over.
    [ ! -e "$lock_file.lost" ] || { moved=1; return 0; }
    git push -q --no-verify "$remote" HEAD:main && return 0
    echo "land: the push failed; trying again holding the land lock" >&2
    git fetch -q "$remote" main
    git merge-base --is-ancestor FETCH_HEAD HEAD || moved=1
    [ -n "$moved" ]
}
RETRY_TRIES=$tries RETRY_WAIT=0 retry push || { echo "land: gave up after $tries tries" >&2; exit 1; }
drop_lock
rm -f "$pending_steps"
phase push
secs=$(( $(date +%s) - t0 ))
printf '{"domain":"%s","landed":%s,"seconds":%s,"tries":%s,"commit":"%s","phases":{%s}}\n' \
    "$domain" "$(date +%s)" "$secs" "$try" "$(git rev-parse HEAD)" \
    "$(for kv in $(phases); do printf '"%s":%s,' "${kv%%=*}" "${kv#*=}"; done | sed 's/,$//')" \
    >>"$(git rev-parse --git-common-dir)/cpl-land.log"
echo "land: pushed $(git rev-parse --short HEAD) to $remote main in ${secs}s (try $try) [$(phases)]"
git rev-parse HEAD >"$seen"
