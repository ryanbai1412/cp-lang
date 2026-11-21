#!/usr/bin/env bash
# Lands HEAD on main without ever making main red: fetch, rebase, run the
# fast gate on the exact rebased tip, push. Landers take turns through a lock
# ref on the remote, so main holds still while the gate runs instead of every
# lane re-gating against a moving tip. A lock older than its TTL is presumed
# abandoned and may be taken over.
#
# Waiters queue in arrival order through ticket refs (refs/locks/queue/<ns>-
# <pid>-<lane>) that they heartbeat while waiting; a ticket whose heartbeat
# stops is skipped, and one silent past the TTL is deleted. A ticket's parent
# is the waiter's HEAD, so the lock holder can carry waiters in a train: it
# rebases up to CPL_LAND_TRAIN queued lanes onto its own commits, gates the
# train once and pushes it, marking each rider refs/locks/done/<ticket>. If
# the train fails the gate, its riders are marked refs/locks/solo/<ticket>
# (they then only land alone) and the holder gates and lands by itself.
# Remote calls retry, so one dropped connection does not lose a turn.
#
# Usage: scripts/land.sh [remote]   (a remote name or URL; default origin)
#        CPL_LANE names you in the lock (default: git user.name).
#        CPL_LAND_GATE replaces the gate (tests only; pre-push still checks).
#        CPL_LAND_TRAIN caps riders per train (default 8; 0 lands alone).
#        CPL_LAND_URGENT=1 queues ahead of everyone: only for fixing a red
#        main or the landing machinery itself.
set -euo pipefail

