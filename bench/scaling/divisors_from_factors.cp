// n: 4096
use std.math.{divisors_from_factors}
let n: Int = read()
let ps = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47]
var e = List.filled(ps.len, 0)
var d = 1
while d < n:
    var best = 0
    var bv = -1
    for i in 0..ps.len:
        var v = 1
        for _ in 0..e[i] + 1: v *= ps[i]
        if bv < 0 or v < bv:
            best = i
            bv = v
    e[best] = 2 * e[best] + 1
    d *= 2
var fs: List<(Int, Int)> = []
for i in 0..ps.len:
    if e[i] > 0: fs.push((ps[i], e[i]))
var acc = 0
for _ in 0..64:
    acc += divisors_from_factors(fs).len // timed: math.divisors_from_factors
print(acc)
