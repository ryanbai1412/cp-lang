# stdlib

Segment: D. Taste and co-design. Owns: `spec/std/` (moving to `std/` in the restructure), `plan/domains/stdlib/`. Status 2026-11-22: 1,976 public items in 39 modules; 294 `todo` stubs; 2012/2699 doc examples and 801/832 std conformance cases pass.

Detail: [PLAN.md](PLAN.md) (workstreams, alternatives, risks) · [ARCHITECTURE.md](ARCHITECTURE.md) (the code today) · [metrics.toml](metrics.toml) · [worklog.md](worklog.md) · normative surface and doc rules: [spec/std/README.md](../../../spec/std/README.md) · compiler gaps: [spec/std/GAPS.md](../../../spec/std/GAPS.md).

## Mission

Every contest facility a contestant reaches for is one guessable call, written in plain CP Lang that a user library could have written, with its cost, traps and an example that runs as a test.

## Covers

- Every `spec/std/*.cp` module: signatures, `///` docs (cost / traps / example / empty / ties / order), bodies in CP Lang, in-file `@test` fns.
- The prelude list (`spec/std/prelude.cp`) and its budget.
- One naming scheme across std (`spec/std/README.md` §2); `by:` options, never `_by` twins.
- The `spec/conformance/std-*` cases (co-owned with spec: stdlib writes them, spec owns the harness rules).
- `spec/std/GAPS.md`: each body the compiler can't run yet, with its request to the compiler lane.

Not covered (owner instead):

| Not here | Owner |
|---|---|
| The library chapters `spec/lib/*.md` (model, examples) | spec |
| Compiler builtins and intrinsics, `lower/call.rs`, the std/builtin boundary in the core | compiler (boundary held by stdlib contract tests) |
| Removing privilege as a ratchet, moving `spec/std` -> `std/` | tech-debt (stdlib does the std side) |
| C++ speed of std bodies, `cpl bench` | perf (stdlib supplies matched benches per module) |
| `cpl doc`, LSP hover/completion over std | tools |
| Site reference pages generated from std (`web/scripts/gen-ref.mjs`) | site |
| `cpl-conform` itself | infra / spec |

## Success criteria for v0.3

1. Zero `todo` in library bodies and in `@test` fns, except lines in `GAPS.md`, each of which names an open request in `plan/domains/compiler/worklog.md`.
2. Every doc example passes `cpl-conform docs`; every std conformance case passes, in the interpreter and in C++ (`cpl-conform diff`).
3. Every public fn has the required doc labels; `spec-stub.py --check` is clean.
4. No stdlib privilege: no public std name is special-cased in the compiler, and a std module copied into a user library runs the same.
5. Prelude matches the chapter tables (decision 150, see Q1); std modules acyclic.
6. Probes: a cold agent writes unseen problems from the docs alone with std calls that compile first time.

## Milestones

- **M1 Land and settle** (phase 2, first hours). Exit: the 4 in-flight branches are on main or noted superseded (`git cherry`); `std_acyclic` runs in the gate; `stdlib_todo` <= 200 (the branches cut 30, 82 and 8 on their own bases; re-measured after rebase); std conformance >= 815/832.
- **M2 Every body runs.** Exit: library-body `todo` = GAPS.md lines only, `@test` `todo` 0; doc labels missing 0; `spec-stub.py --check` 0 problems; docs >= 2430/2699 (90%); std conformance and `diff std-` 0 fail outside GAPS.md.
- **M3 The v0.3 surface.** Exit: prelude <= budget; privileged std names in `prelude.rs` 0; no-privilege test green; docs 100%; every module has conformance cases; probes P1-P3 pass.

## Metrics

All measured 2026-11-22 on main `4b8f504f` with fresh release binaries (`cargo build --release -p cpl-cli -p cpl-conform`). Commands in [metrics.toml](metrics.toml).

| Metric | Command (short) | Today | Target | M |
|---|---|---|---|---|
| `todo` stubs, total | `scripts/debt-ratchet.py --check` | 294 | 0 (GAPS only) | M2 |
| ... in library bodies / in `@test` fns | `measure.py todo-split` | 189 / 105 | GAPS / 0 | M2 |
| Doc examples passing | `cpl-conform docs` | 2012/2699 (74.5%) | 2699 | M3 |
| Std conformance cases passing | `cpl-conform conformance std-` | 801/832 | 832 | M2 |
| Interp vs C++ on std cases | `cpl-conform diff std-` | 735/757 (22 skip) | 0 skip, 0 fail | M2 |
| Pub fns missing a required doc label | `spec-stub.py --audit` | 32 of 2003 | 0 | M2 |
| `spec-stub.py --check` problems | `spec-stub.py --check` | 13 | 0 | M2 |
| Prelude names | `check.sh` prelude count | 192 | chapter tables (decision 150, Q1) | M3 |
| Std names special-cased in the compiler | `measure.py privilege` | 27 in `prelude.rs`; 111 literals in `call.rs` (upper bound) | 0 | M3 |
| Modules that pass standalone `cpl check` | loop in metrics.toml | 12/40 | 40 | M3 |
| `GAPS.md` entries | `grep -c '^- ' spec/std/GAPS.md` | 88 | falling; 0 at M3 | M3 |
| Import cycles (explicit `use`) | `measure.py cycles` | 0 | 0 | M1 |
| Modules without conformance cases | `measure.py uncovered` | 1 (`opt`) | 0 | M3 |
| Banned spellings | `spec-vocab.py` | 0 | 0 | - |

