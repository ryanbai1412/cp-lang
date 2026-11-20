Data structures in CP Lang 0.2.2 do not hide behind wrapper types or class hierarchies. Range structures take either a bare associative operator (`+`, `*`, `^`, `&`, `|`, `min`, `max`, `gcd`) or a lightweight `Monoid(id, op)` value, lazy segment trees take an `Action(id, compose, apply, over)`, and tree path queries live directly on `Rooted<V>`.

## Fenwick trees (`Fenwick`)

For invertible range sums (`+` has inverse `-`), `Fenwick<V, Int>(a)` is lighter and faster than a full segment tree while keeping the same domain-typed interface:

```example fenwick_sum
```

```exercise algebra-fenwick-output
```

## Bare operators and `Monoid(id, op)`

A segment tree only needs two things from your combine operation: an **identity** `id` (so `op(id, x) == x == op(x, id)` and empty ranges return `id`) and **associativity** (`op(op(a, b), c) == op(a, op(b, c))`, so the tree can regroup subranges freely). It does **not** require commutativity.

For built-in operators, pass the operator directly (`SegTree<V, Int>(+, a)` or `SegTree<V, Int>(max, a)`). For any custom node combine, build a `Monoid<T>(id, op)` from `std.algebra` and pass it to `SegTree<V, T>`:

```example segtree_monoid
```

```exercise algebra-segtree-output
```

For static idempotent queries (`min`, `max`, `gcd`), `SparseTable<V, Int>(min, a)` answers queries in `O(1)`. Custom segment-tree nodes such as `(best, count)` or maximum subarray sum pass `Monoid(id, op)` the exact same way:

```cp CP Lang 0.2.2 · spec fragment
let best_op = Monoid(
    id: (0, 0),
    op: (x, y) => if x.0 != y.0: max(x, y) else: (x.0, x.1 + y.1),
)
var st = SegTree<V, (Int, Int)>(best_op, init)
```

For mixed-order heaps or sorts, wrap descending keys with `desc(x)` (`MinHeap<(Int, Desc<V>)>`).

## Lazy propagation with `Action`

A lazy segment tree pairs a value monoid `over` with a tag monoid `F` using `Action(id, compose, apply, over)`. The argument order is fixed so you never guess which tag came first:

- `id`: the no-op lazy tag.
- `compose(new_f, old_f)`: applies `old_f` first, then `new_f`.
- `apply(f, val, span)`: applies tag `f` to a node summary `val` covering `span` leaves.
- `over`: the underlying value `Monoid`.

Here is a complete range-add, range-sum `LazySegTree` running in real WebAssembly:

```example lazy_action
```

```exercise algebra-lazy-output
```

When updates are non-commutative—such as affine range updates `x -> mul * x + add`—`compose(new_f, old_f)` applies `old_f` first and `new_f` second:

```cp CP Lang 0.2.2 · spec fragment
let aff = Action(
    (1, 0),
    (new_f, old_f) => (new_f.0 * old_f.0, new_f.0 * old_f.1 + new_f.1),
    (f, val, span) => f.0 * val + f.1 * span,
    over: +,
)
var affine = LazySegTree<V, Int>(aff, a)
```

> **WASM status.** Both `SegTree<V, Int>(+, a)` and stock `LazySegTree<V, Int>(AddSum, a)` work directly from the prelude in WASM. Custom `Monoid<T>`, `Action<T, F>`, and `LazySegTree<D, T, F>` also work in WASM via `use std.algebra.{Monoid, Action}` and `use std.lazy.{LazySegTree}` with explicit type arguments (as in `segtree_monoid.cp` and `lazy_action.cp`).

## Sliding windows and `Rooted<V>`

0.2.2 keeps two-pointer windows as a plain 3-line loop (no callback helper) so you can maintain arbitrary incremental state:

```cp CP Lang 0.2.2 · spec fragment
var l = 0
for r in 0..n:
    while pre.sum(l..=r) > budget: l += 1
    best max= r - l + 1
```

For trees, `Rooted<V>(g, root)` unifies DFS order (`t.Pre`), LCA, subtree ranges (`t.sub(u)`), virtual trees, and heavy-light path queries (`fold_path` / `apply_path`) into one type:

```cp CP Lang 0.2.2 · spec fragment
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

## Write a program

```exercise algebra-write-range-min
```
