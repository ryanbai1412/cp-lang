// expect: ok, output "0 0 1 1 1"
// Two flips cancel and one flip shows: the stock flip tag is a parity, and flips commute.
use std.algebra.{Action}
use std.lazy.{Flip}
fn b(x: Bool) -> Int = if x: 1 else: 0
let f = Action<Flip<Int>, Bool>.of(flip, Flip<Int>.op(+))
print(b(f.id), b(f.compose(true, true)), b(f.compose(true, false)), b(f.commutes), f.apply(true, Flip(3, 1), 4).now)
