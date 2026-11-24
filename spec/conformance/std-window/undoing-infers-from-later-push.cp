// expect: ok, output "3"
// rule: 08.1.2
// `undoing`'s `T` is open after the call: the tuple pattern `(u, v)` leaves it to the later `w.push((V(0), V(1)))`, not to a default `Int`.
use std.dsu.{Dsu}
domain V = 0..4
var w = undoing(Dsu<V>(rollback: true), (mut d, (u, v)) => d.unite(u, v))
w.push((V(0), V(1)))
print(w.state.components)
