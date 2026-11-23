#!/usr/bin/env bash
# The site's own gate: rebuild the browser compiler, build web/ (which holds
# cpl.wasm to its gzipped size cap and runs the snippet gate), check its
# internal links, then run the browser smoke tests against web/out.
#
#   scripts/site-check.sh             skip when nothing it reads has changed
#   scripts/site-check.sh --no-cache  always run
#
# CPL names the cpl binary the site build runs; without it, a release build.
#
# Needs pnpm, the wasm32 target, and a browser: Playwright's Chromium
# (pnpm --dir web exec playwright install chromium), or CDP=<url> of a
# running Chrome.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
use_cache=1
[ "${CHECK_NO_CACHE:-0}" = "1" ] && use_cache=0
for arg in "$@"; do
    case "$arg" in
        --no-cache) use_cache=0 ;;
        -h | --help) sed -n '2,14s/^# \{0,1\}//p' "$0"; exit 0 ;;
        *) echo "site-check: unknown argument \`$arg\`" >&2; exit 2 ;;
    esac
done

. scripts/gate-env.sh
cache_dir="$target/check-cache"
manifest="$cache_dir/manifest.site.$$"
trap 'rm -f "$manifest" "$manifest.reads"' EXIT
if [ "$use_cache" -eq 1 ]; then
    mkdir -p "$cache_dir"
    python3 scripts/check_cache.py manifest "$manifest" || use_cache=0
fi
cache_hit() {
    [ "$use_cache" -eq 1 ] || return 1
    python3 scripts/check_cache.py check "$cache_dir" "$1" "$manifest"
}
cache_save() {
    [ "$use_cache" -eq 1 ] || return 0
    python3 scripts/check_cache.py save "$cache_dir" "$1" "$manifest"
}
if cache_hit site_smoke; then
    echo "site-check: cached (tree unchanged outside its excludes)"
    exit 0
fi

command -v pnpm >/dev/null || { echo "site-check: pnpm is not installed (mise install sets it up)" >&2; exit 1; }
if [ -z "${CPL:-}" ]; then
    cargo build --release -p cpl-cli
    export CPL="$target/release/cpl"
fi
scripts/build-wasm.sh
[ -d web/node_modules ] || pnpm --dir web install --frozen-lockfile
pnpm --dir web build
pnpm --dir web links
pnpm --dir web smoke
cache_save site_smoke
echo "site-check: ok"
