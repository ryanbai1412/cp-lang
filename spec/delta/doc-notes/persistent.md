# persistent doc notes

- `PersistentSegTree.new`, `PersistentLazySegTree.new`: contradiction. `spec/lib/ops-ranges.md` lists one `PersistentSegTree(op, a, act:)`; the stub splits out `PersistentLazySegTree(act:)`. Followed the stub.
- `PMultiset`: contradiction. `spec/lib/containers.md` lists `PMultiset` as deferred until a second user; the stub defines it. Documented the stub.
- `PersistentSegTree.new`, `PersistentLazySegTree.new`: chose a trap when both `a:` and `fill:` are given; neither gives the identity.
- `PersistentArray.new`: chose a trap when both or neither of `a:` and `fill:` are given (there is no identity to fall back on).
- every method taking a `Version`: chose "traps when `v` did not come from this tree"; a `Version` is an untyped root index, so an implementation can only catch indices out of range. Flagged: the handle could carry its tree's identity.
- `PersistentSegTree.right_while`, `left_while`: chose the `SegTree` rules (monotone `ok`, trap when `ok(identity)` is false, rightmost/leftmost cut).
- `PersistentSegTree.diff_kth`: contradiction in convention. It is 1-based per the stub while `RangeKth.kth` and `WaveletMatrix.kth` are 0-based; kept 1-based, chose a trap on `k < 1`, and that it assumes a count tree under `+`.
- `RangeKth` costs: the stub mixes O(log n) and O(log |D|); documented O(log n) for n = `a.len` (the tree is over distinct values).
- `RangeKth.count_in`: chose 0 when `lo_v > hi_v`.
- `PersistentLazySegTree.set`: chose that the set value replaces any tags pending above position `i`.
- `PersistentSegTree.reserve`: chose a trap on negative `nodes`.
- `PMultiset.without`: chose to return an equal multiset when `x` is absent.
- `PMultiset.cmp`: the stub says "lexicographic"; chose comparison of the sorted element sequences (ascending, or descending with `desc: true`) where a proper prefix is smaller.
- `PMultiset.with`, `without`, `count`: chose a trap when `x` is not in `D`.
