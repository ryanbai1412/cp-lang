// n: 1024
use std.math.{BigInt}
let n: Int = read()
var acc = 0
var s = ""
for i in 0..n: s += "7"
var t = ""
for i in 0..n / 2: t += "3"
for i in 0..64 * n:
    acc += BigInt.from_int(i).to_str().len // timed: math.BigInt.from_int
let a = BigInt.from_str(s)
let b = BigInt.from_str(t)
for _ in 0..64 * n:
    if a.abs() > b: acc += 1 // timed: math.BigInt.abs
for _ in 0..1024:
    acc += BigInt.from_str(s).to_str().len // timed: math.BigInt.from_str
    acc += a.to_str().len // timed: math.BigInt.to_str
for _ in 0..8:
    let (q, r) = a.divmod(b) // timed: math.BigInt.divmod
    acc += q.to_str().len + r.to_str().len
    acc += BigInt.gcd(a, b).to_str().len // timed: math.BigInt.gcd
print(acc)
