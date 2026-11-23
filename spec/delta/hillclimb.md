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
- ioi09p6: `std/grid.cp` now exists (the prelude imported `std.grid` with no spec stub);
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
  drain`, `split_best`. Rejected: `ok` on the wrapped state (it sees the `Undoing`, so `u.state`), bare Dijkstra sources (`(s, 0)` keeps one spelling), `Mod(Bool)` (write `.int`), `by: desc(.alt)` field shorthand (`w: r => desc(r.alt)`), `kt.tree` (the tree methods live on `KruskalTree`), `SortedBag(gen)` (`SortedBag.of(List(...))`), a value as a type argument (`Settle<roads.keys, _>`; name the domain), and a non-optional `mean` (empty input has none). Checker gap, not spec: `peel` inferring `V` from the callback's parameter.
  `spec.cp` idiom bugs the diff exposed (spec already allowed the dream's form): count-prefixed
  `List` fields in noi09p4, `IntervalMap` and `Lift.climb` in apio09p2, `g.next` in noi05p4,
  `Range` reads in btoi20p3.
- Second blind batch (19 dreams, 19 `spec.cp` drafts, all checking).
  - Adopted, general:
    - `Seq.majority(eq:)` (ioi15p6): Boyer-Moore with only an equality oracle.
    - `Rooted.roots` (egoi22p3): the roots of a `forest`.
    - `PreferredPaths(t).access(v)` (sdoi17r1p2): the access half of a link-cut tree over a
      static tree, yielding each path switch. That is the shape of "colours painted along
      root paths".
  - Rejected:
    - `@journal` + `timeline` (cco20p5): an online segment tree over coordinates is simpler
      than offline deletion.
    - `PolyHash` (noi17p2): `RollingHash.hash_of` and `push` + `all` cover it.
    - A `_` field in a read record (sdoi17r2p4): name the column.
    - Calling an `LcaTable` as a function: write `lca.lca(a, b)`.
    - `cur.flip()` on a two-variant enum (noi06p3): the solution defines it if needed.
    - `T.counts(by:)`: `tally` then sort.
    - `T.max_by`/`min(of:)`: `max(by:)`, `map(...).min()`.
    - `Fenwick` over a product domain (noi08p5): `Fenwick2D` already exists.
    - `sorting_swaps` (ioi15p5): `perm_cycles`.
  - Dream bugs:
    - `in` used as a field name (ioi14p5io).
    - `.pow` for `**`, plus an O(cap) sum with cap up to 1e9 (shoi16p3): interpolate with
      `lagrange_at`.
    - Missing binary search on rounds (ioi15p5).
    - O(deg²) turn arcs (sdoi17r2p4): two preorder chains with one cross arc per gap.
    - `Fenwick(max)` with removals (cco20p5).
    - `where` used as a loop filter (ioi22p5).
  - Existing idioms:
    - Position arithmetic returns `?H`, so `(h + step).map(...)` (noi06p3).
    - `[edges.keys] e => (u:, v:)` builds keyed edges from grader lists (egoi22p3).
    - `extern fn` + `@grader` for interactive entry points.
  - ioi19p3: adopted `[D]T.valleys()`, the ranges strictly below both outside neighbours
    (monotone-stack walls). Rejected `h.row(r)` as a method (a comprehension view is enough).
  - ioi19p6: rejected `sweep_below` (one problem's shape); the port sweeps buildings with a
    `SortedBag` of active walks. `RangeInclusive` was a dream bug: `a..=b` is a `Range`.
  - btoi18p4: rejected `Seg.ring.nesting` and `coverage(...).min_over` (one problem's helpers).
    Open: the port unrolls the ring onto `1..=2n`, which is what a wrapping range over a domain
    (`RingRange<D>`, not tied to an array like `RingSlice`) would hide. The draft also rebuilds
    coverage per candidate pair (O(k·n)) and mishandles the pair that wraps, so it is not
    accepted yet.
- Third blind batch (10 dreams for problems that only had `spec.cp`: btoi17p2, cco21p5, ceoi18p1,
  noi15p4, apio23p1, noi07p1, shoi16p1, ioi09p6, noi02p1, noi15p1). Seven `spec.cp` files were
  rewritten from the dream. cco21p5 and noi15p1 dreams check as written. ioi09p6 keeps its old
  `spec.cp` because the dream's `@tag` cells are a logged parser gap.
  - Adopted:
    - `g.only(from: f)` (apio23p1): a stop vertex you can reach but not leave.
    - `?T.and_then` and `is_none_or` are written into core 2.4. The checker already had them.
  - Rejected:
    - Domain-typed `Mat<R, C>` (btoi17p2): `Mat` stays runtime-sized. A `[R, C]T` with a
      semiring product would be the general form, so it stays open.
    - `Heap.extend` (noi15p4): use `Heap.of(list)`.
    - `t.up((v, kids) => ...)` with the children's values as a list (shoi16p1): use a reverse
      preorder loop over `t.children`. `up` stays a monoid fold.
    - `grid4(P)` (ioi09p6): use `p.adj4`, which is a getter (the graphs chapter wrote `adj4()`;
      fixed).
    - Character-literal variant tags (`'T' Tree`): use `@tag("T")`.
  - Dream bugs:
    - `Some(..)` patterns (noi07p1): the pattern is `some`.
    - `PotentialDsu<V, W>` (noi02p1): the stub takes one parameter and `unite(a, b, w)`.
- Fourth blind batch (apio09p3, ccc02s4, ccc03s4, ccc04s5, ccc18s4, ccc19s2, ccc20s4, cco04p4,
  cco18p4, cco19p5). Two `spec.cp` files were rewritten. For the other eight, the existing port
  matched or beat the dream.
  - Adopted:
    - `RingRange<D>` (ccc20s4; also answers the open btoi18p4 question). It is the keys-only half
      of `RingSlice`, made with `a.ring.span(i, k)` or taken from `slice.range`.
      `Prefix.cyclic_sum` now takes one instead of a `Range<D>` that cannot wrap. The ccc20s4 port
      moved from `"ABC"[k]` indexes and an `s + s` prefix to `enum Who` and ring spans.
    - `floor_quotients(n, from:)` (ccc18s4): sums over `k >= 2` no longer clip the first block
      by hand.
  - Rejected:
    - `dag.up(order:, ...)` (apio09p3): `@memo` recursion on the DAG is the general form.
    - `people[c..].prefixes(max_len:)` (ccc02s4): use `(c.next..).take(m)`.
    - `argmin_convex` (cco18p4): `ternary_min` already exists.
    - `wire.ask`: a read on `Wire` flushes first.
    - Direct `out[f].absorb(mut out[i])` (cco19p5): two `mut` lends into one array are not
      proven disjoint. `take()` first.
  - Dream bugs:
    - `#` comments: the language uses `//`.
    - `@memo(hash)` was renamed to `@memo(map)`.
    - ccc20s4 counted each block in O(n) per rotation, O(n²) overall. It needs prefix counts.
    - cco04p4 guessed the input format.
    - ccc04s5 used a nonexistent `R.span` and is cubic per column, where the port's walk-outward
      generators are linear.
  - Open: a row of single-letter enum tokens with no separators (ccc20s4) reads as a `Str`, then
    `parse<Who>()` per character. An enum whose tags are all one character could read like
    `Char`.
