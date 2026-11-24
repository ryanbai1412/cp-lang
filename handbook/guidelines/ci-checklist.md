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
finished in 30 min, edge is over 2 hours old, or cp-lang.org has served a
commit over 2 hours old while a newer one passed its site jobs.
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
`scripts/release/shippable.sh`, `scripts/site-deployable.sh` (which skips
superseded runs, so it stays inside the API rate limit; `scripts/test_workflows.py`).

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

### ci/R27

`land.sh` pushes only committed work: it stops when nothing is ahead of
main, and never regenerates files from uncommitted changes.
*Why:* a commit whose hook failed leaves its edits uncommitted, and
regenerating from them pushed generated files without their source, so main
went red. *Example:* the hook rejects a stdlib fix; `land.sh` then says
"nothing to land" instead of pushing the new conformance case alone.
*Enforced by:* `scripts/land.sh`, tested in `scripts/test_check_cache.py`
`test_land_refuses_when_nothing_is_ahead_of_main` and
`test_land_never_regenerates_from_uncommitted_changes`.

### ci/R28

Every GitHub Actions job sets `timeout-minutes`.
*Why:* an uncapped job runs for GitHub's 6 h default, and on main it holds
the run slot so no later commit gets a result; a hung Windows CLI job held
main for 18 min before it had a cap. *Example:* a new `lint` job without
`timeout-minutes: 10` fails `test_every_job_has_a_timeout`.
*Enforced by:* `scripts/test_workflows.py`, run by `scripts/check.sh`.

### ci/R29

The blocking `vscode` and `neovim` jobs run on Linux, macOS and Windows,
and `cli` on macOS and Windows next to the Linux gate. No ci.yml job is
marked `continue-on-error`: an outage check (edge installable) reports from
the watchdog instead.
*Why:* contestants install on all three, and a path, CRLF or clipboard bug
on one OS ships silently when only Linux blocks. *Example:* dropping
`windows-latest` from `cli`, or making it informational, fails
`test_blocking_jobs_cover_every_os`.
*Enforced by:* `scripts/test_workflows.py`, run by `scripts/check.sh`.

### ci/R30

A shrink-only list never fails main because an entry got fixed: the check
drops the entry itself and says to commit the shorter list.
*Why:* a fix in std made a listed doc example run, `gen-ref` failed the site
build on it, and cp-lang.org went 4 h without a deploy; failing on a
regression is the list's job, failing on an improvement only stalls
shipping. *Example:* `seq.Rev.src` starts running; the site build passes,
prints the line it dropped from `web/scripts/doc-examples-failing.txt`, and
the next commit there carries the shorter file.
*Enforced by:* `web/scripts/gen-ref.mjs`, `crates/cpl-conform` (newly
passing cases print, never fail).

### ci/R31

Every `fail` in `scripts/check.sh` says what to do next: the command to
rerun, the fix, or where the finding above names it.
*Why:* a lane that sees only "tests failed" spends minutes finding which
check failed and how to reproduce it, and that time counts against the 5%
devex budget. *Example:* `fail "tests failed"` fails
`test_every_gate_failure_says_how_to_fix_it`; `fail "tests failed; rerun:
cargo nextest run -p cpl-check"` passes.
*Enforced by:* `scripts/test_check_cache.py`, run by `scripts/check.sh`.

### ci/R32

A cached `target/` records the commit it was built at (`target/.ci-sha`) only
when the job that built it passed; a red job saves it with the base it
restored.
*Why:* `scripts/ci-mtime.sh` back-dates every file unchanged since that
commit, so cargo trusts the artifacts. A job that failed before compiling a
changed crate, saved as built at HEAD, made every later run reuse a stale
`cpl-lint` build script and fail on a `lint.toml` key main already accepted.
*Example:* the clippy gate fails at check; its cache is saved with the old
`.ci-sha`, and the next run rebuilds what changed since then.
*Enforced by:* `scripts/test_workflows.py`, run by `scripts/check.sh`.

### ci/R33

Every archive a release publishes carries a Sigstore build-provenance
attestation from the `sign` job in `.github/workflows/release.yml`.
*Why:* `SHA256SUMS` proves a download matches the release, not that the
release came from this repository's workflow; an attestation ties each
archive to the commit and workflow that built it, with no key to leak.
*Example:* `gh attestation verify cpl-aarch64-apple-darwin.tar.gz -R
ryanbai1412/cp-lang` passes for an edge archive; a rebuilt or swapped one
fails.
*Enforced by:* `scripts/test_workflows.py`, run by `scripts/check.sh`.

