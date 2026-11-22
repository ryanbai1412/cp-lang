#!/usr/bin/env bash
# Check every dream against the spec: the stubs in spec/std shadow stdlib/
# module by module, so a dream passes when the spec, not the implementation,
# covers everything it uses. With no arguments the gold dreams (spec/GOLD.txt)
# are checked first: their pass rate is the headline; the rest is coverage.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
export CPL_SPEC=1 CPL_STD="$root/spec/std"
if grep -lE "^(<<<<<<<|>>>>>>>|\|\|\|\|\|\|\|) " "$root"/spec/std/*.cp "$root"/spec/gold/*.cp; then
    echo "spec-check: unresolved merge conflict markers in the files above" >&2; exit 1
fi
if [ $# -eq 0 ]; then
    python3 "$root/scripts/spec-vocab.py" | tail -1
    python3 "$root/scripts/spec-blocks.py" --ratchet
    python3 "$root/scripts/spec-stub.py" --check >/dev/null ||
        echo "spec-check: WARNING: spec/std has fn bodies; run scripts/spec-stub.py" >&2
    mapfile -t gold < <(grep -v '^#' "$root/spec/GOLD.txt" | sed '/^$/d; s|^|'"$root"'/|')
    python3 "$root/scripts/dream-gaps.py" "${gold[@]}" | sed '1s/^dreams:/gold:/'
    echo
fi
exec python3 "$root/scripts/dream-gaps.py" "$@"
