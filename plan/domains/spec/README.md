# spec

Segment: D (taste and co-design). Owns: `spec/core`, `spec/lib`, `spec/tools`, `spec/conformance`.
Status (2026-11-22): 527 numbered rules, all cited by a case; 1711 cases, 1617 pass on today's compiler; matrix and trait generators gated in `scripts/check.sh`.
Detail: [PLAN.md](PLAN.md) · code: [ARCHITECTURE.md](ARCHITECTURE.md) · numbers: [metrics.toml](metrics.toml) · log: [worklog.md](worklog.md)

## Mission

Say exactly what CP Lang is, in one place, so that a contestant, a model or a second implementer gets the same answer to every "does this compile, what does it print, what traps?" question; and hold the compiler to it with conformance cases that catch every bug before a contestant does.

## Covers / not covered

| Covers (spec) | Not covered (owner instead) |
|---|---|
| `spec/core/` chapters 00-13 and their `(Rnn.n.n)` rule ids | `std/*.cp` signatures and bodies (stdlib) |
| `spec/lib/` library design chapters | `spec/diagnostics.md`, the error catalogue (diagnostics) |
| `spec/tools/` tool contracts | Making failing cases pass (compiler, perf, tools) |
| `spec/conformance/`: cases, `COVERAGE.md`, the matrix and trait generators' data; `crates/cpl-conform` (the runner) and `scripts/conformance-*.py` | Other `scripts/` and the gate itself (infra) |
| Settling spec ambiguities in `decisions.md` | Teaching docs on the site (site), idioms and the skill (idioms) |

Spec writes cases, never compiler code: a failing case goes to the owning domain's worklog as a list of paths.

## Milestone 1

`plan/product/MILESTONE-1.md`, due about 09:30 PST 2026-11-23 on edge: a Codeforces 1500+ friend learns CPL, solves unfamiliar problems, tests and competes unaided, on Linux, macOS or Windows, from Neovim/Vim or VS Code. Ryan: language quality over judge-specific features; the site's in-depth chapters (ownership, mutation, design decisions) draw on spec's rationale. From spec: "a coherent, documented, sufficiently complete subset ... gaps clearly identified rather than disguised", and "documentation examples must actually work".

Exit criteria (each checkable):
1. **Subset frozen.** Done: `spec/SUBSET.md` tags every core and tool rule section; each failing case in a *supported* section is a known gap with its owner; `conformance-subset.py --check` passes (119 supported, 21 documented, 133 known-gap cases; `core-09/deep-recursion-trap.cp` times out only under a loaded suite, owed by compiler as an interpreter recursion limit). Keep it passing, and shrink the list.
2. **Examples run.** Every code block in a *supported* chapter passes `cpl-conform spec-blocks` or is `spec-only` with a reason (today 87/185 over all chapters; every failing block in a supported core chapter has a case naming its owner).
3. **Writable from the docs.** A cold rebuild on Codeforces Div. 2 A-D problems, reading only `spec/` and `std/`, records 0 contradictions and < 5 holes inside the subset (round 12, Codeforces: 2 contradictions, 16 holes, all in the subset; 5 are stdlib stub holes repeated since round 9).
4. **Range keys folded.** Decision 289 is in every chapter and its 7 cases pass (today 4/7; the 3 failures are compiler-owned and listed in SUBSET.md).
5. **Journey contracts pinned.** `check`, `run`, `test`, `stress`, `build` and the four Neovim LSP features (diagnostics, completion, formatting, definition) carry rule ids with cases or named owner tests (cli, fmt, lint, bundle, stress, lsp done; the judge-profile ids landed but are not a milestone criterion, per Ryan).
6. **Portable.** CRLF source and input, and paths, are specified the same on all three platforms, with cases.
7. **Rationale readable.** `spec/RATIONALE.md` gives every subset area its overview chapter, decision ids and the alternatives that lost; every supported chapter opens with a model-in-one-paragraph overview the site's in-depth chapters build on.

Deferred (in parallel, after blockers): trait/matrix cells outside the subset; trace, DAP and explain rule ids; bug-catch rounds; `cpl migrate` (R11.4.3); advanced lib chapters beyond *documented*; prelude gaps for non-subset names. New language features only where a journey step is blocked.

Next, in hours (checkpoint in worklog.md every 2 hours):
1. 21:30-23:30: RATIONALE.md and the missing chapter overviews; round-6 contradictions settled; route failures.
2. 23:30-03:30: round-6 holes inside the subset get a rule, a case or a SUBSET entry; spec-blocks in supported chapters.
3. 03:30-09:30: rebuild round 7 on Codeforces Div. 2 problems; fix every contradiction; shrink known gaps with owners.