### ci/R34

A test that asserts a wall-clock bound under 5 s is listed in
`WALL_CLOCK_TESTS` in `scripts/ci.py`, so it runs only in the
non-blocking timing job and `--full` (decision 225).
*Why:* a busy hosted runner can be several times slower than a laptop, so a
50 ms budget in a blocking tier fails main with no code change and the red
does not reproduce locally. Bounds of 5 s or more guard against hangs, and
those may block.
*Example:* a new `assert!(t.elapsed() < Duration::from_millis(50))` in
`crates/cpl-lsp/tests/` fails
`test_every_short_timing_budget_is_kept_out_of_the_blocking_tiers` until its
test is listed.
*Enforced by:* `scripts/test_wall_clock.py`, run by `scripts/check.sh`.

### ci/R35

**Tests run once: no retries.** A test partition runs nextest without
`--retries`, so a flaky test fails the run on its first failure.
*Why:* a retry that passes hid each flake while main stayed green, and even
`--flaky-result fail` spends a second run on it; the audit's target for test
retries and flaky tests is 0. Failing on the flake makes its owner fix it.
*Example:* a test that races a temp file fails its partition; its owner
reruns it with `cargo nextest run -E 'test(<name>)'` and fixes the race.
*Enforced by:* `ci_retries` in `scripts/debt-ratchet.py` (hard zero), which
counts any `--retries` outside `scripts/retry.sh` and `scripts/retry.py`.

### ci/R36

Every contributor task has one `just` recipe: each command in infra's
"Suites and tools" table runs from a recipe, every recipe has a comment that
`just --list` shows, and every `just <name>` a live doc names exists.
*Why:* a contributor who has to find a script's path and its interpreter
(`python3`, `uv run`, a shell script) loses minutes per task, and a doc that
names a recipe nobody defined sends them nowhere.
*Example:* adding `scripts/foo_stats.py` to the table without a `foo-stats`
recipe fails the gate with `handbook/domains/infra.md: add a justfile recipe
that runs it`.
*Enforced by:* `scripts/test_justfile.py`, run by `scripts/check.sh`.

### ci/R37

CI compiles the workspace test binaries once per run: `tests (build)` packs
them with `cargo nextest archive`, and every `tests (hash:N/M)` shard runs
that archive (`CPL_NEXTEST_ARCHIVE`) without restoring a target/ or
invoking the compiler.
*Why:* six shards that each compiled the same binaries spent about 2.5 min
apiece before their first test, which kept main CI far from its 3-minute
target and cost one runner-hour every few runs.
*Example:* adding a `Restore target/` step or a `cargo build` to the `tests`
job fails `test_the_workspace_tests_compile_once`.
*Enforced by:* `scripts/test_workflows.py`, run by `scripts/check.sh`.

### ci/R38

Every workflow sets Rust up through `.github/actions/setup-rust`: the pinned
toolchain, the cargo caches and, with `target:`, the target/ that ci.yml
saves under `TARGET_KEY: target-ci-v2`, re-stamped by `scripts/ci-mtime.sh`.
*Why:* 20 hand-copied setups had drifted: ci-slow and culprit still
restored `target-ci-v1`, which nothing has saved since the key moved to v2,
so both rebuilt the workspace from scratch.
*Example:* a job that runs `rustup toolchain install` itself, or a
`TARGET_KEY: target-ci-v1`, fails
`test_every_workflow_sets_rust_up_through_one_action`.
*Enforced by:* `scripts/test_workflows.py`, run by `scripts/check.sh`.

### ci/R39

A touched path runs its owners' slice first: `SLICES` in
`scripts/affected.py` names the tests (e.g. `std/` runs cpl-lint's
`corpus_warnings_only_go_down` and the trace goldens), and a slice whose
slowest test takes over 20 s fails the commit.
*Why:* the commit tier is cut at 25 s, so a `std/` commit spent it on clippy
and never reached the lint test it can break; main went red on it twice in
30 minutes. *Example:* a std edit that raises L0550 fails
`touched-path slices: vm-trace corpus-warnings`.
*Enforced by:* `scripts/affected.py`, `scripts/test_affected.py`.
