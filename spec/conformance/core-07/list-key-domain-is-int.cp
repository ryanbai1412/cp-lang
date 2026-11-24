// expect: ok, output "2 2 7"
// rule: 07.3.5
// A list is `Indexed<Int, T>`: a struct built from one over `[D]T` has `D = Int`, the type written.
use std.sparse.{LinearRmq}
struct H:
    r: LinearRmq<Int, Int>
let xs = [5, 3, 1, 4, 1]
let h = H(LinearRmq(xs))
let k: LinearRmq<Int, Int> = LinearRmq<Int, Int>(xs)
domain V = 0..4
let a: [V]Int = [V] v => 10 - Int(v.ord) * 3 % 7
print(h.r.argbest(1..5), k.argbest(0..3), LinearRmq(a).best(V(0)..V(2)))
