# devex

Segment: A. Service. Owns: `scripts/` (gate, land, hooks, gen), `justfile`, `mise.toml`, Python tooling, setup docs. Worklog: [worklog.md](worklog.md).

## Covers
Everything a contributor (agent or person) touches before writing code: setup, the gate, hooks, `just` tasks, generated files, CI speed, the landing script's mechanics.

## Improve
Warm gate 5.5 min on a warm 2-core VM today -> seconds (cache by input, not tree hash; split slow tests out of the commit gate); `land.sh` already lands trains of up to 8 queued lanes with one gate run (and bisects a failing train), so landing throughput is bounded by gate time: make the gate fast first; `just gen --check` as the one way to regenerate; uv everywhere; Devin snapshot warm and green.

## Success criteria
Warm `just check` ~10 s (Ryan's target; today 68 s on 8 cores, 52 s of it nextest, led by a handful of 10-33 s end-to-end tests), cold < 10 min on CI; land queue wait < 30 min with every lane running; a fresh VM goes from clone to green gate with the documented 5 commands.

## Ryan's role
None day to day. Signs off the gate-time and setup targets once.
