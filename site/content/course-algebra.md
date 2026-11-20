Data structures in CP Lang 0.2.2 do not hide behind wrapper types or inheritance hierarchies. Range structures take bare operators (`+`, `*`, `^`, `&`, `|`, `min`, `max`, `gcd`) or a lightweight `Monoid(id, op)` value, lazy segment trees take a stock tag or `Action(id, compose, apply, over: op)`, and tree path queries live directly on `Rooted<V>`.

## Bare operators and `Monoid(id, op)`

Whenever a range structure only needs an associative operation with an identity element, pass the bare operator directly:

```cp CP Lang 0.2.2
var bit = Fenwick<V, Int>(a)            // sum-only: .add(i, d), .sum(l..=r)
var rmq = SparseTable<V, Int>(min, a)   // O(1) idempotent .fold(l..=r) -> Int
var st = SegTree<V, Int>(+, a)          // bare operator
```

`Fenwick` is intentionally sum-only; use `SegTree` for arbitrary associative operations. When you need a custom node type—like maximum subarray sum or `(best, count)`—pass `Monoid(id, op)`:

```cp CP Lang 0.2.2
let best_op = Monoid(
    id: (0, 0),
    op: (x, y) => if x.0 != y.0: max(x, y) else: (x.0, x.1 + y.1),
)
var st = SegTree<V, (Int, Int)>(best_op, init)
```

For mixed-order sorting or priority queues, wrap descending fields with `desc(x)`:

```cp CP Lang 0.2.2
var pq = MinHeap<(Int, Desc<V>)>()
pq.push((d, desc(u)))
```

## Lazy propagation with `Action`

`LazySegTree` ships stock tags (`AddSum`, `AddMin`, `AddMax`, `AssignSum`, `AssignMin`, `AssignMax`) and accepts custom lazy updates via `Action(id, compose, apply, over: op)`. It takes two type arguments, `LazySegTree<D, T>`, and infers the tag type. The argument order is fixed: `compose(new_f, old_f)` applies `old_f` first and then `new_f`, and `apply(f, val, span)` updates a node covering `span` elements:

```cp CP Lang 0.2.2
var lz = LazySegTree<V, Int>(AddSum, a)
lz.apply(l..=r, delta)
let total = lz.fold(l..=r)

let aff = Action(
    (1, 0),
    (new_f, old_f) => (new_f.0 * old_f.0, new_f.0 * old_f.1 + new_f.1),
    (f, val, span) => f.0 * val + f.1 * span,
    over: +,
)
var affine = LazySegTree<V, Int>(aff, a)
```

Both `SegTree` and `LazySegTree` have `.all`, an O(1) fold of the whole domain, and `.max_right(l, ok)` / `.min_left(r, ok)` for O(log n) binary search on the tree.

## Sliding windows: an explicit loop

0.2.2 has no window helper in the core. A two-pointer window is three lines, and unlike a callback it can hold incremental state:

```cp CP Lang 0.2.2
var l = 0
for r in 0..n:
    while pre.sum(l..=r) > budget: l += 1
    best max= r - l + 1
```

A monotone queue for sliding minima (`MonoDeque`) is ordinary userland code in `examples/local-lib/`, not a core type.

## Tree paths with `Rooted<V>`

`Rooted<V>(g, root)` unifies DFS order, lazy LCA, subtree ranges (`tree.sub(u)`), virtual trees, and heavy-light path queries into one type:

```cp CP Lang 0.2.2
let tree = Rooted(g, root)
var seg = LazySegTree<tree.Pre, Int>(AddSum, 0)

tree.apply_path(a, b, mut seg, delta)
let path_ans = tree.fold_path(a, b, seg)
let sub_ans = seg.fold(tree.sub(u))
```

## Practice

```exercise algebra-sum-identity
```

```exercise algebra-assoc
```

```exercise algebra-inverse
```

```exercise algebra-noncommutative
```

```exercise algebra-runtime-op
```

```exercise algebra-local-library
```
