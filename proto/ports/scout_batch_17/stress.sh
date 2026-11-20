#!/usr/bin/env bash
# Stress every port in this folder against the original named on its
# `// original:` line. Usage: stress.sh [tests] [size]
cd "$(dirname "$0")/../.."
ulimit -s unlimited 2>/dev/null
tmp=$(mktemp -d)
fail=0
for f in ports/scout_batch_17/*.cp; do
    orig=$(sed -n 's|^// original: \([^ ]*\).*|\1|p' "$f")
    cp "$f" "$tmp/port.cp"                  # cp.py stamps the file it builds
    r=$(python3 cp.py stress "$tmp/port.cp" "../$orig" "${1:-200}" "${2:-6}" 2>&1 | tail -1)
    echo "$(basename "$f" .cp): $r"
    [[ $r == stress\ OK* ]] || fail=1
done
rm -rf "$tmp"
exit $fail
