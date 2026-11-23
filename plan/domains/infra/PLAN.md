# infra plan

The detail behind [README.md](README.md). Dated 2026-11-22.

## Workstreams, in order

| # | Workstream | Why first / why here | Done when |
|---|---|---|---|
| 1 | **Install-smoke on 3 OSes** | Milestone 1 step 3, on Linux, macOS and Windows. | After `publish`, the sh one-liner on Linux and macOS and the PowerShell one-liner on Windows install edge, `cpl run` a sample, and the editor assets install. |
| 2 | **macOS and Windows in CI** | All three OSes supported and tested. | CI runs the tests on each OS (the extra OSes on fewer jobs); paths and line endings covered. |
| 3 | **Main green, CI <= 3 min** | Edge only publishes from green commits. | Red-to-green median < 30 min; CI <= 3 min from caching and input-hash skipping; per-job times in the worklog. |
| 4 | **Landing friction < 5%** | Lanes skip slow checks, and skipped checks turn main red. | `land.sh` runs the checks behind most reds in seconds; `land_p95_s` <= 30. |
| 5 | **Deferred** | Not needed for the milestone. | Stable `v0.3.0`, per-crate commit-tier selection, the gate step registry, status-board polish, CONTRIBUTING/SECURITY. |

## Why these milestones
- **M1 (green + accountable)** first because every lane's work is judged against main, and Ryan asked how domains stay inside their lines: the trailer and the board are the mechanism.
- **M2 (10 s gate)** second: the largest daily cost to all lanes, but it needs a green CI to prove the land tier still catches what the commit tier skips.
- **M3 (releases)** last: the pipeline exists and dry-ran green; what remains is publishing, smoke on 3 OSes and the version bump, which wait on the v0.3 bar anyway.

## Alternatives considered
- **Required GitHub PR checks instead of `land.sh`**: rejected; Ryan wants direct pushes to main, and `land.sh` already gates the exact tip and trains lanes.
- **Self-hosted fast CI runner**: faster CI but private infrastructure the public repo cannot mention or reproduce; GitHub-hosted runners stay the reference.
- **External scheduler for the nightly**: Ryan chose the GitHub schedule; it can skip runs under load, so the board flags a missing dated edge.
- **Auto-revert on red CI**: rejected for now; a revert on a timing flake costs more than a 30-min triage. Revisit with Q2.

## Risks
- **Test selection misses a dependency** (a test reading corpus files a change touched): the land tier still runs everything, so the cost is a failed land, not red main.
- **GitHub runner differs from the VM** (libstdc++ headers, core count): CI is the reference; timing budgets must be CPU time or relative.
- **Trailer enforcement blocks lanes mid-sprint**: ship as a warning for one day, then enforce.
- **Status board commands are slow or flaky**: per-command timeout, failures shown as `error`, never as a stale number.

## First steps in phase 2
1. Rebase and land `devin/repo-layout-doc` and `devin/consolidate-docs`; delete superseded branches.
2. Diagnose CI red; file the `<utility>` request in `plan/domains/perf/worklog.md`; fix or move timing tests.
3. `Domain:` trailer check (warn) + land log in `land.sh`, with tests.
4. `scripts/status.py` v1 over every existing `metrics.toml`.
