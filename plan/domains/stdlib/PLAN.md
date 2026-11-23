# stdlib plan

The detail behind [README.md](README.md). Numbers are from the runs in [metrics.toml](metrics.toml) (2026-11-22, main `4b8f504f`).

## Where the debt is

- **Bodies.** 294 `todo` stubs: 189 in library bodies, 105 in `@test` fns (`measure.py todo-split`). 146 doc examples reach a `todo` (`cpl-conform docs --causes`, R0402).
- **Examples that don't compile.** Of 687 non-passing doc examples, two causes cover 263 (`--causes`): 174 E0204 "takes a domain by name, and `0..5` is a range" (an int range where a type argument names a domain), 89 R0001 "an array literal needs one value per position" (a list literal for `[D]T`, which spec R04.8.10 allows).
- **Whole modules.** `flow` 0/72 and `dsu` 0/64 examples pass; `tree` 123/239, `graph` 87/141 (`cpl-conform docs <m>.cp`). Only 12 of 40 files pass standalone `cpl check` (the `SPEC_STD` ratchet in `crates/cpl-cli/tests/corpus.rs` lists the same 12).
- **Compiler gaps.** 88 entries in `std/GAPS.md`: bodies the compiler can't run yet.
- **Surface.** Prelude 192 names (`check.sh` still warns above 80, a budget decision 150 dropped; usage in `prelude-usage.md`); 32 public fns lack a doc label; 13 `spec-stub --check` problems (e.g. `ternary_max` is in no chapter).
- **Privilege.** The builtin count, agreed with tech-debt, is `ir_builtin_names` (`scripts/debt-ratchet.py`): 89 distinct names string-matched in match arms of `cpl-ir/src/lower/call.rs`; 40 of them are public std names (`measure.py privilege`).

## Workstreams, in order

1. **W1 Land in-flight work** (M1). In the order the coordinator gave, each rebased, `check.sh --fast`, `land.sh`; `git cherry` first to drop superseded commits.

   | Branch | Ahead | `todo` on branch (main 294) | Note |
   |---|---|---|---|
   | `devin/wip-std-import-cycles` | 3 | 294 | `std_acyclic` test, cycle chains in E-codes; touches `cpl-check` (compiler): note in compiler worklog |
   | `devin/wip-std-todos` | 11 | 264 | geometry, strings, heap, mod, graph bodies |
   | `devin/wip-stdlib-bodies` | 4 | 212 | poly, math, matrix, algebra, lazy, seq, prefix |
   | `devin/wip-std-graph-batch-b` | 3 | 286 | also edits `cpl-ir/src/lower/{expr,range}.rs` and `cpl-interp/src/tests.rs`: compiler-owned; split, and hand the IR part to the compiler lane |

2. **W2 Classify every failing example and case** (M2). `cpl-conform docs --causes` and `conformance std-` failures, each labelled: *std bug* (fix here), *compiler gap* (GAPS.md line + request in `plan/domains/compiler/worklog.md` + a contract test in `spec/conformance/std-*`), or *open question* (QUESTIONS.md). Ranked by examples unblocked. This is what turns 687 into a worklist instead of a number.
3. **W3 Write bodies** (M2). `@test` bodies first (105, cheap, they test the rest), then library bodies by module, largest unblocked count first. Each body: plain CP Lang, `cost:` checked against it, example runs.
4. **W4 Doc labels and `spec-stub --check`** (M2). Mechanical: 32 labels, 13 problems. One commit per module.
5. **W5 Prelude budget** (M3, decision 281). Count use of each prelude name across gold ports (`corpus/`) and hill-climb solutions; cut by the rule; `cpl fix` migrates `use` lines (lint L0208 `redundant_prelude_import` in `cpl-lint` already knows the list).
6. **W6 No privilege** (M3, with tech-debt and compiler). One definition of "privileged" (decision 266); the metric in `measure.py privilege` sharpened to it; a no-privilege test: copy a std module into a temp user library and run its conformance cases there; then the per-name moves.
7. **W7 `std` -> `std/`** (M3, run by tech-debt). The stdlib side: path updates in `std/README.md`, `GAPS.md`, conformance case headers.

W2 runs before W3 because writing bodies whose examples fail for a compiler reason wastes the body's test.

## Why each milestone

- **M1** banks work already done (about 120 stubs) and puts the cycle guard in the gate before more bodies add `use` edges. Cheapest big move.
- **M2** is "the library is real": every name that exists runs, documents its cost and is tested in both backends. Everything a cold agent can be probed on depends on it.
- **M3** is "the library is coherent": the small prelude, no privilege, one home. These needed Q1-Q3 (now decisions 281, 255, 266) and other lanes, so they go last; defaults keep M2 unblocked.

## Alternatives considered

- **Cut unimplementable names from v0.3 instead of writing them.** Faster to zero `todo`, but each name was chosen on real ports; cutting loses designed surface. Kept as the fallback for GAPS entries the compiler lane can't close by the v0.3 date, decided per name and logged.
- **Rewrite the 263 examples to today's compiler** (a declared `domain D = 0..3` and `Dsu<D>()`). Turns the metric green in a day, but the examples show what contestants should write, and Q2/R04.8.10 say they are right. Rejected (decision 255).
- **Keep fast builtins and make std bodies documentation only.** Zero perf risk, but a body that never runs is untested and a user library can't match it: that is privilege. Rejected; decision 266 sets the floor.
- **One monolithic "stdlib done" milestone.** Rejected: M1 is hours, M2 weeks, M3 depends on Ryan; one milestone would hide all three.

## Risks

| Risk | Sign | Mitigation |
|---|---|---|
| Bodies blocked on the compiler pile up | GAPS.md count rises | W2 requests ranked by examples unblocked; contract test per gap |
| Branches conflict with each other and with main | rebase conflicts in `graph.cp`, `seq.cp` | land in the given order, one at a time; re-measure after each |
| Moving a builtin to a std body slows C++ | `cpl bench` regressions | move only behind perf's gate; builtins stay until the body matches |
| Prelude cut breaks old programs | corpus/check failures | `cpl fix` adds the `use`; ratchet on corpus green |
| Std files grow past review size (`geometry` 3599, `seq` 3373, `strings` 3347 lines) | file size | split by topic with a `pub use` re-export when a module passes 3k lines and has two topics |
| Bodies "work" but are slow or opaque | rubric 2/4 scores | cold-agent rubric run per milestone |

## Phase 2, first

1. `git cherry` each branch; rebase `devin/wip-std-import-cycles`; `check.sh --fast`; land; note the `cpl-check` change in the compiler worklog.
2. Same for `wip-std-todos`, then `wip-stdlib-bodies`; re-run `debt-ratchet.py --update` with each.
3. Split `wip-std-graph-batch-b`: land `std/graph.cp` + `std-graph` cases; hand the `cpl-ir` diff to the compiler lane as a request.
4. Re-measure every metric in `metrics.toml`, then start W2 on `flow` and `dsu` (0 examples pass).
