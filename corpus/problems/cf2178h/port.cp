// Source: Codeforces 2178H (Create or Duplicate), https://codeforces.com/contest/2178/problem/H
// limits: 6000 ms, 1024 MB
// Problem: three present types of values a < b < c < m <= 5e5, one present of each. Create adds
//   a present of a type for its value in mana; Duplicate doubles one type's presents for k
//   mana. Print the least mana making the total value a multiple of m. Multitest, sums of m and
//   of k <= 5e5.
// Solution: build each type's count by Horner's rule: its first create is the free initial
//   present, and a duplicate doubles every type already started (k per started type), so the
//   state is the value sum mod m and the set of started types. All moves have fixed costs, so
//   fixed_cost_paths (no heap) finds the cheapest way to (0, all started) over 8m states and 7
//   moves, O(56 m); subtract a + b + c for the free initial presents.
// Bounds: x, k: 1..=500_000; m: 2..=500_000
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
        Double(j) => if s.len == j: (R(2 * r.ord % m), s) else: None

let moves = [Create(t) for t in T] + [Double(j) for j in 1..=3]
let goal: State = (R(0), Subset<T>.full)
let dist = fixed_cost_paths(from: [(R(0), Subset<T>.empty)], moves:, cost:, step:, until: goal)
print(dist[goal]! - x.sum())
