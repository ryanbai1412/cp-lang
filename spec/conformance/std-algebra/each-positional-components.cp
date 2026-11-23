// expect: ok, output "12 5 1"
// `each(+, max, min)` is one op on tuples, one component per op (lib/ops-ranges.md §2).
use std.algebra.{each}
domain V = 0..4
let a: [V]Int = [3, 5, 1, 3]
var st = SegTree(each(+, max, min), a.map(x => (x, x, x)))
let (s, hi, lo) = st.fold(V.first!..=V.last!)
print(s, hi, lo)
