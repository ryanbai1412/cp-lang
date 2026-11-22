// expect: ok, output "YES NO"
use std.graph.{two_color}
domain V = 0..4
let g: [V]List<V> = [[V(1)], [V(0)], [V(3)], [V(2)]]
let c = two_color(g)!
domain W = 0..3
let tri: [W]List<W> = [[W(1), W(2)], [W(0), W(2)], [W(0), W(1)]]
print(yes(c[0] != c[1] and c[2] != c[3]), yes(two_color(tri) != None))
