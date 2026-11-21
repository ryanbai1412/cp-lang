# Wave 2 blind critic: 30 ports, CP Lang 0.2.2

Reviewed at `720783e`. Every "Fixable" rewrite below is applied to the port in this commit and all 30 ports pass
`proto/ports/wave2/gate.sh <name> 100` afterwards. Every "Blocked" spelling was tried against the release `cpl`;
the error quoted is the exact one it printed. Line numbers refer to the files as they are after this commit.

Totals: **10 fixable** (F1..F10, in 10 ports), **33 blocked** (B1..B33, in 18 ports), **3 correctness doubts, none
with a failing input** (D1..D3). Ten ports came out clean: ccc18s4, cco18p1, dpk, dpu, dpx, ioi04p4, ioi94p1,
noip01p2, noip06p1, noip18p2.

Recurring blockers worth fixing once, in the order they hurt most:

- `Mod<P>` is not a number to the helpers: no `sum(...)` (B9, B12, B15, B16, B19, B33), no `prefix` (B6, B17), no
  `Z(bool)` (B13), no `**` on a square matrix (B14). Every modular DP in the wave (dpm, dpo, dpr, dpt, dpy, yac1p1)
  pays an accumulator `var` plus a loop for what the cheat sheet writes as one expression.
- Ranges with a missing end, and ranges over a domain: `..i` and `i + 1..` have no `for` support (B20, B23, B24),
  and `l + 1..=I.last` over a domain is a range of `?I` (B20). This is what keeps ioi00p1 and noip04p3 on plain
  `0..n` instead of a `domain`.
- A neighbour the guard already proves in range still needs `!`: `(l + 1)!` after `l == r` returned, `(m + 1)!`
  for `m in l..r` (B5, B7). The suggested `.r` boundary does not help (`l.r` is an `I.cuts`).
- `Graph` ergonomics: the edge list must be a named `domain E` (B10, B18, B26, B30), neighbours are
  `for (v, _) in g.out(u)` rather than `for v in g(u)` (B27, B31), `bfs` needs a `use` (B1, B29) and rejects a
  plain adjacency array with an error pointing at line 1 of the user's file (B2).
- `read()` has gaps: no `Bool` (B8), no `?Item` for a "0 means none" field (B25), no bare `read()` in an
  expression (B22).
- Inline one-based dimensions `[1..=n]T` are rejected, and the named-domain way around it loses arithmetic on the
  index (B3, B4).
- `.items` is missing on arrays, `List` and `Str` (B28); `+=` is missing on `List` (B32); `{x:X}` is not a format
  (B21).

What the author's README listed and the critic could not confirm:

- noip14p5 `print(x, none: "-1")`: `print(dist[t] ?? -1)` is accepted and shorter (F9).
- noip14p2 `by: x => -x` because `by: desc` is missing: `desc` works after `use std.algebra.{desc}`, and
  `sorted(...).rev` needs neither (F8).
