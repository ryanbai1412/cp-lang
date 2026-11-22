// expect: ok, output "2 2 3 2"
// rule: 08.1.2
use std.window.{undoing, Undoing}
use std.dsu.{Dsu}
domain V = 0..4
var u: Undoing<Dsu<V>, (V, V)> = undoing(Dsu<V>(rollback: true), (mut d, e: (V, V)) => d.unite(e.0, e.1))
u.push((V(0), V(1)))
u.push((V(1), V(2)))
let c1 = u.state.components
u.pop((V(0), V(1)))
u.push((V(2), V(3)))
let c2 = u.state.components
u.pop((V(1), V(2)))
let c3 = u.state.components
u.push((V(0), V(3)))
let c4 = u.state.components
print(c1, c2, c3, c4)
