// expect: ok, output "3 2\n4 2 4 1\n0"
// rule: std.algebra.MaxCount
// `MaxCount` has `+` (its `join`) and the identity `zero`, so a fold or a `SegTree` takes `+` alone.
use std.algebra.{MaxCount}
use std.segtree.{SegTree}
let m = [3, 1, 3].map(v => MaxCount.of(v)).fold(+)
print(m.val, m.count)
domain D = 0..4
let t = SegTree(+, [D] i => MaxCount.of([1, 4, 2, 4][i.ord]))
let r = t.fold(D(0)..D(3))
print(t.all.val, t.all.count, r.val, r.count)
print(MaxCount<Int>.zero.count)