- dpy and noip04p1 index loops forced by the missing `.items`: neither port needs an index at all (F4, F7).
- "`[1..=n]T` is rejected": true for an inline dimension only. `domain I = 1..=n` with `[I]T` works (dpl, dpn and
  ccc20s2's `[Num]` already use it); the real blocker is B3.

Everything else in the README's list was reproduced and is quoted below.

## Correctness doubts

No port failed. Beyond the 100-case gates, every port was run on hand-written edge inputs (n = 1, empty graphs,
all-equal values, extreme constraints) and on one maximum-size input, emitted C++ against the original; all
outputs matched.

- **D1** dmopc21c5p1 lines 1-2: the header comment said neighbours must not *differ* by a prime. The construction
  (odds, then 5 and 4, then evens) puts 1 next to 3, so that reading makes the port look wrong. The construction
  and the `-1` answers for n = 2..=4 only fit a forbidden prime *sum*. Comment corrected; the code was right.
- **D2** ioi04p4 at the maximum 600 x 600 plate: `cpl run` did not finish in 60 s, and the emitted C++ takes
  0.88 s against the original's 0.23 s (3.8x), because `@memo` over `(1..=w, 1..=h)` recurses where the original
  fills a table. Correct, but past the 1.5x-2x worst-case budget and one slow judge away from a TLE.
- **D3** The same `@memo` overhead at maximum size in emitted C++, `-O2`: dpn 0.08 s vs 0.01 s, ccc18s4 0.30 s vs
  0.10 s, dpo 0.36 s vs 0.14 s, dpl 0.15 s vs 0.06 s, dpu 0.13 s vs 0.06 s. All answers match; dpl's depth-3000
  recursion fits an 8 MB stack. The other 24 ports are within 1.5x or under 0.1 s.

## ccc20s2

Blocked:
- **B1** line 5: `use std.graph.{bfs}` should not be needed. Without it: `error[E0201]: no `bfs` in scope` /
  `help: `bfs` is in the standard library: add `use std.graph.{bfs}``.
- **B2** line 14: wanted `bfs(leads, from: [room[0, 0]])` instead of the `x => leads[x]` wrapper.
  `error[E0203]: a `[V]List<V>` has no method `out` that `cpl run` knows`, reported at line 1 column 1 of the
  user's file (the `use` line), not at the call.
- **B3** lines 9 and 13: wanted `let room: [1..=rows, 1..=cols]Num` and `leads[Num(r * c)]` instead of `(r + 1) *
  (c + 1)`. `error[E0501]: `cpl run` and `cpl build` can't handle array dimensions that don't start at 0 yet`.
  With named `domain Row = 1..=rows` / `domain Col = 1..=cols` the array is accepted but the product is not:
  `error[E0309]: `*` can't be applied to `Row` and `Col``.

## ccc21s2

Blocked:
- **B4** lines 6-9: wanted `var row_odd: [1..=rows]Bool` so that `row_odd[at]` needs no `- 1`. Same E0501 as B3.
  A named domain would need `Row(at)` / `Col(at)` at the use, which is no shorter than `at - 1`.

## dmopc21c5p1

Fixable:
- **F1** lines 7-8: `[x for x in 1..=n if x % 2 == 1 and x != 5]` -> `[x for x in (1..=n).step(2) if x != 5]`
  (and `(2..=n).step(2)` for the evens). Also see D1.

## dpl

Blocked:
- **B5** line 10: `lead((l + 1)!, r)` and `lead(l, (r - 1)!)`; the `l == r` branch has already returned, so the
  `!` is noise. Without it: `error[E0301]: expected I, got ?I: `l + 1` may step outside I`. The boundary spelling
  `lead(l.r!, r)` gives `error[E0302]: `!` takes the value out of an optional, but this is a `I.cuts``.

## dpm

Blocked:
- **B6** lines 12-16: the hand-slid `window` should be `let pre = prefix(ways)` and
  `next[c] = pre.sum(c - cap..=c)`. `error[E0302]: `prefix` takes a `[D]Int` or a `[R, C]Int`, found
  `[Int]Mod_1000000007_2``.

## dpn

Blocked:
- **B7** line 10: `cost((m + 1)!, r) for m in l..r`; `m < r` is given by the range. Same E0301 as B5.

## dpo

Fixable:
- **F2** line 15: `Man.from_idx(s.len - 1)!` -> `Man(s.len - 1)`.

Blocked:
- **B8** lines 8 and 16: wanted `let ok: [Man, Woman]Bool = read()` and `for w in s if ok[man, w]` instead of
  `0..=1` and `== 1`. `error[E0501]: `read()` has no input format for a `Bool``.
- **B9** line 16: wanted `ways[s] = sum(ways[s.without(w)] for w in s if ok[man, w])`.
  `error[E0309]: `sum` adds numbers, not `Mod_1000000007_2``.

## dpp

Blocked:
- **B10** lines 6-7: `domain E = 1..n` exists only to type the edge list; wanted `let edges: [0..n - 1](u: V, v: V)`.
  `error[E0201]: `cpl run` doesn't know the name `E` here`, reported at line 1 column 1 with the unrelated
  `help: a top-level `let` is visible only after the line that sets it`.

## dpr

Fixable:
- **F3** lines 22-23: `for i in V: for j in V: total += walks[i, j]` (three lines) -> `for ways in walks: total += ways`.

Blocked:
- **B11** lines 9-14: the six-line `mul` should be
  `fn mul(a: Mat, b: Mat) -> Mat = [V, V] (i, j) => sum(a[i, m] * b[m, j] for m in V)`.
  `error[E0302]: expected `[V, V]Mod_7_2`, found `[V, V]_`` followed by `error[E0309]: `sum` adds numbers, not `Mod_7_2``.
- **B12** lines 22-23: after F3, still wanted `print(sum(walks))`. Same E0309.
- **B13** line 16: wanted `[V, V] (i, j) => Z(i == j)` (or no cast at all). `error[E0302]: expected `Int`, found `Bool``.
  `Int(i == j)` is the stand-in.
- **B14** lines 9-21: the whole of `mul` plus the square-and-multiply loop is `edge ** k` in spirit; `**` is the
  cheat sheet's power operator. `error[E0309]: `**` can't be applied to `[V, V]Mod_7_2` and `Int``. Thirteen of
  the port's 21 code lines exist for this.

## dpt

Blocked:
- **B15** lines 23-24: wanted `print(sum(ways))`. `error[E0309]: `sum` adds numbers, not `Mod_1000000007_2``.
- **B16** line 24: the next-shortest form `sum(Int(x) for x in ways)` is also out:
  `error[E0302]: a `Mod_1000000007_2` can't be turned into a `Int``.
- **B17** lines 13-21: both running sums are `prefix(ways)` range queries. Same E0302 as B6.

## dpv

Fixable:
- **F4** line 28: `print([down[v] * up[v] for v in V], sep: "\n")` -> `for v in V: print(down[v] * up[v])`.

Blocked:
- **B18** lines 6-7: named `domain E` for the edge list, as B10.

## dpy

Fixable:
- **F5** lines 14-19: the index loop over `stops[i]` / `clean[j]` (8 lines, two parallel lists) -> one list of
  `(at: Cell, ways: Z)` walked directly: `for (at, before) in clean if at.c <= cell.c: ways -= before * paths(at, cell)`.
  No index and no `.items` needed.

Blocked:
- **B19** lines 16-17: wanted `let ways = paths((1, 1), cell) - sum(before * paths(at, cell) for (at, before) in clean if at.c <= cell.c)`.
  `error[E0309]: `sum` adds numbers, not `Mod_1000000007_2``.

## ioi00p1

Blocked:
- **B20** line 10: `for r in l + 1..n` on plain `0..n`; wanted `domain I = 0..n` and `for r in l + 1..`.
  `error[E0501]: `cpl run` and `cpl build` can't handle a range without an end in a `for` yet`, and with the end
  written out, `error[E0302]: a range runs over numbers or positions, not `?I``.

## noip00p1

Fixable:
- **F6** lines 7-8: `let d = left % base` plus an `if d < 10` digit/letter branch -> one lookup,
  `digits = "0123456789ABCDEFGHIJ"[left % base] + digits`.

Blocked:
- **B21** line 7: wanted `"{left % base:X}"`. `error[E0107]: `:X` is not a number format` /
  `help: a format is a width (`:5`, `:05`), digits after the point (`:.3`, `:8.3f`), `:x` or `:b``. `:x` exists
  but prints lower case and stops at base 16.

## noip04p1

Fixable:
- **F7** lines 5-9: reading all twelve budgets into `[0..12]` and indexing `budget[month - 1]` -> read inside the
  loop, `for month in 1..=12: let budget: 0..=350 = read()`. No array, no `- 1`.

Blocked:
- **B22** lines 7-8: wanted `cash += 300 - read()`. `error[E0302]: can't tell what to read here` /
  `help: `read()` reads what the type says: `let n: Int = read()``.

## noip04p3

Blocked:
- **B23** line 9: `for j in 0..i` should be `for j in ..i`.
  `error[E0501]: `cpl run` and `cpl build` can't handle a range without a start in a `for` yet`.
- **B24** line 10: `for j in i + 1..n` should be `for j in i + 1..`. `error[E0501]: ... a range without an end in a
  `for` yet`. Over a domain, `i.r..` inside the comprehension does not parse:
  `error[E0103]: expected `:` after the condition, found `)``.

