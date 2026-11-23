# Round 1: indices and ranges

Everything else in the stdlib indexes by these, so they come first.

## The model in one paragraph

A **domain** is a finite, ordered index type. Its members are **positions**.
Between and around positions sit **cuts**: a domain of n positions has n + 1
cuts, `D.cuts`. A **range** is a contiguous run of positions, and its two
ends are cuts. Arrays are `[D]T`, indexed only by `D`, so a vertex never
indexes an edge array and a cut never indexes a position array.

<!-- spec-only: declaration tour over `n`, `xs`, `Row` and `Col` from the text -->
```python
domain V = 1..=n                       // positions 1..n
domain Pos = 0..n                      // positions 0..n-1; Pos.cuts is cuts 0..=n
let (domain C, comp) = compress(xs)    // fresh domain from a computation
domain Cell = (Row, Col)               // product: [Cell]T and [Row, Col]T are one 2-D array type
domain State = (comps: 0..=n, used: 0..l)  // named factors: s.comps, State.at((c, u)) -> ?State
enum Dir = N | E | S | W               // a fieldless enum is a domain too: [Dir]Int
```

## Domains

| Form | Meaning | Cost |
|---|---|---|
| `for v in V`, `V.rev` | positions in order or reverse order | O(1) per step (R04.2.1) |
| `V.count` | number of positions | O(1) (R04.2.2) |
| `V.find(f)`, `V.count(f)`, `V.filter(f)`, `V.min(by: f)` | a domain is a `Seq<V>` of its positions in walk order (product domains row-major), so every `Seq` method applies to it; `V.count` with no call is still the number of positions | as the `Seq` method (R04.2.20) |
| `V.first`, `V.last` | `?V`, `None` when `V` is empty; write `V.first!` when you know it isn't | O(1) (R04.2.3) |
| `V.range` | the whole range, same as `..` (`a.keys` for an array's) | O(1) (R04.2.4) |
| `V.cuts` | the domain of cuts (type `Cut<V>`): `V.cuts.first` is the start, `.last` the end | O(1) (R04.2.5) |
| `x in V` | is integer `x` a position | O(1) (R04.2.6) |
| `V(x)` | integer to position, trapping outside `V` | O(1) (R04.2.7) |
| `V.at(x)` | integer to position, `?V`: `None` outside `V` | O(1) (R04.2.8) |
| `P.nth(k)` | a product domain walks row-major (the last factor fastest), so `P.nth(k)` decodes a flat 0-based cell number and `for (r, c) in P` visits rows in order | O(1) (R04.2.9) |
| `P.at((x, y))`, `P((x, y))` | for a product domain `P = (A, B)`, a tuple of integers to a position: `None` (or a trap) unless every part lies in its factor, so a neighbour that may fall off is `if let q = P.at((r - 1, c))` | O(1) (R04.2.10) |
| `let (r, c) = p`, `p.0` | a position of a product domain is a tuple of factor positions: tuple patterns and `.0`/`.1` work on it, named factors too | O(1) (R04.2.10) |
| `v.after`, `v.before` | the pieces `v.r..` and `..v.l` of `v`'s domain, so `for j in v.after` walks the positions after `v` | O(1) (R04.2.19) |
| `a[p]`, `a[x, y]` | for `P = (A, B)`, `[P]T` and `[A, B]T` are the same type: a `p: P` indexes it whole, its parts index it one by one, and `[P] p => f(p)` builds it | O(1) (R04.2.11) |
| `Pair<V>` | the unordered pairs `{u, v}` of distinct positions, `V.count * (V.count - 1) / 2` of them, walked by `hi` then `lo` (lower-triangle order); `[Pair<V>]T` is packed, and `d[u, v] == d[v, u]` (decision 164) | O(1) (R04.2.12) |
| `V.wrap(x)` | integer to position, mod `V.count`: rings and cyclic shifts | O(1) (R04.2.13) |
| `if let V(j) = i - 10:` | the pattern form of `V.at`: matches only inside `V` | O(1) (R04.2.14) |
| `Int(v)` | position to its integer (for `1..=n`, that's 1..n) | free (R04.2.15) |
| `Int(c)` | cut to its integer: the number of positions before it, 0..=n whatever the base (`1..=n`: `v.l` is v-1, `v.r` is v; `0..n`: v, v+1) | free (R04.2.16) |
| `V.cuts(k)`, `V.cuts.at(k)` | integer to cut: trapping, or `?V.cuts` | O(1) (R04.2.17) |
| `5` where a `V` or `V.cuts` is expected | integer literal is that position or cut (integer domains; also fills `[d]V.cuts = 0`); outside a static bound it is a compile error | free (R04.2.18) |

**Pieces.** Coordinate compression for intervals: `let (domain P, piece) =
D.pieces(cuts: cs)` splits a range at the given cuts into the pieces between them,
with `piece[p]` the `Range<D>` each covers (so `piece[p].len` is its real length)
and `piece.covering(a..=b)` the `Range<P>` that tiles `a..=b`. Inputs are cuts,
pieces are ranges, so the `b + 1` and `- 1` of hand-rolled compression never
appear (decision 162).

## Positions and cuts move the same way

| | neighbour of the same kind | the other kind, either side |
|---|---|---|
| position `v` | `v.prev`, `v.next`: `?V` | `v.l`, `v.r`: its cuts (R04.3.1) |
| cut `c` | `c.prev`, `c.next`: `?V.cuts` | `c.l`, `c.r`: the positions beside it, `?V` (R04.3.2) |

`v.next(k)`, `v.prev(k)` step k at once, `?V` (cuts too). `c.shift(r)` is
the cuts at distances `r` after cut `c`, clipped to the domain and possibly empty
(chords of length `lo..=hi` from `c` end in `c.shift(lo..=hi)`). `D.mask(xs)` is the
`[D]Bool` marking the positions a sequence yields. `D.ring.step(x, d)` steps `d`
positions with wraparound. `.next` is
`.next(k = 1)`: only methods marked `@get` (which promises O(1)) read without
parentheses, and an `@get` method may take defaulted parameters. A plain
method always needs `()`, so nothing costly hides behind a bare name.

One rule: `.l`/`.r` switch kind (left or right side), `.prev`/`.next` keep it.
The receiver's kind is static, so a mix-up is a type error. `v.l..v.r` is
exactly `v`; a prefix DP reads `dp[v.r] = f(dp[v.l], a[v])`.

Arithmetic on a position gives an `Int` (`v + 1`); getting back into the
domain is explicit (`v.next`, `V(x)`). (R04.3.3) Arrays over an anonymous range
(`[0..=n]Int`) take any `Int`, checked, so ordinary DP tables stay plain.

## Ranges

### Two kinds of range, one type

`Range<D>` is a contiguous run of a domain `D`. `V.cuts` is a domain like
any other, so there are two kinds you meet in practice, and every method
below is generic over `D`: it means the same thing on both.

| | `Range<V>`: over positions | `Range<V.cuts>`: over cuts |
|---|---|---|
| written | `l..r`, `l..=r` with position ends | `j..i`, `j..=i` with cut ends (R04.4.1) |
| elements (`for`, `in`, `len`) | positions | cuts (R04.4.2) |
| typical use | a piece of the array: slice, fold, cost | split points, prefix lengths, answers that may be 0 or n |
| example | `a[l..=r]`, `seg.fold(l..r)` | `for j in ..i`, `for k in a..=b` |

**The rule: a range's elements are the kind of its ends.** Cuts in, cuts out:
`for k in a..=b` walks cuts a through b, `for j in ..i` walks the cuts before i.
A range whose start is past its end is empty, never a trap: `5..=3` and
`(5..=3).rev` walk nothing, so `for c in (p..=x).rev` needs no `p <= x` guard. (R04.4.29)

### One exception, in how a literal is typed: the piece

Cuts `j` and `i` bound a piece of the array: the positions between them. So
a half-open range literal with cut ends, written where a `Range<V>` is
expected, builds that `Range<V>` directly:

<!-- spec-only: fragment: `dp`, `cost` and `i` come from the text -->
```python
for j in ..i:                        // ..i: no Range<V> expected, so cuts
    dp[i] min= dp[j] + cost(j..i)    // cost takes Range<V>: j..i is the piece
pre.fold(j..i)   a[j..i]   seg.fold(j..i)       // same: these take Range<V>
```

This is **not a conversion and there is no implicit `Into`.** It decides what
the literal builds, exactly like `5` becoming a position where a `V` is
expected. Consequences:
- Only the literal `a..b` does it. A stored cut range never becomes a piece:
  `let s = j..i; cost(s)` is a type error whose fix is to write `cost(j..i)`. (R04.4.5)
- Only half-open: `j..=i` on cuts names no piece, so it's an error where a
  `Range<V>` is expected. (R04.4.6)
- It's safe because the two readings are the same set: cut `c` in `j..i`
  stands for the position `c.r`, and the cuts j..i-1 are exactly the left
  edges of the positions between `j` and `i`.

### Open-ended ranges

Leaving out an end means the edge of the other end's domain, so the domain
never has to be named and never has to be non-empty:

| written | means | elements |
|---|---|---|
| `..i` | `D.first..i`, where `i: D` | everything before `i` (R04.4.8) |
| `..=i` | `D.first..=i` | everything up to and including `i` (R04.4.9) |
| `i..` | `i..=D.last` | `i` and everything after it (R04.4.10) |
| `..` | all of the indexed dimension | only inside `[...]`: `grid[r, ..]` (R04.4.11) |

A literal end is a **position** of the indexed domain unless the other end
is a cut (then it is a cut, as in the piece rule above). On `domain V =
1..=n`, `a[..5]` is positions 1..=4 and `a[..=5]` is 1..=5; `a[..V.cuts(5)]`
writes the cut. Integer literals never mean a count of positions.

They follow the ends rule like any range: `for j in ..i` with a cut walks
cuts, and `for j in v.r..` walks cuts too. A half-open open-ended literal
with a cut end is a piece where a `Range<V>` is expected, so `a[..c]` and
`a[c..]` slice. For the positions after `v`, write `for j in v.next..` only
when `v.next` exists, or `for j in a[v.r..].keys`; see Open. `..i` needs no
`!`: `i` existing proves its domain is non-empty. Cost: O(1) to build, like
any range.

<!-- spec-only: fragment: `dp`, `cost`, `i` and `c` come from the text -->
```python
for j in ..i:                        // every cut before i
    dp[i] min= dp[j] + cost(j..i)
let best = a[..c].max()              // the prefix left of cut c
```

The language's implicit conversions are this closed list, none
user-extensible. Each applies wherever an expected type is known: a
binding or field with a declared type, an argument, a return value, an
assignment, and either operand of `==`, `!=` or a comparison whose other
operand fixes the type. Nothing else converts. (R04.4.22)

1. **`T` to `?T`.** `u != from` with `from: ?V` compares `Some(u)` with
   `from`. Free. (R04.4.23)
2. **A range type to a wider one** (`1..=5` to `Int`, `0..=9` to
   `-5..=100`). Free. Arithmetic on range types yields `Int` (chapter 3),
   so a sum of weights `1..=1e9` is an `Int`, and a path distance or
   `dist: [V]?W` built from one is `Int` unless declared narrower.
   An array's element type does not widen this way: `[V]1..=9` to
   `[V]Int` rewrites every cell, an O(n) cost a move must not hide
   (R05.2.2), so it is an error; write `xs.map(x => Int(x))` (decision 246).
   (R04.4.24)
3. **Integer literal to position or cut** of an integer domain (R04.2.18).
   For a domain with a runtime count (`domain V = 1..=n`, `n: 1..=2e5`),
   a literal outside the domain at its largest count is a compile error;
   one inside the domain at its smallest count is proven; one between is
   checked like any unproven position. (R04.4.25)
4. **Cut range to piece:** a literal `j..i` whose ends are cuts, where a
   `Range<V>` is expected, is the piece between them (see R04.4.21).
   (R04.4.26)
5. **Scalar to array fill:** a value `x: T` where `[D]T` is expected is
   the array over `D` with every element `x`, evaluated once:
   `var dist: [V]?Int = None`, `var dp: [0..=n]Int = 0`. Same as
   `[D].filled(x)`; `[D]` followed by a value without `=>` is a type,
   never a table. (R04.4.27)
6. **`Int` to `Mod<P>`:** an integer (a literal or any `Int` value) where
   a `Mod<P>` is expected is `Mod<P>.from_int(x)`, reduced mod `P`:
   `var ways: [0..=n]Z = 0`, `ways[0] = 1`, `cnt[v] = deg[v]`. (R04.4.28)

**Integer ranges as domains.** An integer range `lo..hi` (or `lo..=hi`,
or a counted `[n]`, which is `0..n`) used as an answer space or array
dimension is an anonymous domain: it satisfies `Domain`, its positions are
the `Int`s in it, and the cut before position `x` is the `Int` `x`, so its
cuts run `lo..=hi` for `lo..hi`. `(1..=5).bsearch(x => x >= 3)` is `3`,
the first position where `ok` holds, and `Graph.directed` accepts the
counted edges of `read<[m](u: V, v: V)>()`. R04.2.16 (`Int(c)` counts the
positions before `c`) is for declared domains, where the two agree when
the base is 0. (R04.4.30)

A general implicit `Into` is the C++ trap: conversions that don't show and
overloads that turn ambiguous.

### Methods

All of these are on every `Range<D>`. The two right columns spell out what
the generic meaning comes to for each kind.

| Form | On `Range<V>` (positions) | On `Range<V.cuts>` (cuts) | Cost |
|---|---|---|---|
| `r.len`, `r.is_empty`, `x in r` | count of positions; membership of a position | count of cuts; membership of a cut | O(1) (R04.4.12) |
| `for x in r`, `r.rev`, `r.step(k)` | positions | cuts | O(1) per step (R04.4.13) |
| `r.first`, `r.last` | end positions, `?V` | end cuts, `?V.cuts` | O(1) (R04.4.14) |
| `r.l`, `r.r` | the bounding cuts, `V.cuts` | the bounds one level up (`V.cuts.cuts`); you rarely want them: use `r.first`/`r.last` | O(1) (R04.4.15) |
| `r.clip(s)` | intersection | intersection | O(1) (R04.4.16) |
| `r.halve()` | `?(r.l..r.mid, r.mid..r.r)`, two non-empty halves; `None` when `len <= 1` | same, over cuts | O(1) (R04.4.17) |
| `r.mid` | middle cut, `V.cuts`, always present (`r.l` when empty) | the middle gap, one level up | O(1) (R04.4.18) |
| `r.first_true(ok)`, `r.last_true(ok)` | first position where false→true `ok` holds; last where true→false `ok` holds; `?V` | same over cuts, `?V.cuts` | O(log n) calls (R04.4.19) |
| `r.bsearch(ok)`, `r.bsearch(ok, last: true)` | the cut where `ok` flips (false→true; with `last:`, true→false), `V.cuts`; always exists | the gap between cuts, one level up; use `first_true` | O(log n) calls (R04.4.20) |
| `a[r]`, `pre.fold(r)`, `seg.fold(r)` | the piece `r` | n/a: takes `Range<V>`; a literal `j..i` is the piece | as the API says (R04.4.21) |

**Binary search: which element, or where it flips.** `first_true` and
`last_true` return the element, so the predicate reads like the statement
and is never negated; `None` when no element qualifies (an empty range is
the case with no elements). `bsearch` returns the cut where `ok` turns true,
which always exists, for answers that are boundaries: insertion points,
counts, prefix lengths. They agree: `r.first_true(ok)` is `r.bsearch(ok).r`
and `r.last_true(ok)` is `r.bsearch(ok, last: true).l`.

<!-- spec-only: fragment over `cost`, `budget`, `pre` and `a` from the text -->
```python
let k = (0..=n).last_true(k => cost(k) <= budget) ?? -1      // largest k that fits
let k = (0..=n).first_true(k => cost(k) >= x)!               // smallest k that reaches x
let c = V.cuts.range.last_true(c => pre[c] <= s)!              // longest prefix with sum <= s: a cut
let below = Int(a.keys.bsearch(i => a[i] >= t))               // how many of sorted a are < t
```

`ok` must be monotone in the direction the name says: false→true for
`first_true` and `bsearch`, true→false for `last_true` and
`bsearch(ok, last: true)`. Checked builds check that the
probes already made agree with one boundary (every true probe after every
false one, O(1) per probe), which catches most non-monotone predicates;
they never call `ok` more often than release does (see R09.2.3). Answer spaces that aren't a domain are integer ranges, an
anonymous domain whose positions and cuts are both `Int`:
`(0..=1e18).first_true(t => fits(t))`.

<!-- spec-only: fragment: `cross`, `cost` and `dp` come from the text -->
```python
fn inversions(r: Range<Pos>) -> Int:          // merge-sort shape
    let (left, right) = r.halve() else: return 0
    inversions(left) + inversions(right) + cross(left, right)

fn solve(rows: Range<Pos>, opts: Range<Pos>): // D&C DP optimisation
    if rows.is_empty: return
    let m = rows.mid.r!
    let best = opts.clip(..m.r).min_by(k => cost(k, m))!
    dp[m] = cost(best, m)
    solve(rows.l..m.l, opts.l..best.r)
    solve(m.r..rows.r, best.l..opts.r)
```

These replace `bsearch_first` and `bsearch_last`. Rejected: only the flip
cut, with `.l`/`.r` picking the side (last-true needs a negated predicate, a
`>` vs `>=` bug source); `bsearch(ok, last: true)` returning an element (a
flag where a name says it better; the flag stays on `bsearch`, where it only
says which way `ok` runs); detecting the direction (ambiguous when `ok` is constant); a
sentinel end when nothing qualifies (looks like a real answer).

## Arrays over domains

<!-- spec-only: expression tour, one form per line -->
```python
var dist: [V]?Int = None                 // fill
let deg = [V] v => g.deg(v)              // build from a fn
let a: [Pos]Int = read()                 // read n values
pre[i] - pre[j]       pre.fold(j..i)      // prefix sums are indexed by cuts
a[l..=r]                                 // a slice view, O(1), lent
a.keys  a.items  a.map(f)                // V, (V, T), [V]U
grid[r]                                  // a row view of [Row, Col]T
```

## Empty domains

Domains may be empty: iteration, arrays and folds over an empty domain just
work. `V.first`/`V.last` are `?V`, so the empty case is visible where you
ask for an end:

```python
let m = read<0..=2e5>()
domain E = 0..m
let e = E.first!             // traps naming `E` when m == 0
let first = E.first ?? return    // or handle it
```

No `NonEmpty` bound: the checker does not track emptiness. Tried and dropped
for now (decision 137); `!` is one character and says the same thing.

## Middle of a range

`r.mid` is the middle cut: ⌊n/2⌋ elements before it (C++'s `(l + r) / 2`
on a half-open range), always present. `r.halve()` is
`(r.l..r.mid, r.mid..r.r)`, `None` when either half would be empty.

| n | `r.halve()` | `r.mid` | `r.mid.r` |
|---|---|---|---|
| 0 | `None` | `r.l` | outside `r` (guard first) (R04.7.1) |
| 1 | `None` | `r.l` | the element (R04.7.2) |
| ≥ 2 | ⌊n/2⌋ and ⌈n/2⌉ elements | `left.r` | `right.first` (R04.7.3) |

`halve` is for the merge-sort shape (stop at one element). The element shape
(handle the middle, recurse around it, stop at empty) is
`if r.is_empty: return` then `r.mid.r!`; lint flags `.mid.r` with no
emptiness guard. Rejected: `r.mid: ?V`, the middle element (safe on empty,
but the cut is what both shapes share, and `halve` reads as its definition).

## Decided (decision 137)

- `.l`/`.r` switch kind, `.prev`/`.next` keep it; `.next(k)`/`.prev(k)` step k. (R04.8.1)
- Position arithmetic gives an `Int`; back into the domain is explicit. (R04.8.2)
- `r.first_true(ok)`/`r.last_true(ok)` return `?` the element; `r.bsearch(ok)`
  returns the flip cut, always present; `last: true` for true→false. They replace `bsearch_first`/`bsearch_last`. (R04.8.3)
- Empty domains are valid; `V.first`/`V.last` are `?V` (no `NonEmpty` bound). (R04.8.4)
- One generic `Range<D>`; elements are the kind of the ends, so `a..=b` on cuts
  is cuts. The half-open literal `j..i` with cut ends builds the piece where a
  `Range<V>` is expected (124): literal typing, not a conversion; no implicit
  `Into`. A position-only reading was rejected: it broke
  `for j in ..i: dp[j] + cost(j..i)`. (R04.8.5)
- `..i`, `..=i`, `i..` are open-ended ranges over the end's domain; bare `..` only inside `[...]`. (R04.8.6)
- `r.mid` is the middle cut, always present; `r.halve()` is `?(r.l..r.mid, r.mid..r.r)`, `None` at length <= 1. (R04.8.7)
- Integer literals convert to positions and cuts of integer domains. (R04.8.8)
- `V(x)` traps outside `V`; `V.at(x)` is `?V`; the pattern `V(j)` refutes. (R04.8.9)
- A list literal for `[D]T` gives one value per cell, so `[]` fills an empty `D`; a list whose length differs from a written count is a compile error, and from a run-time count a trap. (R04.8.10)

Rejected: `.start`/`.end`/`.before`/`.after` (four words for one idea);
`.n`/`.p` (`n` reads as a count, `p` as parent); `V.first` always trapping
(hides the n = 1 case until a hidden test hits it); `@get`-free bare calls
(silent cost); `V.try(x)` (`try` reads like error handling).
`r.split(at: c)` (`r.l..c` and `c..r.r` say it directly, and also say
"all but `v`": `r.l..v.l`, `v.r..r.r`).

## Open

None. (The positions after `v` are `v.after`, decision 186.)
