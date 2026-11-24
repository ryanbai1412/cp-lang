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
# Usage: scripts/land.sh [remote]   (default origin; CPL_LAND_TRIES, default 5)
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
remote="${1:-origin}"
tries="${CPL_LAND_TRIES:-5}"
t0=$(date +%s)
git fetch -q "$remote" main
# A checkout behind main lands with main's land.sh, so a fix to landing reaches
# every lane at once instead of after its next rebase.
if [ -z "${CPL_LAND_MAIN:-}" ] && git cat-file -e FETCH_HEAD:scripts/land.sh 2>/dev/null &&
    ! git show FETCH_HEAD:scripts/land.sh | cmp -s - scripts/land.sh; then
    dir="$(git rev-parse --git-common-dir)/cpl-land-main"
    mkdir -p "$dir"
    for f in land.sh retry.sh; do git show "FETCH_HEAD:scripts/$f" >"$dir/$f"; done
    CPL_LAND_MAIN=1 exec bash "$dir/land.sh" "$@"
fi
for c in $(git rev-list "FETCH_HEAD..HEAD"); do
    git log -1 --format='%(trailers:key=Domain,valueonly)' "$c" | grep -q . ||
        { echo "land: $(git log -1 --format='%h %s' "$c") has no Domain: trailer" >&2; exit 1; }
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
pull() {
    before=$(git rev-parse HEAD)
    if git pull -q --rebase --autostash "$remote" main; then
        stash_conflict && return 1
        remeasure
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
    remeasure
}
# Where this commit's hook ran: main's tip before the pull, as far as HEAD has it.
checked=$(git merge-base HEAD FETCH_HEAD 2>/dev/null || git rev-parse FETCH_HEAD)
pull || exit 1
# The hook ran the commit tier on the old base; a rebase past a change to the
# gates themselves (scripts/, the hooks, gen.toml, the justfile) reruns it on the
# rebased tip, staged against main, since passing the old gates says nothing about
# the new ones (5 fixtures passed on an old base after cafd6ffbf's check landed).
gate_paths="scripts .githooks gen.toml justfile"
rechecked=
recheck() {
    moved=$(git diff --name-only "$checked" FETCH_HEAD -- $gate_paths)
    checked=$(git rev-parse FETCH_HEAD)
    [ -n "$moved" ] && [ -x .githooks/pre-commit ] || return 0
    # Once per land: rerunning on every lost push race livelocks landing while
    # main keeps changing scripts/; CI gates the tip that lands.
    [ -z "$rechecked" ] || { echo "land: main changed the gates again; landing on the rerun's result, CI checks this tip" >&2; return 0; }
    rechecked=1
    echo "land: main changed the gates ($(printf '%s\n' $moved | head -3 | tr '\n' ' ' | sed 's/ $//')); rerunning the commit tier on the rebased tip" >&2
    tip=$(git rev-parse HEAD)
    prev=$(trap -p EXIT)
    trap 'git reset -q --soft "$tip"' EXIT
    git reset -q --soft FETCH_HEAD
    # No cpl is handed over: the rebase rewrote main's crate files, and the commit tier
    # rebuilds target/debug/cpl before the corpus audit and the ratchet run it.
    set +e
    .githooks/pre-commit >&2
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
if [ -z "$(git rev-list FETCH_HEAD..HEAD)" ]; then
    echo "land: nothing to land: HEAD is already on main (did the commit or its hook fail?)" >&2
    [ ! -s "$failed_log" ] || { echo "land: the last failed pre-commit ended:" >&2; tail -15 "$failed_log" >&2; }
    exit 1
fi
rm -f "$failed_log"
recheck
# The slices a hook's 25 s cut skipped (a cold build of their tests) run here,
# on the rebased tip, before anything lands.
pending="$(git rev-parse --git-dir)/cpl-pending-slices"
if [ -s "$pending" ]; then
    slices=$(sort -u "$pending" | tr '\n' ' ' | sed 's/ $//')
    echo "land: running the touched-path slices the hook's cut skipped: $slices" >&2
    { uv run --quiet python scripts/affected.py --build-slices $slices >&2 &&
        uv run --quiet python scripts/affected.py --run-slices $slices >&2; } ||
        { echo "land: a touched-path slice fails at the rebased tip; fix it and land again: uv run scripts/affected.py --run-slices $slices" >&2; exit 1; }
    rm -f "$pending"
fi
# Pending files are per worktree (git-dir): one lane's land neither runs nor
# deletes another's. The commit-tier steps a hook's cut never reached (b3b3c30dd's came before its
# wasm check), on the rebased tip staged against main as the hook stages them,
# and again on the new tip after a lost push race.
pending_steps="$(git rev-parse --git-dir)/cpl-pending-steps"
steps_runs=0
run_pending_steps() {
    [ -s "$pending_steps" ] || return 0
    steps=$(sort -u "$pending_steps" | tr '\n' ',' | sed 's/,$//')
    # Once more after a lost race, as recheck: every rerun gives main another
    # window to move, and at its pace one per race livelocked landing (5 tries).
    steps_runs=$((steps_runs + 1))
    [ "$steps_runs" -le 2 ] || { echo "land: main moved again; landing on the skipped steps' rerun, CI checks this tip" >&2; return 0; }
    echo "land: running the commit-tier steps the hook's cut skipped: $steps" >&2
    tip=$(git rev-parse HEAD)
    prev=$(trap -p EXIT)
    trap 'git reset -q --soft "$tip"' EXIT
    git reset -q --soft FETCH_HEAD
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
# a std trait with no traits.toml case while its hook was cut).
std_changed=$(git diff --name-only FETCH_HEAD HEAD -- 'std/*.cp' spec/conformance/traits.toml)
if [ -n "$std_changed" ] && [ -f scripts/spec-gate.py ]; then
    uv run --quiet python scripts/spec-gate.py traits >&2 ||
        { echo "land: a std trait impl has no spec/conformance/traits.toml case; write it, then: scripts/spec-gate.py traits --write" >&2; exit 1; }
fi
pending_cases="$(git rev-parse --git-dir)/cpl-pending-cases"
# A std pub fn signature change runs every case and corpus program that calls
# it (8f1685382 changed splice and three cases went red on main).
sig_callers=
if [ -n "$std_changed" ] && [ -f scripts/affected.py ]; then
    sig_callers=$(uv run --quiet python scripts/affected.py --sig-callers FETCH_HEAD)
    printf '%s\n' $sig_callers | grep '^spec/conformance/' >>"$pending_cases" || :
fi
if [ -s "$pending_cases" ]; then
    cases=$(sort -u "$pending_cases" | while read -r c; do [ ! -f "$c" ] || echo "$c"; done | tr '\n' ' ' | sed 's/ $//')
    if [ -n "$cases" ]; then
        git diff --quiet HEAD -- crates std Cargo.toml Cargo.lock ||
            { echo "land: uncommitted changes under crates/ or std/ would be tested in place of what lands; commit or stash them, then land again" >&2; exit 1; }
        echo "land: running the staged conformance cases the hook's cut skipped: $cases" >&2
        uv run --quiet python scripts/conform_staged.py --build $cases >&2 ||
            { echo "land: a staged conformance case fails at the rebased tip; fix it and land again: uv run scripts/conform_staged.py --build $cases" >&2; exit 1; }
    fi
    rm -f "$pending_cases"
fi
sig_programs=$(printf '%s\n' $sig_callers | grep -v '^spec/conformance/' | tr '\n' ' ' | sed 's/ $//') || :
if [ -n "$sig_programs" ]; then
    echo "land: cpl check of the corpus programs that call a changed std signature: $sig_programs" >&2
    cargo build -q -p cpl-cli && for f in $sig_programs; do
        "${CARGO_TARGET_DIR:-target}/debug/cpl" check "$f" >&2 ||
            { echo "land: $f no longer checks against the changed std signature; update it in this change" >&2; exit 1; }
    done
fi
changed=$(git diff --name-only --diff-filter=ACMR FETCH_HEAD HEAD)
# The touched crates' cpl.wasm budgets, from the named build .githooks/post-commit
# started (scripts/wasm-warm.sh), never waiting on one: 849a0be7d's commit hook
# was cut before its wasm check and grew cpl_lsp 544 B gz past its budget. Held to
# main's sizes, a crate over its budget refuses only if it grew, so main never gets
# worse while lanes cut a crate back under.
wasm_touched=$(printf '%s\n' "$changed" | sed -n 's|^crates/\([^/]*\)/.*|\1|p' | sort -u)
if [ -n "$wasm_touched" ] && [ -x scripts/wasm-warm.sh ]; then
    if ! wasm=$(scripts/wasm-warm.sh --current $wasm_touched); then
        echo "land: no current wasm build; CI checks it" >&2
    elif ! parent=$(scripts/wasm-warm.sh --parent FETCH_HEAD $wasm_touched); then
        echo "land: no wasm sizes of main's tip to compare with; CI checks it" >&2
    else
        node web/scripts/wasm-crates.mjs --data "$wasm" --parent "$parent" >&2 ||
            { echo "land: a cpl.wasm crate grew past main while over its budget, or crossed its budget (above), touched here or not; cut it at its source and land again" >&2; exit 1; }
    fi
fi
seen="$(git rev-parse --git-dir)/cpl-last-land"
if [ -s "$seen" ] && git cat-file -e "$(cat "$seen")^{commit}" 2>/dev/null; then
    product=$(git diff --stat "$(cat "$seen")" FETCH_HEAD -- AGENTS.md handbook plan/product)
    [ -z "$product" ] || printf 'land: product direction or the handbook changed since your last land; read it:\n%s\n' "$product"
fi
if [ -n "$changed" ]; then
    marks=$(git grep -nE '^(<{7}|>{7}|\|{7})( |$)' HEAD -- $changed ':!*.png' ':!*.gz' || [ $? = 1 ])
    [ -z "$marks" ] || { echo "land: merge conflict markers:"$'\n'"$marks" >&2; exit 1; }
fi
# The rebase rewrote main's crate files: corpus_fmt_dirty refuses a stale cpl, so build it.
[ ! -f scripts/debt-ratchet.py ] || [ ! -f Cargo.toml ] || cargo build -q -p cpl-cli ||
    { echo "land: cargo build -p cpl-cli failed at the rebased tip; the ratchet runs that cpl" >&2; exit 1; }
[ ! -f scripts/debt-ratchet.py ] || uv run --quiet python scripts/debt-ratchet.py --check >/dev/null ||
    { uv run --quiet python scripts/debt-ratchet.py --check >&2; exit 1; }
# A decision number is taken once it is on main (decision 505): a heading
# main never had, even as a `--reserve` stub, would collide with another lane.
[ ! -f scripts/specgate/decisions.py ] || uv run --quiet python scripts/spec-gate.py decisions --check-landing FETCH_HEAD || exit 1
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
if [ -n "$crates" ]; then
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
try=0
push() {
    try=$((try + 1))
    if [ "$try" != 1 ]; then
        pull || exit 1
        recheck
        run_pending_steps
        # What landed meanwhile can break this change's files: check again.
        [ -z "$users" ] || refuse_if_broken "$(cargo check -q $users --all-targets 2>&1)"
    fi
    git push -q --no-verify "$remote" HEAD:main
}
RETRY_TRIES=$tries RETRY_WAIT=0 retry push || { echo "land: gave up after $tries tries" >&2; exit 1; }
rm -f "$pending_steps"
secs=$(( $(date +%s) - t0 ))
printf '{"domain":"%s","landed":%s,"seconds":%s,"tries":%s,"commit":"%s"}\n' \
    "$domain" "$(date +%s)" "$secs" "$try" "$(git rev-parse HEAD)" \
    >>"$(git rev-parse --git-common-dir)/cpl-land.log"
echo "land: pushed $(git rev-parse --short HEAD) to $remote main in ${secs}s (try $try)"
git rev-parse HEAD >"$seen"
