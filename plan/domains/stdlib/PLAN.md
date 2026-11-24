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

Milestone 1 (`handbook/product/milestones.md`) sets the order, in hours: a 2-hour checkpoint in the worklog; the workstreams below serve it. Work that unblocks a step of the acceptance scenario comes first:

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

- **`convolve`'s fast product covers only NTT-prime `Mod<P>`.** Other moduli, `Int` and `F64` still take the \(O(n m)\) loop. General fix: three-prime CRT and FFT as further `FastProduct` impls (stdlib).
- **`same(over.add, +)` is a run-time test,** so `convolve`'s semiring branch is lowered for every `T` and a user ring type must define `one` even for the plain product (`std-ntt/convolve-fast-product`). General fix: settle `same` of two operation values at compile time, as `if T: Trait` now is, so an untaken branch imposes nothing (compiler).
- **A missing `T.zero` inside std reports "`cpl run` doesn't know the name `T`"** instead of "`Tal` has no `zero`, which `convolve` needs" at the call. General fix: the type-qualified lookup error names the bound type and the missing member (compiler, diagnostics).
- **A generic domain bound by a literal-sized table dimension is just `Int`** (5 `sparse.cp`
  `SparseTable2D` doc examples, `[2, 2] (r, c) => ..`): `R.K`, `R.first!` and `for r in R` need the
  size, which `Int` loses ("doesn't know the name `R`"). Works with declared domains. General fix:
  bind `R` to the table's anonymous `0..2` domain (R04.4.30), not `Int`. Owner: compiler.

- **`Subset` is an IR builtin in `cpl run`** (6 `subset.cp` doc examples): std's `complement`,
  `shifted` and `single(B)` are never reached (`Subset<B>` has no method `complement`). General
  fix: Q3, lower `Subset` from `std/subset.cp`'s struct instead of `Ty::Subset`. Owner: stdlib + compiler.
- **`dfs_tree`'s return type is mangled** (`Rooted<V,\n _>`) and its body is `todo`. Owner: stdlib.

- **A generic bound only by a lambda parameter defaults to `Int`** (`window.cp#undoing`, 4 doc
  examples): `undoing(Dsu<V>(..), (mut d, e: (V, V)) => ..)` binds `T` to `Int`, even with the
  parameter annotated. General fix: unify `T` with an annotated lambda parameter's type. Owner: compiler.

- **`.count()` on a `Bool` sequence counts every element** (P0 `count-of-bools-counts-all`):
  no-arg `count` is specced for walks (core 05, `walk.count()`), so making it count `true`s
  only for `Bool` would be a special case. General fix: a typed lint on `.count()` whose element
  type is `Bool`, fix `.count(x => x)` (or `.len` when the length is meant). Owner: lint-fmt.
- **Reversed views rekeyed by copying** (`a.rev.keyed()`, f33be6bdb): a view flag for "keys
  follow the walk" would keep it \(O(1)\). Owner: compiler/interp.

