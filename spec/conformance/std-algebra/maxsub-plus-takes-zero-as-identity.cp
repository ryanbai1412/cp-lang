// expect: ok, output "4 -1\n6"
// rule: std.algebra.MaxSub std.algebra.Monoid.of
// A record with `+` brings its identity as `zero`, so `+` alone names MaxSub's monoid: `fold(+, ..)`, `Monoid.of(+)` and `SegTree(+, ..)` all find `MaxSub.zero`.
use std.algebra.{MaxSub, Monoid, fold}
use std.segtree.{SegTree}
let xs = [MaxSub.of(v) for v in [2, -5, 3, 1]]
print(fold(+, xs).try_best!, Monoid<MaxSub>.of(+).id.try_best ?? -1)
domain D = 0..4
let t = SegTree(+, [D] i => MaxSub.of([3, -1, 4, -9][i.ord]))
print(t.all.try_best!)