Not measured yet: implicit auto-import cycles (the `std_acyclic` test is on an unlanded branch); C++ time per std fn vs a matched library (perf owns `cpl bench`; no std-wide number exists); probe and rubric scores (no cold-agent run yet).

## Judgment rubric

A cold agent scores 10 sampled public fns 1-5 per question, with a reason; the trend is the signal.

1. **Guessable.** Knowing only the neighbours' names, would you have guessed this name, its argument order and its `by:`/`from:` options?
2. **Cost honest.** Does the body do what `cost:` says, and does anything O(n) or allocating look like it?
3. **One way.** Is this the only std call for its job, or does a neighbour do the same thing?
4. **Teaches.** Would a contestant learn the algorithm by reading the body, as from a good reference library (KACTL, ACL)?
5. **No privilege.** Could a user library have written this module, with nothing the compiler gives std alone?

Probes (cold agent, docs only; pass/fail, time, files touched):

- **P1 Write from the docs.** Solve 5 unseen problems using only `spec/std` docs; std calls compile and run first try (tests 1, 3).
- **P2 Add a std fn.** Add `kth_smallest`-sized fn following `spec/std/README.md`; gate green first try, <= 3 files touched (tests 1, 4).
- **P3 Copy a module.** Copy `std.fenwick` into a user library, run its cases from there (test 5).
- **P4 Audit costs.** Given 10 bodies, find the one with a planted wrong `cost:` (test 2).

## Improve (highest impact first)

1. Land the 4 in-flight branches: about 120 bodies, delaunay/voronoi/planar, the import-cycle guard.
2. Make std bodies the ones that run: 27 public std names (`sort`, `sorted`, `first`, `Counter`, `Subset`, ...) are also in compiler prelude tables and up to 111 are special-cased in `lower/call.rs`, so a builtin can shadow the body. tree, flow, dsu, graph hold 306 of the 687 non-passing examples; flow and dsu pass 0. Classify, then contract tests + compiler requests.
3. Unblock the 263 examples failing on two causes: a range where a domain is named (`Dsu<0..3>()`, 174, see Q2) and a list literal for `[D]T` (89, spec R04.8.10 already allows it: compiler gap).
4. Write the 105 `@test` bodies and the 189 library bodies not blocked by GAPS.md.
5. Cut the prelude from 192 toward the budget with a usage-driven rule (Q1).
6. Fix the 32 missing doc labels and the 13 `spec-stub --check` problems.
7. Get all 40 modules through standalone `cpl check` (12 today; 418 errors, part of them false self-cycles).
8. With tech-debt: one intrinsics boundary (Q3), the no-privilege test, `spec/std` -> `std/`.

## Ryan's role

Co-designer, through `plan/QUESTIONS.md` only: API shape, naming, what belongs in std and in the prelude, decided on real ports. 2-3 samples per digest (a body, a doc page, a port diff). Everything else is decided here and logged in `decisions.md`.

## Decisions needed from Ryan

Each is in [plan/QUESTIONS.md](../../QUESTIONS.md) with options and a reversible default.

- **Q1 Prelude: what earns a name.** Decision 150 already dropped the 80 budget for "the chapter tables, minus common locals". Usage ([prelude-usage.md](prelude-usage.md)): 5 of 192 names are used by >= 5% of 405 ports and hill-climb solutions, 77 by none. Recommend: keep decision 150 and retire the 80 warning. Default: keep 192.
- **Q2 A range where a domain is named.** `Dsu<0..3>()`, `BeatsSegTree<0..3>(fill: 2)`: accept an int range as an anonymous domain argument? Recommend yes (decision 159 already makes `(V, V)` an inline domain). Default: leave the 174 examples as written.
- **Q3 The compiler/std boundary.** Recommend: only storage, I/O, bit ops and numeric primitives stay intrinsics; `sort`, `min`, `sum`, `first`, ... become std bodies, guarded by a perf gate. Default: move nothing that costs speed; std bodies become the reference the builtins are tested against.
