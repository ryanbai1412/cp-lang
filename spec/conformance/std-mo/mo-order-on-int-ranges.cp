// expect: ok, output "2 0 1 3\n0\n3 5"
// rule: std.mo.mo_order
// `mo_order` and `mo` take plain `Range<Int>` queries, not only ranges of a declared domain.
use std.mo.{mo_order, mo}

print(mo_order([0..3, 2..4, 0..1, 3..4]))
print(mo_order([5..5]))
let a = [1, 2, 3]
var s = 0
print(mo([0..2, 1..3], add: i => s += a[i], remove: i => s -= a[i], answer: _ => s))
