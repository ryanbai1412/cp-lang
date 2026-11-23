# infra plan

The detail behind [README.md](README.md). Dated 2026-11-22.

## Workstreams, in order

| # | Workstream | Why first / why here | Done when |
|---|---|---|---|
| 1 | **Install-smoke after every publish** | Milestone 1 step 3: a friend installs without help. | A job after `publish` runs `install.sh` on clean Linux and macOS and `install.ps1` on Windows, then `cpl run` on a sample and installs the `.vsix` into VS Code. |
| 2 | **Main green, CI <= 3 min** | Releases only come from green commits; slow CI makes each red cover many commits. | Red-to-green median < 30 min; CI wall time <= 3 min from caching and input-hash skipping; per-job times in the worklog. |
| 3 | **One-step VS Code install** | Milestone 1 "Editor support". | The extension installs with one command or click (Open VSX, or `cpl` installing the release's `.vsix`). |
| 4 | **Landing friction < 5%** | Lanes skip slow checks, and skipped checks turn main red. | `land.sh` runs the checks behind most reds in seconds; `land_p95_s` <= 30. |
| 5 | **Stable `v0.3.0`** | Milestone 1 "Release quality". | Cut on Ryan's OK from a green commit; installers default to it. |
| 6 | **Deferred** | On the milestone's "can wait" list. | Per-crate commit-tier selection, the gate step registry, status-board polish, CONTRIBUTING/SECURITY. |

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
