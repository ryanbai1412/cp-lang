Data structures in CP Lang v2 do not hide behind wrapper types or inheritance hierarchies. Range structures take bare operators (`+`, `*`, `^`, `&`, `|`, `min`, `max`, `gcd`) or a lightweight `Monoid(id, op)` value, lazy segment trees take a stock tag or `Action(id, compose, apply)`, and tree path queries live directly on `Rooted<V>`.

## Bare operators and `Monoid(id, op)`

Whenever a range structure only needs an associative operation with an identity element, pass the bare operator directly:

```cp CP Lang v2
var bit = Fenwick<V, Int>(a)            // sum-only: .add(i, d), .sum(l..=r)
var rmq = SparseTable<V, Int>(min, a)   // O(1) idempotent .fold(l..=r)
var st = SegTree<V, Int>(+, a)          // bare operator
```

`Fenwick` is intentionally sum-only; use `SegTree` for arbitrary associative operations. When you need a custom node type—like maximum subarray sum or `(best, count)`—pass `Monoid(id, op)`:

```cp CP Lang v2
let best_op = Monoid(
    id: (0, 0),
    op: (x, y) => if x.0 != y.0: max(x, y) else: (x.0, x.1 + y.1),
)
var st = SegTree<V, (Int, Int)>(best_op, init)
```

For mixed-order sorting or priority queues, wrap descending fields with `desc(x)`:

```cp CP Lang v2
var pq = MinHeap<(Int, Desc<V>)>()
pq.push((d, desc(u)))
```

## Lazy propagation with `Action`

`LazySegTree` ships stock tags (`AddSum`, `AddMin`, `AddMax`, `AssignSum`, `AssignMin`, `AssignMax`) and accepts custom lazy updates via `Action(id, compose, apply)`:

```cp CP Lang v2
var lz = LazySegTree<V, Int>(AddSum, a)
lz.apply(l..=r, delta)
let total = lz.fold(l..=r)

let aff = Action(
    id: (1, 0),
    compose: (outer, inner) => (outer.0 * inner.0, outer.0 * inner.1 + outer.1),
    apply: (tag, val, len) => tag.0 * val + tag.1 * len,
)
var affine = LazySegTree<V, Int>(aff, a)
```

Both `SegTree` and `LazySegTree` support `.max_right(from, ok)` and `.min_left(to, ok)` for O(log n) binary search on the tree.

## Sliding windows: `MonoDeque` and `slide_until`

Two focused helpers replace ad-hoc window wrappers:

```cp CP Lang v2
var q = MonoDeque<Int>(max)
for (i, x) in a.items:
    q.push(i, x)
    q.drop_before(i - k + 1)

for (l, r) in slide_until(0..n, (l, r) => pre.sum(l..=r) <= budget):
    best max= r - l + 1
```

## Tree paths with `Rooted<V>`

`Rooted<V>(g, root)` unifies DFS order, lazy LCA, subtree ranges (`tree.sub(u)`), virtual trees, and heavy-light path queries into one type:

```cp CP Lang v2
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
