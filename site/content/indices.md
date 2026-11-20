Most off-by-one bugs come from storing two kinds of numbers in the same `int`: an element position, a gap between elements, a count, or a vertex from a different graph. CP Lang separates them with two complementary tools: **structural range arrays** `[0..n]T` / `[0..=k]T` for quick integer-indexed tables, and **nominal domains** `domain V = 1..=n` when you want compile-time separation between index spaces.

## Structural range arrays (`[0..n]T`, `[0..=k]T`)

When an array is just a scratch table indexed by plain integers—like a frequency count or digit bucket—you do not need to name a domain:

```cp CP Lang 0.2.2
let n: 0..=100 = read()
let k: 0..=100 = read()
let a: [0..n]0..=k = read()
var cnt: [0..=k]Int = 0
for i in 0..n:
    cnt[a[i]] += 1
cnt.fill(0)                     // reset the whole array in place
```

Structural range arrays unify across functions whenever their written bounds match (`[0..n]Int` matches `[0..n]Int`). `for i in 0..n` binds `i: Int`, so `i + 1` is `Int`. Use `a.fill(x)` (or `seen.fill(none)` on `[D]?T`) to clear an array between test cases.

## Nominal domains and `Index`

Declare a nominal domain with `domain Name = lo..=hi` whenever you want zero-check indexing and compile-time protection against mixing different index spaces:

```cp CP Lang 0.2.2
let n: 1..=2e5 = read()
let m: 1..=2e5 = read()
domain Row = 1..=n
domain Col = 1..=m
```

`Row` and `Col` are distinct types even when `n == m`. An array `[Row, Col]Int` accepts `(r, c)` in that order only; swapping `r` and `c` is a type error before the program runs.

```example sum
```

A domain implements the `Index` and `Seq` traits:

- `V.K` is the number of elements; `V.first` and `V.last` are the endpoints (`V.K > 0`).
- `v.to_idx` converts to `0..V.K` and `V.from_idx(i) -> ?V` converts back safely.
- `v + k -> ?V` and `v - k -> ?V` step by an integer offset and return `none` past the ends; `b - a -> Int` is the signed distance; `v.cyc(k) -> V` wraps cyclically.

Array endpoints are **values**, not domain indices: `a.first` and `a.last` return `?T`, while `V.first` and `V.last` are indices. Like lists, empty arrays return `none`; unwrap with `a.first!` or `a.last!` only when nonempty. An unwrapped endpoint is also a mutable place, so `a.last! += 1` works on a mutable array or list:

```cp CP Lang 0.2.2
domain V = 5..8
let a: [V]Int = [V] v => 10 + v.to_idx
let endpoints: (?Int, ?Int) = (a.first, a.last)
print(V.first, V.last, endpoints.0, endpoints.1) // 5 7 10 12
domain Empty = 0..0
let empty: [Empty]Int = 0
print(empty.first ?? -1, empty.last ?? -1) // -1 -1
```

## Boundaries: `.l` and `.r`

Prefix sums, difference arrays, and knapsack tables live on the `n + 1` gaps around `n` elements. Every ordered domain `V` has a companion boundary space `V.bounds` with `V.K + 1` positions.

```example bounds
```

Every element `v: V` sits between two boundaries:

- `v.l` is the gap immediately to the left of `v`; `v.r` is the gap immediately to the right of `v`.
- `v.l(k)` and `v.r(k)` jump to the `k`-element window boundaries counting `v` (`v.l == v.l(1)`, `v.r == v.r(1)`).
- `b.left -> ?V` and `b.right -> ?V` move from a boundary back to the adjacent element.

The same `.l`/`.r` notation works directly in multi-dimensional DP tables such as 0/1 knapsack (`[I.bounds, 0..=cap]Int`):

```example knapsack
```

## Ranges, open ends, and slices

One `Range<I>` type covers half-open `l..r`, inclusive `l..=r`, and open-ended `..i`, `..=i`, `i..`, `i.r..`, and `l.r..r.l` (strictly between `l` and `r`). Every range provides `.first -> ?I`, `.last -> ?I`, `.mid -> ?I`, `.len`, `.is_empty`, `.rev`, `.step(k)`, and `(lo..=hi).clamp(D)`.

Range data structures accept range arguments such as `st.fold(..h[i])` and `pre.sum(..i)`. Read-only slice aggregates work today. In-place slice mutation and parent-indexed slice access remain specification work; use the explicit [copy-and-writeback example](course-index-slices.html#ranges-slices-and-compress-all) when sorting a window.

An open-ended `Int` range `l..` also works in a loop. Unlike an open-ended slice, it has no container to supply its endpoint: stop explicitly rather than trying to collect an unbounded iteration.

```cp CP Lang 0.2.2
let l = 3
for i in l..:
    if i == 6: break
    print(i) // 3, then 4, then 5
```

## Grids and `Cell = (Row, Col)`

A 2D array `[Row, Col]T` uses row and column domains. The example below names their pair `Cell`, iterates the two domains separately, and indexes `g[r, c]`. Product-domain iteration and fully generic cell-indexed structures are still being completed.

```cp CP Lang 0.2.2
use std.grid.{adj4}

let h: 1..=50 = read()
let w: 1..=50 = read()
domain Row = 1..=h
domain Col = 1..=w
type Cell = (Row, Col)
let g: [Row, Col]Char = read()
fn visit(p: Cell): print(p)

for r in Row:
    for c in Col:
        let p: Cell = (r, c)
        for q in p.adj4:
            if g[q.0, q.1] != '#':
                visit(q)
```

The imported `p.adj4` yields in-bounds orthogonal neighbors. `std.grid` also declares `adj8`, `step`, `ray`, and `Dir4`. Generic row/column lends and transformed grid views remain specification work rather than runnable APIs.

## Coordinate compression with `compress(all)`

Coordinate compression produces a fresh nominal domain `xs.D` together with value lookup and inverse mapping:

```cp CP Lang 0.2.2
let all_coords = [30, 10, 20]
let queries = [10, 30]
let xs = compress(all_coords)
var bit = Fenwick<xs.D, Int>()
var ans = 0

for x in queries:
    let id = xs.at(x)           // exact lookup -> xs.D
    let original = xs[id]       // original value at id
    bit.add(id, 1)
    if let ge = xs.first_ge(x): // lower bound -> ?xs.D
        ans += bit.sum(..ge)
print(ans)
```

You never accidentally pass an uncompressed value into `bit` or mix a compressed index with raw coordinates.

## Algebraic `Index` and escape hatches

The language's `Index` model covers `Bool`, finite enums, `?T`, integer ranges, tuples/records, and `Subset<D>` for bitmask DP. Support across generic library calls is still being completed: for example, flow over payload-enum vertices is not yet runnable. The [graph course](course-graphs-math.html#network-flow-with-typed-vertices) shows a working flow network over a numeric vertex domain.

For numeric offsets, use a structural range array such as `[0..n]Int`. The specified `a.raw[i]` escape for nominal arrays is not yet supported by the execution backend. `unchecked:` is available for a loop you have already proved safe; see the [C++ guide](cpp-guide.html#performance-controls-and-escape-hatches).
