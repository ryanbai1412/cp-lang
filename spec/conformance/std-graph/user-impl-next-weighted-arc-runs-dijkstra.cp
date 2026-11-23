// expect: ok, output "4\n8"
use std.graph.{Next}
domain V = 0..3
struct Chain:
    w: Int
impl Next<V> for Chain:
    type Arc = (to: V, w: Int)
    fn out(self, u: V) -> yields (to: V, w: Int):
        if Int(u) + 1 < 3: yield (to: V(Int(u) + 1), w: self.w)
let c = Chain(4)
print([a.w for a in c.out(V(0))])
print(dijkstra(c, from: [(V(0), 0)])[2]!)
