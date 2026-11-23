#!/usr/bin/env bash
# Every corpus port (corpus/problems/*/port.cp) passes `cpl check`, check only,
# in parallel. Ports in corpus/port-check-failing.txt may still fail (ratchet:
# the list only shrinks); a port that newly fails turns the gate red.
# Enforces idiomatic-code/R9 (handbook/guidelines/idiomatic-code-checklist.md#r9).
#   scripts/port-check.sh [cpl]       check against the list
#   scripts/port-check.sh --update    rewrite the list from today's failures
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
update=0
[ "${1:-}" != --update ] || { update=1; shift; }
# Default: the newest of the debug and release builds, so a stale one never
# reports false regressions.
cpl=${1:-$(ls -t target/release/cpl target/debug/cpl 2>/dev/null | head -1)}
cpl=${cpl:-target/debug/cpl}
# --update writes the ratchet, so a cpl older than the compiler it stands for
# would bake its bugs into the list: build first, or refuse a binary given.
if [ "$update" = 1 ]; then
    if [ -z "${1:-}" ]; then
        cargo build -q -p cpl-cli --bin cpl
        cpl=target/debug/cpl
    elif stale=$(find crates -name '*.rs' -newer "$cpl" -print -quit) && [ -n "$stale" ]; then
        echo "port-check: $cpl is older than $stale; rebuild it before --update" >&2
        exit 1
    fi
fi
list=corpus/port-check-failing.txt
# Read std/ as it is now, so a std edit needs no rebuild of cpl.
export CPL_STD_DIR=${CPL_STD_DIR:-$PWD/std}
jobs=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)
start=$(date +%s)
# xargs exits 123 when any port fails; the diagnostics say which.
now=$( { ls corpus/problems/*/port.cp | xargs -P "$jobs" -n 16 "$cpl" check 2>&1 || true; } |
    sed -nE 's|^ *--> corpus/problems/([^/]+)/port\.cp:.*|\1|p' | sort -u)
if [ "$update" = 1 ]; then
    printf '%s\n' "$now" | sed '/^$/d' >"$list"
    echo "port-check: wrote $list ($(grep -c . "$list") failing)"
    exit 0
fi
new=$(comm -23 <(printf '%s\n' "$now" | sed '/^$/d') <(sort -u "$list"))
fixed=$(comm -13 <(printf '%s\n' "$now" | sed '/^$/d') <(sort -u "$list") | grep -c . || true)
echo "port-check: $(ls corpus/problems/*/port.cp | wc -l | tr -d ' ') ports in $(( $(date +%s) - start ))s; $fixed newly pass"
[ "$fixed" = 0 ] || echo "port-check: shrink the list: scripts/port-check.sh --update"
if [ -n "$new" ]; then
    echo "port-check: these ports no longer pass cpl check:" >&2
    printf '  %s\n' $new >&2
    exit 1
fi
