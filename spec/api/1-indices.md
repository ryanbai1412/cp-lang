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
| `V.first`, `V.last` | `V` if the declaration proves non-empty, else `?V` | O(1) |
| `V.all` | the whole range, same as `..` | O(1) |
| `V.bounds` | the domain of cuts: `V.bounds.first` is the start, `.last` the end | O(1) |
| `x in V` | is integer `x` a position | O(1) |
| `V.at(x)` | integer to position, trapping outside `V` | O(1) |
| `Int(v)` | position to its integer (for `1..=n`, that's 1..n) | free |
| `5` where a `V` is expected | constant literal, checked against `V` at compile time | free |

## Positions and cuts move the same way

| | neighbour of the same kind | the other kind, either side |
|---|---|---|
| position `v` | `v.prev`, `v.next`: `?V` | `v.before`, `v.after`: cuts |
| cut `c` | `c.prev`, `c.next`: `?V.bounds` | `c.left`, `c.right`: `?V` |

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
| `r.start`, `r.end` | the end cuts | O(1) |
| `r.first`, `r.last` | end positions, `?V` | O(1) |
| `for v in r`, `r.rev`, `r.step(k)` | iteration | O(1) per step |
| `r.clip(s)` | intersection | O(1) |
| `r.split(at: c)` | `(left, right)` at cut `c` | O(1) |
| `r.halve()` | `?(left, mid, right)` for divide and conquer; `none` when empty | O(1) |
| `r.bsearch(ok)` | **the cut** where monotone `ok` turns from false to true | O(log n) calls |

Binary search returns a cut, so "first true" vs "last false" is a one-word
change: `r.bsearch(ok).right` vs `r.bsearch(ok).left`. This replaces
`bsearch_first` and `bsearch_last`. Answer spaces that aren't a domain use
plain integer ranges: `(0..=1e18).bsearch(t => fits(t)).right`.

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

## Open questions (my recommendation first)

1. **`V.first` type.** `V` when the declaration proves non-empty (`n: 1..=N`), else `?V`. Alternative: always `V`, trapping on empty.
2. **Position arithmetic.** `v + 1` is an `Int`, never a `V`. Alternative: `v + k` is `?V`.
3. **Binary search spelling.** `r.bsearch(ok)` returning a cut. Alternative: keep `bsearch_first` / `bsearch_last`.
