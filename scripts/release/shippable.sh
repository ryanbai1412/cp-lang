#!/usr/bin/env bash
# Prints the newest main commit whose CI shipping jobs are no worse than at
# the commit edge last shipped (handbook/guidelines/ci-checklist.md ci/R26):
# every shipping job green there is green here, and one already red there
# may stay red. Shipping jobs are the conformance gate parts, the cli jobs
# on every OS and the acceptance solutions; ratchets, editors and unit-test
# shards don't hold edge. With no edge yet every shipping job must pass. A
# run still going counts once its shipping jobs have all finished. The
# release run then builds every target and smoke-tests the archives and
# installers before publishing. Stable still needs a fully green run.
set -euo pipefail
. "$(dirname "$0")/../retry.sh"
repo=${GITHUB_REPOSITORY:?}
shipping='(gate \(conform:|cli \(|acceptance solutions stay AC)'
runs=$(gh run list -R "$repo" --workflow ci.yml --branch main --event push -L 80 \
    --json databaseId,headSha,status \
    --jq '.[] | select(.status == "completed" or .status == "in_progress") | "\(.databaseId) \(.headSha)"')
# "<conclusion>\t<name>" per shipping job; nothing while one hasn't finished.
jobs_of() {
    local j
    j=$(gh run view "$1" -R "$repo" --json jobs --jq '.jobs[] | "\(.conclusion)\t\(.name)"' | grep -P "\t$shipping" || [ $? = 1 ])
    grep -qP '^\t' <<<"$j" && return 0
    printf '%s\n' "$j"
}
green() { grep -P '^success\t' <<<"$1" | cut -f2 || [ $? = 1 ]; }
edge=$(fetch_optional "https://github.com/$repo/releases/download/edge/latest.json" | jq -r '.commit // empty')
must="" base_id=""
if [ -n "$edge" ]; then
    while read -r id sha; do
        [ -n "$id" ] || continue
        rel=$(retry gh api "repos/$repo/compare/$sha...$edge" --jq .status)
        [ "$rel" = identical ] || [ "$rel" = ahead ] || continue
        base=$(jobs_of "$id")
        [ -n "$base" ] || continue
        must=$(green "$base") base_id=$id
        echo "shippable: baseline run $id (${sha:0:9}, at or before edge ${edge:0:9}): $(grep -c . <<<"$must" || [ $? = 1 ]) shipping jobs green" >&2
        break
    done <<<"$runs"
fi
while read -r id sha; do
    [ -n "$id" ] || continue
    jobs=$(jobs_of "$id")
    [ -n "$jobs" ] || continue
    if [ -n "$base_id" ]; then
        worse=$(grep -vxFf <(green "$jobs") <<<"$must" || [ $? = 1 ])
    else
        worse=$(grep -vP '^success\t' <<<"$jobs" | cut -f2 || [ $? = 1 ])
    fi
    if [ -z "$worse" ]; then
        red=$(grep -vP '^success\t' <<<"$jobs" | cut -f2 | paste -sd, - || [ $? = 1 ])
        echo "shippable: $sha (run $id)${red:+; red as at edge: $red}" >&2
        echo "$sha"
        exit 0
    fi
    echo "shippable: not ${sha:0:9} (run $id): worse than edge on $(paste -sd, - <<<"$worse")" >&2
    [ "$id" != "$base_id" ] || exit 0
done <<<"$runs"
