# Sourced by scripts/check.sh from the checkout root. Sets `target` to the
# absolute cargo target directory and pins CPL_STD to this checkout's
# stdlib: without it, a test that runs `cpl` on a program outside the tree
# falls back to the `stdlib/` above the binary, which under a shared or
# symlinked target directory is another checkout's.
target="${CARGO_TARGET_DIR:-target}"
case "$target" in
    /*) ;;
    *) target="$PWD/$target" ;;
esac
export CPL_STD="$PWD/stdlib"