- Fifth blind batch (cco16p4, cco20p4, cco21p2, cco24p1, ceoi17p2, ceoi19pp1, cheerio1s3,
  cmt21r2ztr, coci20c6p3, coci21c1p1). cco21p2, cco24p1 and cheerio1s3 match their ports almost
  token for token.
  - Spec change, decision 154: adjacent pairs are now `a.adjacent` and `a.pairs()` is every pair
    (O(n²) to walk). Three blind dreams and a gold dream read `pairs()` as every pair, and the
    ceoi19pp1 port was wrong because of it (it counted only adjacent gaps). Ports using the
    adjacent meaning were moved: ioi19p6, noip20p3 and cmt21r2ztr.
  - Rejected:
    - `read lower_triangle [V, V]` (cco20p4): read `[V.drop(1)]Str` rows.
    - `rotated(to:)`: use `rotated_left(i)`.
    - Per-vertex keyed `TwoSat` variables (cmt21r2ztr): the port's `compress` over `(v, x)` pins
      plus adjacency is cleaner.
    - The greedy loop (ceoi17p2): kept the port's prefix plus `first_true`. Both are linear-ish
      and the port says the invariant.
  - Idiom fix: coci21c1p1 used `map(..) ?? false` and a `var` where `is_some_and` and an `if`
    expression say it directly.
  - Dream bugs: `#` comments again, and a guessed `Counter(...)[k]`. `tally()` returns a
    `HashMap`, so a missing key needs `get(k) ?? 0`.
