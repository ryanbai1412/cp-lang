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
- egoi23p4: `floyd_warshall(mut d, over:)` takes any path semiring (`Semiring(max, min)` is
  widest paths); `kruskal(es, by:)`; `[P]T` is `[A, B]T` for a product domain (checker gap).
- etsr23r3p3: orders flip with `desc` (decision 151): `kruskal_tree(es, w: e => desc(e.alt))`
  and `reach(v, limit: desc(p + 1))` replace negated altitudes.
- prelude: membership is now a rule, the lib chapters' tables (decision 150); `spec-stub.py
  --check` fails a prelude name no chapter shows. Six helpers left the prelude.
- etsr23r4p2: `phi` and `pow_mod` join the prelude; a fixed-length `[Slot]Z` node with a
  commuting shift action is plain `SegTree(op, a, act: Action.new(apply:))`, no new mechanism.
- yac7p3: `Functional` (cycles, `root_of`, `depth`, `cycle_len`) joins the prelude; it replaces a
  hand-written cycle-finding DFS.
- dmpg16g2: inclusion-exclusion over `Letter.subsets`; needs the enum-as-domain `E.at(i)` and the
  `D.subsets` domain, both decided and logged as checker gaps.
- ucrpc21h: product domains convert from tuples (`P.at((x, y))`, core 04), so a 3-D knapsack
  step is one `if let p = Split.at(from)`; `for (a, b, c) in Split.rev` is its reverse sweep.
- yac6p4: `Seq.find(p)` was missing from `std.seq` though chapter 7 uses it; added.
- noip09p4: `Subset<B>` (lib/search-dp.md's `B.subsets`) had prose but no stub; `std.subset` now
  declares it (one word, set algebra, `items`, `subsets`, `supersets`) and the prelude imports
  it. The duplicate `submasks` in `std.dp` is gone (`std.math` keeps the raw-`Int` one).
- noip14p6: `Str.parse<T>()` reads any `Read` type from a string by `read`'s rules, and a
  `Mod<P>` token is any-length decimal reduced mod `P` (core 10.2). Polynomial `eval` is one
  method over any ring, `coef.eval(x)`, in the prelude.
- yac6p3: no change; `Wire` plus `print`/`read` was enough.
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

- btoi20p3 (review): the dream translated C++ indices (`Int` bounds, `last[l - 1]`). Rewritten
  as one `street.ring.minimal(undoing(ParityDsu<V>(), add:), ok:)` pass: two pointers on
  `a.ring` (`RingSlice`, `ring.next/prev`) and `undoing`, the queue-undo `Track` over any
  `Undo` structure (decision 152). Lambdas gain `mut` parameters.
- noi10p2 (review): chord bounds became `Note.cuts` instead of raw prefix indices.
- cco21p4 (review): requirements and profits ride on the arcs (`a.r`, `a.p`, `a.e` from the
  default `EdgeArc`) of `g` and `g.transpose()`, instead of a filtered edge list and `into`
  buckets indexed back into it.
- noip18p3, coci21c4p4 (review): `Rooted` loses `w:`; lengths come from `Weighted` arcs, as in
  every graph algorithm, so a tree read with a `w` field is weighted with no option.
- ioi16p6, noi09p4, apio09p2, noi05p4 (review): `desc` instead of `-s.hi`, `with_prev`
  instead of `spans[i - 1]`, a comprehension instead of a push loop, `partition_point` and
  `inf` instead of `Int.MIN + 1` sentinels, `g.deg(m)`.
- ioi16p6 (review): diagonals are `domain Diag`; a point needs the range `min..=max`, nesting is dropped with the prefix-records idiom over right cuts, the overlap is `p.clip(s).len`, and the hull argument is `Int(s.r)` instead of `hi + 1`. No spec change.
- egoi23p7: `complement(g)`, a `Next` whose searches run in O(n + m) by scanning only
  unvisited vertices; "BFS in the complement graph" is the ordinary `bfs`/`components`.

Open questions raised by dreams:

- Legacy `directed_of_pairs` and friends in `std.graph` predate
  `Graph.directed`; remove.
- coci21c2p5, cco21p5, cheerio1s3, cco16p4, btoi15p3, noip99p1, btoi17p6, noi02p3 (review):
  raw `0..n` loops, sentinels and offset indices replaced by domains, cuts and open-ended
  ranges (`a.r..`, `pts[p.r..]`), `g.deg`, `by: desc`, `inf`, a `Head` enum instead of `[0..2]`.
  Forced: `track(state, add:, remove:)` now has `undoing`'s shape (closures take `mut state`,
  `ok` reads `t.state`), so a two-pointer state can be any structure; `maximal`/`minimal` on a
  `[D]T` return `[D]Slice` keyed by right end, like the ring forms (coci21c2p5 binary-searches
  the starts); `Rooted.up_arc(v)` returns the arc into `v` with its fields, replacing the
  parent-cost side array built from the edge list (noi02p3).
- Third idiom pass (review): command streams read as tagged enums (`etsr23r1p4`, `ioi01p1`,
  `sac22cc2p5`) instead of `read<1..=4>()` and an `if` chain; `mut` lambdas instead of nested `fn`
  workarounds (`btoi20p3`, `coci21c2p5`), so both now fail only on logged parser gaps; pins in
  `cmt21r2ztr` are a compressed `(v, x)` domain with adjacent pairs instead of a list plus a
  `HashMap` of ids; `desc` for rank keys (`ioi09p3`); range `clip` for the feasible interval
  (`dmopc21c9p1`); `partition_point` and `Prefix.fold` over an open cut range (`dmopc21c2p1`);
  cut keys of `scan` as counts (`ceoi17p2`, which also fixed an off-by-one in the bet count);
  enum and domain keys (`coci21c1p1` teams, `noip09p4` bands, `ioi19p1` seats). No spec change:
  every form was already decided.
- apio09p2 (review): interval ends are compressed into `domain X`, each interval is a
  `Range<X>`, the free gaps are cut ranges keyed by their left cut, and splitting a gap is
  `g.gap.l..span[i].l` / `span[i].r..g.gap.r` instead of `l - 1` / `r + 1` and `±inf` keys.
  No spec change.
- Blind dreams (decision 153): `dream.cp` is now the unconstrained ideal and `spec.cp` the
  checked solution; the 81 earlier files moved to `spec.cp`. First ten blind dreams (btoi20p3,
  cco21p4, ioi16p6, noi09p4, etsr23r3p3, coci21c2p4, noip18p3, apio09p2, noi10p2, noi05p4)
  forced: lambda parameter patterns, named product factors, `is_some_and`, `sum(of:)`, `mean`,
  `print(none:)`, `c.shift(r)`, `D.mask`, `Ring.step`, `outermost`, `g.only`, graph-shaped
  `max_closure`, `child_arcs`, `Lift.of`, `KruskalTree.top/up`, `Settle`, `SortedBag.take_first/
  drain`, `split_best`. Rejected: `ok` on the wrapped state, bare Dijkstra sources, `Mod(Bool)`.
  `spec.cp` idiom bugs the diff exposed (spec already allowed the dream's form): count-prefixed
  `List` fields in noi09p4, `IntervalMap` and `Lift.climb` in apio09p2, `g.next` in noi05p4,
  `Range` reads in btoi20p3.
