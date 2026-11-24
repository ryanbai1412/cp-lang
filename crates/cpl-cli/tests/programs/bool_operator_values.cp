// `and` and `or` alone are operator values over Bool: Monoid identities, folds and a SegTree op.
use std.algebra.{Monoid, each}
print(Int(Monoid<Bool>.of(or).id), Int(Monoid<Bool>.of(and).id))
print(Int([true, false].fold(and)), Int([false, true].fold(or)))
let g = SegTree(or, [false, true, false])
print(Int(g.all))