- Sixth blind batch (coci21c2p5, coci21c4p4, dmopc20c4p4, dmopc21c2p1, dmopc21c4p1, dmopc21c4p2,
  dmopc21c5p2, dmopc21c9p1, dmpg16g2, egoi23p4).
  - Spec change, decision 155: `maximal`/`minimal` take `anchor: Anchor.Start`, which keys the
    pieces by left end. The coci21c2p5 dream asked for `longest_from`. The port had recovered
    starts with a `partition_point` over the right-keyed results; it now reads
    `Lift.of(s => group[s].last!.next)`.
  - Spec idiom bugs fixed:
    - dmopc21c2p1 now ternary-searches the convex cost, as the dream did, instead of using
      prefix sums at breakpoints: 4 lines, not 8.
    - coci21c4p4 marks the chosen parks with `V.mask` instead of `tally()`.
    - egoi23p4 uses `kruskal(by: desc(.w))` instead of negating weights. It drops the
      product-domain tuple index (a parser gap) for plain `[u, v]` loops, and it now checks.
  - Rejected:
    - `Polygon(..).contains(p, boundary:)` (dmopc21c4p1): `locate(poly, p) != Outside` is the
      one point-in-polygon spelling and already says what happens on the boundary.
    - `read lower_triangle` again (egoi23p4), for the same reason as cco20p4.
    - `g.closure(semiring)` and `g.bottleneck(semiring)`: `floyd_warshall(mut d, over:)` covers both.
    - `R.group(pairs)` into `[R]List` (dmpg16g2): `group(by:)` plus pushes is fine at this frequency.
  - The dream was worse than the port:
    - dmopc21c9p1: the port's `ok.clip(..)` over a range of feasible x₁ beats the dream's
      `scan` over sign records.
    - dmpg16g2: the port starts every column blocked and frees the clear ones, which beats the
      dream's run-by-run initialization.
  - Dream bugs:
    - dmopc20c4p4 printed only the final cost; the statement wants one per prefix.
    - coci21c4p4 tested `a.to in open` on a `List` (O(k) per arc) and `v not in chosen`
      (O(nk)).
    - dmopc21c4p2 used a hashed `Set` for dense keys up to 5e6.
    - coci21c2p5 wrote `.end.r` where it meant the next position.
    - dmpg16g2 indexed a list by a subset.
  - Parser gaps, all already logged: `(mut t, s) =>` lambdas (coci21c2p5), and `Letter.subsets`
    / `Letter.at` on enum domains (dmpg16g2).
- Seventh blind batch (ioi01p1, ioi05p2, ioi09p3, ioi18p1, ioi19p1, ioi96p1, egoi23p7, noi14p5,
  noip09p4, noip10p1). Five ports (ioi09p3, ioi19p1, noip09p4, noip10p1, ioi18p1) already said
  what their dreams said.
  - Spec change, decision 156: product domains walk row-major and `P.nth(k)` decodes a flat
    cell number. The noi14p5 port now swaps on `Grid.nth(..)` and destructures `at[v]`; it no
    longer divides by `m`.
  - Spec idiom bug: ioi96p1 rebuilt sub-ranges with `r.first!.next!..=r.last!` and failed
    `cpl check`. It now uses `r.drop(1)` and `r.drop_last(1)`, and checks.
  - Rejected:
    - `Fenwick(+, [X, X]0)` over a product domain (ioi01p1). It needs a rectangle-range type that
      product domains don't have yet, and `Fenwick2D.sum(rows, cols)` reads the same. Revisit
      when product-domain slicing lands.
    - `read stream Cmd until End`: `while true: match read<Cmd>()` with an end variant is one
      line longer and general.
    - `Rail.pieces(cuts:)` (ioi05p2): `compress(cuts)` already gives the pieces.
    - `?Rooted` from invalid parent arcs (egoi23p7): validating the input is problem logic.
  - Dream bugs:
    - egoi23p7 required a DFS forest of the complement to have one root. Only the last root
      started can reach everything; the port finds it and then BFSes from it.
    - ioi05p2 guarded the slope with `max(1, ..)`; the car can't stop on a non-rising piece.
  - Parser gaps, already logged: `@tag` variants (ioi01p1, ioi05p2).