# Everything runs inside main so a rebase that rewrites this file mid-run
# cannot change the commands bash has not read yet.
main() {
remote="${1:-origin}"
lock=refs/locks/land
queue=refs/locks/queue
ttl="${CPL_LAND_TTL:-1200}"
gate="${CPL_LAND_GATE:-scripts/check.sh --fast}"
me="${CPL_LANE:-$(git config user.name || echo lander)}"
beat="${CPL_LAND_BEAT:-120}"
poll="${CPL_LAND_POLL:-20}"
train_max="${CPL_LAND_TRAIN:-8}"
markers_ttl=21600
name="$( [ -n "${CPL_LAND_URGENT:-}" ] && echo 0 || date +%s%N)-$$-$(printf %s "$me" | tr -c 'A-Za-z0-9_-' '_')"
ticket="$queue/$name"
cd "$(git rev-parse --show-toplevel)"
[ -z "$(git status --porcelain --untracked-files=no)" ] ||
    { echo "land: commit or stash your changes first" >&2; exit 1; }
empty=$(git hash-object -t tree -w /dev/null)
start=$(git rev-parse HEAD)

held=""
queued=""
back=""
trap 'cleanup' EXIT
trap 'exit 130' INT TERM HUP

for attempt in 1 2 3 4 5; do
    if ! acquire; then
        rode
        exit 0
    fi
    sweep
    retry git fetch -q "$remote" main
    base=$(git rev-parse FETCH_HEAD)
    git rebase -q "$base" || {
        git rebase --abort
        echo "land: HEAD conflicts with main; rebase and resolve by hand, then rerun" >&2
        exit 1
    }
    own=$(git rev-parse HEAD)
    back=$(git symbolic-ref -q --short HEAD || echo "$own")
    if [ "$own" = "$base" ]; then
        echo "land: nothing to land; HEAD is already on main"
        exit 0
    fi
    riders=()
    tip=$own
    [ "$train_max" -gt 0 ] && board
    git checkout -q --detach "$tip"
    if [ ${#riders[@]} -gt 0 ]; then
        echo "land: gating a train of ${#riders[@]} rider(s): ${riders[*]}" >&2
        if ! $gate; then
            echo "land: the train failed the gate; marking its riders solo and gating alone" >&2
            mark solo "${riders[@]}"
            riders=()
            tip=$own
            git checkout -q -f --detach "$tip"
            $gate
        fi
    else
        $gate
    fi
    refs=("$tip:refs/heads/main")
    for r in "${riders[@]}"; do
        refs+=("$(git commit-tree "$empty" -p "$tip" -m "landed: $r"):refs/locks/done/$r")
    done
    if git push -q --atomic "$remote" "${refs[@]}"; then
        for r in "${riders[@]}"; do
            git push -q "$remote" ":$queue/$r" 2>/dev/null || true
        done
        git push -q "$remote" ":refs/locks/solo/$name" 2>/dev/null || true
        git checkout -q -f "$back"
        git merge -q --ff-only "$tip"
        back=""
        echo "land: pushed $(git rev-parse --short "$tip") to main${riders[*]:+ with ${#riders[@]} rider(s)}"
        exit 0
    fi
    git checkout -q -f "$back"
    back=""
    release
    echo "land: main moved during the gate (someone pushed without land.sh), retrying ($attempt/5)" >&2
done
echo "land: gave up after 5 attempts" >&2
exit 1
}

retry() {
    local i
    for i in 1 2 3 4 5 6; do
        "$@" && return 0
        sleep $((i * 5))
    done
    "$@"
}

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

cleanup() {
    if [ -n "$back" ]; then
        git rebase --abort 2>/dev/null || true
        git checkout -q -f "$back" 2>/dev/null || true
    fi
    release
    dequeue
}

age() { echo $(( $(date +%s) - $(git log -1 --format=%ct "$1") )); }

# Live tickets (heartbeat within $beat), oldest first, as "<ref>".
live_tickets() {
    local sha ref
    retry git ls-remote "$remote" "$queue/*" | sort -k2 | while read -r sha ref; do
        git fetch -q "$remote" "$ref" 2>/dev/null || continue
        [ "$(age FETCH_HEAD)" -lt "$beat" ] && echo "$ref"
    done
    return 0
}

# Heartbeat our ticket. The lease means a holder that deleted it after
# carrying us is never undone by a late heartbeat.
heartbeat() {
    local next
    next=$(git commit-tree "$empty" -p "$start" -m "land ticket: $me")
    if git push -q --force-with-lease="$ticket:$queued" "$remote" "$next:$ticket" 2>/dev/null; then
        queued=$next
    elif ! carried; then
        queued=$(git ls-remote "$remote" "$ticket" 2>/dev/null | cut -f1) || queued=""
        echo "land: heartbeat failed; still waiting" >&2
    fi
}

carried() { [ -n "$(retry git ls-remote "$remote" "refs/locks/done/$name")" ]; }

# Returns 0 holding the lock, or 1 when a train already carried our commits.
acquire() {
    local token cur since holder
    while :; do
        carried && return 1
        heartbeat
        if ! cur=$(git ls-remote "$remote" "$lock" | cut -f1); then
            sleep "$poll"
            continue
        fi
        if [ -n "$cur" ]; then
            git fetch -q "$remote" "$lock" 2>/dev/null || continue
            since=$(age FETCH_HEAD)
            holder=$(git log -1 --format=%s FETCH_HEAD)
            if [ "$since" -lt "$ttl" ]; then
                echo "land: waiting for $holder (${since}s of ${ttl}s)" >&2
                sleep "$poll"
                continue
            fi
            echo "land: taking over stale lock from $holder" >&2
        fi
        holder=$(live_tickets | head -n1)
        if [ -n "$holder" ] && [ "$holder" != "$ticket" ]; then
            echo "land: waiting in line behind ${holder##*/}" >&2
            sleep "$poll"
            continue
        fi
        token=$(git commit-tree "$empty" -m "land lock: $me")
        if git push -q --force-with-lease="$lock:$cur" "$remote" "$token:$lock" 2>/dev/null; then
            held=$token
            if carried; then
                release
                return 1
            fi
            dequeue
            return 0
        fi
    done
}

rode() {
    retry git push -q "$remote" ":refs/locks/done/$name" 2>/dev/null || true
    queued=""
    retry git fetch -q "$remote" main
    if git rebase -q FETCH_HEAD 2>/dev/null; then
        echo "land: carried to main in a train; HEAD is now $(git rev-parse --short HEAD)"
    else
        git rebase --abort 2>/dev/null || true
        echo "land: carried to main in a train, but HEAD did not rebase onto it cleanly; rebase by hand" >&2
    fi
}

# Delete tickets silent past the TTL and done/solo markers nobody collected.
sweep() {
    local sha ref limit
    retry git ls-remote "$remote" 'refs/locks/*/*' | while read -r sha ref; do
        case "$ref" in
            "$queue"/*) limit=$ttl ;;
            refs/locks/done/*|refs/locks/solo/*) limit=$markers_ttl ;;
            *) continue ;;
        esac
        [ "$ref" = "$ticket" ] && continue
        git fetch -q "$remote" "$ref" 2>/dev/null || continue
        if [ "$(age FETCH_HEAD)" -ge "$limit" ]; then
            git push -q "$remote" ":$ref" 2>/dev/null || true
        fi
    done
    return 0
}

mark() {
    local kind=$1 r
    shift
    for r in "$@"; do
        git push -q "$remote" "$(git commit-tree "$empty" -m "$kind: $r"):refs/locks/$kind/$r" 2>/dev/null || true
    done
}

# Rebase queued lanes, oldest first, onto $tip; a lane that does not apply
# cleanly or was marked solo waits for its own turn.
board() {
    local ref r solo cand from
    solo=$(retry git ls-remote "$remote" 'refs/locks/solo/*' | cut -f2)
    for ref in $(live_tickets); do
        [ ${#riders[@]} -lt "$train_max" ] || break
        r=${ref#"$queue"/}
        [ "$ref" = "$ticket" ] && continue
        grep -qx "refs/locks/solo/$r" <<<"$solo" && continue
        git fetch -q "$remote" "$ref" 2>/dev/null || continue
        cand=$(git rev-parse -q --verify FETCH_HEAD^1) || continue
        from=$(git merge-base "$cand" "$base") || continue
        if git rebase -q --onto "$tip" "$from" "$cand" >/dev/null 2>&1; then
            tip=$(git rev-parse HEAD)
            riders+=("$r")
        else
            git rebase --abort 2>/dev/null || true
            echo "land: ${r} does not apply on top of the train; it lands on its own turn" >&2
        fi
    done
    git checkout -q -f "$back"
}

main "$@"; exit
