# idioms

This folder holds the idioms domain's time-bound work: the milestone plans
below, [PLAN.md](PLAN.md), the targets and current values in
[metrics.toml](metrics.toml), and the [worklog](worklog.md). What the domain
is, owns and guarantees (mission, scope, interfaces, invariants, the quality
register) is the charter, [handbook/domains/idioms.md](../../../handbook/domains/idioms.md).

## Milestone 1 (priority over everything below; due about 09:30 PST 2026-11-23)
[Milestone 1](../../../handbook/product/milestones.md): a Codeforces 1500+ friend who has never used CPL understands it, tries it in the browser, installs it, solves unfamiliar problems, tests them, builds them for Codeforces, and competes, all without help. Idioms owns the **acceptance run** that says whether this works, grown out of the blind skill evals. The run ships on edge, needs no stable v0.3.0, and makes no real judge submissions.

**What the milestone needs from idioms (exit criteria)**
- **Protocol and harness** (done, `c325c1b05`). [research/acceptance/PROTOCOL.md](../../../research/acceptance/PROTOCOL.md) defines the run: a cold agent plays the friend using only https://cp-lang.org, the pages it links and Codeforces statements, with Neovim as the editor. Each run writes `research/acceptance/runs/<date>-<n>.json` with pass or fail per step, minutes per step, the OS, and every blocker, workaround and hesitation, each tagged with its lane.
- **Three OSes.** Every run covers Linux, macOS and Windows. The agent drives Linux; CI runners on each OS do the `install` step and the CLI half of `test` and `submit`, following the site's instructions word for word.
- **Unseen problems** (picked, `6165c9599`). Ten Div2 problems (Codeforces 2285A-E and 2289A-E) plus a timed virtual round (2272A-E, 2 h), all newer than the corpus. Each needs a local judge: reference C++, `gen.py` and a checker where answers aren't unique.
- **Language quality is the score** (Ryan, 2026-11-22). Findings are blockers, workarounds, non-idiomatic code the friend was pushed into, and hesitations. The run reports them per accepted problem and BLOCKERS.md ranks by them. On the build side the run only checks that the generated C++ compiles and passes the local judge (Competitive Companion if documented); it does not test judge-specific profiles.
- **Blocker list** (generator done). [plan/product/BLOCKERS.md](../../product/BLOCKERS.md) is generated from the runs by `research/acceptance/blockers.py` (registered in `gen.toml`), ranked by kind, then frequency, then minutes lost. It is refreshed every 2 hours, and each lane gets its rows in its worklog.
- **Advanced reader** (Ryan, 2026-11-22). Step 8, `design`: a PL-curious reader reads the ownership and mutation chapter plus one other, predicts 5 probe snippets, and checks that the design reasons are given. Each gap is a blocker for site or spec.
- **Editors.** Neovim is driven through every step, with Vim checked too. VS Code and JetBrains get install, setup and the five LSP features.
- **Done when:** the latest run passes 8/8 steps on all three OSes; Solve gets local-judge accepts on at least 8 of 10; Compete solves at least 4 of 5 in 2 h with nothing lost to missing ordinary functionality or a tool failure; BLOCKERS.md has no open blocker row.

**Deferred** (cheap or gate-only until the milestone passes; continues in parallel): idiomatic re-ports from `dream.cp` and the token and taste targets (M3); corpus growth past 100 and `cpl-conform corpus` 100/100; port lint-warning triage, except lints the friend meets; the unseen-20 skill gate, which the acceptance set replaces for now. Mechanical port migration continues only when a stdlib or compiler fix unblocks a port.

**Top 3 next steps**
1. Local judges for the 15 held-out problems, so run 1 (started 21:29, Linux) can be scored.
2. Score run 1, publish BLOCKERS.md, send each lane its rows, then rerun every 2 hours, with a worklog checkpoint each time.
3. A CI job on Linux, macOS and Windows that follows the site's install and CLI steps and writes its results into the run.

## Milestone 2 (after the Milestone 1 exit criteria pass)
[Milestone 2](../../../handbook/product/milestones.md): stable, for strangers. The most ambitious version of idioms: **every CPL program a model or a person writes from the docs is correct, idiomatic and fast on the first try, and we can prove it on every release.**
- **Acceptance for strangers.** The run covers J1-J10 on all three OSes for each release candidate. It uses several personas: beginner, Codeforces 1500+, an elite contestant, and a PL-curious reader. Each run gets a fresh unseen set: 10 Div2, 5 Div1, plus a timed round. Stable is promoted only when every step passes on every OS and BLOCKERS.md has no P0.
- **Idiomatic corpus.** All 305 ports are re-ported from `dream.cp`. Taste-review median is at least 4, with none under 3. Contestant tokens: median at most 0.6x C++, none above 1.0x. Every port is judge-covered and checks byte for byte across releases, with pins and codemods. The corpus reaches 100/100 on `cpl-conform corpus`.
- **Performance, on the corpus.** Idiomatic code runs at p50 <= 1.1x, p95 <= 1.2x and max <= 1.5x of matched C++. Tuned code runs at max <= 1.1x. Every solution is under 32 KB.
- **Full lint catalog.** Every catalog row is implemented or explicitly rejected. The newcomer set reaches >= 95% flagged, with 0 broken fixes and every lint linked to its `/lints/<name>` page. Every port is lint-clean, or carries a reason.
- **Skill and docs.** The bundled skill and the site pass a fresh unseen set of 20 problems blind each release. Writable-from-docs is 55/55.

**Exit criteria:** the latest release-candidate run passes J1-J10 on Linux, macOS and Windows for every persona; the corpus, perf, lint and skill numbers above are met and measured by `metrics.toml` commands; BLOCKERS.md has 0 open P0 or blocker rows.

**Next steps:** (1) the first fresh held-out set, `research/acceptance/problems/sets/m2-1.txt` (10 Div2, 5 Div1, a 5-problem round), with local judges and `ref.cpp` references: all 20 have samples, a generator, a brute (or checker) and a max input, and every `ref.cpp` is AC under `judge.py`; (2) the four personas and the M2 gate are in PROTOCOL.md's "Milestone 2 runs" section, to drive from run 7 on; (3) idiomatic re-ports, worst-reviewed first (`scripts/review-check.py`).