- Blind batches eight to eleven (20 dreams: aac5p4, btoi15p3, btoi17p6, etsr23r1p4, etsr23r4p2,
  mccc4j1, mccc4j4, nccc8s3, noi02p3, noip14p6, noip99p1, sac22cc2p5, sac22cc3jp2, sac22cc3jp4,
  sac22cc4jp2, ucrpc21c, ucrpc21h, yac6p3, yac6p4, yac7p3). Process slip: for the first five
  I printed the existing `spec.cp` next to the C++ before drafting, so those dreams are not
  fully blind. The other fifteen were drafted before their ports were read. No spec changes.
  - Spec idiom bugs, fixed: mccc4j1 sums with `sum(of:)` instead of `map(..).sum()`. mccc4j4
    sorts by `desc(pts[t])` instead of `-pts[t]`.
  - Rejected:
    - `SegTree.apply_beats(r, settled, shift)` (etsr23r1p4): a general beats hook needs a "break"
      and a "tag" condition per operation. `BeatsSegTree` stays its own structure, as the
      ops-ranges table says, until a second beats operation shows the shape.
    - `a.last_index_of()` (nccc8s3): a `[V]` write in a loop over `a.items` is two lines.
    - `query(prompt, read)` and `.then` (yac6p3): a plain fn with `print` and `read` under
      `Wire()` already flushes before each read.
    - `f.tail_len`, `f.cycle_entry`, `tree_from(c).depths()` (yac7p3): `Functional` already has
      `depth`, `root_of`, `on_cycle` and `cycle_len`, which say the same thing.
    - `iterate(x, f)` and an `Exp` value type (etsr23r4p2): one problem's power-tower helper.
  - Dream bugs: ucrpc21h rebuilt the whole 301³ table per distance (O(n·300³) allocation);
    the port updates in place in reverse order. noi02p3 and btoi17p6 fold with `t.up`, the
    ports recurse with a named fn; both are fine.
  - Parser gaps, already logged: `@tag` variants (etsr23r1p4, mccc4j4, sac22cc2p5,
    sac22cc4jp2), loop `else` (ucrpc21c), product-domain tuple `at` (ucrpc21h).
  - Naming: dreams and ports mixed `Big`/`Win` with `some`/`Anchor.start`, and chapters 01
    and 02 disagreed. Decision 157 makes every variant `PascalCase` (`Some`, `None`,
    `Anchor.Start`, `Sign.Neg`); the spec, stubs, dreams and ports are swept.
- aac4p6 (last dream, 100 of 100): the corpus has no statement, so the dream follows the C++'s
  idea instead of rethinking it, and the port matches it. Dream slip: `.enumerate()` where
  `a.rev().take(m)` keys already serve. No spec change.
- btoi18p4 port reworked to the accepted algorithm. `covers` sorts the given arcs (each repeated a
  lap either way) and sweeps, O(s log s) instead of building an O(n) depth array per call, so
  the per-top checks cost O(m log m) in total. The port now precomputes which tops can share a
  direction (`alone`), handles k = 1 and k = 3, rejects early when all wires miss a segment,
  and shifts the pair's end by n when the pair wraps past the last top. Both the old port and
  the dream (`dist(b, t)` parity with c flipped) alternated from the wrong top; the shared pair
  now alternates from c, as in the C++. Checked with `cpl check` only: spec-only stubs don't run.
