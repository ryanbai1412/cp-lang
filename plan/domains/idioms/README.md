# idioms

Segment: D. Taste and co-design. Owns: bundled skill, principles, idiom catalog, `research/dryrun`, `corpus/` (ports + new problems), `crates/cpl-judge`, scoreboard (corpus suite), `plan/domains/spec/delta/`. Status (2026-11-22, late): skill on main (`skill-gen.py --check`: 50 files, 76 blocks); 289/305 tracked ports check on v0.3; `corpus/sets/v03.txt` has 309 judge-proven entries, all 8 profiles.
Packet: [PLAN.md](PLAN.md) (detail), [metrics.toml](metrics.toml) (commands + today's values), [ARCHITECTURE.md](ARCHITECTURE.md) (code as it is), [worklog.md](worklog.md).

## Milestone 1 (priority over everything below; due about 09:30 PST 2026-11-23)
[plan/product/MILESTONE-1.md](../../product/MILESTONE-1.md): a Codeforces 1500+ friend who has never used CPL understands it, tries it in the browser, installs it, solves unfamiliar problems, tests them, builds them for Codeforces, and competes, all without help. Idioms owns the **acceptance run** that says whether this works, grown out of the blind skill evals. The run ships on edge, needs no stable v0.3.0, and makes no real judge submissions.

**What the milestone needs from idioms (exit criteria)**
- **Protocol and harness** (done, `c325c1b05`). [research/acceptance/PROTOCOL.md](../../../research/acceptance/PROTOCOL.md) defines the run: a cold agent plays the friend using only https://cp-lang.org, the pages it links and Codeforces statements, with Neovim as the editor. Each run writes `research/acceptance/runs/<date>-<n>.json` with pass or fail per step, minutes per step, the OS, and every blocker, workaround and hesitation, each tagged with its lane.
- **Three OSes.** Every run covers Linux, macOS and Windows. The agent drives Linux; CI runners on each OS do the `install` step and the CLI half of `test` and `submit`, following the site's instructions word for word.
- **Unseen problems** (picked, `6165c9599`). Ten Div2 problems (Codeforces 2285A-E and 2289A-E) plus a timed virtual round (2272A-E, 2 h), all newer than the corpus. Each needs a local judge: reference C++, `gen.py` and a checker where answers aren't unique.
- **Codeforces profile.** Every solution is built with `--judge codeforces`, compiled with Codeforces' g++ flags, and kept under 64 KB. Every v03 entry already builds under that profile (`every_v03_port_checks`).
- **Blocker list** (generator done). [plan/product/BLOCKERS.md](../../product/BLOCKERS.md) is generated from the runs by `research/acceptance/blockers.py` (registered in `gen.toml`), ranked by kind, then step blocked, then frequency, then minutes lost. It is refreshed every 2 hours, and each lane gets its rows in its worklog.
- **Done when:** the latest run passes 7/7 steps on all three OSes; Solve gets local-judge accepts on at least 8 of 10; Compete solves at least 4 of 5 in 2 h with nothing lost to missing ordinary functionality or a tool failure; BLOCKERS.md has no open blocker row.

**Deferred** (cheap or gate-only until the milestone passes; continues in parallel): idiomatic re-ports from `dream.cp` and the token and taste targets (M3); corpus growth past 100 and `cpl-conform corpus` 100/100; port lint-warning triage, except lints the friend meets; the unseen-20 skill gate, which the acceptance set replaces for now. Mechanical port migration continues only when a stdlib or compiler fix unblocks a port.

**Top 3 next steps**
1. Local judges for the 15 held-out problems, so run 1 (started 21:29, Linux) can be scored.
2. Score run 1, publish BLOCKERS.md, send each lane its rows, then rerun every 2 hours, with a worklog checkpoint each time.
3. A CI job on Linux, macOS and Windows that follows the site's install and CLI steps and writes its results into the run.

## Mission
What idiomatic v0.3 looks like, written down (skill, principles, idiom catalog) and proven (re-ports on every judge, dry runs, blind evals, new problems). A cold model given the skill writes code that says the editorial's idea, and the corpus proves it is accepted on every judge.

## Covers
**Skill.** The bundled AI skill (`skills/cp-lang/`), the principles, the idiom catalog (which lint and docs build on), blind evals, the dry-run set.

**Corpus.** The ports and their C++ originals, judge profiles (`crates/cpl-judge`), the corpus suite of the scoreboard, bug-replay and revision suites; growing the problem corpus (`meta.json`, `dream.cp`/`spec.cp`), with gaps written up in `plan/domains/spec/delta/`.

## Not covered (owner instead)
- Lint rules and their engine: **lint-fmt** (idioms supplies the catalog and the FP corpus).
- Stdlib APIs the skill documents: **stdlib**; compiler/codegen bugs found by evals: **compiler**.
- Docs site and `writable-from-docs` publishing: **site** (shared eval).
- `cpl-conform` runner and `scripts/` (review-check, corpus.py): **infra** (idioms is the main user).
- Normative spec text: **spec**; idioms only writes `plan/domains/spec/delta/` (non-normative).

## Success criteria for v0.3
**Skill.** On main and gated; a blind skill-only run of the 20 dry-run problems passes 20/20 with 0 compiler bugs, 0 workarounds and 0 non-idiomatic lint flags; then 18/20 on a fresh, unseen set.

**Corpus.** 305/305 v0.3 re-ports accepted under every applicable judge profile; tokens median <= 0.6x C++, none > 1.0x; this corpus is lint's and fmt's false-positive gate; new problems each week with dream + spec ports; dream-vs-spec gap count trending to 0.

## Milestones
**M1. Skill on main, measured blind.** Exit: `skills/cp-lang/` and `scripts/skill-gen.py --check` on main in the fast gate; principles + catalog in `spec/idioms/`; one blind run recorded in `research/skill-eval/` with per-problem pass/workaround/lint; every finding filed in the owning domain's worklog.

**M2. Corpus measured on v0.3, every judge.** Exit: the token-ratio command works again; `cpl-conform corpus` 100/100 check, committed scoreboard matches a fresh run; `judges.rs` proves the 305 tracked ports (as they pass) on every builtin judge; the 23 port lint warnings triaged into true/false with FPs sent to lint-fmt.

**M3. Idiomatic re-ports.** Exit: 305/305 re-ported on v0.3, each reviewed (median >= 4, 0 under 3), token median <= 0.6x, none > 1.0x; blind eval on an unseen 20 at >= 18/20.

## Metrics (run 2026-11-22; commands in metrics.toml)
| Metric | Command | Today | Target |
|---|---|---|---|
| Dry run v0.3 (written with compiler) | `research/dryrun/v03/loop.sh <p> 20` x20 | 20/20 (15 `_w` workaround variants exist) | 20/20 at 200 cases, 0 `_w` |
| Blind skill eval | cold agent + `loop.sh` | 20/20, 12/20 first attempt, 0 workarounds, 3 lint kept (known 20, round 3) | 20/20, 0 workarounds, 0 lint |
| Skill check | `scripts/skill-gen.py --check` | absent on main; branch ok, 48 files, 75 blocks | on main, gated |
| Tracked ports check on v0.3 | `cpl check corpus/problems/<id>/port.cp` x305 | 175/305 | 305/305 |
| Dreams check on v0.3 | same, `dream.cp` | 145/305 | tracked |
| Tracked ports judge-proven | `cargo test --release -p cpl-cli --test judges -- --include-ignored every_` | 223/305 (`corpus/sets/v03.txt`), all 8 judges, 0 fail | 305/305 |
| Corpus suite | `cpl-conform corpus` | 12/100 (committed board says 7) | 100/100 |
| Lint warnings on ports | `cpl lint` over the 175 | 23 (untriaged) | 0 false positives |
| Taste reviews | `scripts/review-check.py` | median 4, 18 under 4 (pre-0.3 ports) | median >= 4, 0 under 3 on v0.3 |
| Token ratio vs C++ | `scripts/ports-page.py --tokens` | port 0.52x median, max 1.10x (1 over); dream 0.49x, max 0.97x (n=294) | median <= 0.6x, max <= 1.0x |
| Writable from docs | `research/writable-from-docs.json` latest | 49/55 (read, not re-run) | 55/55 |

## Judgment rubric (what numbers miss)
1. **Says the idea.** Would a reader who knows the editorial find each of its steps named in the code, in its words?
2. **Every fact once.** Does each constraint, index space and absence appear once, as a type or name, not as `+1`, `-1` sentinels or repeated bounds?
3. **No hacks.** Did the writer ever route around the language (workaround, `_w` file, raw escape) where a general mechanism should exist?
4. **Revises locally.** When one requirement changes, does the edit stay in one chunk?
5. **Trust.** Would the contestant submit it without re-reading the C++?

**Probes** (a cold agent with only the skill):
- Solve the 20 dry-run problems; score each on 1-3 and count workarounds. (1, 2, 3)
- Port 10 random tracked ports from `orig.cpp` without seeing `port.cp`; a reviewer scores against `research/design-history/EXEMPLARS-PORTS.md`. (1, 2, 5)
- Revision test: solve an unseen problem, then change one requirement (end position, extra DP dimension, map instead of array); count touched chunks. (4)
- Debug probe: given a port with a planted off-by-one, find it with stress/trace using only skill guidance. (3, 5)

## Improve (highest impact first)
1. Land the skill (`devin/skill-v2`) and its eval; gate `skill-gen.py --check`.
2. Run the blind eval and route each finding to its owner (5 repros already on `devin/skill-eval-1`).
3. Land principles + idiom catalog (`devin/idioms-docs` docs part) so lint-fmt has the catalog.
4. Extend `judges.rs` to the tracked ports and fix the token-ratio script; refresh the scoreboard.
5. Re-port the 305 on v0.3 in batches of ~20, worst review score first.
6. Grow the hill-climb past 100 in thin contest areas; each gap a spec proposal or conformance case.

## Ryan's role
**Skill.** Co-designer: the principles and the idiom catalog are Ryan's taste written down; evals check them.

**Corpus.** Reviews a sample of re-ports and dream programs for idiom (they define "idiomatic"); picks problem areas. Judge proof needs no Ryan time.

## Decisions for Ryan (each filed in plan/QUESTIONS.md)
1. **Principles are normative taste?** Put principles + catalog in `spec/idioms/`, changes via QUESTIONS. Default: land them there, marked non-normative.
2. **Re-port bar.** Rethink each port from the dream (slow, idiomatic) vs migrate `port.cp` mechanically (fast, judge proof). Default: migrate first for judge proof, rethink in review-score order.
3. **Headline skill metric.** The 20 dry-run problems are known to us; recommend a fresh unseen 20 per release as the gate. Default: report both, gate on the known 20 until a fresh set exists.
