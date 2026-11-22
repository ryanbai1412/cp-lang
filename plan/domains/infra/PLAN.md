# infra plan

The detail behind [README.md](README.md). Dated 2026-11-22.

## Workstreams, in order

| # | Workstream | Why first / why here | Done when |
|---|---|---|---|
| 1 | **Land the queue backlog** | Phase-2 brief; stale branches confuse every lane. | `devin/repo-layout-doc`, `devin/consolidate-docs` landed or split; 11 superseded `devin/*` branches deleted (`git cherry` shows 0 ahead) and noted in the worklog. |
| 2 | **CI green** | All SLOs measure against CI; today CI is red (run 45067797421: 3 `std::exchange` without `<utility>` in emitted C++, 3 timing tests). | Codegen include requested from perf (or trivially fixed with a note); timing tests made CPU-time based or moved to `--full`; 10 green main runs. |
| 3 | **Domain trailer + land log** | Ryan's ask: know who landed what, keep domains in their paths. | `land.sh` refuses a commit without `Domain: <d>` (d in `plan/DOMAINS.md`), warns on paths outside the domain's owned list, appends a JSONL record per landing; tests in `scripts/test_check_cache.py`. |
| 4 | **Status board** | The coordinator's hourly read. | `scripts/status.py` runs each `plan/domains/*/metrics.toml` command with a timeout, keeps history in `target/status/history.jsonl`, writes a board (per domain: now / 1 h / 1 d / target, landings and reverts from trailers, open questions); scheduled hourly (GitHub schedule writes it to an artifact; locally `just status`). |
| 5 | **Two-tier gate** | 77 s -> 10 s per commit, times 12 lanes. [design/gate-tiers.md](design/gate-tiers.md). | `commit_gate_edit_s` <= 10; land tier = today's gate; test count unchanged. |
| 6 | **Releases** | v0.3 is "production ready" only if people can install it. | Nightly edge publishes; `install-smoke` on Linux/macOS/Windows after publish; dated edges 7 days in a row; `v0.3.0` on Ryan's OK. |
| 7 | **Site deploy from CI** | cp-lang.org is live only from a manual deploy. | `site.yml` green; blocked on Actions secrets (coordinator). |
| 8 | **Gate step registry** | `check.sh` had 56 commits in 3 days from many lanes. | Steps declared in one table (name, tier, cache stage, owner domain); lanes add a row, not shell. |
| 9 | **Public-repo surface** | Going public. | CONTRIBUTING.md, SECURITY.md, stale `check.sh` header fixed. |

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
