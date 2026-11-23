// expect: ok, output "1"
// A tuple holding a Subset is a product domain for a generic algorithm's state.
use std.graph.{fixed_cost_paths}
domain T = 0..3
domain R = 0..2
type State = (R, Subset<T>)
fn step(mv: Int, st: State) -> ?State:
    let (r, s) = st
    if r.ord + 1 < 2: (R(r.ord + 1), s.with(T(mv))) else: None
let dist = fixed_cost_paths(from: [(R(0), Subset<T>.empty)], moves: [1], cost: (mv: Int) => mv, step:)
print(dist[(R(1), Subset<T>.single(T(1)))]!)
