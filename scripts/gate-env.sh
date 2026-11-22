# Sourced by scripts/check.sh from the checkout root. Sets `target` to the
# absolute cargo target directory.
target="${CARGO_TARGET_DIR:-target}"
case "$target" in
    /*) ;;
    *) target="$PWD/$target" ;;
esac