- `cpl check std/*.cp` checker false positives left after the bound check (40 of the 41 errors on 2026-11-23, counting `maximal`/`minimal` as one); general fix for each, owner compiler: (a) `D: Domain` in a bound is E0204 "no trait named `Domain`" (graph.cp:132, seq.cp:761), so a generic domain's positions can't be bounded; (b) position arithmetic on a generic domain parameter (`R.at(self.0 - 1)` in grid.cp `up`, `p + 1` in search.cp:357) asks for `R: Sub`, but R04 says position arithmetic gives `Int` for any domain; (c) a const generic `P` (`Mod<P>`, ntt, poly) asks for `P: Sub`; (d) `Neg`, `BitOr`, `BitAnd`, `BitXor` are "not a trait" though R08.1.5 lists them (math.cp:1472, subset.cp:145); (e) an enum variant named `Sub` (`SosOver`) shadows the `Sub` trait in bound position (ntt.cp:288, 342); (f) parser: an associated type with a bound (`type Arc: Target<V>`, graph.cp:29), a trait alias with `where` (graph.cp:80), and the `(I:, [I] i => ...)` domain-tuple return (seq.cp:967); (g) a `var` re-declared in each loop iteration stays "already moved" (matrix.cp:505); (h) `?D` with `D` a domain position isn't `Eq` (seq.cp:897); (i) std's own `@get l`/`r` (seq.cp:992, 1000) are defined on the primitive `self.start`/`self.end`, which the checker now rejects while lowering accepts: the primitive needs one intrinsic spelling both agree on (or the getters become intrinsic with their docs kept).
- `cpl lint --fix std/` (L0549/L0540 rewrites) put `x.bit(i)` and `r.first_true(..)`/`r.last_true(..)` into std modules that don't import them, so `cpl check` passed but `cpl run` rejected 94 doc/spec examples ("no method `bit` that `cpl run` knows"). Local fix: the `use std.math.{bit}` / `use std.seq.{first_true, last_true}` lines added by hand. General fix, owner lint-fmt: a fix that calls a std extension method adds its `use` when the file is a std module, and is skipped when a local name shadows it (std/bitset.cp's own `fn bit` made us revert one rewrite). Owner compiler: the checker should reject the same unimported method `cpl run` rejects, so the gap shows in `cpl check`.
- Adding `use std.seq.{last_true}` to std/math.cp made seq.cp:2862 (a test with `Monoid(..)` and no `use std.algebra`) fail E0201, so a name in seq.cp was resolving through another module's imports depending on load order. Local fix: math.cp keeps its hand-written binary search. General fix, owner compiler: resolve each std module's names from its own `use` lines only, independent of load order; then seq.cp's test gets its own `use`.
- `a min= b` on two `List`s (or one-index arrays) takes the min cell by cell (`is_row` in cpl-ir lower/stmt.rs), while `min(a, b)` and `a < b` compare lexicographically, so `best min= rot` is not `best = min(best, rot)`. Lint's fix rewrote `if rot < best: best = rot` into it and broke `planar_faces` (std/geometry.cp `least_rotation`, 3 doc examples trapped). Local fix: the `if` kept. General fix: owner spec, make `x min= y` mean `x = min(x, y)` for every type, with the cell-by-cell form spelled separately (e.g. `a[..] min= b`); owner lint-fmt, until then don't offer the `min=`/`max=` rewrite when the operands are rows.
Each: the instance, the general fix, the owner when it is not stdlib.

