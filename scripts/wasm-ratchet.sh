#!/usr/bin/env bash
# CI's shrink-only cpl.wasm ratchet per tip (web/scripts/wasm-crates.mjs ratchet), the
# rule land.sh applies from a warm build: every crate over its budget in the nearest
# first-parent with saved sizes may not grow a byte, and one within it stays within.
# Most main tips are cancelled runs that never save, so it walks back up to 20.
#   scripts/wasm-ratchet.sh record DIR/cpl.wasm SIZES_DIR   save HEAD's sizes
#   scripts/wasm-ratchet.sh check DIR/cpl.wasm SIZES_DIR    hold HEAD to them
set -euo pipefail
cmd=$1 wasm=$2 sizes=$3
case $cmd in
record)
    mkdir -p "$sizes"
    node web/scripts/wasm-crates.mjs --sizes "$wasm" "$sizes/$(git rev-parse HEAD).json"
    ls -t "$sizes"/*.json | tail -n +61 | xargs -r rm -f ;;
check)
    for c in $(git rev-list --first-parent --max-count=21 HEAD | tail -n +2); do
        if [ -s "$sizes/$c.json" ]; then
            echo "wasm-ratchet: against $(git log -1 --format='%h %s' "$c")"
            exec node web/scripts/wasm-crates.mjs --data "$wasm" --parent "$sizes/$c.json"
        fi
    done
    echo "wasm-ratchet: no saved sizes in HEAD's last 20 first-parents; only the budgets apply (wasm-size)" ;;
*) echo "usage: scripts/wasm-ratchet.sh record|check DIR/cpl.wasm SIZES_DIR" >&2; exit 2 ;;
esac
