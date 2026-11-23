# Rebuild round 30: holes

Cold rebuild against `spec/` and root `std/` only, at main `4d089cc93`. Ten
Codeforces problems from contests 1900-1949, rated 1700-2300, none named under
`plan/domains/spec/delta/` or in `research/rebuild-rounds/` (checked with `ls`
and `git grep -l`, file names only).

Before writing the solutions I saw a few lines of my own notes from earlier
sessions about the implementation (not the spec): that `cpl build` writes C++
source, and that a `SweepAdd` helper exists. Neither is used here.

Selection gap: no problem in 1900-1949 rated 1700-2300 that is not excluded
needs a prefix function, Z-function or hashing (the only one, 1943B, is
excluded). The string slot is 1930D2 (a string DP walked over a reversed
range); 1948D adds a second string problem over `runs`.

## Counts

| Class | Count | In M1 subset |
|---|---|---|
| silent | 38 | 36 |
| contradiction | 1 | 1 |
| missing-stdlib | 3 | 2 |
| found-late | 0 | 0 |
| **total** | **42** | **39** |

M1 means the hole sits in a family that `spec/SUBSET.md` lists as supported
(sequences, prefix sums, segment trees, graphs/trees, DP, modular math, I/O,
search). `SortedSet` (containers beyond heaps and DSU) is outside it.

## Top five risks

1. **H07** dijkstra over a tuple vertex `(v, b)`: `dijkstra` stores `[V]?W`,
   so the tuple must be a domain. I declared `domain S = (V, V)` and hoped the
   tuple indexes it.
2. **H09/H10** a hand-written `Action` with tag `?(Int, Int)` and a `Monoid`
   value passed as both the fold op and `over:`.
3. **H13** reading an enum with payload variants by `@tag`, and a block body
   after `=>` in a match arm.
4. **H01** what `maximal` returns at a right end where even the one-element
   piece fails `ok`.
5. **H26** implicit `Int` to `Mod` conversions in mixed arithmetic
   (`1 - p`, `Mod * Int`, `Int * Mod`).

## Holes

| ID | Class | Location | M1 | Guess |
|---|---|---|---|---|
| H01 | silent | `spec/lib/sequence-patterns.md` two pointers | yes | `maximal` gives an empty piece ending at `r` when `a[r]` alone fails `ok`; its `.first` is `None` |
| H02 | silent | `std/window.cp` `all`; sequence-patterns `ok` | yes | `ok` sees `w.all` as `?T` and is also called on an empty window, so it needs `?? 0` |
| H03 | missing-stdlib | `std/window.cp` | yes | `Window(min)` has no key of its front, so a left bound that moves by a computed amount needs a separate counter (`front`) beside the window |
| H04 | missing-stdlib | `std/window.cp`, `lib/ops-ranges.md` §8 | yes | the sum of all suffix minima of a sliding window (1905D) has no std form; hand-rolled with `Deque<(v, c)>` |
| H05 | contradiction | `spec/lib/containers.md` §1.5, §2.6 vs `lib/ops-ranges.md` §8, sequence-patterns | yes | containers still specifies `MonoDeque<T>`; ops-ranges says it became `Window(op)`. Used `Window` |
| H06 | silent | `std/deque.cp` `@get last(self, k = 0)` | no | a getter with a defaulted parameter reads as `q.last` |
| H07 | silent | `lib/graphs-trees.md` customizing, `std/graph.cp` `dijkstra` | yes | a lambda `(v, b) => ...` over tuple states needs `domain S = (V, V)` declared for array storage; otherwise hashed |
| H08 | silent | graphs-trees shortest paths; sequence-patterns `.present()` | yes | `p[(v, b)]` indexes `Paths` by tuple; `V.map(f)` walks a domain; `.present()` works on any `Seq<?T>` |
| H09 | silent | `std/algebra.cp` `Action.new` | yes | `id: None` with `F = ?(Int, Int)`; `over:` takes a `Monoid` value (`Monoid.of` of a monoid is itself) |
| H10 | silent | `lib/ops-ranges.md` §7, `std/segtree.cp` `new` | yes | `SegTree(m, init, act:)` accepts a `Monoid` value as the op |
| H11 | silent | ops-ranges §4, `std/algebra.cp` | yes | `compose(new, old)`; assign is `nf ?? of` |
| H12 | missing-stdlib | `std/bag.cp` `SortedSet` | no | no named predecessor/successor; used `last_true(h => h < x)` / `first_true(h => h > x)`, cost undocumented (guessed \(O(\log n)\)) |
| H13 | silent | `core/02` enums, `core/10` R10.2.3, `core/01` match | yes | `enum Query:` with `@tag(1) Add(x: P, v: ..)` lines reads by tag; a match arm may be `Add(x, v) =>` and an indented block |
| H14 | silent | `core/07` adapters | yes | `.adjacent` works on a generator (`harbours.each().adjacent`) |
| H15 | silent | `core/04` ranges | yes | `l.next!..r` is a half-open range of positions; empty when `l.next == r` |
| H16 | silent | `core/02` `Str of "01"`; `core/10` | yes | `read<Str of "01">()` validates; `s[i]` with an `Int` gives a `Char` |
| H17 | silent | `core/04` `rev`, `core/07` | yes | `for i in (0..n).rev` walks backwards; `[0..n + 3]Int` with a scalar fill, `.sum()` over it |
| H18 | silent | `core/10` R10.1.1-2, §10.4 | yes | `Wire()` above `@!multitest` is captured by `fn ask` below; `return` inside a nested `if` at top level ends the case |
| H19 | silent | `lib/random-stress.md` interactors | yes | `sol.send("{n} {m}")` sends one line; `sol.recv()` reads one token at the expected type, `?` as `Str` |
| H20 | silent | `core/01` loops | yes | no `loop` keyword found; `while true:` is the infinite loop |
| H21 | silent | `core/01` operators | yes | no chained comparison documented; wrote `x >= 1 and x <= n` |
| H22 | silent | graphs-trees `Rooted` | yes | `t.is_anc(a, b)` is "`a` is an ancestor of `b`", true when `a == b` |
| H23 | silent | `core/01` let-else | yes | `let d = dp[s] else: continue` |
| H24 | silent | `std/subset.cp` `impl BitOr`; search-dp mask DP | yes | `s \| e` is union; `[B.subsets]T` tables index by `Subset<B>`; `Subset<B>.full` for `domain B = 0..k` |
| H25 | silent | `core/10` R10.2.1 | yes | `read<[B](a: V, b: V)>()` reads `B.count` records over a declared domain |
| H26 | silent | `core/02` conversions, `lib/math.md` §2.2 | yes | `Z(x)` builds a `Mod`; `1 - p`, `Mod * Int`, `Int * Mod` convert the `Int` implicitly; `p ** c` with an `Int` exponent |
| H27 | silent | `core/03` literals | yes | `2e5` is an `Int` argument (`Comb<Z>(2e5)`) |
| H28 | silent | `core/01` lambdas | yes | `.f` is the field-access lambda (`map(.f)`) |
| H29 | silent | `core/01` precedence | yes | `1..=n / 2` parses as `1..=(n / 2)` |
| H30 | silent | sequence-patterns pieces | yes | `.runs()` with no `by:` groups equal values of a mapped `Bool` view; `r.first!` and `r.len` on a run |
| H31 | silent | `core/04` arrays | yes | `var start: [I]I = I.first!` fills with a position; `r.keys.first!` is a run's first key |
| H32 | silent | `core/07` scans | yes | `pre[j]` folds through position `j`; `pre[x.l]` folds before `x` |
| H33 | silent | `core/04` `last_true` | yes | `(I.first!..=l).last_true(ok)` on a position range returns `?I` |
| H34 | silent | `core/02` optionals | yes | `out[i] = 1` stores into a `?Int` slot as `Some(1)` |
| H35 | silent | `core/04` `V(x)` | yes | `I(n + 1 - Int(i))` makes a position, trapping outside |
| H36 | silent | `core/02` optionals, `core/10` R10.3.1 | yes | builtin `min` on two `?Int` treats `None` as the identity; `print(table, none: -1)` |
| H37 | silent | `core/02` tables | yes | a table literal `[I] i => ...` passes where `[I]Int` is expected |
| H38 | silent | `core/04` R04.4.30 | yes | `raw[Int(i) - 1]` indexes a counted `[n]` array by `Int`; `i == P.first!` compares positions |
| H39 | silent | `lib/search-dp.md` | yes | a named `fn` passes as `first_true`'s predicate |
| H40 | silent | `core/04` ranges | yes | `gap[j].first` on a `Range<P>` is `?P` |
| H41 | silent | `core/01` while-let | yes | `while let b = q.last, b.v > x:` binds then tests |
| H42 | silent | graphs-trees `Rooted` (rev) | yes | `t.up_arc(v)!.c` carries the read record's extra field `c` through `Graph.undirected` |

