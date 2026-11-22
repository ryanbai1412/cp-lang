# main-green

Segment: A. Service. Owns: CI results, flaky tests, the land queue, reverts. Worklog: [worklog.md](worklog.md).

## Covers
Keeping main green and moving: lands the backlog (`BRANCHES.md`), runs the queue, triages any red within 15 min (fix or revert, never weaken a test), kills flaky tests, deletes merged/dead branches and lock refs (with Ryan's OK for unmerged).

## Success criteria
CI green on 100% of main commits over a week; median time from red to green < 30 min; 0 branches older than 48 h without an owner; land queue wait < 30 min with all lanes running.

## Ryan's role
None day to day. Approves deleting unmerged branches and anything that rewrites history.
