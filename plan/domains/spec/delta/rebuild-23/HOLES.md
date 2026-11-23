# Rebuild 23 holes

Cold read of `spec/` and root `std/` only, then ten Codeforces problems from
contests 2011-2045. Written before any `cpl` or compiler run.

## Problems

| # | CF | Rating | Covers |
|---|----|--------|--------|
| 01 | 2045H Missing Separators | 2200 | string (Z-function / LCP), several output lines of varying length |
| 02 | 2014H Robin Hood Archery | 1900 | game theory, hashing (xor hash), range query, multitest |
| 03 | 2037F Ardent Flames | 2100 | range query with lazy updates (range add, max), binary search |
| 04 | 2027D2 The Endspeaker (Hard) | 2200 | range query (segment tree over a (min, ways) monoid), mod 1e9+7 |
| 05 | 2025E Card Game | 2200 | combinatorics mod 998244353 (factorials, binomials) |
| 06 | 2030D QED's Favorite Permutation | 1700 | multitest, point toggles |
| 07 | 2039D Shohag Loves GCD | 1700 | number theory (sieve, prime factors) |
| 08 | 2042D Recommendations | 1900 | greedy on intervals / sort by a key, segment tree |
| 09 | 2018C Tree Pruning | 1700 | multitest with large per-test state (sum n <= 5e5) |
| 10 | 2045M Mirror Maze | 1800 | grid simulation; output lines of varying length |

Selection note: no Codeforces problem tagged `games` and rated 1600-2200 exists
in contests 2011-2045 (API listing); 2014H is a two-player optimal-play game
(the Sheriff does not lose iff every value occurs an even number of times), so
it fills the game-theory slot. The second string problem is 2014H (hashing);
no other `strings`/`hashing` problem in range is rated 1600-2200 except 2034E
(constructive). None of the ten IDs appears under any `rebuild-*` folder name.

## Summary

| Class | Count | Inside M1 subset |
|-------|-------|------------------|
| silent | 24 | 24 |
| contradiction | 1 | 1 |
| missing-stdlib | 2 | 1 |
| found-late | 0 | 0 |
| total | 27 | 26 |

M1 status comes from `spec/SUBSET.md`: core sections R01-R10 and the contest
staple libraries (prefix sums, segment trees, strings, modular math, graphs and
trees, sequences, I/O) are *supported*; `std-algebra` and the `compress`
wrappers are *documented* only.

## Top five risks

1. **H1** Two-dimensional arrays over plain Int ranges (`[0..=n, 0..=n]0..=5000 = 0`).
   `04-indices` shows `[R, C]T` over domains and `[0..=n]Int` over Int ranges, never
   both; 01 depends on it for 75 MB of tables.
2. **H8 / H12** What index a `scan` result takes. `std/prefix.cp` keys it by cuts;
   04 indexes a scan of `[n]T` by an `Int`, 06 indexes a `max` scan by `i.r`.
3. **H18** `if let a = x, let b = y:` and `while let a = .., let b = ..:` with two
   bindings. `01-lexical` shows `if let v = .., cond` only.
4. **H27** `SegTree<J, Best>(Monoid(none, join), fill: none)` with a user struct and
   a user fn as the operation; the doc examples only show bare operators and
   `Run.join`-style statics.
5. **H2** Turning a `Str` slice into an owned `Str` (`Str(s[i..j])`) to store in a
   `List<Str>`; `02-values` documents `Str(cs)` from chars only.

## Holes

