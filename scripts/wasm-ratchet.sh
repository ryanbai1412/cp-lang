#!/usr/bin/env bash
# CI's shrink-only cpl.wasm ratchet per tip (web/scripts/wasm-crates.mjs ratchet), the
# rule land.sh applies from a warm build: every crate the tips since the nearest
# first-parent with saved sizes changed, and the data segment, may not grow a byte
# while over its budget, and one within it stays within; other crates only warn.
# Most main tips are cancelled runs that never save, so it walks back up to 20.
#   scripts/wasm-ratchet.sh record DIR/cpl.wasm SIZES_DIR   save HEAD's sizes
#   scripts/wasm-ratchet.sh check DIR/cpl.wasm SIZES_DIR    hold HEAD to them
#   scripts/wasm-ratchet.sh lower DIR/cpl.wasm SIZES_DIR    check, then push the budgets
#     it lowers (main only: budgets only go down, to each size plus some room)
set -euo pipefail
cmd=$1 wasm=$2 sizes=$3
budgets=web/scripts/wasm-crate-budgets.json
case $cmd in
record)
    mkdir -p "$sizes"
    node web/scripts/wasm-crates.mjs --sizes "$wasm" "$sizes/$(git rev-parse HEAD).json"
    ls -t "$sizes"/*.json | tail -n +61 | xargs -r rm -f ;;
check)
    for c in $(git rev-list --first-parent --max-count=21 HEAD | tail -n +2); do
        if [ -s "$sizes/$c.json" ]; then
            echo "wasm-ratchet: against $(git log -1 --format='%h %s' "$c")"
            touched=$(git diff --name-only "$c" HEAD | sed -n 's|^crates/\([^/]*\)/.*|\1|p' | sort -u | tr - _ | paste -sd, -)
            exec node web/scripts/wasm-crates.mjs --data "$wasm" --parent "$sizes/$c.json" --touched "$touched" "${@:4}"
        fi
    done
    echo "::warning::wasm-ratchet: no saved sizes in HEAD's last 20 first-parents; only the budgets apply (wasm-size)" ;;
lower)
    "$0" check "$wasm" "$sizes" --lower-budgets
    ! git diff --quiet -- "$budgets" || exit 0
    # The environment's identity, as a git hook sets it, would beat `git config`.
    export GIT_AUTHOR_NAME="github-actions[bot]" GIT_COMMITTER_NAME="github-actions[bot]"
    export GIT_AUTHOR_EMAIL="41898282+github-actions[bot]@users.noreply.github.com"
    export GIT_COMMITTER_EMAIL="$GIT_AUTHOR_EMAIL"
    git commit -q -m "ci: lower wasm budgets to $(git rev-parse --short HEAD)'s sizes" -m "Domain: infra" -- "$budgets"
    # One attempt, as regen's: a lost race means a newer tip, whose run lowers them.
    if git pull -q --rebase origin main && git push -q origin HEAD:main; then
        gh workflow run ci.yml --ref main
    else
        [ ! -d .git/rebase-merge ] || git rebase --abort
        echo "::notice::main moved on; its own run lowers the wasm budgets"
    fi ;;
*) echo "usage: scripts/wasm-ratchet.sh record|check|lower DIR/cpl.wasm SIZES_DIR" >&2; exit 2 ;;
esac