## Per-problem mapping

| File | Problem | Covers | Std used | Holes |
|---|---|---|---|---|
| 01-1918D | Blocking Elements | monotone deque 1, `maximal(Window(+))` two pointers, binary search | `Window(min)`, `Window(+)`, `maximal`, `first_true` | H01 H02 H03 H38 H39 H40 |
| 02-1905D | Cyclic MEX | monotone deque 2 (sliding suffix minima) | `Deque` | H04 H05 H06 H41 |
| 03-1915G | Bicycles | Dijkstra over (vertex, bike) | `dijkstra` | H07 H08 |
| 04-1924B | Space Harbour | lazy segment tree, range assign of a line | `SegTree(act:)`, `Action`, `Monoid`, `SortedSet` | H09 H10 H11 H12 H13 H14 H15 |
| 05-1930D2 | Sum over all Substrings (hard) | string, reversed view | `.rev` | H16 H17 |
| 06-1934C | Find a Mine | interactive | `Wire`, `@interactor` | H18 H19 H20 H21 |
| 07-1929E | Happy Tree Cutting | DP over subsets | `Rooted`, `Subset`, `B.subsets` | H22 H23 H24 H25 |
| 08-1925D | Good Trip | multitest, per-test arrays, shared precomputation | `Mod`, `Comb` | H26 H27 H28 |
| 09-1948D | Tandem Repeats? | indexes runs; string | `runs`, `find` | H29 H30 |
| 10-1923D | Slimes | indexes runs and slices, prefix sums | `runs`, `scan`, `last_true` | H31 H32 H33 H34 H35 H36 H37 |

## Revision test

| File | Change | Lines changed | Chunks | Hesitation | Duplicated fact | Unrelated rewrite |
|---|---|---|---|---|---|---|
| 07-1929E-rev | each edge has a colouring cost; minimise total cost | 6 (1 comment, read, mask table type, mask table write, mask list, transition) | 3 (input, mask table, transition) | how to reach the edge's cost from the child vertex (`t.up_arc(v)!.c`, H42); `if let` inside a comprehension filter | none | none |
| 03-1915G-rev | print the shortest time to every city 2..n | 1 (+1 comment) | 1 (output) | none | none: "min over bikes" still written once | none |

## Verification

Filled in after the runs below.
