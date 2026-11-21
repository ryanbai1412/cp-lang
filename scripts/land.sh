#!/usr/bin/env bash
# Lands HEAD on main without ever making main red: fetch, rebase, run the
# fast gate on the exact rebased tip, push. Landers take turns through a lock
# ref on the remote, so main holds still while the gate runs instead of every
# lane re-gating against a moving tip. A lock older than its TTL is presumed
# abandoned and may be taken over.
# Usage: scripts/land.sh [remote]   (a remote name or URL; default origin)
#        CPL_LANE names you in the lock (default: git user.name).
#        CPL_LAND_GATE replaces the gate (tests only; pre-push still checks).
set -euo pipefail
remote="${1:-origin}"
lock=refs/locks/land
ttl="${CPL_LAND_TTL:-1200}"
gate="${CPL_LAND_GATE:-scripts/check.sh --fast}"
me="${CPL_LANE:-$(git config user.name || echo lander)}"
cd "$(git rev-parse --show-toplevel)"
[ -z "$(git status --porcelain --untracked-files=no)" ] ||
    { echo "land: commit or stash your changes first" >&2; exit 1; }

held=""
release() {
    [ -n "$held" ] || return 0
    git push -q --force-with-lease="$lock:$held" "$remote" ":$lock" 2>/dev/null || true
    held=""
}
trap release EXIT

acquire() {
    local empty token cur since holder
    empty=$(git hash-object -t tree -w /dev/null)
    while :; do
        cur=$(git ls-remote "$remote" "$lock" | cut -f1)
        if [ -n "$cur" ]; then
            git fetch -q "$remote" "$lock"
            since=$(( $(date +%s) - $(git log -1 --format=%ct FETCH_HEAD) ))
            holder=$(git log -1 --format=%s FETCH_HEAD)
            if [ "$since" -lt "$ttl" ]; then
                echo "land: waiting for $holder (${since}s of ${ttl}s)" >&2
                sleep 20
                continue
            fi
            echo "land: taking over stale lock from $holder" >&2
        fi
        token=$(git commit-tree "$empty" -m "land lock: $me")
        if git push -q --force-with-lease="$lock:$cur" "$remote" "$token:$lock" 2>/dev/null; then
            held=$token
            return 0
        fi
    done
}

for attempt in 1 2 3 4 5; do
    acquire
    git fetch -q "$remote" main
    git rebase -q FETCH_HEAD || {
        git rebase --abort
        echo "land: HEAD conflicts with main; rebase and resolve by hand, then rerun" >&2
        exit 1
    }
    $gate
    if git push -q "$remote" HEAD:main; then
        echo "land: pushed $(git rev-parse --short HEAD) to main"
        exit 0
    fi
    release
    echo "land: main moved during the gate (someone pushed without land.sh), retrying ($attempt/5)" >&2
done
echo "land: gave up after 5 attempts" >&2
exit 1
