One general index mechanism—the `Index` trait—powers domains, 2D grids, compressed coordinates, algebraic DP keys, and opt-in nominal string indexing. Instead of memorizing separate rules for each container, you learn one set of rules once.

## Grids, `Cell = (Row, Col)`, and `p.adj4` / `p.adj8`

A two-dimensional grid `[Row, Col]T` is indexed by the product domain `type Cell = (Row, Col)`. You index with `g[p]` directly without unpacking `(r, c)`, and iterate in-bounds neighbors without writing `dr`/`dc` arrays or bounds checks:

```cp CP Lang 0.2.2
domain Row = 1..=h
domain Col = 1..=w
type Cell = (Row, Col)
let g: [Row, Col]Char = read()

for p in (Row, Col):
    for q in p.adj4:            // orthogonal in-bounds neighbors; use p.adj8 for 8-way
        if g[q] != '#':
            visit(q)
```

Because `Cell` implements `Index`, `[Cell]T`, `bfs`, `Dsu<Cell>`, and `@memo` all accept grid cells out of the box.

## Ranges, slices, and `compress(all)`

One `Range<I>` type covers `l..r`, `l..=r`, and open-ended `..i`, `..=i`, `i..`, `i.r..`, and `l.r..r.l`. Sub-ranges index arrays as mutable slices (`sort(mut a[l..=r])`, `reverse(mut a[l..=r])`), and all range structures accept open and empty ranges (`st.fold(..h[i])`, `pre.sum(..i)`).

When values are huge (up to `1e9`) but only their relative order matters, `compress(all)` creates a fresh nominal domain `xs.D` alongside exact lookup `xs.at(val) -> xs.D`, lower-bound lookup `xs.first_ge(val) -> ?xs.D`, and inverse lookup `xs[id]`:

```cp CP Lang 0.2.2
let xs = compress(all_vals)
var bit = Fenwick<xs.D, Int>()
for x in queries:
    bit.add(xs.at(x), 1)
```

## Value-semantic `Str` and opt-in `[domain S]Char`

Default `Str` is a flat, value-semantic, `0..s.len`-indexed ASCII byte sequence (zero UTF-8, O(1) indexing `s[i]`, O(1) slicing `s[l..r]`). When a multi-string DP benefits from nominal position types, opt into `[domain S]Char`:

```example ascii
```

Every `Index` type compiles to a dense `0..D.K` integer offset in C++17, including `Bool`, unit/payload `enum`, `?T`, ranges `lo..=hi`, tuples/records `(A, B)`, and `Subset<D>` (`D.K <= 25`).

```cp Prototype tested · grid-sum.cp
let h: 1..=50 = read()
let w: 1..=50 = read()
domain Row = 1..=h
domain Col = 1..=w

var g: [Row, Col]Int = 0
var next_val = 1
for r in Row:
    for c in Col:
        g[r, c] = next_val
        next_val += 1

var total = 0
for r in Row:
    for c in Col:
        total += g[r, c]
print(total, sum(g), total == sum(g))
```

```cp Prototype tested · slices.cp
let n: 3..=50 = read()
domain V = 1..=n
var a: [V]Int = read()

let mid = a[V(2)..=V(4)]
print(mid.len, mid[V(3)], sum(mid))
a[V(2)..=V(4)].sort()
print(a)
```

## Multi-dimensional boundaries in action

Combining `I.bounds` with a structural range `0..=cap` gives a zero-boilerplate 0/1 knapsack table:

```example knapsack
```

## Practice

```exercise index-slices-predict-ordinal
```

```exercise index-slices-frozen-k
```

```exercise index-slices-slice-membership
```

```exercise index-slices-range-or-lend
```
