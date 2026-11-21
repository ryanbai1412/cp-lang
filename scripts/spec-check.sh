#!/usr/bin/env bash
# Check every dream against the spec: the stubs in spec/std shadow stdlib/
# module by module, so a dream passes when the spec, not the implementation,
# covers everything it uses. Extra arguments go to dream-gaps.py.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
CPL_SPEC=1 CPL_STD="$root/spec/std" exec python3 "$root/scripts/dream-gaps.py" "$@"
