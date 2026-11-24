// expect: ok, output "4 1\n9 1"
// rule: std.algebra.opt
// Two closures lifted with `opt()` in one file stay two ops (adversarial round 47).
use std.algebra.{opt}
let hi = (a: Int, b: Int) => if a >= b: a else: b
let lo = (a: Int, b: Int) => if a <= b: a else: b
let xs = [Some(4), None, Some(1)]
print(xs.fold(hi.opt())!, xs.fold(lo.opt())!)
print([Some(9)].fold(hi.opt())!, [None, Some(1)].fold(lo.opt())!)
