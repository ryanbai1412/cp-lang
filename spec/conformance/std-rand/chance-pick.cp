// expect: ok, output "YES"
use std.rand.{Rng}
var r = Rng(3)
let a = r.chance(1.0) and not r.chance(0.0)
var s = Rng(1)
let e: List<Int> = []
print(yes(a and s.pick(e) == None))
