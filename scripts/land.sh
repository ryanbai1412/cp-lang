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
for c in $(git rev-list "FETCH_HEAD..HEAD"); do
    git log -1 --format='%(trailers:key=Domain,valueonly)' "$c" | grep -q . ||
        { echo "land: $(git log -1 --format='%h %s' "$c") has no Domain: trailer" >&2; exit 1; }
done
domain=$(git log -1 --format='%(trailers:key=Domain,valueonly,separator=)' HEAD)
# A conflict only in generated files is resolved by taking main's copy; CI's
# regen job regenerates it on the pushed tip. Any other conflict stops here.
generated='^(spec/conformance/|skills/|pnpm-lock\.yaml$)'
ratchet=scripts/debt-ratchet.json index=corpus/index.json
rebasing() { [ -d "$(git rev-parse --git-path rebase-merge)" ] || [ -d "$(git rev-parse --git-path rebase-apply)" ]; }
pull() {
    git pull -q --rebase --autostash "$remote" main && return 0
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
        if [ -z "$conflicts" ] || printf '%s\n' "$conflicts" | grep -qvE "$generated"; then
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
}
pull
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
changed=$(git diff --name-only --diff-filter=ACMR FETCH_HEAD HEAD)
seen="$(git rev-parse --git-dir)/cpl-last-land"
if [ -s "$seen" ] && git cat-file -e "$(cat "$seen")^{commit}" 2>/dev/null; then
    product=$(git diff --stat "$(cat "$seen")" FETCH_HEAD -- AGENTS.md handbook plan/product)
    [ -z "$product" ] || printf 'land: product direction or the handbook changed since your last land; read it:\n%s\n' "$product"
fi
if [ -n "$changed" ]; then
    marks=$(git grep -nE '^(<{7}|>{7}|\|{7})( |$)' HEAD -- $changed ':!*.png' ':!*.gz' || [ $? = 1 ])
    [ -z "$marks" ] || { echo "land: merge conflict markers:"$'\n'"$marks" >&2; exit 1; }
fi
[ ! -f scripts/debt-ratchet.py ] || uv run --quiet python scripts/debt-ratchet.py --check >/dev/null ||
    { uv run --quiet python scripts/debt-ratchet.py --check >&2; exit 1; }
# A decision number is taken once it is on main (decision 505): a heading
# main never had, even as a `--reserve` stub, would collide with another lane.
[ ! -f scripts/specgate/decisions.py ] || uv run --quiet python scripts/spec-gate.py decisions --check-landing FETCH_HEAD || exit 1
crates=$(printf '%s\n' "$changed" | sed -n 's|^crates/\([^/]*\)/.*\.rs$|-p \1|p' | sort -u | tr '\n' ' ')
[ -z "$crates" ] || cargo fmt --check $crates >/dev/null 2>&1 ||
    { echo "land: not formatted; run: cargo fmt $crates" >&2; exit 1; }
# The files of this change that rustc rejects (error[E...]) in cargo's output
# on stdin: two changes each green alone can combine into one that doesn't
# compile, and that one must not land.
own_compile_errors() {
    awk '/^(error|warning)/ {e = /^error\[E/} e && /^ *--> / {sub(/^ *--> /, ""); sub(/:.*/, ""); print}' |
        sort -u | grep -Fx -- "$changed" || [ $? = 1 ]
}
refuse_if_broken() {
    bad=$(printf '%s\n' "$1" | own_compile_errors)
    [ -z "$bad" ] || { printf '%s\n' "$1" | grep -E '^error|-->' | head -20 >&2
        echo "land: $(echo $bad) no longer compiles at the rebased tip; fix it and land again" >&2; exit 1; }
}
# Clippy as CI runs it (-D warnings, every target) on the touched crates and
# the workspace crates that depend on them, at the rebased tip. A compile
# error in this change's own files refuses the land; past
# CPL_LAND_CLIPPY_SECS (cold cache) or on any other failure it warns and
# lands, leaving it to CI (whose regen job applies `clippy --fix`).
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
    out=$(timeout "${CPL_LAND_CLIPPY_SECS:-40}" cargo clippy -q $users --all-targets -- -D warnings 2>&1)
    rc=$?
    set -e
    echo "land: clippy on $(printf '%s\n' $users | grep -vc -- -p) crate(s) in $(( $(date +%s) - gate_start ))s" >&2
    case "$rc" in
        0) ;;
        124) echo "land: clippy over ${CPL_LAND_CLIPPY_SECS:-40}s; landing, CI will check it" >&2 ;;
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
        # What landed meanwhile can break this change's files: check again.
        [ -z "$users" ] || refuse_if_broken "$(timeout "${CPL_LAND_CLIPPY_SECS:-40}" cargo check -q $users --all-targets 2>&1)"
    fi
    git push -q --no-verify "$remote" HEAD:main
}
RETRY_TRIES=$tries RETRY_WAIT=0 retry push || { echo "land: gave up after $tries tries" >&2; exit 1; }
secs=$(( $(date +%s) - t0 ))
printf '{"domain":"%s","landed":%s,"seconds":%s,"tries":%s,"commit":"%s"}\n' \
    "$domain" "$(date +%s)" "$secs" "$try" "$(git rev-parse HEAD)" \
    >>"$(git rev-parse --git-common-dir)/cpl-land.log"
echo "land: pushed $(git rev-parse --short HEAD) to $remote main in ${secs}s (try $try)"
git rev-parse HEAD >"$seen"
