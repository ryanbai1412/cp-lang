# Hill-climb log

One line per spec change that a corpus dream forced. The problem set is
`corpus/hillclimb.json`; each dream lives in `corpus/problems/<p>/dream.cp`
and its notes in that record's `notes`.

- aac4p6: `Deque.pop_while`, matching `List.pop_while`.
- apio09p3: `s.filter_map(f)` in `std.seq`.
- btoi17p2: `Semiring(add, mul)` built from two operation values and
  `Mat(over:, a)` replace `Semiring.min_plus()` and `mul_semiring`.
- btoi17p6: `Rooted.from_parents` takes any keyed parents, so the
  "parents of 2..n" input is `read<[V.drop(1)]V>()`.
- btoi17p2: `Mat` and `Semiring` join the prelude, next to `SegTree` and `each`.
- btoi20p3: `ParityDsu` joins the prelude beside `PotentialDsu`; `yes(b)`
  gets a stub (`std.io`) and returns the answer `Str`.
- ccc03s4: `SuffixArray.new(s)` stub, `SuffixArray` joins the prelude.
- ccc18s4: `floor_quotients` joins the prelude.
- cco18p4: `Wire` gets a stub; the ternary searches join the prelude and
  document golden-section probe reuse (interactive query budgets).
- cco20p4: `reachable(g, from:)` becomes the free generator decided in round 4
  (the `Graph.reachable` method goes); `Rooted.euler_tour()`; `dfs`, `Visit`
  and `reachable` join the prelude.
- cco21p2: `bfs`, `bfs01` and `dijkstra` take `hashed: true` for state spaces
  that are not bounded types; `Paths` documents both storages.
- cco21p5: `inf` (the `Int.MAX` that search-dp promised) gets a stub and
  joins the prelude.
- cmt21r2ztr: `TwoSat` and `Lit` join the prelude.
- coci21c2p5: `Lift.climb(v, ok)` for greedy binary-lifting jump counts; `Lift`,
  `LcaTable` and `JumpTree` join the prelude.
- dmopc21c4p1: `locate` and `Location` join the prelude.
- coci21c2p5 (follow-up): `compress` returns `(domain C, Compressed<C, T>)` as chapters 4 and
  7 say; `Rooted.new`/`from_parents`/`forest`, `virtual` and `PermTree.new` declare their
  fresh-domain results with `_`; `spec-stub.py --check` now fails a pub fn that mutates
  nothing and declares no return type, since methods on its result went unchecked.
- coci20c6p3 (follow-up): map and `SortedMap` `keys`, `values` and `items` are O(1)-to-make
  getters like `a.keys`/`a.items` on any keyed sequence (map `each()` became `items`); `Dsu.values`
  too. Two stale ports lost their `.last()`/`.values()` drift.
- ioi96p1: a `Range<V>` passes where `V.ranges` is expected, with non-emptiness as a checked
  contract like a range-typed slot.
- sac22cc3jp4: `Str.parse_int(base:)` and `Int.to_str(base:)` convert between bases.
- Folds: `sum`, `min`, `max`, `count`, `any`, `all` are now `std.seq` stubs, not checker-only
  builtins (no stdlib privilege). `min`/`max` stay out of the prelude list until a receiver
  picks the fold over the binary form (parser gap).
- etsr23r3p3: `kruskal_tree` binds its node domain at the call (`let (domain N, kt) = ...`), as
  `compress` does, and accepts edge records with more fields.
- noi14p5: chapter 5's exercise named `a.swap(i, j)`; the stub is `swap_at`, so the doc now says
  `a.swap_at(i, j)` (the free `swap` keeps its two-place meaning).
- noi09p4: `max_closure` joins the prelude.
- noi02p3: one `convolve(a, b, on:, over:, shape:, cyclic:)` as ops-ranges §13 already said;
  `convolve_mod`/`_int`/`_real`/`_or`/`_and`/`_xor`/`_subset`/`_gcd`/`_lcm` and
  `min_plus_convolve`/`max_plus_convolve` are gone. `Curved.Neither` names the O(n m) loop that
  a small tree knapsack wants.
- noip99p1: one `lis` (the `std.seq` method, now with `by:`); the duplicate free `lis` in
  `std.dp` is gone and `lis_ends` is a method. `read_all<T>()` reads until end of input.
- noip18p3: `Rooted.children(v)` yields from the stored child layout instead of returning a
  fresh `List` per call.
- etsr23r1p4: `BeatsSegTree` joins the prelude.
- ioi16p6: `MonotoneHull<P>` lines carry a tag and `best_next` returns the winner's tag, ties to
  the smaller one, so an `aliens` DP knows its count; `MonotoneHull`, `LineContainer` and
  `aliens` join the prelude.
- ioi05p2: enums read by variant: `@tag("Q")` names a variant's token (default: its name), so a
  command stream is `read<Cmd>()` in a loop. Grammar allows decorators on variants; recorded in
  `parser-gaps.md`.
- ioi09p6: `spec/std/grid.cp` now exists (the prelude imported `std.grid` with no spec stub);
  `adj4`, `adj8` and `ray` are generators, and `transposed` is a call because it is O(n²).
  `bfs` gained `enter: (V, Int) -> Bool`, so "the cell must still be safe at distance d" is a
  filter instead of a hand-written layered search.
- dmopc21c4p2: `Sieve.divisors(x)` yields divisors (no allocation, unordered) instead of
  returning a sorted `List`.
- containers: `OrderedSet`/`OrderedMap` in prose renamed to the stubs'
  `SortedSet`/`SortedMap`.

Open questions raised by dreams:

- Legacy `directed_of_pairs` and friends in `std.graph` predate
  `Graph.directed`; remove.
