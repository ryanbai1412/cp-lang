# Landing, CI and releases

| | |
|---|---|
| Kind | Guideline |
| Owner | infra |
| Applies to | `scripts/land.sh`, the gate, CI workflows, releases, and any change to them |

This guideline covers how changes reach main and users, and what any change
to that machinery must keep true. It is for whoever touches `land.sh`, a
hook, a CI workflow or the release, and for anyone deciding whether a new
check is worth its cost. The exhaustive rules, each with an id, are in
[ci-checklist.md](ci-checklist.md).

## The idea: throughput first, then a green main

Many lanes land work all day. If landing a commit costs a minute, or waits
for a reviewer, every lane slows down, and a slower factory is a bigger
loss than an occasionally red main. So landing does almost nothing (rebase
and push, in seconds), and the expensive checking happens after the push,
in CI, where it costs no lane any time.

A green main is still the goal. It just comes from infra's own work after
the push: tight time limits so a stuck job ends, fast culprit finding when
a run goes red, then a fix forward or a revert. It never comes from adding
cost before a push.

## A picture of it working

A lane commits. The pre-commit hook runs only the tests that commit could
break and finishes within 25 seconds. `land.sh` pushes in about 7 seconds.
CI picks up the newest main commit when the current run ends and answers
within minutes. When a run goes red, infra reads the failing log, names the
commit that broke it, and tells its lane with the failing test. The lane
fixes it forward, or infra reverts it. Within the hour, edge ships from the
newest commit whose shipping checks pass, and anyone can install it with
one line.

## How to decide a case nobody wrote down

- **Would it slow a lane before their push?** Then don't, or make it
  cheaper first. After the push, in CI, almost anything is fine.
- **Does it hold main's CI queue?** A job that decides nothing (a report, a
  timing) runs in its own workflow, not in the run that gates main.
- **Could it hang?** Give it a time limit that fails with a name.
- **Is a test failing?** Find the cause and fix it, or move a slow test to
  a slower tier. Never weaken or delete it to go green.
- **Did you change landing or CI?** Check with the lanes 5 to 15 minutes
  later and compare the commit rate with the hour before. If they're worse
  off, revert first and diagnose after.

## Changes

- 2026-11-23: first version.
