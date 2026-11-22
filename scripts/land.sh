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
# the train fails, the holder binary-searches for the first breaking rider,
# marks it refs/locks/solo/<ticket>, rebases the riders behind it onto the
# passing prefix and gates again, so one bad lane drops out and the rest
# land in the same cycle.
# Remote calls retry, so one dropped connection does not lose a turn.
#
# Each landing appends one JSON line per lane (queue wait, gate time, train
# size, splits) to a git note on refs/notes/land, written by the lock holder
# after its push; `git log --notes=land` or scripts/status.py read it.
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
ttl="${CPL_LAND_TTL:-120}"
lock_beat="${CPL_LAND_LOCK_BEAT:-30}"
max_hold="${CPL_LAND_MAX_HOLD:-480}"
gate="${CPL_LAND_GATE:-scripts/check.sh --land}"
me="${CPL_LANE:-$(git config user.name || echo lander)}"
beat="${CPL_LAND_BEAT:-60}"
poll="${CPL_LAND_POLL:-10}"
train_max="${CPL_LAND_TRAIN:-8}"
markers_ttl=21600
name="$( [ -n "${CPL_LAND_URGENT:-}" ] && echo 0 || date +%s%N)-$$-$(printf %s "$me" | tr -c 'A-Za-z0-9_-' '_')"
ticket="$queue/$name"
cd "$(git rev-parse --show-toplevel)"
[ -z "$(git status --porcelain --untracked-files=no)" ] ||
    { echo "land: commit or stash your changes first" >&2; exit 1; }
if [ -z "${CPL_LAND_GATE:-}" ] && [ -f Cargo.toml ] && command -v cargo >/dev/null 2>&1; then
    cargo fmt --all --check >/dev/null ||
        { echo "land: Rust files are not formatted; run: cargo fmt --all" >&2; exit 1; }
fi
empty=$(git hash-object -t tree -w /dev/null)
start=$(git rev-parse HEAD)

