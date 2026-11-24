#!/usr/bin/env bash
# A named cpl.wasm (scripts/build-wasm.sh --names) of the tree HEAD has, built
# nice'd in the background, with that tree recorded beside it, so land.sh holds
# a change's crates to their wasm budgets without waiting on a build.
#   scripts/wasm-warm.sh            start the build in the background (`just wasm-warm`;
#                                   .githooks/post-commit after a commit touching crates/)
#   scripts/wasm-warm.sh --wait     build in the foreground
#   scripts/wasm-warm.sh --current CRATE...
#                                   print the build's cpl.wasm if it is of HEAD's tree, or of
#                                   one that differs from it outside these crates; else fail
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
target="${CARGO_TARGET_DIR:-target}"
case "$target" in /*) ;; *) target="$PWD/$target" ;; esac
out="$target/wasm-warm"
inputs=(crates std Cargo.toml Cargo.lock scripts/build-wasm.sh)
case "${1:-}" in
    --current)
        shift
        built=$(cat "$out/tree" 2>/dev/null) && [ -s "$out/cpl.wasm" ] || exit 1
        dirs=(); for c in "$@"; do dirs+=("crates/$c"); done
        [ "$built" = "$(git rev-parse 'HEAD^{tree}')" ] ||
            git diff --quiet "$built" 'HEAD^{tree}' -- "${dirs[@]}" 2>/dev/null || exit 1
        echo "$out/cpl.wasm" ;;
    --wait)
        mkdir -p "$out"
        exec 9>"$out.lock"
        flock 9
        tree=$(git rev-parse 'HEAD^{tree}')
        # build-wasm.sh builds the working tree: only one that is HEAD's is recorded.
        git diff --quiet HEAD -- "${inputs[@]}" || { echo "wasm-warm: uncommitted crate changes; not building"; exit 0; }
        [ "$(cat "$out/tree" 2>/dev/null)" != "$tree" ] || { echo "wasm-warm: $out is current"; exit 0; }
        rm -f "$out/tree"
        scripts/build-wasm.sh --names "$out"
        git diff --quiet "$tree" -- "${inputs[@]}" && echo "$tree" >"$out/tree"
        echo "wasm-warm: built $out/cpl.wasm for tree $tree" ;;
    "")
        mkdir -p "$out"
        nohup nice -n 10 "$0" --wait >"$out.log" 2>&1 </dev/null &
        echo "wasm-warm: building $out/cpl.wasm in the background (log: $out.log)" ;;
    *) echo "usage: scripts/wasm-warm.sh [--wait | --current CRATE...]" >&2; exit 2 ;;
esac
