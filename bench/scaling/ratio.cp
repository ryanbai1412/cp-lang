use std.math.{Ratio}
let n: Int = read()
var acc = 0
for i in 0..8 * n:
    let x = n + i % n
    let y = n + i * 7919 % n
    let r = Ratio(x, y) // timed: math.Ratio.new
    acc += Ratio.from_int(x).floor // timed: math.Ratio.from_int
    acc += r.inv.floor // timed: math.Ratio.inv
    acc += r.floor // timed: math.Ratio.floor
    acc += r.ceil // timed: math.Ratio.ceil
    acc += r.abs.ceil // timed: math.Ratio.abs
print(acc)
