// The cpl-rt bit intrinsics: one instruction each, whatever the value.
use std.math.{ctz, clz, bit_width, bit_ceil}
let n: Int = read()
var s = 0
for i in 0..64 * n:
    let x = i * 7919 % n + 1
    s += ctz(x) // timed: math.ctz
    s += clz(x) // timed: math.clz
    s += bit_width(x) // timed: math.bit_width
    s += bit_ceil(x) // timed: math.bit_ceil
print(s)
