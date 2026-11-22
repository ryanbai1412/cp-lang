# Contest dry run log

Default `cpl` (no `CPL_SPEC`), solutions written from the user docs only.
Each problem has `<p>.cp`, `<p>.tests/`, a C++ reference `<p>_ref.cpp` and a
generator `<p>_gen.py`. `verify.sh <p>` builds for `--judge codeforces`,
compiles the emitted C++ with `g++ -O2 -std=c++17` and compares it with the
reference on random inputs.

## Problems

- p01_ordinary (CF 1520B, multitest math): about 1 min, solved. check, run,
  test, stress (200) and verify (200) all pass. No friction.
- p02_movies (CSES 1629, greedy + sort): about 2 min, solved. All checks
  pass. No friction.
- p03_forest (CSES 1652, 2-D prefix sums via `prefix([Row, Col] ...)` and
  `.sum(rows, cols)`): about 2 min, solved. All checks pass. No friction.
- p04_factory (CSES 1620, binary search on answer with `bsearch_first`): about
  2 min, solved. All checks pass. No friction.
- p05_subarray (CSES 1660, two pointers): about 1 min, solved. No friction.
- p06_labyrinth (CSES 1193, grid BFS with `std.grid`, `bfs`, `.arcs`, path
  output): about 6 min, solved. Constructive output, judged by
  `p06_labyrinth_check.py` in both `cpl stress --checker` and `verify.sh`.
- p07_dijkstra (CSES 1671, `Graph.directed` + `dijkstra`): about 5 min,
  solved. Max test (n = 1e5, m = 2e5): 0.03 s vs 0.05 s for the reference.
- p08_roads (CSES 1676, `Dsu`): about 1 min, solved. Max test 0.02 s.
- p09_topo (CSES 1679 variant, lexicographically smallest topological order,
  `MinHeap` Kahn with `while let`): about 2 min, solved.
- p10_matching (CSES 1130, tree DP over `t.at.rev`): about 3 min, solved.
  Path of 2e5 vertices: 0.02 s, no recursion trouble.
- p11_distances (CSES 1133, rerooting with `t.reroot` and a custom `Monoid`):
  about 6 min, solved. Path of 2e5: 0.04 s.
- p12_rangeupd (CSES 1735, lazy segment tree, add + assign as one affine
  `Action`): about 3 min, solved. Max test 0.10 s.
- p13_dynsum (CSES 1648, `Fenwick`): about 2 min, solved.
- p14_elevator (CSES 1653, `Subset<P>` DP with `?(Int, Int)` and `min=`):
  about 2 min, solved. n = 20: 0.09 s.
- p15_bookshop (CSES 1158, 0/1 knapsack): about 1 min, solved. Max test
  0.02 s, matches a fast C++ knapsack.
- p16_lis (CSES 1145, patience LIS with `first_ge`): about 3 min, solved.
- p17_match (CSES 1753, `z_function`): about 3 min, solved. |t| = 1e6: 0.01 s.
- p18_binom (CSES 1079, `Mod` + `Comb`, multi-query): about 1 min, solved.
  1e5 queries: 0.03 s.
- p19_guess (interactive guess the number, 30 queries, `ask` +
  `@interactor`): about 8 min, solved only after replacing `bsearch_last`
  with a hand-written loop (P0 below). `cpl test`, `cpl stress` (300, with
  the co-located interactor) and the emitted C++ against
  `p19_guess_interact.py` (200 random + 3 edge cases) pass.
- p20_point (CSES 2189, `@!multitest` + `std.geometry.turn`): about 5 min,
  solved. 1e5 tests: 0.01 s vs 0.26 s for the reference.

Unless noted, every problem passed `cpl check`, `cpl test`, `cpl stress`
(300 cases against the reference) and `verify.sh` (200 random cases,
emitted C++ built with `g++ -O2 -std=c++17`). Every emitted file is at most
34% of the codeforces limit.

## P0

- p19, `bsearch_first` / `bsearch_last` waste queries. Over `1..=1e9` they
  call the predicate up to 33 / 31 times; 30 calls suffice. With
  `ask(limit: 30)` the documented interactive example in
  `io-and-formatting.md` (`bsearch_first(1..=1_000_000_000, at_least)`)
  breaks its own budget. Locally: "query limit exceeded: the limit is 30";
  the emitted C++ also stops after 30 queries without an answer.
  Expected: at most ceil(log2(hi - lo + 1)) calls. Workaround: hand-written
  `lo`/`hi` loop. Repro: `repro/bsearch_query_count.cp` (prints
  `1 33`, `2 32`, `1000000000 31` for inputs 1, 2, 1e9).

