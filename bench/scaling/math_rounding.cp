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
print(s, t.floor)
