# infra

Segment: A. Service. Owns: CI and workflows (`.github/`), the gate (`scripts/check.sh`, `check_cache.py`, `cxx-cache.py`), landing (`scripts/land.sh`, rebase and push) and the sheriff duty of keeping main green, hooks (`.githooks/`), `scripts/` machinery, `justfile`, `mise.toml`, setup docs, the release pipeline (`release.yml`, `scripts/release/`), installers (`web/public/install.{sh,ps1}`), the site deploy workflow (`site.yml`), public-repo hygiene. Status 2026-11-23: first edge release published (edge-2026-11-23, from 528cddc42); `install.sh` installs it on a clean Ubuntu 24.04; version 0.3.0, two channels (edge, stable by hand); main CI red on lane reds the sheriff is fixing; `land.sh` is rebase, fast checks, clippy on touched and dependent crates, push (about 5-10 s).

Detail: [PLAN.md](PLAN.md) · code: [ARCHITECTURE.md](ARCHITECTURE.md) · numbers: [metrics.toml](metrics.toml) · log: [worklog.md](worklog.md) · gate design: [design/gate-tiers.md](design/gate-tiers.md)

## Mission
Keep every other domain moving: main green at every commit, a gate fast enough that nobody batches around it, landing that never waits on a person, and `curl | sh` installs that just work. Interrupts (red main, broken landing, broken release) beat roadmap work.

## Rollout rules (from Ryan)
- Throughput beats a green badge. Main stays green through infra's own work (regen, quarantine, fix or revert), never by taxing lanes: no blocking pre-merge gate or pre-push hook; lanes push directly.
- Every lane spends under 5% of its time on devex; infra measures it (`land.sh` and hook timings, devex-only commits, red windows).
- After any devex-affecting rollout, check in with the lanes 5-15 min later and compare commit velocity with the hour before. If it hurts them, revert first and diagnose after. A velocity drop is the first alarm.
- Policy messages to all lanes are allowed only within these rules.

## Milestone 1
[Milestone 1](../../../handbook/product/milestones.md) step 3 (Install) and "Release quality" are infra's, on Linux, macOS and Windows, on edge. Deadline 2026-11-23 09:30 PST. Exit criteria:
1. Edge publishes after every green main CI that edge is behind (one release in flight, newest green SHA wins) and nightly, building all three OSes.
2. One line installs a working `cpl` on each OS: `curl -fsSL https://cp-lang.org/install.sh | sh` (Linux, macOS) and `irm https://cp-lang.org/install.ps1 | iex` (Windows), checked end to end by an install-smoke job on all three after every publish.
3. CI runs the tests on Linux, macOS and Windows (the extra OSes on fewer jobs), still <= 3 min end to end; main red-to-green median < 30 min.
4. The release ships the editor assets editors needs (Neovim/Vim config, VS Code `.vsix`, JetBrains plugin), and the smoke job checks they install.
5. Lanes spend < 5% of their time on landing and devex.

Deferring: a stable `v0.3.0`, per-crate commit-tier test selection, the gate step registry, status-board polish, the 16-core runner.

Next 3: (1) install-smoke on all three OSes after each publish; (2) macOS and Windows in the CI test matrix; (3) main green and CI <= 3 min.

## Milestone 2
[Milestone 2](../../../handbook/product/milestones.md), the stable release. The complete infra: nobody thinks about landing, CI or releases. Exit criteria:
1. Main is green at least 95% of the time (7-day window); any red is fixed or reverted within 15 min, and a commit that turns the core gate red and stays red 15 min is auto-reverted.
2. Landing p95 at most 30 s, never over 1 min; every lane's devex time under 5%.
3. Main CI p50 at most 3 min end to end on GitHub-hosted runners, on all three OSes, with every deterministic step cached or skipped by input hash.
4. Two channels: edge from every green tip (one release in flight, newest wins), stable promoted by hand (`v*` tag or dispatch) with CHANGELOG notes, beta while 0.x; Ryan promotes `v0.3.0`.
5. Every release passes install-smoke on Linux, macOS and Windows (sh, PowerShell, `cpl self update`, VS Code `.vsix`, Neovim/Vim setup) before it is marked latest; signed checksums; reproducible builds rebuild byte for byte.
6. The macOS, Windows and editor CI jobs block main again (each after 3 greens in a row).

