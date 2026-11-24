// n: 16384
// let: size = n
use std.rand.{Rng}
use std.tree.{cartesian_tree}
let n: Int = read()
domain I = 0..n
var rng = Rng(11)
let arr: [I]Int = [I] i => rng.int(0, 1000000)
var s = 0
for _ in 0..200:
    s += if cartesian_tree(arr)[I(0)] == None: 0 else: 1 // timed: tree.cartesian_tree
print(s)
