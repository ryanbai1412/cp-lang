// expect: ok, output "0"
// engines: next
use std.literal.{ListLiteral}
fn mk<A: ListLiteral<Int>>() -> A = A.with_room(3)
let xs: List<Int> = mk()
print(xs.len)