Next: (1) CI at 3 min (nextest archive shared by shards, compiled-C++ cache, rebalanced shards); (2) auto-revert job for the core gate; (3) install-smoke as a gate on marking a release latest.

## Covers
- **Main green**: CI results, triage of any red within 30 min (fix or revert, never weaken a test), flaky tests, stale `devin/*` branches and lock refs.
- **Devex**: setup (mise, pnpm, uv, just), the gate and its cache, hooks, `just` tasks, CI speed, landing.
- **Release**: release workflow, installers, versioning, the site deploy workflow, CONTRIBUTING/SECURITY/license, history hygiene.
- **Status board**: `scripts/status.py`, running every domain's `metrics.toml` hourly.

Not covered (owner instead): what a gate step checks (the domain whose test it is); emitted-C++ bugs that CI finds (perf); the site's content and the Worker's code (site); `cpl self`/launcher code (tools); scripts that are another domain's tool, e.g. `review-check.py`, `spec-stub.py`, `debt-ratchet.py` (idioms, spec, tech-debt; infra keeps only their gate wiring); Cloudflare and GitHub secrets themselves (coordinator, until a token can write them).

## Success criteria
The bar for v0.3:
1. CI green on every main commit for 7 days; red-to-green median < 30 min.
2. Commit gate (pre-commit) <= 10 s warm after a one-crate edit, 8 cores; land gate (`land.sh`, CI) catches everything the fast gate catches today, with the same test count (1116 + scripts).
3. Every landing carries `Domain: <d>`; `land.sh` refuses without it and warns on paths outside the domain.
4. Hourly status board for all 12 domains, generated, never hand-typed.
5. `curl -fsSL https://cp-lang.org/install.sh | sh` installs a working `cpl` on Linux, macOS and Windows from edge and stable; nightly edge 7 days in a row; `v0.3.0` cut.
6. A fresh clone reaches a green gate with the 5 documented commands.

## Milestones
- **M1 Main is green and accountable** — CI `gate` and `site build` green on main 10 commits in a row; `land.sh` requires the `Domain:` trailer and warns on foreign paths (tested in `scripts/test_check_cache.py`); `land.sh` logs each landing (domain, seconds, tries) and `land_stats.py` reports `land_p95_s` <= 30; superseded `devin/*` branches deleted; `scripts/status.py` writes `plan/STATUS-BOARD.md` from every `metrics.toml`.
- **M2 10-second commit gate** — two tiers ([design/gate-tiers.md](design/gate-tiers.md)): pre-commit runs only what the change can affect (per-crate test selection from the cargo graph, cached by inputs), the full fast gate runs in `land.sh` and CI. Exit: `metrics.toml` `commit_gate_edit_s` <= 10 on 8 cores, `land_gate_warm_s` <= 60, test count unchanged.
- **M3 Releases that install** — edge published nightly by the GitHub schedule; `install-smoke` job green on 3 OSes after publish; site deploys from CI; `v0.3.0` stable cut on Ryan's OK; CONTRIBUTING.md and SECURITY.md exist.

## Metrics
All measured 2026-11-22 on main `4b8f504f`, 8-core VM; commands in [metrics.toml](metrics.toml).

