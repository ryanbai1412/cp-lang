#!/usr/bin/env bash
# Lands HEAD on main: rebase onto the remote's main, push, and on a rejected
# push (someone landed first) rebase and push again. Nothing is gated here:
# the commit hook ran the commit tier, CI runs the full gate on every main
# tip, and the infra sheriff reverts or fixes a red one. Every commit to land
# needs a `Domain: <d>` trailer. Before the first push it runs the checks
# behind most red mains, in a few seconds: conflict markers in the changed
# files, the debt ratchet, `cargo fmt --check` on the touched crates, and
# generated files (regenerated and committed, as CI's regen job would).
# It also says when AGENTS.md or plan/product/ changed since this worktree's
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
git pull -q --rebase "$remote" main
changed=$(git diff --name-only --diff-filter=ACMR FETCH_HEAD HEAD)
seen="$(git rev-parse --git-dir)/cpl-last-land"
if [ -s "$seen" ] && git cat-file -e "$(cat "$seen")^{commit}" 2>/dev/null; then
    product=$(git diff --stat "$(cat "$seen")" FETCH_HEAD -- AGENTS.md plan/product)
    [ -z "$product" ] || printf 'land: product direction changed since your last land; read it:\n%s\n' "$product"
fi
if [ -n "$changed" ]; then
    marks=$(git grep -nE '^(<{7}|>{7}|\|{7})( |$)' HEAD -- $changed ':!*.png' ':!*.gz' || true)
    [ -z "$marks" ] || { echo "land: merge conflict markers:"$'\n'"$marks" >&2; exit 1; }
fi
[ ! -f scripts/debt-ratchet.py ] || python3 scripts/debt-ratchet.py --check >/dev/null ||
    { python3 scripts/debt-ratchet.py --check >&2; exit 1; }
crates=$(printf '%s\n' "$changed" | sed -n 's|^crates/\([^/]*\)/.*\.rs$|-p \1|p' | sort -u | tr '\n' ' ')
[ -z "$crates" ] || cargo fmt --check $crates >/dev/null 2>&1 ||
    { echo "land: not formatted; run: cargo fmt $crates" >&2; exit 1; }
regen=$([ ! -x scripts/regen.sh ] || scripts/regen.sh)
if [ -n "$regen" ]; then
    git add -- $regen
    git commit -q --no-verify -m "factory: regenerate generated files" -m "Domain: $domain"
    echo "land: regenerated $(echo $regen)"
fi
for try in $(seq "$tries"); do
    [ "$try" = 1 ] || git pull -q --rebase "$remote" main
    if git push -q --no-verify "$remote" HEAD:main; then
        secs=$(( $(date +%s) - t0 ))
        printf '{"domain":"%s","landed":%s,"seconds":%s,"tries":%s,"commit":"%s"}\n' \
            "$domain" "$(date +%s)" "$secs" "$try" "$(git rev-parse HEAD)" \
            >>"$(git rev-parse --git-common-dir)/cpl-land.log"
        echo "land: pushed $(git rev-parse --short HEAD) to $remote main in ${secs}s (try $try)"
        git rev-parse HEAD >"$seen"
        exit 0
    fi
    echo "land: push rejected (try $try of $tries); rebasing again" >&2
done
echo "land: gave up after $tries tries" >&2
exit 1
