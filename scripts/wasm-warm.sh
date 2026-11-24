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
#   scripts/wasm-warm.sh --parent BASE CRATE...
#                                   print the per-crate sizes of a build of BASE's tree, or of
#                                   one that differs from it outside these crates; else fail
# Each recorded build writes its sizes to sizes/<tree>.json, and --wait also builds
# HEAD's merge-base with origin/main when it has none, so land can hold a crate over
# its budget to its parent's size (web/scripts/wasm-crates.mjs --parent).
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
    --parent)
        base=$(git rev-parse "$2^{tree}" 2>/dev/null) || exit 1
        shift 2
        dirs=(); for c in "$@"; do dirs+=("crates/$c"); done
        for f in $(ls -t "$out"/sizes/*.json 2>/dev/null); do
            t=$(basename "$f" .json)
            if [ "$t" = "$base" ] || git diff --quiet "$t" "$base" -- "${dirs[@]}" 2>/dev/null; then
                echo "$f"
                exit 0
            fi
        done
        exit 1 ;;
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
        if ! git diff --quiet "$tree" -- "${inputs[@]}"; then
            echo "wasm-warm: the tree changed during the build; $out/cpl.wasm is not recorded"
            exit 0
        fi
        echo "$tree" >"$out/tree"
        mkdir -p "$out/sizes"
        node web/scripts/wasm-crates.mjs --sizes "$out/cpl.wasm" "$out/sizes/$tree.json"
        echo "wasm-warm: built $out/cpl.wasm for tree $tree"
        # The parent's sizes, unless land would already find them.
        base=$(git merge-base HEAD "${CPL_WARM_MAIN:-origin/main}" 2>/dev/null) || exit 0
        [ ! -s "$out/sizes/$(git rev-parse "$base^{tree}").json" ] || exit 0
        src="$out/base-src"
        git worktree remove --force "$src" 2>/dev/null || rm -rf "$src"
        git worktree add -q --detach "$src" "$base"
        (cd "$src" && CARGO_TARGET_DIR="$target" scripts/build-wasm.sh --names "$out/base")
        node web/scripts/wasm-crates.mjs --sizes "$out/base/cpl.wasm" "$out/sizes/$(git rev-parse "$base^{tree}").json"
        git worktree remove --force "$src"
        ls -t "$out"/sizes/*.json | tail -n +9 | xargs -r rm -f
        echo "wasm-warm: recorded the sizes of $base, HEAD's merge-base" ;;
    "")
        mkdir -p "$out"
        nohup nice -n 10 "$0" --wait >"$out.log" 2>&1 </dev/null &
        echo "wasm-warm: building $out/cpl.wasm in the background (log: $out.log)" ;;
    *) echo "usage: scripts/wasm-warm.sh [--wait | --current CRATE... | --parent BASE CRATE...]" >&2; exit 2 ;;
esac
