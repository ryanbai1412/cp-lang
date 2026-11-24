// expect: ok, output "2 1 1"
// `G.Arc` of a `[V]List<A>` is `A`, found through `Next`'s `out`, so a std fn returning
// `List<G.Arc>` hands back the table's own records.
use std.graph.{negative_cycle}
domain V = 0..3
let g: [V]List<(to: V, w: Int)> = [[(to: 1, w: 1)], [(to: 0, w: -2)], [(to: 0, w: 1)]]
let cyc = negative_cycle(g)!
print(cyc.len, cyc[0].to, cyc[0].w)
