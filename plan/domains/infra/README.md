# infra

Segment: A. Service. Owns: CI results and workflows, the land queue, reverts, `scripts/` (gate, land, hooks, gen), `justfile`, `mise.toml`, Python tooling, setup docs, `scripts/release`, installers, public-repo hygiene. Worklog: [worklog.md](worklog.md). Architecture: [ARCHITECTURE.md](ARCHITECTURE.md).

Keeps every other lane moving: main green, a ~10 s warm gate, fast landing, and stable/edge/nightly releases that just work. Interrupts (red main, broken landing, broken release) beat roadmap work.

## Covers
**Main green.** Keeping main green and moving: lands the backlog (`BRANCHES.md`), runs the queue, triages any red within 15 min (fix or revert, never weaken a test), kills flaky tests, deletes merged/dead branches and lock refs (with Ryan's OK for unmerged).

**Devex.** Everything a contributor (agent or person) touches before writing code: setup, the gate, hooks, `just` tasks, generated files, CI speed, the landing script's mechanics.

**Release.** CI workflows, release pipeline, installers, versioning, the public-repo surface (CONTRIBUTING, SECURITY, license, history hygiene).

## Improve
**Main green.** 

**Devex.** Warm gate 5.5 min on a warm 2-core VM today -> seconds (cache by input, not tree hash; split slow tests out of the commit gate); `land.sh` already lands trains of up to 8 queued lanes with one gate run (and bisects a failing train), so landing throughput is bounded by gate time: make the gate fast first; `just gen --check` as the one way to regenerate; uv everywhere; Devin snapshot warm and green.

**Release.** First edge + nightly; install smoke on 3 OSes; CI on push/PR; commit-message + pushed-range scan in CI; `full.yml`; version 0.3.0; fresh public repo when Ryan says.

## Success criteria
**Main green.** CI green on 100% of main commits over a week; median time from red to green < 30 min; 0 branches older than 48 h without an owner; land queue wait < 30 min with all lanes running.

**Devex.** Warm `just check` ~10 s (Ryan's target; today 68 s on 8 cores, 52 s of it nextest, led by a handful of 10-33 s end-to-end tests), cold < 10 min on CI; land queue wait < 30 min with every lane running; a fresh VM goes from clone to green gate with the documented 5 commands.

**Release.** `curl -fsSL https://cp-lang.org/install.sh | sh` installs a working `cpl` on Linux/macOS/Windows from stable and edge; nightly edge 7 days in a row; `v0.3.0` cut.

## Ryan's role
**Main green.** None day to day. Approves deleting unmerged branches and anything that rewrites history.

**Devex.** None day to day. Signs off the gate-time and setup targets once.

**Release.** Approves cutting `v0.3.0` and the fresh public repo; otherwise none.
