# stdlib plan

The detail behind [README.md](README.md). Numbers are from the runs in [metrics.toml](metrics.toml) (2026-11-22, main `4b8f504f`).

## Where the debt is

- **Bodies.** 294 `todo` stubs: 189 in library bodies, 105 in `@test` fns (`measure.py todo-split`). 146 doc examples reach a `todo` (`cpl-conform docs --causes`, R0402).
- **Examples that don't compile.** Of 687 non-passing doc examples, two causes cover 263 (`--causes`): 174 E0204 "takes a domain by name, and `0..5` is a range" (an int range where a type argument names a domain), 89 R0001 "an array literal needs one value per position" (a list literal for `[D]T`, which spec R04.8.10 allows).
- **Whole modules.** `flow` 0/72 and `dsu` 0/64 examples pass; `tree` 123/239, `graph` 87/141 (`cpl-conform docs <m>.cp`). Only 12 of 40 files pass standalone `cpl check` (the `SPEC_STD` ratchet in `crates/cpl-cli/tests/corpus.rs` lists the same 12).
- **Compiler gaps.** 88 entries in `std/GAPS.md`: bodies the compiler can't run yet.
- **Surface.** Prelude 192 names (`check.sh` still warns above 80, a budget decision 150 dropped; usage in `prelude-usage.md`); 32 public fns lack a doc label; 13 `spec-stub --check` problems (e.g. `ternary_max` is in no chapter).
- **Privilege.** The builtin count, agreed with tech-debt, is `ir_builtin_names` (`scripts/debt-ratchet.py`): 89 distinct names string-matched in match arms of `cpl-ir/src/lower/call.rs`; 40 of them are public std names (`measure.py privilege`).

## Milestone 1 order (from 2026-11-22, due 09:30 PST 2026-11-23)

Milestone 1 (`plan/product/MILESTONE-1.md`) sets the order, in hours: a 2-hour checkpoint in the worklog; the workstreams below serve it. Work that unblocks a step of the acceptance scenario comes first:

1. **Core-set examples** (README "Milestone 1"): `seq` and `graph` first, then `bag`, `fenwick`, `strings`, the rest of the core set. Each failure is a std bug fixed here or a compiler gap filed with a contract test (W2 + W3, restricted to the core set).
2. **Acceptance-run misses**: every std call an idioms acceptance run or writable-from-docs run reached for and didn't find, or found wrong. Fix, or GAPS.md + compiler request.
3. **Judge path**: each core module through `cpl build --judge codeforces` and `cpl-conform diff`; CRLF input.
4. **Core perf floor**: core containers within 1.5x of builtin/hand C++ (the `sort` body, generator streaming for `sum`/`count`/`all`/`any`), only where a builtin isn't already serving the call.
5. Then the rest, unchanged: W3 bodies outside the core set, W4 labels, W5 prelude, W6 privilege moves, breadth modules. These are "what can wait" unless cheap or a hard gate.

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

## General fixes owed

Each: the instance, the general fix, the owner when it is not stdlib.

