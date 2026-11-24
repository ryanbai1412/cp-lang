#!/usr/bin/env bash
# Every corpus port (corpus/problems/*/port.cp) passes `cpl check`, check only,
# in parallel. A hard gate: any port with an error turns it red; no port is
# exempt (a port that can't check yet is a dream.cp, not a port).
# Enforces idiomatic-code/R9 (handbook/guidelines/idiomatic-code-checklist.md#r9).
#   scripts/port-check.sh [cpl]
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
# Default: the newest of the debug and release builds, so a stale one never
# reports false regressions.
cpl=${1:-$(ls -t target/release/cpl target/debug/cpl 2>/dev/null | head -1)}
cpl=${cpl:-target/debug/cpl}
# Read std/ as it is now, so a std edit needs no rebuild of cpl.
export CPL_STD_DIR=${CPL_STD_DIR:-$PWD/std}
jobs=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)
start=$(date +%s)
# xargs exits 123 when any port fails; the errors say which (a warning never fails).
failing=$( { ls corpus/problems/*/port.cp | xargs -P "$jobs" -n 16 "$cpl" check 2>&1 || true; } |
    awk '/^(error|warning)/ { err = /^error/ }
         err && match($0, /^ *--> corpus\/problems\/[^\/]+\/port\.cp:/) {
             sub(/^ *--> corpus\/problems\//, ""); sub(/\/.*/, ""); print }' | sort -u)
echo "port-check: $(ls corpus/problems/*/port.cp | wc -l | tr -d ' ') ports in $(( $(date +%s) - start ))s"
if [ -n "$failing" ]; then
    echo "port-check: these ports fail cpl check:" >&2
    printf '  %s\n' $failing >&2
    # The log is the only evidence a CI runner keeps: show each port's errors.
    set +e +o pipefail
    for p in $failing; do
        echo "port-check: $cpl check corpus/problems/$p/port.cp" >&2
        "$cpl" check "corpus/problems/$p/port.cp" 2>&1 | head -40 >&2
    done
    exit 1
fi
