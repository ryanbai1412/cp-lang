// expect: ok, output "2 24 24 4"
use std.prefix.{prefix_fold, suffix_fold}
let p = prefix_fold(*, [2, 3, 4])
let s = suffix_fold(*, [2, 3, 4])
print(p[1], p[3], s[0], s[2])
