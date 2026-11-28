// expect: ok, output "-1 0"
// engines: next
use std.algebra.{MaxSub}
fn z<T: Zero>() -> T = T.zero
let m: MaxSub = z()
print(m.try_best ?? -1, m.total)
