#!/bin/bash
# Stress every port in this folder against the original named in its header.
# Usage: proto/ports/scout_batch_4/stress.sh [iterations] [size]
cd "$(dirname "$0")/../.."
ulimit -s unlimited 2>/dev/null     # some originals keep megabytes of tables on the stack
tmp=$(mktemp -d)
fail=0
for f in ports/scout_batch_4/*.cp; do
    orig=$(sed -n 's|^// original: ||p' "$f")
    cp "$f" "$tmp/port.cp"          # built from a copy: the build stamps its source
    r=$(python3 cp.py stress "$tmp/port.cp" "../$orig" "${1:-200}" "${2:-6}" 2>&1 | tail -1)
    echo "$(basename "$f" .cp): $r"
    [[ $r == stress\ OK* ]] || fail=1
done
rm -rf "$tmp"
exit $fail
