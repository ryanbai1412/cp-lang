One general mechanism—the `Index` trait—powers domains, 2D grids, compressed coordinates, algebraic DP keys, and opt-in nominal string indexing. Instead of memorizing separate rules for each container, you learn one contract once.

## First-principles `Index` and runtime-frozen `let K`

Any finite type `I` can index an array `[I]T` if it provides two things:

1. A **runtime-frozen size** `let K = ...` (computed from immutable `let` bindings—never `const` when it depends on input, and never `var` because array storage cannot resize underneath existing indices).
2. A bijection `to_idx: I -> 0..K` and `from_idx: Int -> ?I` mapping every valid value densely into `0..K`.

Because every value of `I` is constructed inside `0..K`, `[I]T` lowers to a flat C++ `std::vector<T>` of length `K` with zero bounds checks in release builds. Five algebraic shapes derive `Index` automatically: `Bool` (`K = 2`), unit and payload `enum`, `?T` (`K = T.K + 1`), ranges `lo..=hi` (`Int` or `Char`), tuples/records `(A, B)` (`K = A.K * B.K`, row-major), plus `Subset<D>` (`K = 2^D.K` for `D.K <= 25`).

## Grids, `Cell = (Row, Col)`, and `p.adj4` / `p.adj8`

A two-dimensional grid `[Row, Col]T` uses row and column domains. Name their pair `type Cell = (Row, Col)` when passing coordinates to a helper. The examples below use supported nested domain loops and direct `g[r, c]` access; fully generic product-domain indexing is still being completed.

```cp CP Lang · grid-sum.cp
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
print(total, g.sum(), total == g.sum())
```

Import `adj4` from `std.grid` to visit in-bounds neighbors without writing `dr`/`dc` arrays:

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

## Ranges, slices, and `compress(all)`

One `Range<I>` type covers `l..r`, `l..=r`, and open-ended `..i`, `..=i`, `i..`, `i.r..`, and `l.r..r.l`. A range is plain data: store it in a variable or return it from a helper, then use it to read a slice or query a data structure. The current compiler materializes array slices rather than providing parent-indexed mutable views.

> **Current slice support.** Parent-indexed slicing and in-place slice sorting are not yet complete in the compiler or browser. To sort part of an array today, copy the selected values into a list, sort that list, and write it back explicitly:

```cp CP Lang · window-copy.cp
let n: 4..=50 = read()
domain V = 1..=n
var a: [V]Int = read()

var mid: List<Int> = []
for v in V(2)..=V(4): mid.push(a[v])
print(mid.len, mid[1], mid.sum())
mid.sort()
for i in 0..mid.len:
    a[V(i + 2)] = mid[i]
print(a)
```

The full slice specification retains parent indices and permits sorting the window in place; that form is not yet implemented. Read-only slice aggregates work today. Keep element access and mutations explicit with the copy-and-writeback technique above:

```cp CP Lang 0.2.2
let n: 4..=50 = read()
domain V = 1..=n
var a: [V]Int = read()

let mid = a[V(2)..=V(4)]
print(mid.len, mid.sum())
```

When input values reach `1e9` and only relative order matters, `compress(all)` creates a fresh nominal domain `xs.D` with `xs.at(val) -> xs.D`, `xs.first_ge(val) -> ?xs.D`, and inverse lookup `xs[id]`:

```cp CP Lang 0.2.2
let all_vals = [1_000_000_000, 7, 42]
let queries = [7, 42, 7]
let xs = compress(all_vals)
var bit = Fenwick<xs.D, Int>()
for x in queries:
    bit.add(xs.at(x), 1)
print(bit.sum(xs.D.first..=xs.D.last))
```

## Value-semantic `Str` and opt-in `[domain S]Char`

Default `Str` is a flat, value-semantic, `0..s.len`-indexed ASCII byte sequence (zero UTF-8, O(1) indexing `s[i]`). Slicing `s[l..r]` currently creates a copy. When a multi-string DP benefits from nominal position types, opt into `[domain S]Char`:

```example ascii
```

## Multi-dimensional boundaries in action

Combining `I.cuts` with a structural range `0..=cap` gives a zero-boilerplate 0/1 knapsack table:

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

## Write a program

```exercise index-write-row-sums
```
