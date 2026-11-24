// expect: ok, output "7 9"
// rule: 08.1.1
// A trait alias's `where Self.Arc: Labelled<L>` binds `L` from the arc's getter field, for an
// adjacency table as for std's `WeightedNext`.
use std.graph.{Next}
trait Labelled<L>:
    @get fn lab(self) -> L
trait LabelledNext<V, L> = Next<V> where Self.Arc: Labelled<L>
fn first_label<G: LabelledNext<V, L>, V, L>(g: G, u: V) -> L = g.out(u).collect()[0].lab
domain V = 0..2
let g: [V]List<(to: V, lab: Int)> = [[(to: 1, lab: 7)], [(to: 0, lab: 9)]]
print(first_label(g, V(0)), first_label(g, V(1)))
