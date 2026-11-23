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

## General fixes owed

| Instance | General fix | Owner |
|---|---|---|
| Lanes `git push` directly and skip land.sh, so the debt ratchet, fmt and clippy reds reach main (5 allowlisted files grew, 2026-11-22) | No new gate (Ryan: throughput first). Infra absorbs it: regen heals derived files, the sheriff fixes or reverts within one CI cycle, and checks only fail on the lane's own diff | infra |
| Hand-picked lint codes collide across lanes (L0553, L0563, 2026-11-22) | `cpl lint --new` allocates the next free code | lint-fmt |
| `cpl fmt` with no path or with stdin rewrote the whole repo (461-file revert, 2026-11-22) | No path means stdin only; never the tree in place | lint-fmt |
| A std/*.cp edit forces a 60 s release rebuild because std is embedded (2026-11-22) | Dev builds read std/ from disk | tools / compiler |
| cpl.wasm at 2,098,812 of its 2,100,000 B gz cap (2026-11-22) | Cap set from a size budget per feature, with headroom, or cut size | site |
| Queued main CI runs get cancelled by newer pushes, so platform fixes take 15-20 min to verify (2026-11-22) | A dispatchable per-SHA platform run, outside the main concurrency group | infra |
| `cpl-check` domains test read `corpus/problems/atcoder_dp_l/port.cp`; idioms' rewrite broke it (2026-11-23) | Tests embed their source; no test reads a corpus port idioms may rewrite | compiler |
| A lint help rewording broke `cpl-lsp` hover.expected (2026-11-23) | LSP fixtures snapshot lint codes and spans, not lint prose already tested in cpl-lint | tools |
| A std change raised the corpus lint baseline (L0204/L0206/L0210/L0509) | The commit tier lints the touched `std/*.cp` against the baseline in seconds | infra (wiring), lint-fmt (check) |
| `IndexOp` in `std/ntt.cp` missing from the VS Code tables | Generate the std names in `languageData.ts` and the grammar from `std/` in regen, instead of a hand list checked by a test | tools |
| L0206 fired on a user `rev` method | Getter lints read the receiver's checked type, not a syntactic guess | lint-fmt |
| A new spec rule (R01.1.4) landed without a case, turning coverage red | `spec-stub.py` writes the case skeleton (or a gap entry) with every new rule id | spec |
| regen never ran skill-gen in CI (no `cpl` built) and one failing step stopped the rest | Done: regen builds `cpl`, runs every step, refreshes `pnpm-lock.yaml` | infra |
| `os` and `debugger-loop` held the runners on every push; a hung job held main for 90 min | Done: `os` hourly, superseded runs cancel, 20 min job timeouts | infra |
| Core-gate reds wait on a sheriff | An auto-revert job: a commit that turns the core gate red and stays red 15 min is reverted, with a note to its owner | infra |
| A skill source changed without regenerating the bundled skill (stale `seq.md`, `strings.md`, red checks) | Done: regen commits derived files on main and the main gate only warns on them (regen red if a generator fails); still owed: the pre-commit hook regenerates them when `std/` is staged | infra |
| cf1943b stopped checking and nothing caught it | Done: `scripts/port-check.sh` ratchet over every port | infra |
