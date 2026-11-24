#!/usr/bin/env bash
# Check every dream against the spec and its standard library, std. With no arguments the gold dreams (corpus/gold/GOLD.txt)
# are checked first: their pass rate is the headline; the rest is coverage.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
if grep -lE "^(<<<<<<<|>>>>>>>|\|\|\|\|\|\|\|) " "$root"/std/*.cp "$root"/corpus/gold/*.cp; then
    echo "spec-check: unresolved merge conflict markers in the files above" >&2; exit 1
fi
if [ $# -eq 0 ]; then
    uv run --quiet --project "$root" python "$root/scripts/spec-vocab.py" | tail -1
    cargo run -q -p cpl-conform -- spec-blocks --check
    uv run --quiet --project "$root" python "$root/scripts/spec-stub.py" --check >/dev/null ||
        echo "spec-check: WARNING: std has doc or signature problems; run scripts/spec-stub.py --check" >&2
    mapfile -t gold < <(grep -v '^#' "$root/corpus/gold/GOLD.txt" | sed '/^$/d; s|^|'"$root"'/|')
    uv run --quiet --project "$root" python "$root/scripts/dream-gaps.py" "${gold[@]}" | sed '1s/^dreams:/gold:/'
    echo
fi
exec uv run --quiet --project "$root" python "$root/scripts/dream-gaps.py" "$@"
