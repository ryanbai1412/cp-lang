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
- containers: `OrderedSet`/`OrderedMap` in prose renamed to the stubs'
  `SortedSet`/`SortedMap`.

Open questions raised by dreams:

- Legacy `directed_of_pairs` and friends in `std.graph` predate
  `Graph.directed`; remove.
