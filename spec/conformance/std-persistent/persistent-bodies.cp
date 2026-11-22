// expect: ok, output "YES YES YES YES\n9 1 3\n7\n6\n26 6\n7\n7 3 2"
use std.persistent.{PersistentSegTree, PersistentArray, PersistentLazySegTree}
use std.algebra.{Action, Monoid}
domain N = 0..4
domain M = 0..3
let z: [N]Int = [0, 0, 0, 0]
var t = PersistentSegTree<N, Int>(+, a: z)
let v1 = t.set(t.base, N(1), 1)
let v2 = t.set(v1, N(3), 2)
print(t.diff_kth(v2, t.base, 1) == Some(N(3)), t.diff_kth(v2, t.base, 3) == None, t.diff_kth(v2, v1, 0) == Some(N(3)), t.diff_kth(v2, t.base, 0) == Some(N(1)))
let a3: [M]Int = [1, 2, 3]
var p = PersistentArray<M, Int>(a: a3)
let w = p.set(p.base, M(0), 9)
print(p.get(w, M(0)), p.get(p.base, M(0)), p.get(p.base, M(2)))
var q = PersistentArray<M, Int>(fill: 7)
print(q.get(q.base, M(0)))
let act = Action<Int, Int>(0, (f, g) => f + g, (d, x, span) => x + d * span, Monoid<Int>(0, (a, b) => a + b))
var l = PersistentLazySegTree<M, Int, Int>(act, a: a3)
print(l.all(l.base))
let lv = l.apply(l.base, M(0)..M(2), 10)
print(l.all(lv), l.all(l.base))
let lw = l.set(l.apply(l.base, M(0)..=M(2), 1), M(0), 0)
print(l.all(lw))
let lx = l.apply(l.base, M(1)..=M(2), 1)
print(l.fold(lx, M(1)..=M(2)), l.get(lx, M(1)), l.get(l.base, M(1)))