- **`[D]List<T>` input (R10.2.3) is E0501 (btoi17p1, mccc4j5).** `read()` rejects `[P]List<P>`, `[P]List<Int>` and a `List<V>` record field, though spec says `[D]List<T>` reads `D`-many count-prefixed rows. General fix: `readable` routes `Ty::List(T)` to a count-prefixed read of `T` (or a library `impl Read for List<T>`, as `Mod<P>` has), so a list nests anywhere a token type does. Owner: compiler.
- **coi06p2 needs a DFS tree.** `Rooted(g, root)` is the BFS spanning tree, so the port's back-edge low-link is wrong on 27 of 40 random cases (checked against a brute force) once it compiles. `dfs_tree` has landed (`_` anywhere in a return type now works); the port should use `tree_edge[a.e]`. Owner: idioms (port).
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
- **Integer literals past 64 bits.** Instance: `norm2_128 == 25_000_000_000_000_000_000` (geometry doc example) and `let a: I128 = 25_000_000_000_000_000_000` give E0005; the example now writes `I128(5_000_000_000) * 5_000_000_000`. General fix: type an integer literal from its context, so an `I128` context takes a 128-bit literal (compiler; spec to state it).
- **Doc-example ids skip `@`-attributed items.** Instance: `docs.rs` skips every line starting with `@`, so the examples of `@get pub fn norm2_128`, `angle`, ... are filed under the next plain item (`geometry.cp#Pt.unit_dir:4` was `norm2_128`'s). General fix: strip leading attributes and name the item, rewriting the scoreboard ids once (spec, which owns cpl-conform).
- **A call whose argument mutates the receiver.** Instance: `t.all(t.apply(t.base, 0..3, 2))` (`PersistentLazySegTree.all` doc example) reads `t` from before `apply` pushed nodes and traps on index 6; the example now binds the version first. General fix: evaluate the receiver after the arguments, or reject a receiver that an argument mutates (compiler; spec to state the order).
- **`Char` as a domain.** `Char.from_idx`/`try_at` now lower in `lower/type_call.rs`, and `Char.K`/`len`/`count` in `call.rs`; each spells the 128 itself. General fix: give `Char` the whole domain protocol (`nth`, `at`, `first`, `last`, iteration) from one declaration of its size, through the path declared domains take (compiler).
- **`SortedRange` over open key ranges.** Instance: `SortedBag.range`, `SortedSet.range`, `SortedSet.sum` and the 6 `SortedRange` members are `todo` (16 doc examples). Two powers are missing: a `Range<K>` with open ends (`a..`, `..b`, `..`) for an ordered key `K` (lowering gives "an unbounded range value"/"a range with neither end", and R04.8.6 allows bare `..` only inside `[...]`), and a struct field that lends the container (R05.7.2). General fix: an ordered-range value carrying `?K` ends and inclusivity, plus lent struct fields, so `SortedRange` is a plain std struct; `Map.range`/`Set.range` then lose their checker special case (compiler, spec for the open-range rule).
- **`PMultiset` needs a shared node pool.** Instance: its 5 methods are `todo` (5 doc examples): a value of `root: Int` with \(O(1)\) hashed `==` needs nodes stored outside the value. General fix: decide how a persistent immutable std value owns its nodes (an arena handle passed in, as `PersistentSegTree` does, or a per-type pool); stdlib decision, then stdlib.
- flow.cp doc examples (26) wrote `[0..2]List<0..2>` and `[0..3](u: 0..4, ..)`: a range type in element or field position binds the generic as `Int`, so `R` in `[R]?L` is E0302 "`R` ... must be a domain". R04.4.30 makes a range an anonymous domain only as an answer space or array dimension (that case, `core-04/range-dim-binds-generic-domain`, passes). General fix: a range type that binds a parameter the generic uses as a domain is that anonymous domain wherever it appears; `core-04/range-type-binds-generic-domain` fails until then. Owner compiler (spec to widen R04.4.30's wording). Local fix: the examples declare `domain L = 0..2` etc., which reads better anyway.
- `f(x) == (size: 2, mate: [Some(1), Some(0)])` is E0302 "can't compare `(size: Int, mate: [L]?R)` with `(size: Int, mate: List<?Int>)`", though `f(x).mate == [Some(1), Some(0)]` works: the expected type doesn't reach list literals inside a tuple/record literal. General fix: one contextual-typing rule for literals, applied field by field. Owner compiler. Local fix: flow examples compare field by field.
- `matroid_intersection(GraphicMatroid(es), ColorMatroid(col, lim))` infers `E` as `V`: `impl Matroid<E> for GraphicMatroid<V, E>` is matched by position instead of through the impl. General fix: solve a bound `A: Matroid<E>` from the impl header. Owner compiler. Local fix: the example writes `matroid_intersection<E>(..)`.
- `let z = PermNode.Leaf(1)` (generic enum, `N` not fixed by the variant's fields) says "`PermNode` is a type, and a value is needed here" instead of naming the unfixed parameter. General fix: one "can't tell `N`; write `PermNode<I, N>.Leaf(..)` or annotate" diagnostic for every generic whose arguments a call leaves open, and `E<..>.Variant` accepted (it is E0203 today). Owner compiler/diagnostics.
- `boruvka(comp => [Some((u: V(0), v: V(1), w: 5)), ..])` can't infer `V`/`W` from the lambda's result, so the doc writes `boruvka<V, Int, Int>(..)`. General fix: unify a function parameter's declared return type with the lambda body's type, as arguments already are. Owner compiler.
- The views `Mapped`, `Rev`, `Items`, `Slice` declare `@get src` with a `todo` body, and `cpl run` lowers `a.map(f)` etc. to its own iterator with no `src`, so their 5 doc examples fail; `Rebased` (`a[1..].rebase.nth(1)`) has no `at` in `cpl run` for the same reason. General fix: lower the views to the std structs (or give the builtin views every member the std struct declares), so std's declaration is the one truth. Owner compiler (Q3 builtin migration).
- `SegTree<0..3, Int>.of(+, i => w[i])` with `w: List<Int>` gives "expected `Int`, found `Range0`": the anonymous domain from `anon_range_domain` is nominal, but R04.4.30 says its positions are the `Int`s in the range. The docs now declare `domain D`. General fix: a position of an anonymous range domain is an `Int` (or converts to one implicitly, as the spec says). Owner compiler.
- `SegTree(act: assign, [1, 2, 3])` put the list into the first positional parameter `op`, and the error was "`cpl run` doesn't know the name `D`" deep inside std. General fix: when a positional argument lands on a parameter its type can't fill, say so at the call ("`[1, 2, 3]` went to `op:`; did you mean `a:`?"). Owner diagnostics.
- Spec disagrees on `filter` keys: R07.3.2 says keys of a `Keyed` survive `filter` (std's `[5, 6, 7].filter(x => x > 5).items` example expects `(1, 6), (2, 7)`), R04a.9.5 says `.filter(p)` has no keys; `cpl run` gives `(0, 6), (1, 7)`. General fix: spec picks one rule, then the builtin view and the doc follow it. Owner spec, then compiler.

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
