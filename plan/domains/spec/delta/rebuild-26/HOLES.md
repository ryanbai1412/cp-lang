# Cold-reader rebuild round 26

Written from `spec/` and root `std/` only, before any build or run.

## Problems

| File | Codeforces | Rating | Coverage |
|---|---|---|---|
| 01-2000E.cp | 2000E Photoshoot for Gorillas | 1400 | prefix sums over a 2-D grid (`Prefix(a)` on `[R, C]Int`, `fold(rows, cols)`) |
| 02-2032C.cp | 2032C Trinity | 1400 | binary search on sorted data, lower bound (`partition_point`) |
| 03-2020D.cp | 2020D Connect the Dots | 1800 | DSU (`Dsu<V>`) + per-step difference arrays (union of progressions) |
| 04-2036F.cp | 2036F XORificator 3000 | 1900 | bit manipulation / xor |
| 05-2004D.cp | 2004D Colored Portals | 1600 | map keyed by strings (`group(by:)` -> `Map<Str, List<City>>`) + lower/upper neighbour by binary search |
| 06-2005C.cp | 2005C Lazy Narek | 1800 | DP over subsequences (of the strings) |
| 07-2041D.cp | 2041D Drunken Maze | 1700 | BFS on a grid with walls (`bfs` over a product-domain state) |
| 08-2039D.cp | 2039D Shohag Loves GCD | 1700 | number theory with a sieve (`Sieve`, `spf`) |
| 09-2042D.cp | 2042D Recommendations | 1900 | sorting with a tie-break (`by: i => (l, desc(r))`), set keyed by pairs (`tally` on `(l, r)`), `SortedSet` successor |
| 10-2034D.cp | 2034D Darius' Wisdom | 1600 | unusual output: a count then a list of pairs, one per line |

None of the ten IDs appears as a folder name under `plan/domains/spec/delta/rebuild-*/` (checked with `ls`).

## Summary

| Class | Count | Inside Milestone 1 subset |
|---|---|---|
| silent | 11 | 11 |
| contradiction | 1 | 1 |
| missing-stdlib | 2 | 2 |
| found-late | 0 | 0 |
| **total** | **14** | **14** |

"Inside" means the section or library the hole sits in is marked `supported` in `spec/SUBSET.md`
(R02.4, R04.2, R06.4, R07.x, R10.3, std-bag, std-dsu, std-graph, std-grid, std-hashmap, std-math).

## Top five risks

1. **H01** n-ary product domain `(R, C, Way, Run)` as a BFS state space: only 2-factor products are shown (07).
2. **H02** no documented ordered-set lower bound: `first_ge` was superseded by `range(r)`, which is `todo` in `std/bag.cp` (09).
3. **H03** `best min= x` where `x: ?Int` (07): the None-identity rule is stated for an optional target only.
4. **H04** `Int(v)` on a position of an explicit domain `1..=n` taken to be its integer, not its ordinal (01, 03, 05).
5. **H05** Dir4 is a struct, not a domain, so a direction cannot be a product factor or array key (07).

## Holes

