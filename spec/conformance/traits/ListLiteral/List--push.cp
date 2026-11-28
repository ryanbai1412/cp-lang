// expect: ok, output "2 5"
// engines: next
use std.literal.{ListLiteral}
fn mk<A: ListLiteral<Int>>() -> A:
    var a = A.with_room(0)
    a.push(4)
    a.push(5)
    a
let xs: List<Int> = mk()
print(xs.len, xs[1])
