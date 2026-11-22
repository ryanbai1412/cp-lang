// expect: ok, output "9 0"
use std.rand.{Rng}
var r = Rng(2)
var a = [9]
r.shuffle(mut a)
var e: List<Int> = []
r.shuffle(mut e)
print(a[0], e.len)