| ID | Class | M1 | Location | What I guessed | Problems |
|---|---|---|---|---|---|
| H01 | silent | yes (R04.2, std-graph) | `spec/core/04-indices.md` R04.2.9 (product domains, only `(A, B)` shown); `std/graph.cp` `bfs` | `domain State = (R, C, Way, Run)` is legal, a tuple of factor positions is a `State` position, and `bfs` allocates `[State]?Int` | 07 |
| H02 | contradiction | yes (std-bag) | `spec/lib/containers.md` (sorted lookup `first_ge`/`first_gt` vs "superseded by decision 103 ... `range(r)` views"); `std/bag.cp` `SortedSet.range` is `todo`; builtin `Set<T>` documents no bound query | `SortedSet<Int>.first_true(x => x >= v)` is the successor query | 09 |
| H03 | silent | yes (R02.4) | `spec/core/02-values-types.md` optionals, `min=`/`max=` | `?Int min= ?Int` keeps the smaller present value, None on both sides is the identity | 07 |
| H04 | silent | yes (R04.2) | `spec/core/04-indices.md` (`Int(v)` vs `v.ord`) | `Int(v)` of `V = 1..=n` is `1..n` and of `0..n` is `0..n-1` | 01, 03, 05, 07 |
| H05 | missing-stdlib | yes (std-grid) | `std/grid.cp` `Dir4(k: Int)` is a struct | a separate `domain Way = 0..4` and `p.step(Dir4(Int(w)))` | 07 |
| H06 | missing-stdlib | yes (std-math) | `std/math.cp` `Sieve`: `spf`, `factor`, `divisors`; no big-omega table, `sieve_multiplicative` doesn't fit an additive function | hand loop `omega[x] = omega[x / spf[x]] + 1` | 08 |
| H07 | silent | yes (std-math) | `spec/core/10-io.md` R10.1 writes `Sieve(1e6)`; `std/math.cp` declares `Sieve.new(n)` | struct name call forwards to `new` | 08 |
| H08 | silent | yes (R06.4, std-dsu, std-bag) | R06.4.8 (dropping a non-unit result is a lint); `Dsu.unite -> Bool`, `SortedSet.add -> Bool` not marked `@droppable` | the drop is a warning only, not an error | 03, 09 |
| H09 | silent | yes (R07.5) | for-pattern destructuring of read records `for (a, d, k) in ops` and `for (i, s) in xs.items` | positional tuple patterns bind record fields in order | 03, 05, 09 |
| H10 | silent | yes (R02.3, R07) | `spec/lib/strings.md`: no `Str.contains(Char)`; whether `Str` is a `Seq<Char>` for `.any` | `s.any(ch => ...)` walks the characters | 05, 06 |
| H11 | silent | yes (R07.11) | R07.11.3 `group(by:)`: order of each group's list unspecified | each list is in walk order, so already sorted by city | 05 |
| H12 | silent | yes (R07.9, R07.12) | `partition_point` on a `List<City>` returns a cut; type for a `List` (Int) vs `Cut<D>` | returns an `Int` index usable as `spots[j]` and in `max(0, cut - 1)` | 02, 05 |
| H13 | silent | yes (R10.3) | R10.3.1: an empty sequence prints an empty line | `print(shown, sep: "\n")` with zero moves prints a blank line; judge is whitespace-tolerant | 10 |
| H14 | silent | yes (R06.2) | top-level `fn` reading a top-level `let` (`moves` reads `maze`) | top-level fns capture earlier top-level bindings | 07 |

## Per-problem mapping

| File | Holes |
|---|---|
| 01-2000E | H04 |
| 02-2032C | H12 |
| 03-2020D | H04, H08, H09 |
| 04-2036F | none |
| 05-2004D | H04, H09, H10, H11, H12 |
| 06-2005C | H10 |
| 07-2041D | H01, H03, H04, H05, H14 |
| 08-2039D | H06, H07 |
| 09-2042D | H02, H08, H09 |
| 10-2034D | H13 |

## Revision test

| Rev file | Requirement change | Chunks / lines changed | Hesitation | Duplicated fact | Unrelated rewrite |
|---|---|---|---|---|---|
| 06-2005C-rev.cp (DP) | the target word is read per test instead of fixed `"narek"`; the DP's state dimension grows from 5 to `word.len` | 3 code chunks, 4 code lines (+1 comment): move `word` below the divider as `read<Str>()`, `domain Got = 0..word.len`, `% word.len` | whether `domain Got = 0..word.len` is legal below `@!multitest` (R10.1.1 shows `.count`, I used `Str.len`) | yes: the word length `5` was written twice (`0..5` and `% 5`); should have been `Got.count` | none |
| 05-2004D-rev.cp (other) | city `i` stands at coordinate `p_i` (increasing, up to 1e9) instead of at `i`; cost is `abs(p_x - p_y)` | 2 code chunks, 3 code lines (+1 comment): one new read, two cost expressions | whether `pos[x] - pos[z]` on `1..=1e9` values widens to a signed `Int` before `abs` | yes: the distance formula was written in two places (shared-colour branch and via-city branch) | none; the binary search is unchanged because coordinates increase with the city key |

## After running

`cargo build -p cpl-cli` (debug). For each file: `cpl check`, `cpl run` on every
statement sample (one multitest file per problem; three for 2041D; one invented sample per rev file), `cpl test` (samples in `NN-<cfid>.tests/`, not committed), `cpl build` +
`g++ -std=c++17 -O2` on the samples, 200 random small inputs compared between
`cpl run`, the native binary and an independent Python brute (all twelve files; for
2034D a validity checker applies the moves instead of a brute), and one maximum-size
input timed on the native binary.

