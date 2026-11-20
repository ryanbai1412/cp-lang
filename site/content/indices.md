Most off-by-one bugs come from storing two kinds of numbers in the same `int`: an element position, a gap between elements, a count, or a vertex from a different graph. CP Lang separates them with two complementary tools: **structural range arrays** `[0..n]T` / `[0..=k]T` for quick integer-indexed tables, and **nominal domains** `domain V = 1..=n` when you want compile-time separation between index spaces.

## Structural range arrays (`[0..n]T`, `[0..=k]T`)

When an array is just a scratch table indexed by plain integers—like a frequency count or digit bucket—you do not need to name a domain:

```cp CP Lang v2.2
var cnt: [0..=k]Int = 0
for i in 0..n:
    cnt[a[i]] += 1
cnt.fill(0)                     // reset the whole array in place
```

Structural range arrays unify across functions whenever their written bounds match (`[0..n]Int` matches `[0..n]Int`). `for i in 0..n` binds `i: Int`, so `i + 1` is `Int`. Use `a.fill(x)` (or `seen.fill(none)` on `[D]?T`) to clear an array between test cases.

## Nominal domains and `Index`

Declare a nominal domain with `domain Name = lo..=hi` whenever you want zero-check indexing and compile-time protection against mixing different index spaces:

```cp CP Lang v2.2
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

All range data structures accept open and empty ranges directly (`st.fold(..h[i])`, `pre.sum(..i)`). Sub-ranges also index arrays as mutable slices: `sort(mut a[l..=r])` and `reverse(mut a[l..=r])` operate in place without allocating.

## Grids and `Cell = (Row, Col)`

A 2D array `[Row, Col]T` is indexed directly by a product-domain cell `type Cell = (Row, Col)`. `g[p]` works without unpacking `p`, and `Cell` implements `Index`, so `[Cell]T`, `bfs`, `Dsu<Cell>`, and `@memo` all work on grid coordinates.

```cp CP Lang v2.2
domain Row = 1..=h
domain Col = 1..=w
type Cell = (Row, Col)
let g: [Row, Col]Char = read()

for p in (Row, Col):
    for q in p.adj4:
        if g[q] != '#':
            visit(q)
```

`p.adj4` and `p.adj8` yield in-bounds orthogonal and 8-way neighbors with zero manual bounds checks. For directional movement, `std.grid` also provides `Dir4` (`.cw`, `.ccw`, `.back`, `Dir4.from("UDLR", ch)`), `p.step(d)`, `p.ray(d)`, `g.row(r)`, `g.col(c)`, `g.transposed`, and `g.flipped`.

## Coordinate compression with `compress(all)`

Coordinate compression produces a fresh nominal domain `xs.D` together with value lookup and inverse mapping:

```cp CP Lang v2.2
let xs = compress(all_coords)
var bit = Fenwick<xs.D, Int>()

for x in queries:
    let id = xs.at(x)           // exact lookup -> xs.D
    let original = xs[id]       // original value at id
    if let ge = xs.first_ge(x): // lower bound -> ?xs.D
        ans += bit.sum(..ge)
```

You never accidentally pass an uncompressed value into `bit` or mix a compressed index with raw coordinates.

## Algebraic `Index` and escape hatches

Five O(1) value-copied shapes implement `Index` automatically: `Bool`, unit or payload `enum` (computed on demand), `?T`, integer ranges `lo..=hi`, and tuples/records `(A, B)`, plus `Subset<D>` for bitmask DP (`D.K <= 25`). That means `var dp: [V, Bool]Int = 0`, `FlowGraph<Node>()` over a payload `enum Node`, and `[Subset<V>]Int` all compile to flat contiguous arrays.

When a tight inner loop genuinely needs raw integer indexing or unchecked speed, CP Lang provides explicit contest escape hatches: `a.raw[i]` indexes by 0-based `Int` (bounds-checked in debug), and an `unchecked:` block elides bounds and overflow traps in judge builds.
