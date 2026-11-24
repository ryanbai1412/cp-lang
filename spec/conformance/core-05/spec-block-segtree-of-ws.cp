// expect: ok, output "2"
// rule: 05.7.10
// input: "5 1 4 2\n"
use std.algebra.opt
domain V = 1..=4
let ws = read<[V]Int>()
let lighter = (i: V, j: V) => if ws[i] <= ws[j]: i else: j
var st = SegTree(lighter.opt(), [V] v => Some(v))   // fine: st dies before ws
print(st.fold(V.range)!)
