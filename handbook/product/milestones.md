# Milestones

| | |
|---|---|
| Kind | Product |
| Owner | Ryan, kept by the coordinator |
| Changes | Fix errors directly; anything that changes direction is an RFC marked `Ryan to review` |

A milestone is a recognizable, testable state of the product, defined from
the user's side: what a person can do, end to end, that they couldn't
before. It is not a list of components. Each one has an acceptance scenario
and the bar each area must meet.

How lanes use milestones:

- Every lane's priorities trace to the current milestone. Work that blocks a
  step of its acceptance scenario comes first. Work listed under "what can
  wait" happens when it is cheap or a hard gate needs it.
- A domain moves on to the next milestone as soon as its own criteria for
  the current one pass, without waiting for the other domains.
- Domain metrics are means. The milestone's acceptance run is the end.

The bars live here. How far along each milestone is, and what blocks it, is
time-bound and lives in `plan/STATUS.md` and `plan/product/BLOCKERS.md`.

## Milestone 1 — Friend-testable, contest-capable CPL

**I can share CPL with an interested competitive programmer, and they can immediately start learning it, solving real problems, and competing with it.**

This is more than a proof of concept, a toy interpreter or a demo website. The friend uses CPL on their own for real competitive programming, including a real or realistically timed contest. The whole vision need not be done, but the supported end-to-end experience must be complete, polished enough to judge fairly, and trustworthy.

### Constraints (Ryan, 2026-11-22)

