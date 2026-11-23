#!/usr/bin/env bash
# Opens one "main CI stalled" issue when no main CI run has finished (success
# or failure) in 30 min, edge is over 2 h old, or cp-lang.org has lacked a
# deployable commit for over 2 h, and closes it once all are fresh again
# (handbook/guidelines/ci-checklist.md#cir23).
set -euo pipefail
repo=${GITHUB_REPOSITORY:?}
now=$(date +%s)
age() { [ -n "$1" ] && echo $((now - $(date -d "$1" +%s))) || echo 999999; }
# Superseded runs end `cancelled` by the hundred, so ask per conclusion.
last() { gh api "repos/$repo/actions/workflows/ci.yml/runs?branch=main&event=push&status=$1&per_page=1" \
    --jq '.workflow_runs[0].updated_at // ""'; }
ran=$(printf '%s\n' "$(last success)" "$(last failure)" | sort | tail -1)
shipped=$(gh release view edge -R "$repo" --json assets \
    --jq '.assets[] | select(.name == "latest.json") | .updatedAt' 2>/dev/null || true)
ci_age=$(age "$ran"); edge_age=$(age "$shipped")
problems=()
[ "$ci_age" -le 1800 ] || problems+=("no main CI run has finished for $((ci_age / 60)) min (last: ${ran:-none})")
[ "$edge_age" -le 7200 ] || problems+=("edge was last published $((edge_age / 60)) min ago (${shipped:-never})")
# The site is behind when a commit whose site jobs passed isn't live and the
# live commit is over 2 h old.
here=$(dirname "$0")
deployable=$("$here/site-deployable.sh" 2>/dev/null || true)
if [ -n "$deployable" ]; then
    live=$(curl -fsSL https://cp-lang.org/version.json 2>/dev/null | jq -r '.commit // empty' || true)
    live_at=$([ -n "$live" ] && gh api "repos/$repo/commits/$live" --jq .commit.committer.date 2>/dev/null || true)
    site_age=$(age "$live_at")
    [ "$site_age" -le 7200 ] ||
        problems+=("cp-lang.org serves ${live:-nothing} from $((site_age / 60)) min ago; ${deployable:0:9} passed its site jobs")
fi
title="main CI stalled"
open=$(gh issue list -R "$repo" --state open --search "\"$title\" in:title" --json number --jq '.[0].number // ""')
if [ ${#problems[@]} -eq 0 ]; then
    echo "watchdog: ok (last finished run ${ci_age}s ago, edge ${edge_age}s old, site current)"
    [ -z "$open" ] || gh issue close -R "$repo" "$open" -c "Recovered: a main run finished ${ci_age}s ago and edge is ${edge_age}s old."
    exit 0
fi
body=$(printf -- '- %s\n' "${problems[@]}")
printf 'watchdog: %s\n' "${problems[@]}"
if [ -n "$open" ]; then
    gh issue comment -R "$repo" "$open" -b "$body"
else
    gh issue create -R "$repo" -t "$title" -b "$body

See handbook/process/landing-and-ci.md#when-main-goes-red."
fi
