// n: 15
// let: k = 2
use std.math.{bit_subsets}
let n: Int = read()
var acc = 0
for _ in 0..16384:
    for s in bit_subsets(n, 2): acc ^= s // timed: math.bit_subsets
print(acc)
