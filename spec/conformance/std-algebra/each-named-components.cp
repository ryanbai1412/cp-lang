// expect: ok, output "12 5"
// `each(sum: +, hi: max)` is one op on records with those fields (lib/ops-ranges.md §2, decision 323).
use std.algebra.{each}
domain V = 0..4
let a: [V]Int = [3, 5, 1, 3]
var nm = SegTree(each(sum: +, hi: max), a.map(x => (sum: x, hi: x)))
let f = nm.fold(V.first!..=V.last!)
print(f.sum, f.hi)
