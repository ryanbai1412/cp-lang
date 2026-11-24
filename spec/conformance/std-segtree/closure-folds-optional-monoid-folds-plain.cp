// expect: ok, output "1 -1 9 0"
// gap: compiler 2026-11-24T05Z
// rule: 08.1.9 std.segtree.SegTree.all
// A closure op has no identity, so its SegTree folds to `?T` and an empty range is `None`; `+` has one, so its folds stay `Int` (decision 456).
domain V = 0..4
let ws: [V]Int = [3, 1, 4, 1]
let low = SegTree((a: Int, b: Int) => if a <= b: a else: b, ws)
let sum = SegTree(+, ws)
let s: Int = sum.all
print(low.all!, low.fold(V(1)..V(1)) ?? -1, s, sum.fold(V(1)..V(1)))