## noip06p2

Blocked:
- **B25** lines 7, 10 and 13: `main: 0..=n` with `!= 0` / `== 0` and `Item(items[i].main)` is a `-1`-style
  sentinel; wanted `main: ?Item`. `error[E0501]: `read()` has no input format for a `?Item``, and
  `error[E0310]: `read` has no argument named `none``.
- **B28** lines 13-14: `for i in Item if ...: let it = items[i]` should be `for (i, it) in items.items if ...`.
  `error[E0203]: a `[I]Int` has no member `items` that `cpl run` knows` (same on `[Int]Int`, `List<Int>`, `Str`).
- **B32** line 18: `bundles = bundles + [...]` should be `bundles += [...]`.
  `error[E0309]: `+=` can't be applied to a `List<(cost: Int, worth: Int)>``.

## noip14p2

Fixable:
- **F8** line 15: `sorted([w[u] for (u, _) in tree.out(mid)], by: x => -x)` -> `sorted(w[u] for (u, _) in tree.out(mid)).rev`.

Blocked:
- **B26** lines 8-9: named `domain E` for the edge list, as B10.
- **B27** line 15: `for (u, _) in tree.out(mid)` should be `for u in tree(mid)`.
  `error[E0303]: `g` has 2 fields (start, arcs), but `arcs` is missing`.

## noip14p5

Fixable:
- **F9** line 15: `print(dist[t], none: "-1")` -> `print(dist[t] ?? -1)`.

Blocked:
- **B29** line 3: `use std.graph.{bfs}`, as B1.
- **B30** lines 7-8: named `domain E = 0..m`, as B10.
- **B31** lines 13-14: `(v, _) in roads.out(u)` twice, as B27.

## yac1p1

Fixable:
- **F10** line 10: `(lo_a[n - 1], lo_b[n - 1])` -> `(max(a), max(b))`.

Blocked:
- **B33** lines 15-16: wanted `Z(x) * y + sum(Z(lo_a[i]) * lo_b[i] for i in 0..n - 1)`.
  `error[E0309]: `sum` adds numbers, not `Mod_1000000007_2``. Pairing the two sorted arrays without an index is
  out too: `error[E0201]: no `zip` in scope`.
