# Landing, CI and releases: checklist

| | |
|---|---|
| Kind | Checklist |
| Owner | infra |
| Applies to | `scripts/land.sh`, the gate, CI workflows, releases |

Every rule for landing, CI and releases, with a stable id. The ideas behind
them are in [ci.md](ci.md). Gates and register rows cite these ids
(`handbook/guidelines/ci-checklist.md#cir3`).

## Landing

### ci/R1

Landing never waits on a person or a blocking pre-merge check.
*Why:* a lane that waits ships less. *Example:* `scripts/land.sh` rebases
and pushes, with no review step. *Enforced by:* `land.sh` itself; review.

### ci/R2

`land.sh` takes under 30 s at p95.
*Why:* lanes land many times an hour. *Example:* `land: pushed <sha> to
origin main in 7s`. *Enforced by:* `scripts/land_stats.py` (report).

### ci/R3

The commit tier runs only what the staged paths can affect, and every path
maps to a selection.
*Why:* an unmapped path falls back to the whole fast gate and blows the
budget. *Example:* a `handbook/` edit runs only the handbook checker.
*Enforced by:* `scripts/affected.py`, `scripts/test_affected.py`.

### ci/R4

The commit tier has a 25 s budget; past it, it warns and lets the commit
through.
*Why:* a slow hook costs every lane; CI runs the rest. *Example:*
`pre-commit: WARNING: the commit tier passed 25s`. *Enforced by:*
`scripts/check.sh`.

### ci/R5

Every commit names its domain in a `Domain:` trailer and has an
`area: summary` subject.
*Why:* culprit routing and per-lane metrics read them. *Enforced by:*
`land.sh`, `.githooks/commit-msg`.

### ci/R6

No added line carries a secret, a home path or an unlisted host.
*Why:* history is public. *Example:* a host must be in
`scripts/public-hosts.txt`. *Enforced by:* the staged scan in
`scripts/check.sh`.

## CI

### ci/R7

A running main CI run is never cancelled by a newer push; the newest tip
waits and runs next.
*Why:* a run that keeps getting cancelled never answers. *Enforced by:*
`concurrency` in `.github/workflows/ci.yml`.

### ci/R8

Every long CI step has a `timeout-minutes` cap.
*Why:* a hung step holds main's only run slot. *Example:* the site `Build`
step stops after 8 minutes. *Enforced by:* review (a lint is missing).

### ci/R9

A job that blocks nothing runs outside `ci.yml`.
*Why:* it holds the run open and delays the next tip. *Example:* wall-clock
budgets run in `ci-slow.yml`. *Enforced by:* review.

### ci/R10

Each commit's `ci/covered` status says whether a finished run covered it.
*Why:* most commits never get a run of their own. *Enforced by:*
`scripts/commit_status.py`, `.github/workflows/commit-status.yml`.

### ci/R11

A red main gets a named culprit: read the log first, bisect only when that
fails, and verify on a fresh `origin/main` build.
*Why:* wall-clock time to a fix is what matters. *Enforced by:*
`scripts/culprit.py`, `.github/workflows/culprit.yml`; process.

### ci/R12

The culprit's lane fixes forward; if main is still red 20 minutes after
they hear, infra reverts.
*Why:* a red main hides the next break. *Enforced by:* process.

### ci/R13

A test is never weakened or deleted to turn main green; a slow test moves
to `--full`.
*Why:* a green badge over a hole is worse than red. *Enforced by:*
`scripts/debt-ratchet.py` (`rust_ignore`, skip lists); review.

### ci/R14

Tests are deterministic, and their time limits hold on a loaded 4-core
runner.
*Why:* a false red costs everyone a triage. *Example:* a conformance case
that needs longer sets `// timeout: 120`. *Enforced by:* the harness
timeouts; review.

### ci/R15

Generated files never block a commit; CI's `regen` job rewrites them.
*Why:* regeneration races with other lanes. *Enforced by:* `scripts/gen.py
--check` (warns), the `regen` job.

### ci/R16

The CI caches (`target/`, C++ objects, check results) are keyed so a hit
can never hide a change.
*Why:* a stale cache makes a red look green. *Enforced by:*
`scripts/check_cache.py`, `scripts/cxx-cache.py` and their tests.

## Rollouts

### ci/R17

After any change to landing, hooks or CI, check with the lanes 5 to 15
minutes later, compare commits per 15 minutes with the hour before, and
revert if they are worse off.
*Why:* throughput comes before a green badge. *Enforced by:*
`scripts/devex_stats.py`; process.

### ci/R18

Each lane spends under 5% of its time on landing, CI and tooling.
*Enforced by:* `scripts/devex_stats.py` (a proxy).

## Releases

### ci/R19

Edge ships at least hourly from the newest main commit whose shipping
checks pass, even when unrelated jobs are red.
*Enforced by:* `scripts/release/shippable.sh`, `.github/workflows/release.yml`.

### ci/R20

Stable ships only from a fully green commit, and only when Ryan decides.
*Enforced by:* the stable path of `release.yml`; process.

### ci/R21

Installers verify every download against `SHA256SUMS`, and each release is
install-smoked on Linux, macOS and Windows.
*Enforced by:* `web/public/install.{sh,ps1}`, `release.yml` `install-smoke`.

### ci/R22

The browser build matches the native compiler.
*Why:* the playground must not disagree with `cpl`. *Enforced by:*
`scripts/build-wasm.sh`, the site build job.

## Changes

- 2026-11-23: first version, from the infra charter and landing process.

### ci/R23

An open "main CI stalled" issue exists exactly while no main run has
finished in 30 min or edge is over 2 hours old.
*Why:* a stuck queue or a stale edge is silent otherwise, and every lane
lands blind until someone notices. *Example:* a hung job holds main's run
slot for 40 min; the issue opens at 30 and closes when the next run ends.
*Enforced by:* `scripts/ci-watchdog.sh`, `.github/workflows/watchdog.yml`.

### ci/R24

Each workflow and job grants only the token permissions it uses, and no
workflow runs on `pull_request_target`.
*Why:* a workflow that runs untrusted code with a write token can push to
main or publish a release. *Example:* `commit-status.yml` gets
`statuses: write` and nothing else; only the release publish job gets
`contents: write`. *Enforced by:* **missing**; review of `.github/` changes.

### ci/R25

Each thing main ships (edge, the site) goes out from the newest commit whose
own jobs passed, not only after a fully green run.
*Why:* at the landing rate most runs are superseded or red on an unrelated
job, so waiting for a full green run holds shipping for hours. *Example:* a
red interpreter test doesn't stop the site deploying from a commit whose
`site build` and `site browser tests` passed. *Enforced by:*
`scripts/release/shippable.sh`, `scripts/site-deployable.sh`.

### ci/R26

Edge ships from the newest main commit that passes the release build and
its archive and installer smoke tests, and whose CI shipping jobs are no
worse than at the last shipped edge.
*Why:* requiring every shipping job green held edge for 7.5 h behind reds
unrelated to what users install, while a regression against the last edge
is exactly what must not ship. *Example:* a conformance part red at edge and
still red now doesn't block; a `cli` job green at edge and red now does.
*Enforced by:* `scripts/release/shippable.sh` (tested in
`scripts/test_release.py` `ShippableTest`), `.github/workflows/release.yml`.
