#!/usr/bin/env bash
# Prints the newest main commit whose `site build` and `site browser tests`
# CI jobs passed, unless the live site already has it or a newer commit
# (handbook/guidelines/ci-checklist.md ci/R25). Prints nothing then. Under
# Actions it also writes `since=<the run's updated_at>` to $GITHUB_OUTPUT, for
# site.yml's `live site is current` step.
set -euo pipefail
. "$(dirname "$0")/retry.sh"
repo=${GITHUB_REPOSITORY:?}
version=$(fetch_optional https://cp-lang.org/version.json)
version=${version:-'{}'}
live=$(jq -r '.commit // empty' <<<"$version")
built=$(jq -r '.built // empty' <<<"$version")
gh run list -R "$repo" --workflow ci.yml --branch main --event push -L 60 \
    --json databaseId,headSha,status,conclusion,updatedAt \
    --jq '.[] | select(.status == "in_progress" or (.status == "completed" and .conclusion != "cancelled")) | "\(.databaseId) \(.headSha) \(.updatedAt)"' |
while read -r id sha updated; do
    # Superseded runs ran no job, and runs older than the live commit can't
    # move the site: skipping both keeps this to a few API calls (ci/R25).
    [ "$sha" != "$live" ] || { echo "site-deployable: the live site ($live) is the newest passing commit" >&2; exit 0; }
    # The live commit's own run is often cancelled and so never listed; a run
    # that ended before the live site was built is no newer than it.
    [ -z "$built" ] || [[ "$updated" > "$built" ]] || { echo "site-deployable: no passing commit since the live build ($built)" >&2; exit 0; }
    jobs=$(gh run view "$id" -R "$repo" --json jobs --jq '.jobs[] | select(.name == "site build" or (.name | startswith("site browser tests"))) | "\(.conclusion) \(.name)"')
    grep -q '^success site build$' <<<"$jobs" || continue
    # Every part of the browser tests passed, and there was at least one.
    grep -q '^success site browser tests' <<<"$jobs" || continue
    ! grep -v '^success ' <<<"$jobs" | grep -q ' site browser tests' || continue
    if [ -n "$live" ]; then
        rel=$(retry gh api "repos/$repo/compare/$sha...$live" --jq .status)
        if [ "$rel" = identical ] || [ "$rel" = ahead ]; then
            echo "site-deployable: the live site ($live) already has $sha" >&2; exit 0
        fi
    fi
    echo "site-deployable: $sha (run $id, finished $updated)" >&2
    [ -z "${GITHUB_OUTPUT:-}" ] || echo "since=$updated" >>"$GITHUB_OUTPUT"
    echo "$sha"; exit 0
done
