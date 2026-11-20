#!/bin/bash
# Stress every port in this folder against the original named in its header.
# Usage: proto/ports/scout_batch_6/today/stress.sh [iterations] [size]
here=$(cd "$(dirname "$0")" && pwd)
cd "$here/../../.."
ulimit -s unlimited 2>/dev/null     # some originals keep large arrays on the stack
tmp=$(mktemp -d)
fail=0
for f in "$here"/*.cp; do
    name=$(basename "$f" .cp)
    orig=$(sed -n 's|^// original: ||p' "$f")
    cp "$f" "$tmp/port.cp"          # built from a copy: the build stamps its source
    if [[ -f "$here/${name}_gen.py" ]]; then
        # the typed generator cannot make this input, so a script does
        python3 cp.py build "$tmp/port.cp" > /dev/null 2>&1 &&
            g++ -std=c++17 -O2 -o "$tmp/port" "$tmp/port.cpp" &&
            g++ -std=c++17 -O2 -o "$tmp/ref" "../$orig" || { echo "$name: build failed"; fail=1; continue; }
        ok=0
        for seed in $(seq 1 "${1:-200}"); do
            python3 "$here/${name}_gen.py" "$seed" "${2:-6}" > "$tmp/in"
            if [[ "$("$tmp/port" < "$tmp/in" | xargs)" == "$("$tmp/ref" < "$tmp/in" | xargs)" ]]; then ok=$((ok + 1))
            else echo "$name: MISMATCH seed=$seed"; fail=1; break; fi
        done
        echo "$name: stress OK: $ok tests agree (script-made inputs)"
        continue
    fi
    r=$(python3 cp.py stress "$tmp/port.cp" "../$orig" "${1:-200}" "${2:-6}" 2>&1 | tail -1)
    echo "$name: $r"
    [[ $r == stress\ OK* ]] || fail=1
done
rm -rf "$tmp"
exit $fail
