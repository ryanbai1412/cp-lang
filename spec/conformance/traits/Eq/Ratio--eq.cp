// expect: ok, output "YES YES NO"
use std.math.{Ratio}
print(yes(Ratio(2, 4) == Ratio(1, 2)), yes(Ratio(3, -6) == Ratio(-1, 2)), yes(Ratio(1, 3) == Ratio(1, 2)))
