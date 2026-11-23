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
| found-late | 9 | 8 |
| **total** | **51** | **47** |

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
| 07-1929E-rev, after F04/F05 | same | 9 (1 comment; read split in 2; 2 new lines for `up_cost`; table type; table write; mask list; transition) | 4 (input, new per-vertex cost, mask table, transition) | `t.up_arc` was the obvious call and is unimplemented (F05); `if let` in a comprehension does not parse (F04) | yes: the `up_cost` loop restates which endpoint of each edge is the child, a fact `Rooted` already holds | none |
| 03-1915G-rev | print the shortest time to every city 2..n | 1 (+1 comment) | 1 (output) | none | none: "min over bikes" still written once | none |

## Found-late holes

Found by the runs below, after everything above was written. Each has the
smallest reproduction I found; the files under `rebuild-30/` now carry the
workaround, and the first commit on the branch keeps the cold version.

| ID | Location | M1 | Reproduction | What happens | Guess held? | Workaround |
|---|---|---|---|---|---|---|
| F01 | `std/seq.cp` `maximal` (lines 3160-3190), `lib/sequence-patterns.md` two pointers | yes | `let a = [3, 1, 4]`<br>`let g = a.maximal(Window(+), ok: w => (w.all ?? 0) <= 4)` | `cpl check`: E0303 inside `std.seq` (`pop` takes 0 arguments, but 1 were given); on a `[P]Int` it is E0301 (`self[r]` with an `Int` `r`). The documented two-pointer pattern cannot be compiled at all | H01 held on reading the std body (an empty piece `cuts.nth(r+1)..cuts.nth(r+1)`), H02 untestable | 01-1918D walks the two pointers by hand over a `Window(+)` |
| F02 | `lib/graphs-trees.md` line 148 (state-space example) | yes | `dijkstra((v: V, k: K) => adj[v].map(a => (to: (a.to, k), w: a.w)), from: [((V(0), K(0)), 0)])` | E0302 expected `Int`, found the lambda. The spec's own example `(v, k) => ...` is a two-parameter lambda; a state is one tuple parameter | H07 failed: storage was fine, the lambda shape was wrong | `((v, b)) => ...` (a tuple pattern parameter) works |
| F03 | `core/02-values-types.md` line 69 | yes | `let s = read<Str of "01?">()` | E0107 expected the end of the statement, found a name. The spec documents the type; the parser rejects it | H16 failed (spec-correct, not implemented) | `read<Str>()` |
| F04 | `core/01` comprehension grammar (not spelled out) | yes | `print([y for x in xs if let y = x, y > 1])` | E0103 expected `]` | H45-style guess (rev only) failed | `if cheapest[e] != None and ...` then `cheapest[e]!` |
| F05 | `lib/graphs-trees.md` line 257 vs `std/tree.cp` line 266 | yes | `t.up_arc(v)!.c` on a tree read with a `c` field | `std/tree.cp` has `up_arc` as `todo`; `cpl check` then says "an `Int` has no member `c`", not that the function is unimplemented | H42 failed | per-vertex `up_cost` filled from the edge list with `t.parent` |
| F06 | `core/02` conversions, `core/03` overflow; my code | yes | `let (s, k): (1..=1e14, 1..=2e5) = read()`<br>`print(s * k * Z(3).inv())` on `100000000000000 200000` | `cpl run` traps R0202 (`1e14 * 2e5` overflows `Int`); the `cpl build` + g++ binary prints a wrong `139218993` (right: `666666998`). The bounds prove the product can overflow, yet `cpl check` is silent, and debug and release disagree | H26 failed: `*` is left-associative, so `Int * Int` happens before any conversion to `Mod` | `Z(friends.map(.f).sum()) * k * p` |
| F07 | `core/07-sequences.md` R07.1.3 ("allocation happens only at `.collect()`, a comprehension or `List(...)`"); `cpl explain` | yes | `for d in 1..=3: print((0..n).map(i => i % d == 0).runs().len)` | the C++ fills a fresh `std::vector<char>` on every iteration (and a vector of run views); `cpl explain` lists only "slice view ... O(1)". A hidden heap allocation inside a loop that explain does not show | n/a (H30 held for meaning) | none needed at \(n = 5000\) |
| F08 | `cpl explain` "Views & copies" | no | `var q = Deque<Int>()`<br>`q.push(1)`<br>`print(q.len)` | explain prints two rows at `1:1`: "explicit copy -> std::vector<Char>, O(n)" and "reverse view -> ListView<Char>", in a program with no `Char`. They appear in 9 of the 12 files here (everything using `Deque` or `Window` through std) | n/a | ignore |
| F09 | `cpl run` (interpreter), `core/07` R07.1.3 | yes | `domain I = 1..=3000`<br>`let a: [I]Int = [I] i => Int(i)`<br>`print(a.runs().count())` | `cpl run` peaks at 540 MB (the same over `List(0..3000)`: 56 MB; native: 3 MB). Memory grows with runs times length, so 10-1923D under `cpl run` is killed at 31 GB already at n = 3e4; the native build takes 40 MB at n = 3e5 | n/a (H30, H31 held for meaning) | none in the file; the native build is what the judge runs |

