#!/usr/bin/env bash
# Lands HEAD on main: rebase onto the remote's main, push, and on a rejected
# push (someone landed first) rebase and push again. Nothing is gated here:
# the commit hook ran the commit tier, CI runs the full gate on every main
# tip, and the infra sheriff reverts or fixes a red one. Every commit to land
# needs a `Domain: <d>` trailer. Each landing appends one JSON line (domain,
# seconds, tries) to .git/cpl-land.log; scripts/land_stats.py reads it.
#
# Usage: scripts/land.sh [remote]   (default origin; CPL_LAND_TRIES, default 5)
set -euo pipefail
remote="${1:-origin}"
tries="${CPL_LAND_TRIES:-5}"
t0=$(date +%s)
git fetch -q "$remote" main
for c in $(git rev-list "FETCH_HEAD..HEAD"); do
    git log -1 --format='%(trailers:key=Domain,valueonly)' "$c" | grep -q . ||
        { echo "land: $(git log -1 --format='%h %s' "$c") has no Domain: trailer" >&2; exit 1; }
done
domain=$(git log -1 --format='%(trailers:key=Domain,valueonly,separator=)' HEAD)
for try in $(seq "$tries"); do
    git pull -q --rebase "$remote" main
    if git push -q --no-verify "$remote" HEAD:main; then
        secs=$(( $(date +%s) - t0 ))
        printf '{"domain":"%s","landed":%s,"seconds":%s,"tries":%s,"commit":"%s"}\n' \
            "$domain" "$(date +%s)" "$secs" "$try" "$(git rev-parse HEAD)" \
            >>"$(git rev-parse --git-common-dir)/cpl-land.log"
        echo "land: pushed $(git rev-parse --short HEAD) to $remote main in ${secs}s (try $try)"
        exit 0
    fi
    echo "land: push rejected (try $try of $tries); rebasing again" >&2
done
echo "land: gave up after $tries tries" >&2
exit 1
