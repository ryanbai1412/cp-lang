#!/usr/bin/env bash
# Rule: handbook/guidelines/ci-checklist.md#cir22.
# A named cpl.wasm (scripts/build-wasm.sh --names) of the tree HEAD has, built
# nice'd in the background, with that tree recorded beside it, so land.sh holds
# a change's crates to their wasm budgets without waiting on a build.
#   scripts/wasm-warm.sh            start the build in the background (`just wasm-warm`;
#                                   .githooks/post-commit after a commit touching crates/)
#   scripts/wasm-warm.sh --wait     build in the foreground
#   scripts/wasm-warm.sh --current CRATE...
#                                   print the build's cpl.wasm if it is of HEAD's tree, or of
#                                   one that differs from it outside these crates and std
#                                   (a crate is named as its directory); else fail
#   scripts/wasm-warm.sh --parent BASE CRATE...
#                                   print the per-crate sizes of a build of BASE's tree, or of
#                                   one that differs from it outside these crates and std; else fail
#   scripts/wasm-warm.sh --check WASM BASE CRATE...
#                                   the wasm rule the commit tier (ci.py) and land.sh share:
#                                   with BASE's sizes, every crate and the data are held to the
#                                   ratchet (one over its budget refuses only if its
#                                   normalized code grew);
#                                   without them, builds BASE's tree to record them first,
#                                   and refuses if it can't
# Each recorded build writes its sizes to sizes/<tree>.json, and --wait also builds
# HEAD's merge-base with origin/main when it has none, so land can hold a crate over
# its budget to its parent's size (web/scripts/wasm-crates.mjs --parent).
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
# A crate by its directory or its Rust name (cpl_ir is crates/cpl-ir); one with
# neither is refused, since a path that matches nothing never differs.
case "${1:-}" in --current | --parent)
    for c in "${@:$([ "$1" = --parent ] && echo 3 || echo 2)}"; do
        [ -d "crates/${c//_/-}" ] || { echo "wasm-warm: no crate directory crates/${c//_/-}" >&2; exit 2; }
    done ;;
esac
target="${CARGO_TARGET_DIR:-target}"
case "$target" in /*) ;; *) target="$PWD/$target" ;; esac
out="$target/wasm-warm"
inputs=(crates std Cargo.toml Cargo.lock scripts/build-wasm.sh)
# Records the per-crate sizes of a build of BASE's tree, built in a scratch worktree.
record_base() {
    local src="$out/base-src" ok=0
    git worktree remove --force "$src" 2>/dev/null || rm -rf "$src"
    git worktree prune
    git worktree add -q --detach "$src" "$1" || return 1
    (cd "$src" && CARGO_TARGET_DIR="$target" scripts/build-wasm.sh --names "$out/base") &&
        mkdir -p "$out/sizes" &&
        node web/scripts/wasm-crates.mjs --sizes "$out/base/cpl.wasm" "$out/sizes/$(git rev-parse "$1^{tree}").json" || ok=1
    git worktree remove --force "$src"
    ls -t "$out"/sizes/*.json 2>/dev/null | tail -n +9 | xargs -r rm -f
    return "$ok"
}
case "${1:-}" in
    --current)
        shift
        built=$(cat "$out/tree" 2>/dev/null) && [ -s "$out/cpl.wasm" ] || exit 1
        dirs=(std Cargo.toml Cargo.lock); for c in "$@"; do dirs+=("crates/$c"); done
        [ "$built" = "$(git rev-parse 'HEAD^{tree}')" ] ||
            git diff --quiet "$built" 'HEAD^{tree}' -- "${dirs[@]}" 2>/dev/null || exit 1
        echo "$out/cpl.wasm" ;;
    --parent)
        # Every crate is held to the parent, touched or not, so only a build whose
        # wasm inputs all equal the parent's stands for it.
        base=$(git rev-parse "$2^{tree}" 2>/dev/null) || exit 1
        for f in $(ls -t "$out"/sizes/*.json 2>/dev/null); do
            t=$(basename "$f" .json)
            # A record from before the normalized sizes can't hold a crate to its code.
            grep -q '"fns"' "$f" || continue
            if [ "$t" = "$base" ] || git diff --quiet "$t" "$base" -- "${inputs[@]}" 2>/dev/null; then
                echo "$f"
                exit 0
            fi
        done
        exit 1 ;;
    --check)
        wasm=$2 base=$3
        shift 3
        touched=$(printf '%s\n' "$@" | tr - _ | paste -sd, -)
        # A base nobody has built yet (main's newest tip) is built now: held to the
        # absolute budgets instead, a crate already over its budget refuses at net 0.
        if ! sizes=$(bash "$0" --parent "$base" "$@"); then
            mkdir -p "$out"
            exec 9>"$out.lock"
            flock 9
            if ! sizes=$(bash "$0" --parent "$base" "$@"); then
                echo "wasm-warm: no wasm sizes of $base's tree; building it to hold the crates to it" >&2
                record_base "$base" >&2 && sizes=$(bash "$0" --parent "$base" "$@") ||
                    { echo "wasm-warm: could not build $base's wasm, so no crate can be held to it; fix that build and run again" >&2; exit 1; }
            fi
            exec 9>&-
        fi
        budgets=$(mktemp)
        git show "$base:web/scripts/wasm-crate-budgets.json" >"$budgets" 2>/dev/null || echo '{}' >"$budgets"
        exec node web/scripts/wasm-crates.mjs --data "$wasm" --parent "$sizes" --parent-budgets "$budgets" --touched "$touched" ;;
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
        record_base "$base"
        echo "wasm-warm: recorded the sizes of $base, HEAD's merge-base" ;;
    "")
        mkdir -p "$out"
        nohup nice -n 10 "$0" --wait >"$out.log" 2>&1 </dev/null &
        echo "wasm-warm: building $out/cpl.wasm in the background (log: $out.log)" ;;
    *) echo "usage: scripts/wasm-warm.sh [--wait | --current CRATE... | --parent BASE CRATE... | --check WASM BASE CRATE...]" >&2; exit 2 ;;
esac
