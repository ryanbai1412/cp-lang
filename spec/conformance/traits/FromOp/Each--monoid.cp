// expect: ok, output "3 5"
// engines: next
use std.algebra.{each}
let (n, hi) = [(1, 5), (2, 3)].fold(each(+, max))
print(n, hi)
