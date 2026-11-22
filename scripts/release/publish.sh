#!/usr/bin/env bash
# Publishes the assets in <dir> (built archives, editor artifacts) as GitHub
# releases with `gh`, after writing latest.json and SHA256SUMS over them.
# stable: release v<version>, marked latest. edge: the rolling `edge`
# prerelease is recreated at this commit, so its assets are always this
# build's; `edge-YYYY-MM-DD` is made only by the day's first edge build and
# never changes after.
#
#   scripts/release/publish.sh <dir> <channel> <version> <commit> <date> <toolchain> "<tags>"
set -euo pipefail
cd "$(dirname "$0")/../.."
dir="$1" channel="$2" version="$3" commit="$4" date="$5" toolchain="$6" tags="$7"
args=()
for tag in $tags; do args+=(--tag "$tag"); done
python3 scripts/release/manifest.py "$dir" --toolchain "$toolchain" --version "$version" \
    --channel "$channel" --commit "$commit" --date "$date" "${args[@]}"
files=("$dir"/*)
notes="CP Lang $toolchain ($channel, ${commit:0:8}). Install: https://cp-lang.org/install.sh"
for tag in $tags; do
    case "$tag" in
        v*)
            gh release create "$tag" "${files[@]}" --target "$commit" --title "CP Lang $version" \
                --notes "$notes" --latest
            ;;
        edge)
            gh release delete edge --cleanup-tag --yes >/dev/null 2>&1 || true
            gh release create edge "${files[@]}" --target "$commit" --prerelease --latest=false \
                --title "CP Lang edge" --notes "$notes. The newest build of main; replaced by every edge build."
            ;;
        edge-*)
            gh release create "$tag" "${files[@]}" --target "$commit" --prerelease --latest=false \
                --title "CP Lang $tag" --notes "$notes. The first edge build of $date; never replaced."
            ;;
        *) echo "publish: unknown tag $tag" >&2; exit 2 ;;
    esac
done
