Almost every off-by-one bug in a CP solution is a confusion between two kinds of integer: an integer that names an element, and an integer that names a gap between elements. This chapter gives each kind its own type. Prefix sums, range queries, and sliding windows then need no `+ 1` and no `- 1`, and mixing the two kinds stops compiling.

> **What runs.** Domains, `.l`, `.r`, `.l(k)`, `.r(k)`, `V.bounds`, and the prefix-sum and window programs below compile and run in the Python prototype. Where the approved design is stricter than the prototype, the text says so. The [indices reference](indices.html) is the short reference version of this chapter.

## A domain is a fresh type

`domain V = 1..=n` declares a new type whose values are the positions `1` through `n`. It is not an alias for `Int`. Indexing `[V]T` requires a `V`, and a plain integer is rejected:

```cp CP Lang · rejected in the prototype
let n: 1..=9 = read()
domain V = 1..=n
let a: [V]Int = read()
let i = 2
print(a[i])           // error: a is indexed by V, not Int
```

Two domains are different types even when their ranges are equal. A graph statement with `n` vertices and `n` edges is the classic case:

```cp CP Lang · rejected in the prototype
domain Vertex = 1..=n
domain Edge = 1..=n
let deg: [Vertex]Int = 0
let e = Edge.first
print(deg[e])         // error: Edge is not Vertex
```

In C++ both of these are `int`, and swapping a vertex for an edge index compiles, runs, and gives a wrong answer on the one test where the arrays differ. Here the mistake is a compile error that names both types. A position converts to an integer only when you ask: `v.label` is the number from the statement, and `v.idx` is its zero-based offset. Writing `V(3)` goes the other way and checks that `3` is a real position.

```exercise domains-why-reject
```

## Elements and boundaries

For `n` elements there are `n + 1` boundaries: one before everything, one between each adjacent pair, and one after everything. `V.bounds` is the type of those boundaries. Every element `v` has two of them: `v.l` immediately to its left and `v.r` immediately to its right.

```text
elements          1       2       3       4       5
boundaries    |       |       |       |       |       |
              1.l     1.r
                      2.l     2.r
                                              5.l     5.r
```

`1.r` and `2.l` are the same boundary. `V.first.l` is the leftmost boundary, `V.bounds.first`, and `V.last.r` is the rightmost, `V.bounds.last`. Both of those equalities hold in the prototype.

The rule to remember is that a boundary is not an element. An array of per-element values is `[V]T`, and an array of per-boundary values is `[V.bounds]T`. Writing `pre[v]` on the second or `a[v.r]` on the first is a type error, because each array is indexed by exactly one of the two kinds.

## Prefix sums are boundary arrays

A prefix sum stores, at every boundary, the total of everything to its left. The leftmost boundary has nothing to its left, so its value is `0`. Each element then moves the total from its left boundary to its right boundary:

```example bounds
```

`var pre: [V.bounds]Int = 0` fills all `n + 1` slots with zero, and `pre[v.r] = pre[v.l] + a[v]` reads like the definition. There is no `pre[0] = 0` special case, no `pre[i + 1]`, and no question of whether the array has size `n` or `n + 1`: the type has exactly `n + 1` boundaries.

```exercise domains-predict-bounds
```

An inclusive range `l..=r` lies between the boundary left of `l` and the boundary right of `r`, so its sum is the difference of those two prefix values. This complete program answers `q` range-sum queries:

```cp Prototype tested · range-sums.cp
let n: 1..=2e5 = read()
let q: 1..=2e5 = read()
domain V = 1..=n
let a: [V]-1e9..=1e9 = read()
let queries: [q](l: V, r: V) = read()

var pre: [V.bounds]Int = 0
for v in V:
    pre[v.r] = pre[v.l] + a[v]

for (l, r) in queries:
    print(pre[r.r] - pre[l.l])
```

For input `5 3`, `3 1 4 1 5`, then queries `1 5`, `2 3`, and `4 4`, it prints `14`, `5`, and `1`. The queries are read straight into `V`, so each endpoint is already a position; `[q](l: V, r: V)` reads `q` records with two named fields. The single-element query `4 4` needs no special case: `pre[4.r] - pre[4.l]` is `a[4]`.

```exercise domains-range-formula
```

## Windows count their own element

Many problems ask about every window of exactly `k` consecutive elements. `v.l(k)` is the left edge of the `k`-element window that ends at `v`, and `v.r(k)` is the right edge of the `k`-element window that starts at `v`. Both windows count `v` itself, so `v.l(1)` is `v.l` and `v.r(1)` is `v.r`.

This is the decision that removes the classic off-by-one. In C++ the window of length `k` ending at index `i` begins at `i - k + 1`, and the prefix difference is `pre[i + 1] - pre[i - k + 1]`. Here it is `pre[v.r] - pre[v.l(k)]`, and both edges are boundaries by construction.

```cp Prototype tested · best-window.cp
let n: 1..=2e5 = read()
let k: 1..=n = read()
domain V = 1..=n
let a: [V]-1e9..=1e9 = read()

var pre: [V.bounds]Int = 0
for v in V:
    pre[v.r] = pre[v.l] + a[v]

var best = pre[V(k).r] - pre[V.first.l]
for v in V(k)..=V.last:
    best = max(best, pre[v.r] - pre[v.l(k)])
print(best)
```

For `5 2` and `3 -1 4 1 5` it prints `6`, from the window `1 5`. With `k = 5` the only window is the whole array and it prints `12`; with `n = 3`, `k = 1`, and `-2 -9 -4` it prints `-2`. Note how the loop starts at `V(k)`, the first element that has `k - 1` elements before it, so `v.l(k)` always exists. The first window seeds `best`, so negative answers are handled without a sentinel.

A window that would run past the edge is an error, not a silent read of garbage. In the prototype, `V(2).l(3)` on `1..=n` stops the program with a range error, because there are not three elements ending at position `2`.

```exercise domains-window-edge
```

## Moving between positions

A position is not an integer, so the approved design rejects `v + 1`: there is no meaningful "position plus number" until you say what should happen at the edge. Say it with a method instead:

- `v.next` and `v.prev` step to a neighbor. At the last or first element they have nothing to return, so use them with `if let` (the [optional example](tour.html#missing-is-a-value) shows the pattern).
- `v.offset(d)` jumps `d` positions in either direction and also returns an optional, so the edge case is visible at the call site.
- `v.cyc(k)` wraps around, for circular arrays. It is approved design; the prototype does not implement it yet.

One honest caveat: the prototype currently still accepts `v + 1` on a domain value. Do not rely on it; the approved rule is that it is an error whose message suggests `.next`, `.offset(1)`, or `.cyc(1)`.

## Common mistakes

**Indexing a prefix array by an element.** `pre[v]` does not mean "prefix through `v`" or "prefix before `v`": it does not compile. Choose the boundary you mean, `v.l` or `v.r`, and the ambiguity is gone.

**Counting the window wrong by one.** `v.l(k)` already counts `v`. Writing `v.l(k + 1)` "to include `v`" makes every window one element too long.

**Forgetting the empty prefix.** The leftmost boundary is a real slot holding `0`; do not start the build loop from the second element. The loop `for v in V` with `pre[v.r] = pre[v.l] + a[v]` is the whole construction.

**Shifting labels by hand.** If the statement uses positions `0..n`, write `domain V = 0..n` instead of reading into `1..=n` and subtracting one. `.label` is then the statement's number, and `.idx` is still the zero-based offset.

Next, [Index and slices](course-index-slices.html) shows that a domain is one instance of a general mechanism, `Index`, and how to take a typed window out of an array without copying it.
