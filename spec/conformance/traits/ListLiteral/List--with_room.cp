// expect: ok, output "0"
use std.literal.{ListLiteral}
fn mk<A: ListLiteral<Int>>() -> A = A.with_room(3)
let xs: List<Int> = mk()
print(xs.len)
