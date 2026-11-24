// expect: ok, output "2"
// rule: 08.2.2
// `SegTree(Monoid(None, best), a)`: the monoid's `None` leaves its `T` half known, so `M: Op<T>` waits and `a` fixes `T = ?Int` (G7.3).
use std.algebra.{Monoid}
use std.segtree.{SegTree}
domain P = 0..3
fn best(x: ?Int, y: ?Int) -> ?Int = if x == None: y elif y == None: x else: min(x!, y!)
let a: [P]?Int = [P] p => if p.ord == 0: None else: p.ord + 1
let t = SegTree(Monoid(None, best), a)
let r: ?Int = t.fold(P(0)..=P(2))
print(r!)