| File | check | samples (`cpl run`) | `cpl test` | C++ samples | fuzz 200 (interp / C++ / brute) | max input (C++) |
|---|---|---|---|---|---|---|
| 01-2000E | ok | 1/1 | pass | 1/1 | 200/200 | 0.07 s |
| 02-2032C | ok | 1/1 | pass | 1/1 | 200/200 | 0.05 s |
| 03-2020D | ok | 1/1 | pass | 1/1 | 200/200 | 0.03 s |
| 04-2036F | ok | 1/1 | pass | 1/1 | 200/200 | 0.00 s |
| 05-2004D | ok | 1/1 | pass | 1/1 | 200/200 | **1.10 s** (B4) |
| 05-2004D-rev | ok | 1/1 | pass | 1/1 | 200/200 | 0.94 s (B4) |
| 06-2005C | ok | 1/1 | pass | 1/1 | 200/200 | 0.02 s |
| 06-2005C-rev | ok | 1/1 | pass | 1/1 | 200/200 | 0.02 s |
| 07-2041D | **E0203** (B1) | 3/3 after B1 workaround | fail (B1) | 3/3 after workaround | 200/200 after workaround | 0.24 s after workaround |
| 08-2039D | ok | 1/1 | pass | 1/1 | 200/200 | 0.00 s |
| 09-2042D | ok | 1/1 | pass | 1/1 | 200/200 | **74.3 s** (B3) |
| 10-2034D | ok | 1/1 valid (checker) | fail: output not unique (R1) | 1/1 valid | 200/200 valid | 0.01 s, **wrong output** (B2) |

No crash and no small-input mismatch. The one large-input mismatch (B2) was rerun under
`-fsanitize=address,undefined`: no report, output still wrong, so it is not UB.

### Failures

| ID | Class | File | Failure | Smallest reproduction |
|---|---|---|---|---|
| B1 | implementation bug (confirms H01 is legal-looking but broken) | 07 | `cpl check`: ``error[E0203]: `bfs` (`std.graph`) does not work on the types it was given here: `State` has no field `to` `` | below |
| B2 | implementation bug (interpreter and C++ disagree) | 10 | once the captured `print(xs, sep:)` output passes 64 KiB, the C++ build writes earlier buffered output (the move count) after the list instead of before it | below |
| B3 | implementation bug (hidden O(n) copy) | 09 | a lambda inside a `fn` that reads a `List` parameter captures it by value (`[xs_2 = xs]`), copying the list on every call: O(n^2) | below |
| B4 | implementation bug (hidden O(n) copy) | 05, 05-rev | `for (t, spots) in at` over a `group(by:)` map copies each `(key, List)` entry into a tuple on every walk: O(n * q) | below |
| R1 | reader omission | 10 | `cpl test` compares bytes; any valid move list is accepted by the judge. I wrote no checker; the output was validated by an external checker that replays the moves | n/a |

B1, `cpl check` fails; replacing `Cell` with `(R, C)` in the signature checks and prints `2`:

```
domain R = 0..3
domain C = 0..3
domain Cell = (R, C)
fn nb(p: Cell) -> yields Cell:
    for q in p.adj4: yield q
let ps = bfs(nb, from: [(R(0), C(0))])
print(ps.get((R(2), C(0))) ?? -1)
```

The workaround for 07 is one line: `fn moves(s: (R, C, Way, Run)) -> yields (R, C, Way, Run):`.
The committed file keeps the spec-faithful `State` spelling.

B2, `cpl run` prints `0` and then 9000 lines; the C++ build starts with `0 0` and
prints `6776 0` / `6776` where `6776 6776` belongs:

```
print(0)
print([(i, i) for i in 0..9000], sep: "\n")
```

Cause in the emitted C++: `print(xs, sep:)` swaps `cpl::out` into a `cpl::Capture`, but
`put_char` / `put_line` still `flush()` at 64 KiB and write the captured bytes to stdout
mid-capture.

B3, native time 0.06 s at n = 20000 and 0.24 s at 40000 (quadratic); the same loop at top
level is linear:

```
fn f(xs: List<Int>) -> Int:
    var s = SortedSet<Int>.new()
    var hits = 0
    for i in 0..xs.len:
        if s.first_true(x => x >= xs[i]) != None: hits += 1
    hits
let n: 1..=1e6 = read()
print(f(List.filled(n, 1)))
```

B4, native time 0.09 s at n = 20000 and 0.31 s at 40000 (quadratic):

```
let n: 1..=1e6 = read()
let at = (0..n).group(by: i => i % 2)
var t = 0
for _ in 0..n:
    for (k, v) in at: t += k
print(t)
```

### Holes after running

H01 (named n-ary product as BFS state) became B1. H11 (group order) and H12
(`partition_point` returns an `Int` on a `List`) held: 05 agrees with the brute. H13 held:
the zero-move case prints an empty line and the checker accepts it. Every other guess
checked and ran as guessed. No hole was found late; the four bugs are implementation bugs,
not spec holes, because the spec says each program means what I wrote and is linear.