## Frictions

- P2, tooling: `cpl judges --help` treats `--help` as a file name:
  "cpl: can't read `--help`: No such file or directory (os error 2)".
  Expected: usage text, like every other subcommand.
- P2, docs: the brief points to `plan/1x-*.md`, but no such files exist;
  the language-as-implemented pages are `plan/10-*.md` to `plan/19-*.md`.
- P1, p11 (about 4 min): a named tuple type is lost through `t.reroot`.
  With `type Part = (cnt: Int, sum: Int)` and
  `Monoid<Part>(id: (0, 0), op: ...)`, unnamed tuple literals in `lift` make
  the generic `U` `(Int, Int)`: "error[E0203]: a `(Int, Int)` has no member
  `cnt` that `cpl run` knows". Naming only `finish`'s literal gives
  "error[E0302]: expected `(cnt: Int, sum: Int)`, found `(cnt: _, sum: _)`";
  annotating the lambda parameters gives "expected `((Int, Int), V) ->
  (cnt: Int, sum: Int)`, found `((cnt: Int, sum: Int), V) -> ...`".
  Expected: `Monoid<Part>` fixes `U = Part` and unnamed literals convert.
  Workaround: name the fields in every tuple literal in `lift` and `finish`.
- P1, p07 (about 3 min): `Graph.directed` needs edge fields named `u` and
  `v`; `(a: V, b: V, c: ...)` gives "error[E0302]: `Graph.directed` takes a
  [_](u: _, v: _) for `edges`, but `flights` is a [0..m](a: V, b: V, c:
  1..=1e9)", with no hint to rename and the source line printed twice. Also
  the arc type depends on how the edge array is indexed: over a declared
  domain `[E]` arcs are `(to, e)`; over `[0..m]` there is no `.e`. The docs
  only show the `[E]` form. Workaround: rename to `u`, `v`; declare
  `domain E`.
- P1, p20 (about 3 min): `stdlib.md` and `course-graphs-math.md` document
  `orient(a, b, c)` in `std.geometry`, but `use std.geometry.{Pt, orient}`
  gives "error[E0201]: `std.geometry` has no `orient`, and not in the stdlib
  (yet)". The function is `turn`. Expected: docs and stdlib agree, or the
  error suggests `turn`.
- P1, p19 (about 3 min): when the solution trips `ask(limit:)`, `cpl stress`
  reports only "interactor: the solution's output ended where a word was
  expected (line 31, column 1 of the solution's output)"; the actual reason
  ("query limit exceeded: the limit is 30") is not shown. Expected: the
  limit message and the transcript, as `io-and-formatting.md` promises.
- P1, p16 (about 2 min): `idioms.md` section 6 says "`lower_bound` /
  `upper_bound` on sorted lists", but "error[E0203]: a `List<Int>` has no
  method `lower_bound` that `cpl run` knows", with no suggestion. The real
  name `first_ge` is documented only under `compress`. Workaround:
  `tails.first_ge(v)`.
- P2, p06: `opt ?? (print(false); return)` gives "error[E0504]: not
  implemented yet: statements in parentheses, `(a; b)` (not in the design)".
  Workaround: `if arcs == none:` block, then `arcs!`.
- P2, p06, performance: open 1000 x 1000 grid BFS takes 0.16 s / 53 MB vs
  0.02 s / 10 MB for the C++ reference (a `List` of moves per cell).
- P2, p07: a `!` on `none` in the emitted C++ prints only "`!` on `none`:
  there is no value here" (exit 3), without the source line. (Hit with an
  input outside the CSES guarantee that every city is reachable.)
- P2, p11: `Graph.undirected([(u: V(1), v: V(2)), ...])` gives "`Graph.undirected`
  takes a List<(V, V)> for `edges`, but ... is a List<(u: V, v: V)>", while
  the same named fields read into `[0..n - 1](u: V, v: V)` are accepted.
- P2, p17: `count(i for i in a..b if z[i] >= m)` gives "error[E0302]:
  expected `Bool`, found `Int`"; `count` wants Bools. Workaround:
  `count(z[i] >= m for i in a..b)`.
- P2, p20: a multi-line `match` as a call argument,
  `print(match turn(...):` with arm `-1 => "RIGHT"`, gives "error[E0103]:
  expected `,` or `)` in the argument list, found `=>`". Workaround:
  `let side = match ...` then `print(side)`.
