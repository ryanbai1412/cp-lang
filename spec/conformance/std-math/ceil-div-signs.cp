// expect: ok, output "-3 4 -3 0"
use std.math.{ceil_div}
print(ceil_div(-7, 2), ceil_div(7, 2), ceil_div(7, -2), ceil_div(0, 5))
