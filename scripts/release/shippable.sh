#!/usr/bin/env bash
# Prints the newest main commit whose CI shipping jobs all passed, even if
# other jobs (ratchets, editors, latency, unit-test shards) are red: the
# conformance gate parts (interpreter and C++ agree on the corpus), the cli
# jobs on macOS and Windows, and the acceptance solutions. The release run
# itself then builds every target and smoke-tests the archives and
# installers. Edge ships from this commit hourly (decision 408); stable
# still needs a fully green run.
set -euo pipefail
repo=${GITHUB_REPOSITORY:?}
shipping='(gate \(conform:|cli \(|acceptance solutions stay AC)'
gh run list -R "$repo" --workflow ci.yml --branch main --event push -L 300 \
    --json databaseId,headSha,status,conclusion \
    --jq '.[] | select(.status == "completed" and (.conclusion == "success" or .conclusion == "failure")) | "\(.databaseId) \(.headSha)"' |
while read -r id sha; do
    jobs=$(gh run view "$id" -R "$repo" --json jobs --jq '.jobs[] | "\(.conclusion)\t\(.name)"' | grep -P "\t$shipping" || true)
    [ -n "$jobs" ] || continue
    if ! grep -qvP '^success\t' <<<"$jobs"; then
        red=$(gh run view "$id" -R "$repo" --json jobs --jq '[.jobs[] | select(.conclusion == "failure") | .name] | join(", ")')
        echo "shippable: $sha (run $id)${red:+; red but not shipping: $red}" >&2
        echo "$sha"
        exit 0
    fi
done
