The previous chapter treated `domain V = 1..=n` as a primitive. It is not: it is shorthand for a small sealed type that implements a general mechanism called `Index`. This chapter opens that mechanism up, then uses it to index grids by tuples and to take typed windows out of arrays without copying them.

> **What runs.** `V.K`, `.to_idx`, `V.from_idx`, character domains, tuple indexing, ranges returned from functions, slices with `.positions`, and `.copy()` run in the Python prototype; the outputs below come from native C++ builds. User-written `Index` implementations run as well. The full lend rules are approved design described in [plan 19](https://github.com/ryanbai1412/cp-lang/blob/main/plan/19-index-and-lends.md); the parts the prototype does not check are marked.

## Every index is a dense ordinal

An `Index` type answers two questions: how many values it has, and which ordinal from `0` to `K - 1` each value occupies. That is all an array needs. `[V]T` stores `V.K` elements, and `a[v]` reads slot `v.to_idx`.

```cp Prototype tested · ordinals.cp
let n: 1..=9 = read()
domain V = 1..=n
let v = V(3)
print(V.K, v.label, v.to_idx, v.idx, V.from_idx(0) == V.first)
```

For input `5` it prints `5 3 2 2 YES`. The label `3` is the statement's number; the ordinal `2` is where it lives in memory. `V.from_idx(i)` converts back and is checked, so it only accepts an ordinal below `V.K`. Character domains work the same way: `domain L = 'a'..='e'` has `L.K == 5`, and `L.last.to_idx` is `4`.

Because the mechanism only needs a count and a bijection, it is not limited to integer ranges. You can write the bijection yourself. This flattened grid cell runs in the prototype:

```cp Prototype tested · user-index.cp
let h: 1..=9 = read()
let w: 1..=9 = read()
domain Row = 1..=h
domain Col = 1..=w

struct Cell:
    r: Row
    c: Col

impl Index for Cell:
    let K = h * w
    fn to_idx(self) -> Int = r.ord * w + c.ord
    fn from_idx(i: Int) -> Cell = Cell(Row.nth(i / w), Col.nth(i % w))

var dist: [Cell]Int = 0
for x in Cell:
    dist[x] = x.r.label * 10 + x.c.label
let last = Cell(Row.last, Col.last)
print(Cell.K, dist[last], last.to_idx, Cell.from_idx(1).c)
```

For input `2 3` it prints `6 23 5 2`. Once `Cell` implements `Index`, `[Cell]Int` is a flat array of `h * w` slots and `for x in Cell` visits every cell, exactly like a domain. The prototype spells the ordinal and its inverse on a domain as `.ord` and `.nth`; they are the same `to_idx` and `from_idx` as above.

`domain`, strings (`Str` is `[domain S]Char`), and permutations are all instances of this one mechanism, which is why one set of array, loop, and slice rules covers them all. The [indices reference](indices.html#the-mechanism-is-index) has the shorter tour.

```exercise index-slices-predict-ordinal
```

## K is frozen at runtime

In a contest, `n` comes from the input, so the size of an index is a runtime value. It still must never change after arrays are built over it, or every array would silently have the wrong length. The rule is to bind it with `let`:

- `let K = n` is correct. `let` is immutable, and it may hold a value read at runtime.
- `const K = n` is rejected. `const` means "known at compile time", and `n` is not.
- `let K = m` where `m` is a `var` is rejected. A `var` can be reassigned later, so the size would no longer describe the variable it came from. Compute the size from immutable `let` values, as `let K = h * w` does above.

The prototype test suite rejects both wrong forms, with messages that name the fix.

```exercise index-slices-frozen-k
```

## Tuple indexing for grids

A grid is an array indexed by a pair. `[Row, Col]T` stores one `T` per pair, and `g[r, c]` is sugar for indexing by the tuple `(r, c)`:

```cp Prototype tested · grid.cp
let h: 1..=9 = read()
let w: 1..=9 = read()
domain Row = 1..=h
domain Col = 1..=w
var g: [Row, Col]Int = 0
for r in Row:
    for c in Col:
        g[r, c] = r.label * 10 + c.label
type Cell = (Row, Col)
let p: Cell = (Row.last, Col.first)
print(g[p], g[Row.last, Col.first], g[p] == g[Row.last, Col.first])
```

For input `2 3` it prints `21 21 YES`. A cell can be stored and passed around as one value `p`, which is what BFS queues want, and the two coordinates keep their own types: `g[c, r]` with the arguments swapped is a type error instead of a transposed read. In C++ that swap is a silent bug on every non-square grid.

## A range is data, a slice is a lend

A range such as `lo..=hi` over `V` is plain data: two positions. You can store it, return it from a function, and compare it. Applying it to an array, `a[range]`, produces a slice: a window onto the array's own storage. The slice keeps the parent's index type, so inside it you still index with `V`, and only positions inside the window are members.

```cp Prototype tested · slices.cp
let n: 1..=9 = read()
domain V = 1..=n
var a: [V]Int = read()

fn widest(lo: V, hi: V) -> Range<V>:
    lo..=hi

let w = widest(V(2), V(4))
let part = a[w]
var best = V(2)
for v in part.positions:
    if part[v] > part[best]:
        best = v
print(best, a[best], part.len, sum(part))
a[w].sort()
print(a)
var keep = a[w].copy()
keep.push(0)
print(keep)
```

For input `5` / `9 2 7 4 5` it prints three lines: `3 7 3 13`, then `9 2 4 7 5`, then `2 4 7 0`. Walk through why:

- `part.positions` yields the parent positions `2`, `3`, `4`, so the best position `3` is valid in both `part` and `a`. There is no "index inside the slice" to translate back.
- `a[w].sort()` sorts only the window in place; elements outside it do not move.
- `a[w].copy()` makes an owned list, which can grow with `push` and outlive the array.

Membership is checked. With `w = V(1)..=V(2)`, the prototype stops `part[V(3)]` with "position is outside the slice". The check disappears when the compiler can prove membership, as it can inside `for v in part.positions`.

```exercise index-slices-slice-membership
```

## What a lend cannot do

A slice borrows the array for the lexical scope in which it is used. The approved rules are deliberately few: a lent value may be read, passed to a call, and iterated, but it may not be stored in an owning field or container, held in a `var`, or returned. If you need the elements later, call `.copy()`. If you need the window later, return the range instead; ranges are data.

```cp CP Lang · rejected in the prototype
fn window(a: [V]Int, lo: V, hi: V) -> [V]Int:
    a[lo..=hi]        // error: a slice cannot be returned; return the range or a copy
```

The prototype also rejects a slice held in a `var` and two overlapping mutable slices of one array. Proving that two mutable projections do not overlap is what `law disjoint` is for; that and the inline-callback model behind `lends` are taught in the [borrowing reference](borrowing.html), and the next course chapter builds them from first principles.

```exercise index-slices-range-or-lend
```

## Common mistakes

**Translating slice positions.** Writing `v - lo` to get an "index inside the slice" is the C++ habit. Slices are indexed by parent positions, so use `v` directly.

**Returning a slice to avoid a copy.** Return the range, and apply it where you use it. Returning the borrow is the one thing that would require lifetime annotations, and the language does not have them.

**Sizing an index from a `var`.** If a size will be used to build arrays, bind it with `let` immediately after reading it.