held_file=$(mktemp)
lock_hb_pid=""
lock_t0=0
hold_t0=0
phase="taking the lock"
queued=""
back=""
t0=$(date +%s)
gate_s=0
splits=0
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
    export CHECK_BASE=$base
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
    rider_cand=()
    rider_from=()
    tip=$own
    prefix=("$own")
    [ "$train_max" -gt 0 ] && board
    git checkout -q --detach "$tip"
    phase="gating ${#riders[@]} rider(s)"
    relock
    start_lock_heartbeat
    gate_t0=$(date +%s)
    if [ ${#riders[@]} -gt 0 ]; then
        echo "land: gating a train of ${#riders[@]} rider(s): ${riders[*]}" >&2
        run_gate || isolate
    else
        run_gate
    fi
    gate_s=$(( $(date +%s) - gate_t0 ))
    stop_lock_heartbeat
    # The lock rides in the atomic push under a lease: a holder whose lock
    # was taken over (it looked dead) cannot push main under the new holder.
    held=$(held_get)
    phase="pushing"
    lease=()
    refs=("$tip:refs/heads/main")
    if [ -n "$held" ]; then
        token=$(lock_token)
        lease=(--force-with-lease="$lock:$held")
        refs+=("$token:$lock")
    fi
    for r in "${riders[@]}"; do
        refs+=("$(git commit-tree "$empty" -p "$tip" -m "landed: $r"):refs/locks/done/$r")
    done
    if push_main; then
        [ -n "$held" ] && held_set "$token"
        for r in "${riders[@]}"; do
            git push -q "$remote" ":$queue/$r" 2>/dev/null || true
        done
        git push -q "$remote" ":refs/locks/solo/$name" 2>/dev/null || true
        log_landing
        git checkout -q -f "$back"
        git merge -q --ff-only "$tip"
        back=""
        echo "land: pushed $(git rev-parse --short "$tip") to main${riders[*]:+ with ${#riders[@]} rider(s)}"
        exit 0
    fi
    git checkout -q -f "$back"
    back=""
    if [ -n "$held" ] && [ "$(git ls-remote "$remote" "$lock" | cut -f1)" != "$held" ]; then
        held_set ""
        echo "land: lost the land lock during the gate (held $(( $(date +%s) - hold_t0 ))s; another lander took it over), retrying ($attempt/5)" >&2
    else
        release
        echo "land: main moved during the gate (someone pushed without land.sh), retrying ($attempt/5)" >&2
    fi
done
echo "land: gave up after 5 attempts" >&2
exit 1
}

# The atomic push of main, the lock and the done markers. GitHub sometimes
# fails a push on its side ("remote rejected ... (failed)", "unable to write
# file"); while main is still $base and the lock still ours, that is retried
# here rather than costing the train another gate.
push_main() {
    local i out
    for i in 1 2 3 4; do
        out=$(git push -q --atomic "${lease[@]}" "$remote" "${refs[@]}" 2>&1) && return 0
        printf '%s\n' "$out" >&2
        [ "$(git ls-remote "$remote" refs/heads/main | cut -f1)" = "$base" ] || return 1
        [ -z "$held" ] || [ "$(git ls-remote "$remote" "$lock" | cut -f1)" = "$held" ] || return 1
        echo "land: the push failed on the remote's side; main and the lock are unchanged, pushing again ($i/3)" >&2
        [ "$i" -lt 4 ] && sleep $((i * 2))
    done
    return 1
}

retry() {
    local i
    for i in 1 2 3 4 5 6; do
        "$@" && return 0
        sleep $((i * 5))
    done
    "$@"
}

held_get() { [ -f "$held_file" ] && cat "$held_file" || true; }

held_set() {
    printf '%s\n' "$1" > "$held_file.tmp" && mv "$held_file.tmp" "$held_file"
}

stop_lock_heartbeat() {
    if [ -n "${lock_hb_pid:-}" ]; then
        kill "$lock_hb_pid" 2>/dev/null || true
        wait "$lock_hb_pid" 2>/dev/null || true
        lock_hb_pid=""
    fi
}

# The lock's subject says who holds it, what they are doing and for how
# long, so a waiter can print why it waits.
lock_token() {
    git commit-tree "$empty" -m "land lock: $me ($phase, held $(( $(date +%s) - hold_t0 ))s)"
}

# Refresh refs/locks/land periodically while gating so a live holder stays
# locked past $ttl, while a dead holder auto-expires in $ttl and a gate that
# hangs stops refreshing $max_hold after it started.
start_lock_heartbeat() {
    stop_lock_heartbeat
    [ "$lock_beat" -gt 0 ] || return 0
    [ -n "$(held_get)" ] || return 0
    (
        trap 'kill $(jobs -p) 2>/dev/null || true; exit 0' TERM INT
        while :; do
            sleep "$lock_beat" &
            wait $! || exit 0
            if [ "$max_hold" -gt 0 ] && [ $(( $(date +%s) - lock_t0 )) -ge "$max_hold" ]; then
                echo "land: lock hold limit (${max_hold}s) reached; stopping lock heartbeat" >&2
                exit 0
            fi
            cur=$(held_get)
            [ -n "$cur" ] || exit 0
            token=$(lock_token)
            if git push -q --force-with-lease="$lock:$cur" "$remote" "$token:$lock" 2>/dev/null; then
                held_set "$token"
            else
                exit 0
            fi
        done
    ) &
    lock_hb_pid=$!
}

release() {
    stop_lock_heartbeat
    local cur
    cur=$(held_get)
    [ -n "$cur" ] && git push -q --force-with-lease="$lock:$cur" "$remote" ":$lock" 2>/dev/null
    held_set ""
    return 0
}

dequeue() {
    [ -n "$queued" ] && git push -q "$remote" ":$ticket" 2>/dev/null
    queued=""
    return 0
}

cleanup() {
    stop_lock_heartbeat
    if [ -n "$back" ]; then
        git rebase --abort 2>/dev/null || true
        git checkout -q -f "$back" 2>/dev/null || true
    fi
    release
    dequeue
    rm -f "$held_file" "$held_file.tmp"
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
    local out
    if out=$(git push -q --force-with-lease="$ticket:$queued" "$remote" "$next:$ticket" 2>&1); then
        queued=$next
    elif grep -q '^pre-push:' <<<"$out"; then
        printf '%s\n' "$out" >&2
        echo "land: the pre-push hook rejects HEAD; fix it and rerun" >&2
        exit 1
    elif ! carried; then
        queued=$(git ls-remote "$remote" "$ticket" 2>/dev/null | cut -f1) || queued=""
        echo "land: heartbeat failed, still waiting: $(grep -m1 -e '^ ! ' -e 'error:' <<<"$out" | sed 's/^ *//')" >&2
    fi
}

carried() { [ -n "$(retry git ls-remote "$remote" "refs/locks/done/$name")" ]; }

# Returns 0 holding the lock, or 1 when a train already carried our commits.
acquire() {
    local token cur since holder stale_holder
    while :; do
        carried && return 1
        heartbeat
        if ! cur=$(git ls-remote "$remote" "$lock" | cut -f1); then
            sleep "$poll"
            continue
        fi
        stale_holder=""
        if [ -n "$cur" ]; then
            git fetch -q "$remote" "$lock" 2>/dev/null || continue
            since=$(age FETCH_HEAD)
            holder=$(git log -1 --format=%s FETCH_HEAD)
            if [ "$since" -lt "$ttl" ]; then
                echo "land: waiting for $holder (${since}s of ${ttl}s)" >&2
                sleep "$poll"
                continue
            fi
            stale_holder="$holder"
        fi
        holder=$(live_tickets | head -n1)
        if [ -n "$holder" ] && [ "$holder" != "$ticket" ]; then
            echo "land: waiting in line behind ${holder##*/} ($(live_tickets | grep -c .) queued)" >&2
            sleep "$poll"
            continue
        fi
        [ -n "$stale_holder" ] && echo "land: taking over stale lock from $stale_holder" >&2
        hold_t0=$(date +%s)
        token=$(lock_token)
        if git push -q --force-with-lease="$lock:$cur" "$remote" "$token:$lock" 2>/dev/null; then
            held_set "$token"
            lock_t0=$hold_t0
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

# Delete tickets silent past the TTL, solo markers whose lane left the
# queue (it gave up or landed), and done markers nobody collected.
sweep() {
    local sha ref limit refs
    refs=$(retry git ls-remote "$remote" 'refs/locks/*/*' | cut -f2)
    for ref in $refs; do
        case "$ref" in
            "$queue"/*) limit=$ttl ;;
            refs/locks/solo/*)
                limit=$markers_ttl
                grep -qx "$queue/${ref#refs/locks/solo/}" <<<"$refs" || limit=$ttl ;;
            refs/locks/done/*) limit=$markers_ttl ;;
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

# One JSON line per lane in this push, as a note on the lane's last commit.
# Best effort: a lost note never blocks or undoes a landing.
log_landing() {
    local now i r ns c lane_wait
    now=$(date +%s)
    git fetch -q "$remote" "+refs/notes/land:refs/notes/land" 2>/dev/null || true
    note() {
        git notes --ref=land append -m "$(printf '{"lane":"%s","landed":%s,"wait_s":%s,"gate_s":%s,"riders":%s,"splits":%s,"holder":"%s"}' \
            "$1" "$now" "$2" "$gate_s" "${#riders[@]}" "$splits" "$me")" "$3" 2>/dev/null || true
    }
    note "$me" $(( now - t0 - gate_s )) "$own"
    for i in "${!riders[@]}"; do
        r=${riders[$i]}
        ns=${r%%-*}
        lane_wait=null
        [ "$ns" != 0 ] && lane_wait=$(( now - ns / 1000000000 - gate_s ))
        c=${prefix[$((i + 1))]}
        r=${r#*-}
        note "${r#*-}" "$lane_wait" "$c"
    done
    git push -q "$remote" refs/notes/land:refs/notes/land 2>/dev/null ||
        echo "land: could not record the landing in refs/notes/land" >&2
    return 0
}

mark() {
    local kind=$1 r
    shift
    for r in "$@"; do
        git push -q "$remote" "$(git commit-tree "$empty" -m "$kind: $r"):refs/locks/$kind/$r" 2>/dev/null || true
    done
}

# Refresh the lock before each gate a split train takes; $max_hold bounds
# each gate, not the whole train, so a live holder bisecting a long train
# keeps the lock instead of being taken over mid-split.
relock() {
    stop_lock_heartbeat
    local cur token
    cur=$(held_get)
    [ -n "$cur" ] || return 0
    token=$(lock_token)
    if retry git push -q --force-with-lease="$lock:$cur" "$remote" "$token:$lock" 2>/dev/null; then
        held_set "$token"
        lock_t0=$(date +%s)
    else
        echo "land: could not refresh the lock; gating anyway" >&2
    fi
}

# The train failed: find the first breaking rider, mark it solo, drop it,
# rebase the riders behind it onto the passing prefix and gate again, until
# a train passes. If the holder alone fails, exit without marking anyone.
# The gate's output, kept so a failed train can be pinned on a rider.
gate_log=$(git rev-parse --absolute-git-dir)/land-gate.log
run_gate() {
    $gate 2>&1 | tee "$gate_log"
    return "${PIPESTATUS[0]}"
}

# The one rider whose diff touches what the failed gate names (a file a
# compiler error points at, or the crate of a failing test), or nothing.
suspect() {
    local files dirs i hit=() pkg
    files=$(grep -aoE -- '--> [^: ]+' "$gate_log" | cut -c5- | sort -u || true)
    dirs=""
    for pkg in $(grep -aoE 'FAIL \[ *[0-9.]+s\] \([^)]*\) [^ ]+' "$gate_log" | awk '{print $NF}' | sed 's/::.*//' | sort -u); do
        dirs+=$(cargo metadata -q --no-deps --format-version 1 2>/dev/null | python3 -c '
import json, os, sys
for p in json.load(sys.stdin)["packages"]:
    if p["name"] == sys.argv[1]:
        print(os.path.relpath(os.path.dirname(p["manifest_path"])) + "/")' "$pkg" || true)$'\n'
    done
    [ -n "$files$dirs" ] || return 0
    for ((i = 0; i < ${#riders[@]}; i++)); do
        if git diff --name-only "${prefix[$i]}" "${prefix[$((i + 1))]}" | awk -v files="$files" -v dirs="$dirs" '
            BEGIN { n = split(files, f, "\n"); m = split(dirs, d, "\n") }
            { for (j = 1; j <= n; j++) if (f[j] != "" && $0 == f[j]) hit = 1
              for (j = 1; j <= m; j++) if (d[j] != "" && index($0, d[j]) == 1) hit = 1 }
            END { exit !hit }'; then
            hit+=("$i")
        fi
    done
    [ ${#hit[@]} -eq 1 ] && echo "${hit[0]}"
    return 0
}

# Mark rider $1 solo and rebase the riders behind it onto the ones ahead.
drop_rider() {
    local k=$1 i n=${#riders[@]} rest=()
    mark solo "${riders[$k]}"
    for ((i = k + 1; i < n; i++)); do rest+=("$i"); done
    local cands=("${rider_cand[@]}") froms=("${rider_from[@]}") names=("${riders[@]}")
    tip=${prefix[$k]}
    riders=("${names[@]:0:$k}")
    rider_cand=("${cands[@]:0:$k}")
    rider_from=("${froms[@]:0:$k}")
    prefix=("${prefix[@]:0:$((k + 1))}")
    for i in "${rest[@]}"; do
        board_one "${names[$i]}" "${cands[$i]}" "${froms[$i]}"
    done
    git checkout -q -f --detach "$tip"
}

isolate() {
    local good=-1 lo hi mid bad s
    echo "land: the train failed the gate; splitting it" >&2
    # One gate instead of a bisection when the failure points at one rider.
    s=$(suspect)
    if [ -n "$s" ]; then
        bad=${riders[$s]}
        echo "land: the failure points at ${bad} alone; gating the train without it" >&2
        drop_rider "$s"
        phase="gating ${#riders[@]} rider(s) without ${bad##*-}"
        relock
        start_lock_heartbeat
        echo "land: gating ${#riders[@]} rider(s) without ${bad}" >&2
        splits=$((splits + 1))
        if run_gate; then stop_lock_heartbeat; return 0; fi
        stop_lock_heartbeat
    fi
    while :; do
        lo=$good
        hi=${#riders[@]}
        while [ $((hi - lo)) -gt 1 ]; do
            mid=$(( (lo + hi) / 2 ))
            git checkout -q -f --detach "${prefix[$mid]}"
            phase="splitting a failed train of ${#riders[@]}: gating ${mid} rider(s)"
            relock
            start_lock_heartbeat
            echo "land: gating the holder with ${mid} rider(s)" >&2
            splits=$((splits + 1))
            if run_gate; then lo=$mid; else hi=$mid; fi
            stop_lock_heartbeat
        done
        if [ "$lo" -lt 0 ]; then
            echo "land: the holder fails the gate alone; nobody is marked solo" >&2
            exit 1
        fi
        bad=${riders[$lo]}
        echo "land: ${bad} broke the train; marking it solo" >&2
        drop_rider "$lo"
        good=$lo
        [ ${#riders[@]} -gt "$good" ] || return 0
        phase="gating ${#riders[@]} rider(s) without ${bad##*-}"
        relock
        start_lock_heartbeat
        echo "land: gating ${#riders[@]} rider(s) without ${bad}" >&2
        splits=$((splits + 1))
        if run_gate; then stop_lock_heartbeat; return 0; fi
        stop_lock_heartbeat
    done
}

# Rebase one queued lane's commits ($3..$2) onto $tip; on success it rides.
board_one() {
    if git rebase -q --onto "$tip" "$3" "$2" >/dev/null 2>&1; then
        tip=$(git rev-parse HEAD)
        riders+=("$1")
        rider_cand+=("$2")
        rider_from+=("$3")
        prefix+=("$tip")
    else
        git rebase --abort 2>/dev/null || true
        echo "land: ${1} does not apply on top of the train; it lands on its own turn" >&2
    fi
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
        board_one "$r" "$cand" "$from"
    done
    git checkout -q -f "$back"
}

main "$@"; exit