- Second look at the ten-more-dreams sample (Ryan asked why dmpg16g2 and egoi23p4 run about
  twice their dreams' length). Two earlier rejections were wrong: each removed a one-problem
  name but left the port paying for a missing general piece.
  - Spec change (decision 158): `a.buckets()` groups keys by a domain-valued value, so
    dmpg16g2's `R.group(pairs)` is `last.buckets()` and the five-line push loop goes.
    `preimages` folds in; `group(by:)` stays hash-keyed.
  - Spec change (decision 160): `d.closure(over:)`, the value form of `floyd_warshall`
    (`sorted` to `sort`). egoi23p4's clone-run-compare helper shrinks to two lines.
  - Spec change (decision 159): inline `(V, V)` product domains and tuple-unpacking lambdas,
    `(V, V).all((u, v) => ...)`.
  - Port fixes: dmpg16g2 sums with `sum(of:)`, filters subsets instead of `if` in a loop, and
    builds `FreeSpans` from defaults. egoi23p4 builds candidates with a comprehension over
    `V.pairs()`.
  - Still open: ragged reads (`read<[i in V][..i]Int>()`) for the triangle inputs of
    egoi23p4 and cco20p4, and reading `?Letter` straight from a `.`/letter grid.
- Full pass over the 100 (Ryan asked for one more look for spec changes). Ranked the ports by
  spec/dream length (median 1.13x) and read the 20 worst, and counted dream calls the stubs lack.
  - Spec change (decision 161): `read_all(until:)` for sentinel-ended command streams (ioi01p1,
    ioi05p2), `group(by:, of:)` (dmpg16g2, ioi19p3).
  - Decision 158 confirmed: ioi19p6's `group(...)` plus `.get(b) ?? List()` becomes
    `walks.map(w => w.span.first!).buckets()` and `starting[b]`.
  - Port idiom fixes: mccc4j4 and ioi05p2 `match` on tagged enums instead of `if`/`elif`;
    ioi19p3 uses the row and column views `h[r]` and `h[.., c]` instead of building tables.
  - Rejected again: `dominance_pairs` (ioi19p3 only; ioi19p6's sweep is a nearest-below query,
    not dominance), `Exp` (etsr23r4p2), `Perm` with a maintained inverse (ioi15p5 only).
  - Dream bugs: sdoi17r2p4 runs Dijkstra over all arc pairs at a vertex (O(deg²)); the port's
    prefix/suffix chains over trie order are the real algorithm. ioi15p5 skips the binary
    search on the round count.
  - Open: egoi23p7 is 2.9x its dream. `Rooted.from_parents` has no failure result for
    invalid parents, and finding the root of the complement graph's DFS forest needs a manual
    Enter/Exit depth count. Both need a closer design look.
  - Spec change (decision 162): `D.pieces(cuts:)` and `piece.covering(r)` (ioi05p2's dream);
    the port drops `compress` of `a`/`b + 1` and its hand-built lengths (32 to 28 lines).
  - Spec question, open: whether `x is pat` in a comprehension's `if` binds for later clauses
    (`[x for c in cmds if c is Set(a, b, _) for x in [a.l, b.r]]`). The checker says no; the
    ioi05p2 port uses `flat_map` instead.
- egoi23p7 port: bans now go through `tally` per top, and `blocked` is built from the ban
  counts, O(m2). The old `[B] k => B.filter(...)` was O(B²), too slow at 3e5 blocks. The root
  search is the last vertex a DFS finishes (3 lines, previously a 7-line depth count). The forest
  check is `Rooted.try_from_parents(up) else: return None` (decision 163), and blocks are the
  forest's roots, compressed. That replaces a component pass and a top-per-block table.
- Process notes from the session that were not yet written down:
  - The dreams and the spec have one author, so the dreams are not independent of the spec and
    probably understate friction. The fix is dreams and questions from an agent that reads only
    `spec/` (the "writable from the docs" test in AGENTS.md), with every disagreement treated
    as a spec hole.
  - The checker is laxer than the spec: it accepted inline `(V, V)` and tuple-unpacking lambdas
    before the spec defined them. "Passes `cpl check`" is not evidence that the spec says so.
    Conformance task: make the checker reject what the spec doesn't define, starting with the
    forms named here.
  - Edge weights: an edge with one weight names it `w`, the implicit weight; with several, use
    meaningful names and `.weighted(.len)`, so the reader sees which one an algorithm uses.
- `spec/delta/dream-comparisons.md`: ten blind dreams side by side with their `spec.cp`, with a
  per-line "vs spec" verdict for each.
- egoi23p4, cco20p4: `Pair<V>`, the unordered-pair domain (decision 164). egoi23p4's 9-line
  `read_triangle` becomes `read<[Pair<V>]...>().map(Some).symmetric(diag: None)`, and cco20p4
  reads `read<[Pair<V>]Char>()` and indexes `road[u, v]` instead of using `max`/`min` into rows.
- cco18p4, yac6p3: `wire.ask(query)` prints, flushes and reads the answer (decision 165), which
  both blind dreams wrote. Each query function goes from three lines to one. Chapter 10 also gained
  the `read`/`print` signatures and output shapes, which Ryan's I/O question showed were missing.
