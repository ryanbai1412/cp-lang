// Matched benchmark: std.poly compose(f, g, n), n = 100,001, 1 round, against ecnerwala's series.

use std.poly.{compose, pow}
type Z = Mod<998_244_353>
let n = 100_001
let reps = 1
var s = 88_172_645
fn next() -> Z:
    s = (s * 1_103_515_245 + 12_345) % 2_147_483_648
    Z(s % 998_244_353)
var f = [next() for _ in 0..n]
var g = [next() for _ in 0..n]
var c = [next() for _ in 0..n]
g[0] = Z(0)
c[0] = Z(1)
var acc = Z(0)
for r in 0..reps:
    f[0] += acc
    c[1] += acc
    let x = compose(f, g, n)
    acc += x[r % n]
print(acc)
