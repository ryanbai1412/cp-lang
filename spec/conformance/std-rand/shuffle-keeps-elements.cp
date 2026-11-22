// expect: ok, output "1 2 3 4"
use std.rand.{Rng}
var r = Rng(7)
var a = [4, 3, 2, 1]
r.shuffle(mut a)
print(a.sorted())
