#!/usr/bin/env bash
# Every corpus port (corpus/problems/*/port.cp) passes `cpl check`, check only,
# in parallel. Ports in corpus/port-check-failing.txt may still fail (ratchet:
# the list only shrinks); a port that newly fails turns the gate red.
#   scripts/port-check.sh [cpl]       check against the list
#   scripts/port-check.sh --update    rewrite the list from today's failures
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
update=0
[ "${1:-}" != --update ] || { update=1; shift; }
cpl=${1:-target/debug/cpl}
list=corpus/port-check-failing.txt
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
