// expect: ok, output "YES"
use std.rand.{Rng}
var r = Rng(5)
var ok = true
for _ in 0..100:
    let x = r.float()
    if x < 0.0 or x >= 1.0: ok = false
print(yes(ok))
