# Round 1: indices and ranges

Everything else in the stdlib indexes by these, so they come first.

## The model in one paragraph

A **domain** is a finite, ordered index type. Its members are **positions**.
Between and around positions sit **cuts**: a domain of n positions has n + 1
cuts, `D.cuts`. A **range** is a contiguous run of positions, and its two
ends are cuts. Arrays are `[D]T`, indexed only by `D`, so a vertex never
indexes an edge array and a cut never indexes a position array.

```python
domain V = 1..=n                       // positions 1..n
domain Pos = 0..n                      // positions 0..n-1; Pos.cuts is cuts 0..=n
let (domain C, comp) = compress(xs)    // fresh domain from a computation
domain Cell = (Row, Col)               // product: [Cell]T and [Row, Col]T are one 2-D array type
enum Dir: N, E, S, W                   // a fieldless enum is a domain too: [Dir]Int
```

## Domains

| Form | Meaning | Cost |
|---|---|---|
| `for v in V`, `V.rev` | positions in order or reverse order | O(1) per step |
| `V.count` | number of positions | O(1) |
| `V.first`, `V.last` | `?V`, `none` when `V` is empty; write `V.first!` when you know it isn't | O(1) |
| `V.range` | the whole range, same as `..` (`a.keys` for an array's) | O(1) |
| `V.cuts` | the domain of cuts (type `Cut<V>`): `V.cuts.first` is the start, `.last` the end | O(1) |
| `x in V` | is integer `x` a position | O(1) |
| `V(x)` | integer to position, trapping outside `V` | O(1) |
| `V.at(x)` | integer to position, `?V`: `none` outside `V` | O(1) |
| `P.at((x, y))`, `P((x, y))` | for a product domain `P = (A, B)`, a tuple of integers to a position: `none` (or a trap) unless every part lies in its factor, so a neighbour that may fall off is `if let q = P.at((r - 1, c))` | O(1) |
| `a[p]`, `a[x, y]` | for `P = (A, B)`, `[P]T` and `[A, B]T` are the same type: a `p: P` indexes it whole, its parts index it one by one, and `[P] p => f(p)` builds it | O(1) |
| `V.wrap(x)` | integer to position, mod `V.count`: rings and cyclic shifts | O(1) |
| `if let V(j) = i - 10:` | the pattern form of `V.at`: matches only inside `V` | O(1) |
| `Int(v)` | position to its integer (for `1..=n`, that's 1..n) | free |
| `Int(c)` | cut to its integer: the number of positions before it, 0..=n whatever the base (`1..=n`: `v.l` is v-1, `v.r` is v; `0..n`: v, v+1) | free |
| `V.cuts(k)`, `V.cuts.at(k)` | integer to cut: trapping, or `?V.cuts` | O(1) |
| `5` where a `V` or `V.cuts` is expected | integer literal is that position or cut (integer domains; also fills `[d]V.cuts = 0`); outside a static bound it is a compile error | free |

## Positions and cuts move the same way

| | neighbour of the same kind | the other kind, either side |
|---|---|---|
| position `v` | `v.prev`, `v.next`: `?V` | `v.l`, `v.r`: its cuts |
| cut `c` | `c.prev`, `c.next`: `?V.cuts` | `c.l`, `c.r`: the positions beside it, `?V` |

`v.next(k)`, `v.prev(k)` step k at once, `?V` (cuts too). `.next` is
`.next(k = 1)`: only methods marked `@get` (which promises O(1)) read without
parentheses, and an `@get` method may take defaulted parameters. A plain
method always needs `()`, so nothing costly hides behind a bare name.

One rule: `.l`/`.r` switch kind (left or right side), `.prev`/`.next` keep it.
The receiver's kind is static, so a mix-up is a type error. `v.l..v.r` is
exactly `v`; a prefix DP reads `dp[v.r] = f(dp[v.l], a[v])`.

Arithmetic on a position gives an `Int` (`v + 1`); getting back into the
domain is explicit (`v.next`, `V(x)`). Arrays over an anonymous range
(`[0..=n]Int`) take any `Int`, checked, so ordinary DP tables stay plain.

## Ranges

### Two kinds of range, one type

`Range<D>` is a contiguous run of a domain `D`. `V.cuts` is a domain like
any other, so there are two kinds you meet in practice, and every method
below is generic over `D`: it means the same thing on both.

| | `Range<V>`: over positions | `Range<V.cuts>`: over cuts |
|---|---|---|
| written | `l..r`, `l..=r` with position ends | `j..i`, `j..=i` with cut ends |
| elements (`for`, `in`, `len`) | positions | cuts |
| typical use | a piece of the array: slice, fold, cost | split points, prefix lengths, answers that may be 0 or n |
| example | `a[l..=r]`, `seg.fold(l..r)` | `for j in ..i`, `for k in a..=b` |

**The rule: a range's elements are the kind of its ends.** Cuts in, cuts out:
`for k in a..=b` walks cuts a through b, `for j in ..i` walks the cuts before i.

### One exception, in how a literal is typed: the piece

Cuts `j` and `i` bound a piece of the array: the positions between them. So
a half-open range literal with cut ends, written where a `Range<V>` is
expected, builds that `Range<V>` directly:

```python
for j in ..i:                        // ..i: no Range<V> expected, so cuts
    dp[i] min= dp[j] + cost(j..i)    // cost takes Range<V>: j..i is the piece
pre.fold(j..i)   a[j..i]   seg.fold(j..i)       // same: these take Range<V>
```

This is **not a conversion and there is no implicit `Into`.** It decides what
the literal builds, exactly like `5` becoming a position where a `V` is
expected. Consequences:
- Only the literal `a..b` does it. A stored cut range never becomes a piece:
  `let s = j..i; cost(s)` is a type error whose fix is to write `cost(j..i)`.
- Only half-open: `j..=i` on cuts names no piece, so it's an error where a
  `Range<V>` is expected.
- It's safe because the two readings are the same set: cut `c` in `j..i`
  stands for the position `c.r`, and the cuts j..i-1 are exactly the left
  edges of the positions between `j` and `i`.

### Open-ended ranges

Leaving out an end means the edge of the other end's domain, so the domain
never has to be named and never has to be non-empty:

| written | means | elements |
|---|---|---|
| `..i` | `D.first..i`, where `i: D` | everything before `i` |
| `..=i` | `D.first..=i` | everything up to and including `i` |
| `i..` | `i..=D.last` | `i` and everything after it |
| `..` | all of the indexed dimension | only inside `[...]`: `grid[r, ..]` |

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

```python
for j in ..i:                        // every cut before i
    dp[i] min= dp[j] + cost(j..i)
let best = a[..c].max()              // the prefix left of cut c
```

The language's implicit conversions stay a closed list, none user-extensible:
`T` to `?T`, a range type to a wider one (`1..=5` to `Int`), and literal
typing (integer to position or cut, cut range to piece). A general implicit
`Into` is the C++ trap: conversions that don't show and overloads that turn
ambiguous.

### Methods

All of these are on every `Range<D>`. The two right columns spell out what
the generic meaning comes to for each kind.

| Form | On `Range<V>` (positions) | On `Range<V.cuts>` (cuts) | Cost |
|---|---|---|---|
| `r.len`, `r.is_empty`, `x in r` | count of positions; membership of a position | count of cuts; membership of a cut | O(1) |
| `for x in r`, `r.rev`, `r.step(k)` | positions | cuts | O(1) per step |
| `r.first`, `r.last` | end positions, `?V` | end cuts, `?V.cuts` | O(1) |
| `r.l`, `r.r` | the bounding cuts, `V.cuts` | the bounds one level up (`V.cuts.cuts`); you rarely want them: use `r.first`/`r.last` | O(1) |
| `r.clip(s)` | intersection | intersection | O(1) |
| `r.halve()` | `?(r.l..r.mid, r.mid..r.r)`, two non-empty halves; `none` when `len <= 1` | same, over cuts | O(1) |
| `r.mid` | middle cut, `V.cuts`, always present (`r.l` when empty) | the middle gap, one level up | O(1) |
| `r.first_true(ok)`, `r.last_true(ok)` | first position where false→true `ok` holds; last where true→false `ok` holds; `?V` | same over cuts, `?V.cuts` | O(log n) calls |
| `r.bsearch(ok)`, `r.bsearch(ok, last: true)` | the cut where `ok` flips (false→true; with `last:`, true→false), `V.cuts`; always exists | the gap between cuts, one level up; use `first_true` | O(log n) calls |
| `a[r]`, `pre.fold(r)`, `seg.fold(r)` | the piece `r` | n/a: takes `Range<V>`; a literal `j..i` is the piece | as the API says |

**Binary search: which element, or where it flips.** `first_true` and
`last_true` return the element, so the predicate reads like the statement
and is never negated; `none` when no element qualifies (an empty range is
the case with no elements). `bsearch` returns the cut where `ok` turns true,
which always exists, for answers that are boundaries: insertion points,
counts, prefix lengths. They agree: `r.first_true(ok)` is `r.bsearch(ok).r`
and `r.last_true(ok)` is `r.bsearch(ok, last: true).l`.

```python
let k = (0..=n).last_true(k => cost(k) <= budget) ?? -1      // largest k that fits
let k = (0..=n).first_true(k => cost(k) >= x)!               // smallest k that reaches x
let c = V.cuts.range.last_true(c => pre[c] <= s)!              // longest prefix with sum <= s: a cut
let below = Int(a.keys.bsearch(i => a[i] >= t))               // how many of sorted a are < t
```

`ok` must be monotone in the direction the name says: false→true for
`first_true` and `bsearch`, true→false for `last_true` and
`bsearch(ok, last: true)`. Debug builds check
`ok` on both sides of the answer (O(1)), which catches most non-monotone
predicates. Answer spaces that aren't a domain are integer ranges, an
anonymous domain whose positions and cuts are both `Int`:
`(0..=1e18).first_true(t => fits(t))`.

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

```python
var dist: [V]?Int = none                 // fill
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
let e = E.first ?? return    // or handle it
```

No `NonEmpty` bound: the checker does not track emptiness. Tried and dropped
for now (decision 137); `!` is one character and says the same thing.

## Middle of a range

`r.mid` is the middle cut: ⌊n/2⌋ elements before it (C++'s `(l + r) / 2`
on a half-open range), always present. `r.halve()` is
`(r.l..r.mid, r.mid..r.r)`, `none` when either half would be empty.

| n | `r.halve()` | `r.mid` | `r.mid.r` |
|---|---|---|---|
| 0 | `none` | `r.l` | outside `r` (guard first) |
| 1 | `none` | `r.l` | the element |
| ≥ 2 | ⌊n/2⌋ and ⌈n/2⌉ elements | `left.r` | `right.first` |

`halve` is for the merge-sort shape (stop at one element). The element shape
(handle the middle, recurse around it, stop at empty) is
`if r.is_empty: return` then `r.mid.r!`; lint flags `.mid.r` with no
emptiness guard. Rejected: `r.mid: ?V`, the middle element (safe on empty,
but the cut is what both shapes share, and `halve` reads as its definition).

## Decided (decision 137)

- `.l`/`.r` switch kind, `.prev`/`.next` keep it; `.next(k)`/`.prev(k)` step k.
- Position arithmetic gives an `Int`; back into the domain is explicit.
- `r.first_true(ok)`/`r.last_true(ok)` return `?` the element; `r.bsearch(ok)`
  returns the flip cut, always present; `last: true` for true→false. They replace `bsearch_first`/`bsearch_last`.
- Empty domains are valid; `V.first`/`V.last` are `?V` (no `NonEmpty` bound).
- One generic `Range<D>`; elements are the kind of the ends, so `a..=b` on cuts
  is cuts. The half-open literal `j..i` with cut ends builds the piece where a
  `Range<V>` is expected (124): literal typing, not a conversion; no implicit
  `Into`. A position-only reading was rejected: it broke
  `for j in ..i: dp[j] + cost(j..i)`.
- `..i`, `..=i`, `i..` are open-ended ranges over the end's domain; bare `..` only inside `[...]`.
- `r.mid` is the middle cut, always present; `r.halve()` is `?(r.l..r.mid, r.mid..r.r)`, `none` at length <= 1.
- Integer literals convert to positions and cuts of integer domains.
- `V(x)` traps outside `V`; `V.at(x)` is `?V`; the pattern `V(j)` refutes.

Rejected: `.start`/`.end`/`.before`/`.after` (four words for one idea);
`.n`/`.p` (`n` reads as a count, `p` as parent); `V.first` always trapping
(hides the n = 1 case until a hidden test hits it); `@get`-free bare calls
(silent cost); `V.try(x)` (`try` reads like error handling).
`r.split(at: c)` (`r.l..c` and `c..r.r` say it directly, and also say
"all but `v`": `r.l..v.l`, `v.r..r.r`).

## Open

- **"The positions after `v`."** Under the ends rule, `for j in v.r..` walks
  cuts, not positions. Today's spellings: `a[v.r..]` (a slice, so the piece
  literal applies), `T.range.clip(v.r..)`, or `for j in T if j > v`. None is as
  direct as the C++ `for (j = i + 1; j < n; j++)`. Seen in ioi25_souvenirs.
