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
| `V.at(x)` | integer to position, trapping outside `V` | O(1) |
| `Int(v)` | position to its integer (for `1..=n`, that's 1..n) | free |
| `5` where a `V` is expected | constant literal, checked against `V` at compile time | free |

## Positions and cuts move the same way

| | neighbour of the same kind | the other kind, either side |
|---|---|---|
| position `v` | `v.prev`, `v.next`: `?V` | `v.l`, `v.r`: its cuts |
| cut `c` | `c.prev`, `c.next`: `?V.bounds` | `c.l`, `c.r`: the positions beside it, `?V` |

One rule: `.l`/`.r` switch kind (left or right side), `.prev`/`.next` keep it.
The receiver's kind is static, so a mix-up is a type error. `v.l..v.r` is
exactly `v`; a prefix DP reads `dp[v.r] = f(dp[v.l], a[v])`.

Arithmetic on a position gives an `Int` (`v + 1`); getting back into the
domain is explicit (`v.next`, `V.at(x)`). Arrays over an anonymous range
(`[0..=n]Int`) take any `Int`, checked, so ordinary DP tables stay plain.

## Ranges

```python
l..=r          # positions l through r            Range<V>
j..i           # cuts j to i                      Range<V.bounds>, also the positions between (§124)
..i   j..   .. # open ends default to V's start and end
```

| Form | Meaning | Cost |
|---|---|---|
| `r.len`, `r.is_empty`, `x in r` | | O(1) |
| `r.l`, `r.r` | the end cuts | O(1) |
| `r.first`, `r.last` | end positions, `?V` | O(1) |
| `for v in r`, `r.rev`, `r.step(k)` | iteration | O(1) per step |
| `r.clip(s)` | intersection | O(1) |
| `r.split(at: c)` | `(left, right)` at cut `c` | O(1) |
| `r.halve()` | `?(left, mid, right)` for divide and conquer; `none` when empty | O(1) |
| `r.bsearch(ok)` | **the cut** where monotone `ok` turns from false to true | O(log n) calls |

Binary search returns a cut, so "first true" vs "last false" is a one-word
change: `r.bsearch(ok).r` vs `r.bsearch(ok).l`. This replaces
`bsearch_first` and `bsearch_last`. Answer spaces that aren't a domain use
plain integer ranges: `(0..=1e18).bsearch(t => fits(t)).r`.

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

## Open questions (my recommendation first)

1. **Position arithmetic.** `v + 1` is an `Int`, never a `V`. Alternative: `v + k` is `?V`.
2. **Binary search spelling.** `r.bsearch(ok)` returning a cut. Alternative: keep `bsearch_first` / `bsearch_last`.
