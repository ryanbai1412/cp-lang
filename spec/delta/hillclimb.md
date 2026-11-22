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
- containers: `OrderedSet`/`OrderedMap` in prose renamed to the stubs'
  `SortedSet`/`SortedMap`.

Open questions raised by dreams:

- Legacy `directed_of_pairs` and friends in `std.graph` predate
  `Graph.directed`; remove.
