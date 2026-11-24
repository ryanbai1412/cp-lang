// The intrinsic-bodied number fns: each call is O(1) at any n.
let n: Int = read()
var s = 0
var t = 0.0
for i in 0..8 * n:
    let x = F64(i % 1000) / 7.0 - 70.0
    let k = i % 2001 - 1000
    s += k.abs // timed: math.abs
    t += F64(i % 1000).sqrt // timed: math.sqrt
    s += x.floor // timed: math.floor
    s += x.ceil // timed: math.ceil
    s += x.round // timed: math.round
    s += x.trunc // timed: math.trunc
    let u = F64(i % 1000) / 1000.0
    t += sin(x) // timed: math.sin
    t += cos(x) // timed: math.cos
    t += tan(u) // timed: math.tan
    t += asin(u) // timed: math.asin
    t += acos(u) // timed: math.acos
    t += atan2(x, 3.0) // timed: math.atan2
    t += hypot(x, u) // timed: math.hypot
    t += exp(u) // timed: math.exp
    t += ln(u + 1.0) // timed: math.ln
print(s, t.floor)
