// Codeforces 2178 H: Create or Duplicate.
// Build every type's count by Horner's rule: its first create is the free
// initial present, and a duplicate doubles every type already started, so
// the state is the value sum mod m and the set of started types. Every move
// has a fixed cost, so the shortest path needs no heap.
use std.graph.{fixed_cost_paths}

domain T = 0..3
enum Move: Create(T), Double(1..=3)

@!multitest
let x: [T]1..=500_000 = read()
let (m, k): (2..=500_000, 1..=500_000) = read()
domain R = 0..m
type State = (R, Subset<T>)

fn cost(mv: Move) -> Int = match mv:
    Create(t) => x[t]
    Double(j) => j * k
fn step(mv: Move, st: State) -> ?State:
    let (r, s) = st
    match mv:
        Create(t) => (R((r.ord + x[t]) % m), s.with(t))
        Double(j) => if s.len == j: (R(2 * r.ord % m), s) else: none

let moves = [Create(t) for t in T] + [Double(j) for j in 1..=3]
let goal: State = (R(0), Subset<T>.full)
let dist = fixed_cost_paths(from: [(R(0), Subset<T>.empty)], moves:, cost:, step:, until: goal)
print(dist[goal]! - x.sum())
