// expect: ok, output "5/6 -1/6 1/6 2/3 -1/2 YES"
use std.math.{Ratio}
let a = Ratio(1, 2)
let b = Ratio(1, 3)
print(a + b, b - a, a * b, b / a, -a, yes(b < a and not (a < b)))