| Metric | Command (short) | Today | Target | M |
|---|---|---|---|---|
| Warm gate, nothing changed | `scripts/check.sh --fast` (2nd run) | 3.7 s | <= 5 s | M2 |
| Warm gate, one-line edit in `cpl-fmt` | append a comment, `check.sh --fast` | 77 s (nextest 57 s) | <= 10 s | M2 |
| Fast gate after a pull (build + tests) | `check.sh --fast` on a fresh tree | 101 s (nextest 79 s) | <= 60 s | M2 |
| Slowest fast-gate test | nextest output | 40.4 s `stress_corpus` | <= 5 s in commit tier | M2 |
| CI green on main, last 10 completed runs | GitHub Actions API | 0/6 (5 cancelled, 1 failed) | 10/10 | M1 |
| Site deploy green, last 10 runs | GitHub Actions API | 0/6 (5 failed) | 10/10 | M3 |
| Commits with `Domain:` trailer, last 50 | `git log` trailers | 0/50 | 50/50 | M1 |
| Superseded `devin/*` branches (`git cherry` 0 ahead) | `git cherry` loop | 11 of 68 | 0 | M1 |
| Unlanded `devin/*` branches > 48 h old | `git cherry` loop | 0 | 0 | M1 |
| Releases published | GitHub releases API | 0 | edge daily + `v0.3.0` | M3 |
| Live `install.sh` == repo file | `curl` + `cmp` | yes | yes | M3 |
| Root CONTRIBUTING.md + SECURITY.md | `ls` | 0/2 | 2/2 | M3 |
| Land queue wait p50 | `land.sh` log (M1) | not measurable: no log yet | <= 5 min | M1 |

## Judgment rubric
Numbers miss these; a cold agent scores 1-5 with a reason, on the probes below.
1. When the gate fails, does its output name the failing check and the exact command to rerun it, without scrolling?
2. Can a newcomer tell which tier a test belongs in, and move it, from `check.sh --help` and this folder alone?
3. Does landing feel like one command that either lands or says precisely why not?
4. Is every number on the status board reproducible by pasting its command?
5. After a red main, does the history explain what broke and how it was fixed?

Probes (cold agent, timed, files touched counted):
- **Fresh clone**: from `README.md` "Setup" only, reach a green `just check`. Pass = green, < 30 min, no undocumented step.
- **Add a gate step**: add a new fast check for a made-up script; must land in the right tier and cache stage.
- **Planted red**: three commits land, one breaks a test; acting as sheriff from the CI log and `git log` over the run's range, the agent must name the culprit and revert or fix it within one CI run.
- **Cut a release**: from docs only, dry-run an edge release and verify the artifacts.

## Improve (highest impact first)
1. Make CI green (std `<utility>` include via perf; timing tests deterministic or `--full`) — every other SLO measures against it.
2. Two-tier gate with per-crate test selection — 77 s -> 10 s per commit for 12 lanes.
3. `Domain:` trailer in `land.sh` + land event log + `scripts/status.py` board.
4. Land `devin/repo-layout-doc`, `devin/consolidate-docs`; delete 11 superseded branches.
5. First edge publish + `install-smoke` on 3 OSes; site deploy from CI once secrets exist.
6. Split `check.sh` into a step registry so lanes stop editing one 376-line file (56 commits in 3 days).
7. CONTRIBUTING.md, SECURITY.md; stale `check.sh` header.

## Decisions for Ryan
Each is in [plan/QUESTIONS.md](../../QUESTIONS.md) with options and a default I work on meanwhile.
1. **Two gate tiers** — pre-commit runs affected-crate tests (<= 10 s); the full fast gate moves to `land.sh` and CI. Default: build it.
2. **CI red blocks landing?** — superseded by decisions.md 234: `land.sh` never gates; CI gates every main tip and the infra sheriff reverts or fixes a red one.
3. **Unlanded branches** — archive (`refs/archive/<name>`, restorable) then delete `devin/*` branches with no owner after 48 h. Default: delete only `git cherry`-superseded ones.

## Ryan's role
None day to day. Signs the gate targets once (decision 1 above), approves cutting `v0.3.0` and the fresh public repo, and anything that rewrites history or deletes unlanded work.
