#!/usr/bin/env bash
# Opens one "main CI stalled" issue when no main CI run has finished (success
# or failure) in 30 min, edge is over 2 h old or an archive of it won't download, or cp-lang.org has lacked a
# deployable commit for over 2 h, and closes it once all are fresh again
# (handbook/guidelines/ci-checklist.md#cir23).
set -euo pipefail
repo=${GITHUB_REPOSITORY:?}
here=$(dirname "$0")
. "$here/retry.sh"
now=$(date +%s)
age() { [ -n "$1" ] && echo $((now - $(date -d "$1" +%s))) || echo 999999; }
# Superseded runs end `cancelled` by the hundred, so ask per conclusion.
last() { gh api "repos/$repo/actions/workflows/ci.yml/runs?branch=main&event=push&status=$1&per_page=1" \
    --jq '.workflow_runs[0].updated_at // ""'; }
ran=$(printf '%s\n' "$(last success)" "$(last failure)" | sort | tail -1)
shipped=$(retry gh release view edge -R "$repo" --json assets \
    --jq '.assets[] | select(.name == "latest.json") | .updatedAt')
ci_age=$(age "$ran"); edge_age=$(age "$shipped")
problems=()
[ "$ci_age" -le 1800 ] || problems+=("no main CI run has finished for $((ci_age / 60)) min (last: ${ran:-none})")
[ "$edge_age" -le 7200 ] || problems+=("edge was last published $((edge_age / 60)) min ago (${shipped:-never})")
# Every archive edge's SHA256SUMS lists must download, so `install.sh` works.
base="https://github.com/$repo/releases/download/edge"
if sums=$(fetch "$base/SHA256SUMS" 2>/dev/null); then
    for a in $(awk '{print $2}' <<<"$sums"); do
        fetch -I -o /dev/null "$base/$a" 2>/dev/null || problems+=("edge is missing $a")
    done
else
    problems+=("edge has no SHA256SUMS")
fi
# The site is behind when a commit whose site jobs passed isn't live and the
# live commit is over 2 h old.
deployable=$("$here/site-deployable.sh") || problems+=("scripts/site-deployable.sh failed; the log says why")
if [ -n "$deployable" ]; then
    live=$(fetch_optional https://cp-lang.org/version.json | jq -r '.commit // empty') ||
        problems+=("cp-lang.org/version.json won't download")
    live_at=""
    [ -z "$live" ] || live_at=$(retry gh api "repos/$repo/commits/$live" --jq .commit.committer.date)
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
