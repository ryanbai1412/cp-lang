// expect: ok, output "2 1"
// rule: 08.2.1
// A trait-typed parameter of a method makes it generic, as in a free fn.
use std.graph.{Next}
struct Deg<V>:
    n: Int
    pub fn new(g: Next<V>, u: V) -> Deg<V> = Deg::<V>(g.out(u).count())
domain V = 0..2
let g: [V]List<V> = [[V(1), V(0)], [V(0)]]
print(Deg(g, V(0)).n, Deg(g, V(1)).n)
