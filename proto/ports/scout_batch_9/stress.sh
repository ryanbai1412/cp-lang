#!/bin/bash
# Stress every port in this folder against the original named in its header.
# A port with a `// checker:` line accepts more than one answer, so its
# outputs go to that checker (harness/judge.py) instead of being compared.
# Usage: proto/ports/scout_batch_9/stress.sh [iterations] [size]
cd "$(dirname "$0")/../.."
ulimit -s unlimited 2>/dev/null     # some originals recurse deeply or keep big tables on the stack
tmp=$(mktemp -d)
fail=0
for f in ports/scout_batch_9/*.cp; do
    orig=$(sed -n 's|^// original: ||p' "$f")
    checker=$(sed -n 's|^// checker: ||p' "$f")
    cp "$f" "$tmp/port.cp"          # built from a copy: the build stamps its source
    if [[ -n $checker ]]; then
        r=$(python3 ports/scout_batch_9/harness/judge.py "$tmp/port.cp" "../$orig" "../$checker" "${1:-200}" "${2:-6}" 2>&1 | tail -1)
    else
        r=$(python3 cp.py stress "$tmp/port.cp" "../$orig" "${1:-200}" "${2:-6}" 2>&1 | tail -1)
    fi
    echo "$(basename "$f" .cp): $r"
    [[ $r == stress\ OK* ]] || fail=1
done
rm -rf "$tmp"
exit $fail
