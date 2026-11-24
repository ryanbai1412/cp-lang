// expect: ok, output "2 24 24 4"
use std.prefix.{prefix_fold}
let p = prefix_fold(*, [2, 3, 4])
let s = [2, 3, 4].rev.scan(*)
print(p[1], p[3], s.cut(0), s.cut(2))
