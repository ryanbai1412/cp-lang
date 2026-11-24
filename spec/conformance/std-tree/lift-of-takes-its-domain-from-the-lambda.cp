// expect: ok, output "5"
// rule: std.tree.Lift.of
// `Lift.of(next)` with the struct's `V` left unwritten takes it from the lambda's written param type, `(v: V) => ..`, before any default would make it `Int`.
use std.tree.{Lift}
domain V = 0..8
let up = Lift.of((v: V) => if v.idx == 0: None else: Some(V(v.idx - 1)))
print(up.depth(V(5)))
