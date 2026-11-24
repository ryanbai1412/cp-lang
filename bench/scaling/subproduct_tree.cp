// n: 2048
use std.poly.{SubproductTree}
type Z = Mod<998_244_353>
let n: Int = read()
var acc = 0
let xs = [Z(i * 3 + 1) for i in 0..n]
let f = [Z(i * 7 + 2) for i in 0..n]
let t = SubproductTree(xs)
for _ in 0..4:
    acc ^= SubproductTree(xs).n // timed: poly.SubproductTree.new
    acc ^= t.eval(f).len // timed: poly.SubproductTree.eval
    acc ^= t.eval_transposed(f).len // timed: poly.SubproductTree.eval_transposed
    acc ^= t.interpolate(f).len // timed: poly.SubproductTree.interpolate
print(acc)
