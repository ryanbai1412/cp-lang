#!/usr/bin/env bash
# Run by .github/workflows/revert-on-red.yml when CI on main tip $BAD failed.
# Finds the newest earlier main tip whose CI passed; if the run just before
# $BAD was green, $BAD's range broke main, so it opens a pull request that
# reverts that range (or, if the revert does not apply, an issue). Main that
# was already red gets nothing new: the first red run's pull request covers it.
set -euo pipefail
: "${BAD:?}" "${RUN_URL:=}"
short=${BAD:0:9}
branch="revert/$short"

runs=$(gh run list -w ci.yml -b main -e push -L 50 --json headSha,conclusion,status \
    -q '.[] | select(.status == "completed") | "\(.headSha) \(.conclusion)"')
good=""
seen=0
while read -r sha conclusion; do
    if [ "$sha" = "$BAD" ]; then seen=1; continue; fi
    [ "$seen" = 1 ] || continue
    git merge-base --is-ancestor "$sha" "$BAD" || continue
    case "$conclusion" in
        success) good=$sha; break ;;
        cancelled | skipped) continue ;;
        *) echo "revert-on-red: main was already red at ${sha:0:9}; nothing new to open"; exit 0 ;;
    esac
done <<<"$runs"
[ -n "$good" ] || { echo "revert-on-red: no green main tip in the last 50 runs"; exit 0; }

if git ls-remote --exit-code origin "refs/heads/$branch" >/dev/null; then
    echo "revert-on-red: $branch exists already"; exit 0
fi
range="${good:0:9}..$short"
commits=$(git log --format='- %h %s' "$good..$BAD")
git config user.name "github-actions[bot]"
git config user.email "41898+github-actions[bot]@users.noreply.github.com"
git checkout -q -b "$branch" "$BAD"
body="CI failed on main at $short ($RUN_URL); it last passed at ${good:0:9}.
Commits in the range:
$commits

Merge this to revert them, or land a fix with scripts/land.sh and close it."
if git revert --no-edit --no-commit "$good..$BAD"; then
    git commit -q -m "revert: $range, red on CI" -m "$body" -m "Domain: infra"
    git push -q origin "$branch"
    gh pr create --base main --head "$branch" --title "revert: $range, red on CI" --body "$body"
else
    git revert --abort || true
    gh issue create --title "CI red on main at $short" --body "$body

The revert does not apply cleanly; fix forward."
fi
