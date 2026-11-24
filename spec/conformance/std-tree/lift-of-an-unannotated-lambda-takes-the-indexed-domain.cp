// expect: ok, output "3"
// rule: 08.2.11
// A struct's static fn infers its generics from a lambda as a generic fn does: `Lift.of(v => a[v])` leaves `v` unwritten, and `V` is the domain the body indexes with it, not a default `Int`.
use std.tree.{Lift}
domain V = 0..8
let a = [V] v => if v.idx == 0: None else: Some(V(v.idx - 1))
let lo = Lift.of(v => a[v])
print(lo.depth(V(3)))
