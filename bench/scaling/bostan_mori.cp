// n: 1024
use std.poly.{bostan_mori}
type Z = Mod<998_244_353>
let n: Int = read()
let p = [Z(i * 7 + 1) for i in 0..n]
let q = [Z(i * 13 + 5) for i in 0..n]
var acc = 0
for _ in 0..2:
    acc ^= bostan_mori(p, q, n * 1000).v // timed: poly.bostan_mori
print(acc)
