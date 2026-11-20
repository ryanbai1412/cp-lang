One general mechanism—the `Index` trait—powers domains, 2D grids, compressed coordinates, algebraic DP keys, and opt-in nominal string indexing. Instead of memorizing separate rules for each container, you learn one contract once.

## First-principles `Index` and runtime-frozen `let K`

Any finite type `I` can index an array `[I]T` if it provides two things:

1. A **runtime-frozen size** `let K = ...` (computed from immutable `let` bindings—never `const` when it depends on input, and never `var` because array storage cannot resize underneath existing indices).
2. A bijection `to_idx: I -> 0..K` and `from_idx: Int -> ?I` mapping every valid value densely into `0..K`.

Because every value of `I` is constructed inside `0..K`, `[I]T` lowers to a flat C++ `std::vector<T>` of length `K` with zero bounds checks in release builds. Five algebraic shapes derive `Index` automatically: `Bool` (`K = 2`), unit and payload `enum`, `?T` (`K = T.K + 1`), ranges `lo..=hi` (`Int` or `Char`), tuples/records `(A, B)` (`K = A.K * B.K`, row-major), plus `Subset<D>` (`K = 2^D.K` for `D.K <= 25`).

## Grids, `Cell = (Row, Col)`, and `p.adj4` / `p.adj8`

A two-dimensional grid `[Row, Col]T` is indexed by the product domain `type Cell = (Row, Col)`. Every `Cell` implements `Index`, so `[Cell]T`, `bfs`, `Dsu<Cell>`, and `@memo` accept grid cells directly.

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

In the full 0.2.2 grid API (`std.grid`, spec), `p.adj4` and `p.adj8` yield in-bounds neighbors without `dr`/`dc` arrays:

```cp CP Lang 0.2.2 · spec fragment
type Cell = (Row, Col)
let g: [Row, Col]Char = read()
for p in (Row, Col):
    for q in p.adj4:
        if g[q] != '#':
            visit(q)
```

## Ranges, slices, and `compress(all)`

One `Range<I>` type covers `l..r`, `l..=r`, and open-ended `..i`, `..=i`, `i..`, `i.r..`, and `l.r..r.l`. A range is plain data: you can store it in a variable or return it from a helper, then apply it to an array at the point of use (`sort(mut a[w])`) to form a temporary slice without changing index numbering.

> **WASM status for slices.** In the approved 0.2.2 design and `proto/cpc`, `a[l..=r]` produces a parent-indexed slice (`mid[V(3)]` keeps parent position `V(3)`) and `sort(mut a[l..=r])` sorts that window in place. The current Rust WASM frontend still types `a[l..=r]` as an integer-indexed sub-list and does not yet lower `a[l..=r].sort()`, so the slice snippet below is **prototype tested only** (not WASM runnable):

```cp Prototype tested · slices.cp (not yet in WASM)
let n: 3..=50 = read()
domain V = 1..=n
var a: [V]Int = read()

let mid = a[V(2)..=V(4)]
print(mid.len, mid[V(3)], sum(mid))
a[V(2)..=V(4)].sort()
print(a)
```

When input values reach `1e9` and only relative order matters, `compress(all)` creates a fresh nominal domain `xs.D` with `xs.at(val) -> xs.D`, `xs.first_ge(val) -> ?xs.D`, and inverse lookup `xs[id]` (spec):

```cp CP Lang 0.2.2 · spec fragment
let xs = compress(all_vals)
var bit = Fenwick<xs.D, Int>()
for x in queries:
    bit.add(xs.at(x), 1)
```

## Value-semantic `Str` and opt-in `[domain S]Char`

Default `Str` is a flat, value-semantic, `0..s.len`-indexed ASCII byte sequence (zero UTF-8, O(1) indexing `s[i]`, O(1) slicing `s[l..r]`). When a multi-string DP benefits from nominal position types, opt into `[domain S]Char`:

```example ascii
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
