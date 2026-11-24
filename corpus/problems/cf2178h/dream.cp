// Codeforces 2178 H: Create or Duplicate (dream).
// Build every type's count by Horner's rule: its first create is the free
// initial present, and a duplicate doubles every type already started, so
// the state is the value sum mod m and the set of started types. Every move
// has a fixed cost, so the shortest path needs no heap.
use std.graph.{fixed_cost_paths}
domain T = 0..3
enum Move: Create(T), Double(1..=3)

@!multitest
let x: [T]1..=5e5 = read()
let (m, k): (2..=5e5, 1..=5e5) = read()
domain R = 0..m
type State = (R, Subset<T>)

fn cost(mv: Move) -> Int = match mv:
    Create(t) => x[t]
    Double(j) => j * k
fn step(mv: Move, st: State) -> ?State:
    let (r, s) = st
    match mv:
        Create(t) => (R((r.ord + x[t]) % m), s.with(t))
        Double(j) => if s.len == j: (R(2 * r.ord % m), s) else: None

let dist = fixed_cost_paths(
    from: [(R(0), Subset<T>.empty)],
    moves: [Create(t) for t in T] + [Double(j) for j in 1..=3],
    cost: cost,
    step: step,
    until: (R(0), Subset<T>.full),
)
print(dist[(R(0), Subset<T>.full)]! - x.sum())
