#!/usr/bin/env bash
# Names a release: what the build stamps, which tags it publishes, and
# whether today's dated edge release is still to be made. Writes key=value
# lines to $GITHUB_OUTPUT (stdout when unset). Asks `gh` which releases exist.
#
#   scripts/release/plan.sh stable|edge
set -euo pipefail
cd "$(dirname "$0")/../.."
channel="${1:?usage: plan.sh stable|edge}"
version="$(sed -n '/^\[workspace.package\]/,/^\[/s/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
[ -n "$version" ] || { echo "plan: no version under [workspace.package] in Cargo.toml" >&2; exit 1; }
commit="$(git rev-parse HEAD)"
date="$(date -u +%F)"
exists() { gh release view "$1" >/dev/null 2>&1; }
case "$channel" in
    stable)
        toolchain="$version"
        tags="v$version"
        if exists "v$version"; then
            echo "plan: v$version is already released; bump the workspace version first" >&2
            exit 1
        fi
        ;;
    edge)
        toolchain="edge-$date-${commit:0:8}"
        tags="edge"
        exists "edge-$date" || tags="edge edge-$date"
        ;;
    *) echo "plan: the channel is stable or edge, not $channel" >&2; exit 2 ;;
esac
{
    echo "channel=$channel"
    echo "version=$version"
    echo "commit=$commit"
    echo "date=$date"
    echo "toolchain=$toolchain"
    echo "tags=$tags"
} >>"${GITHUB_OUTPUT:-/dev/stdout}"
