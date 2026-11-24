// expect: ok, output "YES NO"
use std.math.{Ratio}
print(yes(Ratio(1, 3) < Ratio(1, 2)), yes(Ratio(2, 4) < Ratio(1, 2)))
