#!/usr/bin/env bash
# Prints the newest main commit whose `site build` and `site browser tests`
# CI jobs passed, unless the live site already has it or a newer commit
# (handbook/guidelines/ci-checklist.md ci/R25). Prints nothing then.
set -euo pipefail
repo=${GITHUB_REPOSITORY:?}
live=$(curl -fsSL https://cp-lang.org/version.json 2>/dev/null | jq -r '.commit // empty' || true)
gh run list -R "$repo" --workflow ci.yml --branch main --event push -L 60 \
    --json databaseId,headSha,status,conclusion \
    --jq '.[] | select(.status == "in_progress" or (.status == "completed" and .conclusion != "cancelled")) | "\(.databaseId) \(.headSha)"' |
while read -r id sha; do
    # Superseded runs ran no job, and runs older than the live commit can't
    # move the site: skipping both keeps this to a few API calls (ci/R25).
    [ "$sha" != "$live" ] || { echo "site-deployable: the live site ($live) is the newest passing commit" >&2; exit 0; }
    jobs=$(gh run view "$id" -R "$repo" --json jobs --jq '.jobs[] | select(.name == "site build" or .name == "site browser tests") | "\(.conclusion) \(.name)"')
    grep -q '^success site build$' <<<"$jobs" || continue
    grep -q '^success site browser tests$' <<<"$jobs" || continue
    if [ -n "$live" ]; then
        rel=$(gh api "repos/$repo/compare/$sha...$live" --jq .status 2>/dev/null || true)
        if [ "$rel" = identical ] || [ "$rel" = ahead ]; then
            echo "site-deployable: the live site ($live) already has $sha" >&2; exit 0
        fi
    fi
    echo "site-deployable: $sha (run $id)" >&2
    echo "$sha"; exit 0
done
