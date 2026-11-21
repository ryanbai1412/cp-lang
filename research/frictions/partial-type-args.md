# Partial type arguments do not infer the rest

Not a port: found writing `bench/trees/lazy_lct_bench.cp`. Owner: core
(`cpl-check`).

## A trailing type argument left out is not inferred
- **Where:** `LazyLinkCut<V, Int>.new(AddSum, a)` (`std.tree`) and
  `LazySegTree<V, Int>.new(AddSum)` (`std.lazy`); both types take
  `<D, T, F>` and `new(act: Action<T, F>, ...)`.
- **Repro:**
  ```cp
  use std.lazy.{LazySegTree, AddSum}
  let n: 1..=10 = read()
  domain V = 0..n
  var st = LazySegTree<V, Int>.new(AddSum)
  ```
  gives `error[E0302]: expected Action<Int, _>, found Action<Int, Int>`.
- **What felt wrong:** the message shows `_` as the hole and still rejects
  the one value that fills it. Writing every argument
  (`LazySegTree<V, Int, Int>`) or none (`LazySegTree.new(AddSum, a)`) works,
  so the contestant must know which.
- **Goal it hurts:** express and revise; failures easy to explain.
- **Ideal code:** `LazySegTree<V, Int>.new(AddSum)`, the tag type inferred
  from `AddSum`, as the domain and value are written once.
- **General fix:** core: a type path with fewer arguments than parameters
  binds the rest as inference variables and unifies them with the call.
  Until then E0302 should name the missing parameter (`F`) and the fix.
- **Severity:** low.
