#!/usr/bin/env bash
# Lands HEAD on main without ever making main red: fetch, rebase, run the
# fast gate on the exact rebased tip, push. Landers take turns through a lock
# ref on the remote, so main holds still while the gate runs instead of every
# lane re-gating against a moving tip. A lock older than its TTL is presumed
# abandoned and may be taken over. Waiters queue in arrival order through
# ticket refs they heartbeat while waiting; a ticket whose heartbeat stops is
# skipped. Remote calls retry, so one dropped connection does not lose a turn.
# Usage: scripts/land.sh [remote]   (a remote name or URL; default origin)
#        CPL_LANE names you in the lock (default: git user.name).
#        CPL_LAND_GATE replaces the gate (tests only; pre-push still checks).
set -euo pipefail
remote="${1:-origin}"
lock=refs/locks/land
ttl="${CPL_LAND_TTL:-1200}"
gate="${CPL_LAND_GATE:-scripts/check.sh --fast}"
me="${CPL_LANE:-$(git config user.name || echo lander)}"
beat="${CPL_LAND_BEAT:-120}"
ticket="refs/locks/queue/$(date +%s%N)-$$-$(printf %s "$me" | tr -c 'A-Za-z0-9_-' '_')"
cd "$(git rev-parse --show-toplevel)"
[ -z "$(git status --porcelain --untracked-files=no)" ] ||
    { echo "land: commit or stash your changes first" >&2; exit 1; }

retry() {
    local i
    for i in 1 2 3 4 5 6; do
        "$@" && return 0
        sleep $((i * 5))
    done
    "$@"
}

held=""
queued=""
release() {
    [ -n "$held" ] && git push -q --force-with-lease="$lock:$held" "$remote" ":$lock" 2>/dev/null
    held=""
    return 0
}
dequeue() {
    [ -n "$queued" ] && git push -q "$remote" ":$ticket" 2>/dev/null
    queued=""
    return 0
}
trap 'release; dequeue' EXIT

# The oldest ticket with a live heartbeat, or nothing.
first_in_line() {
    local sha ref
    retry git ls-remote "$remote" 'refs/locks/queue/*' | sort -k2 | while read -r sha ref; do
        git fetch -q "$remote" "$ref" 2>/dev/null || continue
        if [ $(( $(date +%s) - $(git log -1 --format=%ct FETCH_HEAD) )) -lt "$beat" ]; then
            echo "$ref"
            break
        fi
    done
}

acquire() {
    local empty token cur since holder
    empty=$(git hash-object -t tree -w /dev/null)
    while :; do
        retry git push -q -f "$remote" "$(git commit-tree "$empty" -m "land ticket: $me"):$ticket"
        queued=$ticket
        cur=$(retry git ls-remote "$remote" "$lock" | cut -f1)
        if [ -n "$cur" ]; then
            retry git fetch -q "$remote" "$lock"
            since=$(( $(date +%s) - $(git log -1 --format=%ct FETCH_HEAD) ))
            holder=$(git log -1 --format=%s FETCH_HEAD)
            if [ "$since" -lt "$ttl" ]; then
                echo "land: waiting for $holder (${since}s of ${ttl}s)" >&2
                sleep 20
                continue
            fi
            echo "land: taking over stale lock from $holder" >&2
        fi
        holder=$(first_in_line)
        if [ -n "$holder" ] && [ "$holder" != "$ticket" ]; then
            echo "land: waiting in line behind ${holder##*/}" >&2
            sleep 20
            continue
        fi
        token=$(git commit-tree "$empty" -m "land lock: $me")
        if git push -q --force-with-lease="$lock:$cur" "$remote" "$token:$lock" 2>/dev/null; then
            held=$token
            dequeue
            return 0
        fi
    done
}

for attempt in 1 2 3 4 5; do
    acquire
    retry git fetch -q "$remote" main
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