## Milestone 2

`plan/product/MILESTONE-2.md`, the stable release. Spec's complete version: the whole language is normative, every rule is tested independently of the compiler, and a stranger can learn it, trust it and read why it is shaped this way, from `spec/` alone. Spec work on it starts once the Milestone 1 criteria above pass.

Exit criteria (each checkable):
1. **Whole language supported.** `spec/SUBSET.md` has 0 *documented* and 0 *gap* sections, and covers `lib/` as well as core and tools (today 95 supported, 3 documented, 0 gap, 91 known-gap cases: `scripts/conformance-subset.py --check`).
2. **No known correctness bugs.** 0 known-gap cases in supported sections; every rule id, trait method, matrix cell and std signature has a passing case (`conformance-coverage/traits/matrix/sigs.py --check` with 0 gaps).
3. **Every example runs.** `cpl-conform spec-blocks` passes every block; `spec-only` is kept for grammar fragments alone (today 85/185).
4. **Both engines agree.** Every conformance case gives the same output and diagnostics under `cpl run` and the built C++ (`cpl-conform diff`).
5. **Bugs get caught.** At least 95% of reverted compiler fixes are caught by the suite (`scripts/conformance-bugcatch.py`; 92% in round 1).
6. **Writable from the docs.** A cold rebuild on unseen problems for every journey (J1-J10 problem types) records 0 contradictions and 0 holes.
7. **Readable in depth.** Every chapter opens with its model; every decision cited in `spec/` gives its reason inline and its rejected alternatives in `spec/RATIONALE.md`; the site renders them without links outside the site.
8. **Stable to depend on.** The spec carries a version and a changelog of normative changes; the compatibility promise, pins and `cpl migrate` codemods (R11.4.3) have rule ids and cases, including rebuilding old solutions byte for byte.
9. **Every tool is specified.** Trace, DAP, explain, LSP, stress generators and judge profiles carry rule ids with cases. `13-lowering.md` states the performance bar perf measures: idiomatic p50 <= 1.1x, p95 <= 1.2x and max <= 1.5x of matched C++, and tuned max <= 1.1x, with every solution under 32 KB.

Next steps, after Milestone 1: (1) rule ids and cases for trace, DAP and explain; (2) diff mode over every case; (3) version the spec, with a normative changelog and `cpl migrate` cases.

## Success criteria for v0.3

1. Every rule that forbids, traps or limits has a reject case; every rule has an accept case unless it only forbids. (2 forbidding rules lack a reject case today; 3 rule gaps in all; 26 rules that only permit or define are listed with a reason in `spec/conformance/permits.txt`.)
2. Every case cites a rule, a std item or a diagnostic code that exists. (0 bad cites, 0 rule-less core cases today.)
3. The feature x consumer matrix and the trait x claimer x method table have no empty cell (case or written n/a reason).
4. Every spec code block is a case or is marked spec-only. (19 of 185 tagged blocks pass, most fail as fragments naming undefined values; 27 untagged blocks are skipped.)
5. A cold agent rebuilding from `spec/` alone finds fewer than 5 holes. (Round 2 found 32.)
6. All of the above are ratchets in `scripts/check.sh --fast`.

## Milestones

- **M1 Every rule tested both ways.** Exit: `conformance-coverage.py` reports 0 happy-path-only rules, 0 unknown cites, 0 rule-less core cases, 0 diagnostic codes without a reject case; `conformance-coverage.py --check` runs in the fast gate.
- **M2 Generated coverage.** Exit: `devin/wip-conformance-matrix` landed and `conformance-matrix.py --check` gates 0 empty cells; `conformance-traits.py --check` gates the trait table, `trait-gaps.txt` only shrinks; spec-blocks: every fence is a case or `spec-only`.
- **M3 Independently rebuildable.** Exit: a rebuild round (cold agent, `spec/` only, 40 questions + 10 programs) records < 5 holes; the bug-catch probe catches >= 90% of reverted compiler fixes.

## Metrics

All values measured 2026-11-22 on `4b8f504f` with a release build; commands in [metrics.toml](metrics.toml).

