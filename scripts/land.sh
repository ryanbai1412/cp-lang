#!/usr/bin/env bash
# Lands HEAD on main without ever making main red: fetch, rebase, run the
# fast gate on the exact rebased tip, push. If main moved during the gate the
# push is rejected and the loop repeats on the new tip.
# Usage: scripts/land.sh [remote]   (a remote name or URL; default origin)
set -euo pipefail
remote="${1:-origin}"
cd "$(git rev-parse --show-toplevel)"
[ -z "$(git status --porcelain --untracked-files=no)" ] ||
    { echo "land: commit or stash your changes first" >&2; exit 1; }
for attempt in 1 2 3 4 5; do
    git fetch -q "$remote" main
    git rebase -q FETCH_HEAD
    scripts/check.sh --fast
    if git push -q "$remote" HEAD:main; then
        echo "land: pushed $(git rev-parse --short HEAD) to main"
        exit 0
    fi
    echo "land: main moved during the gate, retrying ($attempt/5)" >&2
done
echo "land: gave up after 5 attempts; main is moving too fast, try again" >&2
exit 1
