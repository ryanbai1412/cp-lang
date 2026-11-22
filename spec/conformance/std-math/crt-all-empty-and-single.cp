// expect: ok, output "YES YES"
use std.math.{crt_all}
let none: List<(Int, Int)> = []
print(yes(crt_all(none) == Some((0, 1))), yes(crt_all([(13, 9)]) == Some((4, 9))))
