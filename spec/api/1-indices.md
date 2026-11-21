# Round 1: indices and ranges

Everything else in the stdlib indexes by these, so they come first.

## The model in one paragraph

A **domain** is a finite, ordered index type. Its members are **positions**.
Between and around positions sit **cuts**: a domain of n positions has n + 1
cuts, `D.bounds`. A **range** is a contiguous run of positions, and its two
ends are cuts. Arrays are `[D]T`, indexed only by `D`, so a vertex never
indexes an edge array and a cut never indexes a position array.

```python
domain V = 1..=n                       # positions 1..n
domain Pos = 0..n                      # positions 0..n-1; Pos.bounds is cuts 0..=n
let (domain C, comp) = compress(xs)    # fresh domain from a computation
domain Cell = (Row, Col)               # product: [Row, Col]T is a 2-D array
enum Dir: N, E, S, W                   # a fieldless enum is a domain too: [Dir]Int
```

## Domains

| Form | Meaning | Cost |
|---|---|---|
| `for v in V`, `V.rev` | positions in order or reverse order | O(1) per step |
| `V.count` | number of positions | O(1) |
| `V.first`, `V.last` | `V`; only on `V: NonEmpty` (below) | O(1) |
| `V.all` | the whole range, same as `..` | O(1) |
| `V.bounds` | the domain of cuts: `V.bounds.first` is the start, `.last` the end | O(1) |
| `x in V` | is integer `x` a position | O(1) |
| `V(x)` | integer to position, trapping outside `V` | O(1) |
| `V.at(x)` | integer to position, `?V`: `none` outside `V` | O(1) |
| `if let V(j) = i - 10:` | the pattern form of `V.at`: matches only inside `V` | O(1) |
| `Int(v)` | position to its integer (for `1..=n`, that's 1..n) | free |
| `5` where a `V` or `V.bounds` is expected | integer literal is that position or cut (integer domains; also fills `[d]V.bounds = 0`); outside a static bound it is a compile error | free |

## Positions and cuts move the same way

| | neighbour of the same kind | the other kind, either side |
|---|---|---|
| position `v` | `v.prev`, `v.next`: `?V` | `v.l`, `v.r`: its cuts |
| cut `c` | `c.prev`, `c.next`: `?V.bounds` | `c.l`, `c.r`: the positions beside it, `?V` |

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

`Range<D>` is a contiguous run of a domain `D`. `V.bounds` is a domain like
any other, so there are two kinds you meet in practice, and every method
below is generic over `D`: it means the same thing on both.

| | `Range<V>`: over positions | `Range<V.bounds>`: over cuts |
|---|---|---|
| written | `l..r`, `l..=r` with position ends | `j..i`, `j..=i` with cut ends |
| elements (`for`, `in`, `len`) | positions | cuts |
| typical use | a piece of the array: slice, fold, cost | split points, prefix lengths, answers that may be 0 or n |
| example | `a[l..=r]`, `seg.fold(l..r)` | `for j in lo..i`, `for k in a..=b` |

**The rule: a range's elements are the kind of its ends.** Cuts in, cuts out:
`for k in a..=b` walks cuts a through b, `for j in lo..i` walks cuts lo to i-1.

### One exception, in how a literal is typed: the piece

Cuts `j` and `i` bound a piece of the array: the positions between them. So
a half-open range literal with cut ends, written where a `Range<V>` is
expected, builds that `Range<V>` directly:

```python
for j in lo..i:                      # lo..i: no Range<V> expected, so cuts
    dp[i] min= dp[j] + cost(j..i)    # cost takes Range<V>: j..i is the piece
pre.sum(j..i)   a[j..i]   seg.fold(j..i)       # same: these take Range<V>
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

The language's implicit conversions stay a closed list, none user-extensible:
`T` to `?T`, a range type to a wider one (`1..=5` to `Int`), and literal
typing (integer to position or cut, cut range to piece). A general implicit
`Into` is the C++ trap: conversions that don't show and overloads that turn
ambiguous.

### Methods

All of these are on every `Range<D>`. The two right columns spell out what
the generic meaning comes to for each kind.

| Form | On `Range<V>` (positions) | On `Range<V.bounds>` (cuts) | Cost |
|---|---|---|---|
| `r.len`, `r.is_empty`, `x in r` | count of positions; membership of a position | count of cuts; membership of a cut | O(1) |
| `for x in r`, `r.rev`, `r.step(k)` | positions | cuts | O(1) per step |
| `r.first`, `r.last` | end positions, `?V` | end cuts, `?V.bounds` | O(1) |
| `r.l`, `r.r` | the bounding cuts, `V.bounds` | the bounds one level up (see below); write `r.first`/`r.last` instead | O(1) |
| `r.clip(s)` | intersection | intersection | O(1) |
| `r.halve()` | `?(left, right)`, two non-empty halves meeting at the middle cut; `none` when `len <= 1` | same, over cuts | O(1) |
| `r.mid` | middle position, `?V` | middle cut, `?V.bounds` | O(1) |
| `r.bsearch(ok)` | where monotone `ok` on positions flips; `.l`/`.r` give the last false and first true position | where `ok` on cuts flips; `.l`/`.r` give the last false and first true cut | O(log n) calls |
| `a[r]`, `pre.sum(r)`, `seg.fold(r)` | the piece `r` | n/a: takes `Range<V>`; a literal `j..i` is the piece | as the API says |

**Ends one level up.** A range's ends are cuts of its domain. For a range
over positions those are `V.bounds`, which is what you want. For a range over
cuts they are `V.bounds.bounds`, the gaps between cuts, which you only meet as
`bsearch`'s answer and immediately turn into a cut with `.l`/`.r`, the same
one rule as everywhere else:

```python
let k = V.bounds.all.bsearch(c => pre[c] > s).l!   # longest prefix with sum <= s: a cut
```

Answer spaces that aren't a domain are integer ranges, an anonymous domain
whose positions and cuts are both `Int`: `(0..=1e18).bsearch(t => fits(t)).r`.

```python
fn inversions(r: Range<Pos>) -> Int:          # merge-sort shape
    let (left, right) = r.halve() else: return 0
    inversions(left) + inversions(right) + cross(left, right)

fn solve(rows: Range<Pos>, opts: Range<Pos>): # D&C DP optimisation
    let m = rows.mid else: return
    let best = opts.clip(..m.r).min_by(k => cost(k, m))!
    dp[m] = cost(best, m)
    solve(rows.l..m.l, opts.l..best.r)
    solve(m.r..rows.r, best.l..opts.r)
```

Binary search returns a cut, so "first true" vs "last false" is a one-word
change: `r.bsearch(ok).r` vs `r.bsearch(ok).l`. This replaces
`bsearch_first` and `bsearch_last`.

## Arrays over domains

```python
var dist: [V]?Int = none                 # fill
let deg = [V] v => g.deg(v)              # build from a fn
let a: [Pos]Int = read()                 # read n values
pre[i] - pre[j]       pre.sum(j..i)      # prefix sums are indexed by cuts
a[l..=r]                                 # a slice view, O(1), lent
a.keys  a.items  a.map(f)                # V, (V, T), [V]U
grid[r]                                  # a row view of [Row, Col]T
```

## Empty domains

Domains may be empty: iteration, arrays and folds over an empty domain just
work. Only `first`/`last` need a position, so they exist only on
`D: NonEmpty`, a bound the checker derives at each use from what it knows:

```python
let m = read<0..=2e5>()
domain E = 0..m
E.first                      # error: `E` may be empty: `m: 0..=2e5` allows 0
if m == 0: return            # narrows m to 1..=2e5 for the rest of the block
let e = E.first              # fine: E.count is m, and m >= 1 here
```

- `E.count` has a range type, like any Int; `NonEmpty` holds when its lower
  bound is at least 1. Guards narrow it (`if m == 0: return`,
  `if E.is_empty: return`), the same rule that makes `if x is none: return`
  leave `x` non-optional.
- Possibly empty and you want an optional: `E.all.first` is the sequence
  `.first`, `?E`.
- Generic code asks for it: `fn f<domain D: NonEmpty>()`.
- The check compares one lower bound with 1, so it stays linear.

## Decided (decision 137)

- `.l`/`.r` switch kind, `.prev`/`.next` keep it; `.next(k)`/`.prev(k)` step k.
- Position arithmetic gives an `Int`; back into the domain is explicit.
- `r.bsearch(ok)` returns a cut and replaces `bsearch_first`/`bsearch_last`.
- Empty domains are valid; `first`/`last` need `NonEmpty`, derived and narrowed.
- One generic `Range<D>`; elements are the kind of the ends, so `a..=b` on cuts
  is cuts. The half-open literal `j..i` with cut ends builds the piece where a
  `Range<V>` is expected (124): literal typing, not a conversion; no implicit
  `Into`. A position-only reading was rejected: it broke
  `for j in lo..i: dp[j] + cost(j..i)`.
- `r.halve()` is `?(left, right)`, `none` at length <= 1; `r.mid` is `?V`.
- Integer literals convert to positions and cuts of integer domains.
- `V(x)` traps outside `V`; `V.at(x)` is `?V`; the pattern `V(j)` refutes.

Rejected: `.start`/`.end`/`.before`/`.after` (four words for one idea);
`.n`/`.p` (`n` reads as a count, `p` as parent); `V.first` always trapping
(hides the n = 1 case until a hidden test hits it); `@get`-free bare calls
(silent cost); `V.try(x)` (`try` reads like error handling).
`r.split(at: c)` (`r.l..c` and `c..r.r` say it directly, and also say
"all but `v`": `r.l..v.l`, `v.r..r.r`).

## Open

Nothing yet. New questions land here first.