- **Due:** about 12 hours, by 2026-11-23 09:30 PST. Plan in hours; checkpoints every 2 hours.
- **Platforms:** Linux, macOS and Windows, all supported and all tested.
- **Editors:** Neovim/Vim (the first friend's), VS Code and JetBrains, each with a working LSP.
- **Judge:** Codeforces-style problems. The output is one portable C++17 file that standard g++ compiles. Named judge profiles are kept but are not a milestone focus; agents never submit to real judges. Language quality matters more than judge coverage.
- **Channel:** the edge release is enough. No stable release needed.
- Deferred work continues in parallel, but milestone blockers come first.

### The acceptance scenario

A Codeforces 1500+ friend who has never used CPL gets a link. Without help, they can:

1. **Understand:** open the website, see the value proposition, and read a concise introductory guide.
2. **Try:** run and modify CPL programs in the browser, completing a few introductory exercises.
3. **Install:** set up CPL and their editor integration on Linux, macOS or Windows.
4. **Solve:** implement several unfamiliar, real problems with the language and standard library.
5. **Test:** run samples, add custom tests, and diagnose ordinary mistakes from the CLI and the editor.
6. **Submit:** generate a Codeforces submission that gets accepted.
7. **Compete:** take part in a real or virtual Codeforces round with CPL as their main language, never blocked by missing ordinary functionality or a tool failure.

Afterwards they can give meaningful feedback on expressiveness, correctness mechanisms, tooling and usability, and they have felt at least one compelling advantage over their current workflow.

**Acceptance run:** an agent plays the friend on fresh Linux, macOS and Windows machines, using only the website and docs, every 2 hours. It records the steps passed (out of 7), the time per step, and every hesitation or workaround, and publishes a ranked blocker list. The idioms domain owns it.

### What must be ready

- **Language and compiler:** a coherent, documented subset that handles representative real problems, including nontrivial algorithms and data structures. It is correct and identical across interpreter and C++; gaps are listed, never disguised.
- **Standard library:** the foundational containers, algorithms, math and helpers. It need not be exhaustive, but ordinary solutions never require extensive workarounds.
- **CLI and local testing:** reliable check, run, sample tests, custom tests, checkers and judge-ready builds; sample import through Competitive Companion.
- **Editors:** Neovim/Vim, VS Code and JetBrains with highlighting, diagnostics, completion and formatting, so the user never switches between disconnected tools.
- **Website and learning:** a credible site, a concise intro guide, searchable docs, runnable examples that actually work, and enough practice to teach the core language model.
- **In-depth language chapters:** one per major part (ownership and mutation, domains and ranges, types and proofs, multitest, checked operations, the judge model), precise enough to predict behaviour, each explaining the design decisions behind it. The first friends are PL enthusiasts who will want to understand it fully.
- **Submission:** a reliable self-contained C++17 file that standard g++ compiles. Single-test and multitest problems work; other formats work where feasible and are clearly scoped where not.
- **Debugging:** high-quality diagnostics and runtime errors, print debugging, assertions, and fast reproduction of failing cases. A basic integrated brute/stress workflow is the differentiator to show. Full time-travel debugging is not required.
- **Release quality:** reproducible builds, installers tested on all three platforms, automated regression checks, and no known serious correctness or submission bugs in supported workflows.

### What can wait

These don't block the milestone unless testing shows they are essential:

- Comprehensive advanced standard-library coverage.
- Automatic generation and shrinking for arbitrary input structures.
- Advanced static performance analysis and inline resource annotations.
- Direct submission to judges.
- A third-party package ecosystem.
- Long-range backward compatibility across language versions.
- Exhaustive beginner tutorials.
- Compiler optimization beyond competitive parity.

### Milestone 1.5

Debugger replay: time-travel through a run and view the trace, at most +20% over the interpreter. Work on it in parallel as soon as the milestone's stress loop works everywhere.

Scope is set by whether friends can realistically use CPL, not by whether every planned component is finished.

## Milestone 2 — Stable release

**CPL is ready to release to the world as stable, for anyone to try.** This is the maximally ambitious version: everything in [the vision](vision.md) and [the journeys](journeys.md), everything milestone 1 deferred, and whatever else each domain judges the complete product needs.

A domain moves onto milestone 2 as soon as its milestone 1 exit criteria pass. Its plan (`plan/domains/<domain>/PLAN.md`) has a "Milestone 2" section with that domain's most ambitious complete version, its checkable exit criteria and its next steps.

### Design first

These surfaces are designed, with an owner and a gate, before milestone 2
builds on them, so the first tool that needs one doesn't decide it by
accident. Each is a row in [architecture invariants](../design/architecture-invariants.md).

- the `cpl_check::Session` query API, per revision, before any consumer adds a query;
- the `cpl-json` schema, before a new `--format json` output, LSP message or trace field;
- the diagnostic registry, before a new diagnostic;
- the config schema, before a new `cp.toml` key, judge profile or editor setting;
- the std doc model, before hover, completion or the site read std docs;
- the edit engine, before a new codemod or `cpl fix` rewrite;
- the execution-event model, before the profiler, debugger or replay;
- the problem package and `scripts/corpus.py` walker, before a new corpus consumer.

After milestone 2, and designed only when triggered: incremental analysis
(the first per-keystroke editor feature) and package versions and editions
(the first published library or edition-breaking change).

### Done means

- **Every journey, J1–J10, is complete and polished** for strangers, not just friends: discovery, learning (beginner through in-depth), install, contests, writing and revising, testing, debugging, performance, submission, and managing libraries and old solutions.
- **Every domain hits its north-star targets** in `plan/domains/*/metrics.toml`. Examples:
  - interpreter within 5x of C++, trace overhead at most +20%;
  - runtime p50 under 1.1x, p95 under 1.2x and max under 1.5x of matched C++ for idiomatic code, and max under 1.1x once the user tunes in the language (widths, `unchecked:`, layout);
  - every solution under 32 KB;
  - std within 1.2x of optimized C++ on Library Checker;
  - the full lint catalog;
  - spec code blocks and doc examples all passing.
- **Everything deferred from milestone 1:**
  - time-travel debugging and trace viewing (milestone 1.5);
  - generators and shrinking for arbitrary input structures;
  - static performance analysis;
  - comprehensive standard library;
  - native editor UIs (VS Code, JetBrains) beyond the LSP;
  - personal libraries with stdlib powers;
  - old solutions that rebuild byte for byte, with pins and codemods.
- **Stable release:** two channels, `stable` and `edge`. `stable` is promoted by hand with release notes (v0.3.0 first); while the version is 0.x the site and README label it beta. Signed installers and package managers on Linux, macOS and Windows, a changelog, and a documented compatibility promise.
- **No target date yet.**
- **Public quality:**
  - no known correctness bugs in supported features;
  - interpreter and C++ agree on the whole corpus;
  - CI under 3 minutes and green;
  - a site and docs that stand on their own.