- **Dual `SegTree` (cf2255e1/e2).** Instance: a custom `Action` on a tree with no fold op still needs an `over:` monoid it never uses (`Monoid<Cell>((0, MaxSub.empty()), (x, _) => x)`), and cf2255e1 runs 0.85 s against 0.34 s for the C++ (2.5x). General fix: `Action` without `over:` for tags that only reach points, and tag closures lowered to direct calls on the dual path. Owner: stdlib, with compiler for the closure lowering.
- **`a.sorted().rev[0]` reads the source key (BLOCKERS #2, P0).** Instance: indexing a reversed view returns `a[0]`, silently. Spec R07.5.3 makes `w.rev[i]` E0305 with the fix `w.rev.nth(k)`. General fix: the reversed view is its own type that is walkable but not indexable. Owner: compiler.
- **`[true, false].count()` counts all (BLOCKERS #3).** `count()` is the element count by spec (`walk.count()`). General fix: a lint on `count()` over a `Seq` of `Bool` suggesting `.count(x => x)` or `.len`. Owner: lint-fmt.
- Spec's `core-11/member-*` cases declare a struct inside an inline `mod m:`; lowering registers only an inline module's fns, so `P` is unknown (E0204). General fix: an inline module declares structs, enums, traits and impls exactly as a file module does. Owner: compiler.

- **`nth` on `[D]T` inside std (cf1943b).** Instance: `manacher`, `z_function` and the other `std/strings` functions failed on a `[Pos]Char`. Fixed generally here: `S: Indexed<K, T>` binds `K` to the receiver's key (`mono.rs`), `.at` takes the receiver's key type (`call.rs`), and `cells.rekey(s.keys)` over a whole domain is a `[D]T` (`mod.rs`). Still owed: a std module only sees a method it `use`s (`std/strings.cp` lacked `use std.seq.{nth}` and nothing said so until a `[D]T` reached it); the general fix is a check that every std module resolves its method calls for every `Indexed` shape, e.g. one conformance case per module over `[D]T` (stdlib).
- **`find_all` on `[D]T`.** Its empty pattern yields the end key, which a `[D]T` has no value of, so `find_all` on a domain array doesn't compile. General fix: yield cuts (`D.cuts`) rather than keys, or spec rules the empty pattern yields keys only (spec).
- **Corpus gate misses ports without `spec.cp`.** `cpl-conform corpus` and `crates/cpl-cli/tests/corpus.rs` only run `corpus/problems/*/spec.cp`; cf1943b has only `port.cp`, so its break went unseen. General fix: the corpus job runs every `port.cp` on its `tests/` in the interpreter (idioms, requested in their worklog).
- **Tuple/record monoids.** Instance: `Monoid(id, each((size: +, dist: +)))` inferred an `Int` element (reroot docs, cf1984e). Done generally here: named variadic args make a record (decision 324, checker and IR), `fields(a, b, ...)` walks values of different types with as many fields together, `T(...)` builds a tuple or record type parameter, and `Monoid.of` falls through to the value's `.monoid()`, so `each(n: Monoid(..), h: Monoid(..))` works in both backends (`conformance/std-algebra/each-monoid-components.cp`). A bare operator packed into a tuple or record (`each(+, max)`, `each(n: +, h: max)`) now keeps its identity as a value of its own empty type (decision 353, `conformance/std-algebra/each-of-bare-operators.cp`). Still owed: `Monoid((size: 0, dist: 0), each(size: +, dist: +))`, where the identity names the element type, needs the positional constructor to accept an `Each`; reroot's docs spell it `Monoid<(size: Int, dist: Int)>.of(each(..))` until then (stdlib).

- **A stored `()` in the C++.** Instance: `MonotoneHull<()>` keeps `tags: List<()>`, which the interpreter runs but the C++ emits as `std::vector<void>`; `let x: () = ()` emits `x = (void)0` with no declaration. General fix: a `()` that is stored (a local, a field, a container element) gets an empty C++ type such as `cpl::Unit`, keeping `void` only for returns (codegen).

- **Element types of a two-sided std generic.** Instance: `zip<A: Indexed, B: Indexed, T, U>` bound `T` and `U` both from `a` (the lowerer's "first `Seq` bound gives `T`" guess), so `zip([1], "a")` failed. Fixed locally by writing `A: Indexed<K, T>, B: Indexed<L, U>` in `zip`, `zip_exact`, `product`. General fix: drop the guess and bind element generics only through `Indexed<K, T>`/`Seq<T>` arguments, sweeping std signatures to name them (compiler, then stdlib).
- **Views keep their source.** Instance: `Filtered`/`Mapped`/`Rev`/`Items`/`Slice` `.src` (5 doc examples) and `filter` keys (`[5, 6, 7].filter(p).items` gives keys 0, 1, not the source's 1, 2): the compiler materialises `filter`/`map`/`rev` as a `List`. General fix: lower views to std structs over their source, as spec/lib describes (compiler).

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
