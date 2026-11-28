#!/usr/bin/env bash
# lane-j-check's landing queue: the one runner that pushes to lane-j-check (coordinator 04:34).
#
#   scripts/lane-queue.sh [--once]
#
# A lane hands in work with `git push origin <sha>:refs/heads/queue/<lane>-<name>` (a branch
# push runs only the cheap checks). Oldest first, the runner rebases each queue branch's
# commits not yet on lane-j-check onto its tip, runs the gates through the pre-push hook
# (build, clippy on touched crates, lane tests, the shrink-only ratchet), pushes, and deletes
# the branch. A branch that doesn't rebase or fails a gate stays, its reason in
# target/lane-queue/<branch>.log; a branch whose head moves is taken again.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 1
mkdir -p target/lane-queue
log() { echo "lane-queue $(date -u +%H:%M:%S) $*" | tee -a target/lane-queue/queue.log; }
once=0
[ "${1:-}" = --once ] && once=1

take() {  # $1 = queue branch name, e.g. queue/c-clippy
    local branch=$1 head out=target/lane-queue/${1//\//-}.log
    # The gates measure against the lane tip the push replaces, so that tip must be here.
    git fetch -q origin lane-j-check "+refs/heads/$branch:refs/remotes/origin/$branch" || return 1
    head=$(git rev-parse "origin/$branch")
    [ "$(cat "$out.head" 2>/dev/null)" = "$head" ] && return 1  # failed before, unchanged
    # A gate or cherry-pick stopped partway leaves the tree dirty, and then no checkout works.
    if ! git checkout -q --detach origin/lane-j-check 2>>target/lane-queue/queue.log; then
        log "$branch ($head): the runner's tree was dirty; forcing it back to lane-j-check"
        git cherry-pick --abort >/dev/null 2>&1
        git checkout -q --force --detach origin/lane-j-check && git clean -fdxq -e target/ \
            || { log "$branch ($head): checkout still fails; see queue.log"; return 1; }
    fi
    if ! git cherry-pick --empty=drop $(git rev-list --reverse --no-merges "origin/lane-j-check..$head") >"$out" 2>&1; then
        git cherry-pick --abort >/dev/null 2>&1
        echo "$head" >"$out.head"
        log "$branch ($head): doesn't rebase onto lane-j-check; see $out"
        return 1
    fi
    if [ "$(git rev-parse HEAD)" = "$(git rev-parse origin/lane-j-check)" ]; then
        git push -q origin --delete "$branch" >/dev/null 2>&1
        log "$branch ($head): already on lane-j-check"
        return 0
    fi
    local tip t0=$SECONDS
    tip=$(git rev-parse HEAD)
    if CPL_LANE_QUEUE=1 git push origin HEAD:lane-j-check >>"$out" 2>&1; then
        git push -q origin --delete "$branch" >/dev/null 2>&1
        rm -f "$out.head"
        log "$branch ($head): pushed as $tip in $((SECONDS - t0)) s"
        return 0
    fi
    if grep -q "cannot lock ref\|non-fast-forward\|fetch first" "$out"; then
        log "$branch ($head): lane-j-check moved during the gates; taking it again"
        return 1
    fi
    echo "$head" >"$out.head"
    log "$branch ($head): a gate failed; see $out"
    return 1
}

factory() { git diff --quiet "origin/lane-j-check...origin/queue/$1" -- . ':!scripts'; }

while :; do
    git fetch -q --prune origin lane-j-check '+refs/heads/queue/*:refs/remotes/origin/queue/*'
    # Below 5 GB free the gates fail for want of disk, not of a branch (D lost 40 min to it).
    if [ "$(df --output=avail -k . | tail -1)" -lt 5000000 ]; then
        [ "${low:-0}" = 1 ] || log "under 5 GB free; waiting for disk before the next gate"
        low=1; sleep 30; continue
    fi
    low=0
    all=$(git for-each-ref --sort=committerdate --format='%(refname:lstrip=3)' refs/remotes/origin/queue/)
    # A branch that changes only scripts/ (the queue's own tools) goes first: it gates in
    # minutes and can unblock every branch behind it.
    branches=$(for b in $all; do factory "$b" && echo "$b"; done; for b in $all; do factory "$b" || echo "$b"; done)
    for b in $branches; do take "$b"; done
    [ "$once" = 1 ] && exit 0
    sleep 30
done