| ID | Class | M1 | Location | Guess / workaround | Problems |
|----|-------|----|----------|--------------------|----------|
| H1 | silent | in | core/04-indices (arrays), 02-values 2.8 | `[0..=n, 0..=n]T = 0` is a filled 2-D array indexed `a[i, j]` by Ints | 01 |
| H2 | silent | in | core/02-values 2.3 Strings | `Str(s[i..j])` copies a slice to a `Str` | 01 |
| H3 | silent | in | core/04b-keys, std/strings.cp `z_function` | `z_function(s[k..])[i]` (source key i > k) is lcp(s[k..], s[i..]); value at key k unused | 01 |
| H4 | missing-stdlib | in | std/prefix.cp `Prefix<D, T: Add + Sub>` | no xor prefix fold; `scan(^)` then `pre[r.r] ^ pre[l.l]` by hand | 02 |
| H5 | silent | in | std/prelude.cp, std/math.cp `splitmix64` | not in the prelude, so `use std.math.{splitmix64}`; `U64(x)`, `^` on `U64`, `== U64(0)` | 02 |
| H6 | silent | in | std/seq.cp `Compressed` | `.at(v)` documented; `.last_le(v)` and `.get(p)` guessed from `last_lt` | 03 |
| H7 | missing-stdlib | out (std-algebra documented) | std/algebra.cp `MinCount` | count is `Int`, cannot count ways mod p; wrote a `Best(cost, ways: Z)` struct + `join` | 04 |
| H8 | silent | in | std/prefix.cp `scan`, core/07-sequences | scan of `[n]T` (keys 0..n) indexed by an Int k = sum of first k | 04 |
| H9 | silent | in | core/02-values 2.6 Structs | struct literal `Best(cost: .., ways: ..)` with named fields | 04 |
| H10 | silent | in | core/03-numbers 5 Modular | fill `[0..=m]Z = Z(0)`, `Z(1)`; is `= 0` accepted? used explicit `Z(0)` | 05 |
| H11 | silent | in | std/mod.cp `Comb.choose` | `choose(n, k)` with k > n is 0; `Comb.new(limit)` covers n <= limit | 05 |
| H12 | silent | in | core/07-sequences scan, std/prefix.cp | `p.scan(max)[i.r]` is max of `p[..=i]` | 06 |
| H13 | silent | in | core/06-control 6.2 nested fns | a nested `fn` reads a top-level `var` mutated after the fn is declared | 06 |
| H14 | silent | in | core/04-indices domains, 07 `count` | `V.count(pred)` counts positions of a domain; `[i.prev!, i]` is a `List<V>` | 06 |
| H15 | silent | in | core/10-io 10.1, 06 6.5 | top-level `return` below `@!multitest` ends only this test | 07 |
| H16 | silent | in | std/math.cp `Sieve.factor` | returns `List<(Int, Int)>`; element `.1` is the exponent | 07 |
| H17 | silent | in | core/02-values scalars, std/segtree.cp | `Int.max` exists; `st.fold(c..)` with an open range from a position | 08 |
| H18 | silent | in | core/01-lexical 1.7 Control flow | several `let` bindings in one `if let` / `elif let` / `while let` header | 08, 10 |
| H19 | silent | in | core/02-values 2.5 records, 08.4 rows | a fn param `[U](l: Int, r: Int)` accepts both the read array and a `[U] i => (l: .., r: ..)` table | 08 |
| H20 | silent | in | std/seq.cp `tally` -> `HashMap` | `same[(l, r)]` indexes by a tuple key | 08 |
| H21 | silent | in | std/tree.cp `Rooted.up` | `t.up(max, finish: (s, v) => max(s, t.depth(v)))`: identity of `max` is the far end, the lambda may capture `t` | 09 |
| H22 | silent | in | core/04-indices grids, 02 2.3 | 2-D table literal `[Row, Col] (i, j) => e`; `rows[i][k]` is a `Char` | 10 |
| H23 | silent | in | core/01-lexical 1.6 Statements | `var (i, j, di, dj) = (..)` and tuple assignment `(di, dj) = if ..: (..) else: (..)` | 10 |
| H24 | contradiction | in | core/02-values `Str of "..."`, 01-lexical escapes | a charset with a backslash (`Str of "./\\"`) unclear; read a plain `Str` instead | 10 |
| H25 | silent | in | core/07-sequences, 10-io printing | `[..].max()!` over tuples; `print(words.rev, sep: "\n")` prints a reversed view | 01 |
| H26 | silent | in | core/01-lexical `max=` | `best[t] max= (c, k)` into a `?(Int, Int)` cell: None takes the value | 01 |
| H27 | silent | in | std/segtree.cp `SegTree.new`, std/algebra.cp `Monoid` | user struct + user fn as `Monoid(id, op)`; `st.fold(J(lo)..j)` mixing a converted and a loop position | 04 |

## Per-problem mapping

| File | Holes |
|------|-------|
| 01-2045H | H1 H2 H3 H25 H26 |
| 02-2014H | H4 H5 |
| 03-2037F | H6 |
| 04-2027D2 | H7 H8 H9 H27 |
| 05-2025E | H10 H11 |
| 06-2030D | H12 H13 H14 |
| 07-2039D | H15 H16 |
| 08-2042D | H17 H18 H19 H20 |
| 09-2018C | H21 |
| 10-2045M | H18 H22 H23 H24 |

## Revision tests

| File | Change | Lines changed | Chunks | Hesitation | Duplicated fact | Unrelated rewrite |
|------|--------|---------------|--------|------------|-----------------|-------------------|
| 03-2037F-rev | also print the leftmost p that reaches k (range query: return a position, not a yes/no) | 15 diff lines (5 hunks; 2 are the header comment and the fn rename) | 2 real (`enough` -> `spot` returning `?Int`; the final print) | yes: how to turn the tree's first position with value >= k into a position. `right_while` returns a `Cut<D>`, and the cut-to-position step is not spelled out, so used a linear scan over `P` (O(n log n), fine here) | the binary search now calls `spot(q)` twice (once inside `first_true`, once to print) | none |
| 09-2018C-rev | also print the smallest optimal leaf depth d (end position instead of value) | 9 diff lines (3 hunks; 1 is the comment) | 1 real (the final loop) | small: `max=` has no argmax form, so it became `if kept > best:` with two assignments | `best` and `at` updated together by hand | none |
