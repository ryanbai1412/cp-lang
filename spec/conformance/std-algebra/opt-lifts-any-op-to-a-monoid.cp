// expect: ok, output "1 3 1\n12 3"
// rule: std.algebra.opt std.algebra.Semigroup.opt
// `op.opt()` lifts an op with no identity to `?T` with `None` as identity, so a closure builds a SegTree whose folds say "nothing here" as `None` (ops-ranges.md section 2).
use std.algebra.{Semigroup, opt}
domain V = 0..4
let ws: [V]Int = [3, 1, 4, 1]
let lighter = (i: V, j: V) => if ws[i] <= ws[j]: i else: j
var st = SegTree(lighter.opt(), [V] v => Some(v))
print(st.all!, st.fold(V(2)..)!, Int(st.fold(V(2)..V(2)) == None))
let cat = (a: Int, b: Int) => a * 10 + b
print(fold(cat.opt(), [Some(1), None, Some(2)])!, fold(Semigroup((a: Int, b: Int) => a - b).opt(), [Some(5), Some(2)])!)
