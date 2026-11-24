// expect: ok, output "1 2 7"
use std.persistent.{PersistentArray}
domain P = 0..3
var p = PersistentArray<P, Int>(a: [P] i => i.ord + 1)
let v = p.set(p.base, P(2), 7)
print(v.within(p))
