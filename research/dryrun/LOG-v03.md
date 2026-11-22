# Contest dry run, 0.3

The 20 problems of `LOG.md`, rewritten from `spec/core/*.md`, `spec/std/*.cp` and the
`spec/lib/` guides only. Solutions are in `v03/`; each reuses the 0.2 run's statement, tests,
C++ reference and generator (`../pNN_*`). `v03/tri.sh` runs `cpl check`, `cpl run` on the
first sample and `cpl build --judge codeforces` + `g++ -O2 -std=c++17`; `v03/loop.sh` adds
`cpl test`, `cpl stress` against the reference and random runs of the emitted C++.
`repro-v03/run.sh` replays every repro.

Mode: `CPL_SPEC=1 CPL_STD=spec/std`, because "check: give the prelude only to programs in
0.3" was not on origin/main yet when the run started (tip a7c4810b); `cpl` with no
variables still ran 0.2. Rerun with no variables once it lands.

Cause: `std` = a `todo` stub or a std/compiler gap; `check≠run` = `cpl check` accepts what
`cpl run`/`cpl build` reject; `docs` = the spec shows a form the compiler rejects; `me` = my
mistake, with a correct error.

## Failures

| # | Problem | Construct | Step | Error text | Cause | Repro |
|---|---|---|---|---|---|---|
| 1 | p03 | `Prefix([R, C] (r, c) => ...)` (2-D, as `prefix.cp` says) | check | ``error[E0302]: `Prefix` takes a [Row]Int for `a`, but `[Row, Col] (r, c) => if forest[r, c] ...` is a [Row, Col]Int`` | docs | `repro-v03/prefix_grid_inference.cp` |
| 2 | p03 | `Prefix<R, C, Int>(...).fold(rows, cols)` | run / build | ``error[R0402]: `todo` reached``; g++: `void value not ignored as it ought to be` | std | `repro-v03/prefix_grid_fold_todo.cp` |
| 3 | p04 | `(1..=1e18).first_true(enough)!` | run / build | ``error[R0402]: `todo` reached``; g++: ``could not convert '<lambda closure object>prog::m1_first_true_Int_237(...)::<lambda()>()' from 'void' to 'cpl::OptInt'`` | std | `repro-v03/first_true_todo.cp` |
| 4 | all `todo` rows | any `todo` body in the emitted C++ | build | g++ rejects the file (`void value not ignored`, `could not convert ... from 'void'`) instead of building a program that traps | codegen | rows 2, 3, 11, 13, 14 |
| 5 | p05 | cut API: `c.r!`, `c.next!` on `I.cuts` (R04.3.2) | run / build | ``error[E0302]: expected `Range<Int>`, found `I.cuts` `` (earlier: ``a `I.cuts` has no member `next` that `cpl run` knows``, `` `!` takes the value out of an optional, but this is a `Int` ``) | check≠run | `repro-v03/cut_next.cp` |
| 6 | p06 | `bfs(c => c.adj4.filter(q => g[q] != '#'), from: [s])` (`std.grid`'s own example) | run / build | ``error[E0302]: expected `Int`, found `(Int) -> List<(Int, Int)>` `` | check≠run | `repro-v03/bfs_closure.cp` |
| 7 | p07 | `p[v]` on the `Paths` of `dijkstra` | run / build | ``error[E0302]: expected `Int`, found `V` `` | check≠run | `repro-v03/paths_index.cp` |
| 8 | p09 | `for a in g.out(u)` on `Graph.directed` | run / build | ``error[E0302]: expected `Int`, found `V` `` | check≠run | `repro-v03/graph_out.cp` |
| 9 | p10, p11 | `Rooted(Graph.undirected(edges), root: V.first!)` | run / build | `error[E0005]: expected a compile-time constant integer` | check≠run | `repro-v03/rooted_new.cp` |
| 10 | p12 | `SegTree(+, a, act: affine)` (ops-ranges.md form) | run / build | ``error[E0302]: expected `?Int`, found `(Int, Int) -> Int` `` (at the `+`) | check≠run | `repro-v03/segtree_op.cp` |
| 11 | p13 | `Fenwick(x).fold(a..=b)` | run / build | ``error[R0402]: `todo` reached``; g++: `void value not ignored as it ought to be` | std | `repro-v03/fenwick_fold_todo.cp` |
| 12 | p14 | `dp[{}]` and `dp[Subset<P>.empty]` on `[P.subsets]?T` (search-dp.md form) | check | ``error[E0301]: `dp` is indexed by P.subsets, but `{}` is a ()``; ``... but `Subset<P>.empty` is a Subset<P>`` | docs | `repro-v03/subset_index.cp` |
| 13 | p16 | `tails.keys.bsearch(i => tails[i] >= v)` (04-indices.md form) | run / build | ``error[R0402]: `todo` reached``; g++: `void value not ignored as it ought to be` | std | `repro-v03/bsearch_todo.cp` |
| 14 | p18 | `Comb<Z>.new()`, `.choose(a, b)` | run / build | ``error[R0402]: `todo` reached``; g++: ``could not convert '<lambda closure object>prog::Comb_S21_22_new(ll)::<lambda()>()' from 'void' to 'prog::Comb_S21_22'`` | std | `repro-v03/comb_todo.cp` |
| 15 | p19 | `wire.ask<Str>("? {y}") == ">="` | check | ``error[E0309]: chained comparisons cannot contain `==` or `!=` `` | syntax | `repro-v03/ask_type_args.cp` |
| 16 | p19 | `let reply: Str = wire.ask("? {y}")` (10-io.md form) | run / build | ``error[E0302]: expected `Str`, found `Int` `` | check≠run | `repro-v03/wire_ask_expected.cp` |
| 17 | p19 | `(1..=n).last_true(at_least)!` | run / build | ``error[E0203]: a `Range<Int>` has no method `last_true` that `cpl run` knows`` (alone: ``error[R0402]: `todo` reached``) | std | `repro-v03/last_true_run.cp` |
| 18 | p19 | `cpl interact` (10-io.md, tools/cli.md) | cli | ``cpl: unknown command `interact` `` | docs | none (CLI) |
| 19 | p20 | `turn(p1, p2, p3)` with no `use` (prelude has `Pt` but not `turn`) | check | ``error[E0201]: no `turn` in scope`` | std | `repro-v03/turn_scope.cp` |
| 20 | p05 | `l = l.next!` when the window empties past `r` | run | ``error[R0301]: `!` on `none`: there is no value here`` | me | none |
| 21 | p15 | `(price[i]..=x).rev()` | check | ``error[E0304]: `rev` is read without parentheses: `(price[i]..=x).rev` `` | me | none |
| 22 | p20 | multi-line `match` as a call argument | check | ``error[E0103]: expected `,` or `)` in the argument list, found `=>` `` | me (known 0.2 P2) | none |
| 23 | p07, p09 (workarounds) | `Heap<V>.new()`, `.push`, `.pop` | run / build | ``error[R0402]: `todo` reached``; g++: ``could not convert '<lambda closure object>prog::Heap_D0_V_18_pop(...)::<lambda()>()' from 'void' to 'cpl::OptPos'`` | std | `repro-v03/heap_todo.cp` |
| 24 | p07 (workaround) | `p.get(v)`, the method behind `p[v]` | run / build | ``error[E0302]: expected `Int`, found `V` `` | check≠run | `repro-v03/paths_get.cp` |
| 25 | p14 (workaround) | `var dp: [0..1 << n]?(Int, Int)` | check | ``error[E0103]: expected `,` or `]` in the index list, found `<<` `` | syntax | `repro-v03/shift_in_array_type.cp` |
| 26 | p06 (workaround) | `queue.push_back(s)` on `Deque` | check | ``error[E0203]: Deque<(Row, Col)> has no field or method `push_back` `` | me | none |
| 27 | p06 (workaround) | `p.step(d)` on a cell popped from a `Deque<(Row, Col)>` | run / build | ``error[E0302]: expected `Row`, found `Int` `` (at `g[q]`) | check≠run | `repro-v03/step_domain.cp` |
| 28 | p06 (workaround) | `"".join(moves.rev)` | run / build | ``error[E0302]: `join` takes a list or a 1-D array, not a `List<Str>` `` | check≠run | `repro-v03/join_rev.cp` |
| 29 | p06 (workaround) | `print(yes(true))` (10-io.md: `Bool` prints through `yes`) | run / build | ``error[R0402]: `todo` reached``; g++: ``could not convert '<lambda closure object>prog::m7_yes(...)::<lambda()>()' from 'void' to 'std::string'`` | std | `repro-v03/yes_todo.cp` |

## Results

"Spec" is the solution written the way the docs teach (`v03/pNN.cp`); "workaround" is
`v03/pNN_w.cp`, which replaces the failing constructs with hand-written loops and arrays,
still 0.3 syntax and no `stdlib/`. "Passes" means `cpl check`, `cpl test`, `cpl stress`
(100 cases against the C++ reference) and 100 random runs of the `--judge codeforces`
C++ built with `g++ -O2 -std=c++17` all agree; p19 is stressed against its `@interactor`
and the emitted C++ against `p19_guess_interact.py` (100 random + 3 edge cases).

| Problem | Spec version | First failure (row) | Workaround version |
|---|---|---|---|
| p01 multitest math | passes | none | not needed |
| p02 greedy + sort | passes | none | not needed |
| p03 2-D prefix sums | fails | 1, 2 | passes (hand-built table) |
| p04 binary search on answer | fails | 3 | passes (hand-written search) |
| p05 two pointers | fails | 5 | passes (Int indices) |
| p06 grid BFS | fails | 6 | passes (hand BFS on `Deque`, steps by hand, no `yes`) |
| p07 Dijkstra | fails | 7 | passes (hand Dijkstra and heap) |
| p08 DSU | passes | none | not needed |
| p09 topological sort | fails | 8 | passes (hand adjacency and heap) |
| p10 tree DP | fails | 9 | passes (hand rooting) |
| p11 rerooting | fails | 9 | passes (hand rooting, two passes) |
| p12 lazy segment tree | fails | 10 | passes (hand lazy segment tree) |
| p13 Fenwick | fails | 11 | passes (`prefix(b.r) - prefix(a.l)`) |
| p14 subset DP | fails | 12 | passes (Int masks) |
| p15 knapsack | passes | none | not needed |
| p16 LIS | fails | 13 | passes (hand lower bound) |
| p17 string matching | passes (`find_all`) | none | not needed |
| p18 modular combinatorics | fails | 14 | passes (hand factorial tables on `Mod`) |
| p19 interactive | fails | 15-17 | passes (`print`/`read`, hand search) |
| p20 geometry | fails | 19 | passes (`use std.geometry.turn`) |

## Recheck on f605b8a4

The stdlib bodies landed since the run (a81702ad, f605b8a4, still under `CPL_SPEC=1
CPL_STD=spec/std`) clear eight repros: rows 2, 3, 11, 13, 17, 23 and 29 now run and build
and print the right value (`repro-v03/run.sh`). Row 14 (`Comb`) still reaches `todo`, and
row 4 still applies to it. Every other row reproduces unchanged.

Spec versions rerun with `v03/loop.sh` (check, test, 100 stress cases, 100 emitted-C++ runs):

| Problem | Now |
|---|---|
| p03 (`Prefix<Row, Col, Int>` form) | passes |
| p04 (`first_true`) | passes |
| p13 (`Fenwick.fold`) | passes |
| p16 (`keys.bsearch`) | passes |
| p19 | still fails: rows 16 and 17 (`last_true` is still unknown to `cpl run` next to the `Wire` error) |
| p20 | still fails: row 19 |

That makes 9 of 20 pass as the docs teach: p01, p02, p03, p04, p08, p13, p15, p16, p17.

## Recheck on b4b4f1e9

`repro-v03/run.sh`, same mode. Changes since f605b8a4:

| Row | Construct | Now |
|---|---|---|
| 5 | `I.cuts` with `.next!` / `.r!` | runs and builds; p05 spec version passes the full loop |
| 7, 24 | `dijkstra(Graph.directed(...))` then `p[v]` / `p.get(v)` | now fails at check, one step earlier: ``error[E0606]: `Graph<_, _>` does not implement `WeightedNext` required by `dijkstra` `` (the `w:` field graph of the `std.graph` docs) |
| 15 | `wire.ask<Str>(...) == ">="` | check, run and build now say ``error[E0606]: `fn(query: Str) -> _` does not implement `Ord` `` |

Every other row is as in the f605b8a4 recheck. 10 of 20 pass as the docs teach: p01, p02,
p03, p04, p05, p08, p13, p15, p16, p17.

## Recheck on 2cfe7fa9 (20/20, default v0.3)

Rerun with `CPL_SPEC`, `CPL_STD`, and `CPL_EDITION` all unset using compiled `target/release/cpl` (`v03/loop.sh <name> 100` for p01–p18 and p20; `cpl check`, `cpl test`, `cpl stress --cases 50`, and `--judge codeforces` C++ against `p19_guess_interact.py` for p19).

Of the 10 problems that failed on `b4b4f1e9`, four (`p09_topo`, `p10_matching`, `p11_distances`, `p18_binom`) were already passing on `origin/main` before this pass (`4d2d37ee`). The remaining six were resolved as follows:

| Problem | Failure on `4d2d37ee` | Cause | Fix | Commit |
|---|---|---|---|---|
| `p06_labyrinth` | `E0302`: `Str += Char` rejected when RHS is an `if` yielding `Char` | `cpl-ir` `assign` only handled `Str += Char` for direct char literals, not general `Ty::Char` expressions | Allow any `Ty::Char` RHS on `Str +=` in `crates/cpl-ir/src/lower/stmt.rs`; regression `spec/conformance/core-03/str-plus-eq-char-if.cp` | `8b56aaf2` |
| `p07_dijkstra` | `R0402`: `todo` reached in `std.graph.dijkstra` | `dijkstra` body in `spec/std/graph.cp` was a `todo` stub | Implemented `dijkstra` (single/multi-source, `until:`, weighted adjacency, `Paths` reconstruction) in `spec/std/graph.cp`; regression `spec/conformance/std-graph/dijkstra-multi-source-and-until.cp` | `3155c313` |
| `p12_rangeupd` | `SegTree(+, ..., act: affine)` rejected (`act:` unimplemented, `affine` was `todo`, C++ emitted `std::optional<Action> != std::optional<Action>`) | `SegTree` lacked lazy tag storage/push-down/`apply`, `Action.of` and stock action markers were `todo`, `F` wasn't inferred from `act:`, and codegen only lowered `x == None` to `.has_value()` for `?Fn` | Implemented `SegTree<D, T, F = Int>` lazy `apply`/`apply_all`/push-down in `spec/std/segtree.cp`, `Action.of` and stock action markers in `spec/std/algebra.cp`, `F` inference and marker `same()` in `cpl-check`/`cpl-ir`, and general `Ty::Opt` `.has_value()` in `cpl-codegen`; regression `spec/conformance/std-segtree/lazy-affine-and-add.cp` | `2cfe7fa9` |
| `p14_elevator` | `E0301` on `dp[Subset<P>.empty]` (`P.subsets` domain not recognized as `Subset<P>`) and `E0302` (`expected Subset<P>, found Subset<P>`) | `D.subsets` domain lowering didn't bind `Subset<D>`, and `Sym::GenStruct` shadowed `Subset<D>.empty`/`.full` in `cpl-ir` | Supported `D.subsets` domain arrays/loops and generic method return hints in `cpl-check`/`cpl-ir`; regressions `spec/conformance/core-04/domain-subsets-array-and-loop.cp` and `spec/conformance/core-05/generic-method-return-hint.cp` | `52ce17e0` |
| `p19_guess` | `E0203`: `Range<Int>` has no method `last_true` in `cpl run`, and `wire.ask` return hint didn't infer `Str` | `cpl-check/src/db.rs` didn't scan `{...}` string interpolations when collecting prelude/std method imports, and `wire.ask` needed generic return-type hint propagation | Scanned `{...}` interpolations in `crates/cpl-check/src/db.rs` (`spec/conformance/core-03/interp-std-method-import.cp`) and propagated generic method return hints (`52ce17e0`) | `8d7d2f12`, `52ce17e0` |
| `p20_point` | `E0201`: no `turn` in scope | `turn` lives in `std.geometry` (not prelude); solution omitted the import | Added `use std.geometry.{turn}` to `research/dryrun/v03/p20_point.cp` | `df84de3f` |

Result on `2cfe7fa9`: **20/20** spec solutions pass end-to-end (`p01_ordinary` through `p20_point`).

