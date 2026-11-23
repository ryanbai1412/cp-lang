#!/usr/bin/env bash
# Publishes the assets in <dir> (built archives, editor artifacts) as GitHub
# releases with `gh`, after writing latest.json and SHA256SUMS over them.
# stable: release v<version>, marked latest. edge: the rolling `edge`
# prerelease moves to this commit and takes this build's assets in place,
# never deleted first, so installs work throughout; `edge-YYYY-MM-DD` is made only by the day's first edge build and
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
            # Stable notes are the version's CHANGELOG.md section; 0.x is beta.
            changes=$(awk -v v="$version" '/^## /{on = ($2 == v); next} on' CHANGELOG.md)
            [ -n "${changes//[[:space:]]/}" ] || { echo "publish: CHANGELOG.md has no ## $version section" >&2; exit 2; }
            title="CP Lang $version"
            case "$version" in 0.*) title="$title (beta)" ;; esac
            gh release create "$tag" "${files[@]}" --target "$commit" --title "$title" \
                --notes "$changes"$'\n\n'"$notes" --latest
            ;;
        edge)
            edge_notes="$notes. The newest build of main; replaced by every edge build."
            if ! gh release view edge >/dev/null 2>&1; then
                gh release create edge "${files[@]}" --target "$commit" --prerelease --latest=false \
                    --title "CP Lang edge" --notes "$edge_notes"
                continue
            fi
            # Never delete edge first: installs keep working if any step fails.
            # Move the tag, upload the archives, then the SHA256SUMS and
            # latest.json naming them, and only then drop assets this build lacks.
            gh api -X PATCH "repos/{owner}/{repo}/git/refs/tags/edge" -f sha="$commit" -F force=true >/dev/null
            archives=() index=() names=" "
            for f in "${files[@]}"; do
                names+="${f##*/} "
                case "${f##*/}" in latest.json | SHA256SUMS) index+=("$f") ;; *) archives+=("$f") ;; esac
            done
            gh release upload edge "${archives[@]}" --clobber
            gh release upload edge "${index[@]}" --clobber
            for asset in $(gh release view edge --json assets --jq '.assets[].name'); do
                case "$names" in *" $asset "*) ;; *) gh release delete-asset edge "$asset" --yes ;; esac
            done
            gh release edit edge --target "$commit" --title "CP Lang edge" --notes "$edge_notes" --prerelease --latest=false
            ;;
        edge-*)
            gh release create "$tag" "${files[@]}" --target "$commit" --prerelease --latest=false \
                --title "CP Lang $tag" --notes "$notes. The first edge build of $date; never replaced."
            ;;
        *) echo "publish: unknown tag $tag" >&2; exit 2 ;;
    esac
done
