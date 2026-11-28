// expect: ok, output "7 3"
// engines: next
use std.literal.{Filled}
fn mk<A: Filled<Int>>() -> A = A.filled(7)
domain P = 0..3
let a: [P]Int = mk()
print(a[P(1)], a.len)
