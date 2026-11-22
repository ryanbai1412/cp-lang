// expect: ok, output "2"
// rule: 05.7.10
// input: "5 1 4 2\n"
domain V = 1..=4
use std.segtree.{SegTree}
let ws = read<[V]Int>()
var st = SegTree((i, j) => if ws[i] <= ws[j]: i else: j, [V] v => v)   // fine: st dies before ws
print(st.fold(V.range))
