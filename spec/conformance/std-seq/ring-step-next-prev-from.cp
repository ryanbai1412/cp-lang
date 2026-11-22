// expect: ok, output "0 2 2 2 0\n1 2 0"
use std.seq.{ring}
domain V = 0..3
let a: [V]Int = [1, 2, 3]
let r = a.ring
print(r.next(V(2)), r.prev(V(0)), r.step(V(0), -1), r.step(V(1), 7), r.step(V(2), -8))
print(r.from(V(1)).collect())