| Metric | Command (short) | Today | Target | M |
|---|---|---|---|---|
| Conformance cases passing (compiler's number, spec's suite) | `cpl-conform conformance` | 2130/2297 | 100% | - |
| Cases (only grows) | same | 2286 | ratchet up | - |
| Rules cited by >= 1 case | `cpl-conform --rules` | 528/528 | 100% | M1 |
| Forbidding rules with no reject case | `conformance-coverage.py` | 3 | 0 | M1 |
| Case cites to unknown rules | `cpl-conform --rules` | 0 | 0 | M1 |
| Core cases with no `// rule:` | `conformance-coverage.py` | 10 | 0 | M1 |
| Diagnostic codes with no reject case | `conformance-coverage.py` | 1/136 | 0 | M1 |
| Std items named by a case | `conformance-coverage.py` | 2037/2245 | 2245 | M2 |
| Spec code blocks passing | `cpl-conform spec-blocks` | 84/185 (27 error, 74 skip) | all or spec-only | M2 |
| Untagged code blocks (skipped) | `awk` in metrics.toml | 27 | 0 | M2 |
| Matrix empty cells | `conformance-matrix.py --check` | 0 (gated) | 0 | M2 |
| Trait methods without a case | `conformance-traits.py --check` | 3 of 118 (held range triples) | 0 | M2 |
| Named args in core/lib that std lacks | `conformance-sigs.py --check` | 5 (`sig-gaps.txt`) | 0 | M3 |
| Chapter-table fns missing from the prelude (R11.3.2) | `conformance-prelude.py --check` | 106 (`prelude-gaps.txt`) | 0 | M3 |
| Rebuild holes (cold agent) | probe P1 | 26 (round 6, round 5's problems: contradictions 18 to 3; 31 in round 5, 20 in round 4, 32 in round 2) | < 5 | M3 |
| Reverted compiler fixes caught | `conformance-bugcatch.py`, [bugcatch.md](bugcatch.md) | 23/25 measurable fixes (92%), 3 of 28 fix commits do not build| >= 90% | M3 |
| Banned spellings in spec | `spec-vocab.py` | 0 | 0 | - |
| Suite wall time (2 cores) | `time cpl-conform conformance spec-blocks` | 8.7 s | < 20 s | - |

## Judgment rubric

What numbers miss. A cold agent scores fixed samples 1-5 with a reason; the trend is the signal.

1. **Rebuildable:** could an implementer build this rule from the chapter alone, with no guess?
2. **Three answers:** does each checked operation say what it means, what the compiler proves, and what each build checks?
3. **One spelling:** do the chapter, the `std` signature, the error text and the cases say it the same way?
4. **Sharp case:** is each case minimal, about one rule, and would it fail on a plausible compiler bug (not just on today's)?
5. **Worth the rule:** does each rule earn the thing a contestant must remember (AGENTS.md coherence)?

Probes (a cold agent attempts, pass/fail + time + files touched):

- **P1 Rebuild:** only `spec/`: answer 40 snippet questions and write 10 solutions; every guess is a hole (rubric 1, 3).
- **P2 Bug-catch:** revert N past compiler fixes in `crates/` one at a time; does `cpl-conform conformance` go red (rubric 4)?
- **P3 Add a rule:** add a rule plus accept/reject cases from `spec/conformance/README.md` alone; gate green first try (rubric 2).

## Improve (highest impact first)

1. Land `devin/wip-conformance-matrix` (6 unlanded commits: matrix generator with 211 generated cases, 108 hand cases, rulings for decisions 211-215).
2. Write reject cases for the 3 happy-path-only rules; then gate `conformance-coverage.py --check`.
3. Trait generator: every method on every claimer (17 std traits, 72 `impl` claims, plus core claimers).
4. Spec-block coverage: a `given` prelude so fragments run (166 of 185 fail today), tag the 27 untagged blocks, mark the rest spec-only.
5. Spec vs std contradiction check (e.g. ch. 7 says descending is a key, `sort` has `desc:`); feed rebuild round 2's 32 holes.
6. Rule ids for `spec/tools/` contracts and `spec/lib/` chapters (0 today), so tool and library behaviour is citable.
7. Rebuild round 3 as a repeatable script (bug-catch is `scripts/conformance-bugcatch.py`).

## Ryan's role

Co-designer. Settles calls that change the language's character, from `plan/QUESTIONS.md`; reviews the rubric and one sample per digest (a rule + its cases). Writing cases needs no Ryan time.

## Decisions for Ryan

None open. Range keys (Ryan, decision 289) and the lane's three questions (decided by their recommendations, decisions 290-292) are settled; a new call goes to `plan/QUESTIONS.md` only if it is important, uncertain and hard to reverse.