### Did the other guesses hold?

Every hole not named in F01-F09 held: each file checks clean (after the
workarounds), passes its samples under `cpl run`, `cpl test` and the native
build, and agrees with an independent brute force on 200 random inputs, both
interpreted and native. In particular H03-H05 (hand-rolled deque work), H09-H11
(custom `Action` over a `Monoid`), H12 (`SortedSet` with `first_true` and
`last_true`), H13 (tagged enum reads, block match arms), H22 (`is_anc` order),
H24 (`Subset` union and tables), H26 apart from F06, H30 (`runs()` of a mapped
view), H33 (`last_true` on a position range) and H36 (`min` of optionals) held.

## Verification

All on the files as they now stand (`cargo build -p cpl-cli`, debug).

| File | check | samples (`cpl run`) | `cpl test` | build + g++ -O2 | fuzz (200) | max input, native | ASan/UBSan + `_GLIBCXX_DEBUG` | heap copies in loops (`explain`) |
|---|---|---|---|---|---|---|---|---|
| 01-1918D | ok (F01 cold) | ok | 1/1 | ok | 200/200 | 0.06 s (n = 1e5) | clean | none (F08 rows) |
| 02-1905D | ok | ok | 1/1 | ok | 200/200 | 0.03 s (n = 1e6) | clean | none (F08 rows) |
| 03-1915G | ok (F02 cold) | ok | 1/1 | ok | 200/200 | 0.05 s (n = m = 1000) | clean | none (F08 rows) |
| 03-1915G-rev | ok (F02 cold) | ok | 1/1 | ok | 200/200 | 0.05 s | clean | none (F08 rows) |
| 04-1924B | ok | ok | 1/1 | ok | 200/200 | 0.23 s (n = q = 3e5) | clean | none (F08 rows) |
| 05-1930D2 | ok (F03 cold) | ok | 1/1 | ok | 200/200 | 0.01 s (n = 1e6) | clean | none |
| 06-1934C | ok | ok (interactor accepts) | 1/1 | ok | 200/200 (interpreter with `@interactor`, native with a Python judge; same transcripts) | 0.26 s (t = 3000, Python judge) | clean (200 runs) | none |
| 07-1929E | ok | ok | 1/1 | ok | 200/200 | 0.03 s (n = 1e5, k = 20, 2^20 reachable sets) | clean | none (F08 rows) |
| 07-1929E-rev | ok (F04, F05 cold) | ok | 1/1 | ok | 200/200 | 0.02 s | clean | none (F08 rows) |
| 08-1925D | ok | ok | 1/1 | ok | 200/200 | 0.03 s (k = 2e5) | F06 cold, clean after | none (F08 rows) |
| 09-1948D | ok | ok | 1/1 | ok | 200/200 | 0.08 s (\|s\| = 5000) | clean | F07 (one vector per `d`) |
| 10-1923D | ok | ok | 1/1 | ok | 200/200 | 0.04 s (n = 3e5; all-equal 0.01 s) | clean | none (F08 rows) |

The brute forces are in Python, written from the statements and not from the
solutions (exhaustive blocking sets, every shift, state relaxation, direct
recomputation per query, every pattern string for `f`, every edge subset,
every pick sequence with exact fractions, every substring, and a BFS over
eating orders). Native and interpreter outputs agreed on every fuzz input, so
there was no mismatch to rerun under the sanitizers; each file still ran once
under them on its samples and maximum input (revisions and 06 on 20 and 200
random inputs). The interpreter on the maximum inputs agrees with the native outputs for
every file but 10-1923D (F09): 01 9.9 s, 02 7.3 s, 03 6.4 s, 03-rev 8.3 s,
04 40.9 s, 05 0.1 s, 07 4.2 s (wide 4.5 s), 07-rev 5.3 s, 08 1.6 s, 09 2.8 s,
10 all-equal 0.7 s.

Coverage note: the two-pointer slot (01-1918D) was written with
`maximal(Window(+), ...)` as asked, but F01 makes that call uncompilable, so
the file now walks the two pointers by hand over `Window(+)`.
